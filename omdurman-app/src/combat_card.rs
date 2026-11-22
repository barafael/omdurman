//! Combat Resolution Card (§decision: combat legibility).
//!
//! When a fire or melee attack resolves, the rules engine emits a structured
//! [`Observation::FireResolved`] or [`Observation::MeleeResolved`] carrying the
//! full attack bundle -- firers, target, modifiers, die roll, CRT cell, result,
//! casualties, and the rulebook paragraphs that authorise each piece.
//!
//! This module drains those observations into a small queue of *resolved*
//! cards (unit identities looked up against the live [`GameState`] at drain
//! time, before further mutations can obscure them) and renders the most
//! recent as a legible breakdown: every modifier attributable to its rulebook
//! paragraph, the die roll and its (modified) resolution value, the CRT cell
//! as a deep link into the Rulebook tab, and the casualty list.
//!
//! The card is the "why did this combat go the way it did?" answer -- a player
//! who has not read the manual can follow each bonus back to the paragraph
//! that grants it.

use bevy::ecs::message::MessageReader;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use omdurman_rules::combat_results_table::FireFactorRow;
use omdurman_rules::effects::{MeleeSide, Observation};
use omdurman_rules::{CombatResult, DieRoll, FireAttack, MeleeAttack, MeleeModifier, UnitId};
use omdurman_types::{HexCoord, Player};

use crate::GameStateResource;
use crate::events;
use crate::rulebook::Rulebook;

/// Maximum cards held in the queue. Older cards expire (FIFO evict) so a burst
/// of resolutions can't pile up unbounded.
const MAX_ENTRIES: usize = 4;
/// Seconds a card stays visible after the most recent resolution. Resolving a
/// new combat while a card is up slides the queue forward (newest at bottom).
const CARD_TTL: f32 = 12.0;
/// Seconds of fade-out at end-of-life.
const CARD_FADE: f32 = 1.5;

/// Bundle of the fire-combat resolution outputs (dice, modifiers, CRT row,
/// factor, result) so [`build_fire_card`] stays under clippy's argument limit.
struct FireResolution {
    roll: DieRoll,
    total_modifier: i16,
    modified_roll: DieRoll,
    factor_row: FireFactorRow,
    effective_factor: u16,
    result: CombatResult,
}

pub struct CombatCardPlugin;

impl Plugin for CombatCardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatCardQueue>()
            // Drain runs after the engine's `drain_observations` so the unit
            // identity lookups land in the same frame the engine pushed them
            // -- before later effects can mutate the units we want to name.
            .add_systems(
                Update,
                drain_combat_observations
                    .after(crate::events::drain_observations)
                    .run_if(resource_exists::<crate::events::PendingObservations>),
            )
            .add_systems(Update, report_lost_shells.after(drain_combat_observations))
            .add_systems(
                EguiPrimaryContextPass,
                // Runs after the charts sheet so the card can shift left of
                // the sheet / peek tab (see `ScreenLayout::right_inset`).
                combat_card_ui
                    .after(crate::charts::chart_sheet_ui)
                    .run_if(crate::on_board),
            );
    }
}

// ---------------------------------------------------------------------------
// Resolved card model
// ---------------------------------------------------------------------------

use crate::combat_ui::ModifierLine;
/// One row of the modifier breakdown: the die-roll delta and the rulebook
/// paragraph that authorises it. Both are pre-resolved strings so the card
/// never has to reach back into the rules engine at render time (when state
/// may have moved on).
use crate::combat_ui::describe_fire_modifier;
use crate::combat_ui::describe_melee_modifier;
use crate::combat_ui::describe_result;

/// One side of a combat (attacker for fire; attacker and defender for melee).
#[derive(Clone)]
struct CombatSide {
    player: Player,
    /// Comma-separated names of the firing/meleeing units, resolved from
    /// [`UnitId`]s at drain time. Falls back to a UnitId-ish label when the
    /// unit is gone from state (already eliminated by a later effect).
    units_label: String,
    factor: u16,
    factor_row_label: String,
    roll: DieRoll,
    modifiers: Vec<ModifierLine>,
    net_modifier: i16,
    modified_roll: DieRoll,
    result_label: String,
    /// Names of units on this side lost in the resolution.
    losses: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CombatKind {
    Fire,
    Melee,
}

/// A fully-resolved combat card ready to render. All rules-engine types have
/// been turned into strings; the card no longer holds any borrowed engine
/// state.
struct CombatCardEntry {
    kind: CombatKind,
    target_hex: HexCoord,
    /// Optional name of the *target hex itself* (e.g. a fortification, "the
    /// Palace") -- empty for a normal hex. Useful because the cell is what
    /// the CRT row indexes, not the units standing on it.
    hex_label: String,
    /// §6.64: the hex a howitzer shell was aimed at, when it scattered to
    /// `target_hex` instead -- the result applies where it landed.
    aimed_from: Option<String>,
    attacker: CombatSide,
    /// `None` for fire (single-sided resolution); `Some` for melee, where
    /// both sides roll simultaneously and each result applies to the other.
    defender: Option<CombatSide>,
    /// Rulebook paragraphs the engine cited for this resolution. Rendered as
    /// deep-link chips at the card foot.
    paragraphs: Vec<String>,
    /// What the result opens up: the §7.6 mandatory Dervish advance (already
    /// made), or who may advance into the vacated hex (§6.82/§7.6).
    note: Option<String>,
    /// The hexes the combat concerns (firers or attackers, the target,
    /// where a shell landed): ringed on the board while the card is hovered.
    focus: Vec<HexCoord>,
    /// Seconds shown; frozen while hovered or pinned.
    age: f32,
    /// Hover / click-to-pin state (see [`crate::ui::CardHold`]).
    hold: crate::ui::CardHold,
    /// Stable per-card id (egui click target), assigned on push.
    serial: u64,
    /// Arrival order in the event feed (see `Dispatches::next_feed_seq`).
    seq: u64,
}

#[derive(Resource, Default)]
pub(crate) struct CombatCardQueue {
    entries: Vec<CombatCardEntry>,
    next_serial: u64,
}

impl CombatCardQueue {
    /// Drop every card, pinned ones included (a new game begins).
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    /// Queue a card, evicting the oldest unpinned card beyond
    /// [`MAX_ENTRIES`].
    fn push(&mut self, mut entry: CombatCardEntry, feed_seq: u64) {
        self.next_serial += 1;
        entry.serial = self.next_serial;
        entry.seq = feed_seq;
        self.entries.push(entry);
        while self.entries.len() > MAX_ENTRIES {
            let victim = self
                .entries
                .iter()
                .position(|e| !e.hold.pinned)
                .unwrap_or(0);
            self.entries.remove(victim);
        }
    }
}

// ---------------------------------------------------------------------------
// Drain: ObservationEvent -> resolved card entries
// ---------------------------------------------------------------------------

/// Listen for `FireResolved` / `MeleeResolved` observations and push a fully
/// resolved card onto the queue. Identity lookups go through the live
/// [`GameState`] *this frame*; once captured, the card holds strings only.
fn drain_combat_observations(
    mut reader: MessageReader<events::ObservationEvent>,
    mut queue: ResMut<CombatCardQueue>,
    game_state: Res<GameStateResource>,
    mut dispatches: ResMut<crate::dispatch::Dispatches>,
) {
    let gs = &game_state.0;
    for ev in reader.read() {
        let entry = match &ev.observation {
            Observation::FireResolved {
                attack,
                roll,
                total_modifier,
                modified_roll,
                factor_row,
                effective_factor,
                result,
                eliminations,
                paragraphs,
                impact,
                // `range`/`band` are surfaced by the bot log; the card keeps
                // its existing layout.
                ..
            } => {
                let mut card = build_fire_card(
                    attack,
                    FireResolution {
                        roll: *roll,
                        total_modifier: *total_modifier,
                        modified_roll: *modified_roll,
                        factor_row: *factor_row,
                        effective_factor: *effective_factor,
                        result: *result,
                    },
                    eliminations,
                    paragraphs,
                    gs,
                );
                // §6.64: where the shell landed, and on what roll.
                if let Some((impact_roll, landed)) = impact {
                    card.note = Some(impact_note(*impact_roll, *landed, attack.target_hex, gs));
                    if *landed != attack.target_hex {
                        relocate_scattered(&mut card, attack, *result, eliminations, *landed, gs);
                    }
                    if !card.focus.contains(landed) {
                        card.focus.push(*landed);
                    }
                }
                card
            }
            Observation::MeleeResolved {
                attack,
                attacker,
                defender,
                mandatory_advance,
                paragraphs,
            } => build_melee_card(
                attack,
                *mandatory_advance,
                attacker,
                defender,
                paragraphs,
                gs,
            ),
            // The advance a resolution opened (§6.82/§7.6) belongs on that
            // resolution's card -- it arrives right after it, same frame.
            Observation::HexVacatedByCombat { hex, eligible, .. } => {
                if let Some(card) = queue
                    .entries
                    .iter_mut()
                    .rev()
                    .find(|c| c.target_hex == *hex)
                    .filter(|c| c.note.is_none())
                {
                    let who = list_unit_names(eligible, gs).join(", ");
                    card.note = Some(format!(
                        "Hex {hex} vacated: {who} may advance into it (§6.82, §7.6)."
                    ));
                }
                continue;
            }
            _ => continue,
        };
        dispatches.log_line(log_line(&entry));
        queue.push(entry, dispatches.next_feed_seq());
    }
}

/// §6.64: a howitzer shell that scatters off the map resolves nothing -- no
/// combat card, no log line; the engine notes it only in its turn events.
/// Report each such shell once, as it appears there (the first run only
/// takes note of the ones already recorded, e.g. by a replay).
fn report_lost_shells(
    game_state: Res<GameStateResource>,
    mut dispatches: ResMut<crate::dispatch::Dispatches>,
    mut seen: Local<Option<usize>>,
) {
    if !game_state.is_changed() {
        return;
    }
    let gs = &game_state.0;
    let lost: Vec<String> = gs
        .turn_events
        .iter()
        .filter_map(|e| match e {
            omdurman_rules::turn_summary::TurnEventRecord::HowitzerImpact {
                gunboat,
                aimed,
                at,
                lost: true,
                ..
            } => Some(lost_shell_text(*gunboat, *aimed, *at, gs)),
            _ => None,
        })
        .collect();
    // The turn's events are drained at the end of each player turn.
    let already = match *seen {
        Some(n) if n <= lost.len() => n,
        Some(_) => 0,
        None => lost.len(),
    };
    for text in &lost[already..] {
        dispatches.push("Howitzer Fire", text.clone());
    }
    *seen = Some(lost.len());
}

/// The report of a shell scattered off the map (§6.64).
fn lost_shell_text(
    gunboat: UnitId,
    aimed: HexCoord,
    at: HexCoord,
    gs: &omdurman_rules::effects::GameState,
) -> String {
    let aimed = place_label(aimed, &target_hex_label(aimed, gs));
    format!(
        "{}'s shell aimed at {aimed} scattered off the map ({},{}): no effect (§6.64).",
        crate::combat_ui::unit_name(gunboat, gs),
        at.q,
        at.r
    )
}

/// The table's result, and -- when it was not enough against a fort or a
/// gunboat (§6.61/§6.62: a destroying result or nothing) -- that it missed:
/// a bare "Disrupt" on a fort that stands undisrupted read as a bug.
pub(crate) fn fire_result_label(
    attack: &FireAttack,
    result: omdurman_rules::CombatResult,
    eliminations: &[UnitId],
    gs: &omdurman_rules::effects::GameState,
) -> String {
    let label = describe_result(result);
    // A surviving special target is still on the board, so the threshold
    // reads off the state after the shot.
    if !eliminations.is_empty() {
        return label;
    }
    match omdurman_rules::effects::special_target_threshold(gs, attack) {
        Some(needed) => format!("{label} \u{2014} a miss (Eliminate {needed}+ needed)"),
        None => label,
    }
}

fn build_fire_card(
    attack: &FireAttack,
    resolution: FireResolution,
    eliminations: &[UnitId],
    paragraphs: &[String],
    gs: &omdurman_rules::effects::GameState,
) -> CombatCardEntry {
    let FireResolution {
        roll,
        total_modifier,
        modified_roll,
        factor_row,
        effective_factor,
        result,
    } = resolution;
    let attacker = CombatSide {
        player: attack.firing_player,
        units_label: crate::combat_ui::tally_names(
            attack
                .shots()
                .into_iter()
                .map(|shot| crate::combat_ui::shot_name(shot, gs))
                .collect(),
        )
        .join(", "),
        factor: effective_factor,
        factor_row_label: factor_row.label().to_string(),
        roll,
        modifiers: fire_modifier_lines(attack, total_modifier),
        net_modifier: total_modifier,
        modified_roll,
        result_label: fire_result_label(attack, result, eliminations, gs),
        losses: list_unit_names(eliminations, gs),
    };
    let hex_label = target_hex_label(attack.target_hex, gs);
    // Fire moves no firer: where they stand now is where they fired from.
    let mut focus: Vec<HexCoord> = attack
        .all_firing_units()
        .iter()
        .filter_map(|id| gs.find_unit(*id).map(|u| u.position))
        .collect();
    focus.push(attack.target_hex);
    focus.dedup();
    CombatCardEntry {
        kind: CombatKind::Fire,
        target_hex: attack.target_hex,
        hex_label,
        aimed_from: None,
        attacker,
        defender: None,
        paragraphs: paragraphs.to_vec(),
        note: None,
        focus,
        age: 0.0,
        hold: crate::ui::CardHold::default(),
        serial: 0,
        seq: 0,
    }
}

/// What a melee did beyond its two results, for the card and the log line:
/// the mandatory Dervish advance (§7.6) -- and who stayed behind because
/// disrupted units may not move -- and Anglo-Egyptian leaders lost, whom no
/// combat result takes: they fall to the advance into their hex, or with the
/// last of their units (§6.51, §9.346).
fn melee_note(
    attack: &MeleeAttack,
    mandatory_advance: Option<u8>,
    attacker: &MeleeSide,
    defender: &MeleeSide,
    leaders_alone: bool,
    gs: &omdurman_rules::effects::GameState,
) -> Option<String> {
    let is_ae_leader = |id: &UnitId| {
        omdurman_rules::unit_profiles::profile_for_unit(*id)
            .is_some_and(|p| matches!(p.kind, omdurman_types::UnitKind::BritishLeader { .. }))
    };
    let mut parts: Vec<String> = Vec::new();
    if let Some(n) = mandatory_advance {
        let units = if n == 1 { "unit" } else { "units" };
        if leaders_alone {
            parts.push(format!(
                "A leader alone does not hold a hex: {n} attacking {units} entered it and he \
                 fell, whatever the dice (§6.51)."
            ));
        } else {
            parts.push(format!(
                "{n} surviving attacking {units} advanced into the hex (mandatory, §7.6)."
            ));
        }
    }
    // The defenders' hex emptied: Dervish survivors that are disrupted
    // stay where they are (back cover: disrupted units may not move).
    let cleared = attack
        .defenders
        .iter()
        .all(|id| defender.losses.contains(id));
    if cleared && attack.attacker_player == omdurman_types::Player::Dervish {
        let stayed: Vec<UnitId> = attack
            .attackers
            .iter()
            .copied()
            .filter(|id| !attacker.losses.contains(id))
            .filter(|id| {
                gs.find_unit(*id)
                    .is_some_and(|u| u.state.disrupted && u.position == attack.attacker_hex)
            })
            .collect();
        if !stayed.is_empty() {
            parts.push(format!(
                "{} stayed behind: disrupted units may not advance (§7.6).",
                list_unit_names(&stayed, gs).join(", ")
            ));
        }
    }
    let leaders: Vec<UnitId> = defender
        .losses
        .iter()
        .chain(&attacker.losses)
        .copied()
        .filter(is_ae_leader)
        .collect();
    if !leaders.is_empty() && !leaders_alone {
        let how = if mandatory_advance.is_some()
            && leaders.iter().any(|id| defender.losses.contains(id))
        {
            "to the advance into the hex"
        } else {
            "with the last of their units"
        };
        let gordon = leaders.iter().any(|id| {
            omdurman_rules::unit_profiles::profile_for_unit(*id)
                .is_some_and(|p| p.identity.is_gordon())
        });
        parts.push(format!(
            "{} fell {how}, not to the dice: no combat result takes a leader ({}).",
            list_unit_names(&leaders, gs).join(", "),
            if gordon { "§6.51, §9.346" } else { "§6.51" }
        ));
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

fn build_melee_card(
    attack: &MeleeAttack,
    mandatory_advance: Option<u8>,
    attacker: &MeleeSide,
    defender: &MeleeSide,
    paragraphs: &[String],
    gs: &omdurman_rules::effects::GameState,
) -> CombatCardEntry {
    // A side with no melee strength (a leader alone, §6.51) strikes nothing:
    // "No effect on the attackers" read as if he had fought.
    let struck = |side: &MeleeSide, whom: &str| {
        if side.factor == 0 {
            "no melee strength, no roll".to_string()
        } else {
            format!("{} on the {whom}", describe_result(side.result))
        }
    };
    let attacker_card = CombatSide {
        player: attack.attacker_player,
        units_label: list_units(&attack.attackers, gs),
        factor: attacker.factor,
        factor_row_label: FireFactorRow::from_total(attacker.factor)
            .label()
            .to_string(),
        roll: attacker.roll,
        modifiers: melee_modifier_lines(&attack.attacker_modifiers, attacker.total_modifier),
        net_modifier: attacker.total_modifier,
        modified_roll: attacker.modified_roll,
        // Melee is simultaneous: each roll strikes the other side, so the
        // result says whom -- beside "lost:", the side's own casualties, a
        // bare "→ Disrupt" read as what befell the side itself.
        result_label: struck(attacker, "defenders"),
        losses: list_unit_names(&attacker.losses, gs),
    };
    let defender_card = CombatSide {
        player: attack.attacker_player.opponent(),
        units_label: list_units(&attack.defenders, gs),
        factor: defender.factor,
        factor_row_label: FireFactorRow::from_total(defender.factor)
            .label()
            .to_string(),
        roll: defender.roll,
        modifiers: melee_modifier_lines(&attack.defender_modifiers, defender.total_modifier),
        net_modifier: defender.total_modifier,
        modified_roll: defender.modified_roll,
        result_label: struck(defender, "attackers"),
        losses: list_unit_names(&defender.losses, gs),
    };
    let hex_label = target_hex_label(attack.defender_hex, gs);
    let paragraphs = paragraphs.to_vec();
    // §7.6: a Dervish melee that clears the hex carries a *mandatory*
    // advance, which the engine has already made -- say so on the card.
    // §6.51(a): leaders alone do not hold a hex -- they fall to the
    // attackers who enter it, whatever the dice said.
    let leaders_alone = !attack.defenders.is_empty()
        && attack.defenders.iter().all(|id| {
            omdurman_rules::unit_profiles::profile_for_unit(*id)
                .is_some_and(|p| matches!(p.kind, omdurman_types::UnitKind::BritishLeader { .. }))
        });
    let note = melee_note(
        attack,
        mandatory_advance,
        attacker,
        defender,
        leaders_alone,
        gs,
    );
    CombatCardEntry {
        kind: CombatKind::Melee,
        target_hex: attack.defender_hex,
        hex_label,
        aimed_from: None,
        attacker: attacker_card,
        defender: Some(defender_card),
        paragraphs,
        note,
        focus: vec![attack.attacker_hex, attack.defender_hex],
        age: 0.0,
        hold: crate::ui::CardHold::default(),
        serial: 0,
        seq: 0,
    }
}

/// Translate a fire attack's modifiers into display lines, plus the engine-
/// side terrain modifier (which isn't in `attack.modifiers` -- it's derived
/// from `state.board` at resolution time). The terrain line is the difference
/// between the engine's reported `total_modifier` and the sum of the
/// caller-supplied modifiers.
pub(crate) fn fire_modifier_lines(attack: &FireAttack, total_modifier: i16) -> Vec<ModifierLine> {
    let mut out: Vec<ModifierLine> = attack
        .modifiers
        .iter()
        .map(|m| describe_fire_modifier(*m))
        .collect();
    let app_supplied = attack.net_modifier();
    let terrain_mod = total_modifier - app_supplied;
    if terrain_mod != 0 {
        out.push(ModifierLine {
            label: format!("{terrain_mod:+} terrain defence"),
            paragraph: "6.23".into(),
        });
    }
    out
}

pub(crate) fn melee_modifier_lines(
    modifiers: &[MeleeModifier],
    total_modifier: i16,
) -> Vec<ModifierLine> {
    let mut out: Vec<ModifierLine> = modifiers
        .iter()
        .map(|m| describe_melee_modifier(*m))
        .collect();
    let app_supplied: i16 = modifiers.iter().map(|m| m.die_modifier()).sum();
    let other = total_modifier - app_supplied;
    if other != 0 {
        out.push(ModifierLine {
            label: format!("{other:+} (other)"),
            paragraph: "7.7".into(),
        });
    }
    out
}

/// Resolve a slice of [`UnitId`]s into a comma-separated list of short unit
/// names. Units that no longer exist in the game state (eliminated by this
/// very combat) are named from the static counter roster.
fn list_units(ids: &[UnitId], gs: &omdurman_rules::effects::GameState) -> String {
    if ids.is_empty() {
        return "—".into();
    }
    list_unit_names(ids, gs).join(", ")
}

/// Like [`list_units`] but returns the names separately (repeats counted:
/// "2× Mulazmin"), for casualty lists where each loss is its own line item.
pub(crate) fn list_unit_names(
    ids: &[UnitId],
    gs: &omdurman_rules::effects::GameState,
) -> Vec<String> {
    crate::combat_ui::tally_names(
        ids.iter()
            .map(|id| crate::combat_ui::unit_name(*id, gs))
            .collect(),
    )
}

/// A short label for any landmark at the target hex (fort, palace, etc.).
/// Returns an empty string for an ordinary hex so the renderer can skip it.
/// §6.64: where a howitzer shell landed, and on what roll.
fn impact_note(
    roll: omdurman_rules::DieRoll,
    landed: HexCoord,
    target: HexCoord,
    gs: &omdurman_rules::effects::GameState,
) -> String {
    if landed == target {
        return format!("Impact roll {}: on target (§6.64).", roll.value());
    }
    // An unnamed hex has no label: say where by coordinates (it read
    // "scattered to  (§6.64)").
    let mut at = target_hex_label(landed, gs);
    if at.is_empty() {
        at = format!("({},{})", landed.q, landed.r);
    }
    format!(
        "Impact roll {}: the shell scattered to {at} (§6.64).",
        roll.value()
    )
}

/// §6.64: a scattered shell's result applies where it landed, to whoever
/// stands there (friend or foe), not at the hex it was aimed at -- move the
/// card there, keep the aim as a note, and read the result against the
/// landing hex: a shell in an empty hex did nothing, and the "a miss (… needed)"
/// gloss belongs to a fort or gunboat it actually fell on.
fn relocate_scattered(
    card: &mut CombatCardEntry,
    attack: &FireAttack,
    result: omdurman_rules::CombatResult,
    eliminations: &[UnitId],
    landed: HexCoord,
    gs: &omdurman_rules::effects::GameState,
) {
    card.aimed_from = Some(place_label(attack.target_hex, &card.hex_label));
    card.target_hex = landed;
    card.hex_label = target_hex_label(landed, gs);
    // Fire takes no unit off a hex it does not eliminate, so a hex empty
    // now (with no eliminations) was empty when the shell struck it.
    card.attacker.result_label = if eliminations.is_empty() && gs.units_in_hex(landed).is_empty() {
        "No effect \u{2014} the shell fell on an empty hex".to_string()
    } else {
        let at_landing = FireAttack {
            target_hex: landed,
            at_fort: false,
            ..attack.clone()
        };
        fire_result_label(&at_landing, result, eliminations, gs)
    };
}

/// A hex as a card or log line names it: its place name, or its coordinates.
fn place_label(hex: HexCoord, label: &str) -> String {
    if label.is_empty() {
        format!("({},{})", hex.q, hex.r)
    } else {
        label.to_string()
    }
}

pub(crate) fn target_hex_label(hex: HexCoord, gs: &omdurman_rules::effects::GameState) -> String {
    gs.board
        .location_at(hex)
        .map(|loc| loc.to_string())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Render: queue -> egui cards
// ---------------------------------------------------------------------------

/// The width of the right-hand column of combat cards and slips.
const COLUMN_WIDTH: f32 = 360.0;

/// A combat result as one line of the session log.
fn log_line(entry: &CombatCardEntry) -> String {
    let place = place_label(entry.target_hex, &entry.hex_label);
    let side = |side: &CombatSide| {
        let mut text = format!("{}: {}", side.units_label, side.result_label);
        if !side.losses.is_empty() {
            text.push_str(&format!(", lost {}", side.losses.join(", ")));
        }
        text
    };
    match (&entry.kind, &entry.defender) {
        (CombatKind::Melee, Some(defender)) => {
            // Each side's result is what it dealt the other; its losses are
            // its own -- listed apart, so a "Disrupt" is never read as the
            // cause of the losses printed next to it.
            let lost = |s: &CombatSide| {
                if s.losses.is_empty() {
                    "none".to_string()
                } else {
                    s.losses.join(", ")
                }
            };
            let mut line = format!(
                "Melee at {place}: attackers {} \u{2192} {}; defenders {} \u{2192} {}. \
                 Lost \u{2014} attackers: {}; defenders: {}.",
                entry.attacker.units_label,
                entry.attacker.result_label,
                defender.units_label,
                defender.result_label,
                lost(&entry.attacker),
                lost(defender),
            );
            if let Some(note) = &entry.note {
                line.push(' ');
                line.push_str(note);
            }
            line
        }
        _ => match &entry.aimed_from {
            Some(aimed) => format!(
                "Shell aimed at {aimed} scattered to {place} (§6.64): {}",
                side(&entry.attacker)
            ),
            None => format!("Fire at {place}: {}", side(&entry.attacker)),
        },
    }
}

/// The event feed: combat cards and dispatch slips (refusals, notices) in
/// one column at the top right, newest first -- one place to look instead of
/// three corners. The top bar's Log menu keeps every line after it fades.
fn combat_card_ui(
    mut contexts: EguiContexts,
    mut queue: ResMut<CombatCardQueue>,
    mut dispatches: ResMut<crate::dispatch::Dispatches>,
    time: Res<Time>,
    mut rulebook: ResMut<Rulebook>,
    layout: Res<crate::ScreenLayout>,
    mut card_focus: ResMut<crate::fx::CardFocus>,
) {
    let dt = time.delta_secs();
    for entry in &mut queue.entries {
        entry.hold.age(&mut entry.age, dt, CARD_TTL, CARD_FADE);
    }
    queue.entries.retain(|e| e.age < CARD_TTL);
    // The hovered card's hexes are ringed on the board (last frame's hover:
    // the card is drawn below).
    let focus = queue
        .entries
        .iter()
        .find(|e| e.hold.hovered)
        .map(|e| e.focus.clone())
        .unwrap_or_default();
    card_focus.set_if_neq(crate::fx::CardFocus(focus));
    dispatches.age(dt);
    if queue.entries.is_empty() && dispatches.slips.is_empty() {
        return;
    }
    // Newest first, across both queues.
    enum Item {
        Card(usize),
        Slip(usize),
    }
    let mut items: Vec<(u64, Item)> = queue
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.seq, Item::Card(i)))
        .chain(
            dispatches
                .slips
                .iter()
                .enumerate()
                .map(|(i, s)| (s.seq, Item::Slip(i))),
        )
        .collect();
    items.sort_by_key(|(seq, _)| std::cmp::Reverse(*seq));
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let ctx_height = ctx.content_rect().height();

    let mut clicked_section: Option<String> = None;

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("combat_cards"),
        // Right of the board, below the top bar and clear of the charts
        // sheet / peek tab (see `ScreenLayout::right_inset`).
        egui::Align2::RIGHT_TOP,
        egui::vec2(-(layout.right_inset + 12.0), layout.top_bar_height + 8.0),
        egui::Frame::NONE,
        |ui| {
            // A fixed width: the area is anchored at its right edge, which
            // egui places by the previous frame's size -- a column sized by
            // its content kept the width of the last narrow slip, so a wider
            // one wrapped short and ran off the screen edge, cut off.
            ui.set_width(COLUMN_WIDTH);
            // A volley of resolutions outgrows the window: scroll the column
            // instead of letting the oldest cards run off the bottom unread.
            let max_height = ctx_height - layout.top_bar_height - 60.0;
            egui::ScrollArea::vertical()
                .id_salt("combat_cards_scroll")
                .max_height(max_height.max(120.0))
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                        ui.set_width(COLUMN_WIDTH);
                        // Newest at the top, closest to the screen edge.
                        for (_, item) in &items {
                            let entry = match item {
                                Item::Slip(i) => {
                                    let slip = &mut dispatches.slips[*i];
                                    let fade = ((crate::dispatch::DISPATCH_TTL - slip.age)
                                        / crate::dispatch::DISPATCH_FADE)
                                        .clamp(0.0, 1.0);
                                    slip.hold
                                        .begin(ui, egui::Id::new(("dispatch_slip", slip.serial)));
                                    let (sec, rect) = crate::dispatch::draw_slip(ui, slip, fade);
                                    slip.hold.end(ui, rect);
                                    if let Some(sec) = sec {
                                        clicked_section = Some(sec);
                                    }
                                    ui.add_space(6.0);
                                    continue;
                                }
                                Item::Card(i) => &mut queue.entries[*i],
                            };
                            let fade = ((CARD_TTL - entry.age) / CARD_FADE).clamp(0.0, 1.0);
                            entry
                                .hold
                                .begin(ui, egui::Id::new(("combat_card", entry.serial)));
                            // Fade the whole card -- paper, text and chips together. (Fading
                            // only the text colours left an empty yellow box behind.)
                            let (sec, rect) = ui
                                .scope(|ui| {
                                    ui.set_opacity(fade);
                                    draw_card(ui, entry, &rulebook)
                                })
                                .inner;
                            entry.hold.end(ui, rect);
                            if let Some(sec) = sec {
                                clicked_section = Some(sec);
                            }
                            ui.add_space(6.0);
                        }
                    });
                });
        },
    );

    if let Some(sec) = clicked_section {
        crate::rulebook::request_section(&mut rulebook, &sec);
    }
    // Display-rate frames only while something fades out. Until then the
    // cards age on the idle wake-ups (see `activity`), and a held card --
    // pinned, or under a resting pointer -- does not change at all: asking
    // for every frame here kept a pinned card's window redrawing for good.
    let fading = queue
        .entries
        .iter()
        .any(|e| !e.hold.is_held() && e.age > CARD_TTL - CARD_FADE)
        || dispatches.slips.iter().any(|s| {
            !s.hold.is_held()
                && s.age > crate::dispatch::DISPATCH_TTL - crate::dispatch::DISPATCH_FADE
        });
    if fading {
        ctx.request_repaint();
    }
}

fn draw_card(
    ui: &mut egui::Ui,
    entry: &CombatCardEntry,
    rulebook: &Rulebook,
) -> (Option<String>, egui::Rect) {
    // The caller fades the whole card (`Ui::set_opacity`); colours pass as is.
    let a = |c: egui::Color32| c;
    let mut clicked: Option<String> = None;
    let kind_label = crate::ui::faction_name(entry.attacker.player);
    let stroke = if entry.hold.pinned { 3.0 } else { 2.0 };
    let header_word = match entry.kind {
        CombatKind::Fire => "FIRE COMBAT",
        CombatKind::Melee => "MELEE COMBAT",
    };

    let frame = crate::ui::frames::paper(egui::Stroke::new(stroke, a(crate::ui::palette::INK)))
        .inner_margin(egui::Margin::symmetric(12, 9))
        .show(ui, |ui| {
            // Text runs left to right: the column aligns its cards to the
            // right, which egui would carry into the wrapped rows.
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_max_width(340.0);
                // Header line: "FIRE COMBAT — Anglo-Egyptian"
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(header_word)
                            .color(a(crate::ui::palette::INK))
                            .size(13.0)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!("— {kind_label}"))
                            .color(a(crate::ui::palette::FAINT_INK))
                            .size(12.0),
                    );
                    if entry.hold.pinned {
                        ui.label(
                            egui::RichText::new("(pinned)")
                                .color(a(crate::ui::palette::FAINT_INK))
                                .size(10.0)
                                .italics(),
                        );
                    }
                });
                ui.add_space(2.0);
                // Target line.
                // A named place reads better than its coordinates.
                let place = place_label(entry.target_hex, &entry.hex_label);
                let hex_str = match &entry.aimed_from {
                    Some(aimed) => format!("at {place} \u{2014} aimed at {aimed}, scattered"),
                    None => format!("at {place}"),
                };
                ui.label(
                    egui::RichText::new(hex_str)
                        .color(a(crate::ui::palette::FAINT_INK))
                        .size(12.0),
                );
                ui.add_space(4.0);

                // Attacker side block: firers shoot, melee attackers fight.
                let role = match entry.kind {
                    CombatKind::Fire => "Firers:",
                    CombatKind::Melee => "Attackers:",
                };
                draw_side(ui, role, &entry.attacker, a, rulebook, &mut clicked);
                // Defender block for melee (symmetric).
                if let Some(defender) = &entry.defender {
                    ui.add_space(4.0);
                    draw_side(ui, "Defenders:", defender, a, rulebook, &mut clicked);
                }

                if let Some(note) = &entry.note {
                    ui.add_space(4.0);
                    crate::rulebook::refs_rich(ui, note, 12.0, |t| {
                        t.color(a(crate::ui::palette::INK)).italics()
                    });
                }

                ui.add_space(4.0);
                // Footer: paragraph chips.
                if !entry.paragraphs.is_empty() {
                    let refs: Vec<&str> = entry.paragraphs.iter().map(String::as_str).collect();
                    if let Some(p) = rulebook.render_ref_links(ui, &refs) {
                        clicked = Some(p);
                    }
                }
            })
        });
    (clicked, frame.response.rect)
}

fn draw_side(
    ui: &mut egui::Ui,
    role: &str,
    side: &CombatSide,
    a: impl Fn(egui::Color32) -> egui::Color32,
    rulebook: &Rulebook,
    clicked: &mut Option<String>,
) {
    ui.label(
        egui::RichText::new(format!("{role} {}", side.units_label))
            .color(a(crate::ui::palette::INK))
            .size(13.0),
    );
    // Roll + modifier summary line.
    let mod_str = if side.net_modifier == 0 {
        String::new()
    } else {
        format!(" {:+}", side.net_modifier)
    };
    // A side with no melee factor makes no roll -- the engine resolves it
    // as no effect: Anglo-Egyptian leaders alone (§6.51), or units that are
    // all disrupted ("may not melee", the Disrupted Units note).
    let summary = if side.factor == 0 {
        "no roll — leaders alone (§6.51) or disrupted units do not melee".to_string()
    } else {
        format!(
            "factor {} (row {}) — rolled {}{} = {}  →  {}",
            side.factor,
            side.factor_row_label,
            side.roll.value(),
            mod_str,
            side.modified_roll.value(),
            side.result_label,
        )
    };
    crate::rulebook::refs_rich(ui, &summary, 12.0, |t| {
        t.color(a(crate::ui::palette::FAINT_INK)).monospace()
    });
    // Modifier breakdown, each line deep-linking to its rulebook paragraph
    // (none for a side that made no roll).
    if !side.modifiers.is_empty() && side.factor != 0 {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (i, line) in side.modifiers.iter().enumerate() {
                if i > 0 {
                    ui.label(
                        egui::RichText::new(" · ")
                            .color(a(crate::ui::palette::FAINT_INK))
                            .size(11.0),
                    );
                }
                ui.label(
                    egui::RichText::new(&line.label)
                        .color(a(crate::ui::palette::FAINT_INK))
                        .size(11.0),
                );
                // The bare number; the rule's opening words on hover.
                let chip = ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!(" §{}", line.paragraph))
                            .color(a(crate::ui::palette::INK))
                            .size(11.0)
                            .underline(),
                    )
                    .sense(egui::Sense::click()),
                );
                let chip = match rulebook.title_of(&line.paragraph) {
                    Some(title) => chip.on_hover_text(title),
                    None => chip,
                };
                if chip.clicked() {
                    *clicked = Some(line.paragraph.clone());
                }
            }
        });
    }
    // Casualties.
    if !side.losses.is_empty() {
        ui.label(
            egui::RichText::new(format!("lost: {}", side.losses.join(", ")))
                .color(a(crate::ui::palette::INK_LOSS))
                .size(12.0),
        );
    }
}

#[cfg(test)]
mod impact_tests {
    use super::*;

    #[test]
    fn a_scattered_shell_says_where_it_landed() {
        let note = impact_note(
            omdurman_rules::DieRoll::Four,
            HexCoord::new(24, 10),
            HexCoord::new(23, 10),
            &omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign),
        );
        assert_eq!(
            note,
            "Impact roll 4: the shell scattered to (24,10) (§6.64)."
        );
    }

    /// A melee log line keeps each side's losses apart from the result it
    /// dealt, and says a leader fell to the advance, not the dice (§6.51,
    /// §9.346); a side with no melee strength made no roll.
    #[test]
    fn a_melee_line_names_each_sides_own_losses() {
        use omdurman_rules::effects::MeleeSide;
        let mut gs =
            omdurman_rules::effects::GameState::new(omdurman_types::Scenario::FallOfKhartoum);
        let palace = HexCoord::new(13, 5);
        let from = HexCoord::new(13, 6);
        let (a, b) = (UnitId::MulazminI_0_0, UnitId::MulazminI_0_1);
        // The survivors advanced into the palace; `b` is disrupted and stayed.
        for (id, at, disrupted) in [(a, palace, false), (b, from, true)] {
            gs.units.push(omdurman_rules::UnitPlacement {
                id,
                position: at,
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: omdurman_rules::UnitState {
                    disrupted,
                    ..Default::default()
                },
            });
        }
        let gordon = UnitId::BritishBoats_3_1;
        let battalion = UnitId::BritishArmy_0_0;
        let attack = MeleeAttack {
            attacker_player: Player::Dervish,
            attacker_hex: from,
            defender_hex: palace,
            attackers: vec![a, b],
            defenders: vec![gordon, battalion],
            attacker_modifiers: Vec::new(),
            defender_modifiers: Vec::new(),
        };
        let side = |factor, result, losses: Vec<UnitId>| MeleeSide {
            roll: DieRoll::Five,
            total_modifier: 0,
            modified_roll: DieRoll::Five,
            result,
            factor,
            losses,
        };
        let card = build_melee_card(
            &attack,
            Some(1),
            &side(12, CombatResult::Eliminate(1), Vec::new()),
            &side(5, CombatResult::Disrupt, vec![gordon, battalion]),
            &[],
            &gs,
        );
        let line = log_line(&card);
        assert!(
            line.contains("Lost \u{2014} attackers: none; defenders: Gordon, Cavalry."),
            "{line}"
        );
        assert!(
            line.contains("Gordon fell to the advance into the hex, not to the dice"),
            "{line}"
        );
        assert!(line.contains("§9.346"), "{line}");
        assert!(
            line.contains("Mulazmin stayed behind: disrupted units may not advance"),
            "{line}"
        );
        let lone = build_melee_card(
            &MeleeAttack {
                defenders: vec![gordon],
                ..attack.clone()
            },
            Some(1),
            &side(12, CombatResult::Eliminate(1), Vec::new()),
            &side(0, CombatResult::NoEffect, vec![gordon]),
            &[],
            &gs,
        );
        assert!(
            log_line(&lone).contains("defenders Gordon \u{2192} no melee strength, no roll"),
            "{}",
            log_line(&lone)
        );
    }

    /// A shell scattered off the map is reported too (§6.64) -- it makes no
    /// combat card.
    #[test]
    fn a_shell_off_the_map_is_reported() {
        let gs = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        let text = lost_shell_text(
            UnitId::BritishBoats_3_0,
            HexCoord::new(23, 10),
            HexCoord::new(24, -1),
            &gs,
        );
        assert!(
            text.contains("scattered off the map (24,-1): no effect (§6.64)"),
            "{text}"
        );
    }

    /// A scattered shell is reported where it landed: the log line and the
    /// card name the landing hex (the aim kept as a note), and a shell in an
    /// empty hex did nothing -- not the table's "Eliminate 2" at the aimed
    /// hex, nor a gunboat's "a miss" gloss there.
    #[test]
    fn a_scattered_shell_is_logged_where_it_landed() {
        let gs = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: omdurman_rules::Phase::OffensiveFire(
                omdurman_rules::FireSubPhase::MaximSecondAndHowitzer,
            ),
            kind: omdurman_rules::FireKind::Howitzer,
            firers: vec![UnitId::BritishArmy_0_0],
            target_hex: HexCoord::new(23, 10),
            at_fort: false,
            factor_row: FireFactorRow::Row01to05,
            modifiers: Vec::new(),
            gunboat_maxims: Vec::new(),
        };
        let mut card = build_fire_card(
            &attack,
            FireResolution {
                roll: DieRoll::Nine,
                total_modifier: 0,
                modified_roll: DieRoll::Nine,
                factor_row: FireFactorRow::Row01to05,
                effective_factor: 2,
                result: CombatResult::Eliminate(2),
            },
            &[],
            &[],
            &gs,
        );
        let landed = HexCoord::new(24, 10);
        relocate_scattered(
            &mut card,
            &attack,
            CombatResult::Eliminate(2),
            &[],
            landed,
            &gs,
        );
        assert_eq!(card.target_hex, landed);
        let line = log_line(&card);
        assert!(
            line.starts_with("Shell aimed at ") && line.contains(" scattered to (24,10) (§6.64): "),
            "{line}"
        );
        assert!(
            line.ends_with("No effect \u{2014} the shell fell on an empty hex"),
            "{line}"
        );
    }
}

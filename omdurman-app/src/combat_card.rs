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
use omdurman_rules::effects::Observation;
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

/// Bundle of one melee side's resolution outputs (roll, modifiers, result,
/// factor, losses) so [`build_melee_card`] stays under clippy's argument limit.
/// Used for both the attacker and defender halves of a melee resolution.
struct MeleeSideResolution<'a> {
    roll: DieRoll,
    total_modifier: i16,
    modified_roll: DieRoll,
    result: CombatResult,
    factor: u16,
    losses: &'a [UnitId],
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
            .add_systems(
                EguiPrimaryContextPass,
                // Runs after the charts sheet so the card can shift left of
                // the sheet / peek tab (see `ScreenLayout::right_inset`).
                combat_card_ui
                    .after(crate::charts::chart_sheet_ui)
                    .run_if(crate::map_view_active),
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
    /// Seconds shown; frozen while hovered or pinned.
    age: f32,
    /// Hover / click-to-pin state (see [`crate::ui::CardHold`]).
    hold: crate::ui::CardHold,
    /// Stable per-card id (egui click target), assigned on push.
    serial: u64,
}

#[derive(Resource, Default)]
struct CombatCardQueue {
    entries: Vec<CombatCardEntry>,
    next_serial: u64,
}

impl CombatCardQueue {
    /// Queue a card, evicting the oldest unpinned card beyond
    /// [`MAX_ENTRIES`].
    fn push(&mut self, mut entry: CombatCardEntry) {
        self.next_serial += 1;
        entry.serial = self.next_serial;
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
    game_state: Option<Res<GameStateResource>>,
) {
    let gs = game_state.as_deref().map(|r| &r.0);
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
                // `range`/`band` are surfaced by the bot log; the card keeps
                // its existing layout.
                ..
            } => build_fire_card(
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
            ),
            Observation::MeleeResolved {
                attack,
                attacker_roll,
                attacker_total_modifier,
                attacker_modified_roll,
                attacker_result,
                defender_roll,
                defender_total_modifier,
                defender_modified_roll,
                defender_result,
                attacker_factor,
                defender_factor,
                attacker_losses,
                defender_losses,
                mandatory_advance,
                paragraphs,
            } => build_melee_card(
                attack,
                *mandatory_advance,
                MeleeSideResolution {
                    roll: *attacker_roll,
                    total_modifier: *attacker_total_modifier,
                    modified_roll: *attacker_modified_roll,
                    result: *attacker_result,
                    factor: *attacker_factor,
                    losses: attacker_losses,
                },
                MeleeSideResolution {
                    roll: *defender_roll,
                    total_modifier: *defender_total_modifier,
                    modified_roll: *defender_modified_roll,
                    result: *defender_result,
                    factor: *defender_factor,
                    losses: defender_losses,
                },
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
        queue.push(entry);
    }
}

fn build_fire_card(
    attack: &FireAttack,
    resolution: FireResolution,
    eliminations: &[UnitId],
    paragraphs: &[String],
    gs: Option<&omdurman_rules::effects::GameState>,
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
        units_label: list_units(&attack.firers, gs),
        factor: effective_factor,
        factor_row_label: factor_row_label(factor_row),
        roll,
        modifiers: fire_modifier_lines(attack, total_modifier),
        net_modifier: total_modifier,
        modified_roll,
        result_label: describe_result(result),
        losses: list_unit_names(eliminations, gs),
    };
    let hex_label = target_hex_label(attack.target_hex, gs);
    CombatCardEntry {
        kind: CombatKind::Fire,
        target_hex: attack.target_hex,
        hex_label,
        attacker,
        defender: None,
        paragraphs: paragraphs.to_vec(),
        note: None,
        age: 0.0,
        hold: crate::ui::CardHold::default(),
        serial: 0,
    }
}

fn build_melee_card(
    attack: &MeleeAttack,
    mandatory_advance: Option<u8>,
    attacker: MeleeSideResolution,
    defender: MeleeSideResolution,
    paragraphs: &[String],
    gs: Option<&omdurman_rules::effects::GameState>,
) -> CombatCardEntry {
    let MeleeSideResolution {
        roll: attacker_roll,
        total_modifier: attacker_total_modifier,
        modified_roll: attacker_modified_roll,
        result: attacker_result,
        factor: attacker_factor,
        losses: attacker_losses,
    } = attacker;
    let MeleeSideResolution {
        roll: defender_roll,
        total_modifier: defender_total_modifier,
        modified_roll: defender_modified_roll,
        result: defender_result,
        factor: defender_factor,
        losses: defender_losses,
    } = defender;
    let att_row = FireFactorRow::from_total(attacker_factor);
    let def_row = FireFactorRow::from_total(defender_factor);
    let attacker = CombatSide {
        player: attack.attacker_player,
        units_label: list_units(&attack.attackers, gs),
        factor: attacker_factor,
        factor_row_label: factor_row_label(att_row),
        roll: attacker_roll,
        modifiers: melee_modifier_lines(&attack.attacker_modifiers, attacker_total_modifier),
        net_modifier: attacker_total_modifier,
        modified_roll: attacker_modified_roll,
        result_label: describe_result(attacker_result),
        losses: list_unit_names(attacker_losses, gs),
    };
    let defender_player = attack.attacker_player.opponent();
    let defender = CombatSide {
        player: defender_player,
        units_label: list_units(&attack.defenders, gs),
        factor: defender_factor,
        factor_row_label: factor_row_label(def_row),
        roll: defender_roll,
        modifiers: melee_modifier_lines(&attack.defender_modifiers, defender_total_modifier),
        net_modifier: defender_total_modifier,
        modified_roll: defender_modified_roll,
        result_label: describe_result(defender_result),
        losses: list_unit_names(defender_losses, gs),
    };
    let hex_label = target_hex_label(attack.defender_hex, gs);
    let paragraphs = paragraphs.to_vec();
    // §7.6: a Dervish melee that clears the hex carries a *mandatory*
    // advance, which the engine has already made -- say so on the card.
    let note = mandatory_advance.map(|n| {
        let units = if n == 1 { "unit" } else { "units" };
        format!("{n} surviving attacking {units} advanced into the hex (mandatory, §7.6).")
    });
    CombatCardEntry {
        kind: CombatKind::Melee,
        target_hex: attack.defender_hex,
        hex_label,
        attacker,
        defender: Some(defender),
        paragraphs,
        note,
        age: 0.0,
        hold: crate::ui::CardHold::default(),
        serial: 0,
    }
}

/// Translate a fire attack's modifiers into display lines, plus the engine-
/// side terrain modifier (which isn't in `attack.modifiers` -- it's derived
/// from `state.board` at resolution time). The terrain line is the difference
/// between the engine's reported `total_modifier` and the sum of the
/// caller-supplied modifiers.
fn fire_modifier_lines(attack: &FireAttack, total_modifier: i16) -> Vec<ModifierLine> {
    let mut out: Vec<ModifierLine> = attack
        .modifiers
        .iter()
        .map(|m| describe_fire_modifier(*m))
        .collect();
    let app_supplied: i16 = attack.modifiers.iter().map(|m| m.die_modifier()).sum();
    let terrain_mod = total_modifier - app_supplied;
    if terrain_mod != 0 {
        out.push(ModifierLine {
            label: format!("{terrain_mod:+} terrain defence"),
            paragraph: "6.23".into(),
        });
    }
    out
}

fn melee_modifier_lines(modifiers: &[MeleeModifier], total_modifier: i16) -> Vec<ModifierLine> {
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

fn factor_row_label(row: FireFactorRow) -> String {
    match row {
        FireFactorRow::Row01to05 => "1-5".into(),
        FireFactorRow::Row06to10 => "6-10".into(),
        FireFactorRow::Row11to15 => "11-15".into(),
        FireFactorRow::Row16to20 => "16-20".into(),
        FireFactorRow::Row21to25 => "21-25".into(),
        FireFactorRow::Row26to30 => "26-30".into(),
        FireFactorRow::Row31to35 => "31-35".into(),
        FireFactorRow::Row36to40 => "36-40".into(),
        FireFactorRow::Row41Plus => "41+".into(),
    }
}

/// Resolve a slice of [`UnitId`]s into a comma-separated list of short unit
/// names. Units that no longer exist in the game state (eliminated by this
/// very combat) are named from the static counter roster.
fn list_units(ids: &[UnitId], gs: Option<&omdurman_rules::effects::GameState>) -> String {
    if ids.is_empty() {
        return "—".into();
    }
    list_unit_names(ids, gs).join(", ")
}

/// Like [`list_units`] but returns the per-unit names separately, for casualty
/// lists where each loss is its own line item.
fn list_unit_names(ids: &[UnitId], gs: Option<&omdurman_rules::effects::GameState>) -> Vec<String> {
    ids.iter()
        .map(|id| crate::combat_ui::unit_name(*id, gs))
        .collect()
}

/// A short label for any landmark at the target hex (fort, palace, etc.).
/// Returns an empty string for an ordinary hex so the renderer can skip it.
fn target_hex_label(hex: HexCoord, gs: Option<&omdurman_rules::effects::GameState>) -> String {
    let Some(gs) = gs else { return String::new() };
    gs.board
        .location_at(hex)
        .map(|loc| loc.to_string())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Render: queue -> egui cards
// ---------------------------------------------------------------------------

fn combat_card_ui(
    mut contexts: EguiContexts,
    mut queue: ResMut<CombatCardQueue>,
    time: Res<Time>,
    mut rulebook: ResMut<Rulebook>,
    layout: Res<crate::ScreenLayout>,
) {
    let dt = time.delta_secs();
    for entry in &mut queue.entries {
        entry.hold.age(&mut entry.age, dt, CARD_TTL, CARD_FADE);
    }
    queue.entries.retain(|e| e.age < CARD_TTL);
    if queue.entries.is_empty() {
        return;
    }
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
            ui.set_max_width(360.0);
            // A volley of resolutions outgrows the window: scroll the column
            // instead of letting the oldest cards run off the bottom unread.
            let max_height = ctx_height - layout.top_bar_height - 60.0;
            egui::ScrollArea::vertical()
                .id_salt("combat_cards_scroll")
                .max_height(max_height.max(120.0))
                .show(ui, |ui| {
                    // Newest at the top: render in reverse so the freshest card is
                    // closest to the screen edge.
                    for entry in queue.entries.iter_mut().rev() {
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
        },
    );

    if let Some(sec) = clicked_section {
        crate::rulebook::request_section(&mut rulebook, &sec);
    }
    ctx.request_repaint();
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
            let hex_str = if entry.hex_label.is_empty() {
                format!("at ({},{})", entry.target_hex.q, entry.target_hex.r)
            } else {
                format!(
                    "at {} ({},{})",
                    entry.hex_label, entry.target_hex.q, entry.target_hex.r
                )
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
                ui.label(
                    egui::RichText::new(note)
                        .color(a(crate::ui::palette::INK))
                        .size(12.0)
                        .italics(),
                );
            }

            ui.add_space(4.0);
            // Footer: paragraph chips.
            if !entry.paragraphs.is_empty() {
                let refs: Vec<&str> = entry.paragraphs.iter().map(String::as_str).collect();
                if let Some(p) = rulebook.render_ref_chips(ui, &refs) {
                    clicked = Some(p);
                }
            }
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
    // §6.51: a side with no melee factor (Anglo-Egyptian leaders alone)
    // makes no roll -- the engine resolves it as no effect.
    let summary = if side.factor == 0 {
        "no melee factor — no roll (§6.51)".to_string()
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
    ui.label(
        egui::RichText::new(summary)
            .color(a(crate::ui::palette::FAINT_INK))
            .size(12.0)
            .monospace(),
    );
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
                let title = rulebook.title_of(&line.paragraph);
                let chip = if let Some(t) = title {
                    format!("§{} {}", line.paragraph, t)
                } else {
                    format!("§{}", line.paragraph)
                };
                if ui
                    .add(
                        egui::Label::new(
                            egui::RichText::new(chip)
                                .color(a(crate::ui::palette::INK))
                                .size(11.0)
                                .underline(),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .clicked()
                {
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

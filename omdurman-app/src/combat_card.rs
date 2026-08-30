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
use omdurman_rules::{
    CombatResult, DieRoll, FireAttack, FireModifier, MeleeAttack, MeleeModifier, UnitId,
};
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
                combat_card_ui.after(crate::charts::chart_sheet_ui),
            );
    }
}

// ---------------------------------------------------------------------------
// Resolved card model
// ---------------------------------------------------------------------------

/// One row of the modifier breakdown: the die-roll delta and the rulebook
/// paragraph that authorises it. Both are pre-resolved strings so the card
/// never has to reach back into the rules engine at render time (when state
/// may have moved on).
#[derive(Clone)]
struct ModifierLine {
    label: String,
    paragraph: String,
}

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
    age: f32,
}

#[derive(Resource, Default)]
struct CombatCardQueue {
    entries: Vec<CombatCardEntry>,
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
                mandatory_advance: _,
                paragraphs,
            } => build_melee_card(
                attack,
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
            _ => continue,
        };
        queue.entries.push(entry);
        if queue.entries.len() > MAX_ENTRIES {
            queue.entries.remove(0);
        }
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
        age: 0.0,
    }
}

fn build_melee_card(
    attack: &MeleeAttack,
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
    CombatCardEntry {
        kind: CombatKind::Melee,
        target_hex: attack.defender_hex,
        hex_label,
        attacker,
        defender: Some(defender),
        paragraphs: paragraphs.to_vec(),
        age: 0.0,
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

fn describe_fire_modifier(m: FireModifier) -> ModifierLine {
    let (label, paragraph) = match m {
        FireModifier::AngloEgyptianDirectFire => {
            ("+1 Anglo-Egyptian direct fire".to_string(), "6.24")
        }
        FireModifier::BrigadeIntegrity => ("+1 brigade integrity".to_string(), "5.54"),
        FireModifier::Terrain(n) => (format!("{n:+} terrain defence"), "6.23"),
        FireModifier::ZaribaThornHedge => ("-2 zariba thorn hedge".to_string(), "9.231"),
        FireModifier::ZaribaTrenchEntrenched => {
            ("-4 zariba trench (entrenched)".to_string(), "9.232")
        }
    };
    ModifierLine {
        label,
        paragraph: paragraph.into(),
    }
}

fn describe_melee_modifier(m: MeleeModifier) -> ModifierLine {
    let (label, paragraph) = match m {
        MeleeModifier::DervishStandard => ("+2 Dervish standard".to_string(), "7.7"),
        MeleeModifier::AngloEgyptianStandard => ("+1 Anglo-Egyptian standard".to_string(), "7.7"),
        MeleeModifier::DervishVsTrenchedDefender => {
            ("-2 vs. entrenched defender".to_string(), "9.232")
        }
    };
    ModifierLine {
        label,
        paragraph: paragraph.into(),
    }
}

fn describe_result(result: CombatResult) -> String {
    match result {
        CombatResult::NoEffect => "No effect".to_string(),
        CombatResult::Disrupt => "Disrupt".to_string(),
        CombatResult::Eliminate(n) => format!("Eliminate {n}"),
    }
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
/// names. Units that no longer exist in the game state (already eliminated by
/// a later effect) fall back to a debug-style label so the card still names
/// something -- the casualty list will identify exactly who was lost here.
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
        .map(|id| match gs.and_then(|s| s.find_unit(*id)) {
            Some(u) => u.profile.identity.short_label(),
            None => format!("unit {id:?}"),
        })
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
        entry.age += dt;
    }
    queue.entries.retain(|e| e.age < CARD_TTL);
    if queue.entries.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

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
            // Newest at the top: render in reverse so the freshest card is
            // closest to the screen edge.
            for entry in queue.entries.iter().rev() {
                let fade = ((CARD_TTL - entry.age) / CARD_FADE).clamp(0.0, 1.0);
                if let Some(sec) = draw_card(ui, entry, fade, &rulebook) {
                    clicked_section = Some(sec);
                }
                ui.add_space(6.0);
            }
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
    fade: f32,
    rulebook: &Rulebook,
) -> Option<String> {
    let a = |c: egui::Color32| c.gamma_multiply(fade);
    let mut clicked: Option<String> = None;
    let kind_label = match entry.attacker.player {
        Player::AngloEgyptian => "Anglo-Egyptian",
        Player::Dervish => "Dervish",
    };
    let header_word = match entry.kind {
        CombatKind::Fire => "FIRE COMBAT",
        CombatKind::Melee => "MELEE COMBAT",
    };

    crate::ui::paper_frame(egui::Stroke::new(2.0, a(crate::ui::palette::INK)))
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

            // Attacker side block.
            draw_side(ui, "Firers:", &entry.attacker, a, rulebook, &mut clicked);
            // Defender block for melee (symmetric).
            if let Some(defender) = &entry.defender {
                ui.add_space(4.0);
                draw_side(ui, "Defenders:", defender, a, rulebook, &mut clicked);
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
    clicked
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
    let summary = format!(
        "factor {} (row {}) — rolled {}{} = {}  →  {}",
        side.factor,
        side.factor_row_label,
        side.roll.value(),
        mod_str,
        side.modified_roll.value(),
        side.result_label,
    );
    ui.label(
        egui::RichText::new(summary)
            .color(a(crate::ui::palette::FAINT_INK))
            .size(12.0)
            .monospace(),
    );
    // Modifier breakdown, each line deep-linking to its rulebook paragraph.
    if !side.modifiers.is_empty() {
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
                .color(a(egui::Color32::from_rgb(150, 40, 40)))
                .size(12.0),
        );
    }
}

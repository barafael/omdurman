//! The die: every d10 the game rolls tumbles onto the board, seen from above,
//! comes to rest showing what it rolled, and a slip under it says what the
//! roll does -- the factors and column it was read on, the modifiers, where a
//! shell landed, the result and who it costs.
//!
//! The dice travel inside the effects (pre-rolled by the acting peer), so the
//! throw is raised where a *live* sequenced echo is applied
//! (`net_socket::apply_to_board` fills [`LiveDice`] from the effect and the
//! observations its apply produced): every peer sees the same numbers at the
//! moment the roll takes effect, and a replay or a history install throws
//! nothing. A melee's two dice ride in its declaration but are thrown when it
//! resolves -- before that the defender is still choosing whether to retreat.
//!
//! Presentation only: the result is already in the engine state when the die
//! starts to roll.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_rules::effects::{GameEffect, GameState, MeleeSide, Observation};
use omdurman_rules::{CombatResult, DieRoll, FireAttack, UnitId};
use omdurman_types::HexCoord;

use super::MotionSettings;
use crate::combat_card::{
    fire_modifier_lines, fire_result_label, list_unit_names, melee_modifier_lines, target_hex_label,
};
use crate::combat_ui::{ModifierLine, describe_result};
use crate::ui::palette;

/// Seconds a die tumbles before it rests.
const ROLL_SECS: f32 = 0.95;
/// Seconds a throw rests, readable, before it fades: a base, plus a little
/// for every line of its slip.
const REST_SECS: f32 = 2.2;
const REST_PER_LINE: f32 = 0.6;
const REST_MAX: f32 = 7.0;
/// Seconds of the fade.
const FADE_SECS: f32 = 0.45;
/// The die's radius on screen, in points.
const DIE_RADIUS: f32 = 30.0;
/// Space between two resting dice, centre to centre.
const DIE_SPACING: f32 = 92.0;
/// The slip's distance from the board's bottom edge, and its width.
const SLIP_MARGIN: f32 = 28.0;
const SLIP_WIDTH: f32 = 440.0;
/// The egui font family of the numerals (OPTIMaseltov, registered by
/// `ui_plugin::fonts::setup_egui_fonts`).
pub const DIE_FONT: &str = "DieNumerals";

/// One die of a throw: what it decides, and what it shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Roll {
    pub label: &'static str,
    /// 1..=10.
    pub value: u16,
}

impl Roll {
    fn new(label: &'static str, roll: DieRoll) -> Self {
        Self {
            label,
            value: roll.value(),
        }
    }
}

/// What one throw decided, for the slip: a heading, the working, and the
/// outcome.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Note {
    pub title: String,
    pub lines: Vec<String>,
    pub outcome: Vec<String>,
    /// The outcome costs someone a unit (drawn in casualty ink).
    pub losses: bool,
}

/// The dice of one live effect and what they decided.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Throw {
    pub dice: Vec<Roll>,
    pub notes: Vec<Note>,
}

impl Throw {
    fn line_count(&self) -> usize {
        self.notes
            .iter()
            .map(|n| 1 + n.lines.len() + n.outcome.len())
            .sum()
    }
}

/// The throw a live effect made: its dice, read from the effect and from the
/// `observations` its apply produced, and what they decided. `gs` is the
/// state after the apply (it names the units, the fallen included). `None`
/// for an effect that rolled nothing -- including a melee whose defenders
/// all withdrew, which is called off without a roll.
pub fn throw_of(
    effect: &GameEffect,
    observations: &[Observation],
    gs: &GameState,
) -> Option<Throw> {
    let mut throw = Throw::default();
    match effect {
        GameEffect::FireCombat { .. } | GameEffect::HowitzerFire { .. } => {
            // Every shell's impact roll first (§6.64), then the roll of each
            // attack they made: the one on the intended hex, and one per
            // shell gone astray.
            if let GameEffect::HowitzerFire { shells, .. } = effect {
                throw
                    .dice
                    .extend(shells.iter().map(|shell| Roll::new("Shell", shell.impact)));
            }
            for observation in observations {
                if let Observation::FireResolved {
                    attack,
                    roll,
                    total_modifier,
                    modified_roll,
                    factor_row,
                    effective_factor,
                    result,
                    eliminations,
                    range,
                    band,
                    impact,
                    ..
                } = observation
                {
                    let stray = impact.is_some_and(|(_, landed)| landed != attack.target_hex);
                    throw
                        .dice
                        .push(Roll::new(if stray { "Stray shell" } else { "Fire" }, *roll));
                    throw.notes.push(fire_note(
                        attack,
                        FireWorking {
                            roll: *roll,
                            total_modifier: *total_modifier,
                            modified_roll: *modified_roll,
                            row: factor_row.label(),
                            factor: *effective_factor,
                            range: *range,
                            band: band.as_deref(),
                            impact: *impact,
                        },
                        *result,
                        eliminations,
                        gs,
                    ));
                }
            }
        }
        GameEffect::ResolveMelee => {
            for observation in observations {
                if let Observation::MeleeResolved {
                    attack,
                    attacker,
                    defender,
                    mandatory_advance,
                    ..
                } = observation
                {
                    throw.dice.push(Roll::new("Attackers", attacker.roll));
                    throw.dice.push(Roll::new("Defenders", defender.roll));
                    let mut note = Note {
                        title: format!("Melee at {}", place(attack.defender_hex, gs)),
                        lines: vec![
                            melee_working(
                                "Attackers",
                                attacker,
                                &melee_modifier_lines(
                                    &attack.attacker_modifiers,
                                    attacker.total_modifier,
                                ),
                            ),
                            melee_working(
                                "Defenders",
                                defender,
                                &melee_modifier_lines(
                                    &attack.defender_modifiers,
                                    defender.total_modifier,
                                ),
                            ),
                        ],
                        outcome: vec![
                            melee_outcome("Defenders", attacker.result, &attacker.losses, gs),
                            melee_outcome("Attackers", defender.result, &defender.losses, gs),
                        ],
                        losses: !attacker.losses.is_empty() || !defender.losses.is_empty(),
                    };
                    if let Some(n) = mandatory_advance {
                        let units = if *n == 1 { "unit" } else { "units" };
                        note.outcome
                            .push(format!("{n} attacking {units} advance into the hex (§7.6)"));
                    }
                    throw.notes.push(note);
                }
            }
        }
        GameEffect::RiverMine { roll, .. } => {
            throw.dice.push(Roll::new("Mine", *roll));
            for observation in observations {
                if let Observation::MineResolved {
                    gunboat, result, ..
                } = observation
                {
                    let (outcome, losses) = match result {
                        omdurman_rules::MineResult::NoEffect => ("no effect".to_string(), false),
                        omdurman_rules::MineResult::EnginesLost => (
                            "engines lost: she drifts with the current".to_string(),
                            false,
                        ),
                        omdurman_rules::MineResult::Sunk => ("sunk".to_string(), true),
                    };
                    throw.notes.push(Note {
                        title: format!(
                            "{} strikes a mine",
                            crate::combat_ui::unit_name(*gunboat, gs)
                        ),
                        lines: vec!["1-4 no effect, 5-7 engines lost, 8-10 sunk (§10.12)".into()],
                        outcome: vec![capitalise(&outcome)],
                        losses,
                    });
                }
            }
        }
        GameEffect::SinkChain { roll, .. } => {
            throw.dice.push(Roll::new("Chain", *roll));
            for observation in observations {
                if let Observation::ChainFiredAt { result, sunk, .. } = observation {
                    throw.notes.push(Note {
                        title: "The batteries fire on the river chain".into(),
                        lines: vec!["Eliminate 3 or more sinks it (§10.23)".into()],
                        outcome: vec![format!(
                            "{}: {}",
                            describe_result(*result),
                            if *sunk {
                                "the chain is sunk"
                            } else {
                                "the chain holds"
                            }
                        )],
                        losses: false,
                    });
                }
            }
        }
        GameEffect::ArtilleryBreachWall { roll, .. } => {
            throw.dice.push(Roll::new("Breach", *roll));
            for observation in observations {
                if let Observation::WallBreached {
                    breached,
                    row,
                    adjacent_eliminated,
                    ..
                } = observation
                {
                    let mut lines = Vec::new();
                    if let Some(row) = row {
                        lines.push(format!("Read on the {} column", row.label()));
                    }
                    lines.push("Eliminate 2 or more breaches the wall (§6.63)".into());
                    let mut outcome = vec![
                        if *breached {
                            "The wall is breached"
                        } else {
                            "The wall holds"
                        }
                        .to_string(),
                    ];
                    if let Some(lost) = adjacent_eliminated {
                        outcome.push(format!("Lost: {}", crate::combat_ui::unit_name(*lost, gs)));
                    }
                    throw.notes.push(Note {
                        title: "Artillery fire at the wall".into(),
                        lines,
                        outcome,
                        losses: adjacent_eliminated.is_some(),
                    });
                }
            }
        }
        GameEffect::RollDesertion { roll } => {
            throw.dice.push(Roll::new("Desertion", *roll));
            let n = omdurman_rules::effects::desertion_count(*roll);
            throw.notes.push(Note {
                title: "Dervish desertion".into(),
                lines: vec!["One and a half times the roll, rounded up (§8.2)".into()],
                outcome: vec![format!(
                    "{n} Dervish {} desert",
                    if n == 1 { "unit" } else { "units" }
                )],
                losses: n > 0,
            });
        }
        // The desertion carries the die `RollDesertion` already threw; the
        // melee declaration's dice are thrown when it resolves.
        GameEffect::DervishDesertion { .. }
        | GameEffect::DeclareMelee { .. }
        | GameEffect::AdvancePhase
        | GameEffect::AdvancePhaseFrom { .. }
        | GameEffect::MoveUnit { .. }
        | GameEffect::AllocateFire { .. }
        | GameEffect::TakeLosses { .. }
        | GameEffect::RetreatBeforeMelee { .. }
        | GameEffect::DeclineRetreatBeforeMelee
        | GameEffect::AdvanceAfterCombat { .. }
        | GameEffect::ConstructZariba { .. }
        | GameEffect::Demolition { .. }
        | GameEffect::PlaceReinforcements(_)
        | GameEffect::FriendliesTransport(_)
        | GameEffect::DeployUnit(_)
        | GameEffect::RemoveDeployedUnit { .. }
        | GameEffect::PlaceMine { .. }
        | GameEffect::PlaceChain { .. }
        | GameEffect::ConfirmSetupReady { .. } => {}
    }
    (!throw.dice.is_empty()).then_some(throw)
}

/// What a fire resolution was read on, for [`fire_note`].
struct FireWorking<'a> {
    roll: DieRoll,
    total_modifier: i16,
    modified_roll: DieRoll,
    row: &'static str,
    factor: u16,
    range: Option<u16>,
    band: Option<&'a str>,
    impact: Option<(DieRoll, HexCoord)>,
}

fn fire_note(
    attack: &FireAttack,
    working: FireWorking<'_>,
    result: CombatResult,
    eliminations: &[UnitId],
    gs: &GameState,
) -> Note {
    let landed = working
        .impact
        .map_or(attack.target_hex, |(_, landed)| landed);
    let mut lines = Vec::new();
    if let Some((impact, landed)) = working.impact {
        lines.push(if landed == attack.target_hex {
            format!("Shell impact {}: on target (§6.64)", impact.value())
        } else {
            format!(
                "Shell impact {}: scattered to {} (§6.64)",
                impact.value(),
                place(landed, gs)
            )
        });
    }
    let mut factors = format!(
        "{} fire {} \u{2192} {} column",
        working.factor,
        if working.factor == 1 {
            "factor"
        } else {
            "factors"
        },
        working.row
    );
    match (working.range, working.band) {
        (Some(range), Some(band)) => factors.push_str(&format!(" (range {range}, {band})")),
        (Some(range), None) => factors.push_str(&format!(" (range {range})")),
        _ => {}
    }
    lines.push(factors);
    lines.push(roll_working(
        working.roll,
        &fire_modifier_lines(attack, working.total_modifier),
        working.modified_roll,
    ));
    let mut outcome = vec![fire_result_label(attack, result, eliminations, gs)];
    if !eliminations.is_empty() {
        outcome.push(format!(
            "Lost: {}",
            list_unit_names(eliminations, gs).join(", ")
        ));
    }
    Note {
        title: format!("Fire on {}", place(landed, gs)),
        lines,
        outcome,
        losses: !eliminations.is_empty(),
    }
}

/// "Roll 6, +1 brigade integrity, -1 terrain defence = 6".
fn roll_working(roll: DieRoll, modifiers: &[ModifierLine], modified: DieRoll) -> String {
    if modifiers.is_empty() {
        return format!("Roll {}", roll.value());
    }
    let mods: Vec<&str> = modifiers.iter().map(|m| m.label.as_str()).collect();
    format!(
        "Roll {}, {} = {}",
        roll.value(),
        mods.join(", "),
        modified.value()
    )
}

fn melee_working(side: &str, roll: &MeleeSide, modifiers: &[ModifierLine]) -> String {
    format!(
        "{side}: {} melee {} \u{2192} {} column; {}",
        roll.factor,
        if roll.factor == 1 {
            "factor"
        } else {
            "factors"
        },
        omdurman_rules::combat_results_table::FireFactorRow::from_total(roll.factor).label(),
        roll_working(roll.roll, modifiers, roll.modified_roll).to_lowercase()
    )
}

/// What one side's roll did to the *other* side (melee is simultaneous).
fn melee_outcome(struck: &str, result: CombatResult, losses: &[UnitId], gs: &GameState) -> String {
    let mut line = format!("{}: {}", struck, describe_result(result));
    if !losses.is_empty() {
        line.push_str(&format!(
            " \u{2014} lost {}",
            list_unit_names(losses, gs).join(", ")
        ));
    }
    line
}

/// A hex by its landmark name, else by its coordinates.
fn place(hex: HexCoord, gs: &GameState) -> String {
    let label = target_hex_label(hex, gs);
    if label.is_empty() {
        format!("({},{})", hex.q, hex.r)
    } else {
        label
    }
}

fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Throws raised by live echoes, awaiting [`take_live_dice`]. Never written
/// by a rebuild.
#[derive(Resource, Default)]
pub struct LiveDice(pub Vec<Throw>);

/// The throw on the board and when it was thrown. A newer throw replaces it
/// (an AI fire phase must not queue up behind the animation; the combat
/// cards keep every resolution).
#[derive(Resource, Default)]
pub struct DiceTray(Option<OnTray>);

struct OnTray {
    throw: Throw,
    /// `Time::elapsed_secs` at the throw.
    started: f32,
    /// Varies the tumble from one throw to the next.
    seed: u32,
}

/// Move the latest live throw onto the tray. Runs every frame on every
/// screen, so a throw raised while the board is hidden (a review) has aged
/// out by the time the board is back instead of rolling late.
pub(super) fn take_live_dice(
    mut live: ResMut<LiveDice>,
    mut tray: ResMut<DiceTray>,
    settings: Res<MotionSettings>,
    time: Res<Time>,
    mut counter: Local<u32>,
) {
    let Some(throw) = live.0.drain(..).next_back() else {
        return;
    };
    if !settings.effects() {
        return;
    }
    *counter = counter.wrapping_add(1);
    tray.0 = Some(OnTray {
        throw,
        started: time.elapsed_secs(),
        seed: counter.wrapping_mul(0x9E37_79B9) ^ (time.elapsed().as_millis() as u32),
    });
}

/// Leaving the game drops the throw.
pub(super) fn clear_tray(mut tray: ResMut<DiceTray>, mut live: ResMut<LiveDice>) {
    tray.0 = None;
    live.0.clear();
}

/// Draw the throw: each die slides in from the right, spinning and
/// bouncing, its top face flickering through numbers as it tumbles over,
/// and settles on its roll; the slip under the dice says what the roll
/// decided. Then the throw rests a moment and fades.
pub(super) fn dice_tray_ui(
    mut contexts: EguiContexts,
    mut tray: ResMut<DiceTray>,
    time: Res<Time>,
    settings: Res<MotionSettings>,
    layout: Res<crate::ScreenLayout>,
) {
    let Some(on_tray) = tray.0.as_ref() else {
        return;
    };
    let throw = &on_tray.throw;
    let age = time.elapsed_secs() - on_tray.started;
    let roll_secs = if settings.motion() { ROLL_SECS } else { 0.0 };
    // The last die leaves a little after the first.
    let stagger = |i: usize| i as f32 * 0.08;
    let landed = roll_secs + stagger(throw.dice.len().saturating_sub(1));
    let rest = (REST_SECS + REST_PER_LINE * throw.line_count() as f32).min(REST_MAX);
    let end = landed + rest + FADE_SECS;
    if age >= end || age < 0.0 {
        tray.0 = None;
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let screen = ctx.content_rect();
    let left = screen.left() + layout.left_inset;
    let right = screen.right() - layout.right_inset;
    let centre_x = (left + right) * 0.5;
    let fade = ((end - age) / FADE_SECS).clamp(0.0, 1.0);
    // The slip comes up as the last die lands.
    let slip_alpha = ((age - landed) / 0.3).clamp(0.0, 1.0) * fade;

    let slip = egui::Area::new(egui::Id::new("dice_slip"))
        .order(egui::Order::Foreground)
        .interactable(false)
        .anchor(
            egui::Align2::CENTER_BOTTOM,
            egui::vec2(centre_x - screen.center().x, -SLIP_MARGIN),
        )
        .show(ctx, |ui| {
            ui.set_opacity(slip_alpha);
            ui.set_max_width(SLIP_WIDTH);
            crate::ui::frames::paper(egui::Stroke::new(1.0, palette::FAINT_INK))
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| draw_slip(ui, throw));
        });
    let dice_y = slip.response.rect.top() - 22.0 - DIE_RADIUS - 8.0;

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("dice_tray"),
    ));
    let count = throw.dice.len();
    for (i, roll) in throw.dice.iter().enumerate() {
        let offset = (i as f32 - (count as f32 - 1.0) * 0.5) * DIE_SPACING;
        let rest = egui::pos2(centre_x + offset, dice_y);
        let seed = on_tray
            .seed
            .wrapping_add((i as u32).wrapping_mul(0x85EB_CA6B));
        let since = age - stagger(i);
        let pose = if roll_secs > 0.0 {
            tumble((since / roll_secs).clamp(0.0, 1.0), seed, roll.value)
        } else {
            Pose::resting(roll.value, seed)
        };
        draw_die(&painter, rest, &pose, settle_pulse(since - roll_secs), fade);
        if pose.at_rest {
            let alpha = ((since - roll_secs) / 0.25).clamp(0.0, 1.0) * fade;
            draw_die_label(
                &painter,
                rest + egui::vec2(0.0, DIE_RADIUS + 14.0),
                roll.label,
                alpha,
            );
        }
    }
    ctx.request_repaint();
}

/// The slip's text: per resolution a heading, its working, its outcome.
fn draw_slip(ui: &mut egui::Ui, throw: &Throw) {
    ui.spacing_mut().item_spacing.y = 2.0;
    for (i, note) in throw.notes.iter().enumerate() {
        if i > 0 {
            ui.add_space(4.0);
            ui.separator();
        }
        ui.label(
            egui::RichText::new(&note.title)
                .family(egui::FontFamily::Name("Merriweather".into()))
                .size(14.0)
                .color(palette::INK),
        );
        for line in &note.lines {
            ui.label(
                egui::RichText::new(line)
                    .size(12.0)
                    .color(palette::FAINT_INK),
            );
        }
        let colour = if note.losses {
            palette::INK_LOSS
        } else {
            palette::INK
        };
        for line in &note.outcome {
            ui.label(egui::RichText::new(line).size(13.0).strong().color(colour));
        }
    }
}

/// A die's label on a small dark tab, legible on any part of the map.
fn draw_die_label(painter: &egui::Painter, at: egui::Pos2, text: &str, alpha: f32) {
    let galley = painter.layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(12.0),
        palette::TITLE.gamma_multiply(alpha),
    );
    let rect = egui::Rect::from_center_size(at, galley.size() + egui::vec2(12.0, 4.0));
    painter.rect_filled(
        rect,
        4.0,
        egui::Color32::from_black_alpha((170.0 * alpha) as u8),
    );
    painter.galley(rect.center() - galley.size() * 0.5, galley, palette::TITLE);
}

/// Where a die is and how it lies at one moment of its roll.
struct Pose {
    /// Offset from its resting place, in points.
    offset: egui::Vec2,
    /// Turn about the view axis, in radians.
    angle: f32,
    /// Height above the table, 0..1: the die grows and its shadow drifts.
    height: f32,
    /// Foreshortening across the tumble axis, 0..1 (1: lying flat).
    flat: f32,
    /// The number on top right now.
    face: u16,
    at_rest: bool,
}

impl Pose {
    fn resting(value: u16, seed: u32) -> Self {
        Self {
            offset: egui::Vec2::ZERO,
            angle: rest_angle(seed),
            height: 0.0,
            flat: 1.0,
            face: value,
            at_rest: true,
        }
    }
}

/// A die rests slightly askew, never squared to the screen.
fn rest_angle(seed: u32) -> f32 {
    (unit(seed, 1) - 0.5) * 0.6
}

/// A pseudo-random 0..1 from `seed` and a stream index.
fn unit(seed: u32, stream: u32) -> f32 {
    let mut x = seed ^ stream.wrapping_mul(0x27D4_EB2D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// The die at `u` (0..1) through its roll: thrown in from the right along a
/// shallow arc, bouncing lower each time, spinning down and tumbling over a
/// few times -- each tumble turns another face up -- until it lies still on
/// `value`.
fn tumble(u: f32, seed: u32, value: u16) -> Pose {
    if u >= 1.0 {
        return Pose::resting(value, seed);
    }
    let ease = 1.0 - (1.0 - u).powi(3);
    let start = egui::vec2(
        260.0 + 80.0 * unit(seed, 2),
        -60.0 + 80.0 * (unit(seed, 3) - 0.5),
    );
    let arc = (u * std::f32::consts::PI).sin() * 26.0 * (unit(seed, 4) - 0.5);
    let offset = start * (1.0 - ease) + egui::vec2(0.0, arc);
    let spin = (3.0 + 3.0 * unit(seed, 5)) * if unit(seed, 6) < 0.5 { -1.0 } else { 1.0 };
    let angle = rest_angle(seed) + spin * (1.0 - ease);
    // Three bounces, each lower than the last.
    let height = ((u * 3.0 * std::f32::consts::PI).sin().abs()) * (1.0 - u).powi(2);
    // Tumbles: half a turn over per face change; the last lands flat.
    let flips = 5 + (unit(seed, 7) * 3.0) as u32;
    let phase = flips as f32 * std::f32::consts::PI * ease;
    let flat = 0.55 + 0.45 * phase.cos().abs();
    let k = (phase / std::f32::consts::PI).round() as u32;
    let face = if k >= flips {
        value
    } else {
        // Any face but the one it will rest on, so the result reads as new.
        let mut face = 1 + (unit(seed, 100 + k) * 10.0) as u16 % 10;
        if face == value {
            face = face % 10 + 1;
        }
        face
    };
    Pose {
        offset,
        angle,
        height,
        flat,
        face,
        at_rest: false,
    }
}

/// A short swell when the die comes to rest: 0 before, rising and falling
/// over a third of a second.
fn settle_pulse(since_rest: f32) -> f32 {
    if !(0.0..0.3).contains(&since_rest) {
        return 0.0;
    }
    (since_rest / 0.3 * std::f32::consts::PI).sin()
}

/// The d10 from above: a pentagonal trapezohedron lying on a face shows its
/// top face as a kite, flanked by the slanted faces around it.
fn draw_die(painter: &egui::Painter, rest: egui::Pos2, pose: &Pose, settle: f32, alpha: f32) {
    let scale = DIE_RADIUS * (1.0 + 0.32 * pose.height + 0.08 * settle);
    let centre = rest + pose.offset;
    let (sin, cos) = pose.angle.sin_cos();
    let place = |x: f32, y: f32| {
        let (x, y) = (x * scale, y * scale * pose.flat);
        centre + egui::vec2(x * cos - y * sin, x * sin + y * cos)
    };
    // The outline, and the kite of the top face inside it.
    let outline = [
        (0.0, -1.12),
        (0.92, -0.36),
        (0.80, 0.56),
        (0.0, 1.0),
        (-0.80, 0.56),
        (-0.92, -0.36),
    ];
    let kite = [(0.0, -0.96), (0.70, -0.14), (0.0, 0.80), (-0.70, -0.14)];
    let outline_pts: Vec<egui::Pos2> = outline.iter().map(|&(x, y)| place(x, y)).collect();
    let kite_pts: Vec<egui::Pos2> = kite.iter().map(|&(x, y)| place(x, y)).collect();

    let ivory = egui::Color32::from_rgb(0xF2, 0xE6, 0xC4);
    let side = egui::Color32::from_rgb(0xC9, 0xB6, 0x8A);
    let edge = egui::Color32::from_rgb(0x4A, 0x3E, 0x2A);
    let ink = palette::INK;
    let a = |c: egui::Color32| c.gamma_multiply(alpha);

    // The shadow: offset away from the light, further and softer in the air.
    let lift = egui::vec2(5.0, 7.0) * (1.0 + 2.2 * pose.height);
    let shadow: Vec<egui::Pos2> = outline_pts.iter().map(|p| *p + lift).collect();
    painter.add(egui::Shape::convex_polygon(
        shadow,
        a(egui::Color32::from_black_alpha(
            (110.0 * (1.0 - 0.45 * pose.height)) as u8,
        )),
        egui::Stroke::NONE,
    ));
    painter.add(egui::Shape::convex_polygon(
        outline_pts.clone(),
        a(side),
        egui::Stroke::new(1.4, a(edge)),
    ));
    // The slanted faces' edges, from the top face out to the rim.
    for (k, o) in [(0, 0), (1, 1), (1, 2), (2, 3), (3, 4), (3, 5)] {
        painter.line_segment(
            [kite_pts[k], outline_pts[o]],
            egui::Stroke::new(1.0, a(edge.gamma_multiply(0.8))),
        );
    }
    painter.add(egui::Shape::convex_polygon(
        kite_pts,
        a(ivory),
        egui::Stroke::new(1.2, a(edge)),
    ));

    // The number, turned with the die (and squashed with it mid-tumble).
    let size = if pose.face >= 10 { 0.72 } else { 0.92 } * scale;
    let galley = painter.layout_no_wrap(
        pose.face.to_string(),
        egui::FontId::new(size, egui::FontFamily::Name(DIE_FONT.into())),
        a(ink),
    );
    // Sit the glyphs' centre on the kite's centre (a little above the die's).
    let anchor = place(0.0, -0.08);
    let half = galley.size() * 0.5;
    let pos = anchor - egui::vec2(half.x * cos - half.y * sin, half.x * sin + half.y * cos);
    painter.add(egui::epaint::TextShape::new(pos, galley, a(ink)).with_angle(pose.angle));
    // A six and a nine are told apart by the bar under them.
    if matches!(pose.face, 6 | 9) {
        painter.line_segment(
            [place(-0.18, 0.42), place(0.18, 0.42)],
            egui::Stroke::new(1.6, a(ink)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tumble_ends_on_its_roll_and_at_rest() {
        for seed in [0, 1, 7, 0xDEAD_BEEF] {
            for value in 1..=10 {
                let end = tumble(1.0, seed, value);
                assert_eq!(end.face, value);
                assert!(end.at_rest);
                assert_eq!(end.offset, egui::Vec2::ZERO);
                // Mid-flight it shows faces from the die, never off it.
                for step in 0..100 {
                    let pose = tumble(step as f32 / 100.0, seed, value);
                    assert!((1..=10).contains(&pose.face), "face {}", pose.face);
                    assert!(pose.flat > 0.5 && pose.flat <= 1.0);
                }
            }
        }
    }

    #[test]
    fn the_last_tumble_lands_on_the_roll_before_it_stops() {
        // The face settles before the slide ends, so the number is read
        // while the die is still easing in.
        let pose = tumble(0.97, 3, 8);
        assert_eq!(pose.face, 8);
    }

    #[test]
    fn the_desertion_roll_says_how_many_desert() {
        let gs = GameState::new(omdurman_types::Scenario::Campaign);
        let throw = throw_of(
            &GameEffect::RollDesertion {
                roll: DieRoll::Seven,
            },
            &[],
            &gs,
        )
        .expect("the desertion die is thrown");
        assert_eq!(
            throw.dice,
            vec![Roll {
                label: "Desertion",
                value: 7
            }]
        );
        assert_eq!(throw.notes[0].outcome, vec!["11 Dervish units desert"]);
    }

    #[test]
    fn an_effect_without_dice_throws_nothing() {
        let gs = GameState::new(omdurman_types::Scenario::Campaign);
        assert!(throw_of(&GameEffect::AdvancePhase, &[], &gs).is_none());
        // A melee every defender withdrew from is called off unrolled.
        let lapsed = [Observation::MeleeLapsed {
            attacker_hex: HexCoord::new(0, 0),
            defender_hex: HexCoord::new(1, 0),
        }];
        assert!(throw_of(&GameEffect::ResolveMelee, &lapsed, &gs).is_none());
    }

    #[test]
    fn a_roll_without_modifiers_reads_as_the_roll() {
        assert_eq!(roll_working(DieRoll::Four, &[], DieRoll::Four), "Roll 4");
        let mods = [ModifierLine {
            label: "+1 brigade integrity".into(),
            paragraph: "5.54".into(),
        }];
        assert_eq!(
            roll_working(DieRoll::Four, &mods, DieRoll::Five),
            "Roll 4, +1 brigade integrity = 5"
        );
    }
}

//! Phase & unit action guide (§decision: discoverability).
//!
//! A small panel inside the right sidebar that tells the player, in plain
//! language:
//!
//! * what **phase** they are in and what actions the rulebook authorises in it,
//! * what the **selected unit** is and what it can contribute (factors, weapon,
//!   remaining movement),
//! * a deep link into the Rulebook tab for each action's authorising
//!   paragraph.
//!
//! The goal is that a player who has never read the manual can still discover
//! "what am I supposed to be doing right now?" without leaving the play view.
//!
//! Counts (how many fire targets? how many melee targets?) are computed from
//! the same engine `can_*` predicates the input handlers gate on -- so the
//! panel and the rings on the map cannot disagree about what's legal.

use bevy_egui::egui;

use omdurman_rules::Phase;
use omdurman_types::HexCoord;

use crate::GameStateResource;
use crate::hotkeys::PickerCommand;
use crate::picker::{MovementPath, PickerState, PlacedUnit, selected_unit_id, selected_unit_ids};
use crate::rulebook::Rulebook;
use crate::ui_phase_state::UiPhaseState;

/// One row of the action list. The paragraph is the rulebook section that
/// authorises the action -- rendered as a deep link via [`Rulebook::title_of`].
struct ActionHint {
    /// Short label, e.g. "Move", "Fire — Direct", "Declare Melee".
    label: String,
    /// Optional sub-line: "3 in-range targets", "4 MP remaining".
    detail: Option<String>,
}

/// What [`collect_hints`] needs beyond the engine state: this fire
/// sub-phase's allocations are resolved (§6.41), how many attacks are
/// staged, and whether the counter tray offers counters to place.
struct HintContext {
    fire_committed: bool,
    staged: usize,
    tray_open: bool,
}

#[allow(clippy::too_many_arguments)]
/// Render the action panel into the right sidebar's "Actions" section.
/// Called by [`crate::overview::unit_overview_ui`] so it shares the sidebar
/// with the Game-control and Unit-list sections. `ui_state` is the mirrored
/// §4 turn machine (see `ui_phase_state`), not a per-call derivation.
pub fn draw_actions_section(
    ui: &mut egui::Ui,
    state: &GameStateResource,
    ui_state: UiPhaseState,
    picker: &PickerState,
    placed_units: &bevy::ecs::system::Query<(bevy::prelude::Entity, &PlacedUnit)>,
    rulebook: &Rulebook,
    clicked_section: &mut Option<String>,
    movement_path: &MovementPath,
    fire_targets: &mut crate::fire::FireTargetCache,
    fire_allocation: Option<&mut crate::fire_allocation::FireAllocationState>,
    local_may_act: bool,
    (tray_open, spectator): (bool, bool),
    commands_out: &mut Vec<PickerCommand>,
) {
    crate::ui::section_header(ui, "Next step");

    // The battle is over: nothing left to do but read the result.
    if matches!(ui_state, UiPhaseState::GameOver) {
        ui.label(
            egui::RichText::new("The battle is over. No further actions.")
                .color(crate::ui::palette::RAIL_DIM)
                .size(12.0),
        );
        return;
    }
    if !local_may_act {
        ui.label(
            egui::RichText::new(if spectator {
                "You are watching: the seated commanders play."
            } else {
                "The other side is acting; watch the board."
            })
            .color(crate::ui::palette::RAIL_DIM)
            .size(13.0),
        );
        return;
    }

    // The allocation tray (whose own button resolves the attacks), offered
    // only once an attack is staged.
    let fire_committed = fire_allocation.as_ref().is_some_and(|a| a.committed);
    let staged = fire_allocation
        .as_ref()
        .filter(|a| !a.committed)
        .map_or(0, |a| a.attacks.len());
    if let Some(allocation) = fire_allocation.filter(|a| !a.committed && !a.attacks.is_empty()) {
        let open = allocation.panel_open;
        let s = if staged == 1 { "" } else { "s" };
        let (label, fill) = if open {
            (
                "Hide staged attacks".to_string(),
                crate::ui::palette::BTN_GO,
            )
        } else {
            (
                format!("Resolve {staged} staged attack{s}\u{2026}"),
                crate::ui::palette::BTN_COMBAT,
            )
        };
        if ui
            .add(
                egui::Button::new(label)
                    .fill(fill)
                    .min_size(egui::Vec2::new(160.0, 26.0)),
            )
            .clicked()
        {
            allocation.panel_open = !open;
        }
        ui.add_space(4.0);
    }

    let hints = collect_hints(
        &state.0,
        state.0.phase,
        picker,
        placed_units,
        fire_targets,
        HintContext {
            fire_committed,
            staged,
            tray_open,
        },
    );
    if hints.is_empty() {
        ui.colored_label(
            crate::ui::palette::RAIL_DIM,
            "Nothing to do here \u{2014} end the phase (E).",
        );
    } else {
        for hint in hints {
            // Wrapped: a long hint ("Allocate defensive fire — Maxim /
            // Howitzer (12 target hex(es))") must not widen the rail past
            // its visible width, which clipped every line below it.
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("• ")
                        .color(crate::ui::palette::RAIL_DIM)
                        .size(13.0),
                );
                ui.label(
                    egui::RichText::new(&hint.label)
                        .color(crate::ui::palette::RAIL_TEXT)
                        .size(13.0),
                );
                if let Some(d) = hint.detail {
                    crate::rulebook::refs_label(
                        ui,
                        &format!("({d})"),
                        crate::ui::palette::RAIL_DIM,
                        12.0,
                    );
                }
            });
        }
    }
    // One rules link for the phase, not one per line.
    let section = if matches!(ui_state, UiPhaseState::Setup) {
        setup_section(state.0.scenario)
    } else {
        ui_state.rulebook_section()
    };
    if !section.is_empty() {
        deep_link(ui, rulebook, section, clicked_section);
    }

    ui.add_space(6.0);

    // Selected-unit profile (factors, weapon, remaining movement) -- a player
    // who has not memorised the counter can still see what they're holding.
    // A combat tile selection (the unified fire/melee selection) lists its
    // members; the first carries the detail.
    let selected_ids = selected_unit_ids(picker, placed_units);
    if let Some(&unit_id) = selected_ids.first()
        && let Some(unit) = state.0.find_unit(unit_id)
    {
        crate::ui::section_header(
            ui,
            if selected_ids.len() > 1 {
                "Selected units"
            } else {
                "Selected unit"
            },
        );
        ui.label(
            egui::RichText::new(unit.profile.identity.label_in(state.0.scenario))
                .color(crate::ui::palette::RAIL_TEXT)
                .size(13.0),
        );
        if selected_ids.len() > 1 {
            // Every member, clickable: narrows the selection to that counter
            // (a unit buried in a stack is otherwise a few pixels of edge).
            ui.colored_label(
                crate::ui::palette::RAIL_DIM,
                "click a unit to select it alone:",
            );
            for &id in &selected_ids {
                let Some(member) = state.0.find_unit(id) else {
                    continue;
                };
                let Some((entity, _)) = placed_units.iter().find(|(_, p)| p.unit_id == Some(id))
                else {
                    continue;
                };
                let mut label = member.profile.identity.label_in(state.0.scenario);
                if state.0.units_fired_this_phase.contains(&id) {
                    label.push_str("  (fired)");
                }
                if member.state.disrupted {
                    label.push_str("  (disrupted)");
                }
                if ui
                    .add(
                        egui::Label::new(
                            egui::RichText::new(format!("  \u{25b8} {label}"))
                                .color(crate::ui::palette::RAIL_TEXT)
                                .size(12.0),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    commands_out.push(PickerCommand::SelectMember(entity));
                }
            }
        }
        ui.colored_label(
            crate::ui::palette::RAIL_DIM,
            unit_stats_line(
                unit.profile.fire.map(|f| f.value()),
                unit.profile.melee.map(|m| m.value()),
                &movement_label(&unit.profile.movement, &state.0, unit_id),
                &weapon_label(unit),
            ),
        );
        // Setup pickup (§9.2/§9.3): a deployed counter goes back to the tray.
        // Same command as the Del key.
        if matches!(state.0.phase, Phase::Setup)
            && local_may_act
            && selected_ids.len() == 1
            && matches!(picker, PickerState::Selected { .. })
            && command_button(ui, PickerCommand::ReturnToTray, "Return to tray", false)
        {
            commands_out.push(PickerCommand::ReturnToTray);
        }
        if unit.state.disrupted {
            ui.colored_label(
                crate::ui::palette::RED,
                "disrupted — cannot fire, melee, or move this turn.",
            );
        }
        if unit.state.constructing_zariba {
            ui.colored_label(
                crate::ui::palette::RAIL_DIM,
                "constructing a zariba hexside.",
            );
        }
        if unit.state.demolishing {
            ui.colored_label(crate::ui::palette::RAIL_DIM, "demolishing this turn.");
        }
        // §5.43: a unit in enemy ZOC may withdraw at the start of its next
        // movement phase (or move directly into another enemy ZOC).
        if state.0.hex_in_enemy_zoc(
            unit.position,
            unit.profile.identity.owner(),
            unit.profile.kind,
        ) {
            crate::rulebook::refs_label(
                ui,
                "in enemy ZOC — may withdraw next Movement phase (§5.43).",
                crate::ui::palette::CAUTION,
                13.0,
            );
        }
        // Advance-after-combat prompt (§6.82, §7.6): show when the unit may
        // advance into an adjacent vacated hex after offensive fire or melee.
        if matches!(state.0.phase, Phase::OffensiveFire(_) | Phase::Melee) {
            let advance_targets: usize = unit
                .position
                .neighbors()
                .into_iter()
                .filter(|h| state.0.can_advance_after_combat(unit_id, *h).is_ok())
                .count();
            if advance_targets > 0 {
                crate::rulebook::refs_label(
                    ui,
                    &format!(
                        "May advance into {advance_targets} vacated hex{} (§6.82).",
                        if advance_targets == 1 { "" } else { "es" }
                    ),
                    crate::ui::palette::HINT_GREEN,
                    13.0,
                );
            }
        }
    }

    // -- Pending movement path summary + confirm button -----------------------
    if !movement_path.legs.is_empty() {
        ui.add_space(6.0);
        crate::ui::section_header(ui, "Movement path");
        let legs = movement_path.legs.len();
        let total = movement_path.cost_so_far;
        ui.label(
            egui::RichText::new(format!(
                "{legs} step{}, {total} MP total",
                if legs == 1 { "" } else { "s" }
            ))
            .color(crate::ui::palette::RAIL_TEXT)
            .size(13.0),
        );
        // The same commands as Enter / Backspace / Esc (see `hotkeys`).
        if local_may_act {
            ui.horizontal_wrapped(|ui| {
                if command_button(ui, PickerCommand::ConfirmMove, "Confirm move", true) {
                    commands_out.push(PickerCommand::ConfirmMove);
                }
                if command_button(ui, PickerCommand::UndoStep, "Undo step", false) {
                    commands_out.push(PickerCommand::UndoStep);
                }
                if command_button(ui, PickerCommand::Cancel, "Cancel", false) {
                    commands_out.push(PickerCommand::Cancel);
                }
            });
        }
        // (The arrows on the board show the route itself.)
    }
}

/// What the player can do right now, in plain words -- short, and only what
/// applies: a phase with nothing to do says so instead of listing moves the
/// rules allow in principle. The on-map rings name the individual targets.
fn collect_hints(
    gs: &omdurman_rules::effects::GameState,
    phase: Phase,
    picker: &PickerState,
    placed_units: &bevy::ecs::system::Query<(bevy::prelude::Entity, &PlacedUnit)>,
    fire_targets: &mut crate::fire::FireTargetCache,
    cx: HintContext,
) -> Vec<ActionHint> {
    let hint = |label: &str, detail: Option<String>| ActionHint {
        label: label.to_string(),
        detail,
    };
    let mut out: Vec<ActionHint> = Vec::new();
    let selected = selected_unit_id(picker, placed_units);
    let campaign = gs.scenario == omdurman_types::Scenario::Campaign;
    let ae_moving = gs.active_player == omdurman_types::Player::AngloEgyptian;
    let optional = |rule| gs.optional_rules.contains(&rule);
    // A counter still in hand (auto-next keeps one after each placement)
    // takes every board click: say so first, with the way out.
    if let PickerState::Placing { .. } = picker {
        out.push(hint(
            "A counter is in hand: click a highlighted hex to place it",
            Some("Esc or right-click puts it back".into()),
        ));
    }
    let own_units = gs
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == gs.phase_player())
        .count();
    match phase {
        Phase::Setup => {
            if cx.tray_open {
                out.push(hint("Pick a counter above, then a highlighted hex", None));
            } else {
                out.push(hint("Nothing (more) to deploy \u{2014} press Ready", None));
            }
            // The Dervish set-up's own tasks, while still to do (§10.11 the
            // mines are secret: never hinted to the other side).
            let dervish_deploying = gs.player_to_act() == Some(omdurman_types::Player::Dervish);
            if dervish_deploying
                && optional(omdurman_rules::OptionalRule::RiverMines)
                && gs.mines.len() < 2
            {
                out.push(hint("Lay the two river mines in the Nile", None));
            }
            if dervish_deploying
                && optional(omdurman_rules::OptionalRule::RiverChain)
                && gs.chain.is_none()
            {
                out.push(hint("String the river chain across the Nile", None));
            }
        }
        Phase::Movement => {
            if cx.tray_open {
                out.push(hint(
                    "Bring on reinforcements: pick a counter above, then a green hex",
                    None,
                ));
            }
            if selected.is_some() {
                out.push(hint(
                    "Click a destination, then Enter",
                    selected_movement_detail(gs, selected),
                ));
            } else if own_units > 0 {
                out.push(hint("Select one of your units to move it", None));
            }
            let ae_infantry_on_map = gs.units.iter().any(|u| {
                matches!(
                    u.profile.identity,
                    omdurman_rules::UnitIdentity::AngloEgyptianInfantry { .. }
                )
            });
            if campaign && ae_moving && ae_infantry_on_map {
                // §5.3: only a battalion that has not moved this turn
                // builds -- a newcomer marches in and finds no button.
                out.push(hint(
                    "Build the Zariba with infantry inside it",
                    Some("select a battalion that began the turn there, before it moves".into()),
                ));
                // §5.21: "after, and only after" the Isa Zachneih is gone.
                if gs.isa_zachneih_eliminated {
                    out.push(hint("Load / disembark Friendlies", None));
                }
            }
        }
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_) => {
            if cx.fire_committed {
                out.push(hint(
                    "Fire resolved",
                    Some(FIRE_ALREADY_RESOLVED_HINT.into()),
                ));
            } else if cx.staged > 0 {
                out.push(hint("Stage more attacks, or resolve the staged ones", None));
            } else if fire_targets.side_target_count(gs) == 0 {
                if fire_targets.side_can_breach(gs) {
                    out.push(hint(
                        "Your artillery can fire at the wall to breach it: select a battery, \
                         then pick a wall in the Artillery Breach card",
                        Some("§6.63".into()),
                    ));
                } else {
                    out.push(hint(
                        "No enemy in range of a unit that may fire \u{2014} end the phase (E)",
                        None,
                    ));
                }
            } else if selected.is_some() {
                out.push(hint(
                    "Click an enemy hex to aim at it",
                    fire_target_count(gs, picker, placed_units, fire_targets),
                ));
            } else {
                let n = fire_targets.side_target_count(gs);
                out.push(hint(
                    "Select a unit (double-click a stack), then an enemy hex",
                    Some(format!(
                        "{n} enemy hex{} in range",
                        if n == 1 { "" } else { "es" }
                    )),
                ));
            }
        }
        Phase::Melee => {
            if let Some(pm) = &gs.pending_melee {
                let retreat = crate::melee::defenders_may_retreat(gs, &pm.attack);
                out.push(hint(
                    "Resolve the pending melee",
                    retreat.then(|| "after the defender's reaction window".into()),
                ));
                if retreat {
                    out.push(hint("Retreat before melee (defender)", None));
                }
            } else {
                out.push(hint(
                    "Double-click your stack, then an adjacent enemy",
                    melee_target_count(gs, picker, placed_units),
                ));
            }
        }
    }
    out
}

fn selected_movement_detail(
    gs: &omdurman_rules::effects::GameState,
    selected: Option<(omdurman_rules::UnitId, HexCoord)>,
) -> Option<String> {
    let (id, _) = selected?;
    let unit = gs.find_unit(id)?;
    // The engine's figures: night halving (§8.1), the sticky upstream cap
    // (§5.24), a stop in an enemy ZOC (§5.43) and the like included.
    let left = gs.remaining_movement(id);
    match unit.profile.movement {
        omdurman_rules::UnitMovement::Land(_) => Some(format!("{left} MP remaining")),
        omdurman_rules::UnitMovement::Gunboat(_) => {
            let spent = gs.mp_spent(id);
            let ga = gs.gunboat_allowances(unit)?;
            let up = (ga.upstream.value() as i16 - spent).max(0).min(left);
            Some(
                if gs.gunboats_upstream_this_turn.contains(&id) || left == 0 {
                    format!("{left} MP remaining")
                } else {
                    format!("{up} up / {left} down MP remaining")
                },
            )
        }
        omdurman_rules::UnitMovement::Immobile => Some("immobile".into()),
    }
}

/// The fire hint once this sub-phase's allocations are resolved: no more
/// allocating (all fire of a sub-phase is allocated, then resolved, §6.41),
/// only ending the phase.
/// What a player who tries to allocate after the resolution is told -- in
/// the rail and as the refusal of such a click.
pub(crate) const FIRE_ALREADY_RESOLVED_HINT: &str =
    "this sub-phase's fire is already resolved -- End phase (E) to go on";

fn fire_target_count(
    gs: &omdurman_rules::effects::GameState,
    picker: &PickerState,
    placed_units: &bevy::ecs::system::Query<(bevy::prelude::Entity, &PlacedUnit)>,
    cache: &mut crate::fire::FireTargetCache,
) -> Option<String> {
    let group = crate::fire::fire_selection(picker, placed_units, gs)?;
    let kinds = crate::fire::fire_group_kinds(gs, &group);
    if kinds.is_empty() {
        return None;
    }
    let count = cache.valid_targets(gs, &kinds).len();
    Some(format!("{count} target hex(es)"))
}

fn melee_target_count(
    gs: &omdurman_rules::effects::GameState,
    picker: &PickerState,
    placed_units: &bevy::ecs::system::Query<(bevy::prelude::Entity, &PlacedUnit)>,
) -> Option<String> {
    // Any melee-capable member of the selection represents the tile: every
    // co-stacked attacker shares the hex, so adjacency and hexside checks are
    // identical across members (§7; see `build_melee_attack`).
    let representative = selected_unit_ids(picker, placed_units)
        .into_iter()
        .find(|&id| {
            gs.find_unit(id)
                .is_some_and(|u| u.profile.kind.may_melee_attack() && !u.state.disrupted)
        })?;
    let unit = gs.find_unit(representative)?;
    let count = unit
        .position
        .neighbors()
        .into_iter()
        .filter(|hex| gs.can_melee(representative, *hex).is_ok())
        .count();
    Some(format!("{count} adjacent target hex(es)"))
}

fn movement_label(
    movement: &omdurman_rules::UnitMovement,
    gs: &omdurman_rules::effects::GameState,
    id: omdurman_rules::UnitId,
) -> String {
    let spent = gs.mp_spent(id);
    let spent = if spent > 0 {
        format!(" ({spent} spent)")
    } else {
        String::new()
    };
    match movement {
        omdurman_rules::UnitMovement::Land(a) => format!("{} MP{spent}", a.value()),
        omdurman_rules::UnitMovement::Gunboat(g) => format!(
            "{} up / {} down MP{spent}",
            g.upstream.value(),
            g.downstream.value(),
        ),
        omdurman_rules::UnitMovement::Immobile => "immobile".into(),
    }
}

/// The weapon a counter fires, plus a named gunboat's second weapon: its
/// Maxims, the "6×2" printed after the artillery factor (§2.32).
fn weapon_label(unit: &omdurman_rules::UnitPlacement) -> String {
    let weapon = unit.profile.weapon.to_string();
    match unit.profile.identity {
        omdurman_rules::UnitIdentity::AngloEgyptianGunboat(gb) => match gb.maxim_factor() {
            Some(maxims) => format!("{weapon} + Maxims {}\u{d7}2", maxims.value()),
            None => weapon,
        },
        _ => weapon,
    }
}

/// The selected unit's factor lines, e.g. "Fire 3 · Melee — · Move 4 MP",
/// then the weapon ("Rifle") on a line of its own: a named gunboat's "Move
/// 12 up / 18 down MP" plus "Artillery + Maxims 6×2" is wider than the rail.
/// A missing factor reads as an em dash rather than `None`.
fn unit_stats_line(fire: Option<u16>, melee: Option<u16>, movement: &str, weapon: &str) -> String {
    let factor = |v: Option<u16>| v.map_or_else(|| "\u{2014}".to_string(), |v| v.to_string());
    format!(
        "Fire {} \u{b7} Melee {} \u{b7} Move {movement}\n{weapon}",
        factor(fire),
        factor(melee),
    )
}

/// A rail button for a [`PickerCommand`], labelled with its key
/// ("Confirm move (Enter)"). Returns whether it was clicked.
fn command_button(ui: &mut egui::Ui, cmd: PickerCommand, label: &str, primary: bool) -> bool {
    let text = format!("{label} ({})", cmd.key_label());
    let mut button = egui::Button::new(egui::RichText::new(text).size(12.0));
    if primary {
        button = button.fill(crate::ui::palette::BTN_GO);
    } else if cmd == PickerCommand::ReturnToTray {
        button = button.fill(crate::ui::palette::BTN_DANGER);
    }
    ui.add(button).clicked()
}

/// Render a `§N` chip annotated with the section title (when known) as a
/// clickable deep link into the Rulebook tab. Mutates `clicked_section` if
/// the user follows the link.
fn deep_link(
    ui: &mut egui::Ui,
    rulebook: &Rulebook,
    paragraph: &str,
    clicked_section: &mut Option<String>,
) {
    let title = rulebook.title_of(paragraph);
    let label = if let Some(t) = title {
        format!("§{paragraph} {t}")
    } else {
        format!("§{paragraph}")
    };
    if ui
        .add(
            egui::Label::new(
                egui::RichText::new(label)
                    .color(crate::ui::palette::RAIL_DIM)
                    .size(11.0)
                    .underline(),
            )
            .sense(egui::Sense::click()),
        )
        .clicked()
    {
        *clicked_section = Some(paragraph.to_string());
    }
}

/// The set-up rules of `scenario` (§9.11 Campaign, §9.21 Historical, §9.32
/// FALL OF KHARTOUM).
fn setup_section(scenario: omdurman_types::Scenario) -> &'static str {
    match scenario {
        omdurman_types::Scenario::Campaign => "9.11",
        omdurman_types::Scenario::Historical => "9.21",
        omdurman_types::Scenario::FallOfKhartoum => "9.32",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_line_has_no_debug_formatting() {
        let line = unit_stats_line(Some(3), None, "4 MP", "Rifle");
        assert_eq!(line, "Fire 3 \u{b7} Melee \u{2014} \u{b7} Move 4 MP\nRifle");
        assert!(!line.contains("Some") && !line.contains("None") && !line.contains('"'));
    }
}

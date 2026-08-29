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
use crate::picker::{MovementPath, PickerState, PlacedUnit, selected_unit_id};
use crate::rulebook::Rulebook;
use crate::ui_phase_state::UiPhaseState;

/// One row of the action list. The paragraph is the rulebook section that
/// authorises the action -- rendered as a deep link via [`Rulebook::title_of`].
struct ActionHint {
    /// Short label, e.g. "Move", "Fire — Direct", "Declare Melee".
    label: String,
    /// Optional sub-line: "3 in-range targets", "4 MP remaining".
    detail: Option<String>,
    /// Rulebook citation, e.g. "5" (movement) or "6.41" (direct fire).
    paragraph: String,
}

/// Render the action panel into the right sidebar's "Actions" section.
/// Called by [`crate::overview::unit_overview_ui`] so it shares the sidebar
/// with the Game-control and Unit-list sections.
pub fn draw_actions_section(
    ui: &mut egui::Ui,
    state: &GameStateResource,
    picker: &PickerState,
    placed_units: &bevy::ecs::system::Query<(bevy::prelude::Entity, &PlacedUnit)>,
    rulebook: &Rulebook,
    clicked_section: &mut Option<String>,
    movement_path: &MovementPath,
) {
    crate::ui::section_header(ui, "Actions");
    let ui_state = UiPhaseState::derive(&state.0);

    // Phase title from UiPhaseState (canonical label).
    let phase_label = ui_state.phase_label();
    ui.label(
        egui::RichText::new(phase_label)
            .color(crate::ui::palette::INK)
            .size(14.0)
            .strong(),
    );
    ui.add_space(4.0);

    // Rulebook deep-link for the active phase.
    let section = ui_state.rulebook_section();
    if !section.is_empty() {
        deep_link(ui, rulebook, section, clicked_section);
        ui.add_space(4.0);
    }

    // Firing-player indicator line.
    if let Some(firer) = ui_state.firing_player() {
        let firer_str = match firer {
            omdurman_types::Player::AngloEgyptian => "Anglo-Egyptian",
            omdurman_types::Player::Dervish => "Dervish",
        };
        ui.colored_label(
            crate::ui::palette::RED,
            format!("\u{1f525} {firer_str} fires"),
        );
        ui.add_space(4.0);
    }

    let hints = collect_hints(&state.0, state.0.phase, picker, placed_units);
    if hints.is_empty() {
        ui.colored_label(
            crate::ui::palette::FAINT_INK,
            "no actions available — end the phase when ready.",
        );
    } else {
        for hint in hints {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("• ")
                        .color(crate::ui::palette::FAINT_INK)
                        .size(13.0),
                );
                ui.label(
                    egui::RichText::new(&hint.label)
                        .color(crate::ui::palette::INK)
                        .size(13.0),
                );
                if let Some(d) = hint.detail {
                    ui.label(
                        egui::RichText::new(format!("({d})"))
                            .color(crate::ui::palette::FAINT_INK)
                            .size(12.0),
                    );
                }
            });
            deep_link(ui, rulebook, &hint.paragraph, clicked_section);
        }
    }

    ui.add_space(6.0);

    // Selected-unit profile (factors, weapon, remaining movement) -- a player
    // who has not memorised the counter can still see what they're holding.
    if let Some((unit_id, _)) = selected_unit_id(picker, placed_units)
        && let Some(unit) = state.0.find_unit(unit_id)
    {
        crate::ui::section_header(ui, "Selected unit");
        ui.label(
            egui::RichText::new(unit.profile.identity.short_label())
                .color(crate::ui::palette::INK)
                .size(13.0),
        );
        ui.colored_label(
            crate::ui::palette::FAINT_INK,
            format!(
                "fire {:?}  melee {:?}  move {:?}  weapon {}",
                unit.profile.fire.map(|f| f.value()),
                unit.profile.melee.map(|m| m.value()),
                movement_label(&unit.profile.movement, &state.0, unit_id),
                unit.profile.weapon,
            ),
        );
        if unit.state.disrupted {
            ui.colored_label(
                egui::Color32::from_rgb(180, 90, 90),
                "disrupted — cannot fire, melee, or move this turn.",
            );
        }
        if unit.state.constructing_zariba {
            ui.colored_label(
                crate::ui::palette::FAINT_INK,
                "constructing a zariba hexside.",
            );
        }
        if unit.state.demolishing {
            ui.colored_label(crate::ui::palette::FAINT_INK, "demolishing this turn.");
        }
        // §5.43: a unit in enemy ZOC may withdraw at the start of its next
        // movement phase (or move directly into another enemy ZOC).
        if state.0.hex_in_enemy_zoc(
            unit.position,
            unit.profile.identity.owner(),
            unit.profile.kind,
        ) {
            ui.colored_label(
                egui::Color32::from_rgb(0x8B, 0x7A, 0x40),
                "in enemy ZOC — may withdraw next Movement phase (§5.43).",
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
                ui.colored_label(
                    egui::Color32::from_rgb(0x80, 0xC0, 0x80),
                    format!(
                        "May advance into {advance_targets} vacated hex{} (§6.82).",
                        if advance_targets == 1 { "" } else { "es" }
                    ),
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
            .color(crate::ui::palette::INK)
            .size(13.0),
        );
        ui.label(
            egui::RichText::new("Press Enter to confirm, Right-click to cancel.")
                .color(crate::ui::palette::FAINT_INK)
                .size(12.0),
        );
        // Per-leg breakdown with gunboat direction annotations (§5.24).
        let is_gunboat = selected_unit_id(picker, placed_units)
            .and_then(|(uid, _)| state.0.find_unit(uid))
            .is_some_and(|u| {
                matches!(u.profile.movement, omdurman_rules::UnitMovement::Gunboat(_))
            });
        for (i, &(from, to)) in movement_path.legs.iter().enumerate() {
            let dir = if is_gunboat {
                state
                    .0
                    .board
                    .step_direction(from, to)
                    .map(|d| match d {
                        omdurman_rules::board::StepDirection::Upstream => " \u{2191}", // ↑
                        omdurman_rules::board::StepDirection::Downstream => " \u{2193}", // ↓
                    })
                    .unwrap_or("")
            } else {
                ""
            };
            ui.label(
                egui::RichText::new(format!(
                    "  {}({}, {}) -> ({}, {}){}",
                    i + 1,
                    from.q,
                    from.r,
                    to.q,
                    to.r,
                    dir,
                ))
                .color(crate::ui::palette::FAINT_INK)
                .size(11.0),
            );
        }
    }
}

/// Enumerate the actions the rulebook permits in `phase`, with as much
/// context (counts, selected-unit relevance) as can be cheaply derived. The
/// list is intentionally short -- it points the player at *categories* of
/// action (move, fire, melee, end-phase) rather than enumerating every legal
/// target hex (the on-map rings do that).
fn collect_hints(
    gs: &omdurman_rules::effects::GameState,
    phase: Phase,
    picker: &PickerState,
    placed_units: &bevy::ecs::system::Query<(bevy::prelude::Entity, &PlacedUnit)>,
) -> Vec<ActionHint> {
    let mut out: Vec<ActionHint> = Vec::new();
    let selected = selected_unit_id(picker, placed_units);

    match phase {
        Phase::Setup => {
            out.push(ActionHint {
                label: "Deploy units".into(),
                detail: Some("pick from the sidebar, click a hex in your zone".into()),
                paragraph: "9.2".into(),
            });
            out.push(ActionHint {
                label: "Lay river mines (Dervish)".into(),
                detail: None,
                paragraph: "10.11".into(),
            });
            out.push(ActionHint {
                label: "Sink the river chain (Dervish)".into(),
                detail: None,
                paragraph: "10.21".into(),
            });
            out.push(ActionHint {
                label: "Place zariba (Dervish)".into(),
                detail: None,
                paragraph: "9.231".into(),
            });
        }
        Phase::Movement => {
            out.push(ActionHint {
                label: "Move selected unit".into(),
                detail: selected_movement_detail(gs, selected),
                paragraph: "5.12".into(),
            });
            out.push(ActionHint {
                label: "Construct zariba (engineers / adjacent)".into(),
                detail: None,
                paragraph: "9.231".into(),
            });
            out.push(ActionHint {
                label: "Load / disembark Friendlies".into(),
                detail: None,
                paragraph: "5.21".into(),
            });
            out.push(ActionHint {
                label: "End phase".into(),
                detail: None,
                paragraph: "4".into(),
            });
        }
        Phase::OffensiveFire(sub) => {
            let kind_word = match sub {
                omdurman_rules::FireSubPhase::DirectFire => "Direct",
                omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => "Maxim / Howitzer",
            };
            out.push(ActionHint {
                label: format!("Allocate fire — {kind_word}"),
                detail: fire_target_count(gs, selected),
                paragraph: match sub {
                    omdurman_rules::FireSubPhase::DirectFire => "6.41".into(),
                    omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => "6.42".into(),
                },
            });
            out.push(ActionHint {
                label: "Review & execute allocations".into(),
                detail: Some("click 'Execute All' in the allocation panel".into()),
                paragraph: "6.41".into(),
            });
            out.push(ActionHint {
                label: "Advance after fire".into(),
                detail: Some("into vacated enemy hex (§6.82)".into()),
                paragraph: "6.82".into(),
            });
            out.push(ActionHint {
                label: "End phase".into(),
                detail: None,
                paragraph: "4".into(),
            });
        }
        Phase::DefensiveFire(sub) => {
            let kind_word = match sub {
                omdurman_rules::FireSubPhase::DirectFire => "Direct",
                omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => "Maxim / Howitzer",
            };
            out.push(ActionHint {
                label: format!("Allocate defensive fire — {kind_word}"),
                detail: fire_target_count(gs, selected),
                paragraph: "6.41".into(),
            });
            out.push(ActionHint {
                label: "Review & execute allocations".into(),
                detail: Some("click 'Execute All' in the allocation panel".into()),
                paragraph: "6.41".into(),
            });
            out.push(ActionHint {
                label: "End phase".into(),
                detail: None,
                paragraph: "4".into(),
            });
        }
        Phase::Melee => {
            if gs.pending_melee.is_some() {
                out.push(ActionHint {
                    label: "Resolve the pending melee".into(),
                    detail: Some("after the defender's reaction window".into()),
                    paragraph: "7.5".into(),
                });
                out.push(ActionHint {
                    label: "Retreat before melee (defender)".into(),
                    detail: None,
                    paragraph: "7.5".into(),
                });
            } else {
                out.push(ActionHint {
                    label: "Declare melee".into(),
                    detail: melee_target_count(gs, selected),
                    paragraph: "7.1".into(),
                });
            }
            out.push(ActionHint {
                label: "Advance after combat".into(),
                detail: None,
                paragraph: "7.6".into(),
            });
            out.push(ActionHint {
                label: "End phase".into(),
                detail: None,
                paragraph: "4".into(),
            });
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
    let remaining = gs.mp_spent(id);
    match unit.profile.movement {
        omdurman_rules::UnitMovement::Land(a) => {
            let left = (a.value() as i16 - remaining).max(0);
            Some(format!("{left} MP remaining"))
        }
        omdurman_rules::UnitMovement::Gunboat(g) => {
            let up = (g.upstream.value() as i16 - remaining).max(0);
            let down = (g.downstream.value() as i16 - remaining).max(0);
            Some(format!("{up} up / {down} down MP remaining"))
        }
        omdurman_rules::UnitMovement::Immobile => Some("immobile".into()),
    }
}

/// Count enemy-occupied hexes the selected unit may legally fire at. Used as
/// the "(N targets)" hint next to the Fire action. Walks the engine's
/// `can_fire_at` for each candidate -- the same predicate the input handler
/// uses, so the count matches the on-map rings.
fn fire_target_count(
    gs: &omdurman_rules::effects::GameState,
    selected: Option<(omdurman_rules::UnitId, HexCoord)>,
) -> Option<String> {
    let (id, _) = selected?;
    let unit = gs.find_unit(id)?;
    let enemy = unit.profile.identity.owner().opponent();
    // Named gunboats (§6.64) carry howitzers despite their profile weapon
    // being Artillery; check the identity, not just the profile weapon.
    let is_named_gunboat = matches!(
        unit.profile.identity,
        omdurman_rules::UnitIdentity::AngloEgyptianGunboat(gb) if gb.has_howitzer()
    );
    let kind = match gs.phase {
        Phase::OffensiveFire(s) | Phase::DefensiveFire(s) => match (s, unit.profile.weapon) {
            (omdurman_rules::FireSubPhase::DirectFire, _) => Some(omdurman_rules::FireKind::Direct),
            (
                omdurman_rules::FireSubPhase::MaximSecondAndHowitzer,
                omdurman_rules::WeaponClass::Maxims,
            ) => Some(omdurman_rules::FireKind::MaximSecondFire),
            (
                omdurman_rules::FireSubPhase::MaximSecondAndHowitzer,
                omdurman_rules::WeaponClass::Howitzer,
            ) => Some(omdurman_rules::FireKind::Howitzer),
            (omdurman_rules::FireSubPhase::MaximSecondAndHowitzer, _) if is_named_gunboat => {
                Some(omdurman_rules::FireKind::Howitzer)
            }
            _ => None,
        },
        _ => None,
    };
    let Some(kind) = kind else {
        return Some("0 targets — wrong sub-phase for this weapon".into());
    };
    let mut targets: Vec<HexCoord> = gs
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == enemy)
        .map(|u| u.position)
        .filter(|hex| gs.can_fire_at(id, *hex, kind).is_ok())
        .collect();
    targets.sort_by_key(|h| (h.q, h.r));
    targets.dedup();
    Some(format!("{} target hex(es)", targets.len()))
}

fn melee_target_count(
    gs: &omdurman_rules::effects::GameState,
    selected: Option<(omdurman_rules::UnitId, HexCoord)>,
) -> Option<String> {
    let (id, _) = selected?;
    let unit = gs.find_unit(id)?;
    if !unit.profile.kind.may_melee_attack() || unit.state.disrupted {
        return Some("0 melee targets — unit cannot melee".into());
    }
    let count = unit
        .position
        .neighbors()
        .into_iter()
        .filter(|hex| gs.can_melee(id, *hex).is_ok())
        .count();
    Some(format!("{count} adjacent target hex(es)"))
}

fn movement_label(
    movement: &omdurman_rules::UnitMovement,
    gs: &omdurman_rules::effects::GameState,
    id: omdurman_rules::UnitId,
) -> String {
    let spent = gs.mp_spent(id);
    match movement {
        omdurman_rules::UnitMovement::Land(a) => {
            format!("{} ({} spent)", a.value(), spent)
        }
        omdurman_rules::UnitMovement::Gunboat(g) => {
            format!(
                "up {} down {} ({} spent)",
                g.upstream.value(),
                g.downstream.value(),
                spent
            )
        }
        omdurman_rules::UnitMovement::Immobile => "immobile".into(),
    }
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
                    .color(crate::ui::palette::FAINT_INK)
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

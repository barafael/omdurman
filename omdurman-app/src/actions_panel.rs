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

use omdurman_rules::{Phase, UnitKind};
use omdurman_types::HexCoord;

use crate::GameStateResource;
use crate::picker::{PickerState, PlacedUnit, selected_unit_id};
use crate::rulebook::Rulebook;
use crate::theme;

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
) {
    section_header(ui, "Actions");
    let phase = state.0.phase;
    let active_player = state.0.active_player;

    let (phase_title, phase_para) = phase_label(phase);
    let title_line = format!("{phase_title} — {active_player}");
    ui.label(
        egui::RichText::new(title_line)
            .color(theme::INK)
            .size(14.0)
            .strong(),
    );
    deep_link(ui, rulebook, phase_para, clicked_section);
    ui.add_space(4.0);

    let hints = collect_hints(&state.0, phase, picker, placed_units);
    if hints.is_empty() {
        ui.colored_label(
            theme::INK_FAINT,
            "no actions available — end the phase when ready.",
        );
    } else {
        for hint in hints {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("• ").color(theme::INK_FAINT).size(13.0));
                ui.label(
                    egui::RichText::new(&hint.label)
                        .color(theme::INK)
                        .size(13.0),
                );
                if let Some(d) = hint.detail {
                    ui.label(
                        egui::RichText::new(format!("({d})"))
                            .color(theme::INK_FAINT)
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
        section_header(ui, "Selected unit");
        ui.label(
            egui::RichText::new(identity_short(&unit.profile.identity))
                .color(theme::INK)
                .size(13.0),
        );
        ui.colored_label(
            theme::INK_FAINT,
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
            ui.colored_label(theme::INK_FAINT, "constructing a zariba hexside.");
        }
        if unit.state.demolishing {
            ui.colored_label(theme::INK_FAINT, "demolishing this turn.");
        }
    }
}

fn phase_label(phase: Phase) -> (&'static str, &'static str) {
    match phase {
        Phase::Setup => ("Set-up", "9.2"),
        Phase::Movement => ("Movement", "5"),
        Phase::OffensiveFire(sub) => match sub {
            omdurman_rules::FireSubPhase::DirectFire => ("Offensive Fire — Direct", "6.41"),
            omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => {
                ("Offensive Fire — Maxim / Howitzer", "6.42")
            }
        },
        Phase::DefensiveFire(sub) => match sub {
            omdurman_rules::FireSubPhase::DirectFire => ("Defensive Fire — Direct", "6.41"),
            omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => {
                ("Defensive Fire — Maxim / Howitzer", "6.42")
            }
        },
        Phase::Melee => ("Melee", "7"),
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
                label: format!("Fire — {kind_word}"),
                detail: fire_target_count(gs, selected),
                paragraph: match sub {
                    omdurman_rules::FireSubPhase::DirectFire => "6.41".into(),
                    omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => "6.42".into(),
                },
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
                label: format!("Defensive fire — {kind_word}"),
                detail: fire_target_count(gs, selected),
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

fn identity_short(identity: &omdurman_rules::UnitIdentity) -> String {
    use omdurman_rules::UnitIdentity;
    match identity {
        UnitIdentity::DervishTribal { tribe } => tribe.to_string(),
        UnitIdentity::DervishLeader(leader) => leader.to_string(),
        UnitIdentity::DervishArtillery => "Dervish Artillery".into(),
        UnitIdentity::DervishFort => "Dervish Fort".into(),
        UnitIdentity::DervishGunboat(g) => format!("Dervish Gunboat {g}"),
        UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
            let nat = match brigade.nationality {
                omdurman_rules::BrigadeNationality::British => 'B',
                omdurman_rules::BrigadeNationality::Egyptian => 'E',
                omdurman_rules::BrigadeNationality::Sudanese => 'S',
                omdurman_rules::BrigadeNationality::Friendlies => 'F',
            };
            format!("{}{} {battalion} Btn", brigade.number, nat)
        }
        UnitIdentity::AngloEgyptianCavalry => "Cavalry".into(),
        UnitIdentity::AngloEgyptianCamelCorps => "Camel Corps".into(),
        UnitIdentity::AngloEgyptianArtillery => "Artillery".into(),
        UnitIdentity::AngloEgyptianMaxim => "Maxim".into(),
        UnitIdentity::AngloEgyptianGunboat(g) => format!("Gunboat {g}"),
        UnitIdentity::AngloEgyptianLeader(leader) => leader.to_string(),
        UnitIdentity::RoyalEngineers => "Royal Engineers".into(),
    }
}

/// A bold section title followed by a separator. Mirrors the helper in
/// `overview.rs` so the two read as one panel.
fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).size(16.0).color(theme::INK));
    ui.separator();
    ui.add_space(4.0);
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
                    .color(theme::INK_FAINT)
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

// A marker so we can build a stable Id from a unit's faction for grouping.
fn _faction_marker(_: omdurman_rules::Player) -> UnitKind {
    UnitKind::Infantry
}

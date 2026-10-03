//! The command rail -- the game view's one left panel: phase control, the
//! next step, the counter tray, the score and the forces on the map.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_rules::UnitIdentity;

use crate::GameStateResource;
use crate::peers::Peers;
use crate::picker::{MovementPath, PickerState, PlacedUnit, UnitPicker};
use crate::rulebook::Rulebook;

/// The picker resources the rail reads and -- through the counter tray --
/// writes: the hand (`PickerState`) and the tray itself.
#[derive(bevy::ecs::system::SystemParam)]
pub struct RailPicker<'w, 's> {
    pub picker_state: ResMut<'w, PickerState>,
    pub tray: ResMut<'w, UnitPicker>,
    pub movement_path: Res<'w, MovementPath>,
    pub game_state: Option<Res<'w, GameStateResource>>,
    pub placed_units: Query<'w, 's, (Entity, &'static PlacedUnit)>,
}

#[allow(clippy::too_many_arguments)]
/// The command rail: the one left panel of the game view. Top to bottom:
/// the phase's control (End phase / set-up Ready), **Next step** (what to
/// do now, the selected unit, the plotted move), the **counter tray** when
/// counters may be placed (set-up, reinforcements), the **score**, and the
/// collapsed **forces** list. The turn, who acts and the map overlays live
/// in the top bar.
pub fn unit_overview_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    app_state: Res<State<crate::AppState>>,
    rail: RailPicker,
    phase_machine: Res<State<crate::ui_phase_state::UiPhaseState>>,
    mut rulebook: ResMut<Rulebook>,
    peers: Peers,
    mut fire_targets: ResMut<crate::fire::FireTargetCache>,
    mut allocation: Option<ResMut<crate::fire_allocation::FireAllocationState>>,
    mut pending: Option<ResMut<crate::PendingEdits>>,
    mut local_setup_ready: Option<ResMut<crate::peers::LocalSetupReady>>,
    mut layout: ResMut<crate::ScreenLayout>,
    (mut victory, mut unit_list): (
        ResMut<crate::ui_plugin::VictoryModalState>,
        Local<Option<UnitListCache>>,
    ),
    mut picker_commands: bevy::ecs::message::MessageWriter<crate::hotkeys::PickerCommand>,
) {
    let RailPicker {
        mut picker_state,
        mut tray,
        movement_path,
        game_state,
        placed_units,
    } = rail;
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_play() {
        return;
    }
    // The grouped forces list is rebuilt only when the engine state moved or
    // counters came or went -- not every frame (a sort plus a name lookup
    // per counter).
    let counters = placed_units.iter().count();
    let stale = unit_list.as_ref().is_none_or(|cache| {
        cache.counters != counters || game_state.as_ref().is_some_and(|gs| gs.is_changed())
    });
    if stale {
        let mut units: Vec<_> = placed_units.iter().map(|(_, p)| p).collect();
        units.sort_by_key(|u| (u.section_name.display_name(), u.col, u.row));
        let mut groups: std::collections::BTreeMap<(u8, String), (usize, usize)> =
            std::collections::BTreeMap::new();
        for placed in &units {
            let side = match omdurman_rules::unit_profiles::section_owner(placed.section_name) {
                Some(omdurman_types::Player::AngloEgyptian) => 0,
                Some(omdurman_types::Player::Dervish) => 1,
                None => 2,
            };
            let label = placed_unit_identity(placed, game_state.as_deref());
            let entry = groups.entry((side, label)).or_default();
            entry.0 += 1;
            entry.1 += usize::from(placed.disrupted);
        }
        *unit_list = Some(UnitListCache { counters, groups });
    }
    let unit_list = unit_list.as_ref().expect("filled above");
    let local = peers.local();

    crate::layout::left_rail_panel(
        ctx,
        &mut layout,
        "overview_panel",
        "unit_overview_panel",
        236.0,
        |ui| {
            egui::Panel::left("unit_overview_panel")
                .resizable(true)
                .show_separator_line(false)
                .default_size(230.0)
                .size_range(180.0..=360.0)
                .frame(crate::ui::frames::rail())
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));
                    egui::ScrollArea::vertical()
                        .id_salt("command_rail_scroll")
                        .show(ui, |ui| {
                            if *app_state.get() == crate::AppState::InGame
                                && let Some(state) = game_state.as_deref()
                            {
                                crate::ui_plugin::game_control_section(
                                    ui,
                                    state,
                                    &peers,
                                    pending.as_deref_mut(),
                                    local_setup_ready.as_deref_mut(),
                                    crate::ui_plugin::GameControlExtras {
                                        allocation: allocation.as_deref_mut(),
                                        victory: Some(&mut victory),
                                    },
                                );
                                ui.add_space(8.0);

                                let local_may_act = peers.may_act_now(&state.0);
                                let tray_open =
                                    local_may_act && tray.available.iter().any(|u| u.shown());
                                let mut clicked_section: Option<String> = None;
                                let mut commands_out = Vec::new();
                                crate::actions_panel::draw_actions_section(
                                    ui,
                                    state,
                                    *phase_machine.get(),
                                    &picker_state,
                                    &placed_units,
                                    &rulebook,
                                    &mut clicked_section,
                                    &movement_path,
                                    &mut fire_targets,
                                    allocation.as_deref_mut(),
                                    local_may_act,
                                    tray_open,
                                    &mut commands_out,
                                );
                                for cmd in commands_out {
                                    crate::ui_trace::button(cmd.key_label());
                                    picker_commands.write(cmd);
                                }
                                if let Some(sec) = clicked_section {
                                    crate::rulebook::request_section(&mut rulebook, &sec);
                                }

                                // -- Counter tray: set-up force / this turn's arrivals --
                                if tray_open {
                                    ui.add_space(8.0);
                                    let in_setup =
                                        matches!(state.0.phase, omdurman_rules::Phase::Setup);
                                    crate::ui::section_header(
                                        ui,
                                        if in_setup {
                                            "Your forces to deploy"
                                        } else {
                                            "Reinforcements"
                                        },
                                    );
                                    if let Some(line) =
                                        crate::reinforce::arrival_allowance(&state.0)
                                    {
                                        ui.label(
                                            egui::RichText::new(line)
                                                .size(12.0)
                                                .color(crate::ui::palette::RAIL_DIM),
                                        );
                                    }
                                    let stamp = crate::ui_trace::Stamp::of(Some(state));
                                    // The tray's flags are rewritten every frame by
                                    // `unit_picker_ui` around change detection; only a pick
                                    // marks the hand changed.
                                    let before = picker_state.clone();
                                    let mut hand = before.clone();
                                    crate::picker::draw_tray(
                                        ui,
                                        tray.bypass_change_detection(),
                                        &mut hand,
                                        &rulebook,
                                        &stamp,
                                    );
                                    if !same_hand(&before, &hand) {
                                        *picker_state = hand;
                                    }
                                }

                                ui.add_space(10.0);
                                crate::ui_plugin::score_section(ui, state);
                            }

                            // -- Forces on the map, collapsed by default --
                            ui.add_space(6.0);
                            egui::CollapsingHeader::new(
                                egui::RichText::new(format!(
                                    "Forces on the map ({})",
                                    unit_list.counters
                                ))
                                .size(13.0)
                                .color(crate::ui::palette::HEADING),
                            )
                            .id_salt("forces_list")
                            .default_open(false)
                            .show(ui, |ui| {
                                if unit_list.counters == 0 {
                                    ui.colored_label(
                                        crate::ui::palette::TEXT_DIM,
                                        "no units on the map",
                                    );
                                    return;
                                }
                                let mut last_side = None;
                                for ((side, label), (count, disrupted)) in &unit_list.groups {
                                    if last_side != Some(*side) {
                                        last_side = Some(*side);
                                        let player = match side {
                                            0 => Some(omdurman_types::Player::AngloEgyptian),
                                            1 => Some(omdurman_types::Player::Dervish),
                                            _ => None,
                                        };
                                        if let Some(player) = player {
                                            let yours = if local == Some(player) {
                                                " (yours)"
                                            } else {
                                                ""
                                            };
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{}{yours}",
                                                    crate::ui::faction_name(player)
                                                ))
                                                .size(13.0)
                                                .color(crate::ui::faction_color(player)),
                                            );
                                        }
                                    }
                                    let mut line = format!("{label} \u{00d7}{count}");
                                    if *disrupted > 0 {
                                        line.push_str(&format!(" ({disrupted} disrupted)"));
                                    }
                                    ui.label(
                                        egui::RichText::new(line)
                                            .size(12.0)
                                            .color(crate::ui::palette::TEXT_STRONG),
                                    );
                                }
                            });
                        });
                })
                .response
                .rect
        },
    );
}

/// Whether two hands hold the same thing (a tray pick toggles or swaps the
/// counter in hand; nothing else changes it here).
fn same_hand(a: &PickerState, b: &PickerState) -> bool {
    match (a, b) {
        (
            PickerState::Placing {
                unit_idx: x,
                drag_drop: dx,
                ..
            },
            PickerState::Placing {
                unit_idx: y,
                drag_drop: dy,
                ..
            },
        ) => x == y && dx == dy,
        (PickerState::Placing { .. }, _) | (_, PickerState::Placing { .. }) => false,
        _ => true,
    }
}

/// The unit list's lines ("Mulazmin" -> 32 counters, 3 disrupted), cached
/// across frames by [`unit_overview_ui`].
#[derive(Default)]
pub struct UnitListCache {
    /// How many counters the list was built from.
    counters: usize,
    /// (side: A-E 0 / Dervish 1 / other 2, label) -> (counters, disrupted).
    groups: std::collections::BTreeMap<(u8, String), (usize, usize)>,
}

fn placed_unit_identity(placed: &PlacedUnit, game_state: Option<&GameStateResource>) -> String {
    let Some(gs) = game_state else {
        return format!(
            "{} ({}x{})",
            placed.section_name.display_name(),
            placed.col,
            placed.row
        );
    };
    let Some(uid) = placed.unit_id else {
        return format!(
            "{} ({}x{})",
            placed.section_name.display_name(),
            placed.col,
            placed.row
        );
    };
    let Some(unit) = gs.0.find_unit(uid) else {
        return format!(
            "{} ({}x{})",
            placed.section_name.display_name(),
            placed.col,
            placed.row
        );
    };
    identity_description(&unit.profile.identity)
}

fn identity_description(identity: &UnitIdentity) -> String {
    match identity {
        // The unit list groups battalions under their brigade.
        UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
            format!("{brigade} * {battalion} Btn")
        }
        other => other.short_label(),
    }
}

//! Unit overview panel -- lists all placed units with their identity,
//! position, and state. Shown in both map modes.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_rules::UnitIdentity;

use crate::GameStateResource;
use crate::peers::Peers;
use crate::picker::{PickerReadState, PlacedUnit};
use crate::rulebook::Rulebook;

#[allow(clippy::too_many_arguments)]
/// Left sidebar shown in both map modes. Three stacked sections:
/// **Overlays** (ZOC/LOS map-overlay toggles) at the top, **Game control**
/// (turn/phase info + End Phase + scenario set-up, only while a game is
/// live) below it, then **Unit list** (every placed unit's identity,
/// position, and state) at the bottom.
pub fn unit_overview_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    app_state: Res<State<crate::AppState>>,
    mut zoc: ResMut<crate::zoc::ZocOverlay>,
    mut los: ResMut<crate::los::LosOverlay>,
    picker: PickerReadState,
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
    let PickerReadState {
        picker_state,
        movement_path,
        placed_units,
        game_state,
        ..
    } = picker;
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_play() {
        return;
    }
    // The grouped unit list is rebuilt only when the engine state moved or
    // counters came or went -- not every frame (a sort plus a name lookup
    // per counter).
    let counters = placed_units.iter().count();
    let stale = unit_list.as_ref().is_none_or(|cache| {
        cache.counters != counters || game_state.as_ref().is_some_and(|gs| gs.is_changed())
    });
    if stale {
        let mut units: Vec<_> = placed_units.iter().map(|(_, p)| p).collect();
        units.sort_by_key(|u| (u.section_name.display_name(), u.col, u.row));
        let mut groups: std::collections::BTreeMap<String, (usize, usize)> =
            std::collections::BTreeMap::new();
        for placed in &units {
            let label = placed_unit_identity(placed, game_state.as_deref());
            let entry = groups.entry(label).or_default();
            entry.0 += 1;
            entry.1 += usize::from(placed.disrupted);
        }
        *unit_list = Some(UnitListCache { counters, groups });
    }
    let unit_list = unit_list.as_ref().expect("filled above");

    crate::layout::left_rail_panel(
        ctx,
        &mut layout,
        "overview_panel",
        "unit_overview_panel",
        216.0,
        |ui| {
            egui::Panel::left("unit_overview_panel")
                .resizable(true)
                .show_separator_line(false)
                .default_size(200.0)
                .size_range(140.0..=320.0)
                .frame(
                    crate::ui::frames::rail(),
                )
                .show(ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));

            // -- Map overlays (ZOC / LOS), live game and spectator alike --
            crate::ui::section_header(ui, "Overlays");
            ui.horizontal(|ui| {
                let toggle = |ui: &mut egui::Ui, label: &str, active: bool, hover: &str| {
                    let btn = egui::Button::new(
                        egui::RichText::new(label)
                            .size(12.0)
                            .monospace()
                            .color(if active {
                                crate::ui::palette::HIGHLIGHT
                            } else {
                                crate::ui::palette::BRASS_DIM
                            }),
                    )
                    .fill(if active {
                        crate::ui::palette::HIGHLIGHT_BG
                    } else {
                        crate::ui::palette::CHIP_BG
                    });
                    // A tooltip with a link stays open under the pointer, so
                    // its § citation can be followed.
                    ui.add(btn)
                        .on_hover_ui(|ui| {
                            crate::rulebook::refs_label(
                                ui,
                                hover,
                                ui.visuals().text_color(),
                                12.0,
                            );
                        })
                        .clicked()
                };
                if toggle(
                    ui,
                    "ZOC",
                    zoc.visible,
                    "Toggle enemy ZOC ring overlay (§5.41)",
                ) {
                    zoc.visible = !zoc.visible;
                    crate::ui_trace::button("overlay: ZOC");
                }
                if toggle(
                    ui,
                    "LOS",
                    los.visible,
                    "Toggle line-of-sight overlay (§6.3): hover a hex -- green rings are clear, red blocked",
                ) {
                    los.visible = !los.visible;
                    crate::ui_trace::button("overlay: LOS");
                }
            });
            ui.add_space(10.0);

            // -- Game control (only while a live game is active) --
            if *app_state.get() == crate::AppState::InGame
                && let Some(state) = game_state.as_deref()
            {
                                crate::ui::section_header(ui, "Game control");
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
                ui.add_space(10.0);

                // -- Action discovery --
                // What you can do in the current phase, with selected-unit
                // context and § deep-links into the Rulebook tab.
                let mut clicked_section: Option<String> = None;
                let mut commands_out = Vec::new();
                let local_may_act = peers.may_act_now(&state.0);
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
                    &mut commands_out,
                );
                for cmd in commands_out {
                    crate::ui_trace::button(cmd.key_label());
                    picker_commands.write(cmd);
                }
                if let Some(sec) = clicked_section {
                    crate::rulebook::request_section(&mut rulebook, &sec);
                }
                ui.add_space(10.0);
            }

            // -- Unit list --
            crate::ui::section_header(ui, "Unit list");

            if unit_list.counters == 0 {
                ui.colored_label(crate::ui::palette::TEXT_DIM, "no placed units");
                return;
            }

            egui::ScrollArea::vertical()
                .id_salt("unit_overview_scroll")
                .show(ui, |ui| {
                    // Collapse all units sharing an identity into one summary
                    // line ("Mulazmin (32x)"). One line per counter made this
                    // view unusably long; the per-counter hex coord is rarely
                    // what you scan for here (the map shows positions). Grouped
                    // by label, not by run: one counter-sheet section can mix
                    // tribes (Ali Wad Helu's Kehena and Degheim), which used to
                    // split into a line per counter. (Cached: see
                    // `UnitListCache`.)
                    for (label, (count, disrupted)) in &unit_list.groups {
                        ui.label(
                            egui::RichText::new(format!("{label} ({count}x)"))
                                .size(13.0)
                                .color(crate::ui::palette::TEXT_STRONG),
                        );
                        if *disrupted > 0 {
                            ui.colored_label(
                                crate::ui::palette::RED,
                                format!("{disrupted} disrupted"),
                            );
                        }
                        ui.add_space(2.0);
                    }
                });
                })
                .response
                .rect
        },
    );
}

/// The unit list's lines ("Mulazmin" -> 32 counters, 3 disrupted), cached
/// across frames by [`unit_overview_ui`].
#[derive(Default)]
pub struct UnitListCache {
    /// How many counters the list was built from.
    counters: usize,
    /// Label -> (counters, disrupted).
    groups: std::collections::BTreeMap<String, (usize, usize)>,
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

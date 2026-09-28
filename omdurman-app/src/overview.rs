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
    mut victory: ResMut<crate::ui_plugin::VictoryModalState>,
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
                let local_may_act = peers.may_act(state.0.phase_player());
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

            let mut units: Vec<_> = placed_units.iter().map(|(_, p)| p).collect();
            units.sort_by_key(|u| (u.section_name.display_name(), u.col, u.row));

            if units.is_empty() {
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
                    // split into a line per counter.
                    let mut groups: std::collections::BTreeMap<String, (usize, usize)> =
                        std::collections::BTreeMap::new();
                    for placed in &units {
                        let label = placed_unit_identity(placed, game_state.as_deref());
                        let entry = groups.entry(label).or_default();
                        entry.0 += 1;
                        entry.1 += usize::from(placed.disrupted);
                    }
                    for (label, (count, disrupted)) in &groups {
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
        UnitIdentity::DervishTribal { tribe } => format!("{tribe}"),
        UnitIdentity::DervishLeader(leader) => format!("{leader}"),
        UnitIdentity::DervishArtillery => "Dervish Artillery".into(),
        UnitIdentity::DervishFort => "Dervish Fort".into(),
        UnitIdentity::DervishGunboat(g) => format!("Dervish Gunboat {g}"),
        UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
            format!("{brigade} * {battalion} Btn")
        }
        UnitIdentity::AngloEgyptianCavalry => "Cavalry".into(),
        UnitIdentity::AngloEgyptianCamelCorps => "Camel Corps".into(),
        UnitIdentity::AngloEgyptianArtillery => "Artillery".into(),
        UnitIdentity::AngloEgyptianMaxim => "Maxim".into(),
        UnitIdentity::AngloEgyptianGunboat(g) => format!("Gunboat {g}"),
        UnitIdentity::AngloEgyptianLeader(leader) => format!("{leader}"),
        UnitIdentity::RoyalEngineers => "Royal Engineers".into(),
        UnitIdentity::AngloEgyptianFort => "British Fort".into(),
    }
}

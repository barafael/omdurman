//! Unit overview panel -- lists all placed units with their identity,
//! position, and state. Shown in both map modes.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_rules::UnitIdentity;
use omdurman_types::BrigadeNationality;

use crate::GameStateResource;
use crate::picker::PlacedUnit;
use crate::rulebook::Rulebook;

/// Right sidebar shown in both map modes. Two stacked sections:
/// **Game control** (turn/phase info + End Phase + scenario set-up, only while a
/// game is live) at the top, then **Unit list** (every placed unit's identity,
/// position, and state) below.
pub fn unit_overview_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    picker_state: Res<crate::picker::PickerState>,
    game_state: Option<Res<GameStateResource>>,
    mut rulebook: ResMut<Rulebook>,
    mut control: crate::ui_plugin::GameControl,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_play() {
        return;
    }

    egui::SidePanel::right("unit_overview_panel")
        .resizable(true)
        .default_width(200.0)
        .width_range(140.0..=320.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(8, 8)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));

            // -- Game control (only while a game is active) --
            if let Some(state) = game_state.as_deref() {
                section_header(ui, "Game control");
                crate::ui_plugin::game_control_section(ui, state, &mut control);
                ui.add_space(10.0);

                // -- Action discovery --
                // What you can do in the current phase, with selected-unit
                // context and § deep-links into the Rulebook tab.
                let mut clicked_section: Option<String> = None;
                crate::actions_panel::draw_actions_section(
                    ui,
                    state,
                    &picker_state,
                    &placed_units,
                    &rulebook,
                    &mut clicked_section,
                );
                if let Some(sec) = clicked_section {
                    crate::rulebook::request_section(&mut rulebook, &sec);
                }
                ui.add_space(10.0);
            }

            // -- Unit list --
            section_header(ui, "Unit list");

            let mut units: Vec<_> = placed_units.iter().map(|(_, p)| p).collect();
            units.sort_by_key(|u| (u.section_name.display_name(), u.col, u.row));

            if units.is_empty() {
                ui.colored_label(egui::Color32::from_gray(140), "no placed units");
                return;
            }

            egui::ScrollArea::vertical()
                .id_salt("unit_overview_scroll")
                .show(ui, |ui| {
                    for placed in &units {
                        let identity_label = placed_unit_identity(placed, game_state.as_deref());
                        let coord_label = format!("({}, {})", placed.coord.q, placed.coord.r);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&identity_label)
                                    .size(13.0)
                                    .color(egui::Color32::from_gray(220)),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.colored_label(egui::Color32::from_gray(140), coord_label);
                                },
                            );
                        });
                        if placed.disrupted {
                            ui.colored_label(egui::Color32::from_rgb(200, 100, 100), "disrupted");
                        }
                        ui.add_space(2.0);
                    }
                });
        });
}

/// A bold section title followed by a separator, for the plain labeled-separator
/// sidebar sections.
fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(16.0)
            .color(egui::Color32::from_gray(220)),
    );
    ui.separator();
    ui.add_space(4.0);
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
            let nat = match brigade.nationality {
                BrigadeNationality::British => 'B',
                BrigadeNationality::Egyptian => 'E',
                BrigadeNationality::Sudanese => 'S',
                BrigadeNationality::Friendlies => 'F',
            };
            format!(
                "{} * {battalion} Btn",
                format_args!("{}{}", brigade.number, nat)
            )
        }
        UnitIdentity::AngloEgyptianCavalry => "Cavalry".into(),
        UnitIdentity::AngloEgyptianCamelCorps => "Camel Corps".into(),
        UnitIdentity::AngloEgyptianArtillery => "Artillery".into(),
        UnitIdentity::AngloEgyptianMaxim => "Maxim".into(),
        UnitIdentity::AngloEgyptianGunboat(g) => format!("Gunboat {g}"),
        UnitIdentity::AngloEgyptianLeader(leader) => format!("{leader}"),
        UnitIdentity::RoyalEngineers => "Royal Engineers".into(),
    }
}

//! Dervish desertion turn overlay + selection UI (rulebook §8.2).
//!
//! On the first night turn of the Campaign scenario the Dervish player must
//! roll one die and remove `floor(1.5 × roll)` units. The Khalifa, gunboats,
//! artillery, and forts are exempt.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::{GameEvent, NetMsg};

use crate::{GameStateResource, PendingEdits};
use omdurman_rules::effects::{GameEffect, desertion_count};
use omdurman_rules::{DieRoll, Phase};
use omdurman_types::Player;

/// A marker resource: set to `Some` when the desertion turn arrives and the
/// Dervish player must choose units to remove.
#[derive(Resource)]
pub(crate) struct DesertionTurn {
    /// The number of units that must be removed (determined by the roll).
    pub count: usize,
    /// The pre-rolled die result (d6, 1..=6, stored as a u8).
    pub roll: u8,
    /// Which Dervish units the player has selected so far.
    pub selected: Vec<omdurman_rules::UnitId>,
}

/// Return whether the current game state is on the desertion turn.
pub(crate) fn is_desertion_turn(gs: &omdurman_rules::effects::GameState) -> bool {
    gs.scenario == omdurman_types::Scenario::Campaign
        && gs.day_night == omdurman_types::DayNight::Night
        && gs.phase == Phase::Movement
        && !gs.dervish_deserted
        && omdurman_rules::turn_track::scenario_turn(gs.scenario, gs.current_turn)
            .is_some_and(|t| t.event == omdurman_rules::turn_track::TurnEvent::DervishDesertion)
}

/// Auto-activate the desertion turn resource when the conditions are met.
pub(crate) fn detect_desertion_turn(
    game_state: Option<Res<GameStateResource>>,
    mut commands: Commands,
    existing: Option<Res<DesertionTurn>>,
    mut game_rng: ResMut<crate::GameRng>,
) {
    let Some(gs) = game_state else { return };
    if existing.is_some() {
        return;
    }
    if !is_desertion_turn(&gs.0) {
        return;
    }
    let roll = game_rng.roll_d6();
    let die_roll = DieRoll::try_from(roll as u16).unwrap();
    let count = desertion_count(die_roll);
    commands.insert_resource(DesertionTurn {
        count,
        roll,
        selected: Vec::with_capacity(count),
    });
}

/// Render the desertion overlay panel.
pub(crate) fn desertion_panel_ui(
    mut contexts: EguiContexts,
    desertion: Option<ResMut<DesertionTurn>>,
    game_state: Option<Res<GameStateResource>>,
    placed_units: Query<(&super::picker::PlacedUnit, Entity)>,
    mut pending: ResMut<PendingEdits>,
    mut commands: Commands,
) {
    let Some(mut desertion) = desertion else {
        return;
    };
    let Some(gs) = game_state else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Area::new(egui::Id::new("desertion_panel"))
        .anchor(egui::Align2::RIGHT_CENTER, [-10.0, 0.0])
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.heading("Dervish Desertion (§8.2)");
            ui.add_space(4.0);

            let die_roll = DieRoll::try_from(desertion.roll as u16).unwrap();
            ui.label(
                egui::RichText::new(format!(
                    "Die roll: {} → remove {} unit{}",
                    desertion.roll,
                    desertion.count,
                    if desertion.count == 1 { "" } else { "s" }
                ))
                .size(13.0)
                .strong(),
            );
            ui.add_space(4.0);

            // Roll reference table
            ui.collapsing("Roll table (§8.2)", |ui| {
                egui::Grid::new("desertion_table").striped(true).show(ui, |ui| {
                    ui.label(egui::RichText::new("Die").strong());
                    ui.label(egui::RichText::new("Remove").strong());
                    ui.end_row();
                    for val in 1..=6u16 {
                        let dr = DieRoll::try_from(val).unwrap();
                        let n = desertion_count(dr);
                        ui.label(format!("{val}"));
                        ui.label(format!("{n}"));
                        ui.end_row();
                    }
                });
            });
            ui.add_space(4.0);

            let remaining = desertion.count.saturating_sub(desertion.selected.len());
            if remaining > 0 {
                ui.label(
                    egui::RichText::new(format!(
                        "Select {} more Dervish unit{} to desert.",
                        remaining,
                        if remaining == 1 { "" } else { "s" }
                    ))
                    .color(egui::Color32::from_rgb(180, 80, 60))
                    .size(12.0),
                );
                ui.add_space(4.0);

                // Show eligible Dervish units
                ui.collapsing("Available units", |ui| {
                    for (placed, _entity) in placed_units.iter() {
                        let Some(unit_id) = placed.unit_id else {
                            continue;
                        };
                        if let Some(unit) = gs.0.find_unit(unit_id) {
                            if unit.profile.identity.owner()
                                != Player::Dervish
                            {
                                continue;
                            }
                            if unit.profile.identity.is_desertion_exempt() {
                                continue;
                            }
                            let is_selected =
                                desertion.selected.contains(&unit.id);
                            let label = unit.profile.identity.short_label();
                            if ui
                                .selectable_label(
                                    is_selected,
                                    label,
                                )
                                .clicked()
                            {
                                if is_selected {
                                    desertion
                                        .selected
                                        .retain(|id| id != &unit.id);
                                } else if desertion.selected.len()
                                    < desertion.count
                                {
                                    desertion.selected.push(unit.id);
                                }
                            }
                        }
                    }
                });
            } else {
                ui.label(
                    egui::RichText::new("All units selected.")
                        .color(egui::Color32::from_rgb(60, 140, 60))
                        .size(12.0),
                );
            }

            ui.add_space(8.0);

            // Confirm button
            let ready = desertion.selected.len() == desertion.count;
            if ui
                .add_enabled(
                    ready,
                    egui::Button::new(
                        egui::RichText::new("Confirm desertion")
                            .size(13.0)
                            .strong(),
                    ),
                )
                .clicked()
            {
                let effect = GameEffect::DervishDesertion {
                    roll: die_roll,
                    deserters: desertion.selected.clone(),
                };
                pending
                    .outgoing_broadcast
                    .push(NetMsg::Game(GameEvent::Effect(effect)));
                commands.remove_resource::<DesertionTurn>();
            }
            }); // Frame
        }); // Area
}

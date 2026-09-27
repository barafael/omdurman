//! Dervish desertion turn overlay + selection UI (rulebook §8.2).
//!
//! On the first night turn of the Campaign scenario the Dervish player must
//! roll one die and remove `floor(1.5 × roll)` units. The Khalifa, gunboats,
//! artillery, and forts are exempt.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::GameEvent;

use crate::{GameStateResource, PendingEdits};
use omdurman_rules::effects::{GameEffect, desertion_count};
use omdurman_rules::{DieRoll, Phase};
use omdurman_types::Player;

/// Present while the desertion turn is open for the local Dervish seat: the
/// pre-rolled die, the resulting count, and the player's selection. Removed
/// once the engine records the desertion (`dervish_deserted`) or the turn
/// otherwise ends.
#[derive(Resource)]
pub(crate) struct DesertionTurn {
    /// The number of units that must be removed (determined by the roll).
    pub count: usize,
    /// The pre-rolled die result (d10, §8.2).
    pub roll: DieRoll,
    /// Which Dervish units the player has selected so far.
    pub selected: Vec<omdurman_rules::UnitId>,
    /// Set once the choice was submitted: the panel then waits for the
    /// sequenced echo instead of offering a second submission.
    pub submitted: bool,
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

/// Open the desertion turn for the local Dervish seat when the conditions
/// are met, and close it once they no longer hold (the echo of the submitted
/// desertion sets `dervish_deserted`, or the phase moved on).
///
/// Only the seat that may act for the Dervish rolls: the die comes from the
/// shared `GameRng` stream, and every other peer drawing from it here would
/// skew their stream for nothing (they never submit the effect).
pub(crate) fn detect_desertion_turn(
    game_state: Res<GameStateResource>,
    mut commands: Commands,
    existing: Option<Res<DesertionTurn>>,
    mut game_rng: ResMut<crate::GameRng>,
    peers: crate::peers::Peers,
    seats: Res<crate::seats::Seats>,
) {
    // An AI-commanded Dervish deserts through the bot driver instead.
    let local_dervish = peers.may_act(Player::Dervish)
        && !crate::seats::ai_factions(&seats.0).contains(&Player::Dervish);
    if !is_desertion_turn(&game_state.0) || !local_dervish {
        if existing.is_some() {
            commands.remove_resource::<DesertionTurn>();
        }
        return;
    }
    if existing.is_some() {
        return;
    }
    // §8.2 "the roll of one die": the game's die is the ten-sided one
    // (§2.4 parts inventory; every other roll in the rules is d10), matching
    // the bot driver and the engine's `desertion_count` domain (1..=10).
    let roll = game_rng.roll_d10();
    let count = desertion_count(roll);
    commands.insert_resource(DesertionTurn {
        count,
        roll,
        selected: Vec::with_capacity(count),
        submitted: false,
    });
}

/// Render the desertion overlay panel.
#[allow(clippy::too_many_arguments)]
pub(crate) fn desertion_panel_ui(
    mut contexts: EguiContexts,
    desertion: Option<ResMut<DesertionTurn>>,
    game_state: Res<GameStateResource>,
    placed_units: Query<(Entity, &super::picker::PlacedUnit)>,
    mut pending: ResMut<PendingEdits>,
    layout: Res<crate::ScreenLayout>,
    peers: crate::peers::Peers,
) {
    // The desertion choice is the Dervish player's: other bound seats don't
    // even see the panel (§8.2).
    if !peers.may_act(omdurman_types::Player::Dervish) {
        return;
    }
    let Some(mut desertion) = desertion else {
        return;
    };
    let gs = &game_state;
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("desertion_panel"),
        egui::Align2::RIGHT_CENTER,
        // Clear of the charts sheet / peek tab (see `right_inset`).
        egui::vec2(-(layout.right_inset + 10.0), 0.0),
        egui::Frame::popup(&ctx.style_of(ctx.theme())),
        |ui| {
            ui.heading("Dervish Desertion (§8.2)");
            ui.add_space(4.0);

            let die_roll = desertion.roll;
            ui.label(
                egui::RichText::new(format!(
                    "Die roll: {} → remove {} unit{}",
                    die_roll.value(),
                    desertion.count,
                    if desertion.count == 1 { "" } else { "s" }
                ))
                .size(13.0)
                .strong(),
            );
            ui.add_space(4.0);

            // Roll reference table
            ui.collapsing("Roll table (§8.2)", |ui| {
                egui::Grid::new("desertion_table")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Die").strong());
                        ui.label(egui::RichText::new("Remove").strong());
                        ui.end_row();
                        for val in 1..=10u16 {
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
                    .color(crate::ui::palette::INK_WARN)
                    .size(12.0),
                );
                ui.add_space(4.0);

                // Show eligible Dervish units
                ui.collapsing("Available units", |ui| {
                    for (_entity, placed) in placed_units.iter() {
                        let Some(unit_id) = placed.unit_id else {
                            continue;
                        };
                        if let Some(unit) = gs.0.find_unit(unit_id) {
                            if unit.profile.identity.owner() != Player::Dervish {
                                continue;
                            }
                            if unit.profile.identity.is_desertion_exempt() {
                                continue;
                            }
                            let is_selected = desertion.selected.contains(&unit.id);
                            let label = unit.profile.identity.short_label();
                            if ui.selectable_label(is_selected, label).clicked() {
                                if is_selected {
                                    desertion.selected.retain(|id| id != &unit.id);
                                } else if desertion.selected.len() < desertion.count {
                                    desertion.selected.push(unit.id);
                                }
                            }
                        }
                    }
                });
            } else {
                ui.label(
                    egui::RichText::new("All units selected.")
                        .color(crate::ui::palette::INK_DONE)
                        .size(12.0),
                );
            }

            ui.add_space(8.0);

            if desertion.submitted {
                ui.label(
                    egui::RichText::new("Desertion submitted — awaiting confirmation…").size(12.0),
                );
                return;
            }

            // Confirm button
            let ready = desertion.selected.len() == desertion.count;
            if ui
                .add_enabled(
                    ready,
                    egui::Button::new(egui::RichText::new("Confirm desertion").size(13.0).strong()),
                )
                .clicked()
            {
                let effect = GameEffect::DervishDesertion {
                    roll: die_roll,
                    deserters: desertion.selected.clone(),
                };
                pending.submit_game(GameEvent::Effect(effect));
                // Keep the resource until the echo closes the desertion turn
                // (`detect_desertion_turn`); removing it here let the
                // detector roll a fresh die before the echo arrived.
                desertion.submitted = true;
            }
        },
    );
}

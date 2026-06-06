//! Game-state UI panel — turn/phase display, phase advancement, game log.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::{GameEvent, NetMsg};

use crate::{GameStateResource, PendingEdits};

/// Draw the game-state panel in an Egui window.
pub fn game_state_ui(
    mut contexts: EguiContexts,
    app_state: Res<State<crate::AppState>>,
    game_state: Res<GameStateResource>,
    mut pending: ResMut<PendingEdits>,
) {
    // Only show the in-game panel once the battle has started (§lobby).
    if *app_state.get() != crate::AppState::InGame {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let state = &game_state.0;

    egui::Window::new("Game State")
        .default_pos(egui::pos2(20.0, 120.0))
        .default_size(egui::vec2(320.0, 400.0))
        .show(ctx, |ui| {
            ui.heading("REMEMBER GORDON!");

            ui.separator();

            ui.label(format!("Turn: {}", state.current_turn.0));
            ui.label(format!("Time: {:?}", state.day_night));
            ui.label(format!("Active: {}", state.active_player));
            ui.label(format!("Phase: {:?}", state.phase));

            if state.game_over {
                ui.colored_label(egui::Color32::RED, "GAME OVER");
                return;
            }

            ui.separator();

            if ui.button("Advance Phase").clicked() {
                info!("advancing phase");
                pending
                    .outgoing_broadcast
                    .push(NetMsg::Game(GameEvent::Effect(
                        omdurman_rules::effects::GameEffect::AdvancePhase,
                    )));
            }

            ui.separator();

            let ae_vp = state
                .victory
                .total_for(omdurman_rules::Player::AngloEgyptian);
            let d_vp = state.victory.total_for(omdurman_rules::Player::Dervish);
            ui.label(format!("VP — A-E: {}, Dervish: {}", ae_vp.0, d_vp.0));

            let ae_count = state
                .units
                .iter()
                .filter(|u| u.profile.identity.owner() == omdurman_rules::Player::AngloEgyptian)
                .count();
            let d_count = state
                .units
                .iter()
                .filter(|u| u.profile.identity.owner() == omdurman_rules::Player::Dervish)
                .count();
            ui.label(format!("Units — A-E: {}, Dervish: {}", ae_count, d_count));

            ui.separator();
            ui.label("Game Log:");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in state.log.iter().rev().take(20).rev() {
                        ui.label(line);
                    }
                });
        });
}

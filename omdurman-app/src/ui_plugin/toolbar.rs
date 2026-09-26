//! The top toolbar switching between Menu / Game / Spectate modes.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn mode_toolbar_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    app_state: Res<State<crate::AppState>>,
    mut next_mode: ResMut<NextState<crate::AppMode>>,
    mut next_app_state: ResMut<NextState<crate::AppState>>,
    mut timeline: ResMut<crate::timeline::SpectatorTimeline>,
    game_state: Res<crate::GameStateResource>,
    phase_machine: Option<Res<State<crate::ui_phase_state::UiPhaseState>>>,
    mut layout: ResMut<crate::ScreenLayout>,
    progress: (Res<crate::TurnState>, Res<crate::game_record::GameRecorder>),
) {
    let game_in_progress = crate::game_in_progress(&progress.0, &progress.1);
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let mut bar_height = None;

    egui::Area::new(egui::Id::new("mode_toolbar"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            // Full-width chrome: one continuous bar across the top instead of
            // a floating island (everything below starts under it).
            ui.set_min_width(ctx.content_rect().width());
            let inner = egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 50, 220))
                .corner_radius(0.0)
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Spectator exit: leave review mode back to the lobby.
                        if *app_state.get() == crate::AppState::Spectating {
                            if ui.button("\u{2b05} Back to lobby").clicked() {
                                timeline.record = None;
                                next_mode.set(crate::AppMode::Lobby);
                                next_app_state.set(crate::AppState::Lobby);
                            }
                            ui.separator();
                        }

                        // Mode label
                        ui.label(
                            egui::RichText::new(match **mode {
                                crate::AppMode::Menu => "Menu",
                                crate::AppMode::Lobby => "Lobby",
                                crate::AppMode::Game => "Game",
                            })
                            .strong()
                            .size(13.0),
                        );
                        ui.separator();

                        // Mode switching buttons: the same transitions as the
                        // M key and the menu's buttons.
                        if ui.button("Menu").clicked() {
                            next_mode.set(crate::AppMode::Menu);
                        }
                        if **mode != crate::AppMode::Game {
                            let game = ui
                                .add_enabled(game_in_progress, egui::Button::new("Game"))
                                .on_disabled_hover_text(
                                    "No game in progress — start one from the Lobby",
                                );
                            if game.clicked() {
                                crate::enter_game_view(&mut next_mode, &mut next_app_state);
                            }
                        }

                        // Phase/turn info when in Game mode (from the
                        // mirrored §4 machine; see `ui_phase_state`)
                        if let Some(machine) = phase_machine.as_ref()
                            && **mode == crate::AppMode::Game
                        {
                            let gs = &game_state;
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("Turn {}", gs.0.current_turn.value()))
                                    .size(13.0),
                            );
                            ui.label(egui::RichText::new(machine.get().phase_label()).size(13.0));
                        }

                        // Keyboard / mouse shortcut reference.
                        ui.separator();
                        ui.menu_button("Keys", |ui| {
                            egui::Grid::new("key_help_grid")
                                .num_columns(2)
                                .spacing(egui::vec2(12.0, 4.0))
                                .show(ui, |ui| {
                                    for (key, what) in crate::hotkeys::KEY_HELP {
                                        ui.label(egui::RichText::new(*key).monospace().strong());
                                        ui.label(*what);
                                        ui.end_row();
                                    }
                                });
                        });
                    });
                });
            bar_height = Some(inner.response.rect.height());
        });
    if let Some(height) = bar_height {
        layout.top_bar_height = height.max(crate::layout::TOP_BAR_HEIGHT);
    }
}

// -- Optional-rule setup UI (§10.11, §10.21) --------------------------------

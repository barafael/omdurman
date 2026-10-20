//! The top toolbar switching between Menu / Game / Spectate modes.
use super::*;

/// The two screen axes the toolbar reads and moves (see `Screen`).
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct ScreenNav<'w> {
    mode: Res<'w, State<crate::AppMode>>,
    app_state: Res<'w, State<crate::AppState>>,
    next_mode: ResMut<'w, NextState<crate::AppMode>>,
    next_app_state: ResMut<'w, NextState<crate::AppState>>,
}

/// The board overlays the toolbar switches on and off.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct OverlayToggles<'w> {
    zoc: ResMut<'w, crate::zoc::ZocOverlay>,
    los: ResMut<'w, crate::los::LosOverlay>,
    roads: ResMut<'w, crate::render::RoadOverlay>,
}

pub(crate) fn mode_toolbar_ui(
    mut contexts: EguiContexts,
    mut nav: ScreenNav,
    mut timeline: ResMut<crate::timeline::SpectatorTimeline>,
    game_state: Res<crate::GameStateResource>,
    phase_machine: Option<Res<State<crate::ui_phase_state::UiPhaseState>>>,
    mut layout: ResMut<crate::ScreenLayout>,
    progress: (Res<crate::TurnState>, Res<crate::game_record::GameRecorder>),
    peers: Peers,
    mut toggles: OverlayToggles,
    room: Res<RoomId>,
    telegrams: Option<Res<crate::telegram::TelegramLog>>,
    dispatches: Option<Res<crate::dispatch::Dispatches>>,
    mut motion: ResMut<crate::fx::MotionSettings>,
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
            // a floating island (everything below starts under it). Exactly
            // the window's width: a minimum alone let the area keep a wider
            // window's size after a shrink (an un-maximize), and the
            // right-aligned room name and LOS / ZOC toggles sat off-screen.
            ui.set_width(ctx.content_rect().width());
            let inner = egui::Frame::new()
                .fill(crate::ui::palette::TOOLBAR_BG)
                .corner_radius(0.0)
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Spectator exit: leave review mode back to the lobby.
                        if *nav.app_state.get() == crate::AppState::Spectating {
                            if ui.button("\u{2b05} Back to lobby").clicked() {
                                timeline.record = None;
                                nav.next_mode.set(crate::AppMode::Lobby);
                                nav.next_app_state.set(crate::AppState::Lobby);
                            }
                            ui.separator();
                        }

                        // Mode label (the game view says where it is in
                        // the turn instead).
                        if **nav.mode != crate::AppMode::Game {
                            ui.label(
                                egui::RichText::new(match **nav.mode {
                                    crate::AppMode::Menu => "Menu",
                                    crate::AppMode::Lobby => "Lobby",
                                    crate::AppMode::Game => "Game",
                                })
                                .strong()
                                .size(13.0),
                            );
                            ui.separator();
                        }

                        // Mode switching buttons: the same transitions as the
                        // M key and the menu's buttons.
                        if ui.button("Menu").clicked() {
                            nav.next_mode.set(crate::AppMode::Menu);
                        }
                        if **nav.mode != crate::AppMode::Game {
                            let game = ui
                                .add_enabled(game_in_progress, egui::Button::new("Game"))
                                .on_disabled_hover_text(
                                    "No game in progress — start one from the Lobby",
                                );
                            if game.clicked() {
                                crate::enter_game_view(&mut nav.next_mode, &mut nav.next_app_state);
                            }
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
                        // How much the board animates (and whether the
                        // camera follows the opponent).
                        ui.menu_button("Motion", |ui| crate::fx::motion_menu(ui, &mut motion));

                        // Every message of the session, newest first: what
                        // the event feed showed, after it faded.
                        if **nav.mode == crate::AppMode::Game
                            && let Some(dispatches) = dispatches.as_ref()
                        {
                            ui.menu_button("Log", |ui| {
                                // Room for the scrolling log as it grows (see `with_room`).
                                crate::ui::with_room(ui, |ui| {
                                ui.set_max_width(520.0);
                                if dispatches.log.is_empty() {
                                    ui.label("Nothing yet.");
                                    return;
                                }
                                egui::ScrollArea::vertical()
                                    .max_height(480.0)
                                    .show(ui, |ui| {
                                        for line in dispatches.log.iter().rev() {
                                            ui.label(egui::RichText::new(line).size(12.0));
                                            ui.separator();
                                        }
                                    });
                            });
                        });
                        }

                        // The turn and who acts now: the one place the game
                        // view says it (it used to be said four times).
                        if let Some(machine) = phase_machine.as_ref()
                            && **nav.mode == crate::AppMode::Game
                            && *nav.app_state.get() == crate::AppState::InGame
                        {
                            ui.separator();
                            let telegram_due = telegrams
                                .as_ref()
                                .is_some_and(|log| log.awaiting_ack(&game_state.0));
                            turn_status(ui, &game_state.0, *machine.get(), &peers, telegram_due);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("Room {}", room.as_str()))
                                            .size(12.0)
                                            .color(crate::ui::palette::TEXT_DIM),
                                    )
                                    .on_hover_text("Share this room name to invite players");
                                    ui.separator();
                                    if overlay_toggle(
                                        ui,
                                        "LOS",
                                        toggles.los.visible,
                                        "Line of sight: hover a hex -- green rings are clear, red blocked (§6.3); P pins \
                                         the hovered hex as the origin and draws the line to the next one hovered",
                                    ) {
                                        toggles.los.visible = !toggles.los.visible;
                                        crate::ui_trace::button("overlay: LOS");
                                    }
                                    if overlay_toggle(
                                        ui,
                                        "ZOC",
                                        toggles.zoc.visible,
                                        "Enemy zones of control (§5.41)",
                                    ) {
                                        toggles.zoc.visible = !toggles.zoc.visible;
                                        crate::ui_trace::button("overlay: ZOC");
                                    }
                                    if overlay_toggle(
                                        ui,
                                        "Roads",
                                        toggles.roads.visible,
                                        "Road links: a step along a road costs 1 MP whatever the terrain (§5.11)",
                                    ) {
                                        toggles.roads.visible = !toggles.roads.visible;
                                        crate::ui_trace::button("overlay: Roads");
                                    }
                                },
                            );
                        }
                    });
                });
            bar_height = Some(inner.response.rect.height());
        });
    if let Some(height) = bar_height {
        layout.top_bar_height = height.max(crate::layout::TOP_BAR_HEIGHT);
        // Top-center cards stack from just below the bar.
        layout.center_stack_y = layout.top_bar_height + crate::layout::STACK_GAP;
    }
}

// -- Optional-rule setup UI (§10.11, §10.21) --------------------------------

/// A map-overlay toggle chip; returns whether it was clicked.
fn overlay_toggle(ui: &mut egui::Ui, label: &str, active: bool, hover: &str) -> bool {
    let button = egui::Button::new(egui::RichText::new(label).size(12.0).monospace().color(
        if active {
            crate::ui::palette::HIGHLIGHT
        } else {
            crate::ui::palette::BRASS_DIM
        },
    ))
    .fill(if active {
        crate::ui::palette::HIGHLIGHT_BG
    } else {
        crate::ui::palette::CHIP_BG
    });
    ui.add(button).on_hover_text(hover).clicked()
}

/// The turn line of the top bar: turn, clock and day/night, whose turn it
/// is, the §4 phase ladder, and who acts now -- "you" in gold, a waiting
/// line otherwise (input is gated silently while the other side acts, so
/// the bar must say why nothing responds).
fn turn_status(
    ui: &mut egui::Ui,
    gs: &omdurman_rules::effects::GameState,
    machine: crate::ui_phase_state::UiPhaseState,
    peers: &Peers,
    telegram_due: bool,
) {
    use crate::ui::palette;
    use crate::ui_phase_state::UiPhaseState;
    let name = crate::ui::faction_name;
    let clock = omdurman_rules::turn_track::scenario_turn(gs.scenario, gs.current_turn)
        .map(|entry| format!("  {}", entry.time))
        .unwrap_or_default();
    ui.label(
        egui::RichText::new(format!("Turn {}{clock}", gs.current_turn.value()))
            .size(13.0)
            .strong(),
    );
    if gs.day_night == omdurman_types::DayNight::Night {
        crate::ui::frames::tag(palette::NIGHT_BADGE_BG, 6)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("\u{1f319} Night")
                        .size(11.0)
                        .color(palette::NIGHT_BLUE),
                );
            })
            .response
            .on_hover_text(
                "Night (§8): A-E movement halved, fire ranges halved (min 1), no howitzer fire",
            );
    }
    ui.separator();
    let (text, color) = match machine {
        UiPhaseState::NoGame => ("Setting up\u{2026}".to_string(), palette::TEXT_DIM),
        UiPhaseState::GameOver => (
            match gs.game_result {
                Some(result) => format!("Game over \u{2014} {}", result.display_key()),
                None => "Game over".to_string(),
            },
            palette::GOLD,
        ),
        UiPhaseState::Setup => {
            let first = gs.first_to_set_up();
            let deploying = if gs.setup_ready(first) {
                first.opponent()
            } else {
                first
            };
            if peers.commands_faction(deploying) {
                (
                    format!("\u{25b6} Set-up: you deploy ({})", name(deploying)),
                    palette::GOLD,
                )
            } else {
                (
                    format!("Set-up: waiting for {} to deploy\u{2026}", name(deploying)),
                    palette::TEXT_DIM,
                )
            }
        }
        UiPhaseState::Turn { active, .. } => {
            ui.label(
                egui::RichText::new(format!("{} turn", name(active)))
                    .size(13.0)
                    .color(crate::ui::faction_color(active)),
            );
            ui.label(
                egui::RichText::new(machine.phase_sequence().replace(" > ", " \u{203a} "))
                    .size(12.0)
                    .color(palette::RAIL_DIM),
            );
            ui.separator();
            let actor = machine.acting_player().unwrap_or(active);
            if telegram_due {
                // The host's AI holds its moves until the turn's telegram
                // is read here (`TelegramLog::awaiting_ack`).
                (
                    "Waiting for the turn's telegram (Enter to read on)".to_string(),
                    palette::TEXT_DIM,
                )
            } else if peers.paused() {
                (
                    "Paused \u{2014} waiting for a commander to return\u{2026}".to_string(),
                    palette::CAUTION,
                )
            } else if let Some(choice) = gs.open_loss_choice() {
                // §7.7: a combat waits on its owner's choice of losses --
                // possibly the side whose turn it is not.
                if peers.commands_faction(choice.player) {
                    (
                        "\u{25b6} Your move: choose your losses (§7.7)".to_string(),
                        palette::GOLD,
                    )
                } else {
                    (
                        format!(
                            "Waiting for {} to choose losses\u{2026}",
                            name(choice.player)
                        ),
                        palette::TEXT_DIM,
                    )
                }
            } else if peers.commands_faction(actor) {
                // The ladder just left of it names the phase.
                ("\u{25b6} Your move".to_string(), palette::GOLD)
            } else {
                (
                    format!("Waiting for {}\u{2026}", name(actor)),
                    palette::TEXT_DIM,
                )
            }
        }
    };
    ui.label(egui::RichText::new(text).size(13.0).strong().color(color));
}

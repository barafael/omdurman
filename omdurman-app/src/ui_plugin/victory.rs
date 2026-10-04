//! The end-of-game victory modal.
use super::*;

/// Whether the player closed the victory modal. Reset whenever the shown
/// state is not over (a new game, or scrubbing the timeline back), so the
/// next game over opens it again. Reopened from the rail's "Show result"
/// button or the floating chip drawn while it is closed.
#[derive(Resource, Default)]
pub(crate) struct VictoryModalState {
    pub dismissed: bool,
}

/// What a victory-modal button asked for.
enum VictoryAction {
    Close,
    Review,
    Leave,
}

/// Navigation targets the victory modal's buttons drive. Bundled to keep
/// [`victory_modal`] under clippy's argument limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct VictoryNav<'w> {
    app_state: Res<'w, State<AppState>>,
    next_mode: ResMut<'w, NextState<crate::AppMode>>,
    next_app_state: ResMut<'w, NextState<AppState>>,
    timeline: ResMut<'w, crate::timeline::SpectatorTimeline>,
    recorder: Res<'w, crate::game_record::GameRecorder>,
}

pub(crate) fn victory_modal(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    report: Option<Res<crate::newspaper::NewspaperReport>>,
    mut telegrams: Option<ResMut<crate::telegram::TelegramLog>>,
    mut keepsakes: Option<ResMut<crate::keepsakes::Keepsakes>>,
    mut modal: ResMut<VictoryModalState>,
    mut nav: VictoryNav,
) {
    let Some(state) = game_state else { return };
    if !state.0.game_over {
        modal.dismissed = false;
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let spectating = *nav.app_state.get() == AppState::Spectating;

    if modal.dismissed {
        // Closed: a small chip to bring the result back (the rail has the
        // same button while in a live game).
        crate::ui::anchored_card(
            ctx,
            egui::Id::new("victory_reopen"),
            egui::Align2::CENTER_BOTTOM,
            egui::vec2(0.0, -16.0),
            crate::ui::frames::chip(),
            |ui| {
                if ui.button("Game over \u{2014} show result").clicked() {
                    modal.dismissed = false;
                }
            },
        );
        return;
    }

    let mut action: Option<VictoryAction> = None;
    let page = report.as_ref().and_then(|r| r.page.clone());
    // A keepsake of the game played (live play only, not a review).
    if page.is_some()
        && !spectating
        && let Some(keepsakes) = keepsakes.as_mut()
    {
        keepsakes.request(&nav.recorder, "gazette");
    }
    let has_record = nav
        .recorder
        .record
        .as_ref()
        .is_some_and(|r| !r.events.is_empty());
    // The Gazette replaces the last telegram: mark them all read.
    if let Some(log) = telegrams.as_mut() {
        let filed = log.entries.len();
        if log.acknowledged < filed {
            log.acknowledged = filed;
        }
    }

    // A dimmed desk behind the paper.
    let screen = ctx.viewport_rect();
    egui::Area::new(egui::Id::new("gazette_backdrop"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            ui.set_clip_rect(screen);
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(170));
        });
    let card = egui::Area::new(egui::Id::new("victory_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            // Room to the screen bottom: the front page scrolls in it (see `with_room`).
            crate::ui::with_room(ui, |ui| {
                let width = (screen.width() - 80.0).clamp(600.0, 1180.0);
                let height = screen.height() - 70.0;
                ui.set_max_width(width);
                match &page {
                    Some(page) => {
                        egui::ScrollArea::vertical()
                            .id_salt("gazette_scroll")
                            .max_height(height - 50.0)
                            .show(ui, |ui| super::gazette::draw_front_page(ui, page, width));
                    }
                    None => {
                        crate::ui::frames::modal().show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("GAME OVER")
                                    .size(28.0)
                                    .strong()
                                    .color(crate::ui::palette::TITLE),
                            );
                        });
                    }
                }
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            action = Some(VictoryAction::Close);
                        }
                        if !spectating
                            && ui
                                .add_enabled(has_record, egui::Button::new("Review timeline"))
                                .on_hover_text("Scrub back through the recorded game")
                                .clicked()
                        {
                            action = Some(VictoryAction::Review);
                        }
                        let leave = if spectating {
                            "Back to lobby"
                        } else {
                            "Main menu"
                        };
                        if ui.button(leave).clicked() {
                            action = Some(VictoryAction::Leave);
                        }
                    });
                });
            });
        });
    ctx.move_to_top(card.response.layer_id);

    match action {
        Some(VictoryAction::Close) => modal.dismissed = true,
        Some(VictoryAction::Review) => {
            // Same entry as the lobby's "Review current game".
            if let Some(record) = nav.recorder.record.as_ref() {
                nav.timeline
                    .open(record.clone(), "current game".to_string());
                nav.next_app_state.set(AppState::Spectating);
            }
        }
        Some(VictoryAction::Leave) => {
            if spectating {
                // Same exit as the toolbar's spectator "Back to lobby".
                nav.timeline.record = None;
                nav.next_mode.set(crate::AppMode::Lobby);
                nav.next_app_state.set(AppState::Lobby);
            } else {
                nav.next_mode.set(crate::AppMode::Menu);
            }
        }
        None => {}
    }
}

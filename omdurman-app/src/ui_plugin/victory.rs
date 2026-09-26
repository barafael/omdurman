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
            egui::Frame::new()
                .fill(crate::ui::panel_bg())
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(8, 4)),
            |ui| {
                if ui.button("Game over \u{2014} show result").clicked() {
                    modal.dismissed = false;
                }
            },
        );
        return;
    }

    let paper_bg = egui::Color32::from_rgb(42, 36, 28);
    let paper_border = egui::Color32::from_rgb(180, 160, 110);
    let masthead_color = egui::Color32::from_rgb(200, 180, 120);
    let headline_color = egui::Color32::from_rgb(230, 210, 150);
    let subhead_color = egui::Color32::from_rgb(170, 155, 110);
    let dim_color = egui::Color32::from_rgb(140, 130, 100);
    let mut action: Option<VictoryAction> = None;

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("victory_modal"),
        egui::Align2::CENTER_CENTER,
        egui::Vec2::ZERO,
        egui::Frame::new()
            .fill(paper_bg)
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(32, 24))
            .stroke(egui::Stroke::new(2.0, paper_border)),
        |ui| {
            ui.set_max_width(520.0);
            ui.vertical_centered(|ui| {
                if let Some(r) = report.as_ref() {
                    // Masthead
                    ui.label(
                        egui::RichText::new(&r.masthead)
                            .size(22.0)
                            .strong()
                            .color(masthead_color),
                    );
                    ui.label(
                        egui::RichText::new(&r.date_line)
                            .size(11.0)
                            .color(dim_color),
                    );
                } else {
                    // Fallback before the report is populated.
                    ui.label(
                        egui::RichText::new("GAME OVER")
                            .size(28.0)
                            .strong()
                            .color(headline_color),
                    );
                }

                ui.add_space(6.0);
                // Horizontal rule
                let rect = ui.available_rect_before_wrap();
                let y = rect.min.y;
                ui.painter().line_segment(
                    [
                        egui::pos2(rect.min.x + 8.0, y),
                        egui::pos2(rect.max.x - 8.0, y),
                    ],
                    egui::Stroke::new(1.0, paper_border),
                );
                ui.add_space(6.0);

                // Headline
                if let Some(r) = report.as_ref() {
                    ui.label(
                        egui::RichText::new(&r.headline)
                            .size(20.0)
                            .strong()
                            .color(headline_color),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(&r.subhead)
                            .size(13.0)
                            .italics()
                            .color(subhead_color),
                    );
                }

                ui.add_space(6.0);
                // Horizontal rule
                let rect = ui.available_rect_before_wrap();
                let y = rect.min.y;
                ui.painter().line_segment(
                    [
                        egui::pos2(rect.min.x + 8.0, y),
                        egui::pos2(rect.max.x - 8.0, y),
                    ],
                    egui::Stroke::new(0.5, paper_border),
                );
                ui.add_space(8.0);

                // Stats block
                if let Some(r) = report.as_ref() {
                    ui.label(
                        egui::RichText::new(format!(
                            "Scenario: {}   |   Turns played: {}   |   Result: {}",
                            r.scenario, r.turns_played, r.result_key,
                        ))
                        .size(11.0)
                        .color(dim_color),
                    );
                    ui.add_space(6.0);
                }

                // LLM-generated body paragraphs.
                let body_color = egui::Color32::from_rgb(190, 180, 150);
                if let Some(r) = report.as_ref()
                    && !r.paragraphs.is_empty()
                {
                    for para in &r.paragraphs {
                        ui.label(egui::RichText::new(para).size(12.0).color(body_color));
                        ui.add_space(4.0);
                    }
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        action = Some(VictoryAction::Close);
                    }
                    let has_record = nav
                        .recorder
                        .record
                        .as_ref()
                        .is_some_and(|r| !r.events.is_empty());
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
        },
    );

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

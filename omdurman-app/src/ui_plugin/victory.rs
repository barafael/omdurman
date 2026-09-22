//! The end-of-game victory modal.
use super::*;

pub(crate) fn victory_modal(
    mut contexts: EguiContexts,
    game_state: Option<Res<crate::GameStateResource>>,
    report: Option<Res<crate::newspaper::NewspaperReport>>,
) {
    let Some(state) = game_state else { return };
    if !state.0.game_over {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let paper_bg = egui::Color32::from_rgb(42, 36, 28);
    let paper_border = egui::Color32::from_rgb(180, 160, 110);
    let masthead_color = egui::Color32::from_rgb(200, 180, 120);
    let headline_color = egui::Color32::from_rgb(230, 210, 150);
    let subhead_color = egui::Color32::from_rgb(170, 155, 110);
    let dim_color = egui::Color32::from_rgb(140, 130, 100);

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
            });
        },
    );
}

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::EditorMode;

#[derive(Resource, Default)]
pub struct SecretState {
    pub slide: usize,
}

const SLIDES: &[Slide] = &[
    Slide {
        title: "General Gordon & the Fall of Khartoum",
        body: "In 1884 General Gordon was sent to evacuate Khartoum but chose to defend it instead. After a 10-month siege by Mahdist forces, the city fell two days before a British relief column arrived. Gordon was killed on the palace steps. 'Too late!' became a national outcry that haunted Britain for a generation.",
    },
    Slide {
        title: "Kitchener & the Battle of Omdurman",
        body: "Thirteen years later Kitchener led an Anglo-Egyptian army up the Nile to avenge Gordon. On 2 September 1898 the forces met at Omdurman. Kitchener's 26,000 men armed with rifles and Maxim guns faced 52,000 Mahdists charging across open ground. Over 10,000 Mahdists fell; fewer than 500 on the Anglo-Egyptian side. A young Winston Churchill charged with the 21st Lancers.",
    },
    Slide {
        title: "About This Exploration",
        body: "This interactive sandbox reconstructs the terrain of the Battle of Omdurman — the Nile, desert, forts, and villages that shaped the fight. Browse annotated terrain, place unit markers, study the river corridors, and understand how geography determined the battle. Switch modes via the toolbar or keyboard shortcuts. A work-in-progress tool to think with.",
    },
];

struct Slide {
    title: &'static str,
    body: &'static str,
}

pub fn secret_ui(
    mut contexts: EguiContexts,
    mode: Res<EditorMode>,
    mut state: ResMut<SecretState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if *mode != EditorMode::Secret {
        return;
    }

    ctx.input(|i| {
        if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowDown) {
            state.slide = (state.slide + 1) % SLIDES.len();
        }
        if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowUp) {
            state.slide = (state.slide + SLIDES.len() - 1) % SLIDES.len();
        }
    });

    let slide = &SLIDES[state.slide];
    let bg = egui::Color32::from_rgb(10, 10, 10);

    egui::Area::new(egui::Id::new("secret_backdrop"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            let screen = ui.max_rect();

            ui.painter().rect_filled(screen, 0.0, bg);
            ui.allocate_rect(screen, egui::Sense::click());

            let margin_x = screen.width() * 0.1;
            let inner = egui::Rect::from_min_max(
                screen.min + egui::vec2(margin_x, 80.0),
                screen.max - egui::vec2(margin_x, 80.0),
            );

            ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui: &mut egui::Ui| {
                ui.style_mut().override_font_id = Some(egui::FontId::proportional(34.0));
                ui.colored_label(egui::Color32::from_rgb(220, 180, 100), slide.title);

                ui.add_space(28.0);

                ui.style_mut().override_font_id =
                    Some(egui::FontId::new(26.0, egui::FontFamily::Name("Garamond".into())));
                ui.label(egui::RichText::new(slide.body).color(egui::Color32::WHITE));

                ui.add_space(48.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));
                    ui.label(
                        egui::RichText::new(format!("{}/{}", state.slide + 1, SLIDES.len()))
                            .color(egui::Color32::GRAY),
                    );
                });

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("<  >  navigate  ·  Ctrl+1  exit")
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        });
}

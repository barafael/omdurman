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
        body: &[
            "In 1884, General Charles George Gordon was sent to Khartoum to",
            "evacuate Egyptian garrisons threatened by the Mahdist uprising.",
            "",
            "Ignoring orders, Gordon chose to defend the city. For 10 months,",
            "Khartoum was besieged by Mahdist forces under the Mahdi.",
            "",
            "A British relief column arrived on 28 January 1885 — two days",
            "too late. Khartoum had fallen and Gordon was killed on the",
            "steps of the Governor's palace.",
            "",
            "\"Too late!\" became a national outcry in Britain, and Gordon's",
            "death haunted the Victorian imagination for a generation.",
        ],
    },
    Slide {
        title: "Kitchener & the Battle of Omdurman",
        body: &[
            "Thirteen years later, Major-General Herbert Kitchener led a",
            "Anglo-Egyptian army up the Nile to avenge Gordon and crush",
            "the Mahdist state.",
            "",
            "On 2 September 1898, the forces met at Omdurman, just north",
            "of Khartoum. Kitchener commanded ~26,000 men armed with",
            "Lee-Metford rifles, Maxim machine guns, and artillery.",
            "",
            "The Mahdist army, ~52,000 strong, charged across open ground",
            "in a frontal assault. Modern firepower decimated their ranks",
            "— over 10,000 Mahdists fell, while Anglo-Egyptian losses",
            "numbered fewer than 500.",
            "",
            "Winston Churchill, then a young cavalry officer, charged with",
            "the 21st Lancers in one of the last great cavalry actions.",
        ],
    },
    Slide {
        title: "About This Exploration",
        body: &[
            "This interactive map reconstructs the terrain of the Battle of",
            "Omdurman — the Nile, the desert, the forts, and the villages",
            "that shaped the battle.",
            "",
            "It is a sandbox for historical exploration: browse annotated",
            "terrain, place unit markers on the ground, study the river",
            "corridors, and understand how geography determined the fight.",
            "",
            "Switch between modes using the toolbar or Ctrl+1..6.",
            "Explore the hex overlay (Ctrl+1), edit terrain (Ctrl+2),",
            "place units (Ctrl+3), browse sprites (Ctrl+4), or roll dice",
            "to simulate uncertainty (Ctrl+5).",
            "",
            "This is a work-in-progress — a tool to think with.",
        ],
    },
];

struct Slide {
    title: &'static str,
    body: &'static [&'static str],
}

#[allow(deprecated)]
pub fn secret_ui(
    mut contexts: EguiContexts,
    mode: Res<EditorMode>,
    mut state: ResMut<SecretState>,
) {
    let ctx = guard_mode!(contexts, mode, Secret);

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

            let inner = egui::Rect::from_min_max(
                screen.min + egui::vec2(80.0, 60.0),
                screen.max - egui::vec2(80.0, 60.0),
            );

            ui.allocate_ui_at_rect(inner, |ui| {
                ui.style_mut().override_font_id =
                    Some(egui::FontId::proportional(28.0));
                ui.colored_label(
                    egui::Color32::from_rgb(220, 180, 100),
                    slide.title,
                );

                ui.add_space(24.0);

                ui.style_mut().override_font_id =
                    Some(egui::FontId::proportional(20.0));

                for line in slide.body {
                    if line.is_empty() {
                        ui.add_space(10.0);
                    } else {
                        ui.label(egui::RichText::new(*line).color(egui::Color32::WHITE));
                    }
                }

                ui.add_space(40.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.style_mut().override_font_id =
                        Some(egui::FontId::proportional(14.0));
                    ui.label(
                        egui::RichText::new(format!("{}/{}", state.slide + 1, SLIDES.len()))
                            .color(egui::Color32::GRAY),
                    );
                });

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("←  →  navigate  ·  Ctrl+0  exit")
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        });
}

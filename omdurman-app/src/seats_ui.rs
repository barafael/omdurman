//! Seat UI: the pause notice shown while a seat holder is away.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::seats::{self, SeatView, human_seats};

/// Top-center card under the phase banner while the game is paused: names
/// every absent seat holder and counts down to their seat's abandonment.
pub(crate) fn pause_card_ui(
    mut contexts: EguiContexts,
    view: SeatView,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    if !view.presence.paused() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let absent: Vec<(String, String, Option<f64>)> = human_seats(&view.seats.0)
        .filter(|(key, _)| !view.presence.is_connected(*key))
        .map(|(key, seat)| {
            let until = (!view.presence.abandoned(key))
                .then(|| view.presence.secs_until_abandoned(key))
                .flatten();
            (view.presence.name(key), seats::seat_label(seat), until)
        })
        .collect();
    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("seat_pause_card"),
        crate::ui::frames::hud().stroke(egui::Stroke::new(1.0, crate::ui::palette::CAUTION)),
        |ui| {
            ui.label(
                egui::RichText::new("\u{23f8} Game paused")
                    .size(16.0)
                    .strong()
                    .color(crate::ui::palette::CAUTION),
            );
            for (name, seat, until) in &absent {
                ui.label(
                    egui::RichText::new(format!("Waiting for {name} ({seat})."))
                        .color(crate::ui::palette::TEXT),
                );
                let status = match until {
                    Some(secs) => {
                        format!("Seat claimable by a newcomer in {}s.", secs.ceil() as u32)
                    }
                    None => "Seat abandoned \u{2014} a newcomer may claim it.".to_string(),
                };
                ui.label(
                    egui::RichText::new(status)
                        .size(12.0)
                        .color(crate::ui::palette::TEXT_MUTED),
                );
            }
        },
    );
}

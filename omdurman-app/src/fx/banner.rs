//! "Your move": a short note on the board when play passes to the local
//! player. Every other phase change stays in the toolbar.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::MotionSettings;

/// Seconds the note stays up, fades included.
const BANNER_SECS: f32 = 1.6;
/// Seconds of fade at each end.
const BANNER_FADE: f32 = 0.25;

pub(super) struct Banner {
    phase: String,
    age: f32,
}

/// When the side to act becomes one the local player commands, say so for a
/// moment above the board ("Your move -- Offensive Fire"). Never modal,
/// never clickable; off with marks off; never for a spectator.
#[allow(clippy::too_many_arguments)]
pub(super) fn your_move_banner_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    settings: Res<MotionSettings>,
    machine: Res<State<crate::ui_phase_state::UiPhaseState>>,
    peers: crate::peers::Peers,
    layout: Res<crate::ScreenLayout>,
    mut was_mine: Local<bool>,
    mut banner: Local<Option<Banner>>,
) {
    if machine.is_changed() {
        let mine = machine
            .acting_player()
            .is_some_and(|actor| peers.commands_faction(actor));
        if mine && !*was_mine && settings.effects() {
            *banner = Some(Banner {
                phase: machine.phase_label().to_string(),
                age: 0.0,
            });
        } else if !mine {
            *banner = None;
        }
        *was_mine = mine;
    }
    let Some(shown) = banner.as_mut() else { return };
    shown.age += time.delta_secs();
    if shown.age >= BANNER_SECS {
        *banner = None;
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let alpha = (shown.age / BANNER_FADE)
        .min((BANNER_SECS - shown.age) / BANNER_FADE)
        .clamp(0.0, 1.0);
    let screen = ctx.content_rect();
    let board_centre_x =
        (layout.left_inset + (screen.right() - layout.right_inset)) * 0.5 - screen.center().x;
    egui::Area::new(egui::Id::new("your_move_banner"))
        .order(egui::Order::Foreground)
        .interactable(false)
        .anchor(
            egui::Align2::CENTER_TOP,
            egui::vec2(board_centre_x, layout.center_stack_y + 16.0),
        )
        .show(ctx, |ui| {
            ui.set_opacity(alpha);
            crate::ui::frames::paper(egui::Stroke::new(1.0, crate::ui::palette::FAINT_INK))
                .inner_margin(egui::Margin::symmetric(14, 6))
                .show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    ui.label(
                        egui::RichText::new(format!("Your move \u{2014} {}", shown.phase))
                            .size(14.0)
                            .color(crate::ui::palette::INK),
                    );
                });
        });
    ctx.request_repaint();
}

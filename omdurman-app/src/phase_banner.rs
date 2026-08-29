//! Phase banner — a floating top-center panel that tells the player what is
//! happening right now: turn number, day/night, active player, current phase,
//! the phase-sequence indicator, and (during night) a rules reminder.
//!
//! Also handles phase-transition animation and the "Your turn" popup.
//!
//! Every visible detail is driven by [`UiPhaseState`], which is derived from
//! [`GameStateResource`] each frame.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::peers::Peers;
use crate::ui_phase_state::{FireSubKind, PhaseKind, UiPhaseState};
use crate::{GameStateResource, GameTurn};

// ---------------------------------------------------------------------------
// Animation resource — tracks transitions so we can animate them
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct PhaseBannerAnimation {
    /// The phase state from the previous frame.
    pub prev: Option<UiPhaseState>,
    /// When the current phase began (wall-clock seconds via `Time::elapsed_secs`).
    pub phase_enter_time: f64,
    /// If non-None, a "Your turn" popup is being shown; the value is when it
    /// started (wall-clock seconds).
    pub your_turn_popup: Option<f64>,
}

impl Default for PhaseBannerAnimation {
    fn default() -> Self {
        Self {
            prev: None,
            phase_enter_time: 0.0,
            your_turn_popup: None,
        }
    }
}

/// Duration of the slide-in animation (seconds).
const BANNER_ANIM_SECS: f64 = 0.3;
/// How long the "Your turn" popup stays visible (seconds).
const YOUR_TURN_DURATION: f64 = 2.5;
/// Height offset during slide-in (in egui points).
const BANNER_SLIDE_IN: f32 = -60.0;

// ---------------------------------------------------------------------------
// Update system — detect transitions, animate, manage popup
// ---------------------------------------------------------------------------

pub fn update_phase_banner_animation(
    time: Res<Time>,
    game_state: Option<Res<GameStateResource>>,
    peers: Peers,
    mut anim: ResMut<PhaseBannerAnimation>,
) {
    let Some(gs) = game_state else {
        // No game active — reset.
        *anim = PhaseBannerAnimation::default();
        return;
    };

    let current = UiPhaseState::derive(&gs.0);

    // Phase transition detection.
    if anim.prev != Some(current) {
        let was_turn = matches!(anim.prev, Some(UiPhaseState::Turn { .. }));
        anim.phase_enter_time = time.elapsed_secs_f64();

        // "Your turn" popup: show when the active player changes and the local
        // player can now act.
        if let UiPhaseState::Turn { active, .. } = current
            && (!was_turn
                || !anim.prev.is_some_and(
                    |p| matches!(p, UiPhaseState::Turn { active: a, .. } if a == active),
                ))
            && peers.may_act(active)
        {
            anim.your_turn_popup = Some(time.elapsed_secs_f64());
        }

        anim.prev = Some(current);
    }

    // Auto-dismiss "Your turn" popup.
    if let Some(start) = anim.your_turn_popup
        && time.elapsed_secs_f64() - start > YOUR_TURN_DURATION
    {
        anim.your_turn_popup = None;
    }
}

// ---------------------------------------------------------------------------
// Egui drawing
// ---------------------------------------------------------------------------

/// Colours used in the phase banner.
mod colour {
    use bevy_egui::egui::Color32;
    pub const BG: Color32 = Color32::from_rgb(35, 30, 25);
    pub const BORDER: Color32 = Color32::from_rgb(180, 160, 110);
    pub const TITLE: Color32 = Color32::from_rgb(230, 210, 160);
    pub const DIM: Color32 = Color32::from_rgb(160, 150, 130);
    pub const GOLD: Color32 = Color32::from_rgb(230, 200, 110);
    pub const GREY: Color32 = Color32::from_gray(150);
    pub const NIGHT_BLUE: Color32 = Color32::from_rgb(100, 130, 200);
    pub const POPUP_BG: Color32 = Color32::from_rgb(50, 45, 35);
}

/// Render the phase banner and the "Your turn" popup.
pub fn phase_banner_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<GameStateResource>>,
    turn: Option<Res<GameTurn>>,
    mut anim: ResMut<PhaseBannerAnimation>,
    time: Res<Time>,
    peers: Peers,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Some(gs) = game_state else { return };
    let Some(turn) = turn else { return };

    let state = UiPhaseState::derive(&gs.0);
    let elapsed = time.elapsed_secs_f64() - anim.phase_enter_time;
    let t = (elapsed / BANNER_ANIM_SECS).min(1.0);
    let y_offset = BANNER_SLIDE_IN * (1.0 - ease_out_cubic(t as f32));

    // -- Phase banner (centred top) --
    let day_night_str = match gs.0.day_night {
        omdurman_types::DayNight::Day => "Day",
        omdurman_types::DayNight::Night => "Night",
    };

    let active_player_label = match gs.0.active_player {
        omdurman_types::Player::AngloEgyptian => "Anglo-Egyptian",
        omdurman_types::Player::Dervish => "Dervish",
    };

    // Whose turn
    let my_turn = peers.may_act(gs.0.active_player);

    crate::ui::anchored_card(
        ctx,
        egui::Id::new("phase_banner"),
        egui::Align2::CENTER_TOP,
        egui::vec2(0.0, 8.0 + y_offset),
        egui::Frame::new()
            .fill(colour::BG)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(20, 10))
            .stroke(egui::Stroke::new(1.0, colour::BORDER)),
        |ui| {
            // Line 1: turn / day-night / active-player / night badge
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "Turn {}  {}  {}",
                        **turn, day_night_str, active_player_label,
                    ))
                    .size(13.0)
                    .color(colour::DIM),
                );

                // Turn indicator
                let turn_str = if my_turn {
                    "\u{25b6} Your turn"
                } else {
                    "Waiting on opponent"
                };
                ui.label(egui::RichText::new(turn_str).size(13.0).color(if my_turn {
                    colour::GOLD
                } else {
                    colour::GREY
                }));

                // Night badge
                if gs.0.day_night == omdurman_types::DayNight::Night {
                    ui.add_space(8.0);
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgba_unmultiplied(40, 50, 80, 200))
                        .corner_radius(3.0)
                        .inner_margin(egui::Margin::symmetric(6, 2))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("\u{1f319} Night")
                                    .size(11.0)
                                    .color(colour::NIGHT_BLUE),
                            );
                        });
                }
            });

            ui.add_space(4.0);

            // Line 2: phase label (large)
            ui.label(
                egui::RichText::new(match state {
                    UiPhaseState::Setup => "Setup — Deploy Forces",
                    UiPhaseState::GameOver => "Game Over",
                    UiPhaseState::Turn { phase, .. } => match phase {
                        PhaseKind::Movement => "Movement",
                        PhaseKind::DefensiveFire(FireSubKind::Direct) => "Defensive Fire — Direct",
                        PhaseKind::DefensiveFire(FireSubKind::MaximHowitzer) => {
                            "Defensive Fire — Maxim/Howitzer"
                        }
                        PhaseKind::OffensiveFire(FireSubKind::Direct) => "Offensive Fire — Direct",
                        PhaseKind::OffensiveFire(FireSubKind::MaximHowitzer) => {
                            "Offensive Fire — Maxim/Howitzer"
                        }
                        PhaseKind::Melee => "Melee",
                    },
                })
                .size(20.0)
                .strong()
                .color(colour::TITLE),
            );

            ui.add_space(2.0);

            // Line 3: sequence indicator
            let seq = state.phase_sequence();
            ui.label(egui::RichText::new(seq).size(12.0).color(colour::DIM));

            // Night rules reminder (only during night)
            if gs.0.day_night == omdurman_types::DayNight::Night {
                ui.add_space(4.0);
                ui.label(
                            egui::RichText::new(
                                "\u{2022} A-E movement halved  \u{2022} Ranges halved (min 1)  \u{2022} No howitzer fire",
                            )
                            .size(10.0)
                            .color(colour::NIGHT_BLUE),
                        );
            }
        },
    );

    // -- "Your turn" popup --
    if let Some(start) = anim.your_turn_popup {
        let popup_alpha = ((time.elapsed_secs_f64() - start) / YOUR_TURN_DURATION).clamp(0.0, 1.0);
        let fade = 1.0 - popup_alpha; // fades out over lifetime

        let color = egui::Color32::from_rgba_premultiplied(230, 200, 110, (fade * 200.0) as u8);
        crate::ui::anchored_card(
            ctx,
            egui::Id::new("your_turn_popup"),
            egui::Align2::CENTER_CENTER,
            egui::Vec2::ZERO,
            egui::Frame::new()
                .fill(colour::POPUP_BG)
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(40, 20))
                .stroke(egui::Stroke::new(2.0, color)),
            |ui| {
                ui.label(
                    egui::RichText::new("Your Turn!")
                        .size(28.0)
                        .strong()
                        .color(color),
                );
                ui.label(
                    egui::RichText::new("Select a unit and take your action")
                        .size(14.0)
                        .color(colour::DIM),
                );
                if ui
                    .button("Dismiss")
                    .on_hover_text("click to dismiss")
                    .clicked()
                {
                    anim.your_turn_popup = None;
                }
            },
        );
    }
}

/// Cubic ease-out: starts fast, decelerates toward the end.
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

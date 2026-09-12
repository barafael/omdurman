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
        anim.phase_enter_time = time.elapsed_secs_f64();

        // "Your turn" popup: show when the *acting* player (the player who
        // may act in this specific phase, not just the turn owner) changes to
        // the local player. During Defensive Fire the acting player is the
        // non-moving side (§6.4/§6.7), so the popup correctly greets the
        // defender when control passes to them.
        let cur_actor = current.acting_player();
        let prev_actor = anim.prev.as_ref().and_then(|p| p.acting_player());
        if let Some(acting) = cur_actor
            && prev_actor != Some(acting)
            && peers.may_act(acting)
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

#[allow(clippy::too_many_arguments)]
/// Render the phase banner and the "Your turn" popup. Reads the mirrored
/// §4 turn machine (see `ui_phase_state`) rather than deriving the phase
/// from the engine state itself.
pub fn phase_banner_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<GameStateResource>>,
    turn: Option<Res<GameTurn>>,
    machine: Res<State<crate::ui_phase_state::UiPhaseState>>,
    mut anim: ResMut<PhaseBannerAnimation>,
    time: Res<Time>,
    peers: Peers,
    picker: Option<Res<crate::picker::UnitPicker>>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Some(gs) = game_state else { return };
    let Some(turn) = turn else { return };

    let state = machine.get();
    let elapsed = time.elapsed_secs_f64() - anim.phase_enter_time;
    let t = (elapsed / BANNER_ANIM_SECS).min(1.0);
    let y_offset = BANNER_SLIDE_IN * (1.0 - ease_out_cubic(t as f32));

    // -- Phase banner (top-center stack anchor, slides in from under the bar) --
    let stack_y = layout.center_stack_y;
    let day_night_str = match gs.0.day_night {
        omdurman_types::DayNight::Day => "Day",
        omdurman_types::DayNight::Night => "Night",
    };

    fn player_label(p: omdurman_types::Player) -> &'static str {
        match p {
            omdurman_types::Player::AngloEgyptian => "Anglo-Egyptian",
            omdurman_types::Player::Dervish => "Dervish",
        }
    }

    // Turn owner: the moving player. This stays fixed for the whole player
    // turn even when control passes to the other side for defensive fire.
    let turn_owner = gs.0.active_player;
    let i_am_owner = peers.may_act(turn_owner);
    let owner_text = format!(
        "{} Turn{}",
        player_label(turn_owner),
        if i_am_owner { " (you)" } else { "" }
    );

    // Phase actor: who may act in the current phase. During Defensive Fire the
    // *non-moving* player fires back (§6.4/§6.7), so the actor differs from
    // the turn owner. Outside an active turn the line falls back to the fixed
    // Setup / Game Over titles, which don't name a player.
    let phase_actor = state.acting_player().unwrap_or(turn_owner);
    let i_am_actor = state.acting_player().is_some_and(|p| peers.may_act(p));
    let actor_text = match state {
        UiPhaseState::NoGame | UiPhaseState::Setup => "Setup — Deploy Forces".to_string(),
        UiPhaseState::GameOver => "Game Over".to_string(),
        UiPhaseState::Turn { phase, .. } => {
            let phase_name = match phase {
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
            };
            format!(
                "{} {}{}",
                player_label(phase_actor),
                phase_name,
                if i_am_actor { " (you)" } else { "" }
            )
        }
    };

    let mut banner_height = 0.0f32;
    egui::Area::new(egui::Id::new("phase_banner"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, stack_y + y_offset))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let inner = egui::Frame::new()
                .fill(colour::BG)
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(20, 10))
                .stroke(egui::Stroke::new(1.0, colour::BORDER))
                .show(ui, |ui| {
            // Line 1: turn / day-night / turn-owner (the moving player, whose
            // turn it remains even during the opponent's defensive fire).
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Turn {}  {}  ", **turn, day_night_str))
                        .size(13.0)
                        .color(colour::DIM),
                );

                ui.label(
                    egui::RichText::new(&owner_text)
                        .size(13.0)
                        .strong()
                        .color(if i_am_owner { colour::GOLD } else { colour::DIM }),
                );

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

            // Line 2: phase label (large), showing WHO acts in this phase and
            // what they are doing — e.g. "Anglo-Egyptian Defensive Fire — Direct".
            // During defensive fire this is the non-moving side, so the banner
            // reflects the actual control transfer each phase.
            let actor_color = match state {
                UiPhaseState::Turn { .. } => {
                    if i_am_actor {
                        colour::GOLD
                    } else {
                        colour::GREY
                    }
                }
                _ => colour::TITLE,
            };
            ui.label(
                egui::RichText::new(&actor_text)
                    .size(20.0)
                    .strong()
                    .color(actor_color),
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

            // Reinforcement reminder (§9.112/§9.113): the active side still
            // has counters admitted by this turn's order of appearance.
            if let Some(hint) = picker.as_ref().and_then(|picker| {
                crate::reinforce::reinforcement_hint(&gs.0, picker)
            }) {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(hint).size(10.0).color(colour::DIM));
            }
                });
            banner_height = inner.response.rect.height();
        });
    // Advance the top-center stack cursor so cards below never overlap the
    // banner (measured at rest; the slide-in offset is transient).
    if banner_height > 0.0 {
        layout.center_stack_y = stack_y + banner_height + crate::layout::STACK_GAP;
    }

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

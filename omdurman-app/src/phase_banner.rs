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

use crate::GameStateResource;
use crate::peers::Peers;
use crate::ui_phase_state::{FireSubKind, PhaseKind, UiPhaseState};

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
            && peers.commands_faction(acting)
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

#[allow(clippy::too_many_arguments)]
/// Render the phase banner and the "Your turn" popup. Reads the mirrored
/// §4 turn machine (see `ui_phase_state`) rather than deriving the phase
/// from the engine state itself.
pub fn phase_banner_ui(
    mut contexts: EguiContexts,
    game_state: Res<GameStateResource>,
    machine: Res<State<crate::ui_phase_state::UiPhaseState>>,
    anim: Res<PhaseBannerAnimation>,
    time: Res<Time>,
    peers: Peers,
    picker: Option<Res<crate::picker::UnitPicker>>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let gs = game_state;
    let turn = gs.0.current_turn.value();

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

    let player_label = crate::ui::faction_name;

    // Turn owner: the moving player. This stays fixed for the whole player
    // turn even when control passes to the other side for defensive fire.
    let turn_owner = gs.0.active_player;
    let i_am_owner = peers.commands_faction(turn_owner);
    let paused = peers.paused();
    let game_over = matches!(state, UiPhaseState::GameOver);
    // Set-up is sequential (§9.111/§9.211/§9.321): the side deploying now.
    let deploying = {
        let first = gs.0.first_to_set_up();
        if gs.0.setup_ready(first) {
            first.opponent()
        } else {
            first
        }
    };
    let in_setup = matches!(state, UiPhaseState::Setup);
    // Once the game is over there is no turn owner (the engine has already
    // handed the turn on when it found no next turn): line 1 names only the
    // last turn played. During set-up it names the side deploying.
    let owner_text = if game_over {
        String::new()
    } else if in_setup {
        format!(
            "{} deploying{}",
            player_label(deploying),
            if peers.commands_faction(deploying) {
                " (you)"
            } else {
                ""
            }
        )
    } else {
        format!(
            "{} Turn{}",
            player_label(turn_owner),
            if i_am_owner { " (you)" } else { "" }
        )
    };

    // Phase actor: who may act in the current phase. During Defensive Fire the
    // *non-moving* player fires back (§6.4/§6.7), so the actor differs from
    // the turn owner. Outside an active turn the line falls back to the fixed
    // Setup / Game Over titles, which don't name a player.
    let phase_actor = state.acting_player().unwrap_or(turn_owner);
    let i_am_actor = state
        .acting_player()
        .is_some_and(|p| peers.commands_faction(p));
    let actor_text = match state {
        UiPhaseState::NoGame => "Setup — Deploy Forces".to_string(),
        UiPhaseState::Setup if deploying == gs.0.first_to_set_up() => {
            format!("Setup — {} Deploys First", player_label(deploying))
        }
        UiPhaseState::Setup => format!("Setup — {} Deploys", player_label(deploying)),
        UiPhaseState::GameOver => match gs.0.game_result {
            Some(result) => format!("Game Over \u{2014} {}", result.display_key()),
            None => "Game Over".to_string(),
        },
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
    // Dim the banner frame while it is someone else's sub-phase: paired with
    // the waiting line below, "input does nothing" reads as *waiting*, not as
    // a frozen game.
    let waiting_for_opponent = matches!(state, UiPhaseState::Turn { .. })
        && state.acting_player().is_some()
        && (!i_am_actor || paused);
    let border_stroke = if waiting_for_opponent {
        egui::Stroke::new(1.0, crate::ui::palette::TEXT_DIM)
    } else {
        egui::Stroke::new(1.0, crate::ui::palette::CHROME_BORDER)
    };
    egui::Area::new(egui::Id::new("phase_banner"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, stack_y + y_offset))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let inner = crate::ui::frames::hud()
                .stroke(border_stroke)
                .show(ui, |ui| {
            // Line 1: turn / day-night / turn-owner (the moving player, whose
            // turn it remains even during the opponent's defensive fire).
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Turn {turn}  {day_night_str}  "))
                        .size(13.0)
                        .color(crate::ui::palette::RAIL_DIM),
                );

                ui.label(
                    egui::RichText::new(&owner_text)
                        .size(13.0)
                        .strong()
                        .color(if (i_am_owner && !game_over && !in_setup)
                            || (in_setup && peers.commands_faction(deploying))
                        {
                            crate::ui::palette::GOLD
                        } else {
                            crate::ui::palette::RAIL_DIM
                        }),
                );

                // Night badge
                if gs.0.day_night == omdurman_types::DayNight::Night {
                    ui.add_space(8.0);
                    crate::ui::frames::tag(crate::ui::palette::NIGHT_BADGE_BG, 6)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("\u{1f319} Night")
                                    .size(11.0)
                                    .color(crate::ui::palette::NIGHT_BLUE),
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
                        crate::ui::palette::GOLD
                    } else {
                        crate::ui::palette::TEXT_DIM
                    }
                }
                _ => crate::ui::palette::TITLE,
            };
            ui.label(
                egui::RichText::new(&actor_text)
                    .size(20.0)
                    .strong()
                    .color(actor_color),
            );

            // Explicit waiting line: during the opponent's sub-phase every
            // click is gated by `may_act` *silently*, which read as a frozen
            // game. Name who is acting instead.
            if waiting_for_opponent {
                ui.add_space(2.0);
                let waiting = if paused {
                    "Paused \u{2014} waiting for a commander to return\u{2026}".to_string()
                } else {
                    format!("Waiting for {} to act\u{2026}", player_label(phase_actor))
                };
                ui.label(
                    egui::RichText::new(waiting)
                    .size(12.0)
                    .color(crate::ui::palette::TEXT_DIM),
                );
            }

            ui.add_space(2.0);

            // Line 3: sequence indicator
            let seq = state.phase_sequence();
            ui.label(egui::RichText::new(seq).size(12.0).color(crate::ui::palette::RAIL_DIM));

            // Night rules reminder (only during night)
            if gs.0.day_night == omdurman_types::DayNight::Night {
                ui.add_space(4.0);
                ui.label(
                            egui::RichText::new(
                                "\u{2022} A-E movement halved  \u{2022} Ranges halved (min 1)  \u{2022} No howitzer fire",
                            )
                            .size(10.0)
                            .color(crate::ui::palette::NIGHT_BLUE),
                        );
            }

            // Reinforcement reminder (§9.112/§9.113): the active side still
            // has counters admitted by this turn's order of appearance.
            if let Some(hint) = picker.as_ref().and_then(|picker| {
                crate::reinforce::reinforcement_hint(&gs.0, picker)
            }) {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(hint).size(10.0).color(crate::ui::palette::RAIL_DIM));
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

        let color = crate::ui::palette::with_alpha_premultiplied(
            crate::ui::palette::GOLD,
            (fade * 200.0) as u8,
        );
        // Non-interactable: a notice, not a dialog — clicks pass straight
        // through to the board (it used to swallow them for its lifetime).
        egui::Area::new(egui::Id::new("your_turn_popup"))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(crate::ui::palette::MODAL_BG)
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(40, 20))
                    .stroke(egui::Stroke::new(2.0, color))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Your Turn!")
                                .size(28.0)
                                .strong()
                                .color(color),
                        );
                        ui.label(
                            egui::RichText::new("Select a unit and take your action")
                                .size(14.0)
                                .color(crate::ui::palette::RAIL_DIM),
                        );
                    });
            });
    }
}

/// Cubic ease-out: starts fast, decelerates toward the end.
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

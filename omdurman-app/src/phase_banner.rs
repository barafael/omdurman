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
}

impl Default for PhaseBannerAnimation {
    fn default() -> Self {
        Self {
            prev: None,
            phase_enter_time: 0.0,
        }
    }
}

/// Content width of the phase banner: fits the longest title
/// ("Anglo-Egyptian Defensive Fire — Direct (you)") on one line at 20 pt.
const BANNER_WIDTH: f32 = 470.0;

/// Duration of the slide-in animation (seconds).
const BANNER_ANIM_SECS: f64 = 0.3;
/// Height offset during slide-in (in egui points).
const BANNER_SLIDE_IN: f32 = -60.0;

// ---------------------------------------------------------------------------
// Update system — detect transitions, animate
// ---------------------------------------------------------------------------

/// Restart the banner's slide-in on every phase change. (A centred "Your
/// Turn!" popup used to greet the acting side too; the sliding banner already
/// says "(you)", so the popup was dropped as a duplicate over the board.)
pub fn update_phase_banner_animation(
    time: Res<Time>,
    game_state: Option<Res<GameStateResource>>,
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
        anim.prev = Some(current);
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
            // A fixed width (wide enough for the longest phase title on one
            // line) and fixed line slots below: the cards stacked under the
            // banner -- the melee card with its Resolve button -- must not
            // jump when a line comes or goes (night/day, waiting/acting).
            ui.set_width(BANNER_WIDTH);
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
            // (The slot is kept, empty, while the local player acts.)
            ui.add_space(2.0);
            let waiting = if !waiting_for_opponent {
                " ".to_string()
            } else if paused {
                "Paused \u{2014} waiting for a commander to return\u{2026}".to_string()
            } else {
                format!("Waiting for {} to act\u{2026}", player_label(phase_actor))
            };
            ui.label(
                egui::RichText::new(waiting)
                .size(12.0)
                .color(crate::ui::palette::TEXT_DIM),
            );

            ui.add_space(2.0);

            // Line 3: sequence indicator
            let seq = state.phase_sequence();
            ui.label(egui::RichText::new(seq).size(12.0).color(crate::ui::palette::RAIL_DIM));

            // Night rules reminder (only during night; the slot is kept by day).
            ui.add_space(4.0);
            let night_rules = if gs.0.day_night == omdurman_types::DayNight::Night {
                "\u{2022} A-E movement halved  \u{2022} Ranges halved (min 1)  \u{2022} No howitzer fire"
            } else {
                " "
            };
            ui.label(
                egui::RichText::new(night_rules)
                    .size(10.0)
                    .color(crate::ui::palette::NIGHT_BLUE),
            );

            // Reinforcement reminder (§9.112/§9.113): the active side still
            // has counters admitted by this turn's order of appearance.
            if let Some(hint) = picker.as_ref().and_then(|picker| {
                crate::reinforce::reinforcement_hint(&gs.0, picker)
            }) {
                ui.add_space(4.0);
                crate::rulebook::refs_label(ui, &hint, crate::ui::palette::RAIL_DIM, 10.0);
            }
                });
            banner_height = inner.response.rect.height();
        });
    // Advance the top-center stack cursor so cards below never overlap the
    // banner (measured at rest; the slide-in offset is transient).
    if banner_height > 0.0 {
        layout.center_stack_y = stack_y + banner_height + crate::layout::STACK_GAP;
    }
}

/// Cubic ease-out: starts fast, decelerates toward the end.
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

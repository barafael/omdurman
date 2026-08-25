//! Shared `GameEvent` application path for live messages (`handle_socket`)
//! and snapshot replay (`replay_game_history`).
//!
//! `PlaceUnit` / `MoveUnit` are handled separately by `apply_pending_placement`
//! because they need picker + mesh-asset access; both callers route those
//! events through their own queues and never pass them here.
//!
//! `GameEvent::Effect` is dispatched to the rules engine and mutates
//! [`GameState`]; the remaining variants update map/editor/UI state.

use bevy::prelude::*;
use omdurman_net::GameEvent;
use omdurman_rules::effects::{GameState, apply_effect};

pub struct GameApplyCtx<'a> {
    pub game_state: Option<&'a mut GameState>,
}

pub fn apply_game_event(event: &GameEvent, ctx: &mut GameApplyCtx<'_>) {
    match event {
        GameEvent::StartGame { .. } => {}
        GameEvent::Effect(effect) => {
            if let Some(ref mut state) = ctx.game_state {
                debug!(?effect, "applying game effect");
                if let Err(e) = apply_effect(state, effect) {
                    warn!("effect rejected: {e}");
                } else {
                    debug!(
                        phase = ?state.phase,
                        turn = state.current_turn.value(),
                        active_player = ?state.active_player,
                        "effect applied successfully"
                    );
                }
            } else {
                warn!("GameEvent::Effect received but no GameState available");
            }
        }
        GameEvent::PlaceUnit { .. }
        | GameEvent::MoveUnit { .. }
        | GameEvent::RemoveUnit { .. } => {
            // Callers route these into their own deferred queues before
            // calling apply_game_event; reaching this arm is a routing bug.
            warn!(?event, "placement event reached apply_game_event");
        }
        GameEvent::TurnComplete(summary) => {
            // Turn summaries are already built by `apply_effect(EndPlayerTurn)`
            // during replay. This event is recorded for the canonical log and
            // for late-joiner information; no additional state mutation needed.
            debug!(turn = summary.turn.value(), "TurnComplete (informational)");
        }
    }
}

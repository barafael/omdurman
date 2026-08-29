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
use omdurman_rules::OptionalRule;
use omdurman_rules::board::BoardInfo;
use omdurman_rules::effects::{GameState, apply_effect};
use omdurman_types::{MapKind, Player, Scenario};

pub struct GameApplyCtx<'a> {
    pub game_state: Option<&'a mut GameState>,
}

/// State core shared by the live path (`net_socket::handle_socket`) and the
/// replay path (`timeline::rebuild_state_to`), so the two cannot drift:
///
/// * stage the faction binding (applied to the peer entities later by
///   `peers::apply_faction_bindings`);
/// * seed a fresh engine state — `GameState::new` sets the scenario's
///   first-moving player (§9.113/§9.212/§9.322) — and push the committed
///   optional rule (§10.11/§10.21), which the replay path previously dropped;
/// * attach the scenario's board to the engine state *synchronously*, so
///   movement costing / ZOC never validate against an empty board between
///   `StartGame` and the deferred visual map load;
/// * stage the *visual* board load for the next frame (§dual-map).
///
/// Caller-specific concerns (mode switches, snapshot requests) stay with the
/// callers.
pub(crate) fn apply_start_game(
    assignments: &[(bevy_matchbox::prelude::PeerId, Player)],
    scenario: Scenario,
    optional_rule: Option<OptionalRule>,
    game_state: Option<&mut GameState>,
    queued_factions: &mut crate::peers::QueuedFactions,
    loaded_annotations: &crate::board_state::LoadedAnnotations,
    pending_map_load: &mut crate::board_state::PendingMapLoad,
) -> MapKind {
    queued_factions.0 = Some(
        assignments
            .iter()
            .map(|(pid, faction)| (*pid, *faction))
            .collect(),
    );
    if let Some(gs) = game_state {
        *gs = GameState::new(scenario);
        if let Some(rule) = optional_rule {
            gs.optional_rules.push(rule);
        }
        let map_kind = crate::scenario_setup::map_kind_for_scenario(scenario);
        gs.board = BoardInfo::from_map_data(loaded_annotations.map(map_kind));
        pending_map_load.0 = Some(map_kind);
        map_kind
    } else {
        let map_kind = crate::scenario_setup::map_kind_for_scenario(scenario);
        pending_map_load.0 = Some(map_kind);
        map_kind
    }
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
        GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } | GameEvent::RemoveUnit { .. } => {
            // Callers route these into their own deferred queues before
            // calling apply_game_event; reaching this arm is a routing bug.
            warn!(?event, "placement event reached apply_game_event");
        }
    }
}

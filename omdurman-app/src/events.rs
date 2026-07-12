//! Bevy [`Message`] types that decouple game-action producers (input systems,
//! network handler) from their consumers (state appliers, UI updaters).
//!
//! # Two-way event flow
//!
//! * **Outbound** -- [`LocalAction`] is emitted by local input systems
//!   (picker clicks, combat buttons) to request a game action.
//!   -> [`forward_local_actions`] bridges it into [`PendingEdits`] for the wire.
//!
//! * **Inbound** -- [`GameEventApplied`] is emitted after
//!   a sequenced game event is applied.
//!   -> UI / game systems listen to trigger side-effects (status text, etc.).

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use omdurman_net::GameEvent;
use omdurman_rules::effects::Observation;

use crate::{AppliedEvents, PendingEdits};

// -- Outbound -----------------------------------------------------------------

/// A game action initiated by the local player (unit placement, movement,
/// combat, map edit, ...). The [`forward_local_actions`] system converts it to
/// a `NetMsg::Game(...)` and stages it for broadcast.
#[derive(Message, Clone)]
pub struct LocalAction {
    pub event: GameEvent,
}

/// Bridge: listen for [`LocalAction`] messages and push them to the outbound
/// reliable-broadcast buffer so [`flush_pending`](crate::net_plugin::flush_pending) sends
/// them on the next frame.
pub fn forward_local_actions(
    mut reader: MessageReader<LocalAction>,
    mut pending: ResMut<PendingEdits>,
) {
    for action in reader.read() {
        info!("forward_local_actions: bridging LocalAction to PendingEdits");
        pending
            .outgoing_broadcast
            .push(omdurman_net::NetMsg::Game(action.event.clone()));
    }
}

// -- Inbound ------------------------------------------------------------------

/// Emitted by [`drain_applied_events`] after a sequenced game event has been
/// applied to the local game state. Listening systems can react to specific
/// events (e.g. a unit placement completes -> refresh picker) without polling
/// every frame or coupling directly to the socket handler.
#[derive(Message, Clone)]
#[allow(dead_code)]
pub struct GameEventApplied {
    pub event: GameEvent,
    pub seq: u32,
}

/// Drains [`AppliedEvents`] (written by [`handle_socket`](crate::net_socket::handle_socket))
/// and re-emits each entry as a [`GameEventApplied`] message so decoupled
/// listeners (UI, status text, picker refresh) can react.
pub fn drain_applied_events(
    mut buffer: ResMut<AppliedEvents>,
    mut writer: MessageWriter<GameEventApplied>,
) {
    for (event, seq) in buffer.0.drain(..) {
        writer.write(GameEventApplied { event, seq });
    }
}

// -- Observations -----------------------------------------------------------

/// Staging buffer for [`Observation`]s drained from the rules engine's
/// [`GameState`](omdurman_rules::effects::GameState) after each `apply_effect`
/// call. A scheduled system drains this into [`ObservationEvent`] messages so
/// decoupled listeners (dispatch slips, sounds, VP animations) can react without
/// polling game state every frame.
#[derive(Resource, Default)]
pub struct PendingObservations(pub Vec<(Observation, u32)>);

/// Emitted by [`drain_observations`] for each engine observation. Wraps the
/// observation with the `seq` of the game event that produced it, so listeners
/// can correlate with [`GameEventApplied`].
#[derive(Message, Clone)]
#[allow(dead_code)]
pub struct ObservationEvent {
    pub observation: Observation,
    pub seq: u32,
}

/// Drain [`PendingObservations`] into [`ObservationEvent`] messages.
pub fn drain_observations(
    mut buffer: ResMut<PendingObservations>,
    mut writer: MessageWriter<ObservationEvent>,
) {
    for (obs, seq) in buffer.0.drain(..) {
        writer.write(ObservationEvent {
            observation: obs,
            seq,
        });
    }
}

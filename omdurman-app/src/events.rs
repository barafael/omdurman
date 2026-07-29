//! Bevy [`Message`] types that decouple game-action producers (input systems,
//! network handler) from their consumers (state appliers, UI updaters).
//!
//! * **Outbound** -- [`LocalAction`] is emitted by local input systems
//!   (picker clicks, combat buttons) to request a game action.
//!   -> [`forward_local_actions`] bridges it into [`PendingEdits`] for the wire.
//!
//! * **Inbound** -- [`ObservationEvent`] is emitted for each engine
//!   observation produced by applying a sequenced game event.
//!   -> UI listeners (dispatch slips, combat cards) react without polling.

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use omdurman_rules::effects::Observation;

use crate::PendingEdits;

/// A game action initiated by the local player (unit placement, movement,
/// combat, map edit, ...). The [`forward_local_actions`] system converts it to
/// a `NetMsg::Game(...)` and stages it for broadcast.
#[derive(Message, Clone)]
pub struct LocalAction {
    pub event: omdurman_net::GameEvent,
}

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

/// Staging buffer for [`Observation`]s drained from the rules engine's
/// [`GameState`](omdurman_rules::effects::GameState) after each `apply_effect`
/// call. A scheduled system drains this into [`ObservationEvent`] messages so
/// decoupled listeners (dispatch slips, sounds, VP animations) can react without
/// polling game state every frame.
#[derive(Resource, Default)]
pub struct PendingObservations(pub Vec<Observation>);

#[derive(Message, Clone)]
pub struct ObservationEvent {
    pub observation: Observation,
}

pub fn drain_observations(
    mut buffer: ResMut<PendingObservations>,
    mut writer: MessageWriter<ObservationEvent>,
) {
    for obs in buffer.0.drain(..) {
        writer.write(ObservationEvent { observation: obs });
    }
}

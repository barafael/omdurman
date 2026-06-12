//! Bevy [`Message`] types that decouple game-action producers (input systems,
//! network handler) from their consumers (state appliers, UI updaters).
//!
//! # Two-way event flow
//!
//! * **Outbound** — [`LocalAction`] is emitted by local input systems
//!   (picker clicks, combat buttons) to request a game action.
//!   → [`forward_local_actions`] bridges it into [`PendingEdits`] for the wire.
//!
//! * **Inbound** — [`GameEventApplied`] is emitted by [`handle_socket`] after
//!   a sequenced game event is applied.
//!   → UI / game systems listen to trigger side-effects (status text, etc.).

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use omdurman_net::GameEvent;

use crate::{AppliedEvents, PendingEdits};

// ── Outbound ─────────────────────────────────────────────────────────────────

/// A game action initiated by the local player (unit placement, movement,
/// combat, map edit, …). The [`forward_local_actions`] system converts it to
/// a `NetMsg::Game(…)` and stages it for broadcast.
#[derive(Message, Clone)]
pub struct LocalAction {
    pub event: GameEvent,
}

/// Bridge: listen for [`LocalAction`] messages and push them to the outbound
/// reliable-broadcast buffer so [`flush_pending`](crate::flush_pending) sends
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

// ── Inbound ──────────────────────────────────────────────────────────────────

/// Emitted by [`drain_applied_events`] after a sequenced game event has been
/// applied to the local game state. Listening systems can react to specific
/// events (e.g. a unit placement completes → refresh picker) without polling
/// every frame or coupling directly to the socket handler.
#[derive(Message, Clone)]
#[allow(dead_code)]
pub struct GameEventApplied {
    pub event: GameEvent,
    pub seq: u32,
}

/// Drains [`AppliedEvents`] (written by [`handle_socket`](crate::handle_socket))
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

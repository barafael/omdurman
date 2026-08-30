use crate::peers::{
    LobbyPick, PeerColor, PeerCursor, PeerName, Spectator, apply_faction_bindings,
    sync_peer_entities,
};
use crate::state::AppState;
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_matchbox::prelude::{MatchboxSocket, PeerId};
use omdurman_net::{CH_RELIABLE, Ephemeral, GameEvent, NetMsg, NetState, enc_msg, open_socket};

// -- Net resources (moved from main.rs) -------------------------------------

/// Tracks whether the game has begun (set by the host's `StartGame`). Used by
/// the snapshot / host-failover paths in `net_socket`. The turn itself lives in
/// the rules engine (`GameState.active_player` / `phase`), advanced by the
/// `End Phase` button -- there is no separate app-level turn counter.
#[derive(Resource, Default)]
pub(crate) struct TurnState {
    pub game_started: bool,
}

/// Throttle cursor-position broadcasts to ~10 Hz.
#[derive(Resource)]
pub(crate) struct CursorBroadcastTimer(Timer);

impl Default for CursorBroadcastTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

/// Frame-scoped staging buffer for reliable outbound messages.
///
/// Why a buffer at all if matchbox channels already queue? Two reasons:
/// (1) systems can stage messages without taking `&mut MatchboxSocket`, which
///     would conflict with other socket-using systems; (2) `flush_pending`
///     routes the host's own `NetMsg::Game` entries through `incoming.loopback`
///     (still unsequenced) so `handle_socket` sequences them through the same
///     arm as guest submissions -- a single serialization point. Recording
///     happens via `GameRecorder::push_event` on the `NetMsg::Sequenced` echo,
///     so the host records on echo exactly like every other peer.
///
/// Unreliable messages (cursors, ephemeral UI selections) bypass this and go
/// straight to the socket via `omdurman_net::broadcast_unreliable`.
#[derive(Resource, Default)]
pub struct PendingEdits {
    /// Reliable broadcast to all peers.
    pub outgoing_broadcast: Vec<NetMsg>,
    /// Reliable send to a single peer.
    pub outgoing_targeted: Vec<(NetMsg, PeerId)>,
    /// Base for submission-unique ids (`next_uid + counter`). Seeded randomly
    /// at plugin build so two app instances never generate colliding uids.
    pub next_uid: u64,
    /// Submitted events awaiting confirmation (their `Sequenced` echo).
    /// Retransmitted every [`SUBMIT_RETRANSMIT_SECS`] until confirmed, so
    /// player input survives a host death or an in-flight send loss.
    pub unconfirmed: std::collections::VecDeque<(u64, GameEvent)>,
    /// Retransmission accumulator (seconds of accumulated frame delta).
    pub retransmit_timer: f32,
    /// Seconds since the `unconfirmed` queue was last empty. Nonzero stall
    /// means our submissions are not reaching a sequencing host.
    pub stall_secs: f32,
}

impl PendingEdits {
    /// Stage a game-event submission: assign a fresh submission uid, register
    /// the event for retransmission until confirmed, and queue the wire
    /// message. All `NetMsg::Game` traffic must go through this (or, for
    /// retransmits/forwards, reuse an existing uid).
    pub fn submit_game(&mut self, event: GameEvent) -> u64 {
        let uid = self.next_uid;
        self.next_uid = self.next_uid.wrapping_add(1);
        self.unconfirmed.push_back((uid, event.clone()));
        self.outgoing_broadcast.push(NetMsg::Game { uid, event });
        uid
    }

    /// Mark a submission confirmed (its sequenced echo was applied). Unbounded
    /// growth is impossible: `unconfirmed` only holds events submitted but
    /// not yet echoed, and submissions are rate-bounded by gameplay.
    pub fn confirm(&mut self, uid: u64) {
        self.unconfirmed.retain(|(u, _)| *u != uid);
    }

    /// Re-stage every still-unconfirmed submission, at most once per
    /// [`SUBMIT_RETRANSMIT_SECS`]. At-least-once submission; exactly-once
    /// application is guaranteed by uid dedup on the receive path and
    /// idempotent re-echoing on the host.
    pub fn retransmit_unconfirmed(&mut self, delta_secs: f32) {
        self.retransmit_timer += delta_secs;
        if self.unconfirmed.is_empty() {
            self.stall_secs = 0.0;
        } else {
            self.stall_secs += delta_secs;
        }
        if self.retransmit_timer < SUBMIT_RETRANSMIT_SECS || self.unconfirmed.is_empty() {
            return;
        }
        self.retransmit_timer = 0.0;
        debug!(
            pending = self.unconfirmed.len(),
            "retransmitting unconfirmed submissions"
        );
        for (uid, event) in &self.unconfirmed {
            self.outgoing_broadcast.push(NetMsg::Game {
                uid: *uid,
                event: event.clone(),
            });
        }
    }
}

/// Cadence for retransmitting unconfirmed submissions.
pub(crate) const SUBMIT_RETRANSMIT_SECS: f32 = 0.5;
/// Submissions unconfirmed for this long mean the submission path itself is
/// broken (e.g. the channel to the host died without a disconnect event);
/// `auto_reconnect_on_stall` rebuilds the socket via the standard
/// `handle_reconnect` path. Confirmations in a healthy game arrive in
/// fractions of a second; this is twenty dead retransmit rounds.
pub(crate) const SUBMIT_STALL_RECONNECT_SECS: f32 = 10.0;

#[derive(Resource, Default)]
pub struct PendingIncoming {
    /// `PlaceUnit` / `MoveUnit` events received live -- recorded by
    /// `apply_pending_placement` and applied to the world. Other game
    /// events are applied inline by `handle_socket`; these two are deferred
    /// because they need access to the picker + mesh/material asset pools.
    /// The `Option<u8>` is the pre-computed sender index.
    pub live: Vec<(GameEvent, PeerId, Option<u8>)>,
    /// Same kind of events but injected from a `GameHistory` replay --
    /// already in the canonical event log, so must NOT be re-recorded.
    pub replay: Vec<(GameEvent, PeerId)>,
    /// Ephemeral display messages buffered by `handle_socket` for
    /// `apply_ephemeral` to apply to the peer entities (cursor positions,
    /// player info, lobby picks).
    pub ephemeral: Vec<(Ephemeral, PeerId)>,
    /// Host-only: `NetMsg::Sequenced` events the host just assigned a sequence
    /// number to, queued to be fed back through its own receive path so the
    /// host applies and records them in the same canonical order as everyone
    /// else. Drained at the top of `handle_socket` each frame.
    pub loopback: Vec<NetMsg>,
}

// -- NetPlugin --------------------------------------------------------------

/// Registers all networking-domain resources and systems: socket lifecycle,
/// message processing, cursor/lobby broadcast, game recording, and peer
/// management.
pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app
            // -- Resources ----------------------------------------------
            .insert_resource(NetState::default())
            .insert_resource(PendingEdits {
                // Random uid base: submission ids must be unique across app
                // instances for the whole session lifetime.
                next_uid: omdurman_net::new_seed(),
                ..Default::default()
            })
            .insert_resource(PendingIncoming::default())
            .insert_resource(CursorBroadcastTimer::default())
            .insert_resource(crate::peers::LocalPeer::default())
            .insert_resource(crate::peers::QueuedFactions::default())
            .insert_resource(crate::LocalFaction::default())
            .insert_resource(crate::LocalSpectator::default())
            .insert_resource(crate::LocalOptionalRule::default())
            .insert_resource(crate::LobbyScenario::default())
            .insert_resource(crate::lobby::RemoteScenario::default())
            .insert_resource(crate::events::PendingObservations::default())
            .insert_resource(TurnState::default())
            .insert_resource(crate::LobbyTab::default())
            // -- Startup ------------------------------------------------
            // Offline dev mode (OMDURMAN_OFFLINE): skip the matchbox socket and
            // self-host, so a single instance is authoritative and playable
            // without a signalling server (used for headless verification).
            .add_systems(Startup, open_socket.run_if(|| !offline_mode()))
            .add_systems(Startup, setup_offline.run_if(offline_mode))
            // -- Update -------------------------------------------------
            .add_systems(
                Update,
                (
                    sync_peer_entities.run_if(not(in_state(AppState::Spectating))),
                    apply_faction_bindings.after(sync_peer_entities),
                    crate::events::drain_observations.after(crate::net_socket::handle_socket),
                    apply_ephemeral
                        .after(crate::apply_pending_placement)
                        .after(sync_peer_entities),
                    crate::game_record::init_game_record.after(crate::net_socket::handle_socket),
                    crate::game_record::flush_game_record.after(crate::net_socket::handle_socket),
                    send_player_info_on_connect.after(crate::net_socket::handle_socket),
                    broadcast_cursor.run_if(crate::map_view_active),
                    // `flush_pending` conflicts with the whole receive chain on
                    // `ResMut<MatchboxSocket>` / the staging buffers, so pin it
                    // after the chain: the frame pipeline is then deterministic
                    // (receive + sequence -> route host submissions through
                    // loopback -> flush), instead of letting the scheduler pick
                    // an order that can defer the host's sequencing by a frame.
                    flush_pending.after(crate::net_socket::handle_reconnect),
                ),
            );
    }
}

/// Dev offline mode: `OMDURMAN_OFFLINE` set to any value. Skips the matchbox
/// socket so a single instance runs authoritatively without a signalling
/// server -- used for headless screenshot verification of play-view features.
pub(crate) fn offline_mode() -> bool {
    std::env::var("OMDURMAN_OFFLINE").is_ok()
}

/// Make this instance a self-contained host when offline: assign a fixed local
/// `PeerId` and mark it host, so events flow through the host loopback/apply
/// path with no peers. No socket is spawned, so all `MatchboxSocket` queries
/// simply no-op.
fn setup_offline(mut net: ResMut<NetState>) {
    net.my_id = Some(PeerId(uuid::Uuid::nil()));
    net.is_host = true;
    // Offline self-hosting counts as session evidence: solo play must be
    // able to sequence events (see `sequencing_allowed`).
    net.has_ever_peered = true;
    info!("offline mode: self-hosting without a matchbox socket");
}

pub(crate) fn broadcast_cursor(
    mut timer: ResMut<CursorBroadcastTimer>,
    time: Res<Time>,
    ground: Res<crate::picking::PointerGroundHit>,
    net: Res<NetState>,
    socket: Option<ResMut<MatchboxSocket>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    // `None` while the pointer is over UI or off the board: no cursor ping is
    // broadcast, so remote peers don't see a cursor hovering the sidebar.
    let Some(hit) = **ground else {
        return;
    };
    let Some(mut socket) = socket else {
        return;
    };
    omdurman_net::broadcast_unreliable(
        &mut socket,
        &net.peers,
        &NetMsg::Ephemeral(Ephemeral::CursorPos {
            pos: [hit.x, hit.z],
        }),
    );
}

/// Send our PlayerInfo to every connected peer once.
pub(crate) fn send_player_info_on_connect(
    net: Res<NetState>,
    local: Res<crate::settings::LocalPlayerSettings>,
    local_faction: Res<crate::LocalFaction>,
    local_spectator: Res<crate::LocalSpectator>,
    mut pending: ResMut<PendingEdits>,
    mut notified: Local<Vec<PeerId>>,
) {
    for &peer in &net.peers {
        if !notified.contains(&peer) {
            notified.push(peer);
            let (r, g, b) = local.color_u8();
            pending.outgoing_targeted.push((
                NetMsg::Ephemeral(Ephemeral::PlayerInfo {
                    name: local.name.clone(),
                    color: [r, g, b],
                }),
                peer,
            ));
            pending.outgoing_targeted.push((
                NetMsg::Ephemeral(Ephemeral::FactionChoice(local_faction.0)),
                peer,
            ));
            pending.outgoing_targeted.push((
                NetMsg::Ephemeral(Ephemeral::SpectatorChoice(local_spectator.0)),
                peer,
            ));
        }
    }
}

pub(crate) fn apply_ephemeral(
    mut incoming: ResMut<PendingIncoming>,
    mut commands: Commands,
    mut remote_scenario: ResMut<crate::lobby::RemoteScenario>,
    peers: crate::peers::PeerRouteQuery,
    mut event_viewer: Option<ResMut<crate::event_viewer::EventViewerState>>,
    time: Res<Time>,
) {
    // Index peer entities once so every ephemeral event below is an O(1)
    // lookup instead of a linear scan + re-`get()`.
    let by_id: std::collections::HashMap<PeerId, (Entity, Option<&PeerCursor>, Option<&PeerName>)> =
        peers.iter().map(|(e, k, c, n)| (k.0, (e, c, n))).collect();

    for (eph, peer) in incoming.ephemeral.drain(..) {
        match eph {
            Ephemeral::PlayerInfo {
                name,
                color: [cr, cg, cb],
            } => {
                if let Some(&(entity, _, _)) = by_id.get(&peer) {
                    commands.entity(entity).insert((
                        PeerName(name),
                        PeerColor(egui::Color32::from_rgb(cr, cg, cb)),
                    ));
                }
            }
            Ephemeral::CursorPos { pos: [cx, cy] } => {
                let Some(&(entity, cursor, _)) = by_id.get(&peer) else {
                    continue;
                };
                let pos = Vec2::new(cx, cy);
                let prev = cursor.and_then(|c| c.current).unwrap_or(pos);
                commands.entity(entity).insert(PeerCursor {
                    current: Some(pos),
                    previous: Some(prev),
                    last_update: time.elapsed_secs_f64(),
                    ..default()
                });
            }
            Ephemeral::EventViewerSelect(idx) => {
                if let Some(ref mut viewer) = event_viewer {
                    viewer.selected = if idx < 0 { None } else { Some(idx as usize) };
                }
            }
            Ephemeral::FactionChoice(faction) => {
                if let Some(&(entity, _, _)) = by_id.get(&peer) {
                    commands.entity(entity).insert(LobbyPick(faction));
                }
            }
            Ephemeral::ScenarioChoice(scenario) => {
                remote_scenario.0 = Some(scenario);
            }
            Ephemeral::SpectatorChoice(spectating) => {
                if let Some(&(entity, _, _)) = by_id.get(&peer) {
                    let mut entity_cmd = commands.entity(entity);
                    if spectating {
                        entity_cmd.insert(Spectator);
                        entity_cmd.remove::<LobbyPick>();
                    } else {
                        entity_cmd.remove::<Spectator>();
                    }
                }
            }
        }
    }
}

pub(crate) fn flush_pending(
    mut pending: ResMut<PendingEdits>,
    mut incoming: ResMut<PendingIncoming>,
    net: Res<NetState>,
    time: Res<Time>,
    mut socket: Option<ResMut<MatchboxSocket>>,
) {
    pending.retransmit_unconfirmed(time.delta_secs());

    if pending.outgoing_broadcast.is_empty() && pending.outgoing_targeted.is_empty() {
        return;
    }

    let broadcast_count = pending.outgoing_broadcast.len();
    let targeted_count = pending.outgoing_targeted.len();
    if broadcast_count > 0 || targeted_count > 0 {
        debug!(
            broadcast = broadcast_count,
            targeted = targeted_count,
            is_host = net.is_host,
            peers = net.peers.len(),
            "flushing pending outbound messages"
        );
    }

    let host = net.host_id();

    let staged: Vec<NetMsg> = std::mem::take(&mut pending.outgoing_broadcast);
    let mut to_broadcast: Vec<NetMsg> = Vec::new();
    let mut retained_broadcast: Vec<NetMsg> = Vec::new();

    for msg in staged {
        match msg {
            NetMsg::Game { uid, event } if net.is_host => {
                // The sequencer's *own* game events are not sequenced here.
                // They are looped back unsequenced so `handle_socket` assigns
                // their `seq` through the *same* arm that sequences guest
                // submissions -- a single serialization point. Assigning `seq`
                // in two systems made host-own vs guest-relayed ordering depend
                // on intra-frame system scheduling; routing both through
                // `handle_socket` removes that nondeterminism.
                incoming.loopback.push(NetMsg::Game { uid, event });
            }
            NetMsg::Game { uid, event } => {
                let submission = NetMsg::Game { uid, event };
                let sent = match (host, enc_msg(&submission), socket.as_deref_mut()) {
                    (Some(host), Some(encoded), Some(socket)) => socket
                        .channel_mut(CH_RELIABLE)
                        .try_send(encoded, host)
                        .inspect_err(|e| warn!(error = %e, "submit to host failed; will retry"))
                        .is_ok(),
                    _ => false,
                };
                if !sent {
                    retained_broadcast.push(submission);
                }
            }
            other => to_broadcast.push(other),
        }
    }

    let targeted: Vec<(NetMsg, PeerId)> = std::mem::take(&mut pending.outgoing_targeted);
    let mut retained_targeted: Vec<(NetMsg, PeerId)> = Vec::new();
    for (msg, peer) in targeted {
        let sent = match (enc_msg(&msg), socket.as_deref_mut()) {
            (Some(encoded), Some(socket)) => socket
                .channel_mut(CH_RELIABLE)
                .try_send(encoded, peer)
                .inspect_err(|e| warn!(error = %e, "reliable targeted send failed; will retry"))
                .is_ok(),
            _ => false,
        };
        if !sent {
            retained_targeted.push((msg, peer));
        }
    }

    for msg in to_broadcast {
        if net.peers.is_empty() {
            if !matches!(msg, NetMsg::Sequenced { .. }) {
                retained_broadcast.push(msg);
            }
            continue;
        }
        let Some(socket) = socket.as_deref_mut() else {
            retained_broadcast.push(msg);
            continue;
        };
        let Some(encoded) = enc_msg(&msg) else {
            // Encoding failed (OOM) -- retain for a later retry rather than
            // emit an empty packet that WebRTC may silently drop.
            retained_broadcast.push(msg);
            continue;
        };
        let channel = socket.channel_mut(CH_RELIABLE);
        let mut all_ok = true;
        for &peer in &net.peers {
            if let Err(e) = channel.try_send(encoded.clone(), peer) {
                warn!(error = %e, "reliable broadcast send failed; will retry");
                all_ok = false;
            }
        }
        if !all_ok {
            retained_broadcast.push(msg);
        }
    }

    pending.outgoing_broadcast = retained_broadcast;
    pending.outgoing_targeted = retained_targeted;
}

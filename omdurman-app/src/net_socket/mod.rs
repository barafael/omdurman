use crate::{
    AppState, GameStateParams, PendingEdits, PendingIncoming, ReconnectRoom, TurnState, game_apply,
    game_record, picker, rebuild_state_to,
};
use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use omdurman_net::{
    CH_RELIABLE, CH_UNRELIABLE, Control, GameEvent, NetMsg, NetState, RESYNC_BOOTSTRAP_SECS,
    RoomId, SEQ_STABILIZE_SECS, SequencedDelivery, decode,
};

mod reconnect;
mod resync;
mod sequencing;
#[cfg(test)]
mod tests;

use reconnect::auto_reconnect_on_stall;
pub(crate) use reconnect::handle_reconnect;
use resync::{install_foreign_history, retry_snapshot_request, serve_snapshot, stage_catch_up};
use sequencing::{
    drain_contiguous, expected_seq, has_session_evidence, host_handle_game, receive_sequenced,
};

/// What applying the host-sequenced stream reads and writes: the net state
/// and the outbound queue, the record, the engine and its projections, and
/// the app state a `StartGame` or an installed history moves.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct Session<'w> {
    pub net: ResMut<'w, NetState>,
    pub pending: ResMut<'w, PendingEdits>,
    pub turn: ResMut<'w, TurnState>,
    pub recorder: ResMut<'w, game_record::GameRecorder>,
    pub gsp: GameStateParams<'w>,
    pub state: Res<'w, State<AppState>>,
    pub next_state: ResMut<'w, NextState<AppState>>,
    /// This instance's stable player key (seat binding identity).
    pub local_key: Res<'w, crate::seats::LocalPlayerKey>,
}

pub struct NetSocketPlugin;

impl Plugin for NetSocketPlugin {
    fn build(&self, app: &mut App) {
        // Explicit ordering: `handle_socket` must drain the *current* socket
        // before `handle_reconnect` is allowed to reset `NetState` and swap the
        // socket resource. Without this, Bevy's scheduler is free to run them
        // in either order (they conflict on `ResMut<MatchboxSocket>` so they
        // serialize, but the order is unspecified). If `handle_reconnect`
        // runs first, it resets `net.my_id = None` synchronously while the
        // socket swap is still deferred; `handle_socket` then sees the *old*
        // socket, re-populates `my_id` with the stale id, and -- because the
        // my_id update only fires when it is `None` -- never adopts the new
        // socket's id. The local peer's `sorted_all` then disagrees with every
        // other peer's (they see the new id), and host election diverges.
        app.add_systems(
            Update,
            (
                handle_socket,
                retry_snapshot_request,
                auto_reconnect_on_stall,
                handle_reconnect,
            )
                .chain(),
        );
    }
}

/// Record and apply deliveries that [`receive_sequenced`] /
/// [`drain_contiguous`] released (already marked applied), in order.
fn apply_ready(s: &mut Session, ready: Vec<SequencedDelivery>) {
    for delivery in ready {
        let SequencedDelivery {
            seq,
            uid,
            event: ev,
            from,
        } = delivery;
        let sender_idx = s.net.sender_idx(from);
        s.recorder.push_event(&ev, sender_idx, seq, Some(uid));
        info!(seq, uid, "applied sequenced event");
        // Our own submission made it through the host: stop retransmitting it.
        s.pending.confirm(uid);
        // The board shows a review: the event, now recorded, reaches the live
        // state through `timeline::catch_up_live_stash` instead.
        if !s.gsp.reviewing() {
            apply_to_board(&mut s.gsp, &ev);
        }
        let GameEvent::StartGame {
            seats, scenario, ..
        } = &ev
        else {
            continue;
        };
        let first_start = !s.turn.game_started;
        s.turn.game_started = true;
        // The engine always takes the StartGame (the record is the state); only
        // the *view* switch is gated on the lobby, so a player browsing the menu
        // is not yanked.
        if *s.state.get() != AppState::Lobby {
            info!(%scenario, "StartGame applied; view unchanged (not in lobby)");
            continue;
        }
        // Switch the view to the game board, so play opens on the scenario's
        // board rather than whatever screen preceded the lobby. (The board data
        // loads from `pending_map_load`; the board picked follows the scenario
        // via `sync_board_to_game`.)
        s.gsp.next_app_mode.set(crate::AppMode::Game);
        s.next_state.set(AppState::InGame);
        info!(%scenario, "game started via host StartGame");
        // A guest without a seat is a spectator: request the full record from
        // the host so it converges to every unit already placed, not just
        // events seen after this point. (Seated guests are present from the
        // start, so they don't need it.) Seats are keyed by the stable player
        // key, never by the session `PeerId`.
        let locally_assigned = crate::seats::seat_of(seats, s.local_key.0).is_some();
        if first_start && !s.net.is_host && !locally_assigned && !s.net.snapshot_applied {
            info!("no faction assigned to this peer; requesting snapshot as spectator");
            s.net.request_history(false);
            if let Some(host) = s.net.host_id() {
                s.pending
                    .outgoing_targeted
                    .push((NetMsg::Control(Control::RequestSnapshot), host));
            }
        }
    }
}

/// Apply a sequenced event to the board's engine state, with what it did to
/// the counters (the board effects) and its observations (the UI).
fn apply_to_board(gsp: &mut GameStateParams, ev: &GameEvent) {
    // What the event does to the counters, for the board effects: diffed
    // around the apply, live only (a rebuild never comes through here).
    let before = (gsp.live_applied.is_some()
        && matches!(
            ev,
            GameEvent::Effect(_)
                | GameEvent::PlaceUnit { .. }
                | GameEvent::MoveUnit { .. }
                | GameEvent::RemoveUnit { .. }
        ))
    .then(|| crate::fx::LiveChange::snapshot(&gsp.game_state.0));
    let setup = gsp.game_state.0.phase == omdurman_rules::Phase::Setup;
    // The one application path (shared with replay): every variant reaches
    // the engine synchronously, in seq order.
    let accepted = game_apply::apply_game_event(ev, &mut gsp.sinks());
    if accepted && let Some(before) = before {
        let change = crate::fx::LiveChange::between(&before, &gsp.game_state.0, setup);
        if let Some(live) = gsp.live_applied.as_mut()
            && !change.is_empty()
        {
            live.0.push(change);
        }
    }
    gsp.pending_observations
        .0
        .extend(gsp.game_state.0.drain_observations());
}

/// What [`handle_socket`] remembers between frames: what it has already
/// reported (so each condition is logged once), and that the socket closed.
#[derive(Default)]
pub(crate) struct SocketMemo {
    /// Submissions held while sequencing is not allowed, each logged once.
    held_uids: std::collections::HashSet<u64>,
    /// The socket is closed (logged once).
    socket_lost: bool,
    /// Peers that sent something undecodable (logged once each).
    undecodable_from: std::collections::HashSet<PeerId>,
}

/// What [`poll_socket_roster`] saw change this frame.
struct RosterChange {
    peers_changed: bool,
    my_id_changed: bool,
    newly_connected: Vec<PeerId>,
    /// The socket did I/O this frame (it exists and is open).
    socket_live: bool,
}

/// Socket I/O: reconcile the peer roster and our own id, detect a closed or
/// unanswered signalling connection (offline self-host mode), and report what
/// changed this frame.
fn poll_socket_roster(
    socket: &mut Option<ResMut<MatchboxSocket>>,
    net: &mut NetState,
    socket_lost: &mut bool,
    commands: &mut Commands,
) -> RosterChange {
    let mut peers_changed = false;
    let mut my_id_changed = false;
    let mut newly_connected: Vec<PeerId> = Vec::new();
    // A closed socket (`try_update_peers` errs) does no I/O this frame; the
    // loopback path below still runs.
    let socket_live = match socket.as_deref_mut() {
        Some(socket) => match socket.try_update_peers() {
            Ok(peer_updates) => {
                for (peer, peer_state) in peer_updates {
                    match peer_state {
                        PeerState::Connected if !net.peers().contains(&peer) => {
                            net.add_peer(peer);
                            newly_connected.push(peer);
                            peers_changed = true;
                            info!(%peer, "peer connected");
                        }
                        PeerState::Disconnected => {
                            peers_changed |= net.remove_peer(peer);
                            net.chunks.forget(peer);
                            info!(%peer, "peer disconnected");
                        }
                        _ => {}
                    }
                }
                // Reconcile `my_id` with the socket's actual id. The socket
                // id is `None` until the signalling server assigns one, so we
                // only update when the socket reports `Some`. We update not
                // just on the first assignment but whenever the reported id
                // differs from what we have -- a reconnect swaps the
                // `MatchboxSocket` resource for a fresh one whose local id is
                // a brand-new UUID; if we kept the old id we'd never appear in
                // any other peer's roster and host election would diverge
                // (each peer computing a different `sorted_all` and so a
                // different lowest id).
                let socket_id = socket.id();
                my_id_changed = socket_id.is_some() && net.set_my_id(socket_id);
                true
            }
            Err(_) => false,
        },
        None => false,
    };
    // No signalling: the socket closed for good (matchbox ends it when the
    // server goes away with no peer connected), or the server has not
    // answered (no id) for `SIGNALLING_TIMEOUT_SECS` with nobody here. Say so
    // once; and with nobody connected this instance is offline -- it plays on
    // as its own host (a solo game against the AI must not freeze waiting for
    // an id the server may never assign). A late answer replaces the offline
    // id; reconnecting from the lobby builds a fresh socket.
    let closed = socket.is_some() && !socket_live;
    let unanswered = socket_live
        && net.my_id().is_none()
        && net.peers().is_empty()
        && net.election_stable_secs >= omdurman_net::SIGNALLING_TIMEOUT_SECS;
    if (closed || unanswered) && !net.rejoining {
        if !*socket_lost {
            *socket_lost = true;
            if net.peers().is_empty() {
                warn!(
                    closed,
                    "no signalling server connection and no peer connected; \
                     playing on offline (reconnect from the lobby to meet other players)"
                );
                // Leave the room for good: an offline game must not merge
                // into the room's session if the server answers later.
                commands.remove_resource::<MatchboxSocket>();
                if net.my_id().is_none() {
                    my_id_changed = net.set_my_id(Some(PeerId(uuid::Uuid::nil())));
                }
                // Offline self-hosting is session evidence (see
                // `sequencing_allowed`).
                net.has_ever_peered = true;
            } else {
                warn!(
                    peers = net.peers().len(),
                    "signalling connection closed; no further peers can join"
                );
            }
        }
    } else {
        *socket_lost = false;
    }
    RosterChange {
        peers_changed,
        my_id_changed,
        newly_connected,
        socket_live,
    }
}

/// Election clocks: run the stabilization window, and re-elect the host when
/// the peer-set view changed (§election stabilization).
fn update_election(net: &mut NetState, time: &Time, roster: &RosterChange, game_started: bool) {
    let changed = roster.peers_changed || roster.my_id_changed;
    net.resync_gate_secs = (net.resync_gate_secs - time.delta_secs()).max(0.0);
    if changed {
        // Peer-set view changed: the host-election stabilization window
        // restarts (see `SEQ_STABILIZE_SECS`).
        net.election_stable_secs = 0.0;
    } else {
        net.election_stable_secs += time.delta_secs();
    }
    if has_session_evidence(net) {
        net.has_ever_peered = true;
    }

    if let Some(my_id) = net.my_id()
        && changed
    {
        let new_host_is_me = net.sorted_all().first() == Some(&my_id);
        let promoted = new_host_is_me && !net.is_host;
        if promoted {
            // A freshly promoted host resumes the canonical numbering one
            // past the highest seq it applied (`next_host_seq` enforces the
            // same floor at every assignment, so this is belt and braces).
            // Deliveries buffered from the previous host can never be
            // completed now -- the new line is canonical by election.
            net.next_seq = expected_seq(net);
            net.reorder.clear();
            info!(
                next_seq = net.next_seq,
                game_started, "promoted to host; resumed sequence numbering"
            );
        }
        net.is_host = new_host_is_me;
    }
}

/// Drain the socket channels and turn raw bytes into a decoded message
/// stream: submissions held from earlier frames first (arrival order), then
/// this frame's channel traffic (chunks reassembled in position), then the
/// host's own loopback.
fn collect_decoded(
    socket_live: bool,
    socket: &mut Option<ResMut<MatchboxSocket>>,
    net: &mut NetState,
    incoming: &mut PendingIncoming,
    undecodable_from: &mut std::collections::HashSet<PeerId>,
    my_id: PeerId,
) -> Vec<(PeerId, NetMsg, bool)> {
    let mut received: Vec<(PeerId, Box<[u8]>)> = Vec::new();
    if socket_live && let Some(socket) = socket.as_deref_mut() {
        received.extend(socket.channel_mut(CH_RELIABLE).receive());
        received.extend(socket.channel_mut(CH_UNRELIABLE).receive());
    }

    // Host loopback: events the host sequenced for itself (below). They flow
    // through the identical apply path as remote `Sequenced` events so every
    // peer -- host included -- observes the same ordered stream. `my_id` is the
    // canonical "sender" for these.
    let loopback: Vec<NetMsg> = std::mem::take(&mut incoming.loopback);

    // Submissions held last frame come first, in the order they arrived:
    // whatever arrived since is younger. (Still held, they are re-held in
    // the same order; demoted, they are forwarded to the new host in it.)
    let mut decoded: Vec<(PeerId, NetMsg, bool)> = std::mem::take(&mut incoming.held)
        .into_iter()
        .map(|held| {
            (
                my_id,
                NetMsg::Game {
                    uid: held.uid,
                    event: held.event,
                },
                held.own,
            )
        })
        .collect();
    for (peer, raw) in received {
        match decode(&raw) {
            // A chunk is held until its message is complete (reassembled
            // in arrival position, so the channel's order is kept).
            Some(msg) => decoded.extend(net.chunks.receive(peer, msg).map(|m| (peer, m, false))),
            None => {
                // All peers must run the same build (the wire format is the
                // variant index): a peer that does not will send nothing we
                // can read. Say so once per peer, not once per message.
                if undecodable_from.insert(peer) {
                    warn!(%peer, "undecodable message from peer (a different build?); ignoring its messages");
                }
            }
        }
    }
    decoded.extend(loopback.into_iter().map(|msg| (my_id, msg, true)));
    decoded
}

/// The per-frame receive path: socket I/O (peer roster, election, inbound
/// messages) when a socket exists, then the loopback / sequencing / apply
/// path -- which runs *without* a socket too, so offline self-host mode
/// (`OfflineMode`) sequences and applies its own submissions.
pub(crate) fn handle_socket(
    mut socket: Option<ResMut<MatchboxSocket>>,
    mut s: Session,
    mut incoming: ResMut<PendingIncoming>,
    time: Res<Time>,
    mut commands: Commands,
    mut memo: Local<SocketMemo>,
) {
    let SocketMemo {
        held_uids,
        socket_lost,
        undecodable_from,
    } = &mut *memo;

    let roster = poll_socket_roster(&mut socket, &mut s.net, socket_lost, &mut commands);
    update_election(&mut s.net, &time, &roster, s.turn.game_started);

    // The lobby is entered voluntarily (via the mode picker), not
    // auto-triggered by peers appearing -- so a local editing session
    // is never dragged into someone else's game.

    // Message processing runs in both Lobby and InGame: the lobby needs to
    // receive faction picks, the host's `StartGame`, and snapshot replies.
    // An offline review (`Spectating` with no session underneath) owns the
    // world, so socket processing is suppressed; a review on top of a session
    // does not stop it (the events go to the stashed live state, see
    // `timeline`). During `Splash` there is no socket yet.
    if *s.state.get() == AppState::Splash
        || (*s.state.get() == AppState::Spectating && !s.gsp.reviewing())
    {
        return;
    }

    stage_catch_up(&mut s, &time, &roster.newly_connected);

    let is_host = s.net.is_host;
    let my_id = s.net.my_id().unwrap_or(PeerId(uuid::Uuid::nil()));
    let decoded = collect_decoded(
        roster.socket_live,
        &mut socket,
        &mut s.net,
        &mut incoming,
        undecodable_from,
        my_id,
    );

    for (peer, msg, own_loopback) in decoded {
        match msg {
            NetMsg::Game { uid, event: ev } => {
                if !is_host {
                    // We received an unsequenced submission but we don't believe
                    // we are the host -- most likely a transient election
                    // disagreement right after a peer connect/disconnect (the
                    // sender's view of the lowest PeerId briefly differs from
                    // ours). Dropping it would silently lose real player input,
                    // so re-forward it to whoever *we* currently consider the
                    // host. If we are in fact the host, the two views reconcile
                    // within a frame or two and the resend reaches us.
                    match s.net.host_id() {
                        Some(host) => {
                            warn!(
                                "received unsequenced Game event but we are not host; re-forwarding to current host"
                            );
                            s.pending
                                .outgoing_targeted
                                .push((NetMsg::Game { uid, event: ev }, host));
                        }
                        None => {
                            warn!(
                                "received unsequenced Game event but we are not host and no host is known; retaining for retry"
                            );
                            // Bounce it back onto our own outgoing broadcast so
                            // `flush_pending` re-submits once a host is known.
                            s.pending
                                .outgoing_broadcast
                                .push(NetMsg::Game { uid, event: ev });
                        }
                    }
                    continue;
                }
                host_handle_game(&mut s, &mut incoming, held_uids, uid, ev, own_loopback);
            }
            NetMsg::Sequenced {
                seq,
                uid,
                event: ev,
            } => {
                let ready = receive_sequenced(
                    &mut s.net,
                    &s.recorder,
                    &mut s.pending,
                    own_loopback,
                    SequencedDelivery {
                        seq,
                        uid,
                        event: ev,
                        from: peer,
                    },
                );
                apply_ready(&mut s, ready);
            }
            NetMsg::Ephemeral(eph) => {
                incoming.ephemeral.push((eph, peer));
            }
            NetMsg::Control(Control::RequestSnapshot) => {
                serve_snapshot(&mut s, peer);
            }
            NetMsg::Control(
                control @ (Control::SeatRequest { .. }
                | Control::SeatVoteOpen { .. }
                | Control::SeatVote { .. }
                | Control::SeatVoteClosed { .. }),
            ) => {
                // Seat claims and votes: handled by `seat_arbiter`.
                incoming.seat_control.push((control, peer));
            }
            NetMsg::Control(Control::Chunk { .. }) => {
                // Reassembled above (`ChunkAssembler::receive` never passes
                // a chunk through).
            }
            NetMsg::Control(Control::GameHistory(record)) => {
                install_foreign_history(&mut s, &mut commands, record);
            }
        }
    }
}

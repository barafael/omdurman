use crate::{
    AppState, GameStateParams, PendingEdits, PendingIncoming, ReconnectRoom, TurnState, game_apply,
    game_record, picker, rebuild_state_to, timeline::RebuildState,
};
use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use omdurman_net::{
    CH_RELIABLE, CH_UNRELIABLE, Control, GameEvent, NetMsg, NetState, RESYNC_BOOTSTRAP_SECS,
    RoomId, SEQ_STABILIZE_SECS, decode,
};

/// Host sequencing is only allowed once the election has settled: the peer
/// set must have been unchanged for [`SEQ_STABILIZE_SECS`], and this peer
/// must have actual session evidence (it has seen peers, or runs in offline
/// self-host mode). See `NetState::election_stable_secs` /
/// `NetState::has_ever_peered` for why.
fn sequencing_allowed(net: &NetState) -> bool {
    // A reconnected peer must resync before resuming host authority; the
    // gate lifts on history install or after the bootstrap budget.
    let resynced = net.resync_gate_secs <= 0.0;
    (net.has_ever_peered || crate::net_plugin::offline_mode())
        && resynced
        && net.election_stable_secs >= SEQ_STABILIZE_SECS
}

#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct SocketContext<'w> {
    pub incoming: ResMut<'w, PendingIncoming>,
    pub recorder: ResMut<'w, game_record::GameRecorder>,
}

/// Bundle of the reconnect room resource + the room id so [`handle_reconnect`]
/// stays under the system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct ReconnectInfo<'w> {
    pub reconnect: Option<ResMut<'w, ReconnectRoom>>,
    pub room: ResMut<'w, RoomId>,
}

/// Bundle of the net-side state reset by [`handle_reconnect`] (recorder + next
/// app state) so the system stays under the parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct NetResetState<'w> {
    pub traffic: NetTraffic<'w>,
    pub recorder: ResMut<'w, game_record::GameRecorder>,
    pub next_state: ResMut<'w, NextState<AppState>>,
}

/// Bundle of `PendingEdits` + `NetState` + `TurnState` -- the network-side
/// buffers and turn counter that [`handle_reconnect`] resets and
/// [`handle_socket`] drains each frame. [`handle_reconnect`] additionally takes
/// `PendingIncoming` separately (it does not also use [`SocketContext`], so
/// there is no double-borrow).
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct NetTraffic<'w> {
    pub net: ResMut<'w, NetState>,
    pub pending: ResMut<'w, PendingEdits>,
    pub turn: ResMut<'w, TurnState>,
    /// Frame clock for the election-stability window.
    pub time: Res<'w, Time>,
}

/// Bundle of the picker + picker-state + placed-unit query used by
/// [`handle_reconnect`] to reset placement after reopening the socket.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PickerResetState<'w, 's> {
    pub picker: ResMut<'w, picker::UnitPicker>,
    pub picker_state: ResMut<'w, picker::PickerState>,
    pub placed_unit_q: Query<'w, 's, Entity, With<picker::PlacedUnit>>,
}

/// Bundle of the live `AppState` (read) and its `NextState` (write) used by
/// [`handle_socket`] to transition into `InGame`/`Spectating`.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct AppStateShift<'w> {
    pub state: Res<'w, State<AppState>>,
    pub next_state: ResMut<'w, NextState<AppState>>,
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

pub(crate) fn handle_reconnect(
    mut commands: Commands,
    reconnect: ReconnectInfo,
    net_state: NetResetState,
    picker: PickerResetState,
    mut incoming: ResMut<PendingIncoming>,
    socket: Option<Res<MatchboxSocket>>,
) {
    let ReconnectInfo {
        reconnect,
        mut room,
    } = reconnect;
    let NetResetState {
        traffic,
        mut recorder,
        mut next_state,
    } = net_state;
    let NetTraffic {
        mut net,
        mut turn,
        mut pending,
        ..
    } = traffic;
    let PickerResetState {
        mut picker,
        mut picker_state,
        placed_unit_q,
    } = picker;
    let Some(reconnect) = reconnect else { return };
    let new_room = reconnect.0.clone();

    if new_room.is_empty() {
        commands.remove_resource::<ReconnectRoom>();
        return;
    }

    info!(%new_room, "reconnecting");

    // -- despawn old socket --
    if socket.is_some() {
        commands.remove_resource::<MatchboxSocket>();
    }

    // -- reset state --
    *net = NetState::default();
    *turn = TurnState::default();
    pending.outgoing_broadcast.clear();
    pending.outgoing_targeted.clear();
    pending.stall_secs = 0.0;
    // Our record is wiped below: request the canonical history on reconnect
    // (any peer with a record may serve it) and block host authority until it
    // is installed or the bootstrap budget expires.
    net.needs_snapshot = true;
    net.resync_gate_secs = RESYNC_BOOTSTRAP_SECS;
    incoming.live.clear();
    incoming.replay.clear();
    incoming.ephemeral.clear();
    incoming.loopback.clear();
    *recorder = game_record::GameRecorder::default();

    // -- despawn placed units and restore full picker roster --
    for entity in &placed_unit_q {
        commands.entity(entity).despawn();
    }
    picker.reset_available();
    *picker_state = picker::PickerState::Idle;

    // -- update room id and URL --
    *room = RoomId::new(new_room.clone());

    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return;
        };
        if let Ok(history) = window.history() {
            let href = window.location().href().ok().unwrap_or_default();
            if let Ok(url) = web_sys::Url::new(&href) {
                url.search_params().set("room", &new_room);
                let _ = history.replace_state_with_url(
                    &wasm_bindgen::JsValue::NULL,
                    "",
                    Some(&url.href()),
                );
            }
        }
    }

    // -- open new socket --
    commands.insert_resource(omdurman_net::build_socket(&new_room));

    // -- return to the lobby so the player can re-pick faction / scenario --
    // The socket is fresh; the lobby renders immediately and `handle_socket`
    // processes peer/list state next frame.
    next_state.set(AppState::Lobby);

    commands.remove_resource::<ReconnectRoom>();
}

/// Safety net for a silently dead submission path: when our own submissions
/// stay unconfirmed for [`SUBMIT_STALL_RECONNECT_SECS`] while a game is
/// running, rebuild the socket through the standard `handle_reconnect` path
/// (same room). Everything survives the reset except the record, which is
/// re-downloaded from the host on reconnect; unconfirmed submissions are
/// retransmitted afterwards. Without this, a guest whose channel to the host
/// died without a disconnect event would sit frozen forever: every
/// retransmission and snapshot request travels the same dead link.
pub(crate) fn auto_reconnect_on_stall(
    mut commands: Commands,
    pending: Res<PendingEdits>,
    net: Res<NetState>,
    room: Res<RoomId>,
    state: Res<State<AppState>>,
    reconnect_pending: Option<Res<ReconnectRoom>>,
) {
    if pending.stall_secs < crate::net_plugin::SUBMIT_STALL_RECONNECT_SECS {
        return;
    }
    if crate::net_plugin::offline_mode() || !net.has_ever_peered {
        return;
    }
    if !matches!(state.get(), AppState::InGame) {
        return;
    }
    // Only trigger once per stall: a reconnect already pending is visible
    // via the resource; `handle_reconnect` resets `stall_secs` with the
    // rest of the net state.
    if reconnect_pending.is_some() {
        return;
    }
    warn!(
        stall_secs = pending.stall_secs,
        pending = pending.unconfirmed.len(),
        "submissions stalled unconfirmed; rebuilding socket to resync"
    );
    commands.insert_resource(ReconnectRoom(room.as_str().to_owned()));
}

pub(crate) fn retry_snapshot_request(
    time: Res<Time>,
    mut net: ResMut<NetState>,
    mut pending: ResMut<PendingEdits>,
) {
    if net.needs_snapshot {
        net.snapshot_retry_timer += time.delta_secs_f64();
        if net.snapshot_retry_timer > 2.0 {
            net.snapshot_retry_timer = 0.0;
            info!("guest: retrying snapshot request");
            pending
                .outgoing_broadcast
                .push(NetMsg::Control(Control::RequestSnapshot));
        }
    }
}

pub(crate) fn handle_socket(
    socket: Option<ResMut<MatchboxSocket>>,
    traffic: NetTraffic,
    app_state: AppStateShift,
    mut commands: Commands,
    mut gsp: GameStateParams,
    peers: crate::peers::Peers,
    mut ctx: SocketContext,
) {
    let NetTraffic {
        mut net,
        mut pending,
        mut turn,
        time,
    } = traffic;
    let AppStateShift {
        state,
        mut next_state,
    } = app_state;
    let Some(mut socket) = socket else {
        return;
    };

    let Ok(peer_updates) = socket.try_update_peers() else {
        return;
    };
    let mut peers_changed = false;
    let mut newly_connected: Vec<PeerId> = Vec::new();
    for (peer, peer_state) in peer_updates {
        match peer_state {
            PeerState::Connected if !net.peers.contains(&peer) => {
                net.peers.push(peer);
                newly_connected.push(peer);
                peers_changed = true;
                info!(%peer, "peer connected");
            }
            PeerState::Disconnected => {
                let before = net.peers.len();
                net.peers.retain(|&p| p != peer);
                peers_changed |= net.peers.len() != before;
                info!(%peer, "peer disconnected");
            }
            _ => {}
        }
    }

    // Reconcile `my_id` with the socket's actual id. The socket id is `None`
    // until the signalling server assigns one, so we only update when the
    // socket reports `Some`. We update not just on the first assignment but
    // whenever the reported id differs from what we have -- a reconnect swaps
    // the `MatchboxSocket` resource for a fresh one whose local id is a brand-
    // new UUID; if we kept the old id we'd never appear in any other peer's
    // roster and host election would diverge (each peer computing a different
    // `sorted_all` and so a different lowest id).
    let socket_id = socket.id();
    let my_id_changed = socket_id.is_some_and(|id| Some(id) != net.my_id);
    if my_id_changed {
        net.my_id = socket_id;
    }
    net.resync_gate_secs = (net.resync_gate_secs - time.delta_secs()).max(0.0);
    if peers_changed || my_id_changed {
        net.refresh_sorted();
        // Peer-set view changed: the host-election stabilization window
        // restarts (see `SEQ_STABILIZE_SECS`).
        net.election_stable_secs = 0.0;
    } else {
        net.election_stable_secs += time.delta_secs();
    }
    if !net.peers.is_empty() {
        net.has_ever_peered = true;
    }

    if let Some(my_id) = net.my_id
        && (peers_changed || my_id_changed)
    {
        let new_host_is_me = net.sorted_all().first() == Some(&my_id);
        let promoted = new_host_is_me && !net.is_host;
        if turn.game_started && promoted {
            // A freshly promoted host must resume the canonical sequence
            // numbering where the previous host left off. `next_seq` was only
            // ever incremented on whoever was host, so on a guest it is still 0;
            // adopting it as-is would re-issue sequence numbers that already
            // exist, and the receive-side dedup (`last_applied_seq`) would then
            // silently drop those *new* events -- a permanent desync. Every peer
            // tracks `last_applied_seq`, so it is the correct baseline: the next
            // seq to assign is one past the highest this peer has applied.
            net.next_seq = net.last_applied_seq.map_or(0, |s| s + 1);
            info!(
                next_seq = net.next_seq,
                "promoted to host after previous host disconnect; resumed sequence numbering"
            );
        }
        net.is_host = new_host_is_me;
    }

    // The lobby is entered voluntarily (via the mode picker), not
    // auto-triggered by peers appearing -- so a local editing session
    // is never dragged into someone else's game.

    // Message processing runs in both Lobby and InGame: the lobby needs to
    // receive faction picks, the host's `StartGame`, and snapshot replies.
    // In `Spectating` the timeline owns the world (rebuilt from a record, no
    // live peer), so socket processing is suppressed. During `Splash` there is
    // no socket yet.
    if matches!(*state.get(), AppState::Splash | AppState::Spectating) {
        return;
    }

    let mut targeted: Vec<(NetMsg, PeerId)> = Vec::new();
    let mut sequenced_out: Vec<NetMsg> = Vec::new();

    // Host: proactively push the canonical record to any peer that just
    // connected while a game is in progress. This catches up both a fresh late
    // joiner and -- crucially -- a peer that dropped and reconnected during a
    // WebRTC blip (matchbox reconnects it automatically, but it silently missed
    // every `Sequenced` event sent while it was gone). The receiver only
    // replays a record that is ahead of its local state, so a peer that never
    // fell behind ignores it. This makes reconnection self-healing rather than
    // relying on the joiner noticing it is behind.
    if net.is_host
        && turn.game_started
        && !newly_connected.is_empty()
        && let Some(ref record) = ctx.recorder.record
        && !record.events.is_empty()
    {
        for peer in newly_connected {
            info!(%peer, "host: pushing game history to (re)connected peer");
            targeted.push((NetMsg::Control(Control::GameHistory(record.clone())), peer));
            if !net.snapshot_pending.contains(&peer) {
                net.snapshot_pending.push(peer);
            }
        }
    }

    let reliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_RELIABLE).receive();
    let unreliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_UNRELIABLE).receive();
    let is_host = net.is_host;

    // Host loopback: events the host sequenced for itself (below). They flow
    // through the identical apply path as remote `Sequenced` events so every
    // peer -- host included -- observes the same ordered stream. `my_id` is the
    // canonical "sender" for these.
    let my_id = net.my_id.unwrap_or(PeerId(uuid::Uuid::nil()));
    let loopback: Vec<(PeerId, NetMsg)> = ctx
        .incoming
        .loopback
        .drain(..)
        .map(|msg| (my_id, msg))
        .collect();

    let decoded = reliable
        .into_iter()
        .chain(unreliable)
        .filter_map(|(peer, raw)| match decode(&raw) {
            Some(msg) => Some((peer, msg)),
            None => {
                warn!("unknown message, ignoring");
                None
            }
        })
        .chain(loopback);

    for (peer, msg) in decoded {
        let sender_idx = net.sender_idx(peer);
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
                    match net.host_id() {
                        Some(host) => {
                            warn!(
                                "received unsequenced Game event but we are not host; re-forwarding to current host"
                            );
                            targeted.push((NetMsg::Game { uid, event: ev }, host));
                        }
                        None => {
                            warn!(
                                "received unsequenced Game event but we are not host and no host is known; retaining for retry"
                            );
                            // Bounce it back onto our own outgoing broadcast so
                            // `flush_pending` re-submits once a host is known.
                            pending
                                .outgoing_broadcast
                                .push(NetMsg::Game { uid, event: ev });
                        }
                    }
                    continue;
                }
                // Host-side idempotency: a retransmission of an event we
                // already sequenced -- recorded canonically, or still in
                // flight in this frame's batch -- is re-echoed with its
                // existing seq instead of being sequenced twice.
                let recorded_seq = ctx.recorder.record.as_ref().and_then(|r| {
                    r.events
                        .iter()
                        .rev()
                        .find(|e| e.uid == Some(uid))
                        .map(|e| e.seq)
                });
                let in_flight_seq = sequenced_out.iter().find_map(|m| match m {
                    NetMsg::Sequenced { seq, uid: u, .. } if *u == uid => Some(*seq),
                    _ => None,
                });
                if let Some(seq) = recorded_seq.or(in_flight_seq) {
                    debug!(
                        seq,
                        "host: retransmission of an already-sequenced event; re-echoing"
                    );
                    let sequenced = NetMsg::Sequenced {
                        seq,
                        uid,
                        event: ev,
                    };
                    sequenced_out.push(sequenced.clone());
                    ctx.incoming.loopback.push(sequenced);
                    continue;
                }
                if !sequencing_allowed(&net) {
                    // The peer set may still be forming: sequencing now could
                    // collide with a peer that also (briefly) believes itself
                    // host. Hold the submission; it is retried next frame.
                    debug!("host: holding submission until the election is stable");
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game { uid, event: ev });
                    continue;
                }
                let seq = net.next_seq;
                net.next_seq += 1;
                let sequenced = NetMsg::Sequenced {
                    seq,
                    uid,
                    event: ev,
                };
                sequenced_out.push(sequenced.clone());
                // Push the echo onto our own loopback queue. It is *not* applied
                // here: this `for` loop is already iterating `decoded`, which was
                // chained from `loopback.drain(..)` *above*. The just-pushed echo
                // won't be seen until the next call to this system (next frame),
                // so the host applies its own sequenced events one frame later
                // than a guest that receives the broadcast `Sequenced` echo. This
                // is intentional -- it keeps every peer (host included) on the
                // identical apply-on-echo path instead of special-casing the host
                // to apply inline.
                ctx.incoming.loopback.push(sequenced);
            }
            NetMsg::Sequenced {
                seq,
                uid,
                event: ev,
            } => {
                // Identity dedup: the same event sequenced twice under
                // different seq numbers (transient dual-host streams, or a
                // stale stream meeting a fresh host) must still be applied
                // exactly once.
                if !net.recent_uids.insert(uid) {
                    debug!(
                        seq,
                        "dropping sequenced delivery of an already-applied event"
                    );
                    continue;
                }
                // Apply each sequence number exactly once. The reliable channel
                // is ordered and `seq` is monotonic, so any `seq` at or below
                // the highest already applied is a duplicate delivery -- drop it
                // so its effect (and any state it mutates, e.g. mp_spent) is not
                // applied twice. (push_event already dedups *recording*; this
                // extends the same guarantee to *application*.) A delivery at
                // an already-used seq carrying a *different* event -- or *no*
                // local event at all (our watermark sits on a stale,
                // higher-numbered rogue line) -- is a conflict: on the
                // canonical line the record is contiguous up to the watermark,
                // so any mismatch proves the local record divergent.
                if let Some(last) = net.last_applied_seq {
                    if seq <= last {
                        if ctx
                            .recorder
                            .event_at_seq(seq)
                            .is_none_or(|recorded| recorded.payload != ev)
                        {
                            warn!(
                                seq,
                                "SEQ CONFLICT: canonical delivery disagrees with local record"
                            );
                            if !net.is_host {
                                // We are not the elected host, so our record
                                // may be the wrong one: force-install the
                                // canonical history.
                                net.force_install_history = true;
                                net.needs_snapshot = true;
                                net.snapshot_retry_timer = 0.0;
                            }
                        }
                        continue;
                    }
                    if seq > last + 1 {
                        // Gap: events between `last` and `seq` never reached
                        // us (e.g. broadcasts racing a reconnecting data
                        // channel). Apply what arrived, but request the
                        // canonical history and force-install it -- the local
                        // record is known to be incomplete, so the
                        // "install only if ahead" check must not apply. The
                        // host is exempt: its own line is canonical by
                        // election, and force-installing a shorter foreign
                        // history over it would regress every guest.
                        warn!(seq, last, "seq gap detected; requesting canonical history");
                        if net.is_host {
                            // A foreign stream jumping past our watermark is
                            // a dual-host artifact: ignore it entirely (our
                            // own line is canonical by election).
                            continue;
                        }
                        net.needs_snapshot = true;
                        net.snapshot_retry_timer = 0.0;
                        net.force_install_history = true;
                    }
                }
                net.last_applied_seq = Some(seq);
                ctx.recorder.push_event(&ev, sender_idx, seq, Some(uid));
                // Our own submission made it through the host: stop
                // retransmitting it.
                pending.confirm(uid);
                match &ev {
                    GameEvent::PlaceUnit { .. }
                    | GameEvent::MoveUnit { .. }
                    | GameEvent::RemoveUnit { .. } => {
                        ctx.incoming.live.push((ev, peer, sender_idx));
                    }
                    GameEvent::StartGame {
                        assignments,
                        scenario,
                        optional_rule,
                    } => {
                        if *state.get() != AppState::Lobby {
                            info!(%scenario, "ignoring StartGame; not in lobby");
                        } else {
                            // Shared live/replay core: stage the binding, seed
                            // the engine state (+ optional rule), attach the
                            // board synchronously, defer the visual map load.
                            game_apply::apply_start_game(
                                assignments,
                                *scenario,
                                *optional_rule,
                                Some(&mut gsp.game_state.0),
                                &mut gsp.queued_factions,
                                &gsp.loaded_annotations,
                                &mut gsp.pending_map_load,
                            );
                            // Switch the view to the game board, so play opens on
                            // the scenario's board rather than whatever screen
                            // preceded the lobby. (The board data loads from
                            // `pending_map_load`; the board picked follows the
                            // scenario via `sync_edit_board_to_mode`.)
                            gsp.next_app_mode.set(crate::AppMode::Game);
                            if !turn.game_started {
                                turn.game_started = true;
                                next_state.set(AppState::InGame);
                                info!(%scenario, "game started via host StartGame");
                                // A guest that wasn't assigned a faction is a
                                // spectator: request the full record from the host
                                // so it converges to every unit already placed,
                                // not just events seen after this point. (Playing
                                // guests are assigned and present from the start,
                                // so they don't need it.)
                                if !is_host && peers.local().is_none() && !net.snapshot_applied {
                                    net.needs_snapshot = true;
                                    net.snapshot_retry_timer = 0.0;
                                    if let Some(host) = net.host_id() {
                                        targeted.push((
                                            NetMsg::Control(Control::RequestSnapshot),
                                            host,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        let mut apply_ctx = game_apply::GameApplyCtx {
                            game_state: Some(&mut gsp.game_state.0),
                        };
                        game_apply::apply_game_event(&ev, &mut apply_ctx);
                        for obs in gsp.game_state.0.drain_observations() {
                            gsp.pending_observations.0.push(obs);
                        }
                    }
                }
            }
            NetMsg::Ephemeral(eph) => {
                ctx.incoming.ephemeral.push((eph, peer));
            }
            NetMsg::Control(Control::RequestSnapshot) => {
                // Single-source installs: only the elected host serves
                // arbitrary requesters -- and, for the reconnected-host
                // deadlock (its record was wiped by its own reconnect while
                // the superior line lives on the guests), a guest serves its
                // *own host*. Anyone else serving would let rogue lines
                // masquerade as canonical. Installs stay guarded by the
                // ahead check on the receiving side.
                let requester_is_my_host = Some(peer) == net.host_id();
                if !is_host && !requester_is_my_host {
                    continue;
                }
                info!("late joiner requested game history");
                if turn.game_started
                    && let Some(ref record) = ctx.recorder.record
                    && !record.events.is_empty()
                {
                    targeted.push((NetMsg::Control(Control::GameHistory(record.clone())), peer));
                    net.snapshot_pending.push(peer);
                }
            }
            NetMsg::Control(Control::SnapshotReceived) => {
                info!("host: late joiner acknowledged game history");
                net.snapshot_pending.retain(|&p| p != peer);
            }
            NetMsg::Control(Control::GameHistory(record)) => {
                // Accept a record only if it carries events we haven't applied
                // yet. This covers two cases with one rule:
                //   * fresh late joiner (`last_applied_seq == None`): always
                //     ahead, so replay it;
                //   * reconnecting peer that fell behind during a WebRTC blip:
                //     `snapshot_applied` is already `true` from its first join,
                //     but the host's record now has a higher max seq than we
                //     applied, so we resync to catch up.
                // A record whose highest seq we've already applied is a genuine
                // duplicate (two snapshots racing) -- ignore it. A detected seq
                // conflict or gap flips `force_install_history`: our own record
                // is then known to be wrong, so the canonical history wins even
                // if it is not ahead by seq alone.
                let record_max = record.events.iter().map(|e| e.seq).max();
                let ahead = match (record_max, net.last_applied_seq) {
                    (Some(hi), Some(applied)) => hi > applied,
                    (Some(_), None) => true,
                    (None, _) => false,
                };
                if !ahead && !net.force_install_history {
                    info!("ignoring game history that is not ahead of local state");
                    continue;
                }
                net.snapshot_applied = true;
                net.needs_snapshot = false;
                net.snapshot_retry_timer = 0.0;
                net.force_install_history = false;
                net.resync_gate_secs = 0.0;
                info!(
                    "received game history ({} events), replaying to resync",
                    record.events.len()
                );
                targeted.push((NetMsg::Control(Control::SnapshotReceived), peer));
                ctx.recorder.install_history(record.clone());
                // The snapshot already includes every event up to the highest
                // recorded seq; mark them applied so a live `Sequenced` echo of
                // an event also present in the snapshot isn't applied a second
                // time (only seqs above the watermark are new to this joiner).
                net.last_applied_seq = record.events.iter().map(|e| e.seq).max();
                {
                    let mut state = RebuildState {
                        commands: &mut commands,
                        game_map: &mut gsp.game_map,
                        replay: &mut ctx.incoming.replay,
                        game_state: &mut gsp.game_state.0,
                        queued_factions: &mut gsp.queued_factions,
                        loaded_annotations: &mut gsp.loaded_annotations,
                        pending_map_load: &mut gsp.pending_map_load,
                    };
                    rebuild_state_to(&record, None, peer, &mut state);
                }
                // A mid-game reconnect lands in the Lobby (handle_reconnect
                // reset the app state); the rebuilt record proves the game had
                // started, so return straight to the board instead of leaving
                // the player staring at a lobby whose StartGame is history.
                if record
                    .events
                    .iter()
                    .any(|e| matches!(e.payload, GameEvent::StartGame { .. }))
                {
                    gsp.next_app_mode.set(crate::AppMode::Game);
                    if *state.get() == AppState::Lobby {
                        next_state.set(AppState::InGame);
                    }
                }
            }
        }
    }
    for (msg, peer) in targeted {
        pending.outgoing_targeted.push((msg, peer));
    }
    for msg in sequenced_out {
        pending.outgoing_broadcast.push(msg);
    }
}

use crate::{
    AppState, GameStateParams, PendingEdits, PendingIncoming, ReconnectRoom, TurnState, game_apply,
    game_record, picker, rebuild_state_to, timeline::RebuildState,
};
use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use omdurman_net::{
    CH_RELIABLE, CH_UNRELIABLE, Control, GameEvent, NetMsg, NetState, RESYNC_BOOTSTRAP_SECS,
    RoomId, SEQ_STABILIZE_SECS, SequencedDelivery, decode,
};

/// Host sequencing is only allowed once the election has settled: the peer
/// set must have been unchanged for [`SEQ_STABILIZE_SECS`], and this peer
/// must have actual session evidence (it has seen peers, or runs in offline
/// self-host mode, which sets `has_ever_peered` at startup). See
/// `NetState::election_stable_secs` / `NetState::has_ever_peered` for why.
fn sequencing_allowed(net: &NetState) -> bool {
    // A reconnected peer must resync before resuming host authority; the
    // gate lifts on history install or after the bootstrap budget.
    let resynced = net.resync_gate_secs <= 0.0;
    net.has_ever_peered && resynced && net.election_stable_secs >= SEQ_STABILIZE_SECS
}

/// The next seq this peer expects to apply: one past its watermark.
fn expected_seq(net: &NetState) -> u32 {
    net.last_applied_seq.map_or(0, |s| s + 1)
}

/// The seq a host assigns to the next submission. `next_seq` runs ahead of
/// the watermark by the host's own in-flight echoes (sequenced this frame,
/// applied via loopback next frame); the watermark floor makes the choice
/// robust against a stale `next_seq` -- e.g. a peer promoted to host whose
/// counter was never advanced -- which would otherwise re-issue seqs that
/// already exist and have every guest dedup the new events away.
fn next_host_seq(net: &NetState) -> u32 {
    net.next_seq.max(expected_seq(net))
}

/// Receive-side gate for one `NetMsg::Sequenced` delivery: decide whether it
/// is applied now, buffered, or dropped, and return the deliveries that are
/// ready to apply, in order (the delivery itself, then any contiguous run it
/// unblocked from the reorder buffer). Every returned delivery is already
/// *marked* applied here -- watermark advanced, uid remembered -- and the
/// caller must record + apply exactly these.
///
/// Crucially, the uid is remembered only at the apply point: a delivery
/// dropped as a seq conflict or an ignored gap never reaches the engine, so
/// it must not poison `recent_uids` -- otherwise the (re-)sequenced echo of
/// the same submission would later be identity-dropped as "already applied",
/// leaving a permanent hole in the host's own line.
///
/// `own_loopback` marks the host's own echo (fed back through
/// `PendingIncoming::loopback`), as opposed to a delivery from another peer.
pub(crate) fn receive_sequenced(
    net: &mut NetState,
    recorder: &game_record::GameRecorder,
    pending: &mut PendingEdits,
    own_loopback: bool,
    delivery: SequencedDelivery,
) -> Vec<SequencedDelivery> {
    let SequencedDelivery { seq, uid, .. } = delivery;
    // Identity dedup: the same event sequenced twice under different seq
    // numbers (transient dual-host streams, or a stale stream meeting a
    // fresh host) must still be applied exactly once. It *was* applied, so
    // our own submission is confirmed -- otherwise we would retransmit it
    // forever.
    if net.recent_uids.contains(uid) {
        info!(
            seq,
            uid,
            last_applied = ?net.last_applied_seq,
            "dropping sequenced delivery of an already-applied event"
        );
        pending.confirm(uid);
        return Vec::new();
    }
    let expected = expected_seq(net);
    // Apply each sequence number exactly once. The reliable channel is
    // ordered and `seq` is monotonic, so any `seq` below the next expected
    // one is a duplicate delivery -- drop it so its effect is not applied
    // twice. A delivery at an already-used seq carrying a *different* event
    // -- or *no* local event at all (our watermark sits on a stale,
    // higher-numbered rogue line) -- is a conflict: on the canonical line
    // the record is contiguous up to the watermark, so any mismatch proves
    // the local record divergent.
    if seq < expected {
        if recorder
            .event_at_seq(seq)
            .is_none_or(|recorded| recorded.payload != delivery.event)
        {
            warn!(
                seq,
                uid, "SEQ CONFLICT: canonical delivery disagrees with local record"
            );
            if !net.is_host {
                // We are not the elected host, so our record may be the
                // wrong one: force-install the canonical history.
                net.force_install_history = true;
                net.needs_snapshot = true;
                net.snapshot_retry_timer = 0.0;
            }
        } else {
            // Our own submission, re-echoed at its recorded seq:
            // application is deduped away, but the confirmation must not be.
            info!(
                seq,
                uid, "stale seq delivery matches local record; re-confirming uid"
            );
            pending.confirm(uid);
        }
        return Vec::new();
    }
    if seq > expected {
        if net.is_host {
            if !own_loopback {
                // A foreign stream jumping past our watermark is a dual-host
                // artifact: ignore it (our own line is canonical by
                // election). Nothing is remembered, so a later legitimate
                // sequencing of the same submission still applies.
                info!(
                    seq,
                    expected, uid, "host: ignoring foreign sequenced gap as dual-host artifact"
                );
                return Vec::new();
            }
            // Our *own* line skips seqs (an earlier own echo was dropped).
            // Nobody will ever fill the hole -- we are the sequencer -- so
            // waiting would freeze every future event. Apply over the hole;
            // guests heal via the reorder timeout + history install.
            warn!(
                seq,
                expected, uid, "host: own sequenced line skips seqs; applying over the hole"
            );
        } else {
            // Gap: events between the watermark and `seq` have not reached
            // us yet (e.g. broadcasts racing a reconnecting data channel, or
            // a late joiner whose history has not arrived). Applying `seq`
            // now would run it against a state missing those events: buffer
            // it until the gap fills, or until the timeout in
            // `handle_socket` requests the canonical history.
            warn!(
                seq,
                expected,
                uid,
                buffered = net.reorder.len(),
                "seq gap detected; buffering out-of-order delivery"
            );
            if !net.reorder.insert(delivery) {
                warn!(seq, uid, "reorder buffer full; dropping delivery");
            }
            return Vec::new();
        }
    }
    mark_applied(net, &delivery);
    let mut ready = vec![delivery];
    ready.extend(drain_contiguous(net, pending));
    ready
}

fn mark_applied(net: &mut NetState, delivery: &SequencedDelivery) {
    net.recent_uids.insert(delivery.uid);
    net.last_applied_seq = Some(delivery.seq);
}

/// Pop the contiguous run of buffered deliveries starting at the next
/// expected seq (dropping anything the watermark already covers), marking
/// each applied. Called after every apply and after a history install.
pub(crate) fn drain_contiguous(
    net: &mut NetState,
    pending: &mut PendingEdits,
) -> Vec<SequencedDelivery> {
    let mut ready = Vec::new();
    while let Some(delivery) = net.reorder.pop_next(expected_seq(net)) {
        if net.recent_uids.contains(delivery.uid) {
            // Already applied under another seq: same treatment as the live
            // identity dedup (dropped, confirmed; the next buffered seq then
            // waits for the timeout-driven history install).
            pending.confirm(delivery.uid);
            break;
        }
        mark_applied(net, &delivery);
        ready.push(delivery);
    }
    ready
}

/// Session reset on reconnect. A *same-room* reconnect (stall recovery, a
/// WebRTC blip) keeps unconfirmed submissions -- they are retransmitted into
/// the same game once the history is back -- and this game's record
/// directory. A *room change* abandons the old game entirely: its
/// submissions must never be retransmitted into the new room, and the engine
/// state, seat table and record start fresh (the new room's `StartGame` or
/// history install repopulates them; `init_game_record` opens a new game
/// directory).
pub(crate) fn reset_session_state(
    same_room: bool,
    pending: &mut PendingEdits,
    recorder: &mut game_record::GameRecorder,
    game_state: &mut omdurman_rules::effects::GameState,
    seats: &mut crate::seats::Seats,
) {
    pending.outgoing_broadcast.clear();
    pending.outgoing_targeted.clear();
    pending.stall_secs = 0.0;
    if same_room {
        recorder.reset_for_resync();
    } else {
        pending.unconfirmed.clear();
        pending.retransmit_timer = 0.0;
        *recorder = game_record::GameRecorder::default();
        *game_state = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        seats.0.clear();
    }
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

/// The picker selection plus the per-game state a room change abandons,
/// reset by [`handle_reconnect`] after reopening the socket.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct GameResetState<'w> {
    pub picker_state: ResMut<'w, picker::PickerState>,
    pub game_state: ResMut<'w, crate::GameStateResource>,
    pub seats: ResMut<'w, crate::seats::Seats>,
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
    game: GameResetState,
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
    let GameResetState {
        mut picker_state,
        mut game_state,
        mut seats,
    } = game;
    let Some(reconnect) = reconnect else { return };
    let new_room = reconnect.0.clone();

    if new_room.is_empty() {
        commands.remove_resource::<ReconnectRoom>();
        return;
    }

    let same_room = new_room == room.as_str();
    info!(%new_room, same_room, "reconnecting");

    // -- despawn old socket --
    if socket.is_some() {
        commands.remove_resource::<MatchboxSocket>();
    }

    // -- reset state --
    *net = NetState::default();
    *turn = TurnState::default();
    // Our record is wiped below: request the canonical history on reconnect
    // (any peer with a record may serve it) and block host authority until it
    // is installed or the bootstrap budget expires.
    net.needs_snapshot = true;
    net.resync_gate_secs = RESYNC_BOOTSTRAP_SECS;
    incoming.ephemeral.clear();
    incoming.loopback.clear();
    incoming.seat_control.clear();
    reset_session_state(
        same_room,
        &mut pending,
        &mut recorder,
        &mut game_state.0,
        &mut seats,
    );

    // -- drop any in-progress selection. The counters and the picker tray
    //    need no reset: they are a projection of the engine state
    //    (`reconcile_unit_sprites`), which the history install rebuilds. --
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
///
/// [`SUBMIT_STALL_RECONNECT_SECS`]: crate::net_plugin::SUBMIT_STALL_RECONNECT_SECS
pub(crate) fn auto_reconnect_on_stall(
    mut commands: Commands,
    pending: Res<PendingEdits>,
    net: Res<NetState>,
    room: Res<RoomId>,
    state: Res<State<AppState>>,
    reconnect_pending: Option<Res<ReconnectRoom>>,
    offline: Res<crate::net_plugin::OfflineMode>,
) {
    if pending.stall_secs < crate::net_plugin::SUBMIT_STALL_RECONNECT_SECS {
        return;
    }
    if offline.0 || !net.has_ever_peered {
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
            // Heartbeat of the whole numbering state while a resync loop is
            // active: correlating next_seq / last_applied / unconfirmed over
            // time shows whether sequencing, application, or confirmation is
            // the side that is stuck.
            info!(
                next_seq = net.next_seq,
                last_applied = ?net.last_applied_seq,
                is_host = net.is_host,
                unconfirmed = pending.unconfirmed.len(),
                buffered = net.reorder.len(),
                "guest: retrying snapshot request"
            );
            pending
                .outgoing_broadcast
                .push(NetMsg::Control(Control::RequestSnapshot));
        }
    }
}

/// Everything applying a ready `Sequenced` delivery touches, borrowed from
/// [`handle_socket`]'s parameters.
struct ApplyEnv<'a, 'w> {
    net: &'a mut NetState,
    pending: &'a mut PendingEdits,
    turn: &'a mut TurnState,
    gsp: &'a mut GameStateParams<'w>,
    recorder: &'a mut game_record::GameRecorder,
    state: &'a AppState,
    next_state: &'a mut NextState<AppState>,
    targeted: &'a mut Vec<(NetMsg, PeerId)>,
    /// This instance's stable player key (seat binding identity).
    local_key: omdurman_net::PlayerKey,
}

/// Record and apply deliveries that [`receive_sequenced`] /
/// [`drain_contiguous`] released (already marked applied), in order.
fn apply_ready(env: &mut ApplyEnv<'_, '_>, ready: Vec<SequencedDelivery>) {
    for delivery in ready {
        apply_sequenced(env, delivery);
    }
}

fn apply_sequenced(env: &mut ApplyEnv<'_, '_>, delivery: SequencedDelivery) {
    let SequencedDelivery {
        seq,
        uid,
        event: ev,
        from,
    } = delivery;
    let sender_idx = env.net.sender_idx(from);
    env.recorder.push_event(&ev, sender_idx, seq, Some(uid));
    info!(seq, uid, "applied sequenced event");
    // Our own submission made it through the host: stop retransmitting it.
    env.pending.confirm(uid);
    // The one application path (shared with replay): every variant reaches
    // the engine synchronously, in seq order.
    game_apply::apply_game_event(&ev, &mut env.gsp.sinks());
    env.gsp
        .pending_observations
        .0
        .extend(env.gsp.game_state.0.drain_observations());
    let GameEvent::StartGame {
        seats, scenario, ..
    } = &ev
    else {
        return;
    };
    let first_start = !env.turn.game_started;
    env.turn.game_started = true;
    // The engine always takes the StartGame (the record is the state); only
    // the *view* switch is gated on the lobby, so a player browsing the menu
    // is not yanked.
    if *env.state != AppState::Lobby {
        info!(%scenario, "StartGame applied; view unchanged (not in lobby)");
        return;
    }
    // Switch the view to the game board, so play opens on the scenario's
    // board rather than whatever screen preceded the lobby. (The board data
    // loads from `pending_map_load`; the board picked follows the scenario
    // via `sync_board_to_game`.)
    env.gsp.next_app_mode.set(crate::AppMode::Game);
    env.next_state.set(AppState::InGame);
    info!(%scenario, "game started via host StartGame");
    // A guest without a seat is a spectator: request the full record from
    // the host so it converges to every unit already placed, not just
    // events seen after this point. (Seated guests are present from the
    // start, so they don't need it.) Seats are keyed by the stable player
    // key, never by the session `PeerId`.
    let locally_assigned = crate::seats::seat_of(seats, env.local_key).is_some();
    if first_start && !env.net.is_host && !locally_assigned && !env.net.snapshot_applied {
        info!("no faction assigned to this peer; requesting snapshot as spectator");
        env.net.needs_snapshot = true;
        env.net.snapshot_retry_timer = 0.0;
        if let Some(host) = env.net.host_id() {
            env.targeted
                .push((NetMsg::Control(Control::RequestSnapshot), host));
        }
    }
}

/// The per-frame receive path: socket I/O (peer roster, election, inbound
/// messages) when a socket exists, then the loopback / sequencing / apply
/// path -- which runs *without* a socket too, so offline self-host mode
/// (`OfflineMode`) sequences and applies its own submissions.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_socket(
    mut socket: Option<ResMut<MatchboxSocket>>,
    traffic: NetTraffic,
    app_state: AppStateShift,
    mut commands: Commands,
    mut gsp: GameStateParams,
    mut ctx: SocketContext,
    mut last_held_uid: Local<Option<u64>>,
    local_key: Res<crate::seats::LocalPlayerKey>,
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

    // -- socket: peer roster + own id --
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
                my_id_changed = socket_id.is_some_and(|id| Some(id) != net.my_id);
                if my_id_changed {
                    net.my_id = socket_id;
                }
                true
            }
            Err(_) => false,
        },
        None => false,
    };

    // -- election clocks (socket or not: offline mode needs them too) --
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
        if promoted {
            // A freshly promoted host resumes the canonical numbering one
            // past the highest seq it applied (`next_host_seq` enforces the
            // same floor at every assignment, so this is belt and braces).
            // Deliveries buffered from the previous host can never be
            // completed now -- the new line is canonical by election.
            net.next_seq = expected_seq(&net);
            net.reorder.clear();
            info!(
                next_seq = net.next_seq,
                game_started = turn.game_started,
                "promoted to host; resumed sequence numbering"
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

    // Guest: a seq gap that outlived the reorder timeout will not fill on
    // its own -- request the canonical history (force-installed: the local
    // record is known incomplete).
    if net.is_host {
        net.reorder.clear();
    } else if net.reorder.tick(time.delta_secs()) {
        warn!(
            buffered = net.reorder.len(),
            first_buffered = ?net.reorder.first_seq(),
            last_applied = ?net.last_applied_seq,
            "seq gap persisted; requesting canonical history"
        );
        net.needs_snapshot = true;
        net.force_install_history = true;
        net.snapshot_retry_timer = 0.0;
        if let Some(host) = net.host_id() {
            targeted.push((NetMsg::Control(Control::RequestSnapshot), host));
        }
    }

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
        }
    }

    let mut received: Vec<(PeerId, Box<[u8]>)> = Vec::new();
    if socket_live && let Some(socket) = socket.as_deref_mut() {
        received.extend(socket.channel_mut(CH_RELIABLE).receive());
        received.extend(socket.channel_mut(CH_UNRELIABLE).receive());
    }
    let is_host = net.is_host;

    // Host loopback: events the host sequenced for itself (below). They flow
    // through the identical apply path as remote `Sequenced` events so every
    // peer -- host included -- observes the same ordered stream. `my_id` is the
    // canonical "sender" for these.
    let my_id = net.my_id.unwrap_or(PeerId(uuid::Uuid::nil()));
    let loopback: Vec<NetMsg> = std::mem::take(&mut ctx.incoming.loopback);

    let decoded = received
        .into_iter()
        .filter_map(|(peer, raw)| match decode(&raw) {
            Some(msg) => Some((peer, msg, false)),
            None => {
                warn!("unknown message, ignoring");
                None
            }
        })
        .chain(loopback.into_iter().map(|msg| (my_id, msg, true)));

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
                let recorded_seq = ctx.recorder.seq_of_uid(uid);
                let in_flight_seq = sequenced_out.iter().find_map(|m| match m {
                    NetMsg::Sequenced { seq, uid: u, .. } if *u == uid => Some(*seq),
                    _ => None,
                });
                if let Some(seq) = recorded_seq.or(in_flight_seq) {
                    info!(
                        seq,
                        uid,
                        via_record = recorded_seq.is_some(),
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
                    // (Logged once per uid: the bounce re-delivers it every
                    // frame while the gate is closed, which would spam.)
                    if last_held_uid.is_none_or(|held| held != uid) {
                        info!(
                            uid,
                            stable_secs = net.election_stable_secs,
                            resync_gate = net.resync_gate_secs,
                            "host: holding submission until sequencing is allowed"
                        );
                    }
                    *last_held_uid = Some(uid);
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game { uid, event: ev });
                    continue;
                }
                *last_held_uid = None;
                let seq = next_host_seq(&net);
                net.next_seq = seq + 1;
                info!(seq, uid, event = ?ev, "host: sequenced submission");
                let sequenced = NetMsg::Sequenced {
                    seq,
                    uid,
                    event: ev,
                };
                sequenced_out.push(sequenced.clone());
                // Push the echo onto our own loopback queue. It is *not* applied
                // here: this `for` loop is already iterating `decoded`, which was
                // chained from the loopback taken *above*. The just-pushed echo
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
                let ready = receive_sequenced(
                    &mut net,
                    &ctx.recorder,
                    &mut pending,
                    own_loopback,
                    SequencedDelivery {
                        seq,
                        uid,
                        event: ev,
                        from: peer,
                    },
                );
                apply_ready(
                    &mut ApplyEnv {
                        net: &mut net,
                        pending: &mut pending,
                        turn: &mut turn,
                        gsp: &mut gsp,
                        recorder: &mut ctx.recorder,
                        state: state.get(),
                        next_state: &mut next_state,
                        targeted: &mut targeted,
                        local_key: local_key.0,
                    },
                    ready,
                );
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
                }
            }
            NetMsg::Control(
                control @ (Control::SeatRequest { .. }
                | Control::SeatVoteOpen { .. }
                | Control::SeatVote { .. }
                | Control::SeatVoteClosed { .. }),
            ) => {
                // Seat claims and votes: handled by `seat_arbiter`.
                ctx.incoming.seat_control.push((control, peer));
            }
            NetMsg::Control(Control::SnapshotReceived) => {
                // Legacy acknowledgement (see `Control::SnapshotReceived`).
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
                // An authoritative host never installs foreign histories:
                // its own line is canonical by election, and a longer rogue
                // line would wipe its tail and desynchronize its numbering
                // baseline. The one exception is the resync gate (a freshly
                // reconnected host with a wiped record, which must
                // re-download the canonical line).
                if net.is_host && net.resync_gate_secs <= 0.0 {
                    debug!("host: ignoring foreign game history");
                    continue;
                }
                let record_max = record.events.iter().map(|e| e.seq).max();
                let ahead = match (record_max, net.last_applied_seq) {
                    (Some(hi), Some(applied)) => hi > applied,
                    (Some(_), None) => true,
                    (None, _) => false,
                };
                if !ahead && !net.force_install_history {
                    // A record whose highest seq *equals* our watermark is not
                    // junk -- it is proof that we are already converged. Clear
                    // the snapshot latch so the 2s request loop stops; only a
                    // record strictly *behind* us leaves the latch alone (it
                    // cannot satisfy a pending resync, so keep retrying).
                    if record_max == net.last_applied_seq {
                        info!(
                            watermark = ?net.last_applied_seq,
                            "received game history matching local state; already converged"
                        );
                        net.snapshot_applied = true;
                        net.needs_snapshot = false;
                        net.snapshot_retry_timer = 0.0;
                        net.force_install_history = false;
                    } else {
                        info!("ignoring game history that is not ahead of local state");
                    }
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
                let old_record = ctx.recorder.record.take();
                ctx.recorder.install_history(record.clone());
                // Own events the discarded line applied but the canonical one
                // lacks were confirmed by their (rogue) echo; re-queue them so
                // player input survives the rollback.
                if let Some(old) = old_record {
                    let requeued = pending.requeue_missing_own(&old, &record);
                    if requeued > 0 {
                        warn!(
                            requeued,
                            "re-queued own events missing from the installed history"
                        );
                    }
                }
                // The snapshot already includes every event up to the highest
                // recorded seq; mark them applied so a live `Sequenced` echo of
                // an event also present in the snapshot isn't applied a second
                // time (only seqs above the watermark are new to this joiner).
                // The uid dedup set must describe the installed line too.
                net.last_applied_seq = record_max;
                net.recent_uids
                    .rebuild(record.events.iter().filter_map(|e| e.uid));
                {
                    let GameStateParams {
                        game_state,
                        game_map,
                        loaded_annotations,
                        pending_map_load,
                        seats,
                        local_setup_ready,
                        bot_driver,
                        unit_paths,
                        ..
                    } = &mut gsp;
                    let mut state = RebuildState {
                        commands: &mut commands,
                        game_map,
                        sinks: game_apply::EventSinks {
                            game_state: &mut game_state.0,
                            seats,
                            local_setup_ready,
                            bot_driver,
                            loaded_annotations,
                            pending_map_load,
                            unit_paths,
                        },
                    };
                    rebuild_state_to(&record, None, &mut state);
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
                    // The installed history proves a game is running: host
                    // promotion must resume its numbering, and the
                    // proactive history push must serve reconnectees.
                    turn.game_started = true;
                    gsp.next_app_mode.set(crate::AppMode::Game);
                    if *state.get() == AppState::Lobby {
                        next_state.set(AppState::InGame);
                    }
                }
                // Live deliveries buffered past the old watermark may now be
                // contiguous with the installed history.
                let ready = drain_contiguous(&mut net, &mut pending);
                apply_ready(
                    &mut ApplyEnv {
                        net: &mut net,
                        pending: &mut pending,
                        turn: &mut turn,
                        gsp: &mut gsp,
                        recorder: &mut ctx.recorder,
                        state: state.get(),
                        next_state: &mut next_state,
                        targeted: &mut targeted,
                        local_key: local_key.0,
                    },
                    ready,
                );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_record::GameRecorder;
    use omdurman_types::{SectionName, SpriteRef};

    /// A distinct event per `n`, so payload comparisons can tell them apart.
    fn event(n: u32) -> GameEvent {
        GameEvent::RemoveUnit {
            sprite: SpriteRef {
                section_name: SectionName::Taiasha,
                col: n,
                row: 0,
            },
        }
    }

    fn start_game() -> GameEvent {
        GameEvent::StartGame {
            seats: Vec::new(),
            scenario: omdurman_types::Scenario::Campaign,
            optional_rules: Vec::new(),
        }
    }

    fn delivery(seq: u32, uid: u64) -> SequencedDelivery {
        SequencedDelivery {
            seq,
            uid,
            event: event(uid as u32),
            from: PeerId(uuid::Uuid::nil()),
        }
    }

    /// Feed a delivery through the receive gate and record what it released,
    /// like `apply_sequenced` does. Returns the applied seqs.
    fn receive(
        net: &mut NetState,
        recorder: &mut GameRecorder,
        pending: &mut PendingEdits,
        own_loopback: bool,
        d: SequencedDelivery,
    ) -> Vec<u32> {
        let ready = receive_sequenced(net, recorder, pending, own_loopback, d);
        ready
            .iter()
            .map(|d| {
                recorder.push_event(&d.event, None, d.seq, Some(d.uid));
                d.seq
            })
            .collect()
    }

    fn recorder() -> GameRecorder {
        let mut recorder = GameRecorder::default();
        recorder.install_history(omdurman_net::GameRecord {
            initial_state: omdurman_net::InitialGameState { seed: 7 },
            events: Vec::new(),
        });
        recorder
    }

    fn record(events: Vec<(u32, u64, GameEvent)>) -> omdurman_net::GameRecord {
        omdurman_net::GameRecord {
            initial_state: omdurman_net::InitialGameState { seed: 1 },
            events: events
                .into_iter()
                .map(|(seq, uid, payload)| omdurman_net::RecordedEvent {
                    utc: chrono::Utc::now(),
                    sender_idx: None,
                    seq,
                    uid: Some(uid),
                    payload,
                })
                .collect(),
        }
    }

    // D1: a delivery dropped as a seq conflict must not be remembered as
    // applied -- the host's re-sequenced echo of the same submission would
    // otherwise be identity-dropped and freeze its own line.
    #[test]
    fn dropped_conflict_does_not_poison_recent_uids() {
        let mut net = NetState::default();
        net.is_host = true;
        let mut rec = recorder();
        let mut pending = PendingEdits::default();
        // A foreign (dual-host) stream fills seq 0 first...
        assert_eq!(
            receive(&mut net, &mut rec, &mut pending, false, delivery(0, 1)),
            vec![0]
        );
        // ...so our own echo of uid 2 at seq 0 conflicts and is dropped.
        assert!(receive(&mut net, &mut rec, &mut pending, true, delivery(0, 2)).is_empty());
        assert!(
            !net.recent_uids.contains(2),
            "dropped uid must not be remembered"
        );
        // The retransmitted submission is re-sequenced at the next seq and
        // applies.
        assert_eq!(next_host_seq(&net), 1);
        assert_eq!(
            receive(&mut net, &mut rec, &mut pending, true, delivery(1, 2)),
            vec![1]
        );
        assert!(net.recent_uids.contains(2));
    }

    // D1: a foreign gap ignored by the host is not remembered either.
    #[test]
    fn host_ignored_foreign_gap_does_not_poison_recent_uids() {
        let mut net = NetState::default();
        net.is_host = true;
        let mut rec = recorder();
        let mut pending = PendingEdits::default();
        assert!(receive(&mut net, &mut rec, &mut pending, false, delivery(5, 9)).is_empty());
        assert!(!net.recent_uids.contains(9));
        assert_eq!(net.last_applied_seq, None);
        assert!(net.reorder.is_empty(), "hosts never buffer");
        assert_eq!(
            receive(&mut net, &mut rec, &mut pending, true, delivery(0, 9)),
            vec![0]
        );
    }

    // The host's own line never freezes: an own echo past the watermark is
    // applied over the hole (nobody else could ever fill it).
    #[test]
    fn host_applies_own_echo_over_a_hole() {
        let mut net = NetState::default();
        net.is_host = true;
        net.last_applied_seq = Some(3);
        let mut rec = recorder();
        let mut pending = PendingEdits::default();
        assert_eq!(
            receive(&mut net, &mut rec, &mut pending, true, delivery(5, 4)),
            vec![5]
        );
        assert_eq!(net.last_applied_seq, Some(5));
    }

    // An already-applied uid is dropped but still confirms our own
    // submission, so it is not retransmitted forever.
    #[test]
    fn duplicate_uid_is_dropped_and_confirmed() {
        let mut net = NetState::default();
        let mut rec = recorder();
        let mut pending = PendingEdits::with_uid_base(100);
        let uid = pending.submit_game(event(1));
        assert_eq!(
            receive(&mut net, &mut rec, &mut pending, false, delivery(0, uid)),
            vec![0]
        );
        pending.unconfirmed.push_back((uid, event(1)));
        assert!(receive(&mut net, &mut rec, &mut pending, false, delivery(3, uid)).is_empty());
        assert!(pending.unconfirmed.is_empty());
    }

    // D7: a first delivery past seq 0 on a fresh peer is a gap (buffered,
    // not applied to a state missing seqs 0..n).
    #[test]
    fn first_delivery_past_zero_is_a_gap() {
        let mut net = NetState::default();
        let mut rec = recorder();
        let mut pending = PendingEdits::default();
        assert!(receive(&mut net, &mut rec, &mut pending, false, delivery(2, 12)).is_empty());
        assert_eq!(net.last_applied_seq, None);
        assert_eq!(net.reorder.len(), 1);
        assert!(!net.recent_uids.contains(12));
    }

    // D7: out-of-order deliveries are applied as contiguous runs.
    #[test]
    fn reorder_buffer_applies_contiguous_runs() {
        let mut net = NetState::default();
        let mut rec = recorder();
        let mut pending = PendingEdits::default();
        assert!(receive(&mut net, &mut rec, &mut pending, false, delivery(2, 12)).is_empty());
        assert!(receive(&mut net, &mut rec, &mut pending, false, delivery(4, 14)).is_empty());
        assert_eq!(
            receive(&mut net, &mut rec, &mut pending, false, delivery(0, 10)),
            vec![0]
        );
        assert_eq!(
            receive(&mut net, &mut rec, &mut pending, false, delivery(1, 11)),
            vec![1, 2]
        );
        assert_eq!(
            receive(&mut net, &mut rec, &mut pending, false, delivery(3, 13)),
            vec![3, 4]
        );
        assert!(net.reorder.is_empty());
        let seqs: Vec<u32> = rec
            .record
            .as_ref()
            .unwrap()
            .events
            .iter()
            .map(|e| e.seq)
            .collect();
        assert_eq!(seqs, vec![0, 1, 2, 3, 4], "recorded in seq order");
    }

    // D7: a history install moves the watermark; buffered deliveries it
    // covers are discarded and the rest drain in order.
    #[test]
    fn history_install_drains_buffer_past_watermark() {
        let mut net = NetState::default();
        let mut rec = recorder();
        let mut pending = PendingEdits::default();
        for (seq, uid) in [(3, 13), (5, 15), (6, 16)] {
            assert!(
                receive(&mut net, &mut rec, &mut pending, false, delivery(seq, uid)).is_empty()
            );
        }
        // Installed history covers 0..=4.
        net.last_applied_seq = Some(4);
        let drained: Vec<u32> = drain_contiguous(&mut net, &mut pending)
            .into_iter()
            .map(|d| d.seq)
            .collect();
        assert_eq!(drained, vec![5, 6]);
        assert!(net.reorder.is_empty());
    }

    // A guest conflict below the watermark forces a history install and is
    // not remembered as applied.
    #[test]
    fn guest_conflict_forces_history_install() {
        let mut net = NetState::default();
        let mut rec = recorder();
        let mut pending = PendingEdits::default();
        receive(&mut net, &mut rec, &mut pending, false, delivery(0, 1));
        assert!(receive(&mut net, &mut rec, &mut pending, false, delivery(0, 2)).is_empty());
        assert!(net.force_install_history && net.needs_snapshot);
        assert!(!net.recent_uids.contains(2));
    }

    // D2: the host's next seq is floored at the watermark.
    #[test]
    fn host_seq_never_reuses_applied_seqs() {
        let mut net = NetState::default();
        net.last_applied_seq = Some(41);
        assert_eq!(next_host_seq(&net), 42, "stale next_seq=0 is floored");
        net.next_seq = 45;
        assert_eq!(next_host_seq(&net), 45, "in-flight echoes keep their lead");
    }

    // D3: a room change drops the old game's unconfirmed submissions and
    // engine state; a same-room reconnect keeps submissions for
    // retransmission and keeps the record's game directory.
    #[test]
    fn room_change_drops_unconfirmed_but_same_room_keeps_them() {
        let dir = tempfile::tempdir().unwrap();
        let games = dir.path().to_str().unwrap();

        let mut pending = PendingEdits::with_uid_base(1);
        pending.submit_game(event(1));
        let mut recorder = GameRecorder::init_in(games, 5);
        recorder.push_event(&event(9), None, 0, Some(9));
        let artifacts = recorder.artifacts_dir();
        let mut game_state =
            omdurman_rules::effects::GameState::new(omdurman_types::Scenario::FallOfKhartoum);
        let mut ai = crate::seats::Seats(vec![omdurman_net::Seat {
            faction: omdurman_types::Player::Dervish,
            scope: None,
            holder: omdurman_net::SeatHolder::Ai,
        }]);

        reset_session_state(true, &mut pending, &mut recorder, &mut game_state, &mut ai);
        assert_eq!(pending.unconfirmed.len(), 1, "same room: retransmit later");
        assert!(pending.outgoing_broadcast.is_empty());
        assert_eq!(recorder.artifacts_dir(), artifacts, "same game directory");
        assert!(recorder.record.as_ref().unwrap().events.is_empty());
        assert_eq!(ai.0.len(), 1);

        reset_session_state(false, &mut pending, &mut recorder, &mut game_state, &mut ai);
        assert!(
            pending.unconfirmed.is_empty(),
            "room change: old moves dropped"
        );
        assert!(recorder.record.is_none(), "a fresh record is initialised");
        assert!(ai.0.is_empty());
        assert_eq!(game_state.scenario, omdurman_types::Scenario::Campaign);
    }

    // D6: own events the discarded line applied but the installed canonical
    // line lacks are re-queued for submission, in order; foreign events and
    // StartGame never are.
    #[test]
    fn history_install_requeues_missing_own_events() {
        let mut pending = PendingEdits::with_uid_base(1000);
        let own_start = pending.submit_game(start_game());
        let own_kept = pending.submit_game(event(1));
        let own_lost_a = pending.submit_game(event(2));
        let own_lost_b = pending.submit_game(event(3));
        // All confirmed by their (rogue-line) echoes.
        pending.unconfirmed.clear();
        pending.outgoing_broadcast.clear();
        let old = record(vec![
            (0, own_start, start_game()),
            (1, own_kept, event(1)),
            (2, own_lost_a, event(2)),
            (3, 7, event(7)), // someone else's: not ours to resubmit
            (4, own_lost_b, event(3)),
        ]);
        let new = record(vec![(0, 5, start_game()), (1, own_kept, event(1))]);
        assert!(!pending.is_own_uid(7));
        assert_eq!(pending.requeue_missing_own(&old, &new), 2);
        let requeued: Vec<u64> = pending.unconfirmed.iter().map(|(u, _)| *u).collect();
        assert_eq!(requeued, vec![own_lost_a, own_lost_b]);
        assert_eq!(pending.outgoing_broadcast.len(), 2);
    }
}

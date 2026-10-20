//! Rebuilding the socket: a room change, a same-room reconnect, and the
//! stall safety net that triggers one.

use super::*;

/// What a reconnect keeps of the session (see [`reset_session_state`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SessionReset {
    /// Back into the room we were alone in (a solo game, the offline
    /// fallback, or everyone else left): our line is the only one there is,
    /// so the record and its numbering are kept -- wiping it would lose the
    /// game, with nobody to re-download it from.
    OwnLine,
    /// Back into the room of a session with other peers: the record is
    /// wiped and re-downloaded from them.
    SameRoom,
    /// Another room: the old game is abandoned.
    NewRoom,
}

impl SessionReset {
    /// The reset for a reconnect to the same room or another, with or
    /// without peers connected at the time.
    pub(crate) fn for_reconnect(same_room: bool, alone: bool) -> Self {
        match (same_room, alone) {
            (false, _) => Self::NewRoom,
            (true, true) => Self::OwnLine,
            (true, false) => Self::SameRoom,
        }
    }
}

/// Session reset on reconnect. A *same-room* reconnect (stall recovery, a
/// WebRTC blip) keeps unconfirmed submissions -- they are retransmitted into
/// the same game once the history is back -- and this game's record
/// directory (and, for [`SessionReset::OwnLine`], the record itself). A
/// *room change* abandons the old game entirely: its submissions must never
/// be retransmitted into the new room, and the engine state, seat table and
/// record start fresh (the new room's `StartGame` or history install
/// repopulates them; `init_game_record` opens a new game directory).
pub(super) fn reset_session_state(
    reset: SessionReset,
    pending: &mut PendingEdits,
    recorder: &mut game_record::GameRecorder,
    game_state: &mut omdurman_rules::effects::GameState,
    seats: &mut crate::seats::Seats,
) {
    pending.outgoing_broadcast.clear();
    pending.outgoing_targeted.clear();
    pending.stall_secs = 0.0;
    if reset == SessionReset::OwnLine {
        return;
    }
    if reset == SessionReset::SameRoom {
        recorder.reset_for_resync();
    } else {
        pending.unconfirmed.clear();
        pending.retransmit_timer = 0.0;
        *recorder = game_record::GameRecorder::default();
        *game_state = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        seats.0.clear();
    }
}

pub(crate) fn handle_reconnect(
    mut commands: Commands,
    mut s: Session,
    reconnect: Option<Res<ReconnectRoom>>,
    mut room: ResMut<RoomId>,
    mut picker_state: ResMut<picker::PickerState>,
    mut incoming: ResMut<PendingIncoming>,
    socket: Option<Res<MatchboxSocket>>,
) {
    let Some(reconnect) = reconnect else { return };
    let new_room = reconnect.0.clone();

    if new_room.is_empty() {
        commands.remove_resource::<ReconnectRoom>();
        return;
    }

    let same_room = new_room == room.as_str();
    let reset = SessionReset::for_reconnect(same_room, s.net.peers().is_empty());
    info!(%new_room, ?reset, "reconnecting");

    // -- despawn old socket --
    if socket.is_some() {
        commands.remove_resource::<MatchboxSocket>();
    }

    // -- reset state --
    *s.net = NetState::default();
    if reset == SessionReset::OwnLine {
        // Nobody else was here: our record is the room's line. It is kept
        // (below) and we resume its numbering, as a peer that installed it
        // would -- no resync gate, no snapshot to wait for, and solitude is
        // session evidence again (we are not `rejoining` anyone).
        restore_own_line(&mut s.net, &s.recorder);
    } else {
        *s.turn = TurnState::default();
        // Our record is wiped below: request the canonical history on
        // reconnect (any peer with a record may serve it) and block host
        // authority until it is installed or the bootstrap budget expires.
        s.net.request_history(false);
        s.net.resync_gate_secs = RESYNC_BOOTSTRAP_SECS;
        // Back into the same room: a session lives here, and our record is
        // gone until a history is installed (see `NetState::rejoining`).
        s.net.rejoining = same_room;
    }
    incoming.ephemeral.clear();
    incoming.loopback.clear();
    incoming.seat_control.clear();
    incoming.held.clear();
    reset_session_state(
        reset,
        &mut s.pending,
        &mut s.recorder,
        &mut s.gsp.game_state.0,
        &mut s.gsp.seats,
    );

    // -- drop any in-progress selection. The counters and the picker tray
    //    need no reset: they are a projection of the engine state
    //    (`reconcile_unit_sprites`), which the history install rebuilds. --
    *picker_state = picker::PickerState::Idle;

    // -- update room id and URL --
    *room = RoomId::new(new_room.clone());

    #[cfg(target_arch = "wasm32")]
    omdurman_net::write_fragment(&omdurman_net::with_fragment_param(
        &omdurman_net::read_fragment(),
        "room",
        Some(&new_room),
    ));

    // -- open new socket --
    commands.insert_resource(omdurman_net::build_socket(&new_room));

    // -- return to the lobby so the player can re-pick faction / scenario --
    // The socket is fresh; the lobby renders immediately and `handle_socket`
    // processes peer/list state next frame.
    s.next_state.set(AppState::Lobby);

    commands.remove_resource::<ReconnectRoom>();
}

/// Resume the numbering of the record we keep (a reconnect into the room we
/// were alone in, [`SessionReset::OwnLine`]): watermark, next seq and the
/// uid dedup set describe the kept line, exactly as after a history install.
pub(super) fn restore_own_line(net: &mut NetState, recorder: &game_record::GameRecorder) {
    let Some(record) = &recorder.record else {
        return;
    };
    net.last_applied_seq = record.events.iter().map(|e| e.seq).max();
    net.next_seq = expected_seq(net);
    net.recent_uids
        .rebuild(record.events.iter().filter_map(|e| e.uid));
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
pub(super) fn auto_reconnect_on_stall(
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

use super::reconnect::*;
use super::resync::*;
use super::sequencing::*;
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
        assert!(receive(&mut net, &mut rec, &mut pending, false, delivery(seq, uid)).is_empty());
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
fn a_room_found_empty_is_session_evidence() {
    let mut net = NetState::default();
    net.election_stable_secs = 60.0;
    assert!(
        !has_session_evidence(&net),
        "not registered by the signalling server: no evidence"
    );
    net.set_my_id(Some(PeerId(uuid::Uuid::from_u128(7))));
    net.election_stable_secs = omdurman_net::SOLO_ROOM_SECS * 0.5;
    assert!(
        !has_session_evidence(&net),
        "just connected: the server may still introduce the room"
    );
    net.election_stable_secs = omdurman_net::SOLO_ROOM_SECS;
    assert!(has_session_evidence(&net), "alone long enough: a solo room");
    net.has_ever_peered = true;
    net.resync_gate_secs = 0.0;
    assert!(sequencing_allowed(&net), "the solo host sequences");
}

/// A peer back in the room it had a session in waits for that session:
/// alone, it must not start a line of its own on its wiped record.
#[test]
fn a_rejoining_peer_alone_has_no_session_evidence() {
    let mut net = NetState::default();
    net.set_my_id(Some(PeerId(uuid::Uuid::from_u128(7))));
    net.election_stable_secs = 600.0;
    net.rejoining = true;
    assert!(!has_session_evidence(&net));
    net.add_peer(PeerId(uuid::Uuid::from_u128(9)));
    assert!(has_session_evidence(&net), "a peer present is evidence");
}

/// Run `retry_snapshot_request` for `secs` (in 0.1 s frames); returns how
/// many `RequestSnapshot`s it staged.
fn snapshot_requests(app: &mut App, secs: f32) -> usize {
    let mut sent = 0;
    for _ in 0..(secs * 10.0) as usize {
        app.update();
        let mut pending = app.world_mut().resource_mut::<PendingEdits>();
        sent += pending
            .outgoing_broadcast
            .drain(..)
            .filter(|m| matches!(m, NetMsg::Control(Control::RequestSnapshot)))
            .count();
    }
    sent
}

fn snapshot_app(net: NetState) -> App {
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin)
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(100),
        ))
        .insert_resource(net)
        .insert_resource(PendingEdits::default())
        .add_systems(Update, retry_snapshot_request);
    app
}

/// The snapshot request loop is bounded: every 2 s at first, then only
/// every 15 s once the bootstrap budget passed unanswered -- unless the
/// record is known wrong (a seq conflict), which stays urgent.
#[test]
fn unanswered_snapshot_requests_slow_down_unless_urgent() {
    let mut net = NetState::default();
    net.needs_snapshot = true;
    net.add_peer(PeerId(uuid::Uuid::from_u128(9)));
    let mut app = snapshot_app(net);
    let early = snapshot_requests(&mut app, RESYNC_BOOTSTRAP_SECS);
    assert!(early >= 6, "every ~2 s at first: {early}");
    let late = snapshot_requests(&mut app, 30.0);
    assert!(late <= 2, "every ~15 s once patient: {late}");
    app.world_mut()
        .resource_mut::<NetState>()
        .force_install_history = true;
    let urgent = snapshot_requests(&mut app, 10.0);
    assert!(urgent >= 4, "a conflict is urgent again: {urgent}");
}

/// Nobody to ask: no request is staged (it would be dropped unsent).
#[test]
fn no_snapshot_request_without_peers() {
    let mut net = NetState::default();
    net.needs_snapshot = true;
    let mut app = snapshot_app(net);
    assert_eq!(snapshot_requests(&mut app, 10.0), 0);
    assert!(
        app.world().resource::<NetState>().needs_snapshot,
        "the latch waits"
    );
}

/// A host that bootstrapped its own line never installs a foreign
/// history, so it stops asking for one (every guest would send it the
/// whole record, every round).
#[test]
fn a_bootstrapped_host_stops_requesting_history() {
    let mut net = NetState::default();
    net.needs_snapshot = true;
    net.is_host = true;
    net.resync_gate_secs = 0.0;
    net.last_applied_seq = Some(3);
    net.add_peer(PeerId(uuid::Uuid::from_u128(9)));
    let mut app = snapshot_app(net);
    assert_eq!(snapshot_requests(&mut app, 5.0), 0);
    assert!(!app.world().resource::<NetState>().needs_snapshot);
}

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
    let mut recorder = GameRecorder::init(games, 5);
    recorder.push_event(&event(9), None, 0, Some(9));
    let artifacts = recorder.artifacts_dir();
    let mut game_state =
        omdurman_rules::effects::GameState::new(omdurman_types::Scenario::FallOfKhartoum);
    let mut ai = crate::seats::Seats(vec![omdurman_net::Seat {
        faction: omdurman_types::Player::Dervish,
        scope: None,
        holder: omdurman_net::SeatHolder::Ai,
    }]);

    reset_session_state(
        SessionReset::SameRoom,
        &mut pending,
        &mut recorder,
        &mut game_state,
        &mut ai,
    );
    assert_eq!(pending.unconfirmed.len(), 1, "same room: retransmit later");
    assert!(pending.outgoing_broadcast.is_empty());
    assert_eq!(recorder.artifacts_dir(), artifacts, "same game directory");
    assert!(recorder.record.as_ref().unwrap().events.is_empty());
    assert_eq!(ai.0.len(), 1);

    reset_session_state(
        SessionReset::NewRoom,
        &mut pending,
        &mut recorder,
        &mut game_state,
        &mut ai,
    );
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

// -- the host's receive path, headless ------------------------------

/// `handle_socket` alone, as an elected host with no socket (offline
/// self-hosting): submissions are fed through `PendingIncoming::held`,
/// which is examined ahead of the loopback -- exactly where a guest's
/// submission arriving over the socket sits.
fn host_app(game_started: bool) -> App {
    let mut app = App::new();
    let mut net = NetState::default();
    net.set_my_id(Some(PeerId(uuid::Uuid::nil())));
    net.is_host = true;
    net.has_ever_peered = true;
    net.election_stable_secs = SEQ_STABILIZE_SECS;
    app.add_plugins((bevy::time::TimePlugin, bevy::state::app::StatesPlugin))
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(100),
        ))
        .insert_state(AppState::InGame)
        .init_state::<crate::AppMode>()
        .insert_resource(net)
        .insert_resource(PendingEdits::with_uid_base(1 << 40))
        .insert_resource(TurnState { game_started })
        .insert_resource(PendingIncoming::default())
        .insert_resource(recorder())
        .insert_resource(crate::seats::LocalPlayerKey(omdurman_net::PlayerKey(1)))
        .insert_resource(crate::GameStateResource(
            omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign),
        ))
        .init_resource::<crate::LoadedAnnotations>()
        .init_resource::<crate::PendingMapLoad>()
        .init_resource::<crate::events::PendingObservations>()
        .init_resource::<crate::seats::Seats>()
        .init_resource::<crate::peers::LocalSetupReady>()
        .init_resource::<crate::bot_player::BotDriver>()
        .init_resource::<crate::picker::UnitPaths>()
        .init_resource::<crate::telegram::TelegramLog>()
        .insert_resource(crate::GameRng::from_seed(1))
        .add_systems(Update, handle_socket);
    app
}

/// A guest's submission reaching the host (see [`host_app`]).
fn guest_submits(app: &mut App, uid: u64) {
    app.world_mut()
        .resource_mut::<PendingIncoming>()
        .hold(uid, event(uid as u32), false);
}

fn recorded_seqs(app: &App) -> Vec<(u32, Option<u64>)> {
    app.world()
        .resource::<GameRecorder>()
        .record
        .as_ref()
        .unwrap()
        .events
        .iter()
        .map(|e| (e.seq, e.uid))
        .collect()
}

/// Every `Sequenced` the host staged for broadcast, as (seq, uid).
fn broadcast_seqs(app: &mut App) -> Vec<(u32, u64)> {
    app.world_mut()
        .resource_mut::<PendingEdits>()
        .outgoing_broadcast
        .drain(..)
        .filter_map(|m| match m {
            NetMsg::Sequenced { seq, uid, .. } => Some((seq, uid)),
            _ => None,
        })
        .collect()
}

/// A guest's retransmission arriving after the host sequenced the
/// submission but before its own echo came round the loopback (one
/// frame later) is re-echoed under the seq it already has -- not given
/// a second seq, which left a hole in the host's line that every guest
/// had to heal by a forced history install.
#[test]
fn a_retransmission_before_the_echo_is_re_echoed_not_resequenced() {
    let mut app = host_app(true);
    guest_submits(&mut app, 77);
    app.update();
    assert_eq!(broadcast_seqs(&mut app), vec![(0, 77)]);
    assert!(
        recorded_seqs(&app).is_empty(),
        "applied on echo, next frame"
    );
    // The retransmission lands ahead of the loopback echo.
    guest_submits(&mut app, 77);
    guest_submits(&mut app, 78);
    app.update();
    assert_eq!(
        broadcast_seqs(&mut app),
        vec![(0, 77), (1, 78)],
        "re-echoed at seq 0; the next submission gets seq 1"
    );
    app.update();
    app.update();
    assert_eq!(recorded_seqs(&app), vec![(0, Some(77)), (1, Some(78))]);
    let net = app.world().resource::<NetState>();
    assert_eq!(net.last_applied_seq, Some(1), "no hole in the host's line");
    assert!(net.host_in_flight.is_empty());
    // Once recorded, a retransmission is answered from the record.
    guest_submits(&mut app, 77);
    app.update();
    assert_eq!(broadcast_seqs(&mut app), vec![(0, 77)]);
}

/// An own echo dropped as a conflict leaves the in-flight table, so the
/// retransmission is sequenced afresh instead of being re-echoed at the
/// conflicting seq forever.
#[test]
fn a_dropped_own_echo_frees_the_submission_for_a_fresh_seq() {
    let mut net = NetState::default();
    net.is_host = true;
    let mut rec = recorder();
    let mut pending = PendingEdits::default();
    assert_eq!(sequence_fresh(&mut net, 2), 0);
    assert_eq!(already_sequenced(&net, &rec, 2), Some((0, false)));
    // A foreign stream fills seq 0 first; our echo of uid 2 conflicts.
    receive(&mut net, &mut rec, &mut pending, false, delivery(0, 1));
    assert!(receive(&mut net, &mut rec, &mut pending, true, delivery(0, 2)).is_empty());
    assert_eq!(already_sequenced(&net, &rec, 2), None);
    assert_eq!(sequence_fresh(&mut net, 2), 1);
    assert_eq!(
        receive(&mut net, &mut rec, &mut pending, true, delivery(1, 2)),
        vec![1]
    );
    assert_eq!(already_sequenced(&net, &rec, 2), Some((1, true)));
}

/// Submissions held while sequencing is not allowed are sequenced in
/// the order they arrived once it is -- ahead of anything that arrived
/// during the hold, and a retransmission keeps its first place.
#[test]
fn held_submissions_are_sequenced_in_arrival_order() {
    let mut app = host_app(true);
    app.world_mut().resource_mut::<NetState>().resync_gate_secs = 5.0;
    guest_submits(&mut app, 10);
    app.update();
    guest_submits(&mut app, 11);
    guest_submits(&mut app, 10); // a retransmission during the hold
    app.update();
    guest_submits(&mut app, 12);
    app.update();
    assert!(broadcast_seqs(&mut app).is_empty(), "held");
    let held: Vec<u64> = app
        .world()
        .resource::<PendingIncoming>()
        .held
        .iter()
        .map(|h| h.uid)
        .collect();
    assert_eq!(held, vec![10, 11, 12]);
    app.world_mut().resource_mut::<NetState>().resync_gate_secs = 0.0;
    guest_submits(&mut app, 13);
    app.update();
    assert_eq!(
        broadcast_seqs(&mut app),
        vec![(0, 10), (1, 11), (2, 12), (3, 13)]
    );
}

/// A host without a game (a newcomer that won the election on an empty
/// record) holds the guests' game actions and asks for the history,
/// instead of starting a rogue line under the guests' canonical one;
/// once a game is running the held actions are sequenced.
#[test]
fn a_gameless_host_holds_guest_game_actions_for_the_history() {
    let mut app = host_app(false);
    guest_submits(&mut app, 5);
    app.update();
    app.update();
    assert!(broadcast_seqs(&mut app).is_empty());
    assert!(app.world().resource::<NetState>().needs_snapshot);
    assert_eq!(app.world().resource::<PendingIncoming>().held.len(), 1);
    // A StartGame (session event) is not held for it.
    app.world_mut()
        .resource_mut::<PendingIncoming>()
        .hold(6, start_game(), false);
    app.update();
    assert_eq!(broadcast_seqs(&mut app), vec![(0, 6)]);
    // The game is running (here: the StartGame applied): released.
    app.update();
    assert!(app.world().resource::<TurnState>().game_started);
    app.update();
    assert_eq!(broadcast_seqs(&mut app), vec![(1, 5)]);
}

// Reconnecting to the room this peer was alone in keeps its line: the
// record survives (nobody else could serve it back), the numbering
// resumes after it, and solitude is session evidence again -- the
// instance used to never sequence again.
#[test]
fn reconnecting_alone_to_the_same_room_keeps_the_own_line() {
    assert_eq!(
        SessionReset::for_reconnect(true, true),
        SessionReset::OwnLine
    );
    assert_eq!(
        SessionReset::for_reconnect(true, false),
        SessionReset::SameRoom
    );
    assert_eq!(
        SessionReset::for_reconnect(false, true),
        SessionReset::NewRoom
    );
    let dir = tempfile::tempdir().unwrap();
    let mut pending = PendingEdits::with_uid_base(1);
    let mut recorder = GameRecorder::init(dir.path().to_str().unwrap(), 5);
    recorder.push_event(&start_game(), None, 0, Some(40));
    recorder.push_event(&event(1), None, 1, Some(41));
    let mut game_state =
        omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
    let mut seats = crate::seats::Seats::default();
    reset_session_state(
        SessionReset::OwnLine,
        &mut pending,
        &mut recorder,
        &mut game_state,
        &mut seats,
    );
    assert_eq!(recorder.record.as_ref().unwrap().events.len(), 2);
    let mut net = NetState::default();
    restore_own_line(&mut net, &recorder);
    assert_eq!(net.last_applied_seq, Some(1));
    assert_eq!(next_host_seq(&net), 2);
    assert!(net.recent_uids.contains(41));
    assert!(!net.rejoining && net.resync_gate_secs <= 0.0);
    net.set_my_id(Some(PeerId(uuid::Uuid::from_u128(7))));
    net.election_stable_secs = omdurman_net::SOLO_ROOM_SECS;
    assert!(has_session_evidence(&net), "alone again: a solo room");
}

//! Catching up on the canonical line: asking for the game history, serving
//! it, offering it to a (re)connected peer, and installing a foreign one.

use super::*;

pub(super) fn retry_snapshot_request(
    time: Res<Time>,
    mut net: ResMut<NetState>,
    mut pending: ResMut<PendingEdits>,
    // Seconds the current request has gone unanswered, and whether the latch
    // was set last frame (a fresh request restarts the count).
    mut waited: Local<(f64, bool)>,
) {
    if !net.needs_snapshot {
        *waited = (0.0, false);
        return;
    }
    if !waited.1 {
        *waited = (0.0, true);
    }
    // A host that bootstrapped its own line (resync budget spent, events
    // applied) never installs a foreign history (see the `GameHistory`
    // arm): asking would only make every guest send it the whole record,
    // every round, for good.
    if net.is_host && net.resync_gate_secs <= 0.0 && net.last_applied_seq.is_some() {
        info!("host: own line bootstrapped; no longer requesting a game history");
        net.needs_snapshot = false;
        net.snapshot_retry_timer = 0.0;
        return;
    }
    let dt = time.delta_secs_f64();
    waited.0 += dt;
    net.snapshot_retry_timer += dt;
    // Every 2 s at first; once the bootstrap budget has passed unanswered
    // (nobody in the room has a game -- a lobby before the start), only
    // now and then, quietly: a game starting later pushes its history to
    // everyone anyway.
    // (A detected seq conflict or gap is urgent whatever came before: the
    // local record is known to be wrong.)
    let patient = !net.force_install_history && waited.0 > f64::from(RESYNC_BOOTSTRAP_SECS);
    let interval = if patient { 15.0 } else { 2.0 };
    if net.snapshot_retry_timer <= interval {
        return;
    }
    net.snapshot_retry_timer = 0.0;
    // Nobody to ask: the request would be dropped unsent (keep the latch for
    // the first peer that connects).
    if net.peers().is_empty() {
        return;
    }
    if patient {
        debug!("guest: retrying snapshot request (no answer yet)");
    } else {
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
    }
    pending
        .outgoing_broadcast
        .push(NetMsg::Control(Control::RequestSnapshot));
}

/// Resync staging before any message is read: the guest's timed-out seq gap
/// asks for the canonical history, and each side offers its record to a peer
/// that just (re)connected.
pub(super) fn stage_catch_up(s: &mut Session, time: &Time, newly_connected: &[PeerId]) {
    let Session {
        net,
        pending,
        turn,
        recorder,
        ..
    } = s;
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
        net.request_history(true);
        if let Some(host) = net.host_id() {
            pending
                .outgoing_targeted
                .push((NetMsg::Control(Control::RequestSnapshot), host));
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
        && let Some(ref record) = recorder.record
        && !record.events.is_empty()
    {
        for &peer in newly_connected {
            info!(%peer, "host: pushing game history to (re)connected peer");
            pending
                .outgoing_targeted
                .push((NetMsg::Control(Control::GameHistory(record.clone())), peer));
        }
    }

    // Guest: our host just (re)connected. A host whose *process* was
    // relaunched comes back with an empty record, re-wins the election, and --
    // unlike an in-process reconnect -- has no resync gate asking for the
    // canonical line, which now lives only on the guests. Offer it ours; a
    // host installs a foreign history only while it has applied nothing.
    if !net.is_host
        && turn.game_started
        && let Some(host) = net.host_id()
        && newly_connected.contains(&host)
        && let Some(ref record) = recorder.record
        && !record.events.is_empty()
    {
        info!(%host, "guest: offering game history to (re)connected host");
        pending
            .outgoing_targeted
            .push((NetMsg::Control(Control::GameHistory(record.clone())), host));
    }
}

/// Serve a `RequestSnapshot`: only the elected host serves arbitrary
/// requesters -- and, for the reconnected-host deadlock, a guest serves its
/// *own host*. Anyone else serving would let rogue lines masquerade as
/// canonical. Installs stay guarded by the ahead check on the receiving side.
pub(super) fn serve_snapshot(s: &mut Session, peer: PeerId) {
    let requester_is_my_host = Some(peer) == s.net.host_id();
    if !s.net.is_host && !requester_is_my_host {
        return;
    }
    if s.turn.game_started
        && let Some(ref record) = s.recorder.record
        && !record.events.is_empty()
    {
        info!(
            events = record.events.len(),
            "late joiner requested game history; sending"
        );
        s.pending
            .outgoing_targeted
            .push((NetMsg::Control(Control::GameHistory(record.clone())), peer));
    } else {
        // No game to serve (yet): the requester retries.
        debug!("game history requested, but no game has started");
    }
}

/// Install a foreign `GameHistory`: the one resync path (late joiner,
/// diverged guest, reconnected host behind its guests). Guards decide
/// whether the record wins; the install replaces the local record wholesale,
/// re-queues own events the canonical line lacks, and rebuilds the world
/// from the record (§divergence healing).
pub(super) fn install_foreign_history(
    s: &mut Session,
    commands: &mut Commands,
    record: omdurman_net::GameRecord,
) {
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
    // baseline. The exceptions are the resync gate (a freshly
    // reconnected host with a wiped record, which must
    // re-download the canonical line) and an empty host (below).
    // A host that has applied nothing (a relaunched process that
    // re-won the election) has no line to defend: it takes the
    // record a guest offers it.
    if s.net.is_host && s.net.resync_gate_secs <= 0.0 && s.net.last_applied_seq.is_some() {
        debug!("host: ignoring foreign game history");
        return;
    }
    let record_max = record.events.iter().map(|e| e.seq).max();
    let ahead = match (record_max, s.net.last_applied_seq) {
        (Some(hi), Some(applied)) => hi > applied,
        (Some(_), None) => true,
        (None, _) => false,
    };
    if !ahead && !s.net.force_install_history {
        // A record whose highest seq *equals* our watermark is not
        // junk -- it is proof that we are already converged. Clear
        // the snapshot latch so the 2s request loop stops; only a
        // record strictly *behind* us leaves the latch alone (it
        // cannot satisfy a s.pending resync, so keep retrying).
        if record_max == s.net.last_applied_seq {
            info!(
                watermark = ?s.net.last_applied_seq,
                "received game history matching local state; already converged"
            );
            s.net.history_settled();
        } else {
            info!("ignoring game history that is not ahead of local state");
        }
        return;
    }
    s.net.history_settled();
    s.net.resync_gate_secs = 0.0;
    info!(
        "received game history ({} events), replaying to resync",
        record.events.len()
    );
    let old_record = s.recorder.record.take();
    s.recorder.install_history(record.clone());
    // Own events the discarded line applied but the canonical one
    // lacks were confirmed by their (rogue) echo; re-queue them so
    // player input survives the rollback.
    if let Some(old) = old_record {
        let requeued = s.pending.requeue_missing_own(&old, &record);
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
    s.net.last_applied_seq = record_max;
    s.net
        .recent_uids
        .rebuild(record.events.iter().filter_map(|e| e.uid));
    // (Under a review the board shows the reviewed position: the
    // stashed live state replays the installed record instead,
    // `timeline::catch_up_live_stash`.)
    if !s.gsp.reviewing() {
        let GameStateParams {
            game_state,
            game_map,
            loaded_annotations,
            pending_map_load,
            seats,
            local_setup_ready,
            bot_driver,
            unit_paths,
            press,
            rng,
            ..
        } = &mut s.gsp;
        rebuild_state_to(
            &record,
            None,
            rng,
            game_map,
            loaded_annotations,
            &mut game_apply::EventSinks {
                game_state: &mut game_state.0,
                seats,
                local_setup_ready,
                bot_driver,
                pending_map_load,
                unit_paths,
                press,
            },
        );
    }
    // The installed line replaces ours in one jump: the counters
    // snap to it instead of playing out the difference.
    commands.insert_resource(crate::fx::SnapSprites);
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
        s.turn.game_started = true;
        s.gsp.next_app_mode.set(crate::AppMode::Game);
        if *s.state.get() == AppState::Lobby {
            s.next_state.set(AppState::InGame);
        }
    }
    // Live deliveries buffered past the old watermark may now be
    // contiguous with the installed history.
    let ready = drain_contiguous(&mut s.net, &mut s.pending);
    apply_ready(s, ready);
}

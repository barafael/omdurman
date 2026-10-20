//! Host sequencing and the receive gate: who may sequence, which seq a
//! submission gets, and whether a sequenced delivery applies now, waits in
//! the reorder buffer or is dropped.

use super::*;

/// Host sequencing is only allowed once the election has settled: the peer
/// set must have been unchanged for [`SEQ_STABILIZE_SECS`], and this peer
/// must have actual session evidence (it has seen peers, or runs in offline
/// self-host mode, which sets `has_ever_peered` at startup). See
/// `NetState::election_stable_secs` / `NetState::has_ever_peered` for why.
pub(super) fn sequencing_allowed(net: &NetState) -> bool {
    // A reconnected peer must resync before resuming host authority; the
    // gate lifts on history install or after the bootstrap budget.
    let resynced = net.resync_gate_secs <= 0.0;
    net.has_ever_peered && resynced && net.election_stable_secs >= SEQ_STABILIZE_SECS
}

/// Whether this peer now knows enough about its room to sequence: another
/// peer is connected, or the signalling server has registered us (we have an
/// id) and the room has stayed empty for [`omdurman_net::SOLO_ROOM_SECS`] --
/// a solo game against the AI. (Offline self-hosting sets the flag at
/// startup.)
pub(super) fn has_session_evidence(net: &NetState) -> bool {
    !net.peers().is_empty()
        || (!net.rejoining
            && net.my_id().is_some()
            && net.election_stable_secs >= omdurman_net::SOLO_ROOM_SECS)
}

/// The next seq this peer expects to apply: one past its watermark.
pub(super) fn expected_seq(net: &NetState) -> u32 {
    net.last_applied_seq.map_or(0, |s| s + 1)
}

/// The seq a host assigns to the next submission. `next_seq` runs ahead of
/// the watermark by the host's own in-flight echoes (sequenced this frame,
/// applied via loopback next frame); the watermark floor makes the choice
/// robust against a stale `next_seq` -- e.g. a peer promoted to host whose
/// counter was never advanced -- which would otherwise re-issue seqs that
/// already exist and have every guest dedup the new events away.
pub(super) fn next_host_seq(net: &NetState) -> u32 {
    net.next_seq.max(expected_seq(net))
}

/// Host: the seq submission `uid` already holds, if any -- recorded (its
/// echo was applied), or assigned with the echo still in flight through the
/// loopback (`NetState::host_in_flight`) -- and whether it came from the
/// record. A retransmission is then re-echoed under that seq.
pub(super) fn already_sequenced(
    net: &NetState,
    recorder: &game_record::GameRecorder,
    uid: u64,
) -> Option<(u32, bool)> {
    match recorder.seq_of_uid(uid) {
        Some(seq) => Some((seq, true)),
        None => net.host_in_flight.get(&uid).map(|&seq| (seq, false)),
    }
}

/// Host: assign submission `uid` the next seq, remembering it as in flight
/// until its echo is handled.
pub(super) fn sequence_fresh(net: &mut NetState, uid: u64) -> u32 {
    let seq = next_host_seq(net);
    net.next_seq = seq + 1;
    net.host_in_flight.insert(uid, seq);
    seq
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
pub(super) fn receive_sequenced(
    net: &mut NetState,
    recorder: &game_record::GameRecorder,
    pending: &mut PendingEdits,
    own_loopback: bool,
    delivery: SequencedDelivery,
) -> Vec<SequencedDelivery> {
    let SequencedDelivery { seq, uid, .. } = delivery;
    if own_loopback {
        // Our own echo is being handled: from here on the record (applied)
        // or nothing (dropped -- free to be sequenced afresh) answers a
        // retransmission of it, not the in-flight table. (Only the entry
        // for *this* seq: a re-echo trailing a dropped echo must not erase
        // the submission's fresh assignment.)
        if net.host_in_flight.get(&uid) == Some(&seq) {
            net.host_in_flight.remove(&uid);
        }
    }
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
                net.request_history(true);
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
            if !net.reorder.insert(seq, delivery) {
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

pub(super) fn mark_applied(net: &mut NetState, delivery: &SequencedDelivery) {
    net.recent_uids.insert(delivery.uid);
    net.last_applied_seq = Some(delivery.seq);
}

/// Pop the contiguous run of buffered deliveries starting at the next
/// expected seq (dropping anything the watermark already covers), marking
/// each applied. Called after every apply and after a history install.
pub(super) fn drain_contiguous(
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

/// Host side of an unsequenced `NetMsg::Game` submission: re-echo
/// retransmissions of already-sequenced events idempotently, hold
/// submissions while sequencing is not allowed (or while our record is
/// missing the game they belong to), otherwise assign the next sequence
/// number and echo onto the broadcast + our own loopback.
pub(super) fn host_handle_game(
    s: &mut Session,
    incoming: &mut PendingIncoming,
    held_uids: &mut std::collections::HashSet<u64>,
    uid: u64,
    ev: GameEvent,
    own_loopback: bool,
) {
    let Session {
        net,
        pending,
        turn,
        recorder,
        ..
    } = s;
    // Host-side idempotency: a retransmission of an event we
    // already sequenced -- recorded canonically, or sequenced
    // with its echo still on the way through our loopback -- is
    // re-echoed with its existing seq instead of being
    // sequenced twice.
    if let Some((seq, recorded_seq)) = already_sequenced(net, recorder, uid) {
        info!(
            seq,
            uid,
            via_record = recorded_seq,
            "host: retransmission of an already-sequenced event; re-echoing"
        );
        let sequenced = NetMsg::Sequenced {
            seq,
            uid,
            event: ev,
        };
        pending.outgoing_broadcast.push(sequenced.clone());
        incoming.loopback.push(sequenced);
        return;
    }
    // A host that has no game yet holding a guest's game action
    // is missing the game it belongs to: a newcomer that won
    // the election with an empty record. Sequencing it would
    // start a rogue line under the guests' canonical one (and
    // stop us from installing theirs): wait for their history.
    let missing_game =
        !own_loopback && !turn.game_started && !crate::net_plugin::is_session_event(&ev);
    if missing_game && !net.needs_snapshot {
        warn!(
            uid,
            "host: a guest submitted a game action but we have no game; requesting the history"
        );
        net.request_history(false);
    }
    if missing_game || !sequencing_allowed(net) {
        // The peer set may still be forming: sequencing now could
        // collide with a peer that also (briefly) believes itself
        // host. Hold the submission, in arrival order; it is
        // retried next frame, ahead of anything newer.
        // (Logged once per uid: it is re-examined every frame
        // while the gate is closed, which would spam.)
        if held_uids.insert(uid) {
            info!(
                uid,
                stable_secs = net.election_stable_secs,
                resync_gate = net.resync_gate_secs,
                missing_game,
                "host: holding submission until sequencing is allowed"
            );
        }
        incoming.hold(uid, ev, own_loopback);
        return;
    }
    held_uids.remove(&uid);
    let seq = sequence_fresh(net, uid);
    info!(seq, uid, event = ?ev, "host: sequenced submission");
    let sequenced = NetMsg::Sequenced {
        seq,
        uid,
        event: ev,
    };
    pending.outgoing_broadcast.push(sequenced.clone());
    // Push the echo onto our own loopback queue. It is *not* applied
    // here: the caller is already iterating a stream that was
    // chained from the loopback taken *above*. The just-pushed echo
    // won't be seen until the next call to this system (next frame),
    // so the host applies its own sequenced events one frame later
    // than a guest that receives the broadcast `Sequenced` echo. This
    // is intentional -- it keeps every peer (host included) on the
    // identical apply-on-echo path instead of special-casing the host
    // to apply inline.
    incoming.loopback.push(sequenced);
}

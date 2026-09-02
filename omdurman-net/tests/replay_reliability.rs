//! Empirical reliability harness for the game's P2P architecture, minified.
//!
//! This test recreates — over a *real* WebRTC mesh signalled through the
//! deployed fly.io matchbox server — the exact protocol shape the game uses:
//!
//!   * event sourcing: every participant keeps an append-only record of
//!     `(seq, event)` pairs (mini `GameRecord` / `GameRecorder`),
//!   * host relay for global ordering: submissions travel as unsequenced
//!     `Game` messages to the elected host (lowest `PeerId`), which assigns
//!     the next canonical sequence number and rebroadcasts a `Sequenced`
//!     echo to every peer; *everyone* (host included) applies on echo only
//!     (mini `net_socket::handle_socket`),
//!   * apply-once: `seq <= last_applied_seq` deliveries are dropped,
//!   * host failover: when the host disappears, the lowest remaining peer is
//!     promoted and resumes numbering at `last_applied_seq + 1`,
//!   * late join / rejoin resync: the host proactively pushes its full
//!     history to any peer that (re)connects mid-game; peers also retry
//!     `RequestSnapshot`; a received history is installed only when it is
//!     ahead of the local watermark, and acked with `SnapshotReceived`,
//!   * reconnect reset: a rejoining participant drops its socket and wipes
//!     its net state exactly like `handle_reconnect` (pending outbound and
//!     the local record included),
//!   * staging: outbound messages are staged (`PendingEdits`) and flushed
//!     every tick, retained on send failure (mini `flush_pending`).
//!
//! The shared payload is a pre-generated vector of pseudo-events; each
//! participant submits its slice, the union forms the canonical log, and
//! late joiners / rejoiners resynchronize to it via history replay. After
//! the dust settles, the harness verifies:
//!
//!   1. *consistency* — every participant's final record is identical,
//!   2. *integrity* — no duplicate seqs, no duplicate events, no seq gaps
//!      (gaps are demoted to advisories when a host failover occurred),
//!   3. *completeness* — every pseudo-event ever planned is present exactly
//!      once in the canonical record.
//!
//! Each harness parameter set runs a fresh scenario; the currently elected
//! host is *guaranteed* to be one of the rejoiners (when `rejoins >= 1`), so
//! the host-failover path is exercised every run.
//!
//! # Modes
//!
//! The harness models the current game protocol in `faithful` mode
//! (`ITEST_RETRY_FIX=0`): submissions are sent once, and `handle_reconnect`
//! drops whatever was still unconfirmed — player input in flight at a host
//! death or a rejoin is silently lost, and nothing retries it. In
//! `retry-fix` mode (default) submissions carry identity, are retransmitted
//! until confirmed, and the host re-echoes already-recorded events instead
//! of double-sequencing them — see `docs/` for the architecture notes; the
//! game gains the same guarantee via `NetMsg::Game { uid, .. }`.
//!
//! Runs are `#[ignore]`-gated (they hit the network). Execute with:
//!
//! ```sh
//! cargo test -p omdurman-net --test replay_reliability -- --ignored --test-threads=1 --nocapture
//! ```
//!
//! Tracing output goes to stdout and to `target/itest-logs/itest-<ts>.log`
//! (one file per test process; grep by the `room` field to isolate a run).
//! A per-run human-readable report is written to
//! `target/itest-logs/<room>.report.txt`.
//!
//! Parameters can be overridden per run via env vars: `ITEST_PEERS`,
//! `ITEST_EVENTS`, `ITEST_LATE`, `ITEST_REJOINS`, `ITEST_RETRY_FIX`,
//! `ITEST_DEADLINE_SECS`, `ITEST_SETTLE_SECS`, and the signalling server via
//! `MATCHBOX_SERVER`.

#![cfg(not(target_arch = "wasm32"))]

use std::{
    collections::{BTreeSet, VecDeque},
    fmt, fs,
    io::Write as _,
    sync::{
        Arc, Mutex, Once,
        atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering::SeqCst},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bevy_matchbox::prelude::MatchboxSocket;
use matchbox_socket::{PeerId, PeerState, RtcIceServerConfig, WebRtcSocketBuilder};
use omdurman_net::SIGNALING_SERVER;
use serde::{Deserialize, Serialize};
use test_case::test_case;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

// -- constants ---------------------------------------------------------------

/// Harness tick rate (the game runs `handle_socket` every frame ~16 ms).
const TICK: Duration = Duration::from_millis(50);
/// Interval between a participant's own event submissions.
const EMIT_INTERVAL: Duration = Duration::from_millis(120);
/// Delay before "late joiner" participants open their socket.
const JOIN_DELAY: Duration = Duration::from_secs(4);
/// Offline gap between dropping the socket and rejoining.
const REJOIN_GAP: Duration = Duration::from_millis(1500);
/// Stagger between participants' emission starts (burst shaping).
const START_STAGGER: Duration = Duration::from_millis(250);
/// Retransmit interval for unconfirmed event submissions (retry-fix mode).
const SUBMIT_RETRY: Duration = Duration::from_millis(500);
/// Submissions unconfirmed for this long despite retransmitting trigger a
/// forced canonical resync (the submission path itself is broken).
const STALL_RESYNC_SECS: Duration = Duration::from_secs(5);
/// A rejoining peer must install a canonical history before resuming host
/// authority (its own record was wiped); if nobody serves one within this
/// budget, the room is dead anyway and it may bootstrap on its wiped record.
const RESYNC_BOOTSTRAP: Duration = Duration::from_secs(15);
/// A peer only sequences events once its view of the peer set has been
/// unchanged for this long. Without this gate, two peers that join a room
/// near-simultaneously each elect *themselves* host while alone and
/// self-sequence their own submissions; the colliding seq numbers are then
/// silently dropped by the apply-once dedup on the other side, and the two
/// records diverge permanently. This mirrors a fix to `handle_socket`.
const SEQ_STABILIZE: Duration = Duration::from_millis(1000);
/// Log directory, relative to the workspace root (cargo test cwd).
const LOG_DIR: &str = "target/itest-logs";

// -- wire protocol (mini NetMsg) ----------------------------------------------

/// Minified stand-in for the game's `GameEvent` payloads: globally unique by
/// `(author, idx)`, with a `salt` payload that must survive every hop
/// byte-for-byte.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PseudoEvent {
    author: u8,
    idx: u32,
    salt: u64,
}

impl PseudoEvent {
    fn id(&self) -> (u8, u32) {
        (self.author, self.idx)
    }
}

/// Minified `NetMsg`: same intent split (unsequenced submission vs host-
/// sequenced echo vs control), same reliable-channel routing rules.
#[derive(Serialize, Deserialize, Clone, Debug)]
enum TestMsg {
    /// Unsequenced submission (guest -> host only; never applied directly).
    Game(PseudoEvent),
    /// Canonical host-sequenced event (host -> all). The only form that is
    /// applied and recorded.
    Sequenced {
        seq: u32,
        event: PseudoEvent,
    },
    Control(TestControl),
}

/// Minified `Control`: snapshot handshake, with the record inline instead of
/// a full `GameRecord` (no seed / timestamps needed here).
#[derive(Serialize, Deserialize, Clone, Debug)]
enum TestControl {
    RequestSnapshot,
    SnapshotReceived,
    GameHistory(Vec<(u32, PseudoEvent)>),
}

/// Mini `enc_msg`.
fn enc(msg: &TestMsg) -> Option<Box<[u8]>> {
    match postcard::to_allocvec(msg) {
        Ok(v) if !v.is_empty() => Some(v.into_boxed_slice()),
        Ok(_) => {
            error!("postcard produced an empty TestMsg encoding; dropping");
            None
        }
        Err(e) => {
            error!("postcard encode failed: {e}");
            None
        }
    }
}

/// Mini `decode`.
fn dec(raw: &[u8]) -> Option<TestMsg> {
    postcard::from_bytes(raw)
        .inspect_err(|e| warn!("postcard decode error: {e}"))
        .ok()
}

// -- scenario parameters -------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Params {
    /// Total participants (2..=10).
    peers: usize,
    /// Events each participant submits.
    events: usize,
    /// Participants that join `JOIN_DELAY` late (drawn from the tail indices).
    late: usize,
    /// Participants that abruptly drop and rejoin mid-run. The first one is
    /// always the currently elected host.
    rejoins: usize,
    /// Whether the submission-retry + host-idempotency hardening is active.
    retry_fix: bool,
}

/// Bool env flag that accepts `0`/`1`/`true`/`false` (`bool::from_str` only
/// accepts the words, so `ITEST_RETRY_FIX=0` would silently fall back).
fn env_flag(default: bool, key: &str) -> bool {
    match std::env::var(key).ok().as_deref() {
        Some("0") | Some("false") | Some("FALSE") | Some("no") => false,
        Some("1") | Some("true") | Some("TRUE") | Some("yes") => true,
        _ => default,
    }
}

fn env_or<T: std::str::FromStr>(default: T, key: &str) -> T {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

// -- the parametrized test ------------------------------------------------------

#[test_case(2, 6, 0, 0 ; "smoke_2p")]
#[test_case(2, 8, 1, 0 ; "2p_late_join")]
#[test_case(3, 8, 1, 1 ; "3p_late_join_rejoin")]
#[test_case(4, 10, 1, 1 ; "4p_rejoin")]
#[test_case(5, 10, 2, 2 ; "5p_two_rejoins")]
#[test_case(6, 10, 2, 1 ; "6p_mesh")]
#[test_case(8, 8, 2, 2 ; "8p_dense_mesh")]
#[test_case(10, 6, 3, 2 ; "10p_max_mesh")]
#[test_case(10, 12, 0, 3 ; "10p_burst")]
#[ignore = "requires network + fly.io signalling; run with: cargo test -p omdurman-net --test replay_reliability -- --ignored --test-threads=1"]
fn replay_reliability(peers: usize, events: usize, late: usize, rejoins: usize) {
    let params = Params {
        peers: env_or(peers, "ITEST_PEERS").max(2),
        events: env_or(events, "ITEST_EVENTS").max(1),
        late: env_or(late, "ITEST_LATE"),
        rejoins: env_or(rejoins, "ITEST_REJOINS"),
        retry_fix: env_flag(true, "ITEST_RETRY_FIX"),
    };
    let params = Params {
        late: params.late.min(params.peers - 1),
        rejoins: params.rejoins.min(params.peers),
        ..params
    };

    let report = run_scenario(params);
    write_report_file(&report);
    assert!(
        report.hard.is_empty(),
        "scenario {} failed ({} hard violation(s)):\n{report}",
        report.room,
        report.hard.len()
    );
}

// -- tracing setup ---------------------------------------------------------------

static INIT_TRACING: Once = Once::new();

fn init_tracing() {
    INIT_TRACING.call_once(|| {
        let dir = std::path::Path::new(LOG_DIR);
        let _ = fs::create_dir_all(dir);
        let path = dir.join(format!(
            "itest-{}.log",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let file = fs::File::options()
            .create(true)
            .append(true)
            .open(&path)
            .ok();
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("warn,matchbox_socket=info,replay_reliability=debug")
        });
        let registry = tracing_subscriber::registry().with(filter).with(
            tracing_subscriber::fmt::layer()
                .with_thread_names(true)
                .with_target(false)
                .pretty(),
        );
        match file {
            Some(f) => registry
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_thread_names(true)
                        .with_ansi(false)
                        .with_writer(move || f.try_clone().expect("log file clone"))
                        .pretty(),
                )
                .init(),
            None => registry.init(),
        }
        eprintln!("tracing logfile: {}", path.display());
    });
}

// -- shared coordination state ----------------------------------------------------

/// Cross-thread flags shared between the scenario driver and participants.
struct Shared {
    stop: AtomicBool,
    /// `connected[i]`: participant i was assigned its signalling PeerId.
    connected: Vec<AtomicBool>,
    /// `submitted[i]`: how many of participant i's own events were submitted.
    submitted: Vec<AtomicUsize>,
    /// `sequenced[i]`: how many events participant i has applied to its
    /// record (its view of the canonical log's length).
    sequenced: Vec<AtomicUsize>,
    /// `unconfirmed[i]`: how many of participant i's own submissions are
    /// still awaiting their sequenced echo.
    unconfirmed_ct: Vec<AtomicUsize>,
    /// First participant that reported being host (-1 = unknown yet).
    host_idx: AtomicI32,
    /// Driver -> participant: "drop your socket and rejoin now".
    rejoin: Vec<AtomicBool>,
    /// Driver -> participant: "you look stranded (your record lags the rest);
    /// rebuild your socket and resync".
    repair: Vec<AtomicBool>,
}

impl Shared {
    fn new(n: usize) -> Self {
        Self {
            stop: AtomicBool::new(false),
            connected: (0..n).map(|_| AtomicBool::new(false)).collect(),
            submitted: (0..n).map(|_| AtomicUsize::new(0)).collect(),
            sequenced: (0..n).map(|_| AtomicUsize::new(0)).collect(),
            unconfirmed_ct: (0..n).map(|_| AtomicUsize::new(0)).collect(),
            host_idx: AtomicI32::new(-1),
            rejoin: (0..n).map(|_| AtomicBool::new(false)).collect(),
            repair: (0..n).map(|_| AtomicBool::new(false)).collect(),
        }
    }

    /// True once every participant is at least connected to signalling.
    fn all_connected(&self, n: usize) -> bool {
        (0..n).all(|i| self.connected[i].load(SeqCst))
    }
}

// -- participant -------------------------------------------------------------------

/// Final state handed back to the verifier.
struct FinalState {
    idx: u8,
    connected_ever: bool,
    socket_dead_at_end: bool,
    is_host_at_end: bool,
    rejoins: u32,
    host_promotions: u32,
    /// The append-only canonical record (mini `GameRecord.events`).
    record: Vec<(u32, PseudoEvent)>,
    /// Events this participant submitted but never saw applied.
    unconfirmed: Vec<(u8, u32)>,
}

/// One test participant: a faithful mini-port of the game's net side
/// (`NetState` + `PendingEdits` + `PendingIncoming.loopback` + `GameRecorder`).
struct Participant {
    idx: u8,
    room: String,
    plan: VecDeque<PseudoEvent>,
    plan_total: usize,
    // -- NetState --
    peers: Vec<PeerId>,
    my_id: Option<PeerId>,
    is_host: bool,
    next_seq: u32,
    last_applied_seq: Option<u32>,
    sorted: Vec<PeerId>,
    needs_snapshot: bool,
    snapshot_applied: bool,
    snapshot_retry_at: Option<Instant>,
    // -- GameRecorder --
    record: Vec<(u32, PseudoEvent)>,
    // -- PendingEdits / PendingIncoming --
    outgoing_broadcast: Vec<TestMsg>,
    outgoing_targeted: Vec<(TestMsg, PeerId)>,
    loopback: Vec<TestMsg>,
    // -- harness state --
    socket: Option<MatchboxSocket>,
    socket_dead: bool,
    connected_ever: bool,
    unconfirmed: BTreeSet<(u8, u32, u64)>,
    next_submit_at: Instant,
    next_retry_at: Instant,
    /// When the oldest currently-unconfirmed submission was first submitted
    /// (retry-fix watchdog); `None` while nothing is pending.
    oldest_unconfirmed: Option<Instant>,
    retry_fix: bool,
    /// Set when the peer set or our own id last changed; sequencing is
    /// allowed once it has been `Some` for `SEQ_STABILIZE`.
    election_stable_since: Option<Instant>,
    /// Every event this participant ever submitted (for re-deriving lost
    /// submissions after a canonical resync).
    my_submitted: Vec<PseudoEvent>,
    /// Set when a seq conflict proves our record divergent: the next
    /// `GameHistory` is installed even if it is not "ahead".
    force_install_history: bool,
    /// Set when our record was wiped by a rejoin and not yet reinstalled
    /// from a canonical history: sequencing (host authority in particular)
    /// is blocked until the gate lifts, so a re-elected host cannot spin a
    /// rogue line off its wiped record while a superior line exists.
    resync_gate: Option<Instant>,
    rejoin_requested: bool,
    rebuild_at: Option<Instant>,
    rejoins: u32,
    host_promotions: u32,
    shared: Arc<Shared>,
}

impl Participant {
    fn new(
        idx: u8,
        room: String,
        plan: Vec<PseudoEvent>,
        emit_from: Instant,
        retry_fix: bool,
        shared: Arc<Shared>,
    ) -> Self {
        Self {
            plan_total: plan.len(),
            plan: plan.into(),
            idx,
            room,
            peers: Vec::new(),
            my_id: None,
            is_host: false,
            next_seq: 0,
            last_applied_seq: None,
            sorted: Vec::new(),
            needs_snapshot: false,
            snapshot_applied: false,
            snapshot_retry_at: None,
            record: Vec::new(),
            outgoing_broadcast: Vec::new(),
            outgoing_targeted: Vec::new(),
            loopback: Vec::new(),
            socket: None,
            socket_dead: false,
            connected_ever: false,
            unconfirmed: BTreeSet::new(),
            next_submit_at: emit_from,
            next_retry_at: emit_from,
            oldest_unconfirmed: None,
            retry_fix,
            election_stable_since: None,
            my_submitted: Vec::new(),
            force_install_history: false,
            resync_gate: None,
            rejoin_requested: false,
            rebuild_at: None,
            rejoins: 0,
            host_promotions: 0,
            shared,
        }
    }

    /// Mini `omdurman_net::build_socket` (same ICE config, same unlimited
    /// reconnect). Deliberately *no* `?next=` query: the harness needs
    /// immediate introductions for dynamic join/rejoin, not matchmade groups.
    fn open_socket(&mut self) {
        let server =
            std::env::var("MATCHBOX_SERVER").unwrap_or_else(|_| SIGNALING_SERVER.to_string());
        let server = server.trim_end_matches('/');
        let url = format!("{server}/{}", self.room);
        info!(%url, "opening matchbox socket");
        let ice = RtcIceServerConfig {
            urls: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            username: None,
            credential: None,
        };
        let builder = WebRtcSocketBuilder::new(&url)
            .ice_server(ice)
            .reconnect_attempts(None)
            .add_reliable_channel();
        self.socket = Some(MatchboxSocket::from(builder));
        self.socket_dead = false;
    }

    fn refresh_sorted(&mut self) {
        self.sorted.clear();
        self.sorted.extend(self.peers.iter().copied());
        if let Some(me) = self.my_id {
            self.sorted.push(me);
        }
        self.sorted.sort();
    }

    fn host_id(&self) -> Option<PeerId> {
        self.sorted.first().copied()
    }

    /// Mini `handle_reconnect`: drop the socket, wipe *all* net state
    /// (including the record and pending outbound), then rejoin fresh after
    /// `REJOIN_GAP`. In retry-fix mode unconfirmed submissions survive the
    /// reset and are retransmitted after rejoining.
    fn begin_rejoin(&mut self, now: Instant) {
        info!("REJOIN: dropping socket and resetting net state");
        self.socket = None;
        self.peers.clear();
        self.sorted.clear();
        self.my_id = None;
        self.is_host = false;
        self.next_seq = 0;
        self.last_applied_seq = None;
        self.record.clear();
        self.snapshot_applied = false;
        self.needs_snapshot = false;
        self.snapshot_retry_at = None;
        self.outgoing_broadcast.clear();
        self.outgoing_targeted.clear();
        self.loopback.clear();
        if !self.retry_fix {
            // Faithful game behavior: unconfirmed player input is lost.
            self.unconfirmed.clear();
        } else {
            // Everything we ever submitted is re-queued: whatever the old
            // host sequenced is still in other peers' records (we will
            // re-download it), the rest is genuinely lost and must be
            // resubmitted to the new host.
            for ev in &self.my_submitted {
                self.unconfirmed.insert((ev.author, ev.idx, ev.salt));
            }
            // Our record was wiped: actively request the canonical history
            // (any peer with a record may serve it) and block host authority
            // until it is installed.
            self.needs_snapshot = true;
            self.snapshot_retry_at = None;
            self.resync_gate = Some(Instant::now() + RESYNC_BOOTSTRAP);
        }
        self.election_stable_since = None;
        self.force_install_history = false;
        self.rejoins += 1;
        self.rejoin_requested = true;
        self.rebuild_at = Some(now + REJOIN_GAP);
    }

    /// Mini `retry_snapshot_request`: re-request every 2 s while a snapshot
    /// is outstanding.
    fn tick_snapshot_retry(&mut self, now: Instant) {
        if self.needs_snapshot && self.snapshot_retry_at.is_none_or(|at| now >= at) {
            self.snapshot_retry_at = Some(now + Duration::from_secs(2));
            info!("guest: requesting snapshot");
            self.outgoing_broadcast
                .push(TestMsg::Control(TestControl::RequestSnapshot));
        }
    }

    /// Submit planned events at the emit cadence; in retry-fix mode also
    /// retransmit everything still unconfirmed at the retry cadence.
    fn maybe_submit(&mut self, now: Instant) {
        if now >= self.next_submit_at {
            self.next_submit_at = now + EMIT_INTERVAL;
            if let Some(ev) = self.plan.front().cloned() {
                self.plan.pop_front();
                self.unconfirmed.insert((ev.author, ev.idx, ev.salt));
                self.my_submitted.push(ev.clone());
                self.outgoing_broadcast.push(TestMsg::Game(ev));
                debug!(pending = self.unconfirmed.len(), "submitted own event");
            }
        }
        if !self.retry_fix {
            return;
        }
        if now >= self.next_retry_at && !self.unconfirmed.is_empty() {
            self.next_retry_at = now + SUBMIT_RETRY;
            debug!(
                pending = self.unconfirmed.len(),
                "retransmitting unconfirmed submissions"
            );
            for &(author, idx, salt) in &self.unconfirmed {
                self.outgoing_broadcast
                    .push(TestMsg::Game(PseudoEvent { author, idx, salt }));
            }
        }
        // Watchdog: submissions still unconfirmed after `STALL_RESYNC_SECS`
        // of retrying mean the submission path itself is broken (host churn,
        // sends dropping into vanishing data channels). Request a history to
        // refresh our knowledge -- installs stay ahead-gated: a stall proves
        // nothing about the local *record*, and force-installing a shorter
        // history would wipe a possibly-canonical tail. The retransmit loop
        // keeps carrying the submissions themselves.
        if let Some(oldest) = self.oldest_unconfirmed
            && now.duration_since(oldest) >= STALL_RESYNC_SECS
        {
            warn!(
                pending = self.unconfirmed.len(),
                "submissions stalled unconfirmed; requesting canonical history"
            );
            self.oldest_unconfirmed = None;
            self.needs_snapshot = true;
            self.snapshot_retry_at = None;
        }
        self.oldest_unconfirmed = match (self.oldest_unconfirmed, self.unconfirmed.is_empty()) {
            (None, false) => Some(now),
            (Some(_), false) => self.oldest_unconfirmed,
            (_, true) => None,
        };
    }

    /// Mini `flush_pending`: stage -> send, retain on failure. The sequencer
    /// (host, or a peer with no peers at all) routes its own `Game` events
    /// through the loopback so sequencing happens in exactly one place.
    fn flush(&mut self) {
        if self.outgoing_broadcast.is_empty() && self.outgoing_targeted.is_empty() {
            return;
        }
        let host = self.host_id();

        let staged: Vec<TestMsg> = std::mem::take(&mut self.outgoing_broadcast);
        let mut to_broadcast: Vec<TestMsg> = Vec::new();
        let mut retained_broadcast: Vec<TestMsg> = Vec::new();

        for msg in staged {
            match msg {
                // The elected host sequences its own events through the
                // loopback (single serialization point). A lone host holds
                // its submissions instead: solo self-sequencing is
                // unsound (see `sequencing_allowed`).
                TestMsg::Game(event) if self.is_host && !self.peers.is_empty() => {
                    self.loopback.push(TestMsg::Game(event));
                }
                TestMsg::Game(event) => {
                    let submission = TestMsg::Game(event);
                    let sent = match (host, enc(&submission)) {
                        (Some(host), Some(encoded)) => self.socket.as_mut().is_some_and(|s| {
                            s.channel_mut(0)
                                .try_send(encoded, host)
                                .inspect_err(
                                    |e| warn!(error = %e, "submit to host failed; will retry"),
                                )
                                .is_ok()
                        }),
                        _ => false,
                    };
                    if !sent {
                        retained_broadcast.push(submission);
                    }
                }
                other => to_broadcast.push(other),
            }
        }

        let targeted: Vec<(TestMsg, PeerId)> = std::mem::take(&mut self.outgoing_targeted);
        let mut retained_targeted: Vec<(TestMsg, PeerId)> = Vec::new();
        for (msg, peer) in targeted {
            let sent = match enc(&msg) {
                Some(encoded) => self.socket.as_mut().is_some_and(|s| {
                    s.channel_mut(0)
                        .try_send(encoded, peer)
                        .inspect_err(
                            |e| warn!(error = %e, "reliable targeted send failed; will retry"),
                        )
                        .is_ok()
                }),
                None => false,
            };
            if !sent {
                retained_targeted.push((msg, peer));
            }
        }
        self.outgoing_targeted = retained_targeted;

        for msg in to_broadcast {
            if self.peers.is_empty() {
                // Mirrors the game: Sequenced with nobody to receive it is
                // dropped (the loopback echo already applied it locally);
                // everything else is retained for later peers.
                if !matches!(msg, TestMsg::Sequenced { .. }) {
                    retained_broadcast.push(msg);
                }
                continue;
            }
            let Some(encoded) = enc(&msg) else {
                retained_broadcast.push(msg);
                continue;
            };
            let mut all_ok = true;
            for &peer in &self.peers {
                let ok = self.socket.as_mut().is_some_and(|s| {
                    s.channel_mut(0)
                        .try_send(encoded.clone(), peer)
                        .inspect_err(
                            |e| warn!(error = %e, "reliable broadcast send failed; will retry"),
                        )
                        .is_ok()
                });
                if !ok {
                    all_ok = false;
                }
            }
            if !all_ok {
                retained_broadcast.push(msg);
            }
        }

        self.outgoing_broadcast = retained_broadcast;
    }

    /// Mini `handle_socket` + `NetState` maintenance. One harness tick.
    ///
    /// The socket is polled once up front into owned data (unlike the game,
    /// where the socket is its own `Resource`, here it is a field of the same
    /// struct, so the mutable borrow must end before the state machine runs).
    fn tick(&mut self, now: Instant) {
        self.maybe_submit(now);
        self.tick_snapshot_retry(now);

        enum Poll {
            None,
            Dead,
            Live {
                peer_updates: Vec<(PeerId, PeerState)>,
                socket_id: Option<PeerId>,
                received: Vec<(Option<PeerId>, TestMsg)>,
            },
        }
        let poll = match self.socket.as_mut() {
            None => Poll::None,
            Some(socket) => match socket.try_update_peers() {
                Err(_) => Poll::Dead,
                Ok(peer_updates) => {
                    let socket_id = socket.id();
                    let reliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(0).receive();
                    let received = reliable
                        .into_iter()
                        .filter_map(|(peer, raw)| match dec(&raw) {
                            Some(msg) => Some((Some(peer), msg)),
                            None => {
                                warn!("unknown message, ignoring");
                                None
                            }
                        })
                        .collect();
                    Poll::Live {
                        peer_updates,
                        socket_id,
                        received,
                    }
                }
            },
        };
        let (peer_updates, socket_id, received) = match poll {
            Poll::None => return,
            Poll::Dead => {
                warn!("socket message loop ended; socket is dead");
                self.socket_dead = true;
                return;
            }
            Poll::Live {
                peer_updates,
                socket_id,
                received,
            } => (peer_updates, socket_id, received),
        };

        // -- peer updates --
        let mut peers_changed = false;
        let mut newly_connected: Vec<PeerId> = Vec::new();
        for (peer, state) in peer_updates {
            match state {
                PeerState::Connected if !self.peers.contains(&peer) => {
                    self.peers.push(peer);
                    newly_connected.push(peer);
                    peers_changed = true;
                    info!(%peer, "peer connected");
                }
                PeerState::Disconnected => {
                    let before = self.peers.len();
                    self.peers.retain(|&p| p != peer);
                    peers_changed |= self.peers.len() != before;
                    info!(%peer, "peer disconnected");
                }
                _ => {}
            }
        }

        let my_id_changed = socket_id.is_some_and(|id| Some(id) != self.my_id);
        if my_id_changed {
            info!(new_id = ?socket_id, "signalling assigned our PeerId");
            self.my_id = socket_id;
            if socket_id.is_some() {
                self.connected_ever = true;
                self.shared.connected[self.idx as usize].store(true, SeqCst);
            }
        }
        if peers_changed || my_id_changed {
            self.refresh_sorted();
            // Peer-set view changed: host elections re-derive and the
            // stabilization window restarts.
            self.election_stable_since = None;
        } else if self.election_stable_since.is_none() && self.my_id.is_some() {
            self.election_stable_since = Some(now);
        }

        /// Sequencing requires (a) at least one peer -- a lone peer cannot
        /// know whether a session already exists elsewhere in the room, so
        /// solo self-sequencing is unsound (this mirrors the game's lobby
        /// discipline: events only flow once the roster is visible) -- and
        /// (b) a peer-set view that has been stable for `SEQ_STABILIZE` (see
        /// the const's docs for why).
        fn sequencing_allowed(
            peers: &[PeerId],
            stable_since: Option<Instant>,
            resync_gate: Option<Instant>,
            now: Instant,
        ) -> bool {
            let stable = stable_since.is_some_and(|t| now.duration_since(t) >= SEQ_STABILIZE);
            // A rejoined peer must resync before resuming host authority;
            // the gate lifts on history install or after the bootstrap
            // budget (nobody left to serve a history).
            let resynced = resync_gate.is_none_or(|deadline| now >= deadline);
            !peers.is_empty() && stable && resynced
        }

        // Host election + failover promotion: resume canonical numbering at
        // one past the highest sequence this peer ever applied.
        if let Some(my_id) = self.my_id
            && (peers_changed || my_id_changed)
        {
            let new_host_is_me = self.sorted.first() == Some(&my_id);
            let promoted = new_host_is_me && !self.is_host;
            if promoted {
                self.next_seq = self.last_applied_seq.map_or(0, |s| s + 1);
                self.host_promotions += 1;
                info!(
                    next_seq = self.next_seq,
                    "promoted to host after previous host disconnect; resumed sequence numbering"
                );
            }
            self.is_host = new_host_is_me;
            if self.is_host {
                self.shared.host_idx.store(self.idx as i32, SeqCst);
            }
        }

        let mut targeted: Vec<(TestMsg, PeerId)> = Vec::new();
        let mut sequenced_out: Vec<TestMsg> = Vec::new();

        // Host: proactively push the canonical record to any peer that just
        // connected (late joiner / WebRTC blip catch-up).
        if self.is_host && !newly_connected.is_empty() && !self.record.is_empty() {
            for peer in newly_connected {
                info!(%peer, "host: pushing game history to (re)connected peer");
                targeted.push((
                    TestMsg::Control(TestControl::GameHistory(self.record.clone())),
                    peer,
                ));
            }
        }

        // Host loopback: its own sequenced echoes flow through the identical
        // apply path, one tick later (apply-on-echo for everyone).
        let loopback: Vec<TestMsg> = std::mem::take(&mut self.loopback);
        let is_host = self.is_host;
        let decoded = received
            .into_iter()
            .chain(loopback.into_iter().map(|msg| (None::<PeerId>, msg)));

        for (peer, msg) in decoded {
            match msg {
                TestMsg::Game(ev) => {
                    if !is_host {
                        // Transient election disagreement: re-forward to
                        // whoever we currently consider host (retained when
                        // none is known).
                        match self.host_id() {
                            Some(host) => {
                                warn!(
                                    "received unsequenced Game event but we are not host; re-forwarding to current host"
                                );
                                targeted.push((TestMsg::Game(ev), host));
                            }
                            None => {
                                warn!(
                                    "received unsequenced Game event but we are not host and no host is known; retaining for retry"
                                );
                                self.outgoing_broadcast.push(TestMsg::Game(ev));
                            }
                        }
                        continue;
                    }
                    if !sequencing_allowed(
                        &self.peers,
                        self.election_stable_since,
                        self.resync_gate,
                        now,
                    ) {
                        // The peer set may still be forming: sequencing now
                        // could collide with a peer that also believes itself
                        // host. Hold the submission until the election is
                        // stable.
                        debug!("host: holding submission until the election is stable");
                        self.outgoing_broadcast.push(TestMsg::Game(ev));
                        continue;
                    }
                    // Host-side idempotency (retry-fix): an event already
                    // sequenced -- recorded canonically, or still in flight
                    // in this tick's batch -- is re-echoed with its existing
                    // seq instead of being sequenced twice.
                    let in_flight = sequenced_out.iter().find_map(|m| match m {
                        TestMsg::Sequenced { seq, event } if event.id() == ev.id() => Some(*seq),
                        _ => None,
                    });
                    let recorded = if self.retry_fix {
                        self.record
                            .iter()
                            .rev()
                            .find(|(_, e)| e.id() == ev.id())
                            .map(|(s, _)| *s)
                    } else {
                        None
                    };
                    if let Some(seq) = in_flight.or(recorded) {
                        debug!(
                            seq,
                            "host: resubmission of already-recorded event; re-echoing"
                        );
                        let sequenced = TestMsg::Sequenced { seq, event: ev };
                        sequenced_out.push(sequenced.clone());
                        self.loopback.push(sequenced);
                        continue;
                    }
                    let seq = self.next_seq;
                    self.next_seq += 1;
                    debug!(seq, event = ?ev.id(), "host: sequenced submission");
                    let sequenced = TestMsg::Sequenced { seq, event: ev };
                    sequenced_out.push(sequenced.clone());
                    self.loopback.push(sequenced);
                }
                TestMsg::Sequenced { seq, event } => {
                    // Apply-once: any seq at or below the highest applied is a
                    // duplicate delivery -- unless our record disagrees with
                    // it. A *different* event at that seq (two hosts sequenced
                    // competing streams) or *no* event at all (our watermark
                    // sits on a stale, higher-numbered rogue line) proves the
                    // local record divergent: on the canonical line the record
                    // is contiguous up to the watermark, so any mismatch means
                    // we are off-line and must resync.
                    if self.last_applied_seq.is_some_and(|last| seq <= last) {
                        if self
                            .event_at(seq)
                            .is_none_or(|recorded| recorded.id() != event.id())
                        {
                            warn!(seq, theirs = ?event.id(), "SEQ CONFLICT: canonical delivery disagrees with local record");
                            self.handle_seq_conflict();
                        } else if event.author == self.idx {
                            // A host re-echo of our own event at its recorded
                            // seq: application is deduped away, but the
                            // confirmation must not be -- otherwise we would
                            // retransmit forever.
                            self.unconfirmed
                                .remove(&(event.author, event.idx, event.salt));
                        }
                        continue;
                    }
                    // Identity dedup (retry-fix): the same event must never
                    // be applied twice even if hosts transiently sequenced it
                    // under different numbers.
                    if self.retry_fix && self.applies_event(&event) {
                        debug!(seq, event = ?event.id(), "dropping sequenced delivery of already-applied event");
                        if event.author == self.idx {
                            self.unconfirmed
                                .remove(&(event.author, event.idx, event.salt));
                        }
                        continue;
                    }
                    // Gap detection (retry-fix): a delivery that jumps past
                    // `last_applied + 1` means we missed events (e.g. the
                    // broadcast raced a reconnecting data channel). Apply
                    // what arrived, but immediately request the canonical
                    // history and force-install it: our record is known to
                    // be incomplete, so the "install only if ahead" check
                    // must not apply. The host is exempt: its own line is
                    // canonical by election, and force-installing a shorter
                    // foreign history over it would regress every guest.
                    if self.retry_fix && self.last_applied_seq.is_some_and(|last| seq > last + 1) {
                        warn!(seq, last = ?self.last_applied_seq, "seq gap detected; requesting canonical history");
                        if self.is_host {
                            continue;
                        }
                        self.needs_snapshot = true;
                        self.snapshot_retry_at = None;
                        self.force_install_history = true;
                    }
                    self.last_applied_seq = Some(seq);
                    self.record.push((seq, event.clone()));
                    if event.author == self.idx
                        && self
                            .unconfirmed
                            .remove(&(event.author, event.idx, event.salt))
                    {
                        info!(seq, idx = event.idx, "own event confirmed");
                    }
                }
                TestMsg::Control(TestControl::RequestSnapshot) => {
                    // Single-source installs: only the elected host serves
                    // arbitrary requesters -- and, for the reconnected-host
                    // deadlock (its record was wiped by its own rejoin while
                    // the superior line lives on the guests), a guest serves
                    // its *own host*. Anyone else serving would let rogue
                    // lines masquerade as canonical.
                    let requester_is_my_host =
                        peer.is_some() && self.host_id().is_some() && self.host_id() == peer;
                    if !self.is_host && !requester_is_my_host {
                        continue;
                    }
                    if self.record.is_empty() {
                        continue;
                    }
                    info!("serving game history to a requester");
                    if let Some(peer) = peer {
                        targeted.push((
                            TestMsg::Control(TestControl::GameHistory(self.record.clone())),
                            peer,
                        ));
                    }
                }
                TestMsg::Control(TestControl::SnapshotReceived) => {
                    info!("host: late joiner acknowledged game history");
                }
                TestMsg::Control(TestControl::GameHistory(rec)) => {
                    // An authoritative host never installs foreign histories:
                    // its own line is canonical by election, and a longer
                    // rogue line would wipe its tail and desynchronize its
                    // numbering baseline. The one exception is the resync
                    // gate (a freshly rejoined host with a wiped record,
                    // which must re-download the canonical line).
                    if self.is_host && self.resync_gate.is_none() {
                        debug!("host: ignoring foreign game history");
                        continue;
                    }
                    // Install only when ahead of the local watermark (covers
                    // both fresh late joiners and peers that fell behind); a
                    // stale duplicate is ignored. A detected seq conflict
                    // flips `force_install_history`: our own record is known
                    // to be divergent, so the host's record wins even if it
                    // is not "ahead" by seq alone.
                    let record_max = rec.iter().map(|(s, _)| *s).max();
                    let ahead = match (record_max, self.last_applied_seq) {
                        (Some(hi), Some(applied)) => hi > applied,
                        (Some(_), None) => true,
                        (None, _) => false,
                    };
                    if !ahead && !self.force_install_history {
                        info!("ignoring game history that is not ahead of local state");
                        continue;
                    }
                    if self.force_install_history {
                        info!(
                            events = rec.len(),
                            "conflict resync: force-installing canonical history"
                        );
                    }
                    self.force_install_history = false;
                    self.needs_snapshot = false;
                    self.snapshot_retry_at = None;
                    info!(
                        events = rec.len(),
                        "received game history, replaying to resync"
                    );
                    if let Some(peer) = peer {
                        targeted.push((TestMsg::Control(TestControl::SnapshotReceived), peer));
                    }
                    self.record = rec;
                    self.last_applied_seq = record_max;
                    self.resync_gate = None;
                    // Anything we submitted that IS in the installed record
                    // was sequenced canonically: confirmed. Events that were
                    // only ever confirmed by a rogue/lost host are re-derived
                    // from `my_submitted` and stay/become unconfirmed so the
                    // retransmit loop re-submits them to the current host.
                    let recorded: BTreeSet<(u8, u32)> =
                        self.record.iter().map(|(_, e)| e.id()).collect();
                    for (_, e) in &self.record {
                        if e.author == self.idx {
                            self.unconfirmed.remove(&(e.author, e.idx, e.salt));
                        }
                    }
                    for ev in &self.my_submitted {
                        if !recorded.contains(&ev.id()) {
                            self.unconfirmed.insert((ev.author, ev.idx, ev.salt));
                        }
                    }
                }
            }
        }

        self.outgoing_targeted.extend(targeted);
        self.outgoing_broadcast.extend(sequenced_out);
        self.flush();
    }

    fn applies_event(&self, event: &PseudoEvent) -> bool {
        self.record.iter().any(|(_, e)| e.id() == event.id())
    }

    fn event_at(&self, seq: u32) -> Option<&PseudoEvent> {
        self.record.iter().find(|(s, _)| *s == seq).map(|(_, e)| e)
    }

    /// A `Sequenced` delivery collided with a different event at the same
    /// seq: two hosts sequenced competing streams. If we are not the current
    /// host, our record may be the divergent one -- request the canonical
    /// history and force-install it when it arrives. The host itself keeps
    /// its record: it is the canonical sequencer by election.
    fn handle_seq_conflict(&mut self) {
        if self.is_host {
            warn!("seq conflict but we are the elected host; keeping canonical record");
            return;
        }
        if !self.retry_fix {
            return;
        }
        self.force_install_history = true;
        self.needs_snapshot = true;
        self.snapshot_retry_at = None; // request immediately on this tick
    }

    fn publish_status(&self) {
        self.shared.submitted[self.idx as usize].store(self.plan_total - self.plan.len(), SeqCst);
        self.shared.sequenced[self.idx as usize].store(self.record.len(), SeqCst);
        self.shared.unconfirmed_ct[self.idx as usize].store(self.unconfirmed.len(), SeqCst);
    }

    fn finalize(self) -> FinalState {
        FinalState {
            idx: self.idx,
            connected_ever: self.connected_ever,
            socket_dead_at_end: self.socket_dead,
            is_host_at_end: self.is_host,
            rejoins: self.rejoins,
            host_promotions: self.host_promotions,
            record: self.record,
            unconfirmed: self.unconfirmed.iter().map(|&(a, i, _)| (a, i)).collect(),
        }
    }
}

// -- scenario driver ---------------------------------------------------------------

fn run_scenario(params: Params) -> Report {
    let started = Instant::now();
    init_tracing();
    let room = format!(
        "itest-{}-p{}e{}l{}r{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        params.peers,
        params.events,
        params.late,
        params.rejoins,
    );
    let run = tracing::info_span!("run", %room, peers = params.peers, events = params.events,
        late = params.late, rejoins = params.rejoins, retry_fix = params.retry_fix);
    let _run_guard = run.enter();

    let deadline_secs: u64 = env_or(150, "ITEST_DEADLINE_SECS");
    let deadline = started + Duration::from_secs(deadline_secs);

    // The shared canonical plan: participant i owns slice [i*k, (i+1)*k).
    let k = params.events;
    let plan: Vec<PseudoEvent> = (0..(params.peers * k) as u32)
        .map(|i| PseudoEvent {
            author: (i / k as u32) as u8,
            idx: i % k as u32,
            salt: rand::random(),
        })
        .collect();

    let shared = Arc::new(Shared::new(params.peers));
    let finals: Arc<Mutex<Vec<Option<FinalState>>>> =
        Arc::new(Mutex::new((0..params.peers).map(|_| None).collect()));

    info!("spawning participants");
    let mut handles = Vec::new();
    for i in 0..params.peers {
        let is_late = i >= params.peers - params.late;
        let my_plan: Vec<PseudoEvent> = plan[i * k..(i + 1) * k].to_vec();
        let shared = shared.clone();
        let finals = finals.clone();
        let room = room.clone();
        let join_delay = if is_late { JOIN_DELAY } else { Duration::ZERO };
        let emit_from = Instant::now() + START_STAGGER * (i as u32);
        let retry_fix = params.retry_fix;
        handles.push(
            thread::Builder::new()
                .name(format!("peer-{i}"))
                .spawn(move || {
                    let span = tracing::info_span!("peer", peer = i, room = %room.clone());
                    let _g = span.enter();
                    if join_delay > Duration::ZERO {
                        info!(?join_delay, "late joiner: delaying socket creation");
                        let until = Instant::now() + join_delay;
                        while Instant::now() < until && !shared.stop.load(SeqCst) {
                            thread::sleep(TICK);
                        }
                    }
                    let mut p = Participant::new(
                        i as u8,
                        room,
                        my_plan,
                        emit_from,
                        retry_fix,
                        shared.clone(),
                    );
                    p.open_socket();
                    while !shared.stop.load(SeqCst) && Instant::now() < deadline {
                        let now = Instant::now();
                        if shared.rejoin[i].load(SeqCst) && !p.rejoin_requested {
                            shared.rejoin[i].store(false, SeqCst);
                            p.begin_rejoin(now);
                        }
                        if shared.repair[i].load(SeqCst) && !p.rejoin_requested {
                            shared.repair[i].store(false, SeqCst);
                            warn!("REPAIR: record lags the mesh; rebuilding socket to resync");
                            p.begin_rejoin(now);
                        }
                        if let Some(at) = p.rebuild_at
                            && now >= at
                        {
                            p.rebuild_at = None;
                            info!("rejoining room with a fresh socket");
                            p.open_socket();
                        }
                        p.tick(now);
                        p.publish_status();
                        thread::sleep(TICK);
                    }
                    let fin = p.finalize();
                    info!(
                        events = fin.record.len(),
                        rejoins = fin.rejoins,
                        promotions = fin.host_promotions,
                        unconfirmed = fin.unconfirmed.len(),
                        "participant finished"
                    );
                    finals.lock().unwrap()[i] = Some(fin);
                })
                .expect("spawn participant thread"),
        );
    }

    // -- monitor loop: designate rejoins, wait until everyone finished emitting --
    let mut designated: Vec<usize> = Vec::new();
    let mut all_done_since: Option<Instant> = None;
    while Instant::now() < deadline {
        if designated.len() < params.rejoins && shared.all_connected(params.peers) {
            let host = shared.host_idx.load(SeqCst);
            if host >= 0 {
                let host = host as usize;
                if designated.is_empty() {
                    // Rejoin the host only once the mesh has actually formed
                    // around it and it has *sequenced* at least two events:
                    // designating earlier (e.g. while the host is still alone
                    // and merely staging emissions) would tear the room apart
                    // mid-formation and split the mesh into components with
                    // competing host views.
                    if shared.sequenced[host].load(SeqCst) >= 2 {
                        info!(%host, "designating the elected host for a mid-run rejoin");
                        shared.rejoin[host].store(true, SeqCst);
                        designated.push(host);
                    }
                } else {
                    // Space rejoins out: the previous designee must have
                    // resynced (applied events again) before the next one
                    // drops, otherwise the churn compounds.
                    let last = designated[designated.len() - 1];
                    let second = (host + 1) % params.peers;
                    if !designated.contains(&second)
                        && shared.sequenced[last].load(SeqCst) >= 2
                        && shared.connected[second].load(SeqCst)
                        && shared.submitted[second].load(SeqCst) >= 2
                    {
                        info!(
                            peer = second,
                            "designating a non-host peer for a mid-run rejoin"
                        );
                        shared.rejoin[second].store(true, SeqCst);
                        designated.push(second);
                    }
                }
            }
        }
        let all_done = (0..params.peers).all(|i| {
            shared.connected[i].load(SeqCst) && shared.submitted[i].load(SeqCst) == params.events
        });
        if all_done {
            // On fast-emitting runs the mesh may still be forming (large
            // meshes take seconds to handshake); give pending designations a
            // grace window instead of tearing down immediately.
            if designated.len() == params.rejoins
                || all_done_since.is_some_and(|s| s.elapsed() >= Duration::from_secs(5))
            {
                break;
            }
            if all_done_since.is_none() {
                all_done_since = Some(Instant::now());
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    // -- convergence phase: wait until every participant's record has the
    // same length (a good convergence proxy; content is verified at the end)
    // and repair stranded participants (record lagging the mesh, e.g. a
    // silently dead data channel) by commanding a socket rebuild. Capped by
    // the deadline so a hopeless repair still terminates deterministically.
    let settle: u64 = env_or(8, "ITEST_SETTLE_SECS");
    let repair_grace = Duration::from_secs(5);
    let mut lagging_since: Option<Instant> = None;
    let convergence_started = Instant::now();
    let mut last_progress = Instant::now();
    let mut last_lengths: Vec<usize> = Vec::new();
    loop {
        let now = Instant::now();
        let lengths: Vec<usize> = (0..params.peers)
            .map(|i| shared.sequenced[i].load(SeqCst))
            .collect();
        if lengths != last_lengths {
            last_lengths = lengths.clone();
            last_progress = now;
        }
        let max_len = *lengths.iter().max().unwrap_or(&0);
        let converged = lengths.iter().all(|&l| l == max_len);
        // A participant whose own submissions never confirm while the record
        // lengths agree is stranded on a dead uplink (content-wise divergence
        // is invisible to length comparison): repair it too.
        let stuck: Vec<usize> = (0..params.peers)
            .filter(|&i| shared.unconfirmed_ct[i].load(SeqCst) > 0)
            .collect();
        let converged = converged && stuck.is_empty();
        if converged {
            lagging_since = None;
        } else if max_len > 0 {
            // Only repair once everyone has actually finished emitting:
            // during emission, lagging records are normal (events in flight).
            let all_emitted =
                (0..params.peers).all(|i| shared.submitted[i].load(SeqCst) == params.events);
            if all_emitted {
                let mut lagging: Vec<usize> = lengths
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| **l < max_len)
                    .map(|(i, _)| i)
                    .collect();
                lagging.extend(stuck.iter().copied());
                lagging.sort_unstable();
                lagging.dedup();
                match lagging_since {
                    None => lagging_since = Some(now),
                    Some(since) if now.duration_since(since) >= repair_grace => {
                        for &i in &lagging {
                            if !shared.rejoin[i].load(SeqCst) && !shared.repair[i].load(SeqCst) {
                                warn!(
                                    peer = i,
                                    len = lengths[i],
                                    max_len,
                                    "monitor: peer lagging; requesting socket rebuild"
                                );
                                shared.repair[i].store(true, SeqCst);
                            }
                        }
                        lagging_since = Some(now);
                    }
                    _ => {}
                }
            }
        }
        if converged && now.duration_since(convergence_started) >= Duration::from_secs(2) {
            break;
        }
        // Give up on convergence when the deadline hits, the phase budget is
        // exhausted, or everything has been static for a while (no progress
        // and no pending repairs -- further waiting cannot help).
        let pending_repairs = (0..params.peers)
            .any(|i| shared.repair[i].load(SeqCst) || shared.rejoin[i].load(SeqCst));
        if now >= deadline
            || now.duration_since(convergence_started) >= Duration::from_secs(settle)
                && now.duration_since(last_progress) >= Duration::from_secs(8)
                && !pending_repairs
        {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    info!(
        settle_secs = settle,
        "emission complete; settling to drain in-flight traffic"
    );
    thread::sleep(Duration::from_secs(3));
    shared.stop.store(true, SeqCst);
    for h in handles {
        h.join().expect("participant thread panicked");
    }

    let finals: Vec<FinalState> = {
        let mut guard = finals.lock().unwrap();
        guard
            .iter_mut()
            .map(|f| f.take().expect("participant wrote final state"))
            .collect()
    };
    let host_rejoined = designated.first().is_some_and(|&i| finals[i].rejoins > 0);
    let wall = started.elapsed();
    verify(params, &plan, finals, host_rejoined, designated, wall, room)
}

// -- verification -------------------------------------------------------------------

#[derive(Debug)]
enum Violation {
    NeverConnected {
        peer: u8,
    },
    SocketDeadAtEnd {
        peer: u8,
    },
    RecordsDiffer {
        a: u8,
        b: u8,
        detail: String,
    },
    DuplicateSeq {
        peer: u8,
        seq: u32,
    },
    DuplicateEvent {
        peer: u8,
        id: (u8, u32),
    },
    SeqGap {
        peer: u8,
        missing: Vec<u32>,
        seqs: Vec<u32>,
    },
    MissingEvents {
        events: Vec<(u8, u32)>,
    },
    UnexpectedEvents {
        events: Vec<(u8, u32)>,
    },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::NeverConnected { peer } => write!(f, "peer {peer} never connected"),
            Violation::SocketDeadAtEnd { peer } => {
                write!(f, "peer {peer} socket was dead at the end")
            }
            Violation::RecordsDiffer { a, b, detail } => {
                write!(f, "records of peer {a} and peer {b} differ: {detail}")
            }
            Violation::DuplicateSeq { peer, seq } => {
                write!(f, "peer {peer} recorded seq {seq} twice")
            }
            Violation::DuplicateEvent { peer, id } => {
                write!(f, "peer {peer} recorded event {id:?} twice")
            }
            Violation::SeqGap {
                peer,
                missing,
                seqs,
            } => {
                let (m_head, m_tail) = split_dump(missing);
                let (s_head, s_tail) = split_dump(seqs);
                write!(
                    f,
                    "peer {peer} record has {} seq gaps: [{m_head} ... {m_tail}] ({} seqs: [{s_head} ... {s_tail}])",
                    missing.len(),
                    seqs.len(),
                )
            }
            Violation::MissingEvents { events } => {
                write!(f, "canonical record is missing events: {events:?}")
            }
            Violation::UnexpectedEvents { events } => {
                write!(f, "canonical record contains unplanned events: {events:?}")
            }
        }
    }
}

struct Report {
    room: String,
    params: Params,
    wall: Duration,
    designated_rejoins: Vec<usize>,
    finals: Vec<FinalState>,
    hard: Vec<Violation>,
    advisory: Vec<Violation>,
}

fn verify(
    params: Params,
    plan: &[PseudoEvent],
    finals: Vec<FinalState>,
    host_rejoined: bool,
    designated: Vec<usize>,
    wall: Duration,
    room: String,
) -> Report {
    let mut hard = Vec::new();
    let mut advisory = Vec::new();

    for f in &finals {
        if !f.connected_ever {
            hard.push(Violation::NeverConnected { peer: f.idx });
        }
        if f.socket_dead_at_end {
            hard.push(Violation::SocketDeadAtEnd { peer: f.idx });
        }
    }

    // Pick the canonical record: the longest one (ties -> lowest peer idx).
    let mut by_len = finals.iter().collect::<Vec<_>>();
    by_len.sort_by_key(|f| (std::cmp::Reverse(f.record.len()), f.idx));
    let canonical = by_len.first().expect("at least one participant");

    // 1. consistency: every record identical to the canonical one.
    let mut canonical_sorted = canonical.record.clone();
    canonical_sorted.sort_by_key(|(s, _)| *s);
    for f in &finals {
        if f.idx == canonical.idx {
            continue;
        }
        let mut other = f.record.clone();
        other.sort_by_key(|(s, _)| *s);
        if other != canonical_sorted {
            let detail = first_divergence(&canonical_sorted, &other);
            hard.push(Violation::RecordsDiffer {
                a: canonical.idx,
                b: f.idx,
                detail,
            });
        }
    }

    // 2. integrity of the canonical record.
    let mut seen_seq = BTreeSet::new();
    let mut seen_id = BTreeSet::new();
    for &(seq, _) in &canonical_sorted {
        if !seen_seq.insert(seq) {
            hard.push(Violation::DuplicateSeq {
                peer: canonical.idx,
                seq,
            });
        }
    }
    let mut missing_seqs = Vec::new();
    for (i, &(seq, _)) in canonical_sorted.iter().enumerate() {
        if seq != i as u32 {
            missing_seqs.extend(i as u32..seq);
        }
    }
    if !missing_seqs.is_empty() {
        let v = Violation::SeqGap {
            peer: canonical.idx,
            missing: missing_seqs,
            seqs: canonical_sorted.iter().map(|(s, _)| *s).collect(),
        };
        if host_rejoined {
            advisory.push(v);
        } else {
            hard.push(v);
        }
    }
    for (_, e) in &canonical_sorted {
        if !seen_id.insert(e.id()) {
            hard.push(Violation::DuplicateEvent {
                peer: canonical.idx,
                id: e.id(),
            });
        }
    }

    // 3. completeness: every planned event present exactly once.
    let planned: BTreeSet<(u8, u32)> = plan.iter().map(|e| e.id()).collect();
    let missing: Vec<(u8, u32)> = planned.difference(&seen_id).copied().collect();
    if !missing.is_empty() {
        hard.push(Violation::MissingEvents { events: missing });
    }
    let unexpected: Vec<(u8, u32)> = seen_id.difference(&planned).copied().collect();
    if !unexpected.is_empty() {
        hard.push(Violation::UnexpectedEvents { events: unexpected });
    }

    Report {
        room,
        params,
        wall,
        designated_rejoins: designated,
        finals,
        hard,
        advisory,
    }
}

/// Render at most the first and last 15 elements of a diagnostic dump.
fn split_dump(v: &[u32]) -> (String, String) {
    const N: usize = 15;
    if v.len() <= 2 * N {
        let all = v
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        (all.clone(), all)
    } else {
        (
            v[..N]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            v[v.len() - N..]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

fn first_divergence(a: &[(u32, PseudoEvent)], b: &[(u32, PseudoEvent)]) -> String {
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        if x != y {
            return format!("index {i}: {x:?} vs {y:?}");
        }
    }
    format!("lengths {} vs {}", a.len(), b.len())
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "room            : {}", self.room)?;
        writeln!(f, "params          : {:?}", self.params)?;
        writeln!(f, "wall time       : {:.1}s", self.wall.as_secs_f32())?;
        writeln!(f, "rejoin designees: {:?}", self.designated_rejoins)?;
        for fin in &self.finals {
            writeln!(
                f,
                "peer {:2}: record={:3} unconfirmed={:2} rejoins={} promotions={} host_at_end={} connected={}",
                fin.idx,
                fin.record.len(),
                fin.unconfirmed.len(),
                fin.rejoins,
                fin.host_promotions,
                fin.is_host_at_end,
                fin.connected_ever,
            )?;
        }
        if !self.advisory.is_empty() {
            writeln!(f, "advisories ({}):", self.advisory.len())?;
            for v in &self.advisory {
                writeln!(f, "  [~] {v}")?;
            }
        }
        if self.hard.is_empty() {
            writeln!(
                f,
                "result: OK (all records identical, complete, no duplicates)"
            )
        } else {
            writeln!(f, "result: FAILED ({} hard violation(s))", self.hard.len())?;
            for v in &self.hard {
                writeln!(f, "  [!] {v}")?;
            }
            Ok(())
        }
    }
}

fn write_report_file(report: &Report) {
    let dir = std::path::Path::new(LOG_DIR);
    let _ = fs::create_dir_all(dir);
    let path = dir.join(format!("{}.report.txt", report.room));
    if let Ok(mut f) = fs::File::create(&path) {
        let _ = writeln!(f, "{report}");
        eprintln!("report: {}", path.display());
    }
}

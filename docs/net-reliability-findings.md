# Net-reliability findings and fixes

Empirical results from `omdurman-net/tests/replay_reliability.rs` — a minified but faithful
recreation of the game's event-sourced, host-relayed P2P protocol, run over real WebRTC meshes
signalled by the deployed fly.io matchbox server. The harness shares a pre-generated vector of
pseudo-events among 2–10 participants (`test_case`-parameterized), forces late joins and mid-run
rejoins (always including the currently elected host), and verifies at the end that every
participant's canonical record is **identical**, **complete** (every event exactly once) and free
of duplicate seqs/events.

```sh
cargo test -p omdurman-net --test replay_reliability -- --ignored --test-threads=1 --nocapture
```

Each finding below was first *observed as a harness failure* with log evidence, then fixed on the
architectural level — first in the harness, then ported to the game (`omdurman-net`,
`omdurman-app`), then re-validated.

## Findings

### 1. Join-window split-brain (permanent, silent divergence)

* **Symptom** (2-peer smoke run): both peers record 4 events, all records differ at index 0,
  half the planned events missing forever. Log: both peers log `promoted to host ... next_seq: 0`
  while alone, then `dropping duplicate/late sequenced delivery` on every echo from the other side.
* **Root cause**: a peer elects itself host the moment it is *alone* (its `sorted_all` contains
  only itself). Two peers joining near-simultaneously each self-sequence their own submissions
  from seq 0 before the mesh forms; the apply-once dedup (`seq <= last_applied`) then silently
  drops the other line's echoes. Both sides are frozen on divergent, colliding streams. The game's
  `handle_socket` had exactly the same window.
* **Fix (architectural)**:
  * **Election stabilization** — a host only sequences when its peer-set view has been unchanged
    for `SEQ_STABILIZE_SECS` (1 s): `NetState::election_stable_secs`, gate in the host `Game` arm
    of `net_socket::handle_socket` (held submissions are retried).
  * **Session evidence** — sequencing additionally requires `NetState::has_ever_peered` (or
    offline self-host mode). A peer that has never seen the roster must not sequence: a lone peer
    cannot know whether a session already exists elsewhere in the room. This is the network-side
    analogue of the lobby's own discipline (`all_players_ready` requires both factions, so a
    networked game cannot start solo anyway).
  * Solo self-sequencing (`i_sequence` when `peers.is_empty()`) was removed from
    `flush_pending`/harness flush: submissions are retained until a host is visible.

### 2. matchbox message-loop panics (session-killing transport bug)

* **Symptom** (4-peer rejoin runs): `thread 'matchbox' panicked: couldn't find data channel for
  peer` (and later `channel_ready.try_send(()).unwrap()` in the `on_open` callback). Every
  panic killed the socket's *entire* message loop — every peer connection of that instance — and
  the participant went permanently dead (`socket message loop ended; socket is dead` repeated).
* **Root cause**: an outgoing packet is queued via `try_send` while the target peer is connected,
  but dequeued after that peer's connection was torn down (rejoin under a new id, WebRTC blip).
  The fork treated this routine race as a logic error and panicked.
* **Fix**: the fork is vendored at `vendor/matchbox` (`[patch."https://github.com/barafael/matchbox.git"]`
  in the root `Cargo.toml`, pinned to the same rev `a7ed87b`). Two robustness fixes:
  * the outgoing-queue drain drops the packet with a warning when the peer's data channel is gone;
  * the data-channel `on_open` callback tolerates a torn-down handshake receiver.
  Upper layers treat reliable sends as best-effort and already retransmit.

### 3. Player input lost at host death / rejoin

* **Symptom**: canonical record missing a participant's entire plan (`unconfirmed=N` on that
  peer at the end); worst in runs where the designated host rejoined mid-burst.
* **Root cause**: submissions were sent once. If the host died (or its data channels died)
  between a successful `try_send` and sequencing, the event vanished. `handle_reconnect`
  additionally cleared `PendingEdits` — pending player input was discarded by design.
* **Fix (architectural)**: submissions carry identity and are retried until confirmed.
  * `NetMsg::Game { uid, event }` / `NetMsg::Sequenced { seq, uid, event }`; the uid is a
    per-process random base + counter, assigned once per submission via
    `PendingEdits::submit_game` (all ~20 construction sites in the app now go through it).
  * `PendingEdits::unconfirmed` retransmits every `SUBMIT_RETRANSMIT_SECS` (0.5 s) and **survives
    reconnects** (only the staged copies are cleared).
  * The host dedupes retransmissions idempotently: an event already recorded (or still in flight
    in the same frame batch) is re-echoed with its *existing* seq instead of being sequenced twice.
  * Receive-side identity dedup (`NetState::recent_uids`, bounded ring of 4096) makes events
    double-sequenced under different numbers apply exactly once.
  * `RecordedEvent` gained `uid: Option<u64>` (serde-defaulted; old `games/*.jsonl` still load).

### 4. Unhealable divergence: poisoned watermarks refused the cure

* **Symptom** (4-peer rejoin): peer 0 permanently missing seq 19 while everyone else had it;
  later, a peer on a *stale higher-numbered rogue line* (watermark above the canonical max) that
  nothing could ever drag back — canonical deliveries were dropped by the apply-once check,
  history installs were refused by the "install only if ahead" check, and none of the local
  proof-of-brokenness signals fired (no gap: nothing above its watermark; no conflict: it had no
  local event at those seqs; no unconfirmed: its events were confirmed by the rogue).
* **Root cause**: healing relied solely on the "install only if ahead" comparison, and the
  conflict check only covered "seq used with a *different* event".
* **Fix (architectural)** — the receive path now detects two proof-of-brokenness conditions and
  reacts with a forced canonical resync:
  * **Seq conflict**: a `Sequenced` at `seq <= last_applied` whose event *differs or is absent*
    locally. On the canonical line the record is contiguous up to the watermark, so any mismatch
    proves the local record divergent (this includes watermarks sitting on dead rogue lines).
  * **Seq gap**: a delivery at `seq > last_applied + 1` (broadcasts racing a reconnecting data
    channel).
  * Either sets `needs_snapshot` + `force_install_history`: the canonical record is installed
    *unconditionally* (the local record is known-bad), own events missing from it are re-queued
    into `unconfirmed` for resubmission, and — because the event log *is* the state — the rebuild
    absorbs the rollback (`rebuild_state_to`).

### 5. One-way channel death (frozen guest)

* **Symptom** (10-peer, 3 rejoins): one peer converged to the canonical *content* except its own
  12 events, `unconfirmed=12`, no disconnect ever observed. It received everything (installed the
  108-event history) while every packet it sent to the host vanished; its retransmissions and
  snapshot requests all travelled the same dead link.
* **Root cause**: guests only ever talk to the host. If the host-directed channel dies without a
  disconnect event, no application-level mechanism can reach the host: every retry uses the same
  path. Only a fresh connection can restore the session.
* **Fix (architectural)**: `auto_reconnect_on_stall` — submissions unconfirmed for
  `SUBMIT_STALL_RECONNECT_SECS` (10 s, ≈20 dead retransmit rounds) while `InGame` insert
  `ReconnectRoom` (same room), rebuilding the socket through the standard `handle_reconnect`
  path. The host's proactive history push then resyncs the rebuilt connection; surviving
  `unconfirmed` submissions are retransmitted afterwards. The history install also returns a
  mid-game reconnectee to `InGame` instead of stranding them in the lobby (whose `StartGame` is
  already history).

### 6. The split-host deadlock (a host is never resyncable)

* **Symptom** (6-peer, host rejoin): the rejoining peer was re-elected host (its fresh id really
  was the lowest), but its record had been wiped by its own rejoin — it re-sequenced only its own
  10 events and stayed there forever, while the guests held the complete 60-event line built by
  the interim host during the 1.5 s gap. All six peers *agreed* on the view; the session still
  deadlocked.
* **Root cause**: hosts never request snapshots and guests never serve them — so the only peer
  that needed the canonical line (the host) was the one peer that could never receive it. Host
  privilege plus a wiped record equals a permanently short canonical line.
* **Fix (architectural)**:
  * **RequestSnapshot is served by guests for their own host**: the elected host serves arbitrary
    requesters; a guest additionally serves *its own host*. The rejoined host now actively
    requests (`needs_snapshot` on rejoin) and re-downloads the canonical line.
  * **Resync gate**: a rejoined peer must install a canonical history before resuming host
    authority (`NetState::resync_gate_secs` / harness `resync_gate`), so it cannot spin a rogue
    line off its wiped record while a superior line exists. The gate lifts on history install or
    after a bootstrap budget (`RESYNC_BOOTSTRAP_SECS`, 15 s — nobody left to serve means the room
    is dead anyway).
  * Installs stay ahead-gated, so a lagging guest's answer cannot regress anyone.

### 7. Regression: multi-source installs (rogue lines masquerading as canonical)

* **Symptom** (a full round failing 3 of 9 right after an overly broad fix): franken-records like
  seqs `[0..87, 115..155]`, dual lines at the tail, divergence at high seqs.
* **Root cause**: an earlier draft let *any* peer serve `RequestSnapshot`. Histories then arrived
  from multiple sources; ahead-gated and force-installed foreign lines (including rogue-but-longer
  ones) merged incompatible streams. A force-install also raced in-flight `Sequenced`
  deliveries: the record was replaced while the watermark had already advanced past it, and the
  next promotion inherited the desynchronized baseline.
* **Fix (architectural)**: installs are **single-source** — only the elected host serves
  arbitrary requesters, and guests serve only their own host (finding 6's deadlock case). Force
  installs (conflict/gap) therefore always adopt the elected line, never a rogue one.

### 8. Watchdog force-install wiped a canonical tail

* **Symptom** (6-peer): the canonical record skipped exactly ten seqs (`[0..29, 40..59]`); the
  log showed the host re-sequencing a retransmit burst of already-recorded events into the holes
  (`seq: 30, event: (0,0)` ... `seq: 39, event: (0,9)`), which every guest then identity-dropped.
* **Root cause**: the stall watchdog (finding 5's trigger) set `force_install_history` — but a
  stall proves the *submission path* is broken, not that the local *record* is wrong. The host's
  own broadcast request was served a *shorter* history by a peer, and the host force-installed it
  over its own canonical tail. Its `next_seq` kept counting from the old watermark, so the next
  retransmissions landed in the wiped range and became permanent holes.
* **Fix (architectural)**:
  * The watchdog requests a history but installs stay **ahead-gated** (no force flag).
  * **The host is exempt from force-installs entirely**: on a detected gap (or conflict) the host
    keeps its line — it is canonical by election — and foreign streams jumping past its watermark
    are ignored as dual-host artifacts. Guests force-install; hosts never do.

### 9. Harness defect: `ITEST_RETRY_FIX=0` never parsed

`"0".parse::<bool>()` fails, so the faithful-mode flag silently defaulted to *fix mode*; early
"faithful baselines" were mislabelled. Fixed with an `env_flag` helper accepting
`0/1/true/false/no/yes`. With the flag working, faithful mode fails on rejoin scenarios exactly as
the pre-fix architecture predicts (the rejoined host strands on its wiped record with no resync
machinery).

## Harness-side hardening (scenario scheduling)

Several early "failures" were harness scheduling bugs that nonetheless informed the protocol
work; they are fixed inside the test:

* Host-rejoin designation now requires **all participants connected** and the host to have
  *sequenced* ≥ 2 events (designating while the mesh was still forming tore the room apart and
  split it into components with competing host views).
* Rejoin designations are **spaced**: the previous designee must have resynced (applied events
  again) before the next one drops.
* The settle phase is **convergence-driven**: the monitor waits until every record length agrees
  *and* no participant has unconfirmed submissions (length-equality alone cannot see
  content divergence), commands a socket rebuild for lagging/stuck peers, and gives up only when
  the phase budget is exhausted *and* nothing has progressed for 8 s.

## Validation

* Faithful mode (`ITEST_RETRY_FIX=false`, the pre-fix protocol) fails on rejoin scenarios exactly
  as predicted: the rejoined host strands on its wiped record (`records of peer 1 and peer 0
  differ: lengths 35 vs 0`). This is the documented baseline: the pre-fix architecture "usually
  works".
* Fix mode: after the fixes, 8+ consecutive full-suite rounds green (72+ scenarios), including
  10-peer meshes with three rejoins, a 10-peer × 40-event (400-event) triple-rejoin long run, and
  targeted reruns of every previously failing case.
* `cargo test --workspace` fully green (rules 358, app 42, traceability 3, bot, tools); clippy
  clean on touched code; `wasm32-unknown-unknown` check passes (CI's `trunk build --release`
  path).

## Parameters

`ITEST_PEERS`, `ITEST_EVENTS`, `ITEST_LATE`, `ITEST_REJOINS`, `ITEST_RETRY_FIX`
(`0`/`false` = faithful pre-fix protocol), `ITEST_SETTLE_SECS`, `ITEST_DEADLINE_SECS`,
`MATCHBOX_SERVER`.

Tracing goes to stdout plus a per-process logfile under `omdurman-net/target/itest-logs/`;
per-run reports land next to it (`<room>.report.txt`). Grep by the `room` field to isolate a run.

# Instructions

## Project

Rust/Bevy implementation of *Remember Gordon! — The Battle of Omdurman* (Phoenix Enterprises, 1982).
Runs natively and as a WASM web app (deployed to GitHub Pages via Trunk).
Networking is peer-to-peer via `bevy_matchbox` (WebRTC + a `wss://` signalling server).

## Common commands

Build / run the game native:

```shell
cargo run -p omdurman-app
```

Build / serve the WASM web build:

```shell
trunk serve
trunk build --release
```

Run a single rules test or a single test by name:

```shell
cargo test -p omdurman-rules
cargo test -p omdurman-rules $test_name_substring
```

The rulebook <-> code traceability check (see "Traceability" below):

```shell
cargo test -p omdurman-rules --test traceability
```

Build the traceability report locally (CI builds and publishes it; it is not committed):

```shell
cargo run -p traceability-typst -- docs/traceability.toml traceability.typ tools/traceability-typst/data.json
typst compile traceability.typ traceability.pdf
```

Run the mutation gate (the CI job runs it on the change's diff):

```shell
cargo run -p traceability-lsp --bin mutation-gate -- --section §5.51
git diff --no-ext-diff main > /tmp/change.diff   # a plain unified diff, as CI makes
cargo run -p traceability-lsp --bin mutation-gate -- --in-diff /tmp/change.diff --jobs 2
```

(`--no-ext-diff`: an external diff tool's output is no unified diff. The gate
normalizes path prefixes itself, so `diff.mnemonicPrefix` is fine.)

Run the Kani proof suite (see `docs/architecture.md` §9). Kani has no native Windows
support, so on Windows this shells into WSL. The script bakes in `-Z stubbing` and
`--features kani` (gates the engine's `debug!` call sites out of the proof build);
`KANI_JOBS=<N>` verifies harnesses in parallel:

```shell
./scripts/kani.sh -p omdurman-types -p omdurman-rules
KANI_JOBS=8 ./scripts/kani.sh -p omdurman-types -p omdurman-rules
./scripts/kani.sh -p omdurman-rules --harness verification::die_roll_apply_modifier_is_total
```

The expensive tier (every effect kind's safety, movement legality, effect pairs, wire format,
sequencing; hours and tens of GB per job) needs `--features kani-expensive`; `run-kani.sh`
clones the repo and runs everything on a big machine (`KANI_EXPENSIVE=1 ./run-kani.sh`). The
same harnesses run under `cargo test` as a randomized test (`EXPENSIVE_SAMPLES=<n>` for more).

CI (`.github/workflows/ci.yml`) runs, per push/PR: `cargo fmt --check`, `cargo clippy
--workspace --all-targets -- -D warnings`, `cargo test --workspace`, the traceability
gates (plus the report PDF as an artifact) and the mutation gate — plus the Pages deploy,
which also publishes the report. The Kani suite is *not* a push/PR gate: GitHub
runners kept killing the job with shutdown signals (exit 143, all harnesses green every
time), so the job is gated to `workflow_dispatch` — run it manually when wanted, and
locally via the script above (the authoritative check). CI builds with
`trunk build --release` for the `wasm32-unknown-unknown` target — keep that working
when changing dependencies.
The toolchain is pinned via `rust-toolchain.toml` (stable 1.98.0 + `wasm32-unknown-unknown` target;
the Aug-2026 nightly breaks bevy_render 0.19.1). Bump it deliberately, after a full
`cargo test --workspace` + `trunk build --release`.
The signalling server URL is bakeable via the `MATCHBOX_SERVER` env var at build time (see `omdurman-net/src/lib.rs`).

## Workspace layout

Seven workspace crates plus the traceability tooling (two tools and a proc-macro), all sharing `edition = "2024"`:

- **`omdurman-types`** — leaf crate, no Bevy. Pure serde types shared by everything else (`HexCoord`,
  `SectionName` (+ `SHEET_ORDER`, the canonical counter-sheet section order used by the picker), `MapData`, hexside/Nile/overlay types, `Faction`, `Brigade`),
  plus the net layer's sequencing primitives (`net_seq`: `RecentUids`, `ReorderBuffer`, here so
  Kani can prove them without Bevy).
  Must stay dependency-light so both the rules engine and the net layer can depend on it.
- **`omdurman-rules`** — the rules engine. No Bevy. Defines `GameState`, `GameEffect`, and
  `apply_effect`: every legal mutation flows through `effects::apply_effect`. Effects carry
  pre-rolled dice so the same effect applied on every peer yields the same state. Submodules:
  engine core — `effects/` (a module directory: `effect.rs` the `GameEffect` enum, `error.rs`,
  `observation.rs`, `state.rs` (`GameState` + accessors) with its validators split by domain into
  `state/{setup,movement,fire,melee,engineering,stacking}.rs` (extra `impl GameState` blocks), `dispatch.rs` (`apply_effect` +
  turn flow), `movement.rs` / `fire.rs` / `melee.rs` / `setup.rs` / `river.rs` / `victory.rs`
  (per-domain `apply_*` functions), `tests.rs`; the root `effects.rs` re-exports the flat API),
  `board` (mapless `BoardInfo` topology), `board_data`
  (RON-backed board accessors), `los_table`, `range_effects`,
  `combat_results_table`, `howitzer_scatter`, `turn_track`, `reinforcements`, `unit_id`,
  `terrain_chart` (the Terrain Effects Chart: hex costs/modifiers, road links, hexside
  surcharges and fire modifiers -- the single step-cost rule `land_step_cost` shared by the
  engine, the app's plot/preview surfaces and the bot; parity-checked against the
  transcription `Boardgame - Remember_Gordon/tables/terrain_effects_chart.ron`),
  `tactics` (scripted-playthrough fixtures reused by tests and the bot), `unit_profiles`
  (compiled per-counter roster), `sprite_data` (compiled sprite fallbacks), `tables_data`
  (the four rules tables as `static` consts, transcribed from the RON files under
  `Boardgame - Remember_Gordon/tables/` and parity-checked against them by `#[cfg(test)]`
  tests — no runtime parse, so Kani can reason over the table-backed functions), `rng`
  (`GameRng`, the local dice source the app and the bot both draw from — the app wraps it in a
  Bevy `Resource` newtype), plus
  presentation-adjacent data used by the app:
  `newspaper`, `press` (the deterministic telegrams and Gazette: `telegram`, `gazette`,
  `phrases`), `turn_summary`. The crate-root types are split into private
  modules re-exported from `lib.rs` (`scalars`, `turn`, `unit`, `combat`, `transport`, `victory`;
  tests in `tests.rs`, Kani proofs in `verification.rs`), so public paths stay
  `omdurman_rules::X`. Most rulebook constants are `value_enum!` enums (in `scalars.rs`) so
  match arms are exhaustive at compile time.
- **`omdurman-board-ui`** — board-view plumbing (split out when a map editor shared it; that
  editor is retired, so the app is its only user): RTS camera, input/raycast helpers, egui
  pointer gating (`EguiPointerOverUi` snapshot + `MapPointerInputSet`), night shading
  (driven by the injected `BoardDayNight` resource), the two-board store + board
  bootstrap + map plane. Binaries keep only their small
  local `Plugin` wiring and app-specific hooks (e.g. the game attaches `BoardInfo` to the
  engine state on every board load).
- **`omdurman-hexmap`** — Bevy plugin (`HexMapPlugin`) for the hex grid: `GameMap`, `HexLayout`,
  `MapDims`, world-space conversion, plus the shared board plane (`MapPlane`, `MapTextureCache`,
  `HexOverlay`, `apply_map_data_to_plane`, `terrain_overlay_color`) used by the game. `HexLayout` must be inserted manually with calibration data.
- **`omdurman-net`** — net glue. Defines `NetMsg`, `GameEvent`, `GameRecord` (event log),
  `InitialGameState`, and `room_id()`. `GameEvent` variants are the *only* messages recorded into
  the canonical event log and replayed for late joiners — adding a variant here automatically
  participates in recording/replay.
- **`omdurman-app`** — the Bevy game binary (`omdurman`). Game only: rendering, input, egui UI,
  camera, networking glue, and the event-viewer debug overlay. Entry point:
  `omdurman-app/src/main.rs`.
- **`omdurman-bot`** — bot / strategy advisor over the rules engine.
- **`tools/traceability-typst`** (and `tools/traceability-lsp`) — regenerates the traceability
  PDF / serves live traceability diagnostics.

## Architecture: event-sourced, peer-to-peer, host-relayed

The system is a deterministic event-sourced engine over a peer-to-peer mesh:

1. **Rules engine is authoritative.** Game mutations are `GameEffect`s; `apply_effect` validates
   and mutates `GameState`. Dice are rolled *before* the effect is constructed and embedded in it,
   so re-applying the effect on any peer reproduces the same state.
2. **Host relays for global ordering.** A peer wishing to act sends an unsequenced event; the host
   assigns a sequence number and broadcasts it (`NetMsg::Sequenced`). Every peer (including the
   host) applies events *only* when they receive the sequenced echo — `apply-on-echo`. The host's
   `loopback` queue in `PendingIncoming` feeds its own outgoing sequenced events through the same
   receive path so it doesn't apply them twice or apply them out of order.
3. **Canonical event log.** `GameRecord` records every `GameEvent` in order. Late joiners are sent
   the record and replay it to converge to current state. Live echoes and replays
   (`timeline::rebuild_state_to`) share one apply function, `game_apply::apply_game_event`, called
   synchronously in seq order — including the engine half of `PlaceUnit`/`MoveUnit`/`RemoveUnit`.
   Unit sprites are a projection of `GameState` (`picker::reconcile_unit_sprites`), never a
   second source of truth.
4. **Outbound staging.** `PendingEdits` buffers reliable broadcasts and targeted sends so multiple
   systems can stage messages without contending for `&mut MatchboxSocket`. Game-event submissions
   must go through `PendingEdits::submit_game`, which assigns a submission-unique `uid` (random
   per-process base + counter) carried by `NetMsg::Game`/`NetMsg::Sequenced`. The host routes its own
   outgoing game events through `incoming.loopback` as unsequenced `NetMsg::Game`, so `handle_socket`
   sequences them through the same arm as guest submissions (single serialization point). Recording
   happens via `GameRecorder::push_event` on the `NetMsg::Sequenced` echo — the host records on echo
   exactly like every other peer, preserving the apply-on-echo invariant.
   Unconfirmed submissions stay in `PendingEdits::unconfirmed` and are retransmitted
   (`SUBMIT_RETRANSMIT_SECS`) until their echo arrives, so player input survives a host death or an
   in-flight send loss; the host re-echoes already-sequenced uids idempotently instead of
   double-sequencing them.
   Unreliable traffic (cursor positions, ephemeral selections) bypasses staging.
5. **Election stabilization.** A host only sequences when its peer-set view has been unchanged for
   `SEQ_STABILIZE_SECS` *and* it has session evidence (`NetState::has_ever_peered`, or offline
   self-host mode). Without this gate, two peers joining near-simultaneously each briefly elect
   themselves host, self-sequence their own submissions, and the colliding seqs are silently dropped
   by the other side's apply-once dedup — a permanent divergence.
6. **Divergence healing.** The receive path detects two proof-of-brokenness conditions. A *seq
   conflict* is a `Sequenced` at an already-applied seq carrying a different event (transient
   dual-host streams), or with no local event at that seq at all (our watermark sits on a stale,
   higher-numbered rogue line). A *seq gap* is a jump past `last_applied + 1` (broadcasts racing a
   reconnecting data channel). A conflict immediately forces a `RequestSnapshot` and a
   `force_install_history` install of the canonical record: the local record is known-bad, so the
   "install only if ahead" check must not apply. A guest parks a gap's deliveries in the
   `ReorderBuffer` and applies them in order once the gap fills; only a gap that outlives
   `SEQ_GAP_TIMEOUT_SECS` forces the same snapshot + forced install. A host ignores a foreign gap
   as a dual-host artifact. After an install, own events missing from the installed record are
   re-queued for resubmission. Identity dedup (`NetState::recent_uids`, bounded) makes double-sequenced events
   apply exactly once. The event log is the state, so the rebuild absorbs the rollback
   (`rebuild_state_to`; the history install also returns a mid-game reconnectee to `InGame`).
7. **Stall auto-reconnect.** Submissions unconfirmed for `SUBMIT_STALL_RECONNECT_SECS` while
   `InGame` insert `ReconnectRoom` (same room), rebuilding the socket through the standard
   `handle_reconnect` path. This covers the one-way channel death a guest cannot otherwise detect:
   every retransmission and snapshot request travels the same dead link, so only a fresh connection
   (and the host's proactive history push) restores the session.
8. **Dice travel in the events; the PRNG is local.** Determinism does *not* depend on any
   shared PRNG position: the acting peer rolls the dice and embeds them in the `GameEffect`,
   so replay never draws random numbers. `omdurman_rules::rng::GameRng(ChaCha8Rng)` is each
   peer's *local* roll source — seeded from the fresh per-peer seed in its own record header
   (`InitialGameState`) at startup, and reseeded from fresh entropy after every history
   install / rebuild (`rebuild_state_to`), so a reconnected peer never repeats the rolls
   already made at the start of the game. The implementation lives in the rules crate; the
   app wraps it in a Bevy `Resource` newtype (`omdurman-app/src/state.rs`) and the bot uses
   the same implementation via `BotRng` — one dice-stream implementation, not mirrored copies.

## Architecture: seats, stable player keys, pause, claims

- **`PlayerKey`** (`omdurman-net`) is a player's identity across reconnects (the matchbox `PeerId`
  changes whenever the socket is rebuilt). Native: persisted in a locked slot file
  (`<config dir>/omdurman/player_key_<n>`, `omdurman-app/src/player_key_store.rs`) — a relaunch
  after quitting or crashing reclaims the seat, while concurrently running instances take distinct
  slots (`OMDURMAN_PLAYER_SLOT=<n>` pins one). Web: per browser tab in `sessionStorage` (survives
  a reload). Announced in `Ephemeral::PlayerInfo { name, color,
  key }` (reliable, targeted on connect) and stored on peer entities as `PeerPlayerKey`.
- **Seats.** `GameEvent::StartGame { seats: Vec<Seat>, scenario, optional_rules }`;
  `Seat { faction, scope: Option<CommandScope>, holder: SeatHolder::{Human(PlayerKey), Ai} }`.
  The app's `seats::Seats` resource is written *only* by `game_apply::apply_game_event`, so live
  and replay agree; `peers::Peers` keeps its API (`local`, `may_act`, `scope_allows`,
  `is_spectator`, ...) over `Seats` + `LocalPlayerKey`. A returning player rebinds automatically
  when the history replays. The AI plays factions whose seats are all AI seats.
- **Pause.** `seats::SeatPresence` (local, unrecorded) pauses the game while any human seat holder
  is disconnected (`Peers::may_act` returns false; the host AI waits); after 60 s the seat is
  abandoned.
- **Host-arbitrated seat events.** Guests never submit seat events: they send
  `Control::SeatRequest`; the host (`seat_arbiter::seat_control`, `seats::VoteBook`) grants an
  abandoned-seat claim outright and puts takeovers / AI hand-overs / AI reclaims to a unanimous
  vote of the other connected seated humans, then submits `GameEvent::SeatAssigned` /
  `SeatCarved` (apply arms re-validate deterministically). Seat events and `StartGame` are
  *session events*: accepted unchecked by the submit dry run and never re-queued by
  `PendingEdits::requeue_missing_own`.
- **Wire format.** New `Control` / `GameEvent` variants are *appended* (postcard encodes the
  variant index); the `StartGame` / `PlayerInfo` reshape means all peers must run the same build.

## Empirical net-reliability harness

`omdurman-net/tests/replay_reliability.rs` recreates this protocol in miniature over a real WebRTC
mesh signalled by the deployed fly.io matchbox server: up to 10 participants (`test_case`
parameterized), late joiners, and mid-run rejoins that always include the currently elected host
(forcing failover). It verifies at the end that every participant's record is identical, complete
(every pseudo-event present exactly once) and free of duplicate seqs/events. It is `#[ignore]`-gated
(network); run with:

```sh
cargo test -p omdurman-net --test replay_reliability -- --ignored --test-threads=1 --nocapture
```

Tracing goes to stdout plus `omdurman-net/target/itest-logs/`; per-run reports land next to it.
Parameters can be overridden per run via `ITEST_PEERS`, `ITEST_EVENTS`, `ITEST_LATE`,
`ITEST_REJOINS`, `ITEST_RETRY_FIX` (set 0 for the faithful pre-fix protocol), `ITEST_SETTLE_SECS`,
`ITEST_DEADLINE_SECS`, and `MATCHBOX_SERVER`.

The matchbox dependency comes directly from the `barafael/matchbox` fork (`branch = "main"` in the
root `Cargo.toml`) with three robustness fixes: the socket message loop no longer panics — it drops
the outgoing packet with a warning — when an outgoing send races a peer teardown, a
data-channel `on_open` callback no longer panics after handshake teardown, and a signaling-loop
failure after the initial connect no longer tears down the socket — established peer connections
live entirely on WebRTC data channels, so a signaling server restart mid-game is logged and
survived instead of silently ending every in-progress session (the fly.io auto-stop of
`omdurman-matchbox` triggers exactly this; new peers can't join until the app rebuilds the socket,
but the game continues). The first two panics previously killed every connection of the socket and
were routinely triggered by the harness around rejoins.

## Architecture: dual-map (campaign + Fall-of-Khartoum)

The game uses two boards, switched by `MapKind` (`Campaign`, `FallOfKhartoum`).
`ActiveEditMap` tracks which one is live.
`PendingMapLoad` is set by the `StartGame` handler (and the board reconciler) to (re)load a board
on the next frame; `omdurman-app/src/board_state.rs` owns this bootstrap.

## Board + sprite data (RON data files)

The two boards live as RON data files under `omdurman-app/assets/boards/`
(`campaign.ron`, `fall_of_khartoum.ron`) — edited as text (the map editor that authored them was retired in October 2026 and lives
in git history), embedded at compile time
by `omdurman-rules/src/board_data.rs` (the single `include_str!` owner), and parsed once on first
use. The app's `LoadedAnnotations` and the tactics fixtures both consume those accessors.
Sprite metadata lives in `omdurman-rules/src/sprite_data.rs` (compiled, keyed by `UnitId`
position, one global block). Cut sprite images live under `omdurman-app/assets/sprites/`.

## Mode switching (UI)

The top-level `AppMode`s are `Menu`, `Lobby`, and `Game`.
The splash screen provides the primary mode-switching UI.
There is no in-app editor — the board and asset data files are edited as text.

## Traceability

`docs/traceability.toml` is the rulebook index: one `[[mapping]]` per manual section, naming the
code that implements it, its tests and proofs, and for `implemented` sections the manual's rule
the code enforces (`clause`, quoted verbatim) and the one test or proof whose job it is
(`witness`). It is an index, not a proof of correctness -- the evidence is the witnesses and the
mutation gate. `cargo test -p omdurman-rules --test traceability` enforces (checks shared with
the editor LSP in `tools/traceability-lsp/src/checks.rs`):

- `implemented` mappings list `[[mapping.impl]]` sites (`file`, `symbol` -- no line numbers;
  the symbol must occur in the file's code, comments don't count) and a `clause` found verbatim
  in the manual section's text plus a `witness` listed in its `tests`/`proofs`. An optional
  `approximation` says where the code deliberately departs from the clause.
- Every `§N` citation in Rust source names a mapped section, and mappings and manual sections
  correspond in both directions.
- Every cited symbol is compiler-anchored in `omdurman-rules/tests/traceability_paths.rs`
  (a real `use`/item path -- a rename breaks the build), and every anchor there is cited.
- **Coverage is hard**: every `implemented` mapping lists at least one `tests = [...]` entry
  (fully qualified `crate::module::fn_name`, the file path as module path) whose test carries a
  `#[rulebook("§N")]` attribute for that section and is not `#[ignore]`d. The attribute is the
  only annotation that counts; a `§` in a comment is a citation, never coverage.
- **Kani proofs** are tracked the same way in a `proofs = [...]` array, bijective in both
  directions. They use the qualified `#[traceability_macro::rulebook("§N")]` (the macro is a
  `cfg(kani)`-only dependency of the proof crates). The PDF renders proofs in blue above the
  green tests.
- **The mutation gate** (CI job `mutation-gate`, `cargo run -p traceability-lsp --bin
  mutation-gate [-- --in-diff <diff>]`): every cargo-mutants mutant on a changed line of an
  engine function a section cites must fail one of the engine tests of the sections citing it.
  Accepted equivalent mutants go in `.cargo/mutants.toml`, each with a reason.

When adding code that implements a new rulebook section, cite the section in a comment
(`(rulebook §6.11)`) *and* add the matching `[[mapping]]` with a clause, a witness and at least
one annotated test. Container headings get `status = "descriptive"` (no impls/tests). When
renaming a symbol, update its `symbol` field in `traceability.toml` **and** the anchor in
`traceability_paths.rs`, or the build will fail. Moving code needs no matrix change.

The report (`traceability.typ`, `traceability.pdf`, `tools/traceability-typst/data.json`) is
generated, not committed: CI builds it on every run (downloadable artifact) and the Pages
deploy publishes it at `https://barafael.github.io/omdurman/traceability.pdf`. Locally:
`cargo run -p traceability-typst -- docs/traceability.toml traceability.typ
tools/traceability-typst/data.json && typst compile traceability.typ traceability.pdf`.

### Traceability PDF layout fidelity

The template (`tools/traceability-typst/traceability-template.typ`) renders the manual from
`data.json` using a `#list`/`#enum` function path that must reproduce the old markup path's
layout pixel-identically. Two data-driven flags on every list/enum block control spacing:

- **`blank_before`** (bool) — `true` when the source had a blank line immediately before
  this list/enum. The template emits a `#parbreak()` before the list only when this is true
  (adding the ~19pt gap the markup path produces). Without it, lists attached directly under
  paragraphs (no blank line) get the gap wrongly.
- **`loose`** (bool) — `true` when the source had a blank line *anywhere* between the
  list's items. A loose list in Typst uses paragraph spacing (~18.5pt) between items instead
  of the tight leading gutter (~11.5pt). The template sets `tight: not b.loose`.

These are set automatically by the parser in `main.rs` (`parse_list` for `loose`,
`parse_manual_blocks` for `blank_before`). If you add a new list or enum to the manual in
`traceability.toml`, the flags are picked up on regeneration — no manual intervention needed.

## Conventions to preserve

- The rules engine uses `value_enum!` (defined at the top of `omdurman-rules/src/lib.rs`, used mostly in `scalars.rs`) for any
  quantitative value with a fixed annotated set of possibilities. Match arms then stay
  exhaustive — prefer extending `value_enum!` over adding an `_ =>` arm.
- `omdurman-types` and `omdurman-rules` have no Bevy dependency. Keep it that way; Bevy lives in
  `omdurman-app` (and `omdurman-hexmap` for its plugin shim).
- `GameEvent` is the only enum whose variants get recorded and replayed. Adding a network message
  that should *not* persist (cursor pings, selection hints) belongs in `Ephemeral`, not
  `GameEvent`.
- New game mutations must go through a new `GameEffect` variant + `apply_effect` arm so they
  participate in determinism, replay, and host-relay ordering.

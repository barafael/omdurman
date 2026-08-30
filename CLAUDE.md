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

Run the (native-only) map editor tool:

```shell
cargo run -p map-editor
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

Regenerate `traceability.typ` / `data.json` (the crate has two binaries, so `--bin` is
required; `traceability.pdf` then needs the `typst` CLI):

```shell
cargo run -p traceability-typst --bin traceability-typst -- docs/traceability.toml traceability.typ tools/traceability-typst/data.json
```

Run the Kani proof suite (see `docs/architecture.md` §9). Kani has no native Windows
support, so on Windows this shells into WSL:

```shell
./scripts/kani.sh -p omdurman-types -p omdurman-rules
./scripts/kani.sh -p omdurman-rules --harness verification::die_roll_apply_modifier_is_total
```

CI builds with `trunk build --release` for the `wasm32-unknown-unknown` target — keep that working
when changing dependencies.
The toolchain is pinned via `rust-toolchain.toml` (stable 1.98.0 + `wasm32-unknown-unknown` target;
the Aug-2026 nightly breaks bevy_render 0.19.1). Bump it deliberately, after a full
`cargo test --workspace` + `trunk build --release`.
The signalling server URL is bakeable via the `MATCHBOX_SERVER` env var at build time (see `omdurman-net/src/lib.rs`).

## Workspace layout

Six workspace crates plus three tools, all sharing `edition = "2024"`:

- **`omdurman-types`** — leaf crate, no Bevy. Pure serde types shared by everything else (`HexCoord`,
  `SectionName` (+ `SHEET_ORDER`, the canonical counter-sheet section order shared by the picker
  and the map editor), `MapData`, `SpriteAnnotation`, hexside/Nile/overlay types, `Faction`, `Brigade`).
  Must stay dependency-light so both the rules engine and the net layer can depend on it.
- **`omdurman-rules`** — the rules engine. No Bevy. Defines `GameState`, `GameEffect`, and
  `apply_effect`: every legal mutation flows through `effects::apply_effect`. Effects carry
  pre-rolled dice so the same effect applied on every peer yields the same state. Submodules:
  engine core — `effects` (the `GameEffect` enum, `GameState`, validators, and the per-effect
  `apply_*` functions; the single largest module), `board` (mapless `BoardInfo` topology), `board_data`
  (RON-backed board accessors), `los_table`, `range_effects`, `terrain_chart`,
  `combat_results_table`, `howitzer_scatter`, `turn_track`, `reinforcements`, `unit_id`,
  `tactics` (scripted-playthrough fixtures reused by tests and the bot), `unit_profiles`
  (compiled per-counter roster), `sprite_data` (compiled sprite fallbacks), `tables_data`
  (macro-embedded RON tables), plus presentation-adjacent data used by the app:
  `newspaper`, `telegram_prompt`, `turn_summary`. Most rulebook constants live as `value_enum!`
  enums in `lib.rs` so match arms are exhaustive at compile time.
- **`omdurman-hexmap`** — Bevy plugin (`HexMapPlugin`) for the hex grid: `GameMap`, `HexLayout`,
  `MapDims`, world-space conversion, plus the shared board plane (`MapPlane`, `MapTextureCache`,
  `HexOverlay`, `apply_map_data_to_plane`, `terrain_overlay_color`) used by both the game and the
  map editor. `HexLayout` must be inserted manually with calibration data.
- **`omdurman-net`** — net glue. Defines `NetMsg`, `GameEvent`, `GameRecord` (event log),
  `InitialGameState`, and `room_id()`. `GameEvent` variants are the *only* messages recorded into
  the canonical event log and replayed for late joiners — adding a variant here automatically
  participates in recording/replay.
- **`omdurman-app`** — the Bevy game binary (`omdurman`). Game only: rendering, input, egui UI,
  camera, networking glue, and the event-viewer debug overlay. Entry point:
  `omdurman-app/src/main.rs`.
- **`omdurman-bot`** — bot / strategy advisor over the rules engine.
- **`tools/map-editor`** — native-only Bevy map editor (`map-editor`): board authoring
  (terrain, hexsides, roads, overlay calibration, turn-track bbox, scattergram, setup letters,
  entrance areas), the unit-sheet cutting grid, and the sprite-annotation editor. Saves to the
  RON data files under `omdurman-app/assets/`.
- **`tools/asset-editor`** — eframe/egui desktop tool for the six rules-data RON tables under
  `Boardgame - Remember_Gordon/tables/` (units roster, CRT, scattergram, LOS, range effects,
  order of appearance), with undo/redo and engine cross-checks.
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
   the record and replay it to converge to current state. `PendingIncoming.replay` flags events
   that came from a replay so they aren't re-recorded.
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
6. **Divergence healing.** The receive path detects two proof-of-brokenness conditions: a *seq
   conflict* (`Sequenced` at an already-applied seq carrying a different event — transient dual-host
   streams) and a *seq gap* (a jump past `last_applied + 1` — broadcasts racing a reconnecting data
   channel). Either forces a `RequestSnapshot` and a `force_install_history` install of the
   canonical record (the local record is known-bad, so the "install only if ahead" check must not
   apply); own events missing from the installed record are re-queued for resubmission. Identity
   dedup (`NetState::recent_uids`, bounded) makes double-sequenced events apply exactly once.
   The event log is the state, so the rebuild absorbs the rollback (`rebuild_state_to`).
7. **PRNG is shared and seeded.** `GameRng(ChaCha8Rng)` is seeded from the seed in `InitialGameState`
   so late joiners produce the same sequence on replay.

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
root `Cargo.toml`) with two robustness fixes: the socket message loop no longer panics — it drops
the outgoing packet with a warning — when an outgoing send races a peer teardown, and a
data-channel `on_open` callback no longer panics after handshake teardown. Both panics previously
killed every connection of the socket and were routinely triggered by the harness around rejoins.

## Architecture: dual-map (campaign + Fall-of-Khartoum)

The game uses two boards, switched by `MapKind` (`Campaign`, `FallOfKhartoum`).
`ActiveEditMap` tracks which one is live.
`PendingMapLoad` is set by the `StartGame` handler (and the board reconciler) to (re)load a board
on the next frame; `omdurman-app/src/board_state.rs` owns this bootstrap.

## Board + sprite data (RON data files)

The two boards live as RON data files under `omdurman-app/assets/boards/`
(`campaign.ron`, `fall_of_khartoum.ron`) — authored by `tools/map-editor`, embedded at compile time
by `omdurman-rules/src/board_data.rs` (the single `include_str!` owner), and parsed once on first
use. The app's `LoadedAnnotations` and the tactics fixtures both consume those accessors.
Sprite metadata: compiled fallbacks live in `omdurman-rules/src/sprite_data.rs` (keyed by
`UnitId` position, one global block); editor-authored annotations live in
`omdurman-app/assets/sprite_annotations.ron` and are loaded at startup into
`SpriteAnnotationsResource` as an overlay. Cut sprite images live under
`omdurman-app/assets/sprites/`.

## Mode switching (UI)

The top-level `AppMode`s are `Menu`, `Lobby`, and `Game`.
The splash screen provides the primary mode-switching UI.
There is no in-app editor — board/asset authoring happens in `tools/map-editor`.

## Traceability

`docs/traceability.toml` is the bijective rulebook ↔ code mapping.
`cargo test -p omdurman-rules --test traceability` enforces (checks shared with
the editor LSP in `tools/traceability-lsp/src/checks.rs`):

- `implemented` mappings list `[[mapping.impl]]` sites (`file`, `line`, `symbol`);
  the symbol must really exist near the cited line (comment mentions don't count).
- Every `§N` citation in Rust source has a mapping, and every mapping section
  exists in the OCR manual (matrix ↔ manual, both directions).
- Every cited symbol is compiler-anchored in `omdurman-rules/tests/traceability_paths.rs`
  (a real `use`/item path — a rename breaks the build), and every anchor there
  is cited by the matrix (paths ↔ matrix, both directions).
- **Coverage is hard**: every `implemented` mapping lists at least one
  `tests = [...]` entry (fully qualified `crate::module::fn_name`) whose test
  carries a `#[rulebook("§N")]`/`// §N` annotation for that section and is not
  `#[ignore]`d (`implemented_mappings_are_tested`).
- The generated PDF input is snapshot-checked: committed
  `tools/traceability-typst/data.json` must match a fresh regeneration
  (`committed_data_json_is_fresh` in the traceability-typst crate). Regenerate with
  `cargo run -p traceability-typst --bin traceability-typst -- docs/traceability.toml traceability.typ tools/traceability-typst/data.json`.
- **Kani proofs** are tracked the same way in a `proofs = [...]` array parallel to `tests`,
  bijective in both directions. Annotate a harness with `// §N` above `#[kani::proof]` — *not*
  `#[rulebook]`, because the proof modules are `cfg(kani)` on the lib where dev-dependencies
  (and so the proc-macro) are unavailable. The PDF renders proofs in blue above the green tests.

When adding code that implements a new rulebook section, cite the section in a comment
(`(rulebook §6.11)`) *and* add the matching `[[mapping]]` entry with at least one
annotated test. Container headings get `status = "descriptive"` (no impls/tests).
When renaming a symbol, update its `symbol` field in `traceability.toml` **and** the
anchor in `traceability_paths.rs`, or the build will fail. After moving code, re-sync
`line` fields with `cargo run -p traceability-typst --bin fix_lines`, then regenerate
`traceability.typ` + `tools/traceability-typst/data.json` (`cargo run -p
traceability-typst`) and the PDF, and commit them together with the TOML.

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

- The rules engine uses `value_enum!` (defined at the top of `omdurman-rules/src/lib.rs`) for any
  quantitative value with a fixed annotated set of possibilities. Match arms then stay
  exhaustive — prefer extending `value_enum!` over adding an `_ =>` arm.
- `omdurman-types` and `omdurman-rules` have no Bevy dependency. Keep it that way; Bevy lives in
  `omdurman-app` (and `omdurman-hexmap` for its plugin shim).
- `GameEvent` is the only enum whose variants get recorded and replayed. Adding a network message
  that should *not* persist (cursor pings, selection hints) belongs in `Ephemeral`, not
  `GameEvent`.
- New game mutations must go through a new `GameEffect` variant + `apply_effect` arm so they
  participate in determinism, replay, and host-relay ordering.

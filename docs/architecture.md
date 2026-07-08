# Architecture & System Map

A factual map of the Omdurman implementation: what the system is, which rules are enforced
where, the current state of the code, and the known open work. Companion to
[`traceability.toml`](traceability.toml) (rulebook↔code mapping) and
[`unreferenced-sections.md`](unreferenced-sections.md) (manual sections with no code impact).

Anchors are `file:line` at time of writing; re-verify against source, they drift.

---

## 1. System overview

A deterministic, event-sourced, peer-to-peer digital port of *Remember Gordon! — The Battle of
Omdurman* (Phoenix Enterprises, 1982). ~22k lines of Rust, edition 2024, runs native and as a
WASM web app (Trunk → GitHub Pages). Networking is P2P via `bevy_matchbox` (WebRTC + a `wss://`
signalling server).

Five workspace crates + one tool:

| Crate | Bevy? | Responsibility |
|---|---|---|
| `omdurman-types` | no | Pure serde leaf types shared by everything (`HexCoord`, `SectionName`, `AnnotationsFile`, hexside/Nile/overlay types, `Faction`, `Brigade`). |
| `omdurman-rules` | no | The authoritative rules engine: `GameState`, `GameEffect`, `apply_effect`. |
| `omdurman-hexmap` | yes | `HexMapPlugin`: `GameMap`, `HexLayout`, `MapDims`, world-space conversion. |
| `omdurman-net` | no | Net glue: `NetMsg`, `GameEvent`, `GameRecord`, `InitialGameState`, `room_id()`. |
| `omdurman-app` | yes | The Bevy binary: rendering, input, egui UI, dice physics (avian3d), camera, net glue, editors. |
| `tools/traceability-typst` | — | Regenerates the traceability PDF; `fix_lines` re-syncs line numbers. |

Two boards (`MapKind::{Campaign, FallOfKhartoum}`) live in the same binary. The editor tooling
(overlay calibration, terrain/hexside editor, sprite browser, unit-sheet editor, event-log
viewer) is implementation scaffolding for authoring *this* game's data — not a general-purpose
wargame editor.

---

## 2. Four load-bearing design choices

1. **Pure rules engine.** `omdurman-rules` has no Bevy dependency. Every legal mutation flows
   through `effects::apply_effect`, which validates then mutates `GameState`. Quantitative rule
   values are compile-time-exhaustive `value_enum!` enums (`lib.rs:31`) — fire/melee factors,
   movement allowances, die rolls, range bands — so match arms can't silently miss a case. Errors
   are `thiserror` enums, never strings.

2. **Determinism by construction.** Dice are pre-rolled and embedded in each `GameEffect`, so
   re-applying an effect on any peer reproduces identical state. The PRNG (`GameRng(ChaCha8Rng)`)
   is seeded from `InitialGameState.seed`, shared across peers and replayed identically by late
   joiners. No live RNG calls during effect application.

3. **Event-sourced, host-relayed P2P.** A guest sends an unsequenced `NetMsg::Game(event)`; the
   host assigns a global `seq` and broadcasts `NetMsg::Sequenced`; every peer (host included, via
   its `loopback` queue) applies events *only* on the sequenced echo — "apply-on-echo." `GameRecord`
   is the canonical, byte-identical event log; late joiners request it and replay to converge.

4. **Map-aware, not map-dependent engine.** `GameState.board: BoardInfo` (`board.rs`) carries
   terrain/hexsides/Nile-flow/landmarks. With a board, map-dependent rules (terrain cost, ZOC
   hexside blockers, gunboat up/downstream, Mahdi's Tomb scoring) are enforced; with an empty
   `BoardInfo::default()` the engine still runs (tests/demos) and those rules go rule-neutral.

---

## 3. Rules coverage: what is enforced, and where

### Enforced end-to-end in `apply_effect` (engine is authority)
- **Movement (§5)** incl. night-halving (AE only, §8.1), once-per-turn limit, ZOC-stop (§5.26,
  §5.43) with §5.44 hexside/Nile/gunboat exceptions, Nile-entry ban for land units (§5.22),
  gunboat Nile-only + up/downstream allowance cap (§5.24), forts immobile (§5.25).
- **ZOC (§5.41, §5.44)** — disrupted/leader/gunboat projection rules + hexside & Nile blockers.
- **Stacking (§5.51–53)** — 4-unit limit, gunboat isolation, Dervish tribe separation, leader command.
- **Fire combat (§6)** — phase/player/disruption/once-per-phase gating, per-faction/weapon range
  bands (§6.22), CRT, gunboat 3+ (§6.61) and fort 2+ (§6.62) thresholds, howitzer scatter
  *direction* (§6.64).
- **Melee (§7)** — melee-capable kinds, adjacency, simultaneous CRT, modifiers, the declared-melee
  reaction window (§7.5: `DeclareMelee` → `RetreatBeforeMelee` → `ResolveMelee`, re-deriving
  defenders from current occupants so retreaters are spared), mandatory Dervish advance (§7.6).
- **Turn/phase sequence (§4)**, day-night from turn track, Dervish desertion (§8.2), Friendlies
  gunboat transport (§5.21), river mines (§10.12) incl. Dervish immunity (§10.14), river chain
  (§10.21–23), full victory-point ledger incl. Mahdi's Tomb (§9.14) and per-scenario victory levels.

### Deliberately app-side (engine holds the table; app gates, engine applies the modifier)
- **Line of sight (§6.3)** — matrix lives in `los_table.rs`; the gate is in `fire.rs`, not in
  `apply_effect`. The engine trusts the `FireModifier` the caller supplies.
- **Terrain defence modifiers** (`terrain_chart.rs` data; app constructs `FireModifier::Terrain`).
- **Melee wall/khor hexside blocking (§7.2)** and advance-after-combat hexside blockers — app gates.
- **Zariba thorn/trench modifiers (§9.231–232)** and **brigade-integrity bonus (§5.54)** — app-supplied.

This split is intentional: the cross-hex board iteration LOS/hexside checks require is the app's
job; the engine stays a pure state machine over data it already holds.

### Fall of Khartoum (§9.3) — enforced
- §9.346 GORDON is immobile and eliminated only when a Dervish unit reaches the Palace hex;
  §9.35 victory is the turn-of-death level shifted by the Dervish-loss penalty (`FoKVictoryLevel`).
- §9.343 both players use the Dervish range table in FoK; §9.345 a British gunboat may cross the
  White↔Blue Nile mouths off-board for 6 MP; §9.344 the Dervish hold the North Fort (forts are
  never captured, only destroyed — §6.54, now enforced for movement and advance-after-combat).
- Set-up: `build_setup_plan` auto-places GORDON in the Palace; the §9.322 Dervish turn-1 entry
  edge is highlighted by `fok_entry.rs`. `BoardInfo::from_map_data` populates `locations` from
  named tiles so all of the above resolve at runtime.

### Simplified
- Howitzer scatter: *direction* enforced, exact printed-Scattergram distance simplified to one hex.
- Interactive `MoveUnit` now carries the entered `path` (a single adjacent step per click — the
  picker commits one hex at a time), so the engine costs the step by terrain and classifies gunboat
  up/downstream from it. Multi-hex routes aren't a thing in the interactive model (the player walks
  hex-by-hex, each step validated independently), so per-hex ZOC-stop reduces to the destination
  check on each step.

Rules engine submodules each own one table/domain: `combat_results_table`, `howitzer_scatter`,
`los_table`, `range_effects`, `terrain_chart`, `turn_track`, `unit_id` (generated from
`annotations.ron`), `board`.

---

## 4. Networking & event sourcing

Message types in `omdurman-net/src/lib.rs`; glue in `omdurman-app` (`net_plugin.rs`,
`net_socket.rs`, `game_record.rs`, `game_apply.rs`).

- **`NetMsg`** — `Game(GameEvent)` (unsequenced, guest→host), `Sequenced { seq, event }`
  (host→all, the *only* form applied locally), `Ephemeral` (unreliable, never recorded),
  `Control` (snapshot handshake).
- **`GameEvent`** — the only enum whose variants are recorded/replayed. Game mutations are
  `Effect(GameEffect)`; also map/sprite edits, `StartGame`, `PlaceUnit`/`MoveUnit`. Non-persistent
  messages (cursors, selections) belong in `Ephemeral`.
- **Sequencing** — guest stages to `PendingEdits.outgoing_broadcast`; host assigns `seq` and feeds
  its own events through `PendingIncoming.loopback` so it applies them on echo like everyone else.
  `record_host_events` appends to the log *before* the wire, preserving order. `push_event`
  dedups by `seq` (idempotent on duplicate delivery).
- **Late joiners** — request `GameHistory(GameRecord)`, reset RNG from `initial_state.seed`,
  rebuild `GameState::new(scenario)`, replay every event in canonical order.
  `PendingIncoming.replay` keeps replayed events from being re-recorded; the dual-map board load
  is deferred post-replay so edits land on the right board.
- **Effect application** — no translation layer: the app builds `GameEffect` directly, wraps it
  `GameEvent::Effect`, and on the sequenced echo `game_apply::apply_game_event` calls
  `apply_effect`. A rejected effect is warned, not retried.

### Determinism holds when
Same canonical record on every peer, same seed, pre-rolled dice, deterministic
(sorted-lexicographically) peer ordering. **Breaks if** the record is corrupted/truncated, peers
hold different `annotations.ron`, the reliable channel loses/reorders messages, or
`GameState::new` differs across peers.

### Known netcode fragilities (resilience, not correctness)
- All-or-nothing broadcast retention — one laggy peer stalls the `Sequenced` broadcast to all
  (`net_plugin.rs` ~269–291).
- Host failover mid-flight: a guest's in-flight unsequenced event to a dead host has no
  re-sequencing guarantee; the promoted host inherits `next_seq` without reconciliation.
- Unbounded 2s snapshot-retry with no ceiling (`net_socket.rs` ~104–119).
- Turn counter can point at a "ghost" seat after a disconnect (mitigated: `FactionGate::may_act`
  still blocks input).

These bite at 3+ players or on host loss. Two-player-with-stable-host is solid.

---

## 5. App layer (omdurman-app, omdurman-hexmap)

- **Modes.** `Ctrl+1/2` switch top-level modes; ~13 `EditorMode` states behind them (map play,
  overlay calibration, terrain/hexside editor, unit-sheet grid editor, sprite browser, dice
  physics sandbox, event-log viewer, campaign-timing editor). Behaviour is gated on active mode,
  not a build flag.
- **Dual-map.** `ActiveEditMap` (local) tracks the live board; `PendingMapLoad` defers a (re)load
  to the next frame; `LoadedAnnotations` holds both boards. A play view (Game/Sandbox) follows its
  scenario's board for the whole session; the editor's board follows `EditorBoard`. **Unit sprite
  annotations are top-level on `AnnotationsFile`, not per-board** — one global sprites block.
- **Annotations pipeline.** `AnnotationsDirty` debounce; `editor::flush_annotations_to_disk` writes
  `assets/annotations.ron` after idle exceeds `ANNOTATIONS_FLUSH_SECS`. Never discard unstaged
  edits — they are real synced map state (migrate, don't drop, if invalid).
- **Dice physics** (avian3d) is a dev/visual sandbox only; canonical rolls are pre-rolled into
  effects in `fire.rs`/`melee.rs`.
- **Hexmap.** `HexLayout` (pointy orientation) must be inserted manually with calibration data;
  `world.rs` does axial↔world conversion with round-trip tests.

---

## 6. Current state of the code

Exceptionally clean. The Fall-of-Khartoum scenario is now playable end-to-end (set-up → rules-
enforced turns with visible results → §9.35 verdict).

- **Retreat-before-melee is fully implemented and wired** (`retreat.rs`: defender-gated overlay +
  `RetreatBeforeMelee`, validated by `can_retreat_before_melee`).
- **`overview.rs` is a working unit-overview side panel.**
- **Movement is now turn-gated and engine-authoritative:** the picker uses `MoveGate` so a unit
  can only be moved on its owner's turn, and a move the engine rejects no longer animates
  (`apply_move_effect` returns acceptance). `MoveUnit` carries the entered `path` (one adjacent step
  per click), so the engine costs each step by terrain and classifies gunboat up/downstream.
- **Combat feedback and game end are surfaced:** `sync_eliminated_visuals` despawns eliminated
  counters, `game_log_panel` shows the engine's recent result log, and `victory_modal` shows the
  final scenario verdict when `game_over` is set.

---

## 7. Open directions

Independently pickable; none started. Ordered roughly by leverage-to-effort.

1. **Thread the BFS path into `MoveUnit`** (`main.rs:645`) — the only data-ready-but-unwired
   mechanic; makes terrain/Nile-flow movement cost authoritative.
2. **Phase-gate the UI** so the engine is authoritative over visuals (reject → no animation /
   rollback), removing the last engine↔view drift.
3. **Move LOS / terrain-defence / hexside-melee gating into `apply_effect`** — tables already live
   in the engine; this lets a headless engine adjudicate and stops a buggy/malicious client from
   pushing an illegal attack. Also unlocks legal-move enumeration (hints, undo, AI foundation).
4. **Solo / AI opponent** driving `GameEffect`s through the same path the network uses; the
   pre-rolled-dice determinism makes this clean.
5. **Netcode robustness** for true N-player: per-peer broadcast queues (kill the all-or-nothing
   stall), bounded snapshot retry with fallback, host-failover reconciliation.
6. **Presentation:** wire the dice sandbox as an optional *cosmetic* roll (results stay
   pre-rolled/canonical); engine-driven ZOC/legal-move/legal-target overlays; combat-result toasts
   off the `VictoryLedger`.

---

## 8. Validating changes

- `cargo test -p omdurman-rules` — engine unit + integration tests.
- `cargo test -p omdurman-rules --test traceability` — keeps rulebook↔code mapping honest; a
  symbol rename without a TOML update fails (and `traceability_paths.rs` fails to compile).
- `cargo run -p omdurman-app` (native) / `trunk serve` (WASM). CI gate is
  `trunk build --release` for `wasm32-unknown-unknown` — keep it green on dependency changes.
- After moving code: `cargo run -p traceability-typst --bin fix_lines` re-syncs `line` fields.

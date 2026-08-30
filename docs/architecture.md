# Architecture & System Map

A factual map of the Omdurman implementation: what the system is, which rules are enforced
where, the current state of the code, and the known open work. Companion to
[`traceability.toml`](traceability.toml) (rulebook↔code mapping).

Anchors are `file:line` at time of writing; re-verify against source, they drift.

---

## 1. System overview

A deterministic, event-sourced, peer-to-peer digital port of *Remember Gordon! — The Battle of
Omdurman* (Phoenix Enterprises, 1982). ~77k lines of Rust, edition 2024, runs native and as a
WASM web app (Trunk → GitHub Pages). Networking is P2P via `bevy_matchbox` (WebRTC + a `wss://`
signalling server).

Eleven workspace members. The Bevy-free column is load-bearing: only the two leaf crates can be
model-checked (see §9), and `omdurman-app` enables `bevy/dynamic_linking`, which Kani cannot see
through at all.

| Crate | Bevy? | Responsibility |
|---|---|---|
| `omdurman-types` | no | Pure serde leaf types shared by everything (`HexCoord`, `SectionName`, `AnnotationsFile`, hexside/Nile/overlay types, `Faction`, `Brigade`). |
| `omdurman-rules` | no | The authoritative rules engine: `GameState`, `GameEffect`, `apply_effect`; the four printed tables (from authored RON, see §3). |
| `omdurman-hexmap` | yes | `HexMapPlugin`: `GameMap`, `HexLayout`, `MapDims`, world-space conversion. |
| `omdurman-net` | yes | Net glue: `NetMsg`, `GameEvent`, `GameRecord`, `InitialGameState`, `room_id()`. Pulls Bevy for `Resource` derives and the log macros. |
| `omdurman-app` | yes | The Bevy binary: rendering, input, egui UI, camera, net glue. |
| `omdurman-bot` | via net | Headless AI playthrough driver: random + LLM-advised agents, invariant checks, an offline log auditor. |
| `traceability-macro` | — | The `#[rulebook("§N")]` proc-macro attribute. |
| `tools/traceability-typst` | — | Regenerates the traceability PDF; `fix_lines` re-syncs line numbers. |
| `tools/traceability-lsp` | — | LSP server + VS Code client for rulebook↔code navigation; shares its `checks` with the rules test. |
| `tools/map-editor`, `tools/asset-editor` | yes | Native-only authoring tools (board RON, sprite/table data). |

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

### Fully engine-authoritative (every check in `apply_effect` / `can_*`)
- **Line of sight (§6.3, §6.21)** — `has_los` in `los_table.rs` ray-casts through
  `BoardInfo` terrain and hexsides; `can_fire_at` enforces it. Howitzer fire bypasses.
- **Terrain defence modifiers (§6.23)** — `resolve_fire_attack` looks up the modifier
  from `state.board.terrain_at(target)` and applies it internally; the app no longer
  supplies `FireModifier::Terrain`.
- **Melee wall/thorn-hedge hexside blocking (§7.2)** — `can_melee` checks
  `board.hexside_between().blocks_melee()`.
- **Advance-after-combat hexside blockers (§6.82, §7.6)** — `can_advance_after_combat`
  checks `blocks_advance_after_combat()`.
- **Zariba thorn/trench modifiers (§9.231–232)** and **brigade-integrity bonus (§5.54)** —
  app-supplied via `FireModifier` / `MeleeModifier` (deterministic, terrain-independent).

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
  (Surfaced as a `note` field on the §6.64 `[[mapping]]` in `traceability.toml`.)

Rules engine submodules each own one table/domain: `combat_results_table`, `howitzer_scatter`,
`los_table`, `range_effects`, `terrain_chart`, `turn_track`, `unit_id`,
`board_data` (compiled `MapData`), `sprite_data` (compiled sprite annotations), `board`.

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
use different compiled data, the reliable channel loses/reorders messages, or
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
  overlay calibration, terrain/hexside editor, unit-sheet grid editor, sprite browser,
  event-log viewer, campaign-timing editor). Behaviour is gated on active mode,
  not a build flag.
- **Dual-map.** `ActiveEditMap` (local) tracks the live board; `PendingMapLoad` defers a (re)load
  to the next frame; `LoadedAnnotations` holds both boards. A play view (Game) follows its
  scenario's board for the whole session; the editor's board follows `EditorBoard`. **Unit sprite
  annotations are top-level on `AnnotationsFile`, not per-board** — one global sprites block.
- **Board + sprite data.** `LoadedAnnotations` is seeded from compiled `board_data` and
  `sprite_data` at startup. Edits to map state are in-memory only (no file persistence).
- **Hexmap.** `HexLayout` (pointy orientation) must be inserted manually with calibration data;
  `world.rs` does axial↔world conversion with round-trip tests.

### Legibility surfaces — "what just happened, and what can I do?"

The UI is built around the principle that a player who has not read the manual can still follow
what the engine is doing and why. Every citation deep-links into the in-app Rulebook tab
(searchable, scrollable, parsed from `Boardgame - Remember_Gordon/Manual/RememberGordonManual.md`).

- **§-title index.** `Rulebook::title_of(number)` resolves a section number to its short title
  ("§5.26 Units stop on entering enemy ZOC"); citations rendered via `Rulebook::render_refs`
  carry the title inline rather than appearing as opaque numbers.
- **Combat Resolution Card** (`combat_card.rs`). Every fire/melee resolution emits a structured
  `Observation::FireResolved` / `Observation::MeleeResolved` (in `effects.rs`) carrying the full
  attack bundle — firers, target, per-modifier breakdown with paragraphs, die roll, modified
  roll, CRT factor row, result, casualties. The card surfaces this as a fadeable, deep-link-rich
  breakdown: each modifier ("+1 AE direct fire §6.24", "-1 terrain defence §6.23") attributes
  itself to its rule, and the casualty list names the units lost. Late-join / replay produces
  the same card stream.
- **Dispatch slips** (`dispatch.rs`). Every non-combat `Observation` (LeaderKilled,
  DemolitionResolved, WallBreached, VictoryScored, GordonEliminated, FriendliesDisembarked,
  FortDestroyed, UnitEliminated) renders as a paper-card "field telegraph" slip with its
  authorising § references deep-linked. The slip queue is bounded and ages out.
- **Action discovery panel** (`actions_panel.rs`, in the right sidebar). Names the current
  phase + active player, lists the categories of action the rulebook allows in it (move / fire
  / melee / construct zariba / load Friendlies / end phase), each with a § deep-link, and shows
  context counts ("3 in-range targets") derived from the same `can_*` predicates the input
  handlers gate on — so the panel cannot disagree with the on-map rings about what's legal.
  The selected-unit block shows fire/melee/move factors and live MP remaining.
- **Outcome prediction** (`combat_predict.rs` + `fire.rs` preview). On hovering a fire target,
  the preview shows the outcome bands across raw rolls 1..=10 ("1-3 no effect · 4-5 disrupt ·
  6-8 eliminate 1 · 9-10 eliminate 2"), computed from the CRT given the factor row + net
  modifier. The engine still pre-rolls for canonical resolution.
- **Hover tooltip** (`hover_tooltip.rs`). Hovering any hex shows terrain, coord, landmark,
  occupants with their (fire/melee) factors, and — when a unit is selected — a movement/blocking
  hint that names *why*: terrain cost, wall hexside, ZOC, stacking, out-of-MP, Nile impassability.
  Each clause carries its § paragraph as a deep-link.
- **Picker sprite tooltips** (`picker.rs`). Hovering a sprite in the unit-picker sidebar shows
  the counter's resolved profile (identity, fire/melee/move factors, weapon, kind, printed text,
  fires-twice flag) plus a §2.3x deep-link to its section.

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
- **Engine correctness fixes (audit-driven):**
  - §6.42 Maxim second fire: `units_fired_this_phase` is cleared on entering the
    `MaximSecondAndHowitzer` subphase; non-Maxim/non-Howitzer units are rejected with a typed
    `RuleError::WrongWeaponForSubphase`.
  - §6.53 Royal Engineers demolition: `apply_resolve_demolition` removes forts / breaches walls at
    end of turn when the engineer remains adjacent and undisrupted. The §6.63 breach side-effect
    (adjacent enemy eliminated) is enforced. Auto-emitted via `end_player_turn`.
  - §9.14 VP routing: Khalifa = 10 VP (was 1), Isa Zachneih = 1 VP (distinct source), forts = 0 VP.
    The two auto-decisive conditions (all-Dervish-eliminated / all-AE-west-bank-eliminated) are
    checked in `finish_game`.
  - §9.35 FoK British survival ladder: `resolve` now takes `scenario_end_turn`, distinguishing
    Marginal/Tactical/Decisive based on how long GORDON survived.
  - §8.1 night ranges: the weapon's *max range* is halved (not the distance); the day table is then
    consulted at the *physical* distance, matching the rulebook's AE-rifle worked example.
  - §5.21 Friendlies transport: full gating (Isa-Zachneih prerequisite, adjacency at load,
    turn-sequencing between Loaded/Crossing/ReadyToDisembark). A gunboat sunk while carrying a
    unit eliminates the loaded unit (`ElimCause::LostWithTransport`).
- **Observation side-channel:** `GameState.observations: Vec<Observation>` is pushed by
  `apply_effect` and drained by the app after each event application (`PendingObservations`
  resource → `ObservationEvent` Bevy messages). Carries demolition results, leader deaths, VP
  awards, and fort/wall destruction for dispatch slips, sounds, and animations. Serialized so
  replay produces the same stream.
- **Engine-authoritative LOS / terrain defence / hexside blocking:** `has_los` ray-casts through
  `BoardInfo` (terrain + hexsides stored in `GameState`), enforced by `can_fire_at`; terrain
  defence modifier is computed engine-side in `resolve_fire_attack` from `state.board`; melee
  hexside blocking (§7.2) and advance-after-combat hexside blocking (§6.82, §7.6) are enforced
  in `can_melee` / `can_advance_after_combat`. The app no longer supplies `FireModifier::Terrain`
  or gates on these checks separately — `can_fire_at` / `can_melee` / `can_advance_after_combat`
  are the single authority.
- **Engine-authoritative movement costing:** `BoardInfo` now carries `roads` (§5.11 Terrain Effects
  Chart: road = 1 MP); `movement_cost_for` uses terrain + roads from `state.board`, not just
  terrain. Wall-hexside movement blocking (§5.23) is enforced in `can_move_unit_to` via
  `self.board.hexside_between().blocks_movement()`. The picker's BFS overlay now uses
  `effective_movement_at_night` so night-time reach matches what the engine will accept.

---

## 7. Open directions

Independently pickable. Ordered roughly by leverage-to-effort.

1. **Effect atomicity.** `apply_effect` mutates state before returning `Err` in at least three
   places, so a peer that rejects an effect diverges from one that accepts it:
   - `resolve_fire_attack` pushes every firer into `units_fired_this_phase` before the
     `AlreadyFiredAt` / `ArtilleryOnlyVsGunboatOrFort` guards, so a *rejected* attack burns the
     firers' once-per-phase allowance (§6.14).
   - `apply_resolve_melee` does `pending_melee.take()` before delegating to `apply_melee_combat`,
     which rejects a wrong phase — destroying a declared melee and its pre-rolled dice. The same
     loss is already guarded in `advance_phase` ("audit: 76 declared melees vanished this way").
   - `advance_phase` clears `vacated_by_combat` before the `MeleePendingResolution` /
     `DesertionRollRequired` guards, dropping the §6.82/§7.6 advance-after-combat windows.

   The fix is to hoist all validation ahead of any mutation. Kani harnesses for this exist in
   `effects.rs` but do not yet solve (see §9).
2. **Phase-gate the UI** so the engine is authoritative over visuals (reject → no animation /
   rollback), removing the last engine↔view drift.
3. **Netcode robustness** for true N-player: per-peer broadcast queues (kill the all-or-nothing
   stall), bounded snapshot retry with fallback, host-failover reconciliation.
4. **Presentation:** engine-driven ZOC/legal-move/legal-target overlays; combat-result toasts
   off the `VictoryLedger`.
5. **CI runs no tests.** The only workflow is the GitHub Pages `trunk build --release`. The whole
   test suite, the traceability gates and the bot's invariant checks are local-only.

Since removed from this list: the **solo / AI opponent** is `omdurman-bot`.

---

## 8. Validating changes

- `cargo test -p omdurman-rules` — engine unit + integration tests.
- `cargo test -p omdurman-rules --test traceability` — keeps rulebook↔code mapping honest; a
  symbol rename without a TOML update fails (and `traceability_paths.rs` fails to compile).
- `cargo run -p omdurman-app` (native) / `trunk serve` (WASM). CI gate is
  `trunk build --release` for `wasm32-unknown-unknown` — keep it green on dependency changes.
- After moving code: `cargo run -p traceability-typst --bin fix_lines` re-syncs `line` fields.
  Adding or removing lines in a cited file drifts every `line` below it, so run this before
  trusting a traceability failure.
- After editing `docs/traceability.toml`: regenerate the report, or
  `committed_data_json_is_fresh` fails —
  `cargo run -p traceability-typst --bin traceability-typst -- docs/traceability.toml traceability.typ tools/traceability-typst/data.json`.
- `cargo test -p omdurman-bot` — the strongest whole-engine regression signal (random playthroughs
  with per-effect invariant checks). Takes ~10 minutes; the invariants proptest alone is ~30s.
- `./scripts/kani.sh -p omdurman-types -p omdurman-rules` — the proof suite (§9). Needs WSL on
  Windows.

---

## 9. Formal verification (Kani)

25 proof harnesses verify under the [Kani](https://model-checking.github.io/kani/) model checker:
14 in `omdurman-types/src/lib.rs` (hex geometry) and 11 in `omdurman-rules/src/lib.rs`
(`value_enum!` conversions, die arithmetic — 6 written out plus 5 generated by the
`prove_value_enum!` macro, one per `value_enum!` type). A further 11 in
`omdurman-rules/src/effects.rs` are authored but do not yet solve (see Boundaries below). They
live in `#[cfg(kani)] mod verification` blocks inside the file they verify, so private items stay
reachable.

**Kani has no native Windows support.** `scripts/kani.sh` shells into WSL (Debian) against the
repo at `/mnt/c/...`, with a separate `CARGO_TARGET_DIR` so Linux artifacts never collide with the
host `target/`. On Linux/macOS it calls `cargo kani` directly. Kani ships its own pinned nightly
and ignores `rust-toolchain.toml`, so the 1.98.0 pin does not interfere. No `kani` dependency
belongs in any `Cargo.toml` — the crate is auto-injected.

**CI does not run the proofs** (CI runs no tests at all — see §7). They are a local gate.

What the proofs buy over tests: they close the domain. The hex-geometry set exists because
`distance` used the wrong cube axis (`s = -q-r` instead of `s = r-q`) and disagreed with
`neighbors`, which made `line_between` stall short of its target for 65% of on-board firing pairs
and silently corrupted the §6.3 LOS ray. Every test passed throughout — they sampled
`(0,0) → (3,0)`, a pure-axis case that works under both conventions.
`adjacency_iff_distance_one` now pins the two together permanently.

Proofs participate in the traceability matrix through `proofs = [...]` (parallel to `tests`), and
are annotated `// §N` above `#[kani::proof]`. The comment style rather than `#[rulebook]` is
deliberate: the proof modules are `cfg(kani)` on the *lib*, where dev-dependencies — and so the
proc-macro — are unavailable.

### Boundaries

- **Table contents are not provable.** The four printed tables are authored RON parsed at runtime
  and Kani cannot see through `include_str!`. Their shape (including the row lengths that the
  unchecked `cells[roll - 1]` indexing depends on) is asserted in
  `tables_data::tests::tables_parse_and_have_expected_shape` instead.
- **`apply_effect` harnesses do not yet solve.** `effects.rs` carries a bounded symbolic
  `GameState` generator and atomicity harnesses for the §7 open-direction bugs. CBMC has not
  returned within ~35 minutes even for the payload-free `SinkChain` rung, because `apply_effect`
  reaches most of the 411KB file whatever variant it dispatches to. Making them tractable needs
  stubbing or unwind tuning on the turn-end cascade
  (`end_player_turn` → demolitions → scoring → `finish_game`), not a different property.
- **`omdurman-app` is out of reach** — it enables `bevy/dynamic_linking`.

When adding proofs: unwind assertions are enforced by default in Kani 0.67, so a too-small
`#[kani::unwind(N)]` fails loudly rather than truncating silently. Still, confirm a loop-bearing
harness is non-vacuous with a deliberately-false assertion before trusting a `SUCCESSFUL` verdict.
Counterexamples need `-Z concrete-playback --concrete-playback=print`.

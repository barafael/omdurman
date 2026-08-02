# Architecture & System Map

A factual map of the Omdurman implementation: what the system is, which rules are enforced
where, the current state of the code, and the known open work. Companion to
[`traceability.toml`](traceability.toml) (rulebook↔code mapping).

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
  overlay calibration, terrain/hexside editor, unit-sheet grid editor, sprite browser, dice
  physics sandbox, event-log viewer, campaign-timing editor). Behaviour is gated on active mode,
  not a build flag.
- **Dual-map.** `ActiveEditMap` (local) tracks the live board; `PendingMapLoad` defers a (re)load
  to the next frame; `LoadedAnnotations` holds both boards. A play view (Game) follows its
  scenario's board for the whole session; the editor's board follows `EditorBoard`. **Unit sprite
  annotations are top-level on `AnnotationsFile`, not per-board** — one global sprites block.
- **Board + sprite data.** `LoadedAnnotations` is seeded from compiled `board_data` and
  `sprite_data` at startup. Edits to map state are in-memory only (no file persistence).
- **Dice physics** (avian3d) is a dev/visual sandbox only; canonical rolls are pre-rolled into
  effects in `fire.rs`/`melee.rs`.
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

Independently pickable; none started. Ordered roughly by leverage-to-effort.

1. **Phase-gate the UI** so the engine is authoritative over visuals (reject → no animation /
   rollback), removing the last engine↔view drift.
2. **Solo / AI opponent** driving `GameEffect`s through the same path the network uses; the
   pre-rolled-dice determinism makes this clean.
3. **Netcode robustness** for true N-player: per-peer broadcast queues (kill the all-or-nothing
   stall), bounded snapshot retry with fallback, host-failover reconciliation.
4. **Presentation:** wire the dice sandbox as an optional *cosmetic* roll (results stay
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

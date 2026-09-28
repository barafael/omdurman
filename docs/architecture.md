# Architecture & System Map

A factual map of the Omdurman implementation: what the system is, which rules are enforced
where, and the current state of the code. Companion to
[`traceability.toml`](traceability.toml) (rulebook↔code mapping); known open work lives in
[`open-issues.md`](open-issues.md). The net / seat / replay protocol is specified in
[`CLAUDE.md`](../CLAUDE.md); §4 here summarises it. Code is cited by symbol name — grep for it.

---

## 1. System overview

A deterministic, event-sourced, peer-to-peer digital port of *Remember Gordon! — The Battle of
Omdurman* (Phoenix Enterprises, 1982). ~98k lines of Rust, edition 2024, runs native and as a
WASM web app (Trunk → GitHub Pages). Networking is P2P via `bevy_matchbox` (WebRTC + a `wss://`
signalling server).

Twelve workspace members. The Bevy-free column is load-bearing: only the two leaf crates can be
model-checked (see §9), and `omdurman-app` enables `bevy/dynamic_linking`, which Kani cannot see
through at all.

| Crate | Bevy? | Responsibility |
|---|---|---|
| `omdurman-types` | no | Pure serde leaf types shared by everything (`HexCoord`, `SectionName`, `MapData`, `SpriteAnnotation`, hexside/Nile/overlay types, `Faction`, `Brigade`, `CommandScope`). |
| `omdurman-rules` | no | The authoritative rules engine: `GameState`, `GameEffect`, `apply_effect`; the four printed tables as `static` consts (see §3, §9). |
| `omdurman-hexmap` | yes | `HexMapPlugin`: `GameMap`, `HexLayout`, `MapDims`, world-space conversion, the shared board plane (`MapPlane`, `HexOverlay`). |
| `omdurman-board-ui` | yes | Board-view plumbing shared by the app and the map editor: RTS camera, input/raycast helpers, egui pointer gating (`MapPointerInputSet`), night shading, the two-board store (`LoadedAnnotations`, `PendingMapLoad`). |
| `omdurman-net` | yes | Net glue: `NetMsg`, `GameEvent`, `GameRecord`, `InitialGameState`, `PlayerKey`, `Seat`, `room_id()`. Pulls Bevy for `Resource` derives and the log macros. |
| `omdurman-app` | yes | The Bevy binary: rendering, input, egui UI, camera, net glue, seats, the in-game AI driver (`bot_player.rs`). |
| `omdurman-bot` | via net | Headless AI playthrough driver and the in-game AI's decision logic: random, aggressive, LLM-advised and Kitchener/Khalifa commander agents, invariant checks, an offline log auditor. |
| `traceability-macro` | — | The `#[rulebook("§N")]` proc-macro attribute. |
| `tools/traceability-typst` | — | Regenerates the traceability PDF; `fix_lines` re-syncs line numbers. |
| `tools/traceability-lsp` | — | LSP server + VS Code client for rulebook↔code navigation; shares its `checks` with the rules test. |
| `tools/map-editor` | yes | Native-only board authoring (terrain, hexsides, roads, overlay calibration, set-up letters, entrance areas), the unit-sheet cutting grid and the sprite-annotation editor; writes the RON data files under `omdurman-app/assets/`. |
| `tools/asset-editor` | no (eframe/egui) | Native-only editor for the rules-data RON tables under `Boardgame - Remember_Gordon/tables/`, with undo/redo and engine cross-checks. |

Two boards (`MapKind::{Campaign, FallOfKhartoum}`) live in the same binary. The authoring
tooling (overlay calibration, terrain/hexside editor, sprite browser, unit-sheet editor) lives in
`tools/map-editor`; it and the app's event-viewer overlay are implementation scaffolding for
authoring *this* game's data — not a general-purpose wargame editor.

---

## 2. Four load-bearing design choices

1. **Pure rules engine.** `omdurman-rules` has no Bevy dependency. Every legal mutation flows
   through `effects::apply_effect`, which validates then mutates `GameState`. Quantitative rule
   values are compile-time-exhaustive `value_enum!` enums (macro at the top of `lib.rs`, types
   mostly in `scalars.rs`) — fire/melee factors, movement allowances, die rolls, range bands — so
   match arms can't silently miss a case. Errors are `thiserror` enums (`RuleError`), never
   strings.

2. **Determinism by construction.** The acting peer pre-rolls the dice and embeds them in each
   `GameEffect`, so re-applying an effect on any peer reproduces identical state and replay never
   draws a random number. The PRNG (`rng::GameRng(ChaCha8Rng)`) is each peer's *local* roll
   source: seeded from the fresh per-peer seed in its own record header (`InitialGameState.seed`)
   and reseeded from fresh entropy after every history install or rebuild. Determinism does not
   depend on any shared PRNG position. The bot draws from the same implementation.

3. **Event-sourced, host-relayed P2P.** A peer submits an unsequenced `NetMsg::Game { uid, event }`;
   the host assigns a global `seq` and broadcasts `NetMsg::Sequenced { seq, uid, event }`; every
   peer (host included, via its `loopback` queue) applies events *only* on the sequenced echo —
   "apply-on-echo." The submission `uid` makes retransmission idempotent. `GameRecord` is the
   canonical event log; late joiners request it and replay to converge.

4. **Map-aware, not map-dependent engine.** `GameState.board: Arc<BoardInfo>` (`board.rs`,
   copy-on-write shared across clone-and-try validation) carries terrain (incl. Nile current),
   hexsides, roads, landmarks, entrance areas and the walled city. With a board, map-dependent
   rules (terrain cost, ZOC hexside blockers, gunboat up/downstream, Mahdi's Tomb scoring) are
   enforced; with an empty `BoardInfo::default()` the engine still runs (tests/demos) and those
   rules go rule-neutral.

---

## 3. Rules coverage: what is enforced, and where

### Enforced end-to-end in `apply_effect` (engine is authority)
- **Movement (§5)** incl. night-halving (AE only, §8.1), once-per-turn limit, ZOC-stop (§5.26,
  §5.43) with §5.44 hexside/Nile/gunboat exceptions, Nile-entry ban for land units (§5.22),
  gunboat Nile-only + up/downstream allowance cap (§5.24), forts immobile (§5.25). Every step is
  costed by `terrain_chart::land_step_cost` — the Terrain Effects Chart (road rate only along a
  road link; Khor/Crest/gate/breach hexside surcharges), the one step-cost rule shared by the
  engine, the app's reach overlay and the bot.
- **ZOC (§5.41, §5.44)** — disrupted/leader/gunboat projection rules + hexside & Nile blockers,
  applied directionally (§5.44).
- **Stacking (§5.51–53)** — 4-unit limit, gunboat isolation, Dervish tribe separation, leader command.
- **Fire combat (§6)** — phase/player/disruption/once-per-phase gating, per-faction/weapon range
  bands (§6.22), CRT, gunboat 3+ (§6.61) and fort 2+ (§6.62) thresholds, howitzer scatter
  *direction* (§6.64).
- **Melee (§7)** — melee-capable kinds, adjacency, simultaneous CRT, modifiers, the declared-melee
  reaction window (§7.5: `DeclareMelee` → `RetreatBeforeMelee` → `ResolveMelee`, re-deriving
  defenders from current occupants so retreaters are spared), mandatory Dervish advance (§7.6).
- **Set-up order (§9.111, §9.211, §9.321)** — deployment is sequential: the first side
  (`GameState::first_to_set_up`: the Dervish in the Campaign, the Anglo-Egyptians in the
  Historical scenario and Fall of Khartoum) deploys and confirms Ready, which is final; only then
  does the second side deploy (`require_setup_turn`, `RuleError::SetupOrder`). Scenario-fixed
  counters are exempt (`scenario_setup::is_fixed_placement`).
- **Turn/phase sequence (§4)**, day-night from turn track, Dervish desertion (§8.2), Friendlies
  gunboat transport (§5.21), river mines (§10.12) incl. Dervish immunity (§10.14), river chain
  (§10.21–23), the full §9.14 victory-point ledger (`VpSource::points` / `who_scores`, incl. the
  Mahdi's Tomb) and per-scenario victory levels.

### Fully engine-authoritative (every check in `apply_effect` / `can_*`)
- **Line of sight (§6.3, §6.21)** — `has_los` in `los_table.rs` ray-casts through
  `BoardInfo` terrain and hexsides; `can_fire_at` enforces it. Howitzer fire bypasses.
- **Terrain defence modifiers (§6.23)** — `commit_fire_attack` derives the modifier from
  `state.board`: `terrain_chart::defense_modifier` for the target hex plus
  `target_hexside_fire_modifier` for a Crest (−1) or City Wall (−4) hexside the fire crosses.
- **Mandatory die-roll modifiers** — `mandatory_fire_modifiers` derives the AE direct-fire +1
  (§6.24), brigade integrity (§5.54) and the Dervish-only zariba thorn-hedge / trench penalties
  (§9.231–232); `mandatory_melee_modifiers` derives the §7.7 standard modifiers and the §9.232
  trench inversion. A caller-supplied modifier list that differs is rejected
  (`RuleError::FireModifierMismatch` / `MeleeModifierMismatch`).
- **Melee hexside blocking (§7.2)** — `can_melee` checks
  `board.hexside_between().blocks_melee()` (wall, thorn hedge, Khor).
- **Advance-after-combat hexside blockers (§6.82, §7.6)** — `can_advance_after_combat`
  checks `blocks_advance_after_combat()`.

### Fall of Khartoum (§9.3) — enforced
- §9.346 GORDON is immobile and eliminated only when a Dervish unit reaches the Palace hex;
  §9.35 victory is the turn-of-death level shifted by the Dervish-loss penalty (`FoKVictoryLevel`).
- §9.343 both players use the Dervish range table in FoK; §9.345 a British gunboat may cross the
  White↔Blue Nile mouths off-board for 6 MP; §9.344 the Dervish hold the North Fort (forts are
  never captured, only destroyed — §6.54, enforced for movement and advance-after-combat).
  FoK has no Maxim/Howitzer subphase to play (§9.321), so `advance_phase` skips it.
- Set-up: the fixed placements are data in the rules crate's `scenario_setup` (GORDON in the
  Palace); the app's `build_setup_plan` resolves them against the loaded map and emits them as
  ordinary `PlaceUnit` events. The §9.322 Dervish "enter turn one" is their set-up on the
  south/east edge: leaving Setup charges each unit its edge hex's terrain cost.
  `BoardInfo::from_map_data` populates `locations` from named tiles so all of the above resolve
  at runtime.

### Simplified
- Howitzer scatter: the printed Scattergram's directions are the map's (1 = north-west ... 6 =
  west), one hex off the designated target.

### Rules-crate layout
Submodules each own one table or domain: `combat_results_table`, `howitzer_scatter`, `los_table`,
`range_effects`, `terrain_chart` (Terrain Effects Chart + `land_step_cost`), `tables_data` (the
four printed tables as `static` consts), `turn_track`, `reinforcements`, `scenario_setup`
(fixed-hex placements), `unit_id`, `unit_profiles` (compiled roster), `board` (`BoardInfo`),
`board_data` (the board RON, embedded), `sprite_data` (compiled sprite fallbacks), `rng`
(`GameRng`), `tactics` (scripted fixtures shared by tests and the bot). The engine core is the
`effects/` directory: `effect.rs` (`GameEffect`), `error.rs` (`RuleError`), `observation.rs`,
`state.rs` (`GameState` + accessors) with its validators split by domain into
`state/{setup,movement,fire,melee,engineering,stacking}.rs`, `dispatch.rs` (`apply_effect` + turn
flow) and the per-domain `apply_*` modules (`movement`, `fire`, `melee`, `setup`, `river`,
`victory`). Crate-root types live in private modules (`scalars`, `turn`, `unit`, `combat`,
`transport`, `victory`) re-exported from `lib.rs`.

---

## 4. Networking & event sourcing

Message types in `omdurman-net/src/lib.rs`; glue in `omdurman-app` (`net_plugin.rs`,
`net_socket.rs`, `game_record.rs`, `game_apply.rs`, `submit.rs`, `seats.rs`, `seat_arbiter.rs`).
The robustness machinery — submission retransmit, election stabilization, the reorder buffer,
seq-conflict / seq-gap healing, stall auto-reconnect — is specified in CLAUDE.md
("Architecture: event-sourced, peer-to-peer, host-relayed") and not repeated here.

- **`NetMsg`** — `Game { uid, event }` (unsequenced, peer→host; `uid` is the submission
  identity), `Sequenced { seq, uid, event }` (host→all, the *only* form applied locally),
  `Ephemeral` (unreliable, never recorded), `Control` (snapshot handshake, seat requests and
  votes).
- **`GameEvent`** — the only enum whose variants are recorded/replayed: `StartGame`,
  `Effect(GameEffect)` (every game mutation), the sprite-keyed `PlaceUnit` / `RemoveUnit` /
  `MoveUnit`, and the seat events `SeatAssigned` / `SeatCarved`. Non-persistent messages
  (cursors, selections) belong in `Ephemeral`.
- **Late joiners** — request `GameHistory(GameRecord)`, reseed the local RNG from fresh entropy,
  rebuild `GameState::new(scenario)`, replay every event in canonical order.
  Replay goes through the same `game_apply::apply_game_event` as live echoes, synchronously
  and in record order, and never re-records; unit sprites follow via
  `picker::reconcile_unit_sprites`. The dual-map board load is deferred post-replay
  (`PendingMapLoad`) so the live board matches the replayed scenario.
- **Effect application** — no translation layer: the app builds `GameEffect` directly, wraps it
  `GameEvent::Effect`, and on the sequenced echo `game_apply::apply_game_event` calls
  `apply_effect`. A rejected echo is logged, not retried. The user-facing submit sites dry-run
  each action first (`submit::dry_run`, against the engine state projected over this peer's
  unconfirmed submissions) and show the `RuleError` as a dispatch slip instead of submitting.

### Seats, identity, pause, and rejoin
- **Stable identity.** A player is a `PlayerKey` (`omdurman-net`), not a matchbox `PeerId` (which
  changes on every socket rebuild). Native builds persist it in the first free, exclusively
  locked slot file `<config dir>/omdurman/player_key_<n>` (`player_key_store.rs`; the OS drops
  the lock when the process exits, so a relaunch reclaims the same key while a second concurrent
  window gets its own); the web keeps it in `sessionStorage["omdurman.player_key"]` (a reload
  keeps it); it rides in `Ephemeral::PlayerInfo`, sent reliably to each peer on connect, and
  lands on the peer entity as `PeerPlayerKey`.
- **Seat table.** `StartGame { seats, scenario, optional_rules }` commits `Seat { faction, scope:
  Option<CommandScope>, holder: Human(PlayerKey) | Ai }`. The app mirrors it in `seats::Seats`,
  written *only* by `game_apply::apply_game_event` (live echo and replay alike). `peers::Peers`
  gates (`may_act`, `scope_allows`, `is_spectator`, ...) read it with `LocalPlayerKey`; the pure
  logic lives in `seats.rs`. A reconnecting player re-installs the history and is bound again
  automatically. The AI plays a faction whose seats are all AI seats; an AI sub-seat in a faction
  that still has humans claims no units (they become communal).
- **Presence and pause (local, unrecorded).** `seats::SeatPresence` tracks each human holder as
  connected or disconnected-since. Any absent holder pauses the game at once: `Peers::may_act`
  returns false (every action gate and the End Phase button inherit it) and the host's AI waits.
  After `SEAT_ABANDON_SECS` (60 s) the seat is *abandoned*. The clock is each peer's own view.
- **Claims and votes (host-arbitrated).** Guests never submit seat events. They send
  `Control::SeatRequest` (`ClaimAbandoned` / `TakeOver` / `HandToAi` / `ClaimFromAi`) to the
  host; `seat_arbiter::seat_control` decides against the seat table projected over the host's
  unconfirmed seat events (`seats::decide_request`). An abandoned seat is granted outright; the
  rest open a unanimous vote (`seats::VoteBook`) of every connected seated human except the
  requester (`SeatVoteOpen` / `SeatVote` / `SeatVoteClosed`; any "no" or `SEAT_VOTE_SECS` (60 s)
  denies; zero voters approve). The host then submits `GameEvent::SeatAssigned { seat, previous,
  holder }` or `SeatCarved { faction, scope, holder }`, whose apply arms re-check the table
  deterministically (a stale `previous`, a double seat, or an empty/mismatched scope is rejected
  on every peer). A host failover drops open votes; clients expire their ballots at the deadline.
- **Wire format.** The seat `Control` and `GameEvent` variants are appended, but `StartGame`'s
  shape and `PlayerInfo` changed, so every peer must run the same build. Old JSON saved games
  still load (their `assignments`/`ai`/`commands` are ignored: seatless, reviewable).

### Determinism holds when
Same canonical record on every peer, pre-rolled dice, deterministic (sorted-lexicographically)
peer ordering for host election (`host_id`). **Breaks if** the record is corrupted/truncated,
peers run different builds (different compiled data), or `GameState::new` differs across peers.
Message loss, reordering and transient dual-host streams are healed by the receive path
(CLAUDE.md).

### Remaining netcode gaps (resilience, not correctness)
Two are tracked in [`open-issues.md`](open-issues.md): `retry_snapshot_request` re-sends every
2 s with no ceiling, and `flush_pending` retains a whole reliable broadcast — and resends it to
every peer — when the send to any one peer fails.

---

## 5. App layer (omdurman-app, omdurman-board-ui, omdurman-hexmap)

- **Modes.** Top-level `AppMode::{Menu, Lobby, Game}` (`M` returns to the menu), with
  `AppState::{Splash, Lobby, InGame, Spectating}` underneath; `Spectating` is the timeline
  scrubber (`timeline.rs`) that reviews a recorded game by rebuilding state to any event index.
  There is no in-app editor: board and asset authoring happen in `tools/map-editor` and
  `tools/asset-editor`. Behaviour is gated on the active mode, not a build flag.
- **Dual-map.** `ActiveEditMap` (local) tracks the live board; `PendingMapLoad` defers a (re)load
  to the next frame; `LoadedAnnotations` holds both boards (all three in `omdurman-board-ui`). A
  play view (Game) follows its scenario's board for the whole session; the map editor's board
  follows `EditorBoard`.
- **Board + sprite data.** `LoadedAnnotations` is seeded from the board RON files
  (`omdurman-app/assets/boards/`, embedded by the rules crate's `board_data`). **Unit sprite
  metadata is global, not per-board:** compiled fallbacks in `sprite_data` (keyed by `UnitId`),
  overlaid at startup by `omdurman-app/assets/sprite_annotations.ron` into
  `SpriteAnnotationsResource`. The map editor writes the board RON, the sprite annotations and
  the unit-sheet grids back to disk; the game only reads them.
- **Board input.** Every board click goes through one router: `board_click::route_board_clicks`
  (pointer-gated by `MapPointerInputSet`, so a click over egui never reaches the board) asks the
  pure `click_mode` which `ClickMode` owns the click and emits exactly one mode message per edge
  (picker, fire, melee, advance, retreat, river placement).
- **Hexmap.** `HexLayout` (pointy orientation) must be inserted manually with calibration data;
  `world.rs` does axial↔world conversion with round-trip tests.

### Legibility surfaces — "what just happened, and what can I do?"

The UI is built around the principle that a player who has not read the manual can still follow
what the engine is doing and why. Every citation deep-links into the in-app Rulebook tab
(searchable, scrollable, parsed from `Boardgame - Remember_Gordon/Manual/RememberGordonManual.md`).

- **§-title index.** `Rulebook::title_of(number)` resolves a section number to its short title
  ("§5.26 Units stop on entering enemy ZOC"); citations render as titled chips
  (`Rulebook::render_ref_chips`) or inline links (`rulebook::render_refs_plain`) rather than as
  opaque numbers.
- **Combat Resolution Card** (`combat_card.rs`). Every fire/melee resolution emits a structured
  `Observation::FireResolved` / `Observation::MeleeResolved` (in `effects/observation.rs`)
  carrying the full attack bundle — firers, target, per-modifier breakdown with paragraphs, die
  roll, modified roll, CRT factor row, result, casualties. The card surfaces this as a fadeable,
  deep-link-rich breakdown: each modifier ("+1 AE direct fire §6.24", "-1 terrain defence
  §6.23") attributes itself to its rule, and the casualty list names the units lost. Late-join /
  replay produces the same card stream.
- **Dispatch slips** (`dispatch.rs`). Every non-combat `Observation` (LeaderKilled,
  DemolitionResolved, WallBreached, VictoryScored, GordonEliminated, FriendliesDisembarked,
  FortDestroyed, UnitEliminated) and every refused action renders as a paper-card "field
  telegraph" slip with its authorising § references deep-linked. The slip queue is bounded and
  ages out.
- **Turn telegram** (`telegram.rs`, `ui_plugin::controls::telegram_overlay`). Each turn's
  telegram is a centred modal over the dimmed board; play waits until it is dismissed. Without a
  flavour model it reports the turn's own events (`telegram::fallback_telegram` over
  `turn_summary::TurnSummary`).
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
- **Picker sprite tooltips** (`picker/sidebar.rs`). Hovering a sprite in the unit-picker sidebar
  shows the counter's resolved profile (identity, fire/melee/move factors, weapon, kind, printed
  text, fires-twice flag) plus a §2.3x deep-link to its section.

---

## 6. Current state of the code

The Fall-of-Khartoum scenario is playable end-to-end (set-up → rules-enforced turns with visible
results → §9.35 verdict).

- **Effect atomicity.** `apply_effect` validates before it mutates, so a rejected effect leaves
  the state untouched and a peer that rejects an effect cannot diverge from one that accepts it:
  `resolve_fire_attack` runs `validate_fire_resolution` before `commit_fire_attack` /
  `commit_fired_markers`; `apply_resolve_melee` keeps the declared melee (and its pre-rolled dice)
  until the resolution commits; `advance_phase` clears `vacated_by_combat` only after its guards.
  Covered by the `rejected_*` regression tests, the `*_is_atomic` Kani harnesses (§9) and the
  bot's per-effect invariants.
- **The view is a projection of the engine.** Unit sprites are derived from `GameState`
  (`picker::reconcile_unit_sprites` spawns, moves and despawns counters), so a rejected effect
  never animates; `submit::dry_run` refuses an illegal action before it is submitted.
- **Movement is turn-gated and engine-authoritative:** a unit moves only on its owner's turn
  (`Peers::may_act` in the app, `RuleError::NotYourTurn` in the engine). `MoveUnit` carries the
  entered `path` (one adjacent step per hex), so the engine costs each step with
  `land_step_cost`, classifies gunboat up/downstream and applies the ZOC stop per hex; wall
  hexsides block movement (§5.23) in `can_move_unit_to`. The picker's reach overlay uses the
  same step cost and `effective_movement_at_night`, so night reach matches what the engine will
  accept.
- **Retreat-before-melee is fully implemented and wired** (`retreat.rs`: defender-gated overlay +
  `RetreatBeforeMelee`, validated by `can_retreat_before_melee`).
- **`overview.rs` is a working unit-overview side panel.**
- **Combat feedback and game end are surfaced:** the combat card, dispatch slips and the turn
  telegram (§5), and `ui_plugin::victory::victory_modal` shows the final scenario verdict when
  `game_over` is set.
- **Engine rule details (audit-driven):**
  - §6.42 Maxim second fire: `units_fired_this_phase` is cleared on entering the
    `MaximSecondAndHowitzer` subphase; non-Maxim/non-Howitzer units are rejected with a typed
    `RuleError::WrongWeaponForSubphase`.
  - §6.53 Royal Engineers demolition: `apply_resolve_demolition` removes forts / breaches walls at
    end of turn when the engineer remains adjacent and undisrupted, auto-emitted via
    `end_player_turn`. A breach (§6.53, §6.63) eliminates a unit standing at the wall
    (`breach_victim`); an Anglo-Egyptian leader is never the casualty.
  - §9.14 VP: `VpSource::points` / `who_scores` encode the printed schedule — Khalifa 10 VP,
    Isa Zachneih 1 VP (a distinct source), forts 0 VP, Friendlies losses score for the Dervish
    (1 east bank / 3 west bank), and the Mahdi's Tomb's 25 VP go to whoever controls it at the
    end (`score_mahdis_tomb` awards `MahdisTombTaken` or `MahdisTombHeld`). The two
    auto-decisive conditions (all-Dervish-eliminated / all-AE-west-bank-eliminated) are checked
    in `finish_game`.
  - §9.35 FoK British survival ladder: `FoKVictoryLevel::resolve` takes `scenario_end_turn`,
    distinguishing Marginal/Tactical/Decisive based on how long GORDON survived.
  - §8.1 night ranges: the weapon's *max range* is halved (not the distance); the day table is then
    consulted at the *physical* distance, matching the rulebook's AE-rifle worked example.
  - §5.21 Friendlies transport: full gating (Isa-Zachneih prerequisite, adjacency at load,
    turn-sequencing between Loaded/Crossing/ReadyToDisembark). A gunboat sunk while carrying a
    unit eliminates the loaded unit (`ElimCause::LostWithTransport`).
- **Observation side-channel:** `GameState.observations: Vec<Observation>` is pushed by
  `apply_effect` and drained by the app after each event application (`drain_observations` →
  `PendingObservations` resource → `ObservationEvent` Bevy messages). Carries demolition results,
  leader deaths, VP awards, and fort/wall destruction for dispatch slips, sounds, and animations.
  Serialized so replay produces the same stream.
- **Engine-authoritative LOS / terrain defence / hexside blocking / modifiers** (§3): `can_fire_at`
  / `can_melee` / `can_advance_after_combat` are the single authority, and the engine derives
  every mandatory die-roll modifier itself; the app neither supplies terrain modifiers nor gates
  on these checks separately.

---

## 7. Open work

Known open work — rules deviations, UI gaps, net and app plumbing, bot and verification
follow-ups — is tracked in one list: [`open-issues.md`](open-issues.md). The solo / AI opponent
is `omdurman-bot`, driven in-app by `bot_player.rs`.

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
- `./scripts/kani.sh -p omdurman-types -p omdurman-rules` — the proof suite (§9; the script
  bakes in `-Z stubbing` and `--features kani`, see §9). Needs WSL on Windows.

CI (`.github/workflows/ci.yml`) runs, per push/PR: `cargo fmt --check`, `cargo clippy
--workspace --all-targets -- -D warnings`, `cargo test --workspace` (Linux/macOS/Windows) and
the traceability gates; `deploy.yml` builds the Pages site with `trunk build --release`. The
Kani suite runs only on manual dispatch (§9).

---

## 9. Formal verification (Kani)

91 proof harnesses make up the [Kani](https://model-checking.github.io/kani/) suite:
22 in `omdurman-types/src/lib.rs` (hex geometry); 23 in `omdurman-rules/src/verification.rs`
(`value_enum!` conversions, die arithmetic, victory ladders, the §9.14 VP schedule — 18 written
out plus 5 generated by the `prove_value_enum!` macro, one per `value_enum!` type); 19 in
`omdurman-rules/src/effects.rs` (the `apply_effect` atomicity/monotonicity set) and 10 in the
per-domain effect modules (`effects/river.rs` 5, `effects/fire.rs` 3, `effects/victory.rs` 2);
and the table- and chart-backed remainder in `range_effects` (4), `turn_track` (4), `los_table`
(3), `combat_results_table` (2), `terrain_chart` (2), `howitzer_scatter` (1) and
`reinforcements` (1). The 4 harnesses in the feature-gated `quantifier_experiment.rs` are not
part of the suite (see Boundaries). Harnesses live in `#[cfg(kani)] mod verification` blocks
inside the file they verify, so private items stay reachable; the crate-root set is the
`verification` module declared in `lib.rs`.

Harnesses verify `SUCCESS` given enough memory:
`score_elimination_records_exactly_what_it_scores` peaks around 13–14 GB and OOMs on a 12 GB
machine — a resource limit, not a proof failure. Harnesses awaiting a re-run after engine
changes are listed in [`open-issues.md`](open-issues.md). [`kani.md`](kani.md) is the field
manual: what the proofs taught, diagnosing an OOM, auditing a weak proof.

**Kani has no native Windows support.** `scripts/kani.sh` shells into WSL (Debian) against the
repo at `/mnt/c/...`, with a separate `CARGO_TARGET_DIR` so Linux artifacts never collide with the
host `target/`. On Linux/macOS it calls `cargo kani` directly. Kani ships its own pinned nightly
and ignores `rust-toolchain.toml`, so the 1.98.0 pin does not interfere. No `kani` dependency
belongs in any `Cargo.toml` — the crate is auto-injected.

The script bakes in two defaults every invocation needs:

- `-Z stubbing` — lets harnesses stub heavy-but-property-neutral cascades (e.g. `end_player_turn`
  under the `AdvancePhase` atomicity harness, `resolve_melee_combat` under `ResolveMelee`). A single
  stubbed call site took one harness from ~1M SSA steps (OOM) to ~139k (SUCCESS).
- `--features kani` — both `omdurman-rules` and `omdurman-types` define an empty `kani` cargo
  feature; the rules crate compiles its four `debug!` tracing call sites out under it. Extra args
  are forwarded, so `-Z concrete-playback` and friends still work.

`KANI_JOBS=<N>` (env var, not baked in) verifies harnesses in parallel via cargo-kani `--jobs`,
which requires `--output-format=terse` — the script adds both. Measured 3.0–3.4× wall-clock
(the `omdurman-types` package and a 4-harness `omdurman-rules` slice, 16-core box; per-harness
solve times are unchanged).

**The proofs are gated to manual dispatch** (`.github/workflows/ci.yml`, `kani` job;
Kani 0.67.0 is installed manually there because the action's default `cargo-kani` command
verifies the whole workspace with no package selection). GitHub runners kept killing the job
with shutdown signals (exit 143) mid-suite — every harness green, no proof ever failing — so the
job does not run per push/PR; trigger it via `workflow_dispatch` when wanted, and treat
`scripts/kani.sh` locally as the authoritative check. The job caches `~/.kani` (the bundled
toolchain) *and* the proof `CARGO_TARGET_DIR` per Kani version + lockfile — a cold proof
build costs ~1 min of pure recompilation before the first solve on a fast box — and sets
`KANI_JOBS=4`.

What the proofs buy over tests: they close the domain. The hex-geometry set exists because
`distance` used the wrong cube axis (`s = -q-r` instead of `s = r-q`) and disagreed with
`neighbors`, which made `line_between` stall short of its target for 65% of on-board firing pairs
and silently corrupted the §6.3 LOS ray. Every test passed throughout — they sampled
`(0,0) → (3,0)`, a pure-axis case that works under both conventions.
`adjacency_iff_distance_one` pins the two together permanently.

Proofs participate in the traceability matrix through `proofs = [...]` (parallel to `tests`), and
are annotated `// §N` above `#[kani::proof]`. The comment style rather than `#[rulebook]` is
deliberate: the proof modules are `cfg(kani)` on the *lib*, where dev-dependencies — and so the
proc-macro — are unavailable.

### Boundaries

- **Table contents are provable; their transcription is not.** The four printed tables are
  fixed-size `static` const arrays in `tables_data` (e.g. `CRT: [[CombatResult; 10]; 9]`, so row
  lengths are compile-time facts), which lets the `range_effects`, `combat_results_table` and
  `howitzer_scatter` harnesses close their full input domain. Kani cannot check that the
  constants match the printed tables: the RON transcriptions under
  `Boardgame - Remember_Gordon/tables/` are the authoring source, and `#[cfg(test)]` parity tests
  fail cell-by-cell on drift (`tables_data::tests::{crt_ron_matches_const,
  range_effects_ron_matches_consts, scattergram_ron_matches_const, los_ron_matches_consts}`,
  plus `terrain_chart::tests::engine_matches_the_transcribed_terrain_effects_chart`). A proof is
  exactly as true as the constants it reads.
- **The effects harnesses prove bounded slices, not the whole engine.** What makes them
  tractable, all measured rather than guessed: `GameState`'s per-turn maps and the printed tables
  avoid `HashMap` (hashbrown's probe loop unwinds past iteration 600), and `BoardInfo`'s six
  `IndexMap`/`IndexSet` fields carry a deterministic `BuildHasherDefault<DefaultHasher>`
  (`board.rs`) — the default `RandomState` seeds through `getrandom`, an unmodellable `syscall`
  on a *live* path (`BoardInfo::default()` runs inside every `GameState::new`). Beyond that: unit
  generation is concrete (two units, fixed trip count — a symbolic `take(n)` unrolls the iterator
  protocol past any unwind bound), phases are pinned where a property lives on one arm of the
  7-arm phase match (CBMC explores paths, so symbolic arms multiply the step count ~10×), and
  property-neutral cascades are stubbed. The turn-end cascade's own mutations are therefore
  *not* proved atomic here — that stays with the `rejected_*` regression tests and the bot's
  playthrough invariants.
- **`omdurman-app` is out of reach** — it enables `bevy/dynamic_linking`.
- **Quantifiers (`kani::forall!`) don't pay here.** A measured prototype lives in
  `omdurman-rules/src/quantifier_experiment.rs` (gated behind `--features kani-quantifiers`
  plus cargo-kani's `-Z quantifiers`; never compiled by CI or the suite). Kani 0.67 quantifiers
  are `usize`-range only with constant bounds for the SAT backend, reject *any* function call in
  the quantified body (kani-compiler ICE: "Detected recursions in the usage of quantifiers"),
  and reject loops in the body (CBMC: "quantifier must not contain loops"). The maximally
  flattened formulation — discriminant codes, sentinel-padded static unrolling — verified
  SUCCESS but solved ~1.2–1.4× *slower* than the loop formulation of the same universal
  property (23.8s vs 17.5s seeded, 19.5s vs 17.1s empty, same ~38k SSA steps). Revisit if
  quantifiers gain call/loop support.

When adding proofs: prefer concrete loop trip counts over unwind bounds — a symbolic bound
(`take(n)`) unrolls forever, and a too-small bound does not always fail loudly (one harness
produced a *counterexample that concrete playback could not reproduce* at unwind 6 and verified
at 14; the bound, not the engine, was at fault). Confirm a loop-bearing harness is non-vacuous
with a deliberately-false assertion before trusting a `SUCCESSFUL` verdict. Counterexamples need
`-Z concrete-playback --concrete-playback=print` (insertion with `--concrete-playback=inplace`
currently errors on `cfg(kani)` modules — reproduce by hand instead).

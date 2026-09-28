# Open issues

The single list of known open work. Collected on 2026-09-27 from the docs
audit: the retired audit, plan and play-test documents, plus
`review-2026-09.md` and `playtest-2026-09-fok.md`. Those two documents stay as
dated records, and their open items live here. Items cite symbols rather than
line numbers; re-check an item against the code before acting on it. Delete an
item when it is fixed.

## Rules engine: deviations from the manual

- **§6.42 Maxim + howitzer.** The rule lets howitzer fire combine with Maxim
  fire when it impacts in the intended hex. The engine resolves them as
  independent attacks (a documented AMBIGUITY note), and a combined howitzer
  attack of several gunboats shares one impact roll.
- **§10.11 / §10.21 secrecy.** Mines and the chain are "secretly recorded".
  The board hides them from the Anglo-Egyptian seat, but the event log
  carries them (see also "private dice" below).
- **§10.21 chain shape.** The chain must be a line of adjacent Nile hexes;
  that it runs *across* the river is not checked.
- **§10.22 chain.** A British gunboat should be able to enter a chained hex
  and stop there. Instead `BlockedByChain` refuses the entry (which also
  keeps every gunboat from crossing, §10.23).
- **§6.41 allocate, then resolve.** "Allocate all fire attacks, then resolve"
  is enforced by the UI's allocation tray only; the engine accepts attacks
  one at a time.
- **§7.7 casualty choice.** Losses fall on units in list order; the owning
  player does not choose them.

## Rules questions (the manual is ambiguous; current reading in brackets)

- **§6.3 "Wall (b)".** The LOS table lists "Wall (b)" in the Ground/Ground,
  Ground/Rough and Rough/Ground cells. [The wall blocks: rampart units (note
  b, rough level) can be fired at over a wall only from rough ground,
  hilltops and gunboats.] The alternative reading -- a unit on the rampart
  sees over its own wall -- would let wall defenders and ground attackers
  fire at each other across it, at the Terrain Effects Chart's −4.
- **§5.44 building vs breach.** A building hex just inside a breached wall:
  ZOCs "extend both ways across a breach" but "not into a hut or building
  hex". [The hut/building clause wins.]
- **§6.14 "fired at once".** [Maxims and gunboats may be fired at more than
  once a phase.] The exception may only mean they *fire* twice.
- **§6.24 Maxim second fire.** [No +1 accuracy bonus for Maxim second fire or
  howitzer fire; +1 for batteries breaching walls or firing at the chain.]
- **§7.5 advance after a retreat.** [A hex emptied by a retreat before melee
  opens an advance window for the attackers.] §7.6 grants the advance only
  when a melee *eliminates* the defenders.
- **§7.7 mixed Friendlies.** [The Friendlies' Dervish +2 applies only when the
  whole Anglo-Egyptian side of a melee is Friendlies.]
- **§5.51 stacking.** [Forts count against the four-unit limit; the Dervish
  artillery is its own stacking group.]
- **§9.321 Fall of Khartoum set-up.** "Adjacent to any wall hex": [a hex with
  a wall or gate hexside of its own]. "Hut hexes of Khartoum": [any hut hex
  on the map, Tuti and Hogali included]. "One Egyptian battalion artillery
  unit": [any Anglo-Egyptian battery]. Which side of the Khartoum rampart is
  "inside" for LOS note b: [the side nearer the Palace].
- **§2.32 named gunboats' Maxims.** [Not modelled as a Maxim second fire.]

## UI

- The melee-declared card shows the defender's retreat instructions to
  spectators, too: anyone who is not the attacker gets them (`melee.rs`).
- The advance slip says "may advance" even when the §7.6 Dervish advance is
  mandatory. There is no advance reminder at End Phase.
- "Review allocations" is not gated to the fire phases (`actions_panel.rs`).
- Deployment traps get no warning (e.g. the Hogali pocket in Fall of Khartoum).
- Fired pips are never drawn (TODO in `render.rs`).
- The Mahdi's Tomb shows up only as a VP row: there is no board marker or
  control indicator.
- There is no LOS or Scattergram chart tab, and nothing sends a `ChartSheetRequest`.
- There are no Nile flow arrows (`flow_at`) and no marker for units stopped in
  an enemy ZOC (`zoc_stopped_this_turn`).
- Fall of Khartoum: the picker cannot plot a gunboat's White↔Blue Nile mouth
  crossing (§9.345, a single 6-MP move between non-adjacent hexes); only the
  bot uses it.
- River mine and chain placement (`river_placement.rs`) shows no legal band;
  the engine refuses hexes outside it.
- The event viewer (V key) shows the Dervish mine and chain placements to the
  A-E player. Redact them until mines and the chain are secret (§10.11/§10.21).
- Campaign Dervish set-up has no per-class groups, placed/total counters or
  per-class zone tints (§9.111).
- Historical set-up has no lettered-hex highlight or snap for the leaders and
  no three-hex halo (the hover tooltip names why a hex is refused).
- The desertion panel hides exempt units instead of greying them out, and has
  no "no VP for deserters" note (§8.2).
- The LOS overlay has no ray between two chosen hexes.
- Every fire target gets the same red ring (`fire_target_overlay_mesh`): there
  is no colouring by range band (×3/×2/×1/×½).
- Melee target rings don't mark wall hexsides as blocked, or gates and breaches
  as open (§7.2).
- Artillery-only targets are filtered out silently: no tooltip explains "only
  artillery" or the 3+ (gunboat) and 2+ (fort, wall) thresholds (§6.61–6.63).
- Howitzer fire: impact rings are drawn and the result card names the impact
  hex, but there is no aimed-to-impact arrow and no warning when the shot lands
  on friendly units (§6.64).
- The fire-allocation tray cannot reorder attacks (§6.41: resolve "in any order").
- The retreat overlay shows the destinations but not the two-hex path (§7.5).
- The unit inspector has no status pips (loaded, engines lost/drifting,
  constructing, demolishing, retreated this turn), and there is no right-click
  unit card.
- The game-over screen leaves out the victory arithmetic (§9.14 superiority
  table, §9.24 level subtraction, the alternative decisive victories), and the
  VP ledger hides zero-count rows.
- Melee: every melee-capable unit of the selected hex attacks (no holding
  units back), and the UI declares from one hex only, though the engine
  accepts attackers from several adjacent hexes (§7.6).
- The fire tray lets artillery at a garrisoned fort choose the fort or the
  garrison, but the preview card on hover shows the garrison shot only.
- The phase-sequence indicator (`UiPhaseState::phase_sequence`) has four rungs,
  folding Direct and Maxim/Howitzer fire into one.
- Polish: selection pulse, howitzer scatter animation, Zariba build animation.
- Offline lobby roster.
- Probably still open: zoom to cursor; player names re-randomised on relaunch.
- Unconfirmed, from the play-test and review notes:
  - The unit list doesn't group identical counters (Kehena, Degheim).
  - Fire slips pile up.
  - The lobby shows "(claimed by a player)" twice.
  - Player colours can clash.
  - Turn and phase are shown in five places (review 4.11).
  - Surfaces overlap (review 4.13).

## Net and app plumbing

- `handle_socket` is still about 520 lines; split out a `GameSession`.
- `GameMap` (hexmap) and `BoardInfo` (rules) duplicate board topology.
- Private dice: every roll is visible in the event log.
- Snapshot-request retry is unbounded.
- Broadcast retention is all-or-nothing.
- Per-game app state is not reset between games in one process.
  `generate_newspaper`'s `Local<bool>` "done" flag is never reset, so a second
  game never gets its own newspaper. The telegram cursor, dispatches and cards
  may have the same problem (review 3.3; unconfirmed).
- `SpriteAnnotationsResource` is defined twice, in `omdurman-app/src/sprites.rs`
  and `omdurman-board-ui/src/sprites.rs`, so the two can drift.
- 25 systems still take `Option<Res<GameStateResource>>`.

## Bot / LLM

- The planner overwrites its cache unconditionally (`cache.0 = parsed.cache`
  in `omdurman-bot/src/llm.rs`, `advise_turn`), so a malformed or empty reply
  wipes it. The
  observer keeps its previous cache. Decide which behaviour the planner should
  have.

## Verification

- `score_elimination_records_exactly_what_it_scores` runs out of memory with
  ~10 GB free: reading the pushed `Observation`s back out of the ledger
  dominates the propositional reduction (see `docs/kani.md`). Verify it on a
  bigger box (`run-kani.sh`), or find a cheaper way to state the check.
- 22 Kani harnesses have no `// §N` annotation and are not tracked in
  `traceability.toml` (e.g. `distance_is_symmetric`, `game_over_is_absorbing`,
  `sink_chain_is_atomic`, `place_mine_is_atomic`).

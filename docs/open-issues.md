# Open issues

The single list of known open work. Collected on 2026-09-27 from the docs
audit: the retired audit, plan and play-test documents, plus
`review-2026-09.md` and `playtest-2026-09-fok.md`. Those two documents stay as
dated records, and their open items live here. Items cite symbols rather than
line numbers; re-check an item against the code before acting on it. Delete an
item when it is fixed.

## Rules engine: deviations from the manual

- **§9.113 turn-1 composition.** The Anglo-Egyptian turn-1 order of
  appearance (three gunboats, "Friendlies", Egyptian Cavalry, Horse Artillery,
  two Egyptian Division brigades) is not enforced. `validate_campaign_reinforcements`
  checks only the 12-units-per-turn cap.
- **§9.113 leaders by turn 4.** "All three leaders must be in play by the end of
  turn four" appears only as a doc comment in `reinforcements.rs`. Nothing enforces it.
- **§9.322 Fall of Khartoum entry.** Deploying Dervish units *onto* a south or
  east edge hex, instead of entering through it, gives them a free first hex.
- **§6.54 fort defence.** The −3 die-roll modifier against units inside a fort
  is not applied: `mandatory_fire_modifiers` has no fort term. The FoK forts
  Makran and Buri (printed 4-1-0) are not modelled either.
- **§6.82 / §7.6 multi-unit advance.** `AdvanceNotVacant` blocks a second unit
  advancing into the same vacated hex, although the rules allow advancing up to
  the stacking limit.
- **§7.5 retreat, then attack.** Enemy units whose melee has not been resolved
  may attack a unit that retreated next to them. The engine holds a single
  `pending_melee`, so this is not supported.
- **§10.12 river mine.** `apply_river_mine` never checks that the gunboat is in
  the mined hex.
- **§10.12 engines lost.** A gunboat that has lost its engines should only
  drift, but it can still move under power: the movement validators have no
  engines-lost check.
- **§10.21 river chain.** Any 1–4 Nile hexes are accepted. The rule is a line
  of hexes strung *across* the river.
- **§10.11 / §10.21 secrecy.** Mines and the chain are "secretly recorded", but
  the engine and the event log treat them as public (see also "private dice"
  below).
- **§9.232 entrenched LOS.** "Entrenched units may be fired over": this
  line-of-sight exception is not modelled.
- **§9.211 / §9.212 Historical deployment.** The set-up hexes, the
  out-of-LOS requirement and "within three hexes of the leader" are not enforced.
- **§5.3 Zariba construction.** "Begins *and* ends the player turn adjacent" is
  not checked as such. The UI offers "Construct Zariba" only to the Royal
  Engineers, but the rule allows any Anglo-Egyptian infantry unit.
- **§5.21 Friendlies transport.** The Cross step's destination is the gunboat's
  own hex. `ReadyToDisembark` re-offers Disembark, which the engine then rejects.
- **§6.64 howitzer scatter.** The scatter distance is simplified to one hex.
- **§6.42 Maxim + howitzer.** The rule lets howitzer fire combine with Maxim
  fire when it impacts in the intended hex. The engine resolves them as
  independent attacks (a documented AMBIGUITY note).
- **§9.111 Campaign set-up zones.** The Dervish set-up zones (Isa Zachneih,
  the walled city, forts, south-edge gunboats) are not enforced.
- **§10.11 / §10.21 placement band.** "South of the E–W hexrow in which the
  Khor Shambat empties into the Nile" is not checked for mines or the chain.
- **§10.12 mine stop.** A British gunboat entering a mined hex is not stopped.
- **§10.22 chain.** A British gunboat should be able to enter a chained hex
  and stop there. Instead `BlockedByChain` refuses the entry.
- **Undecided.** Which side of the Fall of Khartoum rampart counts as "inside"
  for the §9.321 set-up zone.

## UI

- The melee card always says "Defenders may retreat", even when no defender can.
- The melee-declared card shows the defender's retreat instructions to
  spectators, too: anyone who is not the attacker gets them (`melee.rs`).
- The advance slip says "may advance" even when the §7.6 Dervish advance is
  mandatory. There is no advance reminder at End Phase.
- "No legal route" is only logged, never shown to the player.
- "Review allocations" is not gated to the fire phases (`actions_panel.rs`).
- Deployment traps get no warning (e.g. the Hogali pocket in Fall of Khartoum).
- Fired pips are never drawn (TODO in `render.rs`).
- The Mahdi's Tomb shows up only as a VP row: there is no board marker or
  control indicator.
- There is no LOS or Scattergram chart tab, and nothing sends a `ChartSheetRequest`.
- These engine features have no UI: `SinkChain` (neither way of sinking the
  chain), `DriftGunboat`, `RiverMine` (so a mine never goes off in play),
  `flow_at` (no Nile flow arrows), `zoc_stopped_this_turn`.
- Friendlies transport: the Cross step offers no destination pick (any Nile hex
  adjacent to the west bank, §5.21), and there is no tracker for the three-turn
  mission. Needs the §5.21 engine fix above.
- River mine and chain placement (`river_placement.rs`) shows no legal band and
  gives no guidance on the chain's shape. Needs the §10.21 engine fix above.
- The event viewer (V key) shows the Dervish mine and chain placements to the
  A-E player. Redact them until mines and the chain are secret (§10.11/§10.21).
- Campaign Dervish set-up has no per-class groups, placed/total counters or
  per-class zone tints (§9.111).
- Historical set-up has no lettered-hex highlight or snap for the leaders, no
  three-hex halo and no out-of-LOS warning (the UI side of §9.211/§9.212 above).
- The desertion panel hides exempt units instead of greying them out, and has
  no "no VP for deserters" note (§8.2).
- The LOS overlay has no ray between two chosen hexes, and it ignores
  intervening units (`los_from` passes `|_| None`).
- Every fire target gets the same red ring (`fire_target_overlay_mesh`): there
  is no colouring by range band (×3/×2/×1/×½).
- Melee target rings don't mark wall hexsides as blocked, or gates and breaches
  as open (§7.2).
- Artillery-only targets are filtered out silently: no tooltip explains "only
  artillery" or the 3+ (gunboat) and 2+ (fort, wall) thresholds (§6.61–6.63).
- Howitzer fire: impact rings are drawn, but there is no aimed-to-impact arrow,
  no scatter display and no warning when the shot lands on friendly units (§6.64).
- The fire-allocation tray cannot reorder attacks (§6.41: resolve "in any order").
- The retreat overlay shows the destinations but not the two-hex path (§7.5).
- The unit inspector has no status pips (loaded, engines lost/drifting,
  constructing, demolishing, retreated this turn), and there is no right-click
  unit card.
- The game-over screen leaves out the victory arithmetic (§9.14 superiority
  table, §9.24 level subtraction, the alternative decisive victories). Campaign
  and Historical have no live "if the game ended now" projection, and the VP
  ledger hides zero-count rows.
- The phase-sequence indicator (`UiPhaseState::phase_sequence`) has four rungs,
  folding Direct and Maxim/Howitzer fire into one. During Movement it already
  shows Def as done.
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

- The river-mine and elimination Kani harnesses have not been re-run since
  the `eliminate_unit` changes.
- 22 Kani harnesses have no `// §N` annotation and are not tracked in
  `traceability.toml` (e.g. `distance_is_symmetric`, `game_over_is_absorbing`,
  `sink_chain_is_atomic`, `place_mine_is_atomic`).

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
- **§6.14 "fired at once".** [The Maxim and gunboat exception is to *firing*
  twice (§6.42); every unit, Maxims and gunboats included, may be fired at
  only once per fire phase. A howitzer shell on its intended hex and a Maxim
  second fire may both strike the same hex in the second subphase.]
- **§6.24 Maxim second fire.** [The +1 accuracy bonus applies to Maxim fire,
  first and second, and to batteries breaching walls or firing at the chain;
  not to howitzer fire, which ignores line of sight and scatters.]
- **§7.5 advance after a retreat.** [A hex emptied by a retreat before melee
  opens an advance window for the attackers.] §7.6 grants the advance only
  when a melee *eliminates* the defenders.
- **§7.7 mixed Friendlies.** [The Friendlies' Dervish +2 applies only when the
  whole Anglo-Egyptian side of a melee is Friendlies.]
- **§6.54 forts in melee.** The manual gives no casualty order for a
  garrisoned fort. [The fort melee-defends with its value but falls LAST:
  "losses must be taken from meleeing units first" (§7.7) -- the garrison,
  then its leaders, then the fort, which only infantry can destroy (§6.54b).
  A fort is never disrupted by a D. An empty enemy fort falls to any
  elimination of one or more (the 2+ is §6.62's artillery rule). A fort that
  outlives its garrison still holds the hex: no advance (§7.6), and it must
  be stormed again.]
- **§5.51 stacking.** [Forts count against the four-unit limit; the Dervish
  artillery is its own stacking group.]
- **§9.321 Fall of Khartoum set-up.** "Adjacent to any wall hex": [a hex with
  a wall or gate hexside of its own]. "Hut hexes of Khartoum": [any hut hex
  on the map, Tuti and Hogali included]. "One Egyptian battalion artillery
  unit": [any Anglo-Egyptian battery]. Which side of the Khartoum rampart is
  "inside" for LOS note b: [the side nearer the Palace].
- **§2.32 named gunboats' Maxims.** The counter prints "5·6×2·12/18". [The
  Maxims (6, "fire twice per turn") are a second weapon, fired once in each
  fire subphase at a target of their own, independently of the artillery and
  howitzer factor (5).]
- **§6.3 footnote pairs.** Box entries such as "Units (3,6)" carry two
  footnotes. [They combine with "and": the entry blocks only when both hold.]
- **§8.2 desertion rounding.** "1½ times the roll of one die". [Rounded up: a
  roll of 1 deserts 2 units, a 3 deserts 5.]
- **§9.14 west-bank decisive victory.** "Eliminates all Anglo-Egyptian units
  on the west bank (excluding gunboats)". [Judged only once there were such
  units: at least one fell on the west bank and none stands there now.
  "Friendlies" count once carried across; on the east bank they do not.]
- **§9.35 loss penalty.** "The Dervish player then loses one victory level".
  [Applies only to a Dervish result (GORDON killed by turn six), which it can
  turn into a British win; it never enlarges a British win.]
- **Combat Results "D".** "½ (round up) of the units in the target hex are
  disrupted" does not say who picks them. [At random among the undisrupted
  units, by a draw rolled into the effect with its dice.]
- **§6.51 leaders under fire.** Anglo-Egyptian leaders fall only by clause
  (a) or (b). [Fire never touches them: a hex holding only leaders is no
  fire target, and they are not counted among the "units in the target hex"
  for a D result.]
- **§6.3 Trees.** The LOS table's levels do not list Trees. [Ground level;
  Trees block only as an intervening feature, more than two of them.]
- **§6.23 hexside modifier of a ray along a hexside.** [A ray running exactly
  along a hexside has two candidate paths; the more protective entry
  hexside counts, and in a combined attack the most protective over all
  firers.]
- **§5.21 Friendlies lost aboard.** A Friendlies unit lost with a sunk or
  mined gunboat is on neither bank. [Scored as east bank, 1 VP (§9.14).]
- **§5.21 aboard.** [A loaded unit neither fires nor is fired at on its own,
  and boards only from the east bank.]
- **§5.53 Dervish artillery.** [The guns wear the Khalifa's colour: only he
  stacks with them.]
- **§8.2 deserters.** [Out of play for good: listed apart from the
  casualties, scoring nothing, never re-entering as reinforcements.]
- **§9.35 GORDON on turn 7 or 8.** The ladder names turns 4-6 only. [Killed on
  turn 7: British marginal; on turn 8: British tactical.]
- **§7.5 retreat.** [The two-hex retreat ignores movement costs and zones of
  control; a mixed infantry-and-mounted attack permits it; each unit of a
  stack retreats on its own.]
- **§7.6 which four advance.** More than four eligible Dervish attackers:
  [the first four in declaration order, leaders free; a unit the stacking or
  walled-city rules refuse is skipped.]
- **§5.23 Khartoum.** The walled-city unit restrictions name Omdurman only.
  [In FALL OF KHARTOUM only the wall hexsides bind; any unit may enter.]
- **§6.42 in FALL OF KHARTOUM.** [The Maxim and howitzer subphase is skipped
  entirely: the order of battle has no Maxims and no named gunboats.]
- **§10.23 "one complete turn".** [Standing beside a chained hex from the
  start of the Anglo-Egyptian player turn to its end, without spending
  movement points.]
- **§6.64 off-map scatter.** [A shell scattered off the map is lost: nothing
  resolves, the gunboat has fired.]
- **§9.345 at night.** The crossing costs six upstream points; on the FALL OF
  KHARTOUM night turn 1 an old gunboat's halved upstream allowance is five.
  [Unaffordable that turn -- a consequence of §8.1, not a bug.]

## UI

- A gunboat that may not move for a rules reason other than its allowance
  (the §5.21 loading turn, a chain or mine stop) gets a route slip that
  blames the §5.24 upstream allowance ("this move has 0 MP left"): the
  engine's `remaining_movement` folds every precondition into 0 and
  `no_route_reason` only knows budgets (`picker/clicks.rs`). Say which rule
  holds the boat. The hover tooltip likewise prints "0 left" beside
  "11↑ 17↓ MP left" for such a boat.
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

## Verification

- Weak clause witnesses (`docs/traceability.toml`): the named test asserts
  only part of its section's clause. Write the missing test, then make it the
  witness.
  - §2.32: nothing checks that Maxims fire on the Maxims line and artillery
    and old gunboats on the Artillery line.
  - §6.11: nothing checks that each counter carries its printed fire factor.
  - §6.41: nothing asserts allocate-everything-then-resolve (the engine does
    not enforce it; see the approximation).
  - §6.42: no test shows a named gunboat firing howitzer fire.
  - §9.33: nothing pins the early end when GORDON falls.
  - §9.344: nothing checks that the North Fort fort may fire its guns.
  - §9.345: nothing checks that six upstream movement points are debited.

- Mutation-gate debt: mutants the citing sections' tests miss, from a pilot
  over the sections the 2026-09 audit fixed (7 of its 20 functions finished
  before the run was stopped for memory, plus §5.51). The gate runs on the
  changed lines of a change, so these block only when that code is edited;
  `cargo run -p traceability-lsp --bin mutation-gate` without `--in-diff`
  lists them all (~500 mutants, a few hours).
  - §6.64 `apply_howitzer_fire`: 5 of 7 missed (the phase, firer and target
    guards).
  - §7.3/§7.6 `apply_resolve_melee`: 2 of 5 (the attacker re-check).
  - §10.12-14 `apply_river_mine`: 1 of 5.
  - §5.51 `check_stacking`: 1 of 4; `stacking_rule`: 8 of 26 (a lone
    gunboat, the group-purity guard, the leader-colour arms).
- The expensive Kani tier (`omdurman-rules/src/effects/expensive.rs`) has not
  been run to completion: each harness takes longer than 20 minutes on a
  desktop. Run it on a big machine (`KANI_EXPENSIVE=1 ./run-kani.sh`) and
  read its `cover!` results; its randomized `cargo test` twin passes.
- The expensive tier runs on the empty board, where the accepted halves of
  `ConstructZariba`, `ArtilleryBreachWall`, `Demolition` and
  `FriendliesTransport` are unreachable. A small real board would need a
  cheap proof-build hasher for `BoardInfo`'s maps (a symbolic hex hashed
  with SipHash on every lookup is prohibitive).
- 22 Kani harnesses have no `#[rulebook]` annotation and are not tracked in
  `traceability.toml` (e.g. `distance_is_symmetric`, `game_over_is_absorbing`,
  `sink_chain_is_atomic`, `place_mine_is_atomic`).

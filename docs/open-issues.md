# Open issues

The single list of known open work. Collected on 2026-09-27 from the docs
audit: the retired audit, plan and play-test documents, plus
`review-2026-09.md` and `playtest-2026-09-fok.md`. Those two documents stay as
dated records, and their open items live here. Items cite symbols rather than
line numbers; re-check an item against the code before acting on it. Delete an
item when it is fixed. Last reviewed against the code on 2026-10-11, after the
deep-review fixes (LOS geometry, board data, net chunking, review mode, AI
seat handling).

## Rules engine: deviations from the manual

- **§10.11 / §10.21 secrecy.** Mines and the chain are "secretly recorded".
  The board and the event viewer hide them from the Anglo-Egyptian seat
  until a gunboat strikes them, and artillery may fire only at a chained hex
  a gunboat ran onto; but the event log still carries them in clear (an
  honour-system scratchpad; commit-reveal is post-1.0). See also "private
  dice" below.

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
  only once per fire *subphase* (§6.42: Maxims "may fire at enemy units
  fired at in Direct Fire Subphase"). Shells and Maxims at one hex are one combined
  attack (§6.42).]
- **§6.41 allocation per side.** [The allocation belongs to the firing
  side: teammates' orders join one allocation, and the first resolution by
  any of them closes it for all. Fire resolved without an allocation is an
  allocation of that one shot. The chain, like a hex, is fired at once per
  subphase.]
- **§6.42 combined howitzer and Maxim fire.** "Howitzer fire may be
  combined with Maxim fire, but only if the howitzer fire impacts in the
  intended hex." [One attack carries the shells and the Maxims; every shell
  rolls its own impact and Combat Results Table dice (§6.64). The shells on
  target and the Maxims resolve as one attack on one roll, without the
  §6.24 +1 (it is howitzer fire); each scattered shell resolves alone where
  it lands; with no shell on target the Maxims fire alone at the intended
  hex, as Maxim fire with the +1.]
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
- **§7.7 owner chooses losses.** The manual does not say who picks the
  units a numbered result eliminates. [Their owner, as the wargame
  convention has it: where a result runs out inside a tier of units that
  differ, the combat waits for the owner's `TakeLosses` (the meleeing units
  fall first, §7.7). The same for the one unit a destroyed fort or a breached
  wall takes (§6.62, §6.63, §6.53). A demolition's victim is chosen after
  the turn ends.]
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
- **§6.3 rays through a hex corner.** Notes d/e cover rays along a hexside
  only. [A ray through a corner is treated alike: the hexes it only touches
  block only when both sides of the tie block (`los_ray_sides`); ruling
  2026-10-11.]
- **§6.3 positional details are not reciprocal.** "The third hut" and the
  like count from the firer, so a Ground firer at a Rough target and the
  Rough unit firing back can disagree (e.g. Huts (1,3) vs (1,4)). [Read as
  printed: each direction counts from its own firer.]
- **§6.63 breach casualty.** "One enemy unit is eliminated". [Any enemy unit
  beside the wall hexside except Anglo-Egyptian leaders, a player's fort
  counter included: no printed fort borders a city wall, so the wall is
  never the fort's own (ruling 2026-10-11). Open: a gunboat on a Nile hex
  beside a wall hexside is also a candidate.] A demolition of a wall the
  artillery already breached that turn takes no second unit.
- **§7.7 excess melee losses.** The manual does not say where losses beyond
  the meleeing units go. [They fall on hex-mates, never on the attackers'
  own fort or gunboats (§6.54, §7.1); losses no unit can take lapse.]
- **§9.113 blocked leader entry.** The turn-4 deadline for the A-E leaders
  says nothing of Dervish units holding every entrance hex. [The deadline
  applies only while a missing leader has a legal entry hex; otherwise the
  leader enters on a later turn (recorded as an approximation).]
- **§7.5 declining the retreat.** The manual's retreat is optional and
  silent on timing. [The defender retreats or explicitly stands fast
  (`DeclineRetreatBeforeMelee`) before the attacker resolves; an AI attacker
  waits for that choice.]

## UI

- Deployment traps get no warning (e.g. the Hogali pocket in Fall of Khartoum).
- The Mahdi's Tomb has a VP-card line naming its holder but no board marker.
- The chart sheet (`charts.rs`) has no LOS table or Scattergram tab.
- There are no Nile flow arrows (`flow_at`).
- River mine and chain placement (`river_placement.rs`) shows no legal band;
  the engine refuses hexes outside it.
- Campaign Dervish set-up has no per-class groups, placed/total counters or
  per-class zone tints (§9.111).
- Historical set-up shows the union of the leaders' zones, not the per-leader
  three-hex halo of the counter in hand (`historical_set_up_area`); the
  hover tooltip names why a hex is refused.
- The desertion panel hides exempt units instead of greying them out, and has
  no "no VP for deserters" note (§8.2).
- The LOS overlay has no ray between two chosen hexes.
- Every fire target gets the same red ring (`fire_target_overlay_mesh`): there
  is no colouring by range band (×3/×2/×1/×½).
- Melee target rings don't mark wall hexsides as blocked, or gates and breaches
  as open (§7.2).
- Artillery-only targets (§6.61–6.63) get a refusal card on hover, but no
  board marker before it.
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
- Player names and colours are re-randomised on every launch
  (`LocalPlayerSettings` is not persisted).
- Unconfirmed, from the play-test and review notes:
  - Fire slips pile up.
  - The lobby shows "(claimed by a player)" twice.
  - Player colours can clash.
  - Turn and phase are shown in five places (review 4.11).
  - Surfaces overlap (review 4.13).

## Net and app plumbing

- `net_socket.rs` is ~2150 lines (`handle_socket` itself is ~200 after the
  helper split); an optional `GameSession` struct.
- `GameMap` (hexmap) and `BoardInfo` (rules) duplicate board topology.
- Private dice: every roll is visible in the event log.
- Snapshot-request retry never gives up (every 2 s, then every 15 s once
  the bootstrap budget has passed).
- Broadcast retention is all-or-nothing.
- 23 systems still take `Option<Res<GameStateResource>>`.
- Duplicated web tab (`seats.rs`): the localStorage probe is compile-checked
  only, never run in a browser. Two tabs that start in the same instant with
  the same key don't answer each other's probe; only the `PlayerInfo`
  fallback covers that, and only before a game starts.
- A newcomer that wins the lobby election replaces the old host's scenario,
  optional-rule and AI picks with its own defaults (every view agrees, but
  the earlier picks are lost).

## Board data

- The road network inside Omdurman has not been audited against the printed
  map (e.g. near (31,42)); the 2026-10 audit covered terrain types and crest
  hexsides only.

## Verification

- Weak clause witnesses (`docs/traceability.toml`): the named test asserts
  only part of its section's clause. Write the missing test, then make it the
  witness.
  - §2.32: nothing checks that Maxims fire on the Maxims line and artillery
    and old gunboats on the Artillery line.
  - §6.11: nothing checks that each counter carries its printed fire factor.
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
- Kani has not been run since the 2026-10-11 fixes. New expensive-tier
  harnesses (`advance_phase_from_is_safe`, `roll_desertion_is_safe`,
  `decline_retreat_before_melee_is_safe`) have passed only their randomized
  `cargo test` twins, and the pending-mine queue edits to the proofs in
  `effects/river.rs` are unverified.
- 26 Kani harnesses have no `#[rulebook]` annotation and are not tracked in
  `traceability.toml` (e.g. `distance_is_symmetric`, `game_over_is_absorbing`,
  `sink_chain_is_atomic`, `place_mine_is_atomic`).

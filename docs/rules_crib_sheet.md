# Rules Crib Sheet — Remember Gordon! (The Battle of Omdurman)

Curated summary of the rulebook (Phoenix Enterprises, 1982) for the offline
rules auditor. Section numbers match the manual's `§N` citations used in the
codebase. Where a value is uncertain, the crib sheet says so; the auditor
should raise a Warning rather than guess. Terrain values come from the
Terrain Effects Chart (`Boardgame - Remember_Gordon/tables/terrain_effects_chart.ron`),
which the manual text does not reproduce.

## Turn & phase sequence (§4, §6.4)

- A **game turn** = two player-turns. Campaign: 22 turns (Sept 1 6:00 am →
  Sept 3 8:00 am, §9.12). Historical: 4 turns (Sept 2 6:00 am → noon, §9.22).
  Fall of Khartoum: variable, at most 8 turns (§9.33, §9.35).
- Each player-turn runs: **Movement → Defensive Fire (the non-moving
  player) → Offensive Fire (the moving player) → Melee**.
- Each fire phase has a **Direct Fire** subphase (§6.41, both sides). The
  Anglo-Egyptian player also runs the **Maxim Second Fire and Howitzer**
  subphase (§6.42) after his direct fire, whether he is firing *offensively*
  (his own turn) or *defensively* (the Dervish turn). The Dervish never get
  §6.42. In Fall of Khartoum the engine skips §6.42: that order of battle has
  no Maxims or named gunboats.
- In every fire subphase, all attacks are allocated first, then resolved in
  any order (§6.41).
- First mover: Campaign, the Anglo-Egyptian (§9.113). Historical and Fall of
  Khartoum, the Dervish (§9.212, §9.322).
- Set-up is sequential: the side that sets up first finishes (and confirms)
  before the other starts. Campaign: the Dervish set up first (§9.111).
  Historical: the Anglo-Egyptian (§9.211). Fall of Khartoum: the British
  (§9.321).

## Movement (§5)

- Movement points to enter a hex: Clear 1, Rough 3, Trees 1, Swamp 3,
  Hilltop 1, Huts 3, Building 3. Moving along a road (between two hexes the
  road connects) costs 1 whatever the other terrain.
- Hexside surcharges: Khor +5; Crest +1; a City Wall is impassable except at
  a Gate or Breach (+1). Zariba hexsides: see §9.23.
- Units move up to their printed allowance (§5.11). Unused MP are lost, never
  carried over or transferred (§5.13). Friendly units may be moved through at
  no extra cost (§5.51).
- Land units never enter a Nile hex (§5.22), except in the "Friendlies"
  transport (§5.21).
- **Walled city of Omdurman** (§5.23): the only Dervish units that may enter
  are the Khalifa, the three Dervish artillery units and the Taiasha. Any
  Anglo-Egyptian unit may enter except gunboats and "Friendlies". Entry and
  exit only through a gate or breach hexside.
- Gunboats (§5.24): separate upstream/downstream allowances; moving even one
  hex upstream caps the whole turn at the upstream allowance.
- Dervish forts never move (§5.25).
- **Zones of control** (§5.4):
  - Every unit except Anglo-Egyptian leaders exerts a ZOC into its six
    adjacent hexes. Gunboats exert one only against enemy gunboats. Disrupted
    units have none (§5.41).
  - Entering or leaving a ZOC costs nothing (§5.42). A unit must stop on
    entering an enemy ZOC. Next movement phase it may withdraw or move
    directly into another enemy ZOC (§5.26, §5.43).
  - A ZOC never extends into or out of a Nile hex (except gunboat vs gunboat),
    across a khor, into a fort, or into a walled-city hex across a wall
    (§5.44). It does extend:
    - out of a fort (even an unoccupied one);
    - out of a walled-city hex across a wall;
    - out of (not into) the walled city across a gate;
    - both ways across a breach;
    - out of (not into) a hut or building hex;
    - out of (not into) the Zariba.
- **Stacking** (§5.5):
  - At most four units per hex. Leaders stack free, on top of the four
    (§5.51).
  - Gunboats may not stack with *any* other unit, except during the §5.21
    transport (§5.51).
  - The limit applies at the end of movement and during combat (§5.51).
  - Units of different Dervish tribes may never stack together (§5.52).
  - A Dervish leader may stack only with units of his command, i.e. his
    color (§5.53).
- **Royal Engineers** (§6.53): end movement adjacent to a fort or wall
  hexside. They may not fire offensively or melee attack that turn. If still
  adjacent and undisrupted at the end of the Anglo-Egyptian player turn, the
  fort is destroyed or the wall breached (effects as §6.62/§6.63).

## Fire combat (§6)

- Fire is voluntary (§6.12). A unit's factor is unitary and may not be split
  between hexes (§6.13).
- Any units that may legally fire at a hex may combine into one attack
  (§6.14). A stack may split to fire at different hexes (§6.15).
- In one fire phase a unit fires once and a unit is fired at once
  (exceptions: Maxims and gunboats, §6.4).
- Procedure (§6.2):
  1. **Line of sight** (§6.21, §6.3). Howitzer fire ignores it.
  2. **Range Effects Table** (§6.22): each unit's factor is tripled, doubled,
     normal, halved, or out of range. Each side has its own table.
     "Friendlies" fire on the Dervish table (§6.52). In Fall of Khartoum both
     sides use the Dervish table (§9.343).
  3. **Sum the factors.** Halving rounds down per unit, never below 1 (§6.16).
  4. **Modifiers.**
     - Target-hex terrain (§6.23): Huts −1, Building −3, fort −3 (§6.54).
     - Hexside the fire crosses into the target: Crest −1, City Wall −4.
     - Historical Zariba (§9.23): thorn hedge −2, trench −4 vs entrenched.
     - Anglo-Egyptian direct fire +1 (§6.24).
     - Brigade integrity +1 (§5.54, §6.24): all four battalions of one
       Anglo-Egyptian infantry brigade stacked together and all firing at the
       same hex.
     - Modifiers are cumulative. A modified roll below 1 counts as 1, above
       10 as 10.
  5. **Roll d10** and cross-index the modified roll with the factor column of
     the Combat Results Table.
- Combat results: `—` no effect. `D`: half the units in the target hex
  (round up) are disrupted. `1`–`5`: that many units in the target hex are
  eliminated; the survivors are not disrupted.
- Disrupted units have no ZOC and may not move, fire or melee. They recover
  at the end of the owning player's turn.
- Maxims fire twice: once in §6.41 and again in §6.42. A Maxim that skipped
  §6.41 still fires only once in §6.42 (§6.42).
- **Howitzers** (§6.64): only the five named British gunboats.
  - They fire their artillery factor as direct fire in §6.41, then again as
    howitzer fire in §6.42.
  - Target any hex 4–10 away, ignoring LOS. Roll twice: the CRT roll, then
    the impact roll.
  - The target hex is hit on an impact roll of 7–10; otherwise the shot
    scatters per the scattergram. The result applies even to friendly units.
  - Howitzer fire may combine with Maxim fire only if it impacts in the
    intended hex (§6.42).
  - No howitzer fire at night (§8.1).
- **Artillery specials** (§6.6): only artillery may fire at gunboats, forts
  or walls.
  - A result of 3+ sinks a gunboat (§6.61).
  - 2+ destroys a fort, and one enemy unit inside dies with it (§6.62).
  - 2+ breaches a wall hexside: it no longer blocks LOS, and one adjacent
    enemy unit is eliminated (§6.63).
- **Leaders** (§6.51):
  - Dervish leaders have fire, melee and movement factors and fight like any
    combat unit.
  - Anglo-Egyptian leaders have movement only. They are eliminated when alone
    in a hex a Dervish unit enters or passes through, or when every combat
    unit they are stacked with is eliminated.
- **Forts** (§6.54):
  - The owner may fire a fort's artillery factor even if the fort is
    unoccupied.
  - Forts melee defend only.
  - Nobody may occupy an enemy fort or advance into an empty one.
  - Entering or leaving a friendly fort costs nothing extra.
- **Defensive fire** never allows advance after combat (§6.7).
- **Advance after offensive fire** (§6.82): into a hex the fire emptied, by
  units that took part in the attack and were adjacent.
  - Artillery never advances.
  - No advancing across a wall (except at a gate or breach), across a khor,
    or across a thorn hedge (§9.231).

## Melee (§7)

- Only adjacent units may melee (§7.2). Not across a wall hexside (a gate or
  breach is fine), a khor, or a thorn hedge (§9.231).
- Gunboats neither attack nor are attacked (§7.1).
- Melee is simultaneous: units eliminated by the attack still roll (§7.3).
- Attackers: infantry, cavalry, camel units and Dervish leaders (§7.4). Every
  unit except gunboats may defend; forts defend only (§6.54).
- Both sides roll d10 on the CRT with their melee modifier: Dervish +2,
  Anglo-Egyptian +1 (§7.7). "Friendlies" use the Dervish +2 (§6.52).
- No terrain modifiers apply to melee, except that a Dervish attack on an
  entrenched unit gets −2 instead of +2 (§9.232).
- Losses come from the meleeing units first (§7.7).
- **Retreat** (§7.5): cavalry and camel units may retreat two hexes from an
  *infantry* melee attack, at most once per unit per turn. Enemy units whose
  melee is not yet resolved may then attack them in their new hex.
- **Advance** (§7.6): if a melee empties the defender's hex, every surviving
  eligible Dervish attacker (adjacent, took part) MUST advance, up to the
  stacking limit. The Anglo-Egyptian player may advance.

## Zariba (§5.3, §9.23)

- Built and in place in the Historical scenario only. In the Campaign the
  hexsides count as clear unless constructed (§5.3).
- **Campaign construction** (§5.3): any Anglo-Egyptian infantry unit that
  begins and ends the Anglo-Egyptian player turn adjacent to (and on the Nile
  side of) Zariba hexsides builds all of them it is adjacent to. It may not
  fire offensively or melee attack that turn.
- **Thorn hedge** (§9.231): −2 to Dervish fire. No melee and no advance
  across it, in either direction.
- **Trench** (§9.232): −4 to Dervish fire against entrenched units. Dervish
  melee against an entrenched unit is −2 instead of +2. Entrenched units do
  not block LOS. "Entrenched" = adjacent to, and on the Nile side of, a
  trench hexside.
- Units enter and leave the Zariba only through the two end hexsides at the
  Nile, paying +2 MP (exception: advance after combat across an entrenched
  hexside) (§9.233).

## Night and desertion (§8)

- **Night** (§8.1):
  - Anglo-Egyptian movement allowances are halved (round down).
  - No howitzer fire.
  - All fire ranges are halved for both sides, rounded down, but range 1
    stays 1. Range *effects* are otherwise as by day.
  - Fall of Khartoum turn 1 is always a night turn (§9.341).
- **Dervish desertion** (§8.2): once per Campaign, in the Dervish movement
  phase of the first night turn, roll d10.
  - 1½ × the roll Dervish units desert (the engine rounds down); the Dervish
    player chooses which.
  - The Khalifa, gunboats, artillery and forts may not desert.
  - No VP for deserters.

## Campaign set-up and reinforcements (§9.11)

- **Dervish set-up** (§9.111):
  - Isa Zachneih: east bank, in or south of El Debeba.
  - The Khalifa: either palace hex of the walled city.
  - The 3 artillery and all Taiasha: in the walled city.
  - 17 forts: south of the Khor Shambat on the west bank and/or south of the
    Halfaya huts on the east bank and the Nile islands.
  - 2 gunboats: any south-edge Nile hexes.
- **Dervish reinforcements** (§9.112): enter on the west edge south of the
  Khor Shambat, each unit paying the terrain cost of its entry hex.
  - Turn 1: Baggara, Jaalin, Danagla, Kehena and Degheim, with Yakub, Sherif
    and Ali Wad Helu.
  - Turn 2: Hadendowa with Osman Digna.
  - Turn 3: Mulazmin and Jehadia with Sheik El Din.
- **Anglo-Egyptian** (§9.113): no units on the map at start; GORDON is not
  used.
  - Entry:
    - Gunboats enter through any north-edge Nile hex, paying 1 MP for the
      first hex.
    - "Friendlies" enter through the Abu Alim hut hex, paying 8 MP per unit.
    - Everything else enters through the west-bank Anglo-Egyptian Entrance
      Area, paying 1 MP to enter the map.
  - Schedule:
    - Turn 1: any three gunboats, the "Friendlies", the Egyptian Cavalry,
      the Horse Artillery, and two Egyptian Division infantry brigades.
    - Turns 2 and 3: any three gunboats plus any twelve land units.
    - Turn 4: everything remaining.
  - Kitchener, Gatacre and Hunter may enter at any time in turns 1–4 and do
    not count against the twelve. All three must be in play by the end of
    turn 4.

## "Friendlies" transport (§5.21)

- Allowed only after the Isa Zachneih unit has been eliminated.
- **Load**: a "Friendlies" unit and a gunboat start a turn adjacent, and the
  unit loads onto (stacks with) the gunboat.
- **Cross**: next Anglo-Egyptian turn, the gunboat moves to any Nile hex
  adjacent to a west-bank hex.
- **Disembark**: the third turn, the unit disembarks and moves normally,
  paying the terrain cost of its first hex.
- "Friendlies" may never enter the walled city (§5.23, §6.52).

## Optional rules (§10, Campaign only)

- **River mines** (§10.1):
  - Two Nile hexes, not the same hex, south of the E–W hexrow where the Khor
    Shambat meets the Nile, secretly recorded before play (§10.11).
  - A British gunboat entering a mined hex must stop and roll (§10.12):
    - 1–4: no effect.
    - 5–7: engines lost. The gunboat drifts two hexes per turn with the
      current for the rest of the game; its guns still fire.
    - 8–10: sunk.
  - Once both mines have been rolled for, none remain (§10.13). Dervish
    gunboats pass mined hexes freely (§10.14).
- **River chain** (§10.2):
  - A line of at most four river hexes, south of the same hexrow, secretly
    recorded (§10.21).
  - A British gunboat entering a chained hex must stop (§10.22).
  - No gunboat of either side may cross until the British sink the chain
    (§10.23), either with an infantry or cavalry unit spending one complete
    turn adjacent to a chained hex on either bank, or with artillery scoring
    3+ on the CRT.
- The rulebook advises against using both options in one game (§10).

## Victory (§9.14, §9.24, §9.35)

- **Campaign** (§9.14):
  - **Mahdi's Tomb**: 25 VP to whoever controls it at the conclusion of play.
    The Dervish control it from the start. The Anglo-Egyptian player takes it
    only if, at the end, the Tomb hex holds a British leader plus a
    non-"Friendlies" Anglo-Egyptian combat unit, both undisrupted.
  - **Dervish receive**:
    - 10 per British leader eliminated.
    - 10 per British gunboat sunk.
    - 3 per Anglo-Egyptian land unit eliminated.
    - Per "Friendlies" unit eliminated: 3 on the west bank, 1 on the east
      bank.
  - **Anglo-Egyptian receive**:
    - 10 for the Khalifa.
    - 1 for Isa Zachneih.
    - 1 per other Dervish unit eliminated, gunboats, artillery and leaders
      included.
    - 0 for forts.
  - Levels by net superiority:
    - Anglo-Egyptian: decisive 50+, tactical 30–49, marginal 15–29, draw
      1–14.
    - Dervish: decisive 30+, tactical 20–29, marginal 10–19, draw 1–9.
  - Alternatively:
    - An Anglo-Egyptian decisive victory if every Dervish unit (gunboats and
      forts included) is eliminated.
    - A Dervish decisive victory if every Anglo-Egyptian unit on the west
      bank (gunboats excluded) is eliminated.
- **Historical** (§9.24): each side's level comes from the number of enemy
  units it eliminated. The lower level is subtracted from the higher to give
  the net result.

  | Level | Anglo-Egyptian (Dervish eliminated) | Dervish (Anglo-Egyptian eliminated) |
  |---|---|---|
  | 5 Decisive | 100+ | 30+ |
  | 4 Strategic | 60–99 | 15–29 |
  | 3 Tactical | 45–59 | 10–14 |
  | 2 Marginal | 30–44 | 5–9 |
  | 1 Draw | 0–29 | 0–4 |

- **Fall of Khartoum** (§9.35): the level is set by the turn GORDON dies.
  - Dervish: decisive on turn 4 or earlier, tactical on turn 5, marginal on
    turn 6.
  - British: marginal if he survives turn 6, tactical if he survives turn 7,
    decisive if he survives turn 8.
  - The Dervish then drop one level for 16–23 own units lost, two for 24–31,
    and three for 32+. "Friendlies" losses are Anglo-Egyptian losses, not
    Dervish.

## Historical set-up (§9.21)

- **Anglo-Egyptian first** (§9.211):
  - No GORDON and no "Friendlies".
  - Gunboats in Nile hexes adjacent to the Zariba.
  - The Camel Corps, Egyptian Cavalry and Horse Artillery in the Kerreri hut
    hexes.
  - Everything else in the 13 Zariba hexes.
- **Dervish second** (§9.212):
  - No Isa Zachneih, gunboats or forts.
  - Every unit out of LOS of every Anglo-Egyptian unit.
  - Leaders on their lettered hexes: A Ali Wad Helu, D Sheik El Din,
    Y Yakub, K Khalifa, S Sherif, O Osman Digna.
  - Every other unit within three hexes of its leader of the same color.

## Fall of Khartoum (§9.3)

- The small Fall of Khartoum map only (§9.31). Every hex is playable,
  including half hexes (§9.342).
- **British set-up first** (§9.321):
  - GORDON in the palace (the engine auto-places him).
  - Two old-style (unnamed) gunboats in any Nile hexes.
  - In building or hut hexes of Khartoum, Forts Makran and/or Buri, and/or
    adjacent to any wall hex: 1 Egyptian artillery, 2 British infantry,
    3 Egyptian infantry, 4 Sudanese infantry, and 4 "Friendlies".
- **Dervish move first** (§9.322), entering on turn 1 through any south- or
  east-edge hex: 32 Mulazmin, 2 Hadendowa, 6 Kehena, 5 Degheim and 3 Dervish
  artillery. They have no leaders and no gunboats.
- The Dervish control the North Fort and may fire its guns (§9.344); the
  engine auto-places it.
- Both sides use the Dervish Range Effects Table (§9.343).
- British gunboats may pass between the White and Blue Nile for 6 upstream
  MP, off-board (§9.345).
- GORDON never moves (§9.346). He dies only when a Dervish unit passes
  through or occupies the palace hex, by movement or advance after combat,
  never by fire. His death ends the game.

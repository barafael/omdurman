# Rules Crib Sheet — Remember Gordon! (The Battle of Omdurman)

Curated summary of the rulebook (Phoenix Enterprises, 1982) for the offline
rules auditor. Section numbers match the manual's `§N` citations used in the
codebase. Where a value is uncertain, the crib sheet says so; the auditor
should raise a Warning rather than guess.

## Turn & phase sequence (§4)

- A **game turn** = both players' player-turns. Campaign: 22 turns (Sept 1
  6:00 am → Sept 3 8:00 am). Historical: 4 turns. Fall of Khartoum: variable.
- Each player-turn runs: **Movement → Offensive Fire → Defensive Fire →
  Melee**.
- Offensive fire has two sub-phases: **Direct fire** (both sides) then
  **Maxim second fire + howitzer** (Anglo-Egyptian only).
- Campaign: Anglo-Egyptian moves first. Historical & Fall of Khartoum:
  Dervish moves first.
- **Night turns** (§8.1): halve all Anglo-Egyptian movement, halve all fire
  ranges (min 1), no howitzer fire.

## Movement (§5)

- Terrain movement costs (per hex entered): Clear 1, Rough 1, Trees 2, Swamp
  2, Hilltop 1, Huts 2, Building 2, Nile impassable to land units. Roads
  reduce cost to 1 for Clear/Rough/Trees/Hilltop.
- Movement points do not carry over between turns (§5.13).
- Stacking (§5.51–5.54): max four units per hex; gunboats may not stack with
  non-gunboats; Dervish units of different tribes may not stack together;
  Dervish leaders may only stack with units of their command; Anglo-Egyptian
  brigade integrity requires all four battalions of one brigade in one hex.
- **Zones of control** (§5.41–5.44): non-disrupted units project ZOC into the
  six adjacent hexes (gunboats only vs gunboats; leaders project none). ZOC
  does not extend across a khor, into a fort, or into the walled city across a
  wall; it extends both ways across a breach.
- **Royal Engineers** demolition (§6.53): spends the whole turn adjacent to
  the target (no offensive fire/melee); resolved at end of turn — destroyed
  only if still adjacent and undisrupted.
- Gunboats (§5.24): split upstream/downstream allowances; moving upstream once
  caps the rest of the turn at the upstream allowance.

## Fire combat (§6)

- Fire factors are halved per unit by range band (§6.22): 1–5 factors row,
  6–10, 11–15, 16–20, 21–25, 26–30, 31–35, 36–40, 41+.
- Modifiers (§6.24): +1 Anglo-Egyptian direct fire, +1 brigade integrity,
  terrain modifier of the defender's hex (§6.23), −2 thorn hedge (§9.231),
  −4 entrenched trench (§9.232).
- Roll d10 + modifiers → Combat Results Table: No Effect / Disrupt / Eliminate.
- Co-stacked firers fire together (§6.14). Maxims fire twice per turn (§6.42).
- Howitzers (§6.64): range 4–10, ignores LOS, hit on impact roll 7–10 else
  scatter. No howitzer fire at night.
- Artillery may breach a **Wall** hexside (§6.63): CRT Eliminate(2) or higher
  flips the wall to a breach and eliminates one adjacent enemy unit.

## Melee (§7)

- Simultaneous: both sides roll d10 + modifier (Dervish +2, Anglo-Egyptian
  +1; §7.7) on the CRT.
- Only infantry, cavalry, camel and Dervish leaders may attack; gunboats
  neither attack nor are attacked.
- Declared melee opens the defender's reaction window (§7.5): cavalry/camel
  defenders may retreat two hexes before resolution.
- Mandatory Dervish advance after melee into a vacated hex (§7.6).
- Advance-after-combat (§6.82): eligible adjacent attackers (not artillery)
  may advance into a hex emptied by combat; may not cross a wall, khor, or
  thorn hedge.

## Zariba (§9.231–9.233)

- Historical scenario: thorn-hedge (−2 to fire into) and trench (−4 to fire
  into) segments. Units may only enter/leave the Zariba via the two trench-end
  hexsides at the Nile, paying +2 MP. Dervish melee into a trenched hex is −2.

## Reinforcements & desertion (§8, §9)

- Dervish desertion (§8.2): once per campaign, first night turn; roll d10,
  floor(1.5×roll) units desert (Dervish chooses). Khalifa, gunboats,
  artillery, forts may not desert.
- Anglo-Egyptian reinforcements enter the west bank paying 1 MP per hex
  (§9.113).

## "Friendlies" transport (§5.21–5.23)

- Load (turn N, start adjacent) → Cross (turn N+1, gunboat moves to a Nile
  hex adjacent to a west-bank hex) → Disembark (turn N+2). Unlocks after the
  Isa Zachneih unit is eliminated. Friendlies may not enter the walled city.

## Optional rules (§10)

- **River mines** (§10.11–10.12): at most two, no shared hex. Gunboat enters a
  mined hex → roll: 1–4 no effect, 5–7 engines lost (drifts two hexes/turn
  with the current), 8–10 sunk.
- **River chain** (§10.21–10.23): up to four contiguous Nile hexes. Cleared by
  infantry/cavalry spending a full turn adjacent on either bank, or artillery
  scoring 3+ on the CRT.

## Victory points (§9.14)

- **Anglo-Egyptian**: Mahdi's Tomb control 25 VP; Khalifa 10; Isa Zachneih 1;
  1 per Dervish unit eliminated (forts 0).
- **Dervish**: 10 per British leader eliminated; 10 per gunboat sunk; 3 per
  Anglo-Egyptian land unit; 3 per Friendlies unit on the west bank, 1 on the
  east bank.
- Campaign levels: AE decisive 50+, tactical 30+, marginal 15+, draw 1–14;
  Dervish decisive 30+, tactical 20+, marginal 10+, draw ≤9.

## Fall of Khartoum deltas (§9.3)

- GORDON fixed at the Palace; North Fort fixed. Old gunboats only (no named).
- Dervish entry zone: south/east map edge only (§9.322).
- GORDON eliminated at the Palace ends the game (§9.346): Dervish base victory
  level set by the turn of his death; Dervish losses shift the level toward
  the British end (§9.35).
- Nile-mouth crossing (§9.345): gunboat may cross to the Blue Nile mouth for
  6 upstream MP.

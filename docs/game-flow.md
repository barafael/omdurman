# Game Flow — Remember Gordon! The Battle of Omdurman

## Top-Level Loop

Each **Game Turn** consists of two **Player Turns** (§4). The first-mover varies by scenario:
- **Campaign (§9.1):** Anglo-Egyptian first.
- **Historical (§9.2):** Dervish first (§9.211 — A-E sets up first but moves second).
- **Fall of Khartoum (§9.3):** Dervish first (§9.322 — British sets up first but moves second).

After both Player Turns complete, advance the Game Turn marker to the next hour (§4).

The diagram below shows the **Campaign** order (A-E first). For Historical and FoK, swap the two Player Turn blocks so Dervish comes first.

```
GAME TURN
  ├── Anglo-Egyptian Player Turn
  │     ├── 1. Movement Phase
  │     ├── 2. Fire Combat Phase
  │     │     ├── a. Dervish Defensive Fire
  │     │     └── b. Anglo-Egyptian Offensive Fire
  │     │           ├── 1. Direct Fire Subphase
  │     │           └── 2. Maxim Second Fire & Howitzer Subphase
  │     └── 3. Melee Phase
  ├── Dervish Player Turn
  │     ├── 1. Movement Phase
  │     ├── 2. Fire Combat Phase
  │     │     ├── a. Anglo-Egyptian Defensive Fire
  │     │     │     ├── 1. Direct Fire Subphase
  │     │     │     └── 2. Maxim Second Fire & Howitzer Subphase
  │     │     └── b. Dervish Offensive Fire (Direct Fire only)
  │     └── 3. Melee Phase
  └── Advance Game Turn marker
```

---

## Phase Details

### 0. Setup Phase (§9.11, §9.21, §9.32)

Before the first Game Turn, players place their units according to the scenario setup instructions. After both players are ready, the game begins with the first-mover's first Player Turn.

---

### 1. Movement Phase (both players)

- Select any/all units, move each up to its printed movement allowance (MA), paying terrain costs per hex (§5.11, §5.12).
- **Stops on enemy ZOC:** Unit must stop immediately upon entering an enemy ZOC. No MP cost to enter or leave ZOC. Next turn they may withdraw or move directly into another enemy ZOC (§5.26, §5.43, §5.42).
- **ZOC rules:** All units except A-E leaders exert ZOC into 6 adjacent hexes (§5.41). Disrupted units have no ZOC. Gunboats exert ZOC only against enemy gunboats. ZOCs do not extend into/out of Nile hexes, across a khor, into a fort, or into walled city across a wall hexside. ZOCs extend out of a fort, out of but not into walled city across a gate hexside, both ways across a breach hexside, out of but not into a hut/building hex, and out of but not into the Zariba (§5.44).
- **No MP accumulation.** Unused MP lost (§5.13).
- **Stacking:** Max 4 units per hex (leaders free; gunboats alone) (§5.51). Dervish different tribes may not stack together (§5.52). Enforced end of movement and during combat.
- **Nile is impassable** to land units — they may never enter a Nile River hex (§5.22).
- **Walled city entry:** Only the Khalifa and his bodyguard may enter Omdurman's walled city. A-E units may not enter. Gunboats may not enter. Not enforced for Fall of Khartoum (§5.23).
- **Gunboat movement:** Gunboats have separate upstream/downstream movement allowances printed on the counter (§5.24).
- **Dervish forts may not move** (§5.25).
- **Reinforcements:** Scheduled reinforcements arrive during the Movement Phase of the owning player's turn (§9.112, §9.113).

#### Anglo-Egyptian Only

- **Zariba construction (campaign only):** Any A-E infantry that begins AND ends this phase adjacent to a Zariba hexside (on Nile side) builds all Zariba hexsides it is adjacent to (§5.3). Place blank counter on constructing unit. That unit may not fire offensively or melee this turn.
- **Transport Friendlies (multi-turn):** (a) Turn 1: Friendlies + gunboat start adjacent, load onto gunboat. (b) Turn 2: gunboat moves to any Nile hex adjacent to west bank (≤ MA). (c) Turn 3: Friendlies disembark and move normally. Gunboat also moves normally (§5.21). Only after Isa Zachneih eliminated.
- **Friendlies Brigade (§6.52):** The Friendlies are a distinct A-E brigade (Egyptian allies). They may be transported by gunboat as above and have their own fire table values.

#### Dervish Only

- **Desertion Roll (campaign, once per game):** First night turn, Dervish rolls 1 die × 1.5 = number of units removed (§8.2). Khalifa, gunboats, artillery, forts exempt. No VP awarded.

---

### 2a. Defensive Fire (non-moving player fires at moving player) (§6.7)

- **Who fires:** All of the non-moving player's units that are in range and have LOS (howitzer fire ignores LOS).
- **Fire combat is always voluntary** — a player may choose not to fire (§6.12).
- **Fire factor is unitary** — a unit's full fire factor must be applied to a single target and may not be divided between different hexes (§6.13).
- **A stack may split fire** — units in the same hex may fire at different target hexes (§6.15).
- **Halving rounds down, minimum 1** (§6.16).
- **How to resolve each attack (§6.4):**
  1. Check Line of Sight (§6.21, §6.3).
  2. Consult Range Effects Table (firing player's own table: A-E or Dervish) for fire factor multiplier (×3, ×2, ×1, ×½, or out of range) (§6.22).
  3. Sum fire factors from all units targeting same hex (§6.14).
  4. Check Terrain Effects Chart for defender's terrain modifier (negative to attacker's die roll) (§6.23).
  5. Roll 1d10, apply modifiers, cross-index total fire factors on Combat Results Table (§CRT).
- **Modifiers:**
  - A-E direct fire: +1 accuracy bonus (§6.24).
  - A-E brigade integrity: extra +1 if all 4 battalions of same brigade stacked together and firing at same hex (§5.54, §6.24), cumulative with +1 accuracy.
  - Die rolls < 1 treated as 1; > 10 treated as 10 (§6.24).
- **CRT results:** `D` = half (round up) of units in target hex disrupted; `1`–`5` = that many eliminated; `—` = no effect (§CRT).
- **Disrupted:** No ZOC, cannot move, cannot fire (offensive or defensive), cannot melee. Flip face-up at end of owning player's turn.
- **No advance after combat** from defensive fire (§6.7).
- **Artillery special rules (§6.6):**
  - Only artillery fires at gunboats (need 3+ CRT for sink) (§6.61).
  - Only artillery fires at forts (need 2+ CRT; if fort had units, one eliminated with fort) (§6.62).
  - Only artillery breaches walls (need 2+ CRT; if enemy adjacent to breached hexside, one eliminated) (§6.63).
- **Forts:** Fire artillery factor even if unstacked (§6.54); −3 defensive modifier to enemy fire on units inside. ZOC even if unoccupied. Cannot be occupied.
- **Fire limits per phase:** Each unit may fire once per phase (Maxims twice — once in Direct Fire subphase, once in Maxim/Howitzer subphase). Each unit may be fired at once per phase (§6.14). All fire targeting the same hex is combined into a single attack, so a given hex can only be targeted once per subphase.
- **Leader units (§6.51):** British and Dervish leaders have zero fire factor — they may not participate in fire combat. A-E leaders may not melee attack (but Dervish leaders may). A-E leaders do not exert ZOC (§5.41).

#### Dervish Defensive Fire (A-E's turn) (§6.7)
- Standard defensive fire rules.

#### Anglo-Egyptian Defensive Fire (Dervish's turn) (§6.42)
- **Direct Fire Subphase:** Standard defensive fire (§6.41). A-E +1 accuracy applies (§6.24).
- **Maxim Second Fire & Howitzer Subphase:** A-E Maxims fire second time (§6.42). Named gunboats fire howitzers (LOS-free, range 4–10, scatter per scattergram, hit on impact 7–10, must apply even if scatters into friendlies, none at night) (§6.64). May combine Maxim + howitzer if howitzer hits intended hex.

---

### 2b. Offensive Fire (moving player fires) (§6.8)

#### Direct Fire Subphase (§6.41) — Both Players

- Moving player allocates all direct fire attacks (combining any units targeting same hex), then resolves in any order (§6.14).
- Same resolution procedure as defensive fire (§6.4).
- If a target hex is vacated: adjacent participating units may advance after combat (§6.82).
  - Artillery cannot advance.
  - Cannot advance across wall hexside (except gate or breach).
  - Cannot advance across a khor.

#### Maxim Second Fire & Howitzer Subphase (§6.42) — Anglo-Egyptian Only

- A-E Maxims that have not yet fired this phase may fire a second time. If they did not fire in Direct Fire, they still only fire once here.
- Named gunboats may fire their artillery factor as howitzer fire (§6.64).
- **Howitzer procedure (§6.42, §6.64):**
  1. Select target hex 4–10 hexes from gunboat (ignore LOS).
  2. Roll 1d10 for CRT result.
  3. Roll 1d10 for impact hex: 7–10 = target hex hit; 1–6 = scatter per Howitzer Fire Scattergram.
  4. Apply CRT result to units in actual impact hex (even if friendly).
  5. Not allowed during night turns (§8.1).
- Maxims and howitzers may be combined if howitzer hits intended hex.
- Units may re-target hexes already fired at in Direct Fire subphase.
- Advance after combat allowed per same rules as Direct Fire (§6.82).

#### Dervish Offensive Fire (§6.8)

- Direct Fire only (no Maxim or Howitzer subphase).
- Dervish uses Dervish Range Effects Table (§6.22).
- No +1 accuracy bonus.
- Dervish artillery fires on "artillery" line of Dervish RET.
- Jehadia, Danagla, Isa Zachneih fire on "rifles" line. All other Dervish units (spears/swords) have printed fire factors (§2.31).
- Fort artillery may fire (§6.54).

---

### 3. Melee Phase (§7) (both players)

- **Who can melee attack:** Infantry, cavalry, camel units, Dervish leaders (§7.4).
- **Who can melee defend:** All units except gunboats.
- **Requirements:** Must be adjacent to target. Cannot melee across a wall hexside; may through gate or breach hexside (§7.2).
- **Each unit may melee once per Melee Phase.**
- **All melee is simultaneous:** Units eliminated by melee still get their melee combat die roll (§7.3).
- **Resolution (§7.7):**
  - Both attacker and defender roll on CRT with melee modifiers (no terrain modifiers except Zariba).
  - Dervish: +2 to die roll.
  - Anglo-Egyptian: +1 to die roll.
  - Losses must be taken from meleeing units first.
- **Cavalry/camel retreat:** May retreat 2 hexes from an infantry melee attack (§7.5). Only one retreat per unit per turn. If retreat places them adjacent to unresolved melee attackers, those may attack the retreating unit.
- **Advance after melee (§7.6):**
  - Dervish MUST advance into vacated hex (adjacent, surviving, participated in attack, up to stacking limit).
  - Anglo-Egyptian MAY advance.
  - Only attacking units may advance.
- **Forts (§6.54):** Defensive melee value only (−3 to enemy die roll). Cannot melee attack. Cannot occupy enemy fort. No advance into unoccupied enemy fort.
- **Royal Engineers demolition (§6.53):** If RE is adjacent to fort or wall hexside and does not fire/melee this turn, and is undisrupted at end of A-E player turn: fort is destroyed or breach marker placed. If enemy adjacent to wall hexside at breach, one eliminated.

---

## Night Turn Modifications (§8)

Applied on specified night turns (§8.1):
- A-E movement allowances halved (round down).
- No A-E howitzer fire.
- All fire ranges halved for both sides (round down; range 1 stays 1).
- Range effects on fire combat unchanged (same multipliers as day, just reduced ranges).

---

## Scenarios

| Scenario | Turns | First Mover | Notes |
|---|---|---|---|
| Campaign (§9.1) | 22 (6am Sep 1 – 8am Sep 3) | Anglo-Egyptian | Dervish sets up first (§9.111). A-E enters as reinforcements (§9.112). |
| Historical (§9.2) | 4 (6am–12pm Sep 2) | Dervish | Zariba already built (§9.23). A-E sets up first (§9.211). |
| Fall of Khartoum (§9.3) | Variable | Dervish | Small map (§9.31). Turn 1 always night (§9.341). Gordon in palace (§9.346). Both players use Dervish Range Effects Table (§9.343). Dervish controls North Fort (§9.344). Gunboat White Nile ↔ Blue Nile crossing allowed (§9.345). |

---

## Victory Conditions

### Campaign Game (§9.14)

**Mahdi's Tomb:** 25 VP to player who controls it at game end. Dervish controls it at start. To take it: one British leader + one non-Friendlies A-E combat unit (both undisrupted) must occupy it at game end.

| VP Superiority | Dervish Victory | Anglo-Egyptian Victory |
|---|---|---|
| Decisive | 30+ | 50+ |
| Tactical | 20–29 | 30–49 |
| Marginal | 10–19 | 15–29 |
| Draw | 1–9 | 1–14 |

**Alternative decisive:** A-E eliminates every Dervish unit. Dervish eliminates all A-E on west bank (excl. gunboats).

**Dervish VP:** +10 per British leader eliminated, +10 per British gunboat sunk, +1 per Friendlies eliminated on east bank, +3 per Friendlies eliminated on west bank, +3 per A-E land unit eliminated.

**A-E VP:** +1 for eliminating Isa Zachneih, +10 for eliminating Khalifa, +1 per Dervish unit eliminated (incl. gunboats, artillery, leaders). No points for forts.

### Historical Scenario (§9.24)

Subtract lower victory level from higher:

| Level | A-E (Dervish eliminated) | Dervish (A-E eliminated) |
|---|---|---|
| 5 Decisive | 100+ | 30+ |
| 4 Strategic | 60–99 | 15–29 |
| 3 Tactical | 45–59 | 10–14 |
| 2 Marginal | 30–44 | 5–9 |
| 1 Draw | 0–29 | 0–4 |

### Fall of Khartoum (§9.35)

| Victory | Condition |
|---|---|
| Dervish Decisive | Eliminate Gordon turn 4 or sooner |
| Dervish Tactical | Eliminate Gordon turn 5 |
| Dervish Marginal | Eliminate Gordon turn 6 |
| British Marginal | Gordon survives end of turn 6 |
| British Tactical | Gordon survives end of turn 7 |
| British Decisive | Gordon survives end of turn 8 |

Dervish loses 1 level if 16–23 units eliminated, 2 levels if 24–31, 3 levels if 32+.

---

## Optional Rules (§10, campaign only)

### River Mines (§10.1)
- Dervish records 2 Nile hexes (south of Khor Shambat E–W hexrow, not same hex) before play (§10.11).
- Gunboat enters mined hex: stop. Roll 1d10: 1–4 no effect, 5–7 drift 2 hexes/turn, 8–10 sunk (§10.12).
- Dervish gunboats pass safely (know mines) (§10.14).
- Only 2 mines; after both rolled, no more (§10.13).

### River Chain (§10.2)
- Dervish records ≤4 chain hexes (same area) before play (§10.21).
- Gunboat enters chained hex: stop for turn (§10.22).
- No gunboat crosses until chain sunk. Sink by: (a) infantry/cavalry spend full turn on bank adjacent to chained hex, or (b) artillery fire at chain, need 3+ on CRT (§10.23).

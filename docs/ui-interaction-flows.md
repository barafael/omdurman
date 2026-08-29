# UI Interaction Flows — Remember Gordon!

Derived from a close reading of the printed rulebook
(`Boardgame - Remember_Gordon/Manual/RememberGordonManual.md`). Part 1 extracts
everything in the manual that constrains or suggests UI; Part 2 turns that into
concrete, click-level player interaction flows. Rulebook sections are cited as
`§N`. Complements `ui-states.md` (state inventory) and `game-flow.md` (rules
digest); flows here are phrased at the level of "what the player clicks and what
the UI shows".

---

# Part 1 — What the manual implies for a UI

## 1.1 Persistent chrome (always on screen during play)

- **Turn record track** (§2.1, §4): current game turn with clock label — each
  turn = 2 hours; Campaign runs 6:00 AM Sep 1 → 8:00 AM Sep 3 (22 turns, §9.12);
  Historical 6:00 AM → 12:00 noon Sep 2 (4 turns, §9.22); FoK variable-length
  (§9.33). Needs a "night" visual state (§8).
- **Turn sequence / phase banner** (§4): the fixed phase ladder
  `Movement → Defensive Fire → Offensive Fire (Direct → Maxim/Howitzer) →
  Melee`, with the *non-active* player's defensive fire nested inside. The UI
  must show which rung is active and who acts on it (defensive fire is the
  *other* player's interaction window).
- **Charts on the "mapsheet"** (§2.1): TEC, Combat Tables (CRT + both Range
  Effects Tables), Howitzer Scattergram, LOS Table, Order of Appearance — all
  must be reachable as in-game reference panels without leaving the map.
- **Set-up letters** `A, D, Y, K, S, O` printed on the Historical map (§2.1,
  §9.212): map overlay labels used during Historical deployment.
- **Scenario switch**: two boards (Omdurman + FoK mini-map abutting the south
  edge, §2.1); Zariba hexsides are *printed* on the map but count as clear
  terrain in Campaign until constructed (§2.1, §5.3) — the map layer must
  render zariba hexsides conditionally.
- **Night badge** (§8.1): A-E movement halved, no A-E howitzers, all fire
  ranges halved (range 1 stays 1). Every range/MP calculation shown must
  reflect the night value.

## 1.2 Map presentation

- Hex ≈ 400–440 yd (§1.2). All full hexes playable — including the seven hexes
  of the printed Howitzer Scattergram (§2.1) — so overlays never remove hexes
  from selection.
- **Nile flow arrows** (§5.24): upstream/downstream direction must be visible;
  gunboat movement is direction-dependent.
- **Hexside features**: wall, gate, breach, khor, zariba, Nile-edge. These gate
  movement, ZOC, LOS, melee and advance, so hexsides must be hoverable and
  highlighted distinctly from hex terrain.
- **Named landmarks**: El Debeba, Khor Shambat, Kerreri huts, Abu Alim hut,
  Mahdi's Tomb, palace hexes, Forts Makran/Buri/North Fort — used by setup
  zones (§9.111, §9.211, §9.321) and victory conditions (§9.14, §9.346).
- **Deployment-zone tinting** implied by setup lists: east/west bank, "south of
  Khor Shambat", "in or south of El Debeba", "within three hexes of leader",
  "the 13 hexes of the Zariba", "any building or hut hex of Khartoum".

## 1.3 Counters and what the UI must read off them

- **Dervish combat unit**: Fire / Melee / Movement + tribe identifier (§2.3).
  Tribe matters everywhere: stacking (§5.52), leader-command matching (§5.53),
  color-coded setup (§9.212).
- **Anglo-Egyptian infantry**: Fire / Melee / Movement + **Battalion ID +
  Brigade ID** (e.g. `2B`, `3E`) — brigade integrity (§5.54) needs the UI to
  group and count battalions per brigade inside a stack.
- **Leaders**: Dervish leaders have fire+melee+move (full combatants, §6.51);
  A-E leaders have movement only, are eliminated if alone when an enemy passes
  through, and are *required* to take the Tomb (§6.51, §9.14). The UI should
  distinguish "fragile leader" vs "combat leader".
- **Gunboats**: old (Artillery line only) vs new **named** (Howitzer + Artillery
  + Maxim lines; §2.32) with two movement values `up/down` (§5.24) — counter
  art and inspector must expose both.
- **Maxims**: fire twice per turn (subphase bookkeeping, §6.42).
- **Forts**: artillery factor (fireable even unstacked), defensive-only melee,
  −3 fire DRM on occupants (§6.54). Immovable (§5.25).
- **Status markers** the game state requires: Disrupted `D` (no ZOC/move/fire/
  melee until rallied face-up at end of own turn), "constructing Zariba" blank
  counter (§5.3), RE demolishing (§6.53), BREACH markers with arrow orientation
  (§6.63), drifted/damaged gunboat (§10.12), loaded-Friendlies-on-gunboat
  (§5.21), "has fired this subphase" (§6.14).

## 1.4 Selection & highlighting vocabulary implied by the rules

- **Enemy ZOC hexes**: entry = forced stop (§5.43) → destination overlay must
  visually separate "stops here" hexes from free hexes.
- **Stacking slots**: ≤4 combat units + leaders free; gunboats alone (§5.51);
  same-tribe-only for Dervish (§5.52); destination tooltips should show
  occupancy and tribe conflict.
- **Wall/gate/breach/khor hexsides** during movement, melee targeting and
  advance: gate = out-only for ZOC and passable for melee/advance; breach =
  fully open; khor = never crossable by advance (§5.44, §7.2, §6.82).
- **In-range / out-of-range**: Range Effects Table gives ×3/×2/×1/×½/out per
  weapon line (§6.22) — target highlight color can encode the multiplier band.
- **LOS blocked**: "you can't fire at anything you can't see" (§6.3) except
  howitzer (§6.64); entrenched units don't block LOS (§9.232).

## 1.5 Attack preview panels (computed, then confirmed)

Every fire/melee attack the manual describes resolves as
*factors → modifiers → 1d10 → CRT row* (§6.22–§6.24, §7.7). A preview panel
must therefore assemble:

1. Total fire factors (with per-unit halving shown: round down each unit,
   never below 1 — §6.16; e.g. four 9-factor battalions halve to 16, not 18).
2. Range-band multiplier (per firing player's own table — A-E vs Dervish;
   Friendlies use the *Dervish* table, §6.52).
3. Terrain DRM of the target hex (§6.23); fort −3 if target stacked in fort
   (§6.54); Zariba thorn −2 / trench −4 (Historical/constructed, §9.23).
4. A-E +1 accuracy; +1 brigade integrity (§6.24); Dervish melee +2, A-E melee
   +1 (§7.7).
5. Die-roll clamp note: <1 → 1, >10 → 10.
6. Predicted CRT row for the current total.

## 1.6 Orchestrated sub-phases: allocate → then resolve

§6.41: *"The firing player must first allocate **all** of his fire attacks …
After all fire has been allocated, the firing player then resolves his attacks
in any order he wishes."* The UI needs a two-stage Direct Fire flow (an
"allocation tray" of planned attacks, then a resolution pass), not
resolve-as-you-click. §6.42 repeats the pattern for Maxim/Howitzer.

## 1.7 What the UI must forbid (validation checklist)

- Land units entering Nile hexes (§5.22); gunboats stacking (§5.51).
- Moving after entering an enemy ZOC in the same phase (§5.43); stacking
  different Dervish tribes (§5.52); Dervish leader with foreign-color units
  (§5.53).
- Entering/leaving the walled city except via gate/breach hexside; FoK: only
  Khalifa, Taiasha, Dervish artillery, any A-E (not gunboats/Friendlies)
  (§5.23).
- Fire: dividing a unit's factor across hexes (§6.13); firing/being fired at
  twice per subphase (§6.14); non-artillery targeting gunboats/forts/walls
  (§6.61–§6.63); howitzer outside 4–10 hexes or at night (§6.64, §8.1);
  no-LOS direct fire (§6.21).
- Melee: non-attackers (artillery, A-E leaders, gunboats attacking — §7.4,
  §7.1); across a wall hexside (§7.2); meleeing a unit not adjacent (§7.2).
- Advance: artillery advancing (§6.82); advancing across walls (except gate/
  breach) or khors (§6.82); into an (un)occupied enemy fort (§6.54).
- Zariba construction by non-infantry or a unit that moved away mid-turn
  (§5.3); RE demolitions if disrupted or if it fires/melees that turn (§6.53).
- Desertion choices excluding Khalifa/gunboats/artillery/forts (§8.2).

## 1.8 Hidden information

- Optional-rule placement is secret vs the A-E player: mines (2 Nile hexes,
  §10.11) and chain (≤4 river hexes, §10.21), both south of the Khor Shambat
  mouth row, Dervish-only visibility, no hostile ping on those hexes.
- Historical setup: Dervish units out of LOS of *all* A-E units (§9.212) —
  a setup-time LOS validator.

---

# Part 2 — Player interaction flows

Conventions: **Actor** = who clicks; **Phase** = when; **Pre** = preconditions;
steps are click-level; **Guard** = what the UI prevents/validates.

## Setup & hidden placement

### F1 — Campaign Dervish deployment (§9.111)

**Actor** Dervish · **Phase** Setup (before A-E "enters" — A-E has no at-start
units, §9.113)

1. Sidebar groups units by placement class: `Isa Zachneih`, `Khalifa`,
   `City garrison (3 artillery + Taiasha)`, `Forts ×17`, `Gunboats ×2`.
2. Click `Isa Zachneih` → east-bank hexes *in or south of El Debeba* tint
   green; click a hex to drop it.
3. Click `KHALIFA` → the two palace hexes pulse; click one.
4. Click each artillery/Taiasha unit → only walled-city hexes are valid.
5. Click a fort → legal region (west bank south of Khor Shambat and/or east
   bank south of the Halfaya hut hexes + Nile islands) tints; click hex. Repeat
   ×17 with a `12/17 placed` counter.
6. Click a gunboat → south-edge Nile hexes only.
7. **Confirm Ready.**

**Feedback**: live zone tinting per selected class; per-class placed/total
counters. **Guard**: zone checks per class; forts never on Nile hexes.

### F2 — Historical Dervish deployment (§9.212)

1. Leaders `A→Ali Wad Helu, D→Sheik El Din, Y→Yakub, K→Khalifa, S→Sherif,
   O→Osman Digna`: clicking a leader highlights *its lettered hex* on the map;
   clicking the lettered hex places it.
2. Click any tribe unit → hexes within 3 of that tribe's leader (color-matched
   halo) are valid.
3. Continuous **LOS validator**: any placement that would be visible from any
   A-E unit is flagged red with tooltip "Must set up out of LOS of all
   Anglo-Egyptian units (§9.212)"; Confirm stays disabled while any unit is
   exposed (A-E set up first).

**Guard**: leader-letter constraint, 3-hex radius, LOS sweep.

### F3 — FoK garrison deployment (§9.321)

Pick units (1 artillery, 2 British, 3 Egyptian, 4 Sudan, 4 Friendlies) → valid
hexes = building/hut hexes, Forts Makran/Buri, or hexes adjacent to a wall
hexside. GORDON auto-locked into the palace with a "may not move" padlock
(§9.346). Old gunboats → any Nile hex.

### F4 — Place river mines (secret, §10.1)

**Actor** Dervish (host) · **Phase** Setup, Campaign, optional rule toggled

1. Toggle "River Mines" in setup options → two-mine placement mode opens;
   map shows the legal band: Nile hexes *south of the E–W hexrow where Khor
   Shambat empties into the Nile*.
2. Click a Nile hex in the band → mine marker shown **only on your screen**
   (ghost icon + "hidden from opponent" padlock).
3. Click a second, different hex. **Confirm.**

**Guard**: Nile hexes only; band check; distinct hexes (§10.11); never
rendered to the A-E client; Dervish gunboats later pass without trigger
(§10.14). Resolution flow when triggered: gunboat enters → forced stop + Dervish
prompt "resolve mine" → 1d10 roll animation → 1–4 no effect / 5–7 "damaged:
drifts 2 hexes/turn" badge / 8–10 sunk (§10.12); mines counter decrements
(`2 → 1 → 0`, §10.13).

### F5 — Place the river chain (secret, §10.2)

**Actor** Dervish · **Phase** Setup, Campaign, optional rule toggled

1. Toggle "River Chain" → chain-drawing mode: legal band = Nile hexes south of
   the Khor Shambat-mouth row (§10.21).
2. Click a first Nile hex, then adjacent Nile hexes — a chain polyline snaps
   hex-to-hex as you click; a `n/4 hexes` counter updates.
3. Click "Place chain" (enabled at ≥2 hexes, disabled >4).

**Feedback**: chain rendered as a linked-barrier sprite across the hexsides,
Dervish-client-only. **Guards**: ≤4 hexes, contiguous river line, band check.

*Chain-in-play interactions (§10.22–§10.23):*
- A British gunboat entering a chained hex: movement halts, toast "The chain!
  Gunboat must stop (§10.22)"; remaining MP greyed out. No gunboat of either
  side may cross chain hexes until it is cut — the path overlay shows the
  chain as a wall across the river.
- **Cutting the chain**, two flows:
  a) **Land cutting:** select an infantry/cavalry unit → click a bank hex
     adjacent to a chained hex → action guide offers "Spend turn cutting the
     chain"; unit shows a "cutting" marker; if it ends the *next* A-E player
     turn still adjacent, chain-sunk animation + toast.
  b) **Artillery cutting:** artillery targets the chained hex like a fire
     attack; preview notes "Chain: sinks on CRT 3+ (§10.23)"; roll ≥3 → chain
     sprite sinks.

## Movement phase

### F6 — Basic infantry move with ZOC stop (§5)

1. Left-click a unit → pulsing selection ring; reachable hexes dotted; hexes
   that end movement (enemy ZOC, §5.43) outlined red.
2. Hover any hex → floating cost breakdown ("Rough 2 MP · khor hexside +1"),
   remaining MP on the unit card.
3. Click a mid-path hex to pin a route, then the destination — path arrows
   render.
4. Click another unit and repeat; moves commit per unit (allow undo until End
   Phase).

**Guards**: ZOC-entry auto-stops the path; stacking preview shows `3/4` on the
destination and blocks a 5th combat unit, blocks cross-tribe Dervish stacking
(§5.52), blocks gunboat+land stacking (§5.51); Nile hexes are never hoverable
for land units (§5.22); night badge halves displayed MA (§8.1).

### F7 — Gunboat move (§5.24)

1. Click a gunboat → two MP pools shown: `▲ up 10 / ▼ down 16` and current
   flow arrow on each adjacent Nile hex.
2. Click destination hexes along the Nile. The first *upstream* hex locks the
   pool to the upstream allowance for the rest of the move — UI swaps the pool
   and toast: "Moved upstream: 10 MP max this turn (§5.24)".
3. Optional-rule interplay: chained hex = hard stop after entering (F5); mined
   hex = stop + mine resolution (F4).

### F8 — A-E reinforcement entry, turn budget (§9.113)

1. Movement phase with pending reinforcements: a tray lists this turn's entry
   pool ("Turn 2: any three gunboats + any twelve land units") with a
   `0/12` budget meter (leaders never charge the meter).
2. Click a pool unit → legal entrance edges highlight: Nile north edge
   (gunboats, 1 MP first hex), Abu Alim hut (Friendlies, 8 MP), west-bank
   entrance area (1 MP).
3. Click an entrance hex to bring it on; the meter ticks. Leaders
   (KITCHENER/GATACRE/HUNTER) appear in a separate "must deploy by end of turn
   4" checklist that turns amber on turn 3 and red on turn 4.
4. Dervish mirror (§9.112): west edge south of Khor Shambat; each entrant pays
   the entry hex terrain cost — tooltip shows the cost per unit.

### F9 — Construct the Zariba (§5.3)

**Actor** A-E · **Phase** own Movement, Campaign only (pre-built in
Historical, §9.23)

1. Select an **infantry** unit that began the phase adjacent to a printed
   (unconstructed) Zariba hexside, Nile side. The action guide shows
   **"Build Zariba"** plus a preview of exactly which adjacent hexsides would
   be built (they light up gold).
2. The player may move first, but the UI tracks the origin hex: if the unit
   leaves and does not end adjacent, the button greys out with tooltip
   "Must begin AND end the turn adjacent (§5.3)".
3. Click **Build Zariba** → a "constructing" blank-counter badge snaps onto
   the unit; the two locked consequences display inline: 🚫 offensive fire,
   🚫 melee this turn (those units are filtered out of later fire/melee
   allocation this turn, with a hover explanation).
4. End of turn: adjacent Zariba hexsides animate from dashed (printed) to
   solid thorn/trench; the unit keeps the badge until its next turn's
   restrictions clear.

**Guard**: infantry-only; adjacency at both ends; hexsides only at the printed
Zariba position; cannot build a hexside the unit is not adjacent to.

### F10 — Royal Engineers demolition (§6.53)

1. Move the RE unit so it ends movement adjacent to a target fort *or* a wall
   hexside → action guide shows **"Demolish"** with the target highlighted.
2. Click **Demolish** → RE gets a demo badge + the same 🚫fire/🚫melee locks as
   F9 (stacked comrades may still fire — only the RE is locked).
3. Outcome deferred: a panel counts down "Resolves at end of A-E player turn if
   RE still adjacent and undisrupted".
4. At resolution: if RE was disrupted (by defensive fire) or moved, toast
   "Demolition failed"; else the fort sprite crumbles or a BREACH marker is
   placed pointing into the wall hexside, with side effects applied (§6.62/
   §6.63: one occupant/adjacent enemy eliminated — highlighted).

### F11 — Friendlies river transport, 3-turn arc (§5.21)

**Pre**: Isa Zachneih eliminated (UI condition shown as a satisfied ✓ in the
action guide).

1. **Turn N:** select a Friendlies unit adjacent to a gunboat → "Load" →
   unit renders stacked *on* the gunboat with a carried-cargo outline.
2. **Turn N+1:** the gunboat sails any Nile hexes (≤ MA) and must end adjacent
   to a west-bank hex; compass hint shows west-bank-adjacent hexes.
3. **Turn N+2:** "Disembark" on the carried unit → it drops onto the adjacent
   west-bank hex paying that hex's terrain cost; it may then move normally;
   the gunboat moves normally too.
A persistent mission tracker ("Friendlies ferry: ● load → ○ sail → ○ land")
survives across turns so the player never loses the arc.

### F12 — Dervish desertion roll (§8.2)

**Phase**: first *night* turn, Dervish Movement, once per game.

1. Banner action "Desertion Roll" appears; click → 1d10 animation.
2. Result panel: "N = roll × 1½ (round down). Choose N units to remove."
3. Click own units to strike them — ineligible units (KHALIFA, gunboats,
   artillery, forts) are greyed with tooltip; a `2/5 removed` meter tracks N.
4. Confirm → counters removed; footnote "no VP awarded to A-E for deserters".

## Fire combat

### F13 — LOS probe between two hexes (§6.3)

**Any time** (inspection tool, no game effect):

1. Right-click (or tool-button) a firing unit → probe mode.
2. Hover any enemy/any hex: a sight-line ray draws from counter to hex.
   - Clear → green ray, "LOS clear".
   - Blocked → brown ray with the blocking feature(s) highlighted and named
     ("Blocks: intervening building (LOS table)"), per the LOS table's
     terrain-vs-terrain intersect box + special notes.
3. The probe panel mirrors the LOS Table row/column ("Firer in Clear × target
   in Building") so players can verify against the printed table.
4. Special notes surfaced contextually: entrenched units (adjacent to a trench
   hexside) don't block (§9.232); howitzer fire is exempt from LOS entirely
   (§6.64); a BREACH marker flips its wall hexside to "does not block"
   (§6.63). Click `Esc` or another unit to re-probe.

### F14 — A-E attack with a full 4× battalion stack (brigade integrity, §5.54, §6.15, §6.24)

**Phase**: A-E Offensive Fire, Direct subphase (allocation stage, §6.41).

1. Click any unit of the stack → its legal targets (in range **and** LOS)
   highlight, colored by range band multiplier (×3 gold / ×2 green / ×1 white /
   ×½ grey).
2. Click a target hex → attack preview opens listing all four battalions with
   checkboxes. Above them, an integrity chip reads
   **"Brigade integrity: 2B — all four battalions present (+1)"**.
3. Tick all four → factor sum assembles with halving already applied per unit
   where applicable (§6.16); modifier stack shown:
   `factors 36 · range ×1 · target terrain −2 · A-E accuracy +1 · integrity +1
   → roll 1–10, clamp [1,10]` and the CRT row shaded for current total.
4. Click **Allocate attack** → the four counters each get a "fire: H2" ghost
   tag; the attack joins the allocation tray (see F16).
5. Hovering the target hex afterwards shows "committed by: 2B (×4)".

**Guard**: the +1 integrity chip applies **only** while all four fire at the
same hex — ticking three shows the chip struck through with reason
"integrity requires all four battalions on one hex (§5.54)". Units already
allocated this subphase are unclickable (§6.14).

### F15 — Splitting a stack's fire (§6.15)

Same start as F14; tick two battalions → target A; the preview keeps running
with a warning strip "Remaining units may fire elsewhere, but integrity +1 is
lost for all of 2B this attack". Allocate a second attack with the other two
battalions at a different hex — both attacks sit in the tray, neither with the
integrity bonus; tooltip explains the trade-off.

### F16 — Allocate-all-then-resolve (§6.41)

1. **Allocation stage:** repeat F14/F15 until the player clicks
   "Done allocating". The tray lists every planned attack
   (`attack 1: 32B+42B+… → hex (5,9)`), each still editable/removable.
2. **Resolution stage:** tray becomes a resolution queue in the player's
   chosen order (drag to reorder). Click "Resolve" on one → dice tray animates
   1d10 + modifiers → CRT result banner ("2 eliminated — counters flash"),
   dead/disrupted markers applied.
3. Units already fired at once are marked exempt as targets (§6.14); Maxims
   and gunboats stay targetable within their exceptions (§6.4).
4. "End subphase" only enabled when the player has passed or resolved all
   allocations.

### F17 — Defensive fire interjection (§6.7)

**Actor**: the *non*-active player, prompted mid-opponent-turn.

1. Prompt bar: "Dervish defensive fire — allocate all attacks". Same
   allocate→resolve mechanics as F16; no advance-after-combat ever (§6.7).
2. A-E defending gets the same plus the two-subphase split (Direct, then
   Maxim/Howitzer, §4-B).
3. Skippable per unit (fire is voluntary, §6.12) and per phase ("Pass").

### F18 — Maxim second fire (§6.42)

In the Maxim/Howitzer subphase, Maxims that fired in Direct show a
"already fired once — one shot left" pip; Maxims that *sat out* Direct get a
"single fire only here" badge (§6.42). Same target flow as F14 minus
integrity (Maxims aren't infantry).

### F19 — Howitzer fire with scatter (§6.64)

**Actor** A-E · **Phase**: own Maxim/Howitzer subphase · **Pre**: named
gunboat, day turn.

1. Select the gunboat → ring overlay of hexes at distance 4–10 (LOS ignored —
   ring renders over blocking terrain, labelled "no LOS needed").
2. Click a target hex → preview (no terrain mods, no LOS mods): "CRT die +
   impact die; hit on impact 7–10".
3. Click **Fire** → two sequential dice:
   - Die 1 resolves the CRT result first (banner shows it but holds
     application);
   - Die 2 → 7–10: impact crosshair locks on the target hex; 1–6: scattergram
     overlay spins and the impact marker hops to the scattered hex, arrow
     rendered.
4. Consequence application: if impact hex is friendly-occupied, a red warning
   "results must apply (§6.64)" — no abort possible. Combine-with-Maxim
   checkbox appears only when impact = intended hex.
5. Night: the whole flow is locked with "No howitzer fire at night (§8.1)".

### F20 — Artillery special targets (§6.61–§6.63)

When an artillery unit is selected, three extra target classes highlight:
- **Gunboat** → preview states "Sink on 3+; anything else = miss".
- **Fort** → "Destroy on 2+; if occupied, one occupant dies with it".
- **Wall hexside** → "Breach on 2+; on breach: marker placed, LOS negated,
  one adjacent enemy eliminated".
Non-artillery units simply never see these target classes (filtered, tooltip
"only artillery (§6.61–6.63)"). Resolution uses the normal CRT roll flow but
binary success/failure banners.

## Melee phase

### F21 — Plan a melee attack (§7)

**Phase**: Melee (own player turn).

1. Click an eligible attacker (infantry, cavalry, camel, Dervish leader; A-E
   leaders and artillery filtered out with reasons) → adjacent enemy hexes
   ring red; wall hexsides show a 🚫 on the ring segment; gates/breaches glow
   (§7.2).
2. Click a target hex → melee preview: `attacker melee total vs defender melee
   total · Dervish +2 / A-E +1 · no terrain mods (except Zariba §9.23) · fort
   −3 if defenders in fort`. A "simultaneous — defenders still roll if
   eliminated" note (§7.3).
3. **Allocate** (melee also follows declare-all-then-resolve so both players'
   attacks can interleave) → "Resolve" → both dice animate together; losses
   deducted from meleeing units first, highlighted (§7.7).
4. Counterattack bookkeeping: if the defender survives with an unused melee,
   UI offers its own attack (each unit attacks once per phase).

### F22 — Cavalry/camel retreat before melee (§7.5)

When a melee attack targets a hex containing enemy cavalry/camel, pre-
resolution prompt to the defender: "Retreat 2 hexes? (once per unit per
turn)". Defender clicks an escape path (2 hexes, pass-through friendly units
allowed); the attack then either re-targets their vacated hex or is declined —
and if the retreat path ends adjacent to *other* attackers not yet resolved,
those attackers get an "attack the retreating unit?" prompt. UI enforces the
once-per-turn retreat pip.

### F23 — Advance after combat (§6.82 melee §7.6)

- **Offensive fire vacated a hex**: participating, adjacent, non-artillery
  units each get an "Advance into [hex]?" chip; walls (except gate/breach) and
  khors are drawn as impassable on the advance arrow.
- **Melee**: Dervish player gets a **mandatory** advance modal — surviving
  eligible participants auto-march (up to stacking limit; player only picks
  which units fill it); A-E player gets an optional chip instead.

## Victory & meta flows

### F24 — Taking the Mahdi's Tomb (§9.14)

A persistent objective tracker (Campaign): "Tomb: held by Dervish". When an
A-E unit + leader occupy it, the tracker verifies and displays capture
requirements live: "Needs 1 British leader + 1 non-Friendlies combat unit,
both undisrupted, at game end" — units failing `undisrupted` are flagged.
At game end the tracker converts to +25 VP for the holder.

### F25 — Victory panel (§9.14, §9.24, §9.35)

Live VP ledger per side (each §9.14 scoring row with its current count:
leaders ×10, gunboats ×10, Friendlies east/west ×1/×3, land units ×3, Khalifa
×10, Isa Zachneih ×1, Dervish units ×1; forts = 0). Game-over screen computes
victory level from the superiority tables and announces alternative-decisive
conditions. Historical mode shows the level-subtraction arithmetic (§9.24);
FoK shows Gordon's death turn and the loss-based level penalties (§9.35).

### F26 — Unit inspector & chart lookup

Right-click any counter → card with printed values (fire/melee/move, tribe,
battalion/brigade IDs, gunboat up/down MA, maxim "fires 2×"), current status
pips (disrupted, has fired, constructing, cutting chain, loaded, drifting),
and eligibility summary ("may not fire: disrupted"). Chart buttons open the
CRT, both Range Effects Tables (A-E / Dervish), TEC, LOS Table and Scattergram
as overlay panels; from any attack preview, each modifier line links to its
chart row.

---

## Flow ↔ rulebook index

| Flow | Rulebook § |
|---|---|
| F1–F3 setup | 9.111–9.113, 9.211–9.212, 9.32 |
| F4 mines | 10.11–10.14 |
| F5 chain | 10.21–10.23 |
| F6 move/ZOC | 5.1, 5.2, 5.4 |
| F7 gunboats | 5.24 |
| F8 reinforcements | 9.112, 9.113 |
| F9 zariba | 5.3 |
| F10 RE | 6.53, 6.62, 6.63 |
| F11 ferry | 5.21 |
| F12 desertion | 8.2 |
| F13 LOS | 6.3, 6.21, 9.232, 6.63 |
| F14–F16 fire planning | 6.13–6.16, 5.54, 6.24, 6.41 |
| F17 defensive fire | 6.7, 6.12 |
| F18 maxims | 6.42 |
| F19 howitzer | 6.64, 8.1 |
| F20 artillery targets | 6.61–6.63, 6.54 |
| F21 melee | 7.1–7.4, 7.7 |
| F22 retreat | 7.5 |
| F23 advance | 6.82, 7.6, 9.233 |
| F24–F25 victory | 9.14, 9.24, 9.35 |
| F26 inspector | 2.3, 2.2 |

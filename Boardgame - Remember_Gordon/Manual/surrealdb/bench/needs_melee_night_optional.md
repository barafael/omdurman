> **Status:** every item below has been applied to the corpus (or answered with an `open_question`); kept as the change log of the benchmark pass.

# Data needs: melee, night, optional rules, history, back cover

From `bench/questions_40_melee_night_optional.json` (§7, §8, §10, §11, CombatResults, Disrupted,
TurnTrack). Five questions fail because the corpus lacks a fact the manual states or implies. Each
entry gives the question id, the manual text, what the database returns, and a fix.

## 1. `s7_5_infantry_retreat`: only cavalry and camels retreat

* Manual, 7.5: "Cavalry and camel units may retreat two hexes from an infantry melee attack."
  This is the only retreat in the rules (the CRT has no retreat result), so infantry, artillery,
  leaders, forts, Maxims, Royal Engineers and gunboats do not retreat.
* DB: `fn::allowed(unit:egyptian_battalion_sample, action:retreat)` returns `NONE` (unstated):
  only `unit_class:cavalry` and `unit_class:camel` have `retreat` edges.
* Fix (`data/07_capabilities.surql`, after the 7.5 block). Explicit `false` edges on the classes
  that are not cavalry or camel. None of them is an ancestor of cavalry or camel, so no edge needs
  `exception: true`:

```sql
FOR $c IN [unit_class:infantry, unit_class:artillery, unit_class:leader, unit_class:fort,
           unit_class:maxim_battery, unit_class:royal_engineers, unit_class:gunboat] {
  RELATE $c->capability->action:retreat CONTENT { allowed: false, rule: rule:r7_5_retreat, basis: "implied",
    note: "7.5 gives the two-hex retreat to cavalry and camel units only; the rules have no other retreat." };
};
```

## 2. `s11_named_gunboat_howitzers`: why the named gunboats have howitzers

* Manual, 11: "By mid-April the railroad had reached the Nile below Berber, bringing with it the
  new shallow draft gunboats designed specifically for river campaigns. … they were formidable
  concentrations of firepower with their 12 pounders, 6 pounders, and Maxim guns on the upper
  deck, and 4 inch howitzers on the gun deck."
* DB: `unit_class:named_gunboat.history` is `NONE`. No record carries this fact.
* Fix (`data/02_units.surql`, after the unit inserts):

```sql
UPDATE unit_class:named_gunboat SET history = "the new shallow draft gunboats of 1898, brought up by the railroad: 12 pounders, 6 pounders and Maxim guns on the upper deck, and 4 inch howitzers on the gun deck (11)", history_section = section:`11`;
```

## 3. `s11_17_forts`: why there are 17 forts

* Manual, 11: "To guard the approaches to the city, seventeen forts were constructed and armed
  with old artillery pieces."
* DB: `constant:campaign_forts.value = 17` (9.111), but `unit:fort.history` is `NONE`.
* Fix (`data/02_units.surql`):

```sql
UPDATE unit:fort SET history = "to guard the approaches to Omdurman, seventeen forts were constructed and armed with old artillery pieces (11)", history_section = section:`11`;
```

## 4. `s11_rifles`: the Danagla's rifles

* Manual, 11: "The few guns available, old Remingtons and brass muzzle loaders using home made
  cartridges, were issued to the Jehadia (commanded by the Khalifa's son, Sheik El Din) and the
  Danagla. The rest of the troops carried swords and spears."
* DB: `tribe:jehadia.history` holds this sentence; `tribe:danagla` has no `history`, so
  `SELECT VALUE id FROM tribe WHERE history CONTAINS "Remingtons"` returns only `tribe:jehadia`.
* Fix (`data/02_units.surql`):

```sql
UPDATE tribe:danagla SET history = "issued the few guns available (old Remingtons and brass muzzle loaders) together with the Jehadia (11)";
```

## 5. `sDisrupted_again`: disrupting an already disrupted unit

* Manual, back cover: "D* = ½ (round up) of the units in the target hex are disrupted
  (inverted)." and "DISRUPTED UNITS: … are turned face up at the end of the owning player's turn."
  Neither says what a D does to units that are already disrupted: whether they count toward the
  half, may be chosen again, or suffer anything more. Part B's own answer is "Not addressed."
* DB: no `open_question` records this. The query
  `SELECT VALUE kind FROM open_question WHERE refs CONTAINS state:disrupted AND question CONTAINS "already disrupted"`
  returns `[]`.
* Fix (`data/09_open_and_history.surql`, in the `open_question` insert; the `raises` loop picks
  it up):

```sql
{ id: open_question:disrupt_disrupted, kind: "gap", question: "What does a D result do to units that are already disrupted: may they be chosen as the disrupted half, and does a second D have any further effect?",
  refs: [section:`CombatResults`, section:`Disrupted`, combat_result:disrupted, state:disrupted] },
```

## Minor (no failing question)

* `s10_23_gunboat_artillery` passes (`fn::allowed(unit:sultan, action:fire_at_chain)` is `NONE`,
  which means unstated). Two small problems remain in the data:
  1. `open_question:artillery_meaning` names 10.23 in its question text, but its `refs` (and so
     its `raises` edges) leave out `section:`10.23``. Fix: add `section:`10.23`` to `refs`.
  2. Gunboats get implied `allowed: true` edges for the other "only artillery" targets
     (`fire_at_gunboat`, `fire_at_fort`, `breach_wall`) but none for `fire_at_chain`. Either add
     the matching implied edge
     `RELATE unit_class:gunboat->capability->action:fire_at_chain CONTENT { allowed: true, scenario: scenario:campaign, rule: rule:r10_23_sink_chain, basis: "implied", note: "As for fire_at_gunboat; see open_question:artillery_meaning." };`
     (then change the question's `expect` to `true` and note that it is implied), or leave it
     unstated on purpose and say so in the open question's `reading`.

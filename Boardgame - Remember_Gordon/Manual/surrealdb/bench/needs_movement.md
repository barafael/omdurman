> **Status:** every item below has been applied to the corpus (or answered with an `open_question`); kept as the change log of the benchmark pass.

# Data needs found by the movement benchmark (§3, §5)

Questions in `questions_20_movement.json` that fail, or pass only because the corpus is quiet,
because of a corpus gap. Each entry has the manual's text, what the database returns, and a
proposed fix.

## 1. `s5_21_lancers_ferry` (FAILS): no land unit other than the Friendlies is barred from loading

* Manual §5.21: "In general, naval transport missions are not allowed, i.e. gunboats may not carry
  any land units. The sole exception is ... the surviving units of the "Friendlies" brigade".
* DB: `fn::allowed(unit:lancers_21, action:load)` returns `NONE` (unstated). The only `load` edge is
  `unit_class:friendlies -> action:load {allowed: true}`.
* Fix (07_capabilities.surql), after the Friendlies `load` edge:
  ```sql
  RELATE unit_class:land_unit->capability->action:load CONTENT { allowed: false, rule: rule:r5_21_no_naval_transport, basis: "explicit" };
  ```
  and add `exception: true` to the existing `unit_class:friendlies->capability->action:load` edge,
  and add `action:load` to `rule:r5_21_no_naval_transport.actions` (06_rules.surql), so the rule
  also `governs` loading. The Friendlies edge is more specific than the new one, so it still wins.

## 2. `s5_53_mv_osman_sheik` (FAILS): `fn::stack_check` does not check Dervish leaders against each other

* Manual §5.53: "If Dervish leaders elect to stack, however, they may only stack with units of
  their command (i.e. color). For example, Sheik El Din may only stack with Mulazmins or Jehadias."
  A Dervish leader is a unit (§6.51: Dervish leaders "may thus attack, melee, and be eliminated
  like any other combat unit"). So Osman Digna stacked with Sheik El Din is stacked with a unit
  outside his command.
* DB: `fn::stack_check([unit:osman_digna, unit:sheik_el_din])` returns `{legal: true, counted_units: 0}`.
  `$v4` compares each Dervish leader only against `$others` (non-leaders).
* Fix (98_functions.surql, `fn::stack_check`): also compare each Dervish leader with the other
  Dervish leaders in the hex. Two leaders are compatible when their commands share a tribe:
  ```sql
  LET $v4 = $info.filter(|$i| $i.dervish_leader).map(|$l|
      IF array::len($info.filter(|$o| $o.item != $l.item AND (
            ($o.dervish_leader AND array::len(array::intersect($o.commands, $l.commands)) = 0)
         OR (!$o.leader AND ($o.tribe = NONE OR $l.commands CONTAINSNOT $o.tribe))))) > 0 {
        string::concat($l.item.name, " may only stack with units of his command (5.53)") } ELSE { NONE }).filter(|$s| $s != NONE);
  ```

## 3. `s5_53_op_turn1_leader_assignment` (passes): `fn::stack_check` claims more than the manual says for the turn-1 leaders

* Manual §9.112 lists "all Baggara, Jaalin, Danagla, Kehena, and Degheim units, and their leaders:
  YAKUB, SHERIF, and ALI WAD HELU" without saying which leader commands which tribe. §5.53 limits a
  leader to "units of their command (i.e. color)". The colors are on the counters, not in the text.
* DB: `command_group:turn1_leaders.individual_assignment = "unstated"` (correct), but
  `fn::stack_check([unit:yakub, tribe:jaalin])` returns `legal: true`, and so would Yakub with any
  of the five tribes.
* Fix (98_functions.surql): when a stacked Dervish leader's command group has
  `individual_assignment = "unstated"` and the hex holds tribe units, return `legal: NONE` (or
  `undetermined: ["leader-to-tribe assignment unstated (9.112, 5.53)"]`) instead of `true`.

## 4. `s5_41_op_gunboat_zoc_scope` (FAILS): no open question for gunboat ZOCs (Part C 16)

* Manual §5.41: "(exception: Gunboats exert a ZOC only against enemy gunboats)". §5.44: "ZOCs do not
  extend into or out of a Nile River hex (exception: Gunboats, see 5.41)". Nothing says whether that
  ZOC reaches across a river chain (§10.2) or how it bears on units on Nile islands.
* DB: `SELECT VALUE id FROM open_question WHERE refs CONTAINS section:`5.41`` returns `[]`.
* Fix (09_open_and_history.surql, add to the `INSERT INTO open_question` array; the `raises`
  loop makes the edges):
  ```sql
  { id: open_question:gunboat_zoc_scope, kind: "gap",
    question: "Gunboats exert a ZOC only against enemy gunboats (5.41). Does it extend to an enemy gunboat across a river chain (10.2), and does any ZOC pass between a gunboat and units on a Nile island?",
    refs: [section:`5.41`, section:`5.44`, section:`10.21`, rule:r5_41_gunboat_zoc] },
  ```

## 5. `s5_51_op_fort_stacking` (FAILS): whether a fort counts toward the four-unit limit is decided, not recorded as open

* Manual §5.51: "No more than four units may occupy a hex, with the exception of leaders and
  gunboats." §6.54: "friendly units stacked inside the fort"; "Players may not occupy an enemy fort".
  The fort counter acts both as a unit (it fires, melee defends, has a ZOC) and as terrain that
  units are "inside". The manual never says whether it counts toward the four.
* DB: no open question. `fn::stack_check([unit:fort, tribe:baggara ×4])` returns `legal: false`
  ("more than four units"), so it counts the fort.
* Fix (09_open_and_history.surql):
  ```sql
  { id: open_question:fort_stacking, kind: "gap",
    question: "Does a Dervish fort counter count as one of the four units allowed in a hex (5.51)? 6.54 speaks of units 'stacked inside the fort', as if the fort were terrain.",
    refs: [section:`5.51`, section:`6.54`, unit_class:fort] },
  ```
  Optionally have `fn::stack_check` leave forts out of `counted_units` and add an
  `undetermined` note when a fort plus four units share a hex.

## 6. `s5_51_op_move_through_enemy` (passes): moving through enemy units is not recorded as open (Part C 18)

* Manual §5.51: "Players may move through friendly units at no additional cost in movement points."
  No rule covers moving through enemy units. ZOC stops (§5.43) usually make it moot, but a disrupted
  enemy has no ZOC (§5.41). The only stated case is §6.51: an Anglo-Egyptian leader alone in a hex
  is eliminated "when a Dervish unit occupies or passes through that hex".
* DB: only `action:move_through_friendly` exists (plus `action:overrun_leader`). There is no open question.
* Fix (09_open_and_history.surql):
  ```sql
  { id: open_question:move_through_enemy, kind: "gap",
    question: "May a unit move through a hex occupied by enemy units that exert no ZOC (disrupted units)? 5.51 permits moving through friendly units only; 6.51 lets a Dervish unit pass through a lone Anglo-Egyptian leader.",
    refs: [section:`5.51`, section:`5.41`, section:`6.51`, action:move_through_friendly] },
  ```

## Part B answers corrected (no data change needed)

* `s5_24_downstream_then_back`: Part B says "(Yes, if the total stays within 10.)". 10 hexes down
  plus 2 up is 12, over the upstream cap of 10 (§5.24), so the answer is No.
* `s5_11_hut_cost`: the hut movement cost is a Terrain Effects Chart cell, not in the manual text.
  The question checks that no cost is recorded and that `open_question:range_cells` logs the gap.
* `s3_home_edge`: §3 gives only seating (Dervish west, Anglo-Egyptian east); "home edge" is not a
  term of the rules.

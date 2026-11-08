# Remember Gordon! — SurrealDB rulebook corpus

A graph database of the 1982 rulebook (`../RememberGordonManual.md`): its nouns (units, places,
terrain, phases, turns, scenarios), verbs (`action`) and mechanisms (`rule` and structured rule
data), all linked to the manual paragraphs they come from. A benchmark of player questions,
each written as a SurrealQL query with the answer the manual supports, lives in the database too.

```sh
./start.sh                 # podman: in-memory SurrealDB 4.0 on 127.0.0.1:8765, then load
python3 rg.py all          # reload + consistency checks + benchmark (writes bench/REPORT.md)
python3 rg.py check        # consistency checks only
python3 rg.py bench -v     # benchmark only
python3 rg.py sql 'RETURN fn::stack_check([unit:osman_digna, tribe:baggara]);'
```

Files: `schema.surql` (tables), `data/*.surql` (records, loaded in name order; `00_sections` is
generated from the manual by `tools/gen_sections.py`), `checks.surql`, `bench/questions_*.json`.

## Provenance

* `section` holds every manual paragraph verbatim (`section:`5.24``, `section:`4.A``,
  `section:`CombatResults``, `section:`Disrupted``, `section:`TurnTrack``, `section:`credits``),
  with `parent`, `title`, `kind`, and `cites` edges for the manual's own cross-references.
* Every rule-bearing record has `section`. Facts the manual states are `basis: "explicit"`;
  facts read off it are `basis: "implied"` with a `note`. Where the manual is silent or
  inconsistent, the corpus records an `open_question` instead of choosing an answer.
* The table cells printed on the mapsheet (Terrain Effects Chart, Range Effects Tables, LOS
  Table, CRT, Scattergram) are **not** in the manual text, so they are not in the corpus.

## Nouns

| table | what | key fields |
|---|---|---|
| `faction` | `anglo_egyptian`, `dervish` | name, player_role, seat, opponent |
| `unit_class` | taxonomy, `is_a` edges, `ancestors` (closure) | name, abstract, faction |
| `unit` | named counters and counter kinds | name, faction, class, classes, fire, melee, move, move_upstream/downstream, artillery_factor, howitzer_factor, maxim_factor, count, generic, tribe, formation, british_leader |
| `tribe` | Dervish tribes | name, unit_class, color (only green is stated) |
| `command_group` | Dervish leaders and their tribes (`leads`, `member_of` edges) | basis, individual_assignment |
| `formation` | Anglo-Egyptian brigades / division | designation, nationality, battalions |
| `weapon`, `range_table`, `range_line` | armament; `fires_on` edges from unit/class/tribe | table, weapon |
| `terrain` | hex / hexside types | kind, effects, zoc |
| `location` | named places, areas, edges, lettered hexes; `member_of {relation: "part_of"}` | map, kind, hexes, bank, controller |
| `map`, `component`, `marker` | mapsheets, play aids, game parts, markers | |
| `state` | disrupted, entrenched, brigade_integrity, night, ... (`prohibits` edges to actions) | effects, recovery, definition |
| `combat_result` | `miss`, `disrupted`, `eliminated` | symbol, meaning |
| `constant` | numbers of the rules (`value` or `min`/`max`) | name, value, section |
| `person`, `historical_event` | section 11 | |

Unit class tree (leaves in bold): unit > combat_unit / land_unit > infantry (**d_infantry**,
ae_infantry > **british_infantry**, **egyptian_infantry**, **sudanese_infantry**; **friendlies**),
cavalry (**ae_cavalry**), camel (**d_camel**, **camel_corps**), artillery (**d_artillery**,
**ae_artillery**), gunboat (**d_gunboat**, **old_gunboat**, **named_gunboat**), leader
(**d_leader**, **ae_leader**), **fort**, **maxim_battery**, **royal_engineers**.

Units: `kitchener`, `gatacre`, `hunter`, `gordon`, `khalifa_abdullah`, `yakub`, `sherif`,
`ali_wad_helu`, `osman_digna`, `sheik_el_din`, `isa_zachneih`, `danagla_sample`,
`dervish_artillery` (×3), `dervish_gunboat` (×2), `fort` (×17), `lancers_21`, `egyptian_cavalry`,
`camel_corps`, `battery_32`, `horse_artillery`, `egyptian_battalion_artillery`, `maxim_battery`,
`royal_engineers`, `egyptian_battalion_sample`, `old_gunboat`, `sultan`.

## Time

* `phase`: tree `game_turn` > `ae_player_turn` / `d_player_turn` > phases > fire steps >
  subphases, plus `end_of_game_turn`. Leaves carry `step` (1 movement … 6 melee),
  `player_turn` (whose turn), `actor` (who acts), `fire_mode`. `next` edges chain the leaves.
* `turn:1` … `turn:22`: the Turn Record Track (n, date, label, time, night, special); `next` edges.
* `scenario`: `campaign`, `historical`, `fall_of_khartoum` (sets_up_first, moves_first,
  length_turns, night_turns, maps, ...).

## Verbs and mechanisms

* `action` — 54 verbs (`move`, `melee_attack`, `howitzer_fire`, `advance_after_combat`, `desert`, ...);
  `allowed_in` edges to phases.
* `rule` — atomic statements: section, statement, kind (definition, permission, prohibition,
  obligation, procedure, modifier, threshold, effect, condition, recommendation, scope, sequence,
  limit), scenarios (NONE = all), params, and `actions` / `concerns` / `exception_to` arrays,
  mirrored as `governs`, `concerns`, `exception_to` edges.
* `capability` edges `(unit|tribe|unit_class) -> action`: `allowed`, `condition`, `rule`,
  `basis`, `exception`, `scenario`. The most specific subject wins (see `fn::can`).
* `entry` edges `(unit|tribe|unit_class) -> (location|terrain)`: may (not) enter.
* `modifier`, `threshold`, `die_outcome`, `zoc_boundary` (5.44: `out_of`, `into`).
* `setup`, `reinforcement`, `vp_award`, `victory_level`, `victory_condition` — scenario data.
* `open_question` — gaps, ambiguities, inconsistencies of the manual; `raises` edges.

## Functions

| function | ruling |
|---|---|
| `fn::can(subject, action)` / `fn::allowed(...)` | capability (object / true, false, NONE = unstated) |
| `fn::may_enter(subject, place)` | entry permission object |
| `fn::lines(subject)`, `fn::range_table(subject, scenario?)` | range lines fired on; table (9.343 override) |
| `fn::stack_check([items])` | 5.51–5.53 legality; items are units, tribes (one unit of it) or classes |
| `fn::halve([factors])` | 6.16 halving |
| `fn::fire_drm({firer, direct, integrity, target_in_fort, terrain, across_thorn_hedge, target_entrenched})` | fire die modifier |
| `fn::net_roll(die, drm)`, `fn::clamp_roll(r)` | clamped net roll |
| `fn::melee_mod(subject, attacking_entrenched)` | 7.7 / 6.52 / 9.232 |
| `fn::disrupted_count(n)` | units disrupted by a D |
| `fn::special_hit(target, result)` | 'gunboat', 'fort', 'wall hexside', 'river chain' |
| `fn::howitzer_range_ok(hexes)`, `fn::howitzer_impact(roll)`, `fn::mine(roll)`, `fn::desertion(roll)` | |
| `fn::night_ma(faction, ma)`, `fn::night_range(range)`, `fn::gunboat_ma(unit, moves_upstream)` | |
| `fn::zoc_extends(terrain, 'into'|'out_of')` | 5.44 |
| `fn::turn(n)`, `fn::is_night(scenario, n)`, `fn::sequence(scenario)`, `fn::arrivals(scenario, faction, turn)` | |
| `fn::campaign_level(faction, superiority)`, `fn::campaign_result(ae_vp, d_vp)` | 9.14 |
| `fn::historical_result(dervish_eliminated, ae_eliminated)` | 9.24 |
| `fn::fok_result(gordon_eliminated_turn | NONE, dervish_lost)` | 9.35 |

## Benchmark

`bench/questions_*.json` — arrays of questions:

```json
{ "id": "s5_24_upstream_cap", "section": "5.24", "kind": "player",
  "text": "If I move one hex upstream, can I still use the 16?",
  "query": "RETURN fn::gunboat_ma(unit:old_gunboat, true);",
  "expect": 10, "match": "exact",
  "answer": "No: one upstream hex caps the turn at the upstream allowance (10).",
  "about": ["unit:old_gunboat", "action:move_upstream"] }
```

`kind`: `player` (from the extracted question list), `basic` (plain rules lookups), `graph`
(relationship traversals), `open` (the manual does not settle it; the query shows that).
`match`: `exact` (default), `set` (order-insensitive list), `contains`. The expected answer is
what the manual supports; the last statement's result is compared. `rg.py load` stores the
questions as `question` records with `about` edges; `rg.py bench` writes `last_result`/`passed`.

## Current state

* ~2,900 records: 128 manual sections, 145 rules, 54 actions, 191 capability edges, 42 open
  questions, 748 benchmark questions (`question` table, with `about` edges to sections/entities).
* `rg.py check`: 21/21 consistency checks pass. The checks also catch deliberately injected
  contradictions (a conflicting permission, a changed constant, overlapping victory bands).
* `rg.py bench`: 748/748 pass — 351 player questions (the extracted list), 284 basic, 64 graph,
  49 open. Per-question results with queries: `bench/REPORT.md`.
* `bench/needs_*.md`: data problems the benchmark found and how each was fixed.

## In the browser

`web/` turns the corpus into a static website that runs it as SurrealDB-in-WebAssembly: the armies as
counters with dossiers, the field, the turn track, the scenarios with victory calculators, the rule
functions as instruments, the rulebook, the open questions, all 748 questions runnable, and a notebook.
See `web/README.md`. Every table grants `select` to everyone so the browser can seal the database as a
read-only guest.

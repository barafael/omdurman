> **Status:** every item below has been applied to the corpus (or answered with an `open_question`); kept as the change log of the benchmark pass.

# Data needs from the §9 (scenarios) benchmark

Questions: `bench/questions_50_scenarios.json`. Two of them fail because of a gap in the data, listed
under A. Section B lists additions that would not change any result but would record the manual's
silences as `open_question` records. Today those silences show up only inside rule functions.

## A. Failing questions

### A1. Campaign turn 4's "all remaining Anglo-Egyptian units" has no subjects

Failing: `s9_graph_historical_not_campaign`, `s9_graph_camel_corps_scenarios`.

* **Manual, §9.113:** "Turn 4) All remaining Anglo-Egyptian units." The campaign brings in every
  Anglo-Egyptian unit by the end of turn 4. Every unit the historical scenario sets up in the Zariba
  or at Kerreri (§9.211) is therefore also in the campaign game.
* **DB:** `reinforcement:c_ae4` has `all_remaining: true` and no `subjects`. The `appears_in` loop
  in `08_scenarios.surql` therefore links the Camel Corps, 21 Lancers, 32 Battery, Maxim Battery,
  Royal Engineers and brigades 2B/3E/4E to `scenario:historical` only.
  * `SELECT VALUE ->appears_in->scenario FROM ONLY unit:camel_corps` returns `["scenario:historical"]`
    (the manual's answer is campaign and historical).
  * The query for "units in historical but not campaign" returns `[royal_engineers, lancers_21,
    battery_32, camel_corps, maxim_battery]` (the manual's answer is none).
* **Fix** (in `data/08_scenarios.surql`, before the two `appears_in` `FOR` loops at the end):

```sql
UPDATE reinforcement:c_ae4 SET
  subjects = [unit:lancers_21, unit:camel_corps, unit:battery_32, unit:maxim_battery, unit:royal_engineers,
              formation:brigade_2b, formation:brigade_3e, formation:brigade_4e],
  subjects_note = "'All remaining Anglo-Egyptian units' (9.113): the named units of the counter mix not scheduled for turn 1; any of them may already have entered among the twelve land units of turns 2-3.";
```

(Alternative that leaves the reinforcement record alone: add
`RELATE <x>->appears_in->scenario:campaign SET via = reinforcement:c_ae4;` for the same eight subjects.)

## B. Proposed `open_question` records (no failing question; the gap is only implicit today)

These are Part C items in §9's scope. The corpus currently records them only inside function bodies:
a `note` string in `fn::campaign_result`, a NONE result from `fn::historical_result`, and a COMMENT on
`fn::fok_result`. Add them to the `INSERT INTO open_question` array in `data/09_open_and_history.surql`.
The `raises` loop that follows then links them. Once they exist, add their ids to the expected set of
`s9_graph_open_questions_s9`.

```sql
{ id: open_question:zero_superiority, kind: "gap",
  question: "Who wins the campaign game when both sides finish with equal victory point totals? 9.14's victory table starts at 1 point of superiority for both players.",
  refs: [section:`9.14`, victory_level:c_ae_draw, victory_level:c_d_draw],
  reading: "fn::campaign_result returns no winner and no level for zero superiority." },
{ id: open_question:historical_equal_levels, kind: "gap",
  question: "In the historical scenario, what is the net result when both players reach the same victory level? Subtracting gives 0, and 9.24's lowest level is 1 (DRAW).",
  refs: [section:`9.24`, victory_condition:h_net, rule:r9_24_net_victory],
  reading: "Presumably a draw; fn::historical_result returns net 0 with no level." },
{ id: open_question:gordon_late_elimination, kind: "gap",
  question: "FALL OF KHARTOUM: what is the result if GORDON is eliminated on turn 7 or 8? 9.35 names Dervish results only through turn six.",
  refs: [section:`9.35`, victory_level:k_b_marginal, victory_level:k_b_tactical],
  reading: "Implied: a turn-7 elimination means GORDON survived the end of turn six (British marginal), a turn-8 elimination that he survived turn seven (British tactical). fn::fok_result encodes this." },
{ id: open_question:fok_gunboats_night, kind: "ambiguity",
  question: "On FALL OF KHARTOUM's night turn 1 (9.341), are the British gunboats' upstream/downstream allowances halved? 8.1 halves 'all Anglo-Egyptian movement allowances'.",
  refs: [section:`9.341`, section:`8.1`, unit:old_gunboat],
  reading: "By the letter of 8.1, yes; fn::night_ma applies it to any Anglo-Egyptian allowance." }
```

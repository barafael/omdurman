> **Status:** every item below has been applied to the corpus (or answered with an `open_question`); kept as the change log of the benchmark pass.

# Data needs found by the fire-combat questions (questions_30_fire.json)

Three questions fail because the corpus is missing something. Each one keeps the manual's answer.

## 1. `s6_42_old_gunboats`: old gunboats have no Maxim second fire

* Manual: 6.42, "Anglo-Egyptian named gunboats may now fire their artillery factor as howitzer
  fire (see 6.64) and all Maxim guns may fire a second time." The old gunboat counter is
  "GUNBOAT, 4·10/16 — Artillery Factor" (2.3) and carries no Maxim guns, unlike the named
  gunboat ("Maxim Guns (fire twice per turn)"). So an old gunboat has nothing to fire in 6.42.
* DB: `fn::allowed(unit:old_gunboat, action:maxim_second_fire)` returns NONE (unstated), because
  no capability edge covers it. The howitzer half already returns false.
* Fix (07_capabilities.surql):

  ```surql
  RELATE unit_class:old_gunboat->capability->action:maxim_second_fire CONTENT { allowed: false,
    rule: rule:r6_42_second_subphase, basis: "implied",
    note: "The old gunboat counter (4·10/16, 2.3) has an artillery factor only and no Maxim guns; 6.42's second fire is for Maxim guns, and its howitzer fire for named gunboats." };
  ```

  Don't add the same edge for `unit_class:d_gunboat`. Whether Dervish gunboats fire twice is
  `open_question:dervish_defensive_maxims`, and `s6_14_open_dervish_fire_twice` expects NONE there.

## 2. `s6_62_separate_attack`: no open question for firing at a fort vs. its garrison

* Manual: 6.62, "Only artillery may fire at forts. A result of 2 or more on the combat results
  table is required to eliminate a fort. … If the fort contains any enemy units at the instant it
  is destroyed, one unit is eliminated with the fort." 6.54: "The −3 defensive value is deducted
  from the die roll of enemy fire attacks on friendly units stacked inside the fort." Neither says
  whether firing at the fort and firing at the units inside are separate attacks, whether one
  artillery attack (one CRT result) can do both, or whether non-artillery units may fire at the
  garrison of a hex where artillery fires at the fort. Part B points to Part C for this, but
  Part C has no entry.
* DB: no open question covers it (`open_question:fort_target_vs_garrison` is NONE).
* Fix (09_open_and_history.surql, in the `INSERT INTO open_question` array):

  ```surql
  { id: open_question:fort_target_vs_garrison, kind: "ambiguity",
    question: "Is artillery fire at a fort (6.62, 2+ destroys it) a separate attack from fire at the units stacked inside it (6.54, −3)? Can one attack do both, and may other units join the attack on the garrison?",
    refs: [section:`6.62`, section:`6.54`, rule:r6_62_fort_target, rule:r6_54_fort_defense, threshold:destroy_fort] },
  ```

  The `raises` edges come from the existing `FOR $q IN (SELECT id, refs FROM open_question)` loop.

## 3. `s6_64_cites`: 6.64's "see 4" is missing from `cites`

* Manual: 6.64, "They may fire their artillery factor as direct fire during the Direct Fire
  Subphase (see 4 and 6.41) and may then fire the same artillery factor as howitzer fire during
  the Maxim Second Fire and Howitzer Subphase (see 4 and 6.42)."
* DB: `SELECT VALUE out.num FROM cites WHERE in = section:`6.64`` returns only "6.41", "6.42".
  The citation regex in `tools/gen_sections.py`, `NUM_REF = (\d{1,2}\.\d{1,3})`, needs a dot, so it
  never matches a bare chapter number like "4".
* Fix: either extend the generator to catch bare chapter references after "see" (for example a
  second pattern `\bsee (\d{1,2})(?=[ )]| and)` whose match names an existing section), or add
  the edge by hand after the sections load (e.g. in 99_derive.surql):

  ```surql
  RELATE section:`6.64`->cites->section:`4`;
  ```

## Minor (no failing question)

* 6.7 says "(see 6.1 to 6.6)". `cites` holds 6.7 → 6.1 and 6.7 → 6.6 only. If a range citation
  should cover the sections in between, add 6.7 → 6.2, 6.3, 6.4, 6.5.
* `fn::fire_drm` adds the Anglo-Egyptian +1 only when `direct: true`. A caller asking about
  howitzer fire who passes `direct: false` gets 0, which quietly picks one reading of
  `open_question:howitzer_accuracy_bonus`; the Friendlies firing as `firer: anglo_egyptian` get +1,
  which quietly picks one reading of `open_question:friendlies_accuracy_bonus`. The function's
  comment could name both open questions, so a caller knows the result depends on a reading.

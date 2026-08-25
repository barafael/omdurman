# Plan: Address Missing/Incomplete Data Sources

## Status of the 6 original gaps

| # | Gap | Current status |
|---|-----|---------------|
| 1 | Howitzer scattergram not wired into code | **Partially fixed** — `tables/howitzer_scattergram.ron` now has the full 10-entry d10→hex mapping, but Rust code ignores it |
| 2 | 16 unannotated counter-sheet cells | **15 remain** — `units.ron` header lists them; cells need reading from `Units/units_photo.png` |
| 3 | Historical/FoK scenario setup manifests | **Partially fixed** — `omdurman-app/src/scenario_setup.rs` handles fixed placements; player-choice zones ("within 3 hexes", "anywhere in walled city") still unstructured |
| 4 | Victory tables not in tables/ | **Still missing** — all VP data lives only in `lib.rs` enums |
| 5 | Mine/chain boundary row concept | **Still missing** — no `NamedBoundary` type; rule §10.11/§10.21 constraint unmodelled |
| 6 | "iower" typo in LOS table | **Fixed** — `.ron` version is clean; old `.txt` archived in `txt-backup/` |

---

## Task 1: Wire howitzer_scattergram.ron into the engine

**Files:** `omdurman-rules/src/howitzer_scatter.rs`, `omdurman-rules/src/effects.rs`, `tables/howitzer_scattergram.ron`

The RON already encodes what the printed scattergram shows — d10 rolls 1–10 map to 7 hex positions around the target (6 directions + centre). The Rust code collapses rolls 1–2 into a single `LeftRight` and loses the distinction between UpperLeft, UpperRight, Left, Right, LowerLeft, LowerRight.

**Sub-tasks:**

1a. **Expand `ScatterDirection` enum** in `howitzer_scatter.rs` from 4 variants to the 7 distinct outcomes from the RON (UpperLeft, UpperRight, Left, Right, LowerLeft, LowerRight, OnTarget). Remove `LeftRight`, `Short`, `Long` (these were approximations).

1b. **Update `howitzer_scatter()`** to match the RON table exactly: roll 1→UpperLeft, 2→UpperRight, 3→Right, 4→LowerRight, 5→LowerLeft, 6→Left, 7–10→OnTarget.

1c. **Update `effects.rs::howitzer_impact_hex()`** (line ~2462) to handle all 7 directions, using hex-direction math to displace one hex in the correct direction relative to the firer→target bearing. The current `LeftRight` code (line 2486) already has the perpendicular logic — split it into left vs right.

1d. **Add a test** that reads the RON file at compile time (or hardcodes the table as a const) and verifies `howitzer_scatter()` agrees with every entry.

1e. **Update the 5 existing tests** in `howitzer_scatter.rs` to cover the new 7-variant enum.

---

## Task 2: Transcribe the 15 unannotated unit cells

**Files:** `tables/units.ron`, `docs/units.md`, `Units/units_photo.png`

The cells whose values are unknown (from `units.ron` header + `docs/units.md:313`):

| Section | Cells | Count |
|---------|-------|-------|
| Sherif | 0,1 | 1 |
| JaalinII | 6,0, 6,1 | 2 |
| Baggara | 6,0, 6,1 | 2 |
| BritishBoats | 0,0 0,1 1,0 1,1 2,0 2,1 | 6 |
| AliWadHelu | 6,0, 6,1 | 2 |
| Jehadia | 5,1, 6,0, 6,1 | 3 |

**Sub-tasks:**

2a. **Read each cell** from `Units/units_photo.png` — fire/melee/movement values, unit type (infantry/boat/fort), colour, tribe, printed text.

2b. **Add entries** to `tables/units.ron` following the existing schema.

2c. **Remove the "not yet known" warnings** from `units.ron` header and `docs/units.md`.

2d. **Regenerate `docs/units.md`** (it's derived from the RON).

> **Note:** This task requires visual reading of the scanned image. The model currently executing this plan cannot read images. Flag for human transcription or use the image-reading MCP tool.

---

## Task 3: Structure the Historical/FoK player-choice setup data

**Files:** new `tables/scenario_setups.ron` (or similar), `omdurman-app/src/scenario_setup.rs`

The fixed-hex placements (leaders on A/D/Y/K/S/O, Gordon in palace) are already in `scenario_setup.rs`. What's missing is a structured description of the *player-choice zones*:

- **Historical Dervish** (§9.212): all remaining Dervish units "within 3 hexes of their leader as identified by color" — per-tribe starting pools.
- **Historical Anglo-Egyptian** (§9.211): gunboats "adjacent to Zariba", Camel Corps/Cavalry/HArt "Kerreri hut hexes", "all remaining units in the 13 Zariba hexes."
- **FoK British** (§9.321): Gordon in palace, 2 old gunboats, 1 Egyptian Battalion, 2 British, 3 Egyptian, 4 Sudan, 4 Friendlies — all "in any building or hut hexes of Khartoum, Forts Makran/Buri, or adjacent to any wall hex."
- **FoK Dervish** (§9.322): 32 Mulazmin, 2 Hadendowa, 6 Kehena, 5 Degheim, 3 Dervish artillery — enter "any hexes on south or east edge."

**Sub-tasks:**

3a. **Create `tables/scenario_setups.ron`** with per-scenario, per-side structured entries: unit counts by type/brigade, entry zones (hex criteria), fixed placements (if any), and not-in-play lists.

3b. **Wire `scenario_setup.rs`** to read from the new RON for the unit-count manifests (validate that players placed the right number of units before the first turn).

3c. **Optionally** use the RON to auto-generate the `scenario_setup.rs` fixed-placement list (currently hand-written `HISTORICAL_LEADERS` const).

---

## Task 4: Transcribe victory tables into tables/

**Files:** new `tables/victory_tables.ron`, `omdurman-rules/src/lib.rs`

Three victory systems exist only as Rust code:

- **Campaign** (§9.14): VP per event (9 sources) + superiority bands (AE: 15/30/50; Dervish: 10/20/30)
- **Historical** (§9.24): Dervish units eliminated thresholds (30/45/60/100) and AE units eliminated thresholds (5/10/15/30)
- **FoK** (§9.35): GORDON turn ladder (turns 4/5/6/7/8 map to D-Desisive…B-Desisive) + Dervish loss penalty thresholds (16/24/32)

**Sub-tasks:**

4a. **Create `tables/victory_tables.ron`** with all three scenarios' victory data in a declarative format.

4b. **Optionally** add a test that deserializes the RON and asserts the Rust constants match — guarding against drift.

> **Priority:** Low. The data is small, stable, and unlikely to drift. This is mainly for completeness of the data-corpus.

---

## Task 5: Model the mine/chain boundary row

**Files:** `omdurman-rules/src/board.rs` (or `omdurman-types`), `omdurman-rules/src/lib.rs`

The rulebook says (§10.11/§10.21): mines and chain must be "south of the E–W hexrow in which the Khor Shambat empties into the Nile." This constraint is not modelled — the engine has `MinePlacement { hex }` but no validation of the row.

**Sub-tasks:**

5a. **Add a `NamedBoundary` or `MinePlacementLimit` concept** — either a const hex-row index derived from board annotations, or a method on `BoardInfo` like `mine_eligible_row() -> HexCoord`.

5b. **Add validation** in the mine-placement effect to reject hexes at or north of the boundary.

5c. **Also check**: does the Khor Shambat emptying point (the specific hex where Khor Shambat meets the Nile) already have a named annotation in the campaign board data? If so, use it. If not, annotate it.

**Note:** This is optional-rule-only (§10), so lower priority.

---

## Recommended order

1. **Task 2** (unannotated cells) — quick win if image is readable; no code changes needed.
2. **Task 1** (wire scattergram RON) — closes a real correctness gap; the RON already exists.
3. **Task 3** (scenario setup manifests) — improves maintainability; scenario_setup.rs already half-done.
4. **Task 5** (mine boundary) — needed only when mines are fully implemented.
5. **Task 4** (victory tables in RON) — nice-to-have for data completeness.

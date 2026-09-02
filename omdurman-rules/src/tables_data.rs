//! Rules tables (§tables-data): the four lookup tables the engine consults
//! — Combat Results, Range Effects, Howitzer Scattergram, Line of Sight —
//! embedded as Rust `static` constants below, transcribed from the RON
//! files under `Boardgame - Remember_Gordon/tables/`. The asset editor
//! (`tools/asset-editor`) edits those files offline; the `#[cfg(test)]`
//! parity tests at the bottom of this module parse the RON and fail the
//! build if it ever drifts from the constants.
//!
//! These used to be `include_str!`-embedded RON parsed at runtime behind a
//! `OnceLock` (see git history). The runtime parse was unmodelable for Kani
//! (CBMC would unroll the whole RON parser and UTF-8 validation on every
//! proof touching a table lookup) and added panic paths for malformed
//! tables. As `static`s the tables are plain data: lookups are
//! compile-time-bounds-checked array indexes, and Kani can prove properties
//! over the tables' full input domain (see the `verification` modules in
//! `range_effects`, `combat_results_table`, and `howitzer_scatter`).
//!
//! The RON files remain the authoring source of truth: edit them via the
//! asset editor, then update the constants here to match (the parity tests
//! enforce exactly that, cell by cell).

use crate::howitzer_scatter::ScatterHexDirection;
use crate::los_table::{BlockingRule, LosCondition, LosFeature, LosLevel};
use crate::{CombatResult, FireFactorRow, RangeBand, WeaponClass};

// ── Combat Results Table (§6.22) ─────────────────────────────────────────

/// The CRT as authored: `CRT[row.index()][roll.value() - 1]` — fire-factor
/// band → result per modified d10 roll. Transcribed from
/// `combat_results_table.ron`; every row covers rolls 1–10 by construction.
pub(crate) static CRT: [[CombatResult; 10]; 9] = [
    // Row01to05:  -  -  -  D  D  1  1  1  2  2
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
    ],
    // Row06to10:  -  -  D  D  1  1  1  2  2  2
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
    ],
    // Row11to15:  -  D  D  1  1  1  2  2  2  3
    [
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
    ],
    // Row16to20:  D  D  1  1  1  2  2  2  3  3
    [
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
    ],
    // Row21to25:  D  1  1  1  2  2  2  3  3  3
    [
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
    ],
    // Row26to30:  1  1  1  2  2  2  3  3  3  4
    [
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
    ],
    // Row31to35:  1  1  2  2  2  3  3  3  4  4
    [
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(4),
    ],
    // Row36to40:  1  2  2  2  3  3  3  4  4  4
    [
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(4),
    ],
    // Row41Plus:  2  2  2  3  3  3  4  4  4  5
    [
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(5),
    ],
];

// Ties the table's row count to `FireFactorRow::ALL` at compile time: adding
// a row without extending the CRT (or vice versa) fails to build.
const _: () = assert!(FireFactorRow::ALL.len() == CRT.len());

// ── Range Effects Table (§6.22) ──────────────────────────────────────────

/// The Anglo-Egyptian range-effects rows as authored: row
/// `AE_RANGE_EFFECTS[weapon.index()]` lists the fire multiplier band for hex
/// distances 1..=10 (index = distance − 1). Transcribed from
/// `range_effects_table.ron`. The `Melee` row is never consulted — spears
/// never reach the table (`range_effects::band_at` short-circuits, §2.31) —
/// and is `OutOfRange` throughout (the RON omits it).
pub(crate) static AE_RANGE_EFFECTS: [[RangeBand; 10]; 5] = [
    // Melee: (unauthored — never consulted; see above)
    [
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    // Rifles: 1: x2, 2-3: x1, 4-5: x1/2, 6-10: -
    [
        RangeBand::Doubled,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    // Maxims: identical to Rifles
    [
        RangeBand::Doubled,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    // Artillery: 1: x3, 2: x2, 3-6: x1, 7-8: x1/2, 9-10: -
    [
        RangeBand::Tripled,
        RangeBand::Doubled,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    // Howitzer: 1-3: -, 4-10: x1/2
    [
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::Halved,
    ],
];

/// The Dervish range-effects rows as authored, same layout as
/// [`AE_RANGE_EFFECTS`]. Transcribed from `range_effects_table.ron`.
pub(crate) static DERVISH_RANGE_EFFECTS: [[RangeBand; 10]; 5] = [
    // Melee ("Spears" on the printed table): 1: x1, 2-10: -
    [
        RangeBand::Normal,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    // Rifles: 1-2: x1, 3-4: x1/2, 5-10: -
    [
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    // Maxims (omitted from the archived txt; rifles pattern per §6.22)
    [
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    // Artillery: 1: x2, 2-4: x1, 5-7: x1/2, 8-10: -
    [
        RangeBand::Doubled,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    // Howitzer (omitted from the archived txt; rifles pattern per §6.22)
    [
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
];

const _: () = assert!(WeaponClass::ALL.len() == AE_RANGE_EFFECTS.len());
const _: () = assert!(WeaponClass::ALL.len() == DERVISH_RANGE_EFFECTS.len());

// ── Howitzer Scattergram (§6.64) ─────────────────────────────────────────

/// The scattergram as authored: `SCATTERGRAM[roll.value() - 1]` maps the
/// impact d10 roll to a scatter direction. Transcribed from
/// `howitzer_scattergram.ron`; rolls 7–10 are `Center` (on target).
pub(crate) static SCATTERGRAM: [ScatterHexDirection; 10] = [
    ScatterHexDirection::UpperLeft,
    ScatterHexDirection::UpperRight,
    ScatterHexDirection::Right,
    ScatterHexDirection::LowerRight,
    ScatterHexDirection::LowerLeft,
    ScatterHexDirection::Left,
    ScatterHexDirection::Center,
    ScatterHexDirection::Center,
    ScatterHexDirection::Center,
    ScatterHexDirection::Center,
];

// ── Line of Sight Table (§6.3) ───────────────────────────────────────────

/// A terrain name as it appears in the LOS table's level grouping.
/// [`omdurman_types::Terrain`] itself carries payloads (`road`, Nile current)
/// the authored table doesn't, so the table lists bare names and
/// [`crate::los_table::los_level`] normalises a [`omdurman_types::Terrain`]
/// into one of these before the lookup.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum LosTerrainName {
    Clear,
    Rough,
    Trees,
    Swamp,
    Nile,
    Hilltop,
    Huts,
    Building,
}

/// The terrain→LOS-level grouping (§6.3), scanned in order by
/// [`crate::los_table::los_level`]. Order matters: the first group listing a
/// name wins (the authored RON's `levels` map has the same Ground < Rough <
/// Hilltop order).
pub(crate) static LOS_LEVELS: [(LosLevel, &[LosTerrainName]); 3] = [
    (
        LosLevel::Ground,
        &[
            LosTerrainName::Clear,
            LosTerrainName::Swamp,
            LosTerrainName::Nile,
            LosTerrainName::Huts,
            LosTerrainName::Building,
        ],
    ),
    (LosLevel::Rough, &[LosTerrainName::Rough]),
    (LosLevel::Hilltop, &[LosTerrainName::Hilltop]),
];

/// The 3×3 blocking-rule grid (§6.3):
/// `LOS_CELLS[firer.index()][target.index()]` lists the features that block,
/// each with the positional conditions (Details footnotes) that must *all*
/// hold. Transcribed from `los_table.ron`.
pub(crate) static LOS_CELLS: [[&[BlockingRule]; 3]; 3] = [
    // Firer = Ground
    [
        // → Ground: Units, Huts(1), Wall, Rough, Trees(1)
        &[
            BlockingRule(LosFeature::Units, &[]),
            BlockingRule(LosFeature::Huts, &[LosCondition::MoreThanTwo]),
            BlockingRule(LosFeature::Wall, &[]),
            BlockingRule(LosFeature::RoughTerrain, &[]),
            BlockingRule(LosFeature::Trees, &[LosCondition::MoreThanTwo]),
        ],
        // → Rough: Units(3,6), Huts(1,3), Wall, Crest(2), Trees(1), Hilltop
        &[
            BlockingRule(
                LosFeature::Units,
                &[
                    LosCondition::CloserToFirer,
                    LosCondition::AdjSameLevelTarget,
                ],
            ),
            BlockingRule(
                LosFeature::Huts,
                &[LosCondition::MoreThanTwo, LosCondition::CloserToFirer],
            ),
            BlockingRule(LosFeature::Wall, &[]),
            BlockingRule(LosFeature::Crest, &[LosCondition::CrestAdjacency]),
            BlockingRule(LosFeature::Trees, &[LosCondition::MoreThanTwo]),
            BlockingRule(LosFeature::HilltopTerrain, &[]),
        ],
        // → Hilltop: Units(3), Huts(1,3), Crest(3), Hilltop
        &[
            BlockingRule(LosFeature::Units, &[LosCondition::CloserToFirer]),
            BlockingRule(
                LosFeature::Huts,
                &[LosCondition::MoreThanTwo, LosCondition::CloserToFirer],
            ),
            BlockingRule(LosFeature::Crest, &[LosCondition::CloserToFirer]),
            BlockingRule(LosFeature::HilltopTerrain, &[]),
        ],
    ],
    // Firer = Rough
    [
        // → Ground: Units(4,5), Huts(1,4), Wall, Crest(2), Trees(1), Hilltop
        &[
            BlockingRule(
                LosFeature::Units,
                &[
                    LosCondition::CloserToTarget,
                    LosCondition::AdjSameLevelFirer,
                ],
            ),
            BlockingRule(
                LosFeature::Huts,
                &[LosCondition::MoreThanTwo, LosCondition::CloserToTarget],
            ),
            BlockingRule(LosFeature::Wall, &[]),
            BlockingRule(LosFeature::Crest, &[LosCondition::CrestAdjacency]),
            BlockingRule(LosFeature::Trees, &[LosCondition::MoreThanTwo]),
            BlockingRule(LosFeature::HilltopTerrain, &[]),
        ],
        // → Rough: Units(7), Hilltop, Crest(2)
        &[
            BlockingRule(LosFeature::Units, &[LosCondition::NotAtLowerLevel]),
            BlockingRule(LosFeature::HilltopTerrain, &[]),
            BlockingRule(LosFeature::Crest, &[LosCondition::CrestAdjacency]),
        ],
        // → Hilltop: Units(3), Crest(2,3), Hilltop
        &[
            BlockingRule(LosFeature::Units, &[LosCondition::CloserToFirer]),
            BlockingRule(
                LosFeature::Crest,
                &[LosCondition::CrestAdjacency, LosCondition::CloserToFirer],
            ),
            BlockingRule(LosFeature::HilltopTerrain, &[]),
        ],
    ],
    // Firer = Hilltop
    [
        // → Ground: Units(3), Huts(1,4), Crest(4), Hilltop
        &[
            BlockingRule(LosFeature::Units, &[LosCondition::CloserToFirer]),
            BlockingRule(
                LosFeature::Huts,
                &[LosCondition::MoreThanTwo, LosCondition::CloserToTarget],
            ),
            BlockingRule(LosFeature::Crest, &[LosCondition::CloserToTarget]),
            BlockingRule(LosFeature::HilltopTerrain, &[]),
        ],
        // → Rough: Units(4), Hilltop, Crest(2,4)
        &[
            BlockingRule(LosFeature::Units, &[LosCondition::CloserToTarget]),
            BlockingRule(LosFeature::HilltopTerrain, &[]),
            BlockingRule(
                LosFeature::Crest,
                &[LosCondition::CrestAdjacency, LosCondition::CloserToTarget],
            ),
        ],
        // → Hilltop: Units, only at hilltop level (HilltopOnly)
        &[BlockingRule(
            LosFeature::Units,
            &[LosCondition::HilltopOnly],
        )],
    ],
];

// ── RON parity tests ─────────────────────────────────────────────────────
//
// The constants above are transcriptions of the authored RON files; these
// tests parse the RON (the asset editor's output format) and compare cell by
// cell, so editing a table in the asset editor without updating the constant
// fails `cargo test` with a precise mismatch. (The RON is *not* parsed by
// the engine at runtime any more — see the module docs.)

#[cfg(test)]
mod ron_parity {
    use super::*;
    use std::collections::BTreeMap;

    macro_rules! ron_text {
        ($file:literal) => {
            include_str!(concat!("../../Boardgame - Remember_Gordon/tables/", $file))
        };
    }

    fn parse<T: serde::de::DeserializeOwned>(file: &'static str, text: &'static str) -> T {
        ron::from_str(text).unwrap_or_else(|e| panic!("failed to parse rules table {file}: {e}"))
    }

    /// The range-effects table as authored by the asset editor.
    #[derive(serde::Deserialize)]
    struct RangeEffectsRon {
        #[serde(rename = "Dervish")]
        dervish: BTreeMap<WeaponClass, Vec<RangeBand>>,
        #[serde(rename = "AngloEgyptian")]
        anglo_egyptian: BTreeMap<WeaponClass, Vec<RangeBand>>,
    }

    /// The LOS table as authored (the `details`/`notes` prose maps are
    /// editor-only and ignored here).
    #[derive(serde::Deserialize)]
    #[allow(clippy::type_complexity)]
    struct LosRon {
        levels: BTreeMap<LosLevel, Vec<LosTerrainName>>,
        cells: BTreeMap<(LosLevel, LosLevel), Vec<(LosFeature, Vec<LosCondition>)>>,
    }

    #[test]
    fn crt_ron_matches_const() {
        let ron: BTreeMap<FireFactorRow, Vec<CombatResult>> = parse(
            "combat_results_table.ron",
            ron_text!("combat_results_table.ron"),
        );
        assert_eq!(ron.len(), 9, "nine fire-factor bands");
        for row in FireFactorRow::ALL {
            let cells = ron
                .get(&row)
                .unwrap_or_else(|| panic!("missing CRT row {row:?}"));
            assert_eq!(cells.len(), 10, "CRT row {row:?} must cover rolls 1-10");
            for (i, cell) in cells.iter().enumerate() {
                assert_eq!(
                    CRT[row.index()][i],
                    *cell,
                    "CRT mismatch: {row:?} roll {}: const {:?} vs ron {cell:?}",
                    i + 1,
                    CRT[row.index()][i]
                );
            }
        }
    }

    #[test]
    fn range_effects_ron_matches_consts() {
        let ron: RangeEffectsRon = parse(
            "range_effects_table.ron",
            ron_text!("range_effects_table.ron"),
        );
        for (ae, faction, name) in [
            (true, &ron.anglo_egyptian, "AngloEgyptian"),
            (false, &ron.dervish, "Dervish"),
        ] {
            let rows = if ae {
                &AE_RANGE_EFFECTS
            } else {
                &DERVISH_RANGE_EFFECTS
            };
            for weapon in WeaponClass::ALL {
                let authored = faction.get(&weapon);
                if let Some(cells) = authored {
                    assert!(
                        cells.len() >= 10,
                        "{name} {weapon:?} line covers only {} of 10 distances",
                        cells.len()
                    );
                }
                for d in 1usize..=10 {
                    let want = authored
                        .map(|cells| cells[d - 1])
                        // A weapon line missing from the RON is out of range
                        // at every distance (only AE Melee is omitted).
                        .unwrap_or(RangeBand::OutOfRange);
                    assert_eq!(
                        rows[weapon.index()][d - 1],
                        want,
                        "{name} {weapon:?} distance {d}: const {:?} vs ron {want:?}",
                        rows[weapon.index()][d - 1]
                    );
                }
            }
        }
        // The AE Melee omission the all-OutOfRange fallback above relies on.
        assert!(
            !ron.anglo_egyptian.contains_key(&WeaponClass::Melee),
            "AE Melee line appeared in the RON: give it a real const row"
        );
    }

    #[test]
    fn scattergram_ron_matches_const() {
        let ron: Vec<ScatterHexDirection> = parse(
            "howitzer_scattergram.ron",
            ron_text!("howitzer_scattergram.ron"),
        );
        assert_eq!(ron.len(), 10, "scattergram must cover impact rolls 1-10");
        assert_eq!(ron[6], ScatterHexDirection::Center);
        assert_eq!(ron[9], ScatterHexDirection::Center);
        for (i, dir) in ron.iter().enumerate() {
            assert_eq!(
                SCATTERGRAM[i],
                *dir,
                "scattergram mismatch: roll {} const {:?} vs ron {dir:?}",
                i + 1,
                SCATTERGRAM[i]
            );
        }
    }

    #[test]
    fn los_ron_matches_consts() {
        let ron: LosRon = parse("los_table.ron", ron_text!("los_table.ron"));
        assert_eq!(ron.cells.len(), 9, "3x3 level grid");
        assert!(ron.levels.contains_key(&LosLevel::Hilltop));

        // levels: the authored grouping, scanned in Ground < Rough < Hilltop
        // order, matches `LOS_LEVELS` exactly.
        for (level, names) in LOS_LEVELS {
            assert_eq!(
                ron.levels.get(&level).map(Vec::as_slice),
                Some(names),
                "level grouping mismatch for {level:?}"
            );
        }

        // cells: every (firer, target) feature+conditions pair matches.
        for firer in [LosLevel::Ground, LosLevel::Rough, LosLevel::Hilltop] {
            for target in [LosLevel::Ground, LosLevel::Rough, LosLevel::Hilltop] {
                let ron_cell = ron
                    .cells
                    .get(&(firer, target))
                    .unwrap_or_else(|| panic!("LOS cell ({firer:?}, {target:?}) missing"));
                let const_cell = LOS_CELLS[firer.index()][target.index()];
                assert_eq!(
                    const_cell.len(),
                    ron_cell.len(),
                    "cell ({firer:?},{target:?}) rule count mismatch"
                );
                for (i, (rule, (feature, conditions))) in
                    const_cell.iter().zip(ron_cell).enumerate()
                {
                    assert_eq!(
                        rule.0, *feature,
                        "cell ({firer:?},{target:?}) rule {i} feature mismatch"
                    );
                    assert_eq!(
                        rule.1, conditions,
                        "cell ({firer:?},{target:?}) rule {i} conditions mismatch"
                    );
                }
            }
        }
    }
}

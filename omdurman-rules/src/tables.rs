//! Lookup tables from the printed mapsheet and rulebook back cover.
//!
//! Every table is encoded as a pure function or a const array so that
//! the engine never needs an external data file to resolve combat.
//!
//! Tables implemented here:
//!
//! | Table | Source |
//! |-------|--------|
//! | [`AngloEgyptianCrt`] / [`DervishCrt`] | Combat Results Table (mapsheet) |
//! | [`AngloEgyptianRangeEffects`] / [`DervishRangeEffects`] | Range Effects Tables |
//! | [`TerrainEffectsChart`] | Terrain Effects Chart |
//! | [`LineOfSightTable`] | Line of Sight Table |
//! | [`HowitzerScattergram`] | Howitzer Fire Scattergram |
//! | [`TurnRecordTrack`] | Turn Record Track |

use crate::{
    CombatResult, DieModifier, DieRoll, FireFactor, HexDistance, MovementAllowance, RangeBand,
    WeaponClass,
};

// ---------------------------------------------------------------------------
// 1) Combat Results Tables (CRT)
// ---------------------------------------------------------------------------

/// Fire-factor row thresholds on the CRT.
///
/// The printed table groups fire factors into bands.  The band index is used
/// to index into the result matrix.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FireFactorRow {
    /// 1–5 factors
    Row01to05,
    /// 6–10 factors
    Row06to10,
    /// 11–15 factors
    Row11to15,
    /// 16–20 factors
    Row16to20,
    /// 21–25 factors
    Row21to25,
    /// 26–30 factors
    Row26to30,
    /// 31–35 factors
    Row31to35,
    /// 36–40 factors
    Row36to40,
    /// 41+ factors
    Row41Plus,
}

impl FireFactorRow {
    /// Determine which row a given total fire factor falls into.
    pub fn from_factor(f: FireFactor) -> Self {
        match f.0 {
            0..=5 => FireFactorRow::Row01to05,
            6..=10 => FireFactorRow::Row06to10,
            11..=15 => FireFactorRow::Row11to15,
            16..=20 => FireFactorRow::Row16to20,
            21..=25 => FireFactorRow::Row21to25,
            26..=30 => FireFactorRow::Row26to30,
            31..=35 => FireFactorRow::Row31to35,
            36..=40 => FireFactorRow::Row36to40,
            _ => FireFactorRow::Row41Plus,
        }
    }
}

/// Look up a result on the **Anglo-Egyptian** Combat Results Table.
///
/// Columns = modified die roll (1–10), rows = total fire factors.
///
/// — = `NoEffect`  
/// D = `Disrupt` (½ of target units, round up)  
/// 1…5 = `Eliminate(n)` (that many units removed)
pub fn anglo_egyptian_crt(row: FireFactorRow, roll: DieRoll) -> CombatResult {
    let r = roll.get() as usize;
    ANGLO_EGYPTIAN_CRT[row as usize][r - 1]
}

/// Look up a result on the **Dervish** Combat Results Table.
pub fn dervish_crt(row: FireFactorRow, roll: DieRoll) -> CombatResult {
    let r = roll.get() as usize;
    DERVISH_CRT[row as usize][r - 1]
}

type CrtRow = [CombatResult; 10];

/// Anglo-Egyptian CRT — these are substantially more lethal than the
/// Dervish table, reflecting the massive firepower advantage (Lee-Enfield
/// rifles, Maxim guns, modern artillery).
///
/// Columns: die roll 1 … 10.  Rows: fire-factor bands top → bottom.
const ANGLO_EGYPTIAN_CRT: [CrtRow; 9] = [
    //                        1    2    3    4    5    6    7    8    9   10
    /*  1–5  */
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
    ],
    /*  6–10 */
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
    ],
    /* 11–15 */
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
    ],
    /* 16–20 */
    [
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
    ],
    /* 21–25 */
    [
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
    ],
    /* 26–30 */
    [
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
    ],
    /* 31–35 */
    [
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(4),
    ],
    /* 36–40 */
    [
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(5),
    ],
    /* 41+   */
    [
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(5),
        CombatResult::Eliminate(5),
    ],
];

/// Dervish CRT — significantly less lethal; many no-effect and disrupt
/// results, few eliminations even at high firepower.
const DERVISH_CRT: [CrtRow; 9] = [
    //                        1    2    3    4    5    6    7    8    9   10
    /*  1–5  */
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
    ],
    /*  6–10 */
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
    ],
    /* 11–15 */
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
    ],
    /* 16–20 */
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
    ],
    /* 21–25 */
    [
        CombatResult::NoEffect,
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
    ],
    /* 26–30 */
    [
        CombatResult::NoEffect,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
    ],
    /* 31–35 */
    [
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
    ],
    /* 36–40 */
    [
        CombatResult::Disrupt,
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
    ],
    /* 41+   */
    [
        CombatResult::Disrupt,
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(1),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(2),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(3),
        CombatResult::Eliminate(4),
        CombatResult::Eliminate(4),
    ],
];

// ---------------------------------------------------------------------------
// 2) Range Effects Tables
// ---------------------------------------------------------------------------

/// A single row of a Range Effects Table: for each range (1 … 10+) the
/// multiplier that applies.
type RangeRow = [RangeBand; 11];

/// Anglo-Egyptian Range Effects Table.
///
/// Columns: range in hexes 1 … 11 (index 0 = range 1, index 10 = range 11+).
/// Source: printed mapsheet.
///
/// Note: `WeaponClass::Melee` units have no ranged fire — they must
/// adjacent-melee.  The row is included for completeness; callers should
/// treat any non-zero range as `OutOfRange`.
const AE_RANGE_TABLE: [RangeRow; 5] = [
    // WeaponClass index order: Melee=0, Rifles=1, Maxims=2, Artillery=3, Howitzer=4
    //                        R1       R2       R3       R4       R5       R6       R7       R8       R9       R10      R11+
    /* Melee     */
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
        RangeBand::OutOfRange,
    ],
    /* Rifles    */
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
        RangeBand::OutOfRange,
    ],
    /* Maxims    */
    [
        RangeBand::Doubled,
        RangeBand::Doubled,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    /* Artillery */
    [
        RangeBand::Tripled,
        RangeBand::Tripled,
        RangeBand::Doubled,
        RangeBand::Doubled,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    /* Howitzer  */
    [
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::OutOfRange,
    ],
];

/// Dervish Range Effects Table.
///
/// Generally shorter ranges and fewer doubled/tripled bands.
const DERVISH_RANGE_TABLE: [RangeRow; 5] = [
    //                        R1       R2       R3       R4       R5       R6       R7       R8       R9       R10      R11+
    /* Melee     */
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
        RangeBand::OutOfRange,
    ],
    /* Rifles    */
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
        RangeBand::OutOfRange,
    ],
    /* Maxims    */
    [
        RangeBand::Doubled,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    /* Artillery */
    [
        RangeBand::Doubled,
        RangeBand::Doubled,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Normal,
        RangeBand::Halved,
        RangeBand::Halved,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
        RangeBand::OutOfRange,
    ],
    /* Howitzer  */
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
        RangeBand::OutOfRange,
    ],
];

/// Look up the range band for a given weapon class and range.
///
/// `range` is in hexes (1‑based).
pub fn ae_range_effects(weapon: WeaponClass, range: HexDistance) -> RangeBand {
    let col = (range.0 as usize).min(10).saturating_sub(1);
    AE_RANGE_TABLE[weapon as usize][col]
}

/// Look up the range band for a Dervish weapon.
pub fn dervish_range_effects(weapon: WeaponClass, range: HexDistance) -> RangeBand {
    let col = (range.0 as usize).min(10).saturating_sub(1);
    DERVISH_RANGE_TABLE[weapon as usize][col]
}

// ---------------------------------------------------------------------------
// 3) Terrain Effects Chart
// ---------------------------------------------------------------------------

/// Terrain types that appear in the Terrain Effects Chart.
///
/// Mapped from the hex terrain in [`omdurman_types::Terrain`] plus the
/// hexside kinds that affect movement/combat.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TerrainType {
    Clear,
    Rough,
    PalmGrove,
    Village,
    Khartoum,
    Nile,
    Fort,
    WalledCity,
    Hut,
    Khor,
    Building,
    Crest,
    Hilltop,
}

impl TerrainType {
    /// Convert from the game-map [`omdurman_types::Terrain`] enum.
    /// Falls back to `Clear` for unknown terrain.
    pub fn from_terrain(t: omdurman_types::Terrain) -> Self {
        use omdurman_types::Terrain;
        match t {
            Terrain::Desert => TerrainType::Clear,
            Terrain::Shrubs => TerrainType::Rough,
            Terrain::Palm => TerrainType::PalmGrove,
            Terrain::BlueNile | Terrain::WhiteNile | Terrain::RiverNile => TerrainType::Nile,
            Terrain::Khartoum => TerrainType::Khartoum,
            Terrain::Tuti | Terrain::Hogali | Terrain::Buri => TerrainType::Village,
            Terrain::Fortress | Terrain::FortBuri | Terrain::FortMakran | Terrain::NorthFort => {
                TerrainType::Fort
            }
            Terrain::Rough => TerrainType::Rough,
            Terrain::Hilltop => TerrainType::Hilltop,
            Terrain::Crest => TerrainType::Crest,
            Terrain::Hut => TerrainType::Hut,
            Terrain::Building => TerrainType::Building,
            // Tree cover behaves like a palm grove (LOS + going); swamp is
            // difficult going, treated as Rough until it has its own chart row.
            Terrain::Trees => TerrainType::PalmGrove,
            Terrain::Swamp => TerrainType::Rough,
            // Named villages (hut clusters).
            Terrain::ShambatVillage
            | Terrain::HalfayaVillage
            | Terrain::ElDebebaVillage
            | Terrain::ElEgeigaVillage
            | Terrain::AbuAlimVillage
            | Terrain::KerreriVillage => TerrainType::Village,
            // Named buildings/points; Makran Point is a fort.
            Terrain::MakranPoint => TerrainType::Fort,
            Terrain::Treasury | Terrain::Grounds => TerrainType::Building,
            // The Zariba defensive perimeter — a building-grade defensive hex.
            Terrain::Zariba => TerrainType::Building,
            // Map-legend code terrains — no special effect yet; treat as Clear.
            Terrain::Y | Terrain::K | Terrain::S | Terrain::O | Terrain::D | Terrain::A => {
                TerrainType::Clear
            }
            // Key objectives — building-grade defensive hexes.
            Terrain::MahdisTomb | Terrain::Palace => TerrainType::Building,
        }
    }
}

/// A single entry in the Terrain Effects Chart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TerrainEntry {
    /// Additional movement points to enter a hex of this terrain
    /// (beyond the 1 MP base cost for clear terrain).
    pub movement_cost: MovementAllowance,
    /// Die-roll modifier for fire attacks targeting units in this terrain
    /// (negative = defender advantage).
    pub defense_modifier: DieModifier,
}

/// Terrain Effects Chart — maps each terrain type to its movement cost
/// and defensive die-roll modifier.
///
/// Source: printed Terrain Effects Chart on the mapsheet.
fn terrain_effects_chart(terrain: TerrainType) -> TerrainEntry {
    match terrain {
        TerrainType::Clear => TerrainEntry {
            movement_cost: MovementAllowance(1),
            defense_modifier: DieModifier(0),
        },
        TerrainType::Rough => TerrainEntry {
            movement_cost: MovementAllowance(2),
            defense_modifier: DieModifier(-1),
        },
        TerrainType::PalmGrove => TerrainEntry {
            movement_cost: MovementAllowance(2),
            defense_modifier: DieModifier(-2),
        },
        TerrainType::Village => TerrainEntry {
            movement_cost: MovementAllowance(1),
            defense_modifier: DieModifier(-2),
        },
        TerrainType::Khartoum => TerrainEntry {
            movement_cost: MovementAllowance(1),
            defense_modifier: DieModifier(-3),
        },
        TerrainType::Nile => TerrainEntry {
            movement_cost: MovementAllowance(u16::MAX),
            defense_modifier: DieModifier(0),
        },
        TerrainType::Fort => TerrainEntry {
            movement_cost: MovementAllowance(1),
            defense_modifier: DieModifier(-3),
        },
        TerrainType::WalledCity => TerrainEntry {
            movement_cost: MovementAllowance(1),
            defense_modifier: DieModifier(-3),
        },
        TerrainType::Hut => TerrainEntry {
            movement_cost: MovementAllowance(1),
            defense_modifier: DieModifier(-1),
        },
        TerrainType::Khor => TerrainEntry {
            movement_cost: MovementAllowance(3),
            defense_modifier: DieModifier(0),
        },
        TerrainType::Building => TerrainEntry {
            movement_cost: MovementAllowance(1),
            defense_modifier: DieModifier(-2),
        },
        TerrainType::Crest => TerrainEntry {
            movement_cost: MovementAllowance(1),
            defense_modifier: DieModifier(-1),
        },
        TerrainType::Hilltop => TerrainEntry {
            movement_cost: MovementAllowance(2),
            defense_modifier: DieModifier(-2),
        },
    }
}

/// Convenience: get the defense modifier for a terrain type.
pub fn defense_modifier(terrain: TerrainType) -> DieModifier {
    terrain_effects_chart(terrain).defense_modifier
}

/// Convenience: get the movement cost for a terrain type.
pub fn movement_cost(terrain: TerrainType) -> MovementAllowance {
    terrain_effects_chart(terrain).movement_cost
}

/// Movement cost to enter a hex, accounting for a road overlay. A road costs a
/// flat 1 MP regardless of the underlying terrain (Terrain Effects Chart, Road
/// row: "1"); without a road it's the terrain's own cost. The road is a
/// movement overlay only — combat/LOS still use the underlying terrain.
pub fn movement_cost_with_road(terrain: TerrainType, road: bool) -> MovementAllowance {
    if road {
        MovementAllowance(1)
    } else {
        movement_cost(terrain)
    }
}

// ---------------------------------------------------------------------------
// 4) Line of Sight (LOS) Table
// ---------------------------------------------------------------------------

/// Terrain type of the *firing* unit's hex for LOS purposes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LosFirerTerrain {
    Ground,
    Rough,
    Hilltop,
}

/// Terrain type of the *target* unit's hex for LOS purposes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LosTargetTerrain {
    Ground,
    /// Units in the hex (including friendly — LOS is blocked if the
    /// intervening hex contains units per LOS note 3, 6, 7).
    Units,
    Huts,
    /// Wall hexside between firer and target.
    Wall,
    Trees,
    Crest,
    Rough,
    Hilltop,
}

/// Whether LOS is blocked.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LosResult {
    Clear,
    Blocked,
}

/// Line of Sight Table (rulebook §6.3, back cover).
///
/// Cross-index the firing unit's terrain with the target unit's terrain.
/// If the cell says "Blocks", LOS is blocked; otherwise it is clear
/// (subject to the special notes below).
pub fn los_table(firer: LosFirerTerrain, target: LosTargetTerrain) -> LosResult {
    use LosFirerTerrain as F;
    use LosResult::*;
    use LosTargetTerrain as T;

    match (firer, target) {
        // --- GROUND firer ---
        (F::Ground, T::Ground) => Clear,
        (F::Ground, T::Units) => Clear,
        (F::Ground, T::Huts) => Blocked,
        (F::Ground, T::Wall) => Blocked,
        (F::Ground, T::Trees) => Clear,
        (F::Ground, T::Crest) => Clear,
        (F::Ground, T::Rough) => Clear,
        (F::Ground, T::Hilltop) => Clear,

        // --- ROUGH firer ---
        (F::Rough, T::Ground) => Clear,
        (F::Rough, T::Units) => Blocked,
        (F::Rough, T::Huts) => Blocked,
        (F::Rough, T::Wall) => Blocked,
        (F::Rough, T::Trees) => Clear,
        (F::Rough, T::Crest) => Clear,
        (F::Rough, T::Rough) => Clear,
        (F::Rough, T::Hilltop) => Clear,

        // --- HILLTOP firer ---
        (F::Hilltop, T::Ground) => Clear,
        (F::Hilltop, T::Units) => Clear,
        (F::Hilltop, T::Huts) => Blocked,
        (F::Hilltop, T::Wall) => Blocked,
        (F::Hilltop, T::Trees) => Clear,
        (F::Hilltop, T::Crest) => Clear,
        (F::Hilltop, T::Rough) => Clear,
        (F::Hilltop, T::Hilltop) => Clear,
    }
}

/// Special LOS notes from the rulebook (§6.3, back cover).
///
/// 1. Units may not fire through more than two hexes of intervening
///    palm trees / huts.
/// 2. LOS not blocked if firing unit *and* target are on Hilltop,
///    provided the LOS segment does not cross a Crest hexside.
/// 3. LOS is blocked if the hex *behind* the target (relative to the
///    firer) contains friendly units.
/// 4. Rough terrain may block LOS if the firer is also in Rough
///    and the range is > 6 hexes.
/// 5. Wall hexsides block LOS unless the hexside is a Gate or Breach.
/// 6. Units in Huts may be seen only by units adjacent to the hut hex.
/// 7. Crest hexsides block LOS unless the firer is on the higher side
///    of the crest.
pub enum LosSpecialNote {
    MaxTwoTreeHutHexes,
    HilltopToHilltop,
    BlockedByFriendlyBehindTarget,
    RoughBeyondSix,
    WallBlock,
    HutsAdjacentOnly,
    CrestBlock,
}

// ---------------------------------------------------------------------------
// 5) Howitzer Fire Scattergram
// ---------------------------------------------------------------------------

/// Scatter direction for howitzer fire.
///
/// The Scattergram on the mapsheet shows which hex a howitzer round
/// scatters into for each impact die roll (1–10), given the current
/// facing of the firing gunboat (upstream/downstream).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScatterDirection {
    /// Hit the target hex (roll 7–10).
    OnTarget,
    /// Scatter one hex north (upstream along the Nile).
    North,
    /// Scatter one hex south (downstream along the Nile).
    South,
    /// Scatter one hex east (toward the east bank).
    East,
    /// Scatter one hex west (toward the west bank).
    West,
    /// Scatter one hex northeast.
    NorthEast,
    /// Scatter one hex northwest.
    NorthWest,
    /// Scatter one hex southeast.
    SouthEast,
    /// Scatter one hex southwest.
    SouthWest,
}

/// Howitzer scatter result.
pub struct HowitzerScatter {
    pub direction: ScatterDirection,
}

/// Resolve howitzer fire scatter (§6.64).
///
/// The first die roll is the CRT roll (handled by [`anglo_egyptian_crt`]).
/// This function determines the *impact hex* from the second die roll:
///
/// | Roll | Result |
/// |------|--------|
/// | 7–10 | On target |
/// | 5–6  | Short (downstream) |
/// | 3–4  | Long (upstream) |
/// | 1–2  | Left/right scatter |
pub fn howitzer_scatter(impact_roll: DieRoll) -> HowitzerScatter {
    let direction = match impact_roll.get() {
        7..=10 => ScatterDirection::OnTarget,
        5..=6 => ScatterDirection::South,
        3..=4 => ScatterDirection::North,
        1..=2 => ScatterDirection::West,
        _ => ScatterDirection::OnTarget,
    };
    HowitzerScatter { direction }
}

// ---------------------------------------------------------------------------
// 6) Turn Record Track
// ---------------------------------------------------------------------------

/// A single entry on the Turn Record Track.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TurnEntry {
    /// 1‑based turn number.
    pub turn: u8,
    /// Wall-clock time.
    pub time: &'static str,
    /// Day or night.
    pub day_night: crate::DayNight,
    /// Any special event on this turn.
    pub event: TurnEvent,
}

/// Special events that occur on specific turns.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TurnEvent {
    None,
    /// Dervish desertion roll (§8.2) — occurs on the first night turn.
    DervishDesertion,
    /// Dervish reinforcements are available.
    DervishReinforcements,
    /// Anglo-Egyptian reinforcements are available.
    AngloEgyptianReinforcements,
}

/// Campaign Game Turn Record Track (§9.12 — 22 turns, Sept 1 6:00 am
/// through Sept 3 8:00 am).
///
/// Turns 1–4 are day turns on Sept 1, then night turns alternate with
/// day turns on Sept 2–3 per the printed track.
const CAMPAIGN_TURN_TRACK: [TurnEntry; 22] = [
    //  Sept 1
    TurnEntry {
        turn: 1,
        time: "6:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 2,
        time: "8:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 3,
        time: "10:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 4,
        time: "12:00 pm",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 5,
        time: "2:00 pm",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 6,
        time: "4:00 pm",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 7,
        time: "6:00 pm",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 8,
        time: "8:00 pm",
        day_night: crate::DayNight::Night,
        event: TurnEvent::DervishDesertion,
    },
    TurnEntry {
        turn: 9,
        time: "10:00 pm",
        day_night: crate::DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 10,
        time: "12:00 am",
        day_night: crate::DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 11,
        time: "2:00 am",
        day_night: crate::DayNight::Night,
        event: TurnEvent::None,
    },
    //  Sept 2
    TurnEntry {
        turn: 12,
        time: "4:00 am",
        day_night: crate::DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 13,
        time: "6:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 14,
        time: "8:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 15,
        time: "10:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 16,
        time: "12:00 pm",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 17,
        time: "2:00 pm",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 18,
        time: "4:00 pm",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 19,
        time: "6:00 pm",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 20,
        time: "8:00 pm",
        day_night: crate::DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 21,
        time: "6:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    //  Sept 3
    TurnEntry {
        turn: 22,
        time: "8:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
];

/// Get the turn entry for a given 1‑based turn index (campaign game).
pub fn campaign_turn(turn: u8) -> Option<&'static TurnEntry> {
    CAMPAIGN_TURN_TRACK.get((turn as usize).saturating_sub(1))
}

/// Historical scenario track (§9.22 — 4 turns, Sept 2 6:00 am → 12:00 pm).
const HISTORICAL_TURN_TRACK: [TurnEntry; 4] = [
    TurnEntry {
        turn: 1,
        time: "6:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 2,
        time: "8:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 3,
        time: "10:00 am",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 4,
        time: "12:00 pm",
        day_night: crate::DayNight::Day,
        event: TurnEvent::None,
    },
];

pub fn historical_turn(turn: u8) -> Option<&'static TurnEntry> {
    HISTORICAL_TURN_TRACK.get((turn as usize).saturating_sub(1))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- CRT smoke tests ---

    #[test]
    fn ae_crt_lowest_is_no_effect() {
        let result = anglo_egyptian_crt(FireFactorRow::Row01to05, DieRoll::new(1));
        assert_eq!(result, CombatResult::NoEffect);
    }

    #[test]
    fn ae_crt_highest_is_eliminate_5() {
        let result = anglo_egyptian_crt(FireFactorRow::Row41Plus, DieRoll::new(10));
        assert_eq!(result, CombatResult::Eliminate(5));
    }

    #[test]
    fn ae_crt_progresses_with_roll() {
        let r1 = anglo_egyptian_crt(FireFactorRow::Row16to20, DieRoll::new(1));
        let r10 = anglo_egyptian_crt(FireFactorRow::Row16to20, DieRoll::new(10));
        assert!(r1 != CombatResult::Eliminate(3));
        assert_eq!(r10, CombatResult::Eliminate(3));
    }

    #[test]
    fn ae_crt_progresses_with_factor() {
        let low = anglo_egyptian_crt(FireFactorRow::Row01to05, DieRoll::new(8));
        let high = anglo_egyptian_crt(FireFactorRow::Row41Plus, DieRoll::new(8));
        assert!(low != high);
        assert_eq!(high, CombatResult::Eliminate(4));
    }

    #[test]
    fn dervish_crt_less_lethal_than_ae() {
        let d = dervish_crt(FireFactorRow::Row21to25, DieRoll::new(8));
        let ae = anglo_egyptian_crt(FireFactorRow::Row21to25, DieRoll::new(8));
        assert!(d != ae);
    }

    #[test]
    fn fire_factor_row_boundaries() {
        assert_eq!(
            FireFactorRow::from_factor(FireFactor(0)),
            FireFactorRow::Row01to05
        );
        assert_eq!(
            FireFactorRow::from_factor(FireFactor(5)),
            FireFactorRow::Row01to05
        );
        assert_eq!(
            FireFactorRow::from_factor(FireFactor(6)),
            FireFactorRow::Row06to10
        );
        assert_eq!(
            FireFactorRow::from_factor(FireFactor(15)),
            FireFactorRow::Row11to15
        );
        assert_eq!(
            FireFactorRow::from_factor(FireFactor(41)),
            FireFactorRow::Row41Plus
        );
        assert_eq!(
            FireFactorRow::from_factor(FireFactor(999)),
            FireFactorRow::Row41Plus
        );
    }

    // --- Range Effects ---

    #[test]
    fn ae_rifles_doubled_at_range_1() {
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance(1)),
            RangeBand::Doubled
        );
    }

    #[test]
    fn ae_rifles_normal_at_range_4() {
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance(4)),
            RangeBand::Normal
        );
    }

    #[test]
    fn ae_howitzer_min_range_4() {
        assert_eq!(
            ae_range_effects(WeaponClass::Howitzer, HexDistance(1)),
            RangeBand::OutOfRange
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Howitzer, HexDistance(4)),
            RangeBand::Normal
        );
    }

    #[test]
    fn dervish_rifles_shorter_range() {
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance(5)),
            RangeBand::OutOfRange
        );
    }

    #[test]
    fn melee_only_range_1() {
        assert_eq!(
            ae_range_effects(WeaponClass::Melee, HexDistance(1)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Melee, HexDistance(2)),
            RangeBand::OutOfRange
        );
    }

    // --- Terrain Effects ---

    #[test]
    fn clear_terrain_no_bonus() {
        assert_eq!(defense_modifier(TerrainType::Clear), DieModifier(0));
    }

    #[test]
    fn fort_gives_minus_3() {
        assert_eq!(defense_modifier(TerrainType::Fort), DieModifier(-3));
    }

    #[test]
    fn palm_grove_gives_minus_2() {
        assert_eq!(defense_modifier(TerrainType::PalmGrove), DieModifier(-2));
    }

    #[test]
    fn nile_is_impassable() {
        let e = terrain_effects_chart(TerrainType::Nile);
        assert!(e.movement_cost.0 >= u16::MAX / 2);
    }

    // --- LOS ---

    #[test]
    fn los_wall_blocks() {
        assert_eq!(
            los_table(LosFirerTerrain::Ground, LosTargetTerrain::Wall),
            LosResult::Blocked
        );
    }

    #[test]
    fn los_ground_to_ground_clear() {
        assert_eq!(
            los_table(LosFirerTerrain::Ground, LosTargetTerrain::Ground),
            LosResult::Clear
        );
    }

    #[test]
    fn los_hilltop_to_huts_blocked() {
        assert_eq!(
            los_table(LosFirerTerrain::Hilltop, LosTargetTerrain::Huts),
            LosResult::Blocked
        );
    }

    // --- Howitzer ---

    #[test]
    fn howitzer_on_target_7_to_10() {
        for roll in 7..=10 {
            assert_eq!(
                howitzer_scatter(DieRoll::new(roll)).direction,
                ScatterDirection::OnTarget
            );
        }
    }

    #[test]
    fn howitzer_scatters_below_7() {
        for roll in 1..=6 {
            assert_ne!(
                howitzer_scatter(DieRoll::new(roll)).direction,
                ScatterDirection::OnTarget
            );
        }
    }

    // --- Turn Track ---

    #[test]
    fn campaign_track_22_turns() {
        assert!(campaign_turn(1).is_some());
        assert!(campaign_turn(22).is_some());
        assert!(campaign_turn(23).is_none());
    }

    #[test]
    fn desertion_on_first_night() {
        let t = campaign_turn(8).unwrap();
        assert_eq!(t.day_night, crate::DayNight::Night);
        assert_eq!(t.event, TurnEvent::DervishDesertion);
    }
}

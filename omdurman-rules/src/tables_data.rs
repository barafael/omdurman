//! Rules tables (§tables-data): the four lookup tables the engine consults
//! — Combat Results, Range Effects, Howitzer Scattergram, Line of Sight —
//! as RON files under `Boardgame - Remember_Gordon/tables/`, embedded at
//! compile time and parsed once on first use (same pattern as
//! [`crate::board_data`]). The asset editor (`tools/asset-editor`) edits
//! those files offline; the engine consumes them through the accessors in
//! the table modules.
//!
//! The `include_str!` paths reach across workspace members on purpose: the
//! canonical data lives next to the archived game materials, and path crates
//! in this workspace are never published, so the cross-crate reach is
//! confined to the repository.

use std::sync::OnceLock;

use crate::howitzer_scatter::ScatterHexDirection;

/// Embed + parse a rules table once. A corrupt table file is an authoring
/// error surfaced by the asset editor's save path; fail loud rather than
/// running on a silently wrong table.
pub(crate) fn load<T: serde::de::DeserializeOwned>(file: &'static str, text: &str) -> T {
    match ron::from_str(text) {
        Ok(table) => table,
        Err(e) => panic!("failed to parse rules table {file}: {e}"),
    }
}

/// The shared `OnceLock` plumbing per table: declares a static, an accessor,
/// and the embedded source text.
macro_rules! rules_table {
    ($file:literal, $ty:ty, $accessor:ident, $static:ident) => {
        static $static: OnceLock<$ty> = OnceLock::new();

        #[doc = concat!("The `", stringify!($file), "` table, parsed once on first use.")]
        pub(crate) fn $accessor() -> &'static $ty {
            $static.get_or_init(|| {
                load(
                    $file,
                    include_str!(concat!("../../Boardgame - Remember_Gordon/tables/", $file)),
                )
            })
        }
    };
}

// ── Combat Results Table (§6.22) ─────────────────────────────────────────

/// The CRT as authored: fire-factor band → result per modified d10 roll
/// (array index = roll − 1).
pub(crate) type CrtTable =
    std::collections::HashMap<crate::FireFactorRow, Vec<crate::CombatResult>>;

rules_table!("combat_results_table.ron", CrtTable, crt_table, CRT);

// ── Range Effects Table (§6.22) ──────────────────────────────────────────

/// The range-effects table as authored: per faction, weapon class → fire
/// multiplier band per hex distance 1..=10 (array index = distance − 1).
#[derive(serde::Deserialize, Clone, Debug)]
pub(crate) struct RangeEffectsTable {
    #[serde(rename = "Dervish")]
    pub dervish: std::collections::HashMap<crate::WeaponClass, Vec<crate::RangeBand>>,
    #[serde(rename = "AngloEgyptian")]
    pub anglo_egyptian: std::collections::HashMap<crate::WeaponClass, Vec<crate::RangeBand>>,
}

rules_table!(
    "range_effects_table.ron",
    RangeEffectsTable,
    range_effects_data,
    RANGE_EFFECTS
);

// ── Howitzer Scattergram (§6.64) ─────────────────────────────────────────

rules_table!(
    "howitzer_scattergram.ron",
    Vec<ScatterHexDirection>,
    scattergram_table,
    SCATTERGRAM
);

// ── Line of Sight Table (§6.3) ───────────────────────────────────────────

/// A terrain name as it appears in the LOS table's level grouping. [`Terrain`]
/// itself carries payloads (`road`, Nile current) the authored table doesn't,
/// so the table lists bare names and [`crate::los_table::los_level`]
/// normalises a [`Terrain`] into one of these before the lookup.
#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

/// One row of the LOS table lives next to the types it references: see
/// [`crate::los_table::BlockingRule`]. The LOS table as authored is the
/// terrain→level grouping plus the blocking rules per (firer level, target
/// level) cell.
#[derive(serde::Deserialize, Clone, Debug)]
pub(crate) struct LosTable {
    pub levels: std::collections::HashMap<crate::los_table::LosLevel, Vec<LosTerrainName>>,
    pub cells: std::collections::HashMap<
        (crate::los_table::LosLevel, crate::los_table::LosLevel),
        Vec<crate::los_table::BlockingRule>,
    >,
}

rules_table!("los_table.ron", LosTable, los_table_data, LOS);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FireFactorRow;
    use crate::los_table::LosLevel;

    /// All four tables parse as-authored and have the expected shape.
    #[test]
    fn tables_parse_and_have_expected_shape() {
        let crt = crt_table();
        assert_eq!(crt.len(), 9, "nine fire-factor bands");
        for row in FireFactorRow::ALL {
            assert!(crt.contains_key(&row), "missing CRT row {row:?}");
        }

        let range = range_effects_data();
        for (name, faction) in [("Dervish", &range.dervish), ("AE", &range.anglo_egyptian)] {
            assert!(
                faction.contains_key(&crate::WeaponClass::Rifles),
                "{name} rifles line"
            );
            assert!(
                faction.contains_key(&crate::WeaponClass::Artillery),
                "{name} artillery line"
            );
        }

        let scatter = scattergram_table();
        assert_eq!(scatter[6], ScatterHexDirection::Center);
        assert_eq!(scatter[9], ScatterHexDirection::Center);

        let los = los_table_data();
        assert_eq!(los.cells.len(), 9, "3x3 level grid");
        assert!(los.levels.contains_key(&LosLevel::Hilltop));
    }
}

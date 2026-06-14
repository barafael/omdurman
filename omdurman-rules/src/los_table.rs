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

#[cfg(test)]
mod tests {
    use super::*;

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
}

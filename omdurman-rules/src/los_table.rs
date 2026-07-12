/// Terrain type of the *firing* unit's hex for LOS purposes (rulebook §6.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LosFirerTerrain {
    Ground,
    Rough,
    Hilltop,
}

/// Terrain type of the *target* unit's hex for LOS purposes (rulebook §6.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LosTargetTerrain {
    Ground,
    /// Units in the hex (including friendly -- LOS is blocked if the
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

/// Whether LOS is blocked (rulebook §6.3).
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

/// Whether the firer at `from` has line of sight to `to` (rulebook §6.21,
/// §6.3).
///
/// Howitzer fire ignores LOS entirely (§6.64), so it is always permitted.
///
/// Blocked by:
/// * a wall or crest **hexside** crossed along the line (gates/breaches pass);
/// * a built-up **intervening hex** (hut/building — §6.3);
/// * more than two intervening palm-grove hexes (§6.3 note 1).
///
/// A firer on a Hilltop sees over intervening *terrain* (§6.3 note 2), but
/// wall/crest hexsides still block.
///
/// Uses the terrain and hexside data from [`BoardInfo`] (which lives inside
/// [`GameState`](crate::effects::GameState)) so the engine can validate LOS
/// without reaching into the app/Bevy layer.
pub fn has_los(
    board: &crate::board::BoardInfo,
    from: omdurman_types::HexCoord,
    to: omdurman_types::HexCoord,
    kind: crate::FireKind,
) -> bool {
    use crate::FireKind;
    use omdurman_types::Terrain;

    if kind == FireKind::Howitzer {
        return true;
    }

    let firer_on_hilltop = board
        .terrain_at(from)
        .is_some_and(|t| t == Terrain::Hilltop);

    // Full hex sequence from firer to target; edges are crossed between
    // consecutive hexes.
    let mut path = vec![from];
    path.extend(from.line_between(to));
    path.push(to);

    let mut trees = 0u32;
    for window in path.windows(2) {
        let (a, b) = (window[0], window[1]);
        // Hexside blocking applies regardless of hilltop.
        if board.hexside_between(a, b).is_some_and(|s| s.blocks_los()) {
            return false;
        }
        // Intervening-hex terrain blocking (skip the endpoints; only the hex
        // we're *entering* and which isn't the target counts as intervening).
        if b != to {
            let Some(terrain) = board.terrain_at(b) else {
                continue;
            };
            if firer_on_hilltop {
                continue; // sees over terrain (but not hexsides, handled above)
            }
            if terrain.blocks_los() {
                return false;
            }
            if terrain.is_los_trees() {
                trees += 1;
                if trees > 2 {
                    return false;
                }
            }
        }
    }
    true
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

    // §6.3
    #[test]
    fn los_wall_blocks() {
        assert_eq!(
            los_table(LosFirerTerrain::Ground, LosTargetTerrain::Wall),
            LosResult::Blocked
        );
    }

    // §6.3
    #[test]
    fn los_ground_to_ground_clear() {
        assert_eq!(
            los_table(LosFirerTerrain::Ground, LosTargetTerrain::Ground),
            LosResult::Clear
        );
    }

    // §6.3
    #[test]
    fn los_hilltop_to_huts_blocked() {
        assert_eq!(
            los_table(LosFirerTerrain::Hilltop, LosTargetTerrain::Huts),
            LosResult::Blocked
        );
    }

    // §6.3
    #[test]
    fn ground_firer_covers_all_targets() {
        use LosFirerTerrain::Ground;
        use LosTargetTerrain as T;
        // Ground → Ground = Clear
        assert_eq!(los_table(Ground, T::Ground), LosResult::Clear);
        assert_eq!(los_table(Ground, T::Units), LosResult::Clear);
        assert_eq!(los_table(Ground, T::Huts), LosResult::Blocked);
        assert_eq!(los_table(Ground, T::Wall), LosResult::Blocked);
        assert_eq!(los_table(Ground, T::Trees), LosResult::Clear);
        assert_eq!(los_table(Ground, T::Crest), LosResult::Clear);
        assert_eq!(los_table(Ground, T::Rough), LosResult::Clear);
        assert_eq!(los_table(Ground, T::Hilltop), LosResult::Clear);
    }

    // §6.3
    #[test]
    fn rough_firer_covers_all_targets() {
        use LosFirerTerrain::Rough;
        use LosTargetTerrain as T;
        assert_eq!(los_table(Rough, T::Ground), LosResult::Clear);
        assert_eq!(los_table(Rough, T::Units), LosResult::Blocked);
        assert_eq!(los_table(Rough, T::Huts), LosResult::Blocked);
        assert_eq!(los_table(Rough, T::Wall), LosResult::Blocked);
        assert_eq!(los_table(Rough, T::Trees), LosResult::Clear);
        assert_eq!(los_table(Rough, T::Crest), LosResult::Clear);
        assert_eq!(los_table(Rough, T::Rough), LosResult::Clear);
        assert_eq!(los_table(Rough, T::Hilltop), LosResult::Clear);
    }

    // §6.3
    #[test]
    fn hilltop_firer_covers_all_targets() {
        use LosFirerTerrain::Hilltop;
        use LosTargetTerrain as T;
        assert_eq!(los_table(Hilltop, T::Ground), LosResult::Clear);
        assert_eq!(los_table(Hilltop, T::Units), LosResult::Clear);
        assert_eq!(los_table(Hilltop, T::Huts), LosResult::Blocked);
        assert_eq!(los_table(Hilltop, T::Wall), LosResult::Blocked);
        assert_eq!(los_table(Hilltop, T::Trees), LosResult::Clear);
        assert_eq!(los_table(Hilltop, T::Crest), LosResult::Clear);
        assert_eq!(los_table(Hilltop, T::Rough), LosResult::Clear);
        assert_eq!(los_table(Hilltop, T::Hilltop), LosResult::Clear);
    }

    // §6.3
    #[test]
    fn all_24_cells_exercised() {
        // Exhaustive: ensure every (firer, target) pair returns Clear or Blocked
        // (no panic / unreachable).  The compiler will warn if we miss a variant.
        use LosFirerTerrain as F;
        use LosTargetTerrain as T;
        for firer in [F::Ground, F::Rough, F::Hilltop] {
            for target in [
                T::Ground,
                T::Units,
                T::Huts,
                T::Wall,
                T::Trees,
                T::Crest,
                T::Rough,
                T::Hilltop,
            ] {
                let _ = los_table(firer, target);
            }
        }
    }

    // §6.3 -- exhaustive cell-by-cell verification against los_table.txt
    #[test]
    fn los_every_cell_matches_the_table() {
        use LosFirerTerrain as F;
        use LosResult::*;
        use LosTargetTerrain as T;

        // Ground firer: Clear, Clear, Blocked, Blocked, Clear, Clear, Clear, Clear
        assert_eq!(los_table(F::Ground, T::Ground), Clear);
        assert_eq!(los_table(F::Ground, T::Units), Clear);
        assert_eq!(los_table(F::Ground, T::Huts), Blocked);
        assert_eq!(los_table(F::Ground, T::Wall), Blocked);
        assert_eq!(los_table(F::Ground, T::Trees), Clear);
        assert_eq!(los_table(F::Ground, T::Crest), Clear);
        assert_eq!(los_table(F::Ground, T::Rough), Clear);
        assert_eq!(los_table(F::Ground, T::Hilltop), Clear);

        // Rough firer: Clear, Blocked, Blocked, Blocked, Clear, Clear, Clear, Clear
        assert_eq!(los_table(F::Rough, T::Ground), Clear);
        assert_eq!(los_table(F::Rough, T::Units), Blocked);
        assert_eq!(los_table(F::Rough, T::Huts), Blocked);
        assert_eq!(los_table(F::Rough, T::Wall), Blocked);
        assert_eq!(los_table(F::Rough, T::Trees), Clear);
        assert_eq!(los_table(F::Rough, T::Crest), Clear);
        assert_eq!(los_table(F::Rough, T::Rough), Clear);
        assert_eq!(los_table(F::Rough, T::Hilltop), Clear);

        // Hilltop firer: Clear, Clear, Blocked, Blocked, Clear, Clear, Clear, Clear
        assert_eq!(los_table(F::Hilltop, T::Ground), Clear);
        assert_eq!(los_table(F::Hilltop, T::Units), Clear);
        assert_eq!(los_table(F::Hilltop, T::Huts), Blocked);
        assert_eq!(los_table(F::Hilltop, T::Wall), Blocked);
        assert_eq!(los_table(F::Hilltop, T::Trees), Clear);
        assert_eq!(los_table(F::Hilltop, T::Crest), Clear);
        assert_eq!(los_table(F::Hilltop, T::Rough), Clear);
        assert_eq!(los_table(F::Hilltop, T::Hilltop), Clear);
    }

    // -- has_los (ray-cast) tests (§6.21, §6.3) ---------------------------

    use crate::FireKind;
    use crate::board::BoardInfo;
    use omdurman_types::{HexCoord, HexsideKind, HexsideRef, Terrain};

    fn board_with_terrain(hexes: &[(i32, i32, Terrain)]) -> BoardInfo {
        let mut board = BoardInfo::default();
        for &(q, r, t) in hexes {
            board.terrain.insert(HexCoord::new(q, r), t);
        }
        board
    }

    fn board_with_hexsides(
        hexes: &[(i32, i32, Terrain)],
        sides: &[(HexCoord, HexCoord, HexsideKind)],
    ) -> BoardInfo {
        let mut board = board_with_terrain(hexes);
        for &(a, b, k) in sides {
            board.hexsides.insert(HexsideRef::new(a, b), k);
        }
        board
    }

    // §6.3
    #[test]
    fn has_los_empty_board_is_clear() {
        let board = BoardInfo::default();
        assert!(has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(5, 0),
            FireKind::Direct
        ));
    }

    // §6.3
    #[test]
    fn has_los_wall_hexside_blocks() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[], &[(a, b, HexsideKind::Wall)]);
        assert!(!has_los(&board, a, b, FireKind::Direct));
    }

    // §6.3
    #[test]
    fn has_los_crest_hexside_blocks() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[], &[(a, b, HexsideKind::Crest)]);
        assert!(!has_los(&board, a, b, FireKind::Direct));
    }

    // §6.3
    #[test]
    fn has_los_gate_hexside_passes() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[], &[(a, b, HexsideKind::Gate)]);
        assert!(has_los(&board, a, b, FireKind::Direct));
    }

    // §6.3
    #[test]
    fn has_los_breach_hexside_passes() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[], &[(a, b, HexsideKind::Breach)]);
        assert!(has_los(&board, a, b, FireKind::Direct));
    }

    // §6.3
    #[test]
    fn has_los_huts_intervening_blocks() {
        // firer at (0,0), intervening hut at (1,0), target at (2,0)
        let board = board_with_terrain(&[(1, 0, Terrain::Huts)]);
        assert!(!has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct
        ));
    }

    // §6.3
    #[test]
    fn has_los_building_intervening_blocks() {
        let board = board_with_terrain(&[(1, 0, Terrain::Building)]);
        assert!(!has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct
        ));
    }

    // §6.3
    #[test]
    fn has_los_two_tree_hexes_pass() {
        let board = board_with_terrain(&[(1, 0, Terrain::Trees), (2, 0, Terrain::Trees)]);
        assert!(has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(3, 0),
            FireKind::Direct
        ));
    }

    // §6.3
    #[test]
    fn has_los_three_tree_hexes_block() {
        let board = board_with_terrain(&[
            (1, 0, Terrain::Trees),
            (2, 0, Terrain::Trees),
            (3, 0, Terrain::Trees),
        ]);
        assert!(!has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(4, 0),
            FireKind::Direct
        ));
    }

    // §6.3
    #[test]
    fn has_los_hilltop_sees_over_intervening_terrain() {
        let board = board_with_terrain(&[(0, 0, Terrain::Hilltop), (1, 0, Terrain::Huts)]);
        assert!(has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct
        ));
    }

    // §6.3
    #[test]
    fn has_los_hilltop_still_blocked_by_wall() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[(0, 0, Terrain::Hilltop)], &[(a, b, HexsideKind::Wall)]);
        assert!(!has_los(&board, a, b, FireKind::Direct));
    }

    // §6.3
    #[test]
    fn has_los_howitzer_bypasses() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[], &[(a, b, HexsideKind::Wall)]);
        assert!(has_los(&board, a, b, FireKind::Howitzer));
    }

    // §6.3
    #[test]
    fn has_los_adjacent_clear() {
        let board = board_with_terrain(&[(0, 0, Terrain::Clear), (1, 0, Terrain::Clear)]);
        assert!(has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            FireKind::Direct
        ));
    }
}

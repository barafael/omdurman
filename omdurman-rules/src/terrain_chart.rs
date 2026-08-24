use omdurman_types::Terrain;

use crate::MovementAllowance;

/// A single entry in the Terrain Effects Chart (rulebook Terrain Effects Chart).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TerrainEntry {
    /// Additional movement points to enter a hex of this terrain
    /// (beyond the 1 MP base cost for clear terrain).
    /// `None` means impassable (Nile).
    pub movement_cost: Option<MovementAllowance>,
    /// Die-roll modifier for fire attacks targeting units in this terrain
    /// (negative = defender advantage).
    pub defense_modifier: i16,
}

/// Terrain Effects Chart -- maps each terrain type to its movement cost
/// and defensive die-roll modifier (rulebook Terrain Effects Chart).
///
/// Source: printed Terrain Effects Chart on the mapsheet.
pub fn terrain_effects_chart(terrain: Terrain) -> TerrainEntry {
    match terrain {
        Terrain::Clear { .. } => TerrainEntry {
            movement_cost: Some(MovementAllowance::One),
            defense_modifier: 0,
        },
        Terrain::Rough { .. } => TerrainEntry {
            movement_cost: Some(MovementAllowance::Two),
            defense_modifier: -1,
        },
        Terrain::Trees { .. } => TerrainEntry {
            movement_cost: Some(MovementAllowance::Two),
            defense_modifier: -2,
        },
        Terrain::Swamp { .. } => TerrainEntry {
            movement_cost: Some(MovementAllowance::Three),
            defense_modifier: 0,
        },
        Terrain::Nile { .. } => TerrainEntry {
            movement_cost: None,
            defense_modifier: 0,
        },
        Terrain::Hilltop { .. } => TerrainEntry {
            movement_cost: Some(MovementAllowance::Two),
            defense_modifier: -2,
        },
        Terrain::Huts { .. } => TerrainEntry {
            movement_cost: Some(MovementAllowance::One),
            defense_modifier: -2,
        },
        Terrain::Building { .. } => TerrainEntry {
            movement_cost: Some(MovementAllowance::One),
            defense_modifier: -3,
        },
    }
}

/// Convenience: get the defense modifier for a terrain type (rulebook §6.23, Terrain Effects Chart).
pub fn defense_modifier(terrain: Terrain) -> i16 {
    terrain_effects_chart(terrain).defense_modifier
}

/// Look up the defense modifier for a hex on the board (§6.23, Terrain Effects Chart).
///
/// Returns `0` if the hex has no terrain annotation (same as Clear).
/// Useful for the fire visualiser and hover tooltip: point at a hex and
/// get the defence modifier directly without calling the full chart.
pub fn defense_modifier_at(board: &crate::board::BoardInfo, hex: omdurman_types::HexCoord) -> i16 {
    board
        .terrain_at(hex)
        .map(|t| terrain_effects_chart(t).defense_modifier)
        .unwrap_or(0)
}

/// Convenience: get the movement cost for a terrain type (rulebook §5.11, Terrain Effects Chart).
/// Returns `None` for impassable terrain (Nile).
pub fn movement_cost(terrain: Terrain) -> Option<MovementAllowance> {
    terrain_effects_chart(terrain).movement_cost
}

/// Movement cost to enter a hex, accounting for a road overlay (rulebook Terrain
/// Effects Chart, Road row). A road costs a flat 1 MP regardless of the
/// underlying terrain; without a road it's the terrain's own cost. The road is
/// a movement overlay only -- combat/LOS still use the underlying terrain.
pub fn movement_cost_with_road(terrain: Terrain, road: bool) -> Option<MovementAllowance> {
    if road {
        Some(MovementAllowance::One)
    } else {
        movement_cost(terrain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::{GroundKind, Road};
    use strum::IntoEnumIterator;
    use traceability_macro::rulebook;

    fn t(kind: GroundKind) -> Terrain {
        Terrain::ground(kind)
    }

    #[rulebook("§5.11", "§6.23")]
    #[test]
    fn clear_terrain_no_bonus() {
        assert_eq!(defense_modifier(t(GroundKind::Clear)), 0);
    }

    #[rulebook("§6.23")]
    #[test]
    fn building_gives_minus_3() {
        assert_eq!(defense_modifier(t(GroundKind::Building)), -3);
    }

    #[rulebook("§6.23")]
    #[test]
    fn palm_grove_gives_minus_2() {
        assert_eq!(defense_modifier(t(GroundKind::Trees)), -2);
    }

    #[rulebook("§5.11")]
    #[test]
    fn nile_is_impassable() {
        let e = terrain_effects_chart(Terrain::Nile { direction: omdurman_types::HexDirection::East });
        assert!(e.movement_cost.is_none());
    }

    #[rulebook("§5.11", "§6.23")]
    #[test]
    fn rough_movement_and_defense() {
        let e = terrain_effects_chart(t(GroundKind::Rough));
        assert_eq!(e.movement_cost, Some(MovementAllowance::Two));
        assert_eq!(e.defense_modifier, -1);
    }

    #[rulebook("§5.11", "§6.23")]
    #[test]
    fn swamp_movement_and_defense() {
        let e = terrain_effects_chart(t(GroundKind::Swamp));
        assert_eq!(e.movement_cost, Some(MovementAllowance::Three));
        assert_eq!(e.defense_modifier, 0);
    }

    #[rulebook("§5.11", "§6.23")]
    #[test]
    fn hilltop_movement_and_defense() {
        let e = terrain_effects_chart(t(GroundKind::Hilltop));
        assert_eq!(e.movement_cost, Some(MovementAllowance::Two));
        assert_eq!(e.defense_modifier, -2);
    }

    #[rulebook("§5.11", "§6.23")]
    #[test]
    fn huts_movement_and_defense() {
        let e = terrain_effects_chart(t(GroundKind::Huts));
        assert_eq!(e.movement_cost, Some(MovementAllowance::One));
        assert_eq!(e.defense_modifier, -2);
    }

    #[rulebook("§6.23")]
    #[test]
    fn defense_modifier_convenience_matches_chart() {
        for kind in GroundKind::iter() {
            let terrain = t(kind);
            assert_eq!(
                defense_modifier(terrain),
                terrain_effects_chart(terrain).defense_modifier,
                "defense_modifier mismatch for {terrain:?}"
            );
        }
    }

    #[rulebook("§5.11")]
    #[test]
    fn movement_cost_convenience_matches_chart() {
        for kind in GroundKind::iter() {
            let terrain = t(kind);
            assert_eq!(
                movement_cost(terrain),
                terrain_effects_chart(terrain).movement_cost,
                "movement_cost mismatch for {terrain:?}"
            );
        }
    }

    #[rulebook("§5.11")]
    #[test]
    fn movement_cost_with_road_always_one() {
        for kind in GroundKind::iter() {
            let terrain = t(kind);
            assert_eq!(
                movement_cost_with_road(terrain, true),
                Some(MovementAllowance::One),
                "road override failed for {terrain:?}"
            );
        }
    }

    #[rulebook("§5.11")]
    #[test]
    fn movement_cost_without_road_matches_terrain() {
        for kind in GroundKind::iter() {
            let terrain = t(kind);
            assert_eq!(
                movement_cost_with_road(terrain, false),
                movement_cost(terrain),
                "no-road fallback mismatch for {terrain:?}"
            );
        }
    }

    #[rulebook("§5.11")]
    #[test]
    fn road_gives_crossroad() {
        let r = Terrain::ground_with_road(GroundKind::Clear, Road::Crossroad);
        assert!(r.is_crossroad());
        assert!(r.has_road());
    }

    // -- Property tests: terrain chart invariants ----------------------------

    #[rulebook("§5.11", "§6.23")]
    #[test]
    fn terrain_movement_costs_in_bounds() {
        for kind in GroundKind::iter() {
            let terrain = t(kind);
            let cost = movement_cost(terrain);
            match cost {
                None => panic!("{kind:?} should be passable (cost is None)"),
                Some(c) => assert!(
                    c.value() >= 1 && c.value() <= 3,
                    "{kind:?} movement cost {:?} out of 1..=3",
                    c.value()
                ),
            }
        }
        // Nile is a separate Terrain variant, not GroundKind.
        let nile = Terrain::Nile { direction: omdurman_types::HexDirection::East };
        assert!(movement_cost(nile).is_none(), "Nile should be impassable");
    }

    #[rulebook("§6.23")]
    #[test]
    fn terrain_defense_modifier_non_positive() {
        for kind in GroundKind::iter() {
            let terrain = t(kind);
            let mod_val = defense_modifier(terrain);
            assert!(
                mod_val <= 0,
                "{kind:?} defense modifier {mod_val} is positive (terrain should only help defender)"
            );
        }
    }

    #[rulebook("§5.11")]
    #[test]
    fn terrain_chart_road_always_costs_one() {
        for kind in GroundKind::iter() {
            let terrain = t(kind);
            let road_cost = movement_cost_with_road(terrain, true);
            assert_eq!(
                road_cost,
                Some(MovementAllowance::One),
                "road override should give 1 MP for {kind:?}"
            );
        }
    }
}

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
        Terrain::Clear => TerrainEntry {
            movement_cost: Some(MovementAllowance::One),
            defense_modifier: 0,
        },
        Terrain::Rough => TerrainEntry {
            movement_cost: Some(MovementAllowance::Two),
            defense_modifier: -1,
        },
        Terrain::Trees => TerrainEntry {
            movement_cost: Some(MovementAllowance::Two),
            defense_modifier: -2,
        },
        Terrain::Swamp => TerrainEntry {
            movement_cost: Some(MovementAllowance::Three),
            defense_modifier: 0,
        },
        Terrain::Nile => TerrainEntry {
            movement_cost: None,
            defense_modifier: 0,
        },
        Terrain::Hilltop => TerrainEntry {
            movement_cost: Some(MovementAllowance::Two),
            defense_modifier: -2,
        },
        Terrain::Huts => TerrainEntry {
            movement_cost: Some(MovementAllowance::One),
            defense_modifier: -2,
        },
        Terrain::Building => TerrainEntry {
            movement_cost: Some(MovementAllowance::One),
            defense_modifier: -3,
        },
    }
}

/// Convenience: get the defense modifier for a terrain type (rulebook §6.23, Terrain Effects Chart).
pub fn defense_modifier(terrain: Terrain) -> i16 {
    terrain_effects_chart(terrain).defense_modifier
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

    #[test]
    fn clear_terrain_no_bonus() {
        assert_eq!(defense_modifier(Terrain::Clear), 0);
    }

    #[test]
    fn building_gives_minus_3() {
        assert_eq!(defense_modifier(Terrain::Building), -3);
    }

    #[test]
    fn palm_grove_gives_minus_2() {
        assert_eq!(defense_modifier(Terrain::Trees), -2);
    }

    #[test]
    fn nile_is_impassable() {
        let e = terrain_effects_chart(Terrain::Nile);
        assert!(e.movement_cost.is_none());
    }
}

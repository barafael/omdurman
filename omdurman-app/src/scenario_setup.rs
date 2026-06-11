//! Automated scenario set-up — emits the unambiguous, fixed-hex unit
//! placements for a scenario as ordinary [`GameEvent::PlaceUnit`] events.
//!
//! The rulebook's set-up (§9) is mostly *player choice* ("all green units set
//! up within three hexes of Sheik El Din", "anywhere in the walled city"). Only
//! a handful of units have a single fixed hex — chiefly the six Dervish leaders,
//! which the Historical scenario pins to the lettered set-up hexes A/D/Y/K/S/O
//! (§9.212). Those letters exist on the campaign board as [`Terrain`] variants,
//! so we resolve each leader's destination from the loaded [`GameMap`] rather
//! than hard-coding coordinates — the placement stays correct if the board is
//! re-annotated.
//!
//! Everything else (tribal retinues, brigades, gunboats) is left for players to
//! drag from the picker, near the leaders this anchors. Placements flow through
//! the same [`GameEvent::PlaceUnit`] path as interactive placement
//! (`apply_pending_placement`), so they are netcode-ordered and acquire rules
//! `UnitId`s identically — see [[project_netcode_host_relay]] in memory.

use omdurman_map::GameMap;
use omdurman_net::GameEvent;
use omdurman_rules::Scenario;
use omdurman_types::{HexCoord, SectionName, Terrain};

/// One fixed-hex placement: which counter (`section`/`col`/`row` on the sprite
/// sheet) goes onto the single hex carrying `anchor` terrain.
struct FixedPlacement {
    section: SectionName,
    col: u32,
    row: u32,
    anchor: Terrain,
}

/// The six Dervish leaders and their Historical-scenario lettered set-up hexes
/// (§9.212). Two leaders (Yakub, Osman Digna) have no sprite section of their
/// own and ride in a tribal block — see `unit_profiles::identity_for_section`,
/// which resolves those specific counters as leaders.
const HISTORICAL_LEADERS: &[FixedPlacement] = &[
    // A: Ali Wad Helu
    FixedPlacement {
        section: SectionName::AliWadHelu,
        col: 0,
        row: 0,
        anchor: Terrain::A,
    },
    // D: Sheik El Din
    FixedPlacement {
        section: SectionName::SheikElDin,
        col: 0,
        row: 0,
        anchor: Terrain::D,
    },
    // Y: Yakub (first counter of the upper_Jaalin block)
    FixedPlacement {
        section: SectionName::UpperJaalin,
        col: 0,
        row: 0,
        anchor: Terrain::Y,
    },
    // K: Khalifa Abdullah
    FixedPlacement {
        section: SectionName::KhalifaAbdullah,
        col: 0,
        row: 0,
        anchor: Terrain::K,
    },
    // S: Sherif
    FixedPlacement {
        section: SectionName::Sherif,
        col: 0,
        row: 0,
        anchor: Terrain::S,
    },
    // O: Osman Digna (second counter of the Hadendowa block)
    FixedPlacement {
        section: SectionName::Hadendowa,
        col: 1,
        row: 0,
        anchor: Terrain::O,
    },
];

/// The single hex carrying `terrain` on the loaded map, if exactly one does.
/// The lettered set-up hexes and named landmarks are unique, so "first match"
/// is the intended one; returns `None` if the terrain isn't on the board (e.g.
/// the wrong scenario's map is loaded).
fn hex_with_terrain(game_map: &GameMap, terrain: Terrain) -> Option<HexCoord> {
    game_map
        .hexes
        .iter()
        .find(|(_, data)| data.terrain == terrain)
        .map(|(coord, _)| *coord)
}

/// Outcome of building a scenario's fixed placements: the `PlaceUnit` events to
/// broadcast, plus the names of any anchors that could not be resolved on the
/// current map (so the caller can surface them rather than silently dropping).
pub struct SetupPlan {
    pub placements: Vec<GameEvent>,
    pub unresolved: Vec<&'static str>,
}

/// Build the fixed-hex placements for `scenario` against the loaded `game_map`.
///
/// Only the Historical scenario has fixed-hex placements today (the six leaders
/// on their lettered hexes). The Campaign and Fall-of-Khartoum scenarios leave
/// all set-up to the players, so they return an empty plan.
pub fn build_setup_plan(scenario: Scenario, game_map: &GameMap) -> SetupPlan {
    let fixed: &[FixedPlacement] = match scenario {
        Scenario::Historical => HISTORICAL_LEADERS,
        Scenario::Campaign | Scenario::FallOfKhartoum => &[],
    };

    let mut placements = Vec::new();
    let mut unresolved = Vec::new();
    for fp in fixed {
        match hex_with_terrain(game_map, fp.anchor) {
            Some(coord) => placements.push(GameEvent::PlaceUnit {
                section_name: fp.section,
                col: fp.col,
                row: fp.row,
                coord_q: coord.q,
                coord_r: coord.r,
                is_boat: false,
            }),
            None => unresolved.push(fp.section.display_name()),
        }
    }
    SetupPlan {
        placements,
        unresolved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::HexData;

    fn map_with(terrains: &[(i32, i32, Terrain)]) -> GameMap {
        let mut m = GameMap::default();
        for &(q, r, t) in terrains {
            m.hexes.insert(HexCoord::new(q, r), HexData::new(t, None));
        }
        m
    }

    #[test]
    fn historical_places_all_six_leaders_when_anchors_present() {
        let map = map_with(&[
            (15, 4, Terrain::A),
            (20, 10, Terrain::D),
            (26, 17, Terrain::Y),
            (28, 21, Terrain::K),
            (31, 23, Terrain::S),
            (33, 23, Terrain::O),
        ]);
        let plan = build_setup_plan(Scenario::Historical, &map);
        assert_eq!(plan.placements.len(), 6);
        assert!(plan.unresolved.is_empty());

        // The Khalifa counter must land on the K hex.
        let khalifa = plan
            .placements
            .iter()
            .find_map(|e| match e {
                GameEvent::PlaceUnit {
                    section_name,
                    coord_q,
                    coord_r,
                    ..
                } if *section_name == SectionName::KhalifaAbdullah => Some((*coord_q, *coord_r)),
                _ => None,
            })
            .expect("Khalifa placement present");
        assert_eq!(khalifa, (28, 21));
    }

    #[test]
    fn missing_anchor_is_reported_not_dropped_silently() {
        // Only the A hex exists; the other five leaders are unresolved.
        let map = map_with(&[(15, 4, Terrain::A)]);
        let plan = build_setup_plan(Scenario::Historical, &map);
        assert_eq!(plan.placements.len(), 1);
        assert_eq!(plan.unresolved.len(), 5);
    }

    #[test]
    fn campaign_has_no_fixed_placements() {
        let map = map_with(&[(28, 21, Terrain::K)]);
        let plan = build_setup_plan(Scenario::Campaign, &map);
        assert!(plan.placements.is_empty());
        assert!(plan.unresolved.is_empty());
    }
}

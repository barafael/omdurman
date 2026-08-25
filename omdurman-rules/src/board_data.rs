//! Board data (§dual-map): the two boards as RON files under
//! `omdurman-app/assets/boards/`, embedded at compile time and parsed once on
//! first use. The map editor (`tools/map-editor`) edits those files offline;
//! the game bootstrap (`omdurman-app/src/board_state.rs`) and the tactics
//! fixtures (`tactics.rs`) consume them through the accessors below.
//!
//! The `include_str!` paths reach across workspace members on purpose: the
//! canonical data lives next to the game's other assets, and path crates in
//! this workspace are never published, so the cross-crate reach is confined to
//! the repository.

use std::sync::OnceLock;

use omdurman_types::MapData;

const CAMPAIGN_RON: &str = include_str!("../../omdurman-app/assets/boards/campaign.ron");
const FOK_RON: &str = include_str!("../../omdurman-app/assets/boards/fall_of_khartoum.ron");

fn parse(text: &str) -> MapData {
    match ron::from_str(text) {
        Ok(map) => map,
        // A corrupt board file is an authoring error surfaced by the map
        // editor's save path; fail loud rather than running on an empty board.
        Err(e) => panic!("failed to parse board RON: {e}"),
    }
}

static CAMPAIGN: OnceLock<MapData> = OnceLock::new();
static FOK: OnceLock<MapData> = OnceLock::new();

/// The Campaign board (also used by the Historical scenario, §9.1/§9.2).
pub fn campaign_map_data() -> MapData {
    CAMPAIGN.get_or_init(|| parse(CAMPAIGN_RON)).clone()
}

/// The Fall-of-Khartoum board (§9.3).
pub fn fall_of_khartoum_map_data() -> MapData {
    FOK.get_or_init(|| parse(FOK_RON)).clone()
}
#[cfg(test)]
mod wall_ring_tests {
    use super::*;
    use crate::board::BoardInfo;
    use omdurman_types::{HexCoord, HexsideKind, HexsideRef, Location, Terrain};

    /// §5.23: the walled city must be the area *enclosed* by the annotated
    /// Wall/Gate/Breach ring, anchored at the Palace (and the Mahdi's Tomb on
    /// the Omdurman board). These tests pin the compiled boards' derivations.
    #[test]
    fn campaign_walled_city_is_enclosed_by_walls() {
        let board = BoardInfo::from_map_data(&campaign_map_data());
        let palace = board.hex_of_location(Location::Palace).unwrap();
        let tomb = board.hex_of_location(Location::MahdisTomb).unwrap();
        assert!(board.is_walled_city(palace) && board.is_walled_city(tomb));
        // The Omdurman walled city is the ~27-hex enclosed block around the
        // palace; the old >=2-of-6 heuristic flagged 33 hexes including 16
        // *outside* the wall (audit §5.23: entry through unannotated fringe
        // sides).
        assert_eq!(board.walled_city.len(), 27, "compiled campaign walled-city set");
        // Enclosure invariant: no interior hex has an unannotated side to an
        // on-map land hex *outside* the city (the ring is closed).
        for h in &board.walled_city {
            for n in h.neighbors() {
                if board.walled_city.contains(&n) {
                    continue;
                }
                if matches!(board.terrain_at(n), None | Some(Terrain::Nile { .. })) {
                    continue;
                }
                let annotated = matches!(
                    board.hexsides.get(&HexsideRef::new(*h, n)),
                    Some(HexsideKind::Wall | HexsideKind::Gate | HexsideKind::Breach)
                );
                assert!(
                    annotated,
                    "city hex {h:?} has an open side to outside hex {n:?} -- ring not closed"
                );
            }
        }
        // The audit's breach hex (29,38) is outside the wall (a fringe hex the
        // heuristic wrongly counted); entering it is not a walled-city entry.
        assert!(!board.is_walled_city(HexCoord::new(29, 38)));
    }

    #[test]
    fn fok_walled_city_is_the_building_block() {
        // FoK (§2.1): the washed-away wall section is a legal gap, so the
        // fill is bounded by Building terrain instead -- the 17-hex city
        // block around the palace.
        let board = BoardInfo::from_map_data(&fall_of_khartoum_map_data());
        let palace = board.hex_of_location(Location::Palace).unwrap();
        assert!(board.is_walled_city(palace));
        assert_eq!(board.walled_city.len(), 17, "compiled FoK walled-city set");
        for h in &board.walled_city {
            assert!(
                matches!(board.terrain_at(*h), Some(Terrain::Building { .. }))
                    || board.locations.get(h) == Some(&Location::Palace),
                "FoK city hex {h:?} must be Building terrain or a landmark"
            );
        }
    }
}

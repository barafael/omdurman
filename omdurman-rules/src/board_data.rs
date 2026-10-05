//! Board data (§dual-map): the two boards as RON files under
//! `omdurman-app/assets/boards/`, embedded at compile time and parsed once on
//! first use. The files are edited as text (the map editor that authored
//! them is retired); the game bootstrap (`omdurman-app/src/board_state.rs`) and the tactics
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

    /// §9.231/§9.232 on the printed map (Terrain Effects Chart legend: the
    /// trench is the solid line with a dashed parapet, the thorn hedge the
    /// line of crosses): the Zariba's northern stretch, from the Nile down
    /// past the "The Zariba" label, is the trench; the southern stretch down
    /// to the river is the thorn hedge. Each end where the line meets the
    /// Nile is an entrance (§9.233) of its own stretch's kind.
    #[traceability_macro::rulebook("§9.231", "§9.232", "§9.233")]
    #[test]
    fn the_zariba_is_trench_in_the_north_and_hedge_in_the_south() {
        let board = BoardInfo::from_map_data(&campaign_map_data());
        let side = |a: (i32, i32), b: (i32, i32)| {
            board.hexside_between(HexCoord::new(a.0, a.1), HexCoord::new(b.0, b.1))
        };
        assert_eq!(side((32, 10), (33, 11)), Some(HexsideKind::ZaribaTrenchEnd));
        assert_eq!(side((32, 11), (33, 11)), Some(HexsideKind::ZaribaTrench));
        assert_eq!(side((33, 14), (33, 15)), Some(HexsideKind::ZaribaTrench));
        assert_eq!(
            side((33, 15), (34, 15)),
            Some(HexsideKind::ZaribaThornHedge)
        );
        assert_eq!(
            side((36, 18), (37, 18)),
            Some(HexsideKind::ZaribaThornHedge)
        );
        assert_eq!(
            side((37, 18), (37, 19)),
            Some(HexsideKind::ZaribaThornHedgeEnd)
        );
        let count =
            |want: fn(HexsideKind) -> bool| board.hexsides.values().filter(|k| want(**k)).count();
        assert_eq!(count(HexsideKind::is_zariba_trench), 9);
        assert_eq!(count(HexsideKind::is_zariba_thorn_hedge), 8);
        assert_eq!(
            board.zariba.len(),
            13,
            "the 13 hexes of the Zariba (§9.211)"
        );
    }

    /// §5.44 / §6.3 note b: Khartoum's rampart is open (§2.1), so the city
    /// side of each wall and gate hexside is derived per hexside -- exactly
    /// one of its two hexes is inside, including on the bastions, where the
    /// straight-line distance to the Palace ties or points the wrong way.
    #[traceability_macro::rulebook("§5.44")]
    #[test]
    fn every_khartoum_wall_hexside_has_one_city_side() {
        let board = BoardInfo::from_map_data(&fall_of_khartoum_map_data());
        let mut walls = 0;
        for (side, kind) in &board.hexsides {
            if !matches!(kind, HexsideKind::Wall | HexsideKind::Gate) {
                continue;
            }
            walls += 1;
            assert_ne!(
                board.is_inside_of_wall(side.a, side.b),
                board.is_inside_of_wall(side.b, side.a),
                "{:?}-{:?}",
                side.a,
                side.b
            );
        }
        assert!(walls > 30);
        // The Kalakla bastion juts out of the wall: it is inside against
        // both re-entrant hexes beside it.
        let bastion = HexCoord::new(13, 12);
        assert!(board.is_inside_of_wall(bastion, HexCoord::new(14, 12)));
        assert!(board.is_inside_of_wall(bastion, HexCoord::new(12, 12)));
        // Omdurman's wall encloses its city: no per-hexside table there.
        assert!(
            BoardInfo::from_map_data(&campaign_map_data())
                .palace_steps
                .is_empty()
        );
    }

    /// §5.23: the walled city must be the area *enclosed* by the annotated
    /// Wall/Gate/Breach ring, anchored at the Palace (and the Mahdi's Tomb on
    /// the Omdurman board). These tests pin the compiled boards' derivations.
    #[traceability_macro::rulebook("§5.23")]
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
        assert_eq!(
            board.walled_city.len(),
            27,
            "compiled campaign walled-city set"
        );
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

    /// The FoK forts are fort hexes (§6.54: no extra cost to enter or leave a
    /// friendly fort; they may be meleed and shot by artillery), not little
    /// walled cities: no wall hexside may ring them. (Their printed outlines
    /// were authored as walls, sealing every fort garrison in for the game.)
    #[traceability_macro::rulebook("§6.54")]
    #[test]
    fn fok_forts_are_not_walled_in() {
        let board = BoardInfo::from_map_data(&fall_of_khartoum_map_data());
        for fort in [
            Location::FortMakran,
            Location::FortBuri,
            Location::NorthFort,
        ] {
            let hex = board.hex_of_location(fort).unwrap();
            for n in hex.neighbors() {
                assert_ne!(
                    board.hexside_between(hex, n),
                    Some(HexsideKind::Wall),
                    "{fort:?} {hex:?} is walled off from {n:?}"
                );
            }
        }
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

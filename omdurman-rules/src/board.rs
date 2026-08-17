//! Static per-board map facts the rules engine needs to enforce
//! map-dependent rules (rulebook §5.11, §5.24, §5.44, §6.6x, §9.14, §10).
//!
//! The rules engine is otherwise mapless: [`GameState`](crate::effects::GameState)
//! tracks units and phase, not terrain. Rules that depend on hexside features,
//! terrain cost, or the Nile current (ZOC across a khor, gunboat
//! upstream/downstream, artillery vs. a fort, mine drift) need the map. Rather
//! than reach into the Bevy/app layer, the app builds a [`BoardInfo`] from the
//! active board's annotations at game start and stores it *in* the serialized
//! `GameState`, so late joiners and `GameRecord` replay reproduce it for free.

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

use omdurman_types::{HexCoord, HexDirection, HexsideKind, HexsideRef, Location, MapData, Terrain};

/// The static map facts the rules engine consults. Keyed lookups are kept as
/// `IndexMap`s so serialization is deterministic (matching the rest of the
/// codebase's `serde`/`indexmap` convention).
///
/// An empty `BoardInfo` (the [`Default`]) means "no map loaded": every lookup
/// returns the rule-neutral answer, so tests and `GameState::new` that do not
/// attach a board behave exactly as before this type existed.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BoardInfo {
    /// Per-edge hexside features (wall/gate/breach/khor/Zariba…), keyed by the
    /// canonical [`HexsideRef`] so the lookup is direction-independent (§5.44).
    pub hexsides: IndexMap<HexsideRef, HexsideKind>,
    /// Terrain per playable hex (§5.11). Absent hexes are treated as off-map.
    pub terrain: IndexMap<HexCoord, Terrain>,
    /// Named landmarks (Palace/Mahdi's Tomb, forts, gates) for victory and
    /// scenario rules (§9.14, §9.344, §9.346).
    pub locations: IndexMap<HexCoord, Location>,
    /// Road edges (§5.11 Terrain Effects Chart: road movement costs 1 MP
    /// regardless of underlying terrain). Stored as canonical hexside refs.
    #[serde(default)]
    pub roads: IndexSet<HexsideRef>,
    /// Reinforcement entrance areas (§9.112/§9.113), authored per-hex in the
    /// map editor via `HexData::named_area`. Empty on boards without entrance
    /// annotations -- callers fall back to geometric approximations.
    #[serde(default)]
    pub entrances: IndexMap<HexCoord, omdurman_types::NamedArea>,
}

impl BoardInfo {
    /// Build the engine's view of a board from its saved [`MapData`]. Pulls
    /// terrain and Nile-current per tile, the per-edge hexside features, and any
    /// named landmarks, discarding the rendering/calibration data the engine
    /// does not need. Excluded (off-map) hexes are skipped.
    pub fn from_map_data(map: &MapData) -> Self {
        let mut board = BoardInfo::default();
        for ((q, r), tile) in &map.tiles {
            if map.excluded.contains(&(*q, *r)) {
                continue;
            }
            let hex = HexCoord::new(*q, *r);
            board.terrain.insert(hex, tile.terrain);
            // Promote rules-significant named tiles (Palace, North Fort, gates,
            // …) to landmarks the engine can locate for §9.14 / §9.34x / §9.346.
            if let Some(location) = tile
                .name
                .as_deref()
                .and_then(omdurman_types::Location::from_tile_name)
            {
                board.locations.insert(hex, location);
            }
        }
        for (edge, kind) in &map.hexsides {
            board.hexsides.insert(*edge, *kind);
        }
        for edge in &map.roads {
            board.roads.insert(*edge);
        }
        // Entrance areas (§9.112/§9.113): promote per-tile named-area
        // annotations onto the engine's board view.
        for ((q, r), tile) in &map.tiles {
            if let Some(area) = tile.named_area {
                board.entrances.insert(HexCoord::new(*q, *r), area);
            }
        }
        board
    }

    /// The hexside feature on the edge between two hexes, if any (§5.44).
    pub fn hexside_between(&self, a: HexCoord, b: HexCoord) -> Option<HexsideKind> {
        self.hexsides.get(&HexsideRef::new(a, b)).copied()
    }

    /// Whether the edge between two hexes carries a feature satisfying `pred`
    /// (e.g. `HexsideKind::blocks_advance_after_combat`). `false` when no
    /// feature is present (§5.44, §6.82, §7.2).
    pub fn hexside_is(&self, a: HexCoord, b: HexCoord, pred: impl Fn(HexsideKind) -> bool) -> bool {
        self.hexside_between(a, b).is_some_and(pred)
    }

    /// The terrain at a hex; `None` if the hex is off-map / unannotated (§5.11).
    pub fn terrain_at(&self, hex: HexCoord) -> Option<Terrain> {
        self.terrain.get(&hex).copied()
    }

    /// Whether any edge of `hex` is a road (§5.11: road movement costs 1 MP).
    /// Matches the app-side `floor_movement_cost` convention so the overlay
    /// and engine agree.
    pub fn has_road(&self, hex: HexCoord) -> bool {
        hex.neighbors()
            .iter()
            .any(|n| self.roads.contains(&HexsideRef::new(hex, *n)))
    }

    /// The `(min_q, max_q, min_r, max_r)` extent of the playable hexes, or `None`
    /// for an empty board. One pass over `terrain`, so callers that need the map
    /// edges repeatedly (e.g. the deployment-zone check across every hex) compute
    /// them once rather than re-scanning per hex.
    pub fn bounds(&self) -> Option<(i32, i32, i32, i32)> {
        let mut keys = self.terrain.keys();
        let first = keys.next()?;
        let (mut min_q, mut max_q, mut min_r, mut max_r) = (first.q, first.q, first.r, first.r);
        for c in keys {
            min_q = min_q.min(c.q);
            max_q = max_q.max(c.q);
            min_r = min_r.min(c.r);
            max_r = max_r.max(c.r);
        }
        Some((min_q, max_q, min_r, max_r))
    }

    /// Whether the hex is a Nile river hex (§5.22, §5.24). Off-map hexes are
    /// not Nile.
    pub fn is_nile(&self, hex: HexCoord) -> bool {
        self.terrain_at(hex).is_some_and(Terrain::is_nile)
    }

    /// The Nile current direction at a hex, if annotated (§5.24).
    pub fn flow_at(&self, hex: HexCoord) -> Option<HexDirection> {
        self.terrain_at(hex)?.nile_direction()
    }

    /// Classify a single gunboat step `from -> to` against the Nile current
    /// (§5.24). The current at `from` flows *toward* `flow.dir`'s neighbour, so
    /// a step that way is downstream and the opposite way is upstream. Returns
    /// `None` when `from` carries no current annotation (direction unknown) or
    /// `to` is not the up/downstream neighbour.
    pub fn step_direction(&self, from: HexCoord, to: HexCoord) -> Option<StepDirection> {
        let direction = self.flow_at(from)?;
        let neighbors = from.neighbors();
        let downstream = neighbors[direction as usize];
        let upstream = neighbors[crate::effects::opposite(direction as usize)];
        if to == downstream {
            Some(StepDirection::Downstream)
        } else if to == upstream {
            Some(StepDirection::Upstream)
        } else {
            None
        }
    }

    /// The named landmark at a hex, if any (§9.14, §9.344).
    pub fn location_at(&self, hex: HexCoord) -> Option<Location> {
        self.locations.get(&hex).copied()
    }

    /// Whether the given hex is "entrenched" — that is, lies on the Nile side
    /// of a ZaribaTrench hexside (§9.232: units Nile-side of a trench hexside
    /// are entrenched; units on the opposite side are not). The trench hexsides
    /// run roughly north–south between the Zariba compound and the Nile, so a
    /// hex is entrenched if one of its edges is a `ZaribaTrench` and the hex
    /// is on the *Nile* side of that edge (i.e. the edge's midpoint lies
    /// between the hex and the river).
    ///
    /// Because the Zariba trench runs *between* the Zariba compound (thorn
    /// hedge) and the Nile, a hex is entrenched if it neighbours a Nile hex
    /// *and* the hexside towards that Nile hex is a trench variant.  A simpler
    /// heuristic: a hex is entrenched if any of its edges is a Zariba trench
    /// and the hex itself is Nile-adjacent (has a neighbour classified as
    /// Nile terrain).
    pub fn is_zariba_entrenched(&self, hex: HexCoord) -> bool {
        // A hex is entrenched if it has at least one ZaribaTrench hexside on
        // an edge leading toward the Nile — meaning the hex itself is adjacent
        // to a Nile hex across a ZaribaTrench edge.
        for n in hex.neighbors() {
            if let Some(kind) = self.hexside_between(hex, n)
                && matches!(
                    kind,
                    omdurman_types::HexsideKind::ZaribaTrench
                        | omdurman_types::HexsideKind::ZaribaTrenchEndA
                        | omdurman_types::HexsideKind::ZaribaTrenchEndB
                ) {
                    // The hex is on the Nile side if the neighbour is a Nile hex.
                    if self.is_nile(n) {
                        return true;
                    }
                }
        }
        false
    }

    /// Whether a given target hex (occupied by enemy units) has any zariba
    /// hexside on its perimeter — i.e. whether the ZaribaThornHedge modifier
    /// applies (§9.231).
    pub fn has_zariba_thorn_hedge(&self, hex: HexCoord) -> bool {
        for n in hex.neighbors() {
            if let Some(kind) = self.hexside_between(hex, n)
                && kind == omdurman_types::HexsideKind::ZaribaThornHedge {
                return true;
            }
        }
        false
    }

    /// The +2 MP cost of crossing a Zariba end hexside (§9.233: "Units may only
    /// enter and/or leave the Zariba via the two end hexsides ... paying +2
    /// movement points to cross"). Returns 2 when the edge between `from` and
    /// `to` is one of the two trench ends, else 0.
    pub fn zariba_entry_surcharge(&self, from: HexCoord, to: HexCoord) -> i16 {
        match self.hexside_between(from, to) {
            Some(k) if k.is_zariba_trench_end() => 2,
            _ => 0,
        }
    }

    /// Whether `hex` lies inside a walled enclosure (§5.23: "the walled portion
    /// of Omdurman"). A hex is inside the walled city when it is the Palace or
    /// Mahdi's Tomb landmark, or when at least two of its six hexsides are
    /// Wall/Gate/Breach -- an interior city hex is bounded by the perimeter
    /// wall on multiple sides, unlike a hex outside the wall that merely
    /// touches it on one. The two-sided threshold keeps the predicate robust to
    /// a map edit that adds or removes a single wall segment.
    pub fn is_walled_city(&self, hex: HexCoord) -> bool {
        if matches!(
            self.location_at(hex),
            Some(Location::Palace) | Some(Location::MahdisTomb)
        ) {
            return true;
        }
        let wall_sides = hex
            .neighbors()
            .iter()
            .filter(|n| {
                matches!(
                    self.hexside_between(hex, **n),
                    Some(HexsideKind::Wall | HexsideKind::Gate | HexsideKind::Breach)
                )
            })
            .count();
        wall_sides >= 2
    }

    /// The hex of a named landmark, if present on this board (§9.14: the
    /// Mahdi's Tomb is the [`Location::MahdisTomb`] hex, distinct from the
    /// [`Location::Palace`] hex in the walled city of Omdurman).
    pub fn hex_of_location(&self, want: Location) -> Option<HexCoord> {
        self.locations
            .iter()
            .find_map(|(hex, loc)| (*loc == want).then_some(*hex))
    }

    /// All hexes annotated as the given entrance area (§9.112/§9.113), in
    /// board order. Empty when the board carries no annotation for `area`.
    pub fn entrance_hexes(&self, area: omdurman_types::NamedArea) -> Vec<HexCoord> {
        self.entrances
            .iter()
            .filter(|(_, a)| **a == area)
            .map(|(hex, _)| *hex)
            .collect()
    }

    /// Which bank of the Nile a hex sits on, used for "Friendlies" victory
    /// scoring (§9.14: east-bank eliminations score 1 pt, west-bank 3 pts) and
    /// the §5.21 transport. The Nile runs roughly north-south down the map, with
    /// the Dervish (west) bank at lower `q` and the Anglo-Egyptian (east) bank
    /// at higher `q`. A hex is classified by comparing its `q` against the Nile
    /// hex(es) on the same map row (`r`); `None` when there is no Nile on that
    /// row to compare against (or no board loaded).
    pub fn bank_of(&self, hex: HexCoord) -> Option<NileBank> {
        let mut min_nile_q: Option<i32> = None;
        let mut max_nile_q: Option<i32> = None;
        for (coord, terrain) in &self.terrain {
            if coord.r == hex.r && terrain.is_nile() {
                min_nile_q = Some(min_nile_q.map_or(coord.q, |q: i32| q.min(coord.q)));
                max_nile_q = Some(max_nile_q.map_or(coord.q, |q: i32| q.max(coord.q)));
            }
        }
        let (min_q, max_q) = (min_nile_q?, max_nile_q?);
        if hex.q < min_q {
            Some(NileBank::West)
        } else if hex.q > max_q {
            Some(NileBank::East)
        } else {
            // The hex is itself in the Nile channel -- neither bank.
            None
        }
    }
}

/// Which side of the Nile a hex lies on (rulebook §5.21, §9.14). The Dervish
/// (west) bank is at lower `q`; the Anglo-Egyptian (east) bank at higher `q`.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum NileBank {
    West,
    East,
}

/// Direction of a single gunboat step relative to the Nile current (§5.24).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepDirection {
    /// With the current (uses the larger downstream allowance).
    Downstream,
    /// Against the current (caps the turn at the upstream allowance).
    Upstream,
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::{GroundKind, HexData, HexDirection};
    use std::collections::BTreeSet;

    fn default_overlay() -> omdurman_types::OverlayParams {
        omdurman_types::OverlayParams::default()
    }
    fn default_calib() -> omdurman_types::CalibAnchors {
        omdurman_types::CalibAnchors {
            p1_px: (0.0, 0.0),
            p1_hex: (0, 0),
            p2_px: (1.0, 1.0),
            p2_hex: (1, 0),
        }
    }

    fn make_map(tiles: Vec<((i32, i32), HexData)>) -> MapData {
        MapData {
            tiles: tiles.into_iter().collect(),
            hexsides: Vec::new(),
            roads: Vec::new(),
            excluded: BTreeSet::new(),
            overlay: default_overlay(),
            img_w: 100.0,
            img_h: 100.0,
            image: "test.webp".into(),
            calib: default_calib(),
            campaign_turn_track: None,
        }
    }

    fn tile(terrain: Terrain) -> HexData {
        HexData::new(terrain, None)
    }

    fn nile_tile(dir: HexDirection) -> HexData {
        HexData {
            terrain: Terrain::Nile { direction: dir },
            ..HexData::new(Terrain::default(), None)
        }
    }

    fn named_tile(terrain: Terrain, name: &str) -> HexData {
        HexData::new(terrain, Some(name.to_string()))    }

    // -- from_map_data --------------------------------------------------

    #[test]
    fn from_map_data_populates_terrain_and_nile() {
        let map = make_map(vec![
            ((0, 0), tile(Terrain::default())),
            ((1, 0), nile_tile(HexDirection::SouthEast)),
            ((2, 1), tile(Terrain::ground(GroundKind::Hilltop))),
        ]);
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(board.terrain_at(HexCoord::new(0, 0)), Some(Terrain::default()));
        assert_eq!(
            board.terrain_at(HexCoord::new(2, 1)),
            Some(Terrain::ground(GroundKind::Hilltop))
        );
        assert!(board.is_nile(HexCoord::new(1, 0)));
        assert!(!board.is_nile(HexCoord::new(0, 0)));
        assert_eq!(
            board.flow_at(HexCoord::new(1, 0)),
            Some(HexDirection::SouthEast)
        );
    }

    #[test]
    fn from_map_data_skips_excluded_hexes() {
        let mut map = make_map(vec![
            ((0, 0), tile(Terrain::default())),
            ((1, 1), tile(Terrain::ground(GroundKind::Rough))),
        ]);
        map.excluded.insert((1, 1));
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(board.terrain_at(HexCoord::new(0, 0)), Some(Terrain::default()));
        assert_eq!(board.terrain_at(HexCoord::new(1, 1)), None);
    }

    #[test]
    fn from_map_data_collects_entrance_annotations() {
        // Entrance areas (§9.112/§9.113) authored per-tile surface on the
        // engine board and are queryable per area.
        let entrance = |area: omdurman_types::NamedArea| HexData {
            named_area: Some(area),
            ..HexData::new(Terrain::default(), None)
        };
        let map = make_map(vec![
            ((0, 0), entrance(omdurman_types::NamedArea::DervishWestEdge)),
            ((0, 1), entrance(omdurman_types::NamedArea::DervishWestEdge)),
            ((1, 0), entrance(omdurman_types::NamedArea::AngloEgyptianEntrance)),
            ((2, 0), tile(Terrain::default())),
        ]);
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(
            board.entrance_hexes(omdurman_types::NamedArea::DervishWestEdge),
            vec![HexCoord::new(0, 0), HexCoord::new(0, 1)]
        );
        assert_eq!(
            board.entrance_hexes(omdurman_types::NamedArea::AngloEgyptianEntrance),
            vec![HexCoord::new(1, 0)]
        );
        // Areas with no annotation yield nothing (callers fall back).
        assert!(board
            .entrance_hexes(omdurman_types::NamedArea::AbuAlimHut)
            .is_empty());
    }

    #[test]
    fn from_map_data_promotes_landmarks() {
        let map = make_map(vec![
            ((3, 5), named_tile(Terrain::ground(GroundKind::Building), "Palace")),
            ((2, 4), named_tile(Terrain::ground(GroundKind::Building), "North Fort")),
            ((0, 0), named_tile(Terrain::default(), "Khartoum")),
        ]);
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(
            board.location_at(HexCoord::new(3, 5)),
            Some(Location::Palace)
        );
        assert_eq!(
            board.location_at(HexCoord::new(2, 4)),
            Some(Location::NorthFort)
        );
        // "Khartoum" is not a rules-significant landmark.
        assert_eq!(board.location_at(HexCoord::new(0, 0)), None);
    }

    #[test]
    fn from_map_data_copies_hexsides() {
        let mut map = make_map(vec![]);
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        map.hexsides
            .push((HexsideRef::new(a, b), HexsideKind::Wall));
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(board.hexside_between(a, b), Some(HexsideKind::Wall));
    }

    // -- location_at ----------------------------------------------------

    #[test]
    fn location_at_returns_inserted_value() {
        let mut board = BoardInfo::default();
        board
            .locations
            .insert(HexCoord::new(5, 5), Location::Arsenal);
        assert_eq!(
            board.location_at(HexCoord::new(5, 5)),
            Some(Location::Arsenal)
        );
        assert_eq!(board.location_at(HexCoord::new(6, 6)), None);
    }

    // -- step_direction -------------------------------------------------

    #[test]
    fn step_direction_downstream() {
        let mut board = BoardInfo::default();
        // Hex (2,3) has flow toward East (dir=0), so neighbor[0] = downstream.
        board.terrain.insert(
            HexCoord::new(2, 3),
            Terrain::Nile { direction: HexDirection::East },
        );
        let from = HexCoord::new(2, 3);
        let downstream = from.neighbors()[0]; // East neighbor
        assert_eq!(
            board.step_direction(from, downstream),
            Some(StepDirection::Downstream)
        );
    }

    #[test]
    fn step_direction_upstream() {
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(2, 3),
            Terrain::Nile { direction: HexDirection::East },
        );
        let from = HexCoord::new(2, 3);
        let upstream = from.neighbors()[3]; // West neighbor
        assert_eq!(
            board.step_direction(from, upstream),
            Some(StepDirection::Upstream)
        );
    }

    #[test]
    fn step_direction_invalid_neighbor() {
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(2, 3),
            Terrain::Nile { direction: HexDirection::East },
        );
        let from = HexCoord::new(2, 3);
        // A diagonal-ish neighbor that is neither up nor downstream.
        let sideways = from.neighbors()[1]; // SouthEast
        assert_eq!(board.step_direction(from, sideways), None);
    }

    #[test]
    fn step_direction_no_flow_at_hex() {
        let board = BoardInfo::default();
        assert_eq!(
            board.step_direction(HexCoord::new(0, 0), HexCoord::new(1, 0)),
            None
        );
    }

    // -- bank_of --------------------------------------------------------

    #[test]
    fn bank_of_west_of_nile() {
        let mut board = BoardInfo::default();
        // Nile hexes at q=5 on row r=3.
        board.terrain.insert(HexCoord::new(5, 3), Terrain::Nile { direction: HexDirection::East });
        // West hex has q < 5.
        assert_eq!(board.bank_of(HexCoord::new(3, 3)), Some(NileBank::West));
    }

    #[test]
    fn bank_of_east_of_nile() {
        let mut board = BoardInfo::default();
        board.terrain.insert(HexCoord::new(5, 3), Terrain::Nile { direction: HexDirection::East });
        // East hex has q > 5.
        assert_eq!(board.bank_of(HexCoord::new(8, 3)), Some(NileBank::East));
    }

    #[test]
    fn bank_of_hex_on_nile_returns_none() {
        let mut board = BoardInfo::default();
        board.terrain.insert(HexCoord::new(5, 3), Terrain::Nile { direction: HexDirection::East });
        // The hex is itself in the Nile channel.
        assert_eq!(board.bank_of(HexCoord::new(5, 3)), None);
    }

    #[test]
    fn bank_of_no_nile_on_row_returns_none() {
        let mut board = BoardInfo::default();
        // Only Clear terrain on row 3 — no Nile.
        board.terrain.insert(HexCoord::new(5, 3), Terrain::default());
        assert_eq!(board.bank_of(HexCoord::new(3, 3)), None);
    }

    #[test]
    fn bank_of_empty_board_returns_none() {
        let board = BoardInfo::default();
        assert_eq!(board.bank_of(HexCoord::new(0, 0)), None);
    }

    // -- bounds / hex_of_location ---------------------------------------

    #[test]
    fn bounds_empty_board() {
        let board = BoardInfo::default();
        assert_eq!(board.bounds(), None);
    }

    #[test]
    fn bounds_computes_extent() {
        let map = make_map(vec![
            ((0, 0), tile(Terrain::default())),
            ((5, 3), tile(Terrain::default())),
            ((2, -1), tile(Terrain::default())),
        ]);
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(board.bounds(), Some((0, 5, -1, 3)));
    }

    #[test]
    fn hex_of_location_finds_correct_hex() {
        let map = make_map(vec![
            ((3, 5), named_tile(Terrain::ground(GroundKind::Building), "Palace")),
            ((7, 2), named_tile(Terrain::ground(GroundKind::Building), "Arsenal")),
        ]);
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(
            board.hex_of_location(Location::Palace),
            Some(HexCoord::new(3, 5))
        );
        assert_eq!(board.hex_of_location(Location::Tuti), None);
    }
}

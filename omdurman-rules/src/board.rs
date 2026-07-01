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

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use omdurman_types::{HexCoord, HexsideKind, HexsideRef, Location, MapData, NileFlow, Terrain};

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
    /// Nile current direction per Nile hex (§5.24). Absent for non-Nile hexes.
    pub nile_flow: IndexMap<HexCoord, NileFlow>,
    /// Terrain per playable hex (§5.11). Absent hexes are treated as off-map.
    pub terrain: IndexMap<HexCoord, Terrain>,
    /// Named landmarks (Palace/Mahdi's Tomb, forts, gates) for victory and
    /// scenario rules (§9.14, §9.344, §9.346).
    pub locations: IndexMap<HexCoord, Location>,
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
            if let Some(flow) = tile.nile_flow {
                board.nile_flow.insert(hex, flow);
            }
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
    pub fn flow_at(&self, hex: HexCoord) -> Option<NileFlow> {
        self.nile_flow.get(&hex).copied()
    }

    /// Classify a single gunboat step `from -> to` against the Nile current
    /// (§5.24). The current at `from` flows *toward* `flow.dir`'s neighbour, so
    /// a step that way is downstream and the opposite way is upstream. Returns
    /// `None` when `from` carries no current annotation (direction unknown) or
    /// `to` is not the up/downstream neighbour.
    pub fn step_direction(&self, from: HexCoord, to: HexCoord) -> Option<StepDirection> {
        let flow = self.flow_at(from)?;
        let neighbors = from.neighbors();
        let downstream = neighbors[flow.dir as usize];
        let upstream = neighbors[(flow.dir as usize + 3) % 6];
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

    /// The hex of a named landmark, if present on this board (§9.14 Mahdi's
    /// Tomb is the [`Location::Palace`] hex of the walled city of Omdurman).
    pub fn hex_of_location(&self, want: Location) -> Option<HexCoord> {
        self.locations
            .iter()
            .find_map(|(hex, loc)| (*loc == want).then_some(*hex))
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

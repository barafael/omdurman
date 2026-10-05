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

use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

use omdurman_types::{HexCoord, HexDirection, HexsideKind, HexsideRef, Location, MapData, Terrain};

/// Deterministic hasher for the engine's board maps. `BoardInfo` is rebuilt
/// from `GameState::default()` on every peer and, crucially, inside the Kani
/// proof harnesses; `IndexMap`/`IndexSet` default their `S` hasher to
/// `std`'s `RandomState`, which seeds from the OS RNG (`getrandom`) on every
/// construction. That foreign `syscall` is unmodellable under Kani and sank
/// every `apply_effect` atomicity proof to UNDETERMINED. SipHash with fixed
/// keys never touches the OS RNG, is what `HashMap::default()` would have
/// used pre-1.x `RandomState`, and keeps `IndexMap`'s insertion-ordered,
/// serde-deterministic behaviour intact — so `BoardInfo` stays honest for
/// rule lookups while the proofs stay tractable.
type DeterministicHasher = BuildHasherDefault<DefaultHasher>;
type Map<K, V> = IndexMap<K, V, DeterministicHasher>;
type Set<T> = IndexSet<T, DeterministicHasher>;

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
    pub hexsides: Map<HexsideRef, HexsideKind>,
    /// Terrain per playable hex (§5.11). Absent hexes are treated as off-map.
    pub terrain: Map<HexCoord, Terrain>,
    /// Named landmarks (Palace/Mahdi's Tomb, forts, gates) for victory and
    /// scenario rules (§9.14, §9.344, §9.346).
    pub locations: Map<HexCoord, Location>,
    /// Road edges (§5.11 Terrain Effects Chart: road movement costs 1 MP
    /// regardless of underlying terrain). Stored as canonical hexside refs.
    #[serde(default)]
    pub roads: Set<HexsideRef>,
    /// Reinforcement entrance areas (§9.112/§9.113), tagged per hex in the
    /// board files via `HexData::named_area`. Empty on boards without entrance
    /// annotations -- callers fall back to geometric approximations.
    #[serde(default)]
    pub entrances: Map<HexCoord, omdurman_types::NamedArea>,
    /// The walled-city hexes (§5.23): the land area enclosed by the annotated
    /// Wall/Gate/Breach ring, derived once at build time by flooding from the
    /// Palace (and, on the Omdurman board, the Mahdi's Tomb). Replaces the
    /// old ">= 2 of 6 hexsides are walls" heuristic, which also flagged the
    /// hexes *outside* the wall that touch its exterior -- making §5.23's
    /// gate porous (audit: units entered Omdurman's "walled city" through
    /// unannotated fringe sides).
    #[serde(default)]
    pub walled_city: Set<HexCoord>,
    /// The Zariba's hexes (§9.211 "the 13 hexes of the Zariba"): the land
    /// enclosed by the Zariba hexsides and the Nile, derived once at build
    /// time (see [`Self::compute_zariba`]). It says which side of a trench
    /// hexside is "the Nile River side" (§9.232) and bounds the
    /// Anglo-Egyptian set-up (§9.211).
    #[serde(default)]
    pub zariba: Set<HexCoord>,
    /// The west bank south of the Khor Shambat (§9.111's fort area, §9.112's
    /// Dervish entry edge): the land reached from the walled city without
    /// crossing the Khor Shambat or the Nile. Empty on boards without it.
    #[serde(default)]
    pub south_of_khor_shambat: Set<HexCoord>,
    /// Per-map-row Nile extent, `(min_q, max_q)` keyed by row `r` (§5.21,
    /// §9.14): the Nile runs roughly north-south, so a hex's bank is decided
    /// by comparing its `q` against the Nile hexes of its row. Derived once
    /// at build time (like `walled_city`); a row holding no Nile hexes
    /// carries the empty extent `(i32::MAX, i32::MIN)`. Absent on boards
    /// serialized before this field existed -- `bank_of` falls back to a
    /// per-call scan there.
    #[serde(default)]
    pub nile_by_row: Map<i32, (i32, i32)>,
    /// Steps from the Palace to each land hex without crossing a city-wall
    /// hexside (wall, gate or breach), on a board whose rampart encloses no
    /// area -- Khartoum, where part of the wall is washed away (§2.1). The
    /// hex of a wall hexside reached in fewer steps is the one inside the
    /// city: the other is only reached round the end of the wall
    /// (see [`Self::is_inside_of_wall`]). Empty on the Omdurman board, whose
    /// walled city is an enclosed area ([`Self::walled_city`]).
    #[serde(default)]
    pub palace_steps: Map<HexCoord, u32>,
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
        // §5.23: derive the walled-city hexes as the area enclosed by the
        // annotated Wall/Gate/Breach ring (see `walled_city`).
        board.walled_city = board.compute_walled_city();
        board.palace_steps = board.compute_palace_steps();
        board.zariba = board.compute_zariba();
        board.south_of_khor_shambat = board.compute_south_of_khor_shambat();
        // Per-row Nile extent for `bank_of` (§5.21): one pass over the
        // terrain instead of a whole-board scan per bank query. Rows without
        // Nile hexes keep the empty `(i32::MAX, i32::MIN)` extent, which
        // `bank_of` reads as "no Nile on this row" (bankless hex).
        for (&coord, &terrain) in &board.terrain {
            let extent = board
                .nile_by_row
                .entry(coord.r)
                .or_insert((i32::MAX, i32::MIN));
            if terrain.is_nile() {
                extent.0 = extent.0.min(coord.q);
                extent.1 = extent.1.max(coord.q);
            }
        }
        board
    }

    /// Flood the west bank from the walled city's landmarks over land,
    /// blocked by the Khor Shambat and the Nile (see
    /// [`Self::south_of_khor_shambat`]). Empty when the board has no Khor
    /// Shambat (FALL OF KHARTOUM).
    pub fn compute_south_of_khor_shambat(&self) -> Set<HexCoord> {
        use omdurman_types::Location;
        let mut south: Set<HexCoord> = Default::default();
        if !self
            .hexsides
            .values()
            .any(|k| *k == HexsideKind::KhorShambat)
        {
            return south;
        }
        let mut queue: std::collections::VecDeque<HexCoord> =
            [Location::Palace, Location::MahdisTomb]
                .iter()
                .filter_map(|loc| self.hex_of_location(*loc))
                .collect();
        for seed in &queue {
            south.insert(*seed);
        }
        while let Some(h) = queue.pop_front() {
            for n in h.neighbors() {
                if south.contains(&n)
                    || self.hexside_between(h, n) == Some(HexsideKind::KhorShambat)
                    || !matches!(self.terrain_at(n), Some(t) if !t.is_nile())
                {
                    continue;
                }
                south.insert(n);
                queue.push_back(n);
            }
        }
        south
    }

    /// The land hexes enclosed by the Zariba hexsides and the Nile (§9.211,
    /// §9.232): flood each side of every Zariba hexside over land, blocked by
    /// Zariba hexsides and the river. A region that reaches the map edge is
    /// the open desert outside; a closed one is the Zariba. Empty on boards
    /// without a closed Zariba line.
    pub fn compute_zariba(&self) -> Set<HexCoord> {
        let is_land =
            |hex: HexCoord| !matches!(self.terrain_at(hex), Some(Terrain::Nile { .. }) | None);
        let mut inside: Set<HexCoord> = Default::default();
        let mut outside: Set<HexCoord> = Default::default();
        for (edge, _) in self.hexsides.iter().filter(|(_, k)| k.is_zariba()) {
            for seed in [edge.a, edge.b] {
                if !is_land(seed) || inside.contains(&seed) || outside.contains(&seed) {
                    continue;
                }
                let mut region: Set<HexCoord> = Default::default();
                region.insert(seed);
                let mut queue = std::collections::VecDeque::from([seed]);
                let mut open = false;
                while let Some(h) = queue.pop_front() {
                    for n in h.neighbors() {
                        if self
                            .hexside_between(h, n)
                            .is_some_and(HexsideKind::is_zariba)
                        {
                            continue;
                        }
                        match self.terrain_at(n) {
                            None => open = true,
                            Some(Terrain::Nile { .. }) => {}
                            Some(_) => {
                                if region.insert(n) {
                                    queue.push_back(n);
                                }
                            }
                        }
                    }
                }
                if open {
                    outside.extend(region);
                } else {
                    inside.extend(region);
                }
            }
        }
        inside
    }

    /// Whether `hex` is one of the Zariba's hexes (see [`Self::zariba`]).
    pub fn is_zariba(&self, hex: HexCoord) -> bool {
        self.zariba.contains(&hex)
    }

    /// Flood the walled city's interior from its landmarks, blocked by
    /// Wall/Gate/Breach hexsides (§5.23). Game-time
    /// `ArtilleryBreachWall` flips keep the set stable (a Breach still bounds
    /// the area -- only passage rules change, §6.63).
    ///
    /// * Omdurman (board carries the Mahdi's Tomb): the fill expands over any
    ///   land hex -- the whole enclosed area counts, whatever its terrain.
    /// * Khartoum / other boards (Tomb absent, e.g. FALL OF KHARTOUM where
    ///   §2.1's washed-away wall section is a legal gap): the fill expands
    ///   only into Building terrain, so the leak through the washed-away
    ///   stretch stops at the city's edge; the Nile and off-map bound it
    ///   elsewhere.
    pub fn compute_walled_city(&self) -> Set<HexCoord> {
        use omdurman_types::Location;
        let mut seeds: Vec<HexCoord> = [Location::Palace, Location::MahdisTomb]
            .iter()
            .filter_map(|loc| self.hex_of_location(*loc))
            .collect();
        seeds.sort_by_key(|h| (h.q, h.r));
        seeds.dedup();
        if seeds.is_empty() {
            return Default::default();
        }
        let omdurman = self.hex_of_location(Location::MahdisTomb).is_some();
        let mut city: Set<HexCoord> = Default::default();
        let mut queue: std::collections::VecDeque<HexCoord> = seeds.into_iter().collect();
        for s in &queue {
            city.insert(*s);
        }
        while let Some(h) = queue.pop_front() {
            for n in h.neighbors() {
                if city.contains(&n) {
                    continue;
                }
                if matches!(
                    self.hexside_between(h, n),
                    Some(HexsideKind::Wall | HexsideKind::Gate | HexsideKind::Breach)
                ) {
                    continue;
                }
                match self.terrain_at(n) {
                    Some(Terrain::Nile { .. }) | None => continue,
                    Some(Terrain::Building { .. }) => {}
                    _ if omdurman => {}
                    _ => continue,
                }
                city.insert(n);
                queue.push_back(n);
            }
        }
        city
    }

    /// The [`Self::palace_steps`] of a board whose rampart is open
    /// (Khartoum): a breadth-first walk from the Palace over land, stopped
    /// by city-wall hexsides. Empty where the Mahdi's Tomb marks Omdurman.
    pub fn compute_palace_steps(&self) -> Map<HexCoord, u32> {
        let mut steps: Map<HexCoord, u32> = Default::default();
        let Some(palace) = self.hex_of_location(Location::Palace) else {
            return steps;
        };
        if self.hex_of_location(Location::MahdisTomb).is_some() {
            return steps;
        }
        steps.insert(palace, 0);
        let mut queue = std::collections::VecDeque::from([(palace, 0u32)]);
        while let Some((hex, n)) = queue.pop_front() {
            for next in hex.neighbors() {
                if steps.contains_key(&next)
                    || matches!(self.terrain_at(next), Some(Terrain::Nile { .. }) | None)
                    || matches!(
                        self.hexside_between(hex, next),
                        Some(HexsideKind::Wall | HexsideKind::Gate | HexsideKind::Breach)
                    )
                {
                    continue;
                }
                steps.insert(next, n + 1);
                queue.push_back((next, n + 1));
            }
        }
        steps
    }

    /// Whether `hex` stands on the city side of a city-wall hexside
    /// (wall, gate or breach) it shares with `across` -- on the ramparts
    /// (§6.3 note b, §5.44). Omdurman's walled city is the enclosed area
    /// ([`Self::walled_city`]); Khartoum's rampart encloses no computable
    /// area (§2.1: part of it is washed away), so there the inside is the
    /// hex reached from the Palace without crossing the wall in fewer steps
    /// ([`Self::palace_steps`]).
    pub fn is_inside_of_wall(&self, hex: HexCoord, across: HexCoord) -> bool {
        if !matches!(
            self.hexside_between(hex, across),
            Some(HexsideKind::Wall | HexsideKind::Gate | HexsideKind::Breach)
        ) {
            return false;
        }
        match (self.is_walled_city(hex), self.is_walled_city(across)) {
            (true, false) => true,
            (false, true) => false,
            _ if !self.palace_steps.is_empty() => {
                let steps = |h| self.palace_steps.get(&h).copied().unwrap_or(u32::MAX);
                steps(hex) < steps(across)
            }
            // A board serialized before `palace_steps` existed.
            _ => self
                .hex_of_location(omdurman_types::Location::Palace)
                .is_some_and(|palace| hex.distance(palace) < across.distance(palace)),
        }
    }

    /// The E–W hexrow "in which the Khor Shambat empties into the Nile"
    /// (§10.11/§10.21): the row of the Nile hex at the khor's mouth -- the
    /// Nile hex sharing a corner with the khor's last hexside. The
    /// southernmost such row when several do; `None` on a board without the
    /// Khor Shambat.
    pub fn khor_shambat_mouth_row(&self) -> Option<i32> {
        self.hexsides
            .iter()
            .filter(|(_, k)| **k == HexsideKind::KhorShambat)
            .flat_map(|(side, _)| {
                side.a
                    .neighbors()
                    .into_iter()
                    .filter(|c| side.b.is_adjacent_to(*c) && self.is_nile(*c))
                    .map(|c| c.r)
                    .collect::<Vec<_>>()
            })
            .max()
    }

    /// The hexside feature on the edge between two hexes, if any (§5.44).
    pub fn hexside_between(&self, a: HexCoord, b: HexCoord) -> Option<HexsideKind> {
        self.hexsides.get(&HexsideRef::new(a, b)).copied()
    }

    /// The terrain at a hex; `None` if the hex is off-map / unannotated (§5.11).
    pub fn terrain_at(&self, hex: HexCoord) -> Option<Terrain> {
        self.terrain.get(&hex).copied()
    }

    /// Whether a road links `from` and `to` (§5.11, Terrain Effects Chart:
    /// moving along a road costs 1 MP). Roads are centre-to-centre links, so
    /// only a step that follows one gets the road rate -- entering a hex a
    /// road merely touches costs its terrain.
    pub fn road_links(&self, from: HexCoord, to: HexCoord) -> bool {
        self.roads.contains(&HexsideRef::new(from, to))
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
    /// (§5.24). The current at `from` flows *toward* `flow.dir`'s neighbour:
    /// a step with the current -- straight down it or 60° off it -- is
    /// downstream; a step against it -- straight up it or 120° off it -- is
    /// upstream ("moving upstream, i.e. against the current"). Returns `None`
    /// when `from` carries no current annotation (direction unknown) or `to`
    /// is not a neighbour.
    pub fn step_direction(&self, from: HexCoord, to: HexCoord) -> Option<StepDirection> {
        let direction = self.flow_at(from)? as usize;
        let k = from.neighbors().iter().position(|n| *n == to)?;
        // The step's bearing relative to the current, in 60° sextants.
        match (k + 6 - direction) % 6 {
            0 | 1 | 5 => Some(StepDirection::Downstream),
            _ => Some(StepDirection::Upstream),
        }
    }

    /// The named landmark at a hex, if any (§9.14, §9.344).
    pub fn location_at(&self, hex: HexCoord) -> Option<Location> {
        self.locations.get(&hex).copied()
    }

    /// Whether `hex` lies inside the walled enclosure (§5.23: "the walled
    /// portion of Omdurman"): membership in the precomputed enclosed set
    /// (see [`Self::walled_city`]; the Palace and Mahdi's Tomb landmark hexes
    /// are its seeds). The set is derived once from the board data, replacing
    /// the older "at least two of six hexsides are Wall/Gate/Breach" heuristic.
    pub fn is_walled_city(&self, hex: HexCoord) -> bool {
        // Membership in the precomputed enclosed area (see `walled_city`).
        // Palace/Tomb hexes are always part of it (they are the seeds).
        self.walled_city.contains(&hex)
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
        let (min_q, max_q) = match self.nile_by_row.get(&hex.r) {
            // Derived board: the row's Nile extent, or the empty sentinel
            // when the row holds no Nile hexes at all (bankless hex).
            Some(&(min, max)) if min <= max => (min, max),
            Some(_) => return None,
            // Board serialized before the derived extent existed (or built
            // field-by-field): derive for this row on the fly, as before.
            None if !self.terrain.is_empty() => {
                let mut min_nile_q: Option<i32> = None;
                let mut max_nile_q: Option<i32> = None;
                for (coord, terrain) in &self.terrain {
                    if coord.r == hex.r && terrain.is_nile() {
                        min_nile_q = Some(min_nile_q.map_or(coord.q, |q: i32| q.min(coord.q)));
                        max_nile_q = Some(max_nile_q.map_or(coord.q, |q: i32| q.max(coord.q)));
                    }
                }
                (min_nile_q?, max_nile_q?)
            }
            // No board loaded: nothing is on a bank.
            None => return None,
        };
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
    use traceability_macro::rulebook;

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
        HexData::new(terrain, Some(name.to_string()))
    }

    // -- from_map_data --------------------------------------------------

    #[test]
    fn from_map_data_populates_terrain_and_nile() {
        let map = make_map(vec![
            ((0, 0), tile(Terrain::default())),
            ((1, 0), nile_tile(HexDirection::SouthEast)),
            ((2, 1), tile(Terrain::ground(GroundKind::Hilltop))),
        ]);
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(
            board.terrain_at(HexCoord::new(0, 0)),
            Some(Terrain::default())
        );
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
        assert_eq!(
            board.terrain_at(HexCoord::new(0, 0)),
            Some(Terrain::default())
        );
        assert_eq!(board.terrain_at(HexCoord::new(1, 1)), None);
    }

    // -- bank_of ----------------------------------------------------------

    /// A Nile hex has neither bank, land west of the row's Nile is the west
    /// bank, land east of it the east bank, and a row without Nile hexes at
    /// all is bankless (§5.21).
    #[test]
    fn bank_of_reads_the_derived_row_extents() {
        let map = make_map(vec![
            ((-2, 0), tile(Terrain::default())),
            ((0, 0), nile_tile(HexDirection::SouthEast)),
            ((2, 0), tile(Terrain::default())),
            // A second row with no Nile at all.
            ((0, 1), tile(Terrain::default())),
        ]);
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(board.bank_of(HexCoord::new(0, 0)), None); // in the Nile
        assert_eq!(board.bank_of(HexCoord::new(-2, 0)), Some(NileBank::West));
        assert_eq!(board.bank_of(HexCoord::new(2, 0)), Some(NileBank::East));
        assert_eq!(board.bank_of(HexCoord::new(0, 1)), None); // no Nile on row
        // Rows absent from the board are bankless too.
        assert_eq!(board.bank_of(HexCoord::new(0, 5)), None);
    }

    /// A board serialized before the derived row extents existed (empty
    /// `nile_by_row` but populated terrain) answers `bank_of` identically
    /// through the per-call fallback scan.
    #[test]
    fn bank_of_legacy_board_without_row_extents_agrees() {
        let map = make_map(vec![
            ((-2, 0), tile(Terrain::default())),
            ((0, 0), nile_tile(HexDirection::SouthEast)),
            ((2, 0), tile(Terrain::default())),
            ((0, 1), tile(Terrain::default())),
        ]);
        let mut board = BoardInfo::from_map_data(&map);
        board.nile_by_row.clear();
        assert_eq!(board.bank_of(HexCoord::new(0, 0)), None);
        assert_eq!(board.bank_of(HexCoord::new(-2, 0)), Some(NileBank::West));
        assert_eq!(board.bank_of(HexCoord::new(2, 0)), Some(NileBank::East));
        assert_eq!(board.bank_of(HexCoord::new(0, 1)), None);
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
            (
                (1, 0),
                entrance(omdurman_types::NamedArea::AngloEgyptianEntrance),
            ),
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
        assert!(
            board
                .entrance_hexes(omdurman_types::NamedArea::AbuAlimHut)
                .is_empty()
        );
    }

    #[test]
    fn from_map_data_promotes_landmarks() {
        let map = make_map(vec![
            (
                (3, 5),
                named_tile(Terrain::ground(GroundKind::Building), "Palace"),
            ),
            (
                (2, 4),
                named_tile(Terrain::ground(GroundKind::Building), "North Fort"),
            ),
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
            Terrain::Nile {
                direction: HexDirection::East,
            },
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
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );
        let from = HexCoord::new(2, 3);
        let upstream = from.neighbors()[3]; // West neighbor
        assert_eq!(
            board.step_direction(from, upstream),
            Some(StepDirection::Upstream)
        );
    }

    // §5.24: "upstream, i.e. against the current" -- a step 120° off the
    // current is against it, a step 60° off it is with it.
    #[rulebook("§5.24")]
    #[test]
    fn step_direction_oblique_steps_follow_the_current() {
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(2, 3),
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );
        let from = HexCoord::new(2, 3);
        let n = from.neighbors();
        for k in [1, 5] {
            assert_eq!(
                board.step_direction(from, n[k]),
                Some(StepDirection::Downstream),
                "60° off the current, neighbour {k}"
            );
        }
        for k in [2, 4] {
            assert_eq!(
                board.step_direction(from, n[k]),
                Some(StepDirection::Upstream),
                "120° off the current, neighbour {k}"
            );
        }
        assert_eq!(board.step_direction(from, HexCoord::new(9, 9)), None);
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
        board.terrain.insert(
            HexCoord::new(5, 3),
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );
        // West hex has q < 5.
        assert_eq!(board.bank_of(HexCoord::new(3, 3)), Some(NileBank::West));
    }

    #[test]
    fn bank_of_east_of_nile() {
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(5, 3),
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );
        // East hex has q > 5.
        assert_eq!(board.bank_of(HexCoord::new(8, 3)), Some(NileBank::East));
    }

    #[test]
    fn bank_of_hex_on_nile_returns_none() {
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(5, 3),
            Terrain::Nile {
                direction: HexDirection::East,
            },
        );
        // The hex is itself in the Nile channel.
        assert_eq!(board.bank_of(HexCoord::new(5, 3)), None);
    }

    #[test]
    fn bank_of_no_nile_on_row_returns_none() {
        let mut board = BoardInfo::default();
        // Only Clear terrain on row 3 — no Nile.
        board
            .terrain
            .insert(HexCoord::new(5, 3), Terrain::default());
        assert_eq!(board.bank_of(HexCoord::new(3, 3)), None);
    }

    #[test]
    fn bank_of_empty_board_returns_none() {
        let board = BoardInfo::default();
        assert_eq!(board.bank_of(HexCoord::new(0, 0)), None);
    }

    // -- hex_of_location -------------------------------------------------

    #[test]
    fn hex_of_location_finds_correct_hex() {
        let map = make_map(vec![
            (
                (3, 5),
                named_tile(Terrain::ground(GroundKind::Building), "Palace"),
            ),
            (
                (7, 2),
                named_tile(Terrain::ground(GroundKind::Building), "Arsenal"),
            ),
        ]);
        let board = BoardInfo::from_map_data(&map);
        assert_eq!(
            board.hex_of_location(Location::Palace),
            Some(HexCoord::new(3, 5))
        );
        assert_eq!(board.hex_of_location(Location::Tuti), None);
    }
}

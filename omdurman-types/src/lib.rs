use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

pub mod section_name;
pub use section_name::SectionName;

/// Pixel bounding box of the campaign turn-track on the campaign-map image.
/// The track is a 9 × 3 grid; turn positions are auto-computed from this box
/// (snake layout: row 0 L→R, row 1 R→L, row 2 L→R, only 4 cells of row 2).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CampaignTurnTrack {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// The calibrated placement of one chart table on its scan: a bounding box plus
/// the left label-column width and top header-row height that are excluded from
/// the even data grid. All coordinates are fractions of the scan's width/height
/// in `[0, 1]`, so they survive the scan being rescaled.
///
/// This is *only* the geometry. The set of tables per chart, their row/column
/// counts, and their cell labels are fixed in code (inferred from the printed
/// scans -- see `charts::chart_layout`); a `ChartBox` is index-aligned with that
/// code list. Calibrated in-app via the editor's Charts tab.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ChartBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Left label-column width, fraction of `w`, excluded from the data grid.
    #[serde(default)]
    pub label_w: f32,
    /// Top header-row height, fraction of `h`, excluded from the data grid.
    #[serde(default)]
    pub header_h: f32,
}

/// Calibrated boxes for the chart scans, keyed by the chart's stable string id
/// (`"crt"`, `"terrain"`, `"timing"`, `"arrivals"`); the `Vec` is index-aligned
/// with the code's fixed table list for that chart. Global (charts are
/// board-independent) and defaulted, so older annotation files still load.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ChartBoxes(BTreeMap<String, Vec<ChartBox>>);

impl ChartBoxes {
    pub fn boxes(&self, chart: &str) -> &[ChartBox] {
        self.0.get(chart).map(Vec::as_slice).unwrap_or(&[])
    }
    pub fn boxes_mut(&mut self, chart: &str) -> &mut Vec<ChartBox> {
        self.0.entry(chart.to_string()).or_default()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SpriteRef {
    pub section_name: SectionName,
    pub col: u32,
    pub row: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnitGrid {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub cols: u32,
    pub rows: u32,
}

/// Hex-grid coordinate in axial form (rulebook §5, §6).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    /// Create a new hex coordinate (rulebook §5, §6).
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// The six axial neighbours, in a fixed order (rulebook §5, §6).
    /// The neighbour convention (which `(q, r)` offsets are adjacent) is
    /// defined here so movement, targeting, and overlays all agree on adjacency.
    pub fn neighbors(self) -> [HexCoord; 6] {
        let HexCoord { q, r } = self;
        [
            HexCoord::new(q + 1, r),
            HexCoord::new(q + 1, r + 1),
            HexCoord::new(q, r + 1),
            HexCoord::new(q - 1, r),
            HexCoord::new(q - 1, r - 1),
            HexCoord::new(q, r - 1),
        ]
    }

    /// Whether `other` is one of the six axial neighbours (rulebook §5, §6).
    pub fn is_adjacent_to(self, other: HexCoord) -> bool {
        self.neighbors().contains(&other)
    }

    /// Hex distance (cube max-norm) between two coordinates, consistent with
    /// the [`neighbors`](Self::neighbors) adjacency (adjacent hexes are at
    /// distance 1) (rulebook §6.22).
    pub fn distance(self, other: HexCoord) -> u32 {
        let dq = (self.q - other.q).unsigned_abs();
        let dr = (self.r - other.r).unsigned_abs();
        let ds = (self.q + self.r - other.q - other.r).unsigned_abs();
        dq.max(dr).max(ds / 2)
    }

    /// The hexes strictly *between* `self` and `other` (endpoints excluded),
    /// in order from `self` toward `other`. Empty for adjacent or identical
    /// hexes. Used for line-of-sight (rulebook §6.3): each step picks the
    /// neighbour that most reduces the remaining distance, so the path is
    /// consistent with this grid's [`neighbors`](Self::neighbors)/
    /// [`distance`](Self::distance) convention regardless of the underlying
    /// coordinate layout.
    pub fn line_between(self, other: HexCoord) -> Vec<HexCoord> {
        let mut path = Vec::new();
        let mut current = self;
        // Bound the walk by the distance to guard against any pathological
        // non-decreasing step (cannot happen for a valid hex grid).
        let max_steps = self.distance(other);
        for _ in 0..max_steps {
            if current == other {
                break;
            }
            let next = current
                .neighbors()
                .into_iter()
                .min_by_key(|n| n.distance(other))
                .expect("hex always has six neighbours");
            if next == other {
                break;
            }
            path.push(next);
            current = next;
        }
        path
    }
}

/// Reference to a specific hex-side (the edge shared by two adjacent hexes)
/// (rulebook §5.23, §5.44, §6.3, §7.2). Endpoints are stored in canonical
/// (low->high) order so the same physical edge always compares and hashes equal
/// regardless of which side names it -- this lets a map key per-edge hexside
/// data by [`HexsideRef`].
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HexsideRef {
    pub a: HexCoord,
    pub b: HexCoord,
}

impl HexsideRef {
    /// Canonicalised edge between two adjacent hexes (order-independent) (rulebook §5.23).
    pub fn new(a: HexCoord, b: HexCoord) -> Self {
        if (a.q, a.r) <= (b.q, b.r) {
            HexsideRef { a, b }
        } else {
            HexsideRef { a: b, b: a }
        }
    }

    #[cfg(test)]
    fn separates(self, from: HexCoord, to: HexCoord) -> bool {
        self == HexsideRef::new(from, to)
    }
}

/// The kind of feature on a hex-side (rulebook §5.23, §5.44, §6.3, §7.2).
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Default,
    strum::Display,
    strum::EnumIter,
)]
pub enum HexsideKind {
    /// City wall (Khartoum, walled city of Omdurman). Blocks LOS, blocks
    /// movement except across gates/breaches (§5.23), blocks ZOC into the city
    /// (§5.44), blocks melee (§7.2), blocks advance-after-combat (§6.82).
    #[default]
    Wall,
    /// Gate hexside in a wall. ZOC extends *out of* the walled city through
    /// gates but not into it (§5.44). Melee may be made through a gate (§7.2).
    Gate,
    /// Breach in a wall (artillery/§6.63 or Royal Engineers/§6.53). ZOC both
    /// ways; LOS no longer blocked across the hexside.
    Breach,
    /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
    /// combat may not cross (§6.82).
    Khor,
    /// Crest line. Blocks LOS unless the firer is on the higher side
    /// (§6.3 note 7).
    Crest,
    /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
    ZaribaThornHedge,
    /// Historical-scenario trench segment of the Zariba (§9.232).
    ZaribaTrench,
    /// One of the two end hexsides of a Zariba trench segment that connect to
    /// the Nile River (§9.233).  Units may only enter/leave the Zariba via
    /// these end hexsides (paying +2 MP).  Behaviour is identical to
    /// [`ZaribaTrench`](HexsideKind::ZaribaTrench) for all classifiers except
    /// [`is_zariba_trench_end`](HexsideKind::is_zariba_trench_end).
    ZaribaTrenchEndA,
    /// The other end hexside of a Zariba trench segment (§9.233).
    /// See [`ZaribaTrenchEndA`](HexsideKind::ZaribaTrenchEndA).
    ZaribaTrenchEndB,
    /// Khor Shambat -- the specific named khor that empties into the Nile (a
    /// scenario landmark; used as a setup/reinforcement boundary). Same blocking
    /// rules as a generic [`Khor`](HexsideKind::Khor), but distinctly named so it
    /// can be marked on the map. Appended last for repr stability.
    KhorShambat,
}

impl HexsideKind {
    /// Whether this hexside blocks line of sight across it (§6.3). Returns
    /// `true` for Wall and Crest hexsides. The directional Crest exceptions
    /// (LOS table conditions 2–4, 7) and note (e) are handled by the engine
    /// in `omdurman_rules::los_table`, not by this predicate.
    pub fn blocks_los(self) -> bool {
        matches!(self, HexsideKind::Wall | HexsideKind::Crest)
    }

    /// Whether melee may *not* be made across this side (§7.2). Gates and
    /// breaches are passable to melee.
    pub fn blocks_melee(self) -> bool {
        matches!(self, HexsideKind::Wall | HexsideKind::ZaribaThornHedge)
    }

    /// Whether advance-after-combat may *not* cross this side (§6.82, §7.6).
    pub fn blocks_advance_after_combat(self) -> bool {
        matches!(
            self,
            HexsideKind::Wall
                | HexsideKind::Khor
                | HexsideKind::KhorShambat
                | HexsideKind::ZaribaThornHedge
        )
    }

    /// Whether land movement may *not* cross this side (§5.23). Walls block
    /// movement except at gates/breaches.
    pub fn blocks_movement(self) -> bool {
        matches!(self, HexsideKind::Wall)
    }

    /// Whether a zone of control may *not* extend across this side (§5.44).
    /// "ZOCs do not extend across a khor, into a fort, or into a hex inside the
    /// walled city across a wall hexside... ZOCs extend both ways across a
    /// breach hexside." Gates and breaches do not block ZOC; walls and khors do.
    /// Crests are line-of-sight only and do not block ZOC.
    ///
    /// The directional "out of, but not into" cases (gate, hut/building, Zariba)
    /// depend on which hex the projecting unit stands in, which a single hexside
    /// cannot express; those are left to the caller. This predicate captures the
    /// symmetric "does not extend across" cases.
    pub fn blocks_zoc(self) -> bool {
        matches!(
            self,
            HexsideKind::Wall
                | HexsideKind::Khor
                | HexsideKind::KhorShambat
                | HexsideKind::ZaribaThornHedge
                | HexsideKind::ZaribaTrench
                | HexsideKind::ZaribaTrenchEndA
                | HexsideKind::ZaribaTrenchEndB
        )
    }

    /// Whether this hexside is one of the two Zariba trench ends that connect
    /// to the Nile River (§9.233).  Units may only enter/leave the Zariba via
    /// these end hexsides.
    pub fn is_zariba_trench_end(self) -> bool {
        matches!(
            self,
            HexsideKind::ZaribaTrenchEndA | HexsideKind::ZaribaTrenchEndB
        )
    }
}

/// Compass direction in a hex grid, matching the canonical neighbour order
/// (`+q`, `+q+r`, `+r`, `-q`, `-q-r`, `-r` for pointy-top hexes) (rulebook §5.11, §5.24).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum HexDirection {
    #[default]
    East = 0,
    SouthEast = 1,
    SouthWest = 2,
    West = 3,
    NorthWest = 4,
    NorthEast = 5,
}

impl std::fmt::Display for HexDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HexDirection::East => write!(f, "E"),
            HexDirection::SouthEast => write!(f, "SE"),
            HexDirection::SouthWest => write!(f, "SW"),
            HexDirection::West => write!(f, "W"),
            HexDirection::NorthWest => write!(f, "NW"),
            HexDirection::NorthEast => write!(f, "NE"),
        }
    }
}

impl HexDirection {
    /// Recover a direction from its neighbour index (taken mod 6) (rulebook §5.11, §5.24).
    pub fn from_index(n: u8) -> Self {
        match n % 6 {
            0 => HexDirection::East,
            1 => HexDirection::SouthEast,
            2 => HexDirection::SouthWest,
            3 => HexDirection::West,
            4 => HexDirection::NorthWest,
            _ => HexDirection::NorthEast,
        }
    }
}

/// Road state for a ground hex (§5.11 Terrain Effects Chart).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Road {
    /// No road touching this hex.
    #[default]
    None,
    /// A road touches this hex but stops at the edge.
    Road,
    /// Roads converge at this hex's centre (crossroad).
    Crossroad,
}

/// Hex terrain types used on the Omdurman map (rulebook Terrain Effects Chart,
/// §5.11, §6.23, §6.3).
///
/// Ground variants carry a [`Road`] flag. The Nile variant carries the current
/// direction for gunboat upstream/downstream movement (§5.24).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
pub enum Terrain {
    Clear { road: Road },
    Rough { road: Road },
    Trees { road: Road },
    Swamp { road: Road },
    Nile { direction: HexDirection },
    Hilltop { road: Road },
    Huts { road: Road },
    Building { road: Road },
}

impl Default for Terrain {
    fn default() -> Self {
        Self::Clear { road: Road::default() }
    }
}

/// The underlying ground type of a hex, stripped of road state.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    strum::Display,
    strum::EnumIter,
    strum::FromRepr,
)]
#[repr(u8)]
pub enum GroundKind {
    Clear,
    Rough,
    Trees,
    Swamp,
    Hilltop,
    Huts,
    Building,
}

impl Terrain {
    /// Convenience constructor: ground terrain with no road.
    pub fn ground(kind: GroundKind) -> Self {
        Self::ground_with_road(kind, Road::None)
    }

    /// Ground terrain with explicit road state.
    pub fn ground_with_road(kind: GroundKind, road: Road) -> Self {
        match kind {
            GroundKind::Clear => Self::Clear { road },
            GroundKind::Rough => Self::Rough { road },
            GroundKind::Trees => Self::Trees { road },
            GroundKind::Swamp => Self::Swamp { road },
            GroundKind::Hilltop => Self::Hilltop { road },
            GroundKind::Huts => Self::Huts { road },
            GroundKind::Building => Self::Building { road },
        }
    }

    /// Strip road state, returning the underlying ground kind.
    /// Returns `None` for Nile, which has no `GroundKind` equivalent.
    pub fn ground_kind(self) -> Option<GroundKind> {
        Some(match self {
            Terrain::Clear { .. } => GroundKind::Clear,
            Terrain::Rough { .. } => GroundKind::Rough,
            Terrain::Trees { .. } => GroundKind::Trees,
            Terrain::Swamp { .. } => GroundKind::Swamp,
            Terrain::Nile { .. } => return None,
            Terrain::Hilltop { .. } => GroundKind::Hilltop,
            Terrain::Huts { .. } => GroundKind::Huts,
            Terrain::Building { .. } => GroundKind::Building,
        })
    }

    /// Whether this terrain may be entered by land units (rulebook §5.11).
    pub fn passable_by_land(self) -> bool {
        !self.is_nile()
    }

    /// Whether an intervening hex of this terrain unconditionally blocks
    /// line of sight in the *simple* LOS model (§6.3). The full LOS table
    /// in `omdurman_rules::los_table` handles the conditional blocking
    /// (footnotes 1–7); this predicate is retained for compatibility and
    /// returns `true` only for Huts and Building (the always-blocking
    /// built-up terrain types).
    pub fn blocks_los(self) -> bool {
        matches!(self, Terrain::Huts { .. } | Terrain::Building { .. })
    }

    /// Whether this terrain counts as "trees" for the LOS palm-grove rule
    /// (§6.3 note 1). Retained for compatibility; the full LOS engine
    /// checks `Terrain::Trees` directly.
    pub fn is_los_trees(self) -> bool {
        matches!(self, Terrain::Trees { .. })
    }

    /// Whether this terrain is the Nile river (rulebook §5.11, §5.24).
    pub fn is_nile(self) -> bool {
        matches!(self, Terrain::Nile { .. })
    }

    /// The Nile current direction, if this is a Nile hex (§5.24).
    pub fn nile_direction(self) -> Option<HexDirection> {
        match self {
            Terrain::Nile { direction } => Some(direction),
            _ => None,
        }
    }

    /// Rotate the Nile current by `delta` steps (positive = clockwise).
    /// No-op for non-Nile terrain.
    pub fn with_rotated_flow(self, delta: i8) -> Self {
        match self {
            Terrain::Nile { direction } => {
                let d = ((direction as i8) + delta).rem_euclid(6);
                Terrain::Nile { direction: HexDirection::from_index(d as u8) }
            }
            other => other,
        }
    }

    /// The road state for ground terrain ([`Road::None`] for Nile).
    pub fn road(self) -> Road {
        match self {
            Terrain::Clear { road }
            | Terrain::Rough { road }
            | Terrain::Trees { road }
            | Terrain::Swamp { road }
            | Terrain::Hilltop { road }
            | Terrain::Huts { road }
            | Terrain::Building { road } => road,
            Terrain::Nile { .. } => Road::None,
        }
    }

    /// Whether this hex has any road touching it.
    pub fn has_road(self) -> bool {
        !matches!(self.road(), Road::None)
    }

    /// Whether roads converge at this hex's centre.
    pub fn is_crossroad(self) -> bool {
        matches!(self.road(), Road::Crossroad)
    }

    /// Return a copy with the road state changed (no-op for Nile).
    pub fn with_road(self, road: Road) -> Self {
        match self {
            Terrain::Clear { .. } => Terrain::Clear { road },
            Terrain::Rough { .. } => Terrain::Rough { road },
            Terrain::Trees { .. } => Terrain::Trees { road },
            Terrain::Swamp { .. } => Terrain::Swamp { road },
            Terrain::Hilltop { .. } => Terrain::Hilltop { road },
            Terrain::Huts { .. } => Terrain::Huts { road },
            Terrain::Building { .. } => Terrain::Building { road },
            Terrain::Nile { .. } => self,
        }
    }
}

/// Named map landmarks (rulebook mapsheet, §9.111, §9.113, §9.212 scenarios).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
pub enum Location {
    FortMakran,
    NorthFort,
    FortBuri,
    AustrianMission,
    Palace,
    Arsenal,
    Barracks,
    KalaklaGate,
    MessalamiaGate,
    BuriGate,
    Tuti,
    Hogali,
    BuriSettlement,
    /// The off-board mouth of the White Nile branch (FALL OF KHARTOUM §9.345) --
    /// a British gunboat may cross to the Blue Nile mouth for 6 upstream MP.
    WhiteNileMouth,
    /// The off-board mouth of the Blue Nile branch (FALL OF KHARTOUM §9.345).
    BlueNileMouth,
    /// The Mahdi's Tomb hex in the walled city of Omdurman (§9.14). Distinct
    /// from [`Location::Palace`]: on the Campaign map the Palace and the Tomb
    /// are at different hexes. Worth 25 VP to the Anglo-Egyptian player if
    /// held at the conclusion of play.
    MahdisTomb,
}

/// Rules-significant named areas spanning multiple hexes (rulebook §9.113).
/// Currently only the Anglo-Egyptian entrance area on the west bank of the
/// Campaign map, where reinforcements enter paying 1 MP per hex.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
pub enum NamedArea {
    AngloEgyptianEntrance,
}

impl Location {
    /// Map a board tile's human-readable `name` (e.g. `"North Fort"`) to its
    /// [`Location`] landmark, if it is one. Names are authored in the map editor
    /// and carry spaces, so the match is on the printed label, case-insensitively.
    /// Returns `None` for ordinary named hexes (villages, etc.) that are not
    /// rules-significant landmarks.
    pub fn from_tile_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "fort makran" => Some(Location::FortMakran),
            "north fort" => Some(Location::NorthFort),
            "fort buri" => Some(Location::FortBuri),
            "austrian mission" => Some(Location::AustrianMission),
            "palace" => Some(Location::Palace),
            "arsenal" => Some(Location::Arsenal),
            "barracks" => Some(Location::Barracks),
            "kalakla gate" => Some(Location::KalaklaGate),
            "messalamia gate" => Some(Location::MessalamiaGate),
            "buri gate" => Some(Location::BuriGate),
            "tuti" => Some(Location::Tuti),
            "hogali" => Some(Location::Hogali),
            "buri" => Some(Location::BuriSettlement),
            "white nile mouth" => Some(Location::WhiteNileMouth),
            "blue nile mouth" => Some(Location::BlueNileMouth),
            "mahdi's tomb" | "mahdis tomb" => Some(Location::MahdisTomb),
            _ => None,
        }
    }
}

/// Map-legend set-up hex codes used in the Historical scenario (rulebook §9.212).
/// Each letter marks a specific hex where a Dervish leader is placed.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
pub enum SetupLetter {
    Y,
    K,
    S,
    O,
    D,
    A,
}

/// Per-hex map data (rulebook mapsheet, §5.11, §6.23, §6.3).
///
/// Road state lives on the [`Terrain`] variant; this struct adds only the
/// display name, location landmark, setup letter, scattergram flag, and named
/// area.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HexData {
    pub terrain: Terrain,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup_letter: Option<SetupLetter>,
    /// Whether this hex is one of the seven printed Howitzer Fire Scattergram
    /// reference hexes (rulebook §6.64). Purely a visual annotation -- all
    /// scattergram hexes are regular playable hexes.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_scattergram: bool,
    /// The rules-significant named area this hex belongs to, if any
    /// (e.g. the Anglo-Egyptian entrance area, rulebook §9.113).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub named_area: Option<NamedArea>,
}

fn is_false(b: &bool) -> bool {
    !b
}

impl HexData {
    pub fn new(terrain: Terrain, name: Option<String>) -> Self {
        Self {
            terrain,
            location: None,
            name,
            setup_letter: None,
            is_scattergram: false,
            named_area: None,
        }
    }
}

#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display, strum::EnumIter,
)]
pub enum SpriteColor {
    BlackWhite,
    GreenRed,
    RedBlack,
    GrayBlack,
    WhiteBlack,
    GrayRed,
    SandBlack,
    BlueBlack,
    BlueRed,
    GreenBlack,
    SandRed,
    SandGreen,
    WhiteSand,
}

/// Dervish tribal/sub-faction identity. Drives the colour-based stacking
/// restriction (§5.52) and the leader->troops command match (§5.53).
#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display, strum::EnumIter,
)]
pub enum DervishTribe {
    Baggara,
    Jaalin,
    Danagla,
    Kehena,
    Degheim,
    Hadendowa,
    Mulazmin,
    Jehadia,
    /// The Khalifa's bodyguard (§9.111 -- may enter the walled city).
    Taiasha,
    /// East-bank infantry (§9.111).
    IsaZachneih,
}

/// The two major factions in the battle (rulebook §2).
///
/// Each carries the identifying information printed on the counter:
/// Dervish units have a tribe; Anglo-Egyptian infantry have an optional
/// brigade designation (`None` = no brigade printed; `Some(BrigadeId::*)`
/// = the printed designation). For Friendlies, the editor sets
/// `Some(BrigadeId::friendlies())`.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Faction {
    Dervish { tribe: DervishTribe },
    BritishEgyptian {
        #[serde(default, deserialize_with = "deserialize_brigade_option")]
        brigade: Option<BrigadeId>,
    },
}

impl std::fmt::Display for Faction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Faction::Dervish { .. } => f.write_str("Dervish"),
            Faction::BritishEgyptian { .. } => f.write_str("BritishEgyptian"),
        }
    }
}

/// The two sides referenced everywhere in the rulebook (rulebook §2). Distinct
/// from [`crate::Faction`] which also includes `Independent`; rule resolution
/// always picks between exactly these two.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum Player {
    AngloEgyptian,
    Dervish,
}

impl Player {
    /// Return the opposing player (rulebook §2).
    pub fn opponent(self) -> Player {
        match self {
            Player::AngloEgyptian => Player::Dervish,
            Player::Dervish => Player::AngloEgyptian,
        }
    }
}

/// A game turn is either a day turn or a night turn; night turns halve all
/// Anglo-Egyptian movement and all fire ranges, and forbid howitzer fire
/// (rulebook §8.1).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DayNight {
    Day,
    Night,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, strum::Display)]
pub enum Scenario {
    /// 9.1 -- 22 game turns, 6:00 am Sept 1 -> 8:00 am Sept 3.
    #[default]
    Campaign,
    /// 9.2 -- 4 game turns, 6:00 am -> 12:00 noon Sept 2.
    Historical,
    /// 9.3 -- variable length, see victory conditions.
    FallOfKhartoum,
}

impl Scenario {
    /// Printed label for the scenario / board picker.
    pub fn label(self) -> &'static str {
        match self {
            Scenario::Campaign => "Campaign",
            Scenario::Historical => "Historical",
            Scenario::FallOfKhartoum => "Fall of Khartoum",
        }
    }

    /// All scenarios in rulebook order (§9.1, §9.2, §9.3).
    pub const ALL: [Scenario; 3] = [
        Scenario::Campaign,
        Scenario::Historical,
        Scenario::FallOfKhartoum,
    ];
}

const fn default_true() -> bool {
    true
}

/// What this unit *is* (rulebook §2.3) -- drives every special-capability
/// branch in the rules. Selected via a dropdown in the unit-annotation screen.
///
/// Used directly as the `SpriteAnnotation::kind` value for a real unit
/// (`Some(UnitKind::...)`); a non-unit marker counter carries `None` instead.
/// Notice that `Infantry`, `Cavalry`, `Camel`, and `DervishLeaderUnit` are the
/// only kinds that may *attack* in melee (§7.4).
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    strum::Display,
    strum::EnumIter,
)]
pub enum UnitKind {
    /// Foot infantry. Includes Anglo-Egyptian infantry, "Friendlies",
    /// Royal Engineers, and Dervish foot tribes.
    Infantry,
    Cavalry,
    Camel,
    Artillery,
    Maxim,
    Gunboat,
    /// Permanent emplacement -- may not move once placed (§5.25).
    Fort,
    /// Dervish leader: has fire/melee/movement factors and may melee attack.
    DervishLeaderUnit,
    /// Anglo-Egyptian leader: movement only (§6.51).
    BritishLeaderUnit,
}

impl UnitKind {
    /// Rulebook §7.4 -- only infantry, cavalry, camel and Dervish leaders may
    /// melee *attack*. All others (except gunboats) may melee *defend* (§7.1).
    pub fn may_melee_attack(self) -> bool {
        matches!(
            self,
            UnitKind::Infantry | UnitKind::Cavalry | UnitKind::Camel | UnitKind::DervishLeaderUnit
        )
    }

    /// Gunboats neither attack nor are attacked in melee (§7.1).
    pub fn may_be_melee_attacked(self) -> bool {
        !matches!(self, UnitKind::Gunboat)
    }

    /// Cavalry and camel units may retreat two hexes from an infantry melee
    /// attack (§7.5).
    pub fn may_retreat_before_melee(self) -> bool {
        matches!(self, UnitKind::Cavalry | UnitKind::Camel)
    }

    /// Gunboats use the split upstream/downstream movement allowance (§5.24).
    pub fn is_boat(self) -> bool {
        matches!(self, UnitKind::Gunboat)
    }

    /// British leaders print a movement factor only (§6.51); other kinds carry
    /// fire and/or melee factors.
    pub fn has_combat_factors(self) -> bool {
        !matches!(self, UnitKind::BritishLeaderUnit)
    }

    /// Maxim guns fire twice per turn -- once in the Direct Fire Subphase and
    /// again in the Maxim Second Fire Subphase (rulebook §6.42). The counter
    /// is marked "x2" in the editor to surface this.
    pub fn fires_twice(self) -> bool {
        matches!(self, UnitKind::Maxim)
    }
}

/// Default value for `SpriteAnnotation::kind` so older `.ron` files that
/// predate the `kind` field still load: a real unit, classified as Infantry,
/// matching the legacy `#[derive(Default)]` on the deleted `UnitFormKind`.
fn default_unit_kind() -> Option<UnitKind> {
    Some(UnitKind::Infantry)
}

/// Best-effort classification of a legacy annotation that predates the `kind`
/// field, from its `is_boat` / `is_unit` flags. Returns `None` (Marker) for a
/// non-unit counter, `Some(Gunboat)` for a boat, otherwise `Some(Infantry)`.
pub fn unit_kind_from_legacy_flags(is_boat: bool, is_unit: bool) -> Option<UnitKind> {
    if !is_unit {
        None
    } else if is_boat {
        Some(UnitKind::Gunboat)
    } else {
        Some(UnitKind::Infantry)
    }
}

/// Nationality of an Anglo-Egyptian infantry brigade, as printed on the
/// counter's brigade designation (rulebook §5.54).
#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display, strum::EnumIter,
)]
pub enum BrigadeNationality {
    /// `xB` -- British.
    British,
    /// `xE` -- Egyptian.
    Egyptian,
    /// `xS` -- Sudanese.
    Sudanese,
    /// Native volunteer brigade -- the Shaggyeh (§6.52). Do not receive
    /// brigade integrity (§5.54 enumerates only British/Egyptian/Sudanese).
    Friendlies,
}

impl BrigadeNationality {
    /// Single-letter suffix used in the printed designation (`B`/`E`/`S`/`F`).
    pub fn letter(self) -> char {
        match self {
            BrigadeNationality::British => 'B',
            BrigadeNationality::Egyptian => 'E',
            BrigadeNationality::Sudanese => 'S',
            BrigadeNationality::Friendlies => 'F',
        }
    }
}

/// Anglo-Egyptian infantry brigade designation printed on the counter
/// (rulebook §2.3, §5.54). The number is the brigade ordinal as printed, e.g.
/// `BrigadeId { number: 3, nationality: Egyptian }` -> the printed `3E`.
///
/// In contexts that distinguish "no brigade" from a specific one (notably the
/// [`Faction::BritishEgyptian`] field and the editor's brigade picker) the
/// `Option<BrigadeId>` representation is used: `None` carries no brigade
/// designation, `Some(...)` carries one. "Friendlies" units
/// ([`BrigadeNationality::Friendlies`]) are modelled with `Some(BrigadeId {
/// nationality: Friendlies, .. })`; they do not receive brigade integrity
/// (§5.54 enumerates only British/Egyptian/Sudanese) but ride along on the
/// same field for uniform handling.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BrigadeId {
    pub number: u8,
    pub nationality: BrigadeNationality,
}

impl BrigadeId {
    /// The twelve Anglo-Egyptian brigade designations that may claim brigade
    /// integrity (rulebook §5.54). Friendlies is intentionally excluded -- it
    /// never integrates -- and is set on the editor dropdown separately.
    pub const ALL: [BrigadeId; 12] = [
        BrigadeId { number: 1, nationality: BrigadeNationality::British },
        BrigadeId { number: 2, nationality: BrigadeNationality::British },
        BrigadeId { number: 3, nationality: BrigadeNationality::British },
        BrigadeId { number: 4, nationality: BrigadeNationality::British },
        BrigadeId { number: 1, nationality: BrigadeNationality::Egyptian },
        BrigadeId { number: 2, nationality: BrigadeNationality::Egyptian },
        BrigadeId { number: 3, nationality: BrigadeNationality::Egyptian },
        BrigadeId { number: 4, nationality: BrigadeNationality::Egyptian },
        BrigadeId { number: 1, nationality: BrigadeNationality::Sudanese },
        BrigadeId { number: 2, nationality: BrigadeNationality::Sudanese },
        BrigadeId { number: 3, nationality: BrigadeNationality::Sudanese },
        BrigadeId { number: 4, nationality: BrigadeNationality::Sudanese },
    ];

    /// Convenience constructor for a British brigade (`xB`).
    pub const fn british(number: u8) -> Self {
        BrigadeId { number, nationality: BrigadeNationality::British }
    }

    /// Convenience constructor for an Egyptian brigade (`xE`).
    pub const fn egyptian(number: u8) -> Self {
        BrigadeId { number, nationality: BrigadeNationality::Egyptian }
    }

    /// Convenience constructor for a Sudanese brigade (`xS`).
    pub const fn sudanese(number: u8) -> Self {
        BrigadeId { number, nationality: BrigadeNationality::Sudanese }
    }

    /// Convenience constructor for the Friendlies counter (§6.52). The brigade
    /// number is irrelevant; we pin it to 0 to flag "no ordinal".
    pub const fn friendlies() -> Self {
        BrigadeId { number: 0, nationality: BrigadeNationality::Friendlies }
    }
}

impl std::fmt::Display for BrigadeId {
    /// Renders as the printed designation, e.g.
    /// `BrigadeId { number: 3, nationality: Egyptian }` -> `"3E"`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.number, self.nationality.letter())
    }
}

/// Deserialize an `Option<BrigadeId>` accepting both the current
/// `None` / `Some(BrigadeId { .. })` representation and the legacy flat
/// `Brigade` enum variants (`None`, `B1`-`B4`, `E1`-`E4`, `S1`-`S4`) used by
/// pre-migration `.ron` files. Old `"None"` and new `None` both yield `None`.
pub fn deserialize_brigade_option<'de, D>(
    deserializer: D,
) -> Result<Option<BrigadeId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{EnumAccess, VariantAccess, Visitor};

    struct BrigadeOptionVisitor;

    impl<'de> Visitor<'de> for BrigadeOptionVisitor {
        type Value = Option<BrigadeId>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(
                "None, Some(BrigadeId { .. }), or a legacy brigade tag \
                 (None, B1..B4, E1..E4, S1..S4)",
            )
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Self::Value, D2::Error> {
            BrigadeId::deserialize(deserializer).map(Some)
        }

        fn visit_enum<A: EnumAccess<'de>>(
            self,
            data: A,
        ) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            enum LegacyBrigade {
                None,
                B1,
                B2,
                B3,
                B4,
                E1,
                E2,
                E3,
                E4,
                S1,
                S2,
                S3,
                S4,
            }

            let (variant, access) = data.variant::<LegacyBrigade>()?;
            // Every legacy variant is a unit variant; consume its (empty) body.
            access.unit_variant()?;
            Ok(match variant {
                LegacyBrigade::None => None,
                LegacyBrigade::B1 => Some(BrigadeId::british(1)),
                LegacyBrigade::B2 => Some(BrigadeId::british(2)),
                LegacyBrigade::B3 => Some(BrigadeId::british(3)),
                LegacyBrigade::B4 => Some(BrigadeId::british(4)),
                LegacyBrigade::E1 => Some(BrigadeId::egyptian(1)),
                LegacyBrigade::E2 => Some(BrigadeId::egyptian(2)),
                LegacyBrigade::E3 => Some(BrigadeId::egyptian(3)),
                LegacyBrigade::E4 => Some(BrigadeId::egyptian(4)),
                LegacyBrigade::S1 => Some(BrigadeId::sudanese(1)),
                LegacyBrigade::S2 => Some(BrigadeId::sudanese(2)),
                LegacyBrigade::S3 => Some(BrigadeId::sudanese(3)),
                LegacyBrigade::S4 => Some(BrigadeId::sudanese(4)),
            })
        }
    }

    // `deserialize_any` lets the underlying format dispatch the visitor by
    // token kind: RON/JSON will call `visit_none`/`visit_some`/`visit_enum`
    // depending on whether the value is `None`, `Some(...)`, or a bare
    // identifier (`B1`, `E3`, ...).
    deserializer.deserialize_any(BrigadeOptionVisitor)
}

/// The authored facts about one counter on the sprite sheet.
///
/// Mirrors what is *printed directly on the counter* in the rulebook (§2.3):
/// the colour-coded command/tribe identity (§5.52-§5.53), the brigade
/// designation in the upper-right corner (§5.54), and the
/// fire-melee-movement factor triple (§6.11, §7.1, §5.11). Gunboats instead
/// print an artillery/howitzer factor and a split upstream/downstream
/// movement allowance (§5.24); leaders print movement only (§6.51).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpriteAnnotation {
    /// Command/tribe colour. A real game indicator: Dervish leaders may only
    /// stack with units of their own colour, and different tribes may not
    /// stack even when sharing a colour (rulebook §5.52, §5.53).
    pub color: SpriteColor,
    /// Faction identity: Dervish units carry their tribe; Anglo-Egyptian
    /// infantry carry their brigade designation (§5.54).
    pub faction: Option<Faction>,
    pub text: String,
    /// The counter kind, chosen from the annotation dropdown. `Some(kind)` is
    /// a real unit; `None` is a non-unit marker (objective token, status
    /// counter, ...). For older files that predate the `kind` field, the
    /// serde default of `Some(UnitKind::Infantry)` is preserved, then the
    /// editor reclassifies from the legacy `is_boat`/`is_unit` flags.
    #[serde(default = "default_unit_kind")]
    pub kind: Option<UnitKind>,
    /// Printed fire-combat factor (rulebook §6.11). `0` for counters that
    /// print no fire value (e.g. leaders, forts' offensive line).
    #[serde(default)]
    pub fire: i32,
    /// Printed melee factor (rulebook §7.1). Gunboats print none.
    #[serde(default)]
    pub melee: i32,
    /// Printed movement allowance for land units (rulebook §5.11).
    #[serde(default)]
    pub movement: i32,
    /// Gunboat movement against the current -- the smaller, slash-separated
    /// allowance (rulebook §5.11, §5.24).
    #[serde(default)]
    pub movement_upstream: i32,
    /// Gunboat movement with the current -- the larger, slash-separated
    /// allowance (rulebook §5.11, §5.24).
    #[serde(default)]
    pub movement_downstream: i32,
    #[serde(default)]
    pub is_boat: bool,
    #[serde(default = "default_true")]
    pub is_unit: bool,
    /// Whether this counter fires twice per turn -- Maxim guns do (rulebook
    /// §6.42). Authored explicitly (rather than derived from `kind`) so it can
    /// be set on any counter the editor decides should fire twice.
    #[serde(default)]
    pub fires_twice: bool,
}

impl SpriteAnnotation {
    /// Re-derive the legacy `is_boat`/`is_unit` flags from `kind` so the two
    /// representations never drift. Call after the user edits `kind`.
    pub fn sync_flags_from_kind(&mut self) {
        self.is_boat = self.kind.is_some_and(|k| k.is_boat());
        self.is_unit = self.kind.is_some();
    }
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SpriteAnnotations {
    /// `(col, row)` tuple keys can't be JSON object keys, so the inner map is
    /// serialized as a list of `[key, value]` pairs. This keeps the game
    /// record (JSONL) and net history serializable while remaining valid RON.
    #[serde_as(
        as = "indexmap::IndexMap<serde_with::Same, Vec<(serde_with::Same, serde_with::Same)>>"
    )]
    pub units: indexmap::IndexMap<SectionName, indexmap::IndexMap<(u32, u32), SpriteAnnotation>>,
}

/// Hex orientation: [diamond] pointy-top (vertices up/down) or [hexagon] flat-top (vertices left/right).
///
/// Affects pixel-hex conversion formulas and which axis is staggered.
/// Source: <https://www.redblobgames.com/grids/hexagons/#basics>
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Orientation {
    #[default]
    Pointy,
    Flat,
}

/// Which rows/columns are staggered in offset coordinates.
///
/// | Variant | Orientation | Staggered axis |
/// |---------|-------------|----------------|
/// | `OddR` / `EvenR` | pointy-top | rows (q-axis) |
/// | `OddQ` / `EvenQ` | flat-top   | columns (r-axis) |
///
/// "Odd" = first row/col (index 0) is staggered; "Even" = it is not.
/// The stagger magnitude (+/-1/2) and direction are derived -- not free parameters.
///
/// Source: <https://www.redblobgames.com/grids/hexagons/#coordinates>
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OffsetVariant {
    #[default]
    OddR,
    EvenR,
    OddQ,
    EvenQ,
}

impl OffsetVariant {
    /// Stagger amount applied to the offset axis (-0.5 for left/down, +0.5 for right/up).
    ///
    /// For pointy-top (OddR/EvenR) this shifts alternate rows along the q-axis;
    /// for flat-top (OddQ/EvenQ) this shifts alternate columns along the r-axis.
    pub const fn stagger(self) -> f32 {
        -0.5
    }

    /// Phase offset: `1.0` when the first row/col is staggered, `0.0` otherwise.
    pub const fn phase(self) -> f32 {
        match self {
            OffsetVariant::OddR | OffsetVariant::OddQ => 1.0,
            OffsetVariant::EvenR | OffsetVariant::EvenQ => 0.0,
        }
    }
}

/// Map topology for the generated hex set.
///
/// Source: <https://www.redblobgames.com/grids/hexagons/implementation.html#shape-rectangle>
/// Source: <https://www.redblobgames.com/grids/hexagons/implementation.html#shape-parallelogram>
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GridShape {
    #[default]
    /// All rows have the same number of hexes.
    /// Uses the offset-coordinate "rectangle trick" -- loop over offset coords,
    /// convert to axial.
    Rectangle,
    /// Rows vary in width naturally (axial-coordinate parallelogram).
    Parallelogram,
    /// Two alternating row kinds: "long" rows of `width` hexes and "short" rows
    /// of `width - 1` hexes nested half a hex inside each end of the long-row
    /// envelope. (On a staggered pointy-top grid, `width - 1` is the only
    /// short-row width that nests symmetrically -- the rows sit exactly half a
    /// hex apart.) Which parity is long is set by
    /// [`OverlayParams::long_rows_even`]. Used by the campaign map.
    AlternatingRows,
}

/// Parameters that define the hex overlay grid: dimensions, size, position, and
/// layout shape.  Shared by serialization, the in-memory game map, and the
/// editor/resource so there is a single source of truth.
///
/// Terminology follows Red Blob Games' hexagonal grid guide:
/// <https://www.redblobgames.com/grids/hexagons/>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OverlayParams {
    pub width: i32,
    pub height: i32,
    pub hex_size: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    #[serde(default)]
    pub orientation: Orientation,
    #[serde(default)]
    pub offset_variant: OffsetVariant,
    #[serde(default)]
    pub shape: GridShape,

    /// For [`GridShape::AlternatingRows`]: when `true`, even offset rows
    /// (0, 2, ...) are the long rows and odd rows are inset; when `false`, the
    /// parity is flipped. Ignored by other shapes. Defaults to `true` and is
    /// `#[serde(default)]` so older files load unchanged.
    #[serde(default = "default_true")]
    pub long_rows_even: bool,
    /// Fine rotation of the whole hex grid about its origin, in degrees, to
    /// register the lattice against a slightly-skewed scanned map. Small by
    /// design (the editor clamps it to +/-4 deg). `#[serde(default)]` (0.0) so older
    /// files load unchanged.
    #[serde(default)]
    pub rotation_deg: f32,
    /// Anisotropic-scale + shear correction for warped scans, applied as a 2x2
    /// linear map to the local hex position *before* rotation and translation.
    /// Scans of physical boards are rarely a perfect uniform grid: the image can
    /// be stretched more along one axis (aspect) or photographed slightly
    /// off-square so rows drift diagonally (shear). Together with `rotation_deg`
    /// and `hex_size` these form a full affine registration.
    ///
    /// The matrix is `[[1, shear_x], [shear_y, aspect_y]]` relative to the
    /// uniform `hex_size` scale, so the identity (`shear_x = shear_y = 0`,
    /// `aspect_y = 1`) reproduces the pre-affine behaviour exactly. All three are
    /// `#[serde(default)]` (identity) so older files load unchanged.
    ///
    /// NOTE: this is a *linear* (affine) correction. On its own it cannot model
    /// perspective keystone (hexes growing toward one edge); the `size_grad_*`
    /// terms below add that gradient, applied *before* this matrix.
    #[serde(default = "default_scale")]
    pub aspect_y: f32,
    #[serde(default)]
    pub shear_x: f32,
    #[serde(default)]
    pub shear_y: f32,
    /// Perspective/keystone correction: a linear gradient on the hex-size scale,
    /// applied to the local position *before* the affine warp. A hex at local
    /// position `(x, z)` is scaled by `1 + size_grad_x * x + size_grad_y * z`, so
    /// hexes grow (or shrink) progressively with distance from the grid origin
    /// along each axis. This models a board photographed at a slight angle, where
    /// the printed hexes get larger toward one edge -- something the affine
    /// terms alone cannot represent.
    ///
    /// Coefficients are per-*unit-of-local-position* (i.e. in units of hex
    /// circumradius), so they are small (order 1e-3). Zero (`#[serde(default)]`)
    /// is the identity, so older files load unchanged and the pre-keystone
    /// behaviour is exactly reproduced.
    #[serde(default)]
    pub size_grad_x: f32,
    #[serde(default)]
    pub size_grad_y: f32,
}

const fn default_scale() -> f32 {
    1.0
}

impl Default for OverlayParams {
    fn default() -> Self {
        Self {
            width: 48,
            height: 16,
            hex_size: 51.0,
            offset_x: -1.0,
            offset_y: 1.0,
            orientation: Orientation::Pointy,
            offset_variant: OffsetVariant::OddR,
            shape: GridShape::Rectangle,
            long_rows_even: true,
            rotation_deg: 0.0,
            aspect_y: 1.0,
            shear_x: 0.0,
            shear_y: 0.0,
            size_grad_x: 0.0,
            size_grad_y: 0.0,
        }
    }
}

impl OverlayParams {
    /// Apply the keystone size-gradient to a local hex position, scaling it by
    /// `1 + size_grad_x * x + size_grad_y * z`. Applied *before* [`Self::warp`].
    pub fn size_gradient(&self, x: f32, z: f32) -> (f32, f32) {
        let s = 1.0 + self.size_grad_x * x + self.size_grad_y * z;
        (x * s, z * s)
    }

    /// Inverse of [`Self::size_gradient`]. Because `x' = x*s`, `z' = z*s` with
    /// `s = 1 + gx*x + gy*z`, substituting `x = x'/s`, `z = z'/s` gives the
    /// closed-form scalar quadratic `s^2 - s - (gx*x' + gy*z') = 0`. We take the
    /// root nearest 1 (the branch continuous with the identity). Returns `None`
    /// if the point lies past the gradient's fold (no real forward preimage),
    /// which the editor's coefficient clamps keep well out of the map.
    pub fn unsize_gradient(&self, x: f32, z: f32) -> Option<(f32, f32)> {
        if self.size_grad_x == 0.0 && self.size_grad_y == 0.0 {
            return Some((x, z));
        }
        let c = self.size_grad_x * x + self.size_grad_y * z;
        let disc = 1.0 + 4.0 * c;
        if disc < 0.0 {
            return None;
        }
        let s = (1.0 + disc.sqrt()) * 0.5;
        if s.abs() < 1e-6 {
            return None;
        }
        Some((x / s, z / s))
    }

    /// Apply the affine warp matrix `[[1, shear_x], [shear_y, aspect_y]]` to a
    /// local hex position (before rotation/translation).
    pub fn warp(&self, x: f32, z: f32) -> (f32, f32) {
        (x + self.shear_x * z, self.shear_y * x + self.aspect_y * z)
    }

    /// Inverse of [`Self::warp`]. Returns `None` if the matrix is singular
    /// (determinant ~ 0), which the editor prevents by clamping the params.
    pub fn unwarp(&self, x: f32, z: f32) -> Option<(f32, f32)> {
        let det = self.aspect_y - self.shear_x * self.shear_y;
        if det.abs() < 1e-6 {
            return None;
        }
        let inv = 1.0 / det;
        Some((
            inv * (self.aspect_y * x - self.shear_x * z),
            inv * (-self.shear_y * x + z),
        ))
    }
}

/// Which board a piece of map data belongs to.
///
/// The game ships two boards: the tactical Fall-of-Khartoum map and the
/// strategic Campaign map. Lives in `omdurman-types` (not `omdurman-rules`)
/// so the annotations format and the net edit-events can name it without a
/// dependency on the rules crate; the app maps `Scenario -> MapKind`
/// (`Campaign -> Campaign`, everything else -> `FallOfKhartoum`).
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Default,
    strum::Display,
    strum::EnumString,
)]
pub enum MapKind {
    #[default]
    Campaign,
    FallOfKhartoum,
}

/// The two pixel<->hex anchor pairs used to calibrate a map's [`crate`]-external
/// `HexLayout`. Each map carries its own, since the two boards have different
/// images, sizes, and grid placements.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CalibAnchors {
    pub p1_px: (f32, f32),
    pub p1_hex: (i32, i32),
    pub p2_px: (f32, f32),
    pub p2_hex: (i32, i32),
}

/// Everything needed to load and render one board: its terrain tiles, hexside
/// features, hex-overlay parameters, sprite/region annotations, source image,
/// world-plane dimensions, and calibration anchors. Self-contained so the two
/// boards never share coordinate state.
#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapData {
    /// `(q, r)` tuple keys can't be JSON object keys, so the map is serialized
    /// as a list of `[key, value]` pairs (valid in JSON, RON, and postcard). A
    /// `BTreeMap` keeps that list in sorted key order, so the saved file is
    /// deterministic (no diff churn from hash-iteration order).
    #[serde_as(as = "Vec<(_, _)>")]
    pub tiles: BTreeMap<(i32, i32), HexData>,
    /// Per-edge hexside features. Empty/omitted on maps that have none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hexsides: Vec<(HexsideRef, HexsideKind)>,
    /// Road connections between adjacent hexes. Roads form a graph overlay on
    /// the map; each edge appears at most once. Empty/omitted on maps with no
    /// roads.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roads: Vec<HexsideRef>,
    /// Editor-time exclusions: `(q, r)` coords that fall *inside* the overlay
    /// grid but are not part of the playable map (covered by a logo, the turn
    /// track, or other board furniture). Subtracted from the generated hex set,
    /// so excluded hexes carry no terrain and reject placement. A `BTreeSet`
    /// of 2-tuples serializes as a sorted sequence (deterministic, no string-key
    /// issue); empty/omitted on maps that have none.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub excluded: BTreeSet<(i32, i32)>,
    pub overlay: OverlayParams,
    /// World-plane + coordinate-space size for this map's image (pixels).
    pub img_w: f32,
    pub img_h: f32,
    /// Image asset filename loaded onto the map plane (Bevy asset path).
    pub image: String,
    /// Pixel<->hex anchors used to calibrate this map's hex layout.
    pub calib: CalibAnchors,
    /// Pixel bounding box of the campaign turn-track on the map image
    /// (campaign map only; absent on Fall-of-Khartoum). Computed from the
    /// 9 x 3 snake-layout grid -- see [`CampaignTurnTrack`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign_turn_track: Option<CampaignTurnTrack>,
}

impl MapData {
    /// An empty Fall-of-Khartoum map seeded with the canonical landscape image,
    /// dimensions, and calibration anchors. Used as a fallback/default.
    fn empty_fall_of_khartoum() -> Self {
        Self {
            tiles: BTreeMap::new(),
            hexsides: Vec::new(),
            roads: Vec::new(),
            excluded: BTreeSet::new(),
            overlay: OverlayParams::default(),
            img_w: 1571.0,
            img_h: 1200.0,
            image: "fall_of_khartoum_1885.webp".to_string(),
            calib: CalibAnchors {
                p1_px: (736.0, 420.0),
                p1_hex: (0, 0),
                p2_px: (1178.0, 572.0),
                p2_hex: (5, -1),
            },
            campaign_turn_track: None,
        }
    }

    /// An empty Campaign map seeded with the portrait campaign image and its
    /// dimensions. The calibration anchors are placeholders to be dialed in via
    /// the in-app Overlay calibration mode.
    fn empty_campaign() -> Self {
        Self {
            tiles: BTreeMap::new(),
            hexsides: Vec::new(),
            roads: Vec::new(),
            excluded: BTreeSet::new(),
            overlay: OverlayParams {
                shape: GridShape::AlternatingRows,
                ..OverlayParams::default()
            },
            img_w: 3258.0,
            img_h: 4124.0,
            image: "campaign_map.webp".to_string(),
            calib: CalibAnchors {
                p1_px: (0.0, 0.0),
                p1_hex: (0, 0),
                p2_px: (100.0, 100.0),
                p2_hex: (5, -1),
            },
            campaign_turn_track: None,
        }
    }
}

/// Both boards' data in one file. `LoadAnnotations` carries the whole thing;
/// the apply path selects the active board by the started scenario.
///
/// `sprites` (the counter-sheet annotations) is a single top-level field rather
/// than per-board state, because the unit model is board-independent.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnnotationsFile {
    pub fall_of_khartoum: MapData,
    pub campaign: MapData,
    /// Unit/region sprite annotations. The counter sheet is the same regardless
    /// of which board is in play, so these are global, not per-[`MapData`].
    #[serde(default)]
    pub sprites: SpriteAnnotations,
    /// Calibrated boxes for the reference-chart scans (geometry only; the table
    /// structure is fixed in code). Global and defaulted, so older files load.
    #[serde(default)]
    pub chart_boxes: ChartBoxes,
}

impl AnnotationsFile {
    /// Both boards empty, each seeded with its image/dims/anchors; no sprites.
    pub fn empty() -> Self {
        Self {
            fall_of_khartoum: MapData::empty_fall_of_khartoum(),
            campaign: MapData::empty_campaign(),
            sprites: SpriteAnnotations::default(),
            chart_boxes: ChartBoxes::default(),
        }
    }

    /// Shared accessor for the board selected by `kind`.
    pub fn map(&self, kind: MapKind) -> &MapData {
        match kind {
            MapKind::FallOfKhartoum => &self.fall_of_khartoum,
            MapKind::Campaign => &self.campaign,
        }
    }

    /// Mutable accessor for the board selected by `kind`.
    pub fn map_mut(&mut self, kind: MapKind) -> &mut MapData {
        match kind {
            MapKind::FallOfKhartoum => &mut self.fall_of_khartoum,
            MapKind::Campaign => &mut self.campaign,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_between_excludes_endpoints_and_steps_toward_target() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(3, 0);
        let line = a.line_between(b);
        // Endpoints excluded; the two intervening hexes, each one step closer.
        assert_eq!(line, vec![HexCoord::new(1, 0), HexCoord::new(2, 0)]);
        // Every hex on the line is strictly between the endpoints.
        for hex in &line {
            assert!(hex != &a && hex != &b);
            assert!(a.distance(*hex) < a.distance(b));
        }
    }

    #[test]
    fn line_between_adjacent_or_same_is_empty() {
        let a = HexCoord::new(2, 2);
        assert!(a.line_between(a).is_empty());
        for n in a.neighbors() {
            assert!(
                a.line_between(n).is_empty(),
                "adjacent hex has no in-between"
            );
        }
    }

    #[test]
    fn hexside_ref_normalizes_and_separates() {
        let a = HexCoord::new(2, 3);
        let b = HexCoord::new(2, 4);
        // Order-independent identity.
        assert_eq!(HexsideRef::new(a, b), HexsideRef::new(b, a));
        // Usable as a stable hash key.
        let mut set = std::collections::HashMap::new();
        set.insert(HexsideRef::new(a, b), HexsideKind::Wall);
        assert_eq!(set.get(&HexsideRef::new(b, a)), Some(&HexsideKind::Wall));
        // separates() is direction-agnostic.
        assert!(HexsideRef::new(a, b).separates(a, b));
        assert!(HexsideRef::new(a, b).separates(b, a));
        assert!(!HexsideRef::new(a, b).separates(a, HexCoord::new(3, 3)));
    }

    #[test]
    fn hexside_kind_blocking_predicates() {
        assert!(HexsideKind::Wall.blocks_los());
        assert!(HexsideKind::Crest.blocks_los());
        assert!(!HexsideKind::Gate.blocks_los());
        assert!(!HexsideKind::Breach.blocks_los());
        assert!(HexsideKind::Wall.blocks_movement());
        assert!(!HexsideKind::Gate.blocks_movement());
        assert!(HexsideKind::Wall.blocks_melee());
        assert!(HexsideKind::ZaribaThornHedge.blocks_melee());
        assert!(!HexsideKind::Gate.blocks_melee());
        assert!(HexsideKind::Khor.blocks_advance_after_combat());
    }

    #[test]
    fn los_terrain_predicates() {
        assert!(Terrain::Huts { road: Road::None }.blocks_los());
        assert!(Terrain::Building { road: Road::None }.blocks_los());
        assert!(!Terrain::Clear { road: Road::None }.blocks_los());
        assert!(!Terrain::Trees { road: Road::None }.blocks_los());
        assert!(Terrain::Trees { road: Road::None }.is_los_trees());
        assert!(!Terrain::Clear { road: Road::None }.is_los_trees());
    }
}

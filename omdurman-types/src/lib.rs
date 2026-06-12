use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
pub use strum::IntoEnumIterator;

pub mod section_name;
pub use section_name::SectionName;

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

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// The six axial neighbours, in a fixed order. The neighbour convention
    /// (which `(q, r)` offsets are adjacent) is defined here so movement,
    /// targeting, and overlays all agree on adjacency.
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

    /// Hex distance (cube max-norm) between two coordinates, consistent with
    /// the [`neighbors`](Self::neighbors) adjacency (adjacent hexes are at
    /// distance 1).
    pub fn distance(self, other: HexCoord) -> u32 {
        let dq = (self.q - other.q).unsigned_abs();
        let dr = (self.r - other.r).unsigned_abs();
        let ds = (self.q + self.r - other.q - other.r).unsigned_abs();
        dq.max(dr).max(ds / 2)
    }

    /// The hexes strictly *between* `self` and `other` (endpoints excluded),
    /// in order from `self` toward `other`. Empty for adjacent or identical
    /// hexes. Used for line-of-sight: each step picks the neighbour that most
    /// reduces the remaining distance, so the path is consistent with this
    /// grid's [`neighbors`](Self::neighbors)/[`distance`](Self::distance)
    /// convention regardless of the underlying coordinate layout.
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HexEdge(pub HexCoord, pub HexCoord);

impl HexEdge {
    pub fn new(a: HexCoord, b: HexCoord) -> Self {
        if (a.q, a.r) <= (b.q, b.r) {
            HexEdge(a, b)
        } else {
            HexEdge(b, a)
        }
    }
}

/// Reference to a specific hex-side (the edge shared by two adjacent hexes).
/// Endpoints are stored in canonical (low→high) order so the same physical
/// edge always compares and hashes equal regardless of which side names it —
/// this lets a map key per-edge hexside data by [`HexsideRef`].
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HexsideRef {
    pub a: HexCoord,
    pub b: HexCoord,
}

impl HexsideRef {
    /// Canonicalised edge between two adjacent hexes (order-independent).
    pub fn new(a: HexCoord, b: HexCoord) -> Self {
        if (a.q, a.r) <= (b.q, b.r) {
            HexsideRef { a, b }
        } else {
            HexsideRef { a: b, b: a }
        }
    }

    /// Whether this edge separates `from` from `to` (in either direction).
    pub fn separates(self, from: HexCoord, to: HexCoord) -> bool {
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
    /// Khor — gully/wadi. ZOCs do not extend across (§5.44); advance after
    /// combat may not cross (§6.82).
    Khor,
    /// Crest line. Blocks LOS unless the firer is on the higher side
    /// (§6.3 note 7).
    Crest,
    /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
    ZaribaThornHedge,
    /// Historical-scenario trench segment of the Zariba (§9.232).
    ZaribaTrench,
    /// Khor Shambat — the specific named khor that empties into the Nile (a
    /// scenario landmark; used as a setup/reinforcement boundary). Same blocking
    /// rules as a generic [`Khor`](HexsideKind::Khor), but distinctly named so it
    /// can be marked on the map. Appended last for repr stability.
    KhorShambat,
}

impl HexsideKind {
    /// Whether this hexside blocks line of sight across it (§6.3). Crest is
    /// directional and handled by the caller; here it is treated as blocking.
    pub fn blocks_los(self) -> bool {
        matches!(self, HexsideKind::Wall | HexsideKind::Crest)
    }

    /// Whether ZOC may *not* extend out of a hex across this side (§5.44).
    pub fn blocks_zoc_outbound(self) -> bool {
        matches!(self, HexsideKind::Khor | HexsideKind::KhorShambat)
    }

    /// Whether ZOC may *not* extend into a hex across this side (§5.44).
    pub fn blocks_zoc_inbound(self) -> bool {
        matches!(
            self,
            HexsideKind::Khor | HexsideKind::KhorShambat | HexsideKind::Wall | HexsideKind::Gate
        )
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
}

/// Direction of the Nile current through an `is_nile` hex, used to interpret
/// gunboat upstream/downstream movement (rulebook §5.11, §5.24 — "the
/// direction of the current is indicated by arrows in the Nile").
///
/// The current flows in a single direction through a hex, so it is stored as
/// one neighbour-index in `0..6` (the canonical neighbour order: `+q`, `+q+r`,
/// `+r`, `-q`, `-q-r`, `-r`). The current flows *toward* `dir`'s neighbour —
/// i.e. a gunboat moving toward that neighbour is going **downstream**, and
/// the opposite way is **upstream**.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct NileFlow {
    /// Neighbour index `0..6` the current flows toward (downstream direction).
    pub dir: u8,
}

impl NileFlow {
    /// Number of distinct directions the arrow can point (one per hex edge).
    pub const STEPS: u8 = 6;

    /// A flow pointing toward neighbour `dir` (taken mod 6).
    pub fn new(dir: u8) -> Self {
        Self {
            dir: dir % Self::STEPS,
        }
    }

    /// Rotate the arrow by `delta` sixths of a turn (positive = toward higher
    /// neighbour index), wrapping around.
    pub fn rotated(self, delta: i8) -> Self {
        let n = Self::STEPS as i16;
        let d = (self.dir as i16 + delta as i16).rem_euclid(n);
        Self { dir: d as u8 }
    }
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Default,
    strum::Display,
    strum::EnumIter,
    strum::FromRepr,
)]
#[repr(u8)]
/// Hex terrain types used on the Omdurman map.
pub enum Terrain {
    #[default]
    Desert,
    Shrubs,
    Palm,
    BlueNile,
    WhiteNile,
    Fortress,
    Khartoum,
    Tuti,
    Hogali,
    Buri,
    FortBuri,
    FortMakran,
    NorthFort,
    // ── LOS / Terrain-Effects features (rulebook §6.3, Terrain Effects Chart).
    // Appended after the original variants so existing `to_u8`/`from_u8` reprs
    // and saved maps stay stable. `Shrubs` historically doubled as rough going;
    // `Rough` is the explicit rough-terrain hex on maps that distinguish it.
    /// Rough terrain — costs extra movement and gives a defensive bonus; can
    /// block line of sight (§6.3).
    Rough,
    /// Hilltop — units on it see over intervening terrain (§6.3 note 2).
    Hilltop,
    /// Crest high ground; crests block LOS unless the firer is on the higher
    /// side (§6.3 note 7).
    Crest,
    /// Hut hex — blocks LOS except to adjacent units (§6.3 note 6); modest
    /// defensive bonus.
    Hut,
    /// Building hex (Khartoum/Forts) — blocks LOS and gives a defensive bonus.
    Building,
    /// River Nile — a Nile water hex like Blue/White Nile (carries a current
    /// direction; impassable to land units). Appended last so existing
    /// `to_u8`/`from_u8` reprs and saved maps stay stable.
    RiverNile,
    /// Tree cover (woods) — counts as "trees" for the line-of-sight palm-grove
    /// rule (§6.3 note 1). Appended last to keep `to_u8`/`from_u8` reprs stable.
    Trees,
    /// Swamp / marsh — difficult going. Appended last (repr stability).
    Swamp,
    // ── Named map locations. Appended last (repr stability). Buildings/forts
    // give a defensive bonus and block LOS; the "…Village" hexes are villages
    // (hut clusters). ──
    /// Makran Point — fortified point on the west bank (fort-like building).
    MakranPoint,
    /// The Treasury (named building inside Omdurman).
    Treasury,
    /// Shambat village (hut cluster).
    ShambatVillage,
    /// Halfaya village (hut cluster).
    HalfayaVillage,
    /// El Debeba village (hut cluster).
    ElDebebaVillage,
    /// El Egeiga village (hut cluster).
    ElEgeigaVillage,
    /// The Zariba — the Anglo-Egyptian defensive perimeter hex (historical /
    /// campaign). Distinct from the per-hexside Zariba features.
    Zariba,
    /// Abu Alim village (hut cluster; Anglo-Egyptian "Friendlies" entry).
    AbuAlimVillage,
    /// Kerreri village (hut cluster).
    KerreriVillage,
    // ── Map-legend code terrains (named by their single-letter map code; no
    // special movement/combat effect yet — treated as Clear). Appended last for
    // repr stability. ──
    Y,
    K,
    S,
    O,
    D,
    A,
    /// Omdurman — the walled Dervish city (counterpart to Khartoum). A built-up
    /// city hex: blocks LOS, strong defensive bonus. Appended last (repr stable).
    Omdurman,
}

/// Named palette colour for a terrain-type overlay. A typed enum (rather than
/// strum string props) so the terrain→colour mapping is total and checked.
/// Palette inspired by the Sudanese landscape (sand, Nile, khaki, earth).
#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display, strum::EnumIter,
)]
pub enum TerrainColor {
    Sandy,
    GreenBrown,
    DarkGreen,
    Blue,
    LightBlue,
    Gray,
    DarkRed,
    LightGreen,
    MediumGreen,
    Olive,
    DarkGray,
    DarkRedBrown,
    DarkBlueGray,
    TanBrown,
    Brown,
    LightBrown,
    Tan,
    StoneGray,
    SwampGreen,
}

impl TerrainColor {
    /// The RGBA value used to paint the overlay (alpha 0.75 for all).
    pub fn rgba(self) -> [f32; 4] {
        match self {
            TerrainColor::Sandy => [0.90, 0.78, 0.40, 0.75],
            TerrainColor::GreenBrown => [0.60, 0.55, 0.22, 0.75],
            TerrainColor::DarkGreen => [0.28, 0.55, 0.15, 0.75],
            TerrainColor::Blue => [0.18, 0.55, 0.68, 0.75],
            TerrainColor::LightBlue => [0.42, 0.78, 0.82, 0.75],
            TerrainColor::Gray => [0.65, 0.50, 0.38, 0.75],
            TerrainColor::DarkRed => [0.62, 0.22, 0.08, 0.75],
            TerrainColor::LightGreen => [0.50, 0.68, 0.22, 0.75],
            TerrainColor::MediumGreen => [0.38, 0.55, 0.18, 0.75],
            TerrainColor::Olive => [0.68, 0.55, 0.10, 0.75],
            TerrainColor::DarkGray => [0.42, 0.32, 0.25, 0.75],
            TerrainColor::DarkRedBrown => [0.58, 0.20, 0.08, 0.75],
            TerrainColor::DarkBlueGray => [0.22, 0.30, 0.45, 0.75],
            TerrainColor::TanBrown => [0.72, 0.58, 0.38, 0.75],
            TerrainColor::Brown => [0.55, 0.40, 0.24, 0.75],
            TerrainColor::LightBrown => [0.78, 0.64, 0.44, 0.75],
            TerrainColor::Tan => [0.82, 0.71, 0.52, 0.75],
            TerrainColor::StoneGray => [0.58, 0.58, 0.55, 0.75],
            TerrainColor::SwampGreen => [0.30, 0.42, 0.30, 0.75],
        }
    }
}

impl Terrain {
    pub fn to_u8(self) -> u8 {
        self as u8
    }
    pub fn from_u8(v: u8) -> Self {
        Self::from_repr(v).unwrap_or(Self::Desert)
    }
    pub fn passable_by_land(self) -> bool {
        !self.is_nile()
    }

    pub fn is_city(self) -> bool {
        matches!(self, Terrain::Khartoum | Terrain::Omdurman)
    }

    /// Whether an intervening hex of this terrain unconditionally blocks line
    /// of sight (§6.3): huts and buildings (and the city/forts that are built
    /// up). Palm groves are handled separately — they only block in quantity
    /// (see [`is_los_trees`](Self::is_los_trees), §6.3 note 1).
    pub fn blocks_los(self) -> bool {
        matches!(
            self,
            Terrain::Hut
                | Terrain::Building
                | Terrain::Khartoum
                | Terrain::Omdurman
                | Terrain::Fortress
                | Terrain::FortBuri
                | Terrain::FortMakran
                | Terrain::NorthFort
        ) || self.is_village()
            || matches!(
                self,
                Terrain::MakranPoint | Terrain::Treasury | Terrain::Zariba
            )
    }

    /// Whether this terrain counts as "trees" for the LOS palm-grove rule:
    /// line of sight is blocked by more than two intervening tree hexes
    /// (§6.3 note 1).
    pub fn is_los_trees(self) -> bool {
        matches!(self, Terrain::Palm | Terrain::Trees)
    }

    pub fn is_village(self) -> bool {
        matches!(
            self,
            Terrain::Tuti
                | Terrain::Hogali
                | Terrain::Buri
                | Terrain::ShambatVillage
                | Terrain::HalfayaVillage
                | Terrain::ElDebebaVillage
                | Terrain::ElEgeigaVillage
                | Terrain::AbuAlimVillage
                | Terrain::KerreriVillage
        )
    }

    pub fn is_nile(self) -> bool {
        matches!(
            self,
            Terrain::BlueNile | Terrain::WhiteNile | Terrain::RiverNile
        )
    }

    pub fn is_fort(self) -> bool {
        matches!(
            self,
            Terrain::Fortress | Terrain::FortBuri | Terrain::FortMakran | Terrain::NorthFort
        )
    }

    /// The palette colour for this terrain's overlay.
    pub fn color(self) -> TerrainColor {
        match self {
            Terrain::Desert => TerrainColor::Sandy,
            Terrain::Shrubs => TerrainColor::GreenBrown,
            Terrain::Palm => TerrainColor::DarkGreen,
            Terrain::BlueNile => TerrainColor::Blue,
            Terrain::WhiteNile => TerrainColor::LightBlue,
            Terrain::Fortress => TerrainColor::Gray,
            Terrain::Khartoum => TerrainColor::DarkRed,
            Terrain::Tuti => TerrainColor::LightGreen,
            Terrain::Hogali => TerrainColor::MediumGreen,
            Terrain::Buri => TerrainColor::Olive,
            Terrain::FortBuri => TerrainColor::DarkGray,
            Terrain::FortMakran => TerrainColor::DarkRedBrown,
            Terrain::NorthFort => TerrainColor::DarkBlueGray,
            Terrain::Rough => TerrainColor::TanBrown,
            Terrain::Hilltop => TerrainColor::Brown,
            Terrain::Crest => TerrainColor::LightBrown,
            Terrain::Hut => TerrainColor::Tan,
            Terrain::Building => TerrainColor::StoneGray,
            Terrain::RiverNile => TerrainColor::Blue,
            Terrain::Trees => TerrainColor::DarkGreen,
            Terrain::Swamp => TerrainColor::SwampGreen,
            // Named buildings/points — stone/earth tones.
            Terrain::MakranPoint => TerrainColor::DarkRedBrown,
            Terrain::Treasury => TerrainColor::StoneGray,
            Terrain::Zariba => TerrainColor::DarkGray,
            // Named villages — green tones, like the other villages.
            Terrain::ShambatVillage => TerrainColor::LightGreen,
            Terrain::HalfayaVillage => TerrainColor::MediumGreen,
            Terrain::ElDebebaVillage => TerrainColor::Olive,
            Terrain::ElEgeigaVillage => TerrainColor::LightGreen,
            Terrain::AbuAlimVillage => TerrainColor::MediumGreen,
            Terrain::KerreriVillage => TerrainColor::Olive,
            // Map-legend code terrains — neutral tint until they get effects.
            Terrain::Y => TerrainColor::Olive,
            Terrain::K => TerrainColor::StoneGray,
            Terrain::S => TerrainColor::SwampGreen,
            Terrain::O => TerrainColor::TanBrown,
            Terrain::D => TerrainColor::Sandy,
            Terrain::A => TerrainColor::GreenBrown,
            Terrain::Omdurman => TerrainColor::DarkRedBrown,
        }
    }

    /// Return an RGBA colour suitable for a terrain-type overlay.
    pub fn overlay_color(self) -> [f32; 4] {
        self.color().rgba()
    }
}

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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HexData {
    pub terrain: Terrain,
    pub location: Option<Location>,
    pub name: Option<String>,
    /// Per-edge Nile current annotation, present only for `is_nile` hexes.
    /// Used to interpret gunboat upstream/downstream movement (§5.11, §5.24).
    #[serde(default)]
    pub nile_flow: Option<NileFlow>,
    /// Whether roads meeting at this hex converge at the centre. When `false`,
    /// roads stop at the hex edge ("mouth into" the hex) instead of reaching
    /// the centre. Default `false` (omitted in serialization).
    #[serde(default)]
    pub is_crossroad: bool,
}

impl HexData {
    /// Hex with terrain and an optional name. Locations are set elsewhere
    /// from the static `LOCATIONS` table.
    pub fn new(terrain: Terrain, name: Option<String>) -> Self {
        Self {
            terrain,
            location: None,
            name,
            nile_flow: None,
            is_crossroad: false,
        }
    }

    /// Hex with terrain, name, and an explicit Nile-flow annotation.
    pub fn with_flow(terrain: Terrain, name: Option<String>, nile_flow: Option<NileFlow>) -> Self {
        Self {
            terrain,
            location: None,
            name,
            nile_flow,
            is_crossroad: false,
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

#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display, strum::EnumIter,
)]
pub enum Faction {
    Independent,
    Dervish,
    BritishEgyptian,
}

const fn default_true() -> bool {
    true
}

/// The kind of counter a sprite annotation describes, mirroring the rules-crate
/// `UnitKind` (rulebook §2.3). Selected via a dropdown in the unit-annotation
/// screen; it drives which combat fields the form shows (e.g. only `Gunboat`
/// exposes the upstream/downstream movement allowances of §5.24, and the two
/// leader kinds print movement only — §6.51).
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Default,
    strum::Display,
    strum::EnumIter,
)]
pub enum UnitFormKind {
    #[default]
    Infantry,
    Cavalry,
    Camel,
    Artillery,
    Maxim,
    Gunboat,
    Fort,
    DervishLeader,
    BritishLeader,
    /// A non-unit marker (objective token, status counter, …) — no combat
    /// stats. Replaces the meaning of the legacy `is_unit = false` flag.
    Marker,
}

impl UnitFormKind {
    /// Gunboats use the split upstream/downstream movement allowance (§5.24).
    pub fn is_boat(self) -> bool {
        matches!(self, UnitFormKind::Gunboat)
    }

    /// Whether this kind represents a playable unit (anything but `Marker`).
    pub fn is_unit(self) -> bool {
        !matches!(self, UnitFormKind::Marker)
    }

    /// British and Dervish leaders print a movement factor only (§6.51); other
    /// playable kinds carry fire and/or melee factors.
    pub fn has_combat_factors(self) -> bool {
        !matches!(self, UnitFormKind::BritishLeader | UnitFormKind::Marker)
    }

    /// Maxim guns fire twice per turn — once in the Direct Fire Subphase and
    /// again in the Maxim Second Fire Subphase (rulebook §6.42). The counter
    /// is marked "×2" in the editor to surface this.
    pub fn fires_twice(self) -> bool {
        matches!(self, UnitFormKind::Maxim)
    }

    /// Best-effort classification of a legacy annotation that predates the
    /// `kind` field, from its `is_boat` / `is_unit` flags.
    pub fn from_legacy_flags(is_boat: bool, is_unit: bool) -> Self {
        if !is_unit {
            UnitFormKind::Marker
        } else if is_boat {
            UnitFormKind::Gunboat
        } else {
            UnitFormKind::Infantry
        }
    }
}

/// Nationality of an Anglo-Egyptian infantry brigade, as printed on the
/// counter's brigade designation (rulebook §5.54).
#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display, strum::EnumIter,
)]
pub enum BrigadeNationality {
    /// `xB` — British.
    British,
    /// `xE` — Egyptian.
    Egyptian,
    /// `xS` — Sudanese.
    Sudanese,
}

impl BrigadeNationality {
    /// Single-letter suffix used in the printed designation (`B`/`E`/`S`).
    pub fn letter(self) -> char {
        match self {
            BrigadeNationality::British => 'B',
            BrigadeNationality::Egyptian => 'E',
            BrigadeNationality::Sudanese => 'S',
        }
    }
}

/// Brigade designation printed in the upper-right corner of an infantry
/// counter (rulebook §5.54). Four battalions of the same brigade stacked in
/// one hex gain brigade integrity (+1 fire die roll). `None` for counters that
/// carry no brigade designation.
///
/// Modelled as a flat enum (rather than a free-text string) so the editor can
/// offer a fixed picker and the rules engine receives a validated value.
#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default, strum::EnumIter,
)]
pub enum Brigade {
    #[default]
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

impl Brigade {
    /// The `(brigade number, nationality)` this designation denotes, or `None`
    /// for [`Brigade::None`].
    pub fn parts(self) -> Option<(u8, BrigadeNationality)> {
        use BrigadeNationality::*;
        Some(match self {
            Brigade::None => return None,
            Brigade::B1 => (1, British),
            Brigade::B2 => (2, British),
            Brigade::B3 => (3, British),
            Brigade::B4 => (4, British),
            Brigade::E1 => (1, Egyptian),
            Brigade::E2 => (2, Egyptian),
            Brigade::E3 => (3, Egyptian),
            Brigade::E4 => (4, Egyptian),
            Brigade::S1 => (1, Sudanese),
            Brigade::S2 => (2, Sudanese),
            Brigade::S3 => (3, Sudanese),
            Brigade::S4 => (4, Sudanese),
        })
    }
}

impl std::fmt::Display for Brigade {
    /// Renders as the printed designation, e.g. `Brigade::E3` → `"3E"`,
    /// `Brigade::None` → `"—"`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.parts() {
            Some((number, nationality)) => write!(f, "{number}{}", nationality.letter()),
            None => f.write_str("—"),
        }
    }
}

/// The authored facts about one counter on the sprite sheet.
///
/// Mirrors what is *printed directly on the counter* in the rulebook (§2.3):
/// the colour-coded command/tribe identity (§5.52–§5.53), the brigade
/// designation in the upper-right corner (§5.54), and the
/// fire–melee–movement factor triple (§6.11, §7.1, §5.11). Gunboats instead
/// print an artillery/howitzer factor and a split upstream/downstream
/// movement allowance (§5.24); leaders print movement only (§6.51).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpriteAnnotation {
    /// Command/tribe colour. A real game indicator: Dervish leaders may only
    /// stack with units of their own colour, and different tribes may not
    /// stack even when sharing a colour (rulebook §5.52, §5.53).
    pub color: SpriteColor,
    pub faction: Faction,
    pub text: String,
    /// The counter kind, chosen from the annotation dropdown. Defaults are
    /// derived from the legacy `is_boat`/`is_unit` flags for older files that
    /// have no `kind` recorded.
    #[serde(default)]
    pub kind: UnitFormKind,
    /// Brigade designation printed in the counter's upper-right corner, e.g.
    /// [`Brigade::B2`] (2nd British) or [`Brigade::E3`] (3rd Egyptian). Four
    /// battalions of the same brigade stacked together gain brigade integrity
    /// (+1 fire die roll). [`Brigade::None`] for non-brigade counters (§5.54).
    #[serde(default)]
    pub brigade: Brigade,
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
    /// Gunboat movement against the current — the smaller, slash-separated
    /// allowance (rulebook §5.11, §5.24).
    #[serde(default)]
    pub movement_upstream: i32,
    /// Gunboat movement with the current — the larger, slash-separated
    /// allowance (rulebook §5.11, §5.24).
    #[serde(default)]
    pub movement_downstream: i32,
    #[serde(default)]
    pub is_boat: bool,
    #[serde(default = "default_true")]
    pub is_unit: bool,
    /// Whether this counter fires twice per turn — Maxim guns do (rulebook
    /// §6.42). Authored explicitly (rather than derived from `kind`) so it can
    /// be set on any counter the editor decides should fire twice.
    #[serde(default)]
    pub fires_twice: bool,
}

impl SpriteAnnotation {
    /// Re-derive the legacy `is_boat`/`is_unit` flags from `kind` so the two
    /// representations never drift. Call after the user edits `kind`.
    pub fn sync_flags_from_kind(&mut self) {
        self.is_boat = self.kind.is_boat();
        self.is_unit = self.kind.is_unit();
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileInfo {
    pub terrain: Terrain,
    pub name: Option<String>,
    /// Per-edge Nile current annotation; serialized only for `is_nile` hexes
    /// that carry at least one current (§5.11, §5.24).
    #[serde(default)]
    pub nile_flow: Option<NileFlow>,
    /// Whether roads converge at this hex's centre rather than stopping at the
    /// edge. Omitted/false on hexes that are not crossroads.
    #[serde(default)]
    pub is_crossroad: bool,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapSection {
    /// `(q, r)` tuple keys can't be JSON object keys, so the map is serialized
    /// as a list of `[key, value]` pairs (valid in JSON, RON, and postcard). A
    /// `BTreeMap` keeps that list in sorted key order for stable, diff-friendly
    /// output.
    #[serde_as(as = "Vec<(_, _)>")]
    pub tiles: BTreeMap<(i32, i32), TileInfo>,
    /// Per-edge hexside features (walls, khors, gates, breaches, crests,
    /// zariba). Serialized as a list of `[edge, kind]` pairs. Empty/omitted on
    /// maps that have none (older files default it).
    #[serde(default)]
    pub hexsides: Vec<(HexsideRef, HexsideKind)>,
}

/// Hex orientation: ⬢ pointy-top (vertices up/down) or ⬣ flat-top (vertices left/right).
///
/// Affects pixel–hex conversion formulas and which axis is staggered.
/// Source: https://www.redblobgames.com/grids/hexagons/#basics
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
/// The stagger magnitude (±½) and direction are derived — not free parameters.
///
/// Source: https://www.redblobgames.com/grids/hexagons/#coordinates
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OffsetVariant {
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
/// Source: https://www.redblobgames.com/grids/hexagons/implementation.html#shape-rectangle
/// Source: https://www.redblobgames.com/grids/hexagons/implementation.html#shape-parallelogram
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GridShape {
    #[default]
    /// All rows have the same number of hexes.
    /// Uses the offset-coordinate "rectangle trick" — loop over offset coords,
    /// convert to axial.
    Rectangle,
    /// Rows vary in width naturally (axial-coordinate parallelogram).
    Parallelogram,
    /// Two alternating row kinds: "long" rows of `width` hexes and "short" rows
    /// of `width - 1` hexes nested half a hex inside each end of the long-row
    /// envelope. (On a staggered pointy-top grid, `width - 1` is the only
    /// short-row width that nests symmetrically — the rows sit exactly half a
    /// hex apart.) Which parity is long is set by
    /// [`OverlayParams::long_rows_even`]. Used by the campaign map.
    AlternatingRows,
}

fn default_long_rows_even() -> bool {
    true
}

fn default_orientation() -> Orientation {
    Orientation::Pointy
}
fn default_offset_variant() -> OffsetVariant {
    OffsetVariant::OddR
}
fn default_grid_shape() -> GridShape {
    GridShape::Rectangle
}

/// Parameters that define the hex overlay grid: dimensions, size, position, and
/// layout shape.  Shared by serialization, the in-memory game map, and the
/// editor/resource so there is a single source of truth.
///
/// Terminology follows Red Blob Games' hexagonal grid guide:
/// https://www.redblobgames.com/grids/hexagons/
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OverlayParams {
    pub width: i32,
    pub height: i32,
    pub hex_size: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    #[serde(default = "default_orientation")]
    pub orientation: Orientation,
    #[serde(default = "default_offset_variant")]
    pub offset_variant: OffsetVariant,
    #[serde(default = "default_grid_shape")]
    pub shape: GridShape,
    /// For [`GridShape::AlternatingRows`]: when `true`, even offset rows
    /// (0, 2, …) are the long rows and odd rows are inset; when `false`, the
    /// parity is flipped. Ignored by other shapes. Defaults to `true` and is
    /// `#[serde(default)]` so older files load unchanged.
    #[serde(default = "default_long_rows_even")]
    pub long_rows_even: bool,
    /// Fine rotation of the whole hex grid about its origin, in degrees, to
    /// register the lattice against a slightly-skewed scanned map. Small by
    /// design (the editor clamps it to ±4°). `#[serde(default)]` (0.0) so older
    /// files load unchanged.
    #[serde(default)]
    pub rotation_deg: f32,
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
        }
    }
}

/// Which board a piece of map data belongs to.
///
/// The game ships two boards: the tactical Fall-of-Khartoum map and the
/// strategic Campaign map. Lives in `omdurman-types` (not `omdurman-rules`)
/// so the annotations format and the net edit-events can name it without a
/// dependency on the rules crate; the app maps `Scenario → MapKind`
/// (`Campaign → Campaign`, everything else → `FallOfKhartoum`).
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
    FallOfKhartoum,
    Campaign,
}

/// The two pixel↔hex anchor pairs used to calibrate a map's [`crate`]-external
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
    pub tiles: BTreeMap<(i32, i32), TileInfo>,
    /// Per-edge hexside features. Empty/omitted on maps that have none.
    #[serde(default)]
    pub hexsides: Vec<(HexsideRef, HexsideKind)>,
    /// Road connections between adjacent hexes. Roads form a graph overlay on
    /// the map; each edge appears at most once. Empty/omitted on maps with no
    /// roads.
    #[serde(default)]
    pub roads: Vec<HexsideRef>,
    /// Editor-time exclusions: `(q, r)` coords that fall *inside* the overlay
    /// grid but are not part of the playable map (covered by a logo, the turn
    /// track, or other board furniture). Subtracted from the generated hex set,
    /// so excluded hexes carry no terrain and reject placement. A `BTreeSet`
    /// of 2-tuples serializes as a sorted sequence (deterministic, no string-key
    /// issue); empty/omitted on maps that have none.
    #[serde(default)]
    pub excluded: BTreeSet<(i32, i32)>,
    pub overlay: OverlayParams,
    /// World-plane + coordinate-space size for this map's image (pixels).
    pub img_w: f32,
    pub img_h: f32,
    /// Image asset filename loaded onto the map plane (Bevy asset path).
    pub image: String,
    /// Pixel↔hex anchors used to calibrate this map's hex layout.
    pub calib: CalibAnchors,
}

impl MapData {
    /// An empty Fall-of-Khartoum map seeded with the canonical landscape image,
    /// dimensions, and calibration anchors. Used as a fallback/default.
    pub fn empty_fall_of_khartoum() -> Self {
        Self {
            tiles: BTreeMap::new(),
            hexsides: Vec::new(),
            roads: Vec::new(),
            excluded: BTreeSet::new(),
            overlay: OverlayParams::default(),
            img_w: 1571.0,
            img_h: 1200.0,
            image: "fall_of_khartoum_1885.png".to_string(),
            calib: CalibAnchors {
                p1_px: (736.0, 420.0),
                p1_hex: (0, 0),
                p2_px: (1178.0, 572.0),
                p2_hex: (5, -1),
            },
        }
    }

    /// An empty Campaign map seeded with the portrait campaign image and its
    /// dimensions. The calibration anchors are placeholders to be dialed in via
    /// the in-app Overlay calibration mode.
    pub fn empty_campaign() -> Self {
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
            image: "campaign_map.png".to_string(),
            calib: CalibAnchors {
                p1_px: (0.0, 0.0),
                p1_hex: (0, 0),
                p2_px: (100.0, 100.0),
                p2_hex: (5, -1),
            },
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
}

impl AnnotationsFile {
    /// Both boards empty, each seeded with its image/dims/anchors; no sprites.
    pub fn empty() -> Self {
        Self {
            fall_of_khartoum: MapData::empty_fall_of_khartoum(),
            campaign: MapData::empty_campaign(),
            sprites: SpriteAnnotations::default(),
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
        assert!(Terrain::Hut.blocks_los());
        assert!(Terrain::Building.blocks_los());
        assert!(Terrain::Khartoum.blocks_los());
        assert!(!Terrain::Desert.blocks_los());
        assert!(!Terrain::Palm.blocks_los());
        assert!(Terrain::Palm.is_los_trees());
        assert!(!Terrain::Desert.is_los_trees());
    }
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
pub use strum::{EnumProperty, IntoEnumIterator};

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
    strum::EnumProperty,
    strum::FromRepr,
)]
#[repr(u8)]
/// Hex terrain types used on the Omdurman map.
pub enum Terrain {
    #[default]
    /// color: sandy
    #[strum(props(Color = "sandy"))]
    Desert,
    /// color: green_brown
    #[strum(props(Color = "green_brown"))]
    Shrubs,
    /// color: dark_green
    #[strum(props(Color = "dark_green"))]
    Palm,
    /// color: blue
    #[strum(props(Color = "blue"))]
    BlueNile,
    /// color: light_blue
    #[strum(props(Color = "light_blue"))]
    WhiteNile,
    /// color: gray
    #[strum(props(Color = "gray"))]
    Fortress,
    /// color: dark_red
    #[strum(props(Color = "dark_red"))]
    Khartoum,
    /// color: light_green
    #[strum(props(Color = "light_green"))]
    Tuti,
    /// color: medium_green
    #[strum(props(Color = "medium_green"))]
    Hogali,
    /// color: olive
    #[strum(props(Color = "olive"))]
    Buri,
    /// color: dark_gray
    #[strum(props(Color = "dark_gray"))]
    FortBuri,
    /// color: dark_red_brown
    #[strum(props(Color = "dark_red_brown"))]
    FortMakran,
    /// color: dark_blue_gray
    #[strum(props(Color = "dark_blue_gray"))]
    NorthFort,
}

impl Terrain {
    pub fn to_u8(self) -> u8 {
        self as u8
    }
    pub fn from_u8(v: u8) -> Self {
        Self::from_repr(v).unwrap_or(Self::Desert)
    }
    pub fn passable_by_land(self) -> bool {
        !matches!(self, Terrain::BlueNile | Terrain::WhiteNile)
    }

    pub fn is_city(self) -> bool {
        matches!(self, Terrain::Khartoum)
    }

    pub fn is_village(self) -> bool {
        matches!(self, Terrain::Tuti | Terrain::Hogali | Terrain::Buri)
    }

    pub fn is_nile(self) -> bool {
        matches!(self, Terrain::BlueNile | Terrain::WhiteNile)
    }

    pub fn is_fort(self) -> bool {
        matches!(
            self,
            Terrain::Fortress | Terrain::FortBuri | Terrain::FortMakran | Terrain::NorthFort
        )
    }

    /// Return a RGBA colour suitable for a terrain-type overlay.
    /// The colour names are stored via `strum(props(Color = …))` and matched
    /// here so the two sources stay in sync.
    /// Warm palette inspired by Sudanese landscape (sand, Nile, khaki, earth).
    pub fn overlay_color(&self) -> [f32; 4] {
        match self.get_str("Color").unwrap_or("sandy") {
            "sandy" => [0.90, 0.78, 0.40, 0.75],
            "green_brown" => [0.60, 0.55, 0.22, 0.75],
            "dark_green" => [0.28, 0.55, 0.15, 0.75],
            "blue" => [0.18, 0.55, 0.68, 0.75],
            "light_blue" => [0.42, 0.78, 0.82, 0.75],
            "gray" => [0.65, 0.50, 0.38, 0.75],
            "dark_red" => [0.62, 0.22, 0.08, 0.75],
            "light_green" => [0.50, 0.68, 0.22, 0.75],
            "medium_green" => [0.38, 0.55, 0.18, 0.75],
            "olive" => [0.68, 0.55, 0.10, 0.75],
            "dark_gray" => [0.42, 0.32, 0.25, 0.75],
            "dark_red_brown" => [0.58, 0.20, 0.08, 0.75],
            "dark_blue_gray" => [0.22, 0.30, 0.45, 0.75],
            _ => [0.50, 0.50, 0.50, 0.75],
        }
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
}

impl HexData {
    /// Hex with terrain and an optional name. Locations are set elsewhere
    /// from the static `LOCATIONS` table.
    pub fn new(terrain: Terrain, name: Option<String>) -> Self {
        Self {
            terrain,
            location: None,
            name,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::EnumIter)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpriteAnnotation {
    pub color: SpriteColor,
    pub faction: Faction,
    pub text: String,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    #[serde(default)]
    pub is_boat: bool,
    #[serde(default = "default_true")]
    pub is_unit: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SpriteAnnotations {
    pub units: indexmap::IndexMap<String, indexmap::IndexMap<(u32, u32), SpriteAnnotation>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileInfo {
    pub terrain: Terrain,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapSection {
    pub tiles: HashMap<(i32, i32), TileInfo>,
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
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnnotationsFile {
    pub map: MapSection,
    pub overlay: OverlayParams,
    pub sprites: SpriteAnnotations,
}

impl AnnotationsFile {
    pub fn empty() -> Self {
        Self {
            map: MapSection {
                tiles: HashMap::new(),
            },
            overlay: OverlayParams::default(),
            sprites: SpriteAnnotations {
                units: indexmap::IndexMap::new(),
            },
        }
    }
}

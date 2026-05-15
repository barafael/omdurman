use std::collections::HashMap;

use serde::{Deserialize, Serialize};
pub use strum::IntoEnumIterator;

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
)]
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
}

impl Terrain {
    pub fn passable_by_land(self) -> bool {
        !matches!(self, Terrain::BlueNile | Terrain::WhiteNile)
    }

    pub fn is_city(self) -> bool {
        matches!(self, Terrain::Khartoum)
    }

    pub fn is_village(self) -> bool {
        matches!(self, Terrain::Tuti | Terrain::Hogali | Terrain::Buri)
    }

    pub fn is_fort(self) -> bool {
        matches!(
            self,
            Terrain::Fortress | Terrain::FortBuri | Terrain::FortMakran | Terrain::NorthFort
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
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

#[derive(Clone, Debug)]
pub struct HexData {
    pub terrain: Terrain,
    pub location: Option<Location>,
    pub name: Option<String>,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpriteAnnotation {
    pub color: SpriteColor,
    pub faction: Faction,
    pub text: String,
    pub a: i32,
    pub b: i32,
    pub c: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

fn default_fp() -> bool { false }
fn default_el() -> bool { true }

/// Parameters that define the hex overlay grid: dimensions, size, position, and
/// row-stagger shape.  Shared by serialization, the in-memory game map, and the
/// editor overlay resource so there is a single source of truth.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OverlayParams {
    pub width: i32,
    pub height: i32,
    pub hex_size: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub stagger: f32,
    #[serde(default = "default_fp")]
    pub flip_parity: bool,
    #[serde(default = "default_el")]
    pub equal_length: bool,
}

impl OverlayParams {
    /// `0.0` when `flip_parity` is false, `1.0` when true.
    /// Used to shift the phase of the row stagger so that even/odd alignment
    /// flips.
    pub fn phase(&self) -> f32 {
        if self.flip_parity { 1.0 } else { 0.0 }
    }
}

impl Default for OverlayParams {
    fn default() -> Self {
        Self {
            width: 48,
            height: 16,
            hex_size: 51.0,
            offset_x: -1.0,
            offset_y: 1.0,
            stagger: -0.5,
            flip_parity: false,
            equal_length: true,
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

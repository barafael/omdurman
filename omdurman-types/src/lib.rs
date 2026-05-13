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
    City,
    Village,
    Fortress,
    Settlement,
}

impl Terrain {
    pub fn passable_by_land(self) -> bool {
        !matches!(self, Terrain::BlueNile | Terrain::WhiteNile)
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

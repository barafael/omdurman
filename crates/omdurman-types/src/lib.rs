use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    pub fn variants() -> &'static [Terrain] {
        &[
            Terrain::Desert,
            Terrain::Shrubs,
            Terrain::Palm,
            Terrain::BlueNile,
            Terrain::WhiteNile,
            Terrain::City,
            Terrain::Village,
            Terrain::Fortress,
            Terrain::Settlement,
        ]
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

#[derive(Clone, Copy, Debug)]
pub struct WallSegment {
    pub gate: Option<&'static str>,
}

#[derive(Default)]
pub struct GameMapData {
    pub hexes: HashMap<HexCoord, HexData>,
}

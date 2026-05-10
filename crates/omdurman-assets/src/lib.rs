use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use omdurman_types::{HexCoord, Location, Terrain};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse: {0}")]
    Parse(#[from] ron::error::SpannedError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileInfo {
    pub terrain: Terrain,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MapInfo {
    pub tiles: HashMap<(i32, i32), TileInfo>,
}

const MAP_INFO_PATH: &str = "assets/map_info.ron";

pub fn load_map_info() -> Result<MapInfo, LoadError> {
    let contents = std::fs::read_to_string(MAP_INFO_PATH)?;
    let info = ron::from_str::<MapInfo>(&contents)?;
    Ok(info)
}

pub fn save_map_info(tiles: HashMap<(i32, i32), TileInfo>) -> Result<(), std::io::Error> {
    let info = MapInfo { tiles };
    let contents = ron::to_string(&info).expect("MapInfo is always serializable");
    std::fs::write(MAP_INFO_PATH, contents)
}

pub const CROSS_REFS: &[(HexCoord, (f32, f32))] = &[
    (HexCoord::new(0, 1), (735.0, 523.0)),
    (HexCoord::new(0, 2), (736.0, 625.0)),
    (HexCoord::new(2, 0), (913.0, 523.0)),
    (HexCoord::new(2, 1), (913.0, 625.0)),
    (HexCoord::new(9, -8), (1532.0, 66.0)),
    (HexCoord::new(-5, 5), (292.0, 677.0)),
    (HexCoord::new(-6, 2), (205.0, 320.0)),
    (HexCoord::new(0, 7), (734.0, 1132.0)),
    (HexCoord::new(1, 6), (823.0, 1081.0)),
    (HexCoord::new(2, 6), (912.0, 1132.0)),
];

pub const LOCATIONS: &[(HexCoord, Location)] = &[
    (HexCoord::new(-4, -2), Location::FortMakran),
    (HexCoord::new(9, -7), Location::NorthFort),
    (HexCoord::new(9, -2), Location::FortBuri),
    (HexCoord::new(2, -1), Location::Palace),
    (HexCoord::new(4, -1), Location::Arsenal),
    (HexCoord::new(0, 0), Location::AustrianMission),
    (HexCoord::new(5, -1), Location::Barracks),
    (HexCoord::new(3, 4), Location::KalaklaGate),
    (HexCoord::new(5, 2), Location::MessalamiaGate),
    (HexCoord::new(9, -1), Location::BuriGate),
    (HexCoord::new(2, -5), Location::Tuti),
    (HexCoord::new(9, -8), Location::Hogali),
];

pub fn terrain_for_location(loc: Location) -> Terrain {
    match loc {
        Location::FortMakran
        | Location::NorthFort
        | Location::FortBuri
        | Location::KalaklaGate
        | Location::MessalamiaGate
        | Location::BuriGate => Terrain::Desert,
        Location::AustrianMission | Location::Palace | Location::Arsenal | Location::Barracks => {
            Terrain::City
        }
        Location::Tuti => Terrain::Palm,
        Location::Hogali | Location::BuriSettlement => Terrain::Settlement,
    }
}

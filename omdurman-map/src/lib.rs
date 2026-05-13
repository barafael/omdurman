use std::collections::HashMap;

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use omdurman_types::{HexCoord, HexData, Location, Terrain};

// ── Serialization types ──────────────────────────────────────────────────

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

#[derive(Asset, TypePath, Serialize, Deserialize, Debug, Clone)]
pub struct MapInfo {
    pub tiles: HashMap<(i32, i32), TileInfo>,
}

#[derive(Default, TypePath)]
pub struct MapInfoLoader;

impl AssetLoader for MapInfoLoader {
    type Asset = MapInfo;
    type Settings = ();
    type Error = LoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let info = ron::de::from_bytes::<MapInfo>(&bytes)?;
        Ok(info)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_map_info(
    path: &str,
    tiles: HashMap<(i32, i32), TileInfo>,
) -> Result<(), std::io::Error> {
    let info = MapInfo { tiles };
    let contents = ron::ser::to_string_pretty(&info, ron::ser::PrettyConfig::default())
        .expect("MapInfo is always serializable");
    std::fs::write(path, contents)
}

// ── Calibration / location tables ────────────────────────────────────────

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

/// Path used to load the map from the `assets/` folder (resolved by Bevy's [`AssetServer`]).
pub const MAP_INFO_ASSET_PATH: &str = "map_info.ron";

/// Resource holding the in-flight handle for [`MapInfo`].
#[derive(Resource, Default)]
pub struct MapInfoHandle {
    pub handle: Handle<MapInfo>,
    pub applied: bool,
}

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

// ── Runtime game map ─────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct GameMap {
    pub hexes: HashMap<HexCoord, HexData>,
}

/// Startup system: kick off async load of `map_info.ron` via the [`AssetServer`].
pub fn start_loading_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load::<MapInfo>(MAP_INFO_ASSET_PATH);
    commands.insert_resource(MapInfoHandle {
        handle,
        applied: false,
    });
}

/// Update system: once the [`MapInfo`] asset has loaded, copy its tiles into [`GameMap`].
/// Runs every frame but only does work once.
pub fn apply_loaded_map(
    mut map_info_handle: ResMut<MapInfoHandle>,
    map_infos: Res<Assets<MapInfo>>,
    mut game_map: ResMut<GameMap>,
    asset_server: Res<AssetServer>,
) {
    if map_info_handle.applied {
        return;
    }
    use bevy::asset::LoadState;
    match asset_server.load_state(&map_info_handle.handle) {
        LoadState::Loaded => {
            let Some(info) = map_infos.get(&map_info_handle.handle) else {
                return;
            };
            for ((q, r), tile) in &info.tiles {
                game_map.hexes.insert(
                    HexCoord::new(*q, *r),
                    HexData {
                        terrain: tile.terrain,
                        location: None,
                        name: tile.name.clone(),
                    },
                );
            }
            info!("loaded {} hexes from map_info.ron", game_map.hexes.len());
            map_info_handle.applied = true;
        }
        LoadState::Failed(error) => {
            warn!("map_info.ron not loaded ({error}), starting with empty map");
            map_info_handle.applied = true;
        }
        _ => {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game_map(game_map: &GameMap, path: &str) {
    let tiles: HashMap<(i32, i32), TileInfo> = game_map
        .hexes
        .iter()
        .map(|(coord, data)| {
            (
                (coord.q, coord.r),
                TileInfo {
                    terrain: data.terrain,
                    name: data.name.clone(),
                },
            )
        })
        .collect();
    match save_map_info(path, tiles) {
        Ok(()) => info!("saved {} hexes to map_info.ron", game_map.hexes.len()),
        Err(e) => error!("failed to save map_info.ron: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn save_game_map(_game_map: &GameMap, _path: &str) {
    warn!("save_game_map is not supported on wasm");
}

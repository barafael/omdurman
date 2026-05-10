use std::collections::HashMap;

use bevy::prelude::*;
use omdurman_assets::{MapInfoPath, TileInfo, save_map_info};
use omdurman_types::{HexCoord, HexData};

#[derive(Resource, Default)]
pub struct GameMap {
    pub hexes: HashMap<HexCoord, HexData>,
}

pub fn load_saved_map(mut game_map: ResMut<GameMap>, path: Res<MapInfoPath>) {
    match omdurman_assets::load_map_info(&path.0) {
        Ok(info) => {
            for ((q, r), tile) in info.tiles {
                game_map.hexes.insert(
                    HexCoord::new(q, r),
                    HexData {
                        terrain: tile.terrain,
                        location: None,
                        name: tile.name,
                    },
                );
            }
            info!("loaded {} hexes from map_info.ron", game_map.hexes.len());
        }
        Err(e) => {
            warn!("map_info.ron not loaded ({e}), starting with empty map");
        }
    }
}

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

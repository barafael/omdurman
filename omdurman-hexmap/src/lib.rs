pub mod layout;
pub mod map;
pub mod plane;
pub mod world;

// Explicit re-exports: only items actually consumed by other workspace crates.
// layout.rs
pub use layout::{
    CalibrationAnchor, HexLayout, IMG_H, IMG_W, MapDims, SQRT_3, pixel_to_world_dims,
};
// map.rs
pub use map::{GameMap, clip_hexes_to_overlay, load_map_data};
// plane.rs
pub use plane::{
    HexOverlay, MapPlane, MapTextureCache, PlaneTextureStores, apply_map_data_to_plane,
    hex_ring_mesh, terrain_overlay_color,
};
// world.rs
pub use world::{hex_local_pos, hex_world_pos, hit_to_hex, local_to_world};

use bevy::prelude::*;

/// Registers the hex map resources (`GameMap`, `MapDims`) with Bevy.
///
/// `HexLayout` is **not** inserted by this plugin because it requires
/// game-specific calibration data. Insert it manually with
/// `app.insert_resource(HexLayout::calibrated(...))`.
pub struct HexMapPlugin;

impl Plugin for HexMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(map::GameMap::default());
        app.insert_resource(layout::MapDims::default());
    }
}

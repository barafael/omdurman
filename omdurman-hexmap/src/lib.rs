pub mod layout;
pub mod plane;
pub mod world;

// Explicit re-exports: only items actually consumed by other workspace crates.
// layout.rs
pub use layout::{CalibrationAnchor, HexLayout, MapDims, pixel_to_world_dims};
// plane.rs
pub use plane::{HexOverlay, MapPlane, MapTextureCache, apply_map_data_to_plane, hex_ring_mesh};
// world.rs
pub use world::{hex_world_pos, hit_to_hex};

use bevy::prelude::*;

/// Registers the hex map resource (`MapDims`) with Bevy. The board's
/// topology is the rules engine's `BoardInfo`; this crate only draws it.
///
/// `HexLayout` is **not** inserted by this plugin because it requires
/// game-specific calibration data. Insert it manually with
/// `app.insert_resource(HexLayout::calibrated(...))`.
pub struct HexMapPlugin;

impl Plugin for HexMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(layout::MapDims::default());
    }
}

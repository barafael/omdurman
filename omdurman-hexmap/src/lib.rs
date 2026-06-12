pub mod layout;
pub mod map;
pub mod world;

pub use layout::*;
pub use map::*;
pub use world::*;

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

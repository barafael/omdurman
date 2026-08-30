//! RTS camera wiring for the editor. The controls (right-drag pan, arrows,
//! scroll zoom, Ctrl+scroll / PgUp/PgDn tilt, touch gestures) are the same as
//! the game's and live in `omdurman-board-ui::camera`; this module only
//! registers them (no night shading, no picking markers, no run conditions).

use bevy::prelude::*;

pub use omdurman_board_ui::camera::{CameraDragState, CameraSettings, RtsCamera};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings::default())
            .insert_resource(CameraDragState::default())
            .add_systems(Startup, omdurman_board_ui::camera::spawn_camera)
            .add_systems(Update, omdurman_board_ui::camera::camera_control);
    }
}

//! RTS camera wiring for the game. The controls (right-drag pan, arrows,
//! scroll zoom, Ctrl+scroll / PgUp/PgDn tilt, touch gestures) live in
//! `omdurman-board-ui::camera`; this module only registers them (with the
//! game's run condition), spawns the camera with its mesh-picking marker,
//! mirrors the replicated day/night into [`BoardDayNight`], and registers the
//! shared night shading.

use bevy::{prelude::*, render::view::ColorGrading};
use omdurman_board_ui::night::{BoardDayNight, night_shading};

pub use omdurman_board_ui::camera::{CameraDragState, CameraSettings, RtsCamera, RtsCameraState};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings::default())
            .insert_resource(CameraDragState::default())
            .init_resource::<BoardDayNight>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    camera_control.run_if(crate::camera_enabled),
                    night_shading,
                    sync_board_day_night,
                ),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        RtsCamera,
        RtsCameraState::default(),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        Tonemapping::None,
        ColorGrading::default(),
        // Picking marker: the mesh backend only casts from marked cameras.
        crate::picking::picking_camera(),
    ));
}

use bevy::core_pipeline::tonemapping::Tonemapping;
use omdurman_board_ui::camera::camera_control;

/// Mirror the replicated rules state's time of day into the shared resource
/// the night shading reads (§night tint).
fn sync_board_day_night(
    game_state: Option<Res<crate::GameStateResource>>,
    mut day_night: ResMut<BoardDayNight>,
) {
    day_night.0 = game_state.as_deref().map(|gs| gs.0.day_night);
}

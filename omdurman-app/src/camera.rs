//! RTS camera wiring for the game. The controls (right-drag pan, arrows,
//! scroll zoom, Ctrl+scroll / PgUp/PgDn tilt, touch gestures, Home to fit) live in
//! `omdurman-board-ui::camera`; this module only registers them (with the
//! game's run condition), spawns the camera with its mesh-picking marker,
//! mirrors the replicated day/night into [`BoardDayNight`], and registers the
//! shared night shading.

use bevy::{prelude::*, render::view::ColorGrading};
use omdurman_board_ui::night::{BoardDayNight, night_shading};

pub use omdurman_board_ui::camera::{
    CameraDragState, CameraFit, CameraSettings, CameraSettling, CameraViewInsets, RtsCamera,
    RtsCameraState,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings::default())
            .insert_resource(CameraDragState::default())
            .init_resource::<CameraFit>()
            // "Still moving" flag: keeps frames coming while the view eases
            // (see `activity`).
            .init_resource::<CameraSettling>()
            .init_resource::<CameraViewInsets>()
            .init_resource::<BoardDayNight>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    camera_control.run_if(crate::camera_enabled),
                    night_shading,
                    sync_board_day_night,
                ),
            )
            .add_systems(Last, publish_camera_insets)
            // Entering the board view (a game start, a rejoin, returning from
            // the menu) frames the whole board beside the freshly shown panels.
            .add_systems(
                OnEnter(crate::state::AppMode::Game),
                |mut fit: ResMut<CameraFit>| fit.request(),
            )
            // A resized window re-frames the board (the old framing may have
            // pushed it off-screen or under the sidebar).
            .add_systems(
                Update,
                |mut resized: MessageReader<bevy::window::WindowResized>,
                 mut fit: ResMut<CameraFit>| {
                    if resized.read().count() > 0 {
                        fit.request();
                    }
                },
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
    // Written only when it changes: the night shading keys on it.
    let now = game_state.as_deref().map(|gs| gs.0.day_night);
    if day_night.0 != now {
        day_night.0 = now;
    }
}

/// Hand the chrome bands this frame's egui pass reserved (left rail panels,
/// top bar, charts sheet) to the camera, so fitting the board (Home, board
/// load) centres it in the uncovered part of the window.
fn publish_camera_insets(layout: Res<crate::ScreenLayout>, mut insets: ResMut<CameraViewInsets>) {
    insets.set_if_neq(CameraViewInsets {
        left: layout.left_inset,
        right: layout.right_inset,
        top: layout.top_bar_height,
        bottom: 0.0,
    });
}

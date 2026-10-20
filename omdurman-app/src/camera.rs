//! RTS camera wiring for the game. The controls (right-drag pan, arrows,
//! scroll zoom, Ctrl+scroll / PgUp/PgDn tilt, touch gestures, Home to fit) live in
//! `omdurman-board-ui::camera`; this module only registers them (with the
//! game's run condition), spawns the board camera with its mesh-picking
//! marker and the UI camera, mirrors the replicated day/night into
//! [`BoardDayNight`], and registers the shared night shading.
//!
//! Two cameras: the board's 3D camera ([`RtsCamera`]) and, drawn over it, a
//! 2D camera ([`UiCamera`]) that every Bevy UI node and the egui pass render
//! through. The board camera is switched off while an opaque screen (the
//! title screen without a game behind it, the lobby) covers it
//! (`splash::screen::sync_board_camera`): a frame then costs the UI alone,
//! not the whole 3D pipeline for a board nobody sees.

use bevy::prelude::*;
use bevy::render::view::{ColorGrading, Tonemapping};
use bevy_egui::{EguiGlobalSettings, PrimaryEguiContext};
use omdurman_board_ui::night::{BoardDayNight, NightFading, night_shading};

pub use omdurman_board_ui::camera::{
    CameraDragState, CameraFit, CameraSettings, CameraSettling, CameraViewInsets, RtsCamera,
    RtsCameraState,
};

/// The 2D camera the UI (Bevy UI and egui) renders through.
#[derive(Component)]
pub struct UiCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // egui would otherwise attach its primary context to the first camera
        // it sees (the board's); it belongs to the UI camera, which carries
        // `PrimaryEguiContext` itself.
        app.world_mut()
            .resource_mut::<EguiGlobalSettings>()
            .auto_create_primary_context = false;
        app.insert_resource(CameraSettings::default())
            .insert_resource(CameraDragState::default())
            .init_resource::<CameraFit>()
            // "Still moving" flag: keeps frames coming while the view eases
            // (see `activity`).
            .init_resource::<CameraSettling>()
            .init_resource::<CameraViewInsets>()
            .init_resource::<BoardDayNight>()
            // "Still fading" flag of the day/night grading (see `activity`).
            .init_resource::<NightFading>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    camera_control.run_if(crate::off_title),
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
        // The identity tone curve: the board scan keeps its own colours,
        // still dithered.
        Tonemapping::Linear,
        ColorGrading::default(),
        // Picking marker: the mesh backend only casts from marked cameras.
        crate::picking::picking_camera(),
    ));
    commands.spawn((
        UiCamera,
        Camera2d,
        Camera {
            // Over the board camera, which clears the window; while that one
            // is off, this one clears (`sync_board_camera`).
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        // (The same multisampling as the board camera, deliberately: Bevy
        // shares a window's main textures between cameras of one MSAA
        // setting, and a camera of another gets its own -- whose blit would
        // then replace the board's with stale pixels instead of drawing the
        // UI over it.)
        IsDefaultUiCamera,
        PrimaryEguiContext,
    ));
}

use omdurman_board_ui::camera::camera_control;

/// Mirror the replicated rules state's time of day into the shared resource
/// the night shading reads (§night tint).
fn sync_board_day_night(
    game_state: Res<crate::GameStateResource>,
    mut day_night: ResMut<BoardDayNight>,
) {
    // Written only when it changes: the night shading keys on it.
    let now = Some(game_state.0.day_night);
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

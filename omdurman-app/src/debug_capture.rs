//! Off-by-default screenshot capture for verifying UI/theme work.
//!
//! Entirely inert unless `OMDURMAN_SCREENSHOT` is set to an output path. When
//! set, the app captures the primary window once, some frames in (so the splash
//! and egui have rendered), writes a PNG, and exits. This is a development
//! verification aid -- the physics/game paths are untouched.
//!
//! ```text
//! OMDURMAN_SCREENSHOT=out.png OMDURMAN_SCREENSHOT_FRAMES=600 cargo run -p omdurman-app
//! ```
//! `OMDURMAN_SCREENSHOT_FRAMES` (optional) sets the capture frame; default 300.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

/// The output path and target frame, read once from the environment.
#[derive(Resource)]
struct CaptureConfig {
    path: String,
    at_frame: u32,
}

#[derive(Resource, Default)]
struct FrameCounter(u32);

pub struct DebugCapturePlugin;

impl Plugin for DebugCapturePlugin {
    fn build(&self, app: &mut App) {
        let Ok(path) = std::env::var("OMDURMAN_SCREENSHOT") else {
            return; // Not requested -- add nothing.
        };
        let at_frame = std::env::var("OMDURMAN_SCREENSHOT_FRAMES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);
        info!(%path, at_frame, "debug capture armed");
        app.insert_resource(CaptureConfig { path, at_frame })
            .init_resource::<FrameCounter>()
            .add_systems(Update, capture_then_exit);
    }
}

fn capture_then_exit(
    mut commands: Commands,
    config: Res<CaptureConfig>,
    mut counter: ResMut<FrameCounter>,
    mut captured_at: Local<Option<u32>>,
    mut exit: MessageWriter<AppExit>,
) {
    counter.0 += 1;
    if let Some(at) = *captured_at {
        // Hold for a margin of frames after the capture request: the screenshot
        // is read back on the render thread and delivered through a channel, so
        // exiting the same frame closes the channel before the image lands on
        // disk ("sending on a closed channel"). ~30 frames is ample.
        if counter.0 >= at + 30 {
            exit.write(AppExit::Success);
        }
        return;
    }
    if counter.0 >= config.at_frame {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(config.path.clone()));
        info!(frame = counter.0, path = %config.path, "capturing screenshot");
        *captured_at = Some(counter.0);
    }
}

/// Off-by-default hex probe for scripted play-testing: with
/// `OMDURMAN_HEX_PROBE=<path>` set, the app rewrites `<path>` twice a second
/// with one `q r x y` line per board hex -- the hex centre in window
/// (logical) pixels under the current camera -- so an external driver
/// (xdotool) can click a hex by coordinate. Creating `<path>.shot` requests a
/// window screenshot, written to `<path>.png` (the request file is removed).
/// Inert when unset.
pub struct HexProbePlugin;

#[derive(Resource)]
struct HexProbePath(String);

impl Plugin for HexProbePlugin {
    fn build(&self, app: &mut App) {
        let Ok(path) = std::env::var("OMDURMAN_HEX_PROBE") else {
            return;
        };
        info!(%path, "hex probe armed");
        app.insert_resource(HexProbePath(path))
            .add_systems(Last, (write_hex_probe, probe_screenshot));
    }
}

/// The board resources the probe projects hex centres from.
#[derive(bevy::ecs::system::SystemParam)]
struct ProbeBoard<'w> {
    map: Option<Res<'w, omdurman_hexmap::GameMap>>,
    layout: Option<Res<'w, omdurman_hexmap::HexLayout>>,
    overlay: Option<Res<'w, omdurman_hexmap::HexOverlay>>,
}

fn write_hex_probe(
    path: Res<HexProbePath>,
    time: Res<Time>,
    mut last: Local<f64>,
    board: ProbeBoard,
    cams: Query<(&Camera, &GlobalTransform), With<crate::camera::RtsCamera>>,
    game_state: Option<Res<crate::GameStateResource>>,
) {
    let now = time.elapsed_secs_f64();
    if now - *last < 0.5 {
        return;
    }
    *last = now;
    let ProbeBoard {
        map,
        layout,
        overlay,
    } = board;
    let (Some(map), Some(layout), Some(overlay), Ok((camera, cam_tf))) =
        (map, layout, overlay, cams.single())
    else {
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
    let mut out = String::new();
    for coord in map.hexes.keys() {
        let world = omdurman_hexmap::hex_world_pos(*coord, origin, &overlay.params);
        if let Ok(screen) = camera.world_to_viewport(cam_tf, world) {
            out.push_str(&format!(
                "{} {} {:.0} {:.0}\n",
                coord.q, coord.r, screen.x, screen.y
            ));
        }
    }
    let _ = std::fs::write(&path.0, out);
    // The rules state beside it: `<path>.state` has the turn/phase and one
    // `U owner q r disrupted id kind` line per unit.
    if let Some(gs) = game_state {
        let gs = &gs.0;
        let mut state = format!(
            "T turn={:?} phase={:?} active={:?}\n",
            gs.current_turn,
            gs.phase,
            gs.player_to_act().unwrap_or(gs.phase_player())
        );
        for u in &gs.units {
            state.push_str(&format!(
                "U {:?} {} {} {} {:?} {:?}\n",
                u.profile.identity.owner(),
                u.position.q,
                u.position.r,
                u.state.disrupted,
                u.id,
                u.profile.identity,
            ));
        }
        let _ = std::fs::write(format!("{}.state", path.0), state);
    }
}

/// Serve a `<probe>.shot` screenshot request (see [`HexProbePlugin`]).
fn probe_screenshot(mut commands: Commands, path: Res<HexProbePath>) {
    let request = format!("{}.shot", path.0);
    if std::fs::remove_file(&request).is_ok() {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("{}.png", path.0)));
    }
}

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

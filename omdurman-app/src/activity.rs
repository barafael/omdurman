//! Frame pacing: the app redraws only when something happens.
//!
//! Bevy's default update mode runs every system and renders a frame at the
//! display rate even when nothing on screen changes -- an idle game (a
//! finished battle left open, the lobby) kept a laptop's fans spinning. The
//! app runs *reactively* instead ([`WinitSettings`]): a frame on input, on a
//! window event, or after [`FOCUSED_WAIT`] at the latest (the network socket
//! is drained on frames, so the wait bounds inbound latency).
//!
//! Whatever needs consecutive frames -- a counter gliding between hexes, the
//! camera easing, the AI's paced moves, retransmissions waiting for an echo,
//! a playing timeline -- marks [`Activity`] for the frame, and
//! [`request_redraws`] asks for the next one. egui's own animations already
//! request repaints through bevy_egui.
//!
//! bevy_egui 0.42 feeds egui a `ModifiersChanged` event on *every* frame,
//! and egui answers any input event with an immediate repaint, which
//! bevy_egui turns into a redraw request -- a self-sustaining 60 fps loop
//! that defeated the reactive mode entirely. [`drop_unchanged_modifiers`]
//! removes that event unless the modifier keys really changed.

use std::time::Duration;

use bevy::prelude::*;
use bevy::window::RequestRedraw;
use bevy::winit::{UpdateMode, WinitSettings};

/// The longest a focused window sleeps between frames. Incoming network
/// messages are only read on a frame, so this is their worst-case latency.
pub const FOCUSED_WAIT: Duration = Duration::from_millis(100);
/// The same for an unfocused window: other players' moves still arrive.
pub const UNFOCUSED_WAIT: Duration = Duration::from_millis(500);
/// Frame rate for slow ambient motion (the title screen's and the lobby's
/// panning maps). They drift at most ~11 points a second (the lobby half
/// that), so at this rate a frame moves the blurred map by under a point;
/// display-rate frames only redrew sub-pixel steps, at ~95% CPU (15 fps:
/// ~13%).
pub const AMBIENT_FPS: f32 = 15.0;
/// The same for an unfocused window: still drifting, at a few frames a
/// second, for a menu left open on another screen.
pub const AMBIENT_UNFOCUSED_FPS: f32 = 5.0;

/// Something on screen or in the game loop needs the next frame now: set by
/// the animating systems during a frame, consumed by [`request_redraws`].
#[derive(Resource, Default, Debug)]
pub struct Activity {
    busy: bool,
    ambient: bool,
}

impl Activity {
    /// Ask for another frame right after this one.
    pub fn keep_running(&mut self) {
        self.busy = true;
    }

    /// Ask for frames at [`AMBIENT_FPS`]: slow continuous motion that needs
    /// regular frames, but not every display refresh.
    pub fn keep_ambient(&mut self) {
        self.ambient = true;
    }

    /// Whether some system asked for another frame this frame.
    pub fn is_busy(&self) -> bool {
        self.busy
    }

    /// Whether some system asked for ambient-rate frames this frame.
    pub fn is_ambient(&self) -> bool {
        self.ambient
    }
}

/// The frame rate for ambient motion: [`ambient_fps`] at start, retunable at
/// run time (the title screen's tuning pane).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct AmbientFps(pub f32);

impl Default for AmbientFps {
    fn default() -> Self {
        Self(ambient_fps())
    }
}

/// Reactive frame pacing plus the redraw requests that keep animations
/// smooth.
pub struct ActivityPlugin;

impl Plugin for ActivityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(pacing(false, ambient_fps()))
            .init_resource::<Activity>()
            .init_resource::<AmbientFps>()
            .add_systems(
                PreUpdate,
                drop_unchanged_modifiers
                    .after(bevy_egui::EguiPreUpdateSet::ProcessInput)
                    .before(bevy_egui::EguiPreUpdateSet::BeginPass),
            )
            .add_systems(Last, (camera_activity, request_redraws).chain());
    }
}

/// The camera eases toward its target over several frames (and is driven
/// by held keys or a drag): keep running while it says it is settling. The
/// flag is consumed here; `camera_control` raises it again each frame it
/// still moves, so a camera that stops being updated (another mode) cannot
/// leave it stuck. The day/night grading fades the same way: keep running
/// until it has settled.
pub fn camera_activity(
    settling: Option<ResMut<crate::camera::CameraSettling>>,
    night: Option<Res<omdurman_board_ui::night::NightFading>>,
    mut activity: ResMut<Activity>,
) {
    if let Some(mut settling) = settling
        && settling.0
    {
        settling.0 = false;
        activity.keep_running();
    }
    if night.is_some_and(|fading| fading.0) {
        activity.keep_running();
    }
}

/// Drop bevy_egui's per-frame `ModifiersChanged` when the modifiers are the
/// ones egui already has (see the module docs): an idle frame then carries
/// no input events, and egui stops asking for the next frame.
pub fn drop_unchanged_modifiers(
    mut inputs: Query<(Entity, &mut bevy_egui::EguiInput)>,
    mut last: Local<bevy::platform::collections::HashMap<Entity, bevy_egui::egui::Modifiers>>,
) {
    use bevy_egui::egui::Event;
    for (entity, mut input) in &mut inputs {
        input.0.events.retain(|event| match event {
            Event::ModifiersChanged(now) => last.insert(entity, *now) != Some(*now),
            _ => true,
        });
    }
}

/// End of the frame: if anything asked to keep running, request the next
/// frame at once (instead of waiting for input or the timeout); for ambient
/// motion only, shorten the reactive wait to the ambient frame time. Then
/// clear the flags for the next frame.
pub fn request_redraws(
    mut activity: ResMut<Activity>,
    fps: Option<Res<AmbientFps>>,
    redraw: Option<ResMut<Messages<RequestRedraw>>>,
    settings: Option<ResMut<WinitSettings>>,
) {
    let busy = std::mem::take(&mut activity.busy);
    let ambient = std::mem::take(&mut activity.ambient);
    if busy && let Some(mut redraw) = redraw {
        redraw.write(RequestRedraw);
    }
    // Write only on a change: the pacing stays put for whole screens.
    let fps = fps.map_or_else(ambient_fps, |fps| fps.0);
    let want = pacing(ambient && !busy, fps);
    if let Some(mut settings) = settings
        && (settings.focused_mode != want.focused_mode
            || settings.unfocused_mode != want.unfocused_mode)
    {
        *settings = want;
    }
}

/// The frame pacing: the idle waits, or the ambient frame times (at `fps`
/// while focused).
fn pacing(ambient: bool, fps: f32) -> WinitSettings {
    if ambient {
        let frame = |fps: f32| Duration::from_secs_f32(1.0 / fps.max(1.0));
        WinitSettings {
            focused_mode: UpdateMode::reactive(frame(fps)),
            unfocused_mode: UpdateMode::reactive_low_power(frame(AMBIENT_UNFOCUSED_FPS)),
        }
    } else {
        WinitSettings {
            focused_mode: UpdateMode::reactive(FOCUSED_WAIT),
            unfocused_mode: UpdateMode::reactive_low_power(UNFOCUSED_WAIT),
        }
    }
}

/// [`AMBIENT_FPS`], or `OMDURMAN_AMBIENT_FPS` when set (for measuring).
fn ambient_fps() -> f32 {
    std::env::var("OMDURMAN_AMBIENT_FPS")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|fps| *fps > 0.0)
        .unwrap_or(AMBIENT_FPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambient_motion_paces_frames_without_forcing_them() {
        let mut app = App::new();
        app.add_message::<RequestRedraw>()
            .init_resource::<Activity>()
            .insert_resource(pacing(false, ambient_fps()))
            .add_systems(Last, request_redraws);
        let wait = |app: &App| match app.world().resource::<WinitSettings>().focused_mode {
            UpdateMode::Reactive { wait, .. } => wait,
            UpdateMode::Continuous => Duration::ZERO,
        };

        app.world_mut().resource_mut::<Activity>().keep_ambient();
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<Messages<RequestRedraw>>()
                .drain()
                .count(),
            0,
            "ambient motion asks for no immediate frame"
        );
        assert_eq!(wait(&app), Duration::from_secs_f32(1.0 / ambient_fps()));

        app.update();
        assert_eq!(
            wait(&app),
            FOCUSED_WAIT,
            "back to the idle wait once it stops"
        );
    }

    #[test]
    fn a_busy_frame_requests_the_next_and_then_resets() {
        let mut app = App::new();
        app.add_message::<RequestRedraw>()
            .init_resource::<Activity>()
            .add_systems(Last, request_redraws);
        let requests = |app: &mut App| {
            app.world_mut()
                .resource_mut::<Messages<RequestRedraw>>()
                .drain()
                .count()
        };

        app.update();
        assert_eq!(requests(&mut app), 0, "an idle frame asks for nothing");

        app.world_mut().resource_mut::<Activity>().keep_running();
        app.update();
        assert_eq!(requests(&mut app), 1, "a busy frame asks for the next");
        assert!(!app.world().resource::<Activity>().is_busy());

        app.update();
        assert_eq!(requests(&mut app), 0, "the flag does not stick");
    }
}

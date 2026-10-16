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
//! Ambient motion (the title screen's and the lobby's drifting maps) asks
//! for frames the same way while the window has focus: one per display
//! refresh, paced by vsync. A timer-paced lower rate (the earlier 15 fps)
//! showed the drift as a slideshow whose steps landed on whichever refresh
//! came next after each timer, and every mouse movement woke extra frames
//! in between -- a visible stutter. Unfocused, the motion drops to a few
//! frames a second ([`AMBIENT_UNFOCUSED_FPS`]) for a menu left open on
//! another screen.
//!
//! bevy_egui 0.42 feeds egui a `ModifiersChanged` event on *every* frame,
//! and egui answers any input event with an immediate repaint, which
//! bevy_egui turns into a redraw request -- a self-sustaining 60 fps loop
//! that defeated the reactive mode entirely. [`drop_unchanged_modifiers`]
//! removes that event unless the modifier keys really changed.

use std::time::Duration;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, RequestRedraw};
use bevy::winit::{UpdateMode, WinitSettings};

/// The longest a focused window sleeps between frames. Incoming network
/// messages are only read on a frame, so this is their worst-case latency.
pub const FOCUSED_WAIT: Duration = Duration::from_millis(100);
/// The same for an unfocused window: other players' moves still arrive.
pub const UNFOCUSED_WAIT: Duration = Duration::from_millis(500);
/// Frame rate of ambient motion (the title screen's and the lobby's panning
/// maps) in an unfocused window: still drifting, at a few frames a second,
/// for a menu left open on another screen. Focused, it runs at the display
/// rate.
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

    /// Ask for ambient frames: slow continuous motion, drawn every display
    /// refresh while the window has focus and at [`AMBIENT_UNFOCUSED_FPS`]
    /// while it has not.
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

/// Reactive frame pacing plus the redraw requests that keep animations
/// smooth.
pub struct ActivityPlugin;

impl Plugin for ActivityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(pacing(false))
            .init_resource::<Activity>()
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

/// End of the frame: if anything asked to keep running -- or for ambient
/// frames while the window has focus -- request the next frame at once
/// (instead of waiting for input or the timeout; the swapchain paces it to
/// the display). Ambient motion in an unfocused window only shortens the
/// reactive wait to its frame time. Then clear the flags for the next frame.
pub fn request_redraws(
    mut activity: ResMut<Activity>,
    window: Query<&Window, With<PrimaryWindow>>,
    redraw: Option<ResMut<Messages<RequestRedraw>>>,
    settings: Option<ResMut<WinitSettings>>,
) {
    let busy = std::mem::take(&mut activity.busy);
    let ambient = std::mem::take(&mut activity.ambient);
    // (No window -- a headless app -- counts as focused.)
    let focused = window.single().is_ok_and(|window| window.focused) || window.is_empty();
    if (busy || (ambient && focused))
        && let Some(mut redraw) = redraw
    {
        redraw.write(RequestRedraw);
    }
    // Write only on a change: the pacing stays put for whole screens.
    let want = pacing(ambient && !busy);
    if let Some(mut settings) = settings
        && (settings.focused_mode != want.focused_mode
            || settings.unfocused_mode != want.unfocused_mode)
    {
        *settings = want;
    }
}

/// The frame pacing: the idle waits, with the unfocused wait shortened to
/// the ambient frame time while something drifts. (Focused ambient motion
/// requests its frames outright, so the focused wait never comes into it.)
fn pacing(ambient: bool) -> WinitSettings {
    let unfocused = if ambient {
        Duration::from_secs_f32(1.0 / AMBIENT_UNFOCUSED_FPS)
    } else {
        UNFOCUSED_WAIT
    };
    WinitSettings {
        focused_mode: UpdateMode::reactive(FOCUSED_WAIT),
        unfocused_mode: UpdateMode::reactive_low_power(unfocused),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ambient motion in a focused window asks for the next frame (the
    /// display paces it); in an unfocused one it only shortens the
    /// unfocused wait to its frame time, and the wait returns to the idle
    /// one as soon as the motion stops.
    #[test]
    fn ambient_motion_runs_at_the_display_rate_focused_and_slowly_unfocused() {
        for focused in [true, false] {
            let mut app = App::new();
            app.add_message::<RequestRedraw>()
                .init_resource::<Activity>()
                .insert_resource(pacing(false))
                .add_systems(Last, request_redraws);
            app.world_mut().spawn((
                Window {
                    focused,
                    ..default()
                },
                PrimaryWindow,
            ));
            let requests = |app: &mut App| {
                app.world_mut()
                    .resource_mut::<Messages<RequestRedraw>>()
                    .drain()
                    .count()
            };
            let unfocused_wait =
                |app: &App| match app.world().resource::<WinitSettings>().unfocused_mode {
                    UpdateMode::Reactive { wait, .. } => wait,
                    UpdateMode::Continuous => Duration::ZERO,
                };

            app.world_mut().resource_mut::<Activity>().keep_ambient();
            app.update();
            assert_eq!(
                requests(&mut app),
                usize::from(focused),
                "focused: {focused}"
            );
            assert_eq!(
                unfocused_wait(&app),
                Duration::from_secs_f32(1.0 / AMBIENT_UNFOCUSED_FPS)
            );
            assert!(matches!(
                app.world().resource::<WinitSettings>().focused_mode,
                UpdateMode::Reactive { wait, .. } if wait == FOCUSED_WAIT
            ));

            app.update();
            assert_eq!(requests(&mut app), 0, "the flag does not stick");
            assert_eq!(
                unfocused_wait(&app),
                UNFOCUSED_WAIT,
                "back to the idle wait once it stops"
            );
        }
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

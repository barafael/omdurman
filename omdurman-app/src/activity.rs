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

/// Something on screen or in the game loop needs the next frame now: set by
/// the animating systems during a frame, consumed by [`request_redraws`].
#[derive(Resource, Default, Debug)]
pub struct Activity {
    busy: bool,
}

impl Activity {
    /// Ask for another frame right after this one.
    pub fn keep_running(&mut self) {
        self.busy = true;
    }

    /// Whether some system asked for another frame this frame.
    #[cfg(test)]
    pub fn is_busy(&self) -> bool {
        self.busy
    }
}

/// Reactive frame pacing plus the redraw requests that keep animations
/// smooth.
pub struct ActivityPlugin;

impl Plugin for ActivityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive(FOCUSED_WAIT),
            unfocused_mode: UpdateMode::reactive_low_power(UNFOCUSED_WAIT),
        })
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
/// leave it stuck.
pub fn camera_activity(
    settling: Option<ResMut<crate::camera::CameraSettling>>,
    mut activity: ResMut<Activity>,
) {
    if let Some(mut settling) = settling
        && settling.0
    {
        settling.0 = false;
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
/// frame at once (instead of waiting for input or the timeout), then clear
/// the flag for the next frame.
pub fn request_redraws(
    mut activity: ResMut<Activity>,
    redraw: Option<ResMut<Messages<RequestRedraw>>>,
) {
    if std::mem::take(&mut activity.busy)
        && let Some(mut redraw) = redraw
    {
        redraw.write(RequestRedraw);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

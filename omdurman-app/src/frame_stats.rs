//! Frame statistics on the log, for measuring instead of guessing.
//!
//! The app paces itself reactively (see [`crate::activity`]): a frame runs
//! only when input arrives, a wait expires, or a system asked for the next
//! one. So the two numbers that matter for "is this costing anything" are
//! *how many frames run* on a given screen (a system that keeps asking for
//! frames while nothing moves is a leak) and *how long the main-world
//! schedule takes per frame*. `OMDURMAN_FRAME_STATS=<secs>` logs both every
//! `<secs>` seconds (`OMDURMAN_FRAME_STATS=1` for every second):
//!
//! ```text
//! frame stats: screen=Board frames=61 fps=12.2 busy=58 egui=3 input=12 motion=0 avg_ms=3.1 max_ms=9.8
//! ```
//!
//! `busy` counts the frames some system asked for (`Activity::keep_running`
//! or `keep_ambient`), `egui` those where egui asked for a repaint (an
//! animation, a tooltip delay, a blinking cursor, `request_repaint`),
//! `input` those that saw pointer or key events on the window, `motion`
//! those that saw raw mouse motion (a device event: winit's reactive mode
//! wakes on it even when the pointer is on another window); the rest were
//! window events or the reactive wait expiring. The frame time is the main world's
//! `First`..`Last` span (the systems), not the GPU's.

use std::time::Duration;

use bevy::platform::time::Instant;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::CursorMoved;
use bevy_egui::EguiContexts;

/// Logs frame statistics when `OMDURMAN_FRAME_STATS` is set; otherwise adds
/// nothing.
pub struct FrameStatsPlugin;

impl Plugin for FrameStatsPlugin {
    fn build(&self, app: &mut App) {
        let Some(period) = period() else {
            return;
        };
        app.insert_resource(FrameStats::new(period))
            .add_systems(First, frame_begins)
            .add_systems(Last, frame_ends.before(crate::activity::request_redraws));
    }
}

/// `OMDURMAN_FRAME_STATS=<secs>`: the reporting period (a bare set variable
/// means 5 s).
fn period() -> Option<Duration> {
    let raw = std::env::var("OMDURMAN_FRAME_STATS").ok()?;
    let secs = raw
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|s| *s > 0.0)
        .unwrap_or(5.0);
    Some(Duration::from_secs_f32(secs))
}

#[derive(Resource)]
struct FrameStats {
    period: Duration,
    window_start: Instant,
    frame_start: Option<Instant>,
    frames: u32,
    busy: u32,
    egui: u32,
    input: u32,
    motion: u32,
    total: Duration,
    max: Duration,
}

impl FrameStats {
    fn new(period: Duration) -> Self {
        Self {
            period,
            window_start: Instant::now(),
            frame_start: None,
            frames: 0,
            busy: 0,
            egui: 0,
            input: 0,
            motion: 0,
            total: Duration::ZERO,
            max: Duration::ZERO,
        }
    }

    fn reset(&mut self, now: Instant) {
        self.window_start = now;
        self.frames = 0;
        self.busy = 0;
        self.egui = 0;
        self.input = 0;
        self.motion = 0;
        self.total = Duration::ZERO;
        self.max = Duration::ZERO;
    }
}

fn frame_begins(mut stats: ResMut<FrameStats>) {
    stats.frame_start = Some(Instant::now());
}

#[allow(clippy::too_many_arguments)]
fn frame_ends(
    mut stats: ResMut<FrameStats>,
    activity: Res<crate::activity::Activity>,
    screen: Option<Res<State<crate::Screen>>>,
    mut contexts: EguiContexts,
    mut moved: MessageReader<CursorMoved>,
    mut buttons: MessageReader<MouseButtonInput>,
    mut wheel: MessageReader<MouseWheel>,
    mut keys: MessageReader<KeyboardInput>,
    mut motion: MessageReader<MouseMotion>,
) {
    let now = Instant::now();
    let Some(start) = stats.frame_start.take() else {
        return;
    };
    let took = now - start;
    stats.frames += 1;
    stats.busy += u32::from(activity.is_busy() || activity.is_ambient());
    stats.egui += u32::from(
        contexts
            .ctx_mut()
            .is_ok_and(|ctx| ctx.has_requested_repaint()),
    );
    let input =
        moved.read().count() + buttons.read().count() + wheel.read().count() + keys.read().count();
    stats.input += u32::from(input > 0);
    stats.motion += u32::from(motion.read().count() > 0);
    stats.total += took;
    stats.max = stats.max.max(took);
    let elapsed = now - stats.window_start;
    if elapsed < stats.period {
        return;
    }
    let frames = stats.frames;
    let avg_ms = stats.total.as_secs_f32() * 1000.0 / frames as f32;
    info!(
        screen = ?screen.as_deref().map(State::get),
        frames,
        fps = format_args!("{:.1}", frames as f32 / elapsed.as_secs_f32()),
        busy = stats.busy,
        egui = stats.egui,
        input = stats.input,
        motion = stats.motion,
        avg_ms = format_args!("{avg_ms:.2}"),
        max_ms = format_args!("{:.2}", stats.max.as_secs_f32() * 1000.0),
        "frame stats"
    );
    stats.reset(now);
}

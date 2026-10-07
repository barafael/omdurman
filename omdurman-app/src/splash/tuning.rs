//! The title screen's tuning pane (a dev aid): hold **Shift** on the title
//! screen for a small egui pane that switches the menu column off and on and
//! sets the pan's, the slideshow's, the blur's and the ambient frame rate's
//! parameters live. Everything starts from [`super::params`]; each finished
//! change logs the whole set, ready to be copied back there. The pane exists
//! only while it shows: nothing of it takes focus or input otherwise.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::map::SlideTiming;
use super::params::*;
use crate::Screen;
use crate::activity::AmbientFps;

/// The title screen's live-tunable look: the menu column, and the pan.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub(super) struct SplashTuning {
    /// The menu column (and the fade that makes room for it) shows.
    pub(super) sidebar: bool,
    pub(super) pan_speed: f32,
    pub(super) title_map: MapLayout,
    pub(super) timing: SlideTiming,
    /// The map's blur, screen points (see [`MAP_BLUR_PX`]).
    pub(super) blur_px: f32,
}

impl Default for SplashTuning {
    fn default() -> Self {
        Self {
            sidebar: true,
            pan_speed: MAP_PAN_SPEED,
            title_map: TITLE_MAP,
            timing: SlideTiming::default(),
            blur_px: MAP_BLUR_PX,
        }
    }
}

/// The values as `params` spells them (and the ambient frame rate, `fps`).
fn describe(tuning: &SplashTuning, fps: f32) -> String {
    let map = &tuning.title_map;
    format!(
        "MAP_PAN_SPEED = {:.2}; TITLE_MAP: pan_amp = ({:.3}, {:.3}), rotate_deg = {:.1}, \
         rotate_swing_deg = {:.1}, tilt_deg = {:.1}, tilt_swing_deg = {:.1}; \
         MAP_SECS = {:.0}; MAP_CROSSFADE_SECS = {:.1}; MAP_BLUR_PX = {:.1}; \
         AMBIENT_FPS = {:.0}; sidebar = {}",
        tuning.pan_speed,
        map.pan_amp.0,
        map.pan_amp.1,
        map.rotate_deg,
        map.rotate_swing_deg,
        map.tilt_deg,
        map.tilt_swing_deg,
        tuning.timing.slide_secs,
        tuning.timing.crossfade_secs,
        tuning.blur_px,
        fps,
        tuning.sidebar,
    )
}

/// One slider of the pane.
#[derive(Clone, Copy, Debug)]
enum Knob {
    PanSpeed,
    PanMajor,
    PanMinor,
    Rotate,
    RotateSwing,
    Tilt,
    TiltSwing,
    SlideSecs,
    CrossfadeSecs,
    Blur,
}

impl Knob {
    const ALL: [Knob; 10] = [
        Knob::PanSpeed,
        Knob::PanMajor,
        Knob::PanMinor,
        Knob::Rotate,
        Knob::RotateSwing,
        Knob::Tilt,
        Knob::TiltSwing,
        Knob::SlideSecs,
        Knob::CrossfadeSecs,
        Knob::Blur,
    ];

    fn label(self) -> &'static str {
        match self {
            Knob::PanSpeed => "Pan speed",
            Knob::PanMajor => "Pan reach, major (box heights)",
            Knob::PanMinor => "Pan reach, minor (box heights)",
            Knob::Rotate => "Rotation (°)",
            Knob::RotateSwing => "Rotation swing (°)",
            Knob::Tilt => "Tilt (°)",
            Knob::TiltSwing => "Tilt swing (°)",
            Knob::SlideSecs => "Seconds per map",
            Knob::CrossfadeSecs => "Crossfade (s)",
            Knob::Blur => "Blur (points)",
        }
    }

    /// Range and decimals.
    fn range(self) -> (f32, f32, i32) {
        match self {
            Knob::PanSpeed => (0.0, 1.5, 2),
            Knob::PanMajor | Knob::PanMinor => (0.0, 0.2, 3),
            Knob::Rotate => (-30.0, 30.0, 1),
            Knob::RotateSwing => (0.0, 10.0, 1),
            Knob::Tilt => (0.0, 60.0, 1),
            Knob::TiltSwing => (0.0, 15.0, 1),
            Knob::SlideSecs => (4.0, 90.0, 0),
            Knob::CrossfadeSecs => (0.5, 20.0, 1),
            Knob::Blur => (0.0, 6.0, 1),
        }
    }

    fn get(self, tuning: &SplashTuning) -> f32 {
        let map = &tuning.title_map;
        match self {
            Knob::PanSpeed => tuning.pan_speed,
            Knob::PanMajor => map.pan_amp.0,
            Knob::PanMinor => map.pan_amp.1,
            Knob::Rotate => map.rotate_deg,
            Knob::RotateSwing => map.rotate_swing_deg,
            Knob::Tilt => map.tilt_deg,
            Knob::TiltSwing => map.tilt_swing_deg,
            Knob::SlideSecs => tuning.timing.slide_secs,
            Knob::CrossfadeSecs => tuning.timing.crossfade_secs,
            Knob::Blur => tuning.blur_px,
        }
    }

    fn set(self, tuning: &mut SplashTuning, value: f32) {
        let map = &mut tuning.title_map;
        match self {
            Knob::PanSpeed => tuning.pan_speed = value,
            Knob::PanMajor => map.pan_amp.0 = value,
            Knob::PanMinor => map.pan_amp.1 = value,
            Knob::Rotate => map.rotate_deg = value,
            Knob::RotateSwing => map.rotate_swing_deg = value,
            Knob::Tilt => map.tilt_deg = value,
            Knob::TiltSwing => map.tilt_swing_deg = value,
            Knob::SlideSecs => tuning.timing.slide_secs = value,
            Knob::CrossfadeSecs => tuning.timing.crossfade_secs = value,
            Knob::Blur => tuning.blur_px = value,
        }
    }
}

/// The pane's width, and its sliders' (logical pixels): wide, so a slider
/// moves in small steps.
const PANE_WIDTH: f32 = 520.0;
const SLIDER_WIDTH: f32 = 360.0;
/// Pixels of drag on a slider's number box that cross the slider's whole
/// range: dragging the number is the fine adjustment (the handle follows
/// the pointer, the number creeps).
const FINE_DRAG_PX: f64 = 2000.0;

/// The range of the frame-rate slider: from the unfocused pace up.
const FPS_RANGE: (f32, f32) = (crate::activity::AMBIENT_UNFOCUSED_FPS, 60.0);

/// The pane, top right, while Shift is held on the title screen. Edits go
/// straight to the resources (written only when a value changed); a finished
/// change -- a drag let go, a click, a typed value -- logs the whole set.
pub(super) fn tuning_pane_ui(
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
    screen: Option<Res<State<Screen>>>,
    mut tuning: ResMut<SplashTuning>,
    mut fps: ResMut<AmbientFps>,
) {
    let title = screen.is_some_and(|screen| **screen == Screen::Title);
    if !title || !keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let (mut edited, mut edited_fps) = (*tuning, fps.0);
    let finished = egui::Area::new(egui::Id::new("splash_tuning"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .show(ui, |ui| tuning_controls(ui, &mut edited, &mut edited_fps))
                .inner
        })
        .inner;
    if edited != *tuning {
        *tuning = edited;
    }
    if edited_fps != fps.0 {
        fps.0 = edited_fps;
    }
    if finished {
        info!("splash tuning: {}", describe(&edited, edited_fps));
    }
}

/// The pane's controls over `tuning` and the ambient `fps`; whether a change
/// was finished this frame (a drag let go, a click, a typed value).
fn tuning_controls(ui: &mut egui::Ui, tuning: &mut SplashTuning, fps: &mut f32) -> bool {
    ui.set_width(PANE_WIDTH);
    ui.spacing_mut().slider_width = SLIDER_WIDTH;
    ui.strong("Title screen");
    let mut finished = ui.checkbox(&mut tuning.sidebar, "Sidebar").changed();
    let mut slider =
        |ui: &mut egui::Ui, value: &mut f32, (min, max, decimals): (f32, f32, i32), label: &str| {
            ui.label(egui::RichText::new(label).small());
            let before = *value;
            // (`min_decimals`, not `fixed_decimals`: a fixed count rounds the
            // value itself the moment the slider shows -- a pan reach of 0.0625
            // became 0.062 -- so opening the pane changed the look.)
            // The number box beside the slider drags slowly, one decimal finer
            // than the slider shows.
            let response = ui.add(
                egui::Slider::new(value, min..=max)
                    .min_decimals(decimals.max(0) as usize)
                    .max_decimals(decimals.max(0) as usize + 1)
                    .drag_value_speed(f64::from(max - min) / FINE_DRAG_PX),
            );
            // (`changed` alone also fires for a value the slider merely clamped
            // or rounded on show; a finished edit is a user's, and moves it.)
            let edited = *value != before;
            finished |= response.drag_stopped() || (edited && !response.dragged());
        };
    for knob in Knob::ALL {
        let mut value = knob.get(tuning);
        slider(ui, &mut value, knob.range(), knob.label());
        knob.set(tuning, value);
    }
    slider(ui, fps, (FPS_RANGE.0, FPS_RANGE.1, 0), "Frames per second");
    finished
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Showing the pane changes nothing and reports nothing: only a user's
    /// edit is a change worth logging.
    #[test]
    fn an_untouched_pane_changes_nothing() {
        let (mut tuning, mut fps) = (SplashTuning::default(), crate::activity::AMBIENT_FPS);
        let mut reported = false;
        crate::ui::headless(3, |ctx, _| {
            egui::Area::new(egui::Id::new("pane")).show(ctx, |ui| {
                reported |= tuning_controls(ui, &mut tuning, &mut fps);
            });
        });
        assert_eq!(tuning, SplashTuning::default());
        assert_eq!(fps, crate::activity::AMBIENT_FPS);
        assert!(!reported);
    }

    // Every knob starts inside its slider's range, and sets exactly what it
    // reads. (The frame rate is not a knob: it starts from the environment.)
    #[test]
    fn the_knobs_start_in_range_and_round_trip() {
        assert!((FPS_RANGE.0..=FPS_RANGE.1).contains(&crate::activity::AMBIENT_FPS));
        let defaults = SplashTuning::default();
        for (i, knob) in Knob::ALL.into_iter().enumerate() {
            let (min, max, _) = knob.range();
            let value = knob.get(&defaults);
            assert!((min..=max).contains(&value), "{knob:?} {value}");
            let mut tuning = defaults;
            let other = (min + max) / 2.0 + 0.125;
            knob.set(&mut tuning, other);
            assert_eq!(knob.get(&tuning), other, "{knob:?}");
            for (j, untouched) in Knob::ALL.into_iter().enumerate() {
                if i != j {
                    assert_eq!(untouched.get(&tuning), untouched.get(&defaults));
                }
            }
        }
    }
}

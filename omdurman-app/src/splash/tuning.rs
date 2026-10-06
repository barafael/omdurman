//! The title screen's tuning pane (a dev aid): hold **Shift** on the title
//! screen for a small Feathers pane that switches the menu column off and on
//! and sets the pan's parameters live. Everything starts from
//! [`super::params`]; each finished change logs the whole set, ready to be
//! copied back there.

use bevy::feathers::{
    containers::{pane, pane_body, pane_header},
    controls::{FeathersCheckbox, FeathersSlider},
    display::{label, label_small},
    theme::ThemedText,
};
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui_widgets::{SliderPrecision, SliderValue, ValueChange};

use super::map::SlideTiming;
use super::params::*;
use crate::{AppMode, AppState};

/// The tuning pane sits over the title screen (`screen::SPLASH_Z`).
const TUNING_Z: i32 = 1_100;

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
    /// The frame rate while the map moves (see `activity::AMBIENT_FPS`).
    pub(super) fps: f32,
}

impl Default for SplashTuning {
    fn default() -> Self {
        Self {
            sidebar: true,
            pan_speed: MAP_PAN_SPEED,
            title_map: TITLE_MAP,
            timing: SlideTiming::default(),
            blur_px: MAP_BLUR_PX,
            fps: crate::activity::AmbientFps::default().0,
        }
    }
}

impl SplashTuning {
    /// The values as `params` spells them.
    fn describe(&self) -> String {
        let map = &self.title_map;
        format!(
            "MAP_PAN_SPEED = {:.2}; TITLE_MAP: pan_amp = ({:.3}, {:.3}), rotate_deg = {:.1}, \
             rotate_swing_deg = {:.1}, tilt_deg = {:.1}, tilt_swing_deg = {:.1}; \
             MAP_SECS = {:.0}; MAP_CROSSFADE_SECS = {:.1}; MAP_BLUR_PX = {:.1}; \
             AMBIENT_FPS = {:.0}; sidebar = {}",
            self.pan_speed,
            map.pan_amp.0,
            map.pan_amp.1,
            map.rotate_deg,
            map.rotate_swing_deg,
            map.tilt_deg,
            map.tilt_swing_deg,
            self.timing.slide_secs,
            self.timing.crossfade_secs,
            self.blur_px,
            self.fps,
            self.sidebar,
        )
    }
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
    Fps,
}

impl Knob {
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
            Knob::Fps => "Frames per second",
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
            Knob::Fps => (5.0, 60.0, 0),
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
            Knob::Fps => tuning.fps,
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
            Knob::Fps => tuning.fps = value,
        }
    }
}

/// Marks the pane.
#[derive(Component, Default, Clone)]
pub(super) struct TuningPane;

pub(super) fn spawn_tuning_pane(mut commands: Commands) {
    commands.spawn_scene(tuning_pane(SplashTuning::default()));
}

fn tuning_pane(tuning: SplashTuning) -> impl Scene {
    bsn! {
        pane()
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            width: px(260),
            display: Display::None,
        }
        TuningPane
        GlobalZIndex(TUNING_Z)
        TabGroup
        Children [
            (pane_header() Children [ label("Title screen") ]),
            (
                pane_body()
                Children [
                    (
                        @FeathersCheckbox {
                            @caption: bsn! { Text("Sidebar") ThemedText }
                        }
                        Checked
                        on(toggle_sidebar)
                    ),
                    knob(Knob::PanSpeed, tuning),
                    knob(Knob::PanMajor, tuning),
                    knob(Knob::PanMinor, tuning),
                    knob(Knob::Rotate, tuning),
                    knob(Knob::RotateSwing, tuning),
                    knob(Knob::Tilt, tuning),
                    knob(Knob::TiltSwing, tuning),
                    knob(Knob::SlideSecs, tuning),
                    knob(Knob::CrossfadeSecs, tuning),
                    knob(Knob::Blur, tuning),
                    knob(Knob::Fps, tuning),
                ]
            ),
        ]
    }
}

/// A captioned slider for `knob`, starting at its value in `tuning`.
fn knob(knob: Knob, tuning: SplashTuning) -> impl Scene {
    let (min, max, decimals) = knob.range();
    let value = knob.get(&tuning);
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
        }
        Children [
            label_small(knob.label()),
            (
                @FeathersSlider {
                    @min: min,
                    @max: max,
                    @value: value,
                }
                SliderPrecision(decimals)
                on(move |change: On<ValueChange<f32>>,
                         mut tuning: ResMut<SplashTuning>,
                         mut commands: Commands| {
                    commands
                        .entity(change.source)
                        .insert(SliderValue(change.value));
                    knob.set(&mut tuning, change.value);
                    if change.is_final {
                        info!("splash tuning: {}", tuning.describe());
                    }
                })
            ),
        ]
    }
}

fn toggle_sidebar(
    change: On<ValueChange<bool>>,
    mut tuning: ResMut<SplashTuning>,
    mut commands: Commands,
) {
    let mut checkbox = commands.entity(change.source);
    if change.value {
        checkbox.insert(Checked);
    } else {
        checkbox.remove::<Checked>();
    }
    tuning.sidebar = change.value;
    info!("splash tuning: {}", tuning.describe());
}

/// The tuned frame rate drives the app's ambient pacing.
pub(super) fn apply_tuned_fps(
    tuning: Res<SplashTuning>,
    mut fps: ResMut<crate::activity::AmbientFps>,
) {
    if tuning.is_changed() && fps.0 != tuning.fps {
        fps.0 = tuning.fps;
    }
}

/// The pane shows while Shift is held on the title screen.
pub(super) fn show_tuning_pane(
    keys: Res<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
    mode: Res<State<AppMode>>,
    mut pane: Query<&mut Node, With<TuningPane>>,
) {
    let title = *app_state.get() == AppState::Splash || *mode.get() == AppMode::Menu;
    let show = title && keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let display = if show { Display::Flex } else { Display::None };
    for mut node in &mut pane {
        if node.display != display {
            node.display = display;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOBS: [Knob; 11] = [
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
        Knob::Fps,
    ];

    // Every knob starts inside its slider's range, and sets exactly what it
    // reads.
    #[test]
    fn the_knobs_start_in_range_and_round_trip() {
        let defaults = SplashTuning::default();
        for (i, knob) in KNOBS.into_iter().enumerate() {
            let (min, max, _) = knob.range();
            let value = knob.get(&defaults);
            assert!((min..=max).contains(&value), "{knob:?} {value}");
            let mut tuning = defaults;
            let other = (min + max) / 2.0 + 0.125;
            knob.set(&mut tuning, other);
            assert_eq!(knob.get(&tuning), other, "{knob:?}");
            for (j, untouched) in KNOBS.into_iter().enumerate() {
                if i != j {
                    assert_eq!(untouched.get(&tuning), untouched.get(&defaults));
                }
            }
        }
    }
}

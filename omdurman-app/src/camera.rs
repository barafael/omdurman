use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    input::{
        mouse::{MouseScrollUnit, MouseWheel},
        touch::Touches,
    },
    prelude::*,
    render::view::ColorGrading,
};
use bevy_egui::{EguiContexts, egui};
use std::f32::consts::PI;

#[derive(Component)]
pub struct RtsCamera;

#[derive(Component)]
pub struct RtsCameraState {
    pub focus: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub smooth_focus: Vec3,
    pub smooth_distance: f32,
    pub smooth_yaw: f32,
    pub smooth_pitch: f32,
}

impl Default for RtsCameraState {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            distance: 1500.0,
            yaw: 0.0,
            pitch: PI / 2.0 - 0.02,
            smooth_focus: Vec3::ZERO,
            smooth_distance: 1500.0,
            smooth_yaw: 0.0,
            smooth_pitch: PI / 2.0 - 0.02,
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraDragState {
    pub active: bool,
    pub last_cursor: Vec2,
}

#[derive(Resource)]
pub struct CameraSettings {
    pub pan_speed: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub smoothing: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            pan_speed: 600.0,
            min_distance: 100.0,
            max_distance: 8000.0,
            min_pitch: PI / 6.0,
            max_pitch: PI / 2.0 - 0.02,
            smoothing: 6.0,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings::default())
            .insert_resource(CameraDragState::default())
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (camera_control.run_if(crate::camera_enabled), night_shading),
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
    ));
}

/// How dark and desaturated the board looks at full night, and how fast it
/// eases there. The scenario plays across day and night turns (§8.1); we shade
/// the *rendered board* (not the egui UI, which renders in its own pass) to
/// make the current time of day legible at a glance. Purely presentational and
/// derived from the replicated `state.day_night`, so it needs no networking and
/// stays identical on every peer.
const NIGHT_EXPOSURE: f32 = -1.3; // EV stops darker at full night
const NIGHT_SATURATION: f32 = 0.35; // post_saturation at full night (1.0 = unchanged)
const NIGHT_FADE_PER_SEC: f32 = 0.67; // ~1.5s day<->night crossfade (§night tint)
// Push colour toward the printed NIGHT-cell green at full night: cooler
// temperature + green tint. Bevy's convention: negative temperature = cooler,
// negative tint = toward green. Scaled by the eased `night` factor so day is
// untouched. UI chrome is unaffected (ColorGrading applies to the camera view
// only, not egui).
const NIGHT_TEMPERATURE: f32 = -0.25;
const NIGHT_TINT: f32 = -0.30;

/// Ease the camera's colour grading toward the day/night target each frame: a
/// `night` factor of 0 is full daylight (grading untouched), 1 is full night
/// (darker + desaturated). Interpolated so the transition fades rather than
/// snaps when a turn crosses dawn/dusk.
fn night_shading(
    time: Res<Time>,
    game_state: Option<Res<crate::GameStateResource>>,
    mut grading: Query<&mut ColorGrading, With<RtsCamera>>,
    mut night: Local<f32>,
) {
    let Ok(mut grading) = grading.single_mut() else {
        return;
    };
    let target = match game_state.as_deref().map(|gs| gs.0.day_night) {
        Some(omdurman_types::DayNight::Night) => 1.0,
        _ => 0.0, // day, or no game state yet
    };
    // Dev: OMDURMAN_FORCE_NIGHT forces the night look for verification.
    let target = if std::env::var("OMDURMAN_FORCE_NIGHT").is_ok() {
        1.0
    } else {
        target
    };
    // Frame-rate-independent ease toward the target, clamped so a long frame
    // can't overshoot past the endpoint.
    let step = (NIGHT_FADE_PER_SEC * time.delta_secs()).min(1.0);
    *night += (target - *night) * step;

    let g = &mut grading.global;
    g.exposure = NIGHT_EXPOSURE * *night;
    g.post_saturation = 1.0 + (NIGHT_SATURATION - 1.0) * *night;
    // Tint toward the night-cell green as night deepens.
    g.temperature = NIGHT_TEMPERATURE * *night;
    g.tint = NIGHT_TINT * *night;
}

fn camera_basis(yaw: f32) -> (Vec3, Vec3) {
    let fwd = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    let right = Vec3::new(fwd.z, 0.0, -fwd.x);
    (fwd, right)
}

fn pan_camera(state: &mut RtsCameraState, pan: Vec2, scale: f32) {
    let (fwd, right) = camera_basis(state.yaw);
    state.focus += fwd * pan.y * scale + right * pan.x * scale;
}

fn camera_drag_pan(
    state: &mut RtsCameraState,
    drag_state: &mut CameraDragState,
    buttons: &ButtonInput<MouseButton>,
    cursor_pos: Option<Vec2>,
    ctx: &egui::Context,
) {
    if !ctx.egui_wants_pointer_input() {
        if buttons.just_pressed(MouseButton::Right) {
            drag_state.active = true;
            if let Some(pos) = cursor_pos {
                drag_state.last_cursor = pos;
            }
        } else if buttons.just_released(MouseButton::Right) {
            drag_state.active = false;
        }
    } else {
        drag_state.active = false;
    }

    if drag_state.active
        && let (Some(pos), false) = (cursor_pos, ctx.egui_wants_pointer_input())
    {
        let delta = Vec2::new(
            pos.x - drag_state.last_cursor.x,
            pos.y - drag_state.last_cursor.y,
        );
        if delta.length_squared() > 0.0 {
            pan_camera(state, delta, (state.distance / 500.0) * 0.6);
        }
        drag_state.last_cursor = pos;
    }
}

fn camera_keyboard_pan(
    state: &mut RtsCameraState,
    settings: &CameraSettings,
    keys: &ButtonInput<KeyCode>,
    ctx: &egui::Context,
    dt: f32,
) {
    let ctrl = crate::util::ctrl_held(keys);
    let mut pan = Vec2::ZERO;
    if !ctx.egui_wants_keyboard_input() && !ctrl {
        if keys.pressed(KeyCode::ArrowUp) {
            pan.y += 1.0;
        }
        if keys.pressed(KeyCode::ArrowDown) {
            pan.y -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            pan.x -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            pan.x += 1.0;
        }
    }
    if pan != Vec2::ZERO {
        pan = pan.normalize() * settings.pan_speed * dt * (state.distance / 500.0).max(0.3);
        pan_camera(state, pan, 1.0);
    }
}

fn camera_scroll_zoom(
    state: &mut RtsCameraState,
    settings: &CameraSettings,
    keys: &ButtonInput<KeyCode>,
    ctx: &egui::Context,
    scroll_events: &mut bevy::ecs::message::MessageReader<MouseWheel>,
) {
    let mut zoom_ticks: f32 = 0.0;
    if !ctx.egui_wants_pointer_input() {
        for ev in scroll_events.read() {
            let notch_scale = match ev.unit {
                MouseScrollUnit::Pixel => 0.01,
                MouseScrollUnit::Line => 1.0,
            };
            zoom_ticks += ev.y * notch_scale;
        }
    }
    if zoom_ticks != 0.0 {
        if crate::util::ctrl_held(keys) {
            state.pitch =
                (state.pitch + zoom_ticks * 0.1).clamp(settings.min_pitch, settings.max_pitch);
        } else {
            let factor = 1.0 - zoom_ticks.clamp(-5.0, 5.0) * 0.12;
            state.distance =
                (state.distance * factor).clamp(settings.min_distance, settings.max_distance);
        }
    }
}

fn camera_page_tilt(
    state: &mut RtsCameraState,
    settings: &CameraSettings,
    keys: &ButtonInput<KeyCode>,
    ctx: &egui::Context,
    dt: f32,
) {
    let ctrl = crate::util::ctrl_held(keys);
    let pitch_step = dt * 0.8;
    if !ctx.egui_wants_keyboard_input() && !ctrl {
        if keys.pressed(KeyCode::PageUp) {
            state.pitch = (state.pitch + pitch_step).min(settings.max_pitch);
        }
        if keys.pressed(KeyCode::PageDown) {
            state.pitch = (state.pitch - pitch_step).max(settings.min_pitch);
        }
    }
}

fn camera_touch_gestures(
    state: &mut RtsCameraState,
    settings: &CameraSettings,
    ctx: &egui::Context,
    touches: &Touches,
) {
    if ctx.egui_wants_pointer_input() {
        return;
    }
    let mut touches_iter = touches.iter();
    if let (Some(t0), Some(t1)) = (touches_iter.next(), touches_iter.next()) {
        let prev_dist = t0.previous_position().distance(t1.previous_position());
        let cur_dist = t0.position().distance(t1.position());
        let pinch_delta = cur_dist - prev_dist;
        if pinch_delta != 0.0 {
            let factor = 1.0 - pinch_delta.clamp(-30.0, 30.0) * 0.02;
            state.distance =
                (state.distance * factor).clamp(settings.min_distance, settings.max_distance);
        }

        let prev_mid_y = (t0.previous_position().y + t1.previous_position().y) * 0.5;
        let cur_mid_y = (t0.position().y + t1.position().y) * 0.5;
        let pitch_delta = cur_mid_y - prev_mid_y;
        if pitch_delta != 0.0 {
            state.pitch =
                (state.pitch - pitch_delta * 0.02).clamp(settings.min_pitch, settings.max_pitch);
        }
    }
}

fn apply_camera_transform(
    state: &mut RtsCameraState,
    settings: &CameraSettings,
    transform: &mut Transform,
    dt: f32,
) {
    let t = (settings.smoothing * dt).min(1.0);
    state.smooth_focus = state.smooth_focus.lerp(state.focus, t);
    state.smooth_distance = state.smooth_distance.lerp(state.distance, t);
    state.smooth_yaw = state.smooth_yaw.lerp(state.yaw, t);
    state.smooth_pitch = state.smooth_pitch.lerp(state.pitch, t);

    let hdist = state.smooth_distance * state.smooth_pitch.cos();
    let vert = state.smooth_distance * state.smooth_pitch.sin();
    let offset = Vec3::new(
        hdist * state.smooth_yaw.sin(),
        vert,
        hdist * state.smooth_yaw.cos(),
    );
    let eye = state.smooth_focus + offset;
    *transform = Transform::from_translation(eye).looking_at(state.smooth_focus, Vec3::Y);
}

/// Bundles the four input sources (keyboard, mouse buttons, scroll wheel,
/// touch) so [`camera_control`] stays under clippy's argument limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct CameraInput<'w, 's> {
    pub keys: Res<'w, ButtonInput<KeyCode>>,
    pub buttons: Res<'w, ButtonInput<MouseButton>>,
    pub scroll_events: bevy::ecs::message::MessageReader<'w, 's, MouseWheel>,
    pub touches: Res<'w, Touches>,
}

pub fn camera_control(
    time: Res<Time>,
    settings: Res<CameraSettings>,
    input: CameraInput,
    mut drag_state: ResMut<CameraDragState>,
    windows: Query<&Window>,
    mut cam_q: Query<(&mut RtsCameraState, &mut Transform), With<RtsCamera>>,
    mut contexts: EguiContexts,
) {
    let CameraInput {
        keys,
        buttons,
        mut scroll_events,
        touches,
    } = input;
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((mut state, mut transform)) = cam_q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    let cursor_pos = windows.single().ok().and_then(|w| w.cursor_position());
    camera_drag_pan(&mut state, &mut drag_state, &buttons, cursor_pos, ctx);
    camera_keyboard_pan(&mut state, &settings, &keys, ctx, dt);
    camera_scroll_zoom(&mut state, &settings, &keys, ctx, &mut scroll_events);
    camera_page_tilt(&mut state, &settings, &keys, ctx, dt);
    camera_touch_gestures(&mut state, &settings, ctx, &touches);
    apply_camera_transform(&mut state, &settings, &mut transform, dt);
}

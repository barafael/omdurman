//! The RTS camera shared by the game and the editor: right-drag pan, arrows,
//! scroll zoom, Ctrl+scroll / PgUp/PgDn tilt, touch gestures. Previously two
//! ~300-line copies that had begun to drift (the game's grew night shading
//! and picking markers, the editor's none).
//!
//! No `Plugin` is provided on purpose: each binary wires the systems itself
//! with its own gating (the game adds a run-condition and night shading, the
//! editor registers them bare) and can swap in its own `spawn_camera` (the
//! game's adds a picking marker).

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

use crate::input::ctrl_held;
use crate::panels::egui_wants_pointer_input;
use omdurman_hexmap::MapDims;

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

/// Screen margins (logical px) covered by UI chrome. A binary with docked
/// panels publishes them so fitting centres the board in the free area
/// instead of under a sidebar; absent, the whole window counts as free.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct CameraViewInsets {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

/// Pending "frame the whole board" request. Set on Home and whenever the
/// board changes ([`MapDims`] changes); held for a few frames so the fit
/// follows the chrome insets as the panels of a freshly entered mode settle.
#[derive(Resource, Default)]
pub struct CameraFit {
    pending_frames: u8,
}

impl CameraFit {
    /// Frames a request keeps re-fitting (chrome insets lag the egui pass).
    const FRAMES: u8 = 4;

    pub fn request(&mut self) {
        self.pending_frames = Self::FRAMES;
    }
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

/// Spawn a plain RTS camera. Binaries that need extra components on the
/// camera (e.g. the game's mesh-picking marker) provide their own spawn
/// system instead.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        RtsCamera,
        RtsCameraState::default(),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        Tonemapping::None,
        ColorGrading::default(),
    ));
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
    if !egui_wants_pointer_input(ctx) {
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
        && let (Some(pos), false) = (cursor_pos, egui_wants_pointer_input(ctx))
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
    let ctrl = ctrl_held(keys);
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

/// Wheel zoom. `anchor` is the ground point under the pointer: the focus is
/// scaled about it by the same factor as the distance, so the point you
/// zoom at stays under the pointer (it used to slide away, the zoom centring
/// on the screen middle).
fn camera_scroll_zoom(
    state: &mut RtsCameraState,
    settings: &CameraSettings,
    keys: &ButtonInput<KeyCode>,
    ctx: &egui::Context,
    scroll_events: &mut bevy::ecs::message::MessageReader<MouseWheel>,
    anchor: Option<Vec3>,
) {
    // Always drain the reader: wheel ticks that land while egui owns the
    // pointer belong to the UI and are discarded, not replayed as a zoom the
    // next time the pointer is over the board.
    let over_ui = egui_wants_pointer_input(ctx);
    let mut zoom_ticks: f32 = 0.0;
    for ev in scroll_events.read() {
        let notch_scale = match ev.unit {
            MouseScrollUnit::Pixel => 0.01,
            MouseScrollUnit::Line => 1.0,
        };
        zoom_ticks += ev.y * notch_scale;
    }
    if over_ui {
        zoom_ticks = 0.0;
    }
    if zoom_ticks != 0.0 {
        if ctrl_held(keys) {
            state.pitch =
                (state.pitch + zoom_ticks * 0.1).clamp(settings.min_pitch, settings.max_pitch);
        } else {
            let factor = 1.0 - zoom_ticks.clamp(-5.0, 5.0) * 0.12;
            let before = state.distance;
            state.distance =
                (state.distance * factor).clamp(settings.min_distance, settings.max_distance);
            if let Some(anchor) = anchor {
                let applied = state.distance / before;
                let y = state.focus.y;
                state.focus = anchor + (state.focus - anchor) * applied;
                state.focus.y = y;
            }
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
    let ctrl = ctrl_held(keys);
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
    if egui_wants_pointer_input(ctx) {
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

/// Point the camera straight down at the board, zoomed so the whole board
/// fits the part of the window not covered by `insets`, and centred there.
fn fit_board(
    state: &mut RtsCameraState,
    settings: &CameraSettings,
    dims: &MapDims,
    window: Vec2,
    insets: CameraViewInsets,
    fov_y: f32,
) {
    let free_min = Vec2::new(insets.left, insets.top);
    let free_max = window - Vec2::new(insets.right, insets.bottom);
    let free = (free_max - free_min).max(Vec2::splat(64.0));
    // World units per screen pixel so both board axes fit, with a margin
    // wide enough for the playable half-hexes that overhang the scan's edge
    // (e.g. the Fall-of-Khartoum entry edge, §9.342) to stay clear of chrome.
    let world_per_px = (dims.img_w / free.x).max(dims.img_h / free.y) * 1.12;
    // Screen offset of the free area's centre from the window centre; the
    // focus (drawn at the window centre) shifts the opposite way so the
    // board centre (the world origin) lands in the free area's centre.
    // Screen right is world +x and screen down is world +z at yaw 0.
    let offset = (free_min + free_max) * 0.5 - window * 0.5;
    state.focus = Vec3::new(-offset.x * world_per_px, 0.0, -offset.y * world_per_px);
    state.distance = (world_per_px * window.y / (2.0 * (fov_y * 0.5).tan()))
        .clamp(settings.min_distance, settings.max_distance);
    state.yaw = 0.0;
    state.pitch = settings.max_pitch;
}

/// Keep the camera focus over the board, so panning can never lose it
/// off-screen.
fn clamp_focus_to_board(state: &mut RtsCameraState, dims: &MapDims) {
    let half = Vec2::new(dims.img_w, dims.img_h) * 0.5;
    state.focus.x = state.focus.x.clamp(-half.x, half.x);
    state.focus.z = state.focus.z.clamp(-half.y, half.y);
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
pub struct CameraInput<'w, 's> {
    pub keys: Res<'w, ButtonInput<KeyCode>>,
    pub buttons: Res<'w, ButtonInput<MouseButton>>,
    pub scroll_events: bevy::ecs::message::MessageReader<'w, 's, MouseWheel>,
    pub touches: Res<'w, Touches>,
}

/// The board extent, the chrome insets, the pending fit request and the
/// frame clock.
#[derive(bevy::ecs::system::SystemParam)]
pub struct CameraFraming<'w> {
    pub time: Res<'w, Time>,
    pub dims: Option<Res<'w, MapDims>>,
    pub insets: Option<Res<'w, CameraViewInsets>>,
    pub fit: ResMut<'w, CameraFit>,
}

pub fn camera_control(
    settings: Res<CameraSettings>,
    input: CameraInput,
    mut framing: CameraFraming,
    mut drag_state: ResMut<CameraDragState>,
    windows: Query<&Window>,
    mut cam_q: Query<
        (
            &mut RtsCameraState,
            &mut Transform,
            &Projection,
            &Camera,
            &GlobalTransform,
        ),
        With<RtsCamera>,
    >,
    mut contexts: EguiContexts,
) {
    let CameraInput {
        keys,
        buttons,
        mut scroll_events,
        touches,
    } = input;
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((mut state, mut transform, projection, camera, camera_transform)) = cam_q.single_mut()
    else {
        return;
    };
    let dt = framing.time.delta_secs();
    let window = windows.single().ok();
    let cursor_pos = window.and_then(|w| w.cursor_position());
    camera_drag_pan(&mut state, &mut drag_state, &buttons, cursor_pos, ctx);
    camera_keyboard_pan(&mut state, &settings, &keys, ctx, dt);
    // The board point under the pointer, on the focus plane (as last drawn).
    let zoom_anchor = cursor_pos
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .and_then(|ray| {
            ray.intersect_plane(
                Vec3::new(0.0, state.focus.y, 0.0),
                InfinitePlane3d::new(Vec3::Y),
            )
            .map(|distance| ray.get_point(distance))
        });
    camera_scroll_zoom(
        &mut state,
        &settings,
        &keys,
        ctx,
        &mut scroll_events,
        zoom_anchor,
    );
    camera_page_tilt(&mut state, &settings, &keys, ctx, dt);
    camera_touch_gestures(&mut state, &settings, ctx, &touches);
    if let Some(dims) = framing.dims.as_deref() {
        if framing.dims.as_ref().is_some_and(|d| d.is_changed())
            || (keys.just_pressed(KeyCode::Home) && !ctx.egui_wants_keyboard_input())
        {
            framing.fit.request();
        }
        if framing.fit.pending_frames > 0
            && let Some(window) = window
        {
            framing.fit.pending_frames -= 1;
            let fov_y = match projection {
                Projection::Perspective(p) => p.fov,
                _ => PI / 4.0,
            };
            fit_board(
                &mut state,
                &settings,
                dims,
                Vec2::new(window.width(), window.height()),
                framing.insets.as_deref().copied().unwrap_or_default(),
                fov_y,
            );
        }
        clamp_focus_to_board(&mut state, dims);
    }
    apply_camera_transform(&mut state, &settings, &mut transform, dt);
}

//! The RTS camera: right-drag pan, arrows, scroll zoom, Ctrl+scroll /
//! PgUp/PgDn tilt, touch gestures.
//!
//! No `Plugin` is provided: the app wires the systems itself with its own
//! gating (a run-condition and night shading) and spawns the camera with
//! its picking marker.

use bevy::{
    input::{
        mouse::{MouseScrollUnit, MouseWheel},
        touch::Touches,
    },
    prelude::*,
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

/// How far (radians) the fitted view leans back from straight down: just
/// enough that the board reads as a table seen from a chair, not a scan --
/// about 7 degrees, well inside the fit's framing margin.
pub const FIT_TILT: f32 = 0.12;

/// The pitch of the fitted view (see [`FIT_TILT`]).
fn fit_pitch(settings: &CameraSettings) -> f32 {
    (settings.max_pitch - FIT_TILT).max(settings.min_pitch)
}

impl Default for RtsCameraState {
    fn default() -> Self {
        let pitch = fit_pitch(&CameraSettings::default());
        Self {
            focus: Vec3::ZERO,
            distance: 1500.0,
            yaw: 0.0,
            pitch,
            smooth_focus: Vec3::ZERO,
            smooth_distance: 1500.0,
            smooth_yaw: 0.0,
            smooth_pitch: pitch,
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

/// Whether the camera still needs frames: easing toward its target, being
/// dragged or keyed, or framing the board. Published by [`camera_control`]
/// (when the binary inserts it) so a reactive app keeps redrawing while the
/// view moves and can sleep once it has settled.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CameraSettling(pub bool);

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

/// Point the camera down at the board, very slightly tilted ([`FIT_TILT`]),
/// zoomed so the whole board fits the part of the window not covered by
/// `insets`, and centred there. (The framing is computed for a straight-down
/// view; the slight tilt stays inside its margin.)
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
    state.pitch = fit_pitch(settings);
}

/// Keep the camera focus over the board, so panning can never lose it
/// off-screen.
fn clamp_focus_to_board(state: &mut RtsCameraState, dims: &MapDims) {
    let half = Vec2::new(dims.img_w, dims.img_h) * 0.5;
    state.focus.x = state.focus.x.clamp(-half.x, half.x);
    state.focus.z = state.focus.z.clamp(-half.y, half.y);
}

/// Ease one smoothed value toward its target, snapping once within `eps` --
/// or once a step no longer changes it (far out, an f32 step can round to
/// nothing short of `eps`, and the camera would never settle). Returns
/// whether it is still on the way.
fn ease<T>(smooth: &mut T, target: T, t: f32, eps: f32, gap: impl Fn(T, T) -> f32) -> bool
where
    T: Copy + PartialEq + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
    T: std::ops::Mul<f32, Output = T>,
{
    if *smooth == target {
        return false;
    }
    if gap(*smooth, target) <= eps {
        *smooth = target;
        return false;
    }
    let next = *smooth + (target - *smooth) * t;
    // (A frame without time passing, `t == 0`, steps by nothing too.)
    if t > 0.0 && next == *smooth {
        *smooth = target;
        return false;
    }
    *smooth = next;
    true
}

/// Ease the smoothed view toward the target view and place the camera. The
/// smoothed values snap onto their targets once close, and the `Transform` is
/// written only when it actually changes -- a resting camera costs nothing
/// downstream (no re-extraction, no changed transform). Returns whether the
/// view is still easing.
fn apply_camera_transform(
    state: &mut RtsCameraState,
    settings: &CameraSettings,
    transform: &mut Mut<Transform>,
    dt: f32,
) -> bool {
    let t = (settings.smoothing * dt).min(1.0);
    let target_focus = state.focus;
    let mut easing = ease(&mut state.smooth_focus, target_focus, t, 0.01, |a, b| {
        a.distance(b)
    });
    let target_distance = state.distance;
    easing |= ease(
        &mut state.smooth_distance,
        target_distance,
        t,
        0.01,
        |a, b| (a - b).abs(),
    );
    let target_yaw = state.yaw;
    easing |= ease(&mut state.smooth_yaw, target_yaw, t, 1e-5, |a, b| {
        (a - b).abs()
    });
    let target_pitch = state.pitch;
    easing |= ease(&mut state.smooth_pitch, target_pitch, t, 1e-5, |a, b| {
        (a - b).abs()
    });

    let hdist = state.smooth_distance * state.smooth_pitch.cos();
    let vert = state.smooth_distance * state.smooth_pitch.sin();
    let offset = Vec3::new(
        hdist * state.smooth_yaw.sin(),
        vert,
        hdist * state.smooth_yaw.cos(),
    );
    let eye = state.smooth_focus + offset;
    transform.set_if_neq(Transform::from_translation(eye).looking_at(state.smooth_focus, Vec3::Y));
    easing
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
    /// Published "still moving" flag, if the binary wants it.
    pub settling: Option<ResMut<'w, CameraSettling>>,
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
    let easing = apply_camera_transform(&mut state, &settings, &mut transform, dt);
    if let Some(settling) = framing.settling.as_mut() {
        let held = drag_state.active
            || [
                KeyCode::ArrowUp,
                KeyCode::ArrowDown,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
                KeyCode::PageUp,
                KeyCode::PageDown,
            ]
            .iter()
            .any(|k| keys.pressed(*k));
        settling.set_if_neq(CameraSettling(
            easing || held || framing.fit.pending_frames > 0,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Far from the origin a small fraction of a gap just above `eps` rounds
    /// to no step at all: the ease snaps onto the target instead of asking
    /// for frames forever without moving.
    #[test]
    fn an_ease_that_cannot_step_settles() {
        let target = 6000.02_f32;
        let mut smooth = 6000.0_f32;
        assert!((target - smooth).abs() > 0.01, "the gap is above eps");
        let gap = |a: f32, b: f32| (a - b).abs();
        let still_easing = ease(&mut smooth, target, 0.01, 0.01, gap);
        assert!(!still_easing);
        assert_eq!(smooth, target);
    }

    #[test]
    fn a_frame_without_time_does_not_snap() {
        let mut smooth = 0.0_f32;
        assert!(ease(&mut smooth, 10.0, 0.0, 0.01, |a: f32, b: f32| (a - b).abs()));
        assert_eq!(smooth, 0.0);
    }

    #[test]
    fn an_ease_that_can_step_steps() {
        let mut smooth = 0.0_f32;
        assert!(ease(&mut smooth, 10.0, 0.5, 0.01, |a: f32, b: f32| (a - b).abs()));
        assert_eq!(smooth, 5.0);
    }
}

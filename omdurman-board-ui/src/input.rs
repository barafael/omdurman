//! Key/raycast helpers shared by the game and the editor. Byte-identical
//! copies used to live in both `util.rs` files.

use bevy::prelude::*;

use crate::camera::RtsCamera;

/// Whether Ctrl is held (either side). The single place these key codes are
/// OR'd together, so input handlers don't each re-derive it.
pub fn ctrl_held(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
}

/// Whether Shift is held (either side).
pub fn shift_held(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

pub fn raycast_ground(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
) -> Option<Vec3> {
    let Ok(window) = windows.single() else {
        return None;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return None;
    };
    let cursor_pos = window.cursor_position()?;
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else {
        return None;
    };
    let dir = ray.direction.as_vec3();
    if dir.y.abs() < 1e-6 {
        return None;
    }
    let t = -ray.origin.y / dir.y;
    if t < 0.0 {
        return None;
    }
    Some(ray.origin + dir * t)
}

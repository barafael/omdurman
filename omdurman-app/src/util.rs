use bevy::prelude::*;
use omdurman_hex::{HexLayout, SQRT_3, cube_round};
use omdurman_types::{HexCoord, OverlayParams};

use crate::RtsCamera;

pub fn adjusted_origin(layout: &HexLayout, offset_x: f32, offset_y: f32) -> Vec2 {
    Vec2::new(layout.origin.x + offset_x, layout.origin.y + offset_y)
}

pub fn hex_world_pos(coord: HexCoord, origin: Vec2, overlay: &OverlayParams) -> Vec3 {
    Vec3::new(
        origin.x + overlay.hex_size * SQRT_3 * (coord.q as f32 + (coord.r as f32 + overlay.phase()) * overlay.stagger),
        0.0,
        origin.y + overlay.hex_size * 1.5 * coord.r as f32,
    )
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

pub fn hit_to_hex(hit: Vec3, origin: Vec2, overlay: &OverlayParams) -> HexCoord {
    let dx = hit.x - origin.x;
    let dz = hit.z - origin.y;
    let fq = dx / (overlay.hex_size * SQRT_3) - (dz / (overlay.hex_size * 1.5) + overlay.phase()) * overlay.stagger;
    let fr = dz * 2.0 / (3.0 * overlay.hex_size);
    cube_round(fq, fr)
}

use bevy::prelude::*;
use omdurman_hex::{HexLayout, SQRT_3, cube_round};
use omdurman_types::{HexCoord, Orientation, OverlayParams};

use crate::RtsCamera;

pub fn adjusted_origin(layout: &HexLayout, offset_x: f32, offset_y: f32) -> Vec2 {
    Vec2::new(layout.origin.x + offset_x, layout.origin.y + offset_y)
}

/// Convert an axial hex coordinate to a 3D world position using overlay params.
///
/// Applies the offset-coordinate stagger on top of the calibrated layout:
/// the q-axis is shifted by `stagger` per row (pointy-top) or the r-axis
/// by `stagger` per column (flat-top), with a phase offset for parity.
///
/// Source: https://www.redblobgames.com/grids/hexagons/#hex-to-pixel
pub fn hex_world_pos(coord: HexCoord, origin: Vec2, overlay: &OverlayParams) -> Vec3 {
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
    let (q, r) = (coord.q as f32, coord.r as f32);
    let (x, z) = match overlay.orientation {
        Orientation::Pointy => (
            origin.x + overlay.hex_size * SQRT_3 * (q + (r + phase) * stagger),
            origin.y + overlay.hex_size * 1.5 * r,
        ),
        Orientation::Flat => (
            origin.x + overlay.hex_size * 1.5 * q,
            origin.y + overlay.hex_size * SQRT_3 * (r + (q + phase) * stagger),
        ),
    };
    Vec3::new(x, 0.0, z)
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

/// Convert a world-space hit point to the nearest axial hex coordinate.
///
/// Inverse of `hex_world_pos`: subtracts the origin, applies the inverse
/// of the hex-to-pixel matrix, then rounds via `cube_round`.
///
/// Source: https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
pub fn hit_to_hex(hit: Vec3, origin: Vec2, overlay: &OverlayParams) -> HexCoord {
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
    let dx = hit.x - origin.x;
    let dz = hit.z - origin.y;
    let (fq, fr) = match overlay.orientation {
        Orientation::Pointy => (
            dx / (overlay.hex_size * SQRT_3) - (dz / (overlay.hex_size * 1.5) + phase) * stagger,
            dz * 2.0 / (3.0 * overlay.hex_size),
        ),
        Orientation::Flat => (
            dx * 2.0 / (3.0 * overlay.hex_size),
            dz / (overlay.hex_size * SQRT_3) - (dx / (overlay.hex_size * 1.5) + phase) * stagger,
        ),
    };
    cube_round(fq, fr)
}

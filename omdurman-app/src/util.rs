use bevy::prelude::*;
use omdurman_hex::{HexLayout, SQRT_3, cube_round};
use omdurman_types::{HexCoord, Orientation, OverlayParams};

use crate::camera::RtsCamera;

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
    // Lattice position relative to the origin (before fine rotation).
    let (lx, lz) = match overlay.orientation {
        Orientation::Pointy => (
            overlay.hex_size * SQRT_3 * (q + (r + phase) * stagger),
            overlay.hex_size * 1.5 * r,
        ),
        Orientation::Flat => (
            overlay.hex_size * 1.5 * q,
            overlay.hex_size * SQRT_3 * (r + (q + phase) * stagger),
        ),
    };
    // Apply the fine rotation about the origin, then translate back.
    let (rx, rz) = rotate_xz(lx, lz, overlay.rotation_deg.to_radians());
    Vec3::new(origin.x + rx, 0.0, origin.y + rz)
}

/// Rotate a 2D point `(x, z)` about the origin by `angle` radians, in the XZ
/// ground plane. Used to register the hex lattice against a slightly-skewed map.
fn rotate_xz(x: f32, z: f32, angle: f32) -> (f32, f32) {
    let (s, c) = angle.sin_cos();
    (x * c - z * s, x * s + z * c)
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
    // Undo the fine rotation before inverting the hex-to-pixel matrix.
    let (dx, dz) = rotate_xz(
        hit.x - origin.x,
        hit.z - origin.y,
        -overlay.rotation_deg.to_radians(),
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::{GridShape, OffsetVariant};

    fn overlay(orientation: Orientation, rotation_deg: f32) -> OverlayParams {
        OverlayParams {
            hex_size: 40.0,
            offset_variant: if orientation == Orientation::Pointy {
                OffsetVariant::OddR
            } else {
                OffsetVariant::OddQ
            },
            orientation,
            shape: GridShape::Rectangle,
            rotation_deg,
            ..OverlayParams::default()
        }
    }

    /// A hex's world position must map back to the same hex under any rotation
    /// in range — i.e. `hit_to_hex` exactly inverts `hex_world_pos`, including
    /// the fine rotation term.
    #[test]
    fn world_pos_round_trips_under_rotation() {
        let origin = Vec2::new(123.0, -45.0);
        for orientation in [Orientation::Pointy, Orientation::Flat] {
            for &deg in &[-4.0, -1.5, 0.0, 0.75, 4.0] {
                let ov = overlay(orientation, deg);
                for q in -5..=5 {
                    for r in -5..=5 {
                        let coord = HexCoord::new(q, r);
                        let world = hex_world_pos(coord, origin, &ov);
                        let back = hit_to_hex(world, origin, &ov);
                        assert_eq!(
                            back, coord,
                            "{orientation:?} rot {deg}°: ({q},{r}) round-trip → ({},{})",
                            back.q, back.r
                        );
                    }
                }
            }
        }
    }

    /// Zero rotation must be identical to the unrotated mapping (no drift from
    /// the added rotate-by-0 path).
    #[test]
    fn zero_rotation_is_identity() {
        let origin = Vec2::new(10.0, 20.0);
        let ov = overlay(Orientation::Pointy, 0.0);
        let p = hex_world_pos(HexCoord::new(3, -2), origin, &ov);
        // Pointy: x = origin.x + size·√3·(q + (r+phase)·stagger); phase(OddR)=1,
        // stagger=-0.5 → (3 + (-2+1)·-0.5) = 3.5; z = origin.y + size·1.5·r.
        assert!((p.x - (10.0 + 40.0 * SQRT_3 * 3.5)).abs() < 1e-3);
        assert!((p.z - (20.0 + 40.0 * 1.5 * -2.0)).abs() < 1e-3);
    }
}

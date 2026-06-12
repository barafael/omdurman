use bevy::prelude::*;
use omdurman_types::{HexCoord, Orientation, OverlayParams};

use crate::layout::{HexLayout, SQRT_3, cube_round};

pub fn adjusted_origin(layout: &HexLayout, offset_x: f32, offset_y: f32) -> Vec2 {
    Vec2::new(layout.origin.x + offset_x, layout.origin.y + offset_y)
}

/// Convert an axial hex coordinate to a 3D world position using overlay params.
///
/// Applies the offset-coordinate stagger on top of the calibrated layout,
/// plus a fine rotation to register against a scanned map image.
pub fn hex_world_pos(coord: HexCoord, origin: Vec2, overlay: &OverlayParams) -> Vec3 {
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
    let (q, r) = (coord.q as f32, coord.r as f32);
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
    let (rx, rz) = rotate_xz(lx, lz, overlay.rotation_deg.to_radians());
    Vec3::new(origin.x + rx, 0.0, origin.y + rz)
}

fn rotate_xz(x: f32, z: f32, angle: f32) -> (f32, f32) {
    let (s, c) = angle.sin_cos();
    (x * c - z * s, x * s + z * c)
}

/// Convert a world-space hit point to the nearest axial hex coordinate.
///
/// Inverse of [`hex_world_pos`]: subtracts the origin, undoes the fine
/// rotation, applies the inverse hex-to-pixel matrix, then rounds.
pub fn hit_to_hex(hit: Vec3, origin: Vec2, overlay: &OverlayParams) -> HexCoord {
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
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

    #[test]
    fn zero_rotation_is_identity() {
        let origin = Vec2::new(10.0, 20.0);
        let ov = overlay(Orientation::Pointy, 0.0);
        let p = hex_world_pos(HexCoord::new(3, -2), origin, &ov);
        assert!((p.x - (10.0 + 40.0 * SQRT_3 * 3.5)).abs() < 1e-3);
        assert!((p.z - (20.0 + 40.0 * 1.5 * -2.0)).abs() < 1e-3);
    }
}

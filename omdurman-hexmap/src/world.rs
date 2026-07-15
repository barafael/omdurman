use bevy::prelude::*;
use omdurman_types::{HexCoord, OverlayParams};

use crate::layout::{HexLayout, rotate_xz};

/// The local lattice position of a hex centre (pre-warp, relative to the
/// origin). Add corner offsets to this before calling [`local_to_world`] to draw
/// warped hex outlines.
pub fn hex_local_pos(coord: HexCoord, overlay: &OverlayParams) -> Vec3 {
    let layout = HexLayout::from_overlay(overlay);
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
    layout.hex_to_world_offset(coord, stagger, phase)
}

/// Push a point in local lattice space (pre-warp, relative to the origin)
/// through the full registration pipeline: keystone size-gradient, affine warp,
/// rotation, then translation by `origin`. Hex *corners* (not just centres) can
/// be mapped this way, so overlay rendering shows the same warp the grid uses.
pub fn local_to_world(local_x: f32, local_z: f32, origin: Vec2, overlay: &OverlayParams) -> Vec3 {
    let (gx, gz) = overlay.size_gradient(local_x, local_z);
    let (wx, wz) = overlay.warp(gx, gz);
    let (rx, rz) = rotate_xz(wx, wz, overlay.rotation_deg.to_radians());
    Vec3::new(origin.x + rx, 0.0, origin.y + rz)
}

/// Convert an axial hex coordinate to a 3D world position using overlay params.
///
/// Prefer [`HexLayout::hex_to_world_pos`] when the `HexLayout` resource is available.
pub fn hex_world_pos(coord: HexCoord, origin: Vec2, overlay: &OverlayParams) -> Vec3 {
    let layout = HexLayout::from_overlay(overlay);
    layout.hex_to_world_pos(coord, origin, overlay)
}

/// Convert a world-space hit point to the nearest axial hex coordinate.
///
/// Prefer [`HexLayout::world_to_hex_from_hit`] when the `HexLayout` resource is available.
pub fn hit_to_hex(hit: Vec3, origin: Vec2, overlay: &OverlayParams) -> HexCoord {
    let layout = HexLayout::from_overlay(overlay);
    layout.world_to_hex_from_hit(hit, origin, overlay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{HEX_HEIGHT_RATIO, SQRT_3};
    use omdurman_types::{GridShape, OffsetVariant, Orientation};

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
    fn world_pos_round_trips_under_affine_warp() {
        let origin = Vec2::new(70.0, -30.0);
        for orientation in [Orientation::Pointy, Orientation::Flat] {
            for &deg in &[-3.0, 0.0, 2.5] {
                for &(aspect_y, shear_x, shear_y) in
                    &[(1.05, 0.08, -0.04), (0.9, -0.1, 0.06), (1.2, 0.0, 0.0)]
                {
                    let ov = OverlayParams {
                        aspect_y,
                        shear_x,
                        shear_y,
                        ..overlay(orientation, deg)
                    };
                    for q in -5..=5 {
                        for r in -5..=5 {
                            let coord = HexCoord::new(q, r);
                            let world = hex_world_pos(coord, origin, &ov);
                            let back = hit_to_hex(world, origin, &ov);
                            assert_eq!(
                                back, coord,
                                "{orientation:?} rot {deg} aspect {aspect_y} \
                                 shear ({shear_x},{shear_y}): ({q},{r}) -> ({},{})",
                                back.q, back.r
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn warp_identity_is_noop() {
        let ov = OverlayParams::default();
        for &(x, z) in &[(1.0, 2.0), (-3.5, 0.0), (0.0, -4.2)] {
            assert_eq!(ov.warp(x, z), (x, z));
            assert_eq!(ov.unwarp(x, z), Some((x, z)));
        }
    }

    #[test]
    fn size_gradient_round_trips() {
        // Coefficients are per hex-radius; at hex_size 40 a coord ~5 hexes out
        // sits at local ~350, so keep the products well inside the fold.
        for &(gx, gy) in &[(0.001, -0.0006), (-0.0008, 0.0012), (0.002, 0.0)] {
            let ov = OverlayParams {
                size_grad_x: gx,
                size_grad_y: gy,
                ..OverlayParams::default()
            };
            for &(x, z) in &[(120.0, -80.0), (-200.0, 40.0), (0.0, 150.0)] {
                let (fx, fz) = ov.size_gradient(x, z);
                let (bx, bz) = ov.unsize_gradient(fx, fz).expect("invertible in range");
                assert!((bx - x).abs() < 1e-2, "x {x} -> {fx} -> {bx}");
                assert!((bz - z).abs() < 1e-2, "z {z} -> {fz} -> {bz}");
            }
        }
    }

    #[test]
    fn world_pos_round_trips_under_keystone() {
        let origin = Vec2::new(50.0, 60.0);
        for orientation in [Orientation::Pointy, Orientation::Flat] {
            for &(gx, gy) in &[(0.0008, 0.0), (0.0, -0.001), (0.0006, 0.0009)] {
                let ov = OverlayParams {
                    size_grad_x: gx,
                    size_grad_y: gy,
                    aspect_y: 1.05,
                    shear_x: 0.04,
                    rotation_deg: 1.5,
                    ..overlay(orientation, 0.0)
                };
                for q in -5..=5 {
                    for r in -5..=5 {
                        let coord = HexCoord::new(q, r);
                        let world = hex_world_pos(coord, origin, &ov);
                        let back = hit_to_hex(world, origin, &ov);
                        assert_eq!(
                            back, coord,
                            "{orientation:?} keystone ({gx},{gy}): ({q},{r}) -> ({},{})",
                            back.q, back.r
                        );
                    }
                }
            }
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
                            "{orientation:?} rot {deg} deg: ({q},{r}) round-trip -> ({},{})",
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
        assert!((p.z - (20.0 + 40.0 * HEX_HEIGHT_RATIO * -2.0)).abs() < 1e-3);
    }
}

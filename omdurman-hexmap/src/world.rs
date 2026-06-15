use bevy::prelude::*;
use omdurman_types::{HexCoord, OverlayParams};

use crate::layout::{rotate_xz, HexLayout};

/// Adjust the layout origin by the overlay offset.
pub fn adjusted_origin(layout: &HexLayout, offset_x: f32, offset_y: f32) -> Vec2 {
    Vec2::new(layout.origin.x + offset_x, layout.origin.y + offset_y)
}

/// Convert an axial hex coordinate to a 3D world position using overlay params.
///
/// Prefer [`HexLayout::hex_to_world_overlay`] when the `HexLayout` resource is available.
pub fn hex_world_pos(coord: HexCoord, origin: Vec2, overlay: &OverlayParams) -> Vec3 {
    let layout = HexLayout::from_overlay(overlay);
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
    let local = layout.hex_to_world_offset(coord, stagger, phase);
    let (rx, rz) = rotate_xz(local.x, local.z, overlay.rotation_deg.to_radians());
    Vec3::new(origin.x + rx, 0.0, origin.y + rz)
}

/// Convert a world-space hit point to the nearest axial hex coordinate.
///
/// Prefer [`HexLayout::world_to_hex_overlay`] when the `HexLayout` resource is available.
pub fn hit_to_hex(hit: Vec3, origin: Vec2, overlay: &OverlayParams) -> HexCoord {
    let layout = HexLayout::from_overlay(overlay);
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
    let (dx, dz) = rotate_xz(
        hit.x - origin.x,
        hit.z - origin.y,
        -overlay.rotation_deg.to_radians(),
    );
    layout.world_to_hex_offset(Vec3::new(dx, 0.0, dz), stagger, phase)
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
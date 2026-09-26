use bevy::prelude::*;
use omdurman_types::{HexCoord, Orientation, OverlayParams};

pub const IMG_W: f32 = 1571.0;
pub const IMG_H: f32 = 1200.0;
/// sqrt3 -- the ratio between a regular hexagon's width and its circumradius
pub const SQRT_3: f32 = 1.732_050_8;
/// The ratio of hex height to circumradius (3/2 in axial math).
pub const HEX_HEIGHT_RATIO: f32 = 1.5;

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct MapDims {
    pub img_w: f32,
    pub img_h: f32,
}

impl Default for MapDims {
    fn default() -> Self {
        Self {
            img_w: IMG_W,
            img_h: IMG_H,
        }
    }
}

pub fn pixel_to_world_dims(px: f32, py: f32, img_w: f32, img_h: f32) -> Vec3 {
    Vec3::new(px - img_w * 0.5, 0.0, py - img_h * 0.5)
}

#[derive(Resource, Debug, Clone)]
pub struct HexLayout {
    pub origin: Vec2,
    pub hex_size: f32,
    pub orientation: Orientation,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalibrationAnchor {
    pub px: Vec2,
    pub hex: HexCoord,
}

impl HexLayout {
    /// Build a layout from overlay params with zero origin (for local-coordinate
    /// computations before the global offset/rotation is applied).
    pub(crate) fn from_overlay(overlay: &OverlayParams) -> Self {
        Self {
            origin: Vec2::ZERO,
            hex_size: overlay.hex_size,
            orientation: overlay.orientation,
        }
    }

    pub fn calibrated(
        orientation: Orientation,
        p1: CalibrationAnchor,
        p2: CalibrationAnchor,
        img_dims: Vec2,
    ) -> Self {
        let dq = (p2.hex.q - p1.hex.q) as f32;
        let dr = (p2.hex.r - p1.hex.r) as f32;
        let dx = p2.px.x - p1.px.x;
        let dz = p2.px.y - p1.px.y;

        let (s_x, s_z) = match orientation {
            Orientation::Pointy => (
                dx / (SQRT_3 * (dq + dr * 0.5)),
                dz / (HEX_HEIGHT_RATIO * dr),
            ),
            Orientation::Flat => (
                dx / (HEX_HEIGHT_RATIO * dq),
                dz / (SQRT_3 * (dr + dq * 0.5)),
            ),
        };
        let hex_size = (s_x + s_z) * 0.5;
        let w1 = pixel_to_world_dims(p1.px.x, p1.px.y, img_dims.x, img_dims.y);

        let origin = match orientation {
            Orientation::Pointy => Vec2::new(
                w1.x - hex_size * SQRT_3 * (p1.hex.q as f32 + p1.hex.r as f32 * 0.5),
                w1.z - hex_size * HEX_HEIGHT_RATIO * p1.hex.r as f32,
            ),
            Orientation::Flat => Vec2::new(
                w1.x - hex_size * HEX_HEIGHT_RATIO * p1.hex.q as f32,
                w1.z - hex_size * SQRT_3 * (p1.hex.r as f32 + p1.hex.q as f32 * 0.5),
            ),
        };

        Self {
            origin,
            hex_size,
            orientation,
        }
    }

    pub fn hex_to_world_offset(&self, coord: HexCoord, stagger: f32, phase: f32) -> Vec3 {
        let (q, r) = (coord.q as f32, coord.r as f32);
        let (x, z) = match self.orientation {
            Orientation::Pointy => (
                self.origin.x + self.hex_size * SQRT_3 * (q + (r + phase) * stagger),
                self.origin.y + self.hex_size * HEX_HEIGHT_RATIO * r,
            ),
            Orientation::Flat => (
                self.origin.x + self.hex_size * HEX_HEIGHT_RATIO * q,
                self.origin.y + self.hex_size * SQRT_3 * (r + (q + phase) * stagger),
            ),
        };
        Vec3::new(x, 0.0, z)
    }

    pub fn world_to_hex_offset(&self, world: Vec3, stagger: f32, phase: f32) -> HexCoord {
        let x = world.x - self.origin.x;
        let z = world.z - self.origin.y;
        let (fq, fr) = match self.orientation {
            Orientation::Pointy => (
                x / (self.hex_size * SQRT_3)
                    - (z / (self.hex_size * HEX_HEIGHT_RATIO) + phase) * stagger,
                z * 2.0 / (3.0 * self.hex_size),
            ),
            Orientation::Flat => (
                x * 2.0 / (3.0 * self.hex_size),
                z / (self.hex_size * SQRT_3)
                    - (x / (self.hex_size * HEX_HEIGHT_RATIO) + phase) * stagger,
            ),
        };
        cube_round(fq, fr)
    }
}

/// Rotate a 2D point (in the XZ plane) by `angle` radians.
pub(crate) fn rotate_xz(x: f32, z: f32, angle: f32) -> (f32, f32) {
    let (s, c) = angle.sin_cos();
    (x * c - z * s, x * s + z * c)
}

impl HexLayout {
    /// Compute the overlay-adjusted origin from this layout's base origin
    /// and an overlay's offset.
    pub fn adjusted_origin(&self, overlay: &OverlayParams) -> Vec2 {
        Vec2::new(
            self.origin.x + overlay.offset_x,
            self.origin.y + overlay.offset_y,
        )
    }

    /// Convert an axial hex coordinate to a 3D world position, applying the
    /// overlay's warp/rotation pipeline. The caller supplies the already-adjusted
    /// origin (see [`Self::adjusted_origin`]).
    pub fn hex_to_world_pos(&self, coord: HexCoord, origin: Vec2, overlay: &OverlayParams) -> Vec3 {
        let stagger = overlay.offset_variant.stagger();
        let phase = overlay.offset_variant.phase();
        let local_layout = Self::from_overlay(overlay);
        let local = local_layout.hex_to_world_offset(coord, stagger, phase);
        let (gx, gz) = overlay.size_gradient(local.x, local.z);
        let (wx, wz) = overlay.warp(gx, gz);
        let (rx, rz) = rotate_xz(wx, wz, overlay.rotation_deg.to_radians());
        Vec3::new(origin.x + rx, 0.0, origin.y + rz)
    }

    /// Convert a world-space hit point to the nearest hex coordinate, applying
    /// the overlay's rotation/warp inverse pipeline. The caller supplies the
    /// already-adjusted origin (see [`Self::adjusted_origin`]).
    pub fn world_to_hex_from_hit(
        &self,
        hit: Vec3,
        origin: Vec2,
        overlay: &OverlayParams,
    ) -> HexCoord {
        let stagger = overlay.offset_variant.stagger();
        let phase = overlay.offset_variant.phase();
        let (dx, dz) = rotate_xz(
            hit.x - origin.x,
            hit.z - origin.y,
            -overlay.rotation_deg.to_radians(),
        );
        let (ux, uz) = overlay.unwarp(dx, dz).unwrap_or((dx, dz));
        let (px, pz) = overlay.unsize_gradient(ux, uz).unwrap_or((ux, uz));
        let local_layout = Self::from_overlay(overlay);
        local_layout.world_to_hex_offset(Vec3::new(px, 0.0, pz), stagger, phase)
    }
}

/// Round fractional lattice coordinates to the nearest hex.
///
/// The grid's stagger is `-0.5` ([`omdurman_types::OffsetVariant::stagger`]),
/// so the lattice basis vectors `q` and `r` sit 120 degrees apart and the
/// third neighbour direction is `q + r`. The matching cube axis is therefore
/// `s = r - q` (the same one [`HexCoord::distance`] uses), with the cube
/// constraint `q - r + s = 0` -- *not* the textbook `s = -q - r`, which
/// belongs to the `+0.5` axial convention and snaps about a third of
/// off-centre points to a neighbour.
pub(crate) fn cube_round(fq: f32, fr: f32) -> HexCoord {
    let fs = fr - fq;
    let mut rq = fq.round();
    let mut rr = fr.round();
    let rs = fs.round();
    let dq = (rq - fq).abs();
    let dr = (rr - fr).abs();
    let ds = (rs - fs).abs();
    if dq > dr && dq > ds {
        rq = rr - rs;
    } else if dr > ds {
        rr = rq + rs;
    }
    HexCoord::new(rq as i32, rr as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tiny deterministic LCG so the test needs no RNG dependency.
    fn next(seed: &mut u64) -> f32 {
        *seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((*seed >> 40) as f32) / ((1u64 << 24) as f32)
    }

    #[test]
    fn world_to_hex_offset_picks_nearest_centre() {
        let stagger = -0.5;
        for orientation in [Orientation::Pointy, Orientation::Flat] {
            for phase in [0.0, 1.0] {
                let layout = HexLayout {
                    origin: Vec2::new(13.0, -7.0),
                    hex_size: 40.0,
                    orientation,
                };
                let mut seed = 0x5eed_u64;
                for _ in 0..5000 {
                    let x = layout.origin.x + (next(&mut seed) - 0.5) * 800.0;
                    let z = layout.origin.y + (next(&mut seed) - 0.5) * 800.0;
                    let p = Vec3::new(x, 0.0, z);
                    let got = layout.world_to_hex_offset(p, stagger, phase);
                    let d_got = layout.hex_to_world_offset(got, stagger, phase).distance(p);
                    // Brute-force the nearest centre around the answer.
                    let mut best = d_got;
                    for dq in -2..=2 {
                        for dr in -2..=2 {
                            let c = HexCoord::new(got.q + dq, got.r + dr);
                            best =
                                best.min(layout.hex_to_world_offset(c, stagger, phase).distance(p));
                        }
                    }
                    assert!(
                        d_got <= best + 1e-3,
                        "{orientation:?} phase {phase}: ({x},{z}) -> ({},{}) at {d_got}, \
                         a neighbour centre is {best} away",
                        got.q,
                        got.r
                    );
                }
            }
        }
    }
}

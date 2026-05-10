use bevy::prelude::*;
use crate::map::HexCoord;

// ── Image / world scale ───────────────────────────────────────────────────────

/// Source image dimensions in pixels.
pub const IMG_W: f32 = 1571.0;
pub const IMG_H: f32 = 1200.0;

/// World-space size of the map plane — 1 px = 1 world unit, so pixel
/// arithmetic and world arithmetic are the same thing.
pub const MAP_W: f32 = IMG_W;
pub const MAP_H: f32 = IMG_H;

/// Convert an image pixel position (origin = top-left) to a Bevy world
/// position on the ground plane (Y = 0, origin = centre of map image).
pub fn pixel_to_world(px: f32, py: f32) -> Vec3 {
    Vec3::new(px - IMG_W * 0.5, 0.0, py - IMG_H * 0.5)
}

/// Inverse of `pixel_to_world`.
pub fn world_to_pixel(world: Vec3) -> Vec2 {
    Vec2::new(world.x + IMG_W * 0.5, world.z + IMG_H * 0.5)
}

// ── Hex layout ────────────────────────────────────────────────────────────────

const SQRT_3: f32 = 1.732_050_8;

/// Calibrated parameters mapping axial hex coordinates to world space.
///
/// Flat-top orientation throughout:
///   world_x = origin.x  +  hex_size × 3/2 × q
///   world_z = origin.y  +  hex_size × √3  × (r + q/2)
#[derive(Resource, Debug, Clone)]
pub struct HexLayout {
    /// World-space XZ position of hex (0, 0).
    pub origin: Vec2,
    /// Circumradius (centre → vertex) in world units (= pixels at 1:1 scale).
    pub hex_size: f32,
}

impl HexLayout {
    /// Derive layout from two reference hexes whose pixel centres are known.
    ///
    /// Calibration (2026-05-10):
    ///   Austrian Mission  pixel (736, 420)  →  axial ( 0,  0)
    ///   Barracks          pixel (1178, 572)  →  axial ( 5, −1)
    ///
    /// From Δx = 442:  s = 442 / (1.5 × 5)      = 58.93
    /// From Δz = 152:  s = 152 / (√3  × 1.5)    = 58.50
    /// Average                                   ≈ 58.72
    pub fn calibrated(
        p1_px: Vec2, p1_hex: HexCoord,
        p2_px: Vec2, p2_hex: HexCoord,
    ) -> Self {
        let dq = (p2_hex.q - p1_hex.q) as f32;
        let dr = (p2_hex.r - p1_hex.r) as f32;

        // World-space delta (1 px = 1 unit).
        let dx = p2_px.x - p1_px.x;
        let dz = p2_px.y - p1_px.y; // pixel-y maps to world-z

        // Two independent estimates of hex_size; average them.
        let s_x = dx / (1.5 * dq);
        let s_z = dz / (SQRT_3 * (dr + dq * 0.5));
        let hex_size = (s_x + s_z) * 0.5;

        // Origin = world position of hex (0,0), derived from the first reference.
        let w1 = pixel_to_world(p1_px.x, p1_px.y);
        let origin = Vec2::new(
            w1.x - hex_size * 1.5 * p1_hex.q as f32,
            w1.z - hex_size * SQRT_3 * (p1_hex.r as f32 + p1_hex.q as f32 * 0.5),
        );

        Self { origin, hex_size }
    }

    /// Axial (q, r) → world XZ.
    pub fn hex_to_world(&self, coord: HexCoord) -> Vec3 {
        let (q, r) = (coord.q as f32, coord.r as f32);
        Vec3::new(
            self.origin.x + self.hex_size * 1.5 * q,
            0.0,
            self.origin.y + self.hex_size * SQRT_3 * (r + q * 0.5),
        )
    }

    /// World XZ → nearest axial hex coord.
    pub fn world_to_hex(&self, world: Vec3) -> HexCoord {
        let x = world.x - self.origin.x;
        let z = world.z - self.origin.y;
        let fq = x * 2.0 / 3.0 / self.hex_size;
        let fr = (z / SQRT_3 - x / 3.0) / self.hex_size;
        cube_round(fq, fr)
    }
}

/// Round fractional axial coordinates to the nearest hex centre.
fn cube_round(fq: f32, fr: f32) -> HexCoord {
    let fs = -fq - fr;
    let mut rq = fq.round();
    let mut rr = fr.round();
    let rs = fs.round();
    let dq = (rq - fq).abs();
    let dr = (rr - fr).abs();
    let ds = (rs - fs).abs();
    if dq > dr && dq > ds {
        rq = -rr - rs;
    } else if dr > ds {
        rr = -rq - rs;
    }
    HexCoord::new(rq as i32, rr as i32)
}

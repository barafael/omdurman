use bevy::prelude::*;
use omdurman_types::{HexCoord, Orientation};

pub const IMG_W: f32 = 1571.0;
pub const IMG_H: f32 = 1200.0;
pub const MAP_W: f32 = IMG_W;
pub const MAP_H: f32 = IMG_H;

/// √3 — the ratio between a regular hexagon's width and its circumradius
pub const SQRT_3: f32 = 1.732_050_8;

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

pub fn world_to_pixel_dims(world: Vec3, img_w: f32, img_h: f32) -> Vec2 {
    Vec2::new(world.x + img_w * 0.5, world.z + img_h * 0.5)
}

pub fn pixel_to_world(px: f32, py: f32) -> Vec3 {
    pixel_to_world_dims(px, py, IMG_W, IMG_H)
}

pub fn world_to_pixel(world: Vec3) -> Vec2 {
    world_to_pixel_dims(world, IMG_W, IMG_H)
}

#[derive(Resource, Debug, Clone)]
pub struct HexLayout {
    pub origin: Vec2,
    pub hex_size: f32,
    pub orientation: Orientation,
}

impl HexLayout {
    #[allow(clippy::too_many_arguments)]
    pub fn calibrated(
        orientation: Orientation,
        p1_px: Vec2,
        p1_hex: HexCoord,
        p2_px: Vec2,
        p2_hex: HexCoord,
        img_w: f32,
        img_h: f32,
    ) -> Self {
        let dq = (p2_hex.q - p1_hex.q) as f32;
        let dr = (p2_hex.r - p1_hex.r) as f32;
        let dx = p2_px.x - p1_px.x;
        let dz = p2_px.y - p1_px.y;

        let (s_x, s_z) = match orientation {
            Orientation::Pointy => (dx / (SQRT_3 * (dq + dr * 0.5)), dz / (1.5 * dr)),
            Orientation::Flat => (dx / (1.5 * dq), dz / (SQRT_3 * (dr + dq * 0.5))),
        };
        let hex_size = (s_x + s_z) * 0.5;
        let w1 = pixel_to_world_dims(p1_px.x, p1_px.y, img_w, img_h);

        let origin = match orientation {
            Orientation::Pointy => Vec2::new(
                w1.x - hex_size * SQRT_3 * (p1_hex.q as f32 + p1_hex.r as f32 * 0.5),
                w1.z - hex_size * 1.5 * p1_hex.r as f32,
            ),
            Orientation::Flat => Vec2::new(
                w1.x - hex_size * 1.5 * p1_hex.q as f32,
                w1.z - hex_size * SQRT_3 * (p1_hex.r as f32 + p1_hex.q as f32 * 0.5),
            ),
        };

        Self {
            origin,
            hex_size,
            orientation,
        }
    }

    pub fn hex_to_world(&self, coord: HexCoord) -> Vec3 {
        let (q, r) = (coord.q as f32, coord.r as f32);
        let (x, z) = match self.orientation {
            Orientation::Pointy => (
                self.origin.x + self.hex_size * SQRT_3 * (q + r * 0.5),
                self.origin.y + self.hex_size * 1.5 * r,
            ),
            Orientation::Flat => (
                self.origin.x + self.hex_size * 1.5 * q,
                self.origin.y + self.hex_size * SQRT_3 * (r + q * 0.5),
            ),
        };
        Vec3::new(x, 0.0, z)
    }

    pub fn world_to_hex(&self, world: Vec3) -> HexCoord {
        let x = world.x - self.origin.x;
        let z = world.z - self.origin.y;
        let (fq, fr) = match self.orientation {
            Orientation::Pointy => (
                (x * SQRT_3 / 3.0 - z / 3.0) / self.hex_size,
                (z * 2.0 / 3.0) / self.hex_size,
            ),
            Orientation::Flat => (
                (x * 2.0 / 3.0) / self.hex_size,
                (-x / 3.0 + SQRT_3 / 3.0 * z) / self.hex_size,
            ),
        };
        cube_round(fq, fr)
    }
}

pub fn cube_round(fq: f32, fr: f32) -> HexCoord {
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

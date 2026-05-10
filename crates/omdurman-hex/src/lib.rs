use bevy::prelude::*;
use omdurman_types::HexCoord;

pub const IMG_W: f32 = 1571.0;
pub const IMG_H: f32 = 1200.0;
pub const MAP_W: f32 = IMG_W;
pub const MAP_H: f32 = IMG_H;
pub const SQRT_3: f32 = 1.732_050_8;

pub fn pixel_to_world(px: f32, py: f32) -> Vec3 {
    Vec3::new(px - IMG_W * 0.5, 0.0, py - IMG_H * 0.5)
}

pub fn world_to_pixel(world: Vec3) -> Vec2 {
    Vec2::new(world.x + IMG_W * 0.5, world.z + IMG_H * 0.5)
}

#[derive(Resource, Debug, Clone)]
pub struct HexLayout {
    pub origin: Vec2,
    pub hex_size: f32,
}

impl HexLayout {
    pub fn calibrated(p1_px: Vec2, p1_hex: HexCoord, p2_px: Vec2, p2_hex: HexCoord) -> Self {
        let dq = (p2_hex.q - p1_hex.q) as f32;
        let dr = (p2_hex.r - p1_hex.r) as f32;
        let dx = p2_px.x - p1_px.x;
        let dz = p2_px.y - p1_px.y;
        let s_x = dx / (SQRT_3 * (dq + dr * 0.5));
        let s_z = dz / (1.5 * dr);
        let hex_size = (s_x + s_z) * 0.5;
        let w1 = pixel_to_world(p1_px.x, p1_px.y);
        let origin = Vec2::new(
            w1.x - hex_size * SQRT_3 * (p1_hex.q as f32 + p1_hex.r as f32 * 0.5),
            w1.z - hex_size * 1.5 * p1_hex.r as f32,
        );
        Self { origin, hex_size }
    }

    pub fn hex_to_world(&self, coord: HexCoord) -> Vec3 {
        let (q, r) = (coord.q as f32, coord.r as f32);
        Vec3::new(
            self.origin.x + self.hex_size * SQRT_3 * (q + r * 0.5),
            0.0,
            self.origin.y + self.hex_size * 1.5 * r,
        )
    }

    pub fn world_to_hex(&self, world: Vec3) -> HexCoord {
        let x = world.x - self.origin.x;
        let z = world.z - self.origin.y;
        let fq = (x * SQRT_3 / 3.0 - z / 3.0) / self.hex_size;
        let fr = (z * 2.0 / 3.0) / self.hex_size;
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

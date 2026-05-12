use std::path::Path;

use image::{RgbaImage, imageops};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
struct UnitGrid {
    name: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    cols: u32,
    rows: u32,
}

/// Splits an interval [start, start+len) into `n` pixel-perfect segments
/// by distributing the remainder across the first segments.
fn split_interval(start: f32, len: f32, n: u32) -> Vec<(u32, u32)> {
    let base = (len / n as f32).floor() as u32;
    let extra = len as u32 - base * n;
    let mut offset = start.round() as u32;
    let mut segs = Vec::with_capacity(n as usize);
    for i in 0..n {
        let w = base + if i < extra { 1 } else { 0 };
        segs.push((offset, w));
        offset += w;
    }
    segs
}

fn main() {
    let manifest = env!("CARGO_MANIFEST_DIR");

    // load source image
    let src_path = Path::new(manifest).join("assets").join("units.png");
    let src = image::open(&src_path)
        .expect("units.png not found")
        .to_rgba8();
    println!(
        "loaded {} ({}×{})",
        src_path.display(),
        src.width(),
        src.height()
    );

    // load grid definitions
    let ron_path = Path::new(manifest).join("assets").join("unit_grids.ron");
    let ron_text = std::fs::read_to_string(&ron_path).expect("unit_grids.ron not found");
    let grids: Vec<UnitGrid> = ron::from_str(&ron_text).expect("failed to parse unit_grids.ron");
    println!("loaded {} grids\n", grids.len());

    // output directory
    let out_dir = Path::new(manifest).join("assets").join("sprites");
    std::fs::create_dir_all(&out_dir).expect("failed to create sprites/");

    let mut total = 0;

    for g in &grids {
        let cols = split_interval(g.x, g.width, g.cols);
        let rows = split_interval(g.y, g.height, g.rows);

        for (ri, &(py, ch)) in rows.iter().enumerate() {
            for (ci, &(px, cw)) in cols.iter().enumerate() {
                let cell: RgbaImage = imageops::crop_imm(&src, px, py, cw, ch).to_image();
                let safe_name = g.name.replace(' ', "_");
                let filename = format!("{}_{}_{}.png", safe_name, ci, ri);
                cell.save(out_dir.join(&filename))
                    .expect("failed to save sprite");
                total += 1;
            }
        }
    }

    println!("wrote {} sprites to {}", total, out_dir.display());
}

//! Build script: emit the `SPRITE_PATHS` sprite index (shared generator in
//! `omdurman-types::build_support`).

use std::path::Path;

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sprite_dir = manifest.join("assets").join("sprites");
    omdurman_types::build_support::generate_sprite_index(&sprite_dir);
    // unit_grids.ron is tracked automatically via include_str!
}

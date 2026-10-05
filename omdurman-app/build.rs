//! Build script: emit the `SPRITE_PATHS` sprite index (shared generator in
//! `omdurman-types::build_support`).

use std::path::PathBuf;

fn main() {
    // Read at run time, not `env!` (compile time): a build script compiled
    // before the workspace moved would otherwise scan the old path.
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let sprite_dir = manifest.join("assets").join("sprites");
    omdurman_types::build_support::generate_sprite_index(&sprite_dir);
}

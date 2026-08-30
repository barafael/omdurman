//! Build script: emit the `SPRITE_PATHS` sprite index (shared generator in
//! `omdurman-types::build_support`). The canonical sprites live in the app's
//! assets dir so both the game and this tool see the same files.

use std::path::Path;

fn main() {
    let sprite_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../omdurman-app/assets/sprites");
    omdurman_types::build_support::generate_sprite_index(&sprite_dir);
}

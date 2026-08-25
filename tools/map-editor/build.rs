//! Build script: scan the game's cut-sprite directory (shared with the app)
//! and emit the `SPRITE_PATHS` index used by the sprite browser.

use std::path::Path;

fn main() {
    // The canonical sprites live in the app's assets dir so both the game and
    // this tool see the same files.
    let sprite_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../omdurman-app/assets/sprites");

    let mut entries: Vec<String> = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&sprite_dir) {
        for entry in dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("webp") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());
            if let Some(stem) = stem {
                let parts: Vec<&str> = stem.rsplitn(3, '_').collect();
                if parts.len() == 3
                    && parts[1].parse::<u32>().is_ok()
                    && parts[0].parse::<u32>().is_ok()
                {
                    entries.push(stem);
                }
            }
        }
    }
    entries.sort();

    let out = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out).join("sprites.rs");

    let mut code = String::from(
        "#[allow(non_upper_case_globals)]\npub static SPRITE_PATHS: &[(&str, u32, u32)] = &[\n",
    );
    for e in &entries {
        let parts: Vec<&str> = e.rsplitn(3, '_').collect();
        code.push_str(&format!("    (\"{}\", {}, {}),\n", e, parts[1], parts[0]));
    }
    code.push_str("];\n");

    std::fs::write(&out_path, &code).unwrap();

    for entry in &entries {
        println!(
            "cargo:rerun-if-changed={}",
            sprite_dir.join(format!("{}.webp", entry)).display()
        );
    }
    println!("cargo:rerun-if-changed={}", sprite_dir.display());
}

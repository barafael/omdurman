use std::path::Path;

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sprite_dir = manifest.join("assets").join("sprites");

    let mut entries: Vec<String> = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&sprite_dir) {
        for entry in dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("png") {
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
            sprite_dir.join(format!("{}.png", entry)).display()
        );
    }
    println!("cargo:rerun-if-changed={}", sprite_dir.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest
            .join("assets")
            .join("sprite_annotations.ron")
            .display()
    );
}

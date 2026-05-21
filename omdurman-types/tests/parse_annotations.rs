use omdurman_types::AnnotationsFile;
use std::path::Path;

#[test]
fn parse_unified_annotations() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("omdurman-app/assets/annotations.ron");
    let ron_str =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("failed to read {:?}", path));
    let data: AnnotationsFile = ron::from_str(&ron_str).expect("failed to parse annotations.ron");
    assert_eq!(data.sprites.units.len(), 17, "expected 17 unit sections");
    let total: usize = data.sprites.units.values().map(|m| m.len()).sum();
    assert_eq!(total, 238, "expected 238 sprites");

    // Verify insertion order is preserved
    let keys: Vec<&str> = data.sprites.units.keys().map(|s| s.as_str()).collect();
    assert_eq!(keys[0], "Taiasha");
    assert_eq!(keys[1], "Khalifa_Abdullah");

    let taiasha = &data.sprites.units["Taiasha"];
    assert_eq!(
        taiasha[&(0, 0)].color,
        omdurman_types::SpriteColor::BlackWhite
    );
    assert_eq!(
        taiasha[&(0, 0)].faction,
        omdurman_types::Faction::Independent
    );
    let british = &data.sprites.units["British_Army"];
    assert_eq!(
        british[&(0, 0)].color,
        omdurman_types::SpriteColor::SandBlack
    );
    assert_eq!(
        british[&(0, 0)].faction,
        omdurman_types::Faction::BritishEgyptian
    );
}

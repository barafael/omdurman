use omdurman_types::SpriteAnnotations;
use std::path::Path;

#[test]
fn parse_sprite_annotations() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("omdurman-app/assets/sprite_annotations.ron");
    let ron_str = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("failed to read {:?}", path));
    let data: SpriteAnnotations =
        ron::from_str(&ron_str).expect("failed to parse sprite_annotations.ron");
    assert_eq!(data.units.len(), 17, "expected 17 units");
    let total: usize = data.units.values().map(|m| m.len()).sum();
    assert_eq!(total, 238, "expected 238 sprites");

    // Verify insertion order is preserved (browser.rs section order)
    let keys: Vec<&str> = data.units.keys().map(|s| s.as_str()).collect();
    assert_eq!(keys[0], "Talasha");
    assert_eq!(keys[1], "Khalifa_Abdullah");

    let talasha = data.units.get("Talasha").unwrap();
    assert_eq!(
        talasha[&(0, 0)].color,
        omdurman_types::SpriteColor::BlackWhite
    );
    assert_eq!(
        talasha[&(0, 0)].faction,
        omdurman_types::Faction::Independent
    );
    let british = data.units.get("British_Army").unwrap();
    assert_eq!(
        british[&(0, 0)].color,
        omdurman_types::SpriteColor::SandBlack
    );
    assert_eq!(
        british[&(0, 0)].faction,
        omdurman_types::Faction::BritishEgyptian
    );
}

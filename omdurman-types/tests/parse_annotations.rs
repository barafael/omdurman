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
    // Sprite annotations are global (board-independent), at the top level.
    let sprites = &data.sprites;
    assert_eq!(sprites.units.len(), 17, "expected 17 unit sections");
    let total: usize = sprites.units.values().map(|m| m.len()).sum();
    assert_eq!(total, 238, "expected 238 sprites");
    assert_eq!(data.fall_of_khartoum.image, "fall_of_khartoum_1885.png");

    // The campaign board exists with its portrait image and alternating-row
    // topology. (Its tile set may be empty or populated depending on how far
    // calibration has progressed, so we don't assert on tile count.)
    assert_eq!(data.campaign.image, "campaign_map.png");
    assert_eq!(
        data.campaign.overlay.shape,
        omdurman_types::GridShape::AlternatingRows
    );

    // Verify insertion order is preserved
    let keys: Vec<&str> = sprites.units.keys().map(|s| s.as_str()).collect();
    assert_eq!(keys[0], "Taiasha");
    assert_eq!(keys[1], "Khalifa_Abdullah");

    let taiasha = &sprites.units["Taiasha"];
    assert_eq!(
        taiasha[&(0, 0)].color,
        omdurman_types::SpriteColor::BlackWhite
    );
    assert_eq!(
        taiasha[&(0, 0)].faction,
        omdurman_types::Faction::Independent
    );
    let british = &sprites.units["British_Army"];
    assert_eq!(
        british[&(0, 0)].color,
        omdurman_types::SpriteColor::SandBlack
    );
    assert_eq!(
        british[&(0, 0)].faction,
        omdurman_types::Faction::BritishEgyptian
    );
}

/// The game record (JSONL) and net history are JSON-serialized. JSON object
/// keys must be strings, so the `(col,row)` / `(q,r)` tuple-keyed maps in
/// `AnnotationsFile` must serialize as lists of pairs (via `serde_as`) rather
/// than as objects. This guards against the regression where `LoadAnnotations`
/// silently failed to serialize ("key must be a string").
#[test]
fn annotations_round_trip_through_json() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("omdurman-app/assets/annotations.ron");
    let ron_str = std::fs::read_to_string(&path).unwrap();
    let data: AnnotationsFile = ron::from_str(&ron_str).unwrap();

    let json = serde_json::to_string(&data).expect("AnnotationsFile must serialize to JSON");
    let back: AnnotationsFile =
        serde_json::from_str(&json).expect("AnnotationsFile must deserialize from JSON");

    assert_eq!(back.sprites.units.len(), data.sprites.units.len());
    let total: usize = back.sprites.units.values().map(|m| m.len()).sum();
    assert_eq!(total, 238);
    assert_eq!(
        back.fall_of_khartoum.tiles.len(),
        data.fall_of_khartoum.tiles.len()
    );
    // A representative tuple-keyed lookup survives the JSON round-trip.
    assert_eq!(
        back.sprites.units["Taiasha"][&(0, 0)].color,
        omdurman_types::SpriteColor::BlackWhite
    );
}

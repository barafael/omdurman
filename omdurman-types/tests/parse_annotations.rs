use omdurman_types::{AnnotationsFile, SectionName};
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
    assert_eq!(data.fall_of_khartoum.image, "fall_of_khartoum_1885.webp");

    // The campaign board exists with its portrait image and alternating-row
    // topology. (Its tile set may be empty or populated depending on how far
    // calibration has progressed, so we don't assert on tile count.)
    assert_eq!(data.campaign.image, "campaign_map.webp");
    assert_eq!(
        data.campaign.overlay.shape,
        omdurman_types::GridShape::AlternatingRows
    );

    // Verify insertion order is preserved
    let keys: Vec<&SectionName> = sprites.units.keys().collect();
    assert_eq!(keys[0], &SectionName::Taiasha);
    assert_eq!(keys[1], &SectionName::KhalifaAbdullah);

    let taiasha = &sprites.units[&SectionName::Taiasha];
    assert_eq!(
        taiasha[&(0, 0)].color,
        omdurman_types::SpriteColor::BlackWhite
    );
    assert_eq!(
        taiasha[&(0, 0)].faction,
        Some(omdurman_types::Faction::Dervish {
            tribe: omdurman_types::DervishTribe::Taiasha,
        })
    );
    let british = &sprites.units[&SectionName::BritishArmy];
    assert_eq!(
        british[&(0, 0)].color,
        omdurman_types::SpriteColor::SandBlack
    );
    assert_eq!(
        british[&(0, 0)].faction,
        Some(omdurman_types::Faction::BritishEgyptian {
            brigade: None,
        })
    );
}

/// Throwaway test: verify old-format annotations.ron migrates to the new
/// data-carrying `UnitKind` format. The file on disk still has the flat
/// `(kind: Some(Infantry), fire: 3, melee: 6, ...)` shape; our custom
/// `Deserialize` must read it and produce the correct `UnitKind::Infantry {
/// fire, melee, movement }` inside each `SpriteAnnotation`.
#[test]
fn old_format_migrates_to_data_carrying_unit_kind() {
    use omdurman_types::UnitKind;

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("omdurman-app/assets/annotations.ron");
    let ron_str = std::fs::read_to_string(&path).unwrap();
    let data: AnnotationsFile = ron::from_str(&ron_str).unwrap();
    let sprites = &data.sprites;

    // --- Taiasha (Infantry, 3-6-9 in old file) ---
    let taiasha = &sprites.units[&SectionName::Taiasha];
    let t0 = taiasha[&(0, 0)].kind.as_ref().expect("Taiasha (0,0) should have a kind");
    match t0 {
        UnitKind::Infantry { fire, melee, movement } => {
            assert_eq!(*fire, 3, "Taiasha fire should be 3");
            assert_eq!(*melee, 6, "Taiasha melee should be 6");
            assert_eq!(*movement, 9, "Taiasha movement should be 9");
        }
        other => panic!("expected Infantry, got {other:?}"),
    }

    // --- Khalifa Abdullah (0,0) is stored as Infantry in the editor (identity
    //     is derived by the rules engine from section+position, not from the
    //     annotation kind).
    let khalifa = &sprites.units[&SectionName::KhalifaAbdullah];
    let k0 = khalifa[&(0, 0)].kind.as_ref().expect("Khalifa (0,0) kind");
    match k0 {
        UnitKind::Infantry { fire, melee, movement } => {
            assert_eq!(*fire, 1, "Khalifa fire");
            assert_eq!(*melee, 1, "Khalifa melee");
            assert_eq!(*movement, 15, "Khalifa movement");
        }
        other => panic!("expected Infantry (editor kind), got {other:?}"),
    }

    // --- BritishArmy (0,0) is infantry ---
    let british = &sprites.units[&SectionName::BritishArmy];
    let b0 = british[&(0, 0)].kind.as_ref().expect("British (0,0) kind");
    assert!(matches!(b0, UnitKind::Infantry { .. }), "British infantry, got {b0:?}");

    // --- BritishBoats should contain a gunboat ---
    if let Some(boats) = sprites.units.get(&SectionName::BritishBoats) {
        // Gunboats are at specific positions; find one with upstream/downstream.
        let has_gunboat = boats.values().any(|a| {
            matches!(
                a.kind.as_ref(),
                Some(UnitKind::Gunboat { upstream, downstream, .. }) if *upstream > 0 && *downstream > 0
            )
        });
        assert!(has_gunboat, "BritishBoats should contain a Gunboat with nonzero movement");
    }

    // --- HadendowaForts should be Fort ---
    if let Some(forts) = sprites.units.get(&SectionName::HadendowaForts) {
        for ((col, row), ann) in forts {
            let k = ann.kind.as_ref().expect("fort cell should have kind");
            assert!(
                matches!(k, UnitKind::Fort { .. }),
                "HadendowaForts ({col},{row}) should be Fort, got {k:?}"
            );
        }
    }

    // --- Kitchener is stored as Infantry with 0-0-15 (identity derived by rules engine) ---
    if let Some(kitcheners) = sprites.units.get(&SectionName::Kitchener) {
        let k0 = kitcheners[&(0, 0)].kind.as_ref().expect("Kitchener kind");
        match k0 {
            UnitKind::Infantry { fire, melee, movement } => {
                assert_eq!(*fire, 0, "Kitchener fire");
                assert_eq!(*melee, 0, "Kitchener melee");
                assert_eq!(*movement, 15, "Kitchener movement");
            }
            other => panic!("expected Infantry (editor kind), got {other:?}"),
        }
    }

    // --- Gunboats should have nonzero upstream/downstream ---
    let mut found_gunboat = false;
    for (_section, map) in &sprites.units {
        for (_pos, ann) in map {
            match &ann.kind {
                Some(UnitKind::Gunboat { upstream, downstream, .. }) if *upstream > 0 => {
                    found_gunboat = true;
                }
                Some(UnitKind::NamedGunboat { .. }) => {
                    // NamedGunboat may or may not exist in the current file.
                }
                _ => {}
            }
        }
    }
    assert!(found_gunboat, "should have at least one Gunboat with nonzero movement");
    // NamedGunboat may or may not exist in the current file — it's only
    // assigned when the editor explicitly sets it.  Don't assert on it.

    // --- Round-trip: serialize back to RON, re-parse, data is identical ---
    let re_serialized = ron::to_string(&data).expect("re-serialize to RON");
    let re_parsed: AnnotationsFile = ron::from_str(&re_serialized).expect("re-parse RON");
    // Taiasha kind survives the round-trip.
    let rt = re_parsed.sprites.units[&SectionName::Taiasha][&(0, 0)]
        .kind.as_ref().unwrap();
    assert_eq!(rt, t0, "round-trip should preserve kind data");

    eprintln!("old-format migration test passed: {} sections, {} sprites",
        sprites.units.len(),
        sprites.units.values().map(|m| m.len()).sum::<usize>(),
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
        back.sprites.units[&SectionName::Taiasha][&(0, 0)].color,
        omdurman_types::SpriteColor::BlackWhite
    );
}

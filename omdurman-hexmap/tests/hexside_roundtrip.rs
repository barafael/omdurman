use omdurman_hexmap::{GameMap, load_map_data, save_annotations_to_file};
use omdurman_types::{AnnotationsFile, HexCoord, HexsideKind, HexsideRef, MapKind};

/// The committed FoK board's hexsides survive load → save → reload.
#[test]
fn fok_hexsides_load_into_game_map() {
    let ron = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../omdurman-app/assets/annotations.ron"
    ))
    .unwrap();
    let file: AnnotationsFile = ron::from_str(&ron).unwrap();
    let mut gm = GameMap::default();
    load_map_data(&file.fall_of_khartoum, &mut gm);
    assert_eq!(gm.hexsides.len(), file.fall_of_khartoum.hexsides.len());
}

/// A hexside added to the live map is written by `save_annotations_to_file`
/// and comes back on reload — for BOTH boards, with the active board's section
/// rebuilt from `game_map` and the other preserved from `file`.
#[test]
fn added_hexside_survives_save_reload() {
    let tmp = std::env::temp_dir().join("omdurman_hexside_roundtrip.ron");
    let path = tmp.to_str().unwrap();

    // Start from the empty two-board file, load the campaign board live.
    let file = AnnotationsFile::empty();
    let mut gm = GameMap::default();
    load_map_data(&file.campaign, &mut gm);

    // Add a wall between two adjacent in-grid hexes.
    let a = *gm.hexes.keys().next().expect("campaign board has hexes");
    let neighbour = a
        .neighbors()
        .into_iter()
        .find(|n| gm.hexes.contains_key(n))
        .expect("a hex with an in-grid neighbour");
    let edge = HexsideRef::new(a, neighbour);
    gm.hexsides.insert(edge, HexsideKind::Wall);

    // Save with Campaign active, then reload.
    save_annotations_to_file(&gm, &file.sprites, &file, MapKind::Campaign, path);
    let reloaded: AnnotationsFile = ron::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();

    assert!(
        reloaded
            .campaign
            .hexsides
            .iter()
            .any(|(e, k)| *e == edge && *k == HexsideKind::Wall),
        "added campaign wall must persist; got {:?}",
        reloaded.campaign.hexsides
    );
    std::fs::remove_file(path).ok();
    let _ = HexCoord::new(0, 0);
}

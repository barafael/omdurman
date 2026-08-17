//! Codegen fidelity: `MapData::to_board_data_fn` must reproduce the compiled
//! `src/board_data.rs` byte-for-byte, so regenerating the file from the map
//! editor (entrance-area authoring, terrain edits) produces no spurious diff
//! -- only the intended annotation changes.

use omdurman_rules::board_data;
use omdurman_types::MapData;

/// The text of one `pub fn ...` item in the compiled module.
fn fn_text<'a>(src: &'a str, fn_name: &str) -> &'a str {
    let start = src
        .find(&format!("pub fn {fn_name}"))
        .unwrap_or_else(|| panic!("{fn_name} not found in board_data.rs"));
    let end = src[start..]
        .find("\n}\n")
        .expect("fn closing brace not found")
        + start
        + 3;
    &src[start..end]
}

#[test]
fn board_data_codegen_is_byte_stable() {
    let src = include_str!("../src/board_data.rs");
    for (name, data) in [
        ("campaign_map_data", board_data::campaign_map_data()),
        (
            "fall_of_khartoum_map_data",
            board_data::fall_of_khartoum_map_data(),
        ),
    ] {
        let existing = fn_text(src, name);
        let generated = data.to_board_data_fn(name);
        let e: Vec<&str> = existing.trim_end().lines().collect();
        let g: Vec<&str> = generated.trim_end().lines().collect();
        for (i, (a, b)) in e.iter().zip(g.iter()).enumerate() {
            assert_eq!(
                (i, a),
                (i, b),
                "codegen drift for {name} at line {}: regenerating board_data.rs would rewrite unrelated lines",
                i + 1
            );
        }
        assert_eq!(
            e.len(),
            g.len(),
            "codegen drift for {name}: line count {} vs {}",
            e.len(),
            g.len()
        );
    }
}

#[test]
fn codegen_emits_named_area_annotations() {
    // A hand-built map with an entrance annotation round-trips through the
    // generator with the `NamedArea` marker visible in the source.
    let mut map = MapData::empty_campaign();
    map.tiles.insert(
        (2, 3),
        omdurman_types::HexData {
            terrain: omdurman_types::Terrain::Clear {
                road: omdurman_types::Road::None,
            },
            location: None,
            name: None,
            setup_letter: None,
            is_scattergram: false,
            named_area: Some(omdurman_types::NamedArea::DervishWestEdge),
        },
    );
    let src = map.to_board_data_fn("test_map_data");
    assert!(
        src.contains("named_area: Some(NamedArea::DervishWestEdge)"),
        "annotation missing from generated source:\n{src}"
    );
}

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use omdurman_types::{
    AnnotationsFile, GridShape, HexCoord, HexData, Location, OverlayParams, SpriteAnnotations,
    Terrain, TileInfo,
};

// ── Runtime game map ─────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct GameMap {
    pub hexes: HashMap<HexCoord, HexData>,
    pub overlay: OverlayParams,
}

// ── Hex set generation ───────────────────────────────────────────────────

/// Compute the set of hex coordinates implied by the overlay parameters.
///
/// Uses the offset-coordinate **rectangle trick** for pointy-top hexes:
/// loop over rows in offset space, apply the stagger to determine each
/// row's starting q, then convert to axial coordinates.
///
/// With `GridShape::Rectangle` every row has the same number of hexes;
/// with `GridShape::Parallelogram` rows vary in width naturally.
///
/// Source: https://www.redblobgames.com/grids/hexagons/implementation.html#shape-rectangle
pub fn desired_hexes(overlay: &OverlayParams) -> HashSet<HexCoord> {
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
    let mut desired = HashSet::new();
    for r in 0..overlay.height {
        let q_off = (r as f32 + phase) * stagger;
        let q_min = (-q_off).ceil() as i32;
        let q_max = if overlay.shape == GridShape::Rectangle {
            q_min + overlay.width - 1
        } else {
            (overlay.width as f32 - 1.0 - q_off).floor() as i32
        };
        for q in q_min..=q_max {
            desired.insert(HexCoord { q, r });
        }
    }
    desired
}

/// Clip `game_map.hexes` so it contains exactly the hex set implied by the
/// current overlay parameters.  Existing terrain / name data is preserved.
pub fn clip_hexes_to_overlay(game_map: &mut GameMap) {
    let desired = desired_hexes(&game_map.overlay);
    game_map.hexes.retain(|coord, _| desired.contains(coord));
    for coord in &desired {
        game_map.hexes.entry(*coord).or_insert(HexData {
            terrain: Terrain::Desert,
            location: None,
            name: None,
        });
    }
}

// ── Parse / save ─────────────────────────────────────────────────────────

/// Parse an [`AnnotationsFile`] from a RON string, populate [`GameMap`],
/// and return the full struct (caller should insert the sprites section as a
/// resource).  The hex set is automatically clipped to match overlay params.
pub fn load_annotations_from_str(ron_str: &str, game_map: &mut GameMap) -> AnnotationsFile {
    let annotations: AnnotationsFile = ron::from_str(ron_str).unwrap_or_else(|e| {
        warn!("failed to parse annotations.ron: {e}, using empty");
        AnnotationsFile::empty()
    });
    for ((q, r), tile) in &annotations.map.tiles {
        game_map.hexes.insert(
            HexCoord::new(*q, *r),
            HexData {
                terrain: tile.terrain,
                location: None,
                name: tile.name.clone(),
            },
        );
    }
    game_map.overlay = annotations.overlay.clone();
    clip_hexes_to_overlay(game_map);
    info!("loaded {} hexes from annotations.ron", game_map.hexes.len());
    annotations
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_annotations_to_file(
    game_map: &GameMap,
    sprite_annotations: &SpriteAnnotations,
    path: &str,
) {
    let tiles: HashMap<(i32, i32), TileInfo> = game_map
        .hexes
        .iter()
        .map(|(coord, data)| {
            (
                (coord.q, coord.r),
                TileInfo {
                    terrain: data.terrain,
                    name: data.name.clone(),
                },
            )
        })
        .collect();
    let annotations = AnnotationsFile {
        map: omdurman_types::MapSection { tiles },
        overlay: game_map.overlay.clone(),
        sprites: sprite_annotations.clone(),
    };
    let ron_str = ron::ser::to_string_pretty(&annotations, ron::ser::PrettyConfig::default())
        .expect("AnnotationsFile is always serializable");
    match std::fs::write(path, ron_str) {
        Ok(()) => info!("saved {} hexes to {path}", game_map.hexes.len()),
        Err(e) => error!("failed to save annotations.ron: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn save_annotations_to_file(
    _game_map: &GameMap,
    _sprite_annotations: &SpriteAnnotations,
    _path: &str,
) {
    warn!("save_annotations_to_file is not supported on wasm");
}

// ── Calibration / location tables ────────────────────────────────────────

pub const CROSS_REFS: &[(HexCoord, (f32, f32))] = &[
    (HexCoord::new(0, 1), (735.0, 523.0)),
    (HexCoord::new(0, 2), (736.0, 625.0)),
    (HexCoord::new(2, 0), (913.0, 523.0)),
    (HexCoord::new(2, 1), (913.0, 625.0)),
    (HexCoord::new(9, -8), (1532.0, 66.0)),
    (HexCoord::new(-5, 5), (292.0, 677.0)),
    (HexCoord::new(-6, 2), (205.0, 320.0)),
    (HexCoord::new(0, 7), (734.0, 1132.0)),
    (HexCoord::new(1, 6), (823.0, 1081.0)),
    (HexCoord::new(2, 6), (912.0, 1132.0)),
];

pub const LOCATIONS: &[(HexCoord, Location)] = &[
    (HexCoord::new(-4, -2), Location::FortMakran),
    (HexCoord::new(9, -7), Location::NorthFort),
    (HexCoord::new(9, -2), Location::FortBuri),
    (HexCoord::new(2, -1), Location::Palace),
    (HexCoord::new(4, -1), Location::Arsenal),
    (HexCoord::new(0, 0), Location::AustrianMission),
    (HexCoord::new(5, -1), Location::Barracks),
    (HexCoord::new(3, 4), Location::KalaklaGate),
    (HexCoord::new(5, 2), Location::MessalamiaGate),
    (HexCoord::new(9, -1), Location::BuriGate),
    (HexCoord::new(2, -5), Location::Tuti),
    (HexCoord::new(9, -8), Location::Hogali),
];

pub fn terrain_for_location(loc: Location) -> Terrain {
    match loc {
        Location::FortMakran => Terrain::FortMakran,
        Location::NorthFort => Terrain::NorthFort,
        Location::FortBuri => Terrain::FortBuri,
        Location::KalaklaGate | Location::MessalamiaGate | Location::BuriGate => Terrain::Desert,
        Location::AustrianMission | Location::Palace | Location::Arsenal | Location::Barracks => {
            Terrain::Khartoum
        }
        Location::Tuti => Terrain::Tuti,
        Location::Hogali => Terrain::Hogali,
        Location::BuriSettlement => Terrain::Buri,
    }
}

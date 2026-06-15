use std::collections::{BTreeMap, HashMap, HashSet};

use bevy::prelude::*;

use omdurman_types::{
    AnnotationsFile, GridShape, HexCoord, HexData, HexsideKind, HexsideRef, MapData,
    MapKind, OverlayParams, SpriteAnnotations, Terrain, TileInfo,
};

// -- Runtime game map -----------------------------------------------------

/// Active hex map state: hex terrain, hexsides, roads, excluded hexes, and
/// the overlay parameters that define the grid shape and orientation.
#[derive(Resource, Default)]
pub struct GameMap {
    pub hexes: HashMap<HexCoord, HexData>,
    pub hexsides: HashMap<HexsideRef, HexsideKind>,
    pub roads: HashSet<HexsideRef>,
    pub excluded: HashSet<HexCoord>,
    pub overlay: OverlayParams,
}

impl GameMap {
    pub fn hexside_between(&self, from: HexCoord, to: HexCoord) -> Option<HexsideKind> {
        self.hexsides.get(&HexsideRef::new(from, to)).copied()
    }
}

// -- Hex set generation ---------------------------------------------------

pub(crate) fn desired_hexes(overlay: &OverlayParams) -> HashSet<HexCoord> {
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
    let mut desired = HashSet::new();
    let long_row_r = if overlay.long_rows_even { 0 } else { 1 };
    let long_off = (long_row_r as f32 + phase) * stagger;
    let envelope_left = (-long_off).ceil() + long_off;

    for r in 0..overlay.height {
        let q_off = (r as f32 + phase) * stagger;
        let rect_q_min = (-q_off).ceil() as i32;
        let (q_min, q_max) = match overlay.shape {
            GridShape::Rectangle => (rect_q_min, rect_q_min + overlay.width - 1),
            GridShape::Parallelogram => (
                rect_q_min,
                (overlay.width as f32 - 1.0 - q_off).floor() as i32,
            ),
            GridShape::AlternatingRows => {
                let row_is_even = r % 2 == 0;
                let is_long = row_is_even == overlay.long_rows_even;
                if is_long {
                    let q_min = (envelope_left - q_off).round() as i32;
                    (q_min, q_min + overlay.width - 1)
                } else {
                    let short_width = (overlay.width - 1).max(0);
                    let left_wx = envelope_left + 0.5;
                    let q_min = (left_wx - q_off).round() as i32;
                    (q_min, q_min + short_width - 1)
                }
            }
        };
        for q in q_min..=q_max {
            desired.insert(HexCoord { q, r });
        }
    }
    desired
}

pub fn clip_hexes_to_overlay(game_map: &mut GameMap) {
    let mut desired = desired_hexes(&game_map.overlay);
    desired.retain(|coord| !game_map.excluded.contains(coord));
    game_map.hexes.retain(|coord, _| desired.contains(coord));
    for coord in &desired {
        game_map
            .hexes
            .entry(*coord)
            .or_insert(HexData::new(Terrain::Clear, None));
    }
}

// -- Parse / save ---------------------------------------------------------

pub fn load_map_data(map: &MapData, game_map: &mut GameMap) {
    game_map.hexes.clear();
    for ((q, r), tile) in &map.tiles {
        let mut hex = HexData::with_flow(tile.terrain, tile.name.clone(), tile.nile_flow);
        hex.is_crossroad = tile.is_crossroad;
        game_map.hexes.insert(HexCoord::new(*q, *r), hex);
    }
    game_map.hexsides = map
        .hexsides
        .iter()
        .map(|(edge, kind)| (*edge, *kind))
        .collect();
    game_map.roads = map.roads.iter().copied().collect();
    game_map.excluded = map
        .excluded
        .iter()
        .map(|(q, r)| HexCoord::new(*q, *r))
        .collect();
    game_map.overlay = map.overlay.clone();
    clip_hexes_to_overlay(game_map);
    bevy::prelude::info!(
        "loaded {} hexes for {} board",
        game_map.hexes.len(),
        map.image
    );
}

pub fn load_annotations_from_str(
    ron_str: &str,
    kind: MapKind,
    game_map: &mut GameMap,
) -> AnnotationsFile {
    let annotations: AnnotationsFile = ron::from_str(ron_str).unwrap_or_else(|e| {
        bevy::prelude::warn!("failed to parse annotations.ron: {e}, using empty");
        AnnotationsFile::empty()
    });
    load_map_data(annotations.map(kind), game_map);
    annotations
}

fn map_data_from_game_map(game_map: &GameMap, previous: &MapData) -> MapData {
    let tiles: BTreeMap<(i32, i32), TileInfo> = game_map
        .hexes
        .iter()
        .map(|(coord, data)| {
            (
                (coord.q, coord.r),
                TileInfo {
                    terrain: data.terrain,
                    name: data.name.clone(),
                    nile_flow: if data.terrain.is_nile() {
                        data.nile_flow
                    } else {
                        None
                    },
                    is_crossroad: data.is_crossroad,
                },
            )
        })
        .collect();
    let mut hexsides: Vec<(HexsideRef, HexsideKind)> =
        game_map.hexsides.iter().map(|(e, k)| (*e, *k)).collect();
    hexsides.sort_by_key(|(e, _)| (e.a.q, e.a.r, e.b.q, e.b.r));
    let mut roads: Vec<HexsideRef> = game_map.roads.iter().copied().collect();
    roads.sort_by_key(|e| (e.a.q, e.a.r, e.b.q, e.b.r));
    MapData {
        tiles,
        hexsides,
        roads,
        excluded: game_map.excluded.iter().map(|c| (c.q, c.r)).collect(),
        overlay: game_map.overlay.clone(),
        img_w: previous.img_w,
        img_h: previous.img_h,
        image: previous.image.clone(),
        calib: previous.calib.clone(),
        campaign_turn_track: previous.campaign_turn_track,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_annotations_to_file(
    game_map: &GameMap,
    sprite_annotations: &SpriteAnnotations,
    file: &AnnotationsFile,
    active: MapKind,
    path: &str,
) {
    let mut out = file.clone();
    *out.map_mut(active) = map_data_from_game_map(game_map, file.map(active));
    out.sprites = sprite_annotations.clone();
    let ron_str = ron::ser::to_string_pretty(&out, ron::ser::PrettyConfig::default())
        .expect("AnnotationsFile is always serializable");
    match std::fs::write(path, ron_str) {
        Ok(()) => bevy::prelude::info!(
            "saved {} hexes ({active} board) to {path}",
            game_map.hexes.len()
        ),
        Err(e) => bevy::prelude::error!("failed to save annotations.ron: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn save_annotations_to_file(
    _game_map: &GameMap,
    _sprite_annotations: &SpriteAnnotations,
    _file: &AnnotationsFile,
    _active: MapKind,
    _path: &str,
) {
    bevy::prelude::warn!("save_annotations_to_file is not supported on wasm");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alternating_overlay(width: i32, height: i32, long_rows_even: bool) -> OverlayParams {
        OverlayParams {
            width,
            height,
            shape: GridShape::AlternatingRows,
            long_rows_even,
            ..OverlayParams::default()
        }
    }

    fn row_counts(hexes: &HashSet<HexCoord>, height: i32) -> Vec<usize> {
        (0..height)
            .map(|r| hexes.iter().filter(|c| c.r == r).count())
            .collect()
    }

    #[test]
    fn alternating_rows_long_even() {
        let overlay = alternating_overlay(6, 4, true);
        let hexes = desired_hexes(&overlay);
        assert_eq!(row_counts(&hexes, 4), vec![6, 5, 6, 5]);
    }

    #[test]
    fn alternating_short_rows_symmetric_inset() {
        let overlay = alternating_overlay(8, 4, true);
        let hexes = desired_hexes(&overlay);
        let stagger = overlay.offset_variant.stagger();
        let phase = overlay.offset_variant.phase();
        let wx = |c: &HexCoord| c.q as f32 + (c.r as f32 + phase) * stagger;

        let long_left = hexes.iter().map(wx).fold(f32::INFINITY, f32::min);
        let long_right = hexes.iter().map(wx).fold(f32::NEG_INFINITY, f32::max);

        for r in 0..overlay.height {
            let row_is_even = r % 2 == 0;
            if row_is_even == overlay.long_rows_even {
                continue;
            }
            let row: Vec<f32> = hexes.iter().filter(|c| c.r == r).map(wx).collect();
            let left = row.iter().cloned().fold(f32::INFINITY, f32::min);
            let right = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let left_inset = left - long_left;
            let right_inset = long_right - right;
            assert!(
                (left_inset - right_inset).abs() < 1e-3,
                "row {r}: left inset {left_inset} != right inset {right_inset}"
            );
            assert!(
                (left_inset - 0.5).abs() < 1e-3,
                "row {r}: short row should nest 1/2 hex per side, got {left_inset}"
            );
        }
    }

    #[test]
    fn alternating_rows_long_odd_flips_parity() {
        let overlay = alternating_overlay(6, 4, false);
        let hexes = desired_hexes(&overlay);
        assert_eq!(row_counts(&hexes, 4), vec![5, 6, 5, 6]);
    }

    #[test]
    fn rectangle_rows_are_uniform() {
        let overlay = OverlayParams {
            width: 6,
            height: 4,
            shape: GridShape::Rectangle,
            ..OverlayParams::default()
        };
        let hexes = desired_hexes(&overlay);
        assert_eq!(row_counts(&hexes, 4), vec![6, 6, 6, 6]);
    }

    #[test]
    fn clip_drops_excluded_hexes() {
        let mut game_map = GameMap {
            overlay: OverlayParams {
                width: 4,
                height: 2,
                shape: GridShape::Rectangle,
                ..OverlayParams::default()
            },
            ..Default::default()
        };
        clip_hexes_to_overlay(&mut game_map);
        let full = game_map.hexes.len();
        assert_eq!(full, 8, "4x2 rectangle");

        let victim = *game_map.hexes.keys().next().unwrap();
        game_map.excluded.insert(victim);
        clip_hexes_to_overlay(&mut game_map);
        assert_eq!(game_map.hexes.len(), full - 1);
        assert!(!game_map.hexes.contains_key(&victim));

        game_map.excluded.remove(&victim);
        clip_hexes_to_overlay(&mut game_map);
        assert_eq!(game_map.hexes.len(), full);
        assert_eq!(game_map.hexes[&victim].terrain, Terrain::Clear);
    }
}
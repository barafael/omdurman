use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use omdurman_types::TileInfo;
use omdurman_types::{
    AnnotationsFile, GridShape, HexCoord, HexData, HexsideKind, HexsideRef, Location, MapData,
    MapKind, OverlayParams, SpriteAnnotations, Terrain,
};

// ── Runtime game map ─────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct GameMap {
    pub hexes: HashMap<HexCoord, HexData>,
    /// Per-edge hexside features, keyed by canonical [`HexsideRef`].
    pub hexsides: HashMap<HexsideRef, HexsideKind>,
    /// Editor-time exclusions: coords inside the overlay grid that are not part
    /// of the playable map (board furniture). Subtracted by
    /// [`clip_hexes_to_overlay`], so these never appear in `hexes`.
    pub excluded: HashSet<HexCoord>,
    pub overlay: OverlayParams,
}

impl GameMap {
    /// The hexside kind on the edge between two adjacent hexes, if any.
    pub fn hexside_between(&self, from: HexCoord, to: HexCoord) -> Option<HexsideKind> {
        self.hexsides.get(&HexsideRef::new(from, to)).copied()
    }
}

// ── Hex set generation ───────────────────────────────────────────────────

/// Compute the set of hex coordinates implied by the overlay parameters.
///
/// Uses the offset-coordinate **rectangle trick** for pointy-top hexes:
/// loop over rows in offset space, apply the stagger to determine each
/// row's starting q, then convert to axial coordinates.
///
/// With `GridShape::Rectangle` every row has the same number of hexes;
/// with `GridShape::Parallelogram` rows vary in width naturally; with
/// `GridShape::AlternatingRows` rows alternate between a "long" row of `width`
/// hexes and a "short" row inset one hex on each side (`width - 2`), per
/// [`OverlayParams::long_rows_even`].
///
/// Source: https://www.redblobgames.com/grids/hexagons/implementation.html#shape-rectangle
pub fn desired_hexes(overlay: &OverlayParams) -> HashSet<HexCoord> {
    let stagger = overlay.offset_variant.stagger();
    let phase = overlay.offset_variant.phase();
    let mut desired = HashSet::new();
    // The long rows (one fixed parity) define the map's world-x envelope. World-x
    // of a hex is `q + (r+phase)*stagger` (see `hex_world_pos`); the rectangle
    // trick puts every row's leftmost hex in `[0, 1)`. Short rows nest inside the
    // envelope by half a hex on each side — symmetric, because a pointy-top short
    // row sits exactly half a hex offset from the long rows it lies between.
    let long_row_r = if overlay.long_rows_even { 0 } else { 1 };
    let long_off = (long_row_r as f32 + phase) * stagger;
    let envelope_left = (-long_off).ceil() + long_off; // world-x of long-row left edge ∈ [0, 1)

    for r in 0..overlay.height {
        let q_off = (r as f32 + phase) * stagger;
        // Left edge of the rectangle row in axial space.
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
                    // `width - 1` hexes nested half a hex inside each end of the
                    // envelope (the only symmetric short-row fit on a staggered
                    // pointy-top grid).
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

/// Clip `game_map.hexes` so it contains exactly the hex set implied by the
/// current overlay parameters, minus any editor-marked exclusions. Existing
/// terrain / name data is preserved.
pub fn clip_hexes_to_overlay(game_map: &mut GameMap) {
    let mut desired = desired_hexes(&game_map.overlay);
    // Editor-time exclusions are inside the grid but not part of the map.
    desired.retain(|coord| !game_map.excluded.contains(coord));
    game_map.hexes.retain(|coord, _| desired.contains(coord));
    for coord in &desired {
        game_map
            .hexes
            .entry(*coord)
            .or_insert(HexData::new(Terrain::Desert, None));
    }
}

// ── Parse / save ─────────────────────────────────────────────────────────

/// Populate a [`GameMap`] from one board's [`MapData`]: load its tiles,
/// hexsides, and overlay, then clip the hex set to the overlay window.
pub fn load_map_data(map: &MapData, game_map: &mut GameMap) {
    game_map.hexes.clear();
    for ((q, r), tile) in &map.tiles {
        let mut hex = HexData::with_flow(tile.terrain, tile.name.clone(), tile.nile_flow);
        hex.road = tile.road;
        game_map.hexes.insert(HexCoord::new(*q, *r), hex);
    }
    game_map.hexsides = map
        .hexsides
        .iter()
        .map(|(edge, kind)| (*edge, *kind))
        .collect();
    game_map.excluded = map
        .excluded
        .iter()
        .map(|(q, r)| HexCoord::new(*q, *r))
        .collect();
    game_map.overlay = map.overlay.clone();
    // Overlay defines the map shape: clip hexes to the active overlay window
    // (minus exclusions) so any tiles outside / excluded are discarded.
    clip_hexes_to_overlay(game_map);
    info!(
        "loaded {} hexes for {} board",
        game_map.hexes.len(),
        map.image
    );
}

/// Parse an [`AnnotationsFile`] from a RON string, populate [`GameMap`] from the
/// given board, and return the full struct (caller should keep it to access the
/// sprites and the other board's data). The hex set is clipped to overlay
/// params.
pub fn load_annotations_from_str(
    ron_str: &str,
    kind: MapKind,
    game_map: &mut GameMap,
) -> AnnotationsFile {
    let annotations: AnnotationsFile = ron::from_str(ron_str).unwrap_or_else(|e| {
        warn!("failed to parse annotations.ron: {e}, using empty");
        AnnotationsFile::empty()
    });
    load_map_data(annotations.map(kind), game_map);
    annotations
}

/// Build a fresh [`MapData`] for the active board from the live [`GameMap`] and
/// sprite annotations, preserving the board's image/dims/calibration from the
/// previous value.
fn map_data_from_game_map(
    game_map: &GameMap,
    sprites: &SpriteAnnotations,
    previous: &MapData,
) -> MapData {
    let tiles: std::collections::BTreeMap<(i32, i32), TileInfo> = game_map
        .hexes
        .iter()
        .map(|(coord, data)| {
            (
                (coord.q, coord.r),
                TileInfo {
                    terrain: data.terrain,
                    name: data.name.clone(),
                    // Only persist a flow annotation on Nile hexes; drop any
                    // stale current left on non-Nile terrain.
                    nile_flow: data.nile_flow.filter(|_| data.terrain.is_nile()),
                    road: data.road,
                },
            )
        })
        .collect();
    let mut hexsides: Vec<(HexsideRef, HexsideKind)> =
        game_map.hexsides.iter().map(|(e, k)| (*e, *k)).collect();
    // Stable order so the saved file is deterministic.
    hexsides.sort_by_key(|(e, _)| (e.a.q, e.a.r, e.b.q, e.b.r));
    MapData {
        tiles,
        hexsides,
        excluded: game_map.excluded.iter().map(|c| (c.q, c.r)).collect(),
        overlay: game_map.overlay.clone(),
        sprites: sprites.clone(),
        img_w: previous.img_w,
        img_h: previous.img_h,
        image: previous.image.clone(),
        calib: previous.calib.clone(),
    }
}

/// Persist the whole two-board file, updating only the active board's section
/// from the live [`GameMap`]/sprites and leaving the other board untouched.
///
/// `file` is the in-memory [`AnnotationsFile`] (which holds the other board's
/// data); `active` selects which board the live map/sprites belong to.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_annotations_to_file(
    game_map: &GameMap,
    sprite_annotations: &SpriteAnnotations,
    file: &AnnotationsFile,
    active: MapKind,
    path: &str,
) {
    let mut out = file.clone();
    *out.map_mut(active) = map_data_from_game_map(game_map, sprite_annotations, file.map(active));
    let ron_str = ron::ser::to_string_pretty(&out, ron::ser::PrettyConfig::default())
        .expect("AnnotationsFile is always serializable");
    match std::fs::write(path, ron_str) {
        Ok(()) => info!(
            "saved {} hexes ({active} board) to {path}",
            game_map.hexes.len()
        ),
        Err(e) => error!("failed to save annotations.ron: {e}"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::GridShape;

    fn alternating_overlay(width: i32, height: i32, long_rows_even: bool) -> OverlayParams {
        OverlayParams {
            width,
            height,
            shape: GridShape::AlternatingRows,
            long_rows_even,
            ..OverlayParams::default()
        }
    }

    /// Count hexes per offset row `r` in the generated set.
    fn row_counts(hexes: &HashSet<HexCoord>, height: i32) -> Vec<usize> {
        (0..height)
            .map(|r| hexes.iter().filter(|c| c.r == r).count())
            .collect()
    }

    #[test]
    fn alternating_rows_long_even() {
        // width 6 → long rows have 6, short rows have 5 (nested ½-hex each side).
        let overlay = alternating_overlay(6, 4, true);
        let hexes = desired_hexes(&overlay);
        assert_eq!(row_counts(&hexes, 4), vec![6, 5, 6, 5]);
    }

    /// A short row must nest SYMMETRICALLY inside the long-row envelope — inset
    /// half a hex on each side. Regresses the bug where the right edge was one
    /// hex too short (insets were 0.5 vs 1.5) before short rows were nested.
    #[test]
    fn alternating_short_rows_symmetric_inset() {
        let overlay = alternating_overlay(8, 4, true);
        let hexes = desired_hexes(&overlay);
        let stagger = overlay.offset_variant.stagger();
        let phase = overlay.offset_variant.phase();
        // World-x column of a hex (matches `hex_world_pos`'s q-term).
        let wx = |c: &HexCoord| c.q as f32 + (c.r as f32 + phase) * stagger;

        // Long rows define the map's left/right world-x bounds.
        let long_left = hexes.iter().map(wx).fold(f32::INFINITY, f32::min);
        let long_right = hexes.iter().map(wx).fold(f32::NEG_INFINITY, f32::max);

        for r in 0..overlay.height {
            let row_is_even = r % 2 == 0;
            if row_is_even == overlay.long_rows_even {
                continue; // long row
            }
            let row: Vec<f32> = hexes.iter().filter(|c| c.r == r).map(wx).collect();
            let left = row.iter().cloned().fold(f32::INFINITY, f32::min);
            let right = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let left_inset = left - long_left;
            let right_inset = long_right - right;
            // The actual bug: insets must be EQUAL on both ends (was 0.5 vs 1.5).
            assert!(
                (left_inset - right_inset).abs() < 1e-3,
                "row {r}: left inset {left_inset} != right inset {right_inset}"
            );
            // Nested rows sit half a hex inside each end of the envelope.
            assert!(
                (left_inset - 0.5).abs() < 1e-3,
                "row {r}: short row should nest ½ hex per side, got {left_inset}"
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

        // Exclude one in-grid hex; it must drop out and not be re-added.
        let victim = *game_map.hexes.keys().next().unwrap();
        game_map.excluded.insert(victim);
        clip_hexes_to_overlay(&mut game_map);
        assert_eq!(game_map.hexes.len(), full - 1);
        assert!(!game_map.hexes.contains_key(&victim));

        // Re-include it; it comes back as fresh Desert.
        game_map.excluded.remove(&victim);
        clip_hexes_to_overlay(&mut game_map);
        assert_eq!(game_map.hexes.len(), full);
        assert_eq!(game_map.hexes[&victim].terrain, Terrain::Desert);
    }
}

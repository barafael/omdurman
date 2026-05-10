/// Hard-coded map data derived from the annotation wizard session (2026-05-10).
///
/// Calibration: Austrian Mission pixel (736, 420) → axial (0, 0);
///              Barracks         pixel (1178, 572) → axial (5, -1).
/// hex_size ≈ 58.72 world units (= pixels at 1:1 scale).
use super::{GameMap, HexCoord, HexData, Location, Terrain};

// ── Cross-reference points (annotation mode, 2026-05-10) ──────────────────────
///
/// All confirmed < 5 px from their predicted hex centre using the current
/// calibration — useful for validating any future recalibration.
pub const CROSS_REFS: &[(HexCoord, (f32, f32))] = &[
    (HexCoord::new( 0,  1), ( 735.0,  523.0)),  // err (1, 1)  from (736, 522)
    (HexCoord::new( 0,  2), ( 736.0,  625.0)),  // err (0, 2)  from (736, 623)
    (HexCoord::new( 2,  0), ( 913.0,  523.0)),  // err (1, 1)  from (912, 522)
    (HexCoord::new( 2,  1), ( 913.0,  625.0)),  // err (1, 2)  from (912, 623)
    (HexCoord::new( 9, -8), (1532.0,   66.0)),  // Hogali area
    (HexCoord::new(-5,  5), ( 292.0,  677.0)),  // SW desert
    (HexCoord::new(-6,  2), ( 205.0,  320.0)),  // NW desert
    (HexCoord::new( 0,  7), ( 734.0, 1132.0)),  // far south, approx Kalakla line
    (HexCoord::new( 1,  6), ( 823.0, 1081.0)),  // south-central
    (HexCoord::new( 2,  6), ( 912.0, 1132.0)),  // south-central
];

// ── Named locations ───────────────────────────────────────────────────────────

/// Every annotated (hex, location) pair, in the order they were clicked.
pub const LOCATIONS: &[(HexCoord, Location)] = &[
    (HexCoord::new(-4, -2), Location::FortMakran),
    (HexCoord::new( 9, -7), Location::NorthFort),
    (HexCoord::new( 9, -2), Location::FortBuri),
    (HexCoord::new( 2, -1), Location::Palace),
    (HexCoord::new( 4, -1), Location::Arsenal),
    (HexCoord::new( 0,  0), Location::AustrianMission),
    (HexCoord::new( 5, -1), Location::Barracks),
    (HexCoord::new( 3,  4), Location::KalaklaGate),
    (HexCoord::new( 5,  2), Location::MessalamiaGate),
    (HexCoord::new( 9, -1), Location::BuriGate),
    (HexCoord::new( 2, -5), Location::Tuti),
    (HexCoord::new( 9, -8), Location::Hogali),
    // BuriSettlement shares the Fort Buri hex — noted but not duplicated.
];

// ── Map builder ───────────────────────────────────────────────────────────────

/// Populate a `GameMap` with everything we know so far.
/// Terrain for most hexes is still `Desert`; a full terrain pass comes later.
pub fn build(map: &mut GameMap) {
    for &(coord, location) in LOCATIONS {
        let terrain = terrain_for(location);
        map.hexes.insert(coord, HexData { terrain, location: Some(location), name: None });
    }
}

/// Best-guess terrain for each named location based on the map image.
fn terrain_for(loc: Location) -> Terrain {
    match loc {
        Location::FortMakran
        | Location::NorthFort
        | Location::FortBuri
        | Location::KalaklaGate
        | Location::MessalamiaGate
        | Location::BuriGate      => Terrain::Desert,

        Location::AustrianMission
        | Location::Palace
        | Location::Arsenal
        | Location::Barracks      => Terrain::City,

        Location::Tuti            => Terrain::Palm,
        Location::Hogali          => Terrain::Settlement,
        Location::BuriSettlement  => Terrain::Settlement,
    }
}

// Compiled `MapData` for each board.
// Used at startup and for host distribution.

#![allow(unused_imports)]

use std::collections::{BTreeMap, BTreeSet};
use omdurman_types::{CalibAnchors, CampaignTurnTrack, GridShape, HexData, HexDirection, HexsideKind, HexsideRef, HexCoord, MapData, NamedArea, OffsetVariant, Orientation, OverlayParams, Road, SetupLetter, Terrain};

pub fn campaign_map_data() -> MapData {
    let tiles = {
        let mut _m = BTreeMap::new();
        _m.insert((2, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((3, 3), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((3, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((3, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((4, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((4, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((4, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((4, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((4, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((5, 4), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((5, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((5, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((6, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((6, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((6, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((6, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 2), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((7, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((7, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((7, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((7, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 2), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((9, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((9, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((9, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 2), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((10, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((10, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((10, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((10, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 2), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((11, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((11, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((11, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((11, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((11, 21), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((12, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 2), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((12, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((12, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((12, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((12, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((12, 21), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((12, 22), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((12, 23), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((13, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((13, 21), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((13, 22), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((13, 23), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((13, 24), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((13, 25), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((14, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 21), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((14, 22), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((14, 23), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((14, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 25), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((14, 26), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((14, 27), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((15, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((16, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((16, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((16, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((17, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((17, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((17, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((17, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((17, 9), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((17, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((18, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((18, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((18, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((18, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((19, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((19, 5), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((19, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((19, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((19, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((20, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((20, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((20, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((20, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((20, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((20, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((21, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((21, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((21, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((21, 7), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((21, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((21, 9), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((21, 10), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((21, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 40), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((22, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((22, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((22, 6), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((22, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((22, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((22, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 10), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((22, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 12), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((22, 13), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((22, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 40), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 42), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 9), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 10), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 13), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 14), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((23, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 40), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 42), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 44), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 8), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((24, 9), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((24, 10), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 11), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 12), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 13), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 14), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((24, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 40), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 42), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 44), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 46), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 5), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((25, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 9), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 10), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 11), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 12), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 14), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 15), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((25, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 40), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 42), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 44), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 46), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 5), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((26, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 9), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 10), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 11), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 12), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 15), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 16), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((26, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((26, 36), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((26, 37), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((26, 38), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((26, 39), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((26, 40), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((26, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 42), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 44), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 46), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((26, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 0), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((27, 1), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((27, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 9), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 10), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 11), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 12), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 16), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 17), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((27, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((27, 36), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((27, 37), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((27, 38), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((27, 39), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((27, 40), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((27, 41), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((27, 42), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((27, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 44), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 46), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((27, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 0), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((28, 1), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((28, 2), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((28, 3), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 9), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 10), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 11), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 12), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 16), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 17), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 18), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 19), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((28, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 33), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((28, 34), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((28, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((28, 36), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((28, 37), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((28, 38), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((28, 39), HexData::new(Terrain::Building { road: Road::None }, Some("Palace".to_string())));
        _m.insert((28, 40), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((28, 41), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((28, 42), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((28, 43), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((28, 44), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((28, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 46), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((28, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 0), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((29, 1), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((29, 2), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((29, 3), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((29, 4), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 5), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 6), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 7), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 8), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 16), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 17), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 18), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 19), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((29, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((29, 33), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((29, 34), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((29, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((29, 36), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((29, 37), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((29, 38), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((29, 39), HexData::new(Terrain::Building { road: Road::None }, Some("Grounds".to_string())));
        _m.insert((29, 40), HexData::new(Terrain::Clear { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((29, 41), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((29, 42), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((29, 43), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((29, 44), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((29, 45), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((29, 46), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((29, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 1), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((30, 2), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((30, 3), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((30, 4), HexData::new(Terrain::Huts { road: Road::None }, Some("Kerreri".to_string())));
        _m.insert((30, 5), HexData::new(Terrain::Huts { road: Road::None }, Some("Kerreri".to_string())));
        _m.insert((30, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 16), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((30, 17), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((30, 18), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((30, 19), HexData::new(Terrain::Hilltop { road: Road::None }, None));
        _m.insert((30, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((30, 21), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((30, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((30, 33), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((30, 34), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((30, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((30, 36), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((30, 37), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((30, 38), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((30, 39), HexData::new(Terrain::Building { road: Road::None }, Some("Mahdi's Tomb".to_string())));
        _m.insert((30, 40), HexData::new(Terrain::Clear { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((30, 41), HexData::new(Terrain::Building { road: Road::None }, Some("Arsenal".to_string())));
        _m.insert((30, 42), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((30, 43), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((30, 44), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((30, 45), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((30, 46), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((30, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 1), HexData { terrain: Terrain::Clear { road: Road::None }, location: None, name: None, setup_letter: Some(SetupLetter::A), is_scattergram: false, named_area: None });
        _m.insert((31, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 3), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((31, 4), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((31, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 11), HexData { terrain: Terrain::Clear { road: Road::None }, location: None, name: None, setup_letter: Some(SetupLetter::D), is_scattergram: false, named_area: None });
        _m.insert((31, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 17), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((31, 18), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((31, 19), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((31, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((31, 21), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((31, 22), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((31, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((31, 32), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((31, 33), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((31, 34), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((31, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((31, 36), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((31, 37), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((31, 38), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((31, 39), HexData::new(Terrain::Clear { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((31, 40), HexData::new(Terrain::Clear { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((31, 41), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((31, 42), HexData::new(Terrain::Clear { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((31, 43), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((31, 44), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((31, 45), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((31, 46), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((31, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 4), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((32, 5), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((32, 6), HexData { terrain: Terrain::Clear { road: Road::None }, location: None, name: None, setup_letter: Some(SetupLetter::Y), is_scattergram: false, named_area: None });
        _m.insert((32, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 11), HexData { terrain: Terrain::Clear { road: Road::None }, location: None, name: None, setup_letter: Some(SetupLetter::K), is_scattergram: false, named_area: None });
        _m.insert((32, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 17), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((32, 18), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((32, 19), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((32, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((32, 21), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((32, 22), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((32, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 32), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((32, 33), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((32, 34), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((32, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((32, 36), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((32, 37), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((32, 38), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((32, 39), HexData::new(Terrain::Clear { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((32, 40), HexData::new(Terrain::Clear { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((32, 41), HexData::new(Terrain::Clear { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((32, 42), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((32, 43), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((32, 44), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((32, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 46), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((32, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 0), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((33, 1), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((33, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 5), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((33, 6), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((33, 7), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((33, 8), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((33, 9), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((33, 10), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((33, 11), HexData { terrain: Terrain::Clear { road: Road::None }, location: None, name: None, setup_letter: Some(SetupLetter::S), is_scattergram: false, named_area: None });
        _m.insert((33, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 14), HexData::new(Terrain::Building { road: Road::None }, Some("Zariba".to_string())));
        _m.insert((33, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 17), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((33, 18), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((33, 19), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((33, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((33, 21), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((33, 22), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((33, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 31), HexData::new(Terrain::Clear { road: Road::Crossroad }, None));
        _m.insert((33, 32), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((33, 33), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((33, 34), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((33, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((33, 36), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((33, 37), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((33, 38), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((33, 39), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((33, 40), HexData::new(Terrain::Building { road: Road::None }, Some("Treasury".to_string())));
        _m.insert((33, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 42), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 44), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 46), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((33, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 0), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((34, 1), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((34, 2), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((34, 3), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 6), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((34, 7), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 8), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 9), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 10), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 11), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 12), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 18), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((34, 19), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((34, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((34, 21), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((34, 22), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((34, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 32), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((34, 33), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((34, 34), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((34, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((34, 36), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((34, 37), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((34, 38), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((34, 39), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((34, 40), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((34, 42), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((34, 43), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 44), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 45), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 46), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((34, 47), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((35, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 2), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((35, 3), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((35, 4), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((35, 5), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((35, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 11), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((35, 12), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((35, 13), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((35, 14), HexData::new(Terrain::Huts { road: Road::None }, Some("El Egeiga".to_string())));
        _m.insert((35, 15), HexData::new(Terrain::Huts { road: Road::None }, Some("El Egeiga".to_string())));
        _m.insert((35, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 19), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((35, 20), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((35, 21), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((35, 22), HexData::new(Terrain::Rough { road: Road::None }, None));
        _m.insert((35, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 32), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((35, 33), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((35, 34), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((35, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((35, 36), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((35, 37), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((35, 38), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((35, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 40), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((35, 41), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((35, 42), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((35, 43), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((35, 44), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((35, 45), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((35, 46), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((35, 47), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((36, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 4), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((36, 5), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((36, 6), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((36, 7), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((36, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 13), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((36, 14), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((36, 15), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((36, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 32), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((36, 33), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((36, 34), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((36, 35), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Omdurman".to_string())));
        _m.insert((36, 36), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((36, 37), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((36, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((36, 40), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((36, 41), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((36, 42), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((36, 43), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((36, 44), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((36, 45), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((36, 46), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((36, 47), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((37, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 6), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((37, 7), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((37, 8), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((37, 9), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((37, 10), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((37, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 14), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((37, 15), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((37, 16), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((37, 17), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((37, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 32), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((37, 33), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((37, 34), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((37, 35), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((37, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((37, 40), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((37, 41), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((37, 42), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((37, 43), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((37, 44), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((37, 45), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((37, 46), HexData::new(Terrain::Clear { road: Road::None }, Some("Makran Point".to_string())));
        _m.insert((37, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 8), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((38, 9), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 10), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 11), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 12), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 13), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 14), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 15), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 16), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((38, 17), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((38, 18), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((38, 19), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 31), HexData::new(Terrain::Huts { road: Road::None }, Some("Shambat".to_string())));
        _m.insert((38, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 33), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((38, 34), HexData::new(Terrain::Building { road: Road::None }, Some("Omdurman".to_string())));
        _m.insert((38, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((38, 40), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((38, 41), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((38, 42), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 43), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((38, 44), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((38, 45), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((38, 46), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((38, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 11), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((39, 12), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 13), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 14), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 15), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 16), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 17), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 18), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 19), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 20), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((39, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 28), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 31), HexData::new(Terrain::Huts { road: Road::None }, Some("Shambat".to_string())));
        _m.insert((39, 32), HexData::new(Terrain::Huts { road: Road::None }, Some("Shambat".to_string())));
        _m.insert((39, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((39, 36), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((39, 37), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 38), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 39), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 40), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 41), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((39, 42), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((39, 43), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((39, 44), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((39, 45), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((39, 46), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((39, 47), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((40, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((40, 12), HexData { terrain: Terrain::Huts { road: Road::None }, location: None, name: Some("Abu Alim".to_string()), setup_letter: Some(SetupLetter::K), is_scattergram: false, named_area: None });
        _m.insert((40, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((40, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((40, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((40, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((40, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((40, 18), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((40, 19), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((40, 20), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((40, 21), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((40, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((40, 23), HexData::new(Terrain::Huts { road: Road::None }, None));
        _m.insert((40, 24), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((40, 25), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((40, 26), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 27), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 28), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 29), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 30), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 31), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 32), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 33), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 34), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 35), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 36), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 37), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 38), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 39), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 40), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 41), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((40, 42), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((40, 43), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((40, 44), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((40, 45), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((40, 46), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((40, 47), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((41, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((41, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((41, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((41, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((41, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((41, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((41, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((41, 20), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((41, 21), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((41, 22), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((41, 23), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 24), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 25), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 26), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 27), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 28), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 29), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 30), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 31), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 32), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 33), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 34), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 35), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 36), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 37), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 38), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 39), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 40), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 41), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 42), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((41, 43), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((41, 44), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((41, 45), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((41, 46), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((41, 47), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((42, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((42, 16), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((42, 17), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((42, 18), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((42, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((42, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((42, 21), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((42, 22), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((42, 23), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 24), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 25), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 26), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 27), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 28), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 29), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 30), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 31), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 32), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 33), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 34), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((42, 35), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((42, 36), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 37), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 38), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((42, 39), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((42, 40), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((42, 41), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((42, 42), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((42, 43), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((42, 44), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((42, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((42, 46), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((42, 47), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((43, 17), HexData::new(Terrain::Huts { road: Road::None }, Some("El Debeba".to_string())));
        _m.insert((43, 18), HexData::new(Terrain::Huts { road: Road::None }, Some("El Debeba".to_string())));
        _m.insert((43, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((43, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((43, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((43, 22), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((43, 23), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((43, 24), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((43, 25), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 26), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 27), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 28), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 29), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 30), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 31), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 32), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 33), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 34), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((43, 35), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((43, 36), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 37), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 38), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 39), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((43, 40), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((43, 41), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 42), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 43), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 44), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((43, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((43, 46), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((43, 47), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 19), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((44, 20), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((44, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((44, 22), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((44, 23), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((44, 24), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((44, 25), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((44, 26), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((44, 27), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 28), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 29), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 30), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 31), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 32), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 33), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 34), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((44, 35), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((44, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((44, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((44, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((44, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((44, 40), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((44, 41), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((44, 42), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((44, 43), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((44, 44), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((44, 45), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 46), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((44, 47), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((45, 21), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((45, 22), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((45, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((45, 24), HexData::new(Terrain::Huts { road: Road::None }, Some("Halfaya".to_string())));
        _m.insert((45, 25), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((45, 26), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((45, 27), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((45, 28), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((45, 29), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((45, 30), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((45, 31), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((45, 32), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((45, 33), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((45, 34), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((45, 35), HexData::new(Terrain::Nile { direction: HexDirection::East }, None));
        _m.insert((45, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((45, 37), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((45, 38), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((45, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((45, 40), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((45, 41), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((45, 42), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((45, 43), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((45, 44), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((45, 45), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((45, 46), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((45, 47), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((46, 23), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((46, 24), HexData::new(Terrain::Huts { road: Road::None }, Some("Halfaya".to_string())));
        _m.insert((46, 25), HexData::new(Terrain::Huts { road: Road::None }, Some("Halfaya".to_string())));
        _m.insert((46, 26), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((46, 27), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((46, 28), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((46, 29), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((46, 30), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((46, 31), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((46, 32), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((46, 33), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((46, 34), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((46, 35), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((46, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((46, 39), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((46, 40), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((46, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((46, 42), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((46, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((46, 44), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((46, 45), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((46, 46), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((46, 47), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((47, 25), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((47, 26), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((47, 27), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((47, 28), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((47, 29), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((47, 30), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((47, 31), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((47, 32), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((47, 33), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((47, 34), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((47, 35), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((47, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((47, 41), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((47, 42), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((47, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((47, 44), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((47, 45), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((47, 46), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((47, 47), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((48, 27), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((48, 28), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((48, 29), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((48, 30), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((48, 31), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((48, 32), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((48, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((48, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((48, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((48, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((48, 43), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((48, 44), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((48, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((48, 46), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((48, 47), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((49, 29), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 30), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 45), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 46), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((49, 47), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((50, 31), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((50, 32), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((50, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((50, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((50, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((50, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((50, 47), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((51, 33), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((51, 34), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((51, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((51, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((52, 35), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((52, 36), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m
    };
    let hexsides = vec![
        (HexsideRef::new(HexCoord::new(2, 3), HexCoord::new(3, 3)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(3, 3), HexCoord::new(3, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(3, 3), HexCoord::new(4, 3)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(3, 3), HexCoord::new(4, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(4, 3), HexCoord::new(4, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(4, 3), HexCoord::new(5, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(4, 4), HexCoord::new(5, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(5, 3), HexCoord::new(5, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(5, 3), HexCoord::new(6, 3)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(5, 3), HexCoord::new(6, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(5, 4), HexCoord::new(5, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(5, 4), HexCoord::new(6, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(5, 4), HexCoord::new(6, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(5, 5), HexCoord::new(6, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(6, 3), HexCoord::new(7, 3)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(6, 5), HexCoord::new(6, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(7, 3), HexCoord::new(7, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(7, 4), HexCoord::new(8, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(7, 13), HexCoord::new(7, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(7, 13), HexCoord::new(8, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(8, 13), HexCoord::new(8, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(8, 13), HexCoord::new(9, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(9, 3), HexCoord::new(10, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(9, 4), HexCoord::new(10, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(9, 4), HexCoord::new(10, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(9, 13), HexCoord::new(9, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(9, 13), HexCoord::new(10, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(10, 2), HexCoord::new(11, 2)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(10, 2), HexCoord::new(11, 3)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(10, 3), HexCoord::new(10, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(10, 3), HexCoord::new(11, 3)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(10, 3), HexCoord::new(11, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(10, 13), HexCoord::new(10, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(10, 13), HexCoord::new(11, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(11, 13), HexCoord::new(11, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(11, 14), HexCoord::new(12, 14)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(11, 21), HexCoord::new(12, 22)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(11, 22), HexCoord::new(12, 22)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(12, 14), HexCoord::new(12, 15)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(12, 14), HexCoord::new(13, 15)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(12, 20), HexCoord::new(13, 21)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(12, 21), HexCoord::new(12, 22)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(12, 21), HexCoord::new(13, 21)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(12, 21), HexCoord::new(13, 22)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(12, 22), HexCoord::new(12, 23)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(12, 22), HexCoord::new(13, 22)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(12, 22), HexCoord::new(13, 23)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(12, 23), HexCoord::new(13, 23)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(13, 14), HexCoord::new(13, 15)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(13, 15), HexCoord::new(14, 15)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(13, 20), HexCoord::new(13, 21)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(13, 20), HexCoord::new(14, 21)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(13, 21), HexCoord::new(13, 22)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(13, 21), HexCoord::new(14, 21)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(13, 21), HexCoord::new(14, 22)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(13, 23), HexCoord::new(13, 24)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(14, 15), HexCoord::new(14, 16)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(14, 15), HexCoord::new(15, 16)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(15, 15), HexCoord::new(15, 16)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(15, 15), HexCoord::new(16, 16)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(16, 6), HexCoord::new(17, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(16, 7), HexCoord::new(17, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(16, 7), HexCoord::new(17, 8)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(16, 8), HexCoord::new(17, 8)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(16, 8), HexCoord::new(17, 9)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(16, 15), HexCoord::new(16, 16)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(16, 16), HexCoord::new(17, 16)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(17, 5), HexCoord::new(18, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(17, 6), HexCoord::new(17, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(17, 6), HexCoord::new(18, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(17, 6), HexCoord::new(18, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(17, 16), HexCoord::new(17, 17)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(17, 17), HexCoord::new(18, 17)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(18, 4), HexCoord::new(19, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(18, 4), HexCoord::new(19, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(18, 5), HexCoord::new(18, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(18, 5), HexCoord::new(19, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(18, 5), HexCoord::new(19, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(18, 17), HexCoord::new(18, 18)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(18, 18), HexCoord::new(19, 18)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(19, 4), HexCoord::new(19, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(19, 5), HexCoord::new(19, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(19, 5), HexCoord::new(20, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(19, 5), HexCoord::new(20, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(19, 18), HexCoord::new(19, 19)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(19, 19), HexCoord::new(20, 19)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(20, 3), HexCoord::new(21, 3)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(20, 3), HexCoord::new(21, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(20, 4), HexCoord::new(21, 4)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(20, 5), HexCoord::new(20, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(20, 6), HexCoord::new(21, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(20, 6), HexCoord::new(21, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(20, 7), HexCoord::new(21, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(20, 19), HexCoord::new(20, 20)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(20, 19), HexCoord::new(21, 20)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(21, 4), HexCoord::new(21, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(21, 5), HexCoord::new(22, 5)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(21, 5), HexCoord::new(22, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(21, 6), HexCoord::new(21, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(21, 6), HexCoord::new(22, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(21, 7), HexCoord::new(21, 8)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(21, 7), HexCoord::new(22, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(21, 7), HexCoord::new(22, 8)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(21, 8), HexCoord::new(22, 8)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(21, 19), HexCoord::new(21, 20)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(21, 19), HexCoord::new(22, 20)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(22, 5), HexCoord::new(22, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(22, 6), HexCoord::new(22, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(22, 6), HexCoord::new(23, 6)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(22, 6), HexCoord::new(23, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(22, 7), HexCoord::new(23, 7)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(22, 8), HexCoord::new(22, 9)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(22, 9), HexCoord::new(23, 9)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(22, 19), HexCoord::new(22, 20)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(22, 20), HexCoord::new(23, 20)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(22, 44), HexCoord::new(23, 45)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(23, 7), HexCoord::new(23, 8)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(23, 7), HexCoord::new(24, 8)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(23, 8), HexCoord::new(24, 8)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(23, 8), HexCoord::new(24, 9)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(23, 9), HexCoord::new(23, 10)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(23, 9), HexCoord::new(24, 9)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(23, 9), HexCoord::new(24, 10)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(23, 20), HexCoord::new(23, 21)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(23, 20), HexCoord::new(24, 21)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(23, 44), HexCoord::new(23, 45)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(23, 45), HexCoord::new(24, 45)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(24, 8), HexCoord::new(25, 9)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(24, 9), HexCoord::new(24, 10)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(24, 9), HexCoord::new(25, 9)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(24, 9), HexCoord::new(25, 10)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(24, 10), HexCoord::new(25, 10)), HexsideKind::Crest),
        (HexsideRef::new(HexCoord::new(24, 20), HexCoord::new(24, 21)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(24, 21), HexCoord::new(25, 21)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(24, 45), HexCoord::new(24, 46)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(24, 45), HexCoord::new(25, 46)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(25, 21), HexCoord::new(25, 22)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(25, 22), HexCoord::new(26, 22)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(25, 45), HexCoord::new(25, 46)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(25, 45), HexCoord::new(26, 46)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(26, 22), HexCoord::new(26, 23)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(26, 23), HexCoord::new(27, 23)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(26, 45), HexCoord::new(26, 46)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(26, 46), HexCoord::new(27, 46)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(27, 23), HexCoord::new(27, 24)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(27, 23), HexCoord::new(28, 24)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(27, 38), HexCoord::new(28, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(27, 39), HexCoord::new(28, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(27, 46), HexCoord::new(27, 47)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(27, 46), HexCoord::new(28, 47)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(28, 23), HexCoord::new(28, 24)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(28, 24), HexCoord::new(29, 24)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(28, 38), HexCoord::new(28, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(28, 38), HexCoord::new(29, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(28, 39), HexCoord::new(28, 40)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(28, 40), HexCoord::new(29, 40)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(28, 46), HexCoord::new(28, 47)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(28, 47), HexCoord::new(29, 47)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(29, 24), HexCoord::new(29, 25)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(29, 25), HexCoord::new(30, 25)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(29, 38), HexCoord::new(29, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(29, 38), HexCoord::new(30, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(29, 40), HexCoord::new(29, 41)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(29, 41), HexCoord::new(30, 41)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(29, 47), HexCoord::new(29, 48)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(30, 25), HexCoord::new(30, 26)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(30, 25), HexCoord::new(31, 26)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(30, 35), HexCoord::new(31, 36)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(30, 36), HexCoord::new(31, 36)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(30, 36), HexCoord::new(31, 37)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(30, 37), HexCoord::new(31, 37)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(30, 37), HexCoord::new(31, 38)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(30, 38), HexCoord::new(30, 39)), HexsideKind::Gate),
        (HexsideRef::new(HexCoord::new(30, 38), HexCoord::new(31, 38)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(30, 38), HexCoord::new(31, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(30, 41), HexCoord::new(30, 42)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(30, 42), HexCoord::new(31, 42)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(31, 25), HexCoord::new(31, 26)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(31, 26), HexCoord::new(32, 26)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(31, 35), HexCoord::new(31, 36)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(31, 36), HexCoord::new(32, 36)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(31, 42), HexCoord::new(31, 43)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(31, 43), HexCoord::new(32, 43)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(32, 10), HexCoord::new(33, 11)), HexsideKind::ZaribaTrenchEndA),
        (HexsideRef::new(HexCoord::new(32, 11), HexCoord::new(33, 11)), HexsideKind::ZaribaThornHedge),
        (HexsideRef::new(HexCoord::new(32, 11), HexCoord::new(33, 12)), HexsideKind::ZaribaThornHedge),
        (HexsideRef::new(HexCoord::new(32, 12), HexCoord::new(33, 12)), HexsideKind::ZaribaThornHedge),
        (HexsideRef::new(HexCoord::new(32, 12), HexCoord::new(33, 13)), HexsideKind::ZaribaThornHedge),
        (HexsideRef::new(HexCoord::new(32, 13), HexCoord::new(33, 13)), HexsideKind::ZaribaThornHedge),
        (HexsideRef::new(HexCoord::new(32, 13), HexCoord::new(33, 14)), HexsideKind::ZaribaThornHedge),
        (HexsideRef::new(HexCoord::new(32, 14), HexCoord::new(33, 14)), HexsideKind::ZaribaThornHedge),
        (HexsideRef::new(HexCoord::new(32, 26), HexCoord::new(32, 27)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(32, 27), HexCoord::new(33, 27)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(32, 36), HexCoord::new(32, 37)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(32, 36), HexCoord::new(33, 37)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(32, 40), HexCoord::new(33, 41)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(32, 41), HexCoord::new(33, 41)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(32, 41), HexCoord::new(33, 42)), HexsideKind::Gate),
        (HexsideRef::new(HexCoord::new(32, 42), HexCoord::new(33, 42)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(32, 42), HexCoord::new(33, 43)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(32, 43), HexCoord::new(32, 44)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(32, 43), HexCoord::new(33, 43)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(32, 43), HexCoord::new(33, 44)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(33, 14), HexCoord::new(33, 15)), HexsideKind::ZaribaThornHedge),
        (HexsideRef::new(HexCoord::new(33, 15), HexCoord::new(34, 15)), HexsideKind::ZaribaTrench),
        (HexsideRef::new(HexCoord::new(33, 27), HexCoord::new(33, 28)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(33, 27), HexCoord::new(34, 28)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(33, 36), HexCoord::new(33, 37)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(33, 37), HexCoord::new(34, 37)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(33, 39), HexCoord::new(34, 40)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(33, 40), HexCoord::new(33, 41)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(33, 40), HexCoord::new(34, 40)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(33, 40), HexCoord::new(34, 41)), HexsideKind::Gate),
        (HexsideRef::new(HexCoord::new(34, 15), HexCoord::new(34, 16)), HexsideKind::ZaribaTrench),
        (HexsideRef::new(HexCoord::new(34, 16), HexCoord::new(35, 16)), HexsideKind::ZaribaTrench),
        (HexsideRef::new(HexCoord::new(34, 27), HexCoord::new(34, 28)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(34, 28), HexCoord::new(35, 28)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(34, 31), HexCoord::new(34, 32)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(34, 32), HexCoord::new(35, 32)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(34, 37), HexCoord::new(34, 38)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(34, 37), HexCoord::new(35, 38)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(34, 38), HexCoord::new(35, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(34, 39), HexCoord::new(34, 40)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(34, 39), HexCoord::new(35, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(34, 39), HexCoord::new(35, 40)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(34, 40), HexCoord::new(35, 40)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(35, 16), HexCoord::new(35, 17)), HexsideKind::ZaribaTrench),
        (HexsideRef::new(HexCoord::new(35, 17), HexCoord::new(36, 17)), HexsideKind::ZaribaTrench),
        (HexsideRef::new(HexCoord::new(35, 28), HexCoord::new(35, 29)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(35, 29), HexCoord::new(36, 29)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(35, 32), HexCoord::new(35, 33)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(35, 33), HexCoord::new(36, 33)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(35, 37), HexCoord::new(35, 38)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(35, 38), HexCoord::new(35, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(35, 38), HexCoord::new(36, 38)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(35, 38), HexCoord::new(36, 39)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(36, 17), HexCoord::new(36, 18)), HexsideKind::ZaribaTrench),
        (HexsideRef::new(HexCoord::new(36, 18), HexCoord::new(37, 18)), HexsideKind::ZaribaTrench),
        (HexsideRef::new(HexCoord::new(36, 29), HexCoord::new(36, 30)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(36, 30), HexCoord::new(37, 30)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(36, 33), HexCoord::new(36, 34)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(36, 34), HexCoord::new(37, 34)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(37, 18), HexCoord::new(37, 19)), HexsideKind::ZaribaTrenchEndB),
        (HexsideRef::new(HexCoord::new(37, 30), HexCoord::new(37, 31)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(37, 31), HexCoord::new(38, 31)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(37, 34), HexCoord::new(37, 35)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(37, 35), HexCoord::new(38, 35)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(38, 31), HexCoord::new(38, 32)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(38, 32), HexCoord::new(39, 32)), HexsideKind::KhorShambat),
        (HexsideRef::new(HexCoord::new(38, 35), HexCoord::new(38, 36)), HexsideKind::Khor),
        (HexsideRef::new(HexCoord::new(39, 32), HexCoord::new(39, 33)), HexsideKind::KhorShambat),
    ];
    let roads = vec![
        HexsideRef::new(HexCoord::new(26, 35), HexCoord::new(26, 36)),
        HexsideRef::new(HexCoord::new(26, 35), HexCoord::new(27, 35)),
        HexsideRef::new(HexCoord::new(26, 36), HexCoord::new(26, 37)),
        HexsideRef::new(HexCoord::new(26, 36), HexCoord::new(27, 37)),
        HexsideRef::new(HexCoord::new(26, 37), HexCoord::new(26, 38)),
        HexsideRef::new(HexCoord::new(26, 38), HexCoord::new(26, 39)),
        HexsideRef::new(HexCoord::new(26, 38), HexCoord::new(27, 39)),
        HexsideRef::new(HexCoord::new(26, 39), HexCoord::new(27, 40)),
        HexsideRef::new(HexCoord::new(27, 35), HexCoord::new(28, 35)),
        HexsideRef::new(HexCoord::new(27, 35), HexCoord::new(28, 36)),
        HexsideRef::new(HexCoord::new(27, 37), HexCoord::new(28, 38)),
        HexsideRef::new(HexCoord::new(27, 39), HexCoord::new(28, 40)),
        HexsideRef::new(HexCoord::new(27, 40), HexCoord::new(27, 41)),
        HexsideRef::new(HexCoord::new(27, 41), HexCoord::new(28, 42)),
        HexsideRef::new(HexCoord::new(28, 32), HexCoord::new(29, 33)),
        HexsideRef::new(HexCoord::new(28, 35), HexCoord::new(29, 35)),
        HexsideRef::new(HexCoord::new(28, 36), HexCoord::new(29, 37)),
        HexsideRef::new(HexCoord::new(28, 40), HexCoord::new(29, 41)),
        HexsideRef::new(HexCoord::new(28, 42), HexCoord::new(28, 43)),
        HexsideRef::new(HexCoord::new(28, 43), HexCoord::new(29, 44)),
        HexsideRef::new(HexCoord::new(29, 33), HexCoord::new(29, 34)),
        HexsideRef::new(HexCoord::new(29, 34), HexCoord::new(30, 35)),
        HexsideRef::new(HexCoord::new(29, 35), HexCoord::new(30, 35)),
        HexsideRef::new(HexCoord::new(29, 37), HexCoord::new(30, 38)),
        HexsideRef::new(HexCoord::new(29, 39), HexCoord::new(30, 40)),
        HexsideRef::new(HexCoord::new(29, 41), HexCoord::new(30, 42)),
        HexsideRef::new(HexCoord::new(29, 44), HexCoord::new(29, 45)),
        HexsideRef::new(HexCoord::new(29, 44), HexCoord::new(30, 45)),
        HexsideRef::new(HexCoord::new(29, 45), HexCoord::new(29, 46)),
        HexsideRef::new(HexCoord::new(30, 34), HexCoord::new(30, 35)),
        HexsideRef::new(HexCoord::new(30, 34), HexCoord::new(31, 34)),
        HexsideRef::new(HexCoord::new(30, 35), HexCoord::new(30, 36)),
        HexsideRef::new(HexCoord::new(30, 35), HexCoord::new(31, 35)),
        HexsideRef::new(HexCoord::new(30, 36), HexCoord::new(30, 37)),
        HexsideRef::new(HexCoord::new(30, 37), HexCoord::new(30, 38)),
        HexsideRef::new(HexCoord::new(30, 38), HexCoord::new(30, 39)),
        HexsideRef::new(HexCoord::new(30, 39), HexCoord::new(31, 40)),
        HexsideRef::new(HexCoord::new(30, 40), HexCoord::new(30, 41)),
        HexsideRef::new(HexCoord::new(30, 40), HexCoord::new(31, 41)),
        HexsideRef::new(HexCoord::new(30, 42), HexCoord::new(31, 43)),
        HexsideRef::new(HexCoord::new(30, 45), HexCoord::new(31, 46)),
        HexsideRef::new(HexCoord::new(31, 33), HexCoord::new(31, 34)),
        HexsideRef::new(HexCoord::new(31, 33), HexCoord::new(32, 33)),
        HexsideRef::new(HexCoord::new(31, 35), HexCoord::new(32, 35)),
        HexsideRef::new(HexCoord::new(31, 35), HexCoord::new(32, 36)),
        HexsideRef::new(HexCoord::new(31, 38), HexCoord::new(32, 38)),
        HexsideRef::new(HexCoord::new(31, 40), HexCoord::new(32, 41)),
        HexsideRef::new(HexCoord::new(31, 41), HexCoord::new(31, 42)),
        HexsideRef::new(HexCoord::new(31, 41), HexCoord::new(32, 41)),
        HexsideRef::new(HexCoord::new(31, 43), HexCoord::new(32, 44)),
        HexsideRef::new(HexCoord::new(32, 32), HexCoord::new(32, 33)),
        HexsideRef::new(HexCoord::new(32, 32), HexCoord::new(33, 32)),
        HexsideRef::new(HexCoord::new(32, 35), HexCoord::new(33, 35)),
        HexsideRef::new(HexCoord::new(32, 36), HexCoord::new(33, 36)),
        HexsideRef::new(HexCoord::new(32, 37), HexCoord::new(32, 38)),
        HexsideRef::new(HexCoord::new(32, 38), HexCoord::new(32, 39)),
        HexsideRef::new(HexCoord::new(32, 38), HexCoord::new(33, 38)),
        HexsideRef::new(HexCoord::new(32, 39), HexCoord::new(32, 40)),
        HexsideRef::new(HexCoord::new(32, 40), HexCoord::new(32, 41)),
        HexsideRef::new(HexCoord::new(32, 40), HexCoord::new(33, 40)),
        HexsideRef::new(HexCoord::new(32, 41), HexCoord::new(33, 42)),
        HexsideRef::new(HexCoord::new(33, 31), HexCoord::new(33, 32)),
        HexsideRef::new(HexCoord::new(33, 31), HexCoord::new(34, 31)),
        HexsideRef::new(HexCoord::new(33, 35), HexCoord::new(34, 35)),
        HexsideRef::new(HexCoord::new(33, 36), HexCoord::new(34, 37)),
        HexsideRef::new(HexCoord::new(33, 38), HexCoord::new(34, 38)),
        HexsideRef::new(HexCoord::new(34, 35), HexCoord::new(35, 35)),
        HexsideRef::new(HexCoord::new(34, 37), HexCoord::new(35, 37)),
        HexsideRef::new(HexCoord::new(34, 38), HexCoord::new(35, 38)),
        HexsideRef::new(HexCoord::new(35, 35), HexCoord::new(36, 35)),
        HexsideRef::new(HexCoord::new(35, 37), HexCoord::new(36, 38)),
        HexsideRef::new(HexCoord::new(36, 35), HexCoord::new(37, 35)),
    ];
    let excluded: BTreeSet<(i32, i32)> = BTreeSet::from_iter(vec![
        (1, 0),
        (1, 1),
        (2, 0),
        (2, 1),
        (2, 2),
        (3, 0),
        (3, 1),
        (3, 2),
        (4, 0),
        (4, 1),
        (4, 2),
        (5, 0),
        (5, 1),
        (5, 2),
        (6, 0),
        (6, 1),
        (6, 2),
        (7, 0),
        (7, 1),
        (8, 0),
        (8, 1),
        (9, 0),
        (9, 1),
        (10, 0),
        (10, 1),
        (11, 0),
        (11, 1),
        (46, 37),
        (46, 38),
        (47, 37),
        (47, 38),
        (47, 39),
        (47, 40),
        (48, 37),
        (48, 38),
        (48, 39),
        (48, 40),
        (48, 41),
        (48, 42),
        (49, 37),
        (49, 38),
        (49, 39),
        (49, 40),
        (49, 41),
        (49, 42),
        (49, 43),
        (49, 44),
        (50, 37),
        (50, 38),
        (50, 39),
        (50, 40),
        (50, 41),
        (50, 42),
        (50, 43),
        (50, 44),
        (50, 45),
        (50, 46),
        (51, 37),
        (51, 38),
        (51, 39),
        (51, 40),
        (51, 41),
        (51, 42),
        (51, 43),
        (51, 44),
        (51, 45),
        (51, 46),
        (51, 47),
        (52, 37),
        (52, 38),
        (52, 39),
        (52, 40),
        (52, 41),
        (52, 42),
        (52, 43),
        (52, 44),
        (52, 45),
        (52, 46),
        (52, 47),
        (53, 37),
        (53, 38),
        (53, 39),
        (53, 40),
        (53, 41),
        (53, 42),
        (53, 43),
        (53, 44),
        (53, 45),
        (53, 46),
        (53, 47),
        (54, 39),
        (54, 40),
        (54, 41),
        (54, 42),
        (54, 43),
        (54, 44),
        (54, 45),
        (54, 46),
        (54, 47),
        (55, 41),
        (55, 42),
        (55, 43),
        (55, 44),
        (55, 45),
        (55, 46),
        (55, 47),
        (56, 43),
        (56, 44),
        (56, 45),
        (56, 46),
        (56, 47),
        (57, 45),
        (57, 46),
        (57, 47),
        (58, 47),
    ]);
    let overlay = OverlayParams { width: 35, height: 48, hex_size: 53.35, offset_x: 59.0, offset_y: 305.0, orientation: Orientation::Pointy, offset_variant: OffsetVariant::OddR, shape: GridShape::AlternatingRows, long_rows_even: false, rotation_deg: 0.42, aspect_y: 1.0, shear_x: 0.0054, shear_y: -0.0036, size_grad_x: 0.0, size_grad_y: 0.0 };
    MapData { tiles, hexsides, roads, excluded, overlay, img_w: 3258.0, img_h: 4124.0, image: "campaign_map.webp".to_string(), calib: CalibAnchors { p1_px: (0.0, 0.0), p1_hex: (0, 0), p2_px: (100.0, 100.0), p2_hex: (5, -1) }, campaign_turn_track: Some(CampaignTurnTrack { x: 16.0, y: 268.0, w: 999.0, h: 222.0 }) }
}

pub fn fall_of_khartoum_map_data() -> MapData {
    let tiles = {
        let mut _m = BTreeMap::new();
        _m.insert((1, 0), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, Some("White Nile Mouth".to_string())));
        _m.insert((1, 1), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((2, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((2, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((2, 2), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((2, 3), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((3, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((3, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((3, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((3, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((3, 4), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((3, 5), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((4, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((4, 1), HexData::new(Terrain::Building { road: Road::None }, Some("Fort Makran".to_string())));
        _m.insert((4, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((4, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((4, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((4, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((4, 6), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((4, 7), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((5, 0), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((5, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((5, 7), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((5, 8), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((5, 9), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((6, 0), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((6, 1), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((6, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((6, 9), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((6, 10), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((6, 11), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((7, 0), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((7, 1), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((7, 2), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((7, 3), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((7, 11), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((7, 12), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((7, 13), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((8, 0), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((8, 1), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((8, 2), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((8, 3), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((8, 4), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((8, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((8, 13), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((8, 14), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((8, 15), HexData::new(Terrain::Nile { direction: HexDirection::NorthEast }, None));
        _m.insert((9, 0), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((9, 1), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((9, 2), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((9, 3), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((9, 4), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((9, 5), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((9, 6), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((9, 15), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((10, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 1), HexData::new(Terrain::Swamp { road: Road::None }, None));
        _m.insert((10, 2), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((10, 3), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((10, 4), HexData::new(Terrain::Building { road: Road::None }, Some("Khartoum".to_string())));
        _m.insert((10, 5), HexData::new(Terrain::Building { road: Road::None }, Some("Khartoum".to_string())));
        _m.insert((10, 6), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((10, 7), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((10, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((10, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 0), HexData::new(Terrain::Huts { road: Road::None }, Some("Tuti".to_string())));
        _m.insert((11, 1), HexData::new(Terrain::Huts { road: Road::None }, Some("Tuti".to_string())));
        _m.insert((11, 2), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((11, 3), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((11, 4), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((11, 5), HexData::new(Terrain::Building { road: Road::None }, Some("Austrian Mission".to_string())));
        _m.insert((11, 6), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((11, 7), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((11, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((11, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 1), HexData::new(Terrain::Huts { road: Road::None }, Some("Tuti".to_string())));
        _m.insert((12, 2), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((12, 3), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((12, 4), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((12, 5), HexData::new(Terrain::Building { road: Road::None }, Some("Khartoum".to_string())));
        _m.insert((12, 6), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((12, 7), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((12, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((12, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 1), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((13, 2), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((13, 3), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((13, 4), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((13, 5), HexData::new(Terrain::Building { road: Road::None }, Some("Palace".to_string())));
        _m.insert((13, 6), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((13, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((13, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 0), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((14, 1), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 2), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((14, 3), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((14, 4), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((14, 5), HexData::new(Terrain::Building { road: Road::None }, Some("Khartoum".to_string())));
        _m.insert((14, 6), HexData::new(Terrain::Building { road: Road::Crossroad }, Some("Khartoum".to_string())));
        _m.insert((14, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((14, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 0), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((15, 1), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((15, 2), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((15, 3), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((15, 4), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((15, 5), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((15, 6), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((15, 7), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((15, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 1), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, Some("Blue Nile Mouth".to_string())));
        _m.insert((16, 2), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((16, 3), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((16, 4), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((16, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 6), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((16, 7), HexData::new(Terrain::Building { road: Road::None }, Some("Arsenal".to_string())));
        _m.insert((16, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((16, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 1), HexData::new(Terrain::Huts { road: Road::None }, Some("Hogali".to_string())));
        _m.insert((17, 2), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((17, 3), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((17, 4), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((17, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 6), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((17, 7), HexData::new(Terrain::Building { road: Road::None }, Some("Barracks".to_string())));
        _m.insert((17, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 12), HexData::new(Terrain::Clear { road: Road::None }, Some("Kalakla Gate".to_string())));
        _m.insert((17, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((17, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 0), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 1), HexData::new(Terrain::Huts { road: Road::None }, Some("Hogali".to_string())));
        _m.insert((18, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 3), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((18, 4), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((18, 5), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 6), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((18, 7), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((18, 8), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((18, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((18, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 2), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 3), HexData::new(Terrain::Building { road: Road::None }, Some("North Fort".to_string())));
        _m.insert((19, 4), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((19, 5), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((19, 6), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((19, 7), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((19, 8), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((19, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 11), HexData::new(Terrain::Clear { road: Road::None }, Some("Messalamia Gate".to_string())));
        _m.insert((19, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((19, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 4), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 5), HexData::new(Terrain::Nile { direction: HexDirection::NorthWest }, None));
        _m.insert((20, 6), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((20, 7), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((20, 8), HexData::new(Terrain::Trees { road: Road::None }, None));
        _m.insert((20, 9), HexData::new(Terrain::Building { road: Road::None }, Some("Fort Buri".to_string())));
        _m.insert((20, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((20, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 6), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((21, 7), HexData::new(Terrain::Nile { direction: HexDirection::West }, None));
        _m.insert((21, 8), HexData::new(Terrain::Huts { road: Road::None }, Some("Buri".to_string())));
        _m.insert((21, 9), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((21, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 8), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 9), HexData::new(Terrain::Huts { road: Road::None }, Some("Buri".to_string())));
        _m.insert((22, 10), HexData::new(Terrain::Clear { road: Road::None }, Some("Buri Gate".to_string())));
        _m.insert((22, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((22, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 10), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 11), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((23, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 12), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 13), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((24, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 14), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m.insert((25, 15), HexData::new(Terrain::Clear { road: Road::None }, None));
        _m
    };
    let hexsides = vec![
        (HexsideRef::new(HexCoord::new(3, 0), HexCoord::new(4, 1)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(3, 1), HexCoord::new(4, 1)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(4, 0), HexCoord::new(4, 1)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(4, 1), HexCoord::new(4, 2)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(4, 1), HexCoord::new(5, 1)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(4, 1), HexCoord::new(5, 2)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(10, 11), HexCoord::new(11, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(11, 11), HexCoord::new(11, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(11, 11), HexCoord::new(12, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(12, 11), HexCoord::new(12, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(12, 12), HexCoord::new(13, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(13, 11), HexCoord::new(14, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(13, 12), HexCoord::new(13, 13)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(13, 12), HexCoord::new(14, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(13, 12), HexCoord::new(14, 13)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(14, 11), HexCoord::new(14, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(14, 11), HexCoord::new(15, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(15, 11), HexCoord::new(15, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(15, 12), HexCoord::new(16, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(16, 11), HexCoord::new(17, 12)), HexsideKind::Gate),
        (HexsideRef::new(HexCoord::new(16, 12), HexCoord::new(16, 13)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(16, 12), HexCoord::new(17, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(16, 12), HexCoord::new(17, 13)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(17, 11), HexCoord::new(17, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(17, 11), HexCoord::new(18, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(18, 2), HexCoord::new(19, 3)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(18, 3), HexCoord::new(19, 3)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(18, 11), HexCoord::new(18, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(18, 12), HexCoord::new(19, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(19, 2), HexCoord::new(19, 3)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(19, 3), HexCoord::new(19, 4)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(19, 3), HexCoord::new(20, 3)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(19, 3), HexCoord::new(20, 4)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(19, 8), HexCoord::new(20, 9)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(19, 9), HexCoord::new(20, 9)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(19, 11), HexCoord::new(20, 12)), HexsideKind::Gate),
        (HexsideRef::new(HexCoord::new(19, 12), HexCoord::new(19, 13)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(19, 12), HexCoord::new(20, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(19, 12), HexCoord::new(20, 13)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(20, 8), HexCoord::new(20, 9)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(20, 8), HexCoord::new(21, 8)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(20, 9), HexCoord::new(20, 10)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(20, 9), HexCoord::new(21, 9)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(20, 9), HexCoord::new(21, 10)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(20, 11), HexCoord::new(20, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(20, 11), HexCoord::new(21, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(21, 8), HexCoord::new(21, 9)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(21, 9), HexCoord::new(22, 9)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(21, 9), HexCoord::new(22, 10)), HexsideKind::Gate),
        (HexsideRef::new(HexCoord::new(21, 10), HexCoord::new(22, 10)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(21, 10), HexCoord::new(22, 11)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(21, 11), HexCoord::new(21, 12)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(21, 11), HexCoord::new(22, 11)), HexsideKind::Wall),
        (HexsideRef::new(HexCoord::new(21, 11), HexCoord::new(22, 12)), HexsideKind::Wall),
    ];
    let roads = vec![
        HexsideRef::new(HexCoord::new(7, 4), HexCoord::new(8, 4)),
        HexsideRef::new(HexCoord::new(8, 4), HexCoord::new(9, 4)),
        HexsideRef::new(HexCoord::new(9, 4), HexCoord::new(9, 5)),
        HexsideRef::new(HexCoord::new(9, 4), HexCoord::new(10, 4)),
        HexsideRef::new(HexCoord::new(9, 5), HexCoord::new(9, 6)),
        HexsideRef::new(HexCoord::new(9, 6), HexCoord::new(10, 6)),
        HexsideRef::new(HexCoord::new(9, 7), HexCoord::new(10, 7)),
        HexsideRef::new(HexCoord::new(10, 6), HexCoord::new(11, 6)),
        HexsideRef::new(HexCoord::new(10, 7), HexCoord::new(11, 7)),
        HexsideRef::new(HexCoord::new(11, 5), HexCoord::new(11, 6)),
        HexsideRef::new(HexCoord::new(11, 6), HexCoord::new(12, 6)),
        HexsideRef::new(HexCoord::new(11, 6), HexCoord::new(12, 7)),
        HexsideRef::new(HexCoord::new(11, 7), HexCoord::new(12, 7)),
        HexsideRef::new(HexCoord::new(12, 6), HexCoord::new(13, 6)),
        HexsideRef::new(HexCoord::new(12, 6), HexCoord::new(13, 7)),
        HexsideRef::new(HexCoord::new(12, 7), HexCoord::new(12, 8)),
        HexsideRef::new(HexCoord::new(12, 7), HexCoord::new(13, 7)),
        HexsideRef::new(HexCoord::new(13, 5), HexCoord::new(13, 6)),
        HexsideRef::new(HexCoord::new(13, 6), HexCoord::new(14, 6)),
        HexsideRef::new(HexCoord::new(13, 6), HexCoord::new(14, 7)),
        HexsideRef::new(HexCoord::new(14, 6), HexCoord::new(15, 6)),
    ];
    let excluded = BTreeSet::new();
    let overlay = OverlayParams { width: 18, height: 16, hex_size: 51.8, offset_x: -736.0, offset_y: -403.0, orientation: Orientation::Pointy, offset_variant: OffsetVariant::OddR, shape: GridShape::Rectangle, long_rows_even: true, rotation_deg: -0.86, aspect_y: 0.9915, shear_x: -0.0148, shear_y: 0.017, size_grad_x: -0.000001, size_grad_y: 0.000003 };
    MapData { tiles, hexsides, roads, excluded, overlay, img_w: 1571.0, img_h: 1200.0, image: "fall_of_khartoum_1885.webp".to_string(), calib: CalibAnchors { p1_px: (736.0, 420.0), p1_hex: (0, 0), p2_px: (1178.0, 572.0), p2_hex: (5, -1) }, campaign_turn_track: None }
}

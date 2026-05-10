pub mod layout;
pub mod render;

use std::collections::HashMap;

// ── Hex coordinate ─────────────────────────────────────────────────────────────

/// Axial hex coordinate.  `q` runs east, `r` runs south-east (flat-top).
/// Primary key for everything on the map.
#[derive(bevy::prelude::Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
}

// ── Hex edge ──────────────────────────────────────────────────────────────────

/// The edge between two adjacent hexes — where wall segments live.
///
/// Stored in canonical (sorted) order so each physical edge has exactly one key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HexEdge(pub HexCoord, pub HexCoord);

impl HexEdge {
    pub fn new(a: HexCoord, b: HexCoord) -> Self {
        // Canonical: smaller (q, r) first.
        if (a.q, a.r) <= (b.q, b.r) {
            HexEdge(a, b)
        } else {
            HexEdge(b, a)
        }
    }
}

// ── Terrain ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
    Desert,     // plain sand — the majority of the map
    Palm,       // palm-tree hexes (Tuti island, Fort Buri surrounds, etc.)
    BlueNile,   // impassable to land units
    WhiteNile,  // impassable to land units
    City,       // Khartoum urban grid
    Settlement, // Hogali, Tuti, Buri building clusters
}

impl Terrain {
    pub fn passable_by_land(self) -> bool {
        !matches!(self, Terrain::BlueNile | Terrain::WhiteNile)
    }
}

// ── Named locations ───────────────────────────────────────────────────────────

/// Static points of interest — never change during play.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Location {
    FortMakran,
    NorthFort,
    FortBuri,
    AustrianMission,
    Palace,
    Arsenal,
    Barracks,
}

// ── Per-hex data ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct HexData {
    pub terrain: Terrain,
    pub location: Option<Location>,
}

// ── Wall / gate data ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct WallSegment {
    /// Named gate on this edge, if any.
    pub gate: Option<&'static str>,
}

// ── Map resource ──────────────────────────────────────────────────────────────

#[derive(bevy::prelude::Resource, Default)]
pub struct GameMap {
    pub hexes: HashMap<HexCoord, HexData>,
    /// Wall segments keyed by the edge between the two hexes they separate.
    pub walls: HashMap<HexEdge, WallSegment>,
}

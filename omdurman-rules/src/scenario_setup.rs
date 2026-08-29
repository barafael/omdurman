//! Fixed-hex scenario placements (rulebook §9.212, §9.321, §9.344, §9.346).
//!
//! The rulebook's set-up is mostly *player choice*; only a handful of counters
//! have a single unambiguous hex. This module owns the placement *data*; the
//! app (`omdurman-app/src/scenario_setup.rs`) resolves the anchors against the
//! loaded map and emits the placements as ordinary [`crate::GameEvent`]
//! `PlaceUnit`s, so they flow through netcode ordering like interactive
//! placement.

use omdurman_types::{Location, SectionName, SetupLetter};

/// What unambiguously fixes a counter's set-up hex on the board.
pub enum SetupAnchor {
    /// A lettered set-up hex (Historical scenario, §9.212).
    Letter(SetupLetter),
    /// A named landmark hex (e.g. the Palace for GORDON, §9.321/§9.346).
    Location(Location),
}

/// One fixed-hex placement: which counter (`section`/`col`/`row` on the sprite
/// sheet) goes onto the single hex identified by `anchor`.
pub struct FixedPlacement {
    pub section: SectionName,
    pub col: u32,
    pub row: u32,
    pub anchor: SetupAnchor,
}

/// Fall-of-Khartoum fixed placements (§9.321/§9.344/§9.346):
/// - GORDON is the one counter with a single, unambiguous hex -- he starts in
///   (and may never leave) the Palace.
/// - The North Fort is Dervish-controlled per §9.344. The engine treats it as
///   a `Fort` unit placed at the `Location::NorthFort` landmark; its
///   artillery factor fires on the Artillery line and it is enclosed by its
///   own wall ring (it cannot be entered by the British).
///
/// The rest of the British garrison and the Dervish entry forces are
/// player-placed (§9.321 "anywhere in the walled city", §9.322 map-edge
/// entry). GORDON is the "GEN. GORDON" counter at British_Boats (3,1); the
/// North Fort uses a campaign HadendowaForts counter (one of the spare fort
/// sprites).
pub const FALL_OF_KHARTOUM_SETUP: &[FixedPlacement] = &[
    FixedPlacement {
        section: SectionName::BritishBoats,
        col: 3,
        row: 1,
        anchor: SetupAnchor::Location(Location::Palace),
    },
    FixedPlacement {
        section: SectionName::HadendowaForts,
        col: 0,
        row: 0,
        anchor: SetupAnchor::Location(Location::NorthFort),
    },
];

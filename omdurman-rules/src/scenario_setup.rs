//! Fixed-hex scenario placements (rulebook §9.212, §9.321, §9.344, §9.346).
//!
//! The rulebook's set-up is mostly *player choice*; only a handful of counters
//! have a single unambiguous hex. This module owns the placement *data*; the
//! app (`omdurman-app/src/scenario_setup.rs`) resolves the anchors against the
//! loaded map and emits the placements as ordinary `GameEvent`
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
///   artillery factor fires on the Artillery line, and as an enemy fort the
///   British may not occupy it (§6.54).
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

/// The six Dervish leaders and their Historical-scenario lettered set-up hexes
/// (§9.212). Two leaders (Yakub, Osman Digna) have no sprite section of their
/// own and ride in a tribal block -- see `omdurman_rules::unit_profiles::identity_for_section`,
/// which resolves those specific counters as leaders.
pub const HISTORICAL_LEADERS: &[FixedPlacement] = &[
    // A: Ali Wad Helu
    FixedPlacement {
        section: SectionName::AliWadHelu,
        col: 0,
        row: 0,
        anchor: SetupAnchor::Letter(SetupLetter::A),
    },
    // D: Sheik El Din
    FixedPlacement {
        section: SectionName::SheikElDin,
        col: 0,
        row: 0,
        anchor: SetupAnchor::Letter(SetupLetter::D),
    },
    // Y: Yakub (first counter of the Jaalin_I block)
    FixedPlacement {
        section: SectionName::JaalinI,
        col: 0,
        row: 0,
        anchor: SetupAnchor::Letter(SetupLetter::Y),
    },
    // K: Khalifa Abdullah
    FixedPlacement {
        section: SectionName::KhalifaAbdullah,
        col: 0,
        row: 0,
        anchor: SetupAnchor::Letter(SetupLetter::K),
    },
    // S: Sherif
    FixedPlacement {
        section: SectionName::Sherif,
        col: 0,
        row: 0,
        anchor: SetupAnchor::Letter(SetupLetter::S),
    },
    // O: Osman Digna (second counter of the Hadendowa block)
    FixedPlacement {
        section: SectionName::Hadendowa,
        col: 1,
        row: 0,
        anchor: SetupAnchor::Letter(SetupLetter::O),
    },
];

/// The fixed placements of `scenario` (Campaign has none).
pub fn fixed_placements(scenario: omdurman_types::Scenario) -> &'static [FixedPlacement] {
    match scenario {
        omdurman_types::Scenario::Campaign => &[],
        omdurman_types::Scenario::Historical => HISTORICAL_LEADERS,
        omdurman_types::Scenario::FallOfKhartoum => FALL_OF_KHARTOUM_SETUP,
    }
}

/// Whether `id` is one of `scenario`'s fixed-hex counters: placed by the
/// scenario itself at game start, not by a player, so the set-up order
/// (who sets up first) does not bind it.
pub fn is_fixed_placement(scenario: omdurman_types::Scenario, id: crate::UnitId) -> bool {
    fixed_placements(scenario)
        .iter()
        .any(|f| crate::unit_id_for_section_pos(f.section, f.col as u8, f.row as u8) == Some(id))
}

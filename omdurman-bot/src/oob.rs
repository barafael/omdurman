//! Order-of-battle reconstruction for the Setup phase (§9.2/§9.3).
//!
//! The engine tracks only `setup_deployed_count` vs `setup_target` — not *which*
//! units remain to be placed. This module rebuilds each scenario's deployable
//! force list headlessly from `UnitId::ALL` + `sections_for_picker`, so the bot
//! knows what to deploy. It also resolves the fixed-hex placements (GORDON at
//! the Palace, the North Fort) directly from `BoardInfo` landmarks.

use omdurman_rules::effects::GameState;
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_rules::{UnitId, UnitPlacement, UnitState};
use omdurman_types::{Location, Player, Scenario};

/// Unit IDs that are fixed-placed at scenario start (not player-deployed).
/// For FoK: GORDON (BritishBoats 3,1) and the North Fort (HadendowaForts 0,0).
/// For Campaign/Historical: none (all units are player-deployed in this harness;
/// Historical leader pinning to lettered hexes is skipped — leaders deploy in
/// the zone like any other unit).
fn fixed_unit_ids(scenario: Scenario) -> &'static [UnitId] {
    match scenario {
        Scenario::FallOfKhartoum => &[
            // GORDON — BritishBoats (3,1)
            UnitId::BritishBoats_3_1,
            // North Fort — HadendowaForts (0,0)
            UnitId::HadendowaForts_0_0,
        ],
        Scenario::Campaign | Scenario::Historical => &[],
    }
}

/// Resolve the fixed-hex placements for `scenario` against the board landmarks.
/// Returns fully-formed [`UnitPlacement`]s ready for `DeployUnit`. Empty for
/// Campaign; GORDON + North Fort for FoK; empty for Historical (leaders deploy
/// freely in this harness).
pub fn fixed_placements(state: &GameState) -> Vec<UnitPlacement> {
    let board = &state.board;
    let mut out = Vec::new();
    for &id in fixed_unit_ids(state.scenario) {
        let Some(profile) = profile_for_unit(id) else {
            continue;
        };
        // Resolve the landmark hex for this fixed unit.
        let loc = match id {
            UnitId::BritishBoats_3_1 => Some(Location::Palace),
            UnitId::HadendowaForts_0_0 => Some(Location::NorthFort),
            _ => None,
        };
        let Some(loc) = loc else { continue };
        let Some(hex) = board.hex_of_location(loc) else {
            continue;
        };
        out.push(UnitPlacement {
            id,
            position: hex,
            profile,
            state: UnitState::default(),
        });
    }
    out
}

/// The deployable order of battle for a scenario: every `UnitId` that (a)
/// belongs to an in-play section, (b) has a compiled profile, (c) is not a
/// fixed-placement unit, and (d) passes the FoK OOB filter (§9.321/§9.322 —
/// only identities in `fok_cap_group` are in play). Grouped by owning player
/// so the bot deploys each side's force.
pub fn deployable_oob(scenario: Scenario) -> Vec<(Player, UnitId)> {
    let fixed = fixed_unit_ids(scenario);
    let allowed_sections = scenario.sections_for_picker();

    let mut out = Vec::new();
    for &id in UnitId::ALL {
        // Skip fixed-placement units.
        if fixed.contains(&id) {
            continue;
        }
        let (section, _col, _row) = id.section_pos();
        // Section filter: FoK restricts to 10 sections; Campaign/Historical
        // allow all (sections_for_picker returns None).
        if let Some(allowed) = allowed_sections
            && !allowed.contains(&section)
        {
            continue;
        }
        let Some(profile) = profile_for_unit(id) else {
            continue;
        };
        // FoK OOB filter: §9.321/§9.322 — only identities covered by
        // `fok_cap_group` are in play. This hides cavalry, engineers,
        // Maxims, Dervish leaders, Dervish gunboats, named gunboats,
        // Isa Zachneih, etc.
        if scenario == Scenario::FallOfKhartoum
            && omdurman_rules::effects::fok_cap_group(&profile.identity).is_none()
        {
            continue;
        }
        out.push((profile.identity.owner(), id));
    }
    out
}

/// Unit IDs for a specific player's deployable OOB.
pub fn deployable_oob_for(scenario: Scenario, player: Player) -> Vec<UnitId> {
    deployable_oob(scenario)
        .into_iter()
        .filter(|(p, _)| *p == player)
        .map(|(_, id)| id)
        .collect()
}

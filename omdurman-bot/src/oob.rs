//! Order-of-battle reconstruction for the Setup phase (§9.2/§9.3).
//!
//! The engine tracks only `setup_deployed_count` vs `setup_target` — not *which*
//! units remain to be placed. This module rebuilds each scenario's deployable
//! force list headlessly from `UnitId::ALL` + `sections_for_picker`, so the bot
//! knows what to deploy. It also resolves the fixed-hex placements (GORDON at
//! the Palace, the North Fort, the Historical leaders on their letters).

use omdurman_rules::board_data;
use omdurman_rules::effects::GameState;
use omdurman_rules::scenario_setup::SetupAnchor;
use omdurman_rules::unit_id_for_section_pos;
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_rules::{UnitId, UnitPlacement, UnitState};
use omdurman_types::{HexCoord, Player, Scenario};

/// The counters fixed-placed at scenario start (not player-deployed), from
/// the rules crate's [`fixed_placements`](omdurman_rules::scenario_setup::fixed_placements)
/// -- the list the app places from: GORDON, the North Fort and Forts Makran
/// and Buri in Fall of Khartoum (§9.321/§9.344/§9.346), the six Dervish
/// leaders on their lettered hexes in the Historical scenario (§9.212);
/// none in the Campaign game.
fn fixed_unit_ids(scenario: Scenario) -> Vec<UnitId> {
    omdurman_rules::scenario_setup::fixed_placements(scenario)
        .iter()
        .filter_map(|fp| unit_id_for_section_pos(fp.section, fp.col as u8, fp.row as u8))
        .collect()
}

/// The hex a fixed placement's anchor names on the scenario's board: a
/// landmark, or a lettered set-up hex (the printed letters live only in the
/// map data, not in [`BoardInfo`](omdurman_rules::board::BoardInfo)).
fn anchor_hex(state: &GameState, anchor: &SetupAnchor) -> Option<HexCoord> {
    match anchor {
        SetupAnchor::Location(loc) => state.board.hex_of_location(*loc),
        SetupAnchor::Letter(letter) => {
            let map = match state.scenario {
                Scenario::FallOfKhartoum => board_data::fall_of_khartoum_map_data(),
                Scenario::Campaign | Scenario::Historical => board_data::campaign_map_data(),
            };
            map.tiles
                .iter()
                .find(|(_, data)| data.setup_letter == Some(*letter))
                .map(|(&(q, r), _)| HexCoord::new(q, r))
        }
    }
}

/// Resolve the fixed-hex placements for `scenario` against the board.
/// Returns fully-formed [`UnitPlacement`]s ready for `DeployUnit`, as the
/// app's scenario set-up places them.
pub fn fixed_placements(state: &GameState) -> Vec<UnitPlacement> {
    let mut out = Vec::new();
    for fp in omdurman_rules::scenario_setup::fixed_placements(state.scenario) {
        let Some(id) = unit_id_for_section_pos(fp.section, fp.col as u8, fp.row as u8) else {
            continue;
        };
        let Some(profile) = profile_for_unit(id) else {
            continue;
        };
        let Some(hex) = anchor_hex(state, &fp.anchor) else {
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
        // Isa Zachneih, etc. The Ali_Wad_Helu block's Kehena/Degheim
        // "Deghelim" counters resolve to those tribes (see
        // `unit_profiles::ali_wad_helu`), so they stay deployable while the
        // block's leader is hidden.
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

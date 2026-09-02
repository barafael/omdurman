//! Regression: the FALL OF KHARTOUM setup flow must be completable with the
//! *sprite-backed* counters the picker actually offers (§9.2/§9.3, §9.321–§9.322).
//!
//! The Degheim force is printed on Baggara-backed counters in the Ali_Wad_Helu
//! block, and the Kehena force on that block's row 1; there are no separate
//! Kehena/Degheim sprite sections. When those block cells resolved to a
//! Baggara identity (or the picker allowlist named the sprite-less virtual
//! sections), the Dervish deployment stalled at 37+fort of the required 49:
//! the per-faction Ready gate (§9.2/§9.3) could never open and the game was
//! stuck in Setup after both players deployed "everything" the picker offered.
//! This test drives the exact flow both players perform -- fixed placements,
//! per-faction deployment limited to the picker's visible set, Ready in
//! AE-first order -- and asserts the engine leaves Setup for the first
//! Movement turn (§4).

use omdurman_rules::UnitId;
use omdurman_rules::board::BoardInfo;
use omdurman_rules::board_data::fall_of_khartoum_map_data;
use omdurman_rules::effects::{FokCapGroup, GameEffect, GameState, apply_effect, fok_cap_group};
use omdurman_rules::unit_profiles::{profile_for_unit, section_owner};
use omdurman_types::{Player, Scenario};
use traceability_macro::rulebook;

/// Every counter the app's picker offers for `scenario` given the units
/// already on the board: a compiled profile in a section of the scenario's
/// picker allowlist, in the FoK order of battle, and not hidden by a filled
/// per-group OOB cap. Mirrors the picker's three visibility filters, in
/// [`UnitId::ALL`] order (the caps hide counters in list order).
fn picker_offered_ids(scenario: Scenario, placed: &[UnitId]) -> Vec<UnitId> {
    let Some(allowed) = scenario.sections_for_picker() else {
        return UnitId::ALL.to_vec();
    };
    // (group, cap, claimed) per FoK OOB group: seeded from the board, then
    // claimed further by each offered counter, exactly like the picker's
    // (placed, kept) tally.
    let mut caps: Vec<(FokCapGroup, usize, usize)> = Vec::new();
    for &id in placed {
        if let Some(profile) = profile_for_unit(id)
            && let Some((g, cap)) = fok_cap_group(&profile.identity)
        {
            match caps.iter_mut().find(|(eg, _, _)| *eg == g) {
                Some((_, _, claimed)) => *claimed += 1,
                None => caps.push((g, cap, 1)),
            }
        }
    }
    let mut out = Vec::new();
    for &id in UnitId::ALL {
        let (section, _, _) = id.section_pos();
        if !allowed.contains(&section) || profile_for_unit(id).is_none() {
            continue;
        }
        let Some((g, cap)) = fok_cap_group(&profile_for_unit(id).unwrap().identity) else {
            continue; // the picker hides counters outside the §9.321/§9.322 OOB
        };
        if caps.iter().all(|(eg, _, _)| *eg != g) {
            caps.push((g, cap, 0));
        }
        let entry = caps.iter_mut().find(|(eg, _, _)| *eg == g).unwrap();
        if entry.2 >= entry.1 {
            continue; // cap filled: the picker hides the excess counters
        }
        entry.2 += 1;
        out.push(id);
    }
    out
}

/// The fixed scenario placements (host auto-setup): GORDON in the palace
/// (§9.321/§9.346) and the North Fort fort (§9.344).
fn fixed_ids(scenario: Scenario) -> Vec<UnitId> {
    match scenario {
        Scenario::FallOfKhartoum => vec![UnitId::BritishBoats_3_1, UnitId::HadendowaForts_0_0],
        _ => vec![],
    }
}

#[test]
fn fok_setup_completes_with_sprite_backed_counters() {
    let scenario = Scenario::FallOfKhartoum;
    let board = BoardInfo::from_map_data(&fall_of_khartoum_map_data());
    let mut state = GameState::with_board(scenario, board);

    // Deploy a counter on a legal hex (deployment zone + §5.51/§5.52 stacking).
    let land_hexes: Vec<_> = state.board.terrain.keys().copied().collect();
    let deploy = |state: &mut GameState, id: UnitId| {
        let profile = profile_for_unit(id).expect("compiled profile");
        let is_boat = profile.kind.is_boat();
        let group = fok_cap_group(&profile.identity).map(|(g, _)| g);
        for &hex in &land_hexes {
            if !state.in_deployment_zone(profile.identity.owner(), hex, is_boat) {
                continue;
            }
            let occupants: Vec<_> = state.units.iter().filter(|u| u.position == hex).collect();
            // §5.51: the four-unit limit; gunboats never share a hex at all.
            // §5.52: one Dervish tribe per hex (approximated by the FoK cap
            // group, which the picker mirrors).
            if occupants.len() >= 4
                || (is_boat && !occupants.is_empty())
                || occupants.iter().any(|u| u.profile.kind.is_boat())
                || occupants
                    .iter()
                    .any(|u| fok_cap_group(&u.profile.identity).map(|(g, _)| g) != group)
            {
                continue;
            }
            let placement = omdurman_rules::UnitPlacement {
                id,
                position: hex,
                profile,
                state: Default::default(),
            };
            state
                .can_deploy_unit(&placement)
                .unwrap_or_else(|e| panic!("counter {id:?} must be deployable in FoK: {e}"));
            apply_effect(state, &GameEffect::DeployUnit(placement)).expect("deployment accepted");
            return;
        }
        panic!("no legal deploy hex for {id:?}");
    };

    // Host auto-setup first (GORDON + North Fort), as the app emits it.
    let fixed = fixed_ids(scenario);
    for id in &fixed {
        deploy(&mut state, *id);
    }

    // What each side may still deploy, exactly as the picker presents it.
    let placed: Vec<_> = state.units.iter().map(|u| u.id).collect();
    let offered = picker_offered_ids(scenario, &placed);
    let dervish_offered: Vec<_> = offered
        .iter()
        .copied()
        .filter(|&id| section_owner(id.section_pos().0) == Some(Player::Dervish))
        .collect();
    let ae_offered: Vec<_> = offered
        .iter()
        .copied()
        .filter(|&id| section_owner(id.section_pos().0) == Some(Player::AngloEgyptian))
        .collect();
    assert_eq!(
        dervish_offered.len(),
        48,
        "the §9.322 entry force must be placeable from cut sprites (48 counters)"
    );
    assert_eq!(
        ae_offered.len(),
        16,
        "the §9.321 garrison must be placeable from cut sprites (17 less scenario-fixed GORDON)"
    );

    let deploy_side = |state: &mut GameState, player: Player, ids: &[UnitId]| {
        for &id in ids {
            deploy(state, id);
        }
        assert!(
            state.setup_target_met(player),
            "{player:?} must meet its setup target (deployed {})",
            state.setup_deployed_count(player)
        );
    };

    deploy_side(&mut state, Player::AngloEgyptian, &ae_offered);
    deploy_side(&mut state, Player::Dervish, &dervish_offered);

    // Board-wide gate: the game may leave Setup.
    state
        .setup_complete()
        .expect("both §9.321/§9.322 orders of battle fully deployed");

    // AE confirms Ready first (as in the recorded session); the engine must
    // stay in Setup until the Dervish side confirms too (§9.2/§9.3).
    apply_effect(
        &mut state,
        &GameEffect::ConfirmSetupReady {
            player: Player::AngloEgyptian,
        },
    )
    .unwrap();
    assert_eq!(state.phase, omdurman_rules::Phase::Setup);

    apply_effect(
        &mut state,
        &GameEffect::ConfirmSetupReady {
            player: Player::Dervish,
        },
    )
    .unwrap();
    assert_eq!(
        state.phase,
        omdurman_rules::Phase::Movement,
        "both factions ready -> first Movement turn (§4)"
    );
}

/// The five Degheim counters are the Baggara-backed cells of the Ali_Wad_Helu
/// block: they must resolve to the §9.322 Degheim tribe so the FoK order-of-
/// battle gate (identity-keyed) accepts them (§9.322).
#[rulebook("§9.322")]
#[test]
fn degheim_counters_resolve_to_the_degheim_tribe() {
    let cells = [
        UnitId::AliWadHelu_1_0,
        UnitId::AliWadHelu_2_0,
        UnitId::AliWadHelu_3_0,
        UnitId::AliWadHelu_4_0,
        UnitId::AliWadHelu_5_0,
    ];
    for id in cells {
        let profile = profile_for_unit(id).expect("compiled profile");
        assert_eq!(
            profile.identity,
            omdurman_rules::UnitIdentity::DervishTribal {
                tribe: omdurman_types::DervishTribe::Degheim
            },
            "{id:?} must resolve to the Degheim force"
        );
        // ...and the identity must be in the FoK order of battle (cap 5).
        let (group, cap) = fok_cap_group(&profile.identity).expect("Degheim is in the FoK OOB");
        assert_eq!(
            group,
            FokCapGroup::Tribe(omdurman_types::DervishTribe::Degheim)
        );
        assert_eq!(cap, 5);
    }
}

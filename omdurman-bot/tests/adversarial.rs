//! Adversarial validation fuzzing for `apply_effect`.
//!
//! The rules engine is the *security boundary* of a peer-to-peer game: every
//! peer can submit any [`GameEffect`], so `apply_effect` must **reject**
//! illegal effects -- not only accept legal ones. The playthrough runner
//! cannot catch validation holes: its move generator (`legal_actions`) is
//! trusted and only ever offers legal actions, so an engine that *accepts* an
//! illegal effect (e.g. fire resolved against the firer's own unit -- a real
//! session bug) is never probed. These tests attack the other side of the
//! contract:
//!
//! 1. **Directed regressions** for known validation bugs.
//! 2. **Fire-sweep proptest**: across reachable mid-game states, fire at any
//!    hex without opponent units must always be rejected.
//! 3. **Mutation fuzzer**: mutated / nonsensical effects applied to reachable
//!    states must either be rejected or leave every engine invariant holding
//!    -- and must never panic.

use omdurman_bot::agent::Agents;
use omdurman_bot::invariants::check_all_with_tribal;
use omdurman_bot::rng::BotRng;
use omdurman_bot::{PlayConfig, board_for_scenario, playthrough};
use omdurman_net::GameEvent;
use omdurman_rules::combat_results_table::FireFactorRow;
use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
use omdurman_rules::unit_id_for_section_pos;
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_rules::{
    DieRoll, FireAttack, FireKind, FireModifier, MovementPoints, Phase, UnitId, UnitPlacement,
};
use omdurman_types::{HexCoord, Player, Scenario, SectionName};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A live FoK state with the real board, in AE's Direct Fire sub-phase
/// (turn owner Dervish; defensive-fire control derives to the AE).
fn fire_phase_state() -> GameState {
    let mut gs = GameState::with_board(
        Scenario::FallOfKhartoum,
        board_for_scenario(Scenario::FallOfKhartoum),
    );
    gs.phase = Phase::DefensiveFire(omdurman_rules::FireSubPhase::DirectFire);
    gs.active_player = Player::Dervish;
    gs
}

/// Place a real counter (with its real profile) at `hex`.
fn place(gs: &mut GameState, section: SectionName, col: u32, row: u32, hex: HexCoord) -> UnitId {
    let id = unit_id_for_section_pos(section, col as u8, row as u8).expect("counter has a UnitId");
    let profile = profile_for_unit(id).expect("counter has a profile");
    gs.units.push(UnitPlacement {
        id,
        position: hex,
        profile,
        state: Default::default(),
    });
    id
}

fn direct_attack(firer: UnitId, target: HexCoord, modifiers: Vec<FireModifier>) -> GameEffect {
    GameEffect::FireCombat {
        attack: FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: Phase::DefensiveFire(omdurman_rules::FireSubPhase::DirectFire),
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: target,
            factor_row: FireFactorRow::Row01to05,
            modifiers,
        },
        roll: DieRoll::Three,
    }
}

/// Replay a playthrough trace to `cut` applied effects (the same late-joiner
/// path the net layer uses), yielding a *reachable mid-game* state.
fn replay_to(events: &[omdurman_net::GameEvent], cut: usize) -> GameState {
    let mut state: Option<GameState> = None;
    let mut applied = 0usize;
    for ev in events {
        match ev {
            GameEvent::StartGame { scenario, .. } => {
                state = Some(GameState::with_board(
                    *scenario,
                    board_for_scenario(*scenario),
                ));
            }
            GameEvent::Effect(eff) => {
                let s = state.as_mut().expect("Effect before StartGame");
                if applied >= cut {
                    break;
                }
                apply_effect(s, eff).expect("trace replay must accept recorded effects");
                applied += 1;
            }
            _ => {}
        }
    }
    state.expect("trace had no StartGame")
}

/// Replay the trace and capture the first `cap` states whose phase is a
/// *Direct Fire* sub-phase -- the states where the occupancy gate can be
/// exercised (final states are usually game-over and uninformative).
fn direct_fire_states(events: &[omdurman_net::GameEvent], cap: usize) -> Vec<GameState> {
    let mut state: Option<GameState> = None;
    let mut captured: Vec<GameState> = Vec::new();
    let is_direct = |p: &Phase| {
        matches!(
            p,
            Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire)
                | Phase::DefensiveFire(omdurman_rules::FireSubPhase::DirectFire)
        )
    };
    for ev in events {
        if captured.len() >= cap {
            break;
        }
        match ev {
            GameEvent::StartGame { scenario, .. } => {
                state = Some(GameState::with_board(
                    *scenario,
                    board_for_scenario(*scenario),
                ));
            }
            GameEvent::Effect(eff) => {
                let s = state.as_mut().expect("Effect before StartGame");
                apply_effect(s, eff).expect("trace replay must accept recorded effects");
                if is_direct(&s.phase) {
                    captured.push(s.clone());
                }
            }
            _ => {}
        }
    }
    captured
}

/// Pre-checks `can_fire_at`'s *earlier* gates so a sweep only asserts the
/// occupancy gate (any other rejection is correct but uninformative).
fn pre_occupancy_gates_pass(gs: &GameState, firer: UnitId, kind: FireKind) -> bool {
    let Some(u) = gs.find_unit(firer) else {
        return false;
    };
    if u.profile.identity.owner() != gs.phase_player() {
        return false;
    }
    let subphase_ok = matches!(
        (&gs.phase, kind),
        (
            Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire)
                | Phase::DefensiveFire(omdurman_rules::FireSubPhase::DirectFire),
            FireKind::Direct
        ) | (
            Phase::OffensiveFire(omdurman_rules::FireSubPhase::MaximSecondAndHowitzer)
                | Phase::DefensiveFire(omdurman_rules::FireSubPhase::MaximSecondAndHowitzer),
            FireKind::MaximSecondFire
        )
    );
    if !subphase_ok {
        return false;
    }
    if kind == FireKind::Howitzer && gs.day_night == omdurman_types::DayNight::Night {
        return false;
    }
    !u.state.disrupted && u.profile.fire.is_some() && !gs.units_fired_this_phase.contains(&firer)
}

fn seed_config() -> PlayConfig {
    PlayConfig {
        max_actions_per_phase: 50,
        max_turns: 8,
        keep_out: None,
    }
}

// ---------------------------------------------------------------------------
// 1. Directed regressions
// ---------------------------------------------------------------------------

/// Session regression (2026-09-13): a defensive-fire attack targeting the
/// firer's *own* stack was accepted and resolved the CRT against it.
#[test]
fn fire_at_own_hex_is_rejected() {
    let mut gs = fire_phase_state();
    let firer = place(
        &mut gs,
        SectionName::EgyptianArmy,
        0,
        1,
        HexCoord::new(17, 7),
    );
    let own = place(
        &mut gs,
        SectionName::EgyptianArmy,
        1,
        1,
        HexCoord::new(16, 7),
    );

    let attack = direct_attack(
        firer,
        HexCoord::new(16, 7),
        vec![FireModifier::AngloEgyptianDirectFire],
    );
    let err = apply_effect(&mut gs, &attack).expect_err("own-hex fire must be rejected");
    assert!(
        matches!(
            err,
            omdurman_rules::effects::RuleError::FireTargetNotEnemyOccupied
        ),
        "expected occupancy rejection, got {err}"
    );
    // The own unit is untouched.
    assert!(gs.find_unit(own).is_some());
    assert!(!gs.find_unit(own).unwrap().state.disrupted);
}

#[test]
fn fire_at_empty_hex_is_rejected() {
    let mut gs = fire_phase_state();
    let firer = place(
        &mut gs,
        SectionName::EgyptianArmy,
        0,
        1,
        HexCoord::new(17, 7),
    );

    // Adjacent, but nobody home.
    assert!(
        apply_effect(
            &mut gs,
            &direct_attack(
                firer,
                HexCoord::new(16, 7),
                vec![FireModifier::AngloEgyptianDirectFire]
            )
        )
        .is_err()
    );
}

#[test]
fn legal_fire_still_resolves_and_holds_invariants() {
    let mut gs = fire_phase_state();
    let firer = place(
        &mut gs,
        SectionName::EgyptianArmy,
        0,
        1,
        HexCoord::new(17, 7),
    );
    place(&mut gs, SectionName::Hadendowa, 0, 1, HexCoord::new(16, 7)); // enemy, adjacent

    apply_effect(
        &mut gs,
        &direct_attack(
            firer,
            HexCoord::new(16, 7),
            vec![FireModifier::AngloEgyptianDirectFire],
        ),
    )
    .expect("adjacent enemy-hex fire must resolve");
    check_all_with_tribal(&gs).expect("invariants hold after a legal shot");
}

/// A tampered modifier list must be rejected (the engine derives the
/// mandatory set itself; §6.24 etc. are not caller-chosen).
#[test]
fn fire_with_tampered_modifiers_is_rejected() {
    let mut gs = fire_phase_state();
    let firer = place(
        &mut gs,
        SectionName::EgyptianArmy,
        0,
        1,
        HexCoord::new(17, 7),
    );
    place(&mut gs, SectionName::Hadendowa, 0, 1, HexCoord::new(16, 7));

    let attack = direct_attack(
        firer,
        HexCoord::new(16, 7),
        vec![
            FireModifier::AngloEgyptianDirectFire,
            FireModifier::BrigadeIntegrity, // single firer: bogus
        ],
    );
    assert!(apply_effect(&mut gs, &attack).is_err());
}

/// Each physical counter deploys exactly once (§9.2/§9.3/§9.112): a repeated
/// `DeployUnit` for the same sprite must be rejected, not double-placed.
#[test]
fn duplicate_deploy_is_rejected() {
    let mut gs = GameState::with_board(
        Scenario::FallOfKhartoum,
        board_for_scenario(Scenario::FallOfKhartoum),
    );
    let id = unit_id_for_section_pos(SectionName::Hadendowa, 0, 1).expect("counter id");
    let placement = UnitPlacement {
        id,
        position: HexCoord::new(24, 12), // inside the Dervish deployment zone
        profile: profile_for_unit(id).expect("profile"),
        state: Default::default(),
    };
    apply_effect(&mut gs, &GameEffect::DeployUnit(placement)).expect("first deploy must succeed");
    assert!(apply_effect(&mut gs, &GameEffect::DeployUnit(placement)).is_err());
    assert_eq!(gs.units.iter().filter(|u| u.id == id).count(), 1);
}

/// A unit of the non-moving side may not act during the opponent's sub-phase.
#[test]
fn move_of_non_phase_player_is_rejected() {
    let mut gs = fire_phase_state(); // control is AE's; Dervish may not move
    let dervish = place(&mut gs, SectionName::Hadendowa, 0, 1, HexCoord::new(20, 13));

    let eff = GameEffect::MoveUnit {
        unit_id: dervish,
        to: HexCoord::new(20, 12),
        cost: MovementPoints::new(1),
        path: vec![],
    };
    assert!(apply_effect(&mut gs, &eff).is_err());
}

/// Fire during the movement phase is wrong-phase, not silently resolved.
#[test]
fn fire_during_movement_phase_is_rejected() {
    let mut gs = fire_phase_state();
    gs.phase = Phase::Movement;
    let firer = place(
        &mut gs,
        SectionName::EgyptianArmy,
        0,
        1,
        HexCoord::new(17, 7),
    );
    place(&mut gs, SectionName::Hadendowa, 0, 1, HexCoord::new(16, 7));

    assert!(apply_effect(&mut gs, &direct_attack(firer, HexCoord::new(16, 7), vec![])).is_err());
}

// ---------------------------------------------------------------------------
// 2. Fire-sweep proptest
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 12,
        ..proptest::test_runner::Config::default()
    })]

    #[test]
    fn fire_at_hex_without_opponents_is_always_rejected(seed in 0u64..32) {
        let result = futures::executor::block_on(playthrough(
            Scenario::FallOfKhartoum,
            seed,
            seed_config(),
            Agents::random(),
        ));

        // Sweep reachable Direct Fire states: for every firer whose earlier
        // `can_fire_at` gates pass, every neighbor hex without opponent
        // units must be rejected as a fire target.
        let mut probed = 0usize;
        for gs in direct_fire_states(&result.events, 3) {
            for unit in gs.units.clone() {
                if !pre_occupancy_gates_pass(&gs, unit.id, FireKind::Direct) {
                    continue;
                }
                for hex in unit.position.neighbors() {
                    let has_enemy = gs.units.iter().any(|u| {
                        u.position == hex
                            && u.profile.identity.owner() != unit.profile.identity.owner()
                    });
                    if has_enemy {
                        continue;
                    }
                    let attack = FireAttack {
                        firing_player: unit.profile.identity.owner(),
                        phase: gs.phase,
                        kind: FireKind::Direct,
                        firers: vec![unit.id],
                        target_hex: hex,
                        factor_row: FireFactorRow::Row01to05,
                        modifiers: vec![FireModifier::AngloEgyptianDirectFire],
                    };
                    let attempt = apply_effect(
                        &mut gs.clone(),
                        &GameEffect::FireCombat { attack, roll: DieRoll::Three },
                    );
                    prop_assert!(
                        attempt.is_err(),
                        "seed {seed}: fire at non-enemy hex {hex:?} by {:?} was accepted",
                        unit.id
                    );
                    probed += 1;
                }
                if probed >= 24 {
                    break;
                }
            }
            if probed >= 24 {
                break;
            }
        }
        prop_assert!(
            probed > 0,
            "seed {seed}: reached Direct Fire but swept no non-enemy neighbors"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Mutation fuzzer
// ---------------------------------------------------------------------------

/// Build a random *mutated* effect from `gs`: real units, wrong targets,
/// wrong phases, wrong owners, tampered modifiers, duplicate deploys.
fn mutate(state: &GameState, rng: &mut BotRng) -> GameEffect {
    // An empty board (replay cut 0) has nothing to mutate around; phase
    // advance is the degenerate-but-legal candidate.
    if state.units.is_empty() {
        return GameEffect::AdvancePhase;
    }
    let ids: Vec<UnitId> = state.units.iter().map(|u| u.id).collect();
    let any = |rng: &mut BotRng| ids[(rng.roll_d10().value() as usize) % ids.len()];
    let near = |rng: &mut BotRng| {
        let u = &state.units[(rng.roll_d10().value() as usize) % state.units.len()];
        let mut hex = u.position;
        // Random small offset: stays near the board but usually off-unit.
        hex.q += (rng.roll_d10().value() as i32) % 3 - 1;
        hex.r += (rng.roll_d10().value() as i32) % 3 - 1;
        hex
    };
    let players = [Player::AngloEgyptian, Player::Dervish];
    let player = players[(rng.roll_d10().value() as usize) % 2];

    match rng.roll_d10().value() % 6 {
        // Fire by an arbitrary unit at an arbitrary nearby hex.
        0 | 1 => {
            let modifiers = if rng.roll_d10().value().is_multiple_of(2) {
                vec![FireModifier::AngloEgyptianDirectFire]
            } else {
                vec![] // tampered: likely modifier mismatch
            };
            GameEffect::FireCombat {
                attack: FireAttack {
                    firing_player: player,
                    phase: state.phase,
                    kind: FireKind::Direct,
                    firers: vec![any(rng)],
                    target_hex: near(rng),
                    factor_row: FireFactorRow::Row01to05,
                    modifiers,
                },
                roll: DieRoll::Three,
            }
        }
        // Howitzer with two arbitrary rolls.
        2 => GameEffect::HowitzerFire {
            attack: FireAttack {
                firing_player: player,
                phase: state.phase,
                kind: FireKind::Howitzer,
                firers: vec![any(rng)],
                target_hex: near(rng),
                factor_row: FireFactorRow::Row01to05,
                modifiers: vec![],
            },
            combat_results_table_roll: rng.roll_d10(),
            impact_roll: rng.roll_d10(),
        },
        // Move an arbitrary unit (possibly the opponent's) to a nearby hex.
        3 => GameEffect::MoveUnit {
            unit_id: any(rng),
            to: near(rng),
            cost: MovementPoints::new(1),
            path: vec![],
        },
        // Re-deploy an existing unit (duplicate deploy).
        4 => {
            let u = &state.units[(rng.roll_d10().value() as usize) % state.units.len()];
            GameEffect::DeployUnit(*u)
        }
        // Melee between two arbitrary hexes with arbitrary rosters.
        _ => GameEffect::MeleeCombat {
            attack: omdurman_rules::MeleeAttack {
                attacker_player: player,
                attacker_hex: near(rng),
                defender_hex: near(rng),
                attackers: vec![any(rng)],
                defenders: vec![any(rng)],
                attacker_modifiers: vec![],
                defender_modifiers: vec![],
            },
            attacker_roll: rng.roll_d10(),
            defender_roll: rng.roll_d10(),
        },
    }
}

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 12,
        ..proptest::test_runner::Config::default()
    })]

    #[test]
    fn mutated_effects_never_panic_and_accepted_ones_hold_invariants(seed in 0u64..32) {
        let result = futures::executor::block_on(playthrough(
            Scenario::FallOfKhartoum,
            seed,
            seed_config(),
            Agents::random(),
        ));
        let mut rng = BotRng::from_seed(seed ^ 0xada1f);
        for state in direct_fire_states(&result.events, 3)
            .into_iter()
            .chain(std::iter::once(replay_to(
                &result.events,
                (seed as usize * 37) % result.events.len().max(1),
            ))) {
            for _ in 0..16 {
                let eff = mutate(&state, &mut rng);
                // A mutated effect may be a *valid replay payload* (the app
                // sends whole effects over the wire), so it may even be a
                // legal action -- but whatever it does, it must either be
                // rejected or leave the engine consistent.
                let mut s = state.clone();
                if apply_effect(&mut s, &eff).is_ok() {
                    check_all_with_tribal(&s)
                        .map_err(proptest::test_runner::TestCaseError::fail)?;
                }
            }
        }
    }
}

//! Measurement experiment: quantifier-based (`kani::forall!`) vs loop-based
//! checking of the "no dangling tracker refs" post-condition, the property
//! proved by `effects::verification::ok_leaves_no_dangling_tracker_refs`.
//!
//! This module is *not* part of the proof suite: it is compiled only under
//! `--features kani,kani-quantifiers` together with cargo-kani's
//! `-Z quantifiers`, neither of which CI or `scripts/kani.sh` pass by
//! default. Run it with:
//!
//! ```sh
//! ./scripts/kani.sh -p omdurman-rules --features kani-quantifiers \
//!     -Z quantifiers --harness quantifier_experiment::
//! ```
//!
//! The trackers are seeded *after* a successful apply with ids known to be
//! live, so the `forall` antecedents are satisfiable for exactly the seeded
//! ids no matter what `SinkChain` did to the trackers, keeping the comparison
//! non-vacuous on both sides.
//!
//! Kani 0.67 quantifiers are `usize`-range only (`forall!(|i in (0,N)| ...)`,
//! constant bounds for the SAT backend), so the quantified harnesses range
//! over the index space of `UnitId::ALL` and index the array per
//! instantiation -- precisely the "array indexing" pattern the quantifier
//! documentation warns about (deep call stacks under the quantified
//! expression).

#![allow(dead_code)]

use crate::effects::{GameEffect, GameState, apply_effect};
use crate::{
    BattalionOrdinal, MovementAllowance, Phase, UnitId, UnitIdentity, UnitMovement, UnitPlacement,
    UnitProfile, UnitState, WeaponClass,
};
use omdurman_types::{
    BrigadeId, BrigadeNationality, DayNight, DervishTribe, HexCoord, Player, Scenario, UnitKind,
};

fn any_phase() -> Phase {
    let i: u8 = kani::any();
    kani::assume(i < 7);
    match i {
        0 => Phase::Setup,
        1 => Phase::Movement,
        2 => Phase::DefensiveFire(crate::FireSubPhase::DirectFire),
        3 => Phase::DefensiveFire(crate::FireSubPhase::MaximSecondAndHowitzer),
        4 => Phase::OffensiveFire(crate::FireSubPhase::DirectFire),
        5 => Phase::OffensiveFire(crate::FireSubPhase::MaximSecondAndHowitzer),
        _ => Phase::Melee,
    }
}

fn infantry_profile(player: Player) -> UnitProfile {
    let identity = match player {
        Player::AngloEgyptian => UnitIdentity::AngloEgyptianInfantry {
            brigade: BrigadeId {
                number: 1,
                nationality: BrigadeNationality::British,
            },
            battalion: BattalionOrdinal::First,
        },
        Player::Dervish => UnitIdentity::DervishTribal {
            tribe: DervishTribe::Baggara,
        },
    };
    UnitProfile {
        kind: UnitKind::Infantry {
            fire: 4,
            melee: 5,
            movement: 8,
        },
        identity,
        weapon: WeaponClass::Rifles,
        fire: Some(crate::FireFactor::Four),
        melee: Some(crate::MeleeFactor::Five),
        movement: UnitMovement::Land(MovementAllowance::Eight),
    }
}

/// Mirrors `effects::verification::any_state`: symbolic phase / player /
/// latches, two concrete units at symbolic hexes.
fn any_state() -> GameState {
    let mut state = GameState::new(Scenario::Campaign);
    state.phase = any_phase();
    state.active_player = if kani::any() {
        Player::AngloEgyptian
    } else {
        Player::Dervish
    };
    state.day_night = if kani::any() {
        DayNight::Day
    } else {
        DayNight::Night
    };
    state.dervish_deserted = kani::any();
    state.setup_ready_ae = kani::any();
    state.setup_ready_dervish = kani::any();
    for (i, id) in [UnitId::ALL[0], UnitId::ALL[1]].iter().enumerate() {
        let owner = if i == 0 {
            Player::AngloEgyptian
        } else {
            Player::Dervish
        };
        let q: i32 = kani::any();
        let r: i32 = kani::any();
        kani::assume(q >= -2 && q <= 2);
        kani::assume(r >= -2 && r <= 2);
        state.units.push(UnitPlacement {
            id: *id,
            position: HexCoord::new(q, r),
            profile: infantry_profile(owner),
            state: UnitState::default(),
        });
    }
    state
}

/// The loop formulation under measurement: iterate exactly the tracker
/// contents.
fn dangling_tracker_check_loops(state: &GameState) {
    for id in &state.units_fired_this_phase {
        assert!(state.find_unit(*id).is_some());
    }
    for id in &state.units_fired_at_this_phase {
        assert!(state.find_unit(*id).is_some());
    }
}

/// The quantifier formulation under measurement: quantify over the entire
/// `UnitId::ALL` index space, membership in either tracker as the antecedent.
///
/// Two CBMC/Kani codegen invariants shape this formulation:
/// - "quantifier must not contain loops" (goto_clean_expr): the tracker
///   scans are statically unrolled to a fixed capacity with a never-matching
///   sentinel pad;
/// - "Detected recursions in the usage of quantifiers" (kani-compiler): no
///   function call may appear in the body, so discriminant codes
///   (`fieldless enum as usize`) stand in for `UnitId` equality and the body
///   is pure expressions over by-value captures.
const CAP: usize = 3;
const PAD: usize = usize::MAX;

fn dangling_tracker_check_forall(state: &GameState) {
    let mut f = [PAD; CAP];
    for (slot, id) in state.units_fired_this_phase.iter().enumerate() {
        if slot < CAP {
            f[slot] = *id as usize;
        }
    }
    let mut g = [PAD; CAP];
    for (slot, id) in state.units_fired_at_this_phase.iter().enumerate() {
        if slot < CAP {
            g[slot] = *id as usize;
        }
    }
    let mut l = [PAD; CAP];
    for (slot, u) in state.units.iter().enumerate() {
        if slot < CAP {
            l[slot] = u.id as usize;
        }
    }
    let [f0, f1, f2] = f;
    let [g0, g1, g2] = g;
    let [l0, l1, l2] = l;
    let n = UnitId::ALL.len();
    assert!(kani::forall!(|i in (0, n)| {
        let tracked = f0 == i || f1 == i || f2 == i || g0 == i || g1 == i || g2 == i;
        let is_live = l0 == i || l1 == i || l2 == i;
        !tracked || is_live
    }));
}

/// Control: identical to the suite harness
/// `ok_leaves_no_dangling_tracker_refs` (empty trackers).
#[kani::proof]
#[kani::unwind(14)]
fn quantifier_experiment_loop_empty() {
    let mut state = any_state();
    if apply_effect(&mut state, &GameEffect::SinkChain).is_ok() {
        dangling_tracker_check_loops(&state);
    }
}

/// Same property and state as `loop_empty`, `forall!` instead of loops.
#[kani::proof]
#[kani::unwind(14)]
fn quantifier_experiment_forall_empty() {
    let mut state = any_state();
    if apply_effect(&mut state, &GameEffect::SinkChain).is_ok() {
        dangling_tracker_check_forall(&state);
    }
}

/// Loop formulation with both trackers seeded (post-apply) with ids that are
/// in fact on the board, so the checks are non-vacuous.
#[kani::proof]
#[kani::unwind(14)]
fn quantifier_experiment_loop_seeded() {
    let mut state = any_state();
    if apply_effect(&mut state, &GameEffect::SinkChain).is_ok() {
        let a = state.units[0].id;
        let b = state.units[1].id;
        state.units_fired_this_phase.push(a);
        state.units_fired_at_this_phase.push(b);
        dangling_tracker_check_loops(&state);
    }
}

/// Quantifier formulation with the same seeding as `loop_seeded`: 2 of ~500
/// `UnitId` instantiations have a satisfiable antecedent.
#[kani::proof]
#[kani::unwind(14)]
fn quantifier_experiment_forall_seeded() {
    let mut state = any_state();
    if apply_effect(&mut state, &GameEffect::SinkChain).is_ok() {
        let a = state.units[0].id;
        let b = state.units[1].id;
        state.units_fired_this_phase.push(a);
        state.units_fired_at_this_phase.push(b);
        dangling_tracker_check_forall(&state);
    }
}

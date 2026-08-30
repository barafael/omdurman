//! Semantic game effects -- every mutation passes through [`apply_effect`]
//! (rulebook §4, §5, §6, §7, §8, §10).
//!
//! Each [`GameEffect`] carries *all* information (including pre-rolled die
//! values) needed to apply it deterministically.  The processor validates
//! the effect against the current [`GameState`] and, if legal, mutates the
//! state in place.  Network replay works because every peer receives the
//! identical effect with the identical roll.

use serde::{Deserialize, Serialize};
// `BTreeMap`, not `HashMap`, for the two per-turn tracking maps: `GameState` is
// replicated across peers and serialised into the canonical event record, so an
// ordered map is the honest choice for iteration and encoding stability. It also
// keeps `GameState` constructible under Kani -- `HashMap`'s `RandomState` seeds
// itself from the OS RNG via a `syscall` the model checker cannot execute, which
// made every `apply_effect` proof undecidable (see the `verification` module).
use std::collections::BTreeMap;
use tracing::debug;

use crate::combat_results_table::{FireFactorRow, combat_results_table};
use crate::howitzer_scatter::{ScatterHexDirection, howitzer_scatter};
use crate::range_effects::{ae_range_effects, dervish_range_effects};
use crate::turn_summary::{TurnEventRecord, TurnSummary};
use crate::turn_track::{TurnEvent, scenario_turn};
use crate::{
    CampaignVictoryLevel, CombatResult, DemolitionTarget, DieRoll, FireAttack, FireFactor,
    FireKind, FireModifier, FireSubPhase, GameTurnIndex, HexCoord, HexDistance,
    HistoricalVictoryLevel, MeleeAttack, MeleeModifier, MovementAllowance, MovementPoints, Phase,
    UnitId, UnitPlacement, VictoryLedger, VictoryPoints, VpEvent, VpSource, WeaponClass, ZocReason,
};
use omdurman_types::{DayNight, DervishTribe, HexsideKind, HexsideRef, Player, Scenario, UnitKind};

use crate::FriendliesAction;
use crate::TransportState;
use crate::board::BoardInfo;
use crate::{ChainPlacement, MinePlacement, OptionalRule};

// The engine used to be a single ~12k-line file; it is now split by concern
// (effect vocabulary, errors, observations, state+validators, the apply_effect
// dispatcher, and per-domain effect application: movement / fire / melee /
// setup / river / victory). Each submodule pulls the shared scope in via
// `use super::*;` and this root re-exports every item, so the flat public API
// (`omdurman_rules::effects::{GameState, GameEffect, RuleError, ...}`) is
// unchanged.

mod dispatch;
mod effect;
mod error;
mod fire;
mod melee;
mod movement;
mod observation;
mod river;
mod setup;
mod state;
mod victory;

pub use dispatch::*;
pub use effect::*;
pub use error::*;
pub use fire::*;
pub use melee::*;
pub use movement::*;
pub use observation::*;
pub use river::*;
pub use setup::*;
pub use state::*;
pub use victory::*;

#[cfg(test)]
mod tests;

/// Kani proof harnesses for [`apply_effect`] over a *bounded* symbolic
/// [`GameState`] (`cargo kani`, see `scripts/kani.sh`).
///
/// A fully symbolic `GameState` is out of reach: 33 fields, 16 of them `Vec`s,
/// a 228-variant `UnitId`, and a `BoardInfo` of six hashed maps. The generator
/// below builds the smallest state that still reaches the interesting
/// validation paths:
///
/// * `BoardInfo::default()` stays **concrete**. It is documented as
///   rule-neutral ("no map loaded": every lookup returns the neutral answer),
///   so it removes all six hashed containers without disabling any rule these
///   proofs exercise.
/// * `UnitId` is never `kani::any()` -- proofs index a fixed 3-element array of
///   concrete ids. It has 228 fieldless variants, so two symbolic ids would be
///   a ~52k-way case split.
/// * Hexes come from a tiny window and unit counts are capped at 2
///   (`STACKING_LIMIT` is 4, so this stays well inside the legal domain).
///
/// The central property is **atomicity**: if an effect returns `Err`, the
/// rejected state must be unchanged. Peers apply events only on the
/// host-sequenced echo, so a peer that accepts an effect and one that rejects
/// it must not diverge -- a partial mutation on the rejecting side is a desync,
/// and a rejected effect is never retried. (The three sites that violated this
/// are fixed; these harnesses are the standing guard.)
///
/// `observations`, `turn_events` and `turn_summaries` are append-only audit
/// logs whose `Vec<String>` payloads are expensive for the solver, so the
/// snapshot compares `observations.len()` rather than their contents.
///
/// # What made these tractable
///
/// Two blockers had to go, both diagnosed by measurement rather than guessed:
///
/// 1. **The dispatcher.** Routing through [`apply_effect`] makes even a 10-line
///    effect unsolvable: it is a 26-arm `match`, and CBMC explores every arm's
///    callee before it can reason about any one of them. The harnesses below
///    call the per-effect `apply_*` function directly instead, which turned a
///    >35-minute hang into a ~2-second run. The two harnesses that genuinely
///    test `apply_effect` itself (the `game_over` gate, tracker pruning) still
///    go through it, and pick `SinkChain` as the cheapest arm.
/// 2. **`HashMap`.** `GameState`'s two per-turn tracking maps, and the four
///    printed tables in `tables_data`, used to be `std::collections::HashMap`.
///    That pulled `hashbrown`'s probe loop into every proof -- a standalone
///    one-insert probe was still unwinding it at iteration 605 -- plus
///    `RandomState`'s `getrandom` seeding, which bottoms out in a `syscall`
///    Kani cannot model. They are all `BTreeMap` now, which is independently
///    the right choice for state replicated across peers and encoded into the
///    canonical event record.
///
/// # Remaining gap
///
/// The harnesses now *complete* in ~2s instead of hanging, and the atomicity
/// assertions themselves are never violated -- but they report `UNDETERMINED`
/// rather than `SUCCESS`. `std`'s `getrandom` is still linked into the goto
/// binary (reached through `ron`/panic-formatting paths that are dead at
/// runtime but present in the CFG), and CBMC halts the whole path on the
/// unmodellable `syscall` before reaching our assertion. Every one of the 8
/// reported failures is inside `std`, none in engine code.
///
/// So these currently prove *no counterexample was found*, not *none exists*.
/// The three atomicity bugs they were written for are fixed and covered by
/// ordinary regression tests (`rejected_*` in the `tests` module below), which
/// is what actually guards them today. Closing this gap needs the remaining
/// `std` RNG/IO paths stubbed out of the harness build -- tracked as future
/// work rather than claimed as proven.
#[cfg(kani)]
mod verification {
    use super::*;
    use crate::{
        BattalionOrdinal, UnitIdentity, UnitMovement, UnitProfile, UnitState, WeaponClass,
    };
    use omdurman_types::{BrigadeId, BrigadeNationality, DervishTribe};

    /// Three concrete unit ids.
    const IDS: [UnitId; 3] = [UnitId::ALL[0], UnitId::ALL[1], UnitId::ALL[2]];

    /// A hex from a small window around the origin.
    fn any_hex() -> HexCoord {
        let q: i32 = kani::any();
        let r: i32 = kani::any();
        kani::assume(q >= -2 && q <= 2);
        kani::assume(r >= -2 && r <= 2);
        HexCoord::new(q, r)
    }

    fn any_player() -> Player {
        if kani::any() {
            Player::AngloEgyptian
        } else {
            Player::Dervish
        }
    }

    fn any_phase() -> Phase {
        let i: u8 = kani::any();
        kani::assume(i < 7);
        match i {
            0 => Phase::Setup,
            1 => Phase::Movement,
            2 => Phase::DefensiveFire(FireSubPhase::DirectFire),
            3 => Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer),
            4 => Phase::OffensiveFire(FireSubPhase::DirectFire),
            5 => Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
            _ => Phase::Melee,
        }
    }

    fn any_die_roll() -> DieRoll {
        let i: usize = kani::any();
        kani::assume(i < DieRoll::ALL.len());
        DieRoll::ALL[i]
    }

    /// A concrete infantry profile for the given owner. These proofs are about
    /// control flow through `apply_effect`, not factor arithmetic.
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
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        }
    }

    /// A bounded symbolic state: symbolic phase / active player / latches, up
    /// to two units at symbolic hexes, and an empty rule-neutral board.
    fn any_state() -> GameState {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = any_phase();
        state.active_player = any_player();
        state.day_night = if kani::any() {
            DayNight::Day
        } else {
            DayNight::Night
        };
        state.dervish_deserted = kani::any();
        state.setup_ready_ae = kani::any();
        state.setup_ready_dervish = kani::any();

        let n: usize = kani::any();
        kani::assume(n <= 2);
        for (i, id) in IDS.iter().take(n).enumerate() {
            let owner = if i == 0 {
                Player::AngloEgyptian
            } else {
                Player::Dervish
            };
            state.units.push(UnitPlacement {
                id: *id,
                position: any_hex(),
                profile: infantry_profile(owner),
                state: UnitState::default(),
            });
        }
        state
    }

    /// The rule-relevant part of a state, for atomicity comparison.
    ///
    /// `GameState` has no `PartialEq`, and deriving one would pull in the
    /// audit-log `Vec<String>`s. This captures the fields an effect could
    /// legitimately mutate, which is what atomicity is about.
    #[derive(PartialEq)]
    struct Snapshot {
        phase: Phase,
        active_player: Player,
        current_turn: u8,
        game_over: bool,
        dervish_deserted: bool,
        setup_ready_ae: bool,
        setup_ready_dervish: bool,
        unit_count: usize,
        unit_positions: [Option<HexCoord>; 3],
        fired: usize,
        fired_at: usize,
        vacated: usize,
        pending_melee: bool,
        mines: usize,
        chain: bool,
        zariba: usize,
        pending_demolitions: usize,
        observations: usize,
    }

    fn snapshot(state: &GameState) -> Snapshot {
        let mut unit_positions = [None; 3];
        for (i, id) in IDS.iter().enumerate() {
            unit_positions[i] = state.find_unit(*id).map(|u| u.position);
        }
        Snapshot {
            phase: state.phase,
            active_player: state.active_player,
            current_turn: state.current_turn.value(),
            game_over: state.game_over,
            dervish_deserted: state.dervish_deserted,
            setup_ready_ae: state.setup_ready_ae,
            setup_ready_dervish: state.setup_ready_dervish,
            unit_count: state.units.len(),
            unit_positions,
            fired: state.units_fired_this_phase.len(),
            fired_at: state.units_fired_at_this_phase.len(),
            vacated: state.vacated_by_combat.len(),
            pending_melee: state.pending_melee.is_some(),
            mines: state.mines.len(),
            chain: state.chain.is_some(),
            zariba: state.zariba_hexsides.len(),
            pending_demolitions: state.pending_demolitions.len(),
            observations: state.observations.len(),
        }
    }

    /// Run `apply` and assert a rejection left the state untouched.
    fn assert_atomic<F>(state: &mut GameState, apply: F)
    where
        F: FnOnce(&mut GameState) -> Result<(), RuleError>,
    {
        let before = snapshot(state);
        if apply(state).is_err() {
            assert!(
                snapshot(state) == before,
                "effect mutated state on the error path"
            );
        }
    }

    // -- Rung 1: payload-free / scalar effects -----------------------------

    // These call the per-effect `apply_*` functions directly rather than going
    // through `apply_effect`. That matters for tractability, not for the
    // property: `apply_effect` is a 26-arm dispatcher, and CBMC explores every
    // arm's callee before it can reason about any one of them, so routing
    // through it makes even a 10-line effect unsolvable. Targeting the arm
    // keeps the reachable code proportional to the effect under test.

    #[kani::proof]
    fn sink_chain_is_atomic() {
        let mut state = any_state();
        assert_atomic(&mut state, apply_sink_chain);
    }

    #[kani::proof]
    fn confirm_setup_ready_is_atomic() {
        let mut state = any_state();
        let player = any_player();
        assert_atomic(&mut state, |s| apply_confirm_setup_ready(s, player));
    }

    #[kani::proof]
    fn recover_unit_is_atomic() {
        let mut state = any_state();
        let i: usize = kani::any();
        kani::assume(i < IDS.len());
        assert_atomic(&mut state, |s| apply_recover_unit(s, IDS[i]));
    }

    #[kani::proof]
    fn place_mine_is_atomic() {
        let mut state = any_state();
        let hex = any_hex();
        assert_atomic(&mut state, |s| apply_place_mine(s, hex));
    }

    // -- Rung 2: one-way latches -------------------------------------------
    //
    // Documented as monotonic. Unlike atomicity these are expected to hold
    // today, so they anchor the suite: a regression means an effect learned to
    // un-set a latch.

    #[kani::proof]
    fn game_over_is_monotonic() {
        let mut state = any_state();
        state.game_over = true;
        let _ = apply_effect(&mut state, &GameEffect::SinkChain);
        assert!(state.game_over, "game_over was cleared");
    }

    #[kani::proof]
    fn setup_ready_latches_are_monotonic() {
        let mut state = any_state();
        let before_ae = state.setup_ready_ae;
        let before_dervish = state.setup_ready_dervish;
        let player = any_player();
        let _ = apply_confirm_setup_ready(&mut state, player);
        assert!(state.setup_ready_ae >= before_ae);
        assert!(state.setup_ready_dervish >= before_dervish);
    }

    #[kani::proof]
    fn dervish_desertion_latch_is_monotonic() {
        let mut state = any_state();
        state.dervish_deserted = true;
        let _ = apply_dervish_desertion(&mut state, any_die_roll(), &[]);
        assert!(state.dervish_deserted, "desertion latch was cleared");
    }

    // -- Rung 3: post-conditions of a successful apply ---------------------

    /// After a successful apply no per-phase tracker may name a unit that has
    /// left the board -- the `prune_dead_trackers` post-condition.
    #[kani::proof]
    fn ok_leaves_no_dangling_tracker_refs() {
        let mut state = any_state();
        if apply_effect(&mut state, &GameEffect::SinkChain).is_ok() {
            for id in &state.units_fired_this_phase {
                assert!(state.find_unit(*id).is_some());
            }
            for id in &state.units_fired_at_this_phase {
                assert!(state.find_unit(*id).is_some());
            }
        }
    }

    /// Once `game_over` is set every later effect is rejected up front and the
    /// state is frozen.
    #[kani::proof]
    fn game_over_is_absorbing() {
        let mut state = any_state();
        state.game_over = true;
        let before = snapshot(&state);
        assert!(apply_effect(&mut state, &GameEffect::SinkChain).is_err());
        assert!(snapshot(&state) == before);
    }

    // -- Rung 4: ResolveMelee ----------------------------------------------

    /// §7.5: a declared melee carries its pre-rolled dice until it resolves, so
    /// a mistimed `ResolveMelee` must not consume it. `apply_resolve_melee`
    /// used to `take()` `pending_melee` *before* delegating to
    /// `apply_melee_combat`, which rejects a wrong phase -- the same silent loss
    /// `advance_phase` already guards ("audit: 76 declared melees vanished this
    /// way").
    // §7.5
    #[kani::proof]
    fn resolve_melee_is_atomic() {
        let mut state = any_state();
        kani::assume(state.units.len() == 2);
        state.pending_melee = Some(PendingMelee {
            attack: MeleeAttack {
                attacker_player: Player::AngloEgyptian,
                attacker_hex: state.units[0].position,
                defender_hex: state.units[1].position,
                attackers: vec![IDS[0]],
                defenders: vec![IDS[1]],
                attacker_modifiers: vec![],
                defender_modifiers: vec![],
            },
            attacker_roll: any_die_roll(),
            defender_roll: any_die_roll(),
        });
        assert_atomic(&mut state, apply_resolve_melee);
    }

    // -- Rung 5: AdvancePhase ----------------------------------------------

    /// A rejected `AdvancePhase` must not drop the §6.82/§7.6
    /// advance-after-combat windows. `advance_phase` used to clear
    /// `vacated_by_combat` before the `MeleePendingResolution` /
    /// `DesertionRollRequired` guards.
    ///
    /// The heaviest harness here: on the accept path `advance_phase` runs the
    /// whole turn-end cascade (`end_player_turn` -> demolitions -> recovery ->
    /// `advance_game_turn` -> `finish_game`), whose loops iterate `state.units`
    /// and the victory ledger. The generator caps units at 2, so a small unwind
    /// bound covers every reachable iteration; without one CBMC does not return.
    #[kani::proof]
    #[kani::unwind(6)]
    fn advance_phase_is_atomic() {
        let mut state = any_state();
        kani::assume(!state.units.is_empty());
        state
            .vacated_by_combat
            .insert(state.units[0].position, vec![IDS[0]]);
        assert_atomic(&mut state, advance_phase);
    }
}

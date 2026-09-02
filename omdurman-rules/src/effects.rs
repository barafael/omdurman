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
/// * `BoardInfo::default()` stays **concrete** and carries a deterministic
///   hasher (`BuildHasherDefault<DefaultHasher>`, see `board.rs`): documented
///   as rule-neutral ("no map loaded": every lookup returns the neutral
///   answer), and free of `RandomState`'s `getrandom` seeding, which bottoms
///   out in a `syscall` Kani cannot model.
/// * `UnitId` is never `kani::any()` -- proofs index a fixed 3-element array of
///   concrete ids. It has 228 fieldless variants, so two symbolic ids would be
///   a ~52k-way case split.
/// * Hexes come from a tiny window and the state carries exactly two units
///   (one per side), with a **concrete** loop trip count -- a symbolic
///   `take(n)` keeps CBMC unrolling the iterator protocol past any
///   reasonable unwind bound (`STACKING_LIMIT` is 4, so two units stay well
///   inside the legal domain).
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
/// 2. **`HashMap`/`RandomState`.** `GameState`'s two per-turn tracking maps,
///    and the four printed tables in `tables_data`, used to be
///    `std::collections::HashMap`. That pulled `hashbrown`'s probe loop into
///    every proof -- a standalone one-insert probe was still unwinding it at
///    iteration 605 -- plus `RandomState`'s `getrandom` seeding, which bottoms
///    out in a `syscall` Kani cannot model. They are all `BTreeMap` now,
///    which is independently the right choice for state replicated across
///    peers and encoded into the canonical event record. The migration missed
///    one hashed container: `BoardInfo`'s six `IndexMap`/`IndexSet` fields
///    default their hasher to `RandomState`, and `BoardInfo::default()` runs
///    on every `GameState::new` -- a *live* path, not dead CFG -- so every
///    harness still halted on the `syscall` and reported `UNDETERMINED`.
///    Giving `BoardInfo` the deterministic `BuildHasherDefault<DefaultHasher>`
///    (fixed-key SipHash, no OS RNG) closed the gap for good.
/// 3. **Loops with symbolic bounds and property-neutral cascades.** An
///    iterator over a symbolically-`take(n)`'d slice unrolls past any
///    reasonable unwind bound, and CBMC explores *paths*, so a symbolic
///    phase (7 arms) or unit count multiplies the whole dispatcher into the
///    step count. Unit generation is concrete (two units, fixed trip count),
///    harnesses pin the phase where the property lives on one arm, and
///    property-neutral cascades are stubbed (`end_player_turn` under
///    `advance_phase`, `advance_phase` under the latch harness,
///    `apply_melee_combat` under `ResolveMelee`) -- hence the `-Z stubbing`
///    and `--features kani` (logging gated out) defaults in `scripts/kani.sh`.
///
/// # Status
///
/// All harnesses verify `SUCCESS` under `./scripts/kani.sh -p omdurman-types
/// -p omdurman-rules` (63 harnesses across both crates, Kani 0.67). The three
/// atomicity bugs these proofs were written for are fixed, guarded here, and
/// covered by ordinary regression tests (`rejected_*` in the `tests` module
/// below).
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

    /// A bounded symbolic state: symbolic phase / active player / latches,
    /// exactly two units (one per side) at symbolic hexes, and an empty
    /// rule-neutral board. The unit count is deliberately *concrete*: a
    /// symbolic `take(n)` keeps CBMC unrolling the iterator protocol past any
    /// reasonable unwind bound, multiplying every downstream formula; a
    /// concrete trip count unrolls exactly twice and stops. The atomicity /
    /// monotonicity properties under proof do not depend on the unit count,
    /// and both sides being present is the interesting case for them anyway.
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

        for (i, id) in IDS.iter().take(2).enumerate() {
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

    /// Replacement for the real `end_player_turn` cascade in the
    /// `advance_phase_is_atomic` harness (see the harness doc for why).
    fn stub_end_player_turn(_state: &mut GameState) -> Result<(), RuleError> {
        Ok(())
    }

    /// Replacement for `advance_phase` in the latch-monotonicity harness: the
    /// auto-advance when both factions are ready pulls the whole dispatcher
    /// CFG into the proof, and `advance_phase` never writes either latch (the
    /// only two writers are the `= true` arms in `apply_confirm_setup_ready`
    /// itself), so `Ok(())` is property-neutral here.
    fn stub_advance_phase_ok(_state: &mut GameState) -> Result<(), RuleError> {
        Ok(())
    }

    /// Replacement for `apply_melee_combat` in the `ResolveMelee` harness: the
    /// combat resolution (CRT, factors, casualties) is not what the §7.5
    /// atomicity proof is about, and its CFG alone blows the memory budget.
    /// Both rejection paths (`NoMeleePending`, `WrongPhase`) precede it; the
    /// stubbed `Ok` only simplifies the committed-resolution path.
    fn stub_apply_melee_combat_ok(
        _state: &mut GameState,
        _attack: &MeleeAttack,
        _attacker_roll: DieRoll,
        _defender_roll: DieRoll,
    ) -> Result<(), RuleError> {
        Ok(())
    }

    // -- Rung 1: payload-free / scalar effects -----------------------------

    // These call the per-effect `apply_*` functions directly rather than going
    // through `apply_effect`. That matters for tractability, not for the
    // property: `apply_effect` is a 26-arm dispatcher, and CBMC explores every
    // arm's callee before it can reason about any one of them, so routing
    // through it makes even a 10-line effect unsolvable. Targeting the arm
    // keeps the reachable code proportional to the effect under test.

    #[kani::proof]
    #[kani::unwind(14)]
    fn sink_chain_is_atomic() {
        let mut state = any_state();
        assert_atomic(&mut state, apply_sink_chain);
    }

    /// Pinned to the Setup phase: every other phase rejects in
    /// `require_setup_phase` before any write, so those paths are atomic by
    /// construction and only multiply CBMC's path count. The Setup path is
    /// the one that can mutate (the latch plus the auto-advance), which is
    /// what the atomicity proof is about.
    #[kani::proof]
    #[kani::unwind(14)]
    fn confirm_setup_ready_is_atomic() {
        let mut state = any_state();
        state.phase = Phase::Setup;
        let player = any_player();
        assert_atomic(&mut state, |s| apply_confirm_setup_ready(s, player));
    }

    #[kani::proof]
    #[kani::unwind(14)]
    fn recover_unit_is_atomic() {
        let mut state = any_state();
        let i: usize = kani::any();
        kani::assume(i < IDS.len());
        assert_atomic(&mut state, |s| apply_recover_unit(s, IDS[i]));
    }

    #[kani::proof]
    #[kani::unwind(14)]
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
    #[kani::unwind(14)]
    fn game_over_is_monotonic() {
        let mut state = any_state();
        state.game_over = true;
        let _ = apply_effect(&mut state, &GameEffect::SinkChain);
        assert!(state.game_over, "game_over was cleared");
    }

    #[kani::proof]
    #[kani::unwind(14)]
    #[kani::stub(advance_phase, stub_advance_phase_ok)]
    fn setup_ready_latches_are_monotonic() {
        let mut state = any_state();
        // Monotonicity for a bool latch is exactly "once set, never cleared":
        // false >= false is vacuous, and the only writers in the engine are
        // the two `= true` arms of `apply_confirm_setup_ready`. Starting from
        // set latches proves the non-vacuous direction. The auto-advance into
        // `advance_phase` is stubbed (see `stub_advance_phase_ok`): the
        // dispatcher CFG is what blows the memory budget, and it never writes
        // either latch.
        state.setup_ready_ae = true;
        state.setup_ready_dervish = true;
        let player = any_player();
        let _ = apply_confirm_setup_ready(&mut state, player);
        assert!(state.setup_ready_ae, "AE setup-ready latch was cleared");
        assert!(
            state.setup_ready_dervish,
            "Dervish setup-ready latch was cleared"
        );
    }

    #[kani::proof]
    #[kani::unwind(14)]
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
    #[kani::unwind(14)]
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
    #[kani::unwind(14)]
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
    #[kani::unwind(14)]
    #[kani::stub(apply_melee_combat, stub_apply_melee_combat_ok)]
    fn resolve_melee_is_atomic() {
        let mut state = any_state();
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
    /// Two scale reductions keep this tractable. First, the `end_player_turn`
    /// cascade (demolitions -> recovery -> turn advance -> snapshot) that the
    /// accept path reaches from the Melee phase is stubbed to `Ok(())`: it is
    /// not what the proof is about, and it blows CBMC's SAT instance past the
    /// memory budget. Second, the phase is pinned to `Melee` with a declared
    /// melee pending, so the symbolic 7-arm phase match collapses to the one
    /// arm that carries the §7.5 guard -- CBMC explores paths, not formulas,
    /// and a symbolic phase multiplies every arm into the step count. That
    /// guard is representative: all three rejection guards (`MeleePending`,
    /// `DesertionRollRequired`, `setup_complete`) precede the mutation section,
    /// so the ordering this proof establishes covers them alike, and moving
    /// `vacated_by_combat.clear()` above *any* of them trips this harness.
    /// The Setup-side guard additionally needs an under-deployed board, which
    /// `any_state` (two units, one per side) cannot produce.
    #[kani::proof]
    #[kani::unwind(14)]
    #[kani::stub(end_player_turn, stub_end_player_turn)]
    fn advance_phase_is_atomic() {
        let mut state = any_state();
        state.phase = Phase::Melee;
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
        state
            .vacated_by_combat
            .insert(state.units[0].position, vec![IDS[0]]);
        assert_atomic(&mut state, advance_phase);
    }

    // -- The stacking law (§5.51-5.53) -------------------------------------
    //
    // `stacking_rule` is a pure function of an explicit occupant list, so
    // these harnesses need no `GameState` (and no RNG path): two symbolic
    // placements suffice to prove the law's load-bearing properties. A
    // regression here is exactly the class of bug that let a Hadendowa
    // counter stack onto a Dervish gun for the whole life of the project.

    use crate::{
        BritishLeader, DervishLeader, GunboatId, OldGunboat, StackingError, UnitPlacement,
    };

    /// Every Dervish tribe (§2.31 roster).
    const TRIBES: [DervishTribe; 10] = [
        DervishTribe::Baggara,
        DervishTribe::Jaalin,
        DervishTribe::Danagla,
        DervishTribe::Kehena,
        DervishTribe::Degheim,
        DervishTribe::Hadendowa,
        DervishTribe::Mulazmin,
        DervishTribe::Jehadia,
        DervishTribe::Taiasha,
        DervishTribe::IsaZachneih,
    ];

    /// A symbolic identity covering every stacking-relevant shape: Dervish
    /// tribal (any tribe), Dervish artillery, Dervish leader/fort/gunboat,
    /// and Anglo-Egyptian infantry/leader/gunboat.
    fn any_stack_identity() -> UnitIdentity {
        // Index layout: 0..=9 tribal tribes, 10 artillery, 11 Dervish
        // leader, 12 Dervish fort, 13 Dervish gunboat, 14 AE infantry,
        // 15 AE leader, 16 AE gunboat.
        let i: usize = kani::any();
        kani::assume(i < 17);
        match i {
            0..=9 => UnitIdentity::DervishTribal { tribe: TRIBES[i] },
            10 => UnitIdentity::DervishArtillery,
            11 => UnitIdentity::DervishLeader(DervishLeader::SheikElDin),
            12 => UnitIdentity::DervishFort,
            13 => UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1)),
            14 => UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::British,
                },
                battalion: BattalionOrdinal::First,
            },
            15 => UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener),
            _ => UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(OldGunboat::Tamai)),
        }
    }

    /// A kind/weapon consistent with `identity` (the stacking law looks at
    /// the kind's leader/gunboat shape and the identity's owner/group only).
    fn stack_profile(identity: UnitIdentity) -> UnitProfile {
        use omdurman_types::UnitKind::*;
        let kind = match identity {
            UnitIdentity::DervishTribal { .. } | UnitIdentity::AngloEgyptianInfantry { .. } => {
                Infantry {
                    fire: 3,
                    melee: 6,
                    movement: 9,
                }
            }
            UnitIdentity::DervishArtillery => Artillery {
                fire: 3,
                melee: 0,
                movement: 0,
            },
            UnitIdentity::DervishLeader(_) => DervishLeader {
                fire: 1,
                melee: 1,
                movement: 15,
            },
            UnitIdentity::DervishFort => Fort { fire: 2, melee: 4 },
            UnitIdentity::DervishGunboat(_) | UnitIdentity::AngloEgyptianGunboat(_) => Gunboat {
                fire: 0,
                upstream: 10,
                downstream: 16,
            },
            UnitIdentity::AngloEgyptianLeader(_) => BritishLeader { movement: 8 },
            _ => Infantry {
                fire: 4,
                melee: 5,
                movement: 8,
            },
        };
        UnitProfile {
            kind,
            identity,
            weapon: WeaponClass::Melee,
            fire: None,
            melee: None,
            movement: UnitMovement::Immobile,
        }
    }

    fn stack_unit(identity: UnitIdentity, id: UnitId) -> UnitPlacement {
        UnitPlacement {
            id,
            position: HexCoord::new(0, 0),
            profile: stack_profile(identity),
            state: UnitState::default(),
        }
    }

    /// §5.51: two units of opposite factions share a hex exactly when the
    /// leader exception -- a lone Anglo-Egyptian *leader*, who never blocks
    /// (a Dervish unit arriving on his hex eliminates him instead) -- does
    /// not apply. `EnemyCohabitation` fires on the first rule of the law, so
    /// the biconditional is exact.
    // §5.51
    #[kani::proof]
    fn stacking_rule_cohabitation_is_exact() {
        let a = stack_unit(any_stack_identity(), UnitId::ALL[0]);
        let b = stack_unit(any_stack_identity(), UnitId::ALL[1]);
        let occupants = [&a, &b];
        let mixed_factions = a.profile.identity.owner() != b.profile.identity.owner();
        let neither_is_ae_leader = !matches!(a.profile.kind, UnitKind::BritishLeader { .. })
            && !matches!(b.profile.kind, UnitKind::BritishLeader { .. });
        let expected = mixed_factions && neither_is_ae_leader;
        assert_eq!(
            stacking_rule(&occupants) == Err(StackingError::EnemyCohabitation),
            expected
        );
    }

    /// §5.52: two Dervish non-leader, non-gunboat units stack exactly when
    /// they belong to the same stacking group (tribe, or the artillery as
    /// its own group). With both units Dervish, group purity is the only
    /// rule that can fire, so the biconditional is exact.
    // §5.52
    #[kani::proof]
    fn stacking_rule_group_purity_is_exact() {
        // Symbolic group identity: index 0..=9 is a tribe, 10 is artillery.
        let group_of = |i: usize| {
            if i == 10 {
                UnitIdentity::DervishArtillery
            } else {
                UnitIdentity::DervishTribal { tribe: TRIBES[i] }
            }
        };
        let i: usize = kani::any();
        let j: usize = kani::any();
        kani::assume(i <= 10);
        kani::assume(j <= 10);
        let a = stack_unit(group_of(i), UnitId::ALL[0]);
        let b = stack_unit(group_of(j), UnitId::ALL[1]);
        let same_group = a.profile.identity.dervish_stacking_group()
            == b.profile.identity.dervish_stacking_group();
        let verdict = stacking_rule(&[&a, &b]);
        if same_group {
            assert_eq!(verdict, Ok(()));
        } else {
            assert_eq!(verdict, Err(StackingError::DervishTribeMix));
        }
    }

    /// The stacking law is symmetric in its occupants: whether `a` may join
    /// `b`'s hex is the same question as whether `b` may join `a`'s. A
    /// one-directional bug (reject the Hadendowa joining the gun, but accept
    /// the gun joining the Hadendowa) cannot survive this.
    // §5.51
    #[kani::proof]
    fn stacking_rule_is_symmetric() {
        let a = stack_unit(any_stack_identity(), UnitId::ALL[0]);
        let b = stack_unit(any_stack_identity(), UnitId::ALL[1]);
        assert_eq!(stacking_rule(&[&a, &b]), stacking_rule(&[&b, &a]),);
    }

    /// §5.51: leaders are free stacking -- any five leaders, in any mix of
    /// Dervish and British, form a legal stack. (A British leader is exempt
    /// from the enemy-cohabitation check; a Dervish leader's §5.53 command
    /// check constrains only *tribal* units; neither is counted
    /// toward the four-unit limit.)
    // §5.51
    #[kani::proof]
    fn stacking_rule_leaders_are_free_stacking() {
        let leader = |dervish: bool, id: UnitId| {
            let identity = if dervish {
                UnitIdentity::DervishLeader(DervishLeader::SheikElDin)
            } else {
                UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener)
            };
            stack_unit(identity, id)
        };
        let l0 = stack_unit(
            UnitIdentity::DervishLeader(DervishLeader::OsmanDigna),
            UnitId::ALL[0],
        );
        let l1 = stack_unit(
            UnitIdentity::AngloEgyptianLeader(BritishLeader::Gordon),
            UnitId::ALL[1],
        );
        let l2 = leader(kani::any(), UnitId::ALL[2]);
        let l3 = leader(kani::any(), UnitId::ALL[3]);
        let l4 = leader(kani::any(), UnitId::ALL[4]);
        let occupants = [&l0, &l1, &l2, &l3, &l4];
        assert_eq!(stacking_rule(&occupants), Ok(()));
    }

    /// §5.51: the limit binds exactly at four *counted* units -- four same-
    /// tribe Dervish counters stack legally, a fifth is rejected (and, the
    /// limit firing before the group/leader checks, `OverLimit` is the exact
    /// error).
    // §5.51
    #[kani::proof]
    fn stacking_rule_limit_is_four_counted_units() {
        // One symbolic tribe shared by every counter in the stack.
        let t: usize = kani::any();
        kani::assume(t < TRIBES.len());
        let tribal = |id: UnitId| stack_unit(UnitIdentity::DervishTribal { tribe: TRIBES[t] }, id);
        let u0 = tribal(UnitId::ALL[0]);
        let u1 = tribal(UnitId::ALL[1]);
        let u2 = tribal(UnitId::ALL[2]);
        let u3 = tribal(UnitId::ALL[3]);
        let four = [&u0, &u1, &u2, &u3];
        assert_eq!(stacking_rule(&four), Ok(()));
        let u4 = tribal(UnitId::ALL[4]);
        let mut five = four.to_vec();
        five.push(&u4);
        assert_eq!(stacking_rule(&five), Err(StackingError::OverLimit));
    }

    /// §5.51: a gunboat shares a hex with nothing -- any same-owner
    /// non-gunboat counter beside it is rejected with `GunboatStack`
    /// (symbolic across both factions' boats and the counter's tribe), while
    /// a lone gunboat is a legal stack.
    // §5.51
    #[kani::proof]
    fn stacking_rule_gunboat_never_shares() {
        let dervish_boat = stack_unit(
            UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1)),
            UnitId::ALL[0],
        );
        let ae_boat = stack_unit(
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(OldGunboat::Tamai)),
            UnitId::ALL[0],
        );
        // Same-owner companion: a Dervish tribal (symbolic tribe) or the AE
        // infantry representative.
        let t: usize = kani::any();
        kani::assume(t < TRIBES.len());
        let dervish_foot = stack_unit(
            UnitIdentity::DervishTribal { tribe: TRIBES[t] },
            UnitId::ALL[1],
        );
        let ae_foot = stack_unit(
            UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::British,
                },
                battalion: BattalionOrdinal::First,
            },
            UnitId::ALL[1],
        );
        let dervish_pair = [&dervish_boat, &dervish_foot];
        let ae_pair = [&ae_boat, &ae_foot];
        assert_eq!(
            stacking_rule(&dervish_pair),
            Err(StackingError::GunboatStack)
        );
        assert_eq!(stacking_rule(&ae_pair), Err(StackingError::GunboatStack));
        // And a lone gunboat is a legal stack.
        assert_eq!(stacking_rule(&[&dervish_boat]), Ok(()));
    }

    // -- Desertion arithmetic (§8.2) ---------------------------------------

    /// §8.2: "the number of deserting Dervish units is equal to 1½ times the
    /// roll of one die" -- floored, hence `(3r)/2` for every roll. Also
    /// monotone (a higher roll never deserts fewer) and bounded 1..=15 (1½×1
    /// = 1.5 floors to 1; 1½×10 = 15).
    // §8.2
    #[kani::proof]
    fn desertion_count_is_one_and_a_half_times_the_roll() {
        let i: usize = kani::any();
        let j: usize = kani::any();
        kani::assume(i < DieRoll::ALL.len());
        kani::assume(j < DieRoll::ALL.len());
        let a = desertion_count(DieRoll::ALL[i]);
        let b = desertion_count(DieRoll::ALL[j]);
        // Exactly floored 1½ × roll.
        assert!(a == (3 * DieRoll::ALL[i].value() as usize) / 2);
        // Monotone in the roll.
        if i <= j {
            assert!(a <= b);
        }
        // Bounded: at least one deserter, at most 1½ × 10.
        assert!(a >= 1 && a <= 15);
    }

    // -- Zone of control predicates (§5.41) --------------------------------

    /// §5.41's four clauses, proven exact over a symbolic unit and mover:
    /// a disrupted unit projects no ZOC; friendly units never project on each
    /// other; Anglo-Egyptian leaders exert no ZOC; and a gunboat projects ZOC
    /// *only* against an enemy gunboat (every other unit projects normally,
    /// forts included -- §5.44).
    // §5.41
    #[kani::proof]
    fn unit_projects_zoc_matches_manual_clauses() {
        let mut unit = stack_unit(any_stack_identity(), UnitId::ALL[0]);
        unit.state.disrupted = kani::any();
        let mover_player = any_player();
        let mover_kind = if kani::any() {
            UnitKind::Gunboat {
                fire: 0,
                upstream: 10,
                downstream: 16,
            }
        } else {
            UnitKind::Infantry {
                fire: 4,
                melee: 5,
                movement: 8,
            }
        };
        let zoc = unit_projects_zoc_rule(&unit, mover_player, mover_kind);
        let friendly = unit.profile.identity.owner() == mover_player;
        if unit.state.disrupted || friendly {
            assert_eq!(zoc, None);
        } else {
            match unit.profile.kind {
                UnitKind::BritishLeader { .. } => assert_eq!(zoc, None),
                UnitKind::Gunboat { .. } => {
                    assert_eq!(
                        zoc == Some(ZocReason::GunboatVsGunboat),
                        matches!(mover_kind, UnitKind::Gunboat { .. })
                    );
                }
                UnitKind::Fort { .. } => assert_eq!(zoc, Some(ZocReason::Fort)),
                _ => assert_eq!(zoc, Some(ZocReason::Normal)),
            }
        }
    }

    // -- Hex-grid direction helpers (§6.64) --------------------------------
    //
    // `opposite`, `toward_index` and `step_toward` are pure geometry over
    // [`HexCoord`], so these proofs do not pull in any `std` RNG/IO path -- they
    // resolve to a true `SUCCESS` (unlike the `GameState` harnesses above).

    /// §6.64: the scattergram's ring is addressed by direction; `opposite` is
    /// the three-step reversal round the six-sided ring (`(i+3)%6`). It must be
    /// an involution -- flipping a direction twice returns you to the same edge.
    // §6.64
    #[kani::proof]
    fn opposite_is_an_involution() {
        let i: usize = kani::any();
        kani::assume(i < 6);
        let j = opposite(i);
        kani::assume(j < 6);
        // Flipping twice is the identity.
        assert!(opposite(opposite(i)) == i);
        // And it never maps a direction onto itself (a real diametric flip).
        assert!(opposite(i) != i);
    }

    /// §6.64: `step_toward` (used to orient howitzer scatter away from the
    /// firer) must land on a hex *adjacent* to the origin -- a single hex of
    /// progress, never a teleport. This is the load-bearing invariant of the
    /// scatter helpers: folding a multi-hex path through `step_toward` cannot
    /// glide across the board. (When the origin already *is* the target there
    /// is nothing to step toward -- every neighbour is farther -- so that
    /// degenerate case is excluded, as the scatter usage always guarantees.)
    // §6.64
    #[kani::proof]
    fn step_toward_lands_on_an_adjacent_hex() {
        let q0: i32 = kani::any();
        let r0: i32 = kani::any();
        let q1: i32 = kani::any();
        let r1: i32 = kani::any();
        kani::assume(q0 >= -2 && q0 <= 2);
        kani::assume(r0 >= -2 && r0 <= 2);
        kani::assume(q1 >= -2 && q1 <= 2);
        kani::assume(r1 >= -2 && r1 <= 2);
        let origin = HexCoord::new(q0, r0);
        let target = HexCoord::new(q1, r1);
        kani::assume(origin != target);
        let next = step_toward(origin, target);
        assert!(next.is_adjacent_to(origin));
        // One step never *away* from the target: it does not increase the
        // remaining distance.
        assert!(next.distance(target) <= origin.distance(target));
    }
}

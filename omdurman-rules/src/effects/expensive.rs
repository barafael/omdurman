//! The expensive proof tier: one harness per [`GameEffect`] kind, each over
//! an arbitrary effect of that kind on a rich symbolic state. For every
//! input the harness can build, [`apply_effect`]
//!
//! 1. never panics, overflows or indexes out of bounds (Kani checks these in
//!    every harness), so no effect a peer sends can crash the others;
//! 2. leaves the state untouched when it rejects the effect (a rejected
//!    event is never retried, so a partial mutation is a permanent desync);
//! 3. keeps the global invariants when it accepts it: every hex legally
//!    stacked (§5.51-5.53, the post-condition `apply_effect` otherwise only
//!    `debug_assert`s), unit ids unique, no eliminated unit back on the
//!    board, and no per-phase tracker naming a unit that has left it.
//!
//! Proofs only under `--features kani-expensive` (`KANI_EXPENSIVE=1
//! ./run-kani.sh`, or `./scripts/kani.sh -p omdurman-rules --features
//! kani-expensive --harness expensive::`): each harness is sized for a
//! machine with tens of GB per job and an hour or more, not for the
//! everyday suite. Under `cargo test` the same harnesses run on random
//! samples (see the end of this file).
//!
//! # The state
//!
//! Three distinct real counters from the [palette](crate::proof_palette),
//! each on any hex of a 5x5 window, disrupted or not, every scenario, phase,
//! active player, turn and time of day, the optional rules on or off, and
//! each unit independently in or out of the per-phase trackers -- assumed to
//! be legally stacked to start with. Per effect kind the state also carries
//! what that effect acts on (a declared melee, a vacated hex, a mine, the
//! chain, a loaded gunboat, a pending demolition), present or absent.
//!
//! The board is the engine's rule-neutral empty board (`BoardInfo` with no
//! terrain), on which the engine skips every terrain and hexside check: the
//! terrain branches are covered by the unit tests and the table proofs, and
//! a non-empty board would put a hash of a symbolic hex in every lookup.
//!
//! # Payloads
//!
//! Every id in a payload is a palette counter (on the board or not), every
//! hex is in the window or at the far end of `i32` (the coordinate guard),
//! vectors hold up to two or three entries, and modifiers, rolls and table
//! rows range over their whole types -- including a `Terrain` fire modifier
//! of any `i16`.
//!
//! # Stubs
//!
//! `profile_for_unit` is replaced by the palette's `match`
//! (`proof_palette::palette_profile`): every id a harness can produce is a
//! palette counter, and the palette is held to the roster by its unit test,
//! so the stub is exact on every reachable input. `BoardInfo::bank_of` is
//! replaced by `None`, which is what it returns everywhere on the empty
//! board.
//!
//! `kani::cover!` marks both outcomes of every effect; a harness whose
//! "accepted" cover is UNSATISFIABLE proves only its rejection half.

use super::*;
#[cfg(kani)]
use crate::board::{BoardInfo, NileBank};
use crate::combat_results_table::FireFactorRow;
use crate::proof_palette::{PALETTE, PALETTE_LEN};
use crate::transport::ChainPlacement;
use crate::{
    DemolitionTarget, DieRoll, FireAttack, FireKind, FireModifier, FireSubPhase, FriendliesAction,
    GameTurnIndex, MeleeAttack, MeleeModifier, MinePlacement, MovementPoints, OptionalRule, Phase,
    StruckMine, TransportState, UnitId, UnitPlacement, UnitState,
};
use omdurman_types::{DayNight, HexCoord, HexsideRef, Player, Scenario, UnitKind};

#[cfg(kani)]
use kani::{any, assume};
#[cfg(not(kani))]
use sample::{any, assume};

/// `kani::cover!` under Kani; counted when sampled.
macro_rules! cover {
    ($cond:expr, $msg:literal) => {
        #[cfg(kani)]
        kani::cover!($cond, $msg);
        #[cfg(not(kani))]
        sample::cover($cond, $msg);
    };
}

/// Most units a harness state can hold: three on the board, plus two
/// placed by the effect.
const MAX_UNITS: usize = 6;

/// `BoardInfo::bank_of` on the empty board.
#[cfg(kani)]
fn stub_bank_of_none(_board: &BoardInfo, _hex: HexCoord) -> Option<NileBank> {
    None
}

/// A `usize` below `n` (a bounded choice: symbolic under Kani, drawn
/// directly when sampled).
fn any_below(n: usize) -> usize {
    #[cfg(kani)]
    {
        let i: usize = kani::any();
        kani::assume(i < n);
        i
    }
    #[cfg(not(kani))]
    {
        sample::below(n)
    }
}

/// An `i32` in `lo..=hi`.
fn any_in(lo: i32, hi: i32) -> i32 {
    lo + any_below((hi - lo + 1) as usize) as i32
}

fn any_player() -> Player {
    if any() {
        Player::AngloEgyptian
    } else {
        Player::Dervish
    }
}

fn any_scenario() -> Scenario {
    match any_below(3) {
        0 => Scenario::Campaign,
        1 => Scenario::Historical,
        _ => Scenario::FallOfKhartoum,
    }
}

fn any_phase() -> Phase {
    match any_below(7) {
        0 => Phase::Setup,
        1 => Phase::Movement,
        2 => Phase::DefensiveFire(FireSubPhase::DirectFire),
        3 => Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer),
        4 => Phase::OffensiveFire(FireSubPhase::DirectFire),
        5 => Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
        _ => Phase::Melee,
    }
}

fn any_roll() -> DieRoll {
    DieRoll::ALL[any_below(DieRoll::ALL.len())]
}

/// A disruption draw: 24 = 4! values reach every ordered pick of up to four
/// candidates (`DisruptionDraw::pick` reads the draw as mixed radix).
fn any_disruption() -> DisruptionDraw {
    DisruptionDraw(any_below(24) as u32)
}

/// A hex of the 5x5 window around the origin.
fn any_hex() -> HexCoord {
    HexCoord::new(any_in(-2, 2), any_in(-2, 2))
}

/// A payload hex: in the window, or at the far end of `i32`.
fn any_payload_hex() -> HexCoord {
    if any() {
        any_hex()
    } else {
        let q: i32 = if any() { i32::MIN } else { i32::MAX };
        HexCoord::new(q, 0)
    }
}

/// A path of up to three hexes from `from`: either a walk (each hex next to
/// the one before, the shape a legal move has) or arbitrary payload hexes.
fn any_path(from: HexCoord) -> Vec<HexCoord> {
    let len = any_below(4);
    let walk: bool = any();
    let mut path = Vec::new();
    let mut at = from;
    for _ in 0..len {
        let next = if walk {
            at.neighbors()[any_below(6)]
        } else {
            any_payload_hex()
        };
        path.push(next);
        at = next;
    }
    path
}

fn any_hexside() -> HexsideRef {
    HexsideRef {
        a: any_payload_hex(),
        b: any_payload_hex(),
    }
}

fn any_palette_id() -> UnitId {
    PALETTE[any_below(PALETTE_LEN)].0
}

/// Up to `max` (at most 3) palette ids, repeats allowed.
fn any_ids(max: usize) -> Vec<UnitId> {
    let len = any_below(max + 1);
    let mut ids = Vec::new();
    if len > 0 {
        ids.push(any_palette_id());
    }
    if len > 1 {
        ids.push(any_palette_id());
    }
    if len > 2 {
        ids.push(any_palette_id());
    }
    ids
}

fn any_fire_modifiers() -> Vec<FireModifier> {
    let mut mods = Vec::new();
    if any() {
        mods.push(match any_below(5) {
            0 => FireModifier::AngloEgyptianDirectFire,
            1 => FireModifier::BrigadeIntegrity,
            2 => FireModifier::Terrain(any()),
            3 => FireModifier::ZaribaThornHedge,
            _ => FireModifier::ZaribaTrenchEntrenched,
        });
    }
    mods
}

fn any_melee_modifiers() -> Vec<MeleeModifier> {
    let mut mods = Vec::new();
    if any() {
        mods.push(match any_below(4) {
            0 => MeleeModifier::DervishStandard,
            1 => MeleeModifier::AngloEgyptianStandard,
            2 => MeleeModifier::DervishVsTrenchedDefender,
            _ => MeleeModifier::FriendliesStandard,
        });
    }
    mods
}

fn any_fire_attack() -> FireAttack {
    FireAttack {
        firing_player: any_player(),
        phase: any_phase(),
        kind: match any_below(3) {
            0 => FireKind::Direct,
            1 => FireKind::Howitzer,
            _ => FireKind::MaximSecondFire,
        },
        firers: any_ids(2),
        target_hex: any_payload_hex(),
        at_fort: any(),
        factor_row: FireFactorRow::ALL[any_below(FireFactorRow::ALL.len())],
        modifiers: any_fire_modifiers(),
        // The palette's named gunboat may join with its Maxims (§2.32).
        gunboat_maxims: any_ids(1),
    }
}

fn any_melee_attack() -> MeleeAttack {
    MeleeAttack {
        attacker_player: any_player(),
        attacker_hex: any_payload_hex(),
        defender_hex: any_payload_hex(),
        attackers: any_ids(2),
        defenders: any_ids(2),
        attacker_modifiers: any_melee_modifiers(),
        defender_modifiers: any_melee_modifiers(),
    }
}

/// A placement as a peer might send it: a palette counter anywhere, with its
/// own profile or another counter's, fresh or disrupted.
fn any_placement() -> UnitPlacement {
    let (id, own) = PALETTE[any_below(PALETTE_LEN)];
    let profile = if any() {
        own
    } else {
        PALETTE[any_below(PALETTE_LEN)].1
    };
    UnitPlacement {
        id,
        position: any_payload_hex(),
        profile,
        state: UnitState {
            disrupted: any(),
            ..UnitState::default()
        },
    }
}

/// The palette counter at `index`, on any hex of the window.
fn palette_unit(index: usize) -> UnitPlacement {
    let (id, profile) = PALETTE[index];
    let gunboat = matches!(profile.kind, UnitKind::Gunboat { .. });
    UnitPlacement {
        id,
        position: any_hex(),
        profile,
        state: UnitState {
            disrupted: any(),
            engines_lost: gunboat && any(),
            ..UnitState::default()
        },
    }
}

/// Whether every occupied hex holds a legal stack (the engine's
/// `validate_stacking_invariants`, without its grouping map). Each hex is
/// checked once, at its first occupant, and `stacking_rule` always sees a
/// slice of concrete length: with a symbolic length every loop inside it
/// unrolls to the bound.
fn stacking_ok(state: &GameState) -> bool {
    let units = &state.units;
    let mut ok = true;
    for (i, u) in units.iter().enumerate() {
        if units[..i].iter().any(|v| v.position == u.position) {
            continue;
        }
        let mut here: [&UnitPlacement; MAX_UNITS] = [u; MAX_UNITS];
        let mut n = 0;
        for v in units {
            if v.position == u.position {
                here[n] = v;
                n += 1;
            }
        }
        ok &= match n {
            1 => stacking_rule(&here[..1]),
            2 => stacking_rule(&here[..2]),
            3 => stacking_rule(&here[..3]),
            4 => stacking_rule(&here[..4]),
            5 => stacking_rule(&here[..5]),
            _ => stacking_rule(&here[..6]),
        }
        .is_ok();
    }
    ok
}

/// The rich symbolic state (see the module doc), legally stacked.
fn any_state() -> GameState {
    let mut state = GameState::kani_minimal();
    state.scenario = any_scenario();
    state.phase = any_phase();
    state.active_player = any_player();
    state.current_turn = GameTurnIndex::new(any_below(22) as u8 + 1);
    state.day_night = if any() {
        DayNight::Day
    } else {
        DayNight::Night
    };
    state.dervish_deserted = any();
    state.setup_ready_ae = any();
    state.setup_ready_dervish = any();
    if any() {
        state.optional_rules.push(OptionalRule::RiverMines);
    }
    if any() {
        state.optional_rules.push(OptionalRule::RiverChain);
    }
    // Three distinct counters, in palette order.
    let a = any_below(PALETTE_LEN);
    let b = any_below(PALETTE_LEN);
    let c = any_below(PALETTE_LEN);
    assume(a < b && b < c);
    for index in [a, b, c] {
        state.units.push(palette_unit(index));
    }
    for k in 0..3 {
        let id = state.units[k].id;
        if any() {
            state.units_fired_this_phase.push(id);
        }
        if any() {
            state.units_fired_at_this_phase.push(id);
        }
        if any() {
            state.units_meleed_this_turn.push(id);
        }
        if any() {
            state.zoc_stopped_this_turn.push(id);
        }
    }
    if any() {
        let spent = any_in(0, 20) as i16;
        state.mp_spent_this_turn.insert(state.units[0].id, spent);
    }
    assume(stacking_ok(&state));
    state
}

/// One of the state's three units.
fn any_slot() -> usize {
    any_below(3)
}

/// A declared melee, present or not, as `DeclareMelee` leaves one: in the
/// Melee phase, one unit against an adjacent enemy.
fn with_pending_melee(state: &mut GameState) {
    if any() {
        let attacker = state.units[any_slot()];
        let defender = state.units[any_slot()];
        assume(
            attacker.profile.identity.owner() != defender.profile.identity.owner()
                && attacker.position.distance(defender.position) == 1,
        );
        state.phase = Phase::Melee;
        state.active_player = attacker.profile.identity.owner();
        state.pending_melee = Some(PendingMelee {
            attack: MeleeAttack {
                attacker_player: attacker.profile.identity.owner(),
                attacker_hex: attacker.position,
                defender_hex: defender.position,
                attackers: vec![attacker.id],
                defenders: vec![defender.id],
                attacker_modifiers: any_melee_modifiers(),
                defender_modifiers: any_melee_modifiers(),
            },
            attacker_roll: any_roll(),
            defender_roll: any_roll(),
            disruption: any_disruption(),
        });
    }
}

/// A hex vacated by combat, present or not: empty, with one unit eligible
/// to advance into it.
fn with_vacated_hex(state: &mut GameState) {
    if any() {
        let hex = any_hex();
        assume(state.units.iter().all(|u| u.position != hex));
        let id = state.units[any_slot()].id;
        state.vacated_by_combat.insert(hex, vec![id]);
    }
}

/// A mine, present or not, possibly triggered -- or struck: a British
/// gunboat has just entered it (§10.12).
fn with_mine(state: &mut GameState) {
    if any() {
        let struck = if any() {
            let boat = state.units[any_slot()];
            assume(
                matches!(boat.profile.kind, UnitKind::Gunboat { .. })
                    && boat.profile.identity.owner() == Player::AngloEgyptian,
            );
            Some(boat)
        } else {
            None
        };
        let hex = struck.map_or_else(any_hex, |boat| boat.position);
        state.mines.push(MinePlacement {
            hex,
            triggered: struck.is_none() && any(),
        });
        if let Some(boat) = struck {
            state.pending_mine = Some(StruckMine {
                gunboat: boat.id,
                hex,
            });
        }
    }
}

/// The chain, present or not, on one or two hexes, sunk or not.
fn with_chain(state: &mut GameState) {
    if any() {
        let mut hexes = vec![any_hex()];
        if any() {
            hexes.push(any_hex());
        }
        state.chain = Some(ChainPlacement { hexes, sunk: any() });
    }
}

/// A "Friendlies" unit aboard a British gunboat, present or not: on its
/// carrier's hex, as a completed load leaves them (§5.21).
fn with_passenger(state: &mut GameState) {
    if any() {
        let j = any_slot();
        let k = any_slot();
        let carrier = state.units[k];
        assume(
            state.units[j].profile.identity.is_friendlies()
                && matches!(carrier.profile.kind, UnitKind::Gunboat { .. })
                && carrier.profile.identity.owner() == Player::AngloEgyptian,
        );
        state.units[j].position = carrier.position;
        state.units[j].state.loaded_on = Some(carrier.id);
        state.friendlies_transport = Some(TransportState::Loaded {
            unit: state.units[j].id,
            gunboat: carrier.id,
            since: state.current_turn,
        });
        assume(stacking_ok(state));
    }
}

/// A demolition under way, present or not, by the Royal Engineers.
fn with_demolition(state: &mut GameState) {
    if any() {
        let j = any_slot();
        assume(state.units[j].profile.identity == crate::UnitIdentity::RoyalEngineers);
        let target = if any() {
            DemolitionTarget::Fort(state.units[any_slot()].id)
        } else {
            DemolitionTarget::WallHexside(HexsideRef::new(any_hex(), any_hex()))
        };
        state.units[j].state.demolishing = true;
        state.pending_demolitions.push((state.units[j].id, target));
    }
}

/// Every rule-relevant field of `a` and `b` agrees; the append-only logs
/// agree in length (a rejected effect must not append to them either).
fn same_rule_state(a: &GameState, b: &GameState) -> bool {
    let chain = |s: &GameState| s.chain.as_ref().map(|c| (c.hexes.clone(), c.sunk));
    let melee = |s: &GameState| {
        s.pending_melee
            .as_ref()
            .map(|m| (m.attack.clone(), m.attacker_roll, m.defender_roll))
    };
    a.scenario == b.scenario
        && a.current_turn == b.current_turn
        && a.day_night == b.day_night
        && a.active_player == b.active_player
        && a.phase == b.phase
        && a.units == b.units
        && a.victory.events == b.victory.events
        && a.next_alloc_index == b.next_alloc_index
        && a.units_fired_this_phase == b.units_fired_this_phase
        && a.units_fired_at_this_phase == b.units_fired_at_this_phase
        && a.mp_spent_this_turn == b.mp_spent_this_turn
        && a.gunboats_upstream_this_turn == b.gunboats_upstream_this_turn
        && a.zoc_stopped_this_turn == b.zoc_stopped_this_turn
        && a.units_meleed_this_turn == b.units_meleed_this_turn
        && a.vacated_by_combat == b.vacated_by_combat
        && a.reinforcements_placed_this_turn == b.reinforcements_placed_this_turn
        && a.eliminated == b.eliminated
        && a.game_over == b.game_over
        && a.zariba_hexsides == b.zariba_hexsides
        && a.friendlies_transport == b.friendlies_transport
        && a.optional_rules == b.optional_rules
        && a.mines == b.mines
        && chain(a) == chain(b)
        && a.pending_mine == b.pending_mine
        && a.gunboats_stopped_this_turn == b.gunboats_stopped_this_turn
        && std::sync::Arc::ptr_eq(&a.board, &b.board)
        && a.breaches == b.breaches
        && a.dervish_deserted == b.dervish_deserted
        && melee(a) == melee(b)
        && a.gordon_eliminated_turn == b.gordon_eliminated_turn
        && a.setup_ready_ae == b.setup_ready_ae
        && a.setup_ready_dervish == b.setup_ready_dervish
        && a.isa_zachneih_eliminated == b.isa_zachneih_eliminated
        && a.pending_demolitions == b.pending_demolitions
        && a.observations.len() == b.observations.len()
        && a.turn_events.len() == b.turn_events.len()
        && a.turn_summaries.len() == b.turn_summaries.len()
        && a.game_result == b.game_result
}

/// The global invariants every accepted effect keeps: legal stacks, unique
/// unit ids, no eliminated unit back on the board, and no per-phase tracker
/// naming a unit that has left it (`prune_dead_trackers`).
fn invariants_hold(state: &GameState) -> bool {
    let on_board = |id: &UnitId| state.units.iter().any(|u| u.id == *id);
    let units = &state.units;
    let mut unique = true;
    for i in 0..units.len() {
        for j in (i + 1)..units.len() {
            unique &= units[i].id != units[j].id;
        }
    }
    stacking_ok(state)
        && unique
        && state.eliminated.iter().all(|id| !on_board(id))
        && state.units_fired_this_phase.iter().all(on_board)
        && state.units_fired_at_this_phase.iter().all(on_board)
        && state.zoc_stopped_this_turn.iter().all(on_board)
        && state.mp_spent_this_turn.keys().all(on_board)
        && state
            .vacated_by_combat
            .values()
            .all(|eligible| !eligible.is_empty() && eligible.iter().all(on_board))
}

/// Apply `effect` and hold the result to the three properties.
fn check(mut state: GameState, effect: GameEffect) {
    let before = state.clone();
    let result = apply_effect(&mut state, &effect);
    cover!(result.is_ok(), "the effect can be accepted");
    cover!(result.is_err(), "the effect can be rejected");
    if result.is_err() {
        assert!(
            same_rule_state(&before, &state),
            "a rejected effect changed the state"
        );
    } else {
        assert!(
            invariants_hold(&state),
            "an accepted effect broke a global invariant"
        );
    }
    state.kani_discard();
    before.kani_discard();
}

// -- Sampling bias -----------------------------------------------------------
//
// Random payloads almost never form a legal effect (a fire needs the right
// phase, side, firers and target), so a random run would only ever check the
// rejected half. When sampled, half the inputs are therefore shaped the way
// a client shapes them: the state put in the effect's phase with the right
// side to move, the payload built by the engine's own builders. Under Kani
// these are no-ops -- the arbitrary inputs already contain every legal
// effect, and the `cover!`s report whether the solver reached one.

/// When sampled, half the time: `plausible`'s value (if any) in place of
/// `arbitrary`.
fn biased<T>(arbitrary: T, plausible: impl FnOnce() -> Option<T>) -> T {
    #[cfg(not(kani))]
    if any::<bool>()
        && let Some(value) = plausible()
    {
        return value;
    }
    #[cfg(kani)]
    drop(plausible);
    arbitrary
}

/// When sampled, half the time: apply `shape` to the state (then re-assume
/// the starting invariant it may have broken).
fn nudge(state: &mut GameState, shape: impl FnOnce(&mut GameState)) {
    #[cfg(not(kani))]
    if any::<bool>() {
        shape(state);
        assume(stacking_ok(state));
    }
    #[cfg(kani)]
    drop((state, shape));
}

/// Put `state` in `phase` with the owner of one of its units to move.
fn to_move(state: &mut GameState, phase: Phase) {
    let mover = state.units[any_slot()];
    state.phase = phase;
    state.active_player = mover.profile.identity.owner();
}

/// One of the state's units owned by `player`.
fn a_unit_of(state: &GameState, player: Player) -> Option<UnitPlacement> {
    let theirs: Vec<UnitPlacement> = state
        .units
        .iter()
        .copied()
        .filter(|u| u.profile.identity.owner() == player)
        .collect();
    (!theirs.is_empty()).then(|| theirs[any_below(theirs.len())])
}

/// A fire attack as a client builds it: a unit of the side to move at an
/// enemy's hex.
fn plausible_fire(state: &GameState, kind: FireKind) -> Option<FireAttack> {
    let firer = a_unit_of(state, state.active_player)?;
    let target = a_unit_of(state, state.active_player.opponent())?;
    build_fire_attack_from(state, firer.position, &[firer.id], target.position, kind)
}

/// Move an enemy of the side to move next to one of its units, and build
/// the melee between them.
fn plausible_melee(state: &mut GameState) -> Option<MeleeAttack> {
    let attacker = a_unit_of(state, state.active_player)?;
    let defender = a_unit_of(state, state.active_player.opponent())?;
    let hex = attacker.position.neighbors()[any_below(6)];
    let d = state.units.iter().position(|u| u.id == defender.id)?;
    state.units[d].position = hex;
    assume(stacking_ok(state));
    build_melee_attack(state, attacker.position, hex)
}

// -- One harness per effect kind ---------------------------------------------

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn advance_phase_is_safe() {
    let mut state = any_state();
    with_pending_melee(&mut state);
    with_vacated_hex(&mut state);
    with_mine(&mut state);
    with_chain(&mut state);
    with_demolition(&mut state);
    check(state, GameEffect::AdvancePhase);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn move_unit_is_safe() {
    let mut state = any_state();
    with_mine(&mut state);
    with_passenger(&mut state);
    nudge(&mut state, |s| to_move(s, Phase::Movement));
    let unit_id = biased(any_palette_id(), || Some(state.units[any_slot()].id));
    let from = state
        .find_unit(unit_id)
        .map_or(HexCoord::new(0, 0), |u| u.position);
    let path = any_path(from);
    let to = match path.last() {
        Some(last) if any() => *last,
        _ => any_payload_hex(),
    };
    let effect = GameEffect::MoveUnit {
        unit_id,
        to,
        cost: MovementPoints(any()),
        path,
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn fire_combat_is_safe() {
    let mut state = any_state();
    nudge(&mut state, |s| {
        to_move(s, Phase::OffensiveFire(FireSubPhase::DirectFire))
    });
    let effect = GameEffect::FireCombat {
        attack: biased(any_fire_attack(), || {
            plausible_fire(&state, FireKind::Direct)
        }),
        roll: any_roll(),
        disruption: any_disruption(),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn howitzer_fire_is_safe() {
    let mut state = any_state();
    nudge(&mut state, |s| {
        to_move(
            s,
            Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
        )
    });
    let effect = GameEffect::HowitzerFire {
        attack: biased(any_fire_attack(), || {
            plausible_fire(&state, FireKind::Howitzer)
        }),
        combat_results_table_roll: any_roll(),
        impact_roll: any_roll(),
        disruption: any_disruption(),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn declare_melee_is_safe() {
    let mut state = any_state();
    with_pending_melee(&mut state);
    nudge(&mut state, |s| to_move(s, Phase::Melee));
    let arbitrary = any_melee_attack();
    let attack = biased(arbitrary, || plausible_melee(&mut state));
    let effect = GameEffect::DeclareMelee {
        attack,
        attacker_roll: any_roll(),
        defender_roll: any_roll(),
        disruption: any_disruption(),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn resolve_melee_is_safe() {
    let mut state = any_state();
    with_pending_melee(&mut state);
    check(state, GameEffect::ResolveMelee);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn retreat_before_melee_is_safe() {
    let mut state = any_state();
    with_pending_melee(&mut state);
    let defender = state
        .pending_melee
        .as_ref()
        .and_then(|m| state.find_unit(m.attack.defenders[0]).copied());
    let effect = GameEffect::RetreatBeforeMelee {
        unit_id: biased(any_palette_id(), || defender.map(|d| d.id)),
        to: biased(any_payload_hex(), || {
            defender.map(|d| d.position.neighbors()[any_below(6)])
        }),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn advance_after_combat_is_safe() {
    let mut state = any_state();
    with_vacated_hex(&mut state);
    let window = state
        .vacated_by_combat
        .iter()
        .next()
        .map(|(hex, eligible)| (*hex, eligible[0]));
    let effect = GameEffect::AdvanceAfterCombat {
        unit_id: biased(any_palette_id(), || window.map(|(_, id)| id)),
        to: biased(any_payload_hex(), || window.map(|(hex, _)| hex)),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn construct_zariba_is_safe() {
    let state = any_state();
    let effect = GameEffect::ConstructZariba {
        unit_ids: any_ids(2),
        hexside: any_hexside(),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn demolition_is_safe() {
    let mut state = any_state();
    with_demolition(&mut state);
    let target = if any() {
        DemolitionTarget::Fort(any_palette_id())
    } else {
        DemolitionTarget::WallHexside(any_hexside())
    };
    let effect = GameEffect::Demolition {
        unit_id: any_palette_id(),
        target,
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn place_reinforcements_is_safe() {
    let state = any_state();
    let mut placements = Vec::new();
    if any() {
        placements.push(any_placement());
        if any() {
            placements.push(any_placement());
        }
    }
    check(state, GameEffect::PlaceReinforcements(placements));
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn dervish_desertion_is_safe() {
    let mut state = any_state();
    // The first night's Dervish movement phase of the Campaign (§8.2).
    nudge(&mut state, |s| {
        s.scenario = Scenario::Campaign;
        s.phase = Phase::Movement;
        s.active_player = Player::Dervish;
        s.day_night = DayNight::Night;
        s.dervish_deserted = false;
        for turn in 1..=22 {
            s.current_turn = GameTurnIndex::new(turn);
            if s.desertion_due() {
                break;
            }
        }
    });
    let roll = any_roll();
    let dervish: Vec<UnitId> = state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == Player::Dervish)
        .map(|u| u.id)
        .collect();
    let demand = state.desertion_demand(roll).min(dervish.len());
    let effect = GameEffect::DervishDesertion {
        roll,
        deserters: biased(any_ids(3), || Some(dervish[..demand].to_vec())),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn friendlies_transport_is_safe() {
    let mut state = any_state();
    with_passenger(&mut state);
    nudge(&mut state, |s| {
        s.phase = Phase::Movement;
        s.active_player = Player::AngloEgyptian;
    });
    let offers = state.friendlies_transport_offers(Some(state.units[any_slot()].id));
    let arbitrary = if any() {
        FriendliesAction::Load {
            unit: any_palette_id(),
            gunboat: any_palette_id(),
        }
    } else {
        FriendliesAction::Disembark {
            unit: any_palette_id(),
            gunboat: any_palette_id(),
            to: any_payload_hex(),
        }
    };
    let action = biased(arbitrary, || {
        (!offers.is_empty()).then(|| offers[any_below(offers.len())])
    });
    check(state, GameEffect::FriendliesTransport(action));
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn river_mine_is_safe() {
    let mut state = any_state();
    with_mine(&mut state);
    let struck = state.pending_mine;
    let effect = GameEffect::RiverMine {
        gunboat_id: biased(any_palette_id(), || struck.map(|m| m.gunboat)),
        hex: biased(any_payload_hex(), || struck.map(|m| m.hex)),
        roll: any_roll(),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn sink_chain_is_safe() {
    let mut state = any_state();
    with_chain(&mut state);
    nudge(&mut state, |s| {
        s.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        s.active_player = Player::AngloEgyptian;
    });
    let guns: Vec<UnitId> = state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == Player::AngloEgyptian)
        .map(|u| u.id)
        .collect();
    let effect = GameEffect::SinkChain {
        firers: biased(any_ids(2), || (!guns.is_empty()).then(|| vec![guns[0]])),
        roll: any_roll(),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn deploy_unit_is_safe() {
    let mut state = any_state();
    nudge(&mut state, |s| to_move(s, Phase::Setup));
    check(state, GameEffect::DeployUnit(any_placement()));
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn remove_deployed_unit_is_safe() {
    let mut state = any_state();
    nudge(&mut state, |s| to_move(s, Phase::Setup));
    let mine = state.units[any_slot()];
    let effect = GameEffect::RemoveDeployedUnit {
        unit_id: biased(any_palette_id(), || Some(mine.id)),
        player: biased(any_player(), || Some(mine.profile.identity.owner())),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn place_mine_is_safe() {
    let mut state = any_state();
    with_mine(&mut state);
    nudge(&mut state, |s| {
        s.phase = Phase::Setup;
        s.active_player = Player::Dervish;
        s.optional_rules = vec![OptionalRule::RiverMines];
    });
    let effect = GameEffect::PlaceMine {
        hex: any_payload_hex(),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn place_chain_is_safe() {
    let mut state = any_state();
    with_chain(&mut state);
    nudge(&mut state, |s| {
        s.phase = Phase::Setup;
        s.active_player = Player::Dervish;
        s.optional_rules = vec![OptionalRule::RiverChain];
    });
    // Up to one hex past the four-hex limit (§10.21).
    let len = any_below(MAX_CHAIN_HEXES + 2);
    let mut hexes = Vec::new();
    for _ in 0..len {
        hexes.push(any_payload_hex());
    }
    let hexes = biased(hexes, || {
        let start = any_hex();
        let mut line = vec![start];
        line.extend(any_path(start));
        Some(line)
    });
    check(state, GameEffect::PlaceChain { hexes });
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn confirm_setup_ready_is_safe() {
    let mut state = any_state();
    nudge(&mut state, |s| to_move(s, Phase::Setup));
    let effect = GameEffect::ConfirmSetupReady {
        player: any_player(),
    };
    check(state, effect);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn artillery_breach_wall_is_safe() {
    let state = any_state();
    let effect = GameEffect::ArtilleryBreachWall {
        firers: any_ids(2),
        target: any_hexside(),
        roll: any_roll(),
    };
    check(state, effect);
}

// -- Movement legality (§5.1, §5.26, §5.43) --------------------------------

/// Whether `hex` is next to a unit that projects a zone of control over
/// `mover` (`unit_projects_zoc`, itself proven against §5.41/§5.44).
fn next_to_enemy_zoc(state: &GameState, mover: &UnitPlacement, hex: HexCoord) -> bool {
    let owner = mover.profile.identity.owner();
    state.units.iter().any(|u| {
        u.id != mover.id
            && u.position.is_adjacent_to(hex)
            && state
                .unit_projects_zoc(u, owner, mover.profile.kind)
                .is_some()
    })
}

/// An accepted move is a legal move, stated apart from the engine's
/// planner: the unit ends on `to`; every step is to an adjacent hex, from
/// where the unit stood; no hex before `to` lies next to an enemy that
/// projects a zone of control over it (it must stop on entering one); and
/// the points it has spent this turn stay within the allowance it now moves
/// under.
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn an_accepted_move_is_a_legal_move() {
    let mut state = any_state();
    let k = any_slot();
    let mover = state.units[k];
    let path = any_path(mover.position);
    let to = match path.last() {
        Some(last) => *last,
        None => mover.position.neighbors()[any_below(6)],
    };
    let effect = GameEffect::MoveUnit {
        unit_id: mover.id,
        to,
        cost: MovementPoints(0),
        path: path.clone(),
    };
    let before = state.clone();
    let result = apply_effect(&mut state, &effect);
    cover!(result.is_ok(), "a move can be accepted");
    if result.is_ok() {
        let moved = state
            .find_unit(mover.id)
            .expect("the mover stays on the board");
        assert!(moved.position == to, "the unit ends on `to`");
        let steps: &[HexCoord] = if path.is_empty() {
            core::slice::from_ref(&to)
        } else {
            &path
        };
        let mut prev = mover.position;
        for (i, next) in steps.iter().enumerate() {
            assert!(
                prev.is_adjacent_to(*next),
                "every step is to an adjacent hex"
            );
            if i + 1 < steps.len() {
                assert!(
                    !next_to_enemy_zoc(&before, &mover, *next),
                    "the unit passed through an enemy zone of control"
                );
            }
            prev = *next;
        }
        // The allowance it now moves under (a gunboat's upstream one once it
        // has gone upstream, §5.24; halved at night, §8.1).
        if let Some(allowance) = state.current_allowance(mover.id) {
            assert!(
                i32::from(state.mp_spent(mover.id)) <= i32::from(allowance.value()),
                "the unit spent more than its allowance"
            );
        }
    }
    state.kani_discard();
    before.kani_discard();
}

// -- Effect pairs ------------------------------------------------------------
//
// Bugs between effects -- a declared melee lost across a phase change, a
// fire tracker surviving its phase -- are invisible to one-effect proofs.
// These apply two arbitrary effects of the given kinds in sequence and hold
// the global invariants after every accepted step. (A symbolic effect *kind*
// would pull every dispatcher arm into one instance; pairs keep it to two.)

/// Apply `first` then `second`, holding the invariants after each accepted
/// one and atomicity after each rejected one.
fn check_pair(mut state: GameState, first: GameEffect, second: GameEffect) {
    for effect in [first, second] {
        let before = state.clone();
        let result = apply_effect(&mut state, &effect);
        cover!(result.is_ok(), "a step can be accepted");
        if result.is_ok() {
            assert!(
                invariants_hold(&state),
                "an accepted effect broke a global invariant"
            );
        } else {
            assert!(
                same_rule_state(&before, &state),
                "a rejected effect changed the state"
            );
        }
        before.kani_discard();
    }
    state.kani_discard();
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn declare_melee_then_advance_phase_keeps_the_invariants() {
    let mut state = any_state();
    nudge(&mut state, |s| to_move(s, Phase::Melee));
    let arbitrary = any_melee_attack();
    let attack = biased(arbitrary, || plausible_melee(&mut state));
    let declare = GameEffect::DeclareMelee {
        attack,
        attacker_roll: any_roll(),
        defender_roll: any_roll(),
        disruption: any_disruption(),
    };
    check_pair(state, declare, GameEffect::AdvancePhase);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn fire_then_advance_phase_keeps_the_invariants() {
    let mut state = any_state();
    nudge(&mut state, |s| {
        to_move(s, Phase::OffensiveFire(FireSubPhase::DirectFire))
    });
    let fire = GameEffect::FireCombat {
        attack: biased(any_fire_attack(), || {
            plausible_fire(&state, FireKind::Direct)
        }),
        roll: any_roll(),
        disruption: any_disruption(),
    };
    check_pair(state, fire, GameEffect::AdvancePhase);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn declare_melee_then_resolve_melee_keeps_the_invariants() {
    let mut state = any_state();
    nudge(&mut state, |s| to_move(s, Phase::Melee));
    let arbitrary = any_melee_attack();
    let attack = biased(arbitrary, || plausible_melee(&mut state));
    let declare = GameEffect::DeclareMelee {
        attack,
        attacker_roll: any_roll(),
        defender_roll: any_roll(),
        disruption: any_disruption(),
    };
    check_pair(state, declare, GameEffect::ResolveMelee);
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
#[cfg_attr(
    kani,
    kani::stub(
        crate::unit_profiles::profile_for_unit,
        crate::proof_palette::palette_profile
    )
)]
#[cfg_attr(kani, kani::stub(BoardInfo::bank_of, stub_bank_of_none))]
fn two_moves_keep_the_invariants() {
    let mut state = any_state();
    nudge(&mut state, |s| to_move(s, Phase::Movement));
    let ids: Vec<UnitId> = state.units.iter().map(|u| u.id).collect();
    let a_move = || GameEffect::MoveUnit {
        unit_id: biased(any_palette_id(), || Some(ids[any_below(ids.len())])),
        to: any_hex(),
        cost: MovementPoints(0),
        path: Vec::new(),
    };
    check_pair(state, a_move(), a_move());
}

// -- Wire format -------------------------------------------------------------
//
// Peers exchange effects as postcard (the variant index, then the fields);
// a round trip that loses or changes anything is a silent desync, and a
// decoder that panics on a malformed packet is a remote crash.

/// Number of effect kinds (`GameEffect` variants).
const EFFECT_KINDS: usize = 21;

/// An arbitrary effect of kind `kind` (variant order), built from the same
/// payload generators as the safety harnesses.
fn any_effect(kind: usize) -> GameEffect {
    match kind {
        0 => GameEffect::AdvancePhase,
        1 => GameEffect::MoveUnit {
            unit_id: any_palette_id(),
            to: any_payload_hex(),
            cost: MovementPoints(any()),
            path: {
                let mut path = Vec::new();
                if any() {
                    path.push(any_payload_hex());
                }
                path
            },
        },
        2 => GameEffect::FireCombat {
            attack: any_fire_attack(),
            roll: any_roll(),
            disruption: any_disruption(),
        },
        3 => GameEffect::HowitzerFire {
            attack: any_fire_attack(),
            combat_results_table_roll: any_roll(),
            impact_roll: any_roll(),
            disruption: any_disruption(),
        },
        4 => GameEffect::DeclareMelee {
            attack: any_melee_attack(),
            attacker_roll: any_roll(),
            defender_roll: any_roll(),
            disruption: any_disruption(),
        },
        5 => GameEffect::ResolveMelee,
        6 => GameEffect::RetreatBeforeMelee {
            unit_id: any_palette_id(),
            to: any_payload_hex(),
        },
        7 => GameEffect::AdvanceAfterCombat {
            unit_id: any_palette_id(),
            to: any_payload_hex(),
        },
        8 => GameEffect::ConstructZariba {
            unit_ids: any_ids(2),
            hexside: any_hexside(),
        },
        9 => GameEffect::Demolition {
            unit_id: any_palette_id(),
            target: if any() {
                DemolitionTarget::Fort(any_palette_id())
            } else {
                DemolitionTarget::WallHexside(any_hexside())
            },
        },
        10 => GameEffect::PlaceReinforcements(if any() {
            vec![any_placement()]
        } else {
            Vec::new()
        }),
        11 => GameEffect::DervishDesertion {
            roll: any_roll(),
            deserters: any_ids(2),
        },
        12 => GameEffect::FriendliesTransport(if any() {
            FriendliesAction::Load {
                unit: any_palette_id(),
                gunboat: any_palette_id(),
            }
        } else {
            FriendliesAction::Disembark {
                unit: any_palette_id(),
                gunboat: any_palette_id(),
                to: any_payload_hex(),
            }
        }),
        13 => GameEffect::RiverMine {
            gunboat_id: any_palette_id(),
            hex: any_payload_hex(),
            roll: any_roll(),
        },
        14 => GameEffect::SinkChain {
            firers: any_ids(2),
            roll: any_roll(),
        },
        15 => GameEffect::DeployUnit(any_placement()),
        16 => GameEffect::RemoveDeployedUnit {
            unit_id: any_palette_id(),
            player: any_player(),
        },
        17 => GameEffect::PlaceMine {
            hex: any_payload_hex(),
        },
        18 => GameEffect::PlaceChain {
            hexes: {
                let mut hexes = Vec::new();
                if any() {
                    hexes.push(any_payload_hex());
                }
                hexes
            },
        },
        19 => GameEffect::ConfirmSetupReady {
            player: any_player(),
        },
        _ => GameEffect::ArtilleryBreachWall {
            firers: any_ids(2),
            target: any_hexside(),
            roll: any_roll(),
        },
    }
}

/// Encoding `effect` and decoding it again gives back `effect`.
fn round_trips(effect: GameEffect) {
    let mut buf = [0u8; 256];
    let bytes = postcard::to_slice(&effect, &mut buf).expect("an effect fits 256 bytes");
    let back: GameEffect = postcard::from_bytes(bytes).expect("an encoded effect decodes");
    assert!(back == effect, "the wire round trip changed the effect");
}

macro_rules! wire_round_trip {
    ($($name:ident = $kind:expr;)*) => {$(
        #[cfg(kani)]
        #[kani::proof]
        #[cfg_attr(kani, kani::unwind(8))]
        fn $name() {
            round_trips(any_effect($kind));
        }
    )*};
}

wire_round_trip! {
    wire_round_trip_advance_phase = 0;
    wire_round_trip_move_unit = 1;
    wire_round_trip_fire_combat = 2;
    wire_round_trip_howitzer_fire = 3;
    wire_round_trip_declare_melee = 4;
    wire_round_trip_resolve_melee = 5;
    wire_round_trip_retreat_before_melee = 6;
    wire_round_trip_advance_after_combat = 7;
    wire_round_trip_construct_zariba = 8;
    wire_round_trip_demolition = 9;
    wire_round_trip_place_reinforcements = 10;
    wire_round_trip_dervish_desertion = 11;
    wire_round_trip_friendlies_transport = 12;
    wire_round_trip_river_mine = 13;
    wire_round_trip_sink_chain = 14;
    wire_round_trip_deploy_unit = 15;
    wire_round_trip_remove_deployed_unit = 16;
    wire_round_trip_place_mine = 17;
    wire_round_trip_place_chain = 18;
    wire_round_trip_confirm_setup_ready = 19;
    wire_round_trip_artillery_breach_wall = 20;
}

/// Decoding any 16 bytes as an effect returns an effect or an error, never
/// a panic. (Each decoded element consumes at least one byte, so no decode
/// loop runs past 16 trips.)
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(18))]
fn decoding_any_bytes_never_panics() {
    let bytes: [u8; 16] = core::array::from_fn(|_| any());
    let len = any_below(17);
    let _ = postcard::from_bytes::<GameEffect>(&bytes[..len]);
}

/// The kind index of `effect`, in variant order. Exhaustive on purpose: a
/// new `GameEffect` variant fails to compile here until `any_effect` and the
/// wire proofs cover it.
fn kind_of(effect: &GameEffect) -> usize {
    match effect {
        GameEffect::AdvancePhase => 0,
        GameEffect::MoveUnit { .. } => 1,
        GameEffect::FireCombat { .. } => 2,
        GameEffect::HowitzerFire { .. } => 3,
        GameEffect::DeclareMelee { .. } => 4,
        GameEffect::ResolveMelee => 5,
        GameEffect::RetreatBeforeMelee { .. } => 6,
        GameEffect::AdvanceAfterCombat { .. } => 7,
        GameEffect::ConstructZariba { .. } => 8,
        GameEffect::Demolition { .. } => 9,
        GameEffect::PlaceReinforcements(_) => 10,
        GameEffect::DervishDesertion { .. } => 11,
        GameEffect::FriendliesTransport(_) => 12,
        GameEffect::RiverMine { .. } => 13,
        GameEffect::SinkChain { .. } => 14,
        GameEffect::DeployUnit(_) => 15,
        GameEffect::RemoveDeployedUnit { .. } => 16,
        GameEffect::PlaceMine { .. } => 17,
        GameEffect::PlaceChain { .. } => 18,
        GameEffect::ConfirmSetupReady { .. } => 19,
        GameEffect::ArtilleryBreachWall { .. } => 20,
    }
}

/// `any_effect(k)` builds an effect of kind `k` for every kind, so the
/// round-trip harnesses above cover every variant.
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(8))]
fn every_effect_kind_has_a_wire_proof() {
    let kind = any_below(EFFECT_KINDS);
    assert!(kind_of(&any_effect(kind)) == kind);
}

// -- The same harnesses as a randomized test --------------------------------
//
// Under `cargo test` the harnesses above run on random concrete samples: the
// Kani API is replaced by a seeded `GameRng`, an `assume` that fails rejects
// the sample, and any other panic -- a failed property or an engine panic --
// fails the test with the harness name and seed. A proof run takes hours on
// a big machine; this takes seconds and catches most false alarms and real
// bugs first.

#[cfg(not(kani))]
mod sample {
    use crate::rng::GameRng;
    use std::cell::RefCell;

    thread_local! {
        static RNG: RefCell<GameRng> = RefCell::new(GameRng::from_seed(0));
        static COVERED: RefCell<u64> = const { RefCell::new(0) };
    }

    /// Count an "accepted" `cover!` that held: how many samples got an
    /// effect through.
    pub(super) fn cover(cond: bool, msg: &str) {
        if cond && msg.contains("accepted") {
            COVERED.with(|c| *c.borrow_mut() += 1);
        }
    }

    /// Accepted covers counted since the last call.
    pub(super) fn take_covered() -> u64 {
        COVERED.with(|c| std::mem::take(&mut *c.borrow_mut()))
    }

    /// The panic payload of a rejected sample.
    pub(super) struct Rejected;

    pub(super) trait Sample {
        fn sample(rng: &mut GameRng) -> Self;
    }
    impl Sample for bool {
        fn sample(rng: &mut GameRng) -> Self {
            rng.random_u32() & 1 == 1
        }
    }
    impl Sample for u8 {
        fn sample(rng: &mut GameRng) -> Self {
            rng.random_u32() as u8
        }
    }
    impl Sample for i16 {
        fn sample(rng: &mut GameRng) -> Self {
            rng.random_u32() as i16
        }
    }

    pub(super) fn any<T: Sample>() -> T {
        RNG.with(|rng| T::sample(&mut rng.borrow_mut()))
    }

    pub(super) fn below(n: usize) -> usize {
        RNG.with(|rng| rng.borrow_mut().random_u32() as usize % n)
    }

    pub(super) fn assume(cond: bool) {
        if !cond {
            std::panic::panic_any(Rejected);
        }
    }

    /// Run `harness` on samples seeded `0..samples`; the number accepted, or
    /// the first failure.
    pub(super) fn run(
        harness: impl Fn() + std::panic::RefUnwindSafe,
        samples: u64,
    ) -> Result<u64, String> {
        let mut accepted = 0;
        for seed in 0..samples {
            RNG.with(|rng| *rng.borrow_mut() = GameRng::from_seed(seed));
            match std::panic::catch_unwind(&harness) {
                Ok(()) => accepted += 1,
                Err(payload) if payload.is::<Rejected>() => {}
                Err(payload) => {
                    let message = payload
                        .downcast_ref::<&str>()
                        .map(|s| s.to_string())
                        .or_else(|| payload.downcast_ref::<String>().cloned())
                        .unwrap_or_default();
                    return Err(format!("seed {seed}: {message}"));
                }
            }
        }
        Ok(accepted)
    }
}

#[cfg(test)]
mod sampled {
    use super::*;

    const SAFETY: &[(&str, fn())] = &[
        ("advance_phase_is_safe", advance_phase_is_safe),
        ("move_unit_is_safe", move_unit_is_safe),
        ("fire_combat_is_safe", fire_combat_is_safe),
        ("howitzer_fire_is_safe", howitzer_fire_is_safe),
        ("declare_melee_is_safe", declare_melee_is_safe),
        ("resolve_melee_is_safe", resolve_melee_is_safe),
        ("retreat_before_melee_is_safe", retreat_before_melee_is_safe),
        ("advance_after_combat_is_safe", advance_after_combat_is_safe),
        ("construct_zariba_is_safe", construct_zariba_is_safe),
        ("demolition_is_safe", demolition_is_safe),
        ("place_reinforcements_is_safe", place_reinforcements_is_safe),
        ("dervish_desertion_is_safe", dervish_desertion_is_safe),
        ("friendlies_transport_is_safe", friendlies_transport_is_safe),
        ("river_mine_is_safe", river_mine_is_safe),
        ("sink_chain_is_safe", sink_chain_is_safe),
        ("deploy_unit_is_safe", deploy_unit_is_safe),
        ("remove_deployed_unit_is_safe", remove_deployed_unit_is_safe),
        ("place_mine_is_safe", place_mine_is_safe),
        ("place_chain_is_safe", place_chain_is_safe),
        ("confirm_setup_ready_is_safe", confirm_setup_ready_is_safe),
        (
            "artillery_breach_wall_is_safe",
            artillery_breach_wall_is_safe,
        ),
        (
            "an_accepted_move_is_a_legal_move",
            an_accepted_move_is_a_legal_move,
        ),
        (
            "declare_melee_then_advance_phase_keeps_the_invariants",
            declare_melee_then_advance_phase_keeps_the_invariants,
        ),
        (
            "fire_then_advance_phase_keeps_the_invariants",
            fire_then_advance_phase_keeps_the_invariants,
        ),
        (
            "declare_melee_then_resolve_melee_keeps_the_invariants",
            declare_melee_then_resolve_melee_keeps_the_invariants,
        ),
        (
            "two_moves_keep_the_invariants",
            two_moves_keep_the_invariants,
        ),
        (
            "decoding_any_bytes_never_panics",
            decoding_any_bytes_never_panics,
        ),
        (
            "every_effect_kind_has_a_wire_proof",
            every_effect_kind_has_a_wire_proof,
        ),
    ];

    /// Samples per harness: 20 000 by default, `EXPENSIVE_SAMPLES` for a
    /// deeper run.
    fn samples() -> u64 {
        std::env::var("EXPENSIVE_SAMPLES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20_000)
    }

    /// Every harness holds on random samples, and enough samples get past
    /// the harness's assumptions to mean something (a harness that rejects
    /// everything tests nothing). Prints, per harness, the samples that ran
    /// and how many had their effect accepted (`--nocapture` to see them).
    #[test]
    fn every_expensive_harness_holds_on_random_samples() {
        let quiet = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let samples = samples();
        let mut failures = Vec::new();
        for (name, harness) in SAFETY {
            sample::take_covered();
            match sample::run(*harness, samples) {
                Ok(ran) => {
                    let covered = sample::take_covered();
                    println!("{name:<55} ran {ran:>6}  accepted {covered:>6}");
                    if ran < samples / 100 {
                        failures.push(format!("{name}: only {ran} of {samples} samples ran"));
                    }
                }
                Err(e) => failures.push(format!("{name}: {e}")),
            }
        }
        for kind in 0..EFFECT_KINDS {
            let round_trip = move || round_trips(any_effect(kind));
            if let Err(e) = sample::run(round_trip, 500) {
                failures.push(format!("wire round trip of kind {kind}: {e}"));
            }
        }
        std::panic::set_hook(quiet);
        assert!(failures.is_empty(), "{failures:#?}");
    }
}

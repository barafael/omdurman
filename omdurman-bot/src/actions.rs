//! The legal-action enumerator — the core of the bot. For any [`GameState`],
//! [`legal_actions`] returns every `GameEffect` the active player could legally
//! submit right now (dice pre-rolled and embedded), plus `AdvancePhase` as the
//! universal escape hatch.
//!
//! It builds on the engine's existing `can_*` predicates. The 8 `GameEffect`
//! variants with no exposed predicate (`ResolveMelee`, `DervishDesertion`,
//! etc.) use clone-and-try: clone the state, call `apply_effect`, keep the
//! candidate if it succeeds.

use omdurman_rules::effects::{apply_effect, GameState, GameEffect};
use omdurman_rules::terrain_chart::movement_cost_with_road;
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_rules::{
    brigade_integrity, BrigadeIntegrity, DemolitionTarget, FireAttack, FireFactor,
    FireKind, FireModifier, MeleeAttack, MeleeModifier, MovementPoints, Phase,
    UnitId, UnitIdentity, UnitMovement, UnitState, WeaponClass,
};
use omdurman_types::{HexCoord, HexsideKind, HexsideRef, Player, Scenario, Terrain, UnitKind};

use crate::oob;
use crate::rng::BotRng;

/// Sort + dedup helper for `Vec<HexCoord>` (HexCoord doesn't impl Ord).
fn sort_dedup_hexes(hexes: &mut Vec<HexCoord>) {
    hexes.sort_by_key(|h| (h.q, h.r));
    hexes.dedup_by_key(|h| (h.q, h.r));
}

/// Soft cap on the candidate list to prevent the `Vec` from exploding when the
/// branching factor is high (e.g. many units × many target hexes). When the raw
/// enumeration exceeds this, the caller shuffles and truncates.
const MAX_CANDIDATES: usize = 80;

/// Max deploy candidates per player per `setup_actions` call. The driver picks
/// one action per iteration, so batching keeps the list bounded.
const MAX_SETUP_CANDIDATES: usize = 6;

/// All legal `GameEffect`s the active player could submit right now, dice
/// pre-rolled and embedded. Always includes `AdvancePhase`. The caller picks
/// one at random and applies it.
pub fn legal_actions(state: &GameState, rng: &mut BotRng) -> Vec<GameEffect> {
    let mut out = Vec::new();
    match state.phase {
        Phase::Setup => setup_actions(state, rng, &mut out),
        Phase::Movement => movement_actions(state, rng, &mut out),
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_) => fire_actions(state, rng, &mut out),
        Phase::Melee => melee_actions(state, rng, &mut out),
    }
    // Always allow the player to end the phase.
    out.push(GameEffect::AdvancePhase);
    // Trim if the list exploded.
    if out.len() > MAX_CANDIDATES {
        rng.shuffle(&mut out);
        out.truncate(MAX_CANDIDATES);
        // Re-add AdvancePhase (it may have been truncated).
        out.push(GameEffect::AdvancePhase);
    }
    out
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup_actions(state: &GameState, rng: &mut BotRng, out: &mut Vec<GameEffect>) {
    // 1. Fixed placements first (GORDON, North Fort).
    for placement in oob::fixed_placements(state) {
        let already = state
            .units
            .iter()
            .any(|u| u.position == placement.position && u.profile.identity == placement.profile.identity);
        if !already {
            out.push(GameEffect::DeployUnit(placement));
        }
    }

    // 2. Player-deployable OOB units not yet on the board. Stop at the
    //    scenario's setup target (`setup_target_met`): Fall of Khartoum's
    //    Dervish entry zone is only the south/east map edge (§9.322) and the
    //    full Dervish OOB does not fit there -- the target is the minimum the
    //    engine requires to leave Setup, not the whole counter mix.
    let scenario = state.scenario;
    for player in [Player::AngloEgyptian, Player::Dervish] {
        if state.setup_target_met(player) {
            continue;
        }
        let already_ids: Vec<UnitId> = state.units.iter().map(|u| u.id).collect();
        let mut to_deploy: Vec<UnitId> = oob::deployable_oob_for(scenario, player)
            .into_iter()
            .filter(|id| !already_ids.contains(id))
            .collect();
        // Dervish: deploy leaders before their tribes so each leader + its
        // command can cluster (§5.53 -- a leader may only stack with units of
        // its command). Anglo-Egyptian has no such constraint.
        if player == Player::Dervish {
            to_deploy.sort_by_key(|&id| {
                let leader = profile_for_unit(id)
                    .is_some_and(|p| matches!(p.identity, UnitIdentity::DervishLeader(_)));
                !leader
            });
        }
        let mut placed = 0usize;
        for id in to_deploy.iter().take(MAX_SETUP_CANDIDATES) {
            let Some(profile) = profile_for_unit(*id) else {
                continue;
            };
            if let Some(hex) = find_deploy_hex(state, *id, profile, rng) {
                out.push(GameEffect::DeployUnit(omdurman_rules::UnitPlacement {
                    id: *id,
                    position: hex,
                    profile,
                    state: UnitState::default(),
                }));
                placed += 1;
            }
        }
        // Un-deploy fallback: if nothing remaining can find a legal hex (FoK
        // Dervish leader-command deadlock, §5.53), offer to pull a friendly
        // unit back so a later pass can re-place the blocked unit first. This
        // mimics a human rearranging its force and guarantees progress.
        if placed == 0 && !to_deploy.is_empty() {
            let id = to_deploy[0];
            let Some(profile) = profile_for_unit(id) else {
                continue;
            };
            if let Some(victim) = victim_to_remove(state, id, profile, rng) {
                out.push(GameEffect::RemoveDeployedUnit {
                    unit_id: victim,
                    player,
                });
            }
        }
    }

    // 3. Confirm ready for each side whose target is met.
    for player in [Player::AngloEgyptian, Player::Dervish] {
        if state.setup_target_met(player) && !state.setup_ready(player) {
            out.push(GameEffect::ConfirmSetupReady { player });
        }
    }
}

/// Probe board hexes in the deployment zone for a legal placement of `id`.
/// Returns the first legal hex found. For Dervish units the probe order is
/// command-aware (§5.53): a tribe prefers a hex already holding its own tribe
/// or a commanding leader; a leader prefers a hex holding only tribes it
/// commands; empty hexes come before incompatible ones.
fn find_deploy_hex(
    state: &GameState,
    id: UnitId,
    profile: omdurman_rules::UnitProfile,
    rng: &mut BotRng,
) -> Option<HexCoord> {
    let mut hexes: Vec<HexCoord> = state.board.terrain.keys().copied().collect();
    rng.shuffle(&mut hexes);
    hexes.sort_by_key(|h| hex_deploy_preference(state, *h, profile));
    hexes.into_iter().find(|h| {
        let placement = omdurman_rules::UnitPlacement {
            id,
            position: *h,
            profile,
            state: UnitState::default(),
        };
        state.can_deploy_unit(&placement).is_ok()
    })
}

/// Stacking preference of `hex` for a Dervish unit (`profile`): -1 when the
/// hex holds a tribe/leader the mover cannot stack with (§5.52-5.53), 0 when
/// empty, 1 when it already holds a compatible occupant. Non-Dervish units are
/// indifferent (preference 0 unless occupied, then 1).
fn hex_deploy_preference(
    state: &GameState,
    hex: HexCoord,
    profile: omdurman_rules::UnitProfile,
) -> i8 {
    let occupants: Vec<_> = state.units.iter().filter(|u| u.position == hex).collect();
    if occupants.is_empty() {
        return 0;
    }
    let tribe = match profile.identity {
        UnitIdentity::DervishTribal { tribe } => Some(tribe),
        _ => None,
    };
    let leader = match profile.identity {
        UnitIdentity::DervishLeader(l) => Some(l),
        _ => None,
    };
    let compatible = match (tribe, leader) {
        (Some(t), _) => occupants.iter().all(|u| match u.profile.identity {
            UnitIdentity::DervishTribal { tribe: ot } => ot == t || leader.is_some_and(|l| l.commands(ot)),
            UnitIdentity::DervishLeader(l) => l.commands(t),
            _ => true,
        }),
        (None, Some(l)) => occupants.iter().all(|u| match u.profile.identity {
            UnitIdentity::DervishTribal { tribe: ot } => l.commands(ot),
            _ => true,
        }),
        _ => true,
    };
    if compatible {
        1
    } else {
        -1
    }
}

/// Find a friendly unit that, if pulled back off the board, would open a legal
/// deploy hex for `blocked_id`. Prefers non-leaders. Returns `None` when no
/// friendly unit can be spared (should not happen while setup is incomplete).
fn victim_to_remove(
    state: &GameState,
    blocked_id: UnitId,
    blocked_profile: omdurman_rules::UnitProfile,
    rng: &mut BotRng,
) -> Option<UnitId> {
    let owner = blocked_profile.identity.owner();
    let mut victims: Vec<UnitId> = state
        .units
        .iter()
        .filter(|u| {
            u.profile.identity.owner() == owner
                && !matches!(
                    u.profile.kind,
                    UnitKind::DervishLeader { .. } | UnitKind::BritishLeader { .. }
                )
        })
        .map(|u| u.id)
        .collect();
    rng.shuffle(&mut victims);
    for victim in victims {
        let mut test = state.clone();
        test.units.retain(|u| u.id != victim);
        let freed = test.board.terrain.keys().any(|h| {
            let placement = omdurman_rules::UnitPlacement {
                id: blocked_id,
                position: *h,
                profile: blocked_profile,
                state: UnitState::default(),
            };
            test.can_deploy_unit(&placement).is_ok()
        });
        if freed {
            return Some(victim);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

fn movement_actions(state: &GameState, _rng: &mut BotRng, out: &mut Vec<GameEffect>) {
    let mover = state.active_player;
    for unit in &state.units {
        if unit.profile.identity.owner() != mover {
            continue;
        }
        if unit.state.disrupted {
            continue;
        }
        // Land units only here; gunboats handled below.
        let is_boat = unit.profile.kind.is_boat();
        if is_boat {
            gunboat_moves(state, unit.id, out);
            continue;
        }
        if !matches!(unit.profile.movement, UnitMovement::Land(_)) {
            continue;
        }
        // Single-hex steps to each neighbour.
        for dest in unit.position.neighbors() {
            if let Some(cost) = step_cost(state, unit.position, dest) {
                if state
                    .can_move_unit_to(unit.id, Some(dest), cost)
                    .is_ok()
                {
                    out.push(GameEffect::MoveUnit {
                        unit_id: unit.id,
                        to: dest,
                        cost,
                        path: vec![dest],
                    });
                }
            }
        }
    }

    // Royal Engineers may commit to a demolition this turn (§6.53). Resolution
    // is end-of-turn, so the engine accepts the commit without target checks;
    // we gate on engineer identity + adjacency ourselves to avoid committing
    // to nothing.
    demolition_actions(state, out);
}

/// Compute the MP cost of stepping from `from` to `to` (terrain + road, §5.11).
fn step_cost(state: &GameState, _from: HexCoord, to: HexCoord) -> Option<MovementPoints> {
    let terrain = state.board.terrain_at(to).unwrap_or(Terrain::Clear {
        road: omdurman_types::Road::None,
    });
    let road = state.board.has_road(to);
    movement_cost_with_road(terrain, road)
        .map(|c| MovementPoints::new(c.value() as i16))
}

/// Gunboat single-step moves (§5.24): one hex up/down the Nile, respecting the
/// upstream/downstream allowance and the Nile-mouth crossing (§9.345).
fn gunboat_moves(state: &GameState, unit_id: UnitId, out: &mut Vec<GameEffect>) {
    let Some(unit) = state.find_unit(unit_id) else {
        return;
    };
    for dest in unit.position.neighbors() {
        // Cost 1 for a normal Nile step; the engine validates flow direction +
        // allowances inside `can_move_gunboat`.
        let cost = MovementPoints::new(1);
        let path = vec![dest];
        if state.can_move_gunboat(unit_id, dest, &path, cost).is_ok() {
            out.push(GameEffect::MoveUnit {
                unit_id,
                to: dest,
                cost,
                path: path.clone(),
            });
        }
        // §9.345 Nile-mouth crossing (6 flat MP) — try with the higher cost.
        if state.scenario == Scenario::FallOfKhartoum && state.is_nile_mouth_crossing(unit.position, dest) {
            let cross_cost = MovementPoints::new(6);
            if state.can_move_gunboat(unit_id, dest, &path, cross_cost).is_ok() {
                out.push(GameEffect::MoveUnit {
                    unit_id,
                    to: dest,
                    cost: cross_cost,
                    path,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fire combat (§6)
// ---------------------------------------------------------------------------

fn fire_actions(state: &GameState, rng: &mut BotRng, out: &mut Vec<GameEffect>) {
    let firer_player = state.active_player;
    let kind = match state.phase {
        Phase::OffensiveFire(sub) | Phase::DefensiveFire(sub) => {
            fire_kind_for_phase(state, firer_player, sub)
        }
        _ => return,
    };

    // Enumerate unique firer hexes (co-stacked firers fire together, §6.14).
    let mut firer_hexes: Vec<HexCoord> = state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == firer_player)
        .filter(|u| u.profile.fire.is_some())
        .filter(|u| !state.units_fired_this_phase.contains(&u.id))
        .map(|u| u.position)
        .collect();
    sort_dedup_hexes(&mut firer_hexes);

    // Enumerate enemy-occupied hexes as candidate targets.
    let enemy = firer_player.opponent();
    let mut target_hexes: Vec<HexCoord> = state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == enemy)
        .map(|u| u.position)
        .collect();
    sort_dedup_hexes(&mut target_hexes);

    for fhex in &firer_hexes {
        for &target in &target_hexes {
            // Find any firer in this hex that can fire at the target.
            let lead_firer = state.units.iter().find(|u| {
                u.position == *fhex
                    && u.profile.identity.owner() == firer_player
                    && u.profile.fire.is_some()
                    && !state.units_fired_this_phase.contains(&u.id)
                    && state.can_fire_at(u.id, target, kind).is_ok()
            });
            let Some(lead) = lead_firer else { continue };

            if let Some(attack) = build_fire_attack(state, lead.id, *fhex, target, kind) {
                match kind {
                    FireKind::Howitzer => {
                        // Howitzer needs two dice (§6.64).
                        out.push(GameEffect::HowitzerFire {
                            attack,
                            combat_results_table_roll: rng.roll_d10(),
                            impact_roll: rng.roll_d10(),
                        });
                    }
                    _ => {
                        out.push(GameEffect::FireCombat {
                            attack,
                            roll: rng.roll_d10(),
                        });
                    }
                }
            }
        }
    }

    // Artillery may instead target a Wall hexside for breaching (§6.63).
    artillery_breach_actions(state, rng, out);

    // Advance-after-combat (§6.82): if the active player just eliminated all
    // defenders in an adjacent hex via fire, the surviving firers may advance.
    advance_after_combat_actions(state, out);
}

/// Determine the [`FireKind`] for the current sub-phase and weapon (§6.41/§6.42).
fn fire_kind_for_phase(
    state: &GameState,
    firer_player: Player,
    sub: omdurman_rules::FireSubPhase,
) -> FireKind {
    match sub {
        omdurman_rules::FireSubPhase::DirectFire => FireKind::Direct,
        omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => {
            // Pick the kind based on what weapons the player has. For simplicity
            // the bot defaults to MaximSecond; individual firers will be filtered
            // by `can_fire_at` if they can't use it.
            let _ = (state, firer_player);
            FireKind::MaximSecondFire
        }
    }
}

/// Port of `omdurman-app/src/fire.rs::build_fire_attack` — constructs a
/// properly-modifiered [`FireAttack`] grouping co-stacked firers (§6.14).
fn build_fire_attack(
    gs: &GameState,
    firer: UnitId,
    firer_hex: HexCoord,
    target: HexCoord,
    kind: FireKind,
) -> Option<FireAttack> {
    let selected = gs.find_unit(firer)?;
    let owner = selected.profile.identity.owner();

    let firers: Vec<&omdurman_rules::UnitPlacement> = gs
        .units
        .iter()
        .filter(|u| u.position == firer_hex)
        .filter(|u| u.profile.identity.owner() == owner)
        .filter(|u| u.profile.fire.is_some())
        .filter(|u| gs.can_fire_at(u.id, target, kind).is_ok())
        .collect();
    if firers.is_empty() {
        return None;
    }

    let factor_row = FireFactor::sum_to_row(firers.iter().filter_map(|u| u.profile.fire.as_ref()));

    let mut modifiers = Vec::new();
    if kind == FireKind::Direct {
        if owner == Player::AngloEgyptian {
            modifiers.push(FireModifier::AngloEgyptianDirectFire);
        }
        let identities: Vec<_> = firers.iter().map(|u| u.profile.identity).collect();
        if let BrigadeIntegrity::Integrated(_) = brigade_integrity(&identities) {
            modifiers.push(FireModifier::BrigadeIntegrity);
        }
    }
    // Zariba modifiers (§9.231/§9.232) — engine-side board queries.
    if gs.board.has_zariba_thorn_hedge(target) {
        modifiers.push(FireModifier::ZaribaThornHedge);
    }
    if gs.board.is_zariba_entrenched(target) {
        modifiers.push(FireModifier::ZaribaTrenchEntrenched);
    }

    Some(FireAttack {
        firing_player: owner,
        phase: gs.phase,
        kind,
        firers: firers.iter().map(|u| u.id).collect(),
        target_hex: target,
        factor_row,
        modifiers,
    })
}

// ---------------------------------------------------------------------------
// Melee combat (§7)
// ---------------------------------------------------------------------------

fn melee_actions(state: &GameState, rng: &mut BotRng, out: &mut Vec<GameEffect>) {
    let attacker_player = state.active_player;
    let enemy = attacker_player.opponent();

    // If a melee is already declared but unresolved, the defender may retreat
    // before melee (§7.5); otherwise the only way forward is to resolve.
    if let Some(pending) = &state.pending_melee {
        let defender_hex = pending.attack.defender_hex;
        let defenders: Vec<UnitId> = state
            .units
            .iter()
            .filter(|u| u.position == defender_hex && u.profile.identity.owner() == enemy)
            .map(|u| u.id)
            .collect();
        for did in defenders {
            // Retreat is two full hexes (§7.5), not a single step.
            for dest in ring_at_distance(defender_hex, 2) {
                if state.can_retreat_before_melee(did, dest).is_ok() {
                    out.push(GameEffect::RetreatBeforeMelee { unit_id: did, to: dest });
                }
            }
        }
        let probe = GameEffect::ResolveMelee;
        if try_legal(state, &probe) {
            out.push(probe);
        }
        advance_after_combat_actions(state, out);
        return;
    }

    // No pending melee: declare new ones for each attacker hex adjacent to an
    // enemy hex (§7.2/§7.3). We always use DeclareMelee (the full-rules path)
    // rather than the MeleeCombat shortcut: it opens the §7.5 retreat window
    // for any cavalry/camel defender, so ResolveMelee + RetreatBeforeMelee get
    // exercised whenever melees happen.
    let mut attacker_hexes: Vec<HexCoord> = state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == attacker_player)
        .map(|u| u.position)
        .collect();
    sort_dedup_hexes(&mut attacker_hexes);

    let mut enemy_hexes: Vec<HexCoord> = state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == enemy)
        .map(|u| u.position)
        .collect();
    sort_dedup_hexes(&mut enemy_hexes);

    for ahex in &attacker_hexes {
        for &dhex in &enemy_hexes {
            if !ahex.neighbors().contains(&dhex) {
                continue;
            }
            let lead_legal = state.units.iter().any(|u| {
                u.position == *ahex
                    && u.profile.identity.owner() == attacker_player
                    && state.can_melee(u.id, dhex).is_ok()
            });
            if !lead_legal {
                continue;
            }
            let Some(attack) = build_melee_attack(state, *ahex, dhex) else {
                continue;
            };
            out.push(GameEffect::DeclareMelee {
                attack,
                attacker_roll: rng.roll_d10(),
                defender_roll: rng.roll_d10(),
            });
        }
    }

    advance_after_combat_actions(state, out);
}

/// All hexes at exactly `dist` from `center` (the distance-2 ring has 12
/// hexes). Used for §7.5 retreat-before-melee destinations.
fn ring_at_distance(center: HexCoord, dist: u32) -> Vec<HexCoord> {
    let d = dist as i32;
    let mut out = Vec::new();
    for dq in -d..=d {
        for dr in -d..=d {
            let h = HexCoord::new(center.q + dq, center.r + dr);
            if center.distance(h) == dist {
                out.push(h);
            }
        }
    }
    out
}

/// Port of `omdurman-app/src/melee.rs::build_melee_attack`.
fn build_melee_attack(
    gs: &GameState,
    attacker_hex: HexCoord,
    defender_hex: HexCoord,
) -> Option<MeleeAttack> {
    let owner = gs
        .units
        .iter()
        .find(|u| u.position == attacker_hex)
        .map(|u| u.profile.identity.owner())?;
    let enemy = owner.opponent();

    let attackers: Vec<UnitId> = gs
        .units
        .iter()
        .filter(|u| u.position == attacker_hex)
        .filter(|u| u.profile.identity.owner() == owner)
        .filter(|u| u.profile.kind.may_melee_attack() && !u.state.disrupted)
        .map(|u| u.id)
        .collect();
    if attackers.is_empty() {
        return None;
    }

    let defenders: Vec<UnitId> = gs
        .units
        .iter()
        .filter(|u| u.position == defender_hex)
        .filter(|u| u.profile.identity.owner() == enemy)
        .filter(|u| u.profile.kind.may_be_melee_attacked())
        .map(|u| u.id)
        .collect();
    if defenders.is_empty() {
        return None;
    }

    let mut attacker_modifiers = vec![side_modifier(owner)];
    let defender_modifiers = vec![side_modifier(enemy)];

    // §9.232: Dervish melee penalty into entrenched trench hex.
    if owner == Player::Dervish && gs.board.is_zariba_entrenched(defender_hex) {
        attacker_modifiers.push(MeleeModifier::DervishVsTrenchedDefender);
    }

    Some(MeleeAttack {
        attacker_player: owner,
        attacker_hex,
        defender_hex,
        attackers,
        defenders,
        attacker_modifiers,
        defender_modifiers,
    })
}

fn side_modifier(player: Player) -> MeleeModifier {
    match player {
        Player::Dervish => MeleeModifier::DervishStandard,
        Player::AngloEgyptian => MeleeModifier::AngloEgyptianStandard,
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Advance-after-combat (§6.82/§7.6): eligible units may advance into a
/// newly-vacant adjacent hex.
fn advance_after_combat_actions(state: &GameState, out: &mut Vec<GameEffect>) {
    let mover = state.active_player;
    let units: Vec<(UnitId, HexCoord)> = state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == mover)
        .map(|u| (u.id, u.position))
        .collect();
    for (id, hex) in units {
        for dest in hex.neighbors() {
            if state.can_advance_after_combat(id, dest).is_ok() {
                out.push(GameEffect::AdvanceAfterCombat { unit_id: id, to: dest });
            }
        }
    }
}

/// Clone-and-try: return true if `effect` is legal in `state`. Used for the
/// variants with no `can_*` predicate.
fn try_legal(state: &GameState, effect: &GameEffect) -> bool {
    let mut clone = state.clone();
    apply_effect(&mut clone, effect).is_ok()
}

/// Royal Engineers demolition targets (§6.53). The engine accepts the commit
/// without checking the target (resolution is end-of-turn), so eligibility —
/// engineer identity, adjacency to an enemy fort or a Wall hexside — is
/// enforced here to avoid committing the unit to nothing.
fn demolition_actions(state: &GameState, out: &mut Vec<GameEffect>) {
    for eng in &state.units {
        if eng.profile.identity != UnitIdentity::RoyalEngineers
            || eng.state.disrupted
            || eng.state.demolishing
        {
            continue;
        }
        let pos = eng.position;
        let enemy = eng.profile.identity.owner().opponent();
        for nbr in pos.neighbors() {
            // Adjacent enemy fort.
            for fort in state.units.iter().filter(|u| {
                u.position == nbr
                    && u.profile.identity.owner() == enemy
                    && matches!(u.profile.kind, UnitKind::Fort { .. })
            }) {
                let e = GameEffect::Demolition {
                    unit_id: eng.id,
                    target: DemolitionTarget::Fort(fort.id),
                };
                if try_legal(state, &e) {
                    out.push(e);
                }
            }
            // Wall hexside between the engineer and this neighbour.
            if matches!(state.board.hexside_between(pos, nbr), Some(HexsideKind::Wall)) {
                let e = GameEffect::Demolition {
                    unit_id: eng.id,
                    target: DemolitionTarget::WallHexside(HexsideRef::new(pos, nbr)),
                };
                if try_legal(state, &e) {
                    out.push(e);
                }
            }
        }
    }
}

/// Artillery breaching of a Wall hexside (§6.63). The engine fully validates
/// the target (must be a Wall), range and LOS via `can_fire_at_wall`, so this
/// is clone-and-try over (artillery firer × Wall hexside). The Wall set is
/// small and bounded by the map.
fn artillery_breach_actions(state: &GameState, rng: &mut BotRng, out: &mut Vec<GameEffect>) {
    let firer = match state.phase {
        Phase::OffensiveFire(_) => state.active_player,
        Phase::DefensiveFire(_) => state.active_player.opponent(),
        _ => return,
    };
    let artillery: Vec<UnitId> = state
        .units
        .iter()
        .filter(|u| {
            u.profile.identity.owner() == firer
                && matches!(u.profile.weapon, WeaponClass::Artillery)
                && u.profile.fire.is_some()
                && !state.units_fired_this_phase.contains(&u.id)
        })
        .map(|u| u.id)
        .collect();
    if artillery.is_empty() {
        return;
    }
    let walls: Vec<HexsideRef> = state
        .board
        .hexsides
        .iter()
        .filter(|(_, k)| **k == HexsideKind::Wall)
        .map(|(h, _)| *h)
        .collect();
    for &id in &artillery {
        for &target in &walls {
            let e = GameEffect::ArtilleryBreachWall {
                firers: vec![id],
                target,
                roll: rng.roll_d10(),
            };
            if try_legal(state, &e) {
                out.push(e);
            }
        }
    }
}

// Weapon-class detection used nowhere yet but kept for future howitzer routing.
#[allow(dead_code)]
fn weapon_of(state: &GameState, id: UnitId) -> WeaponClass {
    state
        .find_unit(id)
        .map(|u| u.profile.weapon)
        .unwrap_or(WeaponClass::Rifles)
}

// Hexside helpers kept for future zariba-construction routing.
#[allow(dead_code)]
fn hexside_of(a: HexCoord, b: HexCoord) -> HexsideRef {
    HexsideRef::new(a, b)
}

#[allow(dead_code)]
fn is_artillery(state: &GameState, id: UnitId) -> bool {
    state
        .find_unit(id)
        .is_some_and(|u| matches!(u.profile.weapon, WeaponClass::Artillery))
}

#[allow(dead_code)]
fn is_fort(state: &GameState, id: UnitId) -> bool {
    state
        .find_unit(id)
        .is_some_and(|u| matches!(u.profile.kind, UnitKind::Fort { .. }))
}

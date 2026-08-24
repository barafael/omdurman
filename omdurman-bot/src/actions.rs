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
    DemolitionTarget, FireAttack, FireFactor, FireKind, MeleeAttack, MovementPoints, Phase,
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
/// pre-rolled and embedded. Always includes `AdvancePhase` (except during
/// Setup while deployment candidates remain -- see below). The caller picks
/// one at random and applies it.
pub fn legal_actions(state: &GameState, rng: &mut BotRng) -> Vec<GameEffect> {
    let mut out = Vec::new();
    match state.phase {
        Phase::Setup => setup_actions(state, rng, &mut out),
        Phase::Movement => movement_actions(state, rng, &mut out),
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_) => fire_actions(state, rng, &mut out),
        Phase::Melee => melee_actions(state, rng, &mut out),
    }
    // Allow the player to end the phase -- but never while mandatory setup
    // or scheduled arrivals are still on the table: leaving Setup with a
    // skeleton force makes agent playthroughs vacuous (the engine's
    // `setup_complete` is only a minimum), and a Movement phase ended before
    // the §9.112/§9.113 wave (or the once-per-game §8.2 desertion roll) is
    // applied loses those units forever -- the arrival schedule is not
    // optional. A declared-but-unresolved melee (§7.5) also blocks the phase
    // end -- the engine rejects the advance, so never offer it.
    let deploying = state.phase == Phase::Setup
        && out.iter().any(|e| matches!(e, GameEffect::DeployUnit(_)));
    let mandatory_arrival = state.phase == Phase::Movement
        && out.iter().any(|e| {
            matches!(
                e,
                GameEffect::PlaceReinforcements(_) | GameEffect::DervishDesertion { .. }
            )
        });
    let melee_pending = state.phase == Phase::Melee && state.pending_melee.is_some();
    if !deploying && !mandatory_arrival && !melee_pending {
        out.push(GameEffect::AdvancePhase);
    }
    // Trim if the list exploded.
    if out.len() > MAX_CANDIDATES {
        rng.shuffle(&mut out);
        out.truncate(MAX_CANDIDATES);
        // Re-add the must-keep actions (they may have been truncated):
        // AdvancePhase (still suppressed mid-deployment / mid-arrival /
        // mid-melee), the §8.2 desertion roll, and one §9.112/§9.113
        // reinforcement batch -- losing a mandatory arrival to truncation
        // would skip a rules step.
        if !deploying && !mandatory_arrival && !melee_pending {
            out.push(GameEffect::AdvancePhase);
        }
        if !out
            .iter()
            .any(|e| matches!(e, GameEffect::DervishDesertion { .. }))
            && let Some(effect) = dervish_desertion_action(state, rng) {
                out.push(effect);
            }
        if !out
            .iter()
            .any(|e| matches!(e, GameEffect::PlaceReinforcements(_)))
        {
            reinforcement_actions(state, rng, &mut out);
        }
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

    // 2. Player-deployable OOB units not yet on the board.
    //    - Campaign: only the §9.111 Dervish initial force (Isa Zachneih,
    //      the Khalifa, artillery, Taiasha, the forts, the gunboats); the
    //      Anglo-Egyptian side deploys nothing (§9.113 -- it enters as
    //      reinforcements from turn 1).
    //    - Other scenarios: the full order of battle, not just the engine
    //      minimum (`setup_target_met`) -- a playthrough with a skeleton
    //      force exercises nothing. Units that cannot find a legal hex simply
    //      stop generating candidates, so deployment ends when everything
    //      placeable is placed.
    let scenario = state.scenario;
    let mut any_pending = false;
    for player in [Player::AngloEgyptian, Player::Dervish] {
        let already_ids: Vec<UnitId> = state.units.iter().map(|u| u.id).collect();
        let mut to_deploy: Vec<UnitId> = oob::deployable_oob_for(scenario, player)
            .into_iter()
            .filter(|id| !already_ids.contains(id))
            .filter(|id| initial_setup_force(scenario, player, *id, state))
            .collect();
        if !to_deploy.is_empty() {
            any_pending = true;
        }
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

    // 3. Confirm readiness only once nothing placeable remains: confirming
    //    mid-deployment is one-way, and readiness while units are still
    //    pending would let a driver stop deploying early.
    if !any_pending {
        for player in [Player::AngloEgyptian, Player::Dervish] {
            if state.setup_target_met(player) && !state.setup_ready(player) {
                out.push(GameEffect::ConfirmSetupReady { player });
            }
        }
    }
}

/// Whether `id` belongs to the scenario's *initial* (at-setup) force.
/// Campaign (§9.111/§9.113): the Dervish initial force only -- Isa Zachneih,
/// the Khalifa, the artillery, the Taiasha bodyguard, the forts and the two
/// gunboats; the Anglo-Egyptian side deploys nothing (reinforcements from
/// turn 1). Fall of Khartoum (§9.321/§9.322): both orders of battle with
/// their exact per-type counts (mirrors the engine's `fok_setup_cap` gate --
/// a candidate the engine rejects would stall the driver).
fn initial_setup_force(scenario: Scenario, player: Player, id: UnitId, state: &GameState) -> bool {
    let Some(p) = profile_for_unit(id) else {
        return false;
    };
    match scenario {
        // Campaign: only the §9.111 Dervish initial force deploys at setup;
        // the Anglo-Egyptian side deploys nothing (§9.113 -- reinforcements
        // from turn 1).
        Scenario::Campaign => {
            player == Player::Dervish
                && matches!(
                    p.identity,
                    UnitIdentity::DervishTribal { tribe: omdurman_types::DervishTribe::IsaZachneih }
                        | UnitIdentity::DervishTribal { tribe: omdurman_types::DervishTribe::Taiasha }
                        | UnitIdentity::DervishLeader(omdurman_rules::DervishLeader::KhalifaAbdullah)
                        | UnitIdentity::DervishArtillery
                        | UnitIdentity::DervishFort
                        | UnitIdentity::DervishGunboat(_)
                )
        }
        // Historical: GORDON and the "Friendlies" are not in play (§9.211);
        // Isa Zachneih, gunboats and forts are not in play (§9.212).
        Scenario::Historical => match p.identity {
            UnitIdentity::AngloEgyptianLeader(omdurman_rules::BritishLeader::Gordon) => false,
            identity if identity.is_friendlies() => false,
            UnitIdentity::DervishTribal { tribe: omdurman_types::DervishTribe::IsaZachneih } => false,
            UnitIdentity::DervishGunboat(_) | UnitIdentity::DervishFort => false,
            _ => true,
        },
        // Fall of Khartoum: the §9.321/§9.322 orders of battle, each type up
        // to its printed count (plus the scenario-fixed GORDON and North
        // Fort, which deploy via `fixed_placements`).
        Scenario::FallOfKhartoum => {
            if matches!(
                p.identity,
                UnitIdentity::AngloEgyptianLeader(omdurman_rules::BritishLeader::Gordon)
                    | UnitIdentity::DervishFort
            ) {
                // Fixed placements handle these; the free pool offers none.
                return false;
            }
            state.fok_setup_slots_remaining(&p.identity)
                .is_some_and(|n| n > 0)
        }
    }
}

/// Generate the active player's Campaign reinforcement arrivals
/// (§9.112/§9.113): one `PlaceReinforcements` batch per call, leaders first
/// (free), then gunboats (three per turn) and land units under the wave's
/// cap. Entry hexes are approximated from board geometry (see
/// [`reinforcement_entry_hex`]); the engine validates wave membership and
/// quotas regardless of placement.
fn reinforcement_actions(state: &GameState, rng: &mut BotRng, out: &mut Vec<GameEffect>) {
    use omdurman_rules::reinforcements::{
        anglo_egyptian_campaign_schedule, dervish_campaign_schedule, CampaignLeader,
    };

    if state.scenario != Scenario::Campaign || state.phase != Phase::Movement {
        return;
    }
    let player = state.active_player;
    let schedule = match player {
        Player::Dervish => dervish_campaign_schedule(),
        Player::AngloEgyptian => anglo_egyptian_campaign_schedule(),
    };
    let turn = state.current_turn.value();
    let Some(wave) = schedule.wave_for_turn(turn) else {
        return;
    };

    let already_ids: Vec<UnitId> = state.units.iter().map(|u| u.id).collect();
    let waiting: Vec<UnitId> = oob::deployable_oob_for(state.scenario, player)
        .into_iter()
        .filter(|id| !already_ids.contains(id))
        .collect();

    let eligible = |profile: &omdurman_rules::UnitProfile| -> bool {
        match profile.identity {
            UnitIdentity::DervishTribal { tribe } => wave.tribes.contains(&tribe),
            UnitIdentity::DervishLeader(leader) => wave
                .leaders
                .iter()
                .any(|l| matches!(l, CampaignLeader::Dervish(d) if *d == leader)),
            UnitIdentity::AngloEgyptianLeader(leader) => wave.leaders.iter().any(|l| {
                matches!(l, CampaignLeader::British(d) if *d == leader)
            }),
            _ if player == Player::Dervish => false,
            _ => true, // AE non-leader: kind eligibility via quotas below
        }
    };

    let mut batch: Vec<omdurman_rules::UnitPlacement> = Vec::new();
    // §9.113 quotas apply per *turn*, not per batch: count what this player
    // already entered this movement phase (earlier batches) so a follow-up
    // batch cannot exceed the cap / 3-gunboat quota.
    let entered_this_turn = &state.reinforcements_placed_this_turn;
    let entered: Vec<UnitId> = entered_this_turn
        .iter()
        .filter(|(p, _)| *p == player)
        .map(|(_, id)| *id)
        .collect();
    let mut boats: usize = entered
        .iter()
        .filter(|id| {
            profile_for_unit(**id).is_some_and(|pr| matches!(pr.kind, UnitKind::Gunboat { .. }))
        })
        .count();
    let mut land: usize = entered
        .iter()
        .filter(|id| {
            let pr = profile_for_unit(**id);
            let is_leader = pr
                .is_some_and(|pr| matches!(pr.identity, UnitIdentity::AngloEgyptianLeader(_)));
            let is_boat = pr.is_some_and(|pr| matches!(pr.kind, UnitKind::Gunboat { .. }));
            !is_leader && !is_boat
        })
        .count();
    let land_cap = wave.unit_cap; // per-turn cap; None = uncapped (§9.113 T4)
    let batch_bound = 1 + MAX_SETUP_CANDIDATES.max(3); // bounded candidate list
    // Staging clone: entry hexes are chosen against the board *plus* the
    // in-flight batch, so cumulative stacking (§5.51-5.53) holds across the
    // batch and the engine's `can_place_reinforcements` accepts it whole.
    let mut probe = state.clone();
    for id in &waiting {
        if batch.len() >= batch_bound {
            break; // bounded batch per candidate list
        }
        let Some(profile) = profile_for_unit(*id) else { continue };
        if !eligible(&profile) {
            continue;
        }
        let is_leader = matches!(profile.identity, UnitIdentity::AngloEgyptianLeader(_));
        let is_boat = matches!(profile.kind, UnitKind::Gunboat { .. });
        if !is_leader {
            if is_boat {
                if boats >= 3 {
                    continue;
                }
            } else if let Some(cap) = land_cap
                && land >= cap {
                    continue;
                }
        }
        let Some(hex) = reinforcement_entry_hex(&probe, &profile, rng) else {
            continue;
        };
        let placement = omdurman_rules::UnitPlacement {
            id: *id,
            position: hex,
            profile,
            state: UnitState::default(),
        };
        batch.push(placement);
        // Stage onto the probe so later picks respect cumulative stacking
        // (§5.51-5.53: no tribe mixes, no over-stack across the batch).
        probe.units.push(*batch.last().unwrap());
        if is_boat {
            boats += 1;
        } else if !is_leader {
            land += 1;
        }
    }
    if !batch.is_empty() {
        out.push(GameEffect::PlaceReinforcements(batch));
    }
}

/// Pick an entry hex for a reinforcing unit, approximating the manual's
/// entry areas from board geometry (the compiled board carries no entry-area
/// annotations):
/// - Dervish (§9.112): the west map edge, south of the Khor Shambat.
/// - AE land (§9.113): the north-west land edge (the entrance area).
/// - AE gunboats (§9.113): the northmost Nile hexes.
/// - "Friendlies" (§9.113): the east land edge (the Abu Alim hut side).
///   Each candidate must satisfy stacking (§5.51-5.53).
fn reinforcement_entry_hex(
    state: &GameState,
    profile: &omdurman_rules::UnitProfile,
    rng: &mut BotRng,
) -> Option<HexCoord> {
    use omdurman_types::Terrain;

    let is_boat = matches!(profile.kind, UnitKind::Gunboat { .. });

    // Authored entrance areas (§9.112/§9.113) are authoritative when present:
    // pick a stacking-legal hex from the annotation before falling back to
    // the geometric approximation below.
    let area = match profile.identity {
        UnitIdentity::DervishLeader(_) | UnitIdentity::DervishTribal { .. } => {
            Some(omdurman_types::NamedArea::DervishWestEdge)
        }
        UnitIdentity::AngloEgyptianLeader(_) => {
            Some(omdurman_types::NamedArea::AngloEgyptianEntrance)
        }
        _ if is_boat => Some(omdurman_types::NamedArea::GunboatNorthEdge),
        _ if profile.identity.is_friendlies() => Some(omdurman_types::NamedArea::AbuAlimHut),
        _ => Some(omdurman_types::NamedArea::AngloEgyptianEntrance),
    };
    if let Some(area) = area {
        let mut annotated = state.board.entrance_hexes(area);
        if !annotated.is_empty() {
            rng.shuffle(&mut annotated);
            let owner = profile.identity.owner();
            if let Some(hex) = annotated.into_iter().find(|h| {
                // §7.1: an arrival may not appear on top of enemy units
                // (lone AE leaders excepted -- they are overrun, §6.51).
                let enemy = owner.opponent();
                if state.units.iter().any(|u| {
                    u.position == *h
                        && u.profile.identity.owner() == enemy
                        && !matches!(
                            u.profile.kind,
                            omdurman_types::UnitKind::BritishLeader { .. }
                        )
                }) {
                    return false;
                }
                let placement = omdurman_rules::UnitPlacement {
                    id: UnitId::Kitchener_0_0, // dummy id: check_stacking ignores id-equality
                    position: *h,
                    profile: *profile,
                    state: UnitState::default(),
                };
                state.check_stacking(&placement, *h).is_ok()
            }) {
                return Some(hex);
            }
            // Every annotated hex is stacked full: no legal entry this
            // iteration (the driver will retry after the occupant moves).
            return None;
        }
    }

    let min_r = state.board.terrain.keys().map(|h| h.r).min()?;

    // Khor Shambat's southernmost extent (hexsides of that kind).
    let khor_max_r: Option<i32> = {
        let mut any: Option<i32> = None;
        for (hs, kind) in &state.board.hexsides {
            if matches!(kind, omdurman_types::HexsideKind::KhorShambat) {
                any = Some(any.map_or(hs.a.r.max(hs.b.r), |m: i32| m.max(hs.a.r).max(hs.b.r)));
            }
        }
        any
    };

    // Row-wise edges: the westernmost / easternmost *land* hex of each map
    // row. (The literal min-/max-q columns are mostly Nile slivers on this
    // map, so a global q bound is a poor edge proxy.)
    use std::collections::BTreeMap;
    let mut west_edge_by_row: BTreeMap<i32, HexCoord> = BTreeMap::new();
    let mut east_edge_by_row: BTreeMap<i32, HexCoord> = BTreeMap::new();
    for (&h, terrain) in &state.board.terrain {
        if matches!(terrain, Terrain::Nile { .. }) {
            continue;
        }
        west_edge_by_row
            .entry(h.r)
            .and_modify(|e| {
                if h.q < e.q {
                    *e = h;
                }
            })
            .or_insert(h);
        east_edge_by_row
            .entry(h.r)
            .and_modify(|e| {
                if h.q > e.q {
                    *e = h;
                }
            })
            .or_insert(h);
    }

    let candidates: Vec<HexCoord> = match profile.identity.owner() {
        Player::Dervish => {
            // §9.112: west edge, south of the Khor Shambat.
            west_edge_by_row
                .into_iter()
                .filter(|(r, _)| khor_max_r.is_none_or(|kr| *r > kr))
                .map(|(_, h)| h)
                .collect()
        }
        Player::AngloEgyptian => {
            if is_boat {
                // §9.113: north-edge Nile hexes.
                state
                    .board
                    .terrain
                    .iter()
                    .filter(|(h, t)| h.r <= min_r + 2 && matches!(t, Terrain::Nile { .. }))
                    .map(|(h, _)| *h)
                    .collect()
            } else if profile.identity.is_friendlies() {
                // §9.113: the Abu Alim hut on the east bank.
                east_edge_by_row.into_values().collect()
            } else {
                // §9.113: the entrance area at the map's north-west.
                west_edge_by_row
                    .into_iter()
                    .filter(|(r, _)| *r <= min_r + 3)
                    .map(|(_, h)| h)
                    .collect()
            }
        }
    };

    let mut candidates = candidates;
    rng.shuffle(&mut candidates);
    candidates.into_iter().find(|h| {
        let placement = omdurman_rules::UnitPlacement {
            id: UnitId::Kitchener_0_0, // dummy id: check_stacking ignores id-equality
            position: *h,
            profile: *profile,
            state: UnitState::default(),
        };
        state.check_stacking(&placement, *h).is_ok()
    })
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
    // Sort before shuffling: `terrain` is a HashMap, so its iteration order is
    // per-process random -- shuffling an unordered vector would make the
    // same seed play different games across processes (the log/replay
    // determinism guarantee requires a stable pre-shuffle order).
    sort_dedup_hexes(&mut hexes);
    rng.shuffle(&mut hexes);
    // Descending: compatible hexes (1) first, empty (0) next, incompatible
    // (-1) last. Ascending order preferred *empty* hexes, carpeting each
    // tribe one-unit-per-hex -- with a bounded deployment zone (FoK's
    // south/east edge, §9.322) that strands later tribes when every zone
    // hex holds a foreign tribe (§5.52 forbids the mix).
    hexes.sort_by_key(|h| -hex_deploy_preference(state, *h, profile));
    let result = hexes.into_iter().find(|h| {
        let placement = omdurman_rules::UnitPlacement {
            id,
            position: *h,
            profile,
            state: UnitState::default(),
        };
        state.can_deploy_unit(&placement).is_ok()
    });
    result
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

fn movement_actions(state: &GameState, rng: &mut BotRng, out: &mut Vec<GameEffect>) {
    // §8.2: once per Campaign game, during the first night turn's movement
    // phase, the Dervish player rolls a die and removes 1.5x the roll in
    // units of his choice (Khalifa, gunboats, artillery, forts exempt).
    if let Some(effect) = dervish_desertion_action(state, rng) {
        out.push(effect);
    }
    // §9.112/§9.113: the Campaign order of appearance -- each side's wave
    // enters during its movement phase.
    reinforcement_actions(state, rng, out);
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
            if let Some(cost) = step_cost(state, unit.position, dest)
                && state
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

/// The §8.2 Dervish desertion roll, when the conditions hold: the Campaign
/// game's first night turn, during the Dervish movement phase, not yet
/// performed. Pre-rolls the die and picks `floor(1.5 x roll)` eligible
/// deserters (the Khalifa, gunboats, artillery and forts are exempt; the
/// choosing player in the manual is modelled as first-listed units).
fn dervish_desertion_action(state: &GameState, rng: &mut BotRng) -> Option<GameEffect> {
    use omdurman_rules::turn_track::scenario_turn;
    if state.scenario != Scenario::Campaign
        || state.active_player != Player::Dervish
        || state.phase != Phase::Movement
        || state.dervish_deserted
    {
        return None;
    }
    let is_desertion_turn = scenario_turn(state.scenario, state.current_turn).is_some_and(|t| {
        t.event == omdurman_rules::turn_track::TurnEvent::DervishDesertion
            && t.day_night == omdurman_types::DayNight::Night
    });
    if !is_desertion_turn {
        return None;
    }
    let roll = rng.roll_d10();
    let expected = (3 * roll.value() as usize) / 2; // floor(1.5 x roll), §8.2
    let mut candidates: Vec<UnitId> = state
        .units
        .iter()
        .filter(|u| {
            u.profile.identity.owner() == Player::Dervish
                && !u.profile.identity.is_desertion_exempt()
        })
        .map(|u| u.id)
        .collect();
    rng.shuffle(&mut candidates);
    candidates.truncate(expected);
    Some(GameEffect::DervishDesertion {
        roll,
        deserters: candidates,
    })
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

    // Enumerate enemy-occupied hexes as candidate targets. §6.14: a combat
    // unit may only be fired at once per phase (Maxims/gunboats excepted).
    // An attack on a hex targets *all* its occupants, so a hex holding any
    // already-fired-at non-excepted unit is not a legal target again this
    // phase.
    let enemy = firer_player.opponent();
    let fired_at = |u: &omdurman_rules::UnitPlacement| {
        let excepted = matches!(
            u.profile.kind,
            UnitKind::Gunboat { .. } | UnitKind::Maxim { .. }
        );
        !excepted && state.units_fired_at_this_phase.contains(&u.id)
    };
    let mut target_hexes: Vec<HexCoord> = state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == enemy)
        .map(|u| u.position)
        .collect();
    sort_dedup_hexes(&mut target_hexes);
    target_hexes.retain(|h| !state.units.iter().any(|u| u.position == *h && fired_at(u)));

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
    _state: &GameState,
    _firer_player: Player,
    sub: omdurman_rules::FireSubPhase,
) -> FireKind {
    match sub {
        omdurman_rules::FireSubPhase::DirectFire => FireKind::Direct,
        omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => {
            // For simplicity the bot defaults to MaximSecond; individual firers
            // will be filtered by `can_fire_at` if they can't use it. Selecting
            // the kind from the player's weapons would need `_state` /
            // `_firer_player` — a future refinement.
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

    // §6.24/§5.54/§9.231/§9.232: the engine derives the mandatory modifier
    // set (and rejects any other list), so build the attack with the engine's
    // own helper -- including the Dervish-only zariba penalties the previous
    // local assembly got wrong for Anglo-Egyptian attacks.
    let mut attack = FireAttack {
        firing_player: owner,
        phase: gs.phase,
        kind,
        firers: firers.iter().map(|u| u.id).collect(),
        target_hex: target,
        factor_row,
        modifiers: Vec::new(),
    };
    attack.modifiers = omdurman_rules::effects::mandatory_fire_modifiers(gs, &attack);
    Some(attack)
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

    // §7.7/§9.232: engine-derived mandatory modifiers (Dervish +2 / AE +1,
    // trench −2), single source of truth with resolution.
    let mut attack = MeleeAttack {
        attacker_player: owner,
        attacker_hex,
        defender_hex,
        attackers,
        defenders,
        attacker_modifiers: Vec::new(),
        defender_modifiers: Vec::new(),
    };
    let (att, def) = omdurman_rules::effects::mandatory_melee_modifiers(gs, &attack);
    attack.attacker_modifiers = att;
    attack.defender_modifiers = def;
    Some(attack)
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

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_rules::GameTurnIndex;
    use omdurman_types::DayNight;

    /// A Campaign state standing at the first night turn's Dervish movement
    /// phase (§8.2: the desertion window).
    fn first_night_dervish_movement() -> GameState {
        let mut state = GameState::new(Scenario::Campaign);
        state.current_turn = GameTurnIndex::new(9);
        state.day_night = DayNight::Night;
        state.active_player = Player::Dervish;
        state.phase = Phase::Movement;
        // A tribal unit eligible to desert + an exempt fort.
        let fort = state.alloc_unit_id();
        state.units.push(omdurman_rules::UnitPlacement {
            id: fort,
            position: HexCoord::new(10, 10),
            profile: omdurman_rules::UnitProfile {
                kind: UnitKind::Fort { fire: 0, melee: 0 },
                identity: UnitIdentity::DervishFort,
                weapon: WeaponClass::Artillery,
                fire: None,
                melee: None,
                movement: UnitMovement::Immobile,
            },
            state: UnitState::default(),
        });
        state
    }

    #[test]
    fn desertion_offered_once_in_first_night_movement() {
        let mut state = first_night_dervish_movement();
        let tribal = state.alloc_unit_id();
        state.units.push(omdurman_rules::UnitPlacement {
            id: tribal,
            position: HexCoord::new(11, 10),
            profile: omdurman_rules::UnitProfile {
                kind: UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
                identity: UnitIdentity::DervishTribal {
                    tribe: omdurman_types::DervishTribe::Baggara,
                },
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(omdurman_rules::MovementAllowance::Nine),
            },
            state: UnitState::default(),
        });

        let mut rng = crate::rng::BotRng::from_seed(7);
        let actions = legal_actions(&state, &mut rng);
        let offered: Vec<&GameEffect> = actions
            .iter()
            .filter(|e| matches!(e, GameEffect::DervishDesertion { .. }))
            .collect();
        assert_eq!(offered.len(), 1, "§8.2: the roll is offered in the window");

        // Applying it consumes the once-per-game flag...
        let mut applied = state.clone();
        apply_effect(&mut applied, offered[0]).unwrap();
        assert!(applied.dervish_deserted);
        // ...and it is never offered again.
        let mut rng2 = crate::rng::BotRng::from_seed(7);
        assert!(legal_actions(&applied, &mut rng2)
            .iter()
            .all(|e| !matches!(e, GameEffect::DervishDesertion { .. })));

        // Not offered outside the window either (a day turn).
        let mut day = state.clone();
        day.day_night = DayNight::Day;
        day.current_turn = GameTurnIndex::new(5);
        let mut rng3 = crate::rng::BotRng::from_seed(7);
        assert!(legal_actions(&day, &mut rng3)
            .iter()
            .all(|e| !matches!(e, GameEffect::DervishDesertion { .. })));
    }
}

#[cfg(test)]
mod campaign_schedule_tests {
    use super::*;
    use omdurman_rules::board_data::campaign_map_data;
    use omdurman_rules::board::BoardInfo;

    fn campaign_movement(player: Player) -> GameState {
        let mut state =
            GameState::with_board(Scenario::Campaign, BoardInfo::from_map_data(&campaign_map_data()));
        state.phase = Phase::Movement;
        state.active_player = player;
        state
    }

    #[test]
    fn campaign_setup_deploys_only_initial_force() {
        // §9.111/§9.113: the Campaign setup offers only the Dervish initial
        // force -- no wave tribes (Baggara), no Anglo-Egyptian units, and no
        // GORDON (§9.113: not used in this scenario).
        let mut state = campaign_movement(Player::Dervish);
        state.phase = Phase::Setup;
        let mut rng = crate::rng::BotRng::from_seed(11);
        let actions = legal_actions(&state, &mut rng);
        for e in &actions {
            if let GameEffect::DeployUnit(p) = e {
                let is_initial = initial_setup_force(
                    Scenario::Campaign,
                    p.profile.identity.owner(),
                    p.id,
                    &state,
                );
                assert!(
                    is_initial,
                    "non-initial-force unit offered at setup: {:?}",
                    p.profile.identity
                );
                assert_ne!(
                    p.profile.identity.owner(),
                    Player::AngloEgyptian,
                    "AE unit offered at Campaign setup (§9.113)"
                );
                assert!(
                    !p.profile.identity.is_gordon(),
                    "GORDON offered in the Campaign (§9.113: not used)"
                );
            }
        }
    }

    #[test]
    fn campaign_turn_one_offers_both_waves() {
        // §9.112: the Dervish turn-1 movement phase offers wave 1 (Baggara,
        // Jaalin, Danagla, Kehena, Degheim + three leaders).
        let state = campaign_movement(Player::Dervish);
        let mut rng = crate::rng::BotRng::from_seed(3);
        let actions = legal_actions(&state, &mut rng);
        let mut saw_wave = false;
        for e in &actions {
            if let GameEffect::PlaceReinforcements(batch) = e {
                saw_wave = true;
                for p in batch {
                    let ok = match p.profile.identity {
                        UnitIdentity::DervishTribal { tribe } => matches!(
                            tribe,
                            omdurman_types::DervishTribe::Baggara
                                | omdurman_types::DervishTribe::Jaalin
                                | omdurman_types::DervishTribe::Danagla
                                | omdurman_types::DervishTribe::Kehena
                                | omdurman_types::DervishTribe::Degheim
                        ),
                        UnitIdentity::DervishLeader(_) => true,
                        _ => false,
                    };
                    assert!(ok, "non-wave-1 unit in the Dervish T1 batch: {:?}", p.profile.identity);
                }
            }
        }
        assert!(saw_wave, "Dervish T1 movement offers no reinforcements");

        // §9.113: the AE turn-1 movement phase offers its wave too.
        let state = campaign_movement(Player::AngloEgyptian);
        let mut rng = crate::rng::BotRng::from_seed(4);
        let actions = legal_actions(&state, &mut rng);
        assert!(
            actions
                .iter()
                .any(|e| matches!(e, GameEffect::PlaceReinforcements(_))),
            "AE T1 movement offers no reinforcements"
        );
    }
}

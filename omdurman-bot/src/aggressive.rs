//! The greedy-aggressor strategy ([`crate::agent::AgentStrategy::Aggressive`]).
//!
//! Scores every legal action and always takes the best (ties broken randomly
//! through the shared `BotRng`, so runs stay seed-reproducible). The doctrine
//! is the historical FoK Dervish plan: mass the tribes, charge the objective,
//! never retreat:
//!
//! - **Objective-seeking movement.** Every `MoveUnit` is scored by hex
//!   progress toward the side's goal — the Palace (GORDON, §9.346) for the
//!   Dervish in Fall of Khartoum, the nearest enemy unit otherwise. A step
//!   that closes distance always beats ending the phase; lateral or backward
//!   steps score below it.
//! - **Melee over fire over movement** (§7 beats §6 for an army with the
//!   numbers): resolving/declaring a melee scores highest, then
//!   advance-after-combat (it closes distance), then fire combat, then wall
//!   breaching (it opens Khartoum's walls, §6.63).
//! - **Never retreat** (§7.5): `RetreatBeforeMelee` scores far below
//!   everything, including ending the phase.
//! - **End the phase only when nothing better remains.** `AdvancePhase`
//!   scores a flat 1; any scored action above it wins, so the side acts
//!   until its legal surface is exhausted, then advances.

use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{Phase, UnitId};
use omdurman_types::{HexsideKind, Location, Player, Scenario};

use crate::rng::BotRng;

/// Pick the best-scoring legal action (ties broken by rng). `player` is the
/// acting side (the phase's candidate owner), which may differ from
/// `state.active_player` in defensive fire (§6.7).
pub fn pick(
    state: &GameState,
    player: Player,
    candidates: &[GameEffect],
    rng: &mut BotRng,
) -> GameEffect {
    let goal = objective_hex(state, player);
    let mut best: i32 = i32::MIN;
    let mut best_idxs: Vec<usize> = Vec::new();
    for (i, effect) in candidates.iter().enumerate() {
        let score = score(effect, state, goal);
        if score > best {
            best = score;
            best_idxs.clear();
            best_idxs.push(i);
        } else if score == best {
            best_idxs.push(i);
        }
    }
    let pick = rng.choose(&best_idxs).copied().unwrap_or(0);
    candidates[pick].clone()
}

/// Returns `true` if the Dervish side in FoK has at least one wall hexside
/// that has been breached (flipped Wall → Breach by artillery §6.63 or
/// engineers §6.53). Once a breach exists, units can march through it
/// toward the Palace; before that, movement should approach the wall.
fn any_breach_exists(state: &GameState) -> bool {
    state
        .board
        .hexsides
        .values()
        .any(|k| *k == HexsideKind::Breach)
}

/// The side's goal hex. For the Dervish in FoK:
/// - **Before any wall breach:** the nearest wall hex — forces units to
///   approach the wall and breach it, rather than flanking through the
///   western gap.
/// - **After a breach:** the Palace (GORDON's post, §9.346) — units march
///   through the breach toward the objective.
///
/// For other sides/scenarios: the average position of enemy units.
fn objective_hex(state: &GameState, player: Player) -> Option<omdurman_types::HexCoord> {
    if state.scenario == Scenario::FallOfKhartoum && player == Player::Dervish {
        // After a breach exists, march on the Palace.
        if any_breach_exists(state) {
            return state.board.hex_of_location(Location::Palace);
        }
        // Before a breach: find the nearest wall hexside and use its
        // southern hex (the Dervish side) as the goal. This pushes units
        // toward the wall rather than around it through the western gap.
        let wall_goals: Vec<omdurman_types::HexCoord> = state
            .board
            .hexsides
            .iter()
            .filter(|(_, kind)| **kind == HexsideKind::Wall)
            .map(|(hr, _)| hr.b)
            .collect();
        if wall_goals.is_empty() {
            return state.board.hex_of_location(Location::Palace);
        }
        // Find the Dervish unit closest to any wall hex; use that wall hex
        // as the goal so the scoring pushes this unit (and nearby units)
        // toward the wall.
        let dervish_positions: Vec<omdurman_types::HexCoord> = state
            .units
            .iter()
            .filter(|u| u.profile.identity.owner() == Player::Dervish)
            .map(|u| u.position)
            .collect();
        if dervish_positions.is_empty() {
            return wall_goals.first().copied();
        }
        let mut best_dist = i32::MAX;
        let mut best_goal = wall_goals[0];
        for &wh in &wall_goals {
            let min_dist = dervish_positions
                .iter()
                .map(|dp| dp.distance(wh) as i32)
                .min()
                .unwrap_or(i32::MAX);
            if min_dist < best_dist {
                best_dist = min_dist;
                best_goal = wh;
            }
        }
        return Some(best_goal);
    }
    let enemy = player.opponent();
    let mut sum_q = 0i64;
    let mut sum_r = 0i64;
    let mut n = 0i64;
    for u in &state.units {
        if u.profile.identity.owner() == enemy {
            sum_q += u.position.q as i64;
            sum_r += u.position.r as i64;
            n += 1;
        }
    }
    (n > 0).then(|| omdurman_types::HexCoord::new((sum_q / n) as i32, (sum_r / n) as i32))
}

/// Score one candidate action. Higher = more aggressive toward the objective.
fn score(effect: &GameEffect, state: &GameState, goal: Option<omdurman_types::HexCoord>) -> i32 {
    use GameEffect::*;
    match effect {
        // Melee is the point of the army (§7). Resolve pending melees first
        // (nothing else is legal anyway), then declare new ones.
        ResolveMelee | MeleeCombat { .. } => 100,
        DeclareMelee { .. } => 90,
        // Take ground (§6.82/§7.6): progress-scored like movement, plus a
        // base above fire so a vacated hex is always taken.
        AdvanceAfterCombat { unit_id, to } => 60 + progress(state, *unit_id, *to, goal),
        // Breach the walls of Khartoum (§6.63) -- opens the direct route in.
        // Score high so the fallback breaches before marching through.
        ArtilleryBreachWall { .. } => 70,
        // Fire while the enemy is in reach (§6), but never at the expense of
        // closing.
        FireCombat { .. } | HowitzerFire { .. } => 40,
        // Objective-seeking movement (§5): base 20 so any closing or holding
        // step beats ending the phase, plus 10 per hex of progress.
        MoveUnit { unit_id, to, .. } => 20 + 10 * progress(state, *unit_id, *to, goal),
        // Reinforcements (§9.112/§9.113) and the §8.2 desertion roll: get
        // bodies on the board / mandatory bookkeeping through.
        PlaceReinforcements(_) | DervishDesertion { .. } => 30,
        // Setup: keep deploying and confirm readiness when nothing placeable
        // remains (the enumerator only offers it then).
        DeployUnit(_) => 25,
        ConfirmSetupReady { .. } => 15,
        // Demolitions (§6.53) are free shots at walls/forts.
        Demolition { .. } => 45,
        // NEVER retreat before melee (§7.5) -- below even ending the phase.
        RetreatBeforeMelee { .. } => -100,
        // Ending the phase is the leftover: taken only when nothing above
        // score 1 is available. (Setup/arrival blockers keep it out of the
        // candidate list while mandatory, §driver.)
        AdvancePhase => {
            if state.phase == Phase::Setup {
                15
            } else {
                1
            }
        }
        // Unknown variants: neutral.
        _ => 0,
    }
}

/// Hex-distance progress of moving `unit_id` to `to` toward `goal`
/// (positive = closer). 0 when there is no goal or the unit is gone.
fn progress(
    state: &GameState,
    unit_id: UnitId,
    to: omdurman_types::HexCoord,
    goal: Option<omdurman_types::HexCoord>,
) -> i32 {
    let (Some(goal), Some(unit)) = (goal, state.find_unit(unit_id)) else {
        return 0;
    };
    unit.position.distance(goal) as i32 - to.distance(goal) as i32
}

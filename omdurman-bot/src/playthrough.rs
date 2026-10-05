//! The full-game playthrough driver. Loops: enumerate legal actions → pick →
//! apply → log, until `game_over` or the anti-stall caps are hit.
//!
//! Two agents (one per faction) play head-to-head; each side is independently
//! [`crate::agent::AgentStrategy::Random`] (fast, broadest raw coverage),
//! [`crate::agent::AgentStrategy::Aggressive`] or a [`crate::agent::AgentStrategy::Commander`]. The
//! driver also drains the engine's [`Observation`](omdurman_rules::effects::Observation)s
//! and turn summaries into a [`crate::log::GameLog`] that the deterministic
//! [`audit`](crate::audit) scanners check offline.

use omdurman_net::GameEvent;
use omdurman_rules::Phase;
use omdurman_rules::board::BoardInfo;
use omdurman_rules::board_data::{campaign_map_data, fall_of_khartoum_map_data};
use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
use omdurman_types::{Player, Scenario};

use crate::actions::{legal_actions, legal_actions_deep_setup};
use crate::agent::Agents;
use crate::describe::describe_effect;
use crate::log::GameLog;
use crate::rng::BotRng;

/// Anti-stall configuration.
#[derive(Clone, Debug)]
pub struct PlayConfig {
    /// Hard cap on actions per phase before forcing `AdvancePhase`.
    pub max_actions_per_phase: usize,
    /// Hard cap on total turns before stopping.
    pub max_turns: u8,
}

impl Default for PlayConfig {
    fn default() -> Self {
        Self {
            max_actions_per_phase: 200,
            max_turns: 30,
        }
    }
}

/// The result of a full playthrough.
pub struct PlayResult {
    /// The complete event trace (natively replayable by the app's timeline).
    pub events: Vec<GameEvent>,
    /// The seed used (for reproducibility).
    pub seed: u64,
    /// The final game state.
    pub final_state: GameState,
    /// Which `GameEffect` variant kinds appeared in the trace.
    pub variant_coverage: Vec<&'static str>,
    /// Total number of actions applied.
    pub actions_taken: usize,
    /// The human-readable game log (input to the `audit` scanners).
    pub log: GameLog,
    /// Number of engine observations drained into the log.
    pub observations_total: usize,
}

/// Play a full game headlessly from setup to game-over, with two independent
/// per-faction agents.
pub fn playthrough(scenario: Scenario, seed: u64, cfg: PlayConfig, agents: Agents) -> PlayResult {
    // Build the game state with the compiled board attached.
    let board = board_for_scenario(scenario);
    let mut state = GameState::with_board(scenario, board);
    let mut rng = BotRng::from_seed(seed);

    let mut events: Vec<GameEvent> = vec![GameEvent::StartGame {
        seats: Vec::new(),
        scenario,
        optional_rules: Vec::new(),
    }];
    let mut log = GameLog::new(scenario, seed, &agents);
    let mut actions_taken = 0usize;
    let mut variant_coverage: Vec<&'static str> = Vec::new();

    let mut actions_this_phase = 0usize;
    // Intents rejected by `apply_effect` during the current phase (the
    // enumerator's predicates can be weaker than the engine, e.g. §5.52
    // tribe stacking). Filtered out of every candidate list so a bad
    // candidate is never re-picked; cleared when the phase advances.
    let mut rejected_this_phase: Vec<GameEffect> = Vec::new();
    let mut prev_summaries = 0usize;

    // Defense-in-depth against a stalled Setup phase (e.g. an unresolvable
    // leader-command deadlock): hard ceiling on driver iterations so a bug can
    // never hang the caller. Well below a full-game action count.
    let mut iterations = 0usize;
    let mut move_memory = crate::move_memory::MoveMemory::new();

    loop {
        // Termination conditions.
        if state.game_over || state.current_turn.value() > cfg.max_turns {
            break;
        }
        iterations += 1;
        if iterations > MAX_DRIVER_ITERATIONS {
            break;
        }

        // Commanders care *where* to deploy (wall line, breach axis), so give
        // them the per-unit hex options; other strategies keep the lean list.
        let mut candidates = if agents.any_commander() {
            legal_actions_deep_setup(&state, &mut rng)
        } else {
            legal_actions(&state, &mut rng)
        };
        candidates.retain(|c| !rejected_this_phase.iter().any(|r| same_intent(r, c)));
        if candidates.is_empty() {
            // If mandatory arrivals (PlaceReinforcements / DervishDesertion)
            // keep failing, the AdvancePhase was suppressed by
            // `legal_actions`. Force it through so the phase can progress.
            if !state.game_over && state.current_turn.value() <= cfg.max_turns {
                candidates.push(GameEffect::AdvancePhase);
            } else {
                break;
            }
        }

        // §8.2: the first-night-turn desertion roll is not optional -- force
        // it through before any other movement action (the candidate list
        // offers it, but neither a random pick nor a heuristic is guaranteed
        // to choose a mandatory bookkeeping roll).
        if let Some(idx) = candidates
            .iter()
            .position(|e| matches!(e, GameEffect::DervishDesertion { .. }))
        {
            let effect = candidates.remove(idx);
            let turn = state.current_turn.value();
            let phase_name = state.phase.top_level_name();
            // The *acting* side, not the turn owner: in defensive fire the
            // non-moving player performs the action (§6.7), and the trace
            // labels it by who actually did it.
            let actor = state.player_to_act().unwrap_or(state.phase_player());
            let action_text = describe_effect(&effect, &state);
            if apply_effect(&mut state, &effect).is_ok() {
                events.push(GameEvent::Effect(effect));
                log_event_and_observations(
                    &mut state,
                    &mut log,
                    events.len() - 1,
                    turn,
                    phase_name,
                    actor,
                    &action_text,
                );
                continue;
            }
        }

        // --- Pick an action ---
        let pick = if actions_this_phase >= cfg.max_actions_per_phase {
            // Anti-stall: force phase advance -- but never past a mandatory
            // arrival: a §9.112/§9.113 reinforcement wave (or the once-per-
            // game §8.2 desertion roll) that misses its movement phase is
            // lost forever.
            candidates
                .iter()
                .find(|e| {
                    matches!(
                        e,
                        GameEffect::PlaceReinforcements(_)
                            | GameEffect::DervishDesertion { .. }
                            | GameEffect::DeployUnit(_)
                    )
                })
                .or_else(|| {
                    candidates
                        .iter()
                        .find(|e| matches!(e, GameEffect::AdvancePhase))
                })
                .cloned()
                .unwrap_or(GameEffect::AdvancePhase)
        } else {
            // Whoever owns this phase's candidates: the active player, except
            // in defensive fire where the non-moving player fires (§6.7).
            // During Setup the candidates mix both sides' deployments, so a
            // commander pair scores each candidate by its owner's doctrine
            // (see `commanders::pick_setup`).
            let chooser = state.player_to_act().unwrap_or(state.phase_player());
            if state.phase == Phase::Setup && agents.any_commander() {
                crate::commanders::pick_setup(&state, &candidates, &agents, &mut rng)
            } else if agents.is_aggressive(chooser) {
                crate::aggressive::pick(&state, chooser, &candidates, &mut rng)
            } else if let Some(commander) = agents.commander(chooser) {
                // The commander's plan for each unit (`crate::move_memory`),
                // as the app and the arena drive it.
                let _ = commander;
                let pick = crate::commanders::pick_validated(
                    &state,
                    chooser,
                    &candidates,
                    &mut rng,
                    &mut move_memory,
                );
                move_memory.record(&state, &pick);
                pick
            } else {
                rng.choose(&candidates)
                    .cloned()
                    .unwrap_or(GameEffect::AdvancePhase)
            }
        };

        // --- Apply ---
        let is_advance = matches!(pick, GameEffect::AdvancePhase);
        let kind: &'static str = (&pick).into();
        // The *acting* side: fire attacks act with the firing player (which
        // in defensive fire is the non-moving player, §6.7) and melees with
        // the attacker (§7) -- not the phase's active player.
        let actor = match &pick {
            GameEffect::FireCombat { attack, .. } | GameEffect::HowitzerFire { attack, .. } => {
                attack.firing_player
            }
            GameEffect::DeclareMelee { attack, .. } => attack.attacker_player,
            GameEffect::ArtilleryBreachWall { firers, .. } => firers
                .first()
                .and_then(|id| state.find_unit(*id))
                .map(|u| u.profile.identity.owner())
                .unwrap_or(state.phase_player()),
            // Effects that don't encode their owner (movement, mines, advance)
            // belong to whoever acts in this phase — the turn owner except
            // during defensive fire, where the non-moving side acts (§6.7).
            _ => state.phase_player(),
        };
        let turn = state.current_turn.value();
        let phase_name = state.phase.top_level_name();
        // Describe against the *pre-apply* state so positions show where the
        // action started.
        let action_text = describe_effect(&pick, &state);

        match apply_effect(&mut state, &pick) {
            Ok(()) => {
                events.push(GameEvent::Effect(pick));
                if !variant_coverage.contains(&kind) {
                    variant_coverage.push(kind);
                }
                actions_taken += 1;
                log_event_and_observations(
                    &mut state,
                    &mut log,
                    events.len() - 1,
                    turn,
                    phase_name,
                    actor,
                    &action_text,
                );
            }
            Err(e) => {
                // The generator produced an illegal candidate: the enumerator
                // predicates can be weaker than `apply_effect` (e.g. a move
                // that passes `can_move_unit_to` but mixes Dervish tribes at
                // the destination, §5.52). Exclude the rejected intent for
                // the rest of the phase and keep acting — ending the phase
                // here (the old fallback) let ONE bad candidate abort
                // entire movement phases.
                log.push_note(
                    turn,
                    &format!(
                        "generated illegal pick ({kind}) rejected: {e}; excluded for this phase"
                    ),
                );
                rejected_this_phase.push(pick);
            }
        }

        // --- Turn boundary (engine snapshots the completed turn) ---
        if state.turn_summaries.len() > prev_summaries {
            for summary in &state.turn_summaries[prev_summaries..] {
                log.push_turn_boundary(summary, &state);
            }
            prev_summaries = state.turn_summaries.len();
        }

        // Reset per-phase counter when the phase advances.
        if is_advance {
            actions_this_phase = 0;
            rejected_this_phase.clear();
        } else {
            actions_this_phase += 1;
        }
    }

    log.push_footer(&state);
    let observations_total = log.observations_logged();

    PlayResult {
        events,
        seed,
        final_state: state,
        variant_coverage,
        actions_taken,
        log,
        observations_total,
    }
}

/// Log an applied effect line plus the observations it produced (drained from
/// the state so they never accumulate across events).
fn log_event_and_observations(
    state: &mut GameState,
    log: &mut GameLog,
    seq: usize,
    turn: u8,
    phase: &str,
    actor: Player,
    text: &str,
) {
    log.push_event(seq, turn, phase, actor, text);
    let observations = std::mem::take(&mut state.observations);
    for obs in &observations {
        log.push_observation(seq, obs);
    }
}

/// Resolve the compiled board data for a scenario.
pub fn board_for_scenario(scenario: Scenario) -> BoardInfo {
    let map_data = match scenario {
        Scenario::Campaign | Scenario::Historical => campaign_map_data(),
        Scenario::FallOfKhartoum => fall_of_khartoum_map_data(),
    };
    BoardInfo::from_map_data(&map_data)
}

/// Hard ceiling on [`playthrough`] driver iterations. A full game is a few
/// thousand actions; this is a safety valve for unresolvable stalls, not a
/// realistic cap.
const MAX_DRIVER_ITERATIONS: usize = 500_000;

/// Intent equality between two effects: same variant and same semantic
/// subject/target, ignoring pre-rolled dice and cost recomputation. Lets an
/// intent rejected in one enumeration be recognised (and excluded) in a later
/// enumeration of the same phase.
fn same_intent(a: &GameEffect, b: &GameEffect) -> bool {
    use GameEffect::*;
    match (a, b) {
        (
            MoveUnit {
                unit_id: ua,
                to: ta,
                ..
            },
            MoveUnit {
                unit_id: ub,
                to: tb,
                ..
            },
        ) => ua == ub && ta == tb,
        (
            FireCombat { attack: xa, .. } | HowitzerFire { attack: xa, .. },
            FireCombat { attack: xb, .. } | HowitzerFire { attack: xb, .. },
        ) => xa.shots() == xb.shots() && xa.target_hex == xb.target_hex,
        (DeclareMelee { attack: xa, .. }, DeclareMelee { attack: xb, .. }) => {
            xa.attacker_hex == xb.attacker_hex && xa.defender_hex == xb.defender_hex
        }
        (
            ArtilleryBreachWall {
                firers: fa,
                target: ta,
                ..
            },
            ArtilleryBreachWall {
                firers: fb,
                target: tb,
                ..
            },
        ) => fa == fb && ta == tb,
        (
            AdvanceAfterCombat {
                unit_id: ua,
                to: ta,
            },
            AdvanceAfterCombat {
                unit_id: ub,
                to: tb,
            },
        ) => ua == ub && ta == tb,
        (
            Demolition {
                unit_id: ua,
                target: ta,
            },
            Demolition {
                unit_id: ub,
                target: tb,
            },
        ) => ua == ub && ta == tb,
        (
            RetreatBeforeMelee {
                unit_id: ua,
                to: ta,
            },
            RetreatBeforeMelee {
                unit_id: ub,
                to: tb,
            },
        ) => ua == ub && ta == tb,
        (DeployUnit(pa), DeployUnit(pb)) => pa.id == pb.id,
        (PlaceReinforcements(ba), PlaceReinforcements(bb)) => {
            let ids_a: Vec<_> = ba.iter().map(|p| p.id).collect();
            let ids_b: Vec<_> = bb.iter().map(|p| p.id).collect();
            ids_a == ids_b
        }
        // Dice-only or payload-free variants: same variant = same intent.
        _ => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}

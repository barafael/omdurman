//! The full-game playthrough driver. Loops: enumerate legal actions → pick →
//! apply → log, until `game_over` or the anti-stall caps are hit.
//!
//! Two agents (one per faction) play head-to-head; each side is independently
//! [`AgentStrategy::Random`] (fast, broadest raw coverage) or
//! [`AgentStrategy::LlmAdvised`] (per-turn, narrated, with its own cache). The
//! driver also drains the engine's [`Observation`](omdurman_rules::effects::Observation)s
//! and turn summaries into a [`GameLog`] that an offline observer later audits.

use omdurman_net::GameEvent;
use omdurman_rules::board::BoardInfo;
use omdurman_rules::board_data::{campaign_map_data, fall_of_khartoum_map_data};
use omdurman_rules::effects::{apply_effect, GameState, GameEffect};
use omdurman_rules::Phase;
use omdurman_types::{Player, Scenario};

use crate::actions::legal_actions;
use crate::agent::{AgentStrategy, Agents};
use crate::describe::describe_effect;
use crate::llm::{advise_turn, LlmAnnotation, LlmCache};
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
    /// LLM reasoning notes (empty when both sides are Random).
    pub llm_annotations: Vec<LlmAnnotation>,
    /// The seed used (for reproducibility).
    pub seed: u64,
    /// The final game state.
    pub final_state: GameState,
    /// Which `GameEffect` variant kinds appeared in the trace.
    pub variant_coverage: Vec<&'static str>,
    /// Total number of actions applied.
    pub actions_taken: usize,
    /// The final cache state of the Anglo-Egyptian advisor (None when Random).
    pub ae_final_cache: Option<String>,
    /// The final cache state of the Dervish advisor (None when Random).
    pub dervish_final_cache: Option<String>,
    /// The observer-ready game log.
    pub log: GameLog,
    /// Number of engine observations drained into the log.
    pub observations_total: usize,
}

/// Play a full game headlessly from setup to game-over, with two independent
/// per-faction agents.
///
/// In `LlmAdvised` mode this function is `async` (awaits the LLM per turn).
/// In `Random` mode the LLM is never called but the function is still `async`
/// for API uniformity — use `block_on` in a sync caller.
pub async fn playthrough(
    scenario: Scenario,
    seed: u64,
    cfg: PlayConfig,
    agents: Agents,
) -> PlayResult {
    // Build the game state with the compiled board attached.
    let board = board_for_scenario(scenario);
    let mut state = GameState::with_board(scenario, board);
    let mut rng = BotRng::from_seed(seed);

    let mut events: Vec<GameEvent> = vec![GameEvent::StartGame {
        assignments: Default::default(),
        scenario,
        optional_rule: None,
        rules_version: omdurman_rules::RULES_VERSION,
    }];
    let mut annotations = Vec::new();
    let mut cache_ae = LlmCache::default();
    let mut cache_dervish = LlmCache::default();
    let mut log = GameLog::new(scenario, seed, &agents);
    let mut actions_taken = 0usize;
    let mut variant_coverage: Vec<&'static str> = Vec::new();

    let mut actions_this_phase = 0usize;
    let mut prev_turn = state.current_turn.value();
    let mut plan_for: Option<Player> = None;
    let mut llm_plan: Vec<usize> = Vec::new();
    let mut prev_summaries = 0usize;

    // Defense-in-depth against a stalled Setup phase (e.g. an unresolvable
    // leader-command deadlock): hard ceiling on driver iterations so a bug can
    // never hang the caller. Well below a full-game action count.
    let mut iterations = 0usize;

    loop {
        // Termination conditions.
        if state.game_over || state.current_turn.value() > cfg.max_turns {
            break;
        }
        iterations += 1;
        if iterations > MAX_DRIVER_ITERATIONS {
            break;
        }

        let mut candidates = legal_actions(&state, &mut rng);
        if candidates.is_empty() {
            break;
        }

        // §8.2: the first-night-turn desertion roll is not optional -- force
        // it through before any other movement action (the candidate list
        // offers it, but neither a random pick nor an LLM plan is guaranteed
        // to choose a mandatory bookkeeping roll).
        if let Some(idx) = candidates
            .iter()
            .position(|e| matches!(e, GameEffect::DervishDesertion { .. }))
        {
            let effect = candidates.remove(idx);
            let turn = state.current_turn.value();
            let phase_name = state.phase.top_level_name();
            let actor = state.active_player;
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

        // --- LLM-advised plan refresh at the start of the active side's turn ---
        let active = state.active_player;
        let refresh = state.phase == Phase::Movement
            && (llm_plan.is_empty() || plan_for != Some(active) || state.current_turn.value() != prev_turn);
        if refresh {
            prev_turn = state.current_turn.value();
            plan_for = Some(active);
            if let Some((config, brief)) = agents.llm_config(active) {
                let turn = state.current_turn.value();
                let base_idx = events.len();
                let active_cache = match active {
                    Player::AngloEgyptian => &mut cache_ae,
                    Player::Dervish => &mut cache_dervish,
                };
                let (plan, notes, ok) =
                    advise_turn(config, active, brief, &state, &candidates, active_cache).await;
                if ok {
                    llm_plan = plan;
                    for (i, note) in notes.into_iter().enumerate() {
                        let text = note.text.clone();
                        log.push_reasoning(active, turn, &text);
                        annotations.push(LlmAnnotation {
                            event_idx: base_idx + i,
                            text,
                        });
                    }
                } else {
                    llm_plan.clear();
                }
            } else {
                llm_plan.clear();
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
        } else if agents.is_llm(active) {
            // Try to follow the LLM plan; validate each index against the
            // current candidate list.
            if let Some(&idx) = llm_plan.first() {
                llm_plan.remove(0);
                match candidates.get(idx).cloned() {
                    Some(effect) => effect,
                    None => {
                        // The plan was drafted against a different candidate
                        // list (an earlier plan step changed the legal
                        // surface). Fall back to random and record the drop.
                        log.push_note(
                            state.current_turn.value(),
                            &format!(
                                "plan index {idx} stale in {} ({} candidates) -- falling back to random",
                                state.phase.top_level_name(),
                                candidates.len()
                            ),
                        );
                        rng.choose(&candidates).cloned().unwrap_or(GameEffect::AdvancePhase)
                    }
                }
            } else {
                rng.choose(&candidates).cloned().unwrap_or(GameEffect::AdvancePhase)
            }
        } else {
            rng.choose(&candidates).cloned().unwrap_or(GameEffect::AdvancePhase)
        };

        // --- Apply ---
        let is_advance = matches!(pick, GameEffect::AdvancePhase);
        let kind: &'static str = (&pick).into();
        // The *acting* side: fire attacks act with the firing player (which
        // in defensive fire is the non-moving player, §6.7) and melees with
        // the attacker (§7) -- not the phase's active player.
        let actor = match &pick {
            GameEffect::FireCombat { attack, .. }
            | GameEffect::HowitzerFire { attack, .. } => attack.firing_player,
            GameEffect::MeleeCombat { attack, .. } | GameEffect::DeclareMelee { attack, .. } => {
                attack.attacker_player
            }
            GameEffect::ArtilleryBreachWall { firers, .. } => firers
                .first()
                .and_then(|id| state.find_unit(*id))
                .map(|u| u.profile.identity.owner())
                .unwrap_or(state.active_player),
            _ => state.active_player,
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
                // The generator produced an illegal candidate (can happen with
                // clone-and-try races). Skip it and try AdvancePhase as a
                // fallback to guarantee progress -- except when the rejected
                // pick was itself a mandatory arrival (a §9.112/§9.113 batch
                // or a §8.2 desertion roll): the next iteration regenerates a
                // fresh batch, and advancing here would end the phase with
                // the wave lost.
                log.push_note(
                    turn,
                    &format!("generated illegal pick ({kind}) rejected: {e}; falling back"),
                );
                let mandatory = matches!(
                    pick,
                    GameEffect::PlaceReinforcements(_)
                        | GameEffect::DervishDesertion { .. }
                        | GameEffect::DeployUnit(_)
                );
                if !is_advance && !mandatory {
                    let fallback = GameEffect::AdvancePhase;
                    if apply_effect(&mut state, &fallback).is_ok() {
                        events.push(GameEvent::Effect(fallback));
                        let k = "AdvancePhase";
                        if !variant_coverage.contains(&k) {
                            variant_coverage.push(k);
                        }
                        log_event_and_observations(
                            &mut state,
                            &mut log,
                            events.len() - 1,
                            turn,
                            phase_name,
                            actor,
                            &format!("AdvancePhase (end {})", phase_name),
                        );
                    }
                }
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
        } else {
            actions_this_phase += 1;
        }
    }

    log.push_footer(&state);
    let observations_total = log.observations_logged();

    PlayResult {
        ae_final_cache: match agents.ae {
            AgentStrategy::Random => None,
            AgentStrategy::LlmAdvised { .. } => Some(cache_ae.0),
        },
        dervish_final_cache: match agents.dervish {
            AgentStrategy::Random => None,
            AgentStrategy::LlmAdvised { .. } => Some(cache_dervish.0),
        },
        events,
        llm_annotations: annotations,
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

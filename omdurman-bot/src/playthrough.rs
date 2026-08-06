//! The full-game playthrough driver. Loops: enumerate legal actions → pick →
//! apply → log, until `game_over` or the anti-stall caps are hit. Two
//! strategies: uniform-random (fast) and LLM-advised (per-turn, narrated).

use omdurman_net::GameEvent;
use omdurman_rules::board::BoardInfo;
use omdurman_rules::board_data::{campaign_map_data, fall_of_khartoum_map_data};
use omdurman_rules::effects::{apply_effect, GameState, GameEffect};
use omdurman_rules::Phase;
use omdurman_types::{MapKind, Scenario};

use crate::actions::legal_actions;
use crate::llm::{advise_turn, LlmAnnotation, LlmCache};
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

/// Which strategy the bot uses to pick actions.
pub enum PlayStrategy {
    /// Uniform-random over `legal_actions`.
    Random,
    /// Ask the LLM once per player-turn for a plan (needs an API key).
    LlmAdvised {
        config: omdurman_net::llm::LlmConfig,
    },
}

/// The result of a full playthrough.
pub struct PlayResult {
    /// The complete event trace (natively replayable by the app's timeline).
    pub events: Vec<GameEvent>,
    /// LLM reasoning notes (empty in Random mode).
    pub llm_annotations: Vec<LlmAnnotation>,
    /// The final cache state (Some only in LlmAdvised mode).
    pub final_cache: Option<String>,
    /// The seed used (for reproducibility).
    pub seed: u64,
    /// The final game state.
    pub final_state: GameState,
    /// Which `GameEffect` variant kinds appeared in the trace.
    pub variant_coverage: Vec<&'static str>,
    /// Total number of actions applied.
    pub actions_taken: usize,
}

/// Play a full game headlessly from setup to game-over.
///
/// In `LlmAdvised` mode this function is `async` (awaits the LLM per turn).
/// In `Random` mode the LLM is never called but the function is still `async`
/// for API uniformity — use `block_on` in a sync caller.
pub async fn playthrough(
    scenario: Scenario,
    seed: u64,
    cfg: PlayConfig,
    strategy: PlayStrategy,
) -> PlayResult {
    // Build the game state with the compiled board attached.
    let board = board_for_scenario(scenario);
    let mut state = GameState::with_board(scenario, board);
    let mut rng = BotRng::from_seed(seed);

    let mut events: Vec<GameEvent> = vec![GameEvent::StartGame {
        assignments: Default::default(),
        scenario,
        optional_rule: None,
    }];
    let mut annotations = Vec::new();
    let mut cache = LlmCache::default();
    let mut actions_taken = 0usize;
    let mut variant_coverage: Vec<&'static str> = Vec::new();

    let mut actions_this_phase = 0usize;
    let mut prev_turn = state.current_turn.value();
    let mut llm_plan: Vec<usize> = Vec::new();

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

        // Reset per-phase counter on phase change (detected by tracking).
        // (We track via actions_this_phase which is reset after AdvancePhase.)

        let candidates = legal_actions(&state, &mut rng);
        if candidates.is_empty() {
            break;
        }

        // --- LLM-advised plan refresh at the start of each player-turn ---
        if let PlayStrategy::LlmAdvised { config } = &strategy {
            if state.current_turn.value() != prev_turn || llm_plan.is_empty() {
                if state.phase == Phase::Movement {
                    prev_turn = state.current_turn.value();
                    let base_idx = events.len();
                    let (plan, notes, ok) = advise_turn(config, &state, &candidates, &mut cache).await;
                    if ok {
                        llm_plan = plan;
                        for (i, note) in notes.into_iter().enumerate() {
                            annotations.push(LlmAnnotation {
                                event_idx: base_idx + i,
                                text: note.text,
                            });
                        }
                    } else {
                        llm_plan.clear();
                    }
                }
            }
        }

        // --- Pick an action ---
        let pick = if actions_this_phase >= cfg.max_actions_per_phase {
            // Anti-stall: force phase advance.
            candidates
                .iter()
                .find(|e| matches!(e, GameEffect::AdvancePhase))
                .cloned()
                .unwrap_or(GameEffect::AdvancePhase)
        } else if let PlayStrategy::LlmAdvised { .. } = strategy {
            // Try to follow the LLM plan; validate each index against the
            // current candidate list.
            if let Some(&idx) = llm_plan.first() {
                llm_plan.remove(0);
                candidates.get(idx).cloned().unwrap_or_else(|| {
                    rng.choose(&candidates).cloned().unwrap_or(GameEffect::AdvancePhase)
                })
            } else {
                rng.choose(&candidates).cloned().unwrap_or(GameEffect::AdvancePhase)
            }
        } else {
            rng.choose(&candidates).cloned().unwrap_or(GameEffect::AdvancePhase)
        };

        // --- Apply ---
        let is_advance = matches!(pick, GameEffect::AdvancePhase);
        let kind = kind_str(&pick);

        match apply_effect(&mut state, &pick) {
            Ok(()) => {
                events.push(GameEvent::Effect(pick));
                if !variant_coverage.contains(&kind) {
                    variant_coverage.push(kind);
                }
                actions_taken += 1;
            }
            Err(_) => {
                // The generator produced an illegal candidate (can happen with
                // clone-and-try races). Skip it and try AdvancePhase as a
                // fallback to guarantee progress.
                if !is_advance {
                    let fallback = GameEffect::AdvancePhase;
                    if apply_effect(&mut state, &fallback).is_ok() {
                        events.push(GameEvent::Effect(fallback));
                        let k = "AdvancePhase";
                        if !variant_coverage.contains(&k) {
                            variant_coverage.push(k);
                        }
                    }
                }
            }
        }

        // Reset per-phase counter when the phase advances.
        if is_advance {
            actions_this_phase = 0;
        } else {
            actions_this_phase += 1;
        }
    }

    PlayResult {
        final_cache: match strategy {
            PlayStrategy::Random => None,
            PlayStrategy::LlmAdvised { .. } => Some(cache.0),
        },
        events,
        llm_annotations: annotations,
        seed,
        final_state: state,
        variant_coverage,
        actions_taken,
    }
}

/// Resolve the compiled board data for a scenario.
pub fn board_for_scenario(scenario: Scenario) -> BoardInfo {
    let map_data = match scenario {
        Scenario::Campaign | Scenario::Historical => campaign_map_data(),
        Scenario::FallOfKhartoum => fall_of_khartoum_map_data(),
    };
    let _ = MapKind::Campaign; // suppress unused import
    BoardInfo::from_map_data(&map_data)
}

/// Hard ceiling on [`playthrough`] driver iterations. A full game is a few
/// thousand actions; this is a safety valve for unresolvable stalls, not a
/// realistic cap.
const MAX_DRIVER_ITERATIONS: usize = 500_000;

/// The `IntoStaticStr` name of a `GameEffect` variant.
fn kind_str(e: &GameEffect) -> &'static str {
    let dbg = format!("{e:?}");
    // Take the variant name before any '{' or '('. Tuple variants print as
    // `Variant(...)`, struct variants as `Variant { field: ... }`.
    let name = dbg.split(['(', '{']).next().unwrap_or(&dbg).trim();
    // Leak the string to get a 'static — acceptable since there are only ~27
    // distinct names and they repeat.
    match name {
        "AdvancePhase" => "AdvancePhase",
        "MoveUnit" => "MoveUnit",
        "FireCombat" => "FireCombat",
        "HowitzerFire" => "HowitzerFire",
        "MeleeCombat" => "MeleeCombat",
        "DeclareMelee" => "DeclareMelee",
        "ResolveMelee" => "ResolveMelee",
        "RetreatBeforeMelee" => "RetreatBeforeMelee",
        "AdvanceAfterCombat" => "AdvanceAfterCombat",
        "RecoverUnit" => "RecoverUnit",
        "ConstructZariba" => "ConstructZariba",
        "Demolition" => "Demolition",
        "PlaceReinforcements" => "PlaceReinforcements",
        "DervishDesertion" => "DervishDesertion",
        "FriendliesTransport" => "FriendliesTransport",
        "RiverMine" => "RiverMine",
        "SinkChain" => "SinkChain",
        "DeployUnit" => "DeployUnit",
        "RemoveDeployedUnit" => "RemoveDeployedUnit",
        "PlaceMine" => "PlaceMine",
        "PlaceChain" => "PlaceChain",
        "PlaceZariba" => "PlaceZariba",
        "ConfirmSetupReady" => "ConfirmSetupReady",
        "ResolveDemolition" => "ResolveDemolition",
        "DriftGunboat" => "DriftGunboat",
        "ArtilleryBreachWall" => "ArtilleryBreachWall",
        _ => "Other",
    }
}

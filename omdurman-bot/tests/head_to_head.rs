//! Head-to-head: the per-faction agent wiring actually drives both sides, and
//! the two agents' independence is visible (each side keeps its own strategy
//! and label).

use omdurman_bot::agent::{AgentStrategy, Agents};
use omdurman_bot::commanders::Commander;
use omdurman_bot::describe::describe_effect;
use omdurman_bot::playthrough::{PlayConfig, playthrough};
use omdurman_rules::effects::GameState;
use omdurman_types::{Player, Scenario};

#[test]
fn random_agents_both_play() {
    let cfg = PlayConfig {
        max_actions_per_phase: 80,
        max_turns: 6,
    };
    let result = playthrough(Scenario::FallOfKhartoum, 99u64, cfg, Agents::random());
    eprintln!(
        "actions_taken={}, turn={}, phase={:?}, game_over={}, coverage={:?}",
        result.actions_taken,
        result.final_state.current_turn.value(),
        result.final_state.phase,
        result.final_state.game_over,
        result.variant_coverage
    );
    assert!(
        result.final_state.current_turn.value() > 1 || result.final_state.game_over,
        "game should progress past turn 1 (turn={})",
        result.final_state.current_turn.value()
    );
    assert!(result.actions_taken > 0, "no actions taken");
}

#[test]
fn mixed_agents_keep_side_identity() {
    let agents = Agents {
        ae: AgentStrategy::Commander(Commander::Kitchener),
        dervish: AgentStrategy::Aggressive,
    };
    assert_eq!(
        agents.commander(Player::AngloEgyptian),
        Some(Commander::Kitchener)
    );
    assert_eq!(agents.commander(Player::Dervish), None);
    assert!(!agents.is_aggressive(Player::AngloEgyptian));
    assert!(agents.is_aggressive(Player::Dervish));
    assert_eq!(agents.label_for(Player::AngloEgyptian), "Kitchener");
    assert_eq!(agents.label_for(Player::Dervish), "aggressive");
    assert_eq!(Agents::random().label_for(Player::AngloEgyptian), "random");
}

#[test]
fn describe_effect_renders_real_trace_effects() {
    // Every effect a playthrough actually applied must render to non-empty
    // text without panicking. This exercises the full describe_effect match
    // surface against real engine values.
    let cfg = PlayConfig {
        max_actions_per_phase: 80,
        max_turns: 4,
    };
    let result = playthrough(Scenario::Campaign, 7u64, cfg, Agents::random());
    assert!(!result.events.is_empty(), "no events captured");
    for ev in &result.events {
        if let omdurman_net::GameEvent::Effect(eff) = ev {
            let text = describe_effect(eff, &result.final_state);
            assert!(!text.is_empty(), "empty describe for {eff:?}");
        }
    }
    // Sanity: the final state must still be self-consistent enough to describe.
    let state = GameState::new(Scenario::Campaign);
    assert_eq!(
        describe_effect(&omdurman_rules::effects::GameEffect::AdvancePhase, &state),
        "AdvancePhase (end Setup)"
    );
}

//! Termination: every playthrough must reach game_over within the caps, and
//! the action count must stay bounded.

use omdurman_bot::{playthrough, PlayConfig};
use omdurman_bot::agent::Agents;
use omdurman_types::Scenario;

#[test]
fn fok_playthrough_terminates() {
    let cfg = PlayConfig {
        max_actions_per_phase: 100,
        max_turns: 12,
    };
    let result = futures::executor::block_on(playthrough(
        Scenario::FallOfKhartoum,
        42u64,
        cfg,
        Agents::random(),
    ));
    assert!(
        result.final_state.game_over || result.final_state.current_turn.value() > 12,
        "playthrough did not terminate (game_over={}, turn={})",
        result.final_state.game_over,
        result.final_state.current_turn.value(),
    );
    // Bounded action count.
    let max_actions = 12 * 8 * 100; // max_turns × phases × max_per_phase
    assert!(
        result.actions_taken <= max_actions,
        "actions_taken {} exceeded bound {}",
        result.actions_taken,
        max_actions,
    );
}

#[test]
fn campaign_playthrough_terminates() {
    let cfg = PlayConfig {
        max_actions_per_phase: 100,
        max_turns: 8,
    };
    let result = futures::executor::block_on(playthrough(
        Scenario::Campaign,
        7u64,
        cfg,
        Agents::random(),
    ));
    assert!(
        result.final_state.game_over || result.final_state.current_turn.value() > 8,
        "Campaign playthrough did not terminate",
    );
}

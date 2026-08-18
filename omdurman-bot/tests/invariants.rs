//! proptest: random playthroughs must hold engine invariants after EVERY
//! applied effect (not just the final state), and `game_over` must be
//! monotonic across the trace. Failures are automatically shrunk to minimal
//! reproducible seeds and persisted in `proptest-regressions/`.
//!
//! The trace is replayed against a freshly-built state (the same "late-joiner
//! replay" path the net layer uses), so this also verifies the event log is
//! faithfully reconstructible.

use proptest::prelude::*;
use omdurman_bot::agent::Agents;
use omdurman_bot::invariants::{check_all_with_tribal, game_over_monotonic};
use omdurman_bot::{board_for_scenario, playthrough, PlayConfig};
use omdurman_net::GameEvent;
use omdurman_rules::board::BoardInfo;
use omdurman_rules::effects::{apply_effect, GameState};
use omdurman_types::Scenario;

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 16,
        ..proptest::test_runner::Config::default()
    })]

    #[test]
    fn fok_random_playthrough_holds_invariants(seed in 0u64..10_000) {
        let cfg = PlayConfig {
            max_actions_per_phase: 80,
            max_turns: 10,
        };
        let result = futures::executor::block_on(playthrough(
            Scenario::FallOfKhartoum,
            seed,
            cfg,
            Agents::random(),
        ));
        prop_assert!(
            result.actions_taken > 0,
            "playthrough with seed {seed} took no actions"
        );

        // Replay the event log from scratch and assert every per-state
        // invariant after each applied effect, so transient violations
        // (the kind that self-correct) are caught too.
        let mut state: Option<GameState> = None;
        let mut prev_game_over = false;
        for (i, ev) in result.events.iter().enumerate() {
            match ev {
                GameEvent::StartGame { scenario, .. } => {
                    let board: BoardInfo = board_for_scenario(*scenario);
                    state = Some(GameState::with_board(*scenario, board));
                }
                GameEvent::Effect(eff) => {
                    let s = state.as_mut().expect("Effect before StartGame");
                    // The playthrough only records effects it applied
                    // successfully, so replay must agree.
                    apply_effect(s, eff)
                        .map_err(|err| proptest::test_runner::TestCaseError::fail(format!(
                            "event {i}: replay rejected {eff:?}: {err}"
                        )))?;
                    check_all_with_tribal(s)
                        .map_err(proptest::test_runner::TestCaseError::fail)?;
                    game_over_monotonic(prev_game_over, s.game_over)
                        .map_err(proptest::test_runner::TestCaseError::fail)?;
                    prev_game_over = s.game_over;
                }
                _ => {}
            }
        }
    }
}

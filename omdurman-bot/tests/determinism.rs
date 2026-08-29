//! Determinism: the same seed must produce byte-identical event traces.

use omdurman_bot::agent::Agents;
use omdurman_bot::{PlayConfig, playthrough};
use omdurman_types::Scenario;

#[test]
fn fok_random_playthrough_is_deterministic() {
    let cfg = PlayConfig {
        keep_out: None,
        max_actions_per_phase: 50,
        max_turns: 5,
    };
    let seed = 12345u64;
    let agents = Agents::random();
    let a = futures::executor::block_on(playthrough(
        Scenario::FallOfKhartoum,
        seed,
        cfg.clone(),
        agents.clone(),
    ));
    let b = futures::executor::block_on(playthrough(
        Scenario::FallOfKhartoum,
        seed,
        cfg,
        agents.clone(),
    ));
    assert_eq!(a.events.len(), b.events.len(), "event count diverged");
    for (i, (ea, eb)) in a.events.iter().zip(b.events.iter()).enumerate() {
        assert_eq!(format!("{ea:?}"), format!("{eb:?}"), "event {i} diverged");
    }
}

#[test]
fn different_seeds_produce_different_traces() {
    let cfg = PlayConfig {
        keep_out: None,
        max_actions_per_phase: 50,
        max_turns: 5,
    };
    let a = futures::executor::block_on(playthrough(
        Scenario::FallOfKhartoum,
        1u64,
        cfg.clone(),
        Agents::random(),
    ));
    let b = futures::executor::block_on(playthrough(
        Scenario::FallOfKhartoum,
        999u64,
        cfg,
        Agents::random(),
    ));
    // They should almost certainly differ (different random choices).
    let a_dbg: Vec<String> = a.events.iter().map(|e| format!("{e:?}")).collect();
    let b_dbg: Vec<String> = b.events.iter().map(|e| format!("{e:?}")).collect();
    assert_ne!(a_dbg, b_dbg, "different seeds produced identical traces");
}

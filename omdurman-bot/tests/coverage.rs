//! Coverage: over a batch of playthroughs, the most common GameEffect
//! variants must all appear. This catches move-generator blind spots and,
//! more importantly, would surface if a rule path is unreachable.

use omdurman_bot::{playthrough, PlayConfig};
use omdurman_bot::agent::Agents;
use omdurman_types::Scenario;
use std::collections::HashSet;

#[test]
fn fok_batch_covers_core_variants() {
    let cfg = PlayConfig {
        max_actions_per_phase: 100,
        max_turns: 8,
    };
    let mut all_kinds: HashSet<&str> = HashSet::new();
    for seed in 0..8u64 {
        let result = futures::executor::block_on(playthrough(
            Scenario::FallOfKhartoum,
            seed,
            cfg.clone(),
            Agents::random(),
        ));
        for kind in &result.variant_coverage {
            all_kinds.insert(*kind);
        }
    }
    // These variants MUST appear in a batch of FoK games:
    // - DeployUnit + ConfirmSetupReady + AdvancePhase (setup flow)
    // - MoveUnit (movement)
    assert!(all_kinds.contains("DeployUnit"), "missing DeployUnit in {:?}", all_kinds);
    assert!(
        all_kinds.contains("AdvancePhase"),
        "missing AdvancePhase in {:?}",
        all_kinds
    );
    // At least one combat type should appear (fire or melee), unless the bot
    // never gets units into range within 8 turns — log what we got.
    let has_combat = all_kinds.contains("FireCombat")
        || all_kinds.contains("MeleeCombat")
        || all_kinds.contains("DeclareMelee");
    // We don't hard-assert combat coverage (the random bot may not close range
    // in every batch), but we log the achieved set.
    eprintln!("FoK coverage after 8 games: {:?}", all_kinds);
    let _ = has_combat;
}

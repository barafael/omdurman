//! Regression: the AI must not walk a unit back and forth between hexes in
//! one movement phase (seen in play: (41,18)<->(41,19) eight times in a
//! row, a fifth of all moves in a Campaign game). Every scenario, both sides
//! AI, through the app's decision path ([`omdurman_bot::arena`]); a
//! "revisit" is a move onto a hex the unit already occupied this phase
//! (`omdurman_bot::move_memory`).

use omdurman_bot::arena::{Version, play_until};
use omdurman_types::Scenario;

fn revisits(scenario: Scenario, seed: u64, ae: Version, dervish: Version, turns: u8) -> usize {
    let r = play_until(scenario, seed, ae, dervish, turns);
    eprintln!("{}", r.line());
    r.revisits
}

#[test]
fn the_ai_never_steps_back_onto_ground_it_covered_this_phase() {
    for (scenario, turns) in [
        (Scenario::FallOfKhartoum, 8),
        (Scenario::Historical, 4),
        (Scenario::Campaign, 6),
    ] {
        for seed in [1, 2] {
            assert_eq!(
                revisits(scenario, seed, Version::Current, Version::Current, turns),
                0,
                "{scenario:?} seed {seed}: a unit stepped back onto a hex it had left"
            );
        }
    }
}

/// The check has teeth: the frozen October commanders (no movement memory)
/// do shuffle.
#[test]
fn the_baseline_commanders_do_shuffle() {
    let total: usize = [1, 2]
        .iter()
        .map(|&seed| {
            revisits(
                Scenario::Campaign,
                seed,
                Version::Baseline,
                Version::Baseline,
                6,
            )
        })
        .sum();
    assert!(total > 0, "expected the baseline to revisit hexes");
}

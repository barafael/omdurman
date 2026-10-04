//! Log format: a playthrough renders a log the offline audit can consume —
//! header, per-event lines with seq/turn/phase/actor, observations, turn
//! boundaries, footer — and the rendered line counts agree with the log's own
//! counters through a real game.

use omdurman_bot::agent::Agents;
use omdurman_bot::playthrough::{PlayConfig, playthrough};
use omdurman_types::Scenario;

#[test]
fn log_has_header_and_footer() {
    let cfg = PlayConfig {
        max_actions_per_phase: 80,
        max_turns: 4,
    };
    let result = playthrough(Scenario::Campaign, 4242u64, cfg, Agents::random());
    let text = result.log.render();
    assert!(text.contains("GAME LOG"), "missing header");
    assert!(text.contains("scenario"), "missing scenario in header");
    // Header renders the seed in hex after a colon, e.g. `seed:  0x1092`.
    assert!(
        text.contains("seed:") && text.contains("0x1092"),
        "missing seed in header: {text}"
    );
    assert!(
        text.contains("ae=") && text.contains("dervish="),
        "missing agent labels in header"
    );
    assert!(text.contains("GAME OVER"), "missing footer");
    assert!(text.contains("result:"), "missing game result line");
}

#[test]
fn log_lines_have_seq_turn_phase_actor() {
    let cfg = PlayConfig {
        max_actions_per_phase: 80,
        max_turns: 4,
    };
    let result = playthrough(Scenario::Campaign, 7u64, cfg, Agents::random());
    let text = result.log.render();
    let event_lines: Vec<&str> = text
        .lines()
        .filter(|l| l.starts_with('['))
        // `[note, ...]` lines are driver annotations, not events.
        .filter(|l| !l.starts_with("[note,"))
        .collect();
    assert!(!event_lines.is_empty(), "no event lines in log");
    for line in &event_lines {
        // [<seq>] T<turn> <phase> <actor>  <text>
        let after_bracket = line.strip_prefix('[').and_then(|s| s.split(']').next());
        assert!(
            after_bracket.is_some_and(|s| s.parse::<usize>().is_ok()),
            "bad seq in {line}"
        );
        assert!(line.contains("] T"), "missing turn marker in {line}");
    }
}

#[test]
fn observations_drained_and_tagged() {
    let cfg = PlayConfig {
        max_actions_per_phase: 80,
        max_turns: 6,
    };
    let result = playthrough(Scenario::FallOfKhartoum, 123u64, cfg, Agents::random());
    let text = result.log.render();
    // The rendered log must contain exactly one "→" observation line per
    // drained observation. Event lines also use "→" for moves/advances, so
    // count only the indented observation lines (`      → … [event N]`).
    let rendered = text.lines().filter(|l| l.starts_with("      → ")).count();
    assert_eq!(
        rendered, result.observations_total,
        "observation lines mismatch"
    );
    if result.observations_total > 0 {
        assert!(
            text.contains("\n      →"),
            "observations not rendered as indented lines"
        );
    }
}

#[test]
fn turn_boundaries_are_emitted() {
    let cfg = PlayConfig {
        max_actions_per_phase: 80,
        max_turns: 3,
    };
    let result = playthrough(Scenario::Campaign, 99u64, cfg, Agents::random());
    let text = result.log.render();
    let boundaries: Vec<&str> = text
        .lines()
        .filter(|l| l.starts_with("=== Turn ") && l.contains("complete"))
        .collect();
    assert!(!boundaries.is_empty(), "no turn boundaries in {text}");
}

#[test]
fn rendered_line_counts_match_the_log_counters() {
    let cfg = PlayConfig {
        max_actions_per_phase: 80,
        max_turns: 3,
    };
    let result = playthrough(Scenario::Campaign, 5u64, cfg, Agents::random());
    let text = result.log.render();
    let boundaries = text
        .lines()
        .filter(|l| l.starts_with("=== Turn ") && l.contains("complete"))
        .count();
    let events = text
        .lines()
        .filter(|l| l.starts_with('[') && l.contains("] T"))
        .count();
    assert_eq!(boundaries, result.log.turn_boundaries());
    assert_eq!(events, result.log.events_logged());
}

#[test]
fn log_is_deterministic_across_runs() {
    let cfg = PlayConfig {
        max_actions_per_phase: 50,
        max_turns: 3,
    };
    let a = playthrough(Scenario::Campaign, 77u64, cfg.clone(), Agents::random());
    let b = playthrough(Scenario::Campaign, 77u64, cfg, Agents::random());
    assert_eq!(a.log.render(), b.log.render(), "log diverged for same seed");
}

//! Observer: the offline rules audit must parse the tagged FINDINGS protocol,
//! chunk by turn, aggregate across chunks, and degrade gracefully when the
//! API key is missing.

use omdurman_bot::observer::{chunk_log, count_events, review, Completion, Severity};
use omdurman_net::llm::{LlmConfig, LlmError};

/// Canned completion: returns a fixed response with one finding and a summary,
/// regardless of input.
struct Canned(&'static str);

impl Completion for Canned {
    fn complete<'a>(
        &'a self,
        _config: &'a LlmConfig,
        _system: &'a str,
        _user: &'a str,
        _max_tokens: u32,
    ) -> futures::future::BoxFuture<'a, Result<String, LlmError>> {
        Box::pin(async move { Ok(self.0.to_string()) })
    }
}

fn config_with_key() -> LlmConfig {
    LlmConfig {
        api_key: Some("test-key".to_string()),
        base_url: "http://localhost/v1".to_string(),
        model: "test".to_string(),
    }
}

const FINDING_RESPONSE: &str = "\
CACHE:
checked movement MP; all within allowance.

FINDINGS:
- warning|12|§5.24|gunboat may have exceeded upstream allowance
- error|34|§6.24|fire modifier not applied to CRT roll

SUMMARY:
Game is mostly legal; two suspicious events noted.";

#[test]
fn review_parses_tagged_findings() {
    let log = "[0] T1 Movement AngloEgyptian  MoveUnit\n[12] T1 Movement Dervish  MoveUnit\n[34] T2 Fire AngloEgyptian  FireCombat\n";
    let report = futures::executor::block_on(review(
        log,
        &config_with_key(),
        &Canned(FINDING_RESPONSE),
        "crib sheet",
    ));
    assert_eq!(report.findings.len(), 2);
    assert_eq!(report.findings[0].severity, Severity::Warning);
    assert_eq!(report.findings[0].seq, 12);
    assert_eq!(report.findings[0].section.as_deref(), Some("5.24"));
    assert_eq!(report.findings[1].severity, Severity::Error);
    assert!(report.summary.contains("mostly legal"));
    assert_eq!(report.events_audited, 3);
}

#[test]
fn review_aggregates_across_chunks() {
    // Two turn boundaries -> three chunks; the canned response is identical
    // for each, so its findings dedupe to one copy each -- distinct findings
    // from distinct chunks would all survive.
    let mut log = String::new();
    for t in 1..=3 {
        log.push_str(&format!("[{}] T{t} Movement Dervish  MoveUnit\n", t * 10));
        if t < 3 {
            log.push_str(&format!(
                "=== Turn {t} complete (6:00 AM, Day) — 0 fire, 0 melee, 0 eliminations; VP AE 0 / Dervish 0 ===\n"
            ));
        }
    }
    let chunks = chunk_log(&log);
    assert_eq!(chunks.len(), 3, "expected 3 chunks, got {}", chunks.len());
    let report = futures::executor::block_on(review(
        &log,
        &config_with_key(),
        &Canned(FINDING_RESPONSE),
        "",
    ));
    assert_eq!(
        report.findings.len(),
        2,
        "re-flagged identical findings dedupe; distinct ones accumulate"
    );
    assert_eq!(report.turns_audited, 2);
    assert_eq!(report.events_audited, 3);
    assert!(report.summary.contains("mostly legal"));
}

#[test]
fn review_skips_without_api_key() {
    let log = "[0] T1 Movement Dervish  MoveUnit\n";
    let cfg = LlmConfig {
        api_key: None,
        base_url: "http://localhost/v1".to_string(),
        model: "test".to_string(),
    };
    let report = futures::executor::block_on(review(&log, &cfg, &Canned(""), ""));
    assert!(report.findings.is_empty(), "no findings expected without a key");
    assert!(report.summary.contains("No API key"));
    assert_eq!(report.events_audited, 1);
}

#[test]
fn malformed_lines_are_skipped() {
    let log = "[0] T1 Movement Dervish  MoveUnit\n[1] T1 Movement Dervish  MoveUnit\n";
    let bad_response = "FINDINGS:\n- warning|3|§5.24|x\n- garbage line\n- error|7|§6.24|ok\n";
    let report = futures::executor::block_on(review(
        log,
        &config_with_key(),
        &Canned(bad_response),
        "",
    ));
    assert_eq!(report.findings.len(), 2, "should keep the two well-formed lines");
    assert_eq!(report.findings[0].seq, 3);
    assert_eq!(report.findings[1].section.as_deref(), Some("6.24"));
}

#[test]
fn count_events_counts_seq_lines() {
    let log = "[0] T1 Movement Dervish  MoveUnit\n[1] T1 Movement Dervish  MoveUnit\n    → observation [event 1]\n=== Turn 1 complete ===\n";
    assert_eq!(count_events(log), 2);
}

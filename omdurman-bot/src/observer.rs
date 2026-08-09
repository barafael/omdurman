//! Offline review pass over a game log.
//!
//! A full game is too large for a single LLM prompt, so the observer feeds the
//! log to the model **turn by turn**, carrying a running notes/findings cache
//! between chunks (the same `CACHE`/tagged-response pattern as the players'
//! advisor). The result is a [`ObserverReport`] of §-cited [`Finding`]s plus a
//! closing summary.
//!
//! Findings are **advisory**: they surface suspicions for a human to triage,
//! layered on top of the deterministic hard invariants (`invariants::check_all`)
//! and the engine's own `can_*` validation. Automated gating stays on those.

use futures::future::BoxFuture;
use omdurman_net::llm::{LlmConfig, LlmError};

use crate::llm::LlmCache;

/// Severity of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Severity {
    Critical,
    Error,
    Warning,
    Info,
}

impl Severity {
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "critical" => Some(Severity::Critical),
            "error" => Some(Severity::Error),
            "warning" => Some(Severity::Warning),
            "info" => Some(Severity::Info),
            _ => None,
        }
    }

    /// Lower-case label used in the tagged response protocol and report.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

/// A single rule violation (or suspicion) found in the log.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Finding {
    pub severity: Severity,
    /// Sequence number of the log event the finding refers to.
    pub seq: usize,
    /// Rulebook section cited, without the `§` prefix.
    pub section: Option<String>,
    pub explanation: String,
}

/// The aggregated result of an observer pass.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ObserverReport {
    pub findings: Vec<Finding>,
    /// The LLM's closing assessment (from the last `SUMMARY:` section).
    pub summary: String,
    pub turns_audited: usize,
    pub events_audited: usize,
}

impl std::fmt::Display for ObserverReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# Rules audit report")?;
        writeln!(f, "- turns audited: {}", self.turns_audited)?;
        writeln!(f, "- events audited: {}", self.events_audited)?;
        writeln!(f, "- findings: {}\n", self.findings.len())?;
        for (i, finding) in self.findings.iter().enumerate() {
            let section = finding
                .section
                .as_deref()
                .map(|s| format!("§{s}"))
                .unwrap_or_else(|| "no section".to_string());
            writeln!(
                f,
                "{i}. [{:>8}] seq {} ({section}): {}",
                finding.severity.label(),
                finding.seq,
                finding.explanation
            )?;
        }
        if !self.summary.is_empty() {
            writeln!(f, "\n## Observer summary\n{}", self.summary)?;
        }
        Ok(())
    }
}

/// Abstraction over the completion call so tests can run the observer without
/// a network. Mirrors `omdurman_net::llm::request_completion`.
pub trait Completion: Send + Sync {
    fn complete<'a>(
        &'a self,
        config: &'a LlmConfig,
        system: &'a str,
        user: &'a str,
        max_tokens: u32,
    ) -> BoxFuture<'a, Result<String, LlmError>>;
}

/// Real transport: delegates to `omdurman_net::llm::request_completion`.
pub struct ReqwestCompletion;

impl Completion for ReqwestCompletion {
    fn complete<'a>(
        &'a self,
        config: &'a LlmConfig,
        system: &'a str,
        user: &'a str,
        max_tokens: u32,
    ) -> BoxFuture<'a, Result<String, LlmError>> {
        Box::pin(omdurman_net::llm::request_completion(config, system, user, max_tokens))
    }
}

const OBSERVER_SYSTEM_PROMPT: &str = "\
You are an independent rules auditor for the board game 'Remember Gordon!' \
(The Battle of Omdurman, Phoenix Enterprises 1982). You review a \
machine-generated game log and flag every place where the log contradicts the \
rulebook. Use the crib sheet for the rule numbers; where the log alone is \
ambiguous, raise a Warning and say what is uncertain. Respond exactly in the \
tagged format:\n\
CACHE:\n<your working notes; carry open questions and a running tally>\n\n\
FINDINGS:\n- severity|seq|§section|explanation\n\n\
SUMMARY:\n<one-paragraph closing assessment>";

/// Run the offline review pass over `log`.
///
/// `crib` is the rules crib sheet text; `config` supplies the API key/model.
/// Returns an `ObserverReport`. Degrades gracefully: a failed or malformed
/// chunk response keeps the previous cache and continues.
pub async fn review(
    log: &str,
    config: &LlmConfig,
    completion: &impl Completion,
    crib: &str,
) -> ObserverReport {
    if !config.has_key() {
        return ObserverReport {
            findings: Vec::new(),
            summary: "No API key configured — review skipped.".to_string(),
            turns_audited: 0,
            events_audited: count_events(log),
        };
    }

    let chunks = chunk_log(log);
    let mut cache = String::new();
    let mut findings: Vec<Finding> = Vec::new();
    let mut summary = String::new();
    let mut turns = 0usize;

    for (i, chunk) in chunks.iter().enumerate() {
        let user = build_review_prompt(crib, &cache, i, chunks.len(), chunk, i == 0);
        let response = match completion
            .complete(config, OBSERVER_SYSTEM_PROMPT, &user, 2000)
            .await
        {
            Ok(text) => text,
            Err(_) => continue, // degraded chunk: keep previous cache
        };
        let parsed = parse_review_response(&response);
        if !parsed.cache.is_empty() {
            let mut capped = LlmCache(parsed.cache);
            capped.truncate_to_cap();
            cache = capped.0;
        }
        findings.extend(parsed.findings);
        if !parsed.summary.is_empty() {
            summary = parsed.summary;
        }
        if chunk_has_turn_boundary(chunk) {
            turns += 1;
        }
    }

    ObserverReport {
        findings,
        summary,
        turns_audited: turns,
        events_audited: count_events(log),
    }
}

/// Split the log into turn-sized chunks at the `=== Turn N complete ===`
/// markers. Everything up to the first marker is chunk 0 (setup + turn 1).
pub fn chunk_log(log: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    for line in log.lines() {
        if line.starts_with("=== Turn ") && line.contains("complete") && !current.trim().is_empty()
        {
            chunks.push(std::mem::take(&mut current));
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Whether a chunk contains a turn-boundary marker.
fn chunk_has_turn_boundary(chunk: &str) -> bool {
    chunk.lines().any(|l| l.starts_with("=== Turn ") && l.contains("complete"))
}

/// Count `[seq]` event lines in the log.
pub fn count_events(log: &str) -> usize {
    log.lines()
        .filter(|l| l.starts_with('[') && l.contains("] T"))
        .count()
}

/// Build the user prompt for one chunk. The crib sheet is attached to the
/// first chunk only.
fn build_review_prompt(
    crib: &str,
    cache: &str,
    idx: usize,
    total: usize,
    chunk: &str,
    first: bool,
) -> String {
    let mut user = String::new();
    user.push_str(&format!("=== REVIEW CHUNK {}/{} ===\n", idx + 1, total));
    if first {
        user.push_str("\n=== RULES CRIB SHEET ===\n");
        user.push_str(crib);
        user.push_str("\n=== END CRIB SHEET ===\n");
    }
    user.push_str("\n=== RUNNING CONTEXT FROM PREVIOUS CHUNKS ===\n");
    if cache.is_empty() {
        user.push_str("(none)\n");
    } else {
        user.push_str(cache);
    }
    user.push_str("\n=== END RUNNING CONTEXT ===\n\n");
    user.push_str("=== LOG TURN ===\n");
    user.push_str(chunk);
    user.push_str("\n=== END LOG ===\n");
    user
}

struct ParsedReview {
    cache: String,
    findings: Vec<Finding>,
    summary: String,
}

/// Parse the tagged response:
/// ```text
/// CACHE:
/// <notes>
/// FINDINGS:
/// - severity|seq|§section|explanation
/// SUMMARY:
/// <assessment>
/// ```
fn parse_review_response(text: &str) -> ParsedReview {
    let mut cache = String::new();
    let mut findings = Vec::new();
    let mut summary = String::new();
    let mut section = "";
    for line in text.lines() {
        let trimmed = line.trim();
        match trimmed {
            "CACHE:" => section = "cache",
            "FINDINGS:" => section = "findings",
            "SUMMARY:" => section = "summary",
            _ => match section {
                "cache" => {
                    if !cache.is_empty() {
                        cache.push('\n');
                    }
                    cache.push_str(line);
                }
                "findings" => {
                    if let Some(f) = parse_finding(trimmed) {
                        findings.push(f);
                    }
                }
                "summary" => {
                    if !summary.is_empty() {
                        summary.push('\n');
                    }
                    summary.push_str(line);
                }
                _ => {}
            },
        }
    }
    ParsedReview { cache, findings, summary }
}

/// Parse one `- severity|seq|§section|explanation` line. Returns `None` when
/// the line does not match the protocol (tolerated).
fn parse_finding(line: &str) -> Option<Finding> {
    let mut l = line.trim();
    if let Some(rest) = l.strip_prefix('-') {
        l = rest.trim();
    }
    let parts: Vec<&str> = l.splitn(4, '|').map(str::trim).collect();
    if parts.len() != 4 {
        return None;
    }
    let severity = Severity::parse(parts[0])?;
    let seq = parts[1].parse::<usize>().ok()?;
    let section = parts[2].trim_start_matches('§');
    let section = if section.is_empty() {
        None
    } else {
        Some(section.to_string())
    };
    Some(Finding {
        severity,
        seq,
        section,
        explanation: parts[3].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canned completion: returns a fixed response with one finding and a
    /// summary, regardless of input.
    struct Canned(&'static str);

    impl Completion for Canned {
        fn complete<'a>(
            &'a self,
            _config: &'a LlmConfig,
            _system: &'a str,
            _user: &'a str,
            _max_tokens: u32,
        ) -> BoxFuture<'a, Result<String, LlmError>> {
            let out = self.0.to_string();
            Box::pin(async move { Ok(out) })
        }
    }

    #[test]
    fn parses_tagged_findings() {
        let parsed = parse_review_response(
            "CACHE:\nkeep track\nFINDINGS:\n- warning|12|§5.11|move cost\n- error|4|§6.22|wrong row\nSUMMARY:\ndone",
        );
        assert_eq!(parsed.cache, "keep track");
        assert_eq!(parsed.summary, "done");
        assert_eq!(parsed.findings.len(), 2);
        assert_eq!(parsed.findings[0].severity, Severity::Warning);
        assert_eq!(parsed.findings[0].seq, 12);
        assert_eq!(parsed.findings[0].section.as_deref(), Some("5.11"));
        assert_eq!(parsed.findings[0].explanation, "move cost");
        assert_eq!(parsed.findings[1].severity, Severity::Error);
        assert_eq!(parsed.findings[1].section.as_deref(), Some("6.22"));
    }

    #[test]
    fn tolerates_malformed_finding_lines() {
        let parsed = parse_review_response(
            "FINDINGS:\n- warning|12|§5.11|ok\n- nope\n- error|bad|§1|x\n- info|7||no section",
        );
        assert_eq!(parsed.findings.len(), 2);
        assert_eq!(parsed.findings[0].seq, 12);
        assert_eq!(parsed.findings[1].seq, 7);
        assert_eq!(parsed.findings[1].section, None);
    }

    #[test]
    fn chunks_at_turn_boundaries() {
        let log = "header\n[0] T1 Setup AngloEgyptian  x\n=== Turn 1 complete ===\n[3] T2 Movement Dervish  y\n";
        let chunks = chunk_log(log);
        assert_eq!(chunks.len(), 2);
        // A boundary marker begins the next chunk.
        assert!(chunks[0].contains("[0]"));
        assert!(!chunks[0].contains("complete"));
        assert!(chunks[1].contains("complete"));
        assert!(chunks[1].contains("[3]"));
    }

    #[test]
    fn counts_events_from_seq_lines() {
        let log = "[0] T1 Setup AngloEgyptian  a\n[1] T1 Setup AngloEgyptian  b\n      → UnitEliminated\n";
        assert_eq!(count_events(log), 2);
    }

    #[test]
    fn review_aggregates_across_chunks() {
        let log = "[0] T1 Setup AngloEgyptian  a\n=== Turn 1 complete ===\n[3] T2 Movement Dervish  b\n";
        let canned = Canned(
            "CACHE:\nnote\nFINDINGS:\n- info|0|§4|setup\nSUMMARY:\ns1",
        );
        let cfg = LlmConfig {
            api_key: Some("k".to_string()),
            base_url: "http://x".to_string(),
            model: "m".to_string(),
        };
        let report = futures::executor::block_on(review(log, &cfg, &canned, "crib"));
        assert_eq!(report.events_audited, 2);
        assert_eq!(report.turns_audited, 1);
        assert_eq!(report.findings.len(), 2, "one finding per chunk, two chunks");
        assert_eq!(report.summary, "s1");
    }

    #[test]
    fn no_key_skips_review() {
        let cfg = LlmConfig {
            api_key: None,
            base_url: "http://x".to_string(),
            model: "m".to_string(),
        };
        let report = futures::executor::block_on(review(
            "[0] T1 Setup AngloEgyptian  a",
            &cfg,
            &Canned(""),
            "crib",
        ));
        assert!(report.findings.is_empty());
        assert!(report.summary.contains("No API key"));
        assert_eq!(report.events_audited, 1);
    }
}

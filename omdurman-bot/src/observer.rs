//! Offline review pass over a game log.
//!
//! A full game is too large for a single LLM prompt, so the observer feeds the
//! log to the model **turn by turn**, carrying a running notes/findings cache
//! between chunks. Each chunk's reply is a single JSON object deserialized
//! into [`ReviewResponse`] — findings deserialize per-item, so a malformed
//! finding is dropped while its well-formed siblings survive. The result is an
//! [`ObserverReport`] of §-cited [`Finding`]s plus a closing summary.
//!
//! Findings are **advisory**: they surface suspicions for a human to triage,
//! layered on top of the deterministic hard invariants (`invariants::check_all`)
//! and the engine's own `can_*` validation. Automated gating stays on those.

use futures::future::BoxFuture;
use omdurman_net::llm::{LlmConfig, LlmError};

use crate::llm::{LlmCache, strip_json_fence};

/// Severity of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Severity {
    Critical,
    Error,
    Warning,
    Info,
}

impl Severity {
    /// Case-insensitive parse of the wire label (`warning`, `Error`, …).
    /// Used both by the JSON deserializer and by tests.
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "critical" => Some(Severity::Critical),
            "error" => Some(Severity::Error),
            "warning" => Some(Severity::Warning),
            "info" => Some(Severity::Info),
            _ => None,
        }
    }

    /// Lower-case label used in the response protocol and report.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

impl<'de> serde::Deserialize<'de> for Severity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Severity::parse(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown severity {s:?}")))
    }
}

/// A single rule violation (or suspicion) found in the log.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    pub severity: Severity,
    /// Sequence number of the log event the finding refers to.
    pub seq: usize,
    /// Rulebook section cited, without the `§` prefix.
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub explanation: String,
}

/// The aggregated result of an observer pass.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ObserverReport {
    pub findings: Vec<Finding>,
    /// The LLM's closing assessment (from the last chunk's `summary`).
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
        Box::pin(omdurman_net::llm::request_completion(
            config, system, user, max_tokens,
        ))
    }
}

const OBSERVER_SYSTEM_PROMPT: &str = "\
You are an independent rules auditor for the board game 'Remember Gordon!' \
(The Battle of Omdurman, Phoenix Enterprises 1982). You review a \
machine-generated game log and flag every place where the log contradicts the \
rulebook. Use the crib sheet for the rule numbers; where the log alone is \
ambiguous, raise a Warning and say what is uncertain. \
\
LOG FORMAT (how to read a chunk):\n\
  [seq] T<turn> <phase> <player>  <action>   -- one applied game event\n\
        -> <engine observation> [event seq]  -- engine-side consequence\n\
  [reasoning, <side> T<turn>] <text>         -- the acting agent's own note\n\
  [note, T<turn>] <text>                     -- driver annotation (a dropped\
 plan or a rejected pick)\n\
  === Turn N complete (...) ===              -- turn boundary\n\
Key rules of thumb:\n\
  - AdvanceAfterCombat is legal ONLY into a hex the engine marked vacated by\
 combat (a HexVacatedByCombat observation), by a unit listed as eligible,\
 during a fire or melee phase (rulebook 6.82/7.6). There is no advance after\
 defensive fire (6.7).\n\
  - A unit fires at most once per fire subphase; Maxims may fire again in the\
 Maxim Second Fire and Howitzer subphase (6.42).\n\
   - Cite rule numbers from the crib sheet only (e.g. 6.82); never invent\
 sections or write N/A.\n\
Respond with exactly one JSON object, no code fence and no prose:\n\
{\"cache\": \"<working notes; carry open questions and a running tally>\",\n\
 \"findings\": [{\"severity\": \"warning|error|critical|info\", \
 \"seq\": <int>, \"section\": \"<rule number, no '§'>\", \
 \"explanation\": \"<why>\"}],\n\
 \"summary\": \"<one-paragraph closing assessment>\"}\n\
Omit \"findings\" (or an empty array) when a chunk is clean.";

/// The model's structured reply for one review chunk.
///
/// Deserialized from JSON. `findings` is kept as raw JSON values so items are
/// converted one at a time — a single malformed finding is dropped while its
/// well-formed siblings survive.
#[derive(serde::Deserialize, Default)]
struct ReviewResponse {
    #[serde(default)]
    cache: String,
    #[serde(default)]
    findings: Vec<serde_json::Value>,
    #[serde(default)]
    summary: String,
}

impl ReviewResponse {
    /// Split into `(cache, findings, summary)`, converting findings
    /// per-item so a malformed finding is dropped while its well-formed
    /// siblings survive.
    fn into_parts(self) -> (String, Vec<Finding>, String) {
        let findings = self
            .findings
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        (self.cache, findings, self.summary)
    }
}

/// Parse a chunk reply. On any failure returns the defaults, so a malformed
/// chunk keeps the previous cache and contributes nothing.
fn parse_review_response(text: &str) -> ReviewResponse {
    match serde_json::from_str::<ReviewResponse>(strip_json_fence(text)) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("warning: review chunk is not valid JSON; keeping previous cache: {e}");
            ReviewResponse::default()
        }
    }
}

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
    let header = log_header(log);
    // Every chunk reply must be JSON, so enforce it at the transport.
    let json_config = config.clone().with_json_object();
    let mut cache = String::new();
    let mut findings: Vec<Finding> = Vec::new();
    let mut summary = String::new();
    let mut turns = 0usize;

    for (i, chunk) in chunks.iter().enumerate() {
        let user = build_review_prompt(crib, &header, &cache, i, chunks.len(), chunk, i == 0);
        let response = match completion
            .complete(&json_config, OBSERVER_SYSTEM_PROMPT, &user, 2000)
            .await
        {
            Ok(text) => text,
            Err(_) => continue, // degraded chunk: keep previous cache
        };
        let (chunk_cache, chunk_findings, chunk_summary) =
            parse_review_response(&response).into_parts();
        if !chunk_cache.is_empty() {
            let mut capped = LlmCache(chunk_cache);
            capped.truncate_to_cap();
            cache = capped.0;
        }
        findings.extend(chunk_findings);
        // The same issue can be re-flagged from a later chunk (the model
        // carries it in CACHE:) -- dedupe on (severity, seq, section) so the
        // report lists each distinct finding once.
        let mut seen: Vec<(Severity, usize, Option<String>)> = Vec::new();
        findings.retain(|f| {
            let key = (f.severity, f.seq, f.section.clone());
            if seen.contains(&key) {
                false
            } else {
                seen.push(key);
                true
            }
        });
        if !chunk_summary.is_empty() {
            summary = chunk_summary;
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

/// The log's header block (everything before the first `[seq]` event line):
/// scenario, seed, agents, rules version. Attached to every review chunk so
/// the model never audits a turn without knowing which game it belongs to.
pub fn log_header(log: &str) -> String {
    let mut out = String::new();
    for line in log.lines() {
        if line.starts_with('[') && line.contains("] T") {
            break;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Whether a chunk contains a turn-boundary marker.
fn chunk_has_turn_boundary(chunk: &str) -> bool {
    chunk
        .lines()
        .any(|l| l.starts_with("=== Turn ") && l.contains("complete"))
}

/// Count `[seq]` event lines in the log.
pub fn count_events(log: &str) -> usize {
    log.lines()
        .filter(|l| l.starts_with('[') && l.contains("] T"))
        .count()
}

/// Build the user prompt for one chunk. The crib sheet is attached to the
/// first chunk only; the game header rides along on every chunk.
fn build_review_prompt(
    crib: &str,
    header: &str,
    cache: &str,
    idx: usize,
    total: usize,
    chunk: &str,
    first: bool,
) -> String {
    let mut user = String::new();
    user.push_str(&format!("=== REVIEW CHUNK {}/{} ===\n", idx + 1, total));
    if !header.trim().is_empty() {
        user.push_str("\n=== GAME HEADER ===\n");
        user.push_str(header);
        user.push_str("=== END GAME HEADER ===\n");
    }
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

    /// Returns a different canned response per call (per review chunk).
    struct CannedSeq(Vec<&'static str>, std::sync::atomic::AtomicUsize);

    impl Completion for CannedSeq {
        fn complete<'a>(
            &'a self,
            _config: &'a LlmConfig,
            _system: &'a str,
            _user: &'a str,
            _max_tokens: u32,
        ) -> BoxFuture<'a, Result<String, LlmError>> {
            use std::sync::atomic::Ordering;
            let idx = usize::min(self.1.load(Ordering::Relaxed), self.0.len() - 1);
            self.1.store(idx + 1, Ordering::Relaxed);
            let out = self.0[idx].to_string();
            Box::pin(async move { Ok(out) })
        }
    }

    #[test]
    fn parses_json_findings() {
        let parsed = parse_review_response(
            r#"{"cache":"keep track","findings":[
               {"severity":"warning","seq":12,"section":"5.11","explanation":"move cost"},
               {"severity":"error","seq":4,"section":"6.22","explanation":"wrong row"}],
               "summary":"done"}"#,
        );
        assert_eq!(parsed.cache, "keep track");
        assert_eq!(parsed.summary, "done");
        let (_, findings, _) = parsed.into_parts();
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].severity, Severity::Warning);
        assert_eq!(findings[0].seq, 12);
        assert_eq!(findings[0].section.as_deref(), Some("5.11"));
        assert_eq!(findings[0].explanation, "move cost");
        assert_eq!(findings[1].severity, Severity::Error);
        assert_eq!(findings[1].section.as_deref(), Some("6.22"));
    }

    #[test]
    fn tolerates_malformed_finding_items() {
        let parsed = parse_review_response(
            r#"{"cache":"","findings":[
               {"severity":"warning","seq":12,"section":"5.11","explanation":"ok"},
               "not an object",
               {"severity":"error","seq":"bad","section":"1","explanation":"x"},
               {"severity":"info","seq":7,"explanation":"no section"}],
               "summary":""}"#,
        );
        let (_, findings, _) = parsed.into_parts();
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].seq, 12);
        assert_eq!(findings[1].seq, 7);
        assert_eq!(findings[1].section, None);
    }

    #[test]
    fn malformed_chunk_keeps_previous_cache() {
        let parsed = parse_review_response("PLAN:\n[0]\n- not json at all");
        assert_eq!(parsed.cache, "");
        assert!(parsed.into_parts().1.is_empty());
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
        let log =
            "[0] T1 Setup AngloEgyptian  a\n=== Turn 1 complete ===\n[3] T2 Movement Dervish  b\n";
        let canned = CannedSeq(
            vec![
                r#"{"cache":"note","findings":[{"severity":"info","seq":0,"section":"4","explanation":"setup"}],"summary":"s1"}"#,
                r#"{"cache":"note2","findings":[{"severity":"warning","seq":3,"section":"5.11","explanation":"move"}],"summary":"s2"}"#,
            ],
            std::sync::atomic::AtomicUsize::new(0),
        );
        let cfg = LlmConfig {
            api_key: Some("k".to_string()),
            base_url: "http://x".to_string(),
            model: "m".to_string(),
            response_format: None,
        };
        let report = futures::executor::block_on(review(log, &cfg, &canned, "crib"));
        assert_eq!(report.events_audited, 2);
        assert_eq!(report.turns_audited, 1);
        assert_eq!(report.findings.len(), 2, "one distinct finding per chunk");
        assert_eq!(report.summary, "s2");
    }

    #[test]
    fn review_dedupes_repeated_findings_across_chunks() {
        // The model re-flags the same (severity, seq, section) from a later
        // chunk (it carries the finding in CACHE:) -- the report keeps one.
        let log =
            "[0] T1 Setup AngloEgyptian  a\n=== Turn 1 complete ===\n[3] T2 Movement Dervish  b\n";
        let canned = Canned(
            r#"{"cache":"note","findings":[{"severity":"info","seq":0,"section":"4","explanation":"setup"}],"summary":"s1"}"#,
        );
        let cfg = LlmConfig {
            api_key: Some("k".to_string()),
            base_url: "http://x".to_string(),
            model: "m".to_string(),
            response_format: None,
        };
        let report = futures::executor::block_on(review(log, &cfg, &canned, "crib"));
        assert_eq!(report.findings.len(), 1, "identical findings deduplicated");
    }

    #[test]
    fn no_key_skips_review() {
        let cfg = LlmConfig {
            api_key: None,
            base_url: "http://x".to_string(),
            model: "m".to_string(),
            response_format: None,
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

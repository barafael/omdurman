//! Per-turn LLM strategy advisor with a 500 KB persistent cache.
//!
//! In `PlayStrategy::LlmAdvised` mode, the bot asks the LLM once per
//! player-turn for a plan (action indices + reasoning), and the model returns
//! an updated cache that is threaded to the next turn. The tagged response
//! protocol (CACHE/PLAN/REASONING) is robust for large free-form text. The
//! section parser here is also shared by the offline observer
//! (`crate::observer`), which speaks the same tagged protocol.

use std::collections::HashMap;

use omdurman_net::llm::{request_completion, LlmConfig, LlmError};
use omdurman_rules::effects::GameEffect;
use omdurman_rules::effects::GameState;
use omdurman_types::Player;

/// Maximum cache size: 500 KB.
pub const MAX_CACHE_BYTES: usize = 512_000;

/// A per-turn reasoning note attached to an event index in the trace.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmAnnotation {
    pub event_idx: usize,
    pub text: String,
}

/// The LLM's persistent scratchpad across turns. Starts empty; updated each
/// turn by parsing the response.
#[derive(Default, Clone)]
pub struct LlmCache(pub String);

impl LlmCache {
    /// Hard-cap at [`MAX_CACHE_BYTES`] on a char boundary, appending a marker.
    pub fn truncate_to_cap(&mut self) {
        if self.0.len() > MAX_CACHE_BYTES {
            let cut = self
                .0
                .char_indices()
                .take_while(|(i, _)| *i <= MAX_CACHE_BYTES)
                .last()
                .map(|(i, _)| i)
                .unwrap_or(MAX_CACHE_BYTES);
            self.0.truncate(cut);
            self.0.push_str("\n…[cache truncated at 500 KB]");
        }
    }
}

/// Split a tagged LLM response into per-section buckets of raw payload lines.
///
/// Header lines (`CACHE:`, `PLAN:`, `REASONING:`, `FINDINGS:`, `SUMMARY:`)
/// switch the current section; every following line is appended to that
/// section's bucket. Lines before the first header are ignored.
pub(crate) fn parse_sections<'a>(text: &'a str) -> HashMap<&'static str, Vec<&'a str>> {
    let mut sections: HashMap<&'static str, Vec<&'a str>> = HashMap::new();
    let mut current: Option<&'static str> = None;
    for line in text.lines() {
        if let Some(name) = section_name(line.trim()) {
            current = Some(name);
        } else if let Some(name) = current {
            sections.entry(name).or_default().push(line);
        }
    }
    sections
}

/// The section name of a header line (`CACHE:` → `cache`), or `None`.
fn section_name(line: &str) -> Option<&'static str> {
    match line {
        "CACHE:" => Some("cache"),
        "PLAN:" => Some("plan"),
        "REASONING:" => Some("reasoning"),
        "FINDINGS:" => Some("findings"),
        "SUMMARY:" => Some("summary"),
        _ => None,
    }
}

/// The `[q, q, …]` (or comma-separated) index list of one `PLAN:` line.
fn parse_plan_line(line: &str) -> Vec<usize> {
    let cleaned: String = line.trim().trim_matches(|c| c == '[' || c == ']').to_string();
    cleaned
        .split(',')
        .filter_map(|part| part.trim().parse::<usize>().ok())
        .collect()
}

/// Parsed LLM response: the updated cache, a plan (indices into
/// `legal_actions`), and reasoning annotations.
struct ParsedResponse {
    cache: String,
    plan: Vec<usize>,
    reasoning: Vec<String>,
}

/// Parse the tagged response format:
/// ```text
/// CACHE:
/// <notes>
/// PLAN:
/// [3, 7, 12]
/// REASONING:
/// - 3: ...
/// - 7: ...
/// ```
fn parse_response(text: &str) -> ParsedResponse {
    let sections = parse_sections(text);
    ParsedResponse {
        cache: sections.get("cache").map(|l| l.join("\n")).unwrap_or_default(),
        plan: sections
            .get("plan")
            .map(|lines| lines.iter().flat_map(|l| parse_plan_line(l)).collect())
            .unwrap_or_default(),
        reasoning: sections
            .get("reasoning")
            .map(|lines| {
                lines
                    .iter()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// Build the user-prompt for a per-turn strategy query. Describes the current
/// state and lists the enumerated legal actions (indexed).
fn build_prompt(state: &GameState, actions: &[GameEffect]) -> String {
    let mut buf = String::new();
    buf.push_str(&format!(
        "Scenario: {:?}\nTurn: {}  Phase: {:?}  Player: {:?}\n\n",
        state.scenario,
        state.current_turn.value(),
        state.phase,
        state.active_player,
    ));
    buf.push_str("Friendly units:\n");
    for u in state.units.iter().filter(|u| {
        u.profile.identity.owner() == state.active_player
    }) {
        buf.push_str(&format!(
            "  {:?} at ({},{})\n",
            u.profile.identity, u.position.q, u.position.r
        ));
    }
    buf.push_str("\nEnemy units:\n");
    let enemy = state.active_player.opponent();
    for u in state.units.iter().filter(|u| u.profile.identity.owner() == enemy) {
        buf.push_str(&format!(
            "  {:?} at ({},{})\n",
            u.profile.identity, u.position.q, u.position.r
        ));
    }
    buf.push_str(&format!("\nLegal actions ({} total):\n", actions.len()));
    for (i, a) in actions.iter().enumerate() {
        buf.push_str(&format!("  [{i}] {a:?}\n"));
    }
    buf
}

/// Ask the LLM for a per-turn plan. Returns the chosen action indices (into
/// `actions`), reasoning annotations, and the updated cache. On any error or
/// malformed response, returns an empty plan (caller falls back to random).
///
/// `side` names the faction being advised and `brief` is an optional persona
/// brief prepended to the system prompt, so a per-side agent can sound like its
/// commander.
pub async fn advise_turn(
    config: &LlmConfig,
    side: Player,
    brief: &str,
    state: &GameState,
    actions: &[GameEffect],
    cache: &mut LlmCache,
) -> (Vec<usize>, Vec<LlmAnnotation>, bool) {
    if !config.has_key() {
        return (Vec::new(), Vec::new(), false);
    }

    let mut system = format!(
        "You are playing the board game 'Remember Gordon!' as the {side} player. \
         Pick the best plan for this turn by returning action indices. \
         Cite rulebook sections (§N) for each choice. \
         Update your notes each turn — they are your only memory between turns. \
         Respond in this format:\n\
         CACHE:\n<your updated notes>\n\n\
         PLAN:\n[index, index, ...]\n\n\
         REASONING:\n- index: reason (§N)"
    );
    if !brief.is_empty() {
        system = format!("{system}\n\nYour brief: {brief}");
    }

    let mut user = String::new();
    if !cache.0.is_empty() {
        user.push_str("=== NOTES FROM PREVIOUS TURNS ===\n");
        user.push_str(&cache.0);
        user.push_str("\n=== END NOTES ===\n\n");
    }
    user.push_str(&build_prompt(state, actions));

    let response = match request_completion(config, &system, &user, 2000).await {
        Ok(text) => text,
        Err(LlmError::NoApiKey) => return (Vec::new(), Vec::new(), false),
        Err(_) => return (Vec::new(), Vec::new(), false),
    };

    let parsed = parse_response(&response);
    cache.0 = parsed.cache;
    cache.truncate_to_cap();

    let annotations = parsed
        .reasoning
        .into_iter()
        .map(|text| LlmAnnotation {
            event_idx: 0, // filled in by the caller
            text,
        })
        .collect();

    (parsed.plan, annotations, true)
}

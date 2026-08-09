//! Per-turn LLM strategy advisor with a 500 KB persistent cache.
//!
//! In `PlayStrategy::LlmAdvised` mode, the bot asks the LLM once per
//! player-turn for a plan (action indices + reasoning), and the model returns
//! an updated cache that is threaded to the next turn. The tagged response
//! protocol (CACHE/PLAN/REASONING) is robust for large free-form text.

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
    let mut cache = String::new();
    let mut plan = Vec::new();
    let mut reasoning = Vec::new();

    let mut section = "";
    for line in text.lines() {
        let trimmed = line.trim();
        match trimmed {
            "CACHE:" => section = "cache",
            "PLAN:" => section = "plan",
            "REASONING:" => section = "reasoning",
            _ => match section {
                "cache" => {
                    if !cache.is_empty() {
                        cache.push('\n');
                    }
                    cache.push_str(line);
                }
                "plan" => {
                    // Extract numbers from brackets or comma-separated.
                    let cleaned: String = trimmed
                        .trim_matches(|c| c == '[' || c == ']')
                        .chars()
                        .collect();
                    for part in cleaned.split(',') {
                        if let Ok(n) = part.trim().parse::<usize>() {
                            plan.push(n);
                        }
                    }
                }
                "reasoning" => {
                    if !trimmed.is_empty() {
                        reasoning.push(trimmed.to_string());
                    }
                }
                _ => {}
            },
        }
    }
    ParsedResponse {
        cache,
        plan,
        reasoning,
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
    let _ = &mut LlmCache::default(); // suppress unused warning when feature off
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

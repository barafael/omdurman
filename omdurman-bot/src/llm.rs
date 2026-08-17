//! Per-turn LLM strategy advisor with a 500 KB persistent cache.
//!
//! In `AgentStrategy::LlmAdvised` mode, the bot asks the LLM once per
//! player-turn for a plan (action indices + reasoning), and the model returns
//! an updated cache that is threaded to the next turn. The reply is a single
//! JSON object deserialized into [`PlanResponse`] — the same serde machinery
//! the rest of the workspace uses, no ad-hoc line protocol. The planner and
//! the offline observer (`crate::observer`) share the reply shape, so both
//! speak one format. On any malformed reply the caller degrades: empty plan →
//! random move, previous cache kept.

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

/// The model's structured reply to a per-turn strategy query.
///
/// Deserialized from the model's JSON output. Every field defaults, so a
/// missing or malformed section degrades exactly like the old tagged protocol:
/// an absent `plan` → empty vector (caller falls back to random), an absent
/// `cache` → the previous scratchpad is kept.
#[derive(Debug, serde::Deserialize, serde::Serialize, Default)]
pub struct PlanResponse {
    /// Updated notes for the next turn — the agent's only memory between turns.
    #[serde(default)]
    pub cache: String,
    /// Indices into the enumerated `legal_actions` list, in order.
    #[serde(default)]
    pub plan: Vec<usize>,
    /// One reason per planned action, tagged with its index.
    #[serde(default)]
    pub reasoning: Vec<String>,
}

/// Strip a single optional ```json … ``` code fence (and surrounding prose
/// markers) from a completion reply, then trim. LLMs wrap structured output
/// in fences even when told not to; this is the one spot that tolerates it.
/// Shared by the planner and the offline observer.
pub(crate) fn strip_json_fence(text: &str) -> &str {
    let mut t = text.trim();
    if let Some(rest) = t
        .strip_prefix("```json")
        .or_else(|| t.strip_prefix("```"))
    {
        t = rest;
    }
    if let Some(rest) = t.strip_suffix("```") {
        t = rest;
    }
    t.trim()
}

/// Parse the JSON reply into a [`PlanResponse`]. On any parse failure returns
/// the defaults, so the caller keeps its degrade behaviour (empty plan →
/// random pick; empty cache → previous cache kept).
fn parse_response(text: &str) -> PlanResponse {
    match serde_json::from_str::<PlanResponse>(strip_json_fence(text)) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("warning: plan response is not valid JSON; falling back to an empty plan: {e}");
            PlanResponse::default()
        }
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
         Respond with exactly one JSON object, no code fence and no prose:\n\
         {{\"cache\": \"<your updated notes, escaped as a JSON string>\", \
         \"plan\": [<index>, ...], \
         \"reasoning\": [\"- <index>: <reason (§N)>\", ...]}}\n\
         Omit or empty the sections you have nothing to say for."
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

    let json_config = config.clone().with_json_object();
    // 2000 tokens truncated long CACHE/PLAN responses mid-string (the model
    // writes extensive notes with a full order of battle on the board),
    // yielding unparseable JSON and an empty-plan fallback every turn.
    // 6000 covers the largest observed responses with headroom.
    let response = match request_completion(&json_config, &system, &user, 6000).await {
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

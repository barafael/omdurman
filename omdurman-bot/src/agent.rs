//! Per-side agent configuration for head-to-head play.
//!
//! A playthrough runs *two* independent agents — one per faction — so the same
//! seed can pair a Random Anglo-Egyptian against an LLM-advised Dervish (or any
//! other combination). Each side owns its strategy, its persona brief, and its
//! own 500 KB `LlmCache` (see `playthrough`).

use omdurman_net::llm::LlmConfig;
use omdurman_types::Player;

/// What an agent is allowed to be. Owned separately by each side.
#[derive(Clone)]
pub enum AgentStrategy {
    /// Uniform-random over `legal_actions`. Fast; broadest raw coverage.
    Random,
    /// Ask the LLM once per player-turn for a plan (needs an API key).
    LlmAdvised {
        config: LlmConfig,
        /// Short strategic brief prepended to the system prompt, e.g.
        /// "You command the Dervish. Your infantry must close to melee while
        /// the Khalifa survives; you win by British losses."
        brief: String,
    },
    /// Greedy aggressor (no LLM): score every legal action and always take
    /// the best. The Dervish swarm the objective (the Palace / GORDON in
    /// Fall of Khartoum, §9.346), never retreat, and prefer melee over fire
    /// over movement over ending the phase. See `crate::aggressive`.
    Aggressive,
    /// A named historical commander with scenario-adaptive doctrine (no LLM):
    /// **Kitchener** commands the Anglo-Egyptian, **Khalifa** the Dervish.
    /// See `crate::commanders`.
    Commander(crate::commanders::Commander),
}

/// One strategy per faction.
#[derive(Clone)]
pub struct Agents {
    pub ae: AgentStrategy,
    pub dervish: AgentStrategy,
}

impl Agents {
    /// Two uniform-random agents — the default for determinism/coverage runs.
    pub fn random() -> Self {
        Self {
            ae: AgentStrategy::Random,
            dervish: AgentStrategy::Random,
        }
    }

    /// The strategy commanding `player`.
    pub fn strategy_for(&self, player: Player) -> &AgentStrategy {
        match player {
            Player::AngloEgyptian => &self.ae,
            Player::Dervish => &self.dervish,
        }
    }

    /// Whether the side is LLM-advised (vs. random).
    pub fn is_llm(&self, player: Player) -> bool {
        matches!(self.strategy_for(player), AgentStrategy::LlmAdvised { .. })
    }

    /// Whether the side plays the greedy-aggressor heuristic.
    pub fn is_aggressive(&self, player: Player) -> bool {
        matches!(self.strategy_for(player), AgentStrategy::Aggressive)
    }

    /// The side's commander, if it plays one.
    pub fn commander(&self, player: Player) -> Option<crate::commanders::Commander> {
        match self.strategy_for(player) {
            AgentStrategy::Commander(c) => Some(*c),
            _ => None,
        }
    }

    /// Whether any side plays a historical commander (drives deep-setup
    /// candidate generation in the playthrough driver).
    pub fn any_commander(&self) -> bool {
        matches!(
            (&self.ae, &self.dervish),
            (AgentStrategy::Commander(_), _) | (_, AgentStrategy::Commander(_))
        )
    }

    /// The `(config, brief)` of the side's LLM advisor, if it is one.
    pub fn llm_config(&self, player: Player) -> Option<(&LlmConfig, &str)> {
        match self.strategy_for(player) {
            AgentStrategy::LlmAdvised { config, brief } => Some((config, brief)),
            AgentStrategy::Random | AgentStrategy::Aggressive | AgentStrategy::Commander(_) => None,
        }
    }

    /// Human-readable agent label for the log header / run manifest.
    pub fn label_for(&self, player: Player) -> String {
        match self.strategy_for(player) {
            AgentStrategy::Random => "random".to_string(),
            AgentStrategy::Aggressive => "aggressive".to_string(),
            AgentStrategy::Commander(c) => c.name().to_string(),
            AgentStrategy::LlmAdvised { brief, .. } => {
                if brief.is_empty() {
                    "llm".to_string()
                } else {
                    format!("llm({brief})")
                }
            }
        }
    }
}

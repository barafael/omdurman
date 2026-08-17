//! Headless AI playthrough driver for *Remember Gordon!* — plays full games by
//! driving [`omdurman_rules::effects::apply_effect`] directly (no Bevy, no
//! render loop), logging every move as a replayable
//! [`omdurman_net::GameEvent`] trace.
//!
//! Two independent per-faction agents play head-to-head:
//! - [`AgentStrategy::Random`] — uniform-random over [`actions::legal_actions`].
//!   Fast; broadest raw coverage of the action space.
//! - [`AgentStrategy::LlmAdvised`] — asks an LLM once per player-turn for a plan,
//!   with a per-side 500 KB persistent cache threaded turn-to-turn.
//!
//! The playthrough also builds a human-readable [`GameLog`] (actions +
//! engine observations with § citations + turn summaries) that the offline
//! [`observer`] audits for rule violations. The output event traces are
//! byte-compatible with the app's `SpectatorTimeline` replay viewer.

pub mod actions;
pub mod agent;
pub mod audit;
pub mod describe;
pub mod doctrine;
pub mod invariants;
pub mod llm;
pub mod log;
pub mod oob;
pub mod observer;
pub mod playthrough;
pub mod rng;

pub use actions::legal_actions;
pub use agent::{AgentStrategy, Agents};
pub use describe::{describe_effect, describe_observation};
pub use doctrine::{corpus_files, doctrine_brief};
pub use invariants::{check_all, check_all_with_tribal};
pub use llm::{LlmAnnotation, LlmCache, MAX_CACHE_BYTES};
pub use log::GameLog;
pub use oob::{deployable_oob, deployable_oob_for, fixed_placements};
pub use observer::{chunk_log, count_events, review, Completion, Finding, ObserverReport, Severity};
pub use playthrough::{board_for_scenario, playthrough, PlayConfig, PlayResult};
pub use rng::BotRng;

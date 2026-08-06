//! Headless AI playthrough driver for *Remember Gordon!* — plays full games by
//! driving [`omdurman_rules::effects::apply_effect`] directly (no Bevy, no
//! render loop), logging every move as a replayable
//! [`omdurman_net::GameEvent`] trace.
//!
//! Two strategies:
//! - [`PlayStrategy::Random`] — uniform-random over [`actions::legal_actions`].
//!   Fast; broadest raw coverage of the action space.
//! - [`PlayStrategy::LlmAdvised`] — asks an LLM once per player-turn for a plan,
//!   with a 500 KB persistent cache threaded turn-to-turn.
//!
//! The output traces are byte-compatible with the app's `SpectatorTimeline`
//! replay viewer, so a downstream agent (or the app itself) can review them.

pub mod actions;
pub mod invariants;
pub mod llm;
pub mod oob;
pub mod playthrough;
pub mod rng;

pub use actions::legal_actions;
pub use invariants::{check_all, check_all_with_tribal};
pub use llm::{LlmAnnotation, LlmCache, MAX_CACHE_BYTES};
pub use oob::{deployable_oob, deployable_oob_for, fixed_placements};
pub use playthrough::{board_for_scenario, playthrough, PlayConfig, PlayResult, PlayStrategy};
pub use rng::BotRng;

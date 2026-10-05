//! Headless AI playthrough driver for *Remember Gordon!* — plays full games by
//! driving [`omdurman_rules::effects::apply_effect`] directly (no Bevy, no
//! render loop), logging every move as a replayable
//! [`omdurman_net::GameEvent`] trace.
//!
//! Two independent per-faction agents play head-to-head:
//! - [`agent::AgentStrategy::Random`] — uniform-random over [`actions::legal_actions`].
//!   Fast; broadest raw coverage of the action space.
//! - [`agent::AgentStrategy::Aggressive`] — greedy objective-seeking aggressor
//!   (melee over fire, never retreat, march on the Palace).
//! - [`agent::AgentStrategy::Commander`] — the historical commanders (Kitchener /
//!   the Khalifa) with scenario-adaptive doctrine; also the in-game AI.
//!
//! The playthrough also builds a human-readable [`log::GameLog`] (actions +
//! engine observations with § citations + turn summaries) that the
//! deterministic [`audit`] scanners check for rule deviations. The output
//! event traces are byte-compatible with the app's `SpectatorTimeline`
//! replay viewer.

pub mod actions;
pub mod agent;
pub mod aggressive;
pub mod arena;
pub mod audit;
pub mod baseline;
pub mod commanders;
pub mod describe;
pub mod fire_plan;
pub mod invariants;
pub mod log;
pub mod move_memory;
pub mod oob;
pub mod playthrough;
pub mod rng;
pub mod threat;

pub use playthrough::{PlayConfig, board_for_scenario, playthrough};

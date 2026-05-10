use serde::{Deserialize, Serialize};

/// Placeholder for the event-sourcing contract shared across crates.
///
/// Real game actions will replace `data: u32` once rules are implemented.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameEvent {
    Action(u32),
}

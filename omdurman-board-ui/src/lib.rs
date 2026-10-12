//! Board-view plumbing for the game app (`omdurman-app`).
//!
//! * [`input`] — key helpers
//! * [`camera`] — the RTS camera (component, settings, control systems)
//! * [`night`] — day/night colour grading (opt-in, driven by a resource)
//! * [`panels`] — egui pointer gating + the declarative map-input set
//! * [`board_store`] — two-board store, deferred board loads, map plane
//!
//! The binary keeps only its small local `Plugin` wiring and its
//! app-specific hooks (e.g. attaching `BoardInfo` to the engine state).

pub mod board_store;
pub mod camera;
mod input;
pub mod night;
pub mod panels;

pub use board_store::*;
pub use camera::*;
pub use night::*;
pub use panels::*;

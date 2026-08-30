//! Shared board-view plumbing used by both the game app (`omdurman-app`) and
//! the map editor (`tools/map-editor`).
//!
//! Everything here used to exist as two near-verbatim copies (one per
//! binary); any edit to one silently diverged the editor's rendering from the
//! game's. The single copies now live here:
//!
//! * [`input`] — key/raycast helpers
//! * [`camera`] — the RTS camera (component, settings, control systems)
//! * [`night`] — day/night colour grading (opt-in, driven by a resource)
//! * [`panels`] — egui pointer gating + the declarative map-input set
//! * [`sprites`] — the sprite-annotation resource
//! * [`board_store`] — two-board store, deferred board loads, map plane, lights
//!
//! The binaries keep only their small local `Plugin` wiring and their
//! app-specific hooks (e.g. attaching `BoardInfo` to the engine state).

pub mod board_store;
pub mod camera;
pub mod input;
pub mod night;
pub mod panels;
pub mod sprites;

pub use board_store::*;
pub use camera::*;
pub use input::*;
pub use night::*;
pub use panels::*;
pub use sprites::*;

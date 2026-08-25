//! Crate-level prelude: re-exports the most commonly used types to cut down
//! on per-file import boilerplate. Bring it in with `use crate::prelude::*;`.
//!
//! Warnings about "unused imports" here are expected — the prelude is consumed
//! by other modules via `use crate::prelude::*;`.

#![allow(unused_imports)]

pub use crate::state::{
    AppMode, AppState, GameRng, GameSet, GameStateResource, GameTurn,
};

pub use crate::render::{HexOverlay, HexRingAssets, HoveredHex};

pub use crate::board_state::{ActiveEditMap, LoadedAnnotations, PendingMapLoad};

pub use crate::peers::{LocalPeer, PeerKey, QueuedFactions};

pub use crate::net_plugin::PendingEdits;


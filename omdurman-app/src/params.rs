//! `SystemParam` bundles that group related resources so system signatures
//! stay under Bevy's system-parameter limit.
//!
//! Re-exported at the crate root so existing `crate::GameStateParams` paths
//! continue to resolve.

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout};
use std::collections::HashMap;

use crate::browser::SpriteAnnotationsResource;
use crate::editor::{ActiveEditMap, LoadedAnnotations, PendingMapLoad};
use crate::events::PendingObservations;
use crate::net_plugin::PendingIncoming;
use crate::peers::QueuedFactions;
use crate::picker::{MovementAnimation, PlacedUnit, UnitPaths, UnitPicker};
use crate::render::{HexOverlay, HexRingAssets};
use crate::state::{AppMode, GameStateResource};
use omdurman_rules::UnitId;
use omdurman_types::SectionName;

/// Bundles the rules-engine state so `handle_socket` stays under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct GameStateParams<'w> {
    pub game_state: ResMut<'w, GameStateResource>,
    /// Set by the `StartGame` handler so the view switches to the game board
    /// (the board data loads via `pending_map_load`; the view follows `AppMode`).
    pub next_app_mode: ResMut<'w, NextState<AppMode>>,
    /// In-memory two-board annotations file; the `StartGame` handler stores
    /// into it, and `request_map_load` reads from it.
    pub loaded_annotations: ResMut<'w, LoadedAnnotations>,
    /// Set by the `StartGame` handler (and the editor's map toggle) to ask
    /// `apply_map_selection` to (re)load a board on the next frame (§dual-map).
    pub pending_map_load: ResMut<'w, PendingMapLoad>,
    /// Which board is currently live, so map-edit events apply to the right
    /// section (§dual-map).
    pub active_edit_map: Res<'w, ActiveEditMap>,
    pub pending_observations: ResMut<'w, PendingObservations>,
    /// Faction bindings from a `StartGame` (live or replayed), staged here and
    /// applied to peer entities by `peers::apply_faction_bindings`.
    pub queued_factions: ResMut<'w, QueuedFactions>,
}

/// Bundles the domain-specific state consumed by [`apply_pending_placement`]
/// so the function signature stays under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PlacementContext<'w, 's> {
    pub incoming: ResMut<'w, PendingIncoming>,
    pub picker: ResMut<'w, UnitPicker>,
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
    pub game_map: Res<'w, GameMap>,
    pub game_state: Option<ResMut<'w, GameStateResource>>,
    pub annotations: Option<Res<'w, SpriteAnnotationsResource>>,
    pub unit_paths: ResMut<'w, UnitPaths>,
    pub placed_units: Query<'w, 's, (Entity, &'static mut PlacedUnit)>,
    pub anim_query: Query<'w, 's, &'static MovementAnimation>,
    /// Tracks entities spawned this invocation so MoveUnit can find units
    /// placed in the same batch (e.g. during history replay) before Bevy
    /// has flushed the deferred commands.
    pub just_placed: JustPlacedMap<'s>,
}

type JustPlacedMap<'s> =
    Local<'s, HashMap<(SectionName, u32, u32), (Entity, bool, Option<UnitId>)>>;

#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct HexRender<'w> {
    pub assets: Res<'w, HexRingAssets>,
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
}

/// Bundle of the movement-arrow mesh/material assets with the hex-render
/// resources, used by the fire/melee direction-arrow systems to stay under
/// Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct DirectionArrowCtx<'w> {
    pub arrow_assets: Res<'w, crate::render::MovementArrowAssets>,
    pub hex: HexRender<'w>,
}

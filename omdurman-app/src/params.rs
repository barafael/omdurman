//! `SystemParam` bundles that group related resources so system signatures
//! stay under Bevy's system-parameter limit.
//!
//! Re-exported at the crate root so existing `crate::GameStateParams` paths
//! continue to resolve.

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout};

use crate::board_state::{LoadedAnnotations, PendingMapLoad};
use crate::bot_player::AiCommanders;
use crate::events::PendingObservations;
use crate::peers::{QueuedCommands, QueuedFactions};
use crate::picker::UnitPaths;
use crate::render::{HexOverlay, HexRingAssets};
use crate::state::{AppMode, GameStateResource};

/// Bundles the rules-engine state (plus the board it mutates) so
/// `handle_socket` stays under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct GameStateParams<'w> {
    pub game_state: ResMut<'w, GameStateResource>,
    /// The live board, mutated by map-edit events (`OverlayUpdate` etc.).
    pub game_map: ResMut<'w, GameMap>,
    /// Set by the `StartGame` handler so the view switches to the game board
    /// (the board data loads via `pending_map_load`; the view follows `AppMode`).
    pub next_app_mode: ResMut<'w, NextState<AppMode>>,
    /// In-memory two-board annotations file; the `StartGame` handler stores
    /// into it, and `request_map_load` reads from it.
    pub loaded_annotations: ResMut<'w, LoadedAnnotations>,
    /// Set by the `StartGame` handler (and the editor's map toggle) to ask
    /// `apply_map_selection` to (re)load a board on the next frame (§dual-map).
    pub pending_map_load: ResMut<'w, PendingMapLoad>,
    pub pending_observations: ResMut<'w, PendingObservations>,
    /// Faction bindings from a `StartGame` (live or replayed), staged here and
    /// applied to peer entities by `peers::apply_faction_bindings`.
    pub queued_factions: ResMut<'w, QueuedFactions>,
    /// Command scopes from a `StartGame` (live or replayed), staged here and
    /// applied to peer entities by `peers::apply_command_bindings` (§1.1).
    pub queued_commands: ResMut<'w, QueuedCommands>,
    /// The local member's setup readiness flag; reset whenever a `StartGame`
    /// (live or replayed) begins a fresh game (§9.2/§9.3).
    pub local_setup_ready: ResMut<'w, crate::peers::LocalSetupReady>,
    /// The AI-commanded factions from a `StartGame` (live or replayed) — the
    /// host's `bot_player` driver plays these factions' turns.
    pub ai_commanders: ResMut<'w, AiCommanders>,
    /// The always-present AI driver; `apply_start_game` reseeds it when a
    /// fresh game begins. Required existence is deliberate: the resource must
    /// never blink out while `bot_player_act` is running.
    pub bot_driver: ResMut<'w, crate::bot_player::BotDriver>,
    /// Per-turn movement routes, recorded where a move is accepted.
    pub unit_paths: ResMut<'w, UnitPaths>,
}

impl GameStateParams<'_> {
    /// Borrow the event-application sinks for [`crate::game_apply::apply_game_event`].
    pub(crate) fn sinks(&mut self) -> crate::game_apply::EventSinks<'_> {
        crate::game_apply::EventSinks {
            game_state: &mut self.game_state.0,
            queued_factions: &mut self.queued_factions,
            queued_commands: &mut self.queued_commands,
            local_setup_ready: &mut self.local_setup_ready,
            ai_commanders: &mut self.ai_commanders,
            bot_driver: &mut self.bot_driver,
            loaded_annotations: &mut self.loaded_annotations,
            pending_map_load: &mut self.pending_map_load,
            unit_paths: &mut self.unit_paths,
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct HexRender<'w> {
    pub assets: Res<'w, HexRingAssets>,
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
}

/// Bundle of the hex-layout + overlay calibration pair — everything needed to
/// convert between hex coordinates and world space (adjusted origin + hex
/// size).
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct BoardGeometry<'w> {
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

//! `SystemParam` bundles that group related resources so system signatures
//! stay under Bevy's system-parameter limit.
//!
//! Re-exported at the crate root so existing `crate::GameStateParams` /
//! `crate::FactionGate` / `crate::MoveGate` paths continue to resolve.

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout};
use omdurman_net::NetState;
use std::collections::HashMap;

use crate::browser::SpriteAnnotationsResource;
use crate::editor::{ActiveEditMap, LoadedAnnotations, PendingMapLoad};
use crate::events::PendingObservations;
use crate::net_plugin::{PendingIncoming, PlayerFactions};
use crate::picker::{MovementAnimation, PlacedUnit, UnitPaths, UnitPicker};
use crate::render::HexOverlay;
use crate::events::AppliedEvents;
use crate::state::{AppMode, GameStateResource};
use omdurman_rules::UnitId;
use omdurman_types::SectionName;

/// Bundles the rules-engine state with the per-player faction binding so
/// `handle_socket` stays under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct GameStateParams<'w> {
    pub game_state: ResMut<'w, GameStateResource>,
    pub player_factions: ResMut<'w, PlayerFactions>,
    /// In-memory two-board annotations file; the `StartGame`/`LoadAnnotations`
    /// handlers store into it, and `request_map_load` reads from it.
    pub loaded_annotations: ResMut<'w, LoadedAnnotations>,
    /// Set by the `StartGame` handler (and the editor's map toggle) to ask
    /// `apply_map_selection` to (re)load a board on the next frame (§dual-map).
    pub pending_map_load: ResMut<'w, PendingMapLoad>,
    /// Which board is currently live, so map-edit events apply to the right
    /// section (§dual-map).
    pub active_edit_map: Res<'w, ActiveEditMap>,
    /// Sequenced events applied this frame; drained by
    /// [`drain_applied_events`] into `GameEventApplied` messages.
    pub applied_events: ResMut<'w, AppliedEvents>,
    /// Observations drained from the rules engine after `apply_effect`; drained
    /// by [`drain_observations`](crate::events::drain_observations) into
    /// `ObservationEvent` messages.
    pub pending_observations: ResMut<'w, PendingObservations>,
    /// Set by the `StartGame` handler so the view switches to the game board
    /// (the board data loads via `pending_map_load`; the view follows `AppMode`).
    pub next_app_mode: ResMut<'w, NextState<AppMode>>,
}

/// Read-only bundle for the "may the local player act now" check (§lobby),
/// kept as one `SystemParam` so action handlers stay under the param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct FactionGate<'w> {
    pub factions: Res<'w, PlayerFactions>,
    pub net: Res<'w, NetState>,
}

impl FactionGate<'_> {
    /// Whether the local player controls `active` this phase.
    pub fn may_act(&self, active: omdurman_types::Player) -> bool {
        self.factions.local_may_act(&self.net, active)
    }
}

/// Bundle of the rules state + faction gate used by the picker's click handler,
/// so `handle_picker_clicks` stays under the param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct MoveGate<'w> {
    pub game_state: Option<Res<'w, GameStateResource>>,
    pub gate: FactionGate<'w>,
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
    pub just_placed: Local<'s, HashMap<(SectionName, u32, u32), (Entity, bool, Option<UnitId>)>>,
}

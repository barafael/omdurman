//! `SystemParam` bundles that group related resources so system signatures
//! stay under Bevy's system-parameter limit.
//!
//! Re-exported at the crate root so existing `crate::GameStateParams` paths
//! continue to resolve.

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout};

use crate::board_state::{LoadedAnnotations, PendingMapLoad};
use crate::events::PendingObservations;
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
    /// Set by the `StartGame` handler to ask
    /// `apply_map_selection` to (re)load a board on the next frame (§dual-map).
    pub pending_map_load: ResMut<'w, PendingMapLoad>,
    pub pending_observations: ResMut<'w, PendingObservations>,
    /// The committed seat table, written by `StartGame` and the seat events
    /// (live or replayed).
    pub seats: ResMut<'w, crate::seats::Seats>,
    /// The local member's setup readiness flag; reset whenever a `StartGame`
    /// (live or replayed) begins a fresh game (§9.2/§9.3).
    pub local_setup_ready: ResMut<'w, crate::peers::LocalSetupReady>,
    /// The always-present AI driver; `apply_start_game` reseeds it when a
    /// fresh game begins. Required existence is deliberate: the resource must
    /// never blink out while `bot_player_act` is running.
    pub bot_driver: ResMut<'w, crate::bot_player::BotDriver>,
    /// Per-turn movement routes, recorded where a move is accepted.
    pub unit_paths: ResMut<'w, UnitPaths>,
    /// The telegrams and Gazette, filed from recorded press events.
    pub press: ResMut<'w, crate::telegram::TelegramLog>,
    /// What each *live* event did to the counters, for the board effects
    /// (never written by a rebuild). Optional: headless harnesses run the
    /// receive path without the effects plugin.
    pub live_applied: Option<ResMut<'w, crate::fx::LiveApplied>>,
}

impl GameStateParams<'_> {
    /// Borrow the event-application sinks for [`crate::game_apply::apply_game_event`].
    pub(crate) fn sinks(&mut self) -> crate::game_apply::EventSinks<'_> {
        crate::game_apply::EventSinks {
            game_state: &mut self.game_state.0,
            seats: &mut self.seats,
            local_setup_ready: &mut self.local_setup_ready,
            bot_driver: &mut self.bot_driver,
            loaded_annotations: &mut self.loaded_annotations,
            pending_map_load: &mut self.pending_map_load,
            unit_paths: &mut self.unit_paths,
            press: &mut self.press,
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct HexRender<'w> {
    pub assets: Res<'w, HexRingAssets>,
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
}

impl HexRender<'_> {
    /// Whether the board geometry (layout or overlay calibration) changed
    /// since the calling system last ran -- a board was loaded, so every
    /// cached hex position is stale.
    pub(crate) fn geometry_changed(&self) -> bool {
        self.layout.is_changed() || self.overlay.is_changed()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct Seen(Vec<bool>);

    /// A board load (the overlay calibration rewritten) reports the geometry
    /// changed to the caching ring systems, once; quiet frames do not. The
    /// deployment rings used to stay drawn with the default board's
    /// calibration after the Campaign board had loaded.
    #[test]
    fn a_board_load_marks_the_hex_geometry_changed() {
        let mut app = App::new();
        app.insert_resource(crate::render::HexRingAssets::default())
            .insert_resource(omdurman_board_ui::board_store::default_layout())
            .insert_resource(HexOverlay::default())
            .init_resource::<Seen>()
            .add_systems(Update, |hex: HexRender, mut seen: ResMut<Seen>| {
                seen.0.push(hex.geometry_changed());
            });
        app.update();
        app.update();
        app.world_mut().resource_mut::<HexOverlay>().params.hex_size = 53.35;
        app.update();
        app.update();
        assert_eq!(app.world().resource::<Seen>().0, [true, false, true, false]);
    }
}

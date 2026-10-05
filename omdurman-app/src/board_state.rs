//! Game-side board bootstrap (§dual-map).
//!
//! The two-board store, the deferred load request, and the shared loading
//! flow live in `omdurman-board-ui::board_store`; this module keeps the
//! game-specific hook: seeding the engine's `BoardInfo` on every board load.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::GameStateResource;

pub use omdurman_board_ui::board_store::{
    ActiveEditMap, LoadedAnnotations, MapLoadContext, PendingMapLoad,
};

/// The shared load context plus the game's engine state.
#[derive(SystemParam)]
pub(crate) struct GameMapLoadContext<'w> {
    pub shared: MapLoadContext<'w>,
    pub game_state: ResMut<'w, GameStateResource>,
}

/// The game's `apply_map_selection`: attach the engine's view of the board
/// (so map-dependent rules — ZOC across hexsides §5.44, gunboat
/// upstream/downstream §5.24, terrain movement cost §5.11, Friendlies bank
/// §9.14 — are enforced deterministically; carried inside the serialized
/// `GameState`, so replay/late-join reproduce it), then run the shared
/// loading flow.
pub(crate) fn apply_map_selection(
    mut ctx: GameMapLoadContext,
    plane: Query<
        (&Mesh3d, &MeshMaterial3d<bevy::pbr::StandardMaterial>),
        With<omdurman_hexmap::MapPlane>,
    >,
    mut meshes: ResMut<Assets<bevy::render::mesh::Mesh>>,
    mut materials: ResMut<Assets<bevy::pbr::StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Some(kind) = omdurman_board_ui::board_store::take_pending(&mut ctx.shared) else {
        return;
    };
    let map = ctx.shared.loaded.map(kind);
    ctx.game_state.0.board =
        std::sync::Arc::new(omdurman_rules::board::BoardInfo::from_map_data(map));
    omdurman_board_ui::board_store::load_board(
        &mut ctx.shared,
        kind,
        &plane,
        &mut meshes,
        &mut materials,
        &asset_server,
    );
}

/// Reconcile the live board with the active view every frame (§dual-map). In a
/// play view (Game) the board follows the scenario's map. Sets
/// [`PendingMapLoad`] when the desired board differs from what's loaded.
pub(crate) fn sync_board_to_game(
    mode: Res<State<crate::AppMode>>,
    game_state: Res<GameStateResource>,
    active: Res<ActiveEditMap>,
    mut pending: ResMut<PendingMapLoad>,
) {
    let desired = match **mode {
        crate::AppMode::Game => Some(crate::map_kind_for_scenario(game_state.0.scenario)),
        crate::AppMode::Menu | crate::AppMode::Lobby => None,
    };
    if let Some(board) = desired
        && board != active.0
        && pending.0.is_none()
    {
        pending.0 = Some(board);
    }
}

/// Registers the board bootstrap: startup RON load + the per-frame
/// load/reconcile systems (§dual-map).
pub struct BoardStatePlugin;

impl Plugin for BoardStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, omdurman_board_ui::board_store::load_annotations)
            .add_systems(
                Update,
                (
                    sync_board_to_game.before(apply_map_selection),
                    apply_map_selection,
                ),
            );
    }
}

//! Game-side board bootstrap (§dual-map).
//!
//! The two-board store, the deferred load request, and the shared loading
//! flow live in `omdurman-board-ui::board_store` (single copies for the game
//! and the map editor); this module keeps the game-specific hooks: seeding
//! the engine's `BoardInfo` on every board load, and loading the
//! sprite-annotation file authored by the map-editor tool.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexOverlay};

use crate::{GameStateResource, sprites::SpriteAnnotationsResource};

pub use omdurman_board_ui::board_store::{
    ActiveEditMap, LoadedAnnotations, MapLoadContext, PendingMapLoad,
};

/// Startup: the shared board seeding plus the sprite-annotation file load
/// (an empty or missing entry simply means the picker falls back to the
/// compiled sprite data).
pub(crate) fn load_annotations_with_sprites(
    mut commands: Commands,
    game_map: ResMut<GameMap>,
    overlay: ResMut<HexOverlay>,
    loaded: ResMut<LoadedAnnotations>,
) {
    let annotations: omdurman_types::SpriteAnnotations =
        ron::de::from_str(include_str!("../assets/sprite_annotations.ron")).unwrap_or_else(|e| {
            bevy::log::error!("failed to parse sprite_annotations.ron: {e}");
            Default::default()
        });
    commands.insert_resource(SpriteAnnotationsResource(annotations));
    omdurman_board_ui::board_store::load_annotations(commands, game_map, overlay, loaded);
}

/// The shared load context plus the game's engine state and annotations.
#[derive(SystemParam)]
pub(crate) struct GameMapLoadContext<'w> {
    pub shared: MapLoadContext<'w>,
    pub game_state: ResMut<'w, GameStateResource>,
    pub annotations: Option<ResMut<'w, SpriteAnnotationsResource>>,
}

/// The game's `apply_map_selection`: attach the engine's view of the board
/// (so map-dependent rules — ZOC across hexsides §5.44, gunboat
/// upstream/downstream §5.24, terrain movement cost §5.11, Friendlies bank
/// §9.14 — are enforced deterministically; carried inside the serialized
/// `GameState`, so replay/late-join reproduce it), then run the shared
/// loading flow.
pub(crate) fn apply_map_selection(
    mut ctx: GameMapLoadContext,
    mut commands: Commands,
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
    ctx.game_state.0.board = omdurman_rules::board::BoardInfo::from_map_data(map);
    if ctx.annotations.is_none() {
        commands.insert_resource(SpriteAnnotationsResource::default());
    }
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
        app.add_systems(Startup, load_annotations_with_sprites)
            .add_systems(
                Update,
                (
                    sync_board_to_game.before(apply_map_selection),
                    apply_map_selection,
                ),
            );
    }
}

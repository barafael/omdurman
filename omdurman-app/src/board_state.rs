//! Game-side board bootstrap (§dual-map).
//!
//! Owns the two-board [`LoadedAnnotations`] store (seeded from the RON data
//! files under `assets/boards/`), the deferred [`PendingMapLoad`] request, and
//! [`apply_map_selection`], which loads a board into the live
//! `GameMap`/`HexOverlay`/`MapDims`/`HexLayout` and re-textures the map plane.
//! The map *editor* lives in `tools/map-editor` and edits these RON files
//! offline; the app only ever reads them.

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout, MapDims, load_map_data};
use omdurman_types::{HexCoord, MapData, MapKind};

use crate::{
    GameStateResource,
    render::{
        HexOverlay, MapPlane, MapTextureCache, PlaneTextureStores, apply_map_data_to_plane,
    },
    sprites::SpriteAnnotationsResource,
};

/// The full two-board annotations store, kept in memory so map switches and
/// replay re-seeds can address either board without re-reading from disk
/// (§dual-map). Seeded from the RON data files at startup.
#[derive(Resource)]
pub struct LoadedAnnotations {
    pub fall_of_khartoum: MapData,
    pub campaign: MapData,
}

impl LoadedAnnotations {
    pub fn map(&self, kind: MapKind) -> &MapData {
        match kind {
            MapKind::FallOfKhartoum => &self.fall_of_khartoum,
            MapKind::Campaign => &self.campaign,
        }
    }

    /// Seed both boards from the embedded RON data (via the rules crate's
    /// board-data accessors, which own the single embedded copy).
    pub fn from_board_ron() -> Self {
        Self {
            fall_of_khartoum: omdurman_rules::board_data::fall_of_khartoum_map_data(),
            campaign: omdurman_rules::board_data::campaign_map_data(),
        }
    }
}

impl Default for LoadedAnnotations {
    fn default() -> Self {
        Self::from_board_ron()
    }
}

/// Which board is currently live in `GameMap`/`HexOverlay`/`MapDims`/
/// `HexLayout` (§dual-map). Local to each peer; switching it reloads the
/// corresponding board via [`PendingMapLoad`].
#[derive(Resource, Default)]
pub struct ActiveEditMap(pub MapKind);

/// A deferred request to (re)load a board into the live `GameMap`/`HexOverlay`/
/// `MapDims`/`HexLayout` and re-texture the map plane. Set by the `StartGame`
/// handler and the board reconciler; consumed by `apply_map_selection`,
/// which has the asset/material access those handlers lack (§dual-map).
#[derive(Resource, Default)]
pub struct PendingMapLoad(pub Option<MapKind>);

/// Startup seeding: load both boards from the embedded RON data, load the
/// default board (Fall-of-Khartoum) into the live map, and load the
/// sprite-annotation file authored by the map-editor tool into the picker's
/// annotation resource.
pub(crate) fn load_annotations(
    mut commands: Commands,
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<HexOverlay>,
    mut loaded: ResMut<LoadedAnnotations>,
) {
    let kind = MapKind::FallOfKhartoum;
    *loaded = LoadedAnnotations::from_board_ron();
    load_map_data(loaded.map(kind), &mut game_map);
    overlay.params = game_map.overlay.clone();
    // The annotation file is authored offline by `tools/map-editor`; an empty
    // or missing entry simply means the picker falls back to the compiled
    // sprite data.
    let annotations: omdurman_types::SpriteAnnotations =
        ron::de::from_str(include_str!("../assets/sprite_annotations.ron"))
            .unwrap_or_else(|e| {
                bevy::log::error!("failed to parse sprite_annotations.ron: {e}");
                Default::default()
            });
    commands.insert_resource(SpriteAnnotationsResource(annotations));
}

/// Bundle of resources mutated when (re)loading a board into the live
/// `GameMap` / overlay / layout / texture (§dual-map). Keeps
/// [`apply_map_selection`] under the system-parameter limit without hiding
/// framework types (`Commands`, `Query`, asset stores) that other systems also
/// depend on.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct MapLoadContext<'w> {
    pub pending: ResMut<'w, PendingMapLoad>,
    pub loaded: Res<'w, LoadedAnnotations>,
    pub active: ResMut<'w, ActiveEditMap>,
    pub game_state: ResMut<'w, GameStateResource>,
    pub game_map: ResMut<'w, GameMap>,
    pub overlay: ResMut<'w, HexOverlay>,
    pub dims: ResMut<'w, MapDims>,
    pub layout: ResMut<'w, HexLayout>,
    pub annotations: Option<ResMut<'w, SpriteAnnotationsResource>>,
    pub cache: ResMut<'w, MapTextureCache>,
}

pub(crate) fn apply_map_selection(
    mut ctx: MapLoadContext,
    mut commands: Commands,
    plane: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>), With<MapPlane>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Some(kind) = ctx.pending.0.take() else {
        return;
    };
    debug!(?kind, "applying PendingMapLoad");
    let map = ctx.loaded.map(kind);

    // Attach the engine's view of this board so map-dependent rules (ZOC across
    // hexsides §5.44, gunboat upstream/downstream §5.24, terrain movement cost
    // §5.11, Friendlies bank §9.14) can be enforced deterministically. Carried
    // inside the serialized GameState, so replay/late-join reproduce it.
    ctx.game_state.0.board = omdurman_rules::board::BoardInfo::from_map_data(map);

    load_map_data(map, &mut ctx.game_map);
    ctx.overlay.params = ctx.game_map.overlay.clone();
    *ctx.dims = MapDims {
        img_w: map.img_w,
        img_h: map.img_h,
    };
    *ctx.layout = HexLayout::calibrated(
        map.overlay.orientation,
        omdurman_hexmap::CalibrationAnchor {
            px: Vec2::new(map.calib.p1_px.0, map.calib.p1_px.1),
            hex: HexCoord::new(map.calib.p1_hex.0, map.calib.p1_hex.1),
        },
        omdurman_hexmap::CalibrationAnchor {
            px: Vec2::new(map.calib.p2_px.0, map.calib.p2_px.1),
            hex: HexCoord::new(map.calib.p2_hex.0, map.calib.p2_hex.1),
        },
        Vec2::new(map.img_w, map.img_h),
    );
    if ctx.annotations.is_none() {
        commands.insert_resource(SpriteAnnotationsResource::default());
    }
    apply_map_data_to_plane(
        &plane,
        &mut PlaneTextureStores {
            meshes: &mut meshes,
            materials: &mut materials,
            cache: &mut ctx.cache,
            asset_server: &asset_server,
        },
        &map.image,
        map.img_w,
        map.img_h,
    );
    ctx.active.0 = kind;
    info!(%kind, img_w = map.img_w, img_h = map.img_h, "loaded board");
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
        app.add_systems(Startup, load_annotations).add_systems(
            Update,
            (
                sync_board_to_game.before(apply_map_selection),
                apply_map_selection,
            ),
        );
    }
}

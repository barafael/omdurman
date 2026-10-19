//! The two-board store + board bootstrap (§dual-map): [`LoadedAnnotations`]
//! (seeded from the RON data the rules crate embeds), the deferred
//! [`PendingMapLoad`] request, [`take_pending`]/[`load_board`] and the map
//! plane, including the calibration block that turns a board's pixel anchors
//! into a [`HexLayout`].

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use omdurman_hexmap::{
    GameMap, HexLayout, HexOverlay, MapDims, MapPlane, MapTextureCache, apply_map_data_to_plane,
    load_map_data,
};
use omdurman_types::{HexCoord, MapData, MapKind};

/// The full two-board annotations store, kept in memory so board switches
/// and replay re-seeds can address either board without re-reading from disk
/// (§dual-map). Seeded from the RON data files at startup.
#[derive(Resource)]
pub struct LoadedAnnotations {
    pub fall_of_khartoum: &'static MapData,
    pub campaign: &'static MapData,
}

impl LoadedAnnotations {
    pub fn map(&self, kind: MapKind) -> &MapData {
        match kind {
            MapKind::FallOfKhartoum => self.fall_of_khartoum,
            MapKind::Campaign => self.campaign,
        }
    }

    /// Seed both boards from the embedded RON data (borrowed from the rules
    /// crate's board-data accessors, which own the single parsed copy).
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
pub struct ActiveBoard(pub MapKind);

/// A deferred request to (re)load a board into the live `GameMap`/`HexOverlay`/
/// `MapDims`/`HexLayout` and re-texture the map plane. Set by scenario start /
/// the board reconcilers; consumed by [`apply_map_selection`], which has the
/// asset/material access those handlers lack (§dual-map).
#[derive(Resource, Default)]
pub struct PendingMapLoad(pub Option<MapKind>);

/// The `HexLayout` calibrated against the Fall-of-Khartoum scan — the default
/// board the game opens with. The anchor numbers derive from the one embedded
/// [`MapData`].
pub fn default_layout() -> HexLayout {
    calibrated_layout(omdurman_rules::board_data::fall_of_khartoum_map_data())
}

/// The `HexLayout` calibrated against a board's pixel anchors (§dual-map):
/// two (`pixel`, `hex`) anchor pairs plus the image size.
fn calibrated_layout(map: &MapData) -> HexLayout {
    HexLayout::calibrated(
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
    )
}

/// Startup seeding: load both boards from the embedded RON data and load the
/// default board (Fall-of-Khartoum) into the live map, including the
/// calibrated [`HexLayout`], [`MapDims`] and [`ActiveBoard`]. (Binaries that need more at
/// startup — e.g. the game's sprite-annotation file — wrap this system.)
pub fn load_annotations(
    mut commands: Commands,
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<HexOverlay>,
    mut loaded: ResMut<LoadedAnnotations>,
) {
    let kind = MapKind::FallOfKhartoum;
    *loaded = LoadedAnnotations::from_board_ron();
    let map = loaded.map(kind).clone();
    load_map_data(&map, &mut game_map);
    overlay.params = game_map.overlay.clone();
    commands.insert_resource(default_layout());
    commands.insert_resource(MapDims {
        img_w: map.img_w,
        img_h: map.img_h,
    });
    // Keep the live-board marker in step with what was just loaded, so a
    // board reconciler doesn't see a stale `ActiveBoard` and misjudge
    // whether a reload is needed.
    commands.insert_resource(ActiveBoard(kind));
}

/// Bundle of resources mutated when (re)loading a board into the live
/// `GameMap` / overlay / layout / texture (§dual-map). Keeps the shared
/// loading flow under the system-parameter limit; binaries embed this struct
/// in their own wider context (the game adds `GameStateResource` and the
/// sprite annotations).
#[derive(SystemParam)]
pub struct MapLoadContext<'w> {
    pub pending: ResMut<'w, PendingMapLoad>,
    pub loaded: Res<'w, LoadedAnnotations>,
    pub active: ResMut<'w, ActiveBoard>,
    pub game_map: ResMut<'w, GameMap>,
    pub overlay: ResMut<'w, HexOverlay>,
    pub dims: ResMut<'w, MapDims>,
    pub layout: ResMut<'w, HexLayout>,
    pub cache: ResMut<'w, MapTextureCache>,
}

/// Take the pending board request, if any. Callers that need app-specific
/// per-board work (attaching `BoardInfo` to the engine state) take the kind
/// first, do their work, then call [`load_board`].
pub fn take_pending(ctx: &mut MapLoadContext) -> Option<MapKind> {
    let kind = ctx.pending.0.take()?;
    debug!(?kind, "applying PendingMapLoad");
    Some(kind)
}

/// Load `kind` into the live `GameMap`/overlay/layout/dims and re-texture the
/// map plane. The shared core of the game's `apply_map_selection`.
pub fn load_board(
    ctx: &mut MapLoadContext,
    kind: MapKind,
    plane: &Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>), With<MapPlane>>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) {
    let map = ctx.loaded.map(kind);

    load_map_data(map, &mut ctx.game_map);
    ctx.overlay.params = ctx.game_map.overlay.clone();
    *ctx.dims = MapDims {
        img_w: map.img_w,
        img_h: map.img_h,
    };
    *ctx.layout = calibrated_layout(map);
    apply_map_data_to_plane(
        plane,
        meshes,
        materials,
        &mut ctx.cache,
        asset_server,
        &map.image,
        map.img_w,
        map.img_h,
    );
    ctx.active.0 = kind;
    info!(%kind, img_w = map.img_w, img_h = map.img_h, "loaded board");
}

/// Spawn the ground plane the board scan is drawn on, preloading both boards'
/// textures so later switches are instant.
pub fn spawn_map_plane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loaded: Res<LoadedAnnotations>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut cache = MapTextureCache::default();
    for kind in [MapKind::FallOfKhartoum, MapKind::Campaign] {
        cache.texture(&asset_server, &loaded.map(kind).image);
    }
    // The plane opens on the default board; take the texture name and the
    // scan's pixel size from the loaded data, not from literals.
    let default_map = loaded.map(MapKind::FallOfKhartoum);
    let texture = cache.texture(&asset_server, &default_map.image);
    commands.insert_resource(cache);
    commands.spawn((
        MapPlane,
        Name::new("MapPlane"),
        Mesh3d(meshes.add(Rectangle::new(default_map.img_w, default_map.img_h))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
    ));
}

//! The two-board store + board bootstrap shared by the game and the editor
//! (§dual-map): [`LoadedAnnotations`] (seeded from the RON data the rules
//! crate embeds), the deferred [`PendingMapLoad`] request,
//! [`apply_map_selection`], the map plane, and the lights. Previously two
//! drifting copies (`omdurman-app/src/board_state.rs` and
//! `tools/map-editor/src/board.rs`) — including the calibration block that
//! turns a board's pixel anchors into a [`HexLayout`].

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use omdurman_hexmap::{
    GameMap, HexLayout, HexOverlay, MapDims, MapPlane, MapTextureCache, PlaneTextureStores,
    apply_map_data_to_plane, load_map_data,
};
use omdurman_types::{HexCoord, MapData, MapKind};

/// Where the board RON data files live (inside the game's assets dir, so the
/// tool edits the canonical files the game loads).
pub fn boards_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("/../../omdurman-app/assets/boards")
}

/// The full two-board annotations store, kept in memory so edits, saves, and
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

    pub fn map_mut(&mut self, kind: MapKind) -> &mut MapData {
        match kind {
            MapKind::FallOfKhartoum => &mut self.fall_of_khartoum,
            MapKind::Campaign => &mut self.campaign,
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
/// `HexLayout` (§dual-map). Local to each peer/tool; switching it reloads the
/// corresponding board via [`PendingMapLoad`].
#[derive(Resource, Default)]
pub struct ActiveEditMap(pub MapKind);

/// A deferred request to (re)load a board into the live `GameMap`/`HexOverlay`/
/// `MapDims`/`HexLayout` and re-texture the map plane. Set by scenario start /
/// the board reconcilers; consumed by [`apply_map_selection`], which has the
/// asset/material access those handlers lack (§dual-map).
#[derive(Resource, Default)]
pub struct PendingMapLoad(pub Option<MapKind>);

/// The `HexLayout` calibrated against the Fall-of-Khartoum scan — the default
/// board both binaries open with. Previously the same anchor numbers were
/// pasted into each `main.rs`; they now derive from the one embedded
/// [`MapData`].
pub fn default_layout() -> HexLayout {
    let map = LoadedAnnotations::from_board_ron()
        .map(MapKind::FallOfKhartoum)
        .clone();
    calibrated_layout(&map)
}

/// The `HexLayout` calibrated against a board's pixel anchors (§dual-map):
/// two (`pixel`, `hex`) anchor pairs plus the image size.
pub fn calibrated_layout(map: &MapData) -> HexLayout {
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
/// calibrated [`HexLayout`] and [`MapDims`]. (Binaries that need more at
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
    pub active: ResMut<'w, ActiveEditMap>,
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
/// map plane. The shared core of both binaries' `apply_map_selection`.
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
        &mut PlaneTextureStores {
            meshes,
            materials,
            cache: &mut ctx.cache,
            asset_server,
        },
        &map.image,
        map.img_w,
        map.img_h,
    );
    ctx.active.0 = kind;
    info!(%kind, img_w = map.img_w, img_h = map.img_h, "loaded board");
}

/// Serialize both boards to the RON data files under
/// `omdurman-app/assets/boards/` -- the tool's save path, and the files the
/// game embeds. Returns a status note for the caller to display.
pub fn save_boards_to_ron(loaded: &LoadedAnnotations) -> String {
    let pretty = ron::ser::PrettyConfig::default();
    let mut note = String::new();
    for (name, board) in [
        ("campaign", &loaded.campaign),
        ("fall_of_khartoum", &loaded.fall_of_khartoum),
    ] {
        let path = boards_dir().join(format!("{name}.ron"));
        match ron::ser::to_string_pretty(board, pretty.clone()) {
            Ok(text) => match std::fs::write(&path, &text) {
                Ok(()) => note.push_str(&format!(
                    "wrote {} bytes to {}\n",
                    text.len(),
                    path.display()
                )),
                Err(e) => note.push_str(&format!("write {} failed: {e}\n", path.display())),
            },
            Err(e) => note.push_str(&format!("serialize {name} failed: {e}\n",)),
        }
    }
    note.trim().to_string()
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
    let texture = cache.texture(&asset_server, "fall_of_khartoum_1885.webp");
    commands.insert_resource(cache);
    commands.spawn((
        MapPlane,
        Name::new("MapPlane"),
        Mesh3d(meshes.add(Rectangle::new(
            omdurman_hexmap::IMG_W,
            omdurman_hexmap::IMG_H,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
    ));
}

/// Key + fill lights for the board view. No standalone `AmbientLight`: since
/// Bevy 0.19 it `#[require(Camera)]`, so a lone ambient entity spawns a
/// phantom camera that never renders — and bevy_egui's auto primary-context
/// system may attach the UI context to it, making the whole UI invisible.
pub fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        Name::new("Light"),
        DirectionalLight {
            illuminance: 9000.0,
            ..default()
        },
    ));
    commands.spawn((
        Name::new("FillLight"),
        DirectionalLight {
            illuminance: 260.0,
            ..default()
        },
    ));
}

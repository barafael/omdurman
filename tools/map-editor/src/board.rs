//! Board store + loading for the map editor (§dual-map).
//!
//! Owns the two-board [`LoadedAnnotations`] store (seeded from the same RON
//! data the game embeds, via `omdurman_rules::board_data`), the deferred
//! [`PendingMapLoad`] request, [`apply_map_selection`], and the RON save path
//! that writes the boards back to `omdurman-app/assets/boards/`.

use bevy::prelude::*;
use omdurman_hexmap::{
    GameMap, HexLayout, HexOverlay, MapDims, MapPlane, MapTextureCache, PlaneTextureStores,
    apply_map_data_to_plane, load_map_data,
};
use omdurman_types::{HexCoord, MapData, MapKind};

/// Where the board RON data files live (inside the game's assets dir, so the
/// tool edits the canonical files the game loads).
pub(crate) fn boards_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("/../../omdurman-app/assets/boards")
}

/// The full two-board annotations store, kept in memory so edits and saves can
/// address either board without re-reading from disk (§dual-map).
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

    /// Seed both boards from the canonical RON data (shared with the game via
    /// the rules crate's embedded copy).
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
/// `HexLayout` (§dual-map). Switching it reloads via [`PendingMapLoad`].
#[derive(Resource, Default)]
pub struct ActiveEditMap(pub MapKind);

/// A deferred request to (re)load a board into the live `GameMap`/`HexOverlay`/
/// `MapDims`/`HexLayout` and re-texture the map plane (§dual-map).
#[derive(Resource, Default)]
pub struct PendingMapLoad(pub Option<MapKind>);

/// Startup seeding: load both boards and load the default (Fall-of-Khartoum)
/// board into the live map.
pub(crate) fn load_annotations(
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<HexOverlay>,
    mut loaded: ResMut<LoadedAnnotations>,
) {
    let kind = MapKind::FallOfKhartoum;
    *loaded = LoadedAnnotations::from_board_ron();
    load_map_data(loaded.map(kind), &mut game_map);
    overlay.params = game_map.overlay.clone();
}

/// Bundle of resources mutated when (re)loading a board into the live
/// `GameMap` / overlay / layout / texture (§dual-map). Keeps
/// [`apply_map_selection`] under the system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct MapLoadContext<'w> {
    pub pending: ResMut<'w, PendingMapLoad>,
    pub loaded: ResMut<'w, LoadedAnnotations>,
    pub active: ResMut<'w, ActiveEditMap>,
    pub game_map: ResMut<'w, GameMap>,
    pub overlay: ResMut<'w, HexOverlay>,
    pub dims: ResMut<'w, MapDims>,
    pub layout: ResMut<'w, HexLayout>,
    pub cache: ResMut<'w, MapTextureCache>,
}

pub(crate) fn apply_map_selection(
    mut ctx: MapLoadContext,
    plane: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>), With<MapPlane>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Some(kind) = ctx.pending.0.take() else {
        return;
    };
    debug!(?kind, "applying PendingMapLoad");
    let map = ctx.loaded.map(kind).clone();

    load_map_data(&map, &mut ctx.game_map);
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

/// Serialize both boards to the RON data files under
/// `omdurman-app/assets/boards/` -- the tool's save path, and the files the
/// game embeds. Returns a status note for the caller to display.
pub(crate) fn save_boards_to_ron(loaded: &LoadedAnnotations) -> String {
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
            Err(e) => note.push_str(&format!("serialize {name} failed: {e}\n")),
        }
    }
    note.trim().to_string()
}

/// Spawn the ground plane the board scan is drawn on, preloading both boards'
/// textures so later switches are instant.
pub(crate) fn spawn_map_plane(
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

/// The board picker's selection (a plain [`MapKind`] -- the tool has no
/// scenario indirection). Local tool state.
#[derive(Resource)]
pub struct EditorBoard(pub MapKind);

impl Default for EditorBoard {
    fn default() -> Self {
        Self(MapKind::FallOfKhartoum)
    }
}

/// Reconcile the live board with the picker every frame (§dual-map): a
/// board-specific tab wants the picker's board loaded; board-agnostic tabs
/// keep whatever is loaded.
pub(crate) fn sync_board_to_tab(
    tab: Res<State<crate::state::EditorTab>>,
    editor_board: Res<EditorBoard>,
    active: Res<ActiveEditMap>,
    mut pending: ResMut<PendingMapLoad>,
) {
    if tab.is_board_specific() && editor_board.0 != active.0 && pending.0.is_none() {
        pending.0 = Some(editor_board.0);
    }
}

/// Minimal ring/outline assets for the editor's hex highlights and debug
/// outlines (a trimmed version of the game's `HexRingAssets`).
#[derive(Resource)]
pub struct RingAssets {
    pub mesh: Handle<Mesh>,
    pub red: Handle<StandardMaterial>,
    pub green: Handle<StandardMaterial>,
    pub light_green: Handle<StandardMaterial>,
}

pub(crate) fn spawn_ring_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    fn unlit(color: Color) -> StandardMaterial {
        StandardMaterial {
            base_color: color,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }
    }
    commands.insert_resource(RingAssets {
        mesh: meshes.add(omdurman_hexmap::hex_ring_mesh()),
        red: materials.add(unlit(Color::srgb(1.0, 0.0, 0.0))),
        green: materials.add(unlit(Color::srgb(0.0, 1.0, 0.0))),
        light_green: materials.add(unlit(Color::srgb(0.6, 1.0, 0.6))),
    });
}

pub fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        Name::new("Light"),
        DirectionalLight {
            illuminance: 9000.0,
            ..default()
        },
    ));
    // No standalone `AmbientLight`: since Bevy 0.19 it `#[require(Camera)]`,
    // so a lone ambient entity spawns a phantom camera that never renders —
    // and bevy_egui's auto primary-context system may attach the UI context
    // to it, making the whole UI invisible (the game's spawn_lights avoids
    // `AmbientLight` for the same reason).
    commands.spawn((
        Name::new("FillLight"),
        DirectionalLight {
            illuminance: 260.0,
            ..default()
        },
    ));
}

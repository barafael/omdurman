//! Board store + loading for the map editor (§dual-map).
//!
//! The two-board [`LoadedAnnotations`] store, the deferred [`PendingMapLoad`]
//! request, and the shared loading flow live in `omdurman-board-ui`
//! (single copies for the game and this tool); this module keeps the
//! editor-specific pieces: the RON save path, the board picker, and the
//! trimmed ring assets.

use bevy::prelude::*;
use omdurman_board_ui::board_store::boards_dir;
use omdurman_hexmap::MapPlane;
use omdurman_types::MapKind;

pub use omdurman_board_ui::board_store::{
    ActiveEditMap, LoadedAnnotations, MapLoadContext, PendingMapLoad, load_annotations,
    spawn_lights, spawn_map_plane,
};

/// The editor's `apply_map_selection`: the shared loading flow in
/// `omdurman-board-ui`, minus the game's engine-state hook.
pub(crate) fn apply_map_selection(
    mut ctx: MapLoadContext,
    plane: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>), With<MapPlane>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Some(kind) = omdurman_board_ui::board_store::take_pending(&mut ctx) else {
        return;
    };
    omdurman_board_ui::board_store::load_board(
        &mut ctx,
        kind,
        &plane,
        &mut meshes,
        &mut materials,
        &asset_server,
    );
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

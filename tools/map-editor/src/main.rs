//! The Omdurman map editor: a native-only Bevy tool for authoring the two
//! boards (terrain, hexsides, roads, overlay calibration, turn track,
//! scattergram, entrance areas), cutting the unit sheet into sprites, and
//! annotating sprite metadata. Edits land in the RON data files under
//! `omdurman-app/assets/` that the game embeds at compile time.

use bevy::{asset::AssetPlugin, prelude::*};
use bevy_egui::EguiPlugin;
use omdurman_hexmap::{HexLayout, HexMapPlugin, HexOverlay};

mod board;
mod browser;
mod camera;
mod edits;
mod editor;
mod overlay;
mod state;
mod ui;
mod ui_plugin;
mod units;
mod util;

use board::{ActiveEditMap, EditorBoard, LoadedAnnotations, PendingMapLoad};

fn main() {
    // The board scans + cut sprites live in the game's assets dir; serve the
    // asset server from there so both binaries see identical files.
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../omdurman-app/assets");

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Omdurman map editor".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: assets.to_string_lossy().into_owned(),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(camera::CameraPlugin)
        .add_plugins(HexMapPlugin)
        // `HexMapPlugin` registers `GameMap`/`MapDims` but not the overlay
        // (the game inserts it in its `RenderPlugin`, which this tool
        // doesn't use), so seed it here for `board::load_annotations`.
        .insert_resource(HexOverlay::default())
        .add_plugins(ui_plugin::UiChromePlugin)
        .add_plugins(editor::EditorPlugin)
        .add_plugins(overlay::OverlayPlugin)
        .insert_resource(browser::SpriteAnnotationsResource(
            browser::load_sprite_annotations(),
        ))
        .insert_resource(browser::SpriteBrowser::new())
        .insert_resource(browser::SpriteMetaClipboard::default())
        .insert_resource(units::UnitViewer::load_or_default())
        .insert_resource(LoadedAnnotations::default())
        .insert_resource(ActiveEditMap::default())
        .insert_resource(EditorBoard::default())
        .insert_resource(PendingMapLoad::default())
        .insert_resource(HexLayout::calibrated(
            omdurman_types::Orientation::Pointy,
            omdurman_hexmap::CalibrationAnchor {
                px: Vec2::new(736.0, 420.0),
                hex: omdurman_types::HexCoord::new(0, 0),
            },
            omdurman_hexmap::CalibrationAnchor {
                px: Vec2::new(1178.0, 572.0),
                hex: omdurman_types::HexCoord::new(5, -1),
            },
            Vec2::new(1571.0, 1200.0),
        ))
        .init_state::<state::EditorTab>()
        .add_systems(
            Startup,
            (
                board::spawn_lights,
                board::spawn_map_plane,
                board::spawn_ring_assets,
                units::spawn_units_plane,
                browser::spawn_sprite_browser,
            ),
        )
        .add_systems(
            Update,
            (
                units::draw_unit_grids,
                browser::scroll_sprite_browser,
                browser::handle_sprite_clicks,
                browser::update_sprite_selection_marker,
                browser::navigate_sprite_selection,
            ),
        )
        .add_systems(
            bevy_egui::EguiPrimaryContextPass,
            (
                units::unit_grids_ui,
                units::unit_grid_labels,
                browser::sprite_meta_editor_ui,
            ),
        )
        .run();
}

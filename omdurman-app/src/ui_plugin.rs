use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

/// Registers UI-domain systems: egui panels (units browser, event viewer,
/// dice simulator, settings, lobby, mode toolbar, cursor overlay) and their
/// associated update systems (camera control, status text, browser nav).
/// Does NOT own any resources — those are registered by their domain plugins
/// (EditorPlugin, GamePlugin, etc.) or in the root setup.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        use crate::{
            browser, dice, event_viewer, lobby, settings, units,
        };

        app
            // ── Resources ──────────────────────────────────────────────
            .insert_resource(settings::SettingsOverlay::default())
            .insert_resource(settings::LocalPlayerSettings::default())
            .insert_resource(settings::PlayerInfoMap::default())
            .insert_resource(units::UnitViewer::load_or_default())
            .insert_resource(browser::SpriteBrowser::new())
            .insert_resource(browser::SpriteMetaClipboard::default())
            .insert_resource(dice::DiceSimulator::default())
            .insert_resource(event_viewer::EventViewerState::default())
            // ── Startup ────────────────────────────────────────────────
            .add_systems(Startup, (
                crate::setup_ui,
                crate::configure_egui_touch,
                crate::maximize_primary_window,
                units::spawn_units_plane,
                browser::spawn_sprite_browser,
            ))
            // ── Update ─────────────────────────────────────────────────
            .add_systems(Update, (
                crate::setup_egui_fonts,
                crate::camera_control,
                crate::update_status_text,
                crate::update_hex_coord_display,
                units::draw_unit_grids,
                browser::scroll_sprite_browser,
                browser::handle_sprite_clicks,
                browser::update_sprite_selection_marker,
                browser::navigate_sprite_selection,
            ))
            // ── Egui panels ────────────────────────────────────────────
            .add_systems(EguiPrimaryContextPass, (
                crate::cursor_overlay_ui,
                crate::mode_toolbar,
                units::unit_grids_ui,
                units::unit_grid_labels,
                browser::sprite_meta_editor_ui,
                dice::dice_sim_ui,
                event_viewer::event_viewer_ui,
                settings::settings_ui,
                lobby::lobby_ui,
            ));
    }
}

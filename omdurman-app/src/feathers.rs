//! Feathers, Bevy's editor-style widget set, for the app's dev tooling
//! overlays (the title screen's tuning pane, `splash::tuning`), on its dark
//! theme.
//!
//! Feathers manages the window cursor: over one of its widgets the widget's
//! cursor, elsewhere a default. egui sets the window cursor too, but only when
//! its wish changes, so the two would fight (Feathers would reset the I-beam
//! over an egui text field the frame after egui set it). egui's own cursor
//! writes are switched off instead, and its wish becomes Feathers' default.

use bevy::feathers::{
    FeathersPlugins,
    cursor::{DefaultCursor, EntityCursor},
    dark_theme::create_dark_theme,
    theme::UiTheme,
};
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use bevy_egui::{
    EguiGlobalSettings, EguiOutput, EguiPostUpdateSet, PrimaryEguiContext,
    helpers::egui_to_winit_cursor_icon,
};

pub struct FeathersPlugin;

impl Plugin for FeathersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FeathersPlugins)
            .insert_resource(UiTheme(create_dark_theme()))
            .add_systems(
                PostUpdate,
                egui_cursor_to_feathers.after(EguiPostUpdateSet::ProcessOutput),
            );
    }

    fn finish(&self, app: &mut App) {
        // (In `finish`: the egui plugin, which inserts the settings, may be
        // added after this one.)
        if let Some(mut settings) = app.world_mut().get_resource_mut::<EguiGlobalSettings>() {
            settings.enable_cursor_icon_updates = false;
        }
    }
}

/// Hand egui's cursor wish (from its last pass) to Feathers as the cursor
/// away from its widgets.
fn egui_cursor_to_feathers(
    outputs: Query<&EguiOutput, With<PrimaryEguiContext>>,
    mut default: ResMut<DefaultCursor>,
) {
    let Ok(output) = outputs.single() else {
        return;
    };
    let icon = egui_to_winit_cursor_icon(output.platform_output.cursor_icon)
        .unwrap_or(SystemCursorIcon::Default);
    let wanted = EntityCursor::System(icon);
    if default.0 != wanted {
        default.0 = wanted;
    }
}

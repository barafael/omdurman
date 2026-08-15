//! Small shared UI helpers (panel backgrounds, section headers). Kept minimal
//! so the rest of the app pulls common chrome from one place.

use bevy::prelude::{Commands, Entity};
use bevy_egui::egui;

/// Despawn every entity in `entities` via deferred commands. Used by the
/// overlay systems that rebuild their meshes from scratch each change (despawn
/// all, then respawn). Centralising the loop here leaves a single seam for the
/// eventual pool-ification of these overlays.
pub fn despawn_all(commands: &mut Commands, entities: &[Entity]) {
    for &e in entities {
        commands.entity(e).despawn();
    }
}

/// Standard side-panel background colour used across the app.
/// Warm charcoal rather than neutral gray, at the default's luminance, so the
/// chrome sits with the game's sepia palette without costing contrast.
pub fn panel_bg() -> egui::Color32 {
    egui::Color32::from_rgb(44, 40, 35)
}

/// A bold section title followed by a separator. Shared by the side-panel
/// sections (overview, actions) so they read as one panel.
pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .font(egui::FontId::new(
                17.0,
                egui::FontFamily::Name("Garamond".into()),
            ))
            .color(egui::Color32::from_rgb(218, 204, 173)),
    );
    ui.separator();
    ui.add_space(4.0);
}

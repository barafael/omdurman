//! Small shared UI helpers (panel backgrounds, despawn loops).

use bevy::prelude::{Commands, Entity};
use bevy_egui::egui;

/// Despawn every entity in `entities` via deferred commands. Used by the
/// overlay systems that rebuild their meshes from scratch each change (despawn
/// all, then respawn).
pub fn despawn_all(commands: &mut Commands, entities: &[Entity]) {
    for &e in entities {
        commands.entity(e).despawn();
    }
}

/// Standard side-panel background colour (matches the game's chrome).
pub fn panel_bg() -> egui::Color32 {
    egui::Color32::from_rgb(44, 40, 35)
}

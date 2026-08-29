//! Dev-only Bevy world inspector (egui). Compiled to a no-op unless the
//! `dev` cargo feature is enabled, so release and wasm builds carry neither
//! the code nor the dependency.
//!
//! Run with: `cargo run -p omdurman-app --features dev`

use bevy::prelude::*;

pub struct DevInspectorPlugin;

impl Plugin for DevInspectorPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "dev")]
        {
            // Only respond to the local keyboard/mouse (skip in multiplayer
            // windows where the map must stay interactive).
            app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
        }
        #[cfg(not(feature = "dev"))]
        {
            let _ = app;
        }
    }
}

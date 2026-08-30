//! egui chrome plumbing shared with the game: pointer gating
//! ([`egui_wants_pointer_input`] and the snapshot condition it feeds) comes
//! from `omdurman-board-ui`; this module adds the editor's sidebar clip.

use bevy::prelude::*;
use bevy_egui::egui;

pub use omdurman_board_ui::panels::{
    EguiPointerOverUi, PanelUiSet, egui_wants_pointer_input, sync_egui_pointer_over_ui,
};

/// The editor's right-sidebar rect, recorded so overlay placement can avoid
/// it. (The game sizes its sidebar via egui panels directly.)
#[derive(Resource, Default)]
pub struct SidebarClip {
    pub right_sidebar: Option<egui::Rect>,
}

pub struct UiChromePlugin;

impl Plugin for UiChromePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SidebarClip::default())
            .init_resource::<EguiPointerOverUi>()
            .add_systems(First, sync_egui_pointer_over_ui);
    }
}

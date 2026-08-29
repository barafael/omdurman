//! egui chrome plumbing shared by the tool's panels: the per-frame panel-rect
//! registry (so map click handlers can reject clicks over UI) and the sidebar
//! clip.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

#[derive(Resource, Default)]
pub struct SidebarClip {
    pub right_sidebar: Option<egui::Rect>,
}

/// Set for every egui system that renders a `Panel`/`CentralPanel` into a
/// hand-built background `Ui`. `clear_egui_panel_rects` is ordered before it
/// so each frame's panel-rect registry starts empty.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PanelUiSet;

/// egui-data key under which the current frame's panel screen rects are
/// collected (see [`register_panel_rect`]).
fn egui_panel_rects_id() -> egui::Id {
    egui::Id::new("egui_panel_rects")
}

/// Record an egui panel's screen-space rect (the panel's full outer rect,
/// `Panel::show(...).response.rect`) into the frame's registry.
///
/// The panels in this tool are shown inside hand-built `Ui`s on
/// [`egui::LayerId::background()`] rather than the pass's root `Ui`, so
/// `egui::Context::is_pointer_over_egui()` never sees them -- it consults the
/// root UI's *available* rect, which these panels don't shrink. Pointer-driven
/// systems (camera zoom/pan, click handlers) consult this registry instead so
/// input over a sidebar isn't mistaken for input over the map.
pub fn register_panel_rect(ctx: &egui::Context, rect: egui::Rect) {
    if !rect.is_positive() {
        return;
    }
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Vec<egui::Rect>>(egui_panel_rects_id())
            .push(rect);
    });
}

/// Whether the pointer is over any egui panel recorded this/last frame.
pub fn egui_panel_pointer(ctx: &egui::Context) -> bool {
    let Some(pos) = ctx.pointer_latest_pos() else {
        return false;
    };
    ctx.data(|d| {
        d.get_temp::<Vec<egui::Rect>>(egui_panel_rects_id())
            .is_some_and(|rects| rects.iter().any(|r| r.contains(pos)))
    })
}

/// Like [`egui::Context::egui_wants_pointer_input`], but also true when the
/// pointer is over one of the tool's background-layer panels (which the egui
/// API alone cannot detect here -- see [`register_panel_rect`]).
pub fn egui_wants_pointer_input(ctx: &egui::Context) -> bool {
    ctx.egui_wants_pointer_input() || egui_panel_pointer(ctx)
}

/// Empties the per-frame panel-rect registry. Runs at the start of the egui
/// pass, before any [`PanelUiSet`] system.
pub fn clear_egui_panel_rects(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ctx.data_mut(|d| d.insert_temp(egui_panel_rects_id(), Vec::<egui::Rect>::new()));
}

pub struct UiChromePlugin;

impl Plugin for UiChromePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SidebarClip::default()).add_systems(
            EguiPrimaryContextPass,
            clear_egui_panel_rects.before(PanelUiSet),
        );
    }
}

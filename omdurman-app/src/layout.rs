//! Per-frame chrome layout ledger .
//!
//! Every egui surface used to pin itself with hardcoded pixel offsets from
//! the window edges, so two left panels drew at the same x and the top-center
//! cards (phase banner, fire/melee previews, prompts, badges) stacked at
//! overlapping y values. This resource is the fix: each frame the chrome
//! *reserves* its bands and every surface reads the edges instead of
//! hardcoding.
//!
//! Per frame (reset in `First`, before the egui pass):
//!
//! * the top bar publishes its measured height ([`Self::top_bar_height`]);
//! * left rail panels chain side by side, advancing [`Self::left_inset`];
//! * the charts sheet publishes its width as [`Self::right_inset`];
//! * top-center cards stack downward from [`Self::center_stack_y`], which
//!   starts just below the top bar.
//!
//! Systems that consume an inset must run after the system that produces it
//! (see the ordering constraints at the registration sites).

use bevy::prelude::*;
use bevy_egui::egui;

/// Resting height of the top bar. Surfaces that draw before the bar is
/// measured (or when it is hidden) start below this; the bar publishes its
/// real height over it every frame it is shown.
pub const TOP_BAR_HEIGHT: f32 = 36.0;

/// Vertical gap between stacked top-center cards.
pub const STACK_GAP: f32 = 8.0;

/// The per-frame layout ledger. Reset every frame by
/// [`reset_screen_layout`]; see the module docs for the protocol.
#[derive(Resource, Debug)]
pub struct ScreenLayout {
    /// x where the next left-rail panel may start. Grows as rail panels
    /// chain side by side; reset to the window edge each frame.
    pub left_inset: f32,
    /// Measured top-bar height; bands below the bar start here.
    pub top_bar_height: f32,
    /// y where the next top-center card is anchored; starts just below the
    /// top bar and grows as stacked cards are drawn.
    pub center_stack_y: f32,
    /// Width reserved at the right edge (charts sheet open, or its peek
    /// tab). Right-anchored cards shift left by this much.
    pub right_inset: f32,
}

impl Default for ScreenLayout {
    fn default() -> Self {
        Self {
            left_inset: 0.0,
            top_bar_height: TOP_BAR_HEIGHT,
            center_stack_y: TOP_BAR_HEIGHT + STACK_GAP,
            right_inset: 0.0,
        }
    }
}

/// Reset the ledger at the start of the frame (runs in `First`, before the
/// egui pass).
pub fn reset_screen_layout(mut layout: ResMut<ScreenLayout>) {
    *layout = ScreenLayout::default();
}

/// Show one left-rail panel: a background-layer root `Ui` whose rect starts
/// at the current [`ScreenLayout::left_inset`] (and below the top bar), the
/// panel's blocker registered before its content (see
/// `omdurman_board_ui::panels::register_panel_blocker`), and — after
/// `show_panel` returns the panel's outer rect — the rail inset advanced so
/// the next rail panel chains beside it instead of superimposing.
///
/// `show_panel` draws the `egui::Panel` itself (frames/resizability are
/// caller-owned) and returns its outer rect.
pub fn left_rail_panel(
    ctx: &egui::Context,
    layout: &mut ScreenLayout,
    root_id: &str,
    panel_id: &str,
    fallback_width: f32,
    show_panel: impl FnOnce(&mut egui::Ui) -> egui::Rect,
) {
    let vp = ctx.viewport_rect();
    let x0 = layout.left_inset;
    let origin = egui::pos2(vp.min.x + x0, vp.min.y + layout.top_bar_height);
    let mut ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new(root_id),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(egui::Rect::from_min_max(origin, vp.max)),
    );
    omdurman_board_ui::panels::register_panel_blocker(
        &mut ui,
        panel_id,
        egui::Rect::from_min_size(origin, egui::vec2(fallback_width, vp.max.y - origin.y)),
    );
    let outer = show_panel(&mut ui);
    layout.left_inset = outer.max.x;
}

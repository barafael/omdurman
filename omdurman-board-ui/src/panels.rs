//! egui pointer-gating shared by the game and the editor. Both binaries
//! render panels inside hand-built `Ui`s on [`egui::LayerId::background()`],
//! where `egui::Context::is_pointer_over_egui()` never fires — this module
//! provides the working predicate plus the declarative `SystemSet` gate built
//! on it. (Earlier revisions kept a side-channel panel-rect registry; the
//! egui-native hit-testing below replaced it.)

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

/// Set for map-interaction systems that must not fire while the pointer is
/// over UI. The whole set is configured with `run_if(not(ui_wants_pointer))`,
/// so members are gated declaratively -- a new click handler joins the set
/// instead of remembering to hand-roll an egui check. Systems that must keep
/// observing input mid-gesture (e.g. camera drag release) stay ungated and
/// check [`egui_wants_pointer_input`] inline instead.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapPointerInputSet;

/// Set for every egui system that renders a `Panel`/`CentralPanel` into a
/// hand-built background `Ui`. Membership documents the panel systems; the
/// panels self-register with egui's hit-testing (a full-rect, click-sensed
/// blocker per panel), so pointer interest over them needs no side-channel.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PanelUiSet;

/// Per-frame snapshot: does egui claim the pointer (see
/// [`egui_wants_pointer_input`])? Written once per frame in `First` from the
/// last completed pass; `Update` consumers and the [`ui_wants_pointer`]
/// condition read it without extra plumbing.
#[derive(Resource, Default)]
pub struct EguiPointerOverUi(pub bool);

/// Whether the pointer is over one of the app's egui surfaces.
///
/// This deliberately does NOT use [`egui::Context::egui_wants_pointer_input`]:
/// in egui 0.35 that predicate consults the pass's `root_ui_available_rect`,
/// which is only set by the `run_ui` path. bevy_egui runs the primary context
/// in single-pass mode (`begin_pass`), where it is never set -- and the
/// fallback is `true`, making the predicate *always true* and any gate built
/// on it a total input blackout (found by the `ui_gating` tests).
///
/// Instead we hit-test the pointer against
/// [`egui::Context::interactive_rects_last_pass`] -- egui's own "rects that
/// could receive pointer input" (widgets, our panel blockers, tooltips),
/// stable across single- and multi-pass modes.
pub fn egui_wants_pointer_input(ctx: &egui::Context) -> bool {
    ctx.egui_is_using_pointer()
        || ctx.pointer_latest_pos().is_some_and(|pos| {
            ctx.interactive_rects_last_pass()
                .iter()
                .any(|rect| rect.contains(pos))
        })
}

/// Refresh [`EguiPointerOverUi`] for this frame. Runs in `First`, so the
/// snapshot is stable for every `Update` consumer regardless of system order.
pub fn sync_egui_pointer_over_ui(mut contexts: EguiContexts, mut over: ResMut<EguiPointerOverUi>) {
    over.0 = contexts
        .ctx_mut()
        .ok()
        .map(|ctx| egui_wants_pointer_input(&ctx))
        .unwrap_or(false);
}

/// Resource-only run condition for the [`MapPointerInputSet`] gate.
pub fn ui_wants_pointer(over: Res<EguiPointerOverUi>) -> bool {
    over.0
}

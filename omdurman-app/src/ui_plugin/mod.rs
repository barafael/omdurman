//! Built-in UI chrome: the plugin wiring plus the panel submodules.
//!
//! Map-input gating (pointer predicate, `EguiPointerOverUi`, sets) is
//! designed once in `omdurman-board-ui::panels` and re-exported here so
//! `crate::ui_plugin::*` paths keep working.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use bevy_matchbox::prelude::PeerId;
use omdurman_net::NetState;
use std::borrow::Cow;

use crate::peers::{LocalPeer, Peers};
use crate::{AppState, HoveredHex, RoomId, camera::RtsCamera, settings};

// -- Map-input gating ----------------------------------------------------------
//
// The pointer predicate, its per-frame snapshot, the [`MapPointerInputSet`] /
// [`PanelUiSet`] sets, and the `ui_wants_pointer` run condition live in
// `omdurman-board-ui::panels`; the names are re-exported here so
// `crate::ui_plugin::*` paths keep working.

pub use omdurman_board_ui::panels::{
    CarryingDragToBoard, EguiPointerOverUi, MapPointerInputSet, PanelUiSet,
    sync_egui_pointer_over_ui, ui_wants_pointer,
};
// (Directly referenced only by the ui_gating tests in non-inline paths.)
#[cfg_attr(not(test), allow(unused_imports))]
pub use omdurman_board_ui::panels::egui_wants_pointer_input;

/// Schedule set grouping the left-rail panel systems (unit picker, unit
/// overview) so consumers (the game log) can order against the rail without
/// naming a system that is registered more than once (the overview runs in
/// both the InGame and Spectating states, and Bevy refuses to order against
/// an ambiguous system type).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LeftRailSet;

mod controls;
mod cursor_overlay;
mod fonts;
mod gazette;
mod special_actions;
mod status;
mod toolbar;
mod victory;

pub(crate) use controls::*;
pub(crate) use cursor_overlay::*;
pub(crate) use fonts::*;
pub(crate) use special_actions::*;
pub(crate) use status::*;
pub(crate) use toolbar::*;
pub(crate) use victory::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        use crate::{event_viewer, lobby};

        app.insert_resource(settings::LocalPlayerSettings::default())
            .insert_resource(event_viewer::EventViewerState::default())
            .insert_resource(FontsInstalled::default())
            .insert_resource(EguiPointerOverUi::default())
            .init_resource::<CarryingDragToBoard>()
            .init_resource::<crate::hotkeys::EguiKeyboardFocus>()
            .init_resource::<VictoryModalState>()
            .init_resource::<crate::ScreenLayout>()
            // Whole-set gate: map-interaction systems in this set are skipped
            // whenever the pointer is over UI (see `MapPointerInputSet`).
            .configure_sets(Update, MapPointerInputSet.run_if(not(ui_wants_pointer)))
            // Clear the chrome layout ledger before any egui surface draws.
            .add_systems(
                First,
                (
                    sync_egui_pointer_over_ui,
                    crate::hotkeys::sync_egui_keyboard_focus,
                    crate::layout::reset_screen_layout,
                ),
            )
            .add_systems(
                Startup,
                (setup_ui, configure_egui_touch, maximize_primary_window),
            )
            // After this frame's egui pass has filled the layout ledger and
            // before `First` resets it (see `status::inset_bottom_panes`).
            .add_systems(Last, status::inset_bottom_panes)
            // After this frame's sidebar may have started (or ended) a
            // drag-and-drop placement, before next frame's pointer gate.
            .add_systems(Last, track_drag_carry)
            .add_systems(
                Update,
                (
                    // Retries until the egui context exists (a camera is up),
                    // then installs fonts exactly once.
                    setup_egui_fonts,
                    update_status_text,
                    update_hex_coord_display,
                    crate::scenario_setup::auto_trigger_scenario_setup
                        .run_if(bevy::prelude::in_state(crate::AppState::InGame)),
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    mode_toolbar_ui.run_if(not(bevy::prelude::in_state(crate::AppMode::Menu))),
                    cursor_overlay_ui.run_if(crate::map_view_active),
                    // (ZOC/LOS toggles live in the left rail's Overlays
                    // section -- see overview::unit_overview_ui.)
                    // In-game HUD/overlays: only while actually in a game, so
                    // they don't show over the lobby. The top-center cards
                    // stack below the top bar (see `stacked_card`), so
                    // they chain in that order and must run after it; the
                    // game log reads the left-rail inset, so it runs after
                    // the rail panels.
                    (
                        // Fire preview in a fire sub-phase (offensive *or*
                        // defensive, §6.41/§6.42) ...
                        crate::fire::fire_combat_preview_ui.run_if(
                            crate::ui_phase_state::in_defensive_fire_phase
                                .or_else(crate::ui_phase_state::in_offensive_fire_phase),
                        ),
                        // §6.63 artillery breach: fire-phase sibling of the
                        // Movement-phase special-actions card — one battery,
                        // the wall hexsides it can reach.
                        artillery_breach_ui.run_if(
                            crate::ui_phase_state::in_defensive_fire_phase
                                .or_else(crate::ui_phase_state::in_offensive_fire_phase),
                        ),
                        // ... melee preview in Melee (§7) ...
                        crate::melee::melee_combat_preview_ui
                            .run_if(crate::ui_phase_state::in_melee_phase), // Movement-phase action panels (§5.21 transport,
                        // §5.3 zariba / §6.53 demolition). The mirror state
                        // machine (see `ui_phase_state`) gates the phase;
                        // selection/eligibility stays inside.
                        (friendlies_transport_ui, special_actions_ui)
                            .chain()
                            .run_if(crate::ui_phase_state::in_movement_phase),
                        crate::fok_panel::gordon_badge_ui,
                    )
                        .chain()
                        .after(mode_toolbar_ui)
                        .run_if(crate::in_game_view),
                    // (`in_game_view`, not just `InGame`: the menu is shown
                    // with `AppState::InGame`, and must not have the in-game
                    // HUD drawn over it.)
                    // Live game *and* the spectator review (a finished
                    // record ends on the result).
                    victory_modal.run_if(crate::board_view_active),
                    telegram_overlay.run_if(crate::in_game_view),
                    // (Not a run condition: its not-in-Setup branch clears the
                    // staged mine/chain placement on the transition out of
                    // §10 setup -- cleanup a `run_if` would skip.)
                    optional_rule_setup_ui
                        .run_if(crate::in_game_view)
                        .after(crate::charts::chart_sheet_ui),
                    event_viewer::event_viewer_ui
                        .run_if(in_state(AppState::InGame).or_else(in_state(AppState::Spectating))),
                    event_viewer::event_viewer_toggle
                        .run_if(in_state(AppState::InGame).or_else(in_state(AppState::Spectating)))
                        .run_if(crate::hotkeys::keyboard_free),
                    lobby::lobby_ui
                        .in_set(PanelUiSet)
                        .run_if(in_state(AppState::Lobby)),
                ),
            );
    }
}

/// Keep [`CarryingDragToBoard`] in step with the picker: set while a counter
/// dragged out of the sidebar is in hand, so its drop reaches the board.
fn track_drag_carry(
    picker: Option<Res<crate::picker::PickerState>>,
    mut carrying: ResMut<CarryingDragToBoard>,
) {
    let now = picker.is_some_and(|p| {
        matches!(
            *p,
            crate::picker::PickerState::Placing {
                drag_drop: true,
                ..
            }
        )
    });
    if carrying.0 != now {
        carrying.0 = now;
    }
}

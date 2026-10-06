//! Unit picker: the left sidebar of available counters, placement preview,
//! click handling for place / select / move, and movement animation.
//!
//! Placement and movement are *requested* here by broadcasting
//! [`GameEvent::PlaceUnit`] / [`GameEvent::MoveUnit`]; the authoritative state
//! change happens when the host-sequenced echo is applied to the rules engine
//! (`game_apply::apply_game_event`, the one path shared by live play and
//! history replay). The board counters are a projection of the engine state
//! ([`reconcile_unit_sprites`]).

use bevy::app::Plugin;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::{GameMap, HexLayout};
use omdurman_types::{HexCoord, HexsideRef, Scenario, SectionName, Terrain};

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::AppState;
use crate::camera::RtsCamera;
use crate::events;
use crate::render::{HexOverlay, HexRingAssets};
use omdurman_hexmap::{hex_world_pos, hit_to_hex};
use omdurman_net::GameEvent;
use omdurman_rules::effects::FokCapGroup;
use omdurman_rules::{MovementPoints, UnitId, UnitPlacement, UnitState, unit_id_for_section_pos};

mod clicks;
mod movement;
mod overlays;
mod placed;
mod sidebar;
mod state;

pub use clicks::*;
pub(crate) use movement::*;
pub use overlays::*;
pub use placed::*;
pub use sidebar::*;
pub use state::*;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/sprites.rs"));
    include!(concat!(env!("OUT_DIR"), "/sprite_bytes.rs"));
}

pub struct GamePlugin;

/// The asset path of counter sprite `name` (`<section>_<col>_<row>`): the
/// sprites are baked into the binary and served by the `sprite://` asset
/// source (see [`register_sprite_source`]), not read from `assets/sprites/`.
pub(crate) fn sprite_asset_path(name: &str) -> String {
    format!("sprite://{name}.webp")
}

/// Register the `sprite://` asset source: every counter sprite, baked into
/// the binary from the same build-script scan as the sprite index, served
/// from memory. The game never depends on finding `assets/sprites/` beside
/// the binary (or, on the web, on 240 separate fetches). Must run before
/// `AssetPlugin` (i.e. before `DefaultPlugins`) is added.
pub fn register_sprite_source(app: &mut App) {
    use bevy::asset::io::AssetSourceBuilder;
    use bevy::asset::io::memory::{Dir, MemoryAssetReader};
    let root = Dir::default();
    for &(name, bytes) in generated::SPRITE_BYTES {
        root.insert_asset(std::path::Path::new(&format!("{name}.webp")), bytes);
    }
    app.register_asset_source(
        "sprite",
        AssetSourceBuilder::new(move || Box::new(MemoryAssetReader { root: root.clone() })),
    );
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // -- Resources ----------------------------------------------
            .insert_resource(UnitPicker::default())
            .insert_resource(PickerState::default())
            .insert_resource(MovementPath::default())
            .insert_resource(UnitPaths::default())
            .insert_resource(crate::zoc::ZocOverlay::default())
            .init_resource::<OverlayGeneration>()
            // The board click router + its messages (incl. `PickerCommand`).
            .add_plugins(crate::board_click::BoardClickPlugin)
            // -- Mode-exit cleanup: leaving a play view (or the game itself)
            //    despawns all gameplay overlay rings, so none linger over the
            //    editor / lobby (the per-frame overlay systems only clean up
            //    while running).
            .add_systems(OnExit(crate::AppMode::Game), clear_gameplay_overlays)
            .add_systems(OnExit(AppState::InGame), clear_gameplay_overlays)
            .add_systems(OnExit(AppState::Spectating), clear_gameplay_overlays)
            // -- Startup ------------------------------------------------
            .add_systems(
                Startup,
                (
                    spawn_picker_assets,
                    crate::render::spawn_movement_arrow_assets,
                ),
            )
            // -- Update: gameplay (GameSet) -----------------------------
            .add_systems(
                Update,
                (
                    // The board counters follow the engine state in every
                    // app state (live, replay, spectating), right after the
                    // frame's events were applied / the scrub rebuilt.
                    reconcile_unit_sprites
                        .after(crate::net_socket::handle_reconnect)
                        .after(crate::timeline::scrub_rebuild)
                        .after(crate::events::forward_local_actions)
                        .before(animate_unit_movement),
                    (
                        placement_preview_mesh.in_set(crate::GameSet),
                        // Board click consumers: each reads the one mode
                        // message `board_click::route_board_clicks` emits for
                        // a click (the router owns phase / seat / mode
                        // arbitration), so none is ordered against another.
                        crate::fire_allocation::handle_fire_allocation_click
                            .in_set(crate::GameSet)
                            .in_set(crate::board_click::BoardClickHandlerSet),
                        crate::melee::handle_melee_combat
                            .in_set(crate::GameSet)
                            .in_set(crate::board_click::BoardClickHandlerSet),
                        crate::melee::handle_advance_after_combat
                            .in_set(crate::GameSet)
                            .in_set(crate::board_click::BoardClickHandlerSet)
                            .after(crate::fire_allocation::execute_fire_allocations),
                        crate::retreat::handle_retreat
                            .in_set(crate::GameSet)
                            .in_set(crate::board_click::BoardClickHandlerSet),
                        handle_picker_clicks
                            .in_set(crate::GameSet)
                            .in_set(crate::board_click::BoardClickHandlerSet),
                        movement_overlay_mesh.in_set(crate::GameSet),
                        crate::fire::fire_target_overlay_mesh.in_set(crate::GameSet),
                        crate::melee::melee_target_overlay_mesh.in_set(crate::GameSet),
                        crate::retreat::retreat_overlay_mesh.in_set(crate::GameSet),
                        clear_paths_on_turn_change,
                        movement_path_arrows
                            .in_set(crate::GameSet)
                            .after(clear_paths_on_turn_change)
                            .after(reconcile_unit_sprites),
                        deployment_zone_overlay_mesh.in_set(crate::GameSet),
                        crate::fire_allocation::reset_fire_allocation_on_phase_change,
                        // (ZOC + LOS overlays are scheduled in main.rs for
                        // both the live game and the spectator view.)
                        animate_unit_movement,
                        layout_stacked_units
                            .after(animate_unit_movement)
                            .run_if(in_state(crate::AppMode::Game)),
                        // Right-click → Cancel comes from the click router
                        // (ordered before this); the handler itself is not
                        // pointer-gated, so the actions-panel Cancel button
                        // (over UI) reaches it.
                        cancel_placement.in_set(crate::GameSet),
                    ),
                ),
            )
            // -- Selection outline + cursor-hex tint + hover square (separate
            //     block: the big GameSet tuple is at Bevy's schedule-config
            //     arity limit) -----------------------------------------------
            .add_systems(
                Update,
                (
                    selection_outline_mesh.in_set(crate::GameSet),
                    placement_marker_color
                        .in_set(crate::GameSet)
                        .after(placement_preview_mesh),
                    update_hovered_unit
                        .in_set(crate::GameSet)
                        .before(hover_outline_mesh),
                    hover_outline_mesh.in_set(crate::GameSet),
                    // Persistent red arrows per pending fire allocation
                    // (§6.41 allocation preview). Kept here so the big GameSet
                    // tuple stays under Bevy's schedule-config arity limit.
                    crate::fire_allocation::fire_allocation_arrows.in_set(crate::GameSet),
                    // A selection is phase-shaped; never carry it across a
                    // phase boundary (see `select_combat_tile`).
                    reset_selection_on_phase_change.in_set(crate::GameSet),
                    // Counters turning over / popping in (any board view).
                    animate_counter_changes.after(reconcile_unit_sprites),
                ),
            )
            // -- Execute fire allocations (separate block to stay under Bevy's
            //     tuple size limit for schedule configs) --------------------
            .add_systems(
                Update,
                crate::fire_allocation::execute_fire_allocations
                    .in_set(crate::GameSet)
                    .after(crate::fire_allocation::handle_fire_allocation_click)
                    .before(handle_picker_clicks),
            )
            // -- Path annotation + fire/melee direction systems ---------------
            .add_systems(
                Update,
                (
                    clear_movement_path_when_idle
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    // Enter / Backspace / Del / Esc -> PickerCommand, never
                    // while typing into an egui field.
                    crate::hotkeys::picker_hotkeys
                        .in_set(crate::GameSet)
                        .run_if(crate::hotkeys::keyboard_free)
                        .before(confirm_movement_path)
                        .before(undo_movement_leg)
                        .before(delete_selected_unit)
                        .before(cancel_placement),
                    confirm_movement_path
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    undo_movement_leg
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    delete_selected_unit
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    select_stack_member
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    movement_path_shadows
                        .in_set(crate::GameSet)
                        .after(clear_paths_on_turn_change)
                        .after(reconcile_unit_sprites),
                    crate::fire::fire_direction_arrow.in_set(crate::GameSet),
                    crate::melee::melee_direction_arrow.in_set(crate::GameSet),
                    crate::melee::advance_target_overlay_mesh.in_set(crate::GameSet),
                    crate::turn_track_ui::turn_track_gizmos.in_set(crate::GameSet),
                    crate::desertion::detect_desertion_turn.in_set(crate::GameSet),
                    crate::river_placement::roll_for_struck_mine.in_set(crate::GameSet),
                    // §10 optional-rule mine/chain placement: the click
                    // router routes it only in Setup, for the Dervish seat.
                    crate::river_placement::handle_optional_rule_click
                        .in_set(crate::GameSet)
                        .in_set(crate::board_click::BoardClickHandlerSet)
                        .after(reconcile_unit_sprites),
                ),
            )
            // -- Egui UI panels -----------------------------------------
            // These in-game side panels run only while actually in a game, so
            // they don't linger over the lobby.
            .add_systems(
                EguiPrimaryContextPass,
                (
                    // The tray's flags and textures, ahead of the command
                    // rail that draws the tray (below the top bar).
                    unit_picker_ui
                        .in_set(crate::ui_plugin::PanelUiSet)
                        .after(crate::ui_plugin::mode_toolbar_ui),
                    crate::fire_allocation::resolve_fire_on_enter,
                    crate::melee::melee_reaction_ui,
                    crate::overview::unit_overview_ui
                        .in_set(crate::ui_plugin::PanelUiSet)
                        .in_set(crate::ui_plugin::LeftRailSet)
                        .after(unit_picker_ui),
                    movement_path_labels.run_if(crate::map_view_active),
                    crate::turn_track_ui::turn_track_labels,
                    crate::desertion::desertion_panel_ui,
                )
                    .run_if(crate::in_game_view),
            )
            // The same left rail in the spectator view: Overlays toggles +
            // unit list (game-control actions are gated to InGame inside).
            .add_systems(
                EguiPrimaryContextPass,
                crate::overview::unit_overview_ui
                    .in_set(crate::ui_plugin::PanelUiSet)
                    .in_set(crate::ui_plugin::LeftRailSet)
                    .after(crate::ui_plugin::mode_toolbar_ui)
                    .run_if(in_state(crate::AppState::Spectating)),
            );
    }
}

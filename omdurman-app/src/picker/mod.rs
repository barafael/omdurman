//! Unit picker: the left sidebar of available counters, placement preview,
//! click handling for place / select / move, and movement animation.
//!
//! Placement and movement are *requested* here by broadcasting
//! [`GameEvent::PlaceUnit`] / [`GameEvent::MoveUnit`]; the authoritative state
//! change (allocating a rules-engine `UnitId`, validating the move against the
//! unit's movement allowance, updating position) happens in
//! `apply_pending_placement`, which consumes those events. Keeping the request
//! and the application separate is what lets the same code path serve live
//! play and history replay.

use bevy::app::Plugin;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::{GameMap, HexLayout};
use omdurman_types::{HexCoord, HexsideRef, Scenario, SectionName, Terrain};

use std::collections::{HashSet, VecDeque};

use crate::AppState;
use crate::camera::RtsCamera;
use crate::events;
use crate::render::{HexOverlay, HexRingAssets};
use crate::sprites::{SpriteAnnotationsResource, section_order};
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
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // -- Resources ----------------------------------------------
            .insert_resource(UnitPicker::default())
            .insert_resource(PickerState::default())
            .insert_resource(MovementPath::default())
            .insert_resource(UnitPaths::default())
            .insert_resource(crate::zoc::ZocOverlay::default())
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
                    crate::apply_pending_placement.after(crate::net_socket::handle_socket),
                    (
                        placement_preview_mesh.in_set(crate::GameSet),
                        crate::fire_allocation::handle_fire_allocation_click
                            .in_set(crate::GameSet)
                            .before(handle_picker_clicks),
                        // Combat click handlers gate their phase through the
                        // mirrored §4 machine (see `ui_phase_state`): melee
                        // declaration and retreat-before-melee in Melee (§7),
                        // advance after combat in Melee or offensive fire
                        // (§6.82/§7.6).
                        crate::melee::handle_melee_combat
                            .in_set(crate::GameSet)
                            .run_if(crate::ui_phase_state::in_melee_phase)
                            .before(handle_picker_clicks),
                        crate::melee::handle_advance_after_combat
                            .in_set(crate::GameSet)
                            .run_if(crate::ui_phase_state::in_offensive_fire_or_melee_phase)
                            .after(crate::melee::handle_melee_combat)
                            .after(crate::fire_allocation::execute_fire_allocations)
                            .before(handle_picker_clicks),
                        crate::retreat::handle_retreat
                            .in_set(crate::GameSet)
                            .run_if(crate::ui_phase_state::in_melee_phase)
                            .before(handle_picker_clicks),
                        handle_picker_clicks
                            .in_set(crate::GameSet)
                            .in_set(crate::ui_plugin::MapPointerInputSet),
                        movement_overlay_mesh.in_set(crate::GameSet),
                        crate::fire::fire_target_overlay_mesh.in_set(crate::GameSet),
                        crate::melee::melee_target_overlay_mesh.in_set(crate::GameSet),
                        crate::retreat::retreat_overlay_mesh.in_set(crate::GameSet),
                        clear_paths_on_turn_change,
                        movement_path_arrows
                            .in_set(crate::GameSet)
                            .after(clear_paths_on_turn_change)
                            .after(crate::apply_pending_placement),
                        deployment_zone_overlay_mesh.in_set(crate::GameSet),
                        crate::fok_entry::fok_entry_overlay_mesh.in_set(crate::GameSet),
                        crate::fire_allocation::reset_fire_allocation_on_phase_change,
                        // (ZOC + LOS overlays are scheduled in main.rs for
                        // both the live game and the spectator view.)
                        animate_unit_movement,
                        layout_stacked_units.after(animate_unit_movement),
                        sync_disrupted_visuals,
                        sync_eliminated_visuals,
                        cancel_placement
                            .in_set(crate::GameSet)
                            .in_set(crate::ui_plugin::MapPointerInputSet),
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
                    confirm_movement_path
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    undo_movement_leg
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    delete_selected_unit
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    movement_path_shadows
                        .in_set(crate::GameSet)
                        .after(clear_paths_on_turn_change)
                        .after(crate::apply_pending_placement),
                    crate::fire::fire_direction_arrow.in_set(crate::GameSet),
                    crate::melee::melee_direction_arrow.in_set(crate::GameSet),
                    crate::melee::advance_target_overlay_mesh.in_set(crate::GameSet),
                    crate::turn_track_ui::turn_track_gizmos.in_set(crate::GameSet),
                    crate::desertion::detect_desertion_turn.in_set(crate::GameSet),
                    // §10 optional-rule mine/chain placement is a Setup-phase
                    // input (mirrored machine gate; see `ui_phase_state`).
                    crate::river_placement::handle_optional_rule_click
                        .in_set(crate::GameSet)
                        .run_if(crate::ui_phase_state::in_setup_phase)
                        .after(crate::apply_pending_placement),
                ),
            )
            // -- Egui UI panels -----------------------------------------
            // These in-game side panels run only while actually in a game, so
            // they don't linger over the lobby (the EditorMode can still be a
            // map mode in the lobby, which is an AppState, not a mode).
            .add_systems(
                EguiPrimaryContextPass,
                (
                    // Left-rail order matters: picker first, overview chains
                    // beside it (see `ScreenLayout::left_inset`), both below
                    // the top bar. Both carry `LeftRailSet` so downstream
                    // consumers order against the rail, not a (duplicated)
                    // system type.
                    unit_picker_ui
                        .in_set(crate::ui_plugin::PanelUiSet)
                        .in_set(crate::ui_plugin::LeftRailSet)
                        .after(crate::ui_plugin::mode_toolbar_ui),
                    crate::fire_allocation::fire_allocation_review_ui,
                    crate::melee::melee_reaction_ui,
                    crate::overview::unit_overview_ui
                        .in_set(crate::ui_plugin::PanelUiSet)
                        .in_set(crate::ui_plugin::LeftRailSet)
                        .after(unit_picker_ui),
                    movement_path_labels.run_if(crate::map_view_active),
                    crate::turn_track_ui::turn_track_labels,
                    crate::desertion::desertion_panel_ui,
                )
                    .run_if(in_state(crate::AppState::InGame)),
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
            )
            // -- Spectator: mirror the scrubbed engine state onto the board.
            //    Effect-only records (bot playthroughs) have no visual events,
            //    so the sprite world is reconciled from GameState here.
            //    Ordered after the scrub chain + placement so a playback step
            //    reconciles against the freshly rebuilt state in the SAME
            //    frame (scrub no longer despawns units; this system moves/
            //    spawns/despawns the diffs) -- otherwise each step flashed a
            //    frame of empty board.
            .add_systems(
                Update,
                sync_spectator_units
                    .run_if(in_state(crate::AppState::Spectating))
                    .after(crate::timeline::scrub_rebuild)
                    .after(crate::apply_pending_placement),
            );
    }
}

//! Deferred application of `PlaceUnit` / `MoveUnit` events.
//!
//! These two `GameEvent` variants are applied separately from the inline
//! `handle_socket` path because they need picker + mesh/material asset access.
//! Both live play (via `incoming.live`) and history replay (via
//! `incoming.replay`) route through [`apply_pending_placement`].

use bevy::prelude::*;
use omdurman_hexmap::hex_world_pos;
use omdurman_net::GameEvent;
use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
use omdurman_rules::{MovementPoints, UnitId, UnitPlacement, UnitProfile, UnitState};
use omdurman_types::{HexCoord, SectionName};

use crate::browser::SpriteAnnotationsResource;
use crate::picker::{MovementAnimation, PlacedUnit, UnitPaths, spawn_placed_unit};
use crate::PlacementContext;

/// Look up a counter's authored [`SpriteAnnotation`] and build its rules
/// profile. Returns `None` if annotations aren't loaded yet, the counter has
/// no annotation, or its section name is unrecognised -- in every case the
/// unit is placed visually but acquires no rules-engine `UnitId`.
fn profile_for(
    annotations: Option<&SpriteAnnotationsResource>,
    section_name: SectionName,
    col: u32,
    row: u32,
) -> Option<UnitProfile> {
    let annotation = annotations?
        .0
        .units
        .get(&section_name)
        .and_then(|m| m.get(&(col, row)))?;
    omdurman_rules::unit_profiles::profile_from_annotation(section_name, col, row, annotation)
}

/// Route a unit move through the rules engine so it validates the move
/// (allowance, phase, ZOC, night-halving) and updates `unit.position`
/// authoritatively. Returns whether the engine *accepted* the move: the caller
/// must apply the visual update only on `true`, so a rejected move never moves
/// the sprite (the engine is authoritative over position).
#[must_use]
fn apply_move_effect(
    state: &mut GameState,
    unit_id: UnitId,
    to: HexCoord,
    cost: MovementPoints,
    path: &[HexCoord],
) -> bool {
    if state.find_unit(unit_id).is_none() {
        warn!(?unit_id, "MoveUnit for unknown rules unit");
        return false;
    }
    let effect = GameEffect::MoveUnit {
        unit_id,
        to,
        cost,
        path: path.to_vec(),
    };
    if let Err(error) = apply_effect(state, &effect) {
        warn!(%error, ?unit_id, to.q = to.q, to.r = to.r, "move rejected by rules engine");
        return false;
    }
    true
}

/// Extend a unit's turn path with an accepted move. `path` is the sequence of
/// hexes *entered* this move (ending at `to`); when it is empty (legacy record /
/// sandbox) we fall back to a single hop straight to `to`. Each entered hex is
/// appended as its own step so multi-hex moves render as consecutive arrows.
fn record_move_path(
    paths: &mut UnitPaths,
    unit_id: UnitId,
    from: HexCoord,
    path: &[HexCoord],
    to: HexCoord,
) {
    let mut prev = from;
    let steps: &[HexCoord] = if path.is_empty() { &[to] } else { path };
    for &step in steps {
        if step != prev {
            paths.record_step(unit_id, prev, step);
            prev = step;
        }
    }
}

pub(crate) fn apply_pending_placement(
    ctx: PlacementContext,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let PlacementContext {
        mut incoming,
        mut picker,
        layout,
        overlay,
        game_map,
        mut game_state,
        annotations,
        mut unit_paths,
        mut placed_units,
        anim_query,
        mut just_placed,
    } = ctx;
    just_placed.clear();

    // Replay events and live events are both already recorded -- replay by the
    // canonical host log, live by `handle_socket` when the host-sequenced
    // event was applied. Do NOT re-record here.
    let replay_items: Vec<_> = incoming.replay.drain(..).map(|(msg, _peer)| msg).collect();
    let live_items: Vec<_> = incoming.live.drain(..).map(|(msg, _, _)| msg).collect();

    for event in replay_items.into_iter().chain(live_items) {
        match event {
            GameEvent::PlaceUnit {
                sprite,
                coord,
                is_boat,
            } => {
                let section_name = sprite.section_name;
                let col = sprite.col;
                let row = sprite.row;
                if !game_map.hexes.contains_key(&coord) {
                    warn!(
                        ?coord,
                        "ignoring inbound PlaceUnit for off-map coord"
                    );
                    continue;
                }
                // Local entity from handle_picker_clicks has unit_id: None;
                // allocate the rules-engine UnitId and update it in place.
                if let Some((_entity, mut placed)) = placed_units.iter_mut().find(|(_, u)| {
                    u.unit_id.is_none()
                        && u.section_name == section_name
                        && u.col == col
                        && u.row == row
                        && u.coord == coord
                }) {
                    let profile: Option<UnitProfile> =
                        profile_for(annotations.as_deref(), section_name, col, row);
                    let allocated = game_state.as_mut().and_then(|gs| {
                        let id = gs.0.alloc_unit_id();
                        let p = profile?;
                        gs.0.units.push(UnitPlacement {
                            id,
                            position: coord,
                            profile: p,
                            state: UnitState::default(),
                        });
                        Some(id)
                    });
                    placed.unit_id = allocated;
                    continue;
                }
                let unit_idx = picker
                    .available
                    .iter()
                    .position(|u| u.section_name == section_name && u.col == col && u.row == row);
                if let Some(idx) = unit_idx {
                    let unit = picker.available.remove(idx);

                    // Allocate rules-engine UnitId and record placement in
                    // GameState so effect processing can refer to the unit.
                    let profile: Option<UnitProfile> =
                        profile_for(annotations.as_deref(), section_name, col, row);
                    let allocated = game_state.as_mut().and_then(|gs| {
                        let id = gs.0.alloc_unit_id();
                        let p = profile?;
                        gs.0.units.push(UnitPlacement {
                            id,
                            position: coord,
                            profile: p,
                            state: UnitState::default(),
                        });
                        Some(id)
                    });

                    let origin = layout.adjusted_origin(&overlay.params);
                    let pos = hex_world_pos(coord, origin, &overlay.params);
                    let entity = spawn_placed_unit(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        unit.handle.clone(),
                        &overlay,
                        pos,
                        PlacedUnit {
                            coord,
                            section_name,
                            col,
                            row,
                            is_boat,
                            unit_id: allocated,
                            disrupted: false,
                        },
                    );
                    debug!(
                        col,
                        row,
                        coord.q = coord.q,
                        coord.r = coord.r,
                        "applied placement"
                    );
                    just_placed.insert((section_name, col, row), (entity, is_boat, allocated));
                }
            }
            GameEvent::MoveUnit {
                sprite,
                to_q,
                to_r,
                cost,
                path,
            } => {
                let section_name = sprite.section_name;
                let col = sprite.col;
                let row = sprite.row;
                debug!(
                    ?section_name,
                    col, row, to_q, to_r, "apply_pending_placement: processing MoveUnit",
                );
                let target = omdurman_types::HexCoord::new(to_q, to_r);
                if !game_map.hexes.contains_key(&target) {
                    warn!(to_q, to_r, "ignoring inbound MoveUnit to off-map coord");
                    continue;
                }
                let origin = layout.adjusted_origin(&overlay.params);
                let pos = hex_world_pos(target, origin, &overlay.params);
                let new_transform = Transform::from_xyz(pos.x, 1.0, pos.z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0));

                // Try the live world query first (normal gameplay path).
                let mut found = false;
                for (entity, mut placed) in placed_units.iter_mut() {
                    if placed.section_name == section_name && placed.col == col && placed.row == row
                    {
                        debug!(
                            ?section_name,
                            col, row, "apply_pending_placement: found entity for MoveUnit",
                        );
                        // The rules engine is authoritative: validate first and
                        // move the sprite only if the engine accepts. A rejected
                        // move leaves the counter where it was. (The picker's
                        // terrain-aware `cost` rides on the event; the engine
                        // recomputes from it.) Units without a rules id, or a
                        // sandbox with no game state, fall through as accepted.
                        let accepted = match (placed.unit_id, game_state.as_mut()) {
                            (Some(unit_id), Some(gs)) => {
                                apply_move_effect(&mut gs.0, unit_id, target, cost, &path)
                            }
                            _ => true,
                        };
                        if accepted {
                            // Record the route before moving the counter, using
                            // the pre-move hex as this step's origin. Covers the
                            // interactive single-hop (`path == [target]`) and any
                            // multi-hop path carried on the event.
                            if let Some(uid) = placed.unit_id {
                                record_move_path(&mut unit_paths, uid, placed.coord, &path, target);
                            }
                            placed.coord = target;
                            // Don't snap if a local movement animation is already
                            // playing -- let animate_unit_movement finish it.
                            if anim_query.get(entity).is_err() {
                                commands.entity(entity).insert(new_transform);
                                commands
                                    .entity(entity)
                                    .remove::<MovementAnimation>();
                            }
                        }
                        found = true;
                        break;
                    }
                }

                // Fall back to units placed earlier in this same batch
                // (replay path -- Bevy commands are still deferred).
                if !found
                    && let Some(&(entity, is_boat, unit_id)) =
                        just_placed.get(&(section_name, col, row))
                {
                    debug!(
                        ?section_name,
                        col, row, "apply_pending_placement: MoveUnit fell back to just_placed",
                    );
                    // Route through the rules engine (see apply_move_effect).
                    // This batch-fallback is the replay path; the event is
                    // canonical history, so apply it visually regardless.
                    if let Some(uid) = unit_id
                        && let Some(ref mut gs) = game_state
                    {
                        // Capture the pre-move hex for the path before the effect
                        // updates it.
                        let from = gs.0.find_unit(uid).map(|u| u.position);
                        let _ = apply_move_effect(&mut gs.0, uid, target, cost, &path);
                        if let Some(from) = from {
                            record_move_path(&mut unit_paths, uid, from, &path, target);
                        }
                    }
                    commands.entity(entity).insert(PlacedUnit {
                        coord: target,
                        section_name,
                        col,
                        row,
                        is_boat,
                        unit_id,
                        // Re-synced by `sync_disrupted_visuals` next frame.
                        disrupted: false,
                    });
                    debug!(
                        col,
                        row,
                        to.q = target.q,
                        to.r = target.r,
                        "applied move (replay fallback)"
                    );
                    commands.entity(entity).insert(new_transform);
                    // update the map so subsequent moves on the same unit work
                    just_placed.insert((section_name, col, row), (entity, is_boat, unit_id));
                }
                if found {
                    debug!(col, row, to.q = target.q, to.r = target.r, "applied move");
                } else {
                    warn!(
                        ?section_name,
                        col, row, "apply_pending_placement: MoveUnit target entity not found",
                    );
                }
            }
            // Other GameEvent variants are applied inline by handle_socket /
            // rebuild_state_to -- they shouldn't appear in the deferred
            // queues. Warn if one does so the misclassification is visible.
            other => warn!(?other, "non-placement GameEvent in placement queue"),
        }
    }

    // -- Ephemeral messages handled by apply_ephemeral() -- see below --
}

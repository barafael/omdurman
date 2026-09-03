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
use omdurman_rules::{
    MovementPoints, Phase, UnitId, UnitPlacement, UnitProfile, UnitState, unit_id_for_section_pos,
};
use omdurman_types::{HexCoord, SectionName};

use crate::PlacementContext;
use crate::picker::{MovementAnimation, PickerUnit, PlacedUnit, UnitPaths, spawn_placed_unit};

/// Build a rules profile for a counter by its sprite-sheet position.
/// Returns `None` if the position maps to no known [`UnitId`] or no identity.
fn profile_for(section_name: SectionName, col: u32, row: u32) -> Option<UnitProfile> {
    let unit_id = unit_id_for_section_pos(section_name, col as u8, row as u8)?;
    omdurman_rules::unit_profiles::profile_for_unit(unit_id)
}

/// Return a counter to the picker so the player can re-place it, after its
/// placement was rejected by the engine or picked back up. Idempotent: a no-op
/// if the sprite is already in `available` (e.g. a remote peer that never
/// removed it). Looks the sprite up in `picker.all` to recover its image handle
/// and boat flag.
fn return_sprite_to_picker(
    picker: &mut crate::picker::UnitPicker,
    section_name: SectionName,
    col: u32,
    row: u32,
) {
    let already = picker
        .available
        .iter()
        .any(|u| u.section_name == section_name && u.col == col && u.row == row);
    if already {
        return;
    }
    let Some((sn, c, r, handle)) = picker
        .all
        .iter()
        .find(|(sn, c, r, _, _)| *sn == section_name && *c == col && *r == row)
        .map(|(sn, c, r, handle, _)| (*sn, *c, *r, handle.clone()))
    else {
        return;
    };
    // Boat-ness from the sprite profile (the engine's source of truth), not
    // `picker.all`'s flag -- that is initialised `false` and never updated, so
    // it would mistag a returned gunboat as a land unit.
    let is_boat = profile_for(sn, c, r).is_some_and(|p| p.kind.is_boat());
    picker.available.push(PickerUnit {
        section_name: sn,
        col: c,
        row: r,
        handle,
        is_boat,
        visible: true,
        egui_texture: None,
        annotations_loaded: true,
    });
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
/// unbound session) we fall back to a single hop straight to `to`. Each entered hex is
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
        annotations: _annotations,
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
                    warn!(?coord, "ignoring inbound PlaceUnit for off-map coord");
                    continue;
                }
                // Resolve the rules identity deterministically from the sprite
                // position. Each physical counter maps to exactly one UnitId,
                // so two peers placing the same sprite place the same unit and
                // replay converges (no per-peer allocation race).
                let (Some(unit_id), Some(profile)) = (
                    unit_id_for_section_pos(section_name, col as u8, row as u8),
                    profile_for(section_name, col, row),
                ) else {
                    warn!(
                        ?section_name,
                        col, row, "PlaceUnit sprite has no UnitId/profile; ignoring",
                    );
                    continue;
                };
                let placement = UnitPlacement {
                    id: unit_id,
                    position: coord,
                    profile,
                    state: UnitState::default(),
                };

                // The engine is authoritative over placement (§9.2/§9.3): it
                // validates phase, deployment zone, full stacking (§5.51-5.53),
                // and that this counter isn't already on the board. A rejected
                // placement must not leave a sprite on the map.
                //
                // During a Movement phase an off-board counter enters as a
                // *reinforcement* (§9.112/§9.113 Campaign order of appearance;
                // §9.322 FoK turn-1 edge) — `DeployUnit` is Setup-only, so the
                // echo applies `PlaceReinforcements` instead. The same sprite
                // event carries both: the counter identity is resolved above.
                let accepted = match game_state.as_mut() {
                    Some(gs) => {
                        let effect = if matches!(gs.0.phase, Phase::Movement) {
                            GameEffect::PlaceReinforcements(vec![placement])
                        } else {
                            GameEffect::DeployUnit(placement)
                        };
                        match apply_effect(&mut gs.0, &effect) {
                            Ok(()) => true,
                            Err(error) => {
                                warn!(
                                    ?section_name, col, row,
                                    coord.q = coord.q, coord.r = coord.r,
                                    %error,
                                    "PlaceUnit rejected by rules engine (§9.2/§9.3/§9.112)",
                                );
                                false
                            }
                        }
                    }
                    // Unbound session (no GameState): nothing to validate,
                    // accept visually.
                    None => true,
                };

                // Replay dedupe: a timeline scrub re-queues every `PlaceUnit`
                // in `0..=cursor`, but the scrub no longer despawns placed
                // units (that blanked the board for a frame on every playback
                // step). If this counter is already on the board, the engine
                // state above has just been reset from seed and re-deployed,
                // so only the visual spawn must be skipped. In live play the
                // engine rejects a duplicate deploy (unit already on board)
                // and the pre-existing fallthrough skipped the spawn too.
                if placed_units.iter().any(|(_, u)| u.unit_id == Some(unit_id)) {
                    debug!(
                        ?section_name,
                        col, row, "PlaceUnit: unit already on board; skipping visual respawn",
                    );
                    continue;
                }

                // The local click handler optimistically spawned an entity with
                // `unit_id: None` before the host-sequenced echo arrived. Remote
                // peers and replay have not spawned one yet. `unit_id.is_none()`
                // distinguishes the optimistic entity from a real one (which a
                // race-lost placement must not touch).
                let optimistic = placed_units.iter_mut().find(|(_, u)| {
                    u.unit_id.is_none()
                        && u.section_name == section_name
                        && u.col == col
                        && u.row == row
                        && u.coord == coord
                });

                if let Some((entity, mut placed)) = optimistic {
                    // Local peer path: the click handler spawned this.
                    if accepted {
                        placed.unit_id = Some(unit_id);
                    } else {
                        // Engine rejected: despawn the orphan and return the
                        // counter to the picker so the player can re-place it.
                        commands.entity(entity).despawn();
                        return_sprite_to_picker(&mut picker, section_name, col, row);
                    }
                    continue;
                }

                // Remote/replay peer: the counter is still in the picker.
                if !accepted {
                    // Never spawned here, still available -- nothing to undo.
                    continue;
                }
                let unit_idx = picker
                    .available
                    .iter()
                    .position(|u| u.section_name == section_name && u.col == col && u.row == row);
                if let Some(idx) = unit_idx {
                    let unit = picker.available.remove(idx);

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
                            unit_id: Some(unit_id),
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
                    just_placed.insert((section_name, col, row), (entity, is_boat, Some(unit_id)));
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
                        // unbound session with no game state, fall through as accepted.
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
                                commands.entity(entity).remove::<MovementAnimation>();
                            }
                        } else {
                            // Rules engine rejected the move (e.g. ZOC stop,
                            // §5.43). Cancel any in-flight animation so
                            // animate_unit_movement doesn't overwrite
                            // placed.coord with the rejected destination.
                            commands.entity(entity).remove::<MovementAnimation>();
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
            GameEvent::RemoveUnit { sprite } => {
                let section_name = sprite.section_name;
                let col = sprite.col;
                let row = sprite.row;
                // Locate the placed entity (if any) and read its rules id.
                let target = placed_units
                    .iter()
                    .find(|(_, u)| u.section_name == section_name && u.col == col && u.row == row)
                    .map(|(entity, placed)| (entity, placed.unit_id));
                let Some((entity, unit_id)) = target else {
                    debug!(
                        ?section_name,
                        col, row, "RemoveUnit: no placed entity, nothing to do",
                    );
                    continue;
                };

                // The engine is authoritative over setup pickup too (§9.2/§9.3):
                // only legal during Setup, only for an on-board unit, and only
                // the owner's counter. The acting player is the unit's owner
                // (the picker gates which side may pick up; the engine
                // re-validates state legality here).
                let player = omdurman_rules::unit_profiles::section_owner(section_name);
                let accepted = match (unit_id, game_state.as_mut(), player) {
                    (Some(uid), Some(gs), Some(owner)) => {
                        match apply_effect(
                            &mut gs.0,
                            &GameEffect::RemoveDeployedUnit {
                                unit_id: uid,
                                player: owner,
                            },
                        ) {
                            Ok(()) => true,
                            Err(error) => {
                                warn!(
                                    ?section_name, col, row, %error,
                                    "RemoveUnit rejected by rules engine",
                                );
                                false
                            }
                        }
                    }
                    // No engine / no unit_id / unknown owner (unbound session,
                    // editor, or a sprite that never resolved): accept visually.
                    _ => true,
                };
                if !accepted {
                    continue;
                }
                commands.entity(entity).despawn();
                debug!(?section_name, col, row, "applied RemoveUnit");
                return_sprite_to_picker(&mut picker, section_name, col, row);
            }
            // Other GameEvent variants are applied inline by handle_socket /
            // rebuild_state_to -- they shouldn't appear in the deferred
            // queues. Warn if one does so the misclassification is visible.
            other => warn!(?other, "non-placement GameEvent in placement queue"),
        }
    }

    // -- Ephemeral messages handled by apply_ephemeral() -- see below --
}

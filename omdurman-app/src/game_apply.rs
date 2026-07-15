//! Shared `GameEvent` application path for live messages (`handle_socket`)
//! and snapshot replay (`replay_game_history`).
//!
//! `PlaceUnit` / `MoveUnit` are handled separately by `apply_pending_placement`
//! because they need picker + mesh-asset access; both callers route those
//! events through their own queues and never pass them here.
//!
//! `GameEvent::Effect` is dispatched to the rules engine and mutates
//! [`GameState`]; the remaining variants update map/editor/UI state.

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, clip_hexes_to_overlay, load_map_data};
use omdurman_net::GameEvent;
use omdurman_rules::effects::{GameState, apply_effect};
use omdurman_types::{HexData, MapKind};

use crate::{LoadedAnnotations, browser, editor, render, units};

pub struct GameApplyCtx<'a, 'w, 's> {
    pub game_map: &'a mut GameMap,
    pub overlay: &'a mut render::HexOverlay,
    pub editor: &'a mut editor::HexEditor,
    pub annotations: Option<&'a mut browser::SpriteAnnotationsResource>,
    pub viewer: &'a mut units::UnitViewer,
    pub commands: &'a mut Commands<'w, 's>,
    pub game_state: Option<&'a mut GameState>,
    /// In-memory two-board file. Map edits mirror into the targeted board's
    /// section here so the inactive board and the persisted file stay correct
    /// regardless of which board is currently live (§dual-map).
    pub loaded_annotations: Option<&'a mut LoadedAnnotations>,
    /// Which board is currently loaded into the live `GameMap`/overlay/sprites.
    /// An edit whose `map` matches also mutates the live state; an edit for the
    /// other board only updates its stored section (§dual-map).
    pub active_map: MapKind,
}

pub fn apply_game_event(event: &GameEvent, ctx: &mut GameApplyCtx<'_, '_, '_>) {
    match event {
        GameEvent::StartGame { .. } => {}
        GameEvent::Effect(effect) => {
            if let Some(ref mut state) = ctx.game_state {
                debug!(?effect, "applying game effect");
                if let Err(e) = apply_effect(state, effect) {
                    warn!("effect rejected: {e}");
                } else {
                    debug!(
                        phase = ?state.phase,
                        turn = state.current_turn.value(),
                        active_player = ?state.active_player,
                        "effect applied successfully"
                    );
                }
            } else {
                warn!("GameEvent::Effect received but no GameState available");
            }
        }
        GameEvent::LoadAnnotations(f) => {
            let active = ctx.active_map;
            debug!(?active, "applying LoadAnnotations");
            load_map_data(f.map(active), ctx.game_map);
            ctx.overlay.params = ctx.game_map.overlay.clone();
            if let Some(ann) = ctx.annotations.as_deref_mut() {
                ann.0 = f.sprites.clone();
            } else {
                ctx.commands
                    .insert_resource(browser::SpriteAnnotationsResource(f.sprites.clone()));
            }
            if let Some(loaded) = ctx.loaded_annotations.as_deref_mut() {
                loaded.0 = f.as_ref().clone();
            }
        }
        GameEvent::MapEdit {
            map,
            coord,
            terrain,
            name,
        } => {
            debug!(map = ?map, ?coord, terrain = ?terrain, name = %name, "applying MapEdit");
            let tile = HexData {
                terrain: *terrain,
                location: None,
                name: (!name.is_empty()).then(|| name.clone()),
                setup_letter: None,
            };
            // Stored section (always), so the inactive board / disk file stay correct.
            if let Some(loaded) = ctx.loaded_annotations.as_deref_mut() {
                loaded.0.map_mut(*map).tiles.insert((coord.q, coord.r), tile.clone());
            }
            // Live map only when this edit targets the loaded board.
            if *map == ctx.active_map {
                if let Some(slot) = ctx.game_map.hexes.get_mut(coord) {
                    *slot = tile;
                } else {
                    warn!(?coord, "ignoring MapEdit for off-map coord");
                }
            }
        }
        GameEvent::OverlayUpdate { map, params: p } => {
            debug!(map = ?map, "applying OverlayUpdate");
            if let Some(loaded) = ctx.loaded_annotations.as_deref_mut() {
                loaded.0.map_mut(*map).overlay = p.clone();
            }
            if *map == ctx.active_map {
                let p2 = p.clone();
                ctx.overlay.params = p2.clone();
                ctx.game_map.overlay = p2;
                clip_hexes_to_overlay(ctx.game_map);
            }
        }
        GameEvent::ExcludeHex {
            map,
            coord,
            excluded,
        } => {
            debug!(map = ?map, ?coord, excluded, "applying ExcludeHex");
            if let Some(loaded) = ctx.loaded_annotations.as_deref_mut() {
                let set = &mut loaded.0.map_mut(*map).excluded;
                if *excluded {
                    set.insert((coord.q, coord.r));
                } else {
                    set.remove(&(coord.q, coord.r));
                }
            }
            if *map == ctx.active_map {
                if *excluded {
                    ctx.game_map.excluded.insert(*coord);
                } else {
                    ctx.game_map.excluded.remove(coord);
                }
                // Re-derive the live hex set so the excluded hex drops out (or a
                // re-included hex comes back as fresh Desert).
                clip_hexes_to_overlay(ctx.game_map);
            }
        }
        GameEvent::HexsideEdit { map, edge, kind } => {
            debug!(map = ?map, ?edge, ?kind, "applying HexsideEdit");
            if let Some(loaded) = ctx.loaded_annotations.as_deref_mut() {
                let sides = &mut loaded.0.map_mut(*map).hexsides;
                sides.retain(|(e, _)| e != edge);
                if let Some(k) = kind {
                    sides.push((*edge, *k));
                }
            }
            if *map == ctx.active_map {
                match kind {
                    Some(k) => {
                        ctx.game_map.hexsides.insert(*edge, *k);
                    }
                    None => {
                        ctx.game_map.hexsides.remove(edge);
                    }
                }
            }
        }
        GameEvent::RoadEdit { map, edge, present } => {
            debug!(map = ?map, ?edge, present, "applying RoadEdit");
            if *present {
                let a_nile = ctx
                    .game_map
                    .hexes
                    .get(&edge.a)
                    .is_some_and(|h| h.terrain.is_nile());
                let b_nile = ctx
                    .game_map
                    .hexes
                    .get(&edge.b)
                    .is_some_and(|h| h.terrain.is_nile());
                if a_nile || b_nile {
                    return;
                }
            }
            if let Some(loaded) = ctx.loaded_annotations.as_deref_mut() {
                let roads = &mut loaded.0.map_mut(*map).roads;
                if *present {
                    if !roads.contains(edge) {
                        roads.push(*edge);
                    }
                } else {
                    roads.retain(|e| e != edge);
                }
            }
            if *map == ctx.active_map {
                if *present {
                    ctx.game_map.roads.insert(*edge);
                } else {
                    ctx.game_map.roads.remove(edge);
                }
            }
        }
        GameEvent::AnnotateSprite { sprite, annotation } => {
            // Sprite annotations are global (board-independent): write the stored
            // file's top-level sprites and the live resource, regardless of board.
            if let Some(loaded) = ctx.loaded_annotations.as_deref_mut() {
                loaded
                    .0
                    .sprites
                    .units
                    .entry(sprite.section_name)
                    .or_default()
                    .insert((sprite.col, sprite.row), annotation.clone());
            }
            if let Some(ann) = ctx.annotations.as_deref_mut() {
                ann.0
                    .units
                    .entry(sprite.section_name)
                    .or_default()
                    .insert((sprite.col, sprite.row), annotation.clone());
            }
        }
        GameEvent::ShowTerrainOverlay(v) => {
            ctx.editor.show_terrain_overlay = *v;
        }
        GameEvent::UpdateUnitGrids { grids } => {
            ctx.viewer.grids = grids.clone();
            units::save_unit_grids(&ctx.viewer.grids);
        }
        GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } => {
            // Callers route these into their own deferred queues before
            // calling apply_game_event; reaching this arm is a routing bug.
            warn!(?event, "placement event reached apply_game_event");
        }
        GameEvent::TurnComplete(summary) => {
            // Turn summaries are already built by `apply_effect(EndPlayerTurn)`
            // during replay. This event is recorded for the canonical log and
            // for late-joiner information; no additional state mutation needed.
            debug!(turn = summary.turn.value(), "TurnComplete (informational)");
        }
    }
}

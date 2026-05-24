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
use omdurman_map::{GameMap, clip_hexes_to_overlay};
use omdurman_net::GameEvent;
use omdurman_rules::effects::{apply_effect, GameState};
use omdurman_types::{HexCoord, HexData, Terrain};

use crate::{browser, editor, render, units};

pub struct GameApplyCtx<'a, 'w, 's> {
    pub game_map: &'a mut GameMap,
    pub overlay: &'a mut render::HexOverlay,
    pub editor: &'a mut editor::HexEditor,
    pub annotations: Option<&'a mut browser::SpriteAnnotationsResource>,
    pub viewer: &'a mut units::UnitViewer,
    pub commands: &'a mut Commands<'w, 's>,
    pub game_state: Option<&'a mut GameState>,
}

pub fn apply_game_event(event: &GameEvent, ctx: &mut GameApplyCtx<'_, '_, '_>) {
    match event {
        GameEvent::Effect(effect) => {
            if let Some(ref mut state) = ctx.game_state {
                if let Err(e) = apply_effect(state, effect) {
                    warn!("effect rejected: {e}");
                }
            } else {
                warn!("GameEvent::Effect received but no GameState available");
            }
        }
        GameEvent::LoadAnnotations(f) => {
            for ((q, r), tile) in &f.map.tiles {
                ctx.game_map.hexes.insert(
                    HexCoord::new(*q, *r),
                    HexData::new(tile.terrain, tile.name.clone()),
                );
            }
            ctx.game_map.overlay = f.overlay.clone();
            ctx.overlay.params = f.overlay.clone();
            clip_hexes_to_overlay(ctx.game_map);
            if let Some(ann) = ctx.annotations.as_deref_mut() {
                ann.0 = f.sprites.clone();
            } else {
                ctx.commands
                    .insert_resource(browser::SpriteAnnotationsResource(f.sprites.clone()));
            }
        }
        GameEvent::Action(_) => {
            // Turn advancement depends on the live peer count, which the
            // event log doesn't capture — handled by the caller.
        }
        GameEvent::MapEdit {
            q,
            r,
            terrain,
            name,
        } => {
            let coord = HexCoord::new(*q, *r);
            if let Some(slot) = ctx.game_map.hexes.get_mut(&coord) {
                *slot = HexData::new(
                    Terrain::from_u8(*terrain),
                    (!name.is_empty()).then(|| name.clone()),
                );
            } else {
                warn!(q, r, "ignoring MapEdit for off-map coord");
            }
        }
        GameEvent::OverlayUpdate(p) => {
            ctx.overlay.params = p.clone();
            ctx.game_map.overlay = p.clone();
            clip_hexes_to_overlay(ctx.game_map);
        }
        GameEvent::AnnotateSprite {
            section_name,
            col,
            row,
            annotation,
        } => {
            if let Some(ann) = ctx.annotations.as_deref_mut() {
                ann.0
                    .units
                    .entry(section_name.clone())
                    .or_default()
                    .insert((*col, *row), annotation.clone());
            }
        }
        GameEvent::ShowTerrainOverlay(v) => {
            ctx.editor.show_terrain_overlay = *v;
        }
        GameEvent::UpdateUnitGrids(grids) => {
            ctx.viewer.grids = grids.clone();
            units::save_unit_grids(&ctx.viewer.grids);
        }
        GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } => {
            // Callers route these into their own deferred queues before
            // calling apply_game_event; reaching this arm is a routing bug.
            warn!(?event, "placement event reached apply_game_event");
        }
    }
}

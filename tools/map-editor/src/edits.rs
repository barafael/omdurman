//! Local edit application: the tool's equivalent of the game's
//! `GameEvent`-based edit path. Every edit mutates *both* the stored board
//! ([`LoadedAnnotations`], the save target) and, when it targets the loaded
//! board, the live [`GameMap`] -- the same mirroring semantics the game used
//! for its network echoes (§dual-map).

use omdurman_hexmap::{GameMap, HexOverlay, clip_hexes_to_overlay};
use omdurman_types::{
    HexCoord, HexData, HexsideKind, HexsideRef, MapKind, NamedArea, SetupLetter, Terrain,
};

use crate::board::{ActiveEditMap, LoadedAnnotations};

/// Everything an edit needs to land in both the stored and live boards.
pub(crate) struct EditCtx<'a> {
    pub loaded: &'a mut LoadedAnnotations,
    pub game_map: &'a mut GameMap,
    pub overlay: &'a mut HexOverlay,
    pub active: &'a ActiveEditMap,
}

impl EditCtx<'_> {
    fn kind(&self) -> MapKind {
        self.active.0
    }

    fn is_live(&self, map: MapKind) -> bool {
        map == self.active.0
    }
}

/// Set a hex's terrain + name, preserving the other per-hex attributes
/// (setup letter, scattergram flag, named area). Re-derives `location` from
/// the new name so it matches what `BoardInfo::from_map_data` would produce.
pub(crate) fn apply_map_edit(
    ctx: &mut EditCtx<'_>,
    coord: HexCoord,
    edit: impl FnOnce(&HexData) -> (Terrain, Option<String>),
) {
    let map = ctx.kind();
    let Some(prev) = ctx
        .game_map
        .hexes
        .get(&coord)
        .cloned()
        .or_else(|| ctx.loaded.map(map).tiles.get(&(coord.q, coord.r)).cloned())
    else {
        return;
    };
    let (terrain, name) = edit(&prev);
    if prev.terrain == terrain && prev.name == name {
        return;
    }
    let tile = HexData {
        location: name
            .as_deref()
            .and_then(omdurman_types::Location::from_tile_name),
        name,
        terrain,
        setup_letter: prev.setup_letter,
        is_scattergram: prev.is_scattergram,
        named_area: prev.named_area,
    };
    ctx.loaded
        .map_mut(map)
        .tiles
        .insert((coord.q, coord.r), tile.clone());
    if ctx.is_live(map)
        && let Some(slot) = ctx.game_map.hexes.get_mut(&coord)
    {
        *slot = tile;
    }
}

/// Toggle a road connection between two adjacent hexes (roads never touch a
/// Nile hex).
pub(crate) fn apply_road_edit(ctx: &mut EditCtx<'_>, edge: HexsideRef, present: bool) {
    let map = ctx.kind();
    if present {
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
    let roads = &mut ctx.loaded.map_mut(map).roads;
    if present {
        if !roads.contains(&edge) {
            roads.push(edge);
        }
    } else {
        roads.retain(|e| *e != edge);
    }
    if ctx.is_live(map) {
        if present {
            ctx.game_map.roads.insert(edge);
        } else {
            ctx.game_map.roads.remove(&edge);
        }
    }
}

/// Set or clear a hexside feature.
pub(crate) fn apply_hexside_edit(
    ctx: &mut EditCtx<'_>,
    edge: HexsideRef,
    kind: Option<HexsideKind>,
) {
    let map = ctx.kind();
    let sides = &mut ctx.loaded.map_mut(map).hexsides;
    sides.retain(|(e, _)| *e != edge);
    if let Some(k) = kind {
        sides.push((edge, k));
    }
    if ctx.is_live(map) {
        match kind {
            Some(k) => {
                ctx.game_map.hexsides.insert(edge, k);
            }
            None => {
                ctx.game_map.hexsides.remove(&edge);
            }
        }
    }
}

/// Exclude a hex from the map (board furniture) or re-include it (re-enters as
/// playable with its stored tile, or Desert if none).
pub(crate) fn apply_exclude(ctx: &mut EditCtx<'_>, coord: HexCoord, excluded: bool) {
    let map = ctx.kind();
    let set = &mut ctx.loaded.map_mut(map).excluded;
    if excluded {
        set.insert((coord.q, coord.r));
    } else {
        set.remove(&(coord.q, coord.r));
    }
    if ctx.is_live(map) {
        if excluded {
            ctx.game_map.excluded.insert(coord);
        } else {
            ctx.game_map.excluded.remove(&coord);
        }
        // Re-derive the live hex set so the excluded hex drops out (or a
        // re-included hex comes back).
        clip_hexes_to_overlay(ctx.game_map);
        if !excluded
            && !ctx.game_map.hexes.contains_key(&coord)
            && let Some(tile) = ctx.loaded.map(map).tiles.get(&(coord.q, coord.r)).cloned()
        {
            ctx.game_map.hexes.insert(coord, tile);
        }
    }
}

/// Set or clear the historical-scenario setup letter (rulebook §9.212).
pub(crate) fn apply_setup_letter(
    ctx: &mut EditCtx<'_>,
    coord: HexCoord,
    letter: Option<SetupLetter>,
) {
    let map = ctx.kind();
    if let Some(d) = ctx.loaded.map_mut(map).tiles.get_mut(&(coord.q, coord.r)) {
        d.setup_letter = letter;
    }
    if ctx.is_live(map)
        && let Some(d) = ctx.game_map.hexes.get_mut(&coord)
    {
        d.setup_letter = letter;
    }
}

/// Set or clear the named-area membership (rulebook §9.112/§9.113).
pub(crate) fn apply_named_area(ctx: &mut EditCtx<'_>, coord: HexCoord, area: Option<NamedArea>) {
    let map = ctx.kind();
    if let Some(d) = ctx.loaded.map_mut(map).tiles.get_mut(&(coord.q, coord.r)) {
        d.named_area = area;
    }
    if ctx.is_live(map)
        && let Some(d) = ctx.game_map.hexes.get_mut(&coord)
    {
        d.named_area = area;
    }
}

/// Apply new overlay/calibration params to the stored board and (when live)
/// the live overlay + map, re-deriving the hex set.
pub(crate) fn apply_overlay_update(ctx: &mut EditCtx<'_>, params: &omdurman_types::OverlayParams) {
    let map = ctx.kind();
    ctx.loaded.map_mut(map).overlay = params.clone();
    if ctx.is_live(map) {
        ctx.overlay.params = params.clone();
        ctx.game_map.overlay = params.clone();
        clip_hexes_to_overlay(ctx.game_map);
    }
}

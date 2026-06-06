//! Retreat before melee (§7.5) — the defender's reaction.
//!
//! During the *attacker's* Melee phase, the **defending** player may pull a
//! threatened cavalry/camel unit two hexes back before the melee is resolved.
//! This is the non-active player's action, so it is gated on the local faction
//! being the *opponent* of the rules engine's active player (the attacker).
//!
//! Selecting an eligible unit highlights the legal two-hex retreat
//! destinations (empty, passable, within range); clicking one broadcasts a
//! [`GameEffect::RetreatBeforeMelee`]. The engine validates via
//! [`GameState::can_retreat_before_melee`].

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_hex::HexLayout;
use omdurman_map::GameMap;
use omdurman_net::{GameEvent, NetMsg, NetState};
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{Phase, UnitId};
use omdurman_types::{HexCoord, Terrain};

use crate::camera::RtsCamera;
use crate::picker::{PickerState, PlacedUnit};
use crate::render::{HexOverlay, draw_hex_outline};
use crate::util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground};
use crate::{EditorMode, GameStateResource, PendingEdits, PlayerFactions};

/// The selected unit's rules `UnitId` and hex, if it is engine-tracked.
fn selected_unit_id(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
) -> Option<(UnitId, HexCoord)> {
    let PickerState::Selected { source, .. } = *state else {
        return None;
    };
    let (_, placed) = placed_units.get(source).ok()?;
    Some((placed.unit_id?, placed.coord))
}

/// Whether the local player is the *defender* this melee phase — i.e. the
/// active (attacking) player is the opponent of the local faction.
fn local_is_defender(factions: &PlayerFactions, net: &NetState, gs: &GameState) -> bool {
    match factions.local(net) {
        Some(mine) => mine == gs.active_player.opponent(),
        // Unbound sandbox: allow retreat handling (single-seat play/testing).
        None => factions.by_peer.is_empty(),
    }
}

/// Whether a hex is a legal retreat destination: on-map, passable land, empty.
fn passable_empty(game_map: &GameMap, gs: &GameState, hex: HexCoord) -> bool {
    let on_passable_land = game_map
        .hexes
        .get(&hex)
        .is_some_and(|h| h.terrain != Terrain::BlueNile && h.terrain != Terrain::WhiteNile);
    on_passable_land && !gs.units.iter().any(|u| u.position == hex)
}

/// Whether `unit` is currently threatened — adjacent to at least one enemy
/// infantry unit (the trigger for a retreat, §7.5).
fn threatened_by_infantry(unit: UnitId, gs: &GameState) -> bool {
    let Some(u) = gs.find_unit(unit) else {
        return false;
    };
    let enemy = u.profile.identity.owner().opponent();
    let neigh = u.position.neighbors();
    gs.units.iter().any(|e| {
        e.profile.identity.owner() == enemy
            && e.profile.kind == omdurman_rules::UnitKind::Infantry
            && neigh.contains(&e.position)
    })
}

/// Two-hex retreat destinations the selected unit may legally move to.
fn valid_retreat_hexes(unit: UnitId, gs: &GameState, game_map: &GameMap) -> Vec<HexCoord> {
    let Some(u) = gs.find_unit(unit) else {
        return Vec::new();
    };
    // Candidate hexes are exactly two away (the §7.5 retreat distance).
    let mut out: Vec<HexCoord> = game_map
        .hexes
        .keys()
        .copied()
        .filter(|h| u.position.distance(*h) == 2)
        .filter(|h| passable_empty(game_map, gs, *h))
        .filter(|h| gs.can_retreat_before_melee(unit, *h).is_ok())
        .collect();
    out.sort_by_key(|h| (h.q, h.r));
    out
}

/// Highlight legal retreat destinations (cyan) when the defender selects a
/// threatened cavalry/camel unit during the attacker's Melee phase.
pub fn retreat_overlay_gizmo(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    factions: Res<PlayerFactions>,
    net: Res<NetState>,
    mut gizmos: Gizmos,
) {
    if *mode != EditorMode::Normal {
        return;
    }
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) || !local_is_defender(&factions, &net, &gs.0) {
        return;
    }
    let Some((unit, _)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    if !threatened_by_infantry(unit, &gs.0) {
        return;
    }

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    for hex in valid_retreat_hexes(unit, &gs.0, &game_map) {
        let pos = hex_world_pos(hex, origin, &overlay.params);
        draw_hex_outline(
            &mut gizmos,
            pos,
            overlay.params.hex_size,
            Color::srgb(0.2, 0.9, 0.95),
        );
    }
}

/// On left-click of a legal retreat hex while the defender has a threatened
/// cavalry/camel unit selected, broadcast a `RetreatBeforeMelee` effect.
#[allow(clippy::too_many_arguments)]
pub fn handle_retreat(
    mode: Res<EditorMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut state: ResMut<PickerState>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    game_state: Option<Res<GameStateResource>>,
    factions: Res<PlayerFactions>,
    net: Res<NetState>,
    mut pending: ResMut<PendingEdits>,
) {
    if *mode != EditorMode::Normal || !buttons.just_released(MouseButton::Left) {
        return;
    }
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) || !local_is_defender(&factions, &net, &gs.0) {
        return;
    }
    let Some((unit, _)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    if !threatened_by_infantry(unit, &gs.0) {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_pointer_input() {
        return;
    }
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let to = hit_to_hex(hit, origin, &overlay.params);

    if gs.0.can_retreat_before_melee(unit, to).is_err() || !passable_empty(&game_map, &gs.0, to) {
        return;
    }

    info!(?unit, to.q = to.q, to.r = to.r, "retreat before melee");
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::Effect(
            GameEffect::RetreatBeforeMelee { unit_id: unit, to },
        )));
    *state = PickerState::Idle;
}

//! Retreat before melee (§7.5) -- the defender's reaction.
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
use omdurman_hexmap::GameMap;
use omdurman_net::{GameEvent, NetMsg};
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{Phase, UnitId};
use omdurman_types::HexCoord;

use crate::input::CombatClickCtx;
use crate::peers::Peers;
use crate::picker::{PickerState, PlacedUnit, selected_unit_id};
use crate::{GameStateResource, PendingEdits};
use omdurman_hexmap::hex_world_pos;

/// Bundle of the read-only picker state + the placed-units query so
/// [`retreat_overlay_mesh`] stays under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct RetreatSelection<'w, 's> {
    pub state: Res<'w, PickerState>,
    pub placed_units: Query<'w, 's, (Entity, &'static PlacedUnit)>,
}

/// Whether the local player is the *defender* this melee phase -- i.e. the
/// active (attacking) player is the opponent of the local faction.
fn local_is_defender(peers: &Peers, gs: &GameState) -> bool {
    match peers.local() {
        Some(mine) => mine == gs.active_player.opponent(),
        // Unbound session: allow retreat handling (single-seat play/testing).
        None => !peers.any_assigned(),
    }
}

/// Whether a hex is a legal retreat destination: on-map, passable land, empty.
fn passable_empty(game_map: &GameMap, gs: &GameState, hex: HexCoord) -> bool {
    let on_passable_land = game_map
        .hexes
        .get(&hex)
        .is_some_and(|h| h.terrain.passable_by_land());
    on_passable_land && !gs.units.iter().any(|u| u.position == hex)
}

/// Whether `unit` is currently threatened -- adjacent to at least one enemy
/// infantry unit (the trigger for a retreat, §7.5).
fn threatened_by_infantry(unit: UnitId, gs: &GameState) -> bool {
    let Some(u) = gs.find_unit(unit) else {
        return false;
    };
    let enemy = u.profile.identity.owner().opponent();
    let neigh = u.position.neighbors();
    gs.units.iter().any(|e| {
        e.profile.identity.owner() == enemy
            && matches!(e.profile.kind, omdurman_types::UnitKind::Infantry { .. })
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

/// Highlight legal retreat destinations (orange) when the defender selects a
/// threatened cavalry/camel unit during the attacker's Melee phase.
#[derive(Component)]
pub struct RetreatTargetRing;

pub fn retreat_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    game_map: Res<GameMap>,
    selection: RetreatSelection,
    game_state: Option<Res<GameStateResource>>,
    peers: Peers,
    existing: Query<Entity, With<RetreatTargetRing>>,
) {
    let crate::HexRender { assets, layout, overlay } = hex;
    let RetreatSelection { state, placed_units } = selection;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) || !local_is_defender(&peers, &gs.0) {
        return;
    }
    let Some((unit, _)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    if !threatened_by_infantry(unit, &gs.0) {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for hex in valid_retreat_hexes(unit, &gs.0, &game_map) {
        let pos = hex_world_pos(hex, origin, &overlay.params);
        commands.spawn((
            RetreatTargetRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.orange.clone()),
            Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
}

/// On left-click of a legal retreat hex while the defender has a threatened
/// cavalry/camel unit selected, broadcast a `RetreatBeforeMelee` effect.
pub fn handle_retreat(
    mut click: CombatClickCtx,
    mut state: ResMut<PickerState>,
    game_map: Res<GameMap>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    peers: Peers,
    mut pending: ResMut<PendingEdits>,
) {
    let Some(to) = click.clicked_hex() else {
        return;
    };
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) || !local_is_defender(&peers, &gs.0) {
        return;
    }
    let Some((unit, _)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    if !threatened_by_infantry(unit, &gs.0) {
        return;
    }

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

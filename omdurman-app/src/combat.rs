//! Fire combat UI — target selection, validation, effect creation.
//!
//! The player selects a friendly placed unit (enters [`PickerState::Selected`]),
//! sees valid target hexes highlighted, and clicks one to broadcast a
//! [`GameEffect::FireCombat`] with a pre-rolled die.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_hex::HexLayout;
use omdurman_map::GameMap;
use omdurman_net::{GameEvent, GameRng, NetMsg};
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{DieRoll, FireAttack, FireFactor, FireKind, HexDistance, Phase, Player, UnitId};
use omdurman_types::HexCoord;

use crate::picker::{PlacedUnit, PickerState};
use crate::render::{HexOverlay, draw_hex_outline};
use crate::util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground};
use crate::{EditorMode, GameStateResource, PendingEdits};

/// Hex-distance between two axial coordinates (cube max norm).
fn hex_distance(a: HexCoord, b: HexCoord) -> HexDistance {
    let dq = (a.q - b.q).unsigned_abs();
    let dr = (a.r - b.r).unsigned_abs();
    let ds = (a.q + a.r - b.q - b.r).unsigned_abs();
    HexDistance(dq.max(dr.max(ds / 2)) as u16)
}

/// Whether the selected unit may fire in the current game state.
fn can_fire(unit_id: UnitId, state: &GameState) -> bool {
    let Some(unit) = state.find_unit(unit_id) else {
        return false;
    };
    if unit.state.disrupted {
        return false;
    }
    if unit.profile.fire.is_none() {
        return false;
    }
    if state.units_fired_this_phase.contains(&unit_id) {
        return false;
    }
    matches!(
        state.phase,
        Phase::DefensiveFire(_) | Phase::OffensiveFire(_)
    )
}

/// Valid target hexes for the selected unit: within range, enemy-occupied.
fn valid_target_hexes(
    unit: &PlacedUnit,
    state: &GameState,
) -> Vec<HexCoord> {
    let Some(uid) = unit.unit_id else {
        return Vec::new();
    };
    let Some(u) = state.find_unit(uid) else {
        return Vec::new();
    };
    let owner = u.profile.identity.owner();
    let Some(fire) = u.profile.fire else {
        return Vec::new();
    };

    // Max range: rifles 4, artillery 6 (simplified — weapons vary).
    let max_range = if u.profile.weapon == omdurman_rules::WeaponClass::Artillery {
        6
    } else {
        4
    };
    let _ = fire; // used for combined-fire factor computation in future

    state
        .units
        .iter()
        .filter(|u| u.position != unit.coord)
        .filter(|u| u.profile.identity.owner() == owner.opponent())
        .filter(|u| hex_distance(unit.coord, u.position).0 <= max_range)
        .map(|u| u.position)
        .collect()
}

/// Highlight valid target hexes when a unit is selected and can fire.
pub fn fire_target_overlay_gizmo(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    mut gizmos: Gizmos,
) {
    if *mode != EditorMode::Normal {
        return;
    }
    let PickerState::Selected { source, .. } = *state else {
        return;
    };
    let Some(ref gs) = game_state else {
        return;
    };
    let Ok((_, placed)) = placed_units.get(source) else {
        return;
    };
    let Some(uid) = placed.unit_id else {
        return;
    };
    if !can_fire(uid, &gs.0) {
        return;
    }

    let targets = valid_target_hexes(placed, &gs.0);
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);

    for hex in &targets {
        if !game_map.hexes.contains_key(hex) {
            continue;
        }
        let pos = hex_world_pos(*hex, origin, &overlay.params);
        draw_hex_outline(
            &mut gizmos,
            pos,
            overlay.params.hex_size,
            Color::srgb(1.0, 0.3, 0.3),
        );
    }
}

/// Roll a d10 for combat resolution.
fn roll_d10(rng: &mut GameRng) -> DieRoll {
    DieRoll::new(((rng.random_u32() % 10) + 1) as i16)
}

/// Handle left-click on a valid target hex while a unit is selected.
///
/// Validates the target, pre-rolls the attack die, constructs a
/// [`GameEffect::FireCombat`] and broadcasts it to all peers.
pub fn handle_fire_combat(
    mode: Res<EditorMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut state: ResMut<PickerState>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<crate::camera::RtsCamera>>,
    mut pending: ResMut<PendingEdits>,
    mut game_state: Option<ResMut<GameStateResource>>,
    mut rng: Option<ResMut<GameRng>>,
) {
    if *mode != EditorMode::Normal {
        return;
    }
    let PickerState::Selected { source, .. } = *state else {
        return;
    };
    let Some(ref mut gs) = game_state else {
        return;
    };
    let Some(ref mut rng) = rng else {
        return;
    };

    let Ok((_, placed)) = placed_units.get(source) else {
        *state = PickerState::Idle;
        return;
    };
    let Some(uid) = placed.unit_id else {
        return;
    };
    if !can_fire(uid, &gs.0) {
        return;
    }

    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if ctx.wants_pointer_input() {
        return;
    }

    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    if !game_map.hexes.contains_key(&coord) {
        return;
    }

    let targets = valid_target_hexes(placed, &gs.0);
    if !targets.contains(&coord) {
        *state = PickerState::Idle;
        return;
    }

    let owner = gs.0.find_unit(uid).map(|u| u.profile.identity.owner()).unwrap_or(Player::AngloEgyptian);

    let fire_factor = gs.0.find_unit(uid).and_then(|u| u.profile.fire).unwrap_or(FireFactor(2));

    let kind = match gs.0.phase {
        Phase::OffensiveFire(_) => FireKind::Direct,
        Phase::DefensiveFire(_) => FireKind::Direct,
        _ => return,
    };

    let attack = FireAttack {
        firing_player: owner,
        phase: gs.0.phase,
        kind,
        firers: vec![uid],
        target_hex: coord,
        total_factor: fire_factor,
        modifiers: Vec::new(),
    };

    let roll = roll_d10(&mut *rng);

    let effect = GameEffect::FireCombat { attack, roll };

    info!(
        firing_player = %owner,
        phase = ?gs.0.phase,
        target.q = coord.q,
        target.r = coord.r,
        fire_factor = ?fire_factor,
        roll = ?roll,
        "firing combat"
    );

    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::Effect(effect)));

    *state = PickerState::Idle;
}

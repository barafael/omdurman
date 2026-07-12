//! Melee combat -- adjacent target selection and `GameEffect::MeleeCombat`
//! emission (§7).
//!
//! When a friendly melee-capable unit is selected during the Melee phase and
//! the rules engine permits it ([`GameState::can_melee`]), adjacent enemy-
//! occupied hexes are highlighted. Clicking one builds a [`MeleeAttack`] --
//! the co-stacked melee-capable attackers vs. the defenders in the target hex,
//! with the standard side modifiers (Dervish +2, Anglo-Egyptian +1, §7.7) --
//! pre-rolls both dice, and broadcasts a [`GameEffect::MeleeCombat`].

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hexmap::HexLayout;
use omdurman_net::{GameEvent, NetMsg, NetState};
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{DieRoll, MeleeAttack, MeleeModifier, Phase, Player, UnitId};
use omdurman_types::HexCoord;

use crate::{
    GameRng, GameStateResource, PendingEdits,
    camera::RtsCamera,
    picker::{PickerState, PlacedUnit, selected_unit_id},
    render::{HexOverlay, HexRingAssets},
    util::raycast_ground,
};
use omdurman_hexmap::{adjusted_origin, hex_world_pos, hit_to_hex};

/// Adjacent enemy-occupied hexes the selected unit may legally melee-attack.
/// Wall/thorn-hedge hexside blocking (§7.2) is checked inside `can_melee`
/// via `self.board`.
fn valid_target_hexes(attacker: UnitId, gs: &GameState) -> Vec<HexCoord> {
    let Some(unit) = gs.find_unit(attacker) else {
        return Vec::new();
    };
    let from = unit.position;
    from.neighbors()
        .into_iter()
        .filter(|hex| gs.can_melee(attacker, *hex).is_ok())
        .collect()
}

/// Highlight valid melee targets in orange when a unit is selected during the
/// Melee phase.
#[derive(Component)]
pub struct MeleeTargetRing;

pub fn melee_target_overlay_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<MeleeTargetRing>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) {
        return;
    }
    let Some((attacker, _)) = selected_unit_id(&state, &placed_units) else {
        return;
    };

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;
    for hex in valid_target_hexes(attacker, &gs.0) {
        let pos = hex_world_pos(hex, origin, &overlay.params);
        commands.spawn((
            MeleeTargetRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.orange.clone()),
            Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
}

/// On left-click of a valid adjacent enemy hex while a melee-capable unit is
/// selected during the Melee phase, broadcast a `MeleeCombat` effect with both
/// pre-rolled dice.
#[allow(clippy::too_many_arguments)]
pub fn handle_melee_combat(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut state: ResMut<PickerState>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    game_state: Option<Res<GameStateResource>>,
    mut rng: Option<ResMut<GameRng>>,
    mut pending: ResMut<PendingEdits>,
    factions: Res<crate::PlayerFactions>,
    net: Res<NetState>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let (Some(gs), Some(rng)) = (game_state, rng.as_mut()) else {
        return;
    };
    if !matches!(gs.0.phase, Phase::Melee) {
        return;
    }
    // One declaration at a time: a melee already awaiting resolution must be
    // resolved (after the retreat window) before another is declared.
    if gs.0.pending_melee.is_some() {
        return;
    }
    // Only the active player (their faction) melees this phase (§lobby).
    if !factions.local_may_act(&net, gs.0.active_player) {
        return;
    }
    let Some((attacker, attacker_hex)) = selected_unit_id(&state, &placed_units) else {
        return;
    };

    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_pointer_input() {
        return;
    }
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let target = hit_to_hex(hit, origin, &overlay.params);

    // `can_melee` checks hexside blocking (§7.2) internally via `self.board`.
    match gs.0.can_melee(attacker, target) {
        Ok(()) => {}
        Err(omdurman_rules::effects::RuleError::MeleeBlockedByHexside(_, _)) => {
            info!(
                target.q = target.q,
                target.r = target.r,
                "melee blocked by hexside"
            );
            return;
        }
        Err(_) => {
            return;
        }
    }

    let Some(attack) = build_melee_attack(&gs.0, attacker_hex, target) else {
        return;
    };
    let attacker_roll = DieRoll::try_from(((rng.random_u32() % 10) + 1) as u16).unwrap();
    let defender_roll = DieRoll::try_from(((rng.random_u32() % 10) + 1) as u16).unwrap();

    info!(
        ?attacker,
        target.q = target.q,
        target.r = target.r,
        at = %attacker_roll,
        def = %defender_roll,
        "declare melee"
    );

    // Declare the melee (opens the defender's retreat window, §7.5). The
    // attacker resolves it once defenders have reacted -- see
    // `attacker_resolve_ui`.
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::Effect(GameEffect::DeclareMelee {
            attack,
            attacker_roll,
            defender_roll,
        })));

    *state = PickerState::Idle;
}

/// While a melee is pending resolution (§7.5 reaction window), show a small
/// panel: the **attacker** gets a "Resolve Melee" button (the defender has had
/// the chance to retreat); the **defender** is told they may retreat the
/// highlighted unit. The attacker resolves by broadcasting `ResolveMelee`.
pub fn melee_reaction_ui(
    mut contexts: EguiContexts,
    game_state: Option<Res<GameStateResource>>,
    factions: Res<crate::PlayerFactions>,
    net: Res<NetState>,
    mut pending: ResMut<PendingEdits>,
) {
    let Some(gs) = game_state else { return };
    let Some(pm) = &gs.0.pending_melee else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let attacker_player = pm.attack.attacker_player;
    let local_is_attacker = factions.local_may_act(&net, attacker_player);
    let target = pm.attack.defender_hex;

    egui::Window::new("Melee declared")
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 60.0))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(format!(
                "{attacker_player} melee on hex ({}, {})",
                target.q, target.r
            ));
            if local_is_attacker {
                ui.label("Defenders may retreat. Resolve when ready.");
                if ui.button("[swords] Resolve Melee").clicked() {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::Game(GameEvent::Effect(GameEffect::ResolveMelee)));
                }
            } else {
                ui.label("You may retreat the threatened cavalry/camel (click a");
                ui.label("highlighted hex), or wait for the attacker to resolve.");
            }
        });
}

/// Build the `MeleeAttack`: every co-stacked melee-capable friendly unit in
/// `attacker_hex` attacks all enemy units in `defender_hex`, with the standard
/// side melee modifier (§7.7).
fn build_melee_attack(
    gs: &GameState,
    attacker_hex: HexCoord,
    defender_hex: HexCoord,
) -> Option<MeleeAttack> {
    // The selected unit determines the attacking side.
    let owner = gs
        .units
        .iter()
        .find(|u| u.position == attacker_hex)
        .map(|u| u.profile.identity.owner())?;
    let enemy = owner.opponent();

    let attackers: Vec<UnitId> = gs
        .units
        .iter()
        .filter(|u| u.position == attacker_hex)
        .filter(|u| u.profile.identity.owner() == owner)
        .filter(|u| u.profile.kind.may_melee_attack() && !u.state.disrupted)
        .map(|u| u.id)
        .collect();
    if attackers.is_empty() {
        return None;
    }

    // All enemy units in the target hex defend (gunboats can't be melee'd --
    // §7.1).
    let defenders: Vec<UnitId> = gs
        .units
        .iter()
        .filter(|u| u.position == defender_hex)
        .filter(|u| u.profile.identity.owner() == enemy)
        .filter(|u| u.profile.kind.may_be_melee_attacked())
        .map(|u| u.id)
        .collect();
    if defenders.is_empty() {
        return None;
    }

    let attacker_modifiers = vec![side_modifier(owner)];
    let defender_modifiers = vec![side_modifier(enemy)];

    Some(MeleeAttack {
        attacker_player: owner,
        attacker_hex,
        defender_hex,
        attackers,
        defenders,
        attacker_modifiers,
        defender_modifiers,
    })
}

/// The standard per-side melee die modifier (§7.7): Dervish +2, A-E +1.
fn side_modifier(player: Player) -> MeleeModifier {
    match player {
        Player::Dervish => MeleeModifier::DervishStandard,
        Player::AngloEgyptian => MeleeModifier::AngloEgyptianStandard,
    }
}

/// Advance after combat (§6.82, §7.6): during a combat phase, with one of the
/// active player's units selected, clicking an adjacent hex that the engine
/// accepts (vacated, the unit isn't artillery) advances it. Targets empty
/// hexes, so it never collides with the fire/melee attack handlers (which
/// target enemy-occupied hexes).
#[allow(clippy::too_many_arguments)]
pub fn handle_advance_after_combat(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut state: ResMut<PickerState>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    game_state: Option<Res<GameStateResource>>,
    mut pending: ResMut<PendingEdits>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let Some(gs) = game_state else { return };
    // §6.7: no advance after combat from defensive fire -- only after melee
    // (§7.6) and offensive fire (§6.82).
    if !matches!(gs.0.phase, Phase::Melee | Phase::OffensiveFire(_)) {
        return;
    }
    let Some((unit_id, _from)) = selected_unit_id(&state, &placed_units) else {
        return;
    };

    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_pointer_input() {
        return;
    }
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let to = hit_to_hex(hit, origin, &overlay.params);

    // `can_advance_after_combat` checks hexside blocking (§6.82/§7.6)
    // internally via `self.board`.
    match gs.0.can_advance_after_combat(unit_id, to) {
        Ok(()) => {}
        Err(omdurman_rules::effects::RuleError::AdvanceBlockedByHexside(_, _)) => {
            info!(to.q = to.q, to.r = to.r, "advance blocked by hexside");
            return;
        }
        Err(_) => {
            return;
        }
    }

    info!(?unit_id, to.q = to.q, to.r = to.r, "advance after combat");
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::Effect(
            GameEffect::AdvanceAfterCombat { unit_id, to },
        )));
    *state = PickerState::Idle;
}

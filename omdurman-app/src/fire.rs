//! Fire combat — target selection and `GameEffect::FireCombat` emission.
//!
//! When a friendly unit is selected ([`PickerState::Selected`]) during a fire
//! sub-phase and the rules engine says it may fire, enemy-occupied hexes in
//! range are highlighted. Clicking one builds a [`FireAttack`] — firer, total
//! factor, and die-roll modifiers (Anglo-Egyptian +1, target terrain) — pre-
//! rolls the d10, and broadcasts a [`GameEffect::FireCombat`] so every peer
//! resolves the identical attack.
//!
//! The rules engine owns range/CRT resolution; the app supplies the terrain
//! modifier (the engine holds no map) and gates on [`GameState::can_fire_at`].

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_hexmap::{GameMap, HexLayout};
use omdurman_net::{GameEvent, GameRng, NetMsg, NetState};
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::tables::{TerrainType, defense_modifier};
use omdurman_rules::{
    DieModifier, DieRoll, FireAttack, FireKind, FireModifier, Phase, Player, UnitId,
};
use omdurman_types::HexCoord;

use crate::camera::RtsCamera;
use crate::picker::{PickerState, PlacedUnit};
use crate::render::{HexOverlay, draw_hex_outline};
use crate::util::raycast_ground;
use crate::{GameStateResource, PendingEdits};
use omdurman_hexmap::{adjusted_origin, hex_world_pos, hit_to_hex};

/// The selected unit's rules `UnitId`, if it is engine-tracked.
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

/// The fire kind a firer would use in the current sub-phase (§6.42):
/// direct fire in the Direct sub-phase; in the second sub-phase a Maxim uses
/// its second fire and a named gunboat fires howitzer. Returns `None` if the
/// firer can't act in this sub-phase (e.g. a rifle unit in the second sub-
/// phase).
fn fire_kind_for(gs: &GameState, firer: UnitId) -> Option<FireKind> {
    use omdurman_rules::WeaponClass;
    let unit = gs.find_unit(firer)?;
    let sub = match gs.phase {
        Phase::OffensiveFire(s) | Phase::DefensiveFire(s) => s,
        _ => return None,
    };
    match sub {
        omdurman_rules::FireSubPhase::DirectFire => Some(FireKind::Direct),
        omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => match unit.profile.weapon {
            WeaponClass::Maxims => Some(FireKind::MaximSecondFire),
            WeaponClass::Howitzer => Some(FireKind::Howitzer),
            _ => None,
        },
    }
}

/// Whether the firer at `from` has line of sight to `to` (§6.3). Howitzer
/// fire ignores LOS entirely (§6.64), so it is always permitted.
///
/// Blocked by:
/// * a wall or crest **hexside** crossed along the line (gates/breaches pass);
/// * a built-up **intervening hex** (hut/building/city/fort);
/// * more than two intervening palm-grove hexes (§6.3 note 1).
///
/// A firer on a Hilltop sees over intervening *terrain* (§6.3 note 2), but
/// wall/crest hexsides still block.
fn has_los(game_map: &GameMap, from: HexCoord, to: HexCoord, kind: FireKind) -> bool {
    if kind == FireKind::Howitzer {
        return true;
    }

    let firer_on_hilltop = game_map
        .hexes
        .get(&from)
        .is_some_and(|d| d.terrain == omdurman_types::Terrain::Hilltop);

    // Full hex sequence from firer to target; edges are crossed between
    // consecutive hexes.
    let mut path = vec![from];
    path.extend(from.line_between(to));
    path.push(to);

    let mut trees = 0;
    for window in path.windows(2) {
        let (a, b) = (window[0], window[1]);
        // Hexside blocking applies regardless of hilltop.
        if let Some(side) = game_map.hexside_between(a, b)
            && side.blocks_los()
        {
            return false;
        }
        // Intervening-hex terrain blocking (skip the endpoints; only the hex
        // we're *entering* and which isn't the target counts as intervening).
        if b != to {
            let Some(data) = game_map.hexes.get(&b) else {
                continue;
            };
            if firer_on_hilltop {
                continue; // sees over terrain (but not hexsides, handled above)
            }
            if data.terrain.blocks_los() {
                return false;
            }
            if data.terrain.is_los_trees() {
                trees += 1;
                if trees > 2 {
                    return false;
                }
            }
        }
    }
    true
}

/// Enemy-occupied hexes the selected unit may legally fire at right now, given
/// the fire kind for the current sub-phase and line of sight.
fn valid_target_hexes(
    firer: UnitId,
    firer_hex: HexCoord,
    kind: FireKind,
    gs: &GameState,
    game_map: &GameMap,
) -> Vec<HexCoord> {
    let Some(firer_unit) = gs.find_unit(firer) else {
        return Vec::new();
    };
    let enemy = firer_unit.profile.identity.owner().opponent();
    let mut targets: Vec<HexCoord> = gs
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == enemy)
        .map(|u| u.position)
        .filter(|hex| gs.can_fire_at(firer, *hex, kind).is_ok())
        .filter(|hex| has_los(game_map, firer_hex, *hex, kind))
        .collect();
    targets.sort_by_key(|h| (h.q, h.r));
    targets.dedup();
    targets
}

/// Highlight valid fire targets in red when a unit is selected during a fire
/// sub-phase.
pub fn fire_target_overlay_gizmo(
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    mut gizmos: Gizmos,
) {
    let Some(gs) = game_state else { return };
    if !matches!(
        gs.0.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    ) {
        return;
    }
    let Some((firer, firer_hex)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(kind) = fire_kind_for(&gs.0, firer) else {
        return;
    };

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    for hex in valid_target_hexes(firer, firer_hex, kind, &gs.0, &game_map) {
        let pos = hex_world_pos(hex, origin, &overlay.params);
        draw_hex_outline(
            &mut gizmos,
            pos,
            overlay.params.hex_size,
            Color::srgb(1.0, 0.2, 0.2),
        );
    }
}

/// On left-click of a valid target hex while a unit is selected during a fire
/// sub-phase, broadcast a `FireCombat` effect with a pre-rolled die.
#[allow(clippy::too_many_arguments)]
pub fn handle_fire_combat(
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
    if !matches!(
        gs.0.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    ) {
        return;
    }
    // Only the player whose faction is firing this phase may act (§lobby).
    let firing_player = match gs.0.phase {
        Phase::OffensiveFire(_) => gs.0.active_player,
        Phase::DefensiveFire(_) => gs.0.active_player.opponent(),
        _ => return,
    };
    if !factions.local_may_act(&net, firing_player) {
        return;
    }
    let Some((firer, firer_hex)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(kind) = fire_kind_for(&gs.0, firer) else {
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

    // Only act on a legal, visible target; otherwise leave the click for the
    // picker (which will deselect).
    if gs.0.can_fire_at(firer, target, kind).is_err() {
        return;
    }
    if !has_los(&game_map, firer_hex, target, kind) {
        info!(
            target.q = target.q,
            target.r = target.r,
            "no line of sight to target"
        );
        return;
    }

    let Some(attack) = build_fire_attack(&gs.0, &game_map, firer, firer_hex, target, kind) else {
        return;
    };
    let mut d10 = || DieRoll::new(((rng.random_u32() % 10) + 1) as i16);

    // Howitzer fire (§6.64) rolls twice — once for the CRT, once for impact
    // scatter — and uses its own effect; everything else is a single-roll
    // direct/Maxim-second fire.
    let effect = if kind == FireKind::Howitzer {
        let crt_roll = d10();
        let impact_roll = d10();
        info!(
            ?firer,
            target.q = target.q,
            target.r = target.r,
            crt = crt_roll.get(),
            impact = impact_roll.get(),
            "howitzer fire"
        );
        GameEffect::HowitzerFire {
            attack,
            crt_roll,
            impact_roll,
        }
    } else {
        let roll = d10();
        info!(
            ?firer,
            target.q = target.q,
            target.r = target.r,
            roll = roll.get(),
            "firing"
        );
        GameEffect::FireCombat { attack, roll }
    };

    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::Effect(effect)));

    // Consume the click so the picker doesn't also treat it as a move.
    *state = PickerState::Idle;
}

/// Build a combined `FireAttack` (§6.14): every friendly unit stacked in the
/// selected unit's hex (`firer_hex`) that may legally fire at `target` fires
/// together, their fire factors summed. Bakes in the die-roll modifiers the
/// engine can't derive: the Anglo-Egyptian +1 direct-fire bonus (§6.24), the
/// +1 brigade-integrity bonus when all four battalions fire (§5.54), and the
/// target hex's terrain modifier (§6.23).
fn build_fire_attack(
    gs: &GameState,
    game_map: &GameMap,
    firer: UnitId,
    firer_hex: HexCoord,
    target: HexCoord,
    kind: FireKind,
) -> Option<FireAttack> {
    let selected = gs.find_unit(firer)?;
    let owner = selected.profile.identity.owner();

    // Combine all co-stacked friendly units that may legally fire at the
    // target this phase with the *same* kind (§6.14). For Maxim-second and
    // howitzer fire this naturally limits the stack to like weapons.
    let firers: Vec<&omdurman_rules::UnitPlacement> = gs
        .units
        .iter()
        .filter(|u| u.position == firer_hex)
        .filter(|u| u.profile.identity.owner() == owner)
        .filter(|u| u.profile.fire.is_some())
        .filter(|u| gs.can_fire_at(u.id, target, kind).is_ok())
        .collect();
    if firers.is_empty() {
        return None;
    }

    let total_factor = omdurman_rules::FireFactor(
        firers
            .iter()
            .map(|u| u.profile.fire.map(|f| f.0).unwrap_or(0))
            .sum(),
    );

    let mut modifiers = Vec::new();
    // The +1 accuracy bonus and brigade integrity apply to *direct* fire only
    // (§6.24); Maxim second fire and howitzer fire get neither.
    if kind == FireKind::Direct {
        if owner == Player::AngloEgyptian {
            modifiers.push(FireModifier::AngloEgyptianDirectFire);
        }
        let identities: Vec<_> = firers.iter().map(|u| u.profile.identity).collect();
        if let omdurman_rules::BrigadeIntegrity::Integrated(_) =
            omdurman_rules::brigade_integrity(&identities)
        {
            modifiers.push(FireModifier::BrigadeIntegrity);
        }
    }
    if let Some(terrain_mod) = terrain_modifier(game_map, target)
        && terrain_mod.0 != 0
    {
        modifiers.push(FireModifier::Terrain(terrain_mod));
    }

    Some(FireAttack {
        firing_player: owner,
        phase: gs.phase,
        kind,
        firers: firers.iter().map(|u| u.id).collect(),
        target_hex: target,
        total_factor,
        modifiers,
    })
}

/// The terrain defensive die-roll modifier for the units in `hex` (§6.23),
/// from the game map's terrain. `None` if the hex is off-map.
fn terrain_modifier(game_map: &GameMap, hex: HexCoord) -> Option<DieModifier> {
    let terrain = game_map.hexes.get(&hex)?.terrain;
    Some(defense_modifier(TerrainType::from_terrain(terrain)))
}

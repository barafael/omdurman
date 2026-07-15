//! Fire combat -- target selection and `GameEffect::FireCombat` emission.
//!
//! When a friendly unit is selected ([`PickerState::Selected`]) during a fire
//! sub-phase and the rules engine says it may fire, enemy-occupied hexes in
//! range are highlighted. Clicking one builds a [`FireAttack`] -- firer, total
//! factor, and die-roll modifiers (Anglo-Egyptian +1, target terrain) -- pre-
//! rolls the d10, and broadcasts a [`GameEffect::FireCombat`] so every peer
//! resolves the identical attack.
//!
//! The rules engine owns range/Combat Results Table resolution; the app supplies the terrain
//! modifier (the engine holds no map) and gates on [`GameState::can_fire_at`].

use crate::GameRng;
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_hexmap::HexLayout;
use omdurman_net::{GameEvent, NetMsg, NetState};
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{
    FireAttack, FireFactor, FireKind, FireModifier, Phase, UnitId,
};
use omdurman_types::{HexCoord, Player};

use crate::input::CombatClickCtx;
use crate::picker::{PickerState, PlacedUnit, selected_unit_id};
use crate::render::{HexOverlay, HexRingAssets};
use crate::{GameStateResource, PendingEdits};
use omdurman_hexmap::hex_world_pos;

/// Stage a CRT chart spotlight on cell (row, col) -- gentle: pulses the peek
/// tab if closed, applies directly if already open (see `charts.rs`).
fn stage_crt(charts: &mut MessageWriter<crate::charts::ChartSheetRequest>, row: usize, col: usize) {
    charts.write(crate::charts::ChartSheetRequest {
        tab: crate::charts::ChartTab::Crt,
        highlight: Some(crate::charts::ChartHighlight {
            chart: crate::charts::ChartTab::Crt,
            table: 0, // table 0 == the Combat Results Table
            row: Some(row),
            col: Some(col),
        }),
    });
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

/// Enemy-occupied hexes the selected unit may legally fire at right now, given
/// the fire kind for the current sub-phase and line of sight. LOS is now
/// checked inside `can_fire_at` (via `self.board`), so no separate filter is
/// needed.
fn valid_target_hexes(firer: UnitId, kind: FireKind, gs: &GameState) -> Vec<HexCoord> {
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
        .collect();
    targets.sort_by_key(|h| (h.q, h.r));
    targets.dedup();
    targets
}

/// Highlight valid fire targets in red when a unit is selected during a fire
/// sub-phase.
#[derive(Component)]
pub(crate) struct FireTargetRing;

pub fn fire_target_overlay_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<FireTargetRing>>,
) {
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = game_state else { return };
    if !matches!(
        gs.0.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    ) {
        return;
    }
    let Some((firer, _firer_hex)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(kind) = fire_kind_for(&gs.0, firer) else {
        return;
    };

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for hex in valid_target_hexes(firer, kind, &gs.0) {
        let pos = hex_world_pos(hex, origin, &overlay.params);
        commands.spawn((
            FireTargetRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.red.clone()),
            Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
}

/// On left-click of a valid target hex while a unit is selected during a fire
/// sub-phase, broadcast a `FireCombat` effect with a pre-rolled die.
pub fn handle_fire_combat(
    mut click: CombatClickCtx,
    mut state: ResMut<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    mut rng: Option<ResMut<GameRng>>,
    mut pending: ResMut<PendingEdits>,
    factions: Res<crate::PlayerFactions>,
    net: Res<NetState>,
    mut charts: MessageWriter<crate::charts::ChartSheetRequest>,
    mut dispatches: ResMut<crate::dispatch::Dispatches>,
) {
    let Some(target) = click.clicked_hex() else {
        return;
    };
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

    // Only act on a legal, visible target; otherwise leave the click for the
    // picker (which will deselect). `can_fire_at` now checks LOS internally
    // via `self.board` (§6.21/§6.3).
    match gs.0.can_fire_at(firer, target, kind) {
        Ok(()) => {}
        Err(omdurman_rules::effects::RuleError::LineOfSightBlocked(_, _)) => {
            info!(
                target.q = target.q,
                target.r = target.r,
                "no line of sight to target"
            );
            dispatches.push("Field Telegraph", "Fire refused — no line of sight (§6.3).");
            return;
        }
        Err(_) => {
            return;
        }
    }

    let Some(attack) = build_fire_attack(&gs.0, firer, firer_hex, target, kind) else {
        return;
    };
    let mut d10 = || rng.roll_d10();

    // Howitzer fire (§6.64) rolls twice -- once for the Combat Results Table,
    // once for impact scatter -- and uses its own effect; everything else is a
    // single-roll direct/Maxim-second fire.
    // The Combat Results Table row (factor band) and column (die roll) this
    // fire resolves on, for a contextual chart spotlight (§decision 3).
    let crt_row = attack.factor_row.index();
    let effect = if kind == FireKind::Howitzer {
        let combat_results_table_roll = d10();
        let impact_roll = d10();
        info!(
            ?firer,
            target.q = target.q,
            target.r = target.r,
            combat_results_table = %combat_results_table_roll,
            impact = %impact_roll,
            "howitzer fire"
        );
        let crt_col = combat_results_table_roll.value() as usize - 1;
        stage_crt(&mut charts, crt_row, crt_col);
        GameEffect::HowitzerFire {
            attack,
            combat_results_table_roll,
            impact_roll,
        }
    } else {
        let roll = d10();
        info!(
            ?firer,
            target.q = target.q,
            target.r = target.r,
            roll = %roll,
            "firing"
        );
        stage_crt(&mut charts, crt_row, roll.value() as usize - 1);
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
/// Combat preview: while a firer is selected during a fire sub-phase, show what
/// the attack on the *hovered* hex would be -- how many firers combine, their
/// summed fire factor, the net die-roll modifier, and the kind of fire -- so the
/// player can judge the shot before committing. Only shown to the firing player
/// on a legal, in-LOS target.
pub fn fire_combat_preview_ui(
    mut contexts: EguiContexts,
    state: Res<PickerState>,
    game_state: Option<Res<GameStateResource>>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    hovered: Res<crate::HoveredHex>,
    factions: Res<crate::PlayerFactions>,
    net: Res<NetState>,
) {
    let Some(gs) = game_state else { return };
    let Some(target) = hovered.0 else { return };
    if !matches!(
        gs.0.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    ) {
        return;
    }
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
    // Only preview a shot the player could actually take. LOS is checked
    // inside `can_fire_at` now (§6.21/§6.3).
    if gs.0.can_fire_at(firer, target, kind).is_err() {
        return;
    }
    let Some(attack) = build_fire_attack(&gs.0, firer, firer_hex, target, kind) else {
        return;
    };

    let factor: u16 = attack
        .firers
        .iter()
        .filter_map(|id| gs.0.find_unit(*id))
        .filter_map(|u| u.profile.fire)
        .map(|f| f.value())
        .sum();
    // Include the engine-side terrain defence modifier (§6.23) in the preview
    // total so the displayed modifier matches what `resolve_fire_attack` will
    // actually apply.
    let terrain_mod =
        gs.0.board
            .terrain_at(target)
            .map(omdurman_rules::terrain_chart::defense_modifier)
            .unwrap_or(0);
    let net_mod = attack.net_modifier() + terrain_mod;
    let kind_str = match kind {
        FireKind::Direct => "Direct fire",
        FireKind::MaximSecondFire => "Maxim 2nd fire",
        FireKind::Howitzer => "Howitzer",
    };
    let mod_str = if net_mod == 0 {
        "none".to_string()
    } else {
        format!("{net_mod:+}")
    };

    // Predicted outcome bands across raw rolls 1..=10, given the factor row
    // and the net modifier (terrain included). The engine still pre-rolls
    // the die for canonical resolution; this preview only tells the player
    // what each roll *would* produce.
    use omdurman_rules::combat_results_table::FireFactorRow;
    let factor_row = FireFactorRow::from_total(factor);
    let bands = crate::combat_predict::outcome_bands(factor_row, net_mod);
    let bands_str = bands
        .iter()
        .map(|b| b.label())
        .collect::<Vec<_>>()
        .join(" · ");

    let Ok(ctx) = contexts.ctx_mut() else { return };
    bevy_egui::egui::Area::new(bevy_egui::egui::Id::new("fire_preview"))
        .anchor(
            // Below the center-top HUD so the two don't overlap.
            bevy_egui::egui::Align2::CENTER_TOP,
            bevy_egui::egui::Vec2::new(0.0, 44.0),
        )
        .order(bevy_egui::egui::Order::Foreground)
        .show(ctx, |ui| {
            bevy_egui::egui::Frame::new()
                .fill(bevy_egui::egui::Color32::from_rgba_unmultiplied(
                    40, 20, 20, 220,
                ))
                .corner_radius(4.0)
                .inner_margin(bevy_egui::egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id =
                        Some(bevy_egui::egui::FontId::proportional(13.0));
                    ui.colored_label(
                        bevy_egui::egui::Color32::from_rgb(235, 200, 170),
                        format!(
                            "{kind_str} at ({},{}): {} firer(s), factor {}, roll mod {}",
                            target.q,
                            target.r,
                            attack.firers.len(),
                            factor,
                            mod_str,
                        ),
                    );
                    // Outcome preview -- the "what happens on each roll" line.
                    ui.colored_label(
                        bevy_egui::egui::Color32::from_rgb(200, 200, 200),
                        bevy_egui::egui::RichText::new(bands_str)
                            .size(12.0)
                            .monospace(),
                    );
                });
        });
}

fn build_fire_attack(
    gs: &GameState,
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

    let factor_row = FireFactor::sum_to_row(firers.iter().filter_map(|u| u.profile.fire.as_ref()));

    let mut modifiers = Vec::new();
    // The +1 accuracy bonus and brigade integrity apply to *direct* fire only
    // (§6.24); Maxim second fire and howitzer fire get neither.
    // Terrain defence modifier (§6.23) is now computed engine-side in
    // `resolve_fire_attack` from `state.board`, so it is not included here.
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

    Some(FireAttack {
        firing_player: owner,
        phase: gs.phase,
        kind,
        firers: firers.iter().map(|u| u.id).collect(),
        target_hex: target,
        factor_row,
        modifiers,
    })
}

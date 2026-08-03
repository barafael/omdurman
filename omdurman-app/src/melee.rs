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
use omdurman_net::{GameEvent, NetMsg};
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{MeleeAttack, MeleeModifier, Phase, UnitId};
use omdurman_types::{HexCoord, Player};

use crate::{
    GameRng, GameStateResource, PendingEdits,
    input::CombatClickCtx,
    peers::Peers,
    picker::{PickerState, PlacedUnit, selected_unit_id},
};
use omdurman_hexmap::hex_world_pos;

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
    hex: crate::HexRender,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<MeleeTargetRing>>,
) {
    let crate::HexRender { assets, layout, overlay } = hex;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) {
        return;
    }
    let Some((attacker, _)) = selected_unit_id(&state, &placed_units) else {
        return;
    };

    let origin = layout.adjusted_origin(&overlay.params);
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
pub fn handle_melee_combat(
    mut click: CombatClickCtx,
    mut state: ResMut<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    mut rng: Option<ResMut<GameRng>>,
    mut pending: ResMut<PendingEdits>,
    peers: Peers,
) {
    let Some(target) = click.clicked_hex() else {
        return;
    };
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
    if !peers.may_act(gs.0.active_player) {
        return;
    }
    let Some((attacker, attacker_hex)) = selected_unit_id(&state, &placed_units) else {
        return;
    };

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
    let attacker_roll = rng.roll_d10();
    let defender_roll = rng.roll_d10();

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
    peers: Peers,
    mut pending: ResMut<PendingEdits>,
) {
    let Some(gs) = game_state else { return };
    let Some(pm) = &gs.0.pending_melee else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let attacker_player = pm.attack.attacker_player;
    let local_is_attacker = peers.may_act(attacker_player);
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

    let mut attacker_modifiers = vec![side_modifier(owner)];
    let defender_modifiers = vec![side_modifier(enemy)];

    // §9.232: Dervish melee penalty when attacking into an entrenched trench hex.
    if owner == Player::Dervish && gs.board.is_zariba_entrenched(defender_hex) {
        attacker_modifiers.push(MeleeModifier::DervishVsTrenchedDefender);
    }

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
pub fn handle_advance_after_combat(
    mut click: CombatClickCtx,
    mut state: ResMut<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    mut pending: ResMut<PendingEdits>,
) {
    let Some(to) = click.clicked_hex() else {
        return;
    };
    let Some(gs) = game_state else { return };
    // §6.7: no advance after combat from defensive fire -- only after melee
    // (§7.6) and offensive fire (§6.82).
    if !matches!(gs.0.phase, Phase::Melee | Phase::OffensiveFire(_)) {
        return;
    }
    let Some((unit_id, _from)) = selected_unit_id(&state, &placed_units) else {
        return;
    };

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

// -- Melee direction arrow: translucent orange arrow from attacker to hovered target ---

#[derive(Component)]
pub(crate) struct MeleeDirectionArrow;

/// Draw a translucent orange arrow from the attacker hex to the hovered valid
/// melee target hex, giving the player a visual preview of the melee direction.
pub fn melee_direction_arrow(
    mut commands: Commands,
    render: crate::DirectionArrowCtx,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    hovered: Res<crate::HoveredHex>,
    existing: Query<Entity, With<MeleeDirectionArrow>>,
) {
    let crate::DirectionArrowCtx {
        arrow_assets,
        hex: crate::HexRender { assets: hex_assets, layout, overlay },
    } = render;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);

    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) {
        return;
    }
    let Some((attacker, attacker_hex)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(target) = hovered.0 else {
        return;
    };
    if gs.0.can_melee(attacker, target).is_err() {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    let from = hex_world_pos(attacker_hex, origin, &overlay.params);
    let to = hex_world_pos(target, origin, &overlay.params);
    let delta = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
    let len = delta.length();
    if len < f32::EPSILON {
        return;
    }
    let dir = delta / len;
    let inset = size * 0.18;
    let draw_len = (len - inset).max(len * 0.4);
    let tail = from + dir * ((len - draw_len) * 0.5);
    commands.spawn((
        MeleeDirectionArrow,
        Mesh3d(arrow_assets.mesh.clone()),
        MeshMaterial3d(hex_assets.orange.clone()),
        Transform::from_xyz(tail.x, 1.55, tail.z)
            .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
            .with_scale(Vec3::new(size * 0.5, 1.0, draw_len)),
        Visibility::Visible,
    ));
}

/// Melee combat preview: while a melee-capable unit is selected during the
/// Melee phase, show what the attack on the *hovered* hex would be -- attacker
/// and defender sides, modifiers, and expected outcomes.
pub fn melee_combat_preview_ui(
    mut contexts: EguiContexts,
    state: Res<PickerState>,
    game_state: Option<Res<GameStateResource>>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    hovered: Res<crate::HoveredHex>,
) {
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) {
        return;
    }
    if gs.0.pending_melee.is_some() {
        return; // already declared -- show reaction UI instead
    }
    let Some(target) = hovered.0 else { return };
    let Some((attacker, attacker_hex)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    if gs.0.can_melee(attacker, target).is_err() {
        return;
    }
    let Some(attack) = build_melee_attack(&gs.0, attacker_hex, target) else {
        return;
    };

    // Collect attacker and defender details.
    let atk_details: Vec<String> = attack
        .attackers
        .iter()
        .filter_map(|id| gs.0.find_unit(*id))
        .map(|u| {
            let mf = u.profile.melee.map(|m| m.value()).unwrap_or(0);
            format!("{}: {}", u.profile.identity.short_label(), mf)
        })
        .collect();
    let def_details: Vec<String> = attack
        .defenders
        .iter()
        .filter_map(|id| gs.0.find_unit(*id))
        .map(|u| {
            let mf = u.profile.melee.map(|m| m.value()).unwrap_or(0);
            format!("{}: {}", u.profile.identity.short_label(), mf)
        })
        .collect();

    let atk_total: u16 = attack
        .attackers
        .iter()
        .filter_map(|id| gs.0.find_unit(*id))
        .filter_map(|u| u.profile.melee)
        .map(|m| m.value())
        .sum();
    let def_total: u16 = attack
        .defenders
        .iter()
        .filter_map(|id| gs.0.find_unit(*id))
        .filter_map(|u| u.profile.melee)
        .map(|m| m.value())
        .sum();

    let atk_mod: i16 = attack.attacker_modifiers.iter().map(|m| m.die_modifier()).sum();
    let def_mod: i16 = attack.defender_modifiers.iter().map(|m| m.die_modifier()).sum();

    // Per-modifier detail with rulebook § citations.
    let atk_mod_lines: Vec<String> = attack
        .attacker_modifiers
        .iter()
        .map(|m| match m {
            MeleeModifier::DervishStandard => "+2 Dervish standard (\u{00a7}7.7)".to_string(),
            MeleeModifier::AngloEgyptianStandard => "+1 A-E standard (\u{00a7}7.7)".to_string(),
            MeleeModifier::DervishVsTrenchedDefender => {
                "-2 vs trenched defender (\u{00a7}9.232)".to_string()
            }
        })
        .collect();
    let def_mod_lines: Vec<String> = attack
        .defender_modifiers
        .iter()
        .map(|m| match m {
            MeleeModifier::DervishStandard => "+2 Dervish standard (\u{00a7}7.7)".to_string(),
            MeleeModifier::AngloEgyptianStandard => "+1 A-E standard (\u{00a7}7.7)".to_string(),
            MeleeModifier::DervishVsTrenchedDefender => {
                "-2 vs trenched defender (\u{00a7}9.232)".to_string()
            }
        })
        .collect();

    // CRT outcome bands for both sides (shared CRT, §7.3).
    use omdurman_rules::combat_results_table::FireFactorRow;
    let atk_row = FireFactorRow::from_total(atk_total);
    let def_row = FireFactorRow::from_total(def_total);
    let atk_bands = crate::combat_predict::outcome_bands(atk_row, atk_mod);
    let def_bands = crate::combat_predict::outcome_bands(def_row, def_mod);

    let Ok(ctx) = contexts.ctx_mut() else { return };
    bevy_egui::egui::Area::new(bevy_egui::egui::Id::new("melee_preview"))
        .anchor(
            bevy_egui::egui::Align2::CENTER_TOP,
            bevy_egui::egui::Vec2::new(0.0, 44.0),
        )
        .order(bevy_egui::egui::Order::Foreground)
        .show(ctx, |ui| {
            bevy_egui::egui::Frame::new()
                .fill(bevy_egui::egui::Color32::from_rgba_unmultiplied(
                    50, 30, 10, 220,
                ))
                .corner_radius(4.0)
                .inner_margin(bevy_egui::egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id =
                        Some(bevy_egui::egui::FontId::proportional(13.0));
                    ui.colored_label(
                        bevy_egui::egui::Color32::from_rgb(235, 200, 170),
                        format!(
                            "Melee at ({},{})",
                            target.q, target.r,
                        ),
                    );

                    // Attacker side.
                    ui.colored_label(
                        bevy_egui::egui::Color32::from_rgb(180, 220, 180),
                        format!(
                            "Attacker: {} unit(s), factor {} (mod {atk_mod:+})",
                            atk_details.len(),
                            atk_total,
                        ),
                    );
                    for d in &atk_details {
                        ui.label(
                            bevy_egui::egui::RichText::new(format!("  {d}"))
                                .color(bevy_egui::egui::Color32::from_rgb(180, 180, 180))
                                .size(12.0),
                        );
                    }
                    // Per-modifier detail.
                    for line in &atk_mod_lines {
                        ui.label(
                            bevy_egui::egui::RichText::new(format!("  {line}"))
                                .color(bevy_egui::egui::Color32::from_rgb(180, 160, 140))
                                .size(11.0),
                        );
                    }
                    // Attacker outcome bands.
                    let atk_bands_str = atk_bands
                        .iter()
                        .map(|b| b.label())
                        .collect::<Vec<_>>()
                        .join("  \u{00b7}  ");
                    ui.label(
                        bevy_egui::egui::RichText::new(format!("  CRT row {atk_row:?}: {atk_bands_str}"))
                            .color(bevy_egui::egui::Color32::from_rgb(170, 200, 170))
                            .size(11.0)
                            .monospace(),
                    );

                    ui.add_space(2.0);

                    // Defender side.
                    ui.colored_label(
                        bevy_egui::egui::Color32::from_rgb(220, 180, 180),
                        format!(
                            "Defender: {} unit(s), factor {} (mod {def_mod:+})",
                            def_details.len(),
                            def_total,
                        ),
                    );
                    for d in &def_details {
                        ui.label(
                            bevy_egui::egui::RichText::new(format!("  {d}"))
                                .color(bevy_egui::egui::Color32::from_rgb(180, 180, 180))
                                .size(12.0),
                        );
                    }
                    // Per-modifier detail.
                    for line in &def_mod_lines {
                        ui.label(
                            bevy_egui::egui::RichText::new(format!("  {line}"))
                                .color(bevy_egui::egui::Color32::from_rgb(180, 160, 140))
                                .size(11.0),
                        );
                    }
                    // Defender outcome bands.
                    let def_bands_str = def_bands
                        .iter()
                        .map(|b| b.label())
                        .collect::<Vec<_>>()
                        .join("  \u{00b7}  ");
                    ui.label(
                        bevy_egui::egui::RichText::new(format!("  CRT row {def_row:?}: {def_bands_str}"))
                            .color(bevy_egui::egui::Color32::from_rgb(200, 170, 170))
                            .size(11.0)
                            .monospace(),
                    );

                    // Melee outcome preview.
                    ui.add_space(2.0);
                    ui.colored_label(
                        bevy_egui::egui::Color32::from_rgb(200, 200, 200),
                        bevy_egui::egui::RichText::new(
                            "Both sides roll d10 + modifier on CRT simultaneously;\n\
                             losses applied at same time \u{2014} eliminated units still roll (\u{00a7}7.3)."
                        )
                        .size(12.0),
                    );
                });
        });
}

// -- Advance-after-combat target highlighting -------------------------------

#[derive(Component)]
pub(crate) struct AdvanceTargetRing;

/// Highlight adjacent empty hexes the selected unit may advance into after
/// combat (§6.82, §7.6) during OffensiveFire or Melee phases.
pub fn advance_target_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<AdvanceTargetRing>>,
) {
    let crate::HexRender { assets, layout, overlay } = hex;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);

    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee | Phase::OffensiveFire(_)) {
        return;
    }
    let Some((unit_id, _)) = selected_unit_id(&state, &placed_units) else {
        return;
    };

    let unit = match gs.0.find_unit(unit_id) {
        Some(u) => u,
        None => return,
    };
    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for hex in unit.position.neighbors() {
        if gs.0.can_advance_after_combat(unit_id, hex).is_ok() {
            let pos = hex_world_pos(hex, origin, &overlay.params);
            commands.spawn((
                AdvanceTargetRing,
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.light_green.clone()),
                Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
                Visibility::Visible,
            ));
        }
    }
}

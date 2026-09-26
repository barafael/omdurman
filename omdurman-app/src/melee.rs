//! Melee combat -- adjacent target selection and `GameEffect::MeleeCombat`
//! emission (§7).
//!
//! When a hex is selected during the Melee phase — a double-click anywhere on
//! it selects the whole tile as the attacking group
//! ([`PickerState::SelectedTile`], the unified combat selection; a
//! single-clicked counter works too) — and the rules engine permits it
//! ([`GameState::can_melee`]), adjacent enemy-occupied hexes are highlighted.
//! Clicking one builds a [`MeleeAttack`] -- the co-stacked melee-capable
//! attackers vs. the defenders in the target hex, with the standard side
//! modifiers (Dervish +2, Anglo-Egyptian +1, §7.7) -- pre-rolls both dice,
//! and broadcasts a [`GameEffect::MeleeCombat`].

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::GameEvent;
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{MeleeModifier, Phase, UnitId};
use omdurman_types::HexCoord;

use crate::{
    GameRng, GameStateResource,
    input::CombatClickCtx,
    peers::Peers,
    picker::{PickerState, PlacedUnit, selected_origin_hex, selected_unit_ids},
};

/// The acting melee group of the current selection: the origin hex plus a
/// melee-capable representative. A combat-phase double-click selects the
/// whole tile ([`PickerState::SelectedTile`]) and the engine's
/// `build_melee_attack` gathers exactly these co-stacked attackers — so one
/// representative suffices for `can_melee` while the attack still carries the
/// whole tile. A single-clicked counter resolves to itself.
fn selected_melee_group(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    gs: &GameState,
) -> Option<(UnitId, HexCoord)> {
    let origin = selected_origin_hex(state)?;
    selected_unit_ids(state, placed_units)
        .into_iter()
        .find(|&id| {
            gs.find_unit(id)
                .is_some_and(|u| u.profile.kind.may_melee_attack() && !u.state.disrupted)
        })
        .map(|attacker| (attacker, origin))
}

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
    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing.iter());
    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) {
        return;
    }
    let Some((attacker, _)) = selected_melee_group(&state, &placed_units, &gs.0) else {
        return;
    };

    for target in valid_target_hexes(attacker, &gs.0) {
        rings.ring(MeleeTargetRing, target, 1.5, 1.0, &hex.assets.orange);
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
    mut submit: crate::submit::CheckedSubmit,
    peers: Peers,
) {
    let Some(target) = click.clicked_hex() else {
        return;
    };
    let (Some(gs), Some(rng)) = (game_state, rng.as_mut()) else {
        return;
    };
    // (Phase gate: the `in_melee_phase` run condition on registration; see
    // `ui_phase_state`.)
    // One declaration at a time: a melee already awaiting resolution must be
    // resolved (after the retreat window) before another is declared.
    if gs.0.pending_melee.is_some() {
        return;
    }
    // Only the active player (their faction) melees this phase (§lobby).
    if !peers.may_act(gs.0.phase_player()) {
        return;
    }
    let Some((attacker, attacker_hex)) = selected_melee_group(&state, &placed_units, &gs.0) else {
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
    submit.submit(
        &gs.0,
        GameEvent::Effect(GameEffect::DeclareMelee {
            attack,
            attacker_roll,
            defender_roll,
        }),
    );

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
    mut submit: crate::submit::CheckedSubmit,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Some(gs) = game_state else { return };
    let Some(pm) = &gs.0.pending_melee else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let attacker_player = pm.attack.attacker_player;
    let local_is_attacker = peers.may_act(attacker_player);
    let target = pm.attack.defender_hex;

    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("melee_declared"),
        crate::ui::frames::card(crate::ui::palette::CARD_MELEE_DECLARED),
        |ui| {
            ui.colored_label(
                crate::ui::palette::DEFENDER,
                format!(
                    "\u{2694} {attacker_player} melee on hex ({}, {})",
                    target.q, target.r
                ),
            );
            if local_is_attacker {
                ui.label("Defenders may retreat. Resolve when ready.");
                if ui.button("\u{2694} Resolve Melee").clicked() {
                    submit.submit(&gs.0, GameEvent::Effect(GameEffect::ResolveMelee));
                }
            } else {
                ui.label("You may retreat the threatened cavalry/camel (click a");
                ui.label("highlighted hex), or wait for the attacker to resolve.");
            }
        },
    );
}

/// Attack construction lives in the engine (`omdurman_rules::effects::
/// build_melee_attack`, shared verbatim with the bot); imported so the click
/// gate and the preview panel use the same builder.
use omdurman_rules::effects::build_melee_attack;

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
    mut submit: crate::submit::CheckedSubmit,
) {
    let Some(to) = click.clicked_hex() else {
        return;
    };
    let Some(gs) = game_state else { return };
    // §6.7: no advance after combat from defensive fire -- only after melee
    // (§7.6) and offensive fire (§6.82). (Phase gate: the
    // `in_offensive_fire_or_melee_phase` run condition on registration; see
    // `ui_phase_state`.) Any member of the tile may be the advancer — the
    // engine re-validates participation and eligibility (artillery, forts)
    // per unit.
    let candidates = selected_unit_ids(&state, &placed_units);
    let Some(unit_id) = candidates
        .into_iter()
        .find(|&unit_id| gs.0.can_advance_after_combat(unit_id, to).is_ok())
    else {
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
    submit.submit(
        &gs.0,
        GameEvent::Effect(GameEffect::AdvanceAfterCombat { unit_id, to }),
    );
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
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);

    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee) {
        return;
    }
    let Some((attacker, attacker_hex)) = selected_melee_group(&state, &placed_units, &gs.0) else {
        return;
    };
    let Some(target) = hovered.0 else {
        return;
    };
    if gs.0.can_melee(attacker, target).is_err() {
        return;
    }

    crate::combat_ui::direction_arrow(
        &mut commands,
        &render,
        attacker_hex,
        target,
        MeleeDirectionArrow,
    );
}

/// Melee combat preview: while a melee-capable unit is selected during the
/// Melee phase, show what the attack on the *hovered* hex would be -- attacker
/// and defender sides, modifiers, and expected outcomes. (Phase gate: the
/// `in_melee_phase` run condition on registration; see `ui_phase_state`.)
pub fn melee_combat_preview_ui(
    mut contexts: EguiContexts,
    state: Res<PickerState>,
    game_state: Option<Res<GameStateResource>>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    hovered: Res<crate::HoveredHex>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let Some(gs) = game_state else { return };
    if gs.0.pending_melee.is_some() {
        return; // already declared -- show reaction UI instead
    }
    let Some(target) = hovered.0 else { return };
    let Some((attacker, attacker_hex)) = selected_melee_group(&state, &placed_units, &gs.0) else {
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

    let atk_mod: i16 = attack
        .attacker_modifiers
        .iter()
        .map(|m| m.die_modifier())
        .sum();
    let def_mod: i16 = attack
        .defender_modifiers
        .iter()
        .map(|m| m.die_modifier())
        .sum();

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
    use bevy_egui::egui;
    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("melee_preview"),
        crate::ui::frames::card(crate::ui::palette::CARD_MELEE),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));
            ui.colored_label(
                crate::ui::palette::CARD_TITLE,
                format!("Melee at ({},{})", target.q, target.r,),
            );

            // Attacker side.
            ui.colored_label(
                crate::ui::palette::ATTACKER,
                format!(
                    "Attacker: {} unit(s), factor {} (mod {atk_mod:+})",
                    atk_details.len(),
                    atk_total,
                ),
            );
            for d in &atk_details {
                ui.label(
                    bevy_egui::egui::RichText::new(format!("  {d}"))
                        .color(crate::ui::palette::TEXT_SOFT)
                        .size(12.0),
                );
            }
            // Per-modifier detail.
            for line in &atk_mod_lines {
                ui.label(
                    bevy_egui::egui::RichText::new(format!("  {line}"))
                        .color(crate::ui::palette::PANEL_DIM)
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
                    .color(crate::ui::palette::FAVOURABLE)
                    .size(11.0)
                    .monospace(),
            );

            ui.add_space(2.0);

            // Defender side.
            ui.colored_label(
                crate::ui::palette::DEFENDER,
                format!(
                    "Defender: {} unit(s), factor {} (mod {def_mod:+})",
                    def_details.len(),
                    def_total,
                ),
            );
            for d in &def_details {
                ui.label(
                    bevy_egui::egui::RichText::new(format!("  {d}"))
                        .color(crate::ui::palette::TEXT_SOFT)
                        .size(12.0),
                );
            }
            // Per-modifier detail.
            for line in &def_mod_lines {
                ui.label(
                    bevy_egui::egui::RichText::new(format!("  {line}"))
                        .color(crate::ui::palette::PANEL_DIM)
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
                    .color(crate::ui::palette::DEFENDER_DIM)
                    .size(11.0)
                    .monospace(),
            );

            // Melee outcome preview.
            ui.add_space(2.0);
            ui.colored_label(
                        crate::ui::palette::TEXT,
                        bevy_egui::egui::RichText::new(
                            "Both sides roll d10 + modifier on CRT simultaneously;\n\
                             losses applied at same time \u{2014} eliminated units still roll (\u{00a7}7.3)."
                        )
                        .size(12.0),
                    );
        },
    );
}

// -- Advance-after-combat target highlighting -------------------------------

#[derive(Component)]
pub(crate) struct AdvanceTargetRing;

/// Highlight adjacent empty hexes the selected tile may advance into after
/// combat (§6.82, §7.6) during OffensiveFire or Melee phases. The union over
/// the tile's members — an artillery counter cannot advance, its co-stacked
/// infantry can — matches what a click accepts (`handle_advance_after_combat`
/// advances the first member the engine accepts for the clicked hex).
pub fn advance_target_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<AdvanceTargetRing>>,
) {
    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing.iter());

    let Some(gs) = game_state else { return };
    if !matches!(gs.0.phase, Phase::Melee | Phase::OffensiveFire(_)) {
        return;
    }
    let candidates = selected_unit_ids(&state, &placed_units);
    // All members share the origin hex, so the neighbour set is the same;
    // only per-unit eligibility differs.
    let Some(any_unit) = candidates.iter().find_map(|&id| gs.0.find_unit(id)) else {
        return;
    };
    for target in any_unit.position.neighbors() {
        if candidates
            .iter()
            .any(|&unit_id| gs.0.can_advance_after_combat(unit_id, target).is_ok())
        {
            rings.ring(AdvanceTargetRing, target, 1.5, 1.0, &hex.assets.light_green);
        }
    }
}

//! Melee combat -- adjacent target selection and `GameEffect::DeclareMelee`
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
//! and broadcasts a [`GameEffect::DeclareMelee`].

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::GameEvent;
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{MeleeModifier, Phase, UnitId};
use omdurman_types::HexCoord;

use crate::{
    GameRng, GameStateResource,
    board_click::{AdvanceClick, MeleeClick},
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
    // A representative that may melee something now (§7.4/§7.5: not
    // disrupted, not spent, not busy building or demolishing).
    selected_unit_ids(state, placed_units)
        .into_iter()
        .find(|&id| {
            origin
                .neighbors()
                .iter()
                .any(|n| gs.can_melee(id, *n).is_ok())
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
/// selected during the Melee phase, broadcast a `DeclareMelee` effect with both
/// pre-rolled dice.
pub fn handle_melee_combat(
    mut clicks: bevy::ecs::message::MessageReader<MeleeClick>,
    mut state: ResMut<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    mut rng: Option<ResMut<GameRng>>,
    mut submit: crate::submit::CheckedSubmit,
    peers: Peers,
) {
    // Routed by `board_click::route_board_clicks` (Melee-phase release, no
    // melee pending, the phase player's seat).
    let Some(&MeleeClick(target)) = clicks.read().last() else {
        return;
    };
    let (Some(gs), Some(rng)) = (game_state, rng.as_mut()) else {
        return;
    };
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
    // A refused attack on an enemy-held hex says why; clicks elsewhere stay
    // quiet (they are selections, not declarations).
    if let Err(error) = gs.0.can_melee(attacker, target) {
        info!(target.q = target.q, target.r = target.r, %error, "melee refused");
        let owner = gs.0.find_unit(attacker).map(|a| a.profile.identity.owner());
        let enemy_there =
            gs.0.units_in_hex(target)
                .iter()
                .any(|u| Some(u.profile.identity.owner()) != owner);
        if enemy_there && let Some(dispatches) = submit.dispatches.as_deref_mut() {
            dispatches.push("Field Telegraph", format!("Melee refused — {error}."));
        }
        return;
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
    let retreat_possible = defenders_may_retreat(&gs.0, &pm.attack);

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
            let withdrawn = gs.0.melee_defenders_now(&pm.attack).is_empty();
            if local_is_attacker {
                ui.label(if withdrawn {
                    "The defenders withdrew (§7.5): resolving ends the attack."
                } else if retreat_possible {
                    "Defenders may retreat. Resolve when ready."
                } else {
                    "Resolve when ready."
                });
                if ui.button("\u{2694} Resolve Melee").clicked() {
                    submit.submit(&gs.0, GameEvent::Effect(GameEffect::ResolveMelee));
                }
            } else if withdrawn {
                ui.label("Your units withdrew (§7.5). Waiting for the attacker.");
            } else if retreat_possible {
                ui.label("You may retreat threatened cavalry/camel: click the");
                ui.label("attacked hex, then a highlighted hex two away.");
                ui.label("Or wait for the attacker to resolve.");
            } else {
                ui.label("Waiting for the attacker to resolve.");
            }
        },
    );
}

/// Whether the defenders of a declared melee have a §7.5 retreat to make:
/// some unit on the target hex the engine lets retreat somewhere (a cavalry
/// or camel unit, undisrupted, not yet retreated, under an infantry attack,
/// with an open two-hex path).
pub(crate) fn defenders_may_retreat(
    gs: &omdurman_rules::effects::GameState,
    attack: &omdurman_rules::MeleeAttack,
) -> bool {
    crate::retreat::retreat_candidate_at(gs, attack.defender_hex).is_some()
}

/// Attack construction lives in the engine (`omdurman_rules::effects::
/// build_melee_attack`, shared verbatim with the bot); imported so the click
/// gate and the preview panel use the same builder.
use omdurman_rules::effects::build_melee_attack;

/// The selected units that advance into `to` with one click: every eligible
/// one, each checked against the state after the ones before it moved in, so
/// the stack fills the vacated hex up to the stacking limit (§6.82/§7.6).
fn advancing_units(gs: &GameState, candidates: &[UnitId], to: HexCoord) -> Vec<UnitId> {
    let mut after = gs.clone();
    candidates
        .iter()
        .copied()
        .filter(|&unit_id| {
            omdurman_rules::effects::apply_effect(
                &mut after,
                &GameEffect::AdvanceAfterCombat { unit_id, to },
            )
            .is_ok()
        })
        .collect()
}

/// Advance after combat (§6.82, §7.6): during a combat phase, with the
/// active player's units selected, clicking an adjacent hex that the engine
/// accepts (vacated, no enemy in it, the unit isn't artillery) advances every
/// selected unit that may go, up to the stacking limit. Targets hexes free of
/// the enemy, so it never collides with the fire/melee attack handlers (which
/// target enemy-occupied hexes).
pub fn handle_advance_after_combat(
    mut clicks: bevy::ecs::message::MessageReader<AdvanceClick>,
    mut state: ResMut<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    mut submit: crate::submit::CheckedSubmit,
) {
    // Routed by `board_click::route_board_clicks` when a selected unit may
    // enter the clicked hex.
    let Some(&AdvanceClick(to)) = clicks.read().last() else {
        return;
    };
    let Some(gs) = game_state else { return };
    // §6.7: no advance after combat from defensive fire -- only after melee
    // (§7.6) and offensive fire (§6.82). (Phase gate: the click router only
    // routes advances in those phases; the engine re-checks.) Any member of the tile may be the advancer — the
    // engine re-validates participation and eligibility (artillery, forts)
    // per unit.
    // `can_advance_after_combat` (run by each advance) also checks hexside
    // blocking (§6.82/§7.6) via `self.board`.
    let candidates = selected_unit_ids(&state, &placed_units);
    let advancers = advancing_units(&gs.0, &candidates, to);
    if advancers.is_empty() {
        return;
    }
    for unit_id in advancers {
        info!(?unit_id, to.q = to.q, to.r = to.r, "advance after combat");
        submit.submit(
            &gs.0,
            GameEvent::Effect(GameEffect::AdvanceAfterCombat { unit_id, to }),
        );
    }
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
    mut sticky: Local<Option<omdurman_types::HexCoord>>,
) {
    let Some(gs) = game_state else { return };
    if gs.0.pending_melee.is_some() {
        return; // already declared -- show reaction UI instead
    }
    // Follows the hovered hex; stays up while the pointer is on the card, so
    // its § links can be followed.
    let Some(target) = crate::fire::sticky_preview_target(
        &mut contexts,
        hovered.0,
        &mut sticky,
        &["melee_preview"],
    ) else {
        return;
    };
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
            let name = u.profile.identity.short_label();
            if u.state.disrupted {
                // Disrupted units may not melee (reference notes).
                format!("{name}: 0 (disrupted)")
            } else {
                let mf = u.profile.melee.map(|m| m.value()).unwrap_or(0);
                format!("{name}: {mf}")
            }
        })
        .collect();

    // The engine's own totals (§7.7): disrupted units add nothing.
    let atk_total = omdurman_rules::effects::melee_strength(&gs.0, &attack.attackers);
    let def_total = omdurman_rules::effects::melee_strength(&gs.0, &attack.defenders);

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
    let mod_lines = |mods: &[MeleeModifier]| -> Vec<String> {
        mods.iter()
            .map(|m| {
                let line = crate::combat_ui::describe_melee_modifier(*m);
                format!("{} (\u{00a7}{})", line.label, line.paragraph)
            })
            .collect()
    };
    let atk_mod_lines = mod_lines(&attack.attacker_modifiers);
    let def_mod_lines = mod_lines(&attack.defender_modifiers);

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
                crate::rulebook::refs_label(
                    ui,
                    &format!("  {line}"),
                    crate::ui::palette::PANEL_DIM,
                    11.0,
                );
            }
            // Attacker outcome bands.
            let atk_bands_str = atk_bands
                .iter()
                .map(|b| b.label())
                .collect::<Vec<_>>()
                .join("  \u{00b7}  ");
            ui.label(
                bevy_egui::egui::RichText::new(format!(
                    "  CRT row {}: {atk_bands_str}",
                    atk_row.label()
                ))
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
                crate::rulebook::refs_label(
                    ui,
                    &format!("  {line}"),
                    crate::ui::palette::PANEL_DIM,
                    11.0,
                );
            }
            // Defender outcome bands.
            let def_bands_str = def_bands
                .iter()
                .map(|b| b.label())
                .collect::<Vec<_>>()
                .join("  \u{00b7}  ");
            ui.label(
                bevy_egui::egui::RichText::new(format!(
                    "  CRT row {}: {def_bands_str}",
                    def_row.label()
                ))
                .color(crate::ui::palette::DEFENDER_DIM)
                .size(11.0)
                .monospace(),
            );

            // Melee outcome preview.
            ui.add_space(2.0);
            crate::rulebook::refs_label(
                ui,
                "Both sides roll d10 + modifier on CRT simultaneously; losses applied at \
                 the same time \u{2014} eliminated units still roll (\u{00a7}7.3).",
                crate::ui::palette::TEXT,
                12.0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_rules::{FireSubPhase, UnitPlacement};
    use omdurman_types::{Player, Scenario};

    /// §6.82/§7.6: one click advances the whole selected stack into the
    /// vacated hex, up to the four-unit limit (§5.51) -- not just its first
    /// eligible unit.
    #[test]
    fn one_click_advances_the_stack_up_to_the_stacking_limit() {
        let mut gs = GameState::new(Scenario::Campaign);
        gs.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        gs.active_player = Player::Dervish;
        let to = HexCoord::new(5, 5);
        let firers = [
            (UnitId::MulazminI_0_0, HexCoord::new(4, 5)),
            (UnitId::MulazminI_0_1, HexCoord::new(4, 5)),
            (UnitId::MulazminI_1_0, HexCoord::new(4, 5)),
            (UnitId::MulazminI_1_1, HexCoord::new(6, 5)),
            (UnitId::MulazminI_2_0, HexCoord::new(6, 5)),
        ];
        for (id, position) in firers {
            gs.units.push(UnitPlacement {
                id,
                position,
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: Default::default(),
            });
        }
        let ids: Vec<UnitId> = firers.iter().map(|(id, _)| *id).collect();
        gs.vacated_by_combat.insert(to, ids.clone());

        assert_eq!(advancing_units(&gs, &ids, to), ids[..4].to_vec());
    }
}

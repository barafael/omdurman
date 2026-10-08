//! Melee combat -- adjacent target selection and `GameEffect::DeclareMelee`
//! emission (§7).
//!
//! When a hex is selected during the Melee phase — a double-click anywhere on
//! it selects the whole tile as the attacking group
//! ([`PickerState::SelectedTile`], the unified combat selection; a
//! single-clicked counter works too) — and the rules engine permits it
//! ([`GameState::can_melee`]), adjacent enemy-occupied hexes are highlighted.
//! Clicking one builds a [`MeleeAttack`] -- the selection's melee-capable
//! attackers (the whole tile, or a single-clicked counter alone: melee is
//! each unit's own choice, §7.4, and its stackmates stay out) vs. the
//! defenders in the target hex, with the standard side modifiers (Dervish
//! +2, Anglo-Egyptian +1, §7.7) -- pre-rolls both dice, and broadcasts a
//! [`GameEffect::DeclareMelee`].

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::GameEvent;
use omdurman_rules::effects::{GameEffect, GameState, build_melee_attack_from};
use omdurman_rules::{MeleeModifier, Phase, UnitId};
use omdurman_types::HexCoord;

use crate::{
    GameRng, GameStateResource,
    board_click::{AdvanceClick, MeleeClick},
    peers::Peers,
    picker::{PickerState, PlacedUnit, selected_origin_hex, selected_unit_ids},
};

/// The members of the current selection that may melee `target` now
/// (§7.4/§7.5, as the engine judges each): every melee-capable unit of a
/// double-clicked tile ([`PickerState::SelectedTile`]), or a single-clicked
/// counter alone -- melee is each unit's own choice, so the counter's
/// stackmates stay out of the attack (§7.7 takes "losses from meleeing units
/// first"). The declared attack carries exactly these.
fn selected_melee_attackers(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    gs: &GameState,
    target: HexCoord,
) -> Vec<UnitId> {
    selected_unit_ids(state, placed_units)
        .into_iter()
        .filter(|&id| gs.can_melee(id, target).is_ok())
        .collect()
}

/// The acting melee group of the current selection: the origin hex plus a
/// melee-capable representative, for the target rings and the direction
/// arrow (`can_melee` per candidate target). The attack itself is built from
/// [`selected_melee_attackers`].
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

/// What [`melee_target_overlay_mesh`] last drew: the representative attacker
/// and the target hexes the current rings describe. Rebuilt only when the
/// selection, the engine state, or a counter's placement data changed (or the
/// overlays were cleared, see [`crate::picker::OverlayGeneration`]) -- not
/// every frame.
type MeleeOverlayCache = (UnitId, Vec<HexCoord>);

#[allow(clippy::too_many_arguments)]
pub fn melee_target_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<MeleeTargetRing>>,
    mut last: Local<Option<MeleeOverlayCache>>,
    placed_changed: Query<(), Changed<PlacedUnit>>,
    (generation, mut seen_generation): (Res<crate::picker::OverlayGeneration>, Local<u32>),
) {
    let invalidated = generation.invalidates(&mut seen_generation);
    if invalidated {
        // The overlays were cleared (`clear_gameplay_overlays`): forget what
        // we last drew so the "unchanged" checks below rebuild the rings
        // instead of silently leaving them despawned.
        *last = None;
    }
    let inputs_moved = invalidated
        || game_state.as_ref().is_some_and(|gs| gs.is_changed())
        || state.is_changed()
        || !placed_changed.is_empty();
    if !inputs_moved {
        return;
    }
    let drawn = game_state.as_deref().and_then(|gs| {
        if !matches!(gs.0.phase, Phase::Melee) {
            return None;
        }
        let (attacker, _) = selected_melee_group(&state, &placed_units, &gs.0)?;
        Some((attacker, valid_target_hexes(attacker, &gs.0)))
    });
    let Some(drawn) = drawn else {
        // Nothing to highlight: clear any rings left over from a selection
        // that can no longer melee (once, not every frame).
        if last.is_some() {
            let old: Vec<Entity> = existing.iter().collect();
            crate::ui::despawn_all(&mut commands, &old);
            *last = None;
        }
        return;
    };
    // Same attacker, same targets: leave the existing rings in place.
    if last.as_ref() == Some(&drawn) {
        return;
    }
    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing.iter());
    for &target in &drawn.1 {
        rings.ring(MeleeTargetRing, target, 1.5, 1.0, &hex.assets.orange);
    }
    *last = Some(drawn);
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
    let Some((attacker, _)) = selected_melee_group(&state, &placed_units, &gs.0) else {
        return;
    };

    // Only a click on an enemy-held hex is an attempted melee; any other
    // click -- the attacker's own hex (the second click of the double-click
    // that selects the tile), an empty or friendly hex -- is a selection, not
    // a declaration, and stays quiet.
    let owner = gs.0.find_unit(attacker).map(|a| a.profile.identity.owner());
    let enemy_there =
        gs.0.units_in_hex(target)
            .iter()
            .any(|u| Some(u.profile.identity.owner()) != owner);
    if !enemy_there {
        return;
    }
    // `can_melee` checks hexside blocking (§7.2) internally via `self.board`.
    // A refused attack on an enemy-held hex says why.
    if let Err(error) = gs.0.can_melee(attacker, target) {
        info!(target.q = target.q, target.r = target.r, %error, "melee refused");
        if let Some(dispatches) = submit.dispatches.as_deref_mut() {
            dispatches.push("Field Telegraph", format!("Melee refused — {error}."));
        }
        return;
    }

    let attackers = selected_melee_attackers(&state, &placed_units, &gs.0, target);
    let Some(attack) = build_melee_attack_from(&gs.0, &attackers, target) else {
        return;
    };
    let attacker_roll = rng.roll_d10();
    let defender_roll = rng.roll_d10();
    let disruption = rng.disruption_draw();

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
            disruption,
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
/// Rebuilt only when the arrow's endpoints change (selection, hover, engine
/// state, or the overlays were cleared) -- not every frame.
#[allow(clippy::too_many_arguments)]
pub fn melee_direction_arrow(
    mut commands: Commands,
    render: crate::DirectionArrowCtx,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    hovered: Res<crate::HoveredHex>,
    existing: Query<Entity, With<MeleeDirectionArrow>>,
    mut last: Local<Option<Option<(HexCoord, HexCoord)>>>,
    placed_changed: Query<(), Changed<PlacedUnit>>,
    (generation, mut seen_generation): (Res<crate::picker::OverlayGeneration>, Local<u32>),
) {
    let invalidated = generation.invalidates(&mut seen_generation);
    if invalidated {
        // The overlays were cleared (`clear_gameplay_overlays`): forget the
        // arrow we last drew so the "unchanged" check below respawns it
        // instead of silently leaving it despawned.
        *last = None;
    }
    let inputs_moved = invalidated
        || game_state.as_ref().is_some_and(|gs| gs.is_changed())
        || state.is_changed()
        || hovered.is_changed()
        || !placed_changed.is_empty();
    if !inputs_moved {
        return;
    }
    let arrow = game_state.as_deref().and_then(|gs| {
        if !matches!(gs.0.phase, Phase::Melee) {
            return None;
        }
        let (attacker, attacker_hex) = selected_melee_group(&state, &placed_units, &gs.0)?;
        let target = hovered.0?;
        gs.0.can_melee(attacker, target)
            .is_ok()
            .then_some((attacker_hex, target))
    });
    if *last == Some(arrow) {
        return; // unchanged: leave the drawn arrow in place
    }
    let old: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &old);
    *last = Some(arrow);
    if let Some((from, to)) = arrow {
        crate::combat_ui::direction_arrow(&mut commands, &render, from, to, MeleeDirectionArrow);
    }
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
    let Some((attacker, _)) = selected_melee_group(&state, &placed_units, &gs.0) else {
        return;
    };
    if gs.0.can_melee(attacker, target).is_err() {
        return;
    }
    let attackers = selected_melee_attackers(&state, &placed_units, &gs.0, target);
    let Some(attack) = build_melee_attack_from(&gs.0, &attackers, target) else {
        return;
    };

    // Collect attacker and defender details.
    let atk_details: Vec<String> = attack
        .attackers
        .iter()
        .filter_map(|id| gs.0.find_unit(*id))
        .map(|u| {
            let mf = u.profile.melee.map(|m| m.value()).unwrap_or(0);
            format!("{}: {}", u.profile.identity.label_in(gs.0.scenario), mf)
        })
        .collect();
    let def_details: Vec<String> = attack
        .defenders
        .iter()
        .filter_map(|id| gs.0.find_unit(*id))
        .map(|u| {
            let name = u.profile.identity.label_in(gs.0.scenario);
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
    crate::ui::passive_stacked_card(
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
/// advances the first member the engine accepts for the clicked hex). Rebuilt
/// only when the selection or the engine state changed (or the overlays were
/// cleared) -- not every frame.
#[allow(clippy::too_many_arguments)]
pub fn advance_target_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<AdvanceTargetRing>>,
    mut last: Local<Option<Vec<HexCoord>>>,
    placed_changed: Query<(), Changed<PlacedUnit>>,
    (generation, mut seen_generation): (Res<crate::picker::OverlayGeneration>, Local<u32>),
) {
    let invalidated = generation.invalidates(&mut seen_generation);
    if invalidated {
        // The overlays were cleared (`clear_gameplay_overlays`): forget what
        // we last drew so the "unchanged" checks below rebuild the rings
        // instead of silently leaving them despawned.
        *last = None;
    }
    let inputs_moved = invalidated
        || game_state.as_ref().is_some_and(|gs| gs.is_changed())
        || state.is_changed()
        || !placed_changed.is_empty();
    if !inputs_moved {
        return;
    }
    let drawn = game_state.as_deref().and_then(|gs| {
        if !matches!(gs.0.phase, Phase::Melee | Phase::OffensiveFire(_)) {
            return None;
        }
        let candidates = selected_unit_ids(&state, &placed_units);
        // All members share the origin hex, so the neighbour set is the same;
        // only per-unit eligibility differs.
        let any_unit = candidates.iter().find_map(|&id| gs.0.find_unit(id))?;
        Some(
            any_unit
                .position
                .neighbors()
                .into_iter()
                .filter(|&target| {
                    candidates
                        .iter()
                        .any(|&unit_id| gs.0.can_advance_after_combat(unit_id, target).is_ok())
                })
                .collect::<Vec<HexCoord>>(),
        )
    });
    let Some(drawn) = drawn else {
        // Nothing to highlight (no selection / wrong phase): clear any rings
        // left over, once.
        if last.is_some() {
            let old: Vec<Entity> = existing.iter().collect();
            crate::ui::despawn_all(&mut commands, &old);
            *last = None;
        }
        return;
    };
    // Same targets: leave the existing rings in place.
    if last.as_ref() == Some(&drawn) {
        return;
    }
    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing.iter());
    for &target in &drawn {
        rings.ring(AdvanceTargetRing, target, 1.5, 1.0, &hex.assets.light_green);
    }
    *last = Some(drawn);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::picker::{OverlayGeneration, TileSelection};
    use crate::render::{HexOverlay, HexRingAssets};
    use bevy::ecs::system::RunSystemOnce;
    use omdurman_rules::{FireSubPhase, UnitPlacement};
    use omdurman_types::{Player, Scenario, SectionName};

    /// A single-clicked counter melees alone (§7.4: melee is each unit's
    /// choice), the double-clicked tile melees as one -- the declared
    /// attackers follow the selection, not the whole hex.
    #[test]
    fn the_declared_attackers_follow_the_selection() {
        let mut gs = GameState::new(Scenario::Campaign);
        gs.phase = Phase::Melee;
        gs.active_player = Player::Dervish;
        let from = HexCoord::new(4, 5);
        let target = HexCoord::new(5, 5);
        let stack = [UnitId::MulazminI_0_0, UnitId::MulazminI_0_1];
        for id in stack.into_iter().chain([UnitId::BritishArmy_0_0]) {
            gs.units.push(UnitPlacement {
                id,
                position: if id == UnitId::BritishArmy_0_0 {
                    target
                } else {
                    from
                },
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: Default::default(),
            });
        }
        let mut world = World::new();
        let sources: Vec<Entity> = stack
            .iter()
            .map(|&id| {
                world
                    .spawn(PlacedUnit {
                        coord: from,
                        section_name: SectionName::MulazminI,
                        col: 0,
                        row: 0,
                        is_boat: false,
                        unit_id: Some(id),
                        disrupted: false,
                    })
                    .id()
            })
            .collect();
        let single = PickerState::Selected {
            source: sources[0],
            start_coord: from,
            remaining_mp: 0,
            forced_stop: false,
        };
        let tile = PickerState::SelectedTile(TileSelection {
            sources: sources.clone(),
            start_coord: from,
        });
        let attackers_of = |world: &mut World, state: PickerState| {
            let gs = gs.clone();
            world
                .run_system_once(move |placed: Query<(Entity, &PlacedUnit)>| {
                    selected_melee_attackers(&state, &placed, &gs, target)
                })
                .expect("the query runs")
        };
        assert_eq!(attackers_of(&mut world, single), vec![stack[0]]);
        assert_eq!(attackers_of(&mut world, tile), stack.to_vec());
    }

    /// Overlay caches must treat an [`crate::picker::OverlayGeneration`] bump
    /// as "the rings you drew are gone": after `clear_gameplay_overlays`
    /// despawns them (a round trip through the menu) and bumps the
    /// generation, an *unchanged* selection still has to respawn them --
    /// not quietly keep the cache that says they already exist.
    #[test]
    fn advance_rings_respawn_after_an_overlay_generation_bump() {
        // The same board state as the stack-advance test below: five
        // Mulazmin counters, two hexes from a combat-vacated hex, in the
        // offensive-fire phase that permits advancing (§6.82).
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

        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin)
            .insert_resource(HexOverlay::default())
            .insert_resource(omdurman_board_ui::board_store::default_layout())
            .insert_resource(hex_ring_assets())
            .insert_resource(GameStateResource(gs))
            .insert_resource(OverlayGeneration::default())
            .add_systems(Update, advance_target_overlay_mesh);

        // The selection's counters, as `reconcile_unit_sprites` would spawn
        // them.
        let mut sources = Vec::new();
        for (id, coord) in firers {
            let entity = app
                .world_mut()
                .spawn((PlacedUnit {
                    coord,
                    section_name: SectionName::MulazminI,
                    col: 0,
                    row: 0,
                    is_boat: false,
                    unit_id: Some(id),
                    disrupted: false,
                },))
                .id();
            sources.push(entity);
        }
        app.world_mut()
            .insert_resource(PickerState::SelectedTile(TileSelection {
                sources,
                start_coord: HexCoord::new(4, 5),
            }));

        let ring_count = |world: &mut World| {
            world
                .query_filtered::<Entity, With<AdvanceTargetRing>>()
                .iter(world)
                .count()
        };

        // First run: the rings are drawn.
        app.update();
        assert!(ring_count(app.world_mut()) > 0, "rings are drawn");

        // `clear_gameplay_overlays`: the rings despawn and the generation
        // moves. Nothing else changes -- same selection, same engine state.
        for entity in ring_entities(app.world_mut()) {
            app.world_mut().despawn(entity);
        }
        app.world_mut().resource_mut::<OverlayGeneration>().0 = app
            .world()
            .resource::<OverlayGeneration>()
            .0
            .wrapping_add(1);

        // The rings must come back.
        app.update();
        assert!(
            ring_count(app.world_mut()) > 0,
            "rings respawn after the overlays were cleared, despite the \
             unchanged selection"
        );
    }

    fn ring_entities(world: &mut World) -> Vec<Entity> {
        world
            .query_filtered::<Entity, With<AdvanceTargetRing>>()
            .iter(world)
            .collect()
    }

    /// `HexRingAssets` with placeholder handles: the overlay systems only
    /// clone and pass them on; nothing resolves them headless.
    fn hex_ring_assets() -> HexRingAssets {
        HexRingAssets {
            mesh: default(),
            unit_square: default(),
            red: default(),
            green: default(),
            light_green: default(),
            orange: default(),
            blue: default(),
            hover: default(),
            marker_green: default(),
            marker_red: default(),
            gray: default(),
            reach: default(),
            yellow: default(),
            path_shadow: default(),
            fire_arrow: default(),
            acted: default(),
        }
    }

    /// §6.82/§7.6: one click advances the whole selected stack into the
    /// vacated hex, up to the four-unit limit (§5.51) -- not just its first
    /// eligible unit.
    #[traceability_macro::rulebook("§5.51", "§6.82", "§7.6")]
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

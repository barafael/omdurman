//! Fire combat -- target overlay, direction arrow, and combat preview.
//!
//! When a friendly unit is selected ([`PickerState::Selected`]) during a fire
//! sub-phase and the rules engine says it may fire, enemy-occupied hexes in
//! range are highlighted. The hover preview shows the would-be attack breakdown.
//! Actual resolution now happens through the allocation system
//! ([`crate::fire_allocation`]) -- the player builds a battle plan and triggers
//! batch execution with "Execute All".
//!
//! The rules engine owns range/Combat Results Table resolution; the app supplies the terrain
//! modifier (the engine holds no map) and gates on [`GameState::can_fire_at`].

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_rules::effects::GameState;
use omdurman_rules::{FireAttack, FireFactor, FireKind, FireModifier, Phase, UnitId};
use omdurman_types::{HexCoord, Player};

use crate::peers::Peers;
use crate::picker::{PickerState, PlacedUnit, selected_unit_id};
use crate::GameStateResource;
use omdurman_hexmap::hex_world_pos;

/// Bundle of the hovered hex + the existing arrow entities so
/// [`fire_direction_arrow`] stays under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct FireArrowTarget<'w, 's> {
    pub hovered: Res<'w, crate::HoveredHex>,
    pub existing: Query<'w, 's, Entity, With<FireDirectionArrow>>,
}

/// The fire kind a firer would use in the current sub-phase (§6.42):
/// direct fire in the Direct sub-phase; in the second sub-phase a Maxim uses
/// its second fire and a named gunboat fires howitzer. Returns `None` if the
/// firer can't act in this sub-phase (e.g. a rifle unit in the second sub-
/// phase).
pub(crate) fn fire_kind_for(gs: &GameState, firer: UnitId) -> Option<FireKind> {
    use omdurman_rules::{UnitIdentity, WeaponClass};
    let unit = gs.find_unit(firer)?;
    let sub = match gs.phase {
        Phase::OffensiveFire(s) | Phase::DefensiveFire(s) => s,
        _ => return None,
    };
    match sub {
        omdurman_rules::FireSubPhase::DirectFire => Some(FireKind::Direct),
        omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => {
            // Named gunboats (§6.64) carry howitzers despite their profile
            // weapon being Artillery; query the identity, not the profile.
            let is_named_gunboat = matches!(
                unit.profile.identity,
                UnitIdentity::AngloEgyptianGunboat(gb) if gb.has_howitzer()
            );
            match unit.profile.weapon {
                WeaponClass::Maxims => Some(FireKind::MaximSecondFire),
                WeaponClass::Howitzer => Some(FireKind::Howitzer),
                _ if is_named_gunboat => Some(FireKind::Howitzer),
                _ => None,
            }
        }
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
    hex: crate::HexRender,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<FireTargetRing>>,
) {
    let crate::HexRender { assets, layout, overlay } = hex;
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

// -- Fire direction arrow: translucent red arrow from firer to hovered target ---

#[derive(Component)]
pub(crate) struct FireDirectionArrow;

/// Draw a translucent red arrow from the firer hex to the hovered valid
/// target hex, giving the player a visual preview of the fire direction.
/// Rebuilt each frame (lightweight: one arrow mesh at most).
pub fn fire_direction_arrow(
    mut commands: Commands,
    render: crate::DirectionArrowCtx,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    target: FireArrowTarget,
    peers: Peers,
) {
    let crate::DirectionArrowCtx {
        arrow_assets,
        hex: crate::HexRender { assets: hex_assets, layout, overlay },
    } = render;
    let FireArrowTarget { hovered, existing } = target;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);

    let Some(gs) = game_state else { return };
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
    if !peers.may_act(firing_player) {
        return;
    }
    let Some((firer, firer_hex)) = selected_unit_id(&state, &placed_units) else {
        return;
    };
    let Some(kind) = fire_kind_for(&gs.0, firer) else {
        return;
    };
    let Some(target) = hovered.0 else {
        return;
    };
    if gs.0.can_fire_at(firer, target, kind).is_err() {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    let from = hex_world_pos(firer_hex, origin, &overlay.params);
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
        FireDirectionArrow,
        Mesh3d(arrow_assets.mesh.clone()),
        MeshMaterial3d(hex_assets.fire_arrow.clone()),
        Transform::from_xyz(tail.x, 1.55, tail.z)
            .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
            .with_scale(Vec3::new(size * 0.5, 1.0, draw_len)),
        Visibility::Visible,
    ));
}

/// Build a combined `FireAttack` (§6.14): every friendly unit stacked in the
/// selected unit's hex (`firer_hex`) that may legally fire at `target` fires
/// together, their fire factors summed. Bakes in the die-roll modifiers the
/// engine can't derive: the Anglo-Egyptian +1 direct-fire bonus (§6.24), the
/// +1 brigade-integrity bonus when all four battalions fire (§5.54), and the
/// target hex's terrain modifier (§6.23).
/// Combat preview: while a firer is selected during a fire sub-phase, show
/// what the attack on the *hovered* hex would be -- per-firer breakdown,
/// modifier detail, CRT row, and outcome bands -- so the player can judge
/// the shot before committing. Only shown to the firing player on a legal,
/// in-LOS target.
pub fn fire_combat_preview_ui(
    mut contexts: EguiContexts,
    state: Res<PickerState>,
    game_state: Option<Res<GameStateResource>>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    hovered: Res<crate::HoveredHex>,
    peers: Peers,
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
    if !peers.may_act(firing_player) {
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

    let kind_str = match kind {
        FireKind::Direct => "Direct Fire",
        FireKind::MaximSecondFire => "Maxim 2nd Fire",
        FireKind::Howitzer => "Howitzer",
    };
    // Terrain defence modifier at target (§6.23).
    let terrain_mod = gs
        .0
        .board
        .terrain_at(target)
        .map(omdurman_rules::terrain_chart::defense_modifier)
        .unwrap_or(0);
    let net_mod = attack.net_modifier() + terrain_mod;

    // Per-firer detail: identity + fire factor.
    let firer_details: Vec<String> = attack
        .firers
        .iter()
        .filter_map(|id| gs.0.find_unit(*id))
        .map(|u| {
            let factor = u.profile.fire.map(|f| f.value()).unwrap_or(0);
            format!("{}: {}", u.profile.identity.short_label(), factor)
        })
        .collect();

    // Modifiers with rulebook sections.
    let mut mod_lines: Vec<(String, String)> = Vec::new();
    for m in &attack.modifiers {
        match m {
            FireModifier::AngloEgyptianDirectFire => {
                mod_lines.push(("A-E +1".into(), "6.24".into()));
            }
            FireModifier::BrigadeIntegrity => {
                mod_lines.push(("Brigade integrity +1".into(), "5.54".into()));
            }
            FireModifier::Terrain(n) => {
                mod_lines.push((format!("Terrain {n:+}"), "6.23".into()));
            }
            FireModifier::ZaribaThornHedge => {
                mod_lines.push(("Zariba thorn-hedge -2".into(), "9.231".into()));
            }
            FireModifier::ZaribaTrenchEntrenched => {
                mod_lines.push(("Zariba trench entrenched -4".into(), "9.232".into()));
            }
        }
    }
    if terrain_mod != 0 {
        mod_lines.push((format!("Defence {terrain_mod:+}"), "6.23".into()));
    }

    // CRT row + outcome bands.
    use omdurman_rules::combat_results_table::FireFactorRow;
    // Compute the effective range band mirroring the engine logic (§6.22, §8.1):
    // at night, cap the physical distance at the weapon's night max range;
    // within that limit the daytime range-band table applies unchanged.
    let is_night = gs.0.day_night == omdurman_types::DayNight::Night;
    let weapon = gs
        .0
        .find_unit(attack.firers[0])
        .map(|u| u.profile.weapon)
        .unwrap_or(omdurman_rules::WeaponClass::Rifles);
    let distance =
        omdurman_rules::HexDistance::new(firer_hex.distance(target) as u16);
    let effective_range = if is_night {
        let night_max = omdurman_rules::range_effects::night_max_range(
            weapon,
            attack.firing_player == Player::AngloEgyptian,
        );
        if distance.value() > night_max as u16 {
            omdurman_rules::HexDistance::new(night_max as u16 + 1) // force OutOfRange
        } else {
            distance
        }
    } else {
        distance
    };
    let band = omdurman_rules::effects::range_band_for(
        gs.0.scenario,
        attack.firing_player,
        weapon,
        effective_range,
    );
    let effective_total: u16 = attack
        .firers
        .iter()
        .filter_map(|id| gs.0.find_unit(*id))
        .filter_map(|u| u.profile.fire)
        .map(|f| band.apply(f.value()))
        .sum();
    let factor_row = FireFactorRow::from_total(effective_total);
    let row_label = format!("{:?}", factor_row);
    let bands = crate::combat_predict::outcome_bands(factor_row, net_mod);

    let Ok(ctx) = contexts.ctx_mut() else { return };
    bevy_egui::egui::Area::new(bevy_egui::egui::Id::new("fire_preview"))
        .anchor(
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

                    // Header: kind + target.
                    ui.colored_label(
                        bevy_egui::egui::Color32::from_rgb(235, 200, 170),
                        format!(
                            "{kind_str} at ({},{})",
                            target.q, target.r,
                        ),
                    );

                    // Range, band, and night info (§6.22, §8.1).
                    let hex_dist = firer_hex.distance(target);
                    let band_label = match band {
                        omdurman_rules::RangeBand::Tripled => "Tripled",
                        omdurman_rules::RangeBand::Doubled => "Doubled",
                        omdurman_rules::RangeBand::Normal => "Normal",
                        omdurman_rules::RangeBand::Halved => "Halved",
                        omdurman_rules::RangeBand::OutOfRange => "Out of range",
                    };
                    ui.label(
                        bevy_egui::egui::RichText::new(format!(
                            "Range: {hex_dist} hex{pl}  ({band_label} band)",
                            pl = if hex_dist == 1 { "" } else { "es" },
                        ))
                        .color(bevy_egui::egui::Color32::from_rgb(190, 185, 160))
                        .size(12.0),
                    );
                    if is_night {
                        ui.label(
                            bevy_egui::egui::RichText::new("Night fire \u{2014} ranges halved (\u{00a7}8.1)")
                                .color(bevy_egui::egui::Color32::from_rgb(140, 160, 210))
                                .size(11.0),
                        );
                    }
                    // FoK: both sides use the (shorter) Dervish Range Effects
                    // Table (§9.343) -- flag it so the British player knows why
                    // their bands differ from the Campaign game.
                    if gs.0.scenario == omdurman_types::Scenario::FallOfKhartoum {
                        ui.label(
                            bevy_egui::egui::RichText::new(
                                "FoK: Dervish Range Effects Table applies to both sides (\u{00a7}9.343)",
                            )
                            .color(bevy_egui::egui::Color32::from_rgb(180, 160, 120))
                            .size(11.0),
                        );
                    }

                    // LOS status (§6.3).
                    let firer_unit = gs.0.find_unit(firer);
                    let target_unit = gs.0.units.iter().find(|u| u.position == target).copied();
                    let firer_level = firer_unit
                        .map(|u| omdurman_rules::los_table::los_level_for_unit(u.profile.kind, firer_hex, &gs.0.board))
                        .unwrap_or(omdurman_rules::los_table::LosLevel::Ground);
                    let target_level = target_unit
                        .map(|u| omdurman_rules::los_table::los_level_for_unit(u.profile.kind, target, &gs.0.board))
                        .unwrap_or(omdurman_rules::los_table::LosLevel::Ground);
                    let unit_level_at = |h: omdurman_types::HexCoord| -> Option<omdurman_rules::los_table::LosLevel> {
                        gs.0.units.iter().find(|u| u.position == h).map(|u| {
                            omdurman_rules::los_table::los_level_for_unit(u.profile.kind, h, &gs.0.board)
                        })
                    };
                    if kind != FireKind::Howitzer {
                        let analysis = omdurman_rules::los_table::los_path_analysis(
                            &gs.0.board,
                            firer_hex,
                            target,
                            kind,
                            firer_level,
                            target_level,
                            unit_level_at,
                        );
                        let blocked = analysis.iter().find(|(_, r)| matches!(r, omdurman_rules::los_table::LosStepResult::Blocked { .. } | omdurman_rules::los_table::LosStepResult::BlockedHexside { .. }));
                        let los_text = match blocked {
                            Some((_, omdurman_rules::los_table::LosStepResult::Blocked { feature, hex })) => {
                                format!("LOS: Blocked by {feature:?} at ({}, {})", hex.q, hex.r)
                            }
                            Some((_, omdurman_rules::los_table::LosStepResult::BlockedHexside { a, b, feature })) => {
                                format!("LOS: Blocked by {feature:?} hexside ({},{})-({},{})", a.q, a.r, b.q, b.r)
                            }
                            _ => "LOS: Clear".to_string(),
                        };
                        let los_color = if blocked.is_some() {
                            bevy_egui::egui::Color32::from_rgb(200, 130, 100)
                        } else {
                            bevy_egui::egui::Color32::from_rgb(140, 190, 140)
                        };
                        ui.label(
                            bevy_egui::egui::RichText::new(format!("{los_text} (\u{00a7}6.3)"))
                                .color(los_color)
                                .size(11.0),
                        );
                    } else {
                        ui.label(
                            bevy_egui::egui::RichText::new("LOS: bypassed (howitzer, \u{00a7}6.64)")
                                .color(bevy_egui::egui::Color32::from_rgb(170, 170, 170))
                                .size(11.0),
                        );
                    }

                    // Firers column.
                    ui.colored_label(
                        bevy_egui::egui::Color32::from_rgb(200, 200, 200),
                        format!("Firers: {}  (factor {})", firer_details.len(), effective_total),
                    );
                    for detail in &firer_details {
                        ui.label(
                            bevy_egui::egui::RichText::new(format!("  {detail}"))
                                .color(bevy_egui::egui::Color32::from_rgb(180, 180, 180))
                                .size(12.0),
                        );
                    }

                    // Modifier breakdown.
                    if !mod_lines.is_empty() {
                        ui.add_space(2.0);
                        for (label, para) in &mod_lines {
                            ui.label(
                                bevy_egui::egui::RichText::new(format!(
                                    "  {label}  ({para})"
                                ))
                                .color(bevy_egui::egui::Color32::from_rgb(180, 160, 140))
                                .size(12.0),
                            );
                        }
                    }
                    ui.label(
                        bevy_egui::egui::RichText::new(format!(
                            "Net modifier: {net_mod:+}  |  CRT row: {row_label}"
                        ))
                        .color(bevy_egui::egui::Color32::from_rgb(235, 200, 170))
                        .size(12.0),
                    );

                    // Outcome bands.
                    ui.add_space(2.0);
                    let bands_str = bands
                        .iter()
                        .map(|b| b.label())
                        .collect::<Vec<_>>()
                        .join("  ·  ");
                    ui.colored_label(
                        bevy_egui::egui::Color32::from_rgb(200, 200, 200),
                        bevy_egui::egui::RichText::new(bands_str)
                            .size(12.0)
                            .monospace(),
                    );
                });
        });
}

pub(crate) fn build_fire_attack(
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

    // §6.24/§5.54/§9.231/§9.232: the engine derives the mandatory modifier
    // set (and rejects any other list), so build the attack with the engine's
    // own helper -- single source of truth with resolution. The terrain
    // defence modifier (§6.23) is likewise computed engine-side in
    // `resolve_fire_attack` from `state.board`.
    let mut attack = FireAttack {
        firing_player: owner,
        phase: gs.phase,
        kind,
        firers: firers.iter().map(|u| u.id).collect(),
        target_hex: target,
        factor_row,
        modifiers: Vec::new(),
    };
    attack.modifiers = omdurman_rules::effects::mandatory_fire_modifiers(gs, &attack);
    Some(attack)
}

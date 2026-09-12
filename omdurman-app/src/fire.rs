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
use omdurman_rules::effects::{GameState, build_fire_attack_from};
use omdurman_rules::{FireAttack, FireKind, FireModifier, Phase, UnitId};
use omdurman_types::{HexCoord, Player};

use crate::GameStateResource;
use crate::peers::Peers;
use crate::picker::{FireStackSelection, PickerState, PlacedUnit};
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

/// The firing group the current picker selection covers (§6.13/§6.15): the
/// firer hex plus the exact set of rules `UnitId`s that will fire. A single
/// unit selection contributes exactly that unit (unitary factor, §6.13); a
/// fire-phase double-click contributes every armed unit of the hex (§6.14).
pub(crate) struct FireGroupSelection {
    pub(crate) firer_hex: HexCoord,
    pub(crate) units: Vec<UnitId>,
}

/// Resolve the firing group from the picker state, or `None` when nothing is
/// selected that can fire. Disrupted and unarmed units are excluded (they
/// cannot receive fire orders). `units` is sorted by id so downstream keys
/// (the [`FireTargetCache`], allocation dedup) are order-stable.
pub(crate) fn fire_selection(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    gs: &GameState,
) -> Option<FireGroupSelection> {
    let (firer_hex, sources) = match state {
        PickerState::Selected {
            source,
            start_coord,
            ..
        } => (*start_coord, vec![*source]),
        PickerState::FireStack(FireStackSelection {
            sources,
            start_coord,
        }) => (*start_coord, sources.clone()),
        _ => return None,
    };
    let mut units: Vec<UnitId> = Vec::new();
    for &source in &sources {
        let (_, placed) = placed_units.get(source).ok()?;
        let Some(uid) = placed.unit_id else {
            continue;
        };
        let Some(unit) = gs.find_unit(uid) else {
            continue;
        };
        if unit.profile.fire.is_some() && !unit.state.disrupted {
            units.push(uid);
        }
    }
    if units.is_empty() {
        return None;
    }
    units.sort_unstable();
    units.dedup();
    Some(FireGroupSelection { firer_hex, units })
}

/// The `(firer, kind)` pairs the group can act with this sub-phase (§6.42). A
/// unit whose weapon has no role in the current sub-phase (e.g. rifles during
/// Maxim/Howitzer) is simply absent.
pub(crate) fn fire_group_kinds(
    gs: &GameState,
    group: &FireGroupSelection,
) -> Vec<(UnitId, FireKind)> {
    group
        .units
        .iter()
        .filter_map(|&uid| fire_kind_for(gs, uid).map(|kind| (uid, kind)))
        .collect()
}

/// Whether any member of the firing group may legally fire at `target` right
/// now (the same `can_fire_at` predicate the engine applies on echo).
fn group_can_fire_at(gs: &GameState, kinds: &[(UnitId, FireKind)], target: HexCoord) -> bool {
    kinds
        .iter()
        .any(|&(firer, kind)| gs.can_fire_at(firer, target, kind).is_ok())
}

/// The attacks a click on `target` would create from this firing group: one
/// `FireAttack` per weapon *kind*, combining exactly the group's units that
/// can fire at `target` with that kind (§6.14). A single-unit selection
/// therefore yields at most one attack carrying exactly that unit (§6.13); a
/// whole-tile selection with mixed weapons splits into one attack per kind
/// (Maxim + howitzer, §6.42). Empty when nothing can see the target.
pub(crate) fn group_attacks_for(
    gs: &GameState,
    group: &FireGroupSelection,
    kinds: &[(UnitId, FireKind)],
    target: HexCoord,
) -> Vec<FireAttack> {
    let mut attacks: Vec<FireAttack> = Vec::new();
    let mut seen_kinds: Vec<FireKind> = Vec::new();
    for &(_, kind) in kinds {
        if seen_kinds.contains(&kind) {
            continue;
        }
        seen_kinds.push(kind);
        let firers: Vec<UnitId> = kinds
            .iter()
            .filter(|(_, k)| *k == kind)
            .filter(|(id, k)| gs.can_fire_at(*id, target, *k).is_ok())
            .map(|(id, _)| *id)
            .collect();
        if let Some(attack) = build_fire_attack_from(gs, group.firer_hex, &firers, target, kind) {
            attacks.push(attack);
        }
    }
    attacks
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

/// Every wall hexside a battery may currently fire at, nearest first
/// (§6.63). Out-of-range walls are kept (range `u16::MAX`) so the panel can
/// surface them as disabled when nothing is in range.
fn wall_targets_for(gs: &GameState, uid: UnitId) -> Vec<WallTarget> {
    use omdurman_rules::effects::RuleError;
    let mut targets: Vec<(omdurman_types::HexsideRef, u16)> = gs
        .board
        .hexsides
        .iter()
        .filter(|(_, kind)| **kind == omdurman_types::HexsideKind::Wall)
        .filter_map(|(edge, _)| {
            match gs.can_fire_at_wall(uid, *edge) {
                Ok((_, range, _)) => Some((*edge, range.value())),
                // Out-of-range walls are the common case; surface them as
                // disabled buttons only when nothing is in range.
                Err(RuleError::OutOfRange { .. } | RuleError::OutOfRangeAtNight { .. }) => {
                    Some((*edge, u16::MAX))
                }
                Err(_) => None,
            }
        })
        .collect();
    targets.sort_by_key(|(edge, range)| (*range, edge.a.q, edge.a.r, edge.b.q, edge.b.r));
    targets
}

/// Cheap fingerprint of everything a legal-fire-target enumeration depends
/// on: the phase, every unit's identity/position/condition, the fired
/// trackers, and the wall/breach counts (a §6.63 breach changes LOS). Any
/// effect that could change legal targets changes at least one input, so a
/// matching stamp means the cached list is still exact.
fn fire_target_stamp(gs: &GameState) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::hash::DefaultHasher::new();
    gs.phase.hash(&mut h);
    gs.active_player.hash(&mut h);
    for u in &gs.units {
        u.id.hash(&mut h);
        u.position.hash(&mut h);
        u.state.hash(&mut h);
    }
    gs.units_fired_this_phase.hash(&mut h);
    gs.units_fired_at_this_phase.hash(&mut h);
    // Wall breaches (§6.63) are game state and change LOS (§6.3 Wall row).
    gs.breaches.hash(&mut h);
    h.finish()
}

/// One entry of the §6.63 artillery panel: a wall hexside and its range
/// (`u16::MAX` when out of range).
pub(crate) type WallTarget = (omdurman_types::HexsideRef, u16);

/// Cached legal-fire-target enumerations for the selected firing group.
///
/// Every `can_fire_at` (and `can_fire_at_wall`) call runs a full line-of-sight
/// path analysis, and the target overlay, the actions-panel count, the hover
/// preview, and the §6.63 artillery panel all need the *same* enumeration
/// every frame. The cache keys on (firer/kind pairs, state stamp), so the
/// sweep runs at most once per state change instead of once per consumer per
/// frame. The pair list is order-stable (units sorted by id), so a plain
/// `Vec` key compares reliably.
/// Cache key for the fire enumeration: the (unit, kind) pairs that can act,
/// plus the state stamp they were computed against.
type FireCacheKey = (Vec<(UnitId, FireKind)>, u64);

#[derive(Resource, Default)]
pub(crate) struct FireTargetCache {
    fire: Option<(FireCacheKey, Vec<HexCoord>)>,
    walls: Option<((UnitId, u64), Vec<WallTarget>)>,
}

impl FireTargetCache {
    /// Union of the enemy-occupied hexes any member of the firing group may
    /// legally fire at (§6.14/§6.21), recomputed only when the (firer, kind)
    /// set and the state stamp are unchanged.
    pub(crate) fn valid_targets(
        &mut self,
        gs: &GameState,
        kinds: &[(UnitId, FireKind)],
    ) -> &[HexCoord] {
        let key = (kinds.to_vec(), fire_target_stamp(gs));
        if !matches!(&self.fire, Some((k, _)) if *k == key) {
            let mut targets: Vec<HexCoord> = kinds
                .iter()
                .flat_map(|&(firer, kind)| valid_target_hexes(firer, kind, gs))
                .collect();
            targets.sort_by_key(|h| (h.q, h.r));
            targets.dedup();
            self.fire = Some((key, targets));
        }
        &self.fire.as_ref().expect("just cached").1
    }

    /// Wall hexsides battery `uid` may fire at (§6.63), nearest first,
    /// recomputed only when (firer, state) changed.
    pub(crate) fn wall_targets(&mut self, gs: &GameState, uid: UnitId) -> &[WallTarget] {
        let key = (uid, fire_target_stamp(gs));
        if !matches!(&self.walls, Some((k, _)) if *k == key) {
            self.walls = Some((key, wall_targets_for(gs, uid)));
        }
        &self.walls.as_ref().expect("just cached").1
    }
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
    mut cache: ResMut<FireTargetCache>,
) {
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = game_state else { return };
    if !matches!(
        gs.0.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    ) {
        return;
    }
    let Some(group) = fire_selection(&state, &placed_units, &gs.0) else {
        return;
    };
    let kinds = fire_group_kinds(&gs.0, &group);
    if kinds.is_empty() {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for &hex in cache.valid_targets(&gs.0, &kinds) {
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
        hex:
            crate::HexRender {
                assets: hex_assets,
                layout,
                overlay,
            },
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
    let firing_player = gs.0.phase_player();
    if !peers.may_act(firing_player) {
        return;
    }
    let Some(group) = fire_selection(&state, &placed_units, &gs.0) else {
        return;
    };
    let kinds = fire_group_kinds(&gs.0, &group);
    if kinds.is_empty() {
        return;
    }
    let Some(target) = hovered.0 else {
        return;
    };
    if !group_can_fire_at(&gs.0, &kinds, target) {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    let from = hex_world_pos(group.firer_hex, origin, &overlay.params);
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

/// Combat preview: while a firer is selected during a fire sub-phase, show
/// what the attack on the *hovered* hex would be -- per-firer breakdown,
/// modifier detail, CRT row, and outcome bands -- so the player can judge
/// the shot before committing. Only shown to the firing player on a legal,
/// in-LOS target. (Phase gate: the mirrored machine's fire-phase run
/// condition on registration; see `ui_phase_state`.)
#[allow(clippy::too_many_arguments)]
pub fn fire_combat_preview_ui(
    mut contexts: EguiContexts,
    state: Res<PickerState>,
    game_state: Option<Res<GameStateResource>>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    hovered: Res<crate::HoveredHex>,
    peers: Peers,
    mut layout: ResMut<crate::ScreenLayout>,
    mut cache: ResMut<FireTargetCache>,
) {
    let Some(gs) = game_state else { return };
    let Some(target) = hovered.0 else { return };
    let firing_player = gs.0.phase_player();
    if !peers.may_act(firing_player) {
        return;
    }
    let Some(group) = fire_selection(&state, &placed_units, &gs.0) else {
        return;
    };
    let kinds = fire_group_kinds(&gs.0, &group);
    if kinds.is_empty() {
        return;
    }
    // Only preview a shot the player could actually take. Membership in the
    // cached enumeration is exactly `can_fire_at(..).is_ok()` (same predicate,
    // computed once per state change instead of per frame — §6.21/§6.3).
    if !cache.valid_targets(&gs.0, &kinds).contains(&target) {
        return;
    }
    let attacks = group_attacks_for(&gs.0, &group, &kinds, target);
    let Some(attack) = attacks.first() else {
        return;
    };
    // The group's firer hex; the representative weapon comes from the first
    // attack's own firers below (kind may differ per attack, §6.42).
    let firer_hex = group.firer_hex;
    let kind = attack.kind;
    let firer = attack.firers[0];

    let kind_str = match kind {
        FireKind::Direct => "Direct Fire",
        FireKind::MaximSecondFire => "Maxim 2nd Fire",
        FireKind::Howitzer => "Howitzer",
    };
    // Terrain defence modifier at target (§6.23).
    let terrain_mod =
        gs.0.board
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
    let weapon =
        gs.0.find_unit(attack.firers[0])
            .map(|u| u.profile.weapon)
            .unwrap_or(omdurman_rules::WeaponClass::Rifles);
    let distance = omdurman_rules::HexDistance::new(firer_hex.distance(target) as u16);
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
    use bevy_egui::egui;
    crate::ui::stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("fire_preview"),
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(40, 20, 20, 220))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(10, 6)),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));

            // Header: kind + target.
            ui.colored_label(
                bevy_egui::egui::Color32::from_rgb(235, 200, 170),
                format!("{kind_str} at ({},{})", target.q, target.r,),
            );
            // A whole-tile selection with mixed weapons splits into several
            // attacks (§6.42) -- note the ones the preview isn't detailing.
            if attacks.len() > 1 {
                ui.label(
                    bevy_egui::egui::RichText::new(format!(
                        "...plus {} more attack{} with a different weapon in this sub-phase (\u{00a7}6.42)",
                        attacks.len() - 1,
                        if attacks.len() == 2 { "" } else { "s" },
                    ))
                    .color(bevy_egui::egui::Color32::from_rgb(180, 160, 140))
                    .size(11.0),
                );
            }

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
                    bevy_egui::egui::RichText::new(
                        "Night fire \u{2014} ranges halved (\u{00a7}8.1)",
                    )
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
                .map(|u| {
                    omdurman_rules::los_table::los_level_for_unit(
                        u.profile.kind,
                        firer_hex,
                        &gs.0.board,
                    )
                })
                .unwrap_or(omdurman_rules::los_table::LosLevel::Ground);
            let target_level = target_unit
                .map(|u| {
                    omdurman_rules::los_table::los_level_for_unit(
                        u.profile.kind,
                        target,
                        &gs.0.board,
                    )
                })
                .unwrap_or(omdurman_rules::los_table::LosLevel::Ground);
            let unit_level_at =
                |h: omdurman_types::HexCoord| -> Option<omdurman_rules::los_table::LosLevel> {
                    gs.0.units.iter().find(|u| u.position == h).map(|u| {
                        omdurman_rules::los_table::los_level_for_unit(
                            u.profile.kind,
                            h,
                            &gs.0.board,
                        )
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
                    |a, b| gs.0.wall_is_breached(a, b),
                );
                let blocked = analysis.iter().find(|(_, r)| {
                    matches!(
                        r,
                        omdurman_rules::los_table::LosStepResult::Blocked { .. }
                            | omdurman_rules::los_table::LosStepResult::BlockedHexside { .. }
                    )
                });
                let los_text = match blocked {
                    Some((
                        _,
                        omdurman_rules::los_table::LosStepResult::Blocked { feature, hex },
                    )) => {
                        format!("LOS: Blocked by {feature:?} at ({}, {})", hex.q, hex.r)
                    }
                    Some((
                        _,
                        omdurman_rules::los_table::LosStepResult::BlockedHexside { a, b, feature },
                    )) => {
                        format!(
                            "LOS: Blocked by {feature:?} hexside ({},{})-({},{})",
                            a.q, a.r, b.q, b.r
                        )
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
                format!(
                    "Firers: {}  (factor {})",
                    firer_details.len(),
                    effective_total
                ),
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
                        bevy_egui::egui::RichText::new(format!("  {label}  ({para})"))
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
        },
    );
}

/// Shell-burst marker for howitzer impacts (§6.64): an orange ring on every
/// hex where a shell landed this player-turn, including scatters — the aimed
/// hex is not enough, since the CRT applies at the *impact* hex. The engine
/// records `TurnEventRecord::HowitzerImpact` per shot and drains
/// `turn_events` at the end of the player turn, so the markers clear with
/// the phase.
pub fn howitzer_impact_markers(
    mut commands: Commands,
    hex: crate::HexRender,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<HowitzerImpactMarker>>,
) {
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = game_state else { return };

    let impacts: Vec<omdurman_types::HexCoord> = gs
        .0
        .turn_events
        .iter()
        .filter_map(|e| match e {
            omdurman_rules::turn_summary::TurnEventRecord::HowitzerImpact { at, .. } => Some(*at),
            _ => None,
        })
        .collect();
    if impacts.is_empty() {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for hex in impacts {
        let pos = hex_world_pos(hex, origin, &overlay.params);
        commands.spawn((
            HowitzerImpactMarker,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.fire_arrow.clone()),
            Transform::from_xyz(pos.x, 1.3, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
}

/// Marker component for a howitzer shell-burst ring.
#[derive(Component)]
pub struct HowitzerImpactMarker;

//! Fire combat -- target overlay, direction arrow, and combat preview.
//!
//! When a hex is selected during a fire sub-phase — a double-click anywhere
//! on it selects the whole tile as the firing group
//! ([`PickerState::SelectedTile`], the unified combat selection; a
//! single-clicked counter fires alone, §6.13/§6.15) — and the rules engine
//! says its units may fire, enemy-occupied hexes in range are highlighted.
//! The hover preview shows the would-be attack breakdown. Actual resolution
//! happens through the allocation system ([`crate::fire_allocation`]) -- the
//! player builds a battle plan and triggers batch execution with "Fire".
//!
//! The rules engine owns range/Combat Results Table resolution; the app supplies the terrain
//! modifier (the engine holds no map) and gates on [`GameState::can_fire_at`].

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_rules::effects::{GameState, build_fire_attack_from, build_gunboat_maxim_attack};
use omdurman_rules::{FireAttack, FireKind, FireModifier, Phase, UnitId};
use omdurman_types::HexCoord;

use crate::GameStateResource;
use crate::peers::Peers;
use crate::picker::{PickerState, PlacedUnit, TileSelection};

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
/// firer hex plus the exact set of rules `UnitId`s that will fire. A combat
/// tile selection (double-click on the hex — the unified selection model)
/// contributes every armed unit of the hex (§6.14); a single-clicked counter
/// contributes exactly that unit (unitary factor, §6.13).
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
        PickerState::SelectedTile(TileSelection {
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

/// The fire kind a named gunboat's Maxims (§2.32) use in the current fire
/// sub-phase: direct fire, then Maxim second fire (§6.42).
pub(crate) fn gunboat_maxim_kind(gs: &GameState) -> Option<FireKind> {
    match gs.phase {
        Phase::OffensiveFire(sub) | Phase::DefensiveFire(sub) => Some(match sub {
            omdurman_rules::FireSubPhase::DirectFire => FireKind::Direct,
            omdurman_rules::FireSubPhase::MaximSecondAndHowitzer => FireKind::MaximSecondFire,
        }),
        _ => None,
    }
}

/// The Maxims-only attack a named gunboat of the group could fire at
/// `target` now (§2.32), one per gunboat whose Maxims may.
pub(crate) fn gunboat_maxim_attacks_for(
    gs: &GameState,
    group: &FireGroupSelection,
    target: HexCoord,
) -> Vec<FireAttack> {
    let Some(kind) = gunboat_maxim_kind(gs) else {
        return Vec::new();
    };
    group
        .units
        .iter()
        .filter(|&&id| gs.can_fire_gunboat_maxims_at(id, target, kind).is_ok())
        .filter_map(|&id| build_gunboat_maxim_attack(gs, id, target, kind))
        .collect()
}

/// The attacks a click on `target` would allocate from this firing group,
/// given the attacks already `allocated` this sub-phase: the main weapons'
/// attacks ([`group_attacks_for`]), with a named gunboat's Maxims (§2.32)
/// joining its artillery when both reach the hex (§6.14: one gunboat's two
/// weapons combine at one hex) -- one click fires the whole gunboat; a click
/// that allocated only the artillery left the Maxims silently unfired. The
/// Maxims stand in alone once the artillery is allocated or cannot fire
/// here, so a player who detached them in the tray (to fire them at another
/// hex) gets them with the next click. Shared by the click handler and the
/// hover preview, so the preview always shows the shot the click would take.
pub(crate) fn click_attacks(
    gs: &GameState,
    group: &FireGroupSelection,
    kinds: &[(UnitId, FireKind)],
    target: HexCoord,
    allocated: &[FireAttack],
) -> Vec<FireAttack> {
    let mut attacks = group_attacks_for(gs, group, kinds, target);
    let primary_allocated = |id: UnitId| allocated.iter().any(|a| a.firers.contains(&id));
    for maxims in gunboat_maxim_attacks_for(gs, group, target) {
        let gunboat = maxims.gunboat_maxims[0];
        let maxims_allocated = allocated
            .iter()
            .any(|a| a.gunboat_maxims.contains(&gunboat));
        if maxims_allocated {
            continue;
        }
        let with_primary = attacks
            .iter()
            .position(|a| a.firers.contains(&gunboat))
            .filter(|_| !primary_allocated(gunboat));
        match with_primary.and_then(|i| {
            omdurman_rules::effects::combine_fire_attacks(gs, &attacks[i], &maxims)
                .map(|combined| (i, combined))
        }) {
            Some((i, combined)) => attacks[i] = combined,
            None if with_primary.is_none() => {
                attacks.retain(|a| !a.firers.contains(&gunboat));
                attacks.push(maxims);
            }
            None => {}
        }
    }
    attacks
}

/// The attack the hover preview details for a click's `planned` attack:
/// folded into the attack already staged at the same hex, as the click
/// handler folds it (§6.14: a hex is fired at once a phase, so later fire
/// joins the staged attack) -- the combined factors, row and odds, not the
/// newcomer's alone (which read "cannot succeed" against a gunboat the
/// whole attack could sink).
pub(crate) fn preview_attack(
    gs: &GameState,
    planned: &FireAttack,
    allocated: &[FireAttack],
) -> FireAttack {
    allocated
        .iter()
        .filter(|staged| staged.target_hex == planned.target_hex)
        .find_map(|staged| align_and_combine(gs, staged, planned).ok())
        .unwrap_or_else(|| planned.clone())
}

/// Join `joining` to the `staged` attack on the same hex (§6.14), first
/// taking the staged attack's aim at a garrisoned fort (§6.54/§6.62): a
/// staged shot aimed at the fort itself takes only artillery, which then
/// aims at the fort too. `Err` says why the two cannot be one attack.
pub(crate) fn align_and_combine(
    gs: &GameState,
    staged: &FireAttack,
    joining: &FireAttack,
) -> Result<FireAttack, String> {
    let aligned = if staged.at_fort && !joining.at_fort {
        omdurman_rules::effects::aim_at_fort(gs, joining).ok_or_else(|| {
            format!(
                "the attack staged on {} is aimed at the fort itself, which only artillery \
                 may fire at (§6.62) -- aim it at the garrison in the tray, or fire these \
                 units elsewhere",
                staged.target_hex
            )
        })?
    } else if !staged.at_fort && joining.at_fort {
        FireAttack {
            at_fort: false,
            ..joining.clone()
        }
    } else {
        joining.clone()
    };
    omdurman_rules::effects::combine_fire_attacks(gs, staged, &aligned).ok_or_else(|| {
        format!(
            "these units cannot join the attack staged on {} (§6.14: a hex is fired at once \
             a sub-phase)",
            staged.target_hex
        )
    })
}

/// The range a firer's weapon reaches by day, in the player's words: "rifles
/// fire at up to 4 hexes (§6.22)", "howitzers fire at 4 to 10 hexes
/// (§6.64)". `None` for a weapon with no range at all.
pub(crate) fn weapon_reach(
    gs: &GameState,
    firer: UnitId,
    mount: omdurman_rules::FireMount,
    kind: FireKind,
) -> Option<String> {
    use omdurman_rules::WeaponClass;
    let unit = gs.find_unit(firer)?;
    let weapon = unit.weapon_line(mount, kind);
    // §6.52/§9.343: the table the unit fires on.
    let table = if gs.scenario == omdurman_types::Scenario::FallOfKhartoum
        || unit.profile.identity.is_friendlies()
    {
        omdurman_types::Player::Dervish
    } else {
        unit.profile.identity.owner()
    };
    let reach: Vec<u16> = (1..=20)
        .filter(|&d| {
            omdurman_rules::effects::range_band_for(
                gs.scenario,
                table,
                weapon,
                omdurman_rules::HexDistance::new(d),
            )
            .in_range()
        })
        .collect();
    let (&lo, &hi) = (reach.first()?, reach.last()?);
    let (name, section) = match weapon {
        WeaponClass::Melee => ("spears", "§6.22"),
        WeaponClass::Rifles => ("rifles", "§6.22"),
        WeaponClass::Maxims => ("Maxims", "§6.22"),
        WeaponClass::Artillery => ("artillery", "§6.22"),
        WeaponClass::Howitzer => ("howitzers", "§6.64"),
    };
    let span = match (lo, hi) {
        (1, 1) => "1 hex".to_string(),
        (1, hi) => format!("up to {hi} hexes"),
        (lo, hi) if lo == hi => format!("{lo} hexes"),
        (lo, hi) => format!("{lo} to {hi} hexes"),
    };
    let night = if gs.day_night == omdurman_types::DayNight::Night {
        ", halved at night (§8.1)"
    } else {
        ""
    };
    Some(format!("{name} fire at {span} ({section}){night}"))
}

/// What blocks the line of sight from `firer` to `target` (§6.3), when the
/// table can say: "huts at (12, 5)", "a wall between (21, 9) and (22, 9)".
pub(crate) fn los_blocker(
    gs: &GameState,
    firer: UnitId,
    target: HexCoord,
    kind: FireKind,
) -> Option<String> {
    use omdurman_rules::los_table::LosStepResult;
    let unit = gs.find_unit(firer)?;
    let firer_level =
        omdurman_rules::los_table::los_level_for_unit(unit.profile.kind, unit.position, gs.board);
    omdurman_rules::los_table::los_path_analysis(
        gs.board,
        unit.position,
        target,
        kind,
        firer_level,
        gs.target_los_level(target),
        gs.los_unit_blocker(),
        |a, b| gs.wall_is_breached(a, b),
    )
    .into_iter()
    .find_map(|(_, step)| match step {
        LosStepResult::Blocked { feature, hex } => Some(format!("{feature} at {hex}")),
        LosStepResult::BlockedHexside { a, b, feature } => {
            Some(format!("{feature} between {a} and {b}"))
        }
        LosStepResult::Clear => None,
    })
}

/// One weapon's refusal in the player's terms: the engine's reason, with
/// the line-of-sight blocker named and an out-of-range shot told the
/// weapon's reach.
pub(crate) fn fire_refusal_text(
    gs: &GameState,
    shot: omdurman_rules::Shot,
    target: HexCoord,
    kind: FireKind,
    error: &omdurman_rules::effects::RuleError,
) -> String {
    use omdurman_rules::effects::RuleError;
    let name = crate::combat_ui::unit_name(shot.unit, gs);
    match error {
        RuleError::LineOfSightBlocked(..) => match los_blocker(gs, shot.unit, target, kind) {
            Some(blocker) => format!("no line of sight: {blocker} block it (§6.3)"),
            None => "no line of sight (§6.3)".to_string(),
        },
        RuleError::TargetOutOfRange { .. } | RuleError::OutOfRangeAtNight { .. } => {
            let distance = gs
                .find_unit(shot.unit)
                .map(|u| u.position.distance(target))
                .unwrap_or(0);
            let base = format!("{target} is {distance} hexes away, out of range");
            let base = if matches!(error, RuleError::OutOfRangeAtNight { .. }) {
                format!("{base} at night (§8.1)")
            } else {
                base
            };
            match weapon_reach(gs, shot.unit, shot.mount, kind) {
                Some(reach) => format!("{base}: {reach}"),
                None => base,
            }
        }
        RuleError::OnlyHowitzerMayFireHowitzer(_) => format!(
            "{name} carries no howitzer: only the five named gunboats fire howitzer (§6.64)"
        ),
        RuleError::Disrupted(_) => {
            format!("{name} is disrupted and may not fire until it recovers (Combat Results: D)")
        }
        RuleError::AlreadyFired(_) => format!("{name} has already fired this sub-phase (§6.13)"),
        other => other.to_string(),
    }
}

/// Why no weapon of the selection may fire at `target` (§6.13: each unit
/// fires on its own merits), weapon by weapon -- a mixed stack once
/// reported only its first unit's reason (range) when another was blocked
/// by line of sight. Identical reasons are merged; a lone reason reads
/// plainly. `selected` is every selected unit, so one that cannot fire at
/// all in this sub-phase (disrupted, no weapon for it) is explained too.
pub(crate) fn fire_refusal_reason(gs: &GameState, selected: &[UnitId], target: HexCoord) -> String {
    use omdurman_rules::{FireMount, Shot};
    let mut reasons: Vec<(String, Vec<String>)> = Vec::new();
    let mut add = |who: String, why: String| match reasons.iter_mut().find(|(r, _)| *r == why) {
        Some((_, names)) => names.push(who),
        None => reasons.push((why, vec![who])),
    };
    for &id in selected {
        let Some(unit) = gs.find_unit(id) else {
            continue;
        };
        if unit.profile.fire.is_none() {
            continue;
        }
        let main = Shot {
            unit: id,
            mount: FireMount::Main,
        };
        let why = match fire_kind_for(gs, id) {
            Some(kind) => match gs.can_fire_at(id, target, kind) {
                Err(error) => Some(fire_refusal_text(gs, main, target, kind, &error)),
                Ok(()) => None,
            },
            // No weapon of this counter has a role in the sub-phase
            // (§6.42) -- say so plainly (an unnamed gunboat carries no
            // howitzer), unless its Maxims do.
            None if unit.state.disrupted => Some(fire_refusal_text(
                gs,
                main,
                target,
                FireKind::Direct,
                &omdurman_rules::effects::RuleError::Disrupted(id),
            )),
            None if matches!(unit.profile.kind, omdurman_types::UnitKind::Gunboat { .. })
                && !unit.carries_howitzer() =>
            {
                Some(fire_refusal_text(
                    gs,
                    main,
                    target,
                    FireKind::Howitzer,
                    &omdurman_rules::effects::RuleError::OnlyHowitzerMayFireHowitzer(id),
                ))
            }
            None => {
                Some(omdurman_rules::effects::RuleError::WrongWeaponForSubphase(id).to_string())
            }
        };
        if let Some(why) = why {
            add(crate::combat_ui::shot_name(main, gs), why);
        }
        // A named gunboat's Maxims are a weapon of their own (§2.32).
        if unit.fire_factor(FireMount::GunboatMaxims).is_some()
            && let Some(kind) = gunboat_maxim_kind(gs)
            && let Err(error) = gs.can_fire_gunboat_maxims_at(id, target, kind)
        {
            let shot = Shot {
                unit: id,
                mount: FireMount::GunboatMaxims,
            };
            add(
                crate::combat_ui::shot_name(shot, gs),
                fire_refusal_text(gs, shot, target, kind, &error),
            );
        }
    }
    match reasons.as_slice() {
        [] => "the selected units cannot fire there".to_string(),
        [(why, _)] => why.clone(),
        many => many
            .iter()
            .map(|(why, names)| {
                format!(
                    "{}: {why}",
                    crate::combat_ui::tally_names(names.clone()).join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("; "),
    }
}

/// Whether `attack` uses a weapon already allocated this sub-phase (§6.13/
/// §6.14: a unit -- a gunboat's Maxims apart from its artillery -- fires
/// once per sub-phase). Such an attack is refused on click.
pub(crate) fn uses_allocated_weapon(attack: &FireAttack, allocated: &[FireAttack]) -> bool {
    allocated.iter().any(|a| {
        a.firers.iter().any(|f| attack.firers.contains(f))
            || a.gunboat_maxims
                .iter()
                .any(|g| attack.gunboat_maxims.contains(g))
    })
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
        .filter(|hex| {
            gs.can_fire_at(firer, *hex, kind).is_ok()
                // A named gunboat's Maxims reach their own targets (§2.32).
                || gunboat_maxim_kind(gs)
                    .is_some_and(|mk| gs.can_fire_gunboat_maxims_at(firer, *hex, mk).is_ok())
        })
        .collect();
    targets.sort();
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
                Err(RuleError::TargetOutOfRange { .. } | RuleError::OutOfRangeAtNight { .. }) => {
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
    side: Option<(u64, usize)>,
    side_firers: Option<(u64, Vec<HexCoord>)>,
    side_walls: Option<(u64, bool)>,
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
            targets.sort();
            targets.dedup();
            self.fire = Some((key, targets));
        }
        &self.fire.as_ref().expect("just cached").1
    }

    /// How many enemy-occupied hexes the side firing now may fire at with
    /// any unit that has not fired yet -- zero means the phase has nothing
    /// to do. Recomputed only when the state stamp changed.
    pub(crate) fn side_target_count(&mut self, gs: &GameState) -> usize {
        let key = fire_target_stamp(gs);
        if !matches!(self.side, Some((k, _)) if k == key) {
            let firer = gs.phase_player();
            let mut targets: Vec<HexCoord> = gs
                .units
                .iter()
                .filter(|u| u.profile.identity.owner() == firer)
                .filter_map(|u| fire_kind_for(gs, u.id).map(|kind| (u.id, kind)))
                .flat_map(|(id, kind)| valid_target_hexes(id, kind, gs))
                .collect();
            targets.sort();
            targets.dedup();
            self.side = Some((key, targets.len()));
        }
        self.side.expect("just cached").1
    }

    /// The hexes holding a unit of the side firing now that may still fire
    /// at some enemy hex -- what to pick when nothing is selected. Recomputed
    /// only when the state stamp changed.
    pub(crate) fn side_firer_hexes(&mut self, gs: &GameState) -> &[HexCoord] {
        let key = fire_target_stamp(gs);
        if !matches!(&self.side_firers, Some((k, _)) if *k == key) {
            let firer = gs.phase_player();
            let mut hexes: Vec<HexCoord> = gs
                .units
                .iter()
                .filter(|u| u.profile.identity.owner() == firer)
                .filter(|u| {
                    fire_kind_for(gs, u.id)
                        .is_some_and(|kind| !valid_target_hexes(u.id, kind, gs).is_empty())
                })
                .map(|u| u.position)
                .collect();
            hexes.sort();
            hexes.dedup();
            self.side_firers = Some((key, hexes));
        }
        &self.side_firers.as_ref().expect("just cached").1
    }

    /// Whether a battery of the side firing now that has not fired may fire
    /// at a standing wall this sub-phase (§6.63) -- so a phase with no enemy
    /// in range still has work for it. Recomputed only when the state stamp
    /// changed.
    pub(crate) fn side_can_breach(&mut self, gs: &GameState) -> bool {
        let key = fire_target_stamp(gs);
        if !matches!(self.side_walls, Some((k, _)) if k == key) {
            let firer = gs.phase_player();
            let any = gs
                .units
                .iter()
                .filter(|u| u.profile.identity.owner() == firer)
                .filter(|u| matches!(u.profile.weapon, omdurman_rules::WeaponClass::Artillery))
                .any(|u| {
                    gs.board.hexsides.iter().any(|(edge, kind)| {
                        *kind == omdurman_types::HexsideKind::Wall
                            && gs.can_fire_at_wall(u.id, *edge).is_ok()
                    })
                });
            self.side_walls = Some((key, any));
        }
        self.side_walls.expect("just cached").1
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
/// sub-phase -- and, with nothing selected, outline in green the side's own
/// units that still have a target, so "1 enemy hex in range" in the rail is
/// not a hunt across the board for the one counter that can shoot.
#[derive(Component)]
pub(crate) struct FireTargetRing;

/// What [`fire_target_overlay_mesh`] last drew: the (firer, kind) pairs and
/// the target hexes the current rings describe. The rings are rebuilt only
/// when either changes (or the overlays were cleared, see
/// [`crate::picker::OverlayGeneration`]) -- not every frame, matching the
/// movement/deployment overlays' rebuild-only-on-change discipline.
type FireOverlayCache = (Vec<(UnitId, FireKind)>, Vec<HexCoord>);

pub fn fire_target_overlay_mesh(
    mut draw: crate::overlay::Overlay,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    gs: Res<GameStateResource>,
    existing: Query<Entity, With<FireTargetRing>>,
    mut cache: ResMut<FireTargetCache>,
    mut last: Local<Option<FireOverlayCache>>,
    placed_changed: Query<(), Changed<PlacedUnit>>,
    peers: crate::peers::Peers,
    allocation: Res<crate::fire_allocation::FireAllocationState>,
) {
    let invalidated = draw.stale();
    if invalidated {
        // The overlays were cleared (`clear_gameplay_overlays`): forget what
        // we last drew so the "unchanged" checks below rebuild the rings
        // instead of silently leaving them despawned.
        *last = None;
    }
    // The rings depend on the engine state and the picker selection only; on
    // a frame where neither moved (nor any counter's placement data did),
    // what we drew -- or deliberately did not draw -- is still exact.
    let inputs_moved = invalidated
        || gs.is_changed()
        || state.is_changed()
        || allocation.is_changed()
        || !placed_changed.is_empty();
    if !inputs_moved {
        return;
    }
    let in_fire = gs.0.phase.is_fire();
    let drawn = if in_fire {
        fire_selection(&state, &placed_units, &gs.0)
            .map(|group| fire_group_kinds(&gs.0, &group))
            .filter(|kinds| !kinds.is_empty())
            .map(|kinds| {
                let targets = cache.valid_targets(&gs.0, &kinds).to_vec();
                (kinds, targets)
            })
    } else {
        None
    };
    // Nothing selected that has a target: the units that do (no kinds --
    // the cache entry tells the two drawings apart).
    let drawn = drawn
        .filter(|(_, targets)| !targets.is_empty())
        .or_else(|| {
            // (Once the sub-phase's fire is resolved nothing more may fire, §6.41.)
            (in_fire && !allocation.committed && peers.may_act_now(&gs.0))
                .then(|| (Vec::new(), cache.side_firer_hexes(&gs.0).to_vec()))
                .filter(|(_, hexes)| !hexes.is_empty())
        });
    let Some((kinds, targets)) = drawn else {
        // Nothing to highlight: clear any rings left over from a selection
        // that no longer fires (once, not every frame).
        if last.is_some() {
            draw.clear(existing.iter());
            *last = None;
        }
        return;
    };
    // Same selection, same targets: leave the existing rings in place
    // (despawning and respawning them would churn archetypes every frame).
    if last
        .as_ref()
        .is_some_and(|(last_kinds, last_targets)| *last_kinds == kinds && *last_targets == targets)
    {
        return;
    }
    let bands: Vec<omdurman_rules::RangeBand> = targets
        .iter()
        .map(|&target| best_band(&gs.0, &kinds, target))
        .collect();
    // §6.61/§6.62: a gunboat or a fort alone may be fired at by artillery
    // only -- the hexes this selection could reach but for that rule get a
    // small grey inner ring, so they do not read as out of range.
    let artillery_only = artillery_only_hexes(&gs.0, &kinds);
    draw.clear(existing.iter());
    for (&target, band) in targets.iter().zip(bands) {
        // Firers to pick (no kinds) are green; a target is coloured by the
        // best range band any weapon of the selection reaches it in (§6.22).
        let material = if kinds.is_empty() {
            &draw.hex.assets.green
        } else {
            match band {
                omdurman_rules::RangeBand::Tripled => &draw.hex.assets.red,
                omdurman_rules::RangeBand::Doubled => &draw.hex.assets.orange,
                omdurman_rules::RangeBand::Normal => &draw.hex.assets.yellow,
                omdurman_rules::RangeBand::Halved | omdurman_rules::RangeBand::OutOfRange => {
                    &draw.hex.assets.gray
                }
            }
        };
        draw.ring(FireTargetRing, target, 1.5, 1.0, material.clone());
    }
    for at in artillery_only {
        draw.ring(FireTargetRing, at, 1.5, 0.6, draw.hex.assets.gray.clone());
    }
    *last = Some((kinds, targets));
}

/// The enemy hexes `kinds`' weapons could fire at but for §6.61/§6.62: a
/// gunboat or a fort alone, which only artillery may engage.
fn artillery_only_hexes(gs: &GameState, kinds: &[(UnitId, FireKind)]) -> Vec<HexCoord> {
    let mut hexes: Vec<HexCoord> = gs.units.iter().map(|u| u.position).collect();
    hexes.sort();
    hexes.dedup();
    hexes.retain(|&hex| {
        kinds.iter().any(|&(id, kind)| {
            matches!(
                gs.can_fire_at(id, hex, kind),
                Err(omdurman_rules::effects::RuleError::ArtilleryOnlyVsGunboatOrFort(_))
            )
        }) && !kinds
            .iter()
            .any(|&(id, kind)| gs.can_fire_at(id, hex, kind).is_ok())
    });
    hexes
}

/// The best range band (§6.22) any of `kinds`' weapons reaches `target`
/// in: a unit's main weapon, or a named gunboat's Maxims (§2.32).
fn best_band(
    gs: &GameState,
    kinds: &[(UnitId, FireKind)],
    target: HexCoord,
) -> omdurman_rules::RangeBand {
    use omdurman_rules::RangeBand;
    let rank = |b: RangeBand| match b {
        RangeBand::Tripled => 4,
        RangeBand::Doubled => 3,
        RangeBand::Normal => 2,
        RangeBand::Halved => 1,
        RangeBand::OutOfRange => 0,
    };
    let maxims = gunboat_maxim_kind(gs);
    kinds
        .iter()
        .flat_map(|&(id, kind)| {
            let main = gs.find_unit(id).and_then(|u| {
                omdurman_rules::effects::build_fire_attack_from(gs, u.position, &[id], target, kind)
            });
            let gunboat = maxims.and_then(|mk| {
                omdurman_rules::effects::build_gunboat_maxim_attack(gs, id, target, mk)
            });
            main.into_iter().chain(gunboat)
        })
        .flat_map(|attack| omdurman_rules::effects::firer_contributions(gs, &attack))
        .map(|c| c.band)
        .max_by_key(|b| rank(*b))
        .unwrap_or(RangeBand::OutOfRange)
}

// -- Fire direction arrow: red arrow from firer to hovered target ----------

#[derive(Component)]
pub(crate) struct FireDirectionArrow;

/// Draw an arrow from the firer hex to the hovered valid target hex, giving
/// the player a visual preview of the fire direction. Same bold-red look
/// as the melee direction arrow (one visual language for combat targeting);
/// rebuilt only when the arrow's endpoints (or the overlays) change -- the
/// selection itself survives target allocation, so the arrow keeps previewing
/// further shots until the player dismisses the tile or executes the
/// allocations.
pub fn fire_direction_arrow(
    mut draw: crate::overlay::Overlay,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Res<GameStateResource>,
    target: FireArrowTarget,
    peers: Peers,
    mut last: Local<Option<Option<(HexCoord, HexCoord)>>>,
    placed_changed: Query<(), Changed<PlacedUnit>>,
) {
    let FireArrowTarget { hovered, existing } = target;
    let invalidated = draw.stale();
    if invalidated {
        // The overlays were cleared (`clear_gameplay_overlays`): forget the
        // arrow we last drew so the "unchanged" check below respawns it
        // instead of silently leaving it despawned.
        *last = None;
    }
    // The arrow's endpoints depend on the engine state, the selection and the
    // hovered hex; when none moved, the arrow we drew (or deliberately left
    // off) is still exact.
    let inputs_moved = invalidated
        || game_state.is_changed()
        || state.is_changed()
        || hovered.is_changed()
        || peers.changed()
        || !placed_changed.is_empty();
    if !inputs_moved {
        return;
    }
    let gs = &game_state.0;
    let arrow = (gs.phase.is_fire() && peers.may_act(gs.phase_player()))
        .then(|| fire_selection(&state, &placed_units, gs))
        .flatten()
        .zip(hovered.0)
        .filter(|(group, target)| {
            let kinds = fire_group_kinds(gs, group);
            !kinds.is_empty() && group_can_fire_at(gs, &kinds, *target)
        })
        .map(|(group, target)| (group.firer_hex, target));
    if *last == Some(arrow) {
        return; // unchanged: leave the drawn arrow in place
    }
    draw.clear(existing.iter());
    *last = Some(arrow);
    if let Some((from, to)) = arrow {
        draw.arrow(from, to, FireDirectionArrow);
    }
}

/// The memoized hover-preview attack plan ([`fire_combat_preview_ui`]): the
/// (hovered target, firing group, allocation count) it was derived from plus
/// the attacks it produced.
type PlannedPreviewKey = (HexCoord, Vec<UnitId>, usize);
type PlannedPreviewCache = (PlannedPreviewKey, Vec<FireAttack>);

/// The hex a hover-driven preview card describes: the hovered hex, or --
/// while the pointer has moved off the board onto one of the card's own
/// areas (`card_ids`) -- the hex it last described. Shared by the fire and
/// melee previews, whose § citations are links to follow.
pub(crate) fn sticky_preview_target(
    contexts: &mut EguiContexts,
    hovered: Option<HexCoord>,
    sticky: &mut Option<HexCoord>,
    card_ids: &[&str],
) -> Option<HexCoord> {
    if let Some(hex) = hovered {
        *sticky = Some(hex);
        return Some(hex);
    }
    let on_card = contexts.ctx_mut().ok().is_some_and(|ctx| {
        ctx.pointer_hover_pos().is_some_and(|pos| {
            card_ids.iter().any(|id| {
                ctx.memory(|m| m.area_rect(bevy_egui::egui::Id::new(*id)))
                    .is_some_and(|rect| rect.contains(pos))
            })
        })
    });
    if !on_card {
        *sticky = None;
    }
    *sticky
}

/// The engine-derived defence modifiers on a fire attack: the target hex's
/// terrain (§6.23) with a fort's -3 for the units inside it (§6.54), and any
/// Crest / City Wall hexside crossed into it (Terrain Effects Chart). Not part of `attack.modifiers` -- the engine
/// derives them at resolution -- so every display of the net die modifier
/// (preview, allocation tray) must add them, as `resolve_fire_attack` does.
pub(crate) fn target_defence_modifiers(
    gs: &omdurman_rules::effects::GameState,
    attack: &omdurman_rules::FireAttack,
) -> (i16, i16) {
    let hex = attack.target_hex;
    let targets = omdurman_rules::effects::fire_target_units(gs, attack, hex);
    let total = omdurman_rules::effects::target_defence_modifier(gs, attack, hex, &targets);
    let hexside = omdurman_rules::effects::target_hexside_fire_modifier(gs, attack, hex);
    // (terrain and the fort's -3 together, the crossed hexside)
    (total - hexside, hexside)
}

/// Combat preview: while a firer is selected during a fire sub-phase, show
/// what the attack on the *hovered* hex would be -- per-firer breakdown,
/// modifier detail, CRT row, and outcome bands -- so the player can judge
/// the shot before committing. Only shown to the firing player on a legal,
/// in-LOS target. (Phase gate: the mirrored machine's fire-phase run
/// condition on registration; see `ui_phase_state`.)
pub fn fire_combat_preview_ui(
    mut contexts: EguiContexts,
    state: Res<PickerState>,
    gs: Res<GameStateResource>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    hovered: Res<crate::HoveredHex>,
    peers: Peers,
    mut layout: ResMut<crate::ScreenLayout>,
    mut cache: ResMut<FireTargetCache>,
    allocation: Option<Res<crate::fire_allocation::FireAllocationState>>,
    mut sticky: Local<Option<HexCoord>>,
    mut planned: Local<Option<PlannedPreviewCache>>,
) {
    let gs_moved = gs.is_changed();
    // Once this sub-phase's fire is resolved, nothing more can be allocated
    // (§6.41): no shot to preview.
    let allocated: &[FireAttack] = allocation.as_deref().map_or(&[], |a| &a.attacks);
    if allocation.as_deref().is_some_and(|a| a.committed) {
        return;
    }
    // The preview follows the hovered hex, and stays up while the pointer is
    // on the card itself, so its § links can be followed.
    let Some(target) = sticky_preview_target(
        &mut contexts,
        hovered.0,
        &mut sticky,
        &["fire_preview", "fire_preview_refused"],
    ) else {
        return;
    };
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
        // An enemy hex the group cannot fire at: say why on hover (range,
        // line of sight, ...) instead of only after a refused click.
        let enemy_there =
            gs.0.units
                .iter()
                .any(|u| u.position == target && u.profile.identity.owner() != firing_player);
        if enemy_there
            && target != group.firer_hex
            && let Ok(ctx) = contexts.ctx_mut()
        {
            let reason = fire_refusal_reason(&gs.0, &group.units, target);
            crate::ui::passive_stacked_card(
                ctx,
                &mut layout,
                egui::Id::new("fire_preview_refused"),
                crate::ui::frames::card(crate::ui::palette::CARD_FIRE),
                |ui| {
                    crate::rulebook::refs_label(
                        ui,
                        &format!("Cannot fire at {target}: {reason}"),
                        crate::ui::palette::REFUSED,
                        12.0,
                    );
                },
            );
        }
        return;
    }
    // Exactly the shots a click would allocate (a named gunboat's artillery
    // first, its Maxims once that is spent, §2.32), less any weapon already
    // allocated this sub-phase -- the click would refuse those. Deriving the
    // plan runs `can_fire_at` (with its LOS sweep) per firer plus the attack
    // assembly, so it is memoized: re-derived only when the hovered target,
    // the selection, the allocation tray, or the engine state moved. (The
    // staleness check compares against the cached key in place; the key's
    // unit `Vec` is cloned only when the plan is actually re-derived.)
    let planned_stale = gs_moved
        || state.is_changed()
        || planned.as_ref().is_none_or(|(key, _)| {
            key.0 != target || key.1 != group.units || key.2 != allocated.len()
        });
    if planned_stale {
        let planned_key = (target, group.units.clone(), allocated.len());
        let planned_attacks = click_attacks(&gs.0, &group, &kinds, target, allocated);
        // Each shown as the click would stage it: joined to the attack
        // already staged on this hex (§6.14).
        let attacks: Vec<FireAttack> = planned_attacks
            .into_iter()
            .filter(|a| !uses_allocated_weapon(a, allocated))
            .map(|a| preview_attack(&gs.0, &a, allocated))
            .collect();
        *planned = Some((planned_key, attacks));
    }
    let attacks = &planned.as_ref().expect("just cached").1;
    // With the selection's fire already staged on this hex (the usual case
    // right after the click), the card details that staged attack -- the
    // whole combined shot -- as a confirmation.
    let staged_here = allocated.iter().find(|a| {
        a.target_hex == target && a.all_firing_units().iter().any(|u| group.units.contains(u))
    });
    // Whether the click joins an attack already staged here (§6.14).
    let joins_staged = !attacks.is_empty() && allocated.iter().any(|a| a.target_hex == target);
    let Some(attack) = attacks.first().or(staged_here) else {
        if let Ok(ctx) = contexts.ctx_mut() {
            crate::ui::passive_stacked_card(
                ctx,
                &mut layout,
                egui::Id::new("fire_preview_refused"),
                crate::ui::frames::card(crate::ui::palette::CARD_FIRE),
                |ui| {
                    crate::rulebook::refs_label(
                        ui,
                        "These units have already allocated their fire this sub-phase (\u{a7}6.41).",
                        crate::ui::palette::REFUSED,
                        12.0,
                    );
                },
            );
        }
        return;
    };
    let is_staged_view = attacks.is_empty();
    // The group's firer hex; the representative weapon comes from the first
    // attack's own firers below (kind may differ per attack, §6.42).
    let firer_hex = group.firer_hex;
    let kind = attack.kind;
    let Some(first_shot) = attack.shots().first().copied() else {
        return;
    };
    let firer = first_shot.unit;

    let kind_str = match kind {
        FireKind::Direct => "Direct Fire",
        FireKind::MaximSecondFire => "Maxim 2nd Fire",
        FireKind::Howitzer => "Howitzer",
    };
    let (terrain_mod, hexside_mod) = target_defence_modifiers(&gs.0, attack);
    let net_mod = attack.net_modifier() + terrain_mod + hexside_mod;

    // Per-firer detail: identity + fire factor at its range (§6.22).
    // Identical contributions counted ("4\u{d7} Mulazmin: 3").
    let contributions = omdurman_rules::effects::firer_contributions(&gs.0, attack);
    let firer_details: Vec<String> = contributions
        .iter()
        .filter_map(|c| {
            let u = gs.0.find_unit(c.unit)?;
            let printed = u.fire_factor(c.mount).map(|f| f.value()).unwrap_or(0);
            let name = crate::combat_ui::shot_name(
                omdurman_rules::Shot {
                    unit: c.unit,
                    mount: c.mount,
                },
                &gs.0,
            );
            Some(if c.factor == printed {
                format!("{name}: {}", c.factor)
            } else {
                format!(
                    "{name}: {} ({printed} at range {})",
                    c.factor,
                    c.distance.value()
                )
            })
        })
        .collect();
    let firer_details = crate::combat_ui::tally_names(firer_details);

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
    if hexside_mod != 0 {
        let side = if hexside_mod == -4 {
            "City wall"
        } else {
            "Crest"
        };
        mod_lines.push((format!("{side} {hexside_mod:+}"), "6.23".into()));
    }

    // CRT row + outcome bands: every firer banded exactly as resolution
    // bands it (§6.22 per firer, §6.52/§9.343 tables, §8.1 night cap).
    use omdurman_rules::combat_results_table::FireFactorRow;
    let effective_total = omdurman_rules::effects::total_fire_factor(&contributions);
    let factor_row = FireFactorRow::from_total(effective_total);
    let row_label = factor_row.label();
    let is_night = gs.0.day_night == omdurman_types::DayNight::Night;
    // The first firer's band, for the header line (each firer's own is in
    // the per-firer detail).
    let band = contributions
        .first()
        .map_or(omdurman_rules::RangeBand::OutOfRange, |c| c.band);
    let bands = crate::combat_predict::outcome_bands(factor_row, net_mod);
    let special = omdurman_rules::effects::special_target_threshold(&gs.0, attack);

    // §6.54/§6.62: at a garrisoned enemy fort the artillery may aim at the
    // fort itself instead of the units inside -- the card shows that choice
    // too, with its odds (destroyed on a "2" or more).
    if !attack.at_fort
        && let Some(at_fort) = omdurman_rules::effects::aim_at_fort(&gs.0, attack)
        && let Ok(ctx) = contexts.ctx_mut()
    {
        let rolls = fort_destroying_rolls(&gs.0, &at_fort);
        crate::ui::passive_stacked_card(
            ctx,
            &mut layout,
            egui::Id::new("fire_preview_fort"),
            crate::ui::frames::card(crate::ui::palette::CARD_FIRE),
            |ui| {
                crate::rulebook::refs_label(
                    ui,
                    &format!(
                        "Or aim at the fort itself (§6.62): destroyed on {rolls} in 10, and \
                         one unit inside falls with it -- switch the aim in the tray."
                    ),
                    crate::ui::palette::TEXT_MUTED,
                    12.0,
                );
            },
        );
    }

    // §6.64: "the results must take effect, even if the fire scatters into a
    // friendly-occupied hex" -- say beforehand how often a shell would.
    if kind == FireKind::Howitzer
        && let Some((rolls, hexes)) = howitzer_friendly_risk(&gs.0, target, firing_player)
        && let Ok(ctx) = contexts.ctx_mut()
    {
        let at: Vec<String> = hexes.iter().map(|h| h.to_string()).collect();
        crate::ui::passive_stacked_card(
            ctx,
            &mut layout,
            egui::Id::new("fire_preview_scatter"),
            crate::ui::frames::card(crate::ui::palette::CARD_FIRE),
            |ui| {
                crate::rulebook::refs_label(
                    ui,
                    &format!(
                        "Scatter risk: {rolls} in 10, a shell lands on your own units at {} \
                         (§6.64).",
                        at.join(", ")
                    ),
                    crate::ui::palette::CAUTION,
                    12.0,
                );
            },
        );
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };
    use bevy_egui::egui;
    crate::ui::passive_stacked_card(
        ctx,
        &mut layout,
        egui::Id::new("fire_preview"),
        crate::ui::frames::card(crate::ui::palette::CARD_FIRE),
        |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(13.0));

            // Header: kind + target.
            ui.colored_label(
                crate::ui::palette::CARD_TITLE,
                format!("{kind_str} at ({},{})", target.q, target.r,),
            );
            if is_staged_view {
                ui.label(crate::ui::text::note(
                    "Staged \u{2014} resolve it in the tray.",
                ));
            } else if joins_staged {
                crate::rulebook::refs_label(
                    ui,
                    "Joins the attack staged on this hex (\u{a7}6.14): the combined fire below.",
                    crate::ui::palette::PANEL_DIM,
                    11.0,
                );
            }
            // A whole-tile selection with mixed weapons splits into several
            // attacks (§6.42) -- note the ones the preview isn't detailing.
            if attacks.len() > 1 {
                crate::rulebook::refs_label(
                    ui,
                    &format!(
                        "...plus {} more attack{} with a different weapon in this sub-phase (\u{00a7}6.42)",
                        attacks.len() - 1,
                        if attacks.len() == 2 { "" } else { "s" },
                    ),
                    crate::ui::palette::PANEL_DIM,
                    11.0,
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
                .color(crate::ui::palette::HEADING_DIM)
                .size(12.0),
            );
            if is_night {
                crate::rulebook::refs_label(
                    ui,
                    "Night fire \u{2014} ranges halved (\u{00a7}8.1)",
                    crate::ui::palette::INFO,
                    11.0,
                );
            }
            // FoK: both sides use the (shorter) Dervish Range Effects
            // Table (§9.343) -- flag it so the British player knows why
            // their bands differ from the Campaign game.
            if gs.0.scenario == omdurman_types::Scenario::FallOfKhartoum {
                crate::rulebook::refs_label(
                    ui,
                    "FoK: Dervish Range Effects Table applies to both sides (\u{00a7}9.343)",
                    crate::ui::palette::BRASS_DIM,
                    11.0,
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
                        gs.0.board,
                    )
                })
                .unwrap_or(omdurman_rules::los_table::LosLevel::Ground);
            let target_level = target_unit
                .map(|u| {
                    omdurman_rules::los_table::los_level_for_unit(
                        u.profile.kind,
                        target,
                        gs.0.board,
                    )
                })
                .unwrap_or(omdurman_rules::los_table::LosLevel::Ground);
            // The engine's own blocker: units, not gunboats, forts or
            // entrenched units (§6.3 note a, §9.232).
            let unit_level_at = gs.0.los_unit_blocker();
            if kind != FireKind::Howitzer {
                let analysis = omdurman_rules::los_table::los_path_analysis(
                    gs.0.board,
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
                        format!("LOS: Blocked by {feature} at {hex}")
                    }
                    Some((
                        _,
                        omdurman_rules::los_table::LosStepResult::BlockedHexside { a, b, feature },
                    )) => {
                        format!(
                            "LOS: Blocked by {feature} hexside ({},{})-({},{})",
                            a.q, a.r, b.q, b.r
                        )
                    }
                    _ => "LOS: Clear".to_string(),
                };
                let los_color = if blocked.is_some() {
                    crate::ui::palette::REFUSED
                } else {
                    crate::ui::palette::CLEAR
                };
                crate::rulebook::refs_label(
                    ui,
                    &format!("{los_text} (\u{00a7}6.3)"),
                    los_color,
                    11.0,
                );
            } else {
                crate::rulebook::refs_label(
                    ui,
                    "LOS: bypassed (howitzer, \u{00a7}6.64)",
                    crate::ui::palette::TEXT_MUTED,
                    11.0,
                );
            }

            // Firers column.
            ui.colored_label(
                crate::ui::palette::TEXT,
                format!(
                    "Firers: {}  (factor {})",
                    firer_details.len(),
                    effective_total
                ),
            );
            for detail in &firer_details {
                ui.label(
                    bevy_egui::egui::RichText::new(format!("  {detail}"))
                        .color(crate::ui::palette::TEXT_SOFT)
                        .size(12.0),
                );
            }

            // Modifier breakdown.
            if !mod_lines.is_empty() {
                ui.add_space(2.0);
                for (label, para) in &mod_lines {
                    crate::rulebook::refs_label(
                        ui,
                        &format!("  {label}  (\u{00a7}{para})"),
                        crate::ui::palette::PANEL_DIM,
                        12.0,
                    );
                }
            }
            ui.label(
                bevy_egui::egui::RichText::new(format!(
                    "Net modifier: {net_mod:+}  |  CRT row: {row_label}"
                ))
                .color(crate::ui::palette::CARD_TITLE)
                .size(12.0),
            );

            // Outcome bands.
            ui.add_space(2.0);
            let bands_str = match special {
                // A gunboat or a fort: only a big enough result counts
                // (§6.61: 3+ sinks a gunboat; §6.62: 2+ destroys a fort).
                Some(needed) => {
                    let hits: Vec<String> = bands
                        .iter()
                        .filter(|b| {
                            matches!(b.result, omdurman_rules::CombatResult::Eliminate(n) if n >= needed)
                        })
                        .map(|b| {
                            if b.lo == b.hi {
                                b.lo.to_string()
                            } else {
                                format!("{}-{}", b.lo, b.hi)
                            }
                        })
                        .collect();
                    if hits.is_empty() {
                        format!("needs a result of {needed}+: cannot succeed")
                    } else {
                        format!(
                            "destroyed on {} (needs {needed}+), else a miss",
                            hits.join(", ")
                        )
                    }
                }
                None => bands
                    .iter()
                    .map(|b| b.label())
                    .collect::<Vec<_>>()
                    .join("  ·  "),
            };
            ui.colored_label(
                crate::ui::palette::TEXT,
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
    mut draw: crate::overlay::Overlay,
    game_state: Res<GameStateResource>,
    existing: Query<Entity, With<HowitzerImpactMarker>>,
    mut last: Local<Option<Vec<(HexCoord, HexCoord)>>>,
) {
    // Rebuilt only when the impacts change (the engine state moved) or the
    // overlays were cleared -- not every frame.
    if draw.stale() {
        *last = None;
    }
    if last.is_some() && !game_state.is_changed() {
        return;
    }
    let gs = game_state;
    // (aimed, landed) per shell this turn, a shell lost off the map apart.
    let impacts: Vec<(HexCoord, HexCoord)> =
        gs.0.turn_events
            .iter()
            .filter_map(|e| match e {
                omdurman_rules::turn_summary::TurnEventRecord::HowitzerImpact {
                    aimed,
                    at,
                    lost: false,
                    ..
                } => Some((*aimed, *at)),
                _ => None,
            })
            .collect();
    if last.as_ref() == Some(&impacts) {
        return;
    }
    draw.clear(existing.iter());
    for &(_, impact) in &impacts {
        draw.ring(
            HowitzerImpactMarker,
            impact,
            1.3,
            1.0,
            draw.hex.assets.fire_arrow.clone(),
        );
    }
    // §6.64: a scattered shell's way from the hex it was aimed at.
    for &(aimed, impact) in &impacts {
        if aimed != impact {
            draw.arrow(aimed, impact, HowitzerImpactMarker);
        }
    }
    *last = Some(impacts);
}

/// How many of the ten rolls destroy the fort `attack` is aimed at (§6.62:
/// a result of 2 or more), on the attack's factor row and modifiers.
pub(crate) fn fort_destroying_rolls(gs: &GameState, attack: &FireAttack) -> u8 {
    let contributions = omdurman_rules::effects::firer_contributions(gs, attack);
    let row = omdurman_rules::combat_results_table::FireFactorRow::from_total(
        omdurman_rules::effects::total_fire_factor(&contributions),
    );
    let (terrain, hexside) = target_defence_modifiers(gs, attack);
    let net = attack.net_modifier() + terrain + hexside;
    let needed = omdurman_rules::effects::special_target_threshold(gs, attack).unwrap_or(2);
    crate::combat_predict::outcome_bands(row, net)
        .iter()
        .filter(|b| matches!(b.result, omdurman_rules::CombatResult::Eliminate(n) if n >= needed))
        .map(|b| b.hi - b.lo + 1)
        .sum()
}

/// How many of the ten impact rolls put a shell aimed at `target` on a hex
/// holding `player`'s own units (§6.64 Scattergram), and those hexes;
/// `None` when no roll does.
pub(crate) fn howitzer_friendly_risk(
    gs: &GameState,
    target: HexCoord,
    player: omdurman_types::Player,
) -> Option<(usize, Vec<HexCoord>)> {
    let mut rolls = 0;
    let mut hexes: Vec<HexCoord> = Vec::new();
    for value in 1..=10u16 {
        let roll = omdurman_rules::DieRoll::try_from(value).ok()?;
        let at = omdurman_rules::howitzer_scatter::scatter_impact_hex(
            target,
            omdurman_rules::howitzer_scatter::howitzer_scatter(roll),
        );
        if gs
            .units
            .iter()
            .any(|u| u.position == at && u.profile.identity.owner() == player)
        {
            rolls += 1;
            if !hexes.contains(&at) {
                hexes.push(at);
            }
        }
    }
    (rolls > 0).then_some((rolls, hexes))
}

/// Marker component for a howitzer shell-burst ring.
#[derive(Component)]
pub struct HowitzerImpactMarker;

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_rules::{FireSubPhase, UnitPlacement, UnitState};
    use omdurman_types::{Player, Scenario};

    /// A board-less Direct Fire sub-phase with the named gunboat Abu Klea at
    /// (0,0) and a Taiasha stack three hexes off.
    fn naser_in_direct_fire() -> (GameState, FireGroupSelection, HexCoord) {
        let mut gs = GameState::new(Scenario::Campaign);
        gs.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        gs.active_player = Player::AngloEgyptian;
        let target = HexCoord::new(3, 0);
        for (id, position) in [
            (UnitId::BritishBoats_3_0, HexCoord::new(0, 0)),
            (UnitId::Taiasha_0_0, target),
        ] {
            gs.units.push(UnitPlacement {
                id,
                position,
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: UnitState::default(),
            });
        }
        let group = FireGroupSelection {
            firer_hex: HexCoord::new(0, 0),
            units: vec![UnitId::BritishBoats_3_0],
        };
        (gs, group, target)
    }

    /// One click fires the whole named gunboat -- its artillery and its
    /// Maxims at one hex (§2.32, §6.14); detached in the tray, the Maxims
    /// come with the next click on their own.
    #[test]
    fn a_click_takes_both_weapons_and_detached_maxims_come_next() {
        let (gs, group, target) = naser_in_direct_fire();
        let kinds = fire_group_kinds(&gs, &group);
        let naser = UnitId::BritishBoats_3_0;

        let first = click_attacks(&gs, &group, &kinds, target, &[]);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].firers, vec![naser]);
        assert_eq!(first[0].gunboat_maxims, vec![naser]);
        assert!(!uses_allocated_weapon(&first[0], &[]));
        // Nothing left unfired to remind the player of.
        let staged = crate::fire_allocation::FireAllocationState {
            attacks: first.clone(),
            ..Default::default()
        };
        assert!(crate::fire_allocation::unfired_gunboat_maxims(&gs, &staged).is_empty());

        // "Fire the Maxims elsewhere": the artillery stays, the Maxims are
        // free -- and the tray says so.
        let artillery = crate::fire_allocation::without_own_maxims(&gs, &first[0])
            .expect("the gunboat's own Maxims");
        assert_eq!(artillery.firers, vec![naser]);
        assert!(artillery.gunboat_maxims.is_empty());
        let staged = crate::fire_allocation::FireAllocationState {
            attacks: vec![artillery.clone()],
            ..Default::default()
        };
        assert_eq!(
            crate::fire_allocation::unfired_gunboat_maxims(&gs, &staged),
            vec![naser]
        );
        let second = click_attacks(
            &gs,
            &group,
            &kinds,
            target,
            std::slice::from_ref(&artillery),
        );
        assert_eq!(second.len(), 1);
        assert!(second[0].firers.is_empty());
        assert_eq!(second[0].gunboat_maxims, vec![naser]);

        // Everything allocated: whatever a click would offer is spent.
        let third = click_attacks(&gs, &group, &kinds, target, &first);
        assert!(third.iter().all(|a| uses_allocated_weapon(a, &first)));
    }

    /// The hover preview of a second firer shows the attack it joins
    /// (§6.14): the staged firers' factors and its own, together.
    #[test]
    fn the_preview_shows_the_staged_attack_it_joins() {
        let (mut gs, _, target) = naser_in_direct_fire();
        let sultan = UnitId::ALL
            .iter()
            .copied()
            .find(|&id| {
                id != UnitId::BritishBoats_3_0
                    && omdurman_rules::unit_profiles::profile_for_unit(id).is_some_and(|p| {
                        matches!(p.identity, omdurman_rules::UnitIdentity::AngloEgyptianGunboat(gb) if gb.has_howitzer())
                    })
            })
            .expect("a second named gunboat");
        gs.units.push(UnitPlacement {
            id: sultan,
            position: HexCoord::new(0, 1),
            profile: omdurman_rules::unit_profiles::profile_for_unit(sultan).unwrap(),
            state: UnitState::default(),
        });
        let naser = UnitId::BritishBoats_3_0;
        let staged = omdurman_rules::effects::build_fire_attack_from(
            &gs,
            HexCoord::new(0, 0),
            &[naser],
            target,
            FireKind::Direct,
        )
        .expect("Abu Klea fires");
        let joining = omdurman_rules::effects::build_fire_attack_from(
            &gs,
            HexCoord::new(0, 1),
            &[sultan],
            target,
            FireKind::Direct,
        )
        .expect("the second boat fires");
        let shown = preview_attack(&gs, &joining, std::slice::from_ref(&staged));
        let mut both = vec![naser, sultan];
        both.sort_unstable();
        assert_eq!(shown.firers, both);
        // Nothing staged there: the newcomer alone.
        assert_eq!(preview_attack(&gs, &joining, &[]), joining);
    }

    /// A refusal names what blocks and how far the weapon reaches, weapon
    /// by weapon when they differ (§6.13, §6.22, §6.3, §6.64).
    #[test]
    fn a_refusal_says_why_per_weapon() {
        let (mut gs, _, _) = naser_in_direct_fire();
        // Spears reach one hex.
        let baggara = UnitId::Baggara_0_0;
        gs.units.push(UnitPlacement {
            id: baggara,
            position: HexCoord::new(0, 3),
            profile: omdurman_rules::unit_profiles::profile_for_unit(baggara).unwrap(),
            state: UnitState::default(),
        });
        let rifles = UnitId::BritishArmy_0_0;
        gs.units.push(UnitPlacement {
            id: rifles,
            position: HexCoord::new(0, 5),
            profile: omdurman_rules::unit_profiles::profile_for_unit(rifles).unwrap(),
            state: UnitState::default(),
        });
        gs.active_player = Player::Dervish;
        let reason = fire_refusal_reason(&gs, &[baggara], HexCoord::new(0, 5));
        assert!(
            reason.contains("out of range") && reason.contains("spears fire at 1 hex"),
            "{reason}"
        );
        // Howitzers: 4 to 10 hexes.
        gs.active_player = Player::AngloEgyptian;
        gs.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
        let reach = weapon_reach(
            &gs,
            UnitId::BritishBoats_3_0,
            omdurman_rules::FireMount::Main,
            FireKind::Howitzer,
        );
        assert_eq!(
            reach.as_deref(),
            Some("howitzers fire at 4 to 10 hexes (§6.64)")
        );
        // A disrupted gunboat says so rather than nothing.
        gs.units[0].state.disrupted = true;
        let reason = fire_refusal_reason(&gs, &[UnitId::BritishBoats_3_0], HexCoord::new(3, 0));
        assert!(reason.contains("disrupted"), "{reason}");
    }
}

#[cfg(test)]
mod band_ring_tests {
    use omdurman_rules::{FireKind, Phase, RangeBand, UnitId, UnitPlacement};
    use omdurman_types::{HexCoord, Player, Scenario};

    /// A target's ring shows the best band the selection reaches it in
    /// (§6.22): an adjacent target in a better band than a distant one.
    #[test]
    fn a_closer_target_rings_in_a_better_band() {
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        gs.phase = Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire);
        gs.active_player = Player::AngloEgyptian;
        for (id, hex) in [
            (UnitId::BritishArmy_0_0, HexCoord::new(0, 0)),
            (UnitId::Baggara_0_0, HexCoord::new(1, 0)),
            (UnitId::Baggara_0_1, HexCoord::new(4, 0)),
        ] {
            gs.units.push(UnitPlacement {
                id,
                position: hex,
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: Default::default(),
            });
        }
        let kinds = [(UnitId::BritishArmy_0_0, FireKind::Direct)];
        let near = super::best_band(&gs, &kinds, HexCoord::new(1, 0));
        let far = super::best_band(&gs, &kinds, HexCoord::new(4, 0));
        assert!(
            matches!(near, RangeBand::Tripled | RangeBand::Doubled),
            "{near:?}"
        );
        assert_ne!(near, far);
        assert_eq!(
            super::best_band(&gs, &kinds, HexCoord::new(9, 0)),
            RangeBand::OutOfRange
        );
    }
}

#[cfg(test)]
mod scatter_risk_tests {
    use omdurman_rules::{UnitId, UnitPlacement};
    use omdurman_types::{HexCoord, Player, Scenario};

    /// §6.64: a shell aimed beside one's own units may land on them; the
    /// risk counts the impact rolls that would.
    #[test]
    fn the_scatter_risk_counts_rolls_onto_own_units() {
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        let target = HexCoord::new(5, 5);
        let stray = omdurman_rules::howitzer_scatter::scatter_impact_hex(
            target,
            omdurman_rules::howitzer_scatter::howitzer_scatter(
                omdurman_rules::DieRoll::try_from(1u16).unwrap(),
            ),
        );
        assert!(super::howitzer_friendly_risk(&gs, target, Player::AngloEgyptian).is_none());
        gs.units.push(UnitPlacement {
            id: UnitId::BritishArmy_0_0,
            position: stray,
            profile: omdurman_rules::unit_profiles::profile_for_unit(UnitId::BritishArmy_0_0)
                .unwrap(),
            state: Default::default(),
        });
        let (rolls, hexes) =
            super::howitzer_friendly_risk(&gs, target, Player::AngloEgyptian).expect("a risk");
        assert_eq!(rolls, 1);
        assert_eq!(hexes, vec![stray]);
        assert!(super::howitzer_friendly_risk(&gs, target, Player::Dervish).is_none());
    }
}

#[cfg(test)]
mod fort_aim_tests {
    use omdurman_rules::{FireKind, Phase, UnitId, UnitPlacement};
    use omdurman_types::{HexCoord, Player, Scenario};

    /// §6.62: batteries next to a garrisoned fort may aim at the fort; the
    /// preview's odds count the rolls of 2 or more.
    #[test]
    fn a_garrisoned_fort_offers_its_own_odds() {
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        gs.phase = Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire);
        gs.active_player = Player::AngloEgyptian;
        let fort = UnitId::ALL
            .iter()
            .copied()
            .find(|id| {
                omdurman_rules::unit_profiles::profile_for_unit(*id).is_some_and(|p| {
                    matches!(p.identity, omdurman_rules::UnitIdentity::DervishFort)
                })
            })
            .expect("a Dervish fort");
        let battery = UnitId::ALL
            .iter()
            .copied()
            .find(|id| {
                omdurman_rules::unit_profiles::profile_for_unit(*id).is_some_and(|p| {
                    matches!(
                        p.identity,
                        omdurman_rules::UnitIdentity::AngloEgyptianArtillery
                    )
                })
            })
            .expect("a battery");
        let target = HexCoord::new(1, 0);
        for (id, hex) in [
            (battery, HexCoord::new(0, 0)),
            (fort, target),
            (UnitId::Baggara_0_0, target),
        ] {
            gs.units.push(UnitPlacement {
                id,
                position: hex,
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: Default::default(),
            });
        }
        let attack = omdurman_rules::effects::build_fire_attack_from(
            &gs,
            HexCoord::new(0, 0),
            &[battery],
            target,
            FireKind::Direct,
        )
        .expect("the battery may fire");
        let at_fort = omdurman_rules::effects::aim_at_fort(&gs, &attack).expect("artillery");
        let rolls = super::fort_destroying_rolls(&gs, &at_fort);
        assert!((1..=10).contains(&rolls), "{rolls}");
    }

    /// Fire joining a shot staged at the fort itself takes the fort as its
    /// aim too (artillery), or is refused with the reason (anything else)
    /// -- staged as a second attack on the hex, the whole allocation would
    /// be refused at Resolve (§6.14, §6.62).
    #[test]
    fn fire_joining_a_fort_aimed_shot_takes_its_aim_or_says_why() {
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        gs.phase = Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire);
        gs.active_player = Player::AngloEgyptian;
        let find = |want: fn(&omdurman_rules::UnitIdentity) -> bool| -> Vec<UnitId> {
            UnitId::ALL
                .iter()
                .copied()
                .filter(|id| {
                    omdurman_rules::unit_profiles::profile_for_unit(*id)
                        .is_some_and(|p| want(&p.identity))
                })
                .collect()
        };
        let fort = find(|i| matches!(i, omdurman_rules::UnitIdentity::DervishFort))[0];
        let batteries = find(|i| matches!(i, omdurman_rules::UnitIdentity::AngloEgyptianArtillery));
        let rifles = UnitId::BritishArmy_0_0;
        let target = HexCoord::new(1, 0);
        for (id, hex) in [
            (batteries[0], HexCoord::new(0, 0)),
            (batteries[1], HexCoord::new(0, 1)),
            (rifles, HexCoord::new(2, 0)),
            (fort, target),
            (UnitId::Baggara_0_0, target),
        ] {
            gs.units.push(UnitPlacement {
                id,
                position: hex,
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: Default::default(),
            });
        }
        let shot = |firer: UnitId, from: HexCoord| {
            omdurman_rules::effects::build_fire_attack_from(
                &gs,
                from,
                &[firer],
                target,
                FireKind::Direct,
            )
            .expect("may fire")
        };
        let staged =
            omdurman_rules::effects::aim_at_fort(&gs, &shot(batteries[0], HexCoord::new(0, 0)))
                .expect("artillery aims at the fort");
        let joined =
            super::align_and_combine(&gs, &staged, &shot(batteries[1], HexCoord::new(0, 1)))
                .expect("a battery joins at the fort");
        assert!(joined.at_fort);
        assert_eq!(joined.firers.len(), 2);
        let refused = super::align_and_combine(&gs, &staged, &shot(rifles, HexCoord::new(2, 0)))
            .expect_err("rifles may not fire at a fort");
        assert!(refused.contains("only artillery"), "{refused}");
    }
}

#[cfg(test)]
mod artillery_only_tests {
    use omdurman_rules::{FireKind, Phase, UnitId, UnitPlacement};
    use omdurman_types::{HexCoord, Player, Scenario};

    /// §6.61: rifles in range of a lone Dervish gunboat may not fire at it;
    /// its hex is marked artillery-only, a tribal stack beside it is not.
    #[test]
    fn a_lone_gunboat_is_marked_artillery_only_for_rifles() {
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        gs.phase = Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire);
        gs.active_player = Player::AngloEgyptian;
        let boat = UnitId::KhalifaAbdullah_1_0;
        for (id, hex) in [
            (UnitId::BritishArmy_0_0, HexCoord::new(0, 0)),
            (boat, HexCoord::new(1, 0)),
            (UnitId::Baggara_0_0, HexCoord::new(0, 1)),
        ] {
            gs.units.push(UnitPlacement {
                id,
                position: hex,
                profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
                state: Default::default(),
            });
        }
        let kinds = [(UnitId::BritishArmy_0_0, FireKind::Direct)];
        assert_eq!(
            super::artillery_only_hexes(&gs, &kinds),
            vec![HexCoord::new(1, 0)]
        );
    }
}

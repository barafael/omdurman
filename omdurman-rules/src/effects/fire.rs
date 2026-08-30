use super::*;

/// Validate and apply a direct/Maxim-second fire attack (rulebook §6).
pub fn apply_fire_combat(
    state: &mut GameState,
    attack: &FireAttack,
    roll: DieRoll,
) -> Result<(), RuleError> {
    resolve_fire_attack(state, attack, attack.target_hex, roll)
}

/// Validate and apply a howitzer fire attack (scatter path) (rulebook §6.64).
pub fn apply_howitzer_fire(
    state: &mut GameState,
    attack: &FireAttack,
    combat_results_table_roll: DieRoll,
    impact_roll: DieRoll,
) -> Result<(), RuleError> {
    // Legality (incl. no-howitzer-at-night §6.64) is validated in
    // `resolve_fire_attack` -> `validate_fire_attack` -> `can_fire_at`.
    //
    // AMBIGUITY (§6.42): "Howitzer fire may be combined with Maxim fire,
    // but only if the howitzer fire impacts in the intended hex."  The
    // manual does not clarify what happens to the Maxim fire when the
    // howitzer scatters.  The code treats howitzer and Maxim attacks as
    // independent fire actions — a scattered howitzer does not prevent a
    // Maxim from firing at the original target hex separately.

    // §6.64: roll twice -- the first roll resolves on the Combat Results Table,
    // the second (impact) roll places the shell. The target hex is hit on 7-10;
    // otherwise the shell scatters and "the results must take effect, even if
    // the fire scatters into a friendly-occupied hex."
    let scatter = howitzer_scatter(impact_roll);
    let firer_pos = attack
        .firers
        .first()
        .and_then(|id| state.find_unit(*id))
        .map(|u| u.position);
    let actual_target = state.howitzer_impact_hex(attack.target_hex, firer_pos, scatter);
    resolve_fire_attack(state, attack, actual_target, combat_results_table_roll)
}

/// Look up the range-effects band for a firing unit. Normally Anglo-Egyptian
/// units use their own table and Dervish units the Dervish table (§6.22), but
/// in FALL OF KHARTOUM *both* players use the Dervish Range Effects Table
/// (§9.343).
pub fn range_band_for(
    scenario: Scenario,
    player: Player,
    weapon: WeaponClass,
    range: HexDistance,
) -> crate::RangeBand {
    if scenario == Scenario::FallOfKhartoum {
        return dervish_range_effects(weapon, range);
    }
    match player {
        Player::AngloEgyptian => ae_range_effects(weapon, range),
        Player::Dervish => dervish_range_effects(weapon, range),
    }
}

/// Which player's Range Effects Table a given unit fires on (§6.22, §6.52,
/// §9.343). Resolved **per firer**: in FALL OF KHARTOUM every unit uses the
/// Dervish table (§9.343); "Friendlies" units fire their rifles on the
/// Dervish table (§6.52); everyone else uses their own side's table. Used
/// identically by validation (`can_fire_at`) and resolution
/// (`resolve_fire_attack`) so the two can never disagree on range -- the
/// audit class where a Friendlies shot passed validation on the
/// Anglo-Egyptian table (rifle max 5) but resolved on the Dervish table
/// (rifle max 4).
#[allow(clippy::if_same_then_else)] // §9.343 and §6.52 both yield Dervish for different reasons
pub(crate) fn range_table_player_for(scenario: Scenario, unit: &UnitPlacement) -> Player {
    if scenario == Scenario::FallOfKhartoum {
        Player::Dervish // §9.343
    } else if unit.profile.identity.is_friendlies() {
        Player::Dervish // §6.52
    } else {
        unit.profile.identity.owner()
    }
}

/// The weapon line a unit fires on for an attack of `kind` (§6.64): named
/// gunboats carry Artillery on their profile but fire howitzers in the
/// Maxim/Howitzer subphase, so a `Howitzer`-kind attack always uses the
/// howitzer line.
pub(crate) fn effective_fire_weapon(unit: &UnitPlacement, kind: FireKind) -> WeaponClass {
    if kind == FireKind::Howitzer {
        WeaponClass::Howitzer
    } else {
        unit.profile.weapon
    }
}

/// The distance to consult the range tables at, after the §8.1 night cap:
/// halve the weapon's maximum range (on the table the unit fires on), then
/// consult the day table at the *physical* distance. Returns `None` when the
/// physical distance exceeds the night maximum (target out of range at
/// night).
pub(crate) fn night_capped_distance(
    weapon: WeaponClass,
    table_player: Player,
    distance: HexDistance,
) -> Option<HexDistance> {
    let night_max =
        crate::range_effects::night_max_range(weapon, table_player == Player::AngloEgyptian);
    (distance.value() <= night_max as u16).then_some(distance)
}

/// Mark an accepted fire attack's firers and targets as having fired / been
/// fired at (§6.14). Split out of [`resolve_fire_attack`] so every validation
/// runs *before* any mutation: on `Err` the state must be byte-identical, or a
/// peer that rejects the effect diverges from one that accepts it.
///
/// Maxim guns and gunboats are the §6.14 parenthetical exceptions to
/// "may only be fired at once", so they are never added to the fired-at set.
fn commit_fired_markers(state: &mut GameState, attack: &FireAttack, target_units: &[UnitId]) {
    for &id in &attack.firers {
        state.units_fired_this_phase.push(id);
    }
    for &tid in target_units {
        let excepted = state
            .find_unit(tid)
            .is_some_and(|u| fired_at_excepted(u.profile.kind));
        if !excepted {
            state.units_fired_at_this_phase.push(tid);
        }
    }
}

/// Resolve a fire attack: compute range, look up range effects, compute effective factor, roll on CRT (rulebook §6).
pub fn resolve_fire_attack(
    state: &mut GameState,
    attack: &FireAttack,
    target_hex: HexCoord,
    roll: DieRoll,
) -> Result<(), RuleError> {
    validate_fire_attack(state, attack)?;

    // §6.14/§6.61/§6.62 validation that needs the *target* hex runs below,
    // before any mutation: `apply_effect` must leave the state untouched when
    // it returns `Err`, or a peer that rejects an effect diverges from one that
    // accepts it (events are applied only on the host-sequenced echo and a
    // rejected effect is never retried). Marking the firers as having fired is
    // deferred to `commit_fired_markers` below.

    // §6.22: each firer contributes at its *own* distance, on its *own*
    // weapon line and range-effects table (§6.52 Friendlies -> Dervish table,
    // §9.343 FoK -> Dervish table for both sides), with the §8.1 night cap
    // applied per weapon. Previously the whole attack used the *first*
    // firer's weapon/distance/table, which mis-banded mixed attacks (e.g. a
    // spear-armed unit stacked with a fort battery dragged the battery onto
    // the spear line) and let a Friendlies rifle pass validation at range 5
    // (AE table) while resolving on the Dervish table (max 4). The helpers
    // are shared with `can_fire_at` so validation and resolution cannot
    // disagree.
    let mut effective_total: u16 = 0;
    // Representative values for the `FireResolved` observation (first firer's
    // distance/band); with per-firer bands there is no single attack-wide one.
    let mut representative_range: Option<u16> = None;
    let mut representative_band: Option<crate::RangeBand> = None;
    for &id in &attack.firers {
        let Some(u) = state.find_unit(id) else {
            continue;
        };
        let weapon = effective_fire_weapon(u, attack.kind);
        let table_player = range_table_player_for(state.scenario, u);
        let distance = HexDistance(u.position.distance(target_hex) as u16);
        let distance = if state.day_night == DayNight::Night {
            // Beyond the night cap the band is OutOfRange (§8.1) -- validation
            // already rejects that case; a scatter into a night-out-of-range
            // hex simply contributes nothing.
            night_capped_distance(weapon, table_player, distance).unwrap_or(HexDistance(u16::MAX))
        } else {
            distance
        };
        let band = range_band_for(state.scenario, table_player, weapon, distance);
        if representative_range.is_none() {
            representative_range = Some(distance.value());
            representative_band = Some(band);
        }
        if let Some(f) = u.profile.fire {
            effective_total = effective_total.saturating_add(band.apply(f.value()));
        }
    }
    // Engine-authoritative terrain defence modifier (§6.23): derived from
    // `state.board` at the target hex, not from a caller-supplied value. This
    // applies to howitzer scatter too — `target_hex` is the *actual* impact.
    let terrain = state
        .board
        .terrain_at(target_hex)
        .unwrap_or(omdurman_types::Terrain::Clear {
            road: Default::default(),
        });
    let terrain_mod = crate::terrain_chart::defense_modifier(terrain);
    // §6.24/§5.54/§9.231/§9.232: the engine derives the mandatory modifiers
    // itself (like the §6.23 terrain modifier below) -- the caller's list is
    // checked for equality in `validate_fire_attack` but never trusted for
    // the arithmetic.
    let derived_mod: i16 = mandatory_fire_modifiers(state, attack)
        .iter()
        .map(|m| m.die_modifier())
        .sum();
    let total_mod = derived_mod + terrain_mod;
    let modified_roll = roll.apply_modifier(total_mod);
    let row = FireFactorRow::from_total(effective_total);
    let result = combat_results_table(row, modified_roll);
    let target_units: Vec<UnitId> = state
        .player_units_in_hex(target_hex, attack.firing_player.opponent())
        .iter()
        .map(|u| u.id)
        .collect();

    // §6.61/§6.62: gunboats and forts are special targets -- only artillery (or
    // howitzer-class) fire may engage them, and they are destroyed only on a
    // Combat Results Table *cell value* meeting a threshold (gunboat 3+, fort
    // 2+), *not* by the generic disrupt/eliminate effect.  "3 or more on the
    // combat results table" means Eliminate(3) or higher, not a die roll of 3+.
    let opponent = attack.firing_player.opponent();
    // §6.14: "a combat unit may only fire once and may only be fired at once
    // (exceptions: Maxim guns and gunboats)". Any non-excepted target unit
    // already fired at this phase makes the attack illegal -- two attacks on
    // the same hex (or its survivors) in one phase fire at the same units.
    for &tid in &target_units {
        let already = state.units_fired_at_this_phase.contains(&tid);
        let excepted = state
            .find_unit(tid)
            .is_some_and(|u| fired_at_excepted(u.profile.kind));
        if already && !excepted {
            return Err(RuleError::AlreadyFiredAt(tid));
        }
    }
    // §6.61/§6.62 defence-in-depth (per firer, matching `can_fire_at`): every
    // firer must fire on an artillery line to engage a gunboat/fort. Checked
    // here, before any mutation, for the atomicity reason above.
    let special = state.special_fire_target(&target_units);
    if special.is_some() {
        let all_artillery = attack
            .firers
            .iter()
            .filter_map(|id| state.find_unit(*id))
            .all(|u| {
                matches!(
                    effective_fire_weapon(u, attack.kind),
                    WeaponClass::Artillery | WeaponClass::Howitzer
                )
            });
        if !all_artillery {
            return Err(RuleError::ArtilleryOnlyVsGunboatOrFort(attack.firers[0]));
        }
    }

    // ---- validation complete; from here on the state is mutated ----
    commit_fired_markers(state, attack, &target_units);

    if let Some((special_id, special_kind)) = special {
        let needed = match special_kind {
            UnitKind::Gunboat { .. } => 3, // §6.61
            UnitKind::Fort { .. } => 2,    // §6.62
            _ => unreachable!("special_fire_target only returns gunboat/fort"),
        };
        let destroyed = matches!(result, CombatResult::Eliminate(n) if n >= needed);
        // Snapshot the special target's occupants before mutation so the
        // FireResolved observation can report the eliminations accurately --
        // `apply_combat_results_table_result` and the retain() below both
        // mutate `state.units` in place.
        let pre_units: Vec<UnitId> = target_units.clone();
        if destroyed {
            state.units.retain(|u| u.id != special_id);
            // If a gunboat carrying Friendlies is sunk, the loaded unit is
            // lost (§5.21 — design choice).
            if matches!(special_kind, UnitKind::Gunboat { .. }) {
                remove_friendlies_on_gunboat(state, special_id);
            }
            // §6.62: if a destroyed fort contained enemy units, one is
            // eliminated with it.
            if matches!(special_kind, UnitKind::Fort { .. })
                && let Some(&victim) = target_units.iter().find(|&&id| id != special_id)
            {
                state.units.retain(|u| u.id != victim);
            }
        }
        // §6.82 with §6.61/§6.62 (offensive fire only -- §6.7 bars advances
        // from defensive fire): if the special target's destruction left the
        // hex without any enemy units, the participating firers may advance
        // into it.
        let hex_still_defended = state
            .units
            .iter()
            .any(|u| u.position == target_hex && u.profile.identity.owner() == opponent);
        if !hex_still_defended && matches!(state.phase, Phase::OffensiveFire(_)) {
            let mut paragraphs = vec!["6.82".to_string()];
            paragraphs.push(match special_kind {
                UnitKind::Gunboat { .. } => "6.61".to_string(),
                _ => "6.62".to_string(),
            });
            open_advance_window(state, target_hex, &attack.firers, paragraphs);
        }
        let eliminations: Vec<UnitId> = diff_eliminated(state, pre_units);
        state.turn_events.push(TurnEventRecord::FireCombat {
            attacker: attack.firing_player,
            firers: attack.firers.clone(),
            target: target_hex,
            roll,
            modifiers: attack.modifiers.clone(),
            total_modifier: total_mod,
            result,
            kind: attack.kind,
            eliminated: eliminations.clone(),
        });
        state.observations.push(Observation::FireResolved {
            // Deliberate clone: observations are self-contained records for
            // replay/UI and must own their attack, not borrow it.
            attack: attack.clone(),
            roll,
            total_modifier: total_mod,
            modified_roll,
            factor_row: row,
            effective_factor: effective_total,
            result,
            eliminations,
            range: representative_range,
            band: representative_band.map(|b| format!("{b:?}")),
            paragraphs: fire_paragraphs(attack.kind, Some(special_kind)),
        });
        return Ok(());
    }

    let pre_units: Vec<UnitId> = target_units.clone();
    let was_occupied = !target_units.is_empty();
    apply_combat_results_table_result(state, result, &target_units, opponent);
    let eliminations: Vec<UnitId> = diff_eliminated(state, pre_units);
    state.observations.push(Observation::FireResolved {
        // Deliberate clone: observations are self-contained records for
        // replay/UI and must own their attack, not borrow it.
        attack: attack.clone(),
        roll,
        total_modifier: total_mod,
        modified_roll,
        factor_row: row,
        effective_factor: effective_total,
        result,
        eliminations,
        range: representative_range,
        band: representative_band.map(|b| format!("{b:?}")),
        paragraphs: fire_paragraphs(attack.kind, None),
    });
    // §6.82 (offensive fire only -- §6.7: "There is no advance after combat
    // as a result of defensive fires"): if offensive fire left the target
    // hex without enemy units, the participating firers may advance into it.
    // `was_occupied` keeps a howitzer scatter onto a never-occupied hex
    // (§6.64) from opening a bogus window -- §6.82's "enemy-occupied hex is
    // vacated" never held.
    let hex_still_defended = state
        .units
        .iter()
        .any(|u| u.position == target_hex && u.profile.identity.owner() == opponent);
    if was_occupied && !hex_still_defended && matches!(state.phase, Phase::OffensiveFire(_)) {
        open_advance_window(state, target_hex, &attack.firers, vec!["6.82".to_string()]);
    }
    Ok(())
}

/// §6.14's fired-at exception: Maxim guns and gunboats may be fired at more
/// than once per fire phase.
pub(crate) fn fired_at_excepted(kind: UnitKind) -> bool {
    matches!(kind, UnitKind::Gunboat { .. } | UnitKind::Maxim { .. })
}

/// Rulebook paragraphs that authorise a fire resolution, for the UI's
/// combat-resolution card. The set depends on the kind of fire (direct vs
/// howitzer vs Maxim second) and on whether a special target (gunboat/fort)
/// was hit -- each branch carries the sections a player would point at to
/// explain "why did that shot do what it did".
fn fire_paragraphs(kind: FireKind, special: Option<UnitKind>) -> Vec<String> {
    let kind_para = match kind {
        FireKind::Direct => "6.24", // direct-fire modifiers
        FireKind::MaximSecondFire => "6.42",
        FireKind::Howitzer => "6.64",
    };
    let special_para = match special {
        Some(UnitKind::Gunboat { .. }) => "6.61",
        Some(UnitKind::Fort { .. }) => "6.62",
        _ => "6.23", // terrain defence modifier
    };
    // 6.22 is the CRT itself; always cited.
    vec!["6.22".into(), kind_para.into(), special_para.into()]
}

// ---------------------------------------------------------------------------
// 8) Melee combat
// ---------------------------------------------------------------------------

/// Validate that a fire attack is legal in the current state (rulebook §6).
///
/// Single source of truth: every firer is checked through [`can_fire_at`], the
/// same predicate the UI gates clicks on -- so a shot the UI offers is exactly a
/// shot `apply` accepts (phase, owner, sub-phase/kind, weapon class, howitzer-
/// at-night §6.64, disruption, already-fired, gunboat/fort-needs-artillery
/// §6.61/§6.62, and range §6.22). An empty firer list is rejected.
/// The die-roll modifiers the rulebook *mandates* for a fire attack, derived
/// from the game state (rulebook §6.24, §5.54, §9.231, §9.232). The engine is
/// authoritative: resolution applies exactly this set (plus the engine-side
/// terrain modifier §6.23), and a caller-supplied `attack.modifiers` list that
/// differs is rejected in [`validate_fire_attack`] -- a client can neither
/// omit a mandatory bonus/penalty nor smuggle one in (e.g. a `Terrain(n)`
/// entry would double-count the engine's own §6.23 modifier).
pub fn mandatory_fire_modifiers(state: &GameState, attack: &FireAttack) -> Vec<FireModifier> {
    let mut modifiers = Vec::new();
    // §6.24: "+1 modifier to their die roll" for all Anglo-Egyptian *direct*
    // fire attacks. Maxim second fire and howitzer fire get neither this nor
    // brigade integrity.
    if attack.kind == FireKind::Direct && attack.firing_player == Player::AngloEgyptian {
        modifiers.push(FireModifier::AngloEgyptianDirectFire);
        // §5.54/§6.24: brigade integrity (+1, cumulative) when all four
        // battalions of one brigade are stacked in the same hex and all fire
        // at this target hex -- i.e. the firers are exactly such a stack.
        let firers: Vec<&UnitPlacement> = attack
            .firers
            .iter()
            .filter_map(|id| state.find_unit(*id))
            .collect();
        let co_stacked = firers
            .first()
            .is_some_and(|first| firers.iter().all(|u| u.position == first.position));
        if co_stacked {
            let identities: Vec<crate::UnitIdentity> =
                firers.iter().map(|u| u.profile.identity).collect();
            if matches!(
                crate::brigade_integrity(&identities),
                crate::BrigadeIntegrity::Integrated(_)
            ) {
                modifiers.push(FireModifier::BrigadeIntegrity);
            }
        }
    }
    // §9.231/§9.232: the zariba die-roll penalties apply "on all *Dervish*
    // fire attacks" (thorn hedge −2; trench −4 vs. entrenched units) -- never
    // to Anglo-Egyptian fire.
    if attack.firing_player == Player::Dervish {
        if state.board.has_zariba_thorn_hedge(attack.target_hex) {
            modifiers.push(FireModifier::ZaribaThornHedge);
        }
        if state.board.is_zariba_entrenched(attack.target_hex) {
            modifiers.push(FireModifier::ZaribaTrenchEntrenched);
        }
    }
    modifiers
}

pub fn validate_fire_attack(state: &GameState, attack: &FireAttack) -> Result<(), RuleError> {
    if attack.firers.is_empty() {
        return Err(RuleError::NoFirers);
    }
    for &id in &attack.firers {
        state.can_fire_at(id, attack.target_hex, attack.kind)?;
    }
    // §6.24/§5.54/§9.231/§9.232: the caller's modifier list must match the
    // engine-derived mandatory set exactly (the modifiers are documentation
    // for the UI; the engine resolves with its own derivation either way).
    let mandatory = mandatory_fire_modifiers(state, attack);
    if attack.modifiers != mandatory {
        return Err(RuleError::FireModifierMismatch {
            expected: mandatory,
            got: attack.modifiers.clone(),
        });
    }
    Ok(())
}

/// Apply a Combat Results Table result to a list of target units -- eliminate `n` and disrupt
/// half (round up) of the remaining (rulebook §6.22, §7.7).
pub(crate) fn apply_combat_results_table_result(
    state: &mut GameState,
    result: CombatResult,
    target_ids: &[UnitId],
    target_player: Player,
) {
    match result {
        CombatResult::NoEffect => {}
        CombatResult::Disrupt => {
            // Disrupt half (round up) of the target units.
            let n = target_ids.len().div_ceil(2);
            for &id in target_ids.iter().take(n) {
                if let Some(unit) = state.find_unit_mut(id) {
                    unit.state.disrupted = true;
                }
            }
        }
        CombatResult::Eliminate(n) => {
            let n = (n as usize).min(target_ids.len());
            // Half (round up) of the survivors are also disrupted.
            let disrupt_n = target_ids.len().saturating_sub(n).div_ceil(2);

            for &id in target_ids.iter().take(n) {
                score_elimination(state, id, ElimCause::Combat);
            }
            // §5.21: if a gunboat is sunk while carrying a "Friendlies" unit,
            // the loaded unit is lost with the ship (and its explosive
            // ammunition). Cascade the elimination before removing units.
            let mut cascade: Vec<UnitId> = Vec::new();
            for &id in target_ids.iter().take(n) {
                if state
                    .find_unit(id)
                    .map(|u| matches!(u.profile.kind, UnitKind::Gunboat { .. }))
                    .unwrap_or(false)
                {
                    for u in &state.units {
                        if u.state.loaded_on == Some(id) {
                            cascade.push(u.id);
                        }
                    }
                }
            }
            for cid in &cascade {
                state.observations.push(Observation::UnitEliminated {
                    id: *cid,
                    cause: ElimCause::LostWithTransport,
                    vp_source: None,
                });
            }
            state
                .units
                .retain(|u| !target_ids[..n].contains(&u.id) && !cascade.contains(&u.id));

            // §5.44 orphan leader: if all combat units (non-leader) in the
            // target hex were eliminated, any surviving AE leader in that hex
            // is also eliminated (the leader cannot exist alone on the
            // battlefield).
            if target_player == Player::AngloEgyptian {
                let eliminated_hexes: Vec<HexCoord> = target_ids[..n]
                    .iter()
                    .filter_map(|id| state.find_unit(*id).map(|u| u.position))
                    .collect();
                for hex in eliminated_hexes {
                    let leader_ids: Vec<UnitId> = state
                        .units
                        .iter()
                        .filter(|u| {
                            u.position == hex && u.profile.identity.owner() == Player::AngloEgyptian
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .filter(|u| matches!(u.profile.kind, UnitKind::BritishLeader { .. }))
                        .map(|u| u.id)
                        .collect();
                    if leader_ids.is_empty() {
                        continue;
                    }
                    let has_combat_unit = state.units.iter().any(|u| {
                        u.position == hex
                            && u.profile.identity.owner() == Player::AngloEgyptian
                            && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
                    });
                    if !has_combat_unit {
                        for &id in &leader_ids {
                            score_elimination(state, id, ElimCause::Combat);
                        }
                        for &id in &leader_ids {
                            state.observations.push(Observation::UnitEliminated {
                                id,
                                cause: ElimCause::OrphanLeader,
                                vp_source: None,
                            });
                        }
                        state.units.retain(|u| !leader_ids.contains(&u.id));
                    }
                }
            }

            // Disrupt survivors.
            let survivors: Vec<UnitId> = target_ids[n..].to_vec();
            for &id in survivors.iter().take(disrupt_n) {
                if let Some(unit) = state.find_unit_mut(id) {
                    unit.state.disrupted = true;
                    state
                        .turn_events
                        .push(TurnEventRecord::UnitDisrupted { unit: id });
                }
            }
        }
    }
}

/// §6.63 3rd bullet: artillery fire aimed at breaching a wall hexside. Only
/// artillery-class firers may participate; the CRT is rolled with the
/// firers' combined fire factor (each firer's contribution halved per its
/// range band to the *nearer* endpoint of the wall hexside, floored at 1 per
/// unit, §6.16). A result of `Eliminate(2)` or higher breaches the wall --
/// flipping the `Wall` hexside to `Breach` (so it no longer blocks LOS,
/// movement, melee, or ZOC) and eliminating one enemy unit adjacent to the
/// breached hexside. Any other CRT result is a miss.
///
/// This mirrors the Royal-Engineers demolition path (`apply_resolve_demolition`)
/// for the wall case but trades the Engineers' guaranteed success for the
/// artillery's CRT roll -- the rulebook specifies the same "2+ required"
/// threshold for both trigger styles.
pub fn apply_artillery_breach_wall(
    state: &mut GameState,
    firers: &[UnitId],
    target: HexsideRef,
    roll: DieRoll,
) -> Result<(), RuleError> {
    if firers.is_empty() {
        return Err(RuleError::NoFirers);
    }

    // Phase must be a fire-combat phase (defensive or offensive; either
    // sub-phase is fine -- artillery breaching is not tied to the
    // Maxim/Howitzer sub-phase the way Maxims are).
    let firing_player = match state.phase {
        Phase::OffensiveFire(_) => state.active_player,
        Phase::DefensiveFire(_) => state.active_player.opponent(),
        _ => return Err(RuleError::WrongPhase),
    };

    // The target hexside must currently be a Wall. (If it's already a Breach
    // or Gate there's nothing to do; if it's missing entirely the data is
    // wrong. Either way the player has misclicked.)
    match state.board.hexsides.get(&target) {
        Some(HexsideKind::Wall) => {}
        _ => return Err(RuleError::NotAWallHexside(target)),
    }

    // Validate every firer (all-or-nothing) and accumulate the effective CRT
    // factor. See `can_fire_at_wall` for the per-firer rules.
    let mut effective_total: u16 = 0;
    // Ordered set: pure duplicate detection, so ordering is irrelevant to the
    // result and this keeps the fire path free of `hashbrown`.
    let mut seen: std::collections::BTreeSet<UnitId> = std::collections::BTreeSet::new();
    for &id in firers {
        if !seen.insert(id) {
            return Err(RuleError::AlreadyFired(id));
        }
        let (fire_factor, range, nearer_hex) = state.can_fire_at_wall(id, target)?;
        let band = range_band_for(
            state.scenario,
            firing_player,
            state.unit_or_err(id)?.profile.weapon,
            range,
        );
        effective_total = effective_total.saturating_add(band.apply(fire_factor.value()));

        // LOS already verified by `can_fire_at_wall`; the band lookup above is
        // the only additional per-firer work. `nearer_hex` is kept for
        // potential future error reporting.
        let _ = nearer_hex;
    }

    // All firers pass -- mark them as having fired this phase.
    for &id in firers {
        state.units_fired_this_phase.push(id);
    }

    // §6.63: "A result of 2 or more on the combat results table is required
    // to breach a wall." The CRT cell value (Eliminate(N)) is the relevant
    // metric, identical to the §6.61/§6.62 gunboat/fort thresholds.
    let row = FireFactorRow::from_total(effective_total);
    let result = combat_results_table(row, roll);
    let breached = matches!(result, CombatResult::Eliminate(n) if n >= 2);

    let mut adjacent_eliminated: Option<UnitId> = None;
    if breached {
        // Flip Wall → Breach.
        if let Some(kind) = state.board.hexsides.get_mut(&target) {
            if *kind == HexsideKind::Wall {
                *kind = HexsideKind::Breach;
            }
        } else {
            state.board.hexsides.insert(target, HexsideKind::Breach);
        }

        // §6.63: "If any enemy units are adjacent to the wall hexside at the
        // instant it is breached, one enemy unit is eliminated." Pick the
        // first such unit (matching the demolition path's convention).
        let opponent = firing_player.opponent();
        if let Some(victim) = state.units.iter().find_map(|u| {
            let is_enemy = u.profile.identity.owner() == opponent;
            let adjacent =
                u.position.is_adjacent_to(target.a) || u.position.is_adjacent_to(target.b);
            (is_enemy && adjacent).then_some(u.id)
        }) {
            score_elimination(state, victim, ElimCause::Demolition);
            state.units.retain(|u| u.id != victim);
            adjacent_eliminated = Some(victim);
        }
    }

    state.observations.push(Observation::WallBreached {
        hexside: target,
        // Truthful outcome: `breached` reflects whether the §6.63
        // threshold (CRT cell 2+) was met, so a short roll logs as a
        // failed attempt rather than a phantom breach.
        breached,
        row: Some(row),
        adjacent_eliminated,
    });
    state.turn_events.push(TurnEventRecord::FireCombat {
        attacker: firing_player,
        firers: firers.to_vec(),
        target: target.a,
        roll,
        modifiers: Vec::new(),
        total_modifier: 0,
        result,
        kind: FireKind::Direct,
        eliminated: adjacent_eliminated.into_iter().collect(),
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// 10) Reinforcements
// ---------------------------------------------------------------------------

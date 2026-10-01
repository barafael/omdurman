use super::*;

/// Validate and apply a direct/Maxim-second fire attack (rulebook §6).
pub fn apply_fire_combat(
    state: &mut GameState,
    attack: &FireAttack,
    roll: DieRoll,
    disruption: DisruptionDraw,
) -> Result<(), RuleError> {
    // §6.64: howitzer fire always rolls for scatter -- it travels as
    // `HowitzerFire`, never as a plain `FireCombat`.
    if attack.kind == FireKind::Howitzer {
        return Err(RuleError::FireKindMismatch);
    }
    resolve_fire_attack(state, attack, attack.target_hex, roll, disruption)
}

/// Validate and apply a howitzer fire attack (scatter path) (rulebook §6.64).
pub fn apply_howitzer_fire(
    state: &mut GameState,
    attack: &FireAttack,
    combat_results_table_roll: DieRoll,
    impact_roll: DieRoll,
    disruption: DisruptionDraw,
) -> Result<(), RuleError> {
    if attack.kind != FireKind::Howitzer {
        return Err(RuleError::FireKindMismatch);
    }
    // Legality (incl. no-howitzer-at-night §6.64) is validated against the
    // *aimed* hex -- the shot the player declared -- before anything is
    // mutated: a rejected effect must leave the state untouched.
    validate_fire_resolution(state, attack)?;
    //
    // AMBIGUITY (§6.42): "Howitzer fire may be combined with Maxim fire,
    // but only if the howitzer fire impacts in the intended hex."  The
    // manual does not clarify what happens to the Maxim fire when the
    // howitzer scatters.  The code treats howitzer and Maxim attacks as
    // independent fire actions — a scattered howitzer does not prevent a
    // Maxim from firing at the original target hex separately.

    // ---- validation complete; from here on the state is mutated ----
    // §6.64: roll twice -- the first roll resolves on the Combat Results Table,
    // the second (impact) roll places the shell. The target hex is hit on 7-10;
    // otherwise the shell scatters and "the results must take effect, even if
    // the fire scatters into a friendly-occupied hex."
    let actual_target = crate::howitzer_scatter::scatter_impact_hex(
        attack.target_hex,
        howitzer_scatter(impact_roll),
    );
    // §6.64: record where the shell actually landed so every seat (and the
    // UI's impact marker) can show the scatter, not just the aimed hex.
    state.turn_events.push(TurnEventRecord::HowitzerImpact {
        at: actual_target,
        scattered: actual_target != attack.target_hex,
    });
    // §6.64: "the results must take effect" on *everyone* in the impact hex,
    // friend or foe -- at a fort, on the units inside it (§6.54), or on the
    // fort itself when the shell hits the fort it was aimed at or an empty
    // one.
    let everyone: Vec<&UnitPlacement> = state.units_in_hex(actual_target);
    let aimed_at_fort = attack.at_fort && actual_target == attack.target_hex;
    let garrison: Vec<UnitId> = everyone
        .iter()
        .filter(|u| !matches!(u.profile.kind, UnitKind::Fort { .. }))
        .map(|u| u.id)
        .collect();
    let target_units: Vec<UnitId> = if aimed_at_fort || garrison.is_empty() {
        everyone.iter().map(|u| u.id).collect()
    } else {
        garrison
    };
    commit_fire_attack(
        state,
        attack,
        actual_target,
        &target_units,
        combat_results_table_roll,
        disruption,
        Some((impact_roll, actual_target)),
    );
    Ok(())
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

/// Whether every weapon in `attack` fires on an artillery line -- the only
/// fire that may engage a gunboat or a fort itself (§6.61, §6.62). The
/// first that does not, if any.
fn first_non_artillery_shot(state: &GameState, attack: &FireAttack) -> Option<UnitId> {
    attack
        .shots()
        .into_iter()
        .find(|shot| {
            !state.find_unit(shot.unit).is_some_and(|u| {
                matches!(
                    u.weapon_line(shot.mount, attack.kind),
                    WeaponClass::Artillery | WeaponClass::Howitzer
                )
            })
        })
        .map(|shot| shot.unit)
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
    for shot in attack.shots() {
        match shot.mount {
            FireMount::Main => state.units_fired_this_phase.push(shot.unit),
            FireMount::GunboatMaxims => state.gunboat_maxims_fired_this_phase.push(shot.unit),
        }
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

/// Validate and resolve a fire attack at `target_hex`: compute range, look up
/// range effects, compute the effective factor, roll on the CRT (rulebook §6).
/// Every validation runs against the attack's declared `target_hex` before
/// any mutation; `target_hex` is the hex the CRT result lands on (the same
/// hex for direct fire).
pub fn resolve_fire_attack(
    state: &mut GameState,
    attack: &FireAttack,
    target_hex: HexCoord,
    roll: DieRoll,
    disruption: DisruptionDraw,
) -> Result<(), RuleError> {
    validate_fire_resolution(state, attack)?;
    let target_units = fire_target_units(state, attack, target_hex);
    commit_fire_attack(
        state,
        attack,
        target_hex,
        &target_units,
        roll,
        disruption,
        None,
    );
    Ok(())
}

/// The enemy units a fire attack at `hex` strikes (§6.54, §6.62): every
/// enemy unit there -- but at an enemy fort, either the fort itself (with
/// its occupants, one of whom falls with it) when the attack is aimed at
/// the fort, or else only the units stacked inside it.
pub fn fire_target_units(state: &GameState, attack: &FireAttack, hex: HexCoord) -> Vec<UnitId> {
    let enemy = state.player_units_in_hex(hex, attack.firing_player.opponent());
    enemy
        .iter()
        .filter(|u| attack.at_fort || !matches!(u.profile.kind, UnitKind::Fort { .. }))
        .map(|u| u.id)
        .collect()
}

/// The Combat Results Table result an attack needs to destroy its target
/// when that is one of the artillery's special targets -- 3 or more to sink
/// a gunboat (§6.61), 2 or more to destroy a fort (§6.62), anything less a
/// miss; `None` for ordinary targets.
pub fn special_target_threshold(state: &GameState, attack: &FireAttack) -> Option<u8> {
    let targets = fire_target_units(state, attack, attack.target_hex);
    state
        .special_fire_target(&targets)
        .map(|(_, _, needed)| needed)
}

/// §6.54: aim an attack at the enemy fort in its target hex instead of at
/// the units inside it -- artillery only (§6.62). `None` when the hex holds
/// no enemy fort or a firer is not on an artillery line.
pub fn aim_at_fort(state: &GameState, attack: &FireAttack) -> Option<FireAttack> {
    let has_fort = state
        .player_units_in_hex(attack.target_hex, attack.firing_player.opponent())
        .iter()
        .any(|u| matches!(u.profile.kind, UnitKind::Fort { .. }));
    let all_artillery = first_non_artillery_shot(state, attack).is_none();
    (has_fort && all_artillery).then(|| FireAttack {
        at_fort: true,
        ..attack.clone()
    })
}

/// Every fire-attack validation, against the attack's declared
/// `target_hex` (§6.14, §6.61, §6.62 on top of [`validate_fire_attack`]).
/// Read-only: `apply_effect` must leave the state untouched when it returns
/// `Err`, or a peer that rejects an effect diverges from one that accepts it
/// (events are applied only on the host-sequenced echo and a rejected effect
/// is never retried).
fn validate_fire_resolution(state: &GameState, attack: &FireAttack) -> Result<(), RuleError> {
    validate_fire_attack(state, attack)?;
    // §6.54/§6.62: an attack aims at a fort only where there is one; an
    // empty fort is only ever fired at itself.
    let enemy = state.player_units_in_hex(attack.target_hex, attack.firing_player.opponent());
    let is_fort = |u: &&UnitPlacement| matches!(u.profile.kind, UnitKind::Fort { .. });
    if attack.at_fort && !enemy.iter().any(is_fort) {
        return Err(RuleError::NoFortToFireAt(attack.target_hex));
    }
    if !attack.at_fort && !enemy.is_empty() && enemy.iter().all(is_fort) {
        return Err(RuleError::FortStandsEmpty(attack.target_hex));
    }
    let target_units = fire_target_units(state, attack, attack.target_hex);
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
    // firer must fire on an artillery line to engage a gunboat/fort.
    if state.special_fire_target(&target_units).is_some()
        && let Some(unit) = first_non_artillery_shot(state, attack)
    {
        return Err(RuleError::ArtilleryOnlyVsGunboatOrFort(unit));
    }
    Ok(())
}

/// Resolve an already-validated fire attack against `target_units` in
/// `target_hex` (the aimed hex, or a howitzer's §6.64 impact hex).
/// Infallible: all legality was established by [`validate_fire_resolution`].
fn commit_fire_attack(
    state: &mut GameState,
    attack: &FireAttack,
    target_hex: HexCoord,
    target_units: &[UnitId],
    roll: DieRoll,
    disruption: DisruptionDraw,
    impact: Option<(DieRoll, HexCoord)>,
) {
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
    let contributions = firer_contributions(state, attack);
    let effective_total: u16 = contributions
        .iter()
        .fold(0u16, |sum, c| sum.saturating_add(c.factor));
    // Representative values for the `FireResolved` observation (first firer's
    // distance/band); with per-firer bands there is no single attack-wide one.
    let representative_range = contributions.first().map(|c| c.distance.value());
    let representative_band = contributions.first().map(|c| c.band);
    // Engine-authoritative defence modifier (§6.23, §6.54): derived from the
    // board and the counters at the target hex, not from a caller-supplied
    // value. This applies to howitzer scatter too — `target_hex` is the
    // *actual* impact.
    let terrain_mod = target_defence_modifier(state, attack, target_hex, target_units);
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
    // §6.61/§6.62: gunboats and forts are special targets -- only artillery (or
    // howitzer-class) fire may engage them (validated), and they are destroyed
    // only on a Combat Results Table *cell value* meeting a threshold (gunboat
    // 3+, fort 2+), *not* by the generic disrupt/eliminate effect. "3 or more
    // on the combat results table" means Eliminate(3) or higher, not a die
    // roll of 3+.
    let opponent = attack.firing_player.opponent();
    let special = state.special_fire_target(target_units);
    // An advance window needs an *enemy*-occupied hex to have been vacated
    // (§6.82) -- a howitzer scatter onto an empty or friendly hex (§6.64)
    // never opens one.
    let was_occupied = target_units.iter().any(|id| {
        state
            .find_unit(*id)
            .is_some_and(|u| u.profile.identity.owner() == opponent)
    });
    commit_fired_markers(state, attack, target_units);

    if let Some((special_id, special_kind, needed)) = special {
        let destroyed = matches!(result, CombatResult::Eliminate(n) if n >= needed);
        // Snapshot the special target's occupants before mutation so the
        // FireResolved observation can report the eliminations accurately --
        // `apply_combat_results_table_result` and the retain() below both
        // mutate `state.units` in place.
        let pre_units: Vec<UnitId> = target_units.to_vec();
        if destroyed {
            // §6.62: if a destroyed fort contained enemy units, one is
            // eliminated with it (picked before the fort leaves the board).
            let fort_victim = matches!(special_kind, UnitKind::Fort { .. })
                .then(|| target_units.iter().copied().find(|&id| id != special_id))
                .flatten();
            // The shared elimination path scores the kill (§9.14) and takes
            // a sunk gunboat's loaded "Friendlies" down with it (§5.21).
            eliminate_unit(state, special_id, ElimCause::Combat);
            if let Some(victim) = fort_victim {
                eliminate_unit(state, victim, ElimCause::Combat);
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
        if was_occupied && !hex_still_defended && matches!(state.phase, Phase::OffensiveFire(_)) {
            let mut paragraphs = vec!["6.82".to_string()];
            paragraphs.push(match special_kind {
                UnitKind::Gunboat { .. } => "6.61".to_string(),
                _ => "6.62".to_string(),
            });
            open_advance_window(state, target_hex, &attack.all_firing_units(), paragraphs);
        }
        let eliminations: Vec<UnitId> = diff_eliminated(state, pre_units);
        state.turn_events.push(TurnEventRecord::FireCombat {
            attacker: attack.firing_player,
            firers: attack.all_firing_units(),
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
            impact,
            paragraphs: fire_paragraphs(attack.kind, Some(special_kind)),
        });
        return;
    }

    let pre_units: Vec<UnitId> = target_units.to_vec();
    apply_combat_results_table_result(state, result, target_units, disruption);
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
        impact,
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
        open_advance_window(
            state,
            target_hex,
            &attack.all_firing_units(),
            vec!["6.82".to_string()],
        );
    }
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
        // The Direct Fire Subphase itself: §6.24 is the Anglo-Egyptian
        // accuracy bonus, cited by its own modifier line when it applies.
        FireKind::Direct => "6.41",
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

/// The one enemy unit a wall breach eliminates (§6.63: "If any enemy units
/// are adjacent to the wall hexside at the instant it is breached, one enemy
/// unit is eliminated"), shared by artillery (§6.63) and Royal Engineers
/// (§6.53) breaches. Adjacent to a hexside means standing in one of the two
/// hexes that share it (as for crest hexsides, LOS condition 2) -- not
/// merely next to one of them. An Anglo-Egyptian leader is not a combat unit
/// and is never the casualty (it falls only under §6.51 / §9.346); Dervish
/// leaders fight and fall like any other unit (§6.51).
pub(crate) fn breach_victim(
    state: &GameState,
    a: HexCoord,
    b: HexCoord,
    victim_owner: Player,
) -> Option<UnitId> {
    state
        .units
        .iter()
        .find(|u| {
            (u.position == a || u.position == b)
                && u.profile.identity.owner() == victim_owner
                && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
        })
        .map(|u| u.id)
}

/// The Terrain Effects Chart's hexside fire effect on an attack (§6.23):
/// fire that enters the target hex across a Crest (-1) or City Wall (-4,
/// "but see LOS notes": only walls the LOS table lets fire cross) hexside.
/// The side crossed is the last step of each firer's line of fire; when a
/// combined attack's firers come in over different hexsides, the most
/// protective one applies to the single die roll. Howitzer shells (§6.64)
/// are lobbed from 4-10 hexes and ignore LOS, so they cross no hexside.
pub fn target_hexside_fire_modifier(
    state: &GameState,
    attack: &FireAttack,
    target_hex: HexCoord,
) -> i16 {
    if attack.kind == FireKind::Howitzer {
        return 0;
    }
    attack
        .firers
        .iter()
        .filter_map(|id| state.find_unit(*id))
        .filter(|u| u.position != target_hex)
        .map(|u| {
            let entry = omdurman_types::HexLine::new(u.position, target_hex, 1)
                .last()
                .unwrap_or(u.position);
            crate::terrain_chart::hexside_fire_modifier(state.hexside_effective(entry, target_hex))
        })
        .min()
        .unwrap_or(0)
}

/// One firer's share of a fire attack (§6.22): its distance to the aimed
/// hex, the range band on its own weapon line and table (§6.52 Friendlies
/// and §9.343 FoK on the Dervish table, §8.1 night cap), and its fire factor
/// in that band.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirerContribution {
    pub unit: UnitId,
    /// Which of the counter's weapons this is -- a named gunboat contributes
    /// once per weapon (artillery/howitzer, and Maxims, §2.32).
    pub mount: FireMount,
    /// The Range Effects Table line that weapon fires on here.
    pub weapon: WeaponClass,
    pub distance: HexDistance,
    pub band: crate::RangeBand,
    pub factor: u16,
}

/// Every firer's share of `attack` (§6.22), exactly as resolution sums them
/// -- the preview and the tray read the same numbers. Each firer is banded
/// at its *own* distance on its *own* weapon line and range-effects table,
/// with the §8.1 night cap applied per weapon; the shot is ranged at the
/// aimed hex (a §6.64 scatter moves where the result lands, not the band).
pub fn firer_contributions(state: &GameState, attack: &FireAttack) -> Vec<FirerContribution> {
    attack
        .shots()
        .into_iter()
        .filter_map(|shot| state.find_unit(shot.unit).map(|u| (u, shot.mount)))
        .map(|(u, mount)| {
            let weapon = u.weapon_line(mount, attack.kind);
            let fire = u.fire_factor(mount);
            let table_player = range_table_player_for(state.scenario, u);
            let distance = HexDistance(u.position.distance(attack.target_hex) as u16);
            // Beyond the night cap the band is OutOfRange (§8.1).
            let banded_at = if state.day_night == DayNight::Night {
                night_capped_distance(weapon, table_player, distance)
                    .unwrap_or(HexDistance(u16::MAX))
            } else {
                distance
            };
            let band = range_band_for(state.scenario, table_player, weapon, banded_at);
            FirerContribution {
                unit: u.id,
                mount,
                weapon,
                distance,
                band,
                factor: fire.map_or(0, |f| band.apply(f.value())),
            }
        })
        .collect()
}

/// The defensive die-roll modifier of fire at `target_hex` on `target_units`
/// beyond the mandatory list: the target hex's terrain (§6.23), the crest or
/// wall hexside the fire enters it across (Terrain Effects Chart), and the
/// fort's −3 when the units inside a fort are fired at (§6.54). Shared by
/// resolution and the app's fire preview.
pub fn target_defence_modifier(
    state: &GameState,
    attack: &FireAttack,
    target_hex: HexCoord,
    target_units: &[UnitId],
) -> i16 {
    let terrain = state
        .board
        .terrain_at(target_hex)
        .unwrap_or(omdurman_types::Terrain::Clear {
            road: Default::default(),
        });
    // §6.54: "The −3 defensive value is deducted from the die roll of enemy
    // fire attacks on friendly units stacked inside the fort" -- when the
    // fort is not itself the target.
    let fort_defends = !target_units.is_empty()
        && !target_units.iter().any(|id| {
            state
                .find_unit(*id)
                .is_some_and(|u| matches!(u.profile.kind, UnitKind::Fort { .. }))
        })
        && state.is_fort_hex(target_hex);
    crate::terrain_chart::defense_modifier(terrain)
        + target_hexside_fire_modifier(state, attack, target_hex)
        + if fort_defends { FORT_DEFENCE } else { 0 }
}

/// The printed "−3" on a fort counter (§6.54).
const FORT_DEFENCE: i16 = -3;

/// Build a combined `FireAttack` (§6.14): every friendly unit stacked in
/// `firer_hex` that may legally fire at `target` fires together, their fire
/// factors summed. Bakes in the die-roll modifiers the engine can't derive:
/// the Anglo-Egyptian +1 direct-fire bonus (§6.24), the +1 brigade-integrity
/// bonus when all four battalions fire (§5.54), and the target hex's terrain
/// modifier (§6.23).
///
/// This is the effect-construction half of the same contract
/// [`GameState::can_fire_at`] + [`mandatory_fire_modifiers`] enforce at
/// resolution, so every client (app UI, bot) offers exactly the attacks the
/// engine will accept. Returns `None` when no co-stacked unit may fire.
///
/// Combined-fire convenience: a caller that only knows *a* firer on the hex
/// gets the whole co-stacked attack ([`build_fire_attack_from`] with the
/// auto-computed firer list). §6.14 makes combining optional, not mandatory,
/// and §6.15 explicitly allows a stack to be *divided* so individual units
/// fire at different hexes -- the UI's single-unit fire path therefore uses
/// [`build_fire_attack_from`] with an explicit list instead.
pub fn build_fire_attack(
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
    let firers: Vec<UnitId> = gs
        .units
        .iter()
        .filter(|u| u.position == firer_hex)
        .filter(|u| u.profile.identity.owner() == owner)
        .filter(|u| u.profile.fire.is_some())
        .filter(|u| gs.can_fire_at(u.id, target, kind).is_ok())
        .map(|u| u.id)
        .collect();
    if firers.is_empty() {
        return None;
    }
    build_fire_attack_from(gs, firer_hex, &firers, target, kind)
}

/// Build a fire attack from an *explicit* firer list (rulebook §6.13, §6.15).
///
/// §6.13 makes a unit's fire factor unitary -- when a player chooses a single
/// counter, it fires alone, with exactly its own factor; §6.15 lets a stacked
/// group be divided so different units fire at different enemy hexes, so the
/// caller supplies the exact subunits of the attack rather than the whole
/// hex's stack. Every requested firer must independently satisfy
/// [`GameState::can_fire_at`] (phase, owner, sub-phase kind, weapon class,
/// line of sight, range, disrupted/fired trackers) and share both `firer_hex`
/// and its owner; the factors are summed and the mandatory modifier set
/// derived exactly as in [`build_fire_attack`]. Returns `None` for an empty,
/// duplicated, or not-fully-legal list.
pub fn build_fire_attack_from(
    gs: &GameState,
    firer_hex: HexCoord,
    firers: &[UnitId],
    target: HexCoord,
    kind: FireKind,
) -> Option<FireAttack> {
    let mut firers: Vec<UnitId> = firers.to_vec();
    firers.sort_unstable();
    firers.dedup();
    if firers.is_empty() {
        return None;
    }
    let owner = gs.find_unit(firers[0])?.profile.identity.owner();
    for &id in &firers {
        let unit = gs.find_unit(id)?;
        if unit.position != firer_hex
            || unit.profile.identity.owner() != owner
            || unit.profile.fire.is_none()
            || gs.can_fire_at(id, target, kind).is_err()
        {
            return None;
        }
    }

    // §6.24/§5.54/§9.231/§9.232: the engine derives the mandatory modifier
    // set (and rejects any other list), so build the attack with the engine's
    // own helper -- single source of truth with resolution. The terrain
    // defence modifier (§6.23) is likewise computed engine-side in
    // `resolve_fire_attack` from `state.board`.
    // §6.54/§6.62: an empty enemy fort can only be fired at itself.
    let enemy = gs.player_units_in_hex(target, owner.opponent());
    let empty_fort = !enemy.is_empty()
        && enemy
            .iter()
            .all(|u| matches!(u.profile.kind, UnitKind::Fort { .. }));
    let mut attack = FireAttack {
        firing_player: owner,
        phase: gs.phase,
        kind,
        firers,
        target_hex: target,
        at_fort: empty_fort,
        factor_row: FireFactorRow::Row01to05,
        modifiers: Vec::new(),
        gunboat_maxims: Vec::new(),
    };
    attack.factor_row = printed_factor_row(gs, &attack);
    attack.modifiers = mandatory_fire_modifiers(gs, &attack);
    Some(attack)
}

/// The Combat Results Table row of the printed factors an attack sums before
/// range effects (§6.14): the printed factor of every weapon firing.
fn printed_factor_row(gs: &GameState, attack: &FireAttack) -> FireFactorRow {
    let factors: Vec<FireFactor> = attack
        .shots()
        .into_iter()
        .filter_map(|shot| gs.find_unit(shot.unit)?.fire_factor(shot.mount))
        .collect();
    FireFactor::sum_to_row(factors.iter())
}

/// Add named gunboats' Maxim guns to `attack` (§2.32, §6.14), or build a
/// Maxims-only attack from an empty one: each gunboat must satisfy
/// [`GameState::can_fire_gunboat_maxims_at`] for the attack's target and
/// kind, and belong to its firing player; the printed factor row and the
/// mandatory modifiers are re-derived. `None` if a gunboat may not, or its
/// Maxims are already in the attack.
pub fn with_gunboat_maxims(
    gs: &GameState,
    attack: &FireAttack,
    gunboats: &[UnitId],
) -> Option<FireAttack> {
    let mut maxims = attack.gunboat_maxims.clone();
    for &id in gunboats {
        let unit = gs.find_unit(id)?;
        if maxims.contains(&id)
            || unit.profile.identity.owner() != attack.firing_player
            || gs
                .can_fire_gunboat_maxims_at(id, attack.target_hex, attack.kind)
                .is_err()
        {
            return None;
        }
        maxims.push(id);
    }
    maxims.sort_unstable();
    let mut attack = FireAttack {
        gunboat_maxims: maxims,
        modifiers: Vec::new(),
        ..attack.clone()
    };
    attack.factor_row = printed_factor_row(gs, &attack);
    attack.modifiers = mandatory_fire_modifiers(gs, &attack);
    Some(attack)
}

/// A named gunboat's Maxim guns firing alone at `target` (§2.32, §6.42):
/// direct fire in the Direct Fire subphase, Maxim second fire in the second.
/// `None` when they may not.
pub fn build_gunboat_maxim_attack(
    gs: &GameState,
    gunboat: UnitId,
    target: HexCoord,
    kind: FireKind,
) -> Option<FireAttack> {
    let owner = gs.find_unit(gunboat)?.profile.identity.owner();
    let empty = FireAttack {
        firing_player: owner,
        phase: gs.phase,
        kind,
        firers: Vec::new(),
        target_hex: target,
        at_fort: false,
        factor_row: FireFactorRow::Row01to05,
        modifiers: Vec::new(),
        gunboat_maxims: Vec::new(),
    };
    with_gunboat_maxims(gs, &empty, &[gunboat])
}

/// Merge two pending attacks on the same target into one combined attack
/// (rulebook §6.14): units may combine their fire into one attack from any
/// hexes, and a hex may only be fired at once per phase -- so a second group
/// joining an already-allocated target must join that attack, not open a
/// second one the engine would refuse. `None` when the attacks differ in
/// target, kind or firing player, or share a firer (§6.13).
pub fn combine_fire_attacks(
    gs: &GameState,
    existing: &FireAttack,
    joining: &FireAttack,
) -> Option<FireAttack> {
    if existing.target_hex != joining.target_hex
        || existing.kind != joining.kind
        || existing.at_fort != joining.at_fort
        || existing.firing_player != joining.firing_player
        || existing.firers.iter().any(|f| joining.firers.contains(f))
        || existing
            .gunboat_maxims
            .iter()
            .any(|g| joining.gunboat_maxims.contains(g))
    {
        return None;
    }
    let mut firers = [existing.firers.as_slice(), joining.firers.as_slice()].concat();
    firers.sort_unstable();
    let mut gunboat_maxims = [
        existing.gunboat_maxims.as_slice(),
        joining.gunboat_maxims.as_slice(),
    ]
    .concat();
    gunboat_maxims.sort_unstable();
    let mut attack = FireAttack {
        firers,
        gunboat_maxims,
        modifiers: Vec::new(),
        ..existing.clone()
    };
    attack.factor_row = printed_factor_row(gs, &attack);
    attack.modifiers = mandatory_fire_modifiers(gs, &attack);
    Some(attack)
}

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
    // fire attacks -- aimed fire at a hex the firer can see, which a Maxim's
    // second fire (§6.42) is as much as its first. Howitzer fire is indirect
    // (it ignores line of sight and scatters, §6.64) and gets no +1.
    let direct = matches!(attack.kind, FireKind::Direct | FireKind::MaximSecondFire);
    if direct && attack.firing_player == Player::AngloEgyptian {
        modifiers.push(FireModifier::AngloEgyptianDirectFire);
    }
    if attack.kind == FireKind::Direct && attack.firing_player == Player::AngloEgyptian {
        // §5.54/§6.24: brigade integrity (+1, cumulative) when all four
        // battalions of a brigade are stacked in the same hex and all fire
        // at this target hex. Other units may join the attack (§6.14) --
        // another brigade's stack, a Maxim -- without costing the stack its
        // bonus; it applies once per attack.
        let firers: Vec<&UnitPlacement> = attack
            .firers
            .iter()
            .filter_map(|id| state.find_unit(*id))
            .collect();
        let integrated_stack = firers.iter().any(|u| {
            let stack: Vec<crate::UnitIdentity> = firers
                .iter()
                .filter(|o| o.position == u.position)
                .map(|o| o.profile.identity)
                .collect();
            matches!(
                crate::brigade_integrity(&stack),
                crate::BrigadeIntegrity::Integrated(_)
            )
        });
        if integrated_stack {
            modifiers.push(FireModifier::BrigadeIntegrity);
        }
    }
    // §9.231/§9.232: the zariba die-roll penalties apply "on all *Dervish*
    // fire attacks" (thorn hedge −2; trench −4 vs. entrenched units) -- never
    // to Anglo-Egyptian fire.
    // The hedge hampers fire that crosses it (the line of fire from any
    // firer passes a thorn-hedge hexside); the trench protects the units
    // entrenched behind it, whichever way they are fired at.
    if attack.firing_player == Player::Dervish {
        if attack.all_firing_units().iter().any(|id| {
            state
                .find_unit(*id)
                .is_some_and(|u| fire_crosses_thorn_hedge(state, u.position, attack.target_hex))
        }) {
            modifiers.push(FireModifier::ZaribaThornHedge);
        }
        if state.is_zariba_entrenched(attack.target_hex) {
            modifiers.push(FireModifier::ZaribaTrenchEntrenched);
        }
    }
    modifiers
}

/// Whether the line of fire from `from` to `to` crosses a thorn-hedge
/// hexside (§9.231), read through the effective hexsides (in the Campaign,
/// only a constructed Zariba, §5.3). Walks the same hex line as the LOS
/// check (§6.3).
fn fire_crosses_thorn_hedge(state: &GameState, from: HexCoord, to: HexCoord) -> bool {
    let mut prev = from;
    for hex in omdurman_types::HexLine::new(from, to, 1).chain(std::iter::once(to)) {
        if hex != prev
            && state.hexside_effective(prev, hex)
                == Some(omdurman_types::HexsideKind::ZaribaThornHedge)
        {
            return true;
        }
        prev = hex;
    }
    false
}

/// Validate that a fire attack is legal in the current state (rulebook §6).
///
/// Single source of truth: every firer is checked through
/// [`GameState::can_fire_at`], the
/// same predicate the UI gates clicks on -- so a shot the UI offers is exactly a
/// shot `apply` accepts (phase, owner, sub-phase/kind, weapon class, howitzer-
/// at-night §6.64, disruption, already-fired, gunboat/fort-needs-artillery
/// §6.61/§6.62, and range §6.22). An empty firer list is rejected, as is a
/// firer listed twice (§6.13: a unit's factor is unitary -- listing it twice
/// would double it) and an attack whose `firing_player` is not the player
/// whose fire phase it is (§4, §6.41).
pub fn validate_fire_attack(state: &GameState, attack: &FireAttack) -> Result<(), RuleError> {
    if attack.firers.is_empty() && attack.gunboat_maxims.is_empty() {
        return Err(RuleError::NoFirers);
    }
    reject_duplicate_units(&attack.firers)?;
    reject_duplicate_units(&attack.gunboat_maxims)?;
    for &id in &attack.firers {
        state.can_fire_at(id, attack.target_hex, attack.kind)?;
    }
    // A named gunboat's Maxims (§2.32): a weapon of their own, checked on
    // the Maxims line and their own once-per-subphase tracker.
    for &id in &attack.gunboat_maxims {
        state.can_fire_gunboat_maxims_at(id, attack.target_hex, attack.kind)?;
    }
    // Every firer belongs to the phase player (checked above), so the
    // attack's `firing_player` -- which decides the targets, the table and
    // the modifiers -- must name that player too.
    let phase_player = match state.phase {
        Phase::OffensiveFire(_) => state.active_player,
        Phase::DefensiveFire(_) => state.active_player.opponent(),
        _ => return Err(RuleError::WrongPhase),
    };
    if attack.firing_player != phase_player {
        return Err(RuleError::FiringPlayerMismatch);
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

/// Apply a Combat Results Table result to a list of target units -- eliminate `n`, or disrupt
/// half (round up) of them, picked by `disruption` (rulebook §6.22, §7.7,
/// §CombatResults). Every elimination goes through [`eliminate_unit`] (VP,
/// gunboat cascade, GORDON).
pub(crate) fn apply_combat_results_table_result(
    state: &mut GameState,
    result: CombatResult,
    target_ids: &[UnitId],
    disruption: DisruptionDraw,
) {
    // §6.51/§9.346: an Anglo-Egyptian leader is never a combat casualty in
    // its own right -- it falls only when a Dervish unit enters its hex or
    // every combat unit it stacks with is eliminated (handled below). So
    // leaders never absorb a result: GORDON must not be the "1" of an
    // Eliminate(1) on the palace.
    let is_ae_leader = |id: &UnitId| {
        state
            .find_unit(*id)
            .is_some_and(|u| matches!(u.profile.kind, UnitKind::BritishLeader { .. }))
    };
    let target_ids: Vec<UnitId> = target_ids
        .iter()
        .copied()
        .filter(|id| !is_ae_leader(id))
        .collect();
    let target_ids = target_ids.as_slice();
    match result {
        CombatResult::NoEffect => {}
        CombatResult::Disrupt => {
            // §CombatResults: "D* = ½ (round up) of the units in the target
            // hex are disrupted (inverted)". The rulebook does not say who
            // picks them, so the pre-rolled draw picks at random -- among the
            // undisrupted units: re-disrupting a unit already face down would
            // spend the result on nothing.
            let n = target_ids.len().div_ceil(2);
            let fresh: Vec<UnitId> = target_ids
                .iter()
                .copied()
                .filter(|id| state.find_unit(*id).is_some_and(|u| !u.state.disrupted))
                .collect();
            for id in disruption.pick(&fresh, n) {
                if let Some(unit) = state.find_unit_mut(id) {
                    unit.state.disrupted = true;
                }
            }
        }
        CombatResult::Eliminate(n) => {
            // Printed CRT key: "# = That many units in the target hex are
            // eliminated, i.e. removed from play." A number eliminates and
            // nothing more -- only a `D` result disrupts (§6.22).
            let n = (n as usize).min(target_ids.len());

            // The hexes whose units are eliminated, captured *before* the
            // eliminations remove them: the §6.51(b) orphan-leader logic
            // below has to locate surviving leaders in those hexes.
            let eliminated_hexes: Vec<HexCoord> = target_ids[..n]
                .iter()
                .filter_map(|id| state.find_unit(*id).map(|u| u.position))
                .collect();
            // Score, remove and cascade (§9.14; §5.21: a sunk gunboat's
            // loaded "Friendlies" are lost with it).
            for &id in &target_ids[..n] {
                eliminate_unit(state, id, ElimCause::Combat);
            }

            // §6.51(b): if all combat units (non-leader) in a hex were
            // eliminated, any surviving AE leader in that hex is also
            // eliminated (the leader cannot exist alone on the battlefield).
            for hex in eliminated_hexes {
                let has_combat_unit = state.units.iter().any(|u| {
                    u.position == hex
                        && u.profile.identity.owner() == Player::AngloEgyptian
                        && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
                });
                if has_combat_unit {
                    continue;
                }
                // §9.346: GORDON is the exception -- "He may only be
                // eliminated by a Dervish unit passing through or occupying
                // the palace hex", so he outlives his garrison.
                let leader_ids: Vec<UnitId> = state
                    .units
                    .iter()
                    .filter(|u| {
                        u.position == hex
                            && matches!(u.profile.kind, UnitKind::BritishLeader { .. })
                            && !u.profile.identity.is_gordon()
                    })
                    .map(|u| u.id)
                    .collect();
                for id in leader_ids {
                    eliminate_unit(state, id, ElimCause::OrphanLeader);
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

    // The target hexside must currently be a standing Wall. (If it's already
    // a Breach — authored or §6.63-breached — or a Gate there's nothing to
    // do; if it's missing entirely the data is wrong. Either way the player
    // has misclicked.)
    if !state.hexside_effective_is(target.a, target.b, |k| k == HexsideKind::Wall) {
        return Err(RuleError::NotAWallHexside(target));
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
        // LOS already verified by `can_fire_at_wall`; the band lookup is the
        // only additional per-firer work.
        let (fire_factor, range, _) = state.can_fire_at_wall(id, target)?;
        let unit = state.unit_or_err(id)?;
        let band = range_band_for(
            state.scenario,
            range_table_player_for(state.scenario, unit),
            unit.profile.weapon,
            range,
        );
        effective_total = effective_total.saturating_add(band.apply(fire_factor.value()));
    }

    // All firers pass -- mark them as having fired this phase.
    for &id in firers {
        state.units_fired_this_phase.push(id);
    }

    // §6.63: "A result of 2 or more on the combat results table is required
    // to breach a wall." The CRT cell value (Eliminate(N)) is the relevant
    // metric, identical to the §6.61/§6.62 gunboat/fort thresholds.
    // §6.24: an Anglo-Egyptian battery's shot is a direct fire attack (+1).
    let accuracy = i16::from(firing_player == Player::AngloEgyptian);
    let row = FireFactorRow::from_total(effective_total);
    let result = combat_results_table(row, roll.apply_modifier(accuracy));
    let breached = matches!(result, CombatResult::Eliminate(n) if n >= 2);

    let mut adjacent_eliminated: Option<UnitId> = None;
    if breached {
        // Flip Wall → Breach. The breach is game state (`state.breaches`),
        // not a board mutation -- the board is static, so clone-and-try
        // probes share it freely.
        state.breach_wall(target.a, target.b);

        // §6.63: "If any enemy units are adjacent to the wall hexside at the
        // instant it is breached, one enemy unit is eliminated." Pick the
        // first such unit (matching the demolition path's convention).
        if let Some(victim) = breach_victim(state, target.a, target.b, firing_player.opponent()) {
            eliminate_unit(state, victim, ElimCause::WallBreach);
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

/// Kani proof harnesses over the range-table routing and the §8.1 night cap
/// (`cargo kani`, see `scripts/kani.sh`). These pin the functions that both
/// validation (`can_fire_at`) and resolution (`resolve_fire_attack`) call,
/// so the two can never disagree on which faction table a shot consults --
/// the audit class where a Friendlies shot passed validation on the
/// Anglo-Egyptian table (rifle max 5) but resolved on the Dervish table
/// (rifle max 4).
#[cfg(kani)]
mod verification {
    // `use super::*` reaches only this file's own items; everything else is
    // imported from where it is defined.
    use super::*;
    use crate::{
        HexCoord, HexDistance, UnitId, UnitMovement, UnitPlacement, UnitProfile, UnitState,
        WeaponClass,
    };
    use omdurman_types::{Player, Scenario, UnitKind};

    fn any_weapon() -> WeaponClass {
        let i: usize = kani::any();
        kani::assume(i < WeaponClass::ALL.len());
        WeaponClass::ALL[i]
    }

    fn any_player() -> Player {
        if kani::any() {
            Player::AngloEgyptian
        } else {
            Player::Dervish
        }
    }

    fn any_scenario() -> Scenario {
        match kani::any() {
            false => Scenario::Campaign,
            true => Scenario::FallOfKhartoum,
        }
    }

    /// The three table-routing shapes a firing unit can have: an
    /// Anglo-Egyptian unit, a Dervish unit, and a "Friendlies" infantry
    /// unit (an Anglo-Egyptian-nationality brigade that nevertheless fires
    /// on the Dervish table).
    fn any_routing_unit() -> UnitPlacement {
        use crate::{
            BattalionOrdinal, BrigadeNationality, UnitIdentity, UnitMovement, UnitProfile,
        };
        use omdurman_types::{BrigadeId, DervishTribe};
        let which: usize = kani::any();
        let identity = match which % 3 {
            0 => UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::British,
                },
                battalion: BattalionOrdinal::First,
            },
            1 => UnitIdentity::DervishTribal {
                tribe: DervishTribe::Baggara,
            },
            _ => UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::Friendlies,
                },
                battalion: BattalionOrdinal::First,
            },
        };
        UnitPlacement {
            id: crate::UnitId::ALL[0],
            position: HexCoord::new(0, 0),
            profile: UnitProfile {
                kind: UnitKind::Infantry {
                    fire: 4,
                    melee: 5,
                    movement: 8,
                },
                identity,
                weapon: WeaponClass::Rifles,
                fire: None,
                melee: None,
                movement: UnitMovement::Immobile,
            },
            state: crate::UnitState::default(),
        }
    }

    /// Range-table routing, exact over every scenario × player × weapon ×
    /// distance: in Fall of Khartoum *both* players consult the Dervish
    /// table; otherwise each player consults their own. The band returned
    /// is exactly the routed table's answer for the physical distance.
    // §6.22 §9.343
    #[traceability_macro::rulebook("§6.22", "§9.343")]
    #[kani::proof]
    fn range_band_for_routes_to_the_right_faction_table() {
        use crate::range_effects::{ae_range_effects, dervish_range_effects};
        let scenario = any_scenario();
        let player = any_player();
        let weapon = any_weapon();
        let d: u16 = kani::any();
        let dist = HexDistance::new(d);
        let band = range_band_for(scenario, player, weapon, dist);
        if scenario == Scenario::FallOfKhartoum || player == Player::Dervish {
            assert!(band == dervish_range_effects(weapon, dist));
        } else {
            assert!(band == ae_range_effects(weapon, dist));
        }
    }

    /// `range_table_player_for` (the per-firer selector used identically by
    /// validation and resolution) routes exactly per the printed table
    /// ownership: Friendlies always fire on the Dervish table, everyone
    /// else fires on their owner's -- except in Fall of Khartoum, where
    /// every unit in the game fires on the Dervish table.
    // §6.52 §9.343
    #[traceability_macro::rulebook("§6.52", "§9.343")]
    #[kani::proof]
    fn range_table_player_for_routes_friendlies_and_fok_to_the_dervish_table() {
        let scenario = any_scenario();
        let unit = any_routing_unit();
        let routed = range_table_player_for(scenario, &unit);
        let friendlies = unit.profile.identity.is_friendlies();
        if scenario == Scenario::FallOfKhartoum || friendlies {
            assert!(routed == Player::Dervish);
        } else {
            assert!(routed == unit.profile.identity.owner());
            // And the band the unit would fire on is that player's table.
            let weapon = any_weapon();
            let d: u16 = kani::any();
            let dist = HexDistance::new(d);
            let expected = if routed == Player::Dervish {
                crate::range_effects::dervish_range_effects(weapon, dist)
            } else {
                crate::range_effects::ae_range_effects(weapon, dist)
            };
            assert!(range_band_for(scenario, routed, weapon, dist) == expected);
        }
    }

    /// The §8.1 night cap: the distance a night shot consults is the
    /// physical distance exactly when it is within the halved maximum
    /// range (round down, floored at one), and `None` -- target out of
    /// range at night -- beyond it. Validation and resolution share this
    /// function, so neither can admit a shot the other refuses. (Distance
    /// 0 passes the cap but the day table itself rules it out of range;
    /// the cap is only an upper gate.)
    // §8.1
    #[traceability_macro::rulebook("§8.1")]
    #[kani::proof]
    fn night_capped_distance_is_some_exactly_within_the_night_max() {
        use crate::range_effects::night_max_range;
        let weapon = any_weapon();
        let table_player = any_player();
        let d: u16 = kani::any();
        let dist = HexDistance::new(d);
        let capped = night_capped_distance(weapon, table_player, dist);
        let max = night_max_range(weapon, table_player == Player::AngloEgyptian) as u16;
        assert!(capped.is_some() == (d <= max));
        if let Some(c) = capped {
            // The cap never rewrites the distance: the day table is
            // consulted at the physical distance.
            assert!(c == dist);
        }
    }
}

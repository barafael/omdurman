use super::*;

/// Apply a simultaneous melee combat between two adjacent hexes (rulebook §7).
pub fn apply_melee_combat(
    state: &mut GameState,
    attack: &MeleeAttack,
    attacker_roll: DieRoll,
    defender_roll: DieRoll,
) -> Result<(), RuleError> {
    if !matches!(state.phase, Phase::Melee) {
        return Err(RuleError::WrongPhase);
    }

    let attacker_player = attack.attacker_player;
    let defender_player = attacker_player.opponent();

    // Compute total melee factors.
    let attacker_total = crate::MeleeFactor::sum(
        attack
            .attackers
            .iter()
            .filter_map(|id| state.find_unit(*id))
            .filter_map(|u| u.profile.melee.as_ref()),
    );

    let defender_total = crate::MeleeFactor::sum(
        attack
            .defenders
            .iter()
            .filter_map(|id| state.find_unit(*id))
            .filter_map(|u| u.profile.melee.as_ref()),
    );

    // §7.7/§9.232: the engine derives both sides' mandatory melee modifiers
    // itself -- the declared lists are checked for equality in
    // `apply_declare_melee` but never trusted for the arithmetic. Re-deriving
    // here also keeps resolution correct if the state changed between
    // declaration and the §7.5 retreat-window resolution.
    let (derived_att, derived_def) = mandatory_melee_modifiers(state, attack);
    let att_mod: i16 = derived_att.iter().map(|m| m.die_modifier()).sum();
    let def_mod: i16 = derived_def.iter().map(|m| m.die_modifier()).sum();

    let att_net = attacker_roll.apply_modifier(att_mod);
    let def_net = defender_roll.apply_modifier(def_mod);

    // Melee uses the appropriate Combat Results Table with melee factors treated as fire factors.
    let att_row = FireFactorRow::from_total(attacker_total);
    let def_row = FireFactorRow::from_total(defender_total);

    let att_result = combat_results_table(att_row, att_net);
    let def_result = combat_results_table(def_row, def_net);

    let att_units: Vec<UnitId> = attack.attackers.clone();
    let def_units: Vec<UnitId> = attack.defenders.clone();

    // Player-readable melee report: both sides roll simultaneously (§7.7); each
    // side's result is applied to the *other*.

    // Snapshot which units existed on each side before CRT application, so the
    // MeleeResolved observation can report per-side losses after the fact
    // (apply_combat_results_table_result mutates state.units in place).
    let pre_attackers: Vec<UnitId> = att_units.clone();
    let pre_defenders: Vec<UnitId> = def_units.clone();

    // Simultaneous application.
    apply_combat_results_table_result(state, att_result, &def_units, defender_player);
    apply_combat_results_table_result(state, def_result, &att_units, attacker_player);

    // §7.6: if the melee eliminated *all* defenders, the Dervish MUST advance
    // into the vacated hex (up to the stacking limit of 4 units of the same
    // tribe); surviving eligible attackers move in automatically.  Units that
    // would violate stacking (wrong tribe, over limit) are silently skipped.
    // (The Anglo-Egyptian advance is optional and handled interactively via
    // `AdvanceAfterCombat`.)
    let defenders_remain = state
        .units
        .iter()
        .any(|u| u.position == attack.defender_hex);
    let mut mandatory_advance: Option<u8> = None;
    if attacker_player == Player::Dervish && !defenders_remain {
        // §5.51: only *counted* units (non-leaders) consume the four-per-hex
        // stacking budget; leaders are free stacking, so a leader among the
        // attackers advances even once the budget is spent.
        let mut counted_moved = 0;
        let mut moved = 0;
        for &id in &att_units {
            // Only surviving, non-disrupted attackers that may melee (i.e.
            // were eligible participants) advance.
            let Some(mover) = state.find_unit(id).copied() else {
                continue;
            };
            if mover.state.disrupted || !mover.profile.kind.may_melee_attack() {
                continue;
            }
            let counts_toward_limit = !matches!(
                mover.profile.kind,
                UnitKind::DervishLeader { .. } | UnitKind::BritishLeader { .. }
            );
            // The stacking budget gates counted units only (§5.51): a leader
            // later in the participant list must still be considered.
            if counts_toward_limit && counted_moved >= STACKING_LIMIT {
                continue;
            }
            // Respect the full stacking rules (§5.51-5.53), not just the count:
            // skip a unit whose advance would mix tribes or break leader command
            // in the vacated hex (it simply does not advance).
            if state.check_stacking(&mover, attack.defender_hex).is_err() {
                continue;
            }
            if let Some(u) = state.find_unit_mut(id) {
                u.position = attack.defender_hex;
            }
            moved += 1;
            if counts_toward_limit {
                counted_moved += 1;
            }
        }
        if moved > 0 {
            mandatory_advance = Some(moved as u8);
        }
    }
    // §7.6: if the melee vacated the defender hex, the surviving participants
    // may advance into it -- Dervish advances are forced (handled above), the
    // Anglo-Egyptian advance is optional via `AdvanceAfterCombat`.
    if !defenders_remain {
        open_advance_window(
            state,
            attack.defender_hex,
            &att_units,
            vec!["7.6".to_string()],
        );
    }

    let attacker_losses: Vec<UnitId> = diff_eliminated(state, pre_attackers);
    let defender_losses: Vec<UnitId> = diff_eliminated(state, pre_defenders);
    state.turn_events.push(TurnEventRecord::MeleeCombat {
        attacker: attacker_player,
        defender: defender_player,
        hex: attack.defender_hex,
        attacker_roll,
        defender_roll,
        attacker_result: att_result,
        defender_result: def_result,
        attacker_losses: attacker_losses.clone(),
        defender_losses: defender_losses.clone(),
        mandatory_advance,
    });
    state.observations.push(Observation::MeleeResolved {
        // Deliberate clone: observations are self-contained records for
        // replay/UI and must own their attack, not borrow it.
        attack: attack.clone(),
        attacker_roll,
        attacker_total_modifier: att_mod,
        attacker_modified_roll: att_net,
        attacker_result: att_result,
        defender_roll,
        defender_total_modifier: def_mod,
        defender_modified_roll: def_net,
        defender_result: def_result,
        attacker_factor: attacker_total,
        defender_factor: defender_total,
        attacker_losses,
        defender_losses,
        mandatory_advance,
        paragraphs: melee_paragraphs(attack),
    });

    Ok(())
}

/// Rulebook paragraphs that authorise a melee resolution (§7). The CRT is
/// reused for melee (§7.7); the standard side modifiers and the
/// trench-vs-Dervish modifier (§9.232) are the citable rules a player needs
/// to understand the result.
fn melee_paragraphs(attack: &MeleeAttack) -> Vec<String> {
    // 7.7: melee die modifiers + CRT reuse; 7.3: melee resolution / simultaneity.
    let trenched = attack
        .attacker_modifiers
        .iter()
        .any(|m| matches!(m, MeleeModifier::DervishVsTrenchedDefender));
    ["7.7", "7.3"]
        .into_iter()
        .map(String::from)
        .chain(trenched.then_some(String::from("9.232")))
        .collect()
}

/// Declare a melee (§7.5): validate it and store it as the pending attack,
/// opening the defender's reaction window. Resolution waits for
/// [`GameEffect::ResolveMelee`]; in between, eligible defenders may [`GameEffect::RetreatBeforeMelee`].
pub fn apply_declare_melee(
    state: &mut GameState,
    attack: &MeleeAttack,
    attacker_roll: DieRoll,
    defender_roll: DieRoll,
) -> Result<(), RuleError> {
    if state.active_player != attack.attacker_player {
        return Err(RuleError::NotYourTurn);
    }
    if state.pending_melee.is_some() {
        return Err(RuleError::MeleeAlreadyPending);
    }
    if attack.attackers.is_empty() {
        return Err(RuleError::MeleeHasNoAttackers);
    }
    // Single source of truth: every listed attacker must itself be able to
    // melee the target hex (phase, owner, not disrupted, melee-capable kind,
    // adjacent, with a meleeable enemy present) -- the same `can_melee`
    // predicate the UI gates on. This catches a disrupted or non-adjacent
    // attacker that the old ad-hoc check let through.
    for &id in &attack.attackers {
        state.can_melee(id, attack.defender_hex)?;
    }
    // §7.7/§9.232: the declared modifier lists must match the engine-derived
    // mandatory set exactly (the engine resolves with its own derivation).
    let (expected_att, expected_def) = mandatory_melee_modifiers(state, attack);
    if attack.attacker_modifiers != expected_att || attack.defender_modifiers != expected_def {
        return Err(RuleError::MeleeModifierMismatch {
            expected: [expected_att, expected_def].concat(),
            got: [
                attack.attacker_modifiers.clone(),
                attack.defender_modifiers.clone(),
            ]
            .concat(),
        });
    }
    state.pending_melee = Some(PendingMelee {
        attack: attack.clone(),
        attacker_roll,
        defender_roll,
    });
    Ok(())
}

/// Resolve the pending declared melee against whoever still occupies the
/// target hex (so a retreated defender is spared), then clear the window
/// (§7.5).
pub fn apply_resolve_melee(state: &mut GameState) -> Result<(), RuleError> {
    let Some(pending) = state.pending_melee.as_ref() else {
        return Err(RuleError::NoMeleePending);
    };
    // §7.5: the declaration (and its pre-rolled dice) must survive a rejected
    // resolution. `apply_melee_combat` rejects a wrong phase, so this used to
    // `take()` first and drop the melee on that path -- the same silent loss
    // `advance_phase` already guards ("audit: 76 declared melees vanished this
    // way"). Validate here, and only clear the window once the resolution is
    // committed.
    if !matches!(state.phase, Phase::Melee) {
        return Err(RuleError::WrongPhase);
    }
    let attacker_roll = pending.attacker_roll;
    let defender_roll = pending.defender_roll;
    let mut attack = pending.attack.clone();
    // Re-derive defenders from current occupants of the target hex: a unit
    // that retreated during the window is no longer there and is not hit.
    let defender_player = attack.attacker_player.opponent();
    attack.defenders = state
        .units
        .iter()
        .filter(|u| {
            u.position == attack.defender_hex
                && u.profile.identity.owner() == defender_player
                && u.profile.kind.may_be_melee_attacked()
        })
        .map(|u| u.id)
        .collect();
    // Likewise keep only attackers still adjacent and able to melee.
    attack.attackers.retain(|id| {
        state.find_unit(*id).is_some_and(|u| {
            !u.state.disrupted
                && u.profile.kind.may_melee_attack()
                && u.position.neighbors().contains(&attack.defender_hex)
        })
    });
    if attack.defenders.is_empty() {
        // Everyone retreated/eliminated already -- nothing to resolve, but the
        // Dervish may still advance into the vacated hex (§7.6) if attackers
        // remain. Treat as a melee with no defenders.
    }
    let outcome = apply_melee_combat(state, &attack, attacker_roll, defender_roll);
    if outcome.is_ok() {
        // Resolution committed: close the §7.5 reaction window.
        state.pending_melee = None;
    }
    outcome
}

/// The die-roll modifiers the rulebook *mandates* for both sides of a melee
/// (§7.7: Dervish +2 / Anglo-Egyptian +1; §9.232: −2 instead of +2 for a
/// Dervish melee attack on an entrenched unit). Returns
/// `(attacker_modifiers, defender_modifiers)`; the engine applies exactly
/// these at resolution and rejects a declared attack whose lists differ.
pub fn mandatory_melee_modifiers(
    state: &GameState,
    attack: &MeleeAttack,
) -> (Vec<MeleeModifier>, Vec<MeleeModifier>) {
    let mut attacker_modifiers = vec![match attack.attacker_player {
        Player::Dervish => MeleeModifier::DervishStandard,
        Player::AngloEgyptian => MeleeModifier::AngloEgyptianStandard,
    }];
    // §9.232: "−2 (instead of +2) melee modifier to Dervish units melee
    // attacking an entrenched unit".
    if attack.attacker_player == Player::Dervish
        && state.board.is_zariba_entrenched(attack.defender_hex)
    {
        attacker_modifiers.push(MeleeModifier::DervishVsTrenchedDefender);
    }
    let defender_modifiers = vec![match attack.attacker_player.opponent() {
        Player::Dervish => MeleeModifier::DervishStandard,
        Player::AngloEgyptian => MeleeModifier::AngloEgyptianStandard,
    }];
    (attacker_modifiers, defender_modifiers)
}

/// Apply a retreat-before-melee for a cavalry/camel unit (rulebook §7.5).
pub fn apply_retreat_before_melee(
    state: &mut GameState,
    unit_id: UnitId,
    to: HexCoord,
) -> Result<(), RuleError> {
    state.can_retreat_before_melee(unit_id, to)?;
    let from = state.find_unit(unit_id).map(|u| u.position).unwrap_or(to);
    // Mark the unit as having moved so it may not retreat again this turn
    // (§7.5). The can_ check above guarantees `mp_spent == 0` here, so the
    // sentinel `1` does not clobber a real MP total; it is cleared at
    // end-of-turn along with the rest of the map.
    state.mp_spent_this_turn.insert(unit_id, 1);
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.position = to;
    }
    state.turn_events.push(TurnEventRecord::Retreat {
        unit: unit_id,
        from,
        to,
    });
    // §7.5: a retreat that empties the pending melee's target hex vacates it
    // before resolution; the declared attackers may then advance into it
    // (§7.6). Only open the window once the *last* defender has left --
    // a stacked hex still held by any defender is not vacated.
    if let Some(pending) = &state.pending_melee
        && pending.attack.defender_hex == from
        && !state.units.iter().any(|u| u.position == from)
    {
        let attackers = pending.attack.attackers.clone();
        open_advance_window(state, from, &attackers, vec!["7.5".to_string()]);
    }
    Ok(())
}

/// Apply an advance-after-combat for a unit (rulebook §6.82, §7.6).
pub fn apply_advance_after_combat(
    state: &mut GameState,
    unit_id: UnitId,
    to: HexCoord,
) -> Result<(), RuleError> {
    state.can_advance_after_combat(unit_id, to)?;
    let from = state.find_unit(unit_id).map(|u| u.position).unwrap_or(to);
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.position = to;
    }
    state.turn_events.push(TurnEventRecord::AdvanceAfterCombat {
        unit: unit_id,
        from,
        to,
    });

    // §9.346: a Dervish unit reaching the Palace eliminates GORDON (FoK).
    check_gordon_palace(state);

    Ok(())
}

/// Open an advance-after-combat window (§6.82, §7.5, §7.6): record `hex` as
/// vacated by combat with the surviving `participants` as the only units
/// eligible to advance into it. Dead participants are filtered out; a window
/// already open for the hex (e.g. a second attack finishing off survivors)
/// has its eligible list unioned. Emits [`Observation::HexVacatedByCombat`] as
/// the audit record. A no-op when no participant survives.
pub(crate) fn open_advance_window(
    state: &mut GameState,
    hex: HexCoord,
    participants: &[UnitId],
    paragraphs: Vec<String>,
) {
    let survivors: Vec<UnitId> = participants
        .iter()
        .copied()
        // §6.82: "artillery may not advance"; §5.25: forts may never move.
        // Both can *cause* a vacated hex (artillery fire destroys the
        // defenders) but may never enter it, so they are never eligible.
        .filter(|&id| {
            state.find_unit(id).is_some_and(|u| {
                !matches!(
                    u.profile.kind,
                    UnitKind::Artillery { .. } | UnitKind::Fort { .. }
                )
            })
        })
        .collect();
    if survivors.is_empty() {
        return;
    }
    let entry = state.vacated_by_combat.entry(hex).or_default();
    for &id in &survivors {
        if !entry.contains(&id) {
            entry.push(id);
        }
    }
    state.observations.push(Observation::HexVacatedByCombat {
        hex,
        eligible: entry.clone(),
        paragraphs,
    });
}

// ---------------------------------------------------------------------------
// 9) Unit state changes
// ---------------------------------------------------------------------------

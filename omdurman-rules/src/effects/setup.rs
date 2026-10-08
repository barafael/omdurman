use super::*;

/// Set a set of units to constructing the Zariba (rulebook §5.3). They build
/// at the end of the Anglo-Egyptian player turn, if still in place
/// (`end_player_turn`); until then they may neither move, fire offensively
/// nor melee attack.
pub fn apply_construct_zariba(
    state: &mut GameState,
    unit_ids: &[UnitId],
    hexside: HexsideRef,
) -> Result<(), RuleError> {
    state.can_construct_zariba(unit_ids, hexside)?;
    for &id in unit_ids {
        if let Some(unit) = state.find_unit_mut(id) {
            unit.state.constructing_zariba = true;
        }
    }
    Ok(())
}

/// Apply a Royal Engineers demolition action (rulebook §6.53). The Engineers
/// commit to the demolition this turn (flagged `demolishing`); the actual
/// resolution happens at end of turn (`resolve_demolition`), which checks
/// the engineer is still adjacent and undisrupted.
pub fn apply_demolition(
    state: &mut GameState,
    unit_id: UnitId,
    target: DemolitionTarget,
) -> Result<(), RuleError> {
    state.can_demolition(unit_id)?;
    // §6.53: the target must be a standing enemy fort or wall hexside
    // adjacent to the engineers -- the same set the UI offers.
    if !state.demolition_targets(unit_id).contains(&target) {
        return Err(RuleError::InvalidDemolitionTarget);
    }
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.state.demolishing = true;
    }
    state.pending_demolitions.push((unit_id, target));
    Ok(())
}

/// Resolve a Royal Engineers demolition at the end of the Anglo-Egyptian
/// player turn (§6.53), from `end_player_turn`: the engineer must still be
/// adjacent to the target and undisrupted, or the attempt is cancelled. On
/// success a fort is eliminated with one of its occupants (§6.62, 0 VP for
/// the fort, §9.14), or a wall hexside becomes a breach and one adjacent
/// enemy unit is eliminated (§6.63). Either way the engineer is freed.
pub(crate) fn resolve_demolition(state: &mut GameState, unit_id: UnitId, target: DemolitionTarget) {
    // §6.53: the demolition succeeds only if the engineers "remain adjacent
    // to their target and undisrupted at the end of the Anglo-Egyptian player
    // turn" -- an engineer eliminated during the turn did not remain, so the
    // attempt is simply cancelled (an error here would stall the phase
    // advance forever, since the end-of-turn resolution is mandatory).
    let Some(engineer) = state.find_unit(unit_id) else {
        state.observations.push(Observation::DemolitionResolved {
            engineer_id: unit_id,
            target,
            success: false,
        });
        return;
    };
    let (engineer_pos, engineer_owner, engineer_disrupted) = (
        engineer.position,
        engineer.profile.identity.owner(),
        engineer.state.disrupted,
    );

    // If the engineer was disrupted during the turn, the demolition fails.
    if engineer_disrupted {
        if let Some(u) = state.find_unit_mut(unit_id) {
            u.state.demolishing = false;
        }
        state.observations.push(Observation::DemolitionResolved {
            engineer_id: unit_id,
            target,
            success: false,
        });
        return;
    }

    // Check adjacency to the target.
    let (success, adjacent_eliminated) = match target {
        DemolitionTarget::Fort(fort_id) => {
            let fort = state.find_unit(fort_id);
            let adjacent = fort
                .map(|f| engineer_pos.is_adjacent_to(f.position))
                .unwrap_or(false);
            if adjacent {
                // §6.53 "see 6.62 ... for the effects on adjacent enemy
                // units": "if the fort contains any enemy units at the
                // instant it is destroyed, one unit is eliminated with the
                // fort" (picked before the fort leaves the board).
                let victim = fort.and_then(|f| {
                    state
                        .units
                        .iter()
                        .find(|u| {
                            u.position == f.position
                                && u.id != fort_id
                                && u.profile.identity.owner() != engineer_owner
                                && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
                        })
                        .map(|u| u.id)
                });
                if let Some(f) = fort {
                    state.observations.push(Observation::FortDestroyed {
                        id: fort_id,
                        hex: f.position,
                    });
                }
                // Recorded (and scored -- 0 VP for a fort, §9.14) through the
                // shared elimination path.
                eliminate_unit(state, fort_id, ElimCause::Demolition);
                if let Some(victim) = victim {
                    eliminate_unit(state, victim, ElimCause::Demolition);
                }
                (true, victim)
            } else {
                (false, None)
            }
        }
        DemolitionTarget::WallHexside(edge) => {
            // "Adjacent to a wall hexside": in one of the two hexes sharing it.
            let adjacent = engineer_pos == edge.a || engineer_pos == edge.b;
            if adjacent {
                // Mutate the hexside: Wall → Breach (§6.63). The breach is
                // game state (`state.breaches`), not a board mutation -- the
                // board is static, so clone-and-try probes share it freely.
                state.breach_wall(edge.a, edge.b);
                // §6.63: if an enemy unit is adjacent to the wall hexside at
                // the instant of breaching, one enemy unit is eliminated.
                let enemy_adjacent =
                    super::fire::breach_victim(state, edge.a, edge.b, engineer_owner.opponent());
                if let Some(enemy_id) = enemy_adjacent {
                    eliminate_unit(state, enemy_id, ElimCause::WallBreach);
                }
                (true, enemy_adjacent)
            } else {
                (false, None)
            }
        }
    };

    // Free the engineer regardless of outcome.
    if let Some(u) = state.find_unit_mut(unit_id) {
        u.state.demolishing = false;
    }

    state.turn_events.push(TurnEventRecord::Demolition {
        engineer: unit_id,
        target,
        success,
    });
    state.observations.push(Observation::DemolitionResolved {
        engineer_id: unit_id,
        target,
        success,
    });
    if success && let DemolitionTarget::WallHexside(edge) = target {
        state.observations.push(Observation::WallBreached {
            hexside: edge,
            breached: true,
            // §6.53 demolitions have no CRT roll -- success is guaranteed by
            // surviving the turn adjacent and undisrupted.
            row: None,
            adjacent_eliminated,
        });
    }
}

/// Place reinforcements onto the map (rulebook §9.112, §9.113).
pub fn apply_place_reinforcements(
    state: &mut GameState,
    placements: &[UnitPlacement],
) -> Result<(), RuleError> {
    // Full stacking validation (§5.51-5.53), not just the four-unit count, and
    // cumulative across the batch -- plus the Campaign order of appearance
    // (§9.112/§9.113) via `validate_campaign_reinforcements`.
    state.can_place_reinforcements(placements)?;
    for p in placements {
        // §9.112/§9.113: entering the map costs movement points -- the
        // Anglo-Egyptian entrance costs 1 MP (8 for the "Friendlies" through
        // the Abu Alim hut); the Dervish pay the terrain cost of the hex
        // entered. Recorded as MP spent so the allowance cap (§5.11) and
        // retreat gating (§7.5) see it.
        if state.scenario == Scenario::Campaign && matches!(state.phase, Phase::Movement) {
            let owner = p.profile.identity.owner();
            let cost: i16 = match owner {
                Player::AngloEgyptian => {
                    if p.profile.identity.is_friendlies() {
                        8
                    } else {
                        1
                    }
                }
                Player::Dervish => {
                    let terrain = state.board.terrain_at(p.position).unwrap_or(
                        omdurman_types::Terrain::Clear {
                            road: Default::default(),
                        },
                    );
                    crate::terrain_chart::movement_cost(terrain)
                        .map(|allowance| allowance.value() as i16)
                        .unwrap_or(1)
                }
            };
            let spent = state.mp_spent(p.id);
            state
                .mp_spent_this_turn
                .insert(p.id, spent.saturating_add(cost));
        }
        let owner = p.profile.identity.owner();
        // §5.26: a unit entering the map into an enemy ZOC stops there, as
        // on entering one by movement.
        if matches!(state.phase, Phase::Movement)
            && state.hex_in_enemy_zoc(p.position, owner, p.profile.kind)
        {
            state.zoc_stopped_this_turn.push(p.id);
        }
        state.units.push(*p);
        state.reinforcements_placed_this_turn.push((owner, p.id));
        // §6.51(a): a Dervish unit entering a hex held by a lone
        // Anglo-Egyptian leader eliminates him.
        if owner == Player::Dervish {
            super::movement::overrun_lone_leaders(state, &[p.position]);
        }
    }
    if let Some(first) = placements.first() {
        state.turn_events.push(TurnEventRecord::Reinforcements {
            units: placements.iter().map(|p| p.id).collect(),
            player: first.profile.identity.owner(),
            at: first.position,
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 11) Scenario-specific
// ---------------------------------------------------------------------------

/// The number of Dervish units that desert for a given die roll (§8.2): "equal
/// to 1½ times the roll of one die", rounded up -- `ceil(1.5 * roll)`, so an
/// odd roll r deserts (3r + 1) / 2 (a roll of 1 deserts 2, a 3 deserts 5).
/// The manual does not say which way to round; up is the chosen reading.
pub fn desertion_count(roll: DieRoll) -> usize {
    (3 * roll.value() as usize).div_ceil(2)
}

pub fn apply_dervish_desertion(
    state: &mut GameState,
    roll: DieRoll,
    deserters: &[UnitId],
) -> Result<(), RuleError> {
    // §8.2: "Once each campaign game, during the first night turn of the game,
    // the Dervish player rolls one die... made during the movement phase."
    if state.dervish_deserted {
        return Err(DesertionError::AlreadyDeserted.into());
    }
    if state.scenario != Scenario::Campaign {
        return Err(DesertionError::WrongScenario.into());
    }
    if !state.desertion_due() {
        return Err(DesertionError::WrongTime.into());
    }

    // The count is fixed by the roll; the Dervish player chooses which units.
    let expected = state.desertion_demand(roll);
    if deserters.len() != expected {
        return Err(DesertionError::WrongCount {
            roll: roll.value() as u8,
            expected,
            actual: deserters.len(),
        }
        .into());
    }

    // Validate every chosen unit before removing any (all-or-nothing). A
    // unit listed twice would pad the count without deserting twice.
    reject_duplicate_units(deserters)?;
    for &id in deserters {
        let unit = state.unit_or_err(id)?;
        if unit.profile.identity.owner() != Player::Dervish {
            return Err(DesertionError::NotEligible(id).into());
        }
        if unit.profile.identity.is_desertion_exempt() {
            return Err(DesertionError::Exempt(id).into());
        }
    }

    for &id in deserters {
        state.units.retain(|u| u.id != id);
        // A deserter is gone for good: off the board without victory points
        // (§8.2), listed apart from the casualties, and never marching back
        // on as a reinforcement (§9.112).
        state.deserted.push(id);
    }
    state.turn_events.push(TurnEventRecord::Desertion {
        units: deserters.to_vec(),
        roll,
    });
    state.dervish_deserted = true;
    Ok(())
}

/// Deploy one order-of-battle unit during setup (§9.2/§9.3). Validated by
/// [`GameState::can_deploy_unit`]; on success the placement joins `units`.
pub fn apply_deploy_unit(
    state: &mut GameState,
    placement: &UnitPlacement,
) -> Result<(), RuleError> {
    state.can_deploy_unit(placement)?;
    state.units.push(*placement);
    Ok(())
}

/// Remove a deployed unit from the board during setup (§9.2/§9.3) so its
/// counter can be re-placed. Validated by [`GameState::can_remove_deployed_unit`];
/// on success the placement is dropped from `units`.
pub fn apply_remove_deployed_unit(
    state: &mut GameState,
    unit_id: UnitId,
    player: Player,
) -> Result<(), RuleError> {
    state.can_remove_deployed_unit(unit_id, player)?;
    state.units.retain(|u| u.id != unit_id);
    Ok(())
}

/// Lay a river mine during setup (§10.11). Validated by
/// [`GameState::can_place_mine`].
pub fn apply_place_mine(state: &mut GameState, hex: HexCoord) -> Result<(), RuleError> {
    state.can_place_mine(hex)?;
    state.mines.push(MinePlacement {
        hex,
        triggered: false,
    });
    Ok(())
}

/// Lay (or replace) the river chain during setup (§10.21). Validated by
/// [`GameState::can_place_chain`].
pub fn apply_place_chain(state: &mut GameState, hexes: &[HexCoord]) -> Result<(), RuleError> {
    state.can_place_chain(hexes)?;
    state.chain = Some(ChainPlacement {
        hexes: hexes.to_vec(),
        sunk: false,
    });
    Ok(())
}

/// A faction confirms readiness to leave setup (§9.2/§9.3). Sets the one-way
/// ready flag; when *both* factions are ready and `setup_complete` holds, the
/// engine auto-advances to the first Movement turn (via [`advance_phase`], so the
/// transition logic lives in one place). Validated by
/// [`GameState::can_confirm_setup_ready`].
pub fn apply_confirm_setup_ready(state: &mut GameState, player: Player) -> Result<(), RuleError> {
    state.can_confirm_setup_ready(player)?;
    match player {
        Player::AngloEgyptian => state.setup_ready_ae = true,
        Player::Dervish => state.setup_ready_dervish = true,
    }

    // Both sides ready + deployment complete -> begin the battle. `advance_phase`
    // owns the Setup -> Movement transition; a not-yet-complete board just leaves
    // us in Setup (the other side is still deploying).
    if state.setup_ready_ae && state.setup_ready_dervish && state.setup_complete().is_ok() {
        advance_phase(state)?;
    }
    Ok(())
}

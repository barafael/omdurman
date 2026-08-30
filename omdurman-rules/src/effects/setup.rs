use super::*;

/// Remove disrupted status from a unit (rulebook §5, reference notes).
pub fn apply_recover_unit(state: &mut GameState, unit_id: UnitId) -> Result<(), RuleError> {
    state.can_recover_unit(unit_id)?;
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.state.disrupted = false;
    }
    state
        .turn_events
        .push(TurnEventRecord::UnitRecovered { unit: unit_id });
    Ok(())
}

/// Mark a set of units as constructing a Zariba hexside (rulebook §5.3).
pub fn apply_construct_zariba(
    state: &mut GameState,
    unit_ids: &[UnitId],
    hexside: HexsideRef,
) -> Result<(), RuleError> {
    state.can_construct_zariba(unit_ids)?;
    for &id in unit_ids {
        if let Some(unit) = state.find_unit_mut(id) {
            unit.state.constructing_zariba = true;
        }
    }
    state.zariba_hexsides.push(hexside);
    Ok(())
}

/// Apply a Royal Engineers demolition action (rulebook §6.53). The Engineers
/// commit to the demolition this turn (flagged `demolishing`); the actual
/// resolution happens at end of turn via [`apply_resolve_demolition`], which
/// checks the engineer is still adjacent and undisrupted.
pub fn apply_demolition(
    state: &mut GameState,
    unit_id: UnitId,
    target: DemolitionTarget,
) -> Result<(), RuleError> {
    state.can_demolition(unit_id)?;
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.state.demolishing = true;
    }
    state.pending_demolitions.push((unit_id, target));
    Ok(())
}

/// Resolve a pending demolition at end of turn (§6.53). The engineer must still
/// be adjacent to the target and undisrupted; otherwise the demolition is
/// cancelled. On success:
///   - Fort target: the fort unit is eliminated (0 VP per §9.14).
///   - Wall target: the hexside becomes a Breach (§6.63); if an enemy unit is
///     adjacent at the instant of breaching, one is eliminated.
///
/// Either way the engineer is freed (`demolishing = false`).
pub fn apply_resolve_demolition(
    state: &mut GameState,
    unit_id: UnitId,
    target: DemolitionTarget,
) -> Result<(), RuleError> {
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
        return Ok(());
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
        return Ok(());
    }

    // Check adjacency to the target.
    let (success, adjacent_eliminated) = match target {
        DemolitionTarget::Fort(fort_id) => {
            let fort = state.find_unit(fort_id);
            let adjacent = fort
                .map(|f| engineer_pos.is_adjacent_to(f.position))
                .unwrap_or(false);
            if adjacent {
                if let Some(f) = fort {
                    state.observations.push(Observation::FortDestroyed {
                        id: fort_id,
                        hex: f.position,
                    });
                }
                state.units.retain(|u| u.id != fort_id);
                (true, None)
            } else {
                (false, None)
            }
        }
        DemolitionTarget::WallHexside(edge) => {
            let adjacent =
                engineer_pos.is_adjacent_to(edge.a) || engineer_pos.is_adjacent_to(edge.b);
            if adjacent {
                // Mutate the hexside: Wall → Breach (§6.63).
                if let Some(kind) = state.board.hexsides.get_mut(&edge) {
                    if *kind == HexsideKind::Wall {
                        *kind = HexsideKind::Breach;
                    }
                } else {
                    state.board.hexsides.insert(edge, HexsideKind::Breach);
                }
                // §6.63: if an enemy unit is adjacent to the wall hexside at
                // the instant of breaching, one enemy unit is eliminated.
                let enemy_adjacent = state.units.iter().find_map(|u| {
                    let is_enemy = u.profile.identity.owner() != engineer_owner;
                    let adjacent_to_wall =
                        u.position.is_adjacent_to(edge.a) || u.position.is_adjacent_to(edge.b);
                    (is_enemy && adjacent_to_wall).then_some(u.id)
                });
                if let Some(enemy_id) = enemy_adjacent {
                    score_elimination(state, enemy_id, ElimCause::Demolition);
                    state.units.retain(|u| u.id != enemy_id);
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

    Ok(())
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
            let spent = state.mp_spent_this_turn.get(&p.id).copied().unwrap_or(0);
            state.mp_spent_this_turn.insert(p.id, spent + cost);
        }
        state.units.push(*p);
        state
            .reinforcements_placed_this_turn
            .push((p.profile.identity.owner(), p.id));
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
/// to 1½ times the roll of one die", i.e. `floor(1.5 * roll)`.
/// The manual does not specify rounding direction; floor is used here.
pub fn desertion_count(roll: DieRoll) -> usize {
    (3 * roll.value() as usize) / 2
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
    let is_first_night = state.day_night == DayNight::Night
        && scenario_turn(state.scenario, state.current_turn)
            .is_some_and(|t| t.event == TurnEvent::DervishDesertion);
    if !is_first_night || state.phase != Phase::Movement {
        return Err(DesertionError::WrongTime.into());
    }

    // The count is fixed by the roll; the Dervish player chooses which units.
    // The demand is capped by the eligible pool: §8.2 assumes a full army, and
    // a Dervish force already bled below 1.5x the roll simply desert
    // everything eligible ("the number of deserting units is equal to 1.5
    // times the roll" cannot exceed the units that exist).
    let expected = desertion_count(roll).min(
        state
            .units
            .iter()
            .filter(|u| {
                u.profile.identity.owner() == Player::Dervish
                    && !u.profile.identity.is_desertion_exempt()
            })
            .count(),
    );
    if deserters.len() != expected {
        return Err(DesertionError::WrongCount {
            roll: roll.value() as u8,
            expected,
            actual: deserters.len(),
        }
        .into());
    }

    // Validate every chosen unit before removing any (all-or-nothing).
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

/// Pre-place a Zariba hexside during setup (§9.231-9.232). Validated by
/// [`GameState::can_place_zariba`].
pub fn apply_place_zariba(state: &mut GameState, hexside: HexsideRef) -> Result<(), RuleError> {
    state.can_place_zariba()?;
    if !state.zariba_hexsides.contains(&hexside) {
        state.zariba_hexsides.push(hexside);
    }
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

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

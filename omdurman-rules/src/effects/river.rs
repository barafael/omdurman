use super::*;

/// Apply a Friendlies-transport state transition (rulebook §5.21).
///
/// Gates enforced:
///   - `Load`: Isa Zachneih must be eliminated; the unit and gunboat must be
///     adjacent; no transport may already be in progress.
///   - `Cross`: the current state must be `Loaded`.
///   - `Disembark`: the current state must be `Crossing`; on success the
///     unit is freed (a disembarking `MoveUnit` should follow, costed by
///     terrain).
pub fn apply_friendlies_transport(
    state: &mut GameState,
    action: FriendliesAction,
) -> Result<(), RuleError> {
    match action {
        FriendliesAction::Load { unit, gunboat } => {
            // §5.21: transport is only allowed after Isa Zachneih is eliminated.
            if !state.isa_zachneih_eliminated {
                return Err(RuleError::FriendliesIsaZachneihAlive);
            }
            // No concurrent missions (design choice -- manual is ambiguous).
            if state.friendlies_transport.is_some() {
                return Err(RuleError::FriendliesTransportInProgress);
            }
            let u = state.unit_or_err(unit)?;
            let gb = state.unit_or_err(gunboat)?;
            // §5.21: the unit and gunboat must start the turn adjacent.
            if !u.position.is_adjacent_to(gb.position) {
                return Err(RuleError::FriendliesNotAdjacentToGunboat);
            }
            // Mark the unit as loaded onto the gunboat.
            if let Some(u) = state.find_unit_mut(unit) {
                u.state.loaded_on = Some(gunboat);
            }
            state.friendlies_transport = Some(TransportState::Loaded { unit, gunboat });
        }
        FriendliesAction::Cross { unit, gunboat, to } => {
            state.require_transport_state(
                |s| {
                    matches!(
                        s,
                        TransportState::Loaded { unit: cu, gunboat: cg }
                            if *cu == unit && *cg == gunboat
                    )
                },
                RuleError::FriendliesNotLoaded,
            )?;
            state.friendlies_transport = Some(TransportState::Crossing { unit, gunboat, to });
        }
        FriendliesAction::Disembark { unit, gunboat } => {
            state.require_transport_state(
                |s| {
                    matches!(
                        s,
                        TransportState::Crossing { unit: cu, gunboat: cg, .. }
                            if *cu == unit && *cg == gunboat
                    )
                },
                RuleError::FriendliesNotCrossing,
            )?;
            // Disembark: free the unit from the gunboat. A disembarking MoveUnit
            // effect should follow (chained by the caller) to pay the terrain
            // cost of the first hex entered (§5.21).
            if let Some(u) = state.find_unit_mut(unit) {
                u.state.loaded_on = None;
            }
            state.observations.push(Observation::FriendliesDisembarked {
                unit_id: unit,
                at: state
                    .find_unit(gunboat)
                    .map(|g| g.position)
                    .unwrap_or(HexCoord::new(0, 0)),
            });
            state.friendlies_transport = Some(TransportState::ReadyToDisembark { unit, gunboat });
        }
    }
    Ok(())
}

/// If a gunboat carrying a "Friendlies" unit is sunk (by artillery §6.61 or
/// mine §10.12), the loaded unit is lost with it (§5.21 — manual is silent;
/// design choice: loaded Friendlies go down with the ship).
pub(crate) fn remove_friendlies_on_gunboat(state: &mut GameState, gunboat_id: UnitId) {
    if let Some(TransportState::Loaded { unit, gunboat })
    | Some(TransportState::Crossing {
        unit,
        gunboat,
        to: _,
    }) = &state.friendlies_transport
        && *gunboat == gunboat_id
    {
        let unit_id = *unit;
        state.units.retain(|u| u.id != unit_id);
        state.friendlies_transport = None;
    }
}

/// Drift a gunboat with lost engines one hex downstream with the Nile current
/// (rulebook §10.12).  Called automatically at the start of each movement
/// phase for every gunboat with `engines_lost == true`.
///
/// The Nile flow arrows on the map are assumed to always point to a hex that
/// itself has a flow arrow (the user confirmed).  If no flow data exists at
/// the current hex (dead end), the gunboat is stuck and nothing happens.
pub fn apply_drift_gunboat(
    state: &mut GameState,
    unit_id: UnitId,
    mine_roll: DieRoll,
) -> Result<(), RuleError> {
    let unit = state.unit_or_err(unit_id)?;
    if !matches!(unit.profile.kind, UnitKind::Gunboat { .. }) {
        return Err(RuleError::NotAGunboat(unit_id));
    }
    if !unit.state.engines_lost {
        return Err(RuleError::GunboatEnginesNotLost(unit_id));
    }
    let current = unit.position;
    let Some(flow) = state.board.flow_at(current) else {
        // No flow data at this hex — the gunboat is stuck (dead end).
        return Ok(());
    };
    let downstream = current.neighbors()[flow as usize];
    // Move the gunboat downstream.  Gunboats ignore stacking (§5.51).
    if let Some(u) = state.find_unit_mut(unit_id) {
        u.position = downstream;
    }
    // If the gunboat drifts into an untriggered mine, resolve it with the
    // pre-rolled die exactly like a RiverMine effect would (§10.12; §10.14's
    // Dervish exemption is applied inside `apply_river_mine`).
    if state
        .mines
        .iter()
        .any(|m| m.hex == downstream && !m.triggered)
    {
        apply_river_mine(state, unit_id, downstream, mine_roll)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 12) Optional rules
// ---------------------------------------------------------------------------

/// Apply a river-mine resolution (rulebook §10.12).
pub fn apply_river_mine(
    state: &mut GameState,
    gunboat_id: UnitId,
    hex: HexCoord,
    roll: DieRoll,
) -> Result<(), RuleError> {
    // §10.14: the Dervish player's own gunboats pass through mined hexes with
    // no ill effect (he knows where they are).
    if let Some(unit) = state.find_unit(gunboat_id)
        && unit.profile.identity.owner() == Player::Dervish
    {
        return Ok(());
    }

    // §10.13: a mine only fires once. The hex must hold an untriggered mine.
    let Some(mine) = state
        .mines
        .iter_mut()
        .find(|m| m.hex == hex && !m.triggered)
    else {
        return Err(RuleError::NoUntriggeredMine(hex));
    };
    mine.triggered = true;

    let result = crate::MineResult::from_roll(roll);
    match result {
        crate::MineResult::NoEffect => {}
        crate::MineResult::EnginesLost => {
            if let Some(unit) = state.find_unit_mut(gunboat_id) {
                // §10.12: engines lost -- the gunboat drifts two hexes per turn
                // with the current for the rest of the game.
                unit.state.engines_lost = true;
            }
        }
        crate::MineResult::Sunk => {
            state.units.retain(|u| u.id != gunboat_id);
            remove_friendlies_on_gunboat(state, gunboat_id);
        }
    }
    Ok(())
}

/// Sink the river chain (rulebook §10.23). Marks the placed chain cleared so it
/// no longer stops gunboats (§10.22).
pub fn apply_sink_chain(state: &mut GameState) -> Result<(), RuleError> {
    match state.chain.as_mut() {
        Some(chain) if !chain.sunk => {
            chain.sunk = true;
            Ok(())
        }
        Some(_) => Err(RuleError::ChainAlreadySunk),
        None => Err(RuleError::NoChainPlaced),
    }
}

// ---------------------------------------------------------------------------
// Setup / deployment (§9.2/§9.3/§10)
// ---------------------------------------------------------------------------

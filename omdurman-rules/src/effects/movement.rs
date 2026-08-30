use super::*;

/// The neighbour index opposite to `idx` on a hex grid (three steps round the
/// six-sided ring). Used by howitzer scatter (§6.64) and Nile-current upstream
/// derivation.
pub(crate) const fn opposite(idx: usize) -> usize {
    (idx + 3) % 6
}

/// The neighbour index of `origin` that points most directly toward `target`
/// (used for deterministic howitzer scatter, §6.64).
pub fn toward_index(origin: HexCoord, target: HexCoord) -> usize {
    let neighbors = origin.neighbors();
    neighbors
        .iter()
        .enumerate()
        .min_by_key(|(_, n)| n.distance(target))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// One hex from `origin` toward `target` (§6.64 scatter helper).
pub fn step_toward(origin: HexCoord, target: HexCoord) -> HexCoord {
    origin.neighbors()[toward_index(origin, target)]
}

/// Validate and apply a unit movement (rulebook §5). When `path` is supplied
/// (the entered hexes, excluding the start, ending at `to`) the engine computes
/// the true terrain cost (§5.11) and enforces gunboat upstream/downstream
/// allowances (§5.24); otherwise it falls back to the caller-supplied `cost`.
pub fn apply_move_unit(
    state: &mut GameState,
    unit_id: UnitId,
    to: HexCoord,
    cost: MovementPoints,
    path: &[HexCoord],
) -> Result<(), RuleError> {
    let unit = state.unit_or_err(unit_id)?;
    // Copied out of `unit` so the immutable borrow ends before the state
    // mutations below (§5.24 flag, MP accounting, position update, §5.43 stop).
    let mover_owner = unit.profile.identity.owner();
    let mover_kind = unit.profile.kind;
    let is_gunboat = matches!(unit.profile.movement, crate::UnitMovement::Gunboat(_));
    let start_position = unit.position;

    // The effective cost is computed from the board+path when available, so the
    // engine -- not the caller -- is authoritative for movement-point spend.
    let mut effective_cost = state.movement_cost_for(unit, path).unwrap_or(cost);

    // §9.233: an empty-path (single-step) move trusts the caller's base cost
    // but still owes the +2 Zariba-end surcharge for the crossed hexside.
    // Non-empty paths already include it via `movement_cost_for`.
    if path.is_empty() {
        let surcharge = state.board.zariba_entry_surcharge(unit.position, to);
        effective_cost = MovementPoints(effective_cost.value() + surcharge);
    }

    // Phase / disruption / already-moved / allowance / ZOC-stop checks. Land
    // units validate against their (night-adjusted) land allowance; gunboats
    // validate against the up/downstream allowance for the path (§5.24).
    match unit.profile.movement {
        crate::UnitMovement::Immobile => {
            return Err(RuleError::AlreadyPlaced(unit_id));
        }
        crate::UnitMovement::Gunboat(_) => {
            state.can_move_gunboat(unit_id, to, path, effective_cost)?;
        }
        crate::UnitMovement::Land(_) => {
            state.can_move_unit_along(unit_id, to, path, effective_cost)?;
        }
    }

    // §7.1: no hex of the path may be enemy-occupied -- not even in passing.
    // (An enemy unit's own hex is not inside its ZOC ring, so the §5.26
    // transit check alone would let a path slip through it.)
    // §6.51 exception: an Anglo-Egyptian leader that is *alone* in a hex is
    // eliminated "when a Dervish unit occupies or passes through that hex" --
    // such a hex does not block a Dervish mover (the leader dies below).
    {
        let mover = unit.profile.identity.owner();
        let leader_hexes: Vec<HexCoord> = path
            .iter()
            .chain(std::iter::once(&to))
            .copied()
            .filter(|hex| {
                let occupants: Vec<&UnitPlacement> =
                    state.units.iter().filter(|u| u.position == *hex).collect();
                !occupants.is_empty()
                    && occupants
                        .iter()
                        .all(|u| u.profile.identity.owner() == Player::AngloEgyptian)
                    && occupants
                        .iter()
                        .all(|u| matches!(u.profile.kind, UnitKind::BritishLeader { .. }))
            })
            .collect();
        for hex in path.iter().chain(std::iter::once(&to)) {
            if leader_hexes.contains(hex) {
                continue; // §6.51: lone AE leaders are overrun, not obstacles.
            }
            if state
                .units
                .iter()
                .any(|u| u.position == *hex && u.profile.identity.owner() == mover.opponent())
            {
                return Err(RuleError::EnemyOccupied(*hex));
            }
        }
    }

    // §5.51-5.53: the stacking limit is checked at the *end* of the move.
    let mover = state.unit_or_err(unit_id)?;
    state.check_stacking(mover, to)?;

    // §5.24: record any upstream step now that the move is committed, so the
    // upstream allowance caps the gunboat's remaining moves this turn even if
    // they are all downstream (sticky cap). The FoK Nile-mouth crossing
    // (§9.345) spends "upstream" MPs and sets the flag too.
    if is_gunboat {
        let is_mouth_crossing = state.scenario == Scenario::FallOfKhartoum
            && state.is_nile_mouth_crossing(start_position, to);
        let steps: Vec<HexCoord> = if path.is_empty() {
            vec![to]
        } else {
            path.to_vec()
        };
        let mut prev = start_position;
        let mut went_upstream = is_mouth_crossing;
        for &next in &steps {
            if state.board.step_direction(prev, next) == Some(crate::board::StepDirection::Upstream)
            {
                went_upstream = true;
            }
            prev = next;
        }
        if went_upstream && !state.gunboats_upstream_this_turn.contains(&unit_id) {
            state.gunboats_upstream_this_turn.push(unit_id);
        }
    }

    // Record movement and update the unit's position -- the rules engine is
    // authoritative, so callers must not patch position separately. Track the
    // running MP spent this turn (§5.11/§5.12), so further steps are capped
    // cumulatively; "has moved" is derived as `mp_spent > 0` (used by
    // retreat-before-melee, §7.5).
    state
        .mp_spent_this_turn
        .entry(unit_id)
        .and_modify(|mp| *mp += effective_cost.value())
        .or_insert(effective_cost.value());
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.position = to;
    }

    // §5.26/§5.43: the unit has stopped if its destination lies in an enemy
    // ZOC -- it may move no further this turn (a gunboat only stops in an
    // enemy *gunboat's* ZOC, §5.41, which `hex_in_enemy_zoc` encodes).
    {
        let owner = mover_owner;
        let kind = mover_kind;
        if state.hex_in_enemy_zoc(to, owner, kind)
            && !state.zoc_stopped_this_turn.contains(&unit_id)
        {
            state.zoc_stopped_this_turn.push(unit_id);
        }
    }

    // §6.51: an Anglo-Egyptian leader alone in a hex entered (occupied or
    // passed through) by a Dervish unit is eliminated. The §7.1 occupancy
    // check above exempted those hexes from blocking the move.
    if mover_owner == Player::Dervish {
        let entered: Vec<HexCoord> = path.iter().copied().chain(std::iter::once(to)).collect();
        let overrun: Vec<UnitId> = state
            .units
            .iter()
            .filter(|u| {
                entered.contains(&u.position)
                    && matches!(u.profile.kind, UnitKind::BritishLeader { .. })
            })
            .filter(|u| {
                // "alone in a hex" -- no AE combat unit shares the hex.
                !state.units.iter().any(|other| {
                    other.position == u.position
                        && !matches!(other.profile.kind, UnitKind::BritishLeader { .. })
                        && other.profile.identity.owner() == Player::AngloEgyptian
                })
            })
            .map(|u| u.id)
            .collect();
        for leader in overrun {
            let is_gordon = state
                .find_unit(leader)
                .is_some_and(|u| u.profile.identity.is_gordon());
            score_elimination(state, leader, ElimCause::Combat);
            state.units.retain(|u| u.id != leader);
            if is_gordon {
                // §9.346/§9.35: Gordon's death fixes the FoK victory level
                // and ends the game (also for a pass-through overrun).
                eliminate_gordon(state);
            }
        }
    }

    // §9.346: a Dervish unit reaching the Palace eliminates GORDON (FoK).
    check_gordon_palace(state);

    Ok(())
}

// ---------------------------------------------------------------------------
// 7) Fire combat
// ---------------------------------------------------------------------------

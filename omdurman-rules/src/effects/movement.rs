use super::*;

/// Validate and apply a unit movement (rulebook §5). `path` is the ordered
/// hexes entered (excluding the start, ending at `to`; empty means the single
/// step to `to`). The whole path is validated step by step by
/// [`GameState::validate_move`] and the movement-point cost is computed by the
/// engine from the board (§5.11/§5.24): the caller-supplied `cost` is ignored
/// (it survives in the effect only for wire compatibility).
pub fn apply_move_unit(
    state: &mut GameState,
    unit_id: UnitId,
    to: HexCoord,
    _cost: MovementPoints,
    path: &[HexCoord],
) -> Result<(), RuleError> {
    let plan = state.validate_move(unit_id, to, path)?;
    let unit = state.unit_or_err(unit_id)?;
    // §5.51-5.53: the stacking limit is checked at the *end* of the move.
    state.check_stacking(unit, to)?;
    // Copied out of `unit` so the immutable borrow ends before the state
    // mutations below.
    let mover_owner = unit.profile.identity.owner();
    let mover_kind = unit.profile.kind;
    let entered: Vec<HexCoord> = if path.is_empty() {
        vec![to]
    } else {
        path.to_vec()
    };
    // §10.12: "When a British gunboat enters a mined hex, the Dervish player
    // must order it to stop" -- the move ends on the first live mine along
    // the path (the Anglo-Egyptian player does not know where they lie).
    let struck = (mover_owner == Player::AngloEgyptian
        && matches!(mover_kind, UnitKind::Gunboat { .. }))
    .then(|| {
        entered
            .iter()
            .position(|h| state.mines.iter().any(|m| m.hex == *h && !m.triggered))
    })
    .flatten();
    // §10.22: "When a British gunboat enters a 'chained' river hex it must
    // stop and may move no further that turn" -- likewise unknown to it.
    let chained = (mover_owner == Player::AngloEgyptian
        && matches!(mover_kind, UnitKind::Gunboat { .. }))
    .then(|| entered.iter().position(|h| state.chain_covers(*h)))
    .flatten();
    // The move ends at whichever comes first.
    let stop = match (struck, chained) {
        (Some(m), Some(c)) => Some(m.min(c)),
        (m, c) => m.or(c),
    };
    let struck = struck.filter(|m| Some(*m) == stop);
    let chained = chained.filter(|c| Some(*c) == stop);
    let came_from = unit.position;
    let (to, entered, plan) = match stop {
        Some(i) if i + 1 < entered.len() => {
            let cut = entered[..=i].to_vec();
            let plan = state.validate_move(unit_id, cut[i], &cut)?;
            // §5.51: the move now ends here, beside whoever lies here.
            state.check_stacking(unit, cut[i])?;
            (cut[i], cut, plan)
        }
        _ => (to, entered, plan),
    };

    // ---- validation complete; from here on the state is mutated ----

    // §5.24: record any upstream step now that the move is committed, so the
    // upstream allowance caps the gunboat's remaining moves this turn even if
    // they are all downstream (sticky cap). The FoK Nile-mouth crossing
    // (§9.345) spends "upstream" MPs and sets the flag too.
    if plan.went_upstream && !state.gunboats_upstream_this_turn.contains(&unit_id) {
        state.gunboats_upstream_this_turn.push(unit_id);
    }

    // Record movement and update the unit's position -- the rules engine is
    // authoritative, so callers must not patch position separately. Track the
    // running MP spent this turn (§5.11/§5.12), so further steps are capped
    // cumulatively; "has moved" is derived as `mp_spent > 0` (used by
    // retreat-before-melee, §7.5). `validate_move` bounded the total by the
    // allowance, so the addition cannot overflow.
    let spent = state.mp_spent(unit_id).saturating_add(plan.cost.value());
    state.mp_spent_this_turn.insert(unit_id, spent);
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.position = to;
    }
    // §5.21: a gunboat carries the "Friendlies" unit aboard it.
    for passenger in state
        .units
        .iter_mut()
        .filter(|u| u.state.loaded_on == Some(unit_id))
    {
        passenger.position = to;
    }
    if struck.is_some() {
        super::river::strikes_mine(state, unit_id, to);
    }
    match chained {
        Some(i) => {
            let from = if i == 0 { came_from } else { entered[i - 1] };
            state.gunboats_at_chain.insert(unit_id, from);
            if !state.gunboats_stopped_this_turn.contains(&unit_id) {
                state.gunboats_stopped_this_turn.push(unit_id);
            }
            state.observations.push(Observation::ChainStopsGunboat {
                gunboat: unit_id,
                hex: to,
            });
        }
        // Clear of the chain again.
        None => {
            state.gunboats_at_chain.remove(&unit_id);
        }
    }

    // §5.26/§5.43: the unit has stopped if its destination lies in an enemy
    // ZOC -- it may move no further this turn (a gunboat only stops in an
    // enemy *gunboat's* ZOC, §5.41, which `hex_in_enemy_zoc` encodes).
    if state.hex_in_enemy_zoc(to, mover_owner, mover_kind)
        && !state.zoc_stopped_this_turn.contains(&unit_id)
    {
        state.zoc_stopped_this_turn.push(unit_id);
    }

    // §6.51: an Anglo-Egyptian leader alone in a hex entered (occupied or
    // passed through) by a Dervish unit is eliminated. `validate_move`
    // exempted those hexes from blocking the move.
    if mover_owner == Player::Dervish {
        overrun_lone_leaders(state, &entered);
    }

    // §9.346: a Dervish unit reaching the Palace eliminates GORDON (FoK).
    check_gordon_palace(state);

    Ok(())
}

/// §6.51(a): eliminate every Anglo-Egyptian leader standing alone -- no AE
/// combat unit in its hex -- in one of `entered`, the hexes a Dervish unit
/// just occupied or passed through (by movement, or by advance after melee).
/// GORDON falls the same way (§9.346), recorded as his palace death.
pub(crate) fn overrun_lone_leaders(state: &mut GameState, entered: &[HexCoord]) {
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
        // §9.346/§9.35: the shared elimination path also records GORDON's
        // death (FoK), which ends the game -- a pass-through overrun counts.
        let gordon = state
            .find_unit(leader)
            .is_some_and(|u| u.profile.identity.is_gordon());
        let cause = if gordon {
            ElimCause::GordonAtPalace
        } else {
            ElimCause::Overrun
        };
        eliminate_unit(state, leader, cause);
    }
}

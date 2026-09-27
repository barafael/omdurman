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
            // §9.346/§9.35: the shared elimination path also records
            // GORDON's death (FoK), which ends the game -- a pass-through
            // overrun counts.
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

    // §9.346: a Dervish unit reaching the Palace eliminates GORDON (FoK).
    check_gordon_palace(state);

    Ok(())
}

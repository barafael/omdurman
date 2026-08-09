//! Engine-state invariants for proptest verification (§5, §6, §9).
//!
//! Each predicate is asserted after every `apply_effect` in a random
//! playthrough. A violation means the engine produced an illegal state — a
//! rule-implementation bug. proptest's automatic shrinking then reduces the
//! violating trace to a minimal reproducible sequence.

use omdurman_rules::effects::GameState;
use omdurman_types::HexCoord;

/// 1. Every unit's position is a valid board hex.
pub fn all_units_on_board(state: &GameState) -> Result<(), String> {
    for u in &state.units {
        if !state.board.terrain.contains_key(&u.position) {
            return Err(format!(
                "unit {:?} at {:?} is not on the board",
                u.profile.identity, u.position
            ));
        }
    }
    Ok(())
}

/// 2. Stacking limit never exceeded (§5.51 — at most four units per hex,
///    excluding free-stacking leaders and gunboats, which never count toward
///    the limit).
pub fn stacking_ok(state: &GameState) -> Result<(), String> {
    use omdurman_types::UnitKind;
    use std::collections::HashMap;
    let mut counts: HashMap<HexCoord, usize> = HashMap::new();
    for u in &state.units {
        if matches!(
            u.profile.kind,
            UnitKind::DervishLeader { .. } | UnitKind::BritishLeader { .. } | UnitKind::Gunboat { .. }
        ) {
            continue;
        }
        *counts.entry(u.position).or_default() += 1;
    }
    for (hex, n) in &counts {
        if *n > 4 {
            return Err(format!("{n} units stacked at ({},{}) — exceeds §5.51", hex.q, hex.r));
        }
    }
    Ok(())
}

/// 3. No land unit on a Nile hex, no boat on a non-Nile hex (§5.22).
pub fn land_boat_separation(state: &GameState) -> Result<(), String> {
    for u in &state.units {
        let is_nile = state.board.is_nile(u.position);
        let is_boat = u.profile.kind.is_boat();
        if is_boat && !is_nile {
            return Err(format!(
                "boat {:?} on non-Nile hex ({},{}) — §5.22",
                u.profile.identity, u.position.q, u.position.r
            ));
        }
        if !is_boat && is_nile {
            return Err(format!(
                "land unit {:?} on Nile hex ({},{}) — §5.22",
                u.profile.identity, u.position.q, u.position.r
            ));
        }
    }
    Ok(())
}

/// 4. `units_fired_this_phase` only references extant units.
pub fn fired_units_exist(state: &GameState) -> Result<(), String> {
    for id in &state.units_fired_this_phase {
        if !state.units.iter().any(|u| &u.id == id) {
            return Err(format!("fired-units set references missing unit {id:?}"));
        }
    }
    Ok(())
}

/// 5. `mp_spent_this_turn` never exceeds a land unit's effective movement
///    allowance (§5.11; halved at night for the Anglo-Egyptian, §8.1). Boats
///    and immobile units are tracked under separate caps and skipped here.
pub fn mp_within_allowance(state: &GameState) -> Result<(), String> {
    use omdurman_rules::{effective_movement_at_night, UnitMovement};
    for u in &state.units {
        if let UnitMovement::Land(allowance) = u.profile.movement {
            let owner = u.profile.identity.owner();
            let eff = effective_movement_at_night(allowance, owner, state.day_night);
            let spent = state.mp_spent(u.id);
            if spent > eff.value() as i16 {
                return Err(format!(
                    "unit {:?} spent {spent} MP, exceeding allowance {} (§5.11)",
                    u.profile.identity,
                    eff.value()
                ));
            }
        }
    }
    Ok(())
}

/// 5. `game_over` is monotonic (once true, stays true). This is checked across
///    the trace, not per-state; provided here as a helper for callers that
///    track the previous value.
pub fn game_over_monotonic(prev: bool, curr: bool) -> Result<(), String> {
    if prev && !curr {
        return Err("game_over went from true to false".into());
    }
    Ok(())
}

/// Run all per-state invariants. Returns the first failure.
pub fn check_all(state: &GameState) -> Result<(), String> {
    all_units_on_board(state)?;
    stacking_ok(state)?;
    land_boat_separation(state)?;
    fired_units_exist(state)?;
    mp_within_allowance(state)?;
    Ok(())
}

/// Run all invariants including the Dervish-tribal stacking rule (§5.52):
/// no hex may contain two different Dervish tribes.
pub fn check_all_with_tribal(state: &GameState) -> Result<(), String> {
    check_all(state)?;
    // §5.52: different Dervish tribes may not stack.
    use omdurman_rules::UnitIdentity;
    use omdurman_types::DervishTribe;
    use std::collections::HashMap;
    let mut tribes: HashMap<HexCoord, std::collections::HashSet<DervishTribe>> =
        HashMap::new();
    for u in &state.units {
        if let UnitIdentity::DervishTribal { tribe } = u.profile.identity {
            tribes.entry(u.position).or_default().insert(tribe);
        }
    }
    for (hex, set) in &tribes {
        if set.len() > 1 {
            return Err(format!(
                "hex ({},{}) mixes {:?} tribes — §5.52",
                hex.q, hex.r, set
            ));
        }
    }
    Ok(())
}

//! Per-phase movement memory: each unit's plan for the phase, and the hexes
//! it has stood on.
//!
//! A commander plans a unit's move once per movement phase: the best hex it
//! can reach (`commanders`, "position values"), and the way there. The
//! driver then walks the plan one step at a time -- a shortest way never
//! re-enters a hex, so a unit cannot go back and forth. The hexes stood on
//! are the guard behind it: no move back onto one of them is ever taken.
//!
//! Determinism: the memory is derived only from the driver's own picks, so a
//! seeded run reproduces it exactly. A driver that loses it mid-phase (a host
//! rebuilt after a reconnect) merely starts afresh from the units' current
//! hexes.

use std::collections::{BTreeMap, BTreeSet};

use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{GameTurnIndex, Phase, UnitId};
use omdurman_types::{HexCoord, Player};

/// The hexes each unit has occupied in the current movement phase.
#[derive(Clone, Debug, Default)]
pub struct MoveMemory {
    /// The phase the memory is about; another one clears it.
    phase: Option<(GameTurnIndex, Phase, Player)>,
    visited: BTreeMap<UnitId, BTreeSet<HexCoord>>,
    /// Each unit's planned way this phase (the hexes still to enter) and
    /// what reaching its end gains it; an empty way means it holds.
    plans: BTreeMap<UnitId, (Vec<HexCoord>, i32)>,
    /// How often a unit's plan was refused this phase (see
    /// [`MoveMemory::abandon_plan`]).
    refusals: BTreeMap<UnitId, u8>,
}

/// Plans a unit may have refused in one phase before it holds.
const MAX_REPLANS: u8 = 3;

/// How near an arrival a plan's end must lie to be made again (see
/// [`MoveMemory::record`]).
const REPLAN_RADIUS: u32 = 3;

impl MoveMemory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Forget everything once the game has moved on to another phase.
    fn sync(&mut self, state: &GameState) {
        let now = (state.current_turn, state.phase, state.active_player);
        if self.phase != Some(now) {
            self.phase = Some(now);
            self.visited.clear();
            self.plans.clear();
            self.refusals.clear();
        }
    }

    /// Whether `effect` would put a unit back on a hex it has already
    /// occupied this phase (any hex of its path, the destination included).
    pub fn revisits(&mut self, state: &GameState, effect: &GameEffect) -> bool {
        let GameEffect::MoveUnit {
            unit_id, to, path, ..
        } = effect
        else {
            return false;
        };
        self.sync(state);
        let Some(unit) = state.find_unit(*unit_id) else {
            return false;
        };
        let seen = self.visited.entry(*unit_id).or_default();
        seen.insert(unit.position);
        path.iter()
            .chain(std::iter::once(to))
            .any(|h| seen.contains(h))
    }

    /// `unit`'s plan for this phase -- the hexes still to enter and the
    /// gain at the end -- made by `make` the first time it is asked for.
    pub fn plan_for(
        &mut self,
        state: &GameState,
        unit: UnitId,
        make: impl FnOnce() -> (Vec<HexCoord>, i32),
    ) -> (Option<HexCoord>, i32) {
        self.sync(state);
        let (way, gain) = self.plans.entry(unit).or_insert_with(make);
        (way.first().copied(), *gain)
    }

    /// `unit`'s planned step was refused (friends filled the hex it was
    /// making for, or a rule the plan could not foresee): plan again from
    /// the board as it now stands -- up to a few times, then hold.
    pub fn abandon_plan(&mut self, state: &GameState, unit: UnitId) {
        self.sync(state);
        let refused = self.refusals.entry(unit).or_insert(0);
        *refused += 1;
        if *refused >= MAX_REPLANS {
            self.plans.insert(unit, (Vec::new(), 0));
        } else {
            self.plans.remove(&unit);
        }
    }

    /// Note the effect the driver submitted (call with the state it was
    /// picked in, before it is applied).
    pub fn record(&mut self, state: &GameState, effect: &GameEffect) {
        let GameEffect::MoveUnit {
            unit_id, to, path, ..
        } = effect
        else {
            return;
        };
        self.sync(state);
        let seen = self.visited.entry(*unit_id).or_default();
        if let Some(unit) = state.find_unit(*unit_id) {
            seen.insert(unit.position);
        }
        seen.extend(path.iter().copied());
        seen.insert(*to);
        // Walk the plan: a step along it consumes it; any other move (a
        // driver without plans) voids it.
        let mut arrived = false;
        if let Some((way, _)) = self.plans.get_mut(unit_id) {
            let steps = path.len().max(1);
            if way.len() >= steps && way[steps - 1] == *to {
                way.drain(..steps);
                arrived = way.is_empty();
            } else {
                self.plans.remove(unit_id);
            }
        }
        // An arrival changes the board the other plans were made on: the
        // company a hex keeps, the room left in a stack, which gate still
        // wants a plug. Plans that end (or units that hold) near the
        // arrival are made again; in Fall of Khartoum, where a handful of
        // units share a few gates, all of them.
        if arrived {
            let everyone = state.scenario == omdurman_types::Scenario::FallOfKhartoum;
            let near = |hex: HexCoord| hex.distance(*to) <= REPLAN_RADIUS;
            self.plans.retain(|other, (way, _)| {
                if other == unit_id {
                    return true;
                }
                let end = way
                    .last()
                    .copied()
                    .or_else(|| state.find_unit(*other).map(|u| u.position));
                !(everyone || end.is_some_and(near))
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_rules::MovementPoints;

    fn step(unit_id: UnitId, to: HexCoord) -> GameEffect {
        GameEffect::MoveUnit {
            unit_id,
            to,
            cost: MovementPoints::new(1),
            path: vec![to],
        }
    }

    #[test]
    fn a_unit_may_not_step_back_onto_a_hex_it_left_this_phase() {
        let scenario = omdurman_types::Scenario::Campaign;
        let board = crate::playthrough::board_for_scenario(scenario);
        let mut state = GameState::with_board(scenario, board);
        let (_, id) = crate::oob::deployable_oob(scenario)[0];
        state.units.push(omdurman_rules::UnitPlacement {
            id,
            position: HexCoord::new(20, 20),
            profile: omdurman_rules::unit_profiles::profile_for_unit(id).unwrap(),
            state: Default::default(),
        });
        let unit = state.units[0];
        let a = unit.position;
        let b = a.neighbors()[0];
        let c = a.neighbors()[1];
        let mut memory = MoveMemory::new();
        assert!(!memory.revisits(&state, &step(unit.id, b)));
        memory.record(&state, &step(unit.id, b));
        state.units[0].position = b;
        assert!(
            memory.revisits(&state, &step(unit.id, a)),
            "back to the start"
        );
        assert!(memory.revisits(&state, &step(unit.id, b)), "onto itself");
        assert!(
            !memory.revisits(&state, &step(unit.id, c)),
            "new ground is fine"
        );
        // Another phase starts afresh.
        state.phase = Phase::Melee;
        assert!(!memory.revisits(&state, &step(unit.id, a)));
    }
}

//! Per-phase movement memory: a unit whose best move would take it back onto
//! a hex it has already stood on in the current movement phase halts there
//! for the rest of the phase.
//!
//! The commanders pick one one-hex `MoveUnit` step at a time and re-score
//! the board after every step. Their scores are not consistent from one
//! step to the next (a step into a fire lane pays for its progress; the
//! step back out pays for the lane it leaves), so a greedy unit could walk
//! A→B→A→B… until its movement allowance (§5.13: MP don't carry over) was
//! spent. The engine state records how far a unit has moved, not where it
//! has been, so the driver keeps that here: one [`MoveMemory`] per game,
//! fed every effect it picks, and consulted before every pick.
//!
//! Halting, rather than merely forbidding the step back, matters: a unit
//! barred from its old hex would take its next-best step instead -- often
//! deeper into the fire lane it was trying to leave. Once its own best
//! option is to undo a step, the unit has nothing better to do this phase.
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
    /// Units done moving this phase.
    halted: BTreeSet<UnitId>,
}

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
            self.halted.clear();
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

    /// Whether the driver may take `effect`, considering the candidates
    /// best-first: a move of a halted unit is refused, and a unit whose best
    /// move would revisit a hex halts. Everything but `MoveUnit` passes.
    pub fn screen(&mut self, state: &GameState, effect: &GameEffect) -> bool {
        let GameEffect::MoveUnit { unit_id, .. } = effect else {
            return true;
        };
        self.sync(state);
        if self.halted.contains(unit_id) {
            return false;
        }
        if self.revisits(state, effect) {
            self.halted.insert(*unit_id);
            return false;
        }
        true
    }

    /// The candidates without the moves of the units halted this phase.
    pub fn without_halted(
        &mut self,
        state: &GameState,
        candidates: &[GameEffect],
    ) -> Vec<GameEffect> {
        self.sync(state);
        candidates
            .iter()
            .filter(|e| match e {
                GameEffect::MoveUnit { unit_id, .. } => !self.halted.contains(unit_id),
                _ => true,
            })
            .cloned()
            .collect()
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

//! Engineering validators: Royal Engineers demolition (rulebook §6.53) and
//! zariba construction.

use super::*;

impl GameState {
    /// Read-only check of whether a Royal Engineers demolition may begin
    /// (§6.53): the unit is the Royal Engineers, it is its owner's Movement
    /// phase ("the Royal Engineers must move adjacent ... and end their
    /// movement adjacent"), and it is undisrupted and not already committed
    /// to a demolition or a zariba construction. The target itself must be
    /// one of [`Self::demolition_targets`] (checked by the effect).
    pub fn can_demolition(&self, unit_id: UnitId) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if unit.profile.identity != crate::UnitIdentity::RoyalEngineers {
            return Err(RuleError::NotRoyalEngineers(unit_id));
        }
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        if unit.profile.identity.owner() != self.active_player {
            return Err(RuleError::NotYourTurn);
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit_id));
        }
        if unit.state.demolishing || unit.state.constructing_zariba {
            return Err(RuleError::BusyWithEngineering(unit_id));
        }
        Ok(())
    }

    /// Read-only discovery of the demolition targets adjacent to `unit_id`
    /// (§6.53): *enemy* fort units in the six neighbouring hexes plus standing
    /// Wall hexsides on the six neighbouring sides. Pairs with [`GameState::can_demolition`]
    /// and [`GameEffect::Demolition`] so the UI can offer exactly the targets
    /// the rules would accept. Empty when the unit doesn't exist or has no
    /// adjacent target.
    pub fn demolition_targets(&self, unit_id: UnitId) -> Vec<DemolitionTarget> {
        let Ok(unit) = self.unit_or_err(unit_id) else {
            return Vec::new();
        };
        let mut targets = Vec::new();
        for n in unit.position.neighbors() {
            if let Some(fort) = self.units.iter().find(|u| {
                u.position == n
                    && matches!(u.profile.kind, UnitKind::Fort { .. })
                    && u.profile.identity.owner() != unit.profile.identity.owner()
            }) {
                targets.push(DemolitionTarget::Fort(fort.id));
            }
            if self.hexside_effective_is(unit.position, n, |k| k == HexsideKind::Wall) {
                targets.push(DemolitionTarget::WallHexside(HexsideRef::new(
                    unit.position,
                    n,
                )));
            }
        }
        targets
    }

    /// Read-only check of whether the given units may construct the Zariba
    /// (§5.3): a Campaign-game option ("these hexsides are considered clear
    /// terrain in the campaign game ... the Anglo-Egyptian player may,
    /// however, ... construct this defensive position") begun in the
    /// Anglo-Egyptian Movement phase. `hexside` must be one of the printed
    /// Zariba hexsides ("may only be built in their position as displayed on
    /// the mapsheet"), not yet built; each unit undisrupted Anglo-Egyptian
    /// infantry that has not moved this turn ("begins ... the player turn
    /// adjacent"), not already building or demolishing, and standing next to
    /// it on the Nile side -- inside the printed Zariba. The builders then
    /// hold still ("... and ends") and at the end of the Anglo-Egyptian
    /// player turn have "constructed all Zariba hexsides to which [they are]
    /// adjacent" (`end_player_turn`).
    pub fn can_construct_zariba(
        &self,
        unit_ids: &[UnitId],
        hexside: HexsideRef,
    ) -> Result<(), RuleError> {
        if self.scenario != Scenario::Campaign {
            return Err(RuleError::IllegalZariba(
                "the Zariba is only constructed in the campaign game",
            ));
        }
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        if self.active_player != Player::AngloEgyptian {
            return Err(RuleError::NotYourTurn);
        }
        if unit_ids.is_empty() {
            return Err(RuleError::IllegalZariba("no constructing units"));
        }
        crate::effects::reject_duplicate_units(unit_ids)?;
        if !self.is_printed_zariba_side(hexside.a, hexside.b) {
            return Err(RuleError::IllegalZariba(
                "the Zariba may only be built in its printed position",
            ));
        }
        if self.zariba_hexsides.contains(&hexside) {
            return Err(RuleError::IllegalZariba("that Zariba hexside is built"));
        }
        for &id in unit_ids {
            let unit = self.unit_or_err(id)?;
            if unit.profile.identity.owner() != Player::AngloEgyptian {
                return Err(RuleError::NotOwner(id));
            }
            if !matches!(unit.profile.kind, UnitKind::Infantry { .. }) {
                return Err(RuleError::IllegalZariba(
                    "only Anglo-Egyptian infantry construct the Zariba",
                ));
            }
            if unit.state.disrupted {
                return Err(RuleError::Disrupted(id));
            }
            if unit.state.constructing_zariba || unit.state.demolishing {
                return Err(RuleError::BusyWithEngineering(id));
            }
            if self.mp_spent(id) > 0 {
                return Err(RuleError::AlreadyMoved(id));
            }
            if unit.position != hexside.a && unit.position != hexside.b {
                return Err(RuleError::IllegalZariba(
                    "the constructing unit must be adjacent to the hexside",
                ));
            }
            if !self.board.is_zariba(unit.position) {
                return Err(RuleError::IllegalZariba(
                    "the constructing unit must stand on the Nile side, inside the Zariba",
                ));
            }
        }
        Ok(())
    }
}

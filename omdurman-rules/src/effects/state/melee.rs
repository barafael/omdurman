//! Melee validators (rulebook §7): declaring melee, retreat before melee,
//! advance after combat, and recovery from disruption.

use super::*;

impl GameState {
    /// The units `attack` would strike if resolved now: the defending side's
    /// meleeable units still on the target hex (a defender that withdrew
    /// during the §7.5 window is gone). Empty means the melee lapses.
    pub fn melee_defenders_now(&self, attack: &MeleeAttack) -> Vec<UnitId> {
        let defender_player = attack.attacker_player.opponent();
        self.units
            .iter()
            .filter(|u| {
                u.position == attack.defender_hex
                    && u.profile.identity.owner() == defender_player
                    && u.profile.kind.may_be_melee_attacked()
            })
            .map(|u| u.id)
            .collect()
    }

    /// Read-only check of whether `attacker` may melee-attack the adjacent
    /// `defender_hex` in the current state (§7): Melee phase, attacker is the
    /// active player, attacker is a melee-capable kind (§7.4), not disrupted,
    /// adjacent to the target, the target hex holds at least one enemy unit
    /// that may be melee-attacked (gunboats may not -- §7.1), and no wall or
    /// thorn-hedge hexside blocks the attack (§7.2).
    pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
        let unit = self.unit_or_err(attacker)?;

        if !matches!(self.phase, Phase::Melee) {
            return Err(RuleError::WrongPhase);
        }
        if self.active_player != unit.profile.identity.owner() {
            return Err(RuleError::NotYourTurn);
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(attacker));
        }
        // A unit that recovered from disruption early (`RecoverUnit`) is
        // still spent for this turn: recovery happens at the *end* of the
        // owning player's turn (reference notes), after all melee.
        if self.turn_events.iter().any(|e| {
            matches!(e, crate::turn_summary::TurnEventRecord::UnitRecovered { unit } if *unit == attacker)
        }) {
            return Err(RuleError::Disrupted(attacker));
        }
        // §5.3/§6.53: constructing or demolishing units may not melee attack.
        if unit.state.constructing_zariba || unit.state.demolishing {
            return Err(RuleError::BusyWithEngineering(attacker));
        }
        if !unit.profile.kind.may_melee_attack() {
            return Err(RuleError::KindMayNotMelee(attacker));
        }
        if !unit.position.neighbors().contains(&defender_hex) {
            return Err(RuleError::TargetNotAdjacent {
                from: unit.position,
                to: defender_hex,
            });
        }
        let enemy = unit.profile.identity.owner().opponent();
        let has_target = self.units.iter().any(|u| {
            u.position == defender_hex
                && u.profile.identity.owner() == enemy
                && u.profile.kind.may_be_melee_attacked()
        });
        if !has_target {
            return Err(RuleError::NoMeleeableEnemy(defender_hex));
        }
        // §7.2: walls, thorn-hedges and khors block melee across them
        // (gates and breaches pass). Read through `hexside_effective` so a
        // §6.63 breach is an opening.
        if let Some(side) = self
            .hexside_effective(unit.position, defender_hex)
            .filter(|k| k.blocks_melee())
        {
            return Err(RuleError::MeleeBlockedByHexside(
                unit.position,
                defender_hex,
                side,
            ));
        }
        Ok(())
    }

    /// Read-only check of whether `unit_id` may retreat two hexes to `to`
    /// before an impending infantry melee (§7.5): Melee phase, cavalry/camel
    /// kind, not disrupted, not already moved/retreated this turn, `to` exactly
    /// two hexes away and empty. (Does not verify the attacker is infantry --
    /// the caller offers the retreat only in response to one.)
    pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if !matches!(self.phase, Phase::Melee) {
            return Err(RuleError::WrongPhase);
        }
        // Retreat is a *reaction* to a declared *infantry* melee attack on the
        // unit's hex (§7.5): there must be a pending melee targeting where it
        // stands, made by at least one infantry attacker.
        match &self.pending_melee {
            Some(p)
                if p.attack.defender_hex == unit.position
                    && p.attack.attackers.iter().any(|id| {
                        self.find_unit(*id)
                            .is_some_and(|u| matches!(u.profile.kind, UnitKind::Infantry { .. }))
                    }) => {}
            _ => {
                return Err(RuleError::NoInfantryMeleeThreatens(unit_id));
            }
        }
        if !unit.profile.kind.may_retreat_before_melee() {
            return Err(RuleError::MayNotRetreatBeforeMelee(unit_id));
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit_id));
        }
        if self.mp_spent(unit_id) > 0 {
            return Err(RuleError::AlreadyMoved(unit_id));
        }
        if unit.position.distance(to) != 2 {
            return Err(RuleError::RetreatMustBeTwoHexes);
        }
        if self.units.iter().any(|u| u.position == to) {
            return Err(RuleError::RetreatHexOccupied(to));
        }
        // §5.22: a retreating unit must stay on the board (with no board
        // loaded, map constraints don't apply).
        if !self.board.terrain.is_empty() && self.board.terrain_at(to).is_none() {
            return Err(RuleError::OffBoard(to));
        }
        // §5.22: land units may *never* enter a Nile hex -- a retreat is no
        // exception (a cavalry retiring two hexes onto the river is not a
        // legal move).
        if !unit.profile.kind.is_boat() && self.board.is_nile(to) {
            return Err(RuleError::LandIntoNile(to));
        }
        // §6.54: a retreat may not end on an enemy fort -- players may not
        // occupy an enemy fort under any circumstances.
        if self.hex_has_enemy_fort(to, unit.profile.identity.owner()) {
            return Err(RuleError::EnemyFort(to));
        }
        // §5.23: movement may not cross a wall hexside except through a gate
        // or breach -- a retreat is no exception. A two-hex retreat passes
        // through one of the (at most two) common neighbours of `from` and
        // `to`; at least one intermediate must have both legs non-wall.
        let wall_free_path = unit.position.neighbors().iter().any(|mid| {
            mid.neighbors().contains(&to)
                && self.hexside_effective(unit.position, *mid) != Some(HexsideKind::Wall)
                && self.hexside_effective(*mid, to) != Some(HexsideKind::Wall)
        });
        if !wall_free_path {
            return Err(RuleError::RetreatBlockedByWall(unit.position, to));
        }
        Ok(())
    }

    /// Read-only check of whether `unit_id` may advance after combat into the
    /// vacated `to` hex (§6.82, §7.6): a fire or melee phase, the active
    /// player's unit, not artillery, adjacent to `to`, no enemy in `to`, and
    /// the stacking law kept with any friendly units that advanced first.
    pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        // §6.7: there is no advance after combat as a result of defensive fire.
        // Advance is permitted only after melee (§7.6) and offensive fire
        // (§6.82) -- never in a defensive-fire subphase.
        if !matches!(self.phase, Phase::Melee | Phase::OffensiveFire(_)) {
            return Err(RuleError::WrongPhase);
        }
        if matches!(unit.profile.kind, UnitKind::Artillery { .. }) {
            return Err(RuleError::ArtilleryMayNotAdvance(unit_id));
        }
        // §5.25: "Dervish forts may not move in any way once placed" -- an
        // advance-after-combat is movement.
        if matches!(unit.profile.kind, UnitKind::Fort { .. }) {
            return Err(RuleError::FortMayNotAdvance(unit_id));
        }
        if !unit.position.neighbors().contains(&to) {
            return Err(RuleError::AdvanceNotAdjacent);
        }
        // §6.82/§7.6: the hex must have been vacated by combat this phase --
        // an advance answers the attack that emptied it, so merely-empty
        // hexes are not advance targets (this is what stops advance-after-
        // combat being used as free out-of-phase movement).
        let eligible = self
            .vacated_by_combat
            .get(&to)
            .ok_or(RuleError::HexNotVacatedByCombat(to))?;
        // §6.82/§7.6: "the friendly units must have participated in the
        // attack" -- only listed participants may advance.
        if !eligible.contains(&unit_id) {
            return Err(RuleError::UnitDidNotParticipate(unit_id, to));
        }
        // §5.22: a unit may only advance into a hex it could occupy -- boats
        // stay on the Nile, land units stay off it, and nobody advances off
        // the board (with no board loaded, map constraints don't apply).
        if matches!(unit.profile.kind, UnitKind::Gunboat { .. }) {
            if !self.board.terrain.is_empty() && !self.board.is_nile(to) {
                return Err(RuleError::GunboatOffNile(to));
            }
        } else {
            if !self.board.terrain.is_empty() && self.board.terrain_at(to).is_none() {
                return Err(RuleError::OffBoard(to));
            }
            if self.board.is_nile(to) {
                return Err(RuleError::LandIntoNile(to));
            }
        }
        // §6.54: may not advance after combat into an enemy fort, even if the
        // fort is unoccupied (a fort is never captured -- only destroyed).
        if self.hex_has_enemy_fort(to, unit.profile.identity.owner()) {
            return Err(RuleError::EnemyFort(to));
        }
        // The vacated hex must still be free of the enemy. Friendly units
        // that advanced into it first do not close it: "all surviving
        // eligible units ... advance, up to the stacking limit" (§7.6), so a
        // later advancer answers to the stacking law instead (§5.51/§5.52).
        let owner = unit.profile.identity.owner();
        if self
            .units
            .iter()
            .any(|u| u.position == to && u.profile.identity.owner() != owner)
        {
            return Err(RuleError::AdvanceNotVacant(to));
        }
        self.check_stacking(unit, to)?;
        // §6.82 / §7.6: may not advance across a wall (except gate/breach),
        // khor, or thorn-hedge hexside. Read through `hexside_effective` so a
        // §6.63 breach is an opening.
        if self.hexside_effective_is(unit.position, to, HexsideKind::blocks_advance_after_combat) {
            return Err(RuleError::AdvanceBlockedByHexside(unit.position, to));
        }
        Ok(())
    }

    /// Read-only check of whether `unit_id` may recover from disruption
    /// (paired with [`apply_recover_unit`]). Per the reference notes,
    /// disrupted units "are turned face up at the end of the owning player's
    /// turn" -- `end_player_turn` does that automatically. An explicit
    /// `RecoverUnit` is therefore only the owner turning the counter a little
    /// early, at the end of their turn: the unit must be disrupted and the
    /// active player's own, in the Melee phase (the last phase of the player
    /// turn) with no declared melee pending. A unit so recovered still may
    /// not melee this turn (see [`Self::can_melee`]).
    pub fn can_recover_unit(&self, unit_id: UnitId) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if !unit.state.disrupted {
            return Err(RuleError::NotDisrupted(unit_id));
        }
        if unit.profile.identity.owner() != self.active_player {
            return Err(RuleError::NotYourTurn);
        }
        if !matches!(self.phase, Phase::Melee) {
            return Err(RuleError::WrongPhase);
        }
        if self.pending_melee.is_some() {
            return Err(RuleError::MeleeAlreadyPending);
        }
        Ok(())
    }
}

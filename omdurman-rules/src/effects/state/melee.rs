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
        // §5.3/§6.53: constructing or demolishing units may not melee attack.
        if unit.state.constructing_zariba || unit.state.demolishing {
            return Err(RuleError::BusyWithEngineering(attacker));
        }
        // §7.5: a unit makes one melee attack a turn -- only "enemy units
        // whose melee attacks have not yet been resolved" may still attack.
        if self.units_meleed_this_turn.contains(&attacker) {
            return Err(RuleError::AlreadyMeleed(attacker));
        }
        if !unit.profile.kind.may_melee_attack() {
            return Err(RuleError::KindMayNotMelee(attacker));
        }
        // §5.21/§7.1: nobody melees from or into the river -- a "Friendlies"
        // unit aboard a gunboat is out of reach, and fights no one.
        if unit.state.loaded_on.is_some() {
            return Err(RuleError::LoadedOnGunboat(attacker));
        }
        if !unit.position.neighbors().contains(&defender_hex) {
            return Err(RuleError::TargetNotAdjacent {
                from: unit.position,
                to: defender_hex,
            });
        }
        let enemy = unit.profile.identity.owner().opponent();
        if self.board.is_nile(defender_hex) {
            return Err(RuleError::NoMeleeableEnemy(defender_hex));
        }
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
    /// before an impending infantry melee (§7.5): Melee phase, a pending
    /// melee with an infantry attacker on the unit's hex, cavalry/camel kind,
    /// not disrupted, not already retreated this turn, `to` exactly two hexes
    /// away, free of the enemy and within the stacking law, reached through
    /// an open intervening hex.
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
        let owner = unit.profile.identity.owner();
        let enemy_at = |hex: HexCoord| {
            self.units
                .iter()
                .any(|u| u.position == hex && u.profile.identity.owner() != owner)
        };
        if enemy_at(to) {
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
        if self.hex_has_enemy_fort(to, owner) {
            return Err(RuleError::EnemyFort(to));
        }
        // §5.51-§5.53: the retreat ends like a move, under the stacking law.
        self.check_stacking(unit, to)?;
        // The two-hex retreat is movement through one of the (at most two)
        // common neighbours of `from` and `to`: both steps must be ones the
        // unit could move -- no enemy, no Nile, no closed hexside (§5.23
        // wall, §9.231/§9.233 Zariba, the khor is merely costly), no entry
        // into the walled city by a unit barred from it (§5.23).
        let open_path = unit.position.neighbors().iter().any(|&mid| {
            mid.is_adjacent_to(to)
                && !enemy_at(mid)
                && self.check_land_step(unit, unit.position, mid).is_ok()
                && self.check_land_step(unit, mid, to).is_ok()
        });
        if !open_path {
            return Err(RuleError::RetreatPathBlocked(unit.position, to));
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
        // Disrupted units "may not move" (CRT key) -- an attacker disrupted
        // by the defender's simultaneous roll stays where it is.
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit_id));
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
        // §5.23: an advance through a gate or breach is no way into the
        // walled city for a unit barred from it.
        self.check_walled_city_entry(unit, unit.position, to)?;
        Ok(())
    }
}

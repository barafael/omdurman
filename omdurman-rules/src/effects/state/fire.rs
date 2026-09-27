//! Fire-combat validators (rulebook §6), including [`GameState::can_fire_at`]
//! and [`GameState::can_fire_at_wall`].

use super::*;

impl GameState {
    /// The player whose fire attacks are legal right now (§4): the active
    /// player during Offensive Fire, their opponent during Defensive Fire.
    /// `Err(WrongPhase)` outside both fire phases. Shared by
    /// [`Self::can_fire_at`] and [`Self::can_fire_at_wall`].
    fn fire_phase_player(&self) -> Result<Player, RuleError> {
        match self.phase {
            Phase::OffensiveFire(_) => Ok(self.active_player),
            Phase::DefensiveFire(_) => Ok(self.active_player.opponent()),
            _ => Err(RuleError::WrongPhase),
        }
    }

    /// The "Units" line-of-sight blocker (§6.3 note a): a hex occupied by any
    /// non-gunboat, non-fort unit blocks LOS at that hex's terrain level.
    /// Shared by [`Self::can_fire_at`] and [`Self::can_fire_at_wall`].
    fn los_unit_blocker(&self) -> impl Fn(HexCoord) -> Option<crate::los_table::LosLevel> + '_ {
        move |hex| {
            let has_blocking_unit = self.units.iter().any(|u| {
                u.position == hex
                    && !matches!(
                        u.profile.kind,
                        crate::UnitKind::Gunboat { .. } | crate::UnitKind::Fort { .. }
                    )
            });
            if has_blocking_unit {
                self.board.terrain_at(hex).map(crate::los_table::los_level)
            } else {
                None
            }
        }
    }

    /// Read-only check of whether `firer` may fire `kind` at `target_hex` in
    /// the current state (§6): right fire sub-phase for the kind, right player,
    /// firer has a fire factor, weapon class permits the kind, not disrupted,
    /// hasn't already fired this phase, and the target is within (night-
    /// adjusted) range for the firer's weapon.
    ///
    /// Does **not** check line of sight or terrain -- those need the game map,
    /// which the rules engine does not hold; the app supplies the terrain
    /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
    /// (Howitzer fire ignores LOS entirely -- §6.64.)
    pub fn can_fire_at(
        &self,
        firer: UnitId,
        target_hex: HexCoord,
        kind: FireKind,
    ) -> Result<(), RuleError> {
        let unit = self.unit_or_err(firer)?;

        if unit.profile.identity.owner() != self.fire_phase_player()? {
            return Err(RuleError::NotYourTurn);
        }

        // The fire kind must match the current sub-phase (§6.42): direct fire
        // in the Direct sub-phase; Maxim-second / howitzer in the second.
        let sub = match self.phase {
            Phase::OffensiveFire(s) | Phase::DefensiveFire(s) => s,
            _ => return Err(RuleError::WrongPhase),
        };
        let kind_ok = matches!(
            (sub, kind),
            (FireSubPhase::DirectFire, FireKind::Direct)
                | (
                    FireSubPhase::MaximSecondAndHowitzer,
                    FireKind::MaximSecondFire | FireKind::Howitzer
                )
        );
        if !kind_ok {
            return Err(RuleError::WrongPhase);
        }

        // §6.42: the Maxim Second Fire and Howitzer Subphase is restricted to
        // Maxim guns and Howitzer-class units -- no other weapon may fire here
        // even if the FireKind were miscategorised.  Named gunboats (§6.64)
        // carry howitzers even though their profile weapon is Artillery.
        let is_named_gunboat = matches!(
            unit.profile.identity,
            crate::UnitIdentity::AngloEgyptianGunboat(gb) if gb.has_howitzer()
        );
        if sub == FireSubPhase::MaximSecondAndHowitzer
            && !matches!(
                unit.profile.weapon,
                WeaponClass::Maxims | WeaponClass::Howitzer
            )
            && !is_named_gunboat
        {
            return Err(RuleError::WrongWeaponForSubphase(firer));
        }

        // Weapon class must permit the chosen kind.  Named gunboats may fire
        // howitzer despite carrying Artillery on their profile.
        match kind {
            FireKind::Howitzer
                if unit.profile.weapon != WeaponClass::Howitzer && !is_named_gunboat =>
            {
                return Err(RuleError::OnlyHowitzerMayFireHowitzer(firer));
            }
            FireKind::MaximSecondFire if unit.profile.weapon != WeaponClass::Maxims => {
                return Err(RuleError::OnlyMaximSecondFire(firer));
            }
            _ => {}
        }
        // §6.64: no howitzer fire at night.
        if kind == FireKind::Howitzer && self.day_night == DayNight::Night {
            return Err(RuleError::NoHowitzerAtNight);
        }

        if unit.state.disrupted {
            return Err(RuleError::Disrupted(firer));
        }
        // §5.3/§6.53: units constructing a zariba or committed to a
        // demolition "may neither fire offensively nor melee attack" that
        // turn (defensive fire stays legal).
        if matches!(self.phase, Phase::OffensiveFire(_))
            && (unit.state.constructing_zariba || unit.state.demolishing)
        {
            return Err(RuleError::BusyWithEngineering(firer));
        }
        if unit.profile.fire.is_none() {
            return Err(RuleError::NoFireFactor(firer));
        }
        if self.units_fired_this_phase.contains(&firer) {
            return Err(RuleError::AlreadyFired(firer));
        }

        // §6.61/§6.62: only artillery (or howitzer) may fire at a gunboat or
        // fort. Check it here so the app pre-blocks the shot rather than the
        // engine rejecting it after the fact.
        let target_units: Vec<UnitId> = self
            .player_units_in_hex(target_hex, unit.profile.identity.owner().opponent())
            .iter()
            .map(|u| u.id)
            .collect();
        if self.special_fire_target(&target_units).is_some()
            && !matches!(
                unit.profile.weapon,
                WeaponClass::Artillery | WeaponClass::Howitzer
            )
        {
            return Err(RuleError::ArtilleryOnlyVsGunboatOrFort(firer));
        }

        // §6.15: fire may only target *enemy-occupied* hexes. Without this
        // gate a click on a friendly or empty hex passes every other check
        // and resolves the CRT against whoever (if anyone) is there.
        if target_units.is_empty() {
            return Err(RuleError::FireTargetNotEnemyOccupied);
        }

        let range = HexDistance(unit.position.distance(target_hex) as u16);
        // Named gunboats (§6.64) carry Artillery on their profile but fire
        // howitzers in the second subphase; the howitzer CRT line applies.
        let effective_weapon = effective_fire_weapon(unit, kind);
        // §6.52/§9.343: the table this unit fires on (per firer, shared with
        // `resolve_fire_attack` so validation and resolution agree on range).
        let table_player = range_table_player_for(self.scenario, unit);
        // §8.1: at night, "all fire ranges are halved (round down, but range 1
        // stays range 1)." The correct interpretation (verified against the
        // rulebook's worked AE-rifle example: doubled@1, normal@2, out@3+) is
        // to halve the weapon's *maximum* range, then consult the day table at
        // the *physical* distance. Halving the distance and consulting the day
        // table at that reduced distance collapses too many bands.
        let effective_range = if self.day_night == DayNight::Night {
            match night_capped_distance(effective_weapon, table_player, range) {
                Some(capped) => capped, // consult day table at the physical distance
                None => {
                    return Err(RuleError::OutOfRangeAtNight {
                        firer: unit.position,
                        target: target_hex,
                    });
                }
            }
        } else {
            range
        };
        let band = range_band_for(
            self.scenario,
            table_player,
            effective_weapon,
            effective_range,
        );
        if !band.in_range() {
            return Err(RuleError::TargetOutOfRange {
                firer: unit.position,
                target: target_hex,
            });
        }

        // §6.21 / §6.3: line of sight. The engine derives LOS from
        // `self.board` (populated at game start from the board annotations)
        // so it can validate fire legality without app-side help. Howitzer
        // fire bypasses LOS (§6.64).
        //
        // The firer and target LOS levels are computed with notes (b) and
        // (c): gunboats → Rough, forts → Ground, walled-city-wall-adjacent
        // units → Rough. The "Units" blocker excludes gunboats and forts
        // (note a).
        let firer_los_level =
            crate::los_table::los_level_for_unit(unit.profile.kind, unit.position, &self.board);
        let target_los_level = self
            .units
            .iter()
            .find(|u| u.position == target_hex)
            .map(|u| crate::los_table::los_level_for_unit(u.profile.kind, u.position, &self.board))
            .unwrap_or_else(|| {
                self.board
                    .terrain_at(target_hex)
                    .map(crate::los_table::los_level)
                    .unwrap_or(crate::los_table::LosLevel::Ground)
            });
        if !crate::los_table::has_los(
            &self.board,
            unit.position,
            target_hex,
            kind,
            firer_los_level,
            target_los_level,
            self.los_unit_blocker(),
            |a, b| self.wall_is_breached(a, b),
        ) {
            return Err(RuleError::LineOfSightBlocked(unit.position, target_hex));
        }
        Ok(())
    }

    /// Read-only validation for §6.63 artillery-fire wall breaching. The firer
    /// must:
    ///   - exist,
    ///   - belong to the side whose turn it is to fire (active player on
    ///     offensive, opponent on defensive),
    ///   - be artillery- or howitzer-class (§6.63 "only artillery"),
    ///   - not be disrupted,
    ///   - have a printed fire factor,
    ///   - not have already fired this phase,
    ///   - be within range of the *nearer* endpoint of the wall hexside,
    ///     respecting the §8.1 night cap,
    ///   - have line of sight to the nearer endpoint.
    ///
    /// On success returns `(fire_factor, effective_range, nearer_endpoint)`.
    /// The caller is responsible for summing per-firer factors with the
    /// range band and resolving the CRT — this method only validates one
    /// firer at a time.
    pub fn can_fire_at_wall(
        &self,
        firer: UnitId,
        target: HexsideRef,
    ) -> Result<(FireFactor, HexDistance, HexCoord), RuleError> {
        let unit = self.unit_or_err(firer)?;

        let firing_player = self.fire_phase_player()?;
        if unit.profile.identity.owner() != firing_player {
            return Err(RuleError::NotYourTurn);
        }
        if !matches!(
            unit.profile.weapon,
            WeaponClass::Artillery | WeaponClass::Howitzer
        ) {
            return Err(RuleError::OnlyArtilleryMayBreachWall(firer));
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(firer));
        }
        let Some(fire_factor) = unit.profile.fire else {
            return Err(RuleError::NoFireFactor(firer));
        };
        if self.units_fired_this_phase.contains(&firer) {
            return Err(RuleError::AlreadyFired(firer));
        }

        // §6.63 range and LOS to a wall hexside, taken to whichever of its
        // two hexes the battery can see, nearest first (§6.3). The wall under
        // fire never hides its own face, and a battery standing on one of
        // the two hexes fires at it at range 1. (Measuring to the nearer
        // endpoint alone gave range 0 for the wall at the battery's feet,
        // and on a tie could pick the hex *behind* a neighbouring wall,
        // refusing the very rampart the battery faced.)
        let is_target = |a: HexCoord, b: HexCoord| {
            (a, b) == (target.a, target.b) || (a, b) == (target.b, target.a)
        };
        let firer_los =
            crate::los_table::los_level_for_unit(unit.profile.kind, unit.position, &self.board);
        let mut sides = [target.a, target.b];
        sides.sort_by_key(|h| unit.position.distance(*h));
        // Cheap bound first: a wall whose nearer hex is out of range stays
        // out of range whichever hex the LOS lands on -- skip the LOS sweeps.
        let nearest = HexDistance(unit.position.distance(sides[0]).max(1) as u16);
        let max_range = match self.day_night {
            DayNight::Night => crate::range_effects::night_max_range(
                unit.profile.weapon,
                firing_player == Player::AngloEgyptian,
            ) as u16,
            DayNight::Day => u16::MAX,
        };
        if nearest.value() > max_range {
            return Err(RuleError::OutOfRangeAtNight {
                firer: unit.position,
                target: sides[0],
            });
        }
        if !range_band_for(self.scenario, firing_player, unit.profile.weapon, nearest).in_range() {
            return Err(RuleError::TargetOutOfRange {
                firer: unit.position,
                target: sides[0],
            });
        }
        let seen = sides.into_iter().find_map(|hex| {
            if hex == unit.position {
                return Some((hex, 1));
            }
            let target_los = self
                .board
                .terrain_at(hex)
                .map(crate::los_table::los_level)
                .unwrap_or(crate::los_table::LosLevel::Ground);
            crate::los_table::has_los(
                &self.board,
                unit.position,
                hex,
                FireKind::Direct,
                firer_los,
                target_los,
                self.los_unit_blocker(),
                |a, b| self.wall_is_breached(a, b) || is_target(a, b),
            )
            .then(|| (hex, unit.position.distance(hex)))
        });
        let Some((nearer_hex, distance)) = seen else {
            return Err(RuleError::LineOfSightBlocked(unit.position, sides[0]));
        };
        let range = HexDistance(distance as u16);

        let effective_range = if self.day_night == DayNight::Night {
            let night_max = crate::range_effects::night_max_range(
                unit.profile.weapon,
                firing_player == Player::AngloEgyptian,
            );
            if range.value() > night_max as u16 {
                return Err(RuleError::OutOfRangeAtNight {
                    firer: unit.position,
                    target: nearer_hex,
                });
            }
            range
        } else {
            range
        };

        let band = range_band_for(
            self.scenario,
            firing_player,
            unit.profile.weapon,
            effective_range,
        );
        if !band.in_range() {
            return Err(RuleError::TargetOutOfRange {
                firer: unit.position,
                target: nearer_hex,
            });
        }
        Ok((fire_factor, effective_range, nearer_hex))
    }

    /// The hex a howitzer shell actually lands in given its scatter entry
    /// (§6.64). The printed Scattergram is a flower of six hexes around the
    /// designated target; this orients it relative to the firer: "upper"
    /// entries flank the away-from-firer direction (over-shoot), "lower"
    /// entries flank the toward-firer direction (fall-short), and left/right
    /// are the perpendicular sides. Each miss roll (1-6) thus lands on a
    /// distinct, deterministic neighbour; rolls 7-10 (`Center`) hit the
    /// designated hex.
    pub(crate) fn howitzer_impact_hex(
        &self,
        target: HexCoord,
        firer: Option<HexCoord>,
        scatter: ScatterHexDirection,
    ) -> HexCoord {
        use ScatterHexDirection as S;
        let neighbors = target.neighbors();
        // Bearing from target toward the firer (0 when unknown).
        let base = firer.map_or(0, |f| toward_index(target, f));
        let ring = |offset: usize| neighbors[(base + offset) % 6];
        // Upper half = the away-from-firer side of the flower (over-shoots),
        // lower half = the near side (fall-short), laterals in between. Each
        // of the six miss rolls lands on a distinct neighbour.
        match scatter {
            S::Center => target,
            S::UpperLeft => ring(2),
            S::UpperRight => ring(3),
            S::Right => ring(1),
            S::LowerRight => ring(0),
            S::LowerLeft => ring(5),
            S::Left => ring(4),
        }
    }

    /// If `target_ids` contains a gunboat or fort, return it, its kind, and
    /// the elimination threshold its destruction needs — `3` for a gunboat
    /// (§6.61) or `2` for a fort (§6.62) — rather than the generic Combat
    /// Results Table effect. Carrying the threshold out centrally means the
    /// caller never has to map an open `UnitKind` to a number (no
    /// `unreachable!` to drift if a third kind is ever added). A gunboat is
    /// reported in preference to a fort (a gunboat never stacks, so this is
    /// unambiguous in practice).
    pub(crate) fn special_fire_target(
        &self,
        target_ids: &[UnitId],
    ) -> Option<(UnitId, UnitKind, u8)> {
        let mut fort = None;
        for &id in target_ids {
            match self.find_unit(id).map(|u| u.profile.kind) {
                Some(UnitKind::Gunboat { .. }) => {
                    return Some((
                        id,
                        UnitKind::Gunboat {
                            fire: 0,
                            upstream: 0,
                            downstream: 0,
                        },
                        3,
                    ));
                }
                Some(UnitKind::Fort { .. }) if fort.is_none() => {
                    fort = Some((id, UnitKind::Fort { fire: 0, melee: 0 }, 2))
                }
                _ => {}
            }
        }
        fort
    }
}

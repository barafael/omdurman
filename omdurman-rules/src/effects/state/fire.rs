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
    /// non-gunboat, non-fort unit blocks LOS at that hex's terrain level --
    /// except entrenched units, which "may be fired 'over' in both
    /// directions" (§9.232). Shared by [`Self::can_fire_at`],
    /// [`Self::can_fire_at_wall`] and the Historical set-up's out-of-sight
    /// rule (§9.212).
    ///
    /// The blocking hexes are indexed once, when the closure is built: the
    /// LOS walk asks for *every intervening hex of every ray*, so a per-call
    /// scan of all units made one `has_los` query O(ray length × unit count).
    /// A sorted `Vec` + binary search keeps the index allocation-free to use
    /// and deterministic (no hasher), like the rest of the engine's lookups.
    pub fn los_unit_blocker(&self) -> impl Fn(HexCoord) -> Option<crate::los_table::LosLevel> + '_ {
        let mut blockers: Vec<HexCoord> = self
            .units
            .iter()
            .filter(|u| {
                !matches!(
                    u.profile.kind,
                    crate::UnitKind::Gunboat { .. } | crate::UnitKind::Fort { .. }
                )
            })
            .map(|u| u.position)
            .collect();
        blockers.sort_unstable();
        blockers.dedup();
        move |hex| {
            if blockers.binary_search(&hex).is_err() || self.is_zariba_entrenched(hex) {
                return None;
            }
            self.board.terrain_at(hex).map(crate::los_table::los_level)
        }
    }

    /// Read-only check of whether `firer` may fire `kind` at `target_hex` in
    /// the current state (§6): right fire sub-phase for the kind, right player,
    /// firer has a fire factor, weapon class permits the kind, not disrupted,
    /// hasn't already fired this phase, and the target is within (night-
    /// adjusted) range for the firer's weapon.
    ///
    /// Also checks line of sight on the attached board (§6.21/§6.3; howitzer
    /// fire ignores it, §6.64), and that a gunboat, or a fort with nobody
    /// inside, is fired at by artillery only (§6.61/§6.62).
    pub fn can_fire_at(
        &self,
        firer: UnitId,
        target_hex: HexCoord,
        kind: FireKind,
    ) -> Result<(), RuleError> {
        self.can_fire_mount(firer, target_hex, kind, FireMount::Main)
    }

    /// Read-only check of whether the named gunboat `gunboat` may fire its
    /// Maxim guns (§2.32: the "6×2" on the counter) at `target_hex` with an
    /// attack of `kind`: direct fire in the Direct Fire subphase, Maxim
    /// second fire in the Maxim Second Fire and Howitzer subphase (§6.42) --
    /// once in each, whatever its artillery does. The same checks as
    /// [`Self::can_fire_at`], on the Maxims line of the Range Effects Table.
    pub fn can_fire_gunboat_maxims_at(
        &self,
        gunboat: UnitId,
        target_hex: HexCoord,
        kind: FireKind,
    ) -> Result<(), RuleError> {
        self.can_fire_mount(gunboat, target_hex, kind, FireMount::GunboatMaxims)
    }

    /// The shared body of [`Self::can_fire_at`] and
    /// [`Self::can_fire_gunboat_maxims_at`]: `mount` is which of the unit's
    /// weapons fires.
    fn can_fire_mount(
        &self,
        firer: UnitId,
        target_hex: HexCoord,
        kind: FireKind,
        mount: FireMount,
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

        // §6.41/§6.42/§6.64: the weapon must suit the fire. The second
        // subphase belongs to the Maxims (Maxim batteries and named gunboats'
        // Maxims) and the howitzers (named gunboats' artillery, which fires
        // direct in the first subphase too).
        let weapon = unit.weapon_line(mount, kind);
        let howitzer = mount == FireMount::Main && unit.carries_howitzer();
        let maxims = unit.weapon_line(mount, FireKind::Direct) == WeaponClass::Maxims;
        if sub == FireSubPhase::MaximSecondAndHowitzer && !maxims && !howitzer {
            return Err(RuleError::WrongWeaponForSubphase(firer));
        }
        match kind {
            FireKind::Howitzer if !howitzer => {
                return Err(RuleError::OnlyHowitzerMayFireHowitzer(firer));
            }
            FireKind::MaximSecondFire if !maxims => {
                return Err(RuleError::OnlyMaximSecondFire(firer));
            }
            _ => {}
        }
        let factor = unit.fire_factor(mount);
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
        if factor.is_none() {
            return Err(RuleError::NoFireFactor(firer));
        }
        if self.has_fired(Shot { unit: firer, mount }) {
            return Err(RuleError::AlreadyFired(firer));
        }

        // §6.61/§6.62: only artillery (or howitzer) may fire at a gunboat or
        // at a fort itself; the units stacked inside a fort may be fired at
        // by anyone (§6.54, at the fort's −3). Check it here so the app
        // pre-blocks the shot rather than the engine rejecting it after the
        // fact. One pass over the target hex's occupants serves all three
        // §6 rules that read the same stack: this artillery-only rule, the
        // §6.15 enemy-occupied gate below, and the target's LOS level
        // (§6.3 notes b/c — the first unit standing there) further down.
        let opponent = unit.profile.identity.owner().opponent();
        let mut has_enemy = false;
        let mut only_artillery_targets = true;
        let mut first_kind = None;
        for u in &self.units {
            if u.position != target_hex {
                continue;
            }
            if first_kind.is_none() {
                first_kind = Some(u.profile.kind);
            }
            if u.profile.identity.owner() == opponent {
                has_enemy = true;
                only_artillery_targets &= matches!(
                    u.profile.kind,
                    UnitKind::Gunboat { .. } | UnitKind::Fort { .. }
                );
            }
        }
        if only_artillery_targets
            && has_enemy
            && !matches!(weapon, WeaponClass::Artillery | WeaponClass::Howitzer)
        {
            return Err(RuleError::ArtilleryOnlyVsGunboatOrFort(firer));
        }

        // §6.15: fire may only target *enemy-occupied* hexes. Without this
        // gate a click on a friendly or empty hex passes every other check
        // and resolves the CRT against whoever (if anyone) is there.
        if !has_enemy {
            return Err(RuleError::FireTargetNotEnemyOccupied);
        }

        let range = HexDistance(unit.position.distance(target_hex) as u16);
        // Named gunboats (§6.64) carry Artillery on their profile but fire
        // howitzers in the second subphase; the howitzer CRT line applies.
        // Their Maxims fire on the Maxims line.
        let effective_weapon = weapon;
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
        let (firer_los_level, target_los_level) =
            self.fire_los_levels(unit, target_hex, first_kind);
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

    /// The LOS levels of `firer` and of a target in `target_hex` (§6.3 notes
    /// b and c): `target_kind` is the first unit standing there, the bare
    /// terrain's level when the hex is empty.
    fn fire_los_levels(
        &self,
        firer: &UnitPlacement,
        target_hex: HexCoord,
        target_kind: Option<UnitKind>,
    ) -> (crate::los_table::LosLevel, crate::los_table::LosLevel) {
        let firer_level =
            crate::los_table::los_level_for_unit(firer.profile.kind, firer.position, &self.board);
        let target_level = target_kind
            .map(|kind| crate::los_table::los_level_for_unit(kind, target_hex, &self.board))
            .unwrap_or_else(|| {
                self.board
                    .terrain_at(target_hex)
                    .map(crate::los_table::los_level)
                    .unwrap_or(crate::los_table::LosLevel::Ground)
            });
        (firer_level, target_level)
    }

    /// The hexes `firer`'s line of sight enters `target_hex` out of (§6.3,
    /// [`los_entry_hexes`](crate::los_table::los_entry_hexes)): one hex,
    /// or two when the fire runs along hexsides and both sides are clear.
    /// Empty when the firer cannot see the hex.
    pub fn fire_entry_hexes(&self, firer: &UnitPlacement, target_hex: HexCoord) -> Vec<HexCoord> {
        let target_kind = self
            .units
            .iter()
            .find(|u| u.position == target_hex)
            .map(|u| u.profile.kind);
        let (firer_level, target_level) = self.fire_los_levels(firer, target_hex, target_kind);
        crate::los_table::los_entry_hexes(
            &self.board,
            firer.position,
            target_hex,
            firer_level,
            target_level,
            self.los_unit_blocker(),
            |a, b| self.wall_is_breached(a, b),
        )
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
        // §6.41/§6.42: batteries fire in the Direct Fire subphase; the
        // second subphase belongs to the Maxims and the howitzers.
        if !matches!(
            self.phase,
            Phase::OffensiveFire(FireSubPhase::DirectFire)
                | Phase::DefensiveFire(FireSubPhase::DirectFire)
        ) {
            return Err(RuleError::WrongPhase);
        }
        if !matches!(
            unit.profile.weapon,
            WeaponClass::Artillery | WeaponClass::Howitzer
        ) {
            return Err(RuleError::OnlyArtilleryMayBreachWall(firer));
        }
        // §6.22/§6.52/§9.343: the Range Effects Table this battery fires on.
        let table_player = range_table_player_for(self.scenario, unit);
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
                table_player == Player::AngloEgyptian,
            ) as u16,
            DayNight::Day => u16::MAX,
        };
        if nearest.value() > max_range {
            return Err(RuleError::OutOfRangeAtNight {
                firer: unit.position,
                target: sides[0],
            });
        }
        if !range_band_for(self.scenario, table_player, unit.profile.weapon, nearest).in_range() {
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

        if range.value() > max_range {
            return Err(RuleError::OutOfRangeAtNight {
                firer: unit.position,
                target: nearer_hex,
            });
        }
        let effective_range = range;

        let band = range_band_for(
            self.scenario,
            table_player,
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

    /// Read-only check of whether `firer` may fire at the river chain
    /// (§10.23 b: "firing at the chain with artillery"): British artillery
    /// of the player whose fire phase it is, in the Direct Fire subphase,
    /// undisrupted and not yet fired, in range and in sight of a hex of the
    /// unsunk chain -- the nearest such hex. Returns the firer's factor, its
    /// range and that hex.
    pub fn can_fire_at_chain(
        &self,
        firer: UnitId,
    ) -> Result<(FireFactor, HexDistance, HexCoord), RuleError> {
        let unit = self.unit_or_err(firer)?;
        let firing_player = self.fire_phase_player()?;
        if unit.profile.identity.owner() != firing_player || firing_player != Player::AngloEgyptian
        {
            return Err(RuleError::NotYourTurn);
        }
        if !matches!(
            self.phase,
            Phase::OffensiveFire(FireSubPhase::DirectFire)
                | Phase::DefensiveFire(FireSubPhase::DirectFire)
        ) {
            return Err(RuleError::WrongPhase);
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
        let Some(chain) = self.chain.as_ref().filter(|c| !c.sunk) else {
            return Err(RuleError::NoChainPlaced);
        };
        let table_player = range_table_player_for(self.scenario, unit);
        let firer_los =
            crate::los_table::los_level_for_unit(unit.profile.kind, unit.position, &self.board);
        let mut hexes = chain.hexes.clone();
        hexes.sort_by_key(|h| unit.position.distance(*h));
        let mut refusal = RuleError::NoChainPlaced;
        for hex in hexes {
            let range = HexDistance(unit.position.distance(hex).max(1) as u16);
            if self.day_night == DayNight::Night
                && night_capped_distance(unit.profile.weapon, table_player, range).is_none()
            {
                refusal = RuleError::OutOfRangeAtNight {
                    firer: unit.position,
                    target: hex,
                };
                continue;
            }
            if !range_band_for(self.scenario, table_player, unit.profile.weapon, range).in_range() {
                refusal = RuleError::TargetOutOfRange {
                    firer: unit.position,
                    target: hex,
                };
                continue;
            }
            let seen = crate::los_table::has_los(
                &self.board,
                unit.position,
                hex,
                FireKind::Direct,
                firer_los,
                crate::los_table::LosLevel::Ground,
                self.los_unit_blocker(),
                |a, b| self.wall_is_breached(a, b),
            );
            if !seen {
                refusal = RuleError::LineOfSightBlocked(unit.position, hex);
                continue;
            }
            return Ok((fire_factor, range, hex));
        }
        Err(refusal)
    }

    /// Whether `shot`'s weapon has fired this fire subphase (§6.14: each
    /// weapon fires once a subphase; a named gunboat's Maxims and its
    /// artillery are tracked apart, §6.42).
    pub fn has_fired(&self, shot: Shot) -> bool {
        match shot.mount {
            FireMount::Main => self.units_fired_this_phase.contains(&shot.unit),
            FireMount::GunboatMaxims => self.gunboat_maxims_fired_this_phase.contains(&shot.unit),
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

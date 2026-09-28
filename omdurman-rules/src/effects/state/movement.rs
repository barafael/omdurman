//! Movement validators (rulebook §5), including [`GameState::validate_move`]
//! and the Friendlies transport offer (§5.21).

use super::*;

/// Longest `MoveUnit` path the engine accepts: well beyond any allowance
/// (the largest is 18, §5.11/§5.24), so it only bounds hostile input.
pub const MAX_MOVE_PATH_LEN: usize = 64;

/// An engine-validated move (see [`GameState::validate_move`]): the
/// movement points it costs, computed from the board (§5.11/§5.24), and
/// whether a gunboat took an upstream step (§5.24's sticky cap).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovePlan {
    pub cost: MovementPoints,
    pub went_upstream: bool,
}

impl GameState {
    /// Read-only check of whether `unit_id` may move `cost` movement points in
    /// the current state (§5): right phase, right player, not disrupted, not
    /// stopped in an enemy ZOC, land-mobile, within (night-adjusted) allowance.
    /// Returns the same `RuleError` the `MoveUnit` effect would on rejection.
    /// Lets the UI gate input without mutating or duplicating the rules.
    pub fn can_move_unit(&self, unit_id: UnitId, cost: MovementPoints) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        self.movement_preconditions(unit)?;
        let allowance = self.land_allowance(unit)?;
        self.check_move_allowance(unit_id, i32::from(cost.value()), allowance, false)
    }

    /// As [`can_move_unit`](Self::can_move_unit), but when `to` is supplied the
    /// move is validated hex by hex along the straight line from the unit's
    /// current hex to `to` (see [`validate_move`](Self::validate_move)): every
    /// step adjacent, passable and on the board, no pass-through of an enemy
    /// ZOC (§5.26, §5.43 -- the destination itself may be a ZOC hex; a unit
    /// that *begins* in an enemy ZOC may still move out), and the
    /// engine-computed terrain cost (or the caller's `cost`, whichever is
    /// larger) within the allowance. Land units only (gunboats:
    /// [`can_move_gunboat`](Self::can_move_gunboat)).
    ///
    /// Stacking (§5.51) is checked when the move is applied, not here.
    pub fn can_move_unit_to(
        &self,
        unit_id: UnitId,
        to: Option<HexCoord>,
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        let Some(to) = to else {
            return self.can_move_unit(unit_id, cost);
        };
        check_coord(to)?;
        let unit = self.unit_or_err(unit_id)?;
        self.movement_preconditions(unit)?;
        let allowance = self.land_allowance(unit)?;
        let mut path = unit.position.line_between(to);
        path.push(to);
        let plan = self.validate_move(unit_id, to, &path)?;
        let cost = i32::from(plan.cost.value()).max(i32::from(cost.value()));
        self.check_move_allowance(unit_id, cost, allowance, false)
    }

    /// As [`can_move_unit_to`](Self::can_move_unit_to), but along the *actual*
    /// stepped `path` (the entered hexes, excluding the start, ending at `to`)
    /// -- a bent path that avoids ZOC hexes is legal even when the straight
    /// line would cross one. Exactly the `MoveUnit` validation
    /// ([`validate_move`](Self::validate_move)); the engine computes the cost,
    /// and a larger caller `cost` must also fit the allowance.
    pub fn can_move_unit_along(
        &self,
        unit_id: UnitId,
        to: HexCoord,
        path: &[HexCoord],
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        let plan = self.validate_move(unit_id, to, path)?;
        self.check_caller_cost(unit_id, &plan, cost)
    }

    /// For the UI/bot predicates: a caller may ask whether it can afford a
    /// move at a (higher) cost it computed itself -- the engine's own cost
    /// was already checked by [`validate_move`](Self::validate_move), so the
    /// larger of the two must fit the allowance (the `MoveUnit` effect
    /// itself ignores the caller's cost).
    fn check_caller_cost(
        &self,
        unit_id: UnitId,
        plan: &MovePlan,
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        if cost.value() <= plan.cost.value() {
            return Ok(());
        }
        let unit = self.unit_or_err(unit_id)?;
        match unit.profile.movement {
            crate::UnitMovement::Gunboat(printed) => {
                let ga = self.gunboat_allowances(unit).unwrap_or(printed);
                let capped =
                    plan.went_upstream || self.gunboats_upstream_this_turn.contains(&unit_id);
                let allowance = if capped { ga.upstream } else { ga.downstream };
                self.check_move_allowance(unit_id, i32::from(cost.value()), allowance, capped)
            }
            _ => {
                let allowance = self.land_allowance(unit)?;
                self.check_move_allowance(unit_id, i32::from(cost.value()), allowance, false)
            }
        }
    }

    /// The shared movement preconditions (§5): Movement phase, the active
    /// player's unit (§5.1), not disrupted, not already stopped in an enemy
    /// ZOC this turn (§5.26/§5.43), and not GORDON in FALL OF KHARTOUM
    /// (§9.346).
    fn movement_preconditions(&self, unit: &UnitPlacement) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        // §5.1: only the active player's units move during their player turn.
        // Fire (§6.41), melee (§7.1) and reinforcements (§9.112/§9.113) all
        // compare the actor to `active_player` the same way.
        if unit.profile.identity.owner() != self.active_player {
            return Err(RuleError::NotYourTurn);
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit.id));
        }
        // §10.12: a gunboat that has lost its engines only drifts; one that
        // struck a mine was ordered to stop for the turn.
        if unit.state.engines_lost {
            return Err(RuleError::EnginesLost(unit.id));
        }
        if self.gunboats_stopped_this_turn.contains(&unit.id) {
            return Err(RuleError::StruckMine(unit.id));
        }
        // §5.21: a "Friendlies" unit aboard a gunboat moves only with it.
        if unit.state.loaded_on.is_some() {
            return Err(RuleError::LoadedOnGunboat(unit.id));
        }
        // §5.21: the gunboat carries its passenger on the turn after
        // loading ("during the Anglo-Egyptian player's next turn").
        if let Some(TransportState::Loaded { gunboat, since, .. }) = self.friendlies_transport
            && gunboat == unit.id
            && since == self.current_turn
        {
            return Err(RuleError::TransportLoadingTurn(unit.id));
        }
        // §5.3: Zariba builders stay put until the turn ends; §6.53: the
        // Royal Engineers "end their movement adjacent" to their target.
        if unit.state.constructing_zariba || unit.state.demolishing {
            return Err(RuleError::BusyWithEngineering(unit.id));
        }
        // §5.26/§5.43: a unit that entered an enemy ZOC this movement phase
        // "may move no further that turn" (it may withdraw next phase).
        if self.zoc_stopped_this_turn.contains(&unit.id) {
            return Err(RuleError::StoppedInEnemyZoc(unit.id));
        }
        // §9.346: the GORDON leader unit may not move during FALL OF KHARTOUM.
        if self.scenario == Scenario::FallOfKhartoum && unit.profile.identity.is_gordon() {
            return Err(RuleError::GordonMayNotMove);
        }
        Ok(())
    }

    /// A gunboat's upstream and downstream allowances (§5.24), halved at
    /// night for the Anglo-Egyptians like every other of their movement
    /// allowances (§8.1: "all Anglo-Egyptian movement allowances are
    /// halved"). `None` for anything but a gunboat.
    pub fn gunboat_allowances(&self, unit: &UnitPlacement) -> Option<crate::GunboatMovement> {
        let crate::UnitMovement::Gunboat(printed) = unit.profile.movement else {
            return None;
        };
        let owner = unit.profile.identity.owner();
        let at_night = |a| crate::effective_movement_at_night(a, owner, self.day_night);
        Some(crate::GunboatMovement {
            upstream: at_night(printed.upstream),
            downstream: at_night(printed.downstream),
        })
    }

    /// The allowance `unit_id` moves under right now (§5.11): its land
    /// allowance, or a gunboat's downstream allowance -- the upstream one
    /// once it has moved upstream this turn (§5.24) -- all halved for the
    /// Anglo-Egyptians at night (§8.1). `None` for an immobile unit (§5.25)
    /// or one not on the board.
    pub fn current_allowance(&self, unit_id: UnitId) -> Option<MovementAllowance> {
        let unit = self.find_unit(unit_id)?;
        match unit.profile.movement {
            crate::UnitMovement::Land(_) => self.land_allowance(unit).ok(),
            crate::UnitMovement::Gunboat(_) => {
                let ga = self.gunboat_allowances(unit)?;
                Some(if self.gunboats_upstream_this_turn.contains(&unit_id) {
                    ga.upstream
                } else {
                    ga.downstream
                })
            }
            crate::UnitMovement::Immobile => None,
        }
    }

    /// The movement points `unit_id` has left this turn (§5.12/§5.13): its
    /// [current allowance](Self::current_allowance) less what it has spent.
    /// Zero for a unit that may not move now at all -- disrupted, stopped
    /// in an enemy ZOC (§5.43), GORDON (§9.346), out of phase or turn.
    pub fn remaining_movement(&self, unit_id: UnitId) -> i16 {
        let Some(unit) = self.find_unit(unit_id) else {
            return 0;
        };
        if self.movement_preconditions(unit).is_err() {
            return 0;
        }
        self.current_allowance(unit_id)
            .map_or(0, |a| (a.value() as i16 - self.mp_spent(unit_id)).max(0))
    }

    /// The (night-adjusted, §8.1) land movement allowance of `unit`;
    /// `NotMobile` for gunboats and immobile units.
    fn land_allowance(&self, unit: &UnitPlacement) -> Result<MovementAllowance, RuleError> {
        match unit.profile.movement {
            crate::UnitMovement::Land(a) => Ok(crate::effective_movement_at_night(
                a,
                unit.profile.identity.owner(),
                self.day_night,
            )),
            crate::UnitMovement::Gunboat(_) | crate::UnitMovement::Immobile => {
                Err(RuleError::NotMobile(unit.id))
            }
        }
    }

    /// §5.11/§5.12: a unit moves hex by hex up to its allowance; the *running
    /// total* spent this turn plus `cost` must fit `allowance`. Widened
    /// arithmetic, so no caller-supplied value can overflow it.
    fn check_move_allowance(
        &self,
        unit_id: UnitId,
        cost: i32,
        allowance: MovementAllowance,
        upstream_cap: bool,
    ) -> Result<(), RuleError> {
        let total = i32::from(self.mp_spent(unit_id)).saturating_add(cost);
        if total > i32::from(allowance.value()) {
            let cost = MovementPoints(i16::try_from(total).unwrap_or(i16::MAX));
            return Err(if upstream_cap {
                RuleError::GunboatUpstreamCap { cost, allowance }
            } else {
                RuleError::MovementExceedsAllowance { cost, allowance }
            });
        }
        Ok(())
    }

    /// Validate a `MoveUnit` (§5) step by step and return its engine-computed
    /// plan. `path` is the ordered hexes *entered* (excluding the start,
    /// ending at `to`); an empty path means the single step to `to`. The
    /// caller never supplies the cost: the engine is authoritative for it.
    ///
    /// Shared by land and gunboat moves. Every step must be a single hex
    /// (§5.11) that is not enemy-occupied (§7.1; lone Anglo-Egyptian leaders
    /// are overrun instead, §6.51) and not an enemy fort (§6.54); no hex
    /// before the destination may lie in an enemy ZOC (§5.26/§5.43). Land
    /// steps must stay on the board, off the Nile (§5.22), not cross a wall
    /// (§5.23; gates and breaches pass) and respect the walled-city entry
    /// restriction (§5.23); each costs its Terrain Effects Chart value
    /// (§5.11, road overlay) plus the §9.233 zariba-end surcharge. Gunboat
    /// steps stay on the Nile (§5.22), stop at the chain (§10.22) and cost
    /// one MP each, capped by the upstream/downstream allowance (§5.24); the
    /// FALL OF KHARTOUM mouth crossing (§9.345) is a single flat-6 "step".
    /// The running total spent this turn plus the path's cost must fit the
    /// allowance. Stacking (§5.51) is checked at the destination on apply.
    pub fn validate_move(
        &self,
        unit_id: UnitId,
        to: HexCoord,
        path: &[HexCoord],
    ) -> Result<MovePlan, RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        check_coord(to)?;
        for hex in path {
            check_coord(*hex)?;
        }
        if path.len() > MAX_MOVE_PATH_LEN {
            return Err(RuleError::PathTooLong(path.len()));
        }
        let steps: &[HexCoord] = if path.is_empty() {
            std::slice::from_ref(&to)
        } else {
            path
        };
        if steps.last() != Some(&to) {
            return Err(RuleError::PathEndMismatch(to));
        }
        match unit.profile.movement {
            // §5.25: forts may never move once placed.
            crate::UnitMovement::Immobile => Err(RuleError::AlreadyPlaced(unit_id)),
            crate::UnitMovement::Gunboat(printed) => {
                self.movement_preconditions(unit)?;
                let ga = self.gunboat_allowances(unit).unwrap_or(printed);
                self.plan_gunboat_move(unit, ga, to, steps)
            }
            crate::UnitMovement::Land(_) => {
                self.movement_preconditions(unit)?;
                let allowance = self.land_allowance(unit)?;
                self.plan_land_move(unit, allowance, steps)
            }
        }
    }

    /// Per-step checks shared by land and gunboat moves: the step is a
    /// single hex (§5.11), the entered hex is not an enemy fort (§6.54) and
    /// not enemy-occupied (§7.1 with §5.26 -- movement may only bring a unit
    /// adjacent; engaging is melee's job). Lone Anglo-Egyptian leaders do not
    /// block: §6.51 eliminates them when a Dervish unit occupies or passes
    /// through their hex (the overrun in `apply_move_unit`).
    fn check_move_step(
        &self,
        mover: &UnitPlacement,
        from: HexCoord,
        to: HexCoord,
    ) -> Result<(), RuleError> {
        if !from.is_adjacent_to(to) {
            return Err(RuleError::PathNotContiguous { from, to });
        }
        let owner = mover.profile.identity.owner();
        if self.hex_has_enemy_fort(to, owner) {
            return Err(RuleError::EnemyFort(to));
        }
        let enemy = owner.opponent();
        if self.units.iter().any(|u| {
            u.position == to
                && u.profile.identity.owner() == enemy
                && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
        }) {
            return Err(RuleError::EnemyOccupied(to));
        }
        Ok(())
    }

    /// Whether `unit` may step on land from `from` to the adjacent `to`,
    /// terrain and occupants aside from ZOC and cost: not into the Nile
    /// (§5.22), not off the board, not into an enemy fort or enemy-held hex
    /// ([`Self::check_move_step`]), not across a closed hexside (§5.23 wall,
    /// §9.233 Zariba) and not into the walled city by a unit barred from it
    /// (§5.23). The per-step core of a land move, shared with the §7.5
    /// retreat and the app's route planning.
    pub fn check_land_step(
        &self,
        unit: &UnitPlacement,
        from: HexCoord,
        to: HexCoord,
    ) -> Result<(), RuleError> {
        // §5.22: land units may never enter a Nile hex.
        if self.board.is_nile(to) {
            return Err(RuleError::LandIntoNile(to));
        }
        // A unit may never step off the board (with no board loaded, map
        // constraints don't apply).
        if !self.board.terrain.is_empty() && self.board.terrain_at(to).is_none() {
            return Err(RuleError::OffBoard(to));
        }
        self.check_move_step(unit, from, to)?;
        // §5.23: a wall hexside blocks movement (gates and breaches pass).
        // Read through `hexside_effective` so a §6.63 breach is an opening.
        if self.hexside_effective_is(from, to, HexsideKind::blocks_movement) {
            return Err(RuleError::MoveBlockedByHexside(from, to));
        }
        self.check_walled_city_entry(unit, from, to)
    }

    /// §5.23: "Only certain units may enter the walled portion of Omdurman"
    /// -- Dervish: the Khalifa, the artillery, and the Taiasha bodyguard;
    /// Anglo-Egyptian: any unit except gunboats and "Friendlies". Refuses a
    /// step `from` -> `to` that brings a barred unit into it, whether by
    /// movement, retreat or advance after combat. Scoped to the Omdurman
    /// map: FALL OF KHARTOUM is a different walled city (Khartoum) whose
    /// set-up places units inside it freely (§9.32).
    pub fn check_walled_city_entry(
        &self,
        unit: &UnitPlacement,
        from: HexCoord,
        to: HexCoord,
    ) -> Result<(), RuleError> {
        if self.scenario != Scenario::FallOfKhartoum
            && self.board.is_walled_city(to)
            && !self.board.is_walled_city(from)
            && !unit.profile.identity.may_enter_walled_city()
        {
            return Err(RuleError::WalledCityEntry(unit.id, to));
        }
        Ok(())
    }

    /// The land half of [`validate_move`](Self::validate_move).
    fn plan_land_move(
        &self,
        unit: &UnitPlacement,
        allowance: MovementAllowance,
        steps: &[HexCoord],
    ) -> Result<MovePlan, RuleError> {
        let owner = unit.profile.identity.owner();
        let kind = unit.profile.kind;
        let last = steps.len() - 1;
        let mut cost: i32 = 0;
        let mut prev = unit.position;
        for (i, &next) in steps.iter().enumerate() {
            if !prev.is_adjacent_to(next) {
                return Err(RuleError::PathNotContiguous {
                    from: prev,
                    to: next,
                });
            }
            self.check_land_step(unit, prev, next)?;
            // §5.26/§5.43: a unit must stop the instant it enters an enemy
            // ZOC, so no hex entered before the destination may lie in one.
            if i < last && self.hex_in_enemy_zoc(next, owner, kind) {
                return Err(RuleError::BlockedByEnemyZoc(next));
            }
            cost = cost.saturating_add(self.land_step_cost(prev, next));
            prev = next;
        }
        self.check_move_allowance(unit.id, cost, allowance, false)?;
        Ok(MovePlan {
            cost: MovementPoints(i16::try_from(cost).unwrap_or(i16::MAX)),
            went_upstream: false,
        })
    }

    /// Movement points to step `from` -> `to` on land (§5.11, Terrain Effects
    /// Chart): the entered terrain (1 along a road link) plus the crossed
    /// hexside, read through [`Self::hexside_effective`] so a breach costs
    /// like a gate. Closed steps are refused before this is asked; they are
    /// priced out of reach here as a backstop.
    pub fn land_step_cost(&self, from: HexCoord, to: HexCoord) -> i32 {
        let terrain = self
            .board
            .terrain_at(to)
            .unwrap_or(omdurman_types::Terrain::Clear {
                road: Default::default(),
            });
        crate::terrain_chart::land_step_cost(
            terrain,
            self.board.road_links(from, to),
            self.hexside_effective(from, to),
        )
        .map_or(i32::from(i16::MAX), i32::from)
    }

    /// The gunboat half of [`validate_move`](Self::validate_move) (§5.22,
    /// §5.24, §9.345, §10.22).
    fn plan_gunboat_move(
        &self,
        unit: &UnitPlacement,
        ga: crate::GunboatMovement,
        to: HexCoord,
        steps: &[HexCoord],
    ) -> Result<MovePlan, RuleError> {
        // §9.345 (FALL OF KHARTOUM): a British gunboat may cross between the
        // White and Blue Nile mouths off-board for a flat 6 "upstream" MP,
        // bypassing the normal contiguous-Nile path. Only the two named mouth
        // hexes participate; the move is otherwise a normal move (and counts
        // against the upstream allowance, §5.24).
        if self.scenario == Scenario::FallOfKhartoum
            && steps.len() == 1
            && self.is_nile_mouth_crossing(unit.position, to)
        {
            const CROSS_NILE_MP: i32 = 6;
            let owner = unit.profile.identity.owner();
            if self.hex_has_enemy_fort(to, owner)
                || self
                    .units
                    .iter()
                    .any(|u| u.position == to && u.profile.identity.owner() == owner.opponent())
            {
                return Err(RuleError::EnemyOccupied(to));
            }
            self.check_move_allowance(unit.id, CROSS_NILE_MP, ga.upstream, true)?;
            return Ok(MovePlan {
                cost: MovementPoints(CROSS_NILE_MP as i16),
                went_upstream: true,
            });
        }

        let owner = unit.profile.identity.owner();
        let kind = unit.profile.kind;
        let last = steps.len() - 1;
        let mut moved_upstream = false;
        let mut prev = unit.position;
        for (i, &next) in steps.iter().enumerate() {
            if !prev.is_adjacent_to(next) {
                return Err(RuleError::PathNotContiguous {
                    from: prev,
                    to: next,
                });
            }
            // §5.22: gunboats stay on the Nile. With a board loaded, every
            // entered hex must be a Nile hex.
            if !self.board.terrain.is_empty() && !self.board.is_nile(next) {
                return Err(RuleError::GunboatOffNile(next));
            }
            // §10.22: a chained Nile hex stops the gunboat.
            if self.chain_covers(next) {
                return Err(RuleError::BlockedByChain(next));
            }
            self.check_move_step(unit, prev, next)?;
            // §5.41/§5.43: a gunboat stops on entering an enemy *gunboat's*
            // ZOC (which `hex_in_enemy_zoc` encodes for a gunboat mover).
            if i < last && self.hex_in_enemy_zoc(next, owner, kind) {
                return Err(RuleError::BlockedByEnemyZoc(next));
            }
            if self.board.step_direction(prev, next) == Some(crate::board::StepDirection::Upstream)
            {
                moved_upstream = true;
            }
            prev = next;
        }

        // §5.24: any upstream step caps the whole turn at the upstream
        // allowance; otherwise the downstream allowance applies. The cap is
        // *sticky*: an upstream hex taken in an earlier move of the same turn
        // still caps this (all-downstream) move -- "if they move even one hex
        // upstream, their upstream movement allowance is their maximum
        // movement allowance for that turn". Gunboats pay one MP per hex.
        let capped = moved_upstream || self.gunboats_upstream_this_turn.contains(&unit.id);
        let allowance = if capped { ga.upstream } else { ga.downstream };
        // `steps` is bounded by `MAX_MOVE_PATH_LEN`, so the length fits.
        let cost = steps.len() as i32;
        self.check_move_allowance(unit.id, cost, allowance, capped)?;
        Ok(MovePlan {
            cost: MovementPoints(cost as i16),
            went_upstream: moved_upstream,
        })
    }

    /// The true movement-point cost of a move along `path` (the entered hexes,
    /// excluding the start), computed from the board's Terrain Effects Chart
    /// (§5.11). Returns `None` when no board/path is available. Land units pay
    /// each hex's terrain cost (plus the §9.233 zariba-end surcharge);
    /// gunboats pay one MP per Nile hex entered (§5.24 counts hexes, not
    /// terrain). Per-hex passability is enforced separately by
    /// [`validate_move`](Self::validate_move), so an off-map hex here
    /// contributes the clear-terrain base of 1.
    ///
    /// §5.42: entering or leaving an enemy ZOC adds no MP cost.
    pub fn movement_cost_for(
        &self,
        unit: &UnitPlacement,
        path: &[HexCoord],
    ) -> Option<MovementPoints> {
        if path.is_empty() || self.board.terrain.is_empty() {
            return None;
        }
        let total: i32 = match unit.profile.movement {
            crate::UnitMovement::Gunboat(_) => i32::try_from(path.len()).unwrap_or(i32::MAX),
            _ => {
                let mut sum = 0i32;
                let mut prev = unit.position;
                for hex in path {
                    sum = sum.saturating_add(self.land_step_cost(prev, *hex));
                    prev = *hex;
                }
                sum
            }
        };
        Some(MovementPoints(i16::try_from(total).unwrap_or(i16::MAX)))
    }

    /// Validate a gunboat move along `path` (§5.22, §5.24, §10.22): exactly
    /// the `MoveUnit` validation ([`validate_move`](Self::validate_move)) for
    /// a gunboat. Gunboats may move only along Nile hexes; their two
    /// allowances are upstream (smaller) and downstream (larger); and "if they
    /// move even one hex upstream, their upstream movement allowance is their
    /// maximum for that turn." Chained Nile hexes stop the gunboat (§10.22).
    /// The engine computes the cost (one MP per hex, a flat 6 for the §9.345
    /// mouth crossing); a larger caller `cost` must also fit.
    pub fn can_move_gunboat(
        &self,
        unit_id: UnitId,
        to: HexCoord,
        path: &[HexCoord],
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if !matches!(unit.profile.movement, crate::UnitMovement::Gunboat(_)) {
            self.movement_preconditions(unit)?;
            return Err(RuleError::NotAGunboat(unit_id));
        }
        let plan = self.validate_move(unit_id, to, path)?;
        self.check_caller_cost(unit_id, &plan, cost)
    }

    /// Read-only check of a §5.21 Friendlies-transport action.
    ///
    /// * `Load` -- Anglo-Egyptian Movement phase, the Isa Zachneih already
    ///   eliminated ("after, and only after"), no other transport under way;
    ///   an undisrupted "Friendlies" unit and an Anglo-Egyptian gunboat that
    ///   "start their turn adjacent" (neither has moved yet), the gunboat
    ///   otherwise alone.
    /// * `Disembark` -- the unit's own gunboat, Movement phase of turn N+2 or
    ///   later, onto an adjacent west-bank land hex the unit could step to
    ///   and afford.
    pub fn can_friendlies_transport(&self, action: FriendliesAction) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        if self.active_player != Player::AngloEgyptian {
            return Err(RuleError::NotYourTurn);
        }
        match action {
            FriendliesAction::Load { unit, gunboat } => {
                if !self.isa_zachneih_eliminated {
                    return Err(RuleError::FriendliesIsaZachneihAlive);
                }
                if self.friendlies_transport.is_some() {
                    return Err(RuleError::FriendliesTransportInProgress);
                }
                let u = self.unit_or_err(unit)?;
                let g = self.unit_or_err(gunboat)?;
                if !u.profile.identity.is_friendlies()
                    || !matches!(g.profile.kind, UnitKind::Gunboat { .. })
                    || g.profile.identity.owner() != u.profile.identity.owner()
                {
                    return Err(RuleError::FriendliesWrongUnits);
                }
                if u.state.disrupted {
                    return Err(RuleError::Disrupted(unit));
                }
                if u.state.constructing_zariba || u.state.demolishing {
                    return Err(RuleError::BusyWithEngineering(unit));
                }
                if !u.position.is_adjacent_to(g.position) {
                    return Err(RuleError::FriendliesNotAdjacentToGunboat);
                }
                if self.mp_spent(unit) > 0 || self.mp_spent(gunboat) > 0 {
                    return Err(RuleError::FriendliesMustStartTurnAdjacent);
                }
                if self
                    .units
                    .iter()
                    .any(|o| o.position == g.position && o.id != gunboat)
                {
                    return Err(RuleError::Stacking(crate::StackingError::GunboatStack));
                }
                Ok(())
            }
            FriendliesAction::Disembark { unit, gunboat, to } => {
                let Some(TransportState::Loaded {
                    unit: aboard,
                    gunboat: carrier,
                    since,
                }) = self.friendlies_transport
                else {
                    return Err(RuleError::FriendliesNotLoaded);
                };
                if (aboard, carrier) != (unit, gunboat) {
                    return Err(RuleError::FriendliesNotLoaded);
                }
                let earliest = since.value().saturating_add(2);
                if self.current_turn.value() < earliest {
                    return Err(RuleError::FriendliesDisembarkTooEarly(earliest));
                }
                let u = self.unit_or_err(unit)?;
                if u.state.disrupted {
                    return Err(RuleError::Disrupted(unit));
                }
                if !u.position.is_adjacent_to(to)
                    || self.board.bank_of(to) != Some(crate::board::NileBank::West)
                {
                    return Err(RuleError::FriendliesDisembarkHex(to));
                }
                self.check_land_step(u, u.position, to)?;
                self.check_stacking(u, to)?;
                let allowance = self.land_allowance(u)?;
                self.check_move_allowance(
                    unit,
                    self.land_step_cost(u.position, to),
                    allowance,
                    false,
                )
            }
        }
    }

    /// The Friendlies-transport actions the rules would accept now (§5.21),
    /// given the locally selected unit: loading the selected "Friendlies"
    /// unit onto an adjacent gunboat, or -- once the mission has reached its
    /// third turn -- disembarking onto each legal west-bank hex. Pairs with
    /// [`GameEffect::FriendliesTransport`] so the UI offers exactly what the
    /// engine accepts.
    pub fn friendlies_transport_offers(&self, selected: Option<UnitId>) -> Vec<FriendliesAction> {
        let candidates: Vec<FriendliesAction> = match self.friendlies_transport {
            None => {
                let Some(unit) = selected.and_then(|id| self.find_unit(id)) else {
                    return Vec::new();
                };
                self.units
                    .iter()
                    .filter(|g| g.position.is_adjacent_to(unit.position))
                    .map(|g| FriendliesAction::Load {
                        unit: unit.id,
                        gunboat: g.id,
                    })
                    .collect()
            }
            Some(TransportState::Loaded { unit, gunboat, .. }) => self
                .find_unit(gunboat)
                .map(|g| g.position.neighbors().to_vec())
                .unwrap_or_default()
                .into_iter()
                .map(|to| FriendliesAction::Disembark { unit, gunboat, to })
                .collect(),
        };
        candidates
            .into_iter()
            .filter(|a| self.can_friendlies_transport(*a).is_ok())
            .collect()
    }
}

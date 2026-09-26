use super::*;

/// Validate and apply a [`GameEffect`] to `state` (rulebook §4, §5, §6, §7, §8, §10).
///
/// Returns `Ok(())` on success; the state has been mutated.  Returns
/// `Err(RuleError)` if the effect is illegal for the current state; the
/// state is left unchanged.
pub fn apply_effect(state: &mut GameState, effect: &GameEffect) -> Result<(), RuleError> {
    if state.game_over {
        return Err(RuleError::GameOver);
    }
    // Network-supplied coordinates are bounded before any hex arithmetic
    // (distance/neighbour computations would otherwise overflow on extreme
    // values -- a panic on every peer instead of a clean rejection).
    check_effect_coords(effect)?;
    let result = match effect {
        GameEffect::AdvancePhase => advance_phase(state),
        GameEffect::MoveUnit {
            unit_id,
            to,
            cost,
            path,
        } => apply_move_unit(state, *unit_id, *to, *cost, path),
        GameEffect::FireCombat { attack, roll } => apply_fire_combat(state, attack, *roll),
        GameEffect::HowitzerFire {
            attack,
            combat_results_table_roll,
            impact_roll,
        } => apply_howitzer_fire(state, attack, *combat_results_table_roll, *impact_roll),
        GameEffect::MeleeCombat {
            attack,
            attacker_roll,
            defender_roll,
        } => apply_melee_combat(state, attack, *attacker_roll, *defender_roll),
        GameEffect::DeclareMelee {
            attack,
            attacker_roll,
            defender_roll,
        } => apply_declare_melee(state, attack, *attacker_roll, *defender_roll),
        GameEffect::ResolveMelee => apply_resolve_melee(state),
        GameEffect::RetreatBeforeMelee { unit_id, to } => {
            apply_retreat_before_melee(state, *unit_id, *to)
        }
        GameEffect::AdvanceAfterCombat { unit_id, to } => {
            apply_advance_after_combat(state, *unit_id, *to)
        }
        GameEffect::RecoverUnit { unit_id } => apply_recover_unit(state, *unit_id),
        GameEffect::ConstructZariba { unit_ids, hexside } => {
            apply_construct_zariba(state, unit_ids, *hexside)
        }
        GameEffect::Demolition { unit_id, target } => apply_demolition(state, *unit_id, *target),
        GameEffect::PlaceReinforcements(placements) => {
            apply_place_reinforcements(state, placements)
        }
        GameEffect::DervishDesertion { roll, deserters } => {
            apply_dervish_desertion(state, *roll, deserters)
        }
        GameEffect::FriendliesTransport(action) => apply_friendlies_transport(state, *action),
        GameEffect::RiverMine {
            gunboat_id,
            hex,
            roll,
        } => apply_river_mine(state, *gunboat_id, *hex, *roll),
        GameEffect::SinkChain => apply_sink_chain(state),
        GameEffect::DeployUnit(placement) => apply_deploy_unit(state, placement),
        GameEffect::RemoveDeployedUnit { unit_id, player } => {
            apply_remove_deployed_unit(state, *unit_id, *player)
        }
        GameEffect::PlaceMine { hex } => apply_place_mine(state, *hex),
        GameEffect::PlaceChain { hexes } => apply_place_chain(state, hexes),
        GameEffect::PlaceZariba { hexside } => apply_place_zariba(state, *hexside),
        GameEffect::ConfirmSetupReady { player } => apply_confirm_setup_ready(state, *player),
        GameEffect::ResolveDemolition { unit_id, target } => {
            apply_resolve_demolition(state, *unit_id, *target)
        }
        GameEffect::DriftGunboat { unit_id, mine_roll } => {
            apply_drift_gunboat(state, *unit_id, *mine_roll)
        }
        GameEffect::ArtilleryBreachWall {
            firers,
            target,
            roll,
        } => apply_artillery_breach_wall(state, firers, *target, *roll),
    };
    // Post-condition: per-phase trackers never reference eliminated units.
    if result.is_ok() {
        // §9.346/§9.35: GORDON's death (at the palace, overrun in passing,
        // or through any other elimination path) ends FALL OF KHARTOUM. The
        // elimination only records the turn; the game is finished here, once
        // the effect's own bookkeeping is complete.
        if state.gordon_eliminated_turn.is_some() && !state.game_over {
            finish_game(state);
        }
        prune_dead_trackers(state);
        // Post-condition: the stacking invariants (§5.51-5.53) hold over the
        // whole board after every mutation. Any effect arm that produces an
        // illegal stack fails here, at the exact effect, instead of leaking
        // into a recorded replay. Debug builds only (release perf: the game
        // loop calls this per effect and units are few but nonzero cost).
        debug_assert!(
            state.validate_stacking_invariants().is_ok(),
            "stacking invariant violated after applying effect: {:?}",
            effect
        );
    }
    result
}

/// Largest absolute axial coordinate the engine accepts in an effect. Both
/// boards are well under 100 hexes on a side; the bound only exists so that
/// hex arithmetic on network-supplied coordinates cannot overflow.
pub const MAX_COORD_ABS: i32 = 1 << 12;

/// Reject a coordinate outside [`MAX_COORD_ABS`].
pub(crate) fn check_coord(hex: HexCoord) -> Result<(), RuleError> {
    if hex.q.unsigned_abs() > MAX_COORD_ABS as u32 || hex.r.unsigned_abs() > MAX_COORD_ABS as u32 {
        return Err(RuleError::CoordinateOutOfBounds(hex));
    }
    Ok(())
}

/// Reject a unit list that names the same unit twice (a duplicated firer,
/// attacker or deserter would count twice).
pub(crate) fn reject_duplicate_units(ids: &[UnitId]) -> Result<(), RuleError> {
    for (i, id) in ids.iter().enumerate() {
        if ids[..i].contains(id) {
            return Err(RuleError::DuplicateUnit(*id));
        }
    }
    Ok(())
}

fn check_hexside(side: &HexsideRef) -> Result<(), RuleError> {
    check_coord(side.a)?;
    check_coord(side.b)
}

/// Bound every coordinate an effect carries (see [`MAX_COORD_ABS`]).
/// Exhaustive on purpose: a new variant must decide what it carries.
fn check_effect_coords(effect: &GameEffect) -> Result<(), RuleError> {
    match effect {
        GameEffect::AdvancePhase
        | GameEffect::ResolveMelee
        | GameEffect::RecoverUnit { .. }
        | GameEffect::DervishDesertion { .. }
        | GameEffect::SinkChain
        | GameEffect::RemoveDeployedUnit { .. }
        | GameEffect::ConfirmSetupReady { .. }
        | GameEffect::DriftGunboat { .. } => Ok(()),
        GameEffect::MoveUnit { to, path, .. } => {
            check_coord(*to)?;
            path.iter().try_for_each(|h| check_coord(*h))
        }
        GameEffect::FireCombat { attack, .. } | GameEffect::HowitzerFire { attack, .. } => {
            check_coord(attack.target_hex)
        }
        GameEffect::MeleeCombat { attack, .. } | GameEffect::DeclareMelee { attack, .. } => {
            check_coord(attack.attacker_hex)?;
            check_coord(attack.defender_hex)
        }
        GameEffect::RetreatBeforeMelee { to, .. } | GameEffect::AdvanceAfterCombat { to, .. } => {
            check_coord(*to)
        }
        GameEffect::ConstructZariba { hexside, .. } | GameEffect::PlaceZariba { hexside } => {
            check_hexside(hexside)
        }
        GameEffect::Demolition { target, .. } | GameEffect::ResolveDemolition { target, .. } => {
            match target {
                DemolitionTarget::Fort(_) => Ok(()),
                DemolitionTarget::WallHexside(side) => check_hexside(side),
            }
        }
        GameEffect::PlaceReinforcements(placements) => {
            placements.iter().try_for_each(|p| check_coord(p.position))
        }
        GameEffect::DeployUnit(placement) => check_coord(placement.position),
        GameEffect::FriendliesTransport(action) => match action {
            FriendliesAction::Cross { to, .. } => check_coord(*to),
            FriendliesAction::Load { .. } | FriendliesAction::Disembark { .. } => Ok(()),
        },
        GameEffect::RiverMine { hex, .. } | GameEffect::PlaceMine { hex } => check_coord(*hex),
        GameEffect::PlaceChain { hexes } => hexes.iter().try_for_each(|h| check_coord(*h)),
        GameEffect::ArtilleryBreachWall { target, .. } => check_hexside(target),
    }
}

// ---------------------------------------------------------------------------
// 5) Phase advancement
// ---------------------------------------------------------------------------

/// Advance the game state to the next phase (rulebook §4).
pub fn advance_phase(state: &mut GameState) -> Result<(), RuleError> {
    let old_phase = state.phase;
    #[cfg(not(feature = "kani"))]
    debug!(old_phase = ?old_phase, active_player = ?state.active_player, "advance_phase");
    // §6.82/§7.5/§7.6: advance-after-combat windows close on every phase change
    // except the Direct → Maxim/Howitzer subphase bridge (§6.42), which is a
    // single continuous fire subphase for advance purposes.
    let is_642_bridge = match state.phase {
        Phase::DefensiveFire(FireSubPhase::DirectFire)
            if state.active_player == Player::Dervish =>
        {
            true
        }
        Phase::OffensiveFire(FireSubPhase::DirectFire)
            if state.active_player == Player::AngloEgyptian =>
        {
            true
        }
        _ => false,
    };
    // §7/§7.5: a declared melee must be resolved (or vacated by a retreat
    // before melee) before the melee phase may end -- otherwise the attack
    // would be silently dropped and its pre-rolled dice lost (audit: 76
    // declared melees vanished this way in the recorded games).
    if matches!(state.phase, Phase::Melee) && state.pending_melee.is_some() {
        return Err(RuleError::MeleePendingResolution);
    }
    // §8.2: "Once each campaign game, during the first night turn of the
    // game, the Dervish player rolls one die" -- the roll is made during the
    // Dervish movement phase and is mandatory, so that phase cannot end
    // before the effect has been applied (audit: every recorded campaign
    // game silently skipped it).
    if matches!(state.phase, Phase::Movement)
        && state.scenario == Scenario::Campaign
        && state.active_player == Player::Dervish
        && !state.dervish_deserted
        && crate::turn_track::scenario_turn(state.scenario, state.current_turn)
            .is_some_and(|e| e.event == crate::turn_track::TurnEvent::DervishDesertion)
    {
        return Err(RuleError::DesertionRollRequired);
    }

    // Leaving deployment is gated: both sides' required order of battle must be
    // on the board (and within limits) before the first Movement turn
    // (§9.2/§9.3/§10). Checked up here with the other guards so it cannot fail
    // after a mutation.
    if matches!(state.phase, Phase::Setup) {
        state.setup_complete()?;
    }

    // ---- validation complete; from here on the state is mutated ----
    // Clearing the advance-after-combat windows happens *after* the guards
    // above: a rejected phase advance must leave the state untouched, or a peer
    // that rejects it diverges from one that accepts it. This used to run
    // first, so a rejected `AdvancePhase` still dropped the windows.
    if !is_642_bridge {
        state.vacated_by_combat.clear();
    }
    match state.phase {
        Phase::Setup => {
            state.phase = Phase::Movement;
        }
        Phase::Movement => {
            state.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
        }
        Phase::DefensiveFire(FireSubPhase::DirectFire) => {
            if state.active_player == Player::AngloEgyptian {
                // AE turn: Dervish fired direct defensive.  Next: AE
                // offensive fire (Direct, then Maxim/Howitzer).
                state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
            } else {
                // Dervish turn: AE fired direct defensive.  AE also has
                // Maxim / howitzer capability -- they fire again now.
                state.phase = Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer);
                // §6.42: Maxim guns may fire a second time in this subphase.
                // Clear the per-phase fired set so Maxims that fired in Direct
                // Fire may fire again here.  The Maxim-only gate in
                // `can_fire_at` prevents non-Maxim units from exploiting this.
                state.units_fired_this_phase.clear();
                state.units_fired_at_this_phase.clear();
            }
        }
        Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer) => {
            state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        }
        Phase::OffensiveFire(FireSubPhase::DirectFire) => {
            if state.active_player == Player::AngloEgyptian {
                state.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
                // §6.42: same clear as the defensive path above.
                state.units_fired_this_phase.clear();
                state.units_fired_at_this_phase.clear();
            } else {
                state.phase = Phase::Melee;
            }
        }
        Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer) => {
            state.phase = Phase::Melee;
        }
        Phase::Melee => end_player_turn(state),
    }
    #[cfg(not(feature = "kani"))]
    debug!(new_phase = ?state.phase, active_player = ?state.active_player, "advance_phase done");
    Ok(())
}

/// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
pub fn end_player_turn(state: &mut GameState) {
    #[cfg(not(feature = "kani"))]
    debug!(
        old_player = ?state.active_player,
        old_turn = state.current_turn.value(),
        "end_player_turn"
    );
    resolve_pending_demolitions(state);
    recover_disrupted_units(state);
    end_zariba_construction(state);
    clear_per_turn_tracking(state);
    advance_game_turn(state);
    #[cfg(not(feature = "kani"))]
    debug!(
        new_player = ?state.active_player,
        new_turn = state.current_turn.value(),
        day_night = ?state.day_night,
        phase = ?state.phase,
        "end_player_turn done"
    );
}

/// §6.53: resolve all pending Royal Engineers demolitions before recovering
/// disrupted units. Each demolition checks adjacency + undisrupted status.
fn resolve_pending_demolitions(state: &mut GameState) {
    let pending: Vec<(UnitId, DemolitionTarget)> = std::mem::take(&mut state.pending_demolitions);
    for (eid, target) in pending {
        resolve_demolition(state, eid, target);
    }
}

/// Recover every disrupted unit owned by the player whose turn just ended.
fn recover_disrupted_units(state: &mut GameState) {
    let to_recover: Vec<UnitId> = state
        .units
        .iter()
        .filter(|u| u.state.disrupted && u.profile.identity.owner() == state.active_player)
        .map(|u| u.id)
        .collect();
    for id in &to_recover {
        if let Some(unit) = state.find_unit_mut(*id) {
            unit.state.disrupted = false;
        }
    }
}

/// §5.3: zariba construction lasts "the turn of construction" -- the
/// builders' no-offensive-fire/no-melee restriction ends with the player turn
/// in which they built.
fn end_zariba_construction(state: &mut GameState) {
    let ending = state.active_player;
    for unit in state
        .units
        .iter_mut()
        .filter(|u| u.profile.identity.owner() == ending)
    {
        unit.state.constructing_zariba = false;
    }
}

/// Clear per-phase / per-turn tracking (§5.13: MP never carry over).
fn clear_per_turn_tracking(state: &mut GameState) {
    state.units_fired_this_phase.clear();
    state.units_fired_at_this_phase.clear();
    state.mp_spent_this_turn.clear();
    // §5.24: the sticky upstream cap only lasts for the turn.
    state.gunboats_upstream_this_turn.clear();
    // §5.43: a unit stopped in an enemy ZOC may move again next turn.
    state.zoc_stopped_this_turn.clear();
    // Advance-after-combat windows do not survive the turn boundary
    // (§6.82/§7.6).
    state.vacated_by_combat.clear();
    // Reinforcement quotas reset each player-turn (§9.112/§9.113).
    state.reinforcements_placed_this_turn.clear();
    // A declared-but-unresolved melee does not survive the turn boundary.
    state.pending_melee = None;
}

/// Drop per-phase tracker entries for units no longer on the board. Run as a
/// post-condition of [`apply_effect`] so an eliminated unit can never linger
/// in `units_fired_this_phase` or `mp_spent_this_turn` (they are cleared
/// wholesale at phase end, but kept tidy mid-phase).
fn prune_dead_trackers(state: &mut GameState) {
    if state.units_fired_this_phase.is_empty()
        && state.units_fired_at_this_phase.is_empty()
        && state.zoc_stopped_this_turn.is_empty()
        && state.mp_spent_this_turn.is_empty()
        && state.vacated_by_combat.is_empty()
    {
        return;
    }
    state
        .units_fired_this_phase
        .retain(|id| state.units.iter().any(|u| &u.id == id));
    state
        .units_fired_at_this_phase
        .retain(|id| state.units.iter().any(|u| &u.id == id));
    state
        .zoc_stopped_this_turn
        .retain(|id| state.units.iter().any(|u| &u.id == id));
    state
        .mp_spent_this_turn
        .retain(|id, _| state.units.iter().any(|u| &u.id == id));
    // Advance-after-combat windows never reference eliminated units
    // (§6.82/§7.6); drop them, and drop emptied windows whole.
    for eligible in state.vacated_by_combat.values_mut() {
        eligible.retain(|id| state.units.iter().any(|u| &u.id == id));
    }
    state
        .vacated_by_combat
        .retain(|_, eligible| !eligible.is_empty());
}

/// Switch to the next player and, when play returns to the scenario's
/// first-moving player (§4: Anglo-Egyptian in the Campaign §9.113, Dervish in
/// the Historical §9.212 and Fall of Khartoum §9.322 scenarios), roll the turn
/// over -- snapshotting the completed turn and advancing the turn index.
fn advance_game_turn(state: &mut GameState) {
    let next = state.active_player.opponent();
    state.active_player = next;
    state.phase = Phase::Movement;

    if next != first_player(state.scenario) {
        return;
    }

    snapshot_turn(state);

    let next_turn = GameTurnIndex(state.current_turn.value() + 1);
    match scenario_turn(state.scenario, next_turn) {
        Some(entry) => {
            state.current_turn = next_turn;
            state.day_night = entry.day_night;
            // The §8.2 desertion turn marker is consumed by the Dervish
            // player's movement-phase `DervishDesertion` effect (validated
            // against this entry by `apply_dervish_desertion`), not here.
        }
        None => finish_game(state),
    }
}

/// Snapshot the accumulated turn events into a [`TurnSummary`] before the turn
/// index advances, so the just-completed turn is preserved in the record.
fn snapshot_turn(state: &mut GameState) {
    let events = std::mem::take(&mut state.turn_events);
    let current_entry = scenario_turn(state.scenario, state.current_turn);
    state.turn_summaries.push(TurnSummary {
        turn: state.current_turn,
        time: current_entry.map_or(crate::turn_track::GameTime::Noon, |e| e.time),
        day_night: state.day_night,
        first_player: first_player(state.scenario),
        events,
    });
}

/// The player who moves first in a scenario (§4, §9.113, §9.212, §9.322).
pub fn first_player(scenario: Scenario) -> Player {
    match scenario {
        Scenario::Campaign => Player::AngloEgyptian,
        Scenario::Historical | Scenario::FallOfKhartoum => Player::Dervish,
    }
}

/// End-of-scenario bookkeeping: mark the game over and record the victory
/// result for the scenario's victory schedule (§9.14, §9.24, §9.35).
pub fn finish_game(state: &mut GameState) {
    // Snapshot any remaining turn events before marking game over (handles
    // mid-turn endings like FoK GORDON death).
    let events = std::mem::take(&mut state.turn_events);
    if !events.is_empty() {
        let current_entry = scenario_turn(state.scenario, state.current_turn);
        state.turn_summaries.push(TurnSummary {
            turn: state.current_turn,
            time: current_entry.map_or(crate::turn_track::GameTime::Noon, |e| e.time),
            day_night: state.day_night,
            first_player: first_player(state.scenario),
            events,
        });
    }
    state.game_over = true;
    // §9.14 Mahdi's Tomb: score the 25-VP shrine to whoever controls it at the
    // conclusion of play (Campaign only; the Tomb is not in the other maps).
    if state.scenario == Scenario::Campaign {
        score_mahdis_tomb(state);
    }

    match state.scenario {
        Scenario::Campaign => {
            // §9.14 alternative auto-decisive conditions:
            //   AE decisive if every Dervish unit (incl. gunboats and forts)
            //   has been eliminated.
            //   Dervish decisive if all Anglo-Egyptian *west-bank* units
            //   (excl. gunboats) have been eliminated.
            let no_dervish = !state
                .units
                .iter()
                .any(|u| u.profile.identity.owner() == Player::Dervish);
            let no_ae_west_bank = !state.units.iter().any(|u| {
                u.profile.identity.owner() == Player::AngloEgyptian
                    && !matches!(u.profile.kind, UnitKind::Gunboat { .. })
                    && state.board.bank_of(u.position) == Some(crate::board::NileBank::West)
            });
            let ae = state.victory.total_for(Player::AngloEgyptian);
            let d = state.victory.total_for(Player::Dervish);
            let superiority = ae.0 - d.0;
            let level = if no_dervish {
                CampaignVictoryLevel::Decisive(Player::AngloEgyptian)
            } else if no_ae_west_bank {
                CampaignVictoryLevel::Decisive(Player::Dervish)
            } else {
                CampaignVictoryLevel::from_superiority(crate::VictoryPoints(superiority))
            };
            state.game_result = Some(crate::GameResult::Campaign(level));
        }
        Scenario::Historical => {
            // §9.24: each side's level is its own *unit-elimination* tally (not
            // victory points); the net result subtracts the lower level from
            // the higher.
            let dervish_lost = state.victory.units_eliminated_by(Player::AngloEgyptian);
            let ae_lost = state.victory.units_eliminated_by(Player::Dervish);
            let ae_level = HistoricalVictoryLevel::for_anglo_egyptian(dervish_lost);
            let d_level = HistoricalVictoryLevel::for_dervish(ae_lost);
            state.game_result = Some(crate::GameResult::Historical {
                ae: ae_level,
                d: d_level,
            });
        }
        Scenario::FallOfKhartoum => {
            // §9.35: the base level is set by the turn GORDON died (or his
            // survival), then the Dervish player forfeits levels for his own
            // losses. `gordon_eliminated_turn` is `None` if he survived.
            let gordon_died = state.gordon_eliminated_turn.map(|t| t.0);
            let dervish_lost = state.victory.units_eliminated_by(Player::AngloEgyptian);
            let level = crate::FoKVictoryLevel::resolve(
                gordon_died,
                state.current_turn.value(),
                dervish_lost,
            );
            state.game_result = Some(crate::GameResult::FoK(level));
        }
    }
}

/// Score the Mahdi's Tomb (§9.14): 25 VP to the Anglo-Egyptian player if, at the
/// conclusion of play, the Tomb hex is occupied by at least one British leader
/// *and* at least one non-"Friendlies" Anglo-Egyptian combat unit, both
/// undisrupted. Otherwise the Dervish player retains control and no points are
/// scored (they hold it from the start, so there is nothing to record).
///
/// The Tomb is the [`omdurman_types::Location::MahdisTomb`] hex of the walled city
/// of Omdurman (distinct from [`omdurman_types::Location::Palace`] -- on the Campaign
/// map they are at
/// different hexes); its position comes from the attached board. With no board
/// loaded the Tomb cannot be located, so control cannot pass to the
/// Anglo-Egyptian player.
pub fn score_mahdis_tomb(state: &mut GameState) {
    let Some(tomb) = state
        .board
        .hex_of_location(omdurman_types::Location::MahdisTomb)
    else {
        return;
    };
    let occupants: Vec<&UnitPlacement> = state
        .units
        .iter()
        .filter(|u| u.position == tomb && !u.state.disrupted)
        .collect();
    let has_british_leader = occupants
        .iter()
        .any(|u| matches!(u.profile.kind, UnitKind::BritishLeader { .. }));
    // A qualifying combat unit: Anglo-Egyptian, not a leader, not a gunboat,
    // and not a "Friendlies" unit (§9.14).
    let has_combat_unit = occupants.iter().any(|u| {
        u.profile.identity.owner() == Player::AngloEgyptian
            && !matches!(
                u.profile.kind,
                UnitKind::BritishLeader { .. } | UnitKind::Gunboat { .. }
            )
            && !u.profile.identity.is_friendlies()
    });
    if has_british_leader && has_combat_unit {
        state.victory.events.push(VpEvent {
            turn: state.current_turn,
            source: VpSource::MahdisTomb,
        });
    }
}

/// §9.346: in FALL OF KHARTOUM, GORDON is eliminated the instant a Dervish unit
/// passes through or occupies the Palace hex (by normal movement or advance
/// after combat). Records the turn (which fixes the §9.35 victory level) and
/// ends the game -- also when GORDON already fell earlier in the same effect
/// (overrun in passing, §6.51). A no-op outside FoK or while he lives
/// unthreatened.
pub fn check_gordon_palace(state: &mut GameState) {
    if state.scenario != Scenario::FallOfKhartoum {
        return;
    }
    if state.gordon_eliminated_turn.is_none()
        && let Some(palace) = state
            .board
            .hex_of_location(omdurman_types::Location::Palace)
        && state
            .units
            .iter()
            .any(|u| u.position == palace && u.profile.identity.owner() == Player::Dervish)
    {
        eliminate_gordon(state);
    }
    if state.gordon_eliminated_turn.is_some() && !state.game_over {
        finish_game(state);
    }
}

/// Eliminate GORDON (§9.346, §9.35): score and remove him through the shared
/// [`eliminate_unit`] path, which records the turn of his death. The game is
/// then finished by [`apply_effect`]'s post-condition, once the triggering
/// effect's own bookkeeping is complete. A no-op once GORDON is already gone.
pub(crate) fn eliminate_gordon(state: &mut GameState) {
    if state.gordon_eliminated_turn.is_some() {
        return;
    }
    match state
        .units
        .iter()
        .find(|u| u.profile.identity.is_gordon())
        .map(|u| u.id)
    {
        Some(gordon_id) => eliminate_unit(state, gordon_id, ElimCause::GordonAtPalace),
        None => {
            // A palace event with no GORDON counter on the board: still
            // record the turn so the §9.35 victory level is fixed. The
            // Gordon counter's id is `BritishBoats_3_1`.
            state.gordon_eliminated_turn = Some(state.current_turn);
            state.turn_events.push(TurnEventRecord::UnitEliminated {
                unit: UnitId::BritishBoats_3_1,
                cause: ElimCause::GordonAtPalace,
            });
        }
    }
}

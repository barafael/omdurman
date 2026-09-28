use super::*;

/// Apply a Friendlies-transport action (rulebook §5.21), validated by
/// [`GameState::can_friendlies_transport`].
///
/// * `Load`: the unit goes aboard -- onto its gunboat's hex -- and the
///   mission starts this turn.
/// * `Disembark`: the unit lands on the chosen west-bank hex, paying its
///   terrain cost like any first hex entered, and the mission ends.
pub fn apply_friendlies_transport(
    state: &mut GameState,
    action: FriendliesAction,
) -> Result<(), RuleError> {
    state.can_friendlies_transport(action)?;
    match action {
        FriendliesAction::Load { unit, gunboat } => {
            let at = state.unit_or_err(gunboat)?.position;
            if let Some(u) = state.find_unit_mut(unit) {
                u.position = at;
                u.state.loaded_on = Some(gunboat);
            }
            state.friendlies_transport = Some(TransportState::Loaded {
                unit,
                gunboat,
                since: state.current_turn,
            });
        }
        FriendliesAction::Disembark { unit, to, .. } => {
            let from = state.unit_or_err(unit)?.position;
            let cost = state.land_step_cost(from, to) as i16;
            let (owner, kind) = {
                let u = state.unit_or_err(unit)?;
                (u.profile.identity.owner(), u.profile.kind)
            };
            if let Some(u) = state.find_unit_mut(unit) {
                u.position = to;
                u.state.loaded_on = None;
            }
            let spent = state.mp_spent(unit).saturating_add(cost);
            state.mp_spent_this_turn.insert(unit, spent);
            // §5.26: landing in an enemy ZOC stops the unit there.
            if state.hex_in_enemy_zoc(to, owner, kind) {
                state.zoc_stopped_this_turn.push(unit);
            }
            state.friendlies_transport = None;
            state.observations.push(Observation::FriendliesDisembarked {
                unit_id: unit,
                at: to,
            });
        }
    }
    Ok(())
}

/// §10.12: every Anglo-Egyptian gunboat that has lost its engines "must
/// drift two hexes per turn (with the current)". Applied at the start of
/// the Anglo-Egyptian player turn (`end_player_turn`): each hex follows the
/// current of the hex it leaves; the drift stops short of the board's edge,
/// any occupied hex and the chain (§10.23), and stops *on* a mine, which it
/// strikes like any British gunboat entering a mined hex.
pub(crate) fn drift_disabled_gunboats(state: &mut GameState) {
    let drifting: Vec<UnitId> = state
        .units
        .iter()
        .filter(|u| {
            u.state.engines_lost
                && matches!(u.profile.kind, UnitKind::Gunboat { .. })
                && u.profile.identity.owner() == Player::AngloEgyptian
        })
        .map(|u| u.id)
        .collect();
    for id in drifting {
        for _ in 0..2 {
            let Some(at) = state.find_unit(id).map(|u| u.position) else {
                break;
            };
            let Some(flow) = state.board.flow_at(at) else {
                break;
            };
            let next = at.neighbors()[flow as usize];
            if !state.board.is_nile(next)
                || state.chain_covers(next)
                || state.units.iter().any(|u| u.position == next)
            {
                break;
            }
            for u in state
                .units
                .iter_mut()
                .filter(|u| u.id == id || u.state.loaded_on == Some(id))
            {
                u.position = next;
            }
            if strikes_mine(state, id, next) {
                break;
            }
        }
    }
}

/// §10.12: a British gunboat entering `hex` strikes an untriggered mine
/// there -- it stops, and the Dervish player rolls for it (a pending
/// [`GameEffect::RiverMine`]). The Dervish player's own gunboats pass
/// through safely (§10.14). Returns whether the gunboat struck one.
pub(crate) fn strikes_mine(state: &mut GameState, gunboat: UnitId, hex: HexCoord) -> bool {
    let british = state
        .find_unit(gunboat)
        .is_some_and(|u| u.profile.identity.owner() == Player::AngloEgyptian);
    if !british || !state.mines.iter().any(|m| m.hex == hex && !m.triggered) {
        return false;
    }
    state.pending_mine = Some(crate::StruckMine { gunboat, hex });
    if !state.gunboats_stopped_this_turn.contains(&gunboat) {
        state.gunboats_stopped_this_turn.push(gunboat);
    }
    true
}

// ---------------------------------------------------------------------------
// 12) Optional rules
// ---------------------------------------------------------------------------

/// Resolve the mine a British gunboat has struck (rulebook §10.12) with the
/// Dervish player's roll: 1-4 no effect, 5-7 the engines are lost (the
/// gunboat drifts from now on), 8-10 sunk. Only the pending strike may be
/// resolved, and each mine goes off once (§10.13).
pub fn apply_river_mine(
    state: &mut GameState,
    gunboat_id: UnitId,
    hex: HexCoord,
    roll: DieRoll,
) -> Result<(), RuleError> {
    if state.pending_mine
        != Some(crate::StruckMine {
            gunboat: gunboat_id,
            hex,
        })
    {
        return Err(RuleError::NoUntriggeredMine(hex));
    }
    // §10.13: a mine only fires once.
    let Some(mine) = state
        .mines
        .iter_mut()
        .find(|m| m.hex == hex && !m.triggered)
    else {
        return Err(RuleError::NoUntriggeredMine(hex));
    };
    mine.triggered = true;
    state.pending_mine = None;

    let result = crate::MineResult::from_roll(roll);
    match result {
        crate::MineResult::NoEffect => {}
        crate::MineResult::EnginesLost => {
            if let Some(unit) = state.find_unit_mut(gunboat_id) {
                // §10.12: engines lost -- the gunboat drifts two hexes per turn
                // with the current for the rest of the game.
                unit.state.engines_lost = true;
            }
        }
        crate::MineResult::Sunk => {
            // The shared elimination path scores the sunk gunboat (§9.14)
            // and takes any loaded "Friendlies" unit down with it (§5.21).
            eliminate_unit(state, gunboat_id, ElimCause::RiverMine);
        }
    }
    state.observations.push(Observation::MineResolved {
        gunboat: gunboat_id,
        hex,
        roll,
        result,
    });
    Ok(())
}

/// Artillery fire at the river chain (rulebook §10.23 b): every firer
/// passes [`GameState::can_fire_at_chain`]; their factors, each banded at
/// its own range, are summed onto one Combat Results Table row, and a
/// result of 3 or more sinks the chain.
pub fn apply_sink_chain(
    state: &mut GameState,
    firers: &[UnitId],
    roll: DieRoll,
) -> Result<(), RuleError> {
    if firers.is_empty() {
        return Err(RuleError::NoFirers);
    }
    reject_duplicate_units(firers)?;
    let mut total: u16 = 0;
    for &id in firers {
        let (factor, range, _) = state.can_fire_at_chain(id)?;
        let unit = state.unit_or_err(id)?;
        let band = range_band_for(
            state.scenario,
            range_table_player_for(state.scenario, unit),
            unit.profile.weapon,
            range,
        );
        total = total.saturating_add(band.apply(factor.value()));
    }
    // ---- validation complete; from here on the state is mutated ----
    // §6.24: an Anglo-Egyptian direct fire attack.
    let modified = roll.apply_modifier(1);
    let result = combat_results_table(FireFactorRow::from_total(total), modified);
    let sunk = matches!(result, CombatResult::Eliminate(n) if n >= 3);
    state.units_fired_this_phase.extend(firers.iter().copied());
    if sunk && let Some(chain) = state.chain.as_mut() {
        chain.sunk = true;
    }
    state.observations.push(Observation::ChainFiredAt {
        firers: firers.to_vec(),
        roll,
        result,
        sunk,
    });
    Ok(())
}

/// §10.23 a: the British sink the chain "by having an infantry or cavalry
/// unit spend one complete turn on either riverbank adjacent to a 'chained'
/// river hex". Checked at the end of the Anglo-Egyptian player turn: an
/// undisrupted infantry or cavalry unit on land next to a chained hex that
/// has not moved all turn has spent it there.
pub(crate) fn sink_chain_from_the_bank(state: &mut GameState) {
    let Some(chain) = state.chain.as_ref().filter(|c| !c.sunk) else {
        return;
    };
    let on_the_bank = state.units.iter().any(|u| {
        u.profile.identity.owner() == Player::AngloEgyptian
            && matches!(
                u.profile.kind,
                UnitKind::Infantry { .. } | UnitKind::Cavalry { .. }
            )
            && !u.state.disrupted
            && u.state.loaded_on.is_none()
            && !state.board.is_nile(u.position)
            && state.mp_spent(u.id) == 0
            && chain.hexes.iter().any(|h| h.is_adjacent_to(u.position))
    });
    if on_the_bank {
        if let Some(chain) = state.chain.as_mut() {
            chain.sunk = true;
        }
        state.observations.push(Observation::ChainSunkFromTheBank);
    }
}

/// Kani proof harnesses over river-mine resolution (`cargo kani`, see
/// `scripts/kani.sh`). Bounded state in the `any_state` style: one gunboat
/// and one mine on a rule-neutral board, pinned to the only two shapes that
/// matter (British boat in a mined hex; Dervish boat in a mined hex), with
/// the die roll symbolic -- so the whole d10 domain is covered.
#[cfg(kani)]
mod verification {
    // `use super::*` reaches only this file's own items; everything else is
    // imported from where it is defined.
    use super::*;
    use crate::effects::GameState;
    use crate::{
        DieRoll, HexCoord, MinePlacement, UnitId, UnitIdentity, UnitMovement, UnitPlacement,
        UnitProfile, UnitState, WeaponClass,
    };
    use omdurman_types::UnitKind;

    /// A gunboat sitting on an untriggered mine.
    fn state_with_mined_boat(dervish: bool) -> GameState {
        use crate::{GunboatId, OldGunboat, UnitIdentity, UnitMovement, UnitProfile};
        // Roster-free state (see `GameState::kani_minimal`): the sinking arm now
        // runs the shared `eliminate_unit` path, whose ledger pushes on top of
        // `GameState::new` exceed the symex budget.
        let mut state = GameState::kani_minimal();
        let hex = HexCoord::new(0, 0);
        let identity = if dervish {
            UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1))
        } else {
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(OldGunboat::Tamai))
        };
        state.units.push(UnitPlacement {
            id: UnitId::ALL[0],
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::Gunboat {
                    fire: 3,
                    upstream: 10,
                    downstream: 16,
                },
                identity,
                weapon: WeaponClass::Artillery,
                fire: None,
                melee: None,
                movement: UnitMovement::Immobile,
            },
            state: UnitState::default(),
        });
        state.mines.push(MinePlacement {
            hex,
            triggered: false,
        });
        // A British boat entering the hex has struck the mine (§10.12).
        if !dervish {
            state.pending_mine = Some(crate::StruckMine {
                gunboat: UnitId::ALL[0],
                hex,
            });
        }
        state
    }

    fn any_roll() -> DieRoll {
        let i: usize = kani::any();
        kani::assume(i < DieRoll::ALL.len());
        DieRoll::ALL[i]
    }

    // The mine-band *arithmetic* is proven over the whole symbolic d10
    // domain by `MineResult::from_roll`'s harness (`lib.rs`). These state
    // harnesses pin the *wiring* -- that `apply_river_mine` triggers the
    // mine and applies the right mutation for each band -- using concrete
    // worst-case rolls per band. A symbolic roll here merges all three
    // mutation arms into one SAT instance and blows the symex budget
    // (measured: >45 min without finishing vs ~2 min each); each concrete
    // harness is constant-folded and fast.

    /// §10.12/§10.13: even the most harmless roll (1 = NoEffect band)
    /// spends the mine -- the trigger latch is set by every resolution, so
    /// a stream of drifting gunboats can never grind the same mine twice
    /// -- and leaves the boat afloat and unflagged.
    // §10.12 §10.13
    #[kani::proof]
    #[kani::unwind(4)]
    fn river_mine_triggers_even_on_a_harmless_roll() {
        let hex = HexCoord::new(0, 0);
        let mut state = state_with_mined_boat(false);
        let result = apply_river_mine(&mut state, UnitId::ALL[0], hex, DieRoll::One);
        assert!(result.is_ok());
        assert!(state.mines[0].triggered);
        match state.find_unit(UnitId::ALL[0]) {
            Some(boat) => assert!(!boat.state.engines_lost),
            None => panic!("no-effect band sank the gunboat"),
        }
        state.kani_discard();
    }

    /// §10.12: a roll in the engines-lost band (5-7; here the band edge 5)
    /// sets the drift-with-the-current flag on the gunboat, for the rest of
    /// the game.
    // §10.12
    #[kani::proof]
    #[kani::unwind(4)]
    fn river_mine_engine_loss_arms_the_drift_flag() {
        let hex = HexCoord::new(0, 0);
        let mut state = state_with_mined_boat(false);
        let result = apply_river_mine(&mut state, UnitId::ALL[0], hex, DieRoll::Five);
        assert!(result.is_ok());
        assert!(state.mines[0].triggered);
        match state.find_unit(UnitId::ALL[0]) {
            Some(boat) => assert!(boat.state.engines_lost),
            None => panic!("engines-lost band sank the gunboat"),
        }
        state.kani_discard();
    }

    /// §10.12: a roll in the sunk band (8-10; here the band edge 8) removes
    /// the gunboat from the board entirely.
    // §10.12
    #[kani::proof]
    #[kani::unwind(4)]
    fn river_mine_sinking_removes_the_gunboat() {
        let hex = HexCoord::new(0, 0);
        let mut state = state_with_mined_boat(false);
        let result = apply_river_mine(&mut state, UnitId::ALL[0], hex, DieRoll::Eight);
        assert!(result.is_ok());
        assert!(state.mines[0].triggered);
        assert!(state.find_unit(UnitId::ALL[0]).is_none());
        state.kani_discard();
    }

    /// Re-entering an already-triggered mine is rejected with the mine
    /// record and the board untouched -- the latch is one-way and the
    /// rejection is atomic (no band re-rolled, no casualty).
    // §10.13
    #[kani::proof]
    #[kani::unwind(4)]
    fn a_triggered_mine_never_fires_again() {
        let roll = any_roll();
        let hex = HexCoord::new(0, 0);
        let mut state = state_with_mined_boat(false);
        state.mines[0].triggered = true;
        let units_before = state.units.len();
        assert!(apply_river_mine(&mut state, UnitId::ALL[0], hex, roll).is_err());
        assert!(state.mines[0].triggered);
        assert!(state.units.len() == units_before);
        state.kani_discard();
    }

    /// §10.14: the Dervish player's own gunboats pass mined hexes with no
    /// ill effect (he knows where his mines are) -- even on the worst
    /// possible roll, and in particular the mine stays untriggered and
    /// available against the British player.
    // §10.14
    #[kani::proof]
    #[kani::unwind(4)]
    fn dervish_gunboats_pass_mined_hexes_unharmed() {
        let hex = HexCoord::new(0, 0);
        let mut state = state_with_mined_boat(true);
        assert!(!strikes_mine(&mut state, UnitId::ALL[0], hex));
        assert!(state.pending_mine.is_none());
        assert!(state.gunboats_stopped_this_turn.is_empty());
        assert!(!state.mines[0].triggered);
        // ...and nothing to roll for. A fresh state, so the solver does not
        // carry the (refuted) struck branch into the whole resolution.
        let mut fresh = state_with_mined_boat(true);
        assert!(apply_river_mine(&mut fresh, UnitId::ALL[0], hex, DieRoll::Ten).is_err());
        assert!(fresh.find_unit(UnitId::ALL[0]).is_some());
        state.kani_discard();
        fresh.kani_discard();
    }
}

use super::*;

/// Filter `before` down to the IDs of units that have been eliminated (i.e.
/// are no longer present in `state.units`). Used by fire/melee resolution to
/// compute the post-mutation elimination list from a pre-mutation snapshot.
pub(crate) fn diff_eliminated(state: &GameState, before: Vec<UnitId>) -> Vec<UnitId> {
    before
        .into_iter()
        .filter(|id| state.find_unit(*id).is_none())
        .collect()
}

/// Eliminate `unit_id` from play: the single path every unit death takes
/// (fire/melee CRT results, §6.51 overruns and orphaned leaders, §6.61/§6.62
/// special targets, §6.53/§6.63 demolitions and breaches, §10.12 mines).
///
/// 1. score the elimination and record it under `cause` (§9.14);
/// 2. remove the unit from the board;
/// 3. GORDON's death in FALL OF KHARTOUM records the turn that fixes the
///    §9.35 victory level (§9.346); [`apply_effect`] then ends the game;
/// 4. cascade: a sunk gunboat takes its loaded "Friendlies" unit down with
///    it (§5.21, recorded as [`ElimCause::LostWithTransport`], and scored --
///    the unit is just as dead).
///
/// A no-op for a unit that is not on the board.
pub(crate) fn eliminate_unit(state: &mut GameState, unit_id: UnitId, cause: ElimCause) {
    let Some(unit) = take_off_the_board(state, unit_id, cause) else {
        return;
    };
    if matches!(unit.profile.kind, UnitKind::Gunboat { .. }) {
        // §5.21: the loaded unit goes down with the ship. A passenger is
        // never itself a gunboat, so the cascade is one level deep -- spelled
        // without recursion, which Kani would unroll to the unwind bound
        // with the whole scoring path at every level.
        let mut lost: Vec<UnitId> = Vec::new();
        for u in &state.units {
            if u.state.loaded_on == Some(unit_id) {
                lost.push(u.id);
            }
        }
        if let Some(TransportState::Loaded { unit, gunboat, .. }) = state.friendlies_transport
            && gunboat == unit_id
        {
            if !lost.contains(&unit) {
                lost.push(unit);
            }
            state.friendlies_transport = None;
        }
        for id in lost {
            take_off_the_board(state, id, ElimCause::LostWithTransport);
        }
    }
}

/// Steps 1-3 of [`eliminate_unit`] for one unit; the unit as it stood, or
/// `None` when it is not on the board.
fn take_off_the_board(
    state: &mut GameState,
    unit_id: UnitId,
    cause: ElimCause,
) -> Option<UnitPlacement> {
    let pos = state.units.iter().position(|u| u.id == unit_id)?;
    let unit = state.units[pos];
    score_elimination(state, unit_id, cause);
    // Unit ids are unique in `state.units`, so removing the single match is
    // the same filter `retain` would do -- spelled with `position`+`remove`
    // because `Vec::retain`'s closure-driven symex is intractable under Kani
    // (see the river-mine harnesses).
    state.units.remove(pos);
    state.eliminated.push(unit_id);
    if unit.profile.identity.is_gordon()
        && state.scenario == Scenario::FallOfKhartoum
        && state.gordon_eliminated_turn.is_none()
    {
        // §9.346/§9.35: the turn of GORDON's death fixes the victory level.
        state.gordon_eliminated_turn = Some(state.current_turn);
        state.observations.push(Observation::GordonEliminated {
            turn: state.current_turn,
        });
    }
    Some(unit)
}

/// Score victory points for eliminating a unit (rulebook §9.14) and record the
/// elimination under `cause`. The owner is derived from the unit's identity,
/// so unlike the historical signature there is no caller-supplied player.
pub fn score_elimination(state: &mut GameState, unit_id: UnitId, cause: ElimCause) {
    if let Some(unit) = state.find_unit(unit_id) {
        let identity = unit.profile.identity;
        let position = unit.position;
        let vp_source = vp_source_for(&identity, position, &state.board);
        if vp_source == Some(VpSource::IsaZachneihEliminated) {
            state.isa_zachneih_eliminated = true;
        }

        if let Some(source) = vp_source {
            let points = source.points();
            let scorer = source.who_scores();
            state.victory.events.push(crate::VpEvent {
                turn: state.current_turn,
                source,
            });
            state.turn_events.push(TurnEventRecord::VpScored {
                source,
                points,
                for_player: scorer,
            });
            state.observations.push(Observation::VictoryScored {
                source,
                points,
                for_player: scorer,
            });
        }

        // Surface the elimination as an observation regardless of VP.
        state.turn_events.push(TurnEventRecord::UnitEliminated {
            unit: unit_id,
            cause,
        });
        state.observations.push(Observation::UnitEliminated {
            id: unit_id,
            cause,
            vp_source,
        });

        // Leader-specific observation for dispatch-slip flavour.
        if matches!(identity, crate::UnitIdentity::DervishLeader(_))
            | matches!(identity, crate::UnitIdentity::AngloEgyptianLeader(_))
        {
            state.observations.push(Observation::LeaderKilled {
                id: unit_id,
                by: state.active_player,
            });
        }
    }
}

/// VP source awarded for eliminating a unit of `identity` at `position`
/// (rulebook §9.14). `None` means the elimination scores no points (e.g. a
/// Dervish fort, which is worth 0 pts). Pure lookup -- it does not mutate
/// state; the caller owns any side effects (e.g. the Isa Zachneih flag).
fn vp_source_for(
    identity: &crate::UnitIdentity,
    position: HexCoord,
    board: &BoardInfo,
) -> Option<VpSource> {
    if identity.is_friendlies() {
        // §9.14: a "Friendlies" unit scores by the bank it died on -- 1 pt
        // on the east bank, 3 pts on the west bank.
        match board.bank_of(position) {
            Some(crate::board::NileBank::West) => Some(VpSource::FriendliesWestBankEliminated),
            _ => Some(VpSource::FriendliesEastBankEliminated),
        }
    } else {
        match *identity {
            crate::UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah) => {
                Some(VpSource::KhalifaEliminated)
            }
            crate::UnitIdentity::DervishTribal {
                tribe: DervishTribe::IsaZachneih,
            } => Some(VpSource::IsaZachneihEliminated),
            crate::UnitIdentity::DervishTribal { .. }
            | crate::UnitIdentity::DervishLeader(_)
            | crate::UnitIdentity::DervishArtillery
            | crate::UnitIdentity::DervishGunboat(_) => Some(VpSource::DervishUnitEliminated),
            // §9.14: 0 pts for forts.
            crate::UnitIdentity::DervishFort | crate::UnitIdentity::AngloEgyptianFort => None,
            crate::UnitIdentity::AngloEgyptianLeader(_) => Some(VpSource::BritishLeaderEliminated),
            crate::UnitIdentity::AngloEgyptianGunboat(_) => Some(VpSource::BritishGunboatSunk),
            _ => Some(VpSource::AngloEgyptianLandUnitEliminated),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Kani proof harnesses over the victory-point routing (`cargo kani`, see
/// `scripts/kani.sh`). `vp_source_for` is the single lookup that decides
/// what an elimination is worth; these harnesses hold it to the printed
/// §9.14 schedule over a symbolic elimination (identity and hex chosen by
/// the solver, rule-neutral board). `score_elimination`'s ledger
/// post-conditions are an exhaustive test instead
/// (`effects::tests::score_elimination_records_exactly_what_it_scores`).
#[cfg(kani)]
mod verification {
    // `use super::*` reaches only this file's own items; everything else is
    // imported from where it is defined (the crate root, the effects root's
    // public re-exports, or `omdurman-types`).
    use super::*;
    use crate::{HexCoord, UnitIdentity};
    use omdurman_types::DervishTribe;

    /// A symbolic elimination target spanning every §9.14 routing shape:
    /// Khalifa, Isa Zachneih, another Dervish tribal, a Dervish leader,
    /// Dervish artillery, Dervish gunboat, Dervish fort (0 pts), a British
    /// leader, a British gunboat, ordinary Anglo-Egyptian infantry, and a
    /// "Friendlies" unit.
    fn any_vp_identity() -> UnitIdentity {
        use crate::{
            BattalionOrdinal, BritishLeader as BL, DervishLeader as DL, GunboatId, OldGunboat,
        };
        use omdurman_types::{BrigadeId, BrigadeNationality, DervishTribe};
        let i: usize = kani::any();
        let friendly = |nationality| UnitIdentity::AngloEgyptianInfantry {
            brigade: BrigadeId {
                number: 1,
                nationality,
            },
            battalion: BattalionOrdinal::First,
        };
        match i % 12 {
            0 => UnitIdentity::DervishLeader(DL::KhalifaAbdullah),
            1 => UnitIdentity::DervishTribal {
                tribe: DervishTribe::IsaZachneih,
            },
            2 => UnitIdentity::DervishTribal {
                tribe: DervishTribe::Hadendowa,
            },
            3 => UnitIdentity::DervishLeader(DL::OsmanDigna),
            4 => UnitIdentity::DervishArtillery,
            5 => UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1)),
            6 => UnitIdentity::DervishFort,
            7 => UnitIdentity::AngloEgyptianLeader(BL::Kitchener),
            8 => UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(OldGunboat::Tamai)),
            9 => friendly(BrigadeNationality::British),
            10 => UnitIdentity::AngloEgyptianFort,
            _ => friendly(BrigadeNationality::Friendlies),
        }
    }

    /// §9.14: the elimination-to-VP-source routing is exact for every unit
    /// shape: the Khalifa is worth his printed 10, Isa Zachneih her 1,
    /// other Dervish units 1 each, forts nothing at all (the printed 0 --
    /// FALL OF KHARTOUM's British forts included), British leaders and gunboats 10 each to the Dervish,
    /// ordinary Anglo-Egyptian land units 3, and a "Friendlies" unit 1 or 3
    /// by the bank it dies on (the rule-neutral board has no banks, so the
    /// east-bank default applies -- the west-bank case is the same lookup
    /// with `bank_of == Some(West)`).
    // §9.14
    #[traceability_macro::rulebook("§9.14")]
    #[kani::proof]
    #[kani::unwind(4)]
    fn vp_source_for_routes_every_elimination_to_the_printed_source() {
        use crate::VpSource;
        let identity = any_vp_identity();
        let q: i32 = kani::any();
        let r: i32 = kani::any();
        kani::assume(q >= -2 && q <= 2);
        kani::assume(r >= -2 && r <= 2);
        let board = BoardInfo::default();
        let source = vp_source_for(&identity, HexCoord::new(q, r), &board);
        let is_friendlies = identity.is_friendlies();
        match identity {
            UnitIdentity::DervishFort | UnitIdentity::AngloEgyptianFort => {
                assert!(source.is_none());
            }
            _ if is_friendlies => {
                assert!(source == Some(VpSource::FriendliesEastBankEliminated));
            }
            UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah) => {
                assert!(source == Some(VpSource::KhalifaEliminated));
            }
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::IsaZachneih,
            } => {
                assert!(source == Some(VpSource::IsaZachneihEliminated));
            }
            UnitIdentity::DervishTribal { .. }
            | UnitIdentity::DervishLeader(_)
            | UnitIdentity::DervishArtillery
            | UnitIdentity::DervishGunboat(_) => {
                assert!(source == Some(VpSource::DervishUnitEliminated));
            }
            UnitIdentity::AngloEgyptianLeader(_) => {
                assert!(source == Some(VpSource::BritishLeaderEliminated));
            }
            UnitIdentity::AngloEgyptianGunboat(_) => {
                assert!(source == Some(VpSource::BritishGunboatSunk));
            }
            _ => {
                assert!(source == Some(VpSource::AngloEgyptianLandUnitEliminated));
            }
        }
        // Every routed source is a positive award (0-pt shapes are `None`).
        if let Some(source) = source {
            assert!(source.points().value() >= 1);
        }
    }
}

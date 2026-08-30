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
            crate::UnitIdentity::DervishFort => None, // §9.14: 0 pts for forts.
            crate::UnitIdentity::AngloEgyptianLeader(_) => Some(VpSource::BritishLeaderEliminated),
            crate::UnitIdentity::AngloEgyptianGunboat(_) => Some(VpSource::BritishGunboatSunk),
            _ => Some(VpSource::AngloEgyptianLandUnitEliminated),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

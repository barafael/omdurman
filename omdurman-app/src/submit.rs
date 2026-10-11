//! Checked submission of the local player's game actions.
//!
//! The engine re-validates every event when its sequenced echo is applied,
//! and a rejected echo is only logged (`game_apply`). Without feedback the
//! player just sees nothing happen. So the user-facing submit sites dry-run
//! the event first: [`dry_run`] applies it to a clone of the engine state
//! (projected forward over this peer's still-unconfirmed submissions, so a
//! quick second action is judged against the state the first one will
//! produce), and on a [`RuleError`] the action is not submitted and a
//! dispatch slip carries the engine's reason instead.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use omdurman_net::GameEvent;
use omdurman_rules::effects::{GameEffect, GameState, RuleError, apply_effect};

use crate::PendingEdits;
use crate::dispatch::Dispatches;

/// Dispatch-slip header for a refused order.
pub(crate) const REFUSED_HEADER: &str = "Order Refused";

/// Apply `event` to `state` the way the live echo path would. Session events
/// (`StartGame`, the seat events -- not engine effects) and sprite events that name no
/// rules counter are accepted unchecked — `game_apply` handles those.
pub(crate) fn apply_for_check(state: &mut GameState, event: &GameEvent) -> Result<(), RuleError> {
    match event {
        GameEvent::StartGame { .. }
        | GameEvent::SeatAssigned { .. }
        | GameEvent::SeatCarved { .. }
        | GameEvent::Telegram { .. } => Ok(()),
        GameEvent::Effect(effect) => apply_effect(state, effect),
        GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } | GameEvent::RemoveUnit { .. } => {
            match crate::game_apply::sprite_event_effect(event, state) {
                Some(effect) => apply_effect(state, &effect),
                None => Ok(()),
            }
        }
    }
}

/// The engine state this peer's pending submissions will lead to: `gs` with
/// every still-unconfirmed submission applied in order (rejections skipped,
/// as the engine will skip them).
pub(crate) fn projected_state(gs: &GameState, unconfirmed: &[GameEvent]) -> GameState {
    let mut state = gs.clone();
    for event in unconfirmed {
        let _ = apply_for_check(&mut state, event);
    }
    state
}

/// Would the engine accept `event` after this peer's pending submissions?
pub(crate) fn dry_run(
    gs: &GameState,
    unconfirmed: &[GameEvent],
    event: &GameEvent,
) -> Result<(), RuleError> {
    let mut state = projected_state(gs, unconfirmed);
    apply_for_check(&mut state, event)
}

/// Submit `event` if the engine would accept it; otherwise push an
/// "Order Refused" slip with the engine's reason and submit nothing.
/// Returns whether the event was submitted.
pub(crate) fn submit_checked(
    pending: &mut PendingEdits,
    dispatches: Option<&mut Dispatches>,
    gs: &GameState,
    event: GameEvent,
) -> bool {
    let unconfirmed: Vec<GameEvent> = pending.unconfirmed.iter().map(|(_, e)| e.clone()).collect();
    match dry_run(gs, &unconfirmed, &event) {
        Ok(()) => {
            pending.submit_game(event);
            true
        }
        Err(error) => {
            info!(%error, ?event, "local action refused by pre-validation");
            // A game-over refusal while the live game still runs means one
            // of our own pending submissions ends it (the first unit of a
            // stack move reaching the Palace): the rest of the batch is moot,
            // not a mistake to explain.
            let ended_by_our_own_order = matches!(error, RuleError::GameOver) && !gs.game_over;
            if let Some(dispatches) = dispatches
                && !ended_by_our_own_order
            {
                dispatches.push(REFUSED_HEADER, error.to_string());
            }
            false
        }
    }
}

/// Whether the local seat may submit `event` as far as *whose decision* it
/// is goes: the engine takes a §7.5 retreat (or its decline) and a melee
/// resolution from anyone, so the submit path checks that a retreat is the
/// local seat's own unit (faction and §1.1 command scope), the decline its
/// own side's, and the resolution or an advance the attacker's
/// ([`omdurman_bot::actions::effect_actor`]). Every other event passes: the
/// turn/faction gates at the input sites cover them.
pub(crate) fn decision_is_local(
    gs: &GameState,
    event: &GameEvent,
    seats: &[omdurman_net::Seat],
    key: omdurman_net::PlayerKey,
) -> bool {
    let GameEvent::Effect(effect) = event else {
        return true;
    };
    let Some(actor) = omdurman_bot::actions::effect_actor(gs, effect) else {
        return true;
    };
    let unit_in_scope = match effect {
        GameEffect::RetreatBeforeMelee { unit_id, .. }
        | GameEffect::AdvanceAfterCombat { unit_id, .. } => gs
            .find_unit(*unit_id)
            .is_none_or(|u| crate::seats::scope_allows(seats, key, &u.profile.identity)),
        _ => true,
    };
    crate::seats::may_act(seats, key, actor) && unit_in_scope
}

/// System-param bundle for [`submit_checked`]: the outbound queue plus the
/// dispatch queue that carries refusals. `Dispatches` is optional so headless
/// test apps without the dispatch plugin still submit; so are the seat
/// table and the local key the [`decision_is_local`] guard reads.
#[derive(SystemParam)]
pub(crate) struct CheckedSubmit<'w> {
    pub pending: ResMut<'w, PendingEdits>,
    pub dispatches: Option<ResMut<'w, Dispatches>>,
    seats: Option<Res<'w, crate::seats::Seats>>,
    key: Option<Res<'w, crate::seats::LocalPlayerKey>>,
}

impl CheckedSubmit<'_> {
    /// See [`submit_checked`]; first refuses another side's decision (see
    /// [`decision_is_local`]).
    pub(crate) fn submit(&mut self, gs: &GameState, event: GameEvent) -> bool {
        if let (Some(seats), Some(key)) = (self.seats.as_deref(), self.key.as_deref())
            && !decision_is_local(gs, &event, &seats.0, key.0)
        {
            info!(?event, "local action refused: not this seat's decision");
            if let Some(dispatches) = self.dispatches.as_deref_mut() {
                dispatches.push(
                    REFUSED_HEADER,
                    "That is the other side's decision.".to_string(),
                );
            }
            return false;
        }
        submit_checked(&mut self.pending, self.dispatches.as_deref_mut(), gs, event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_rules::effects::GameEffect;
    use omdurman_types::Scenario;

    fn pending() -> PendingEdits {
        PendingEdits::default()
    }

    #[test]
    fn refused_action_is_not_submitted_and_explains_why() {
        // Leaving Setup with nothing deployed is refused (§9.2/§9.3).
        let gs = GameState::new(Scenario::Campaign);
        let reason = dry_run(&gs, &[], &GameEvent::Effect(GameEffect::AdvancePhase))
            .expect_err("empty deployment cannot advance");
        let mut pending = pending();
        let mut dispatches = Dispatches::default();
        let submitted = submit_checked(
            &mut pending,
            Some(&mut dispatches),
            &gs,
            GameEvent::Effect(GameEffect::AdvancePhase),
        );
        assert!(!submitted);
        assert!(pending.unconfirmed.is_empty());
        assert!(pending.outgoing_broadcast.is_empty());
        assert_eq!(dispatches.slips.len(), 1);
        assert_eq!(dispatches.slips[0].header, REFUSED_HEADER);
        assert_eq!(dispatches.slips[0].body, reason.to_string());
    }

    #[test]
    fn accepted_action_is_submitted_without_a_slip() {
        let gs = GameState::new(Scenario::Campaign);
        let event = GameEvent::Effect(GameEffect::ConfirmSetupReady {
            player: omdurman_types::Player::Dervish,
        });
        // Whatever the engine says about this effect, submit_checked must
        // agree with the dry run.
        let expected = dry_run(&gs, &[], &event).is_ok();
        let mut pending = pending();
        let mut dispatches = Dispatches::default();
        let submitted = submit_checked(&mut pending, Some(&mut dispatches), &gs, event);
        assert_eq!(submitted, expected);
        assert_eq!(pending.unconfirmed.len(), usize::from(expected));
        assert_eq!(dispatches.slips.len(), usize::from(!expected));
    }

    #[test]
    fn start_game_is_never_blocked() {
        let gs = GameState::new(Scenario::Campaign);
        let event = GameEvent::StartGame {
            seats: vec![],
            scenario: Scenario::Campaign,
            optional_rules: vec![],
        };
        assert!(dry_run(&gs, &[], &event).is_ok());
    }

    /// §7.5: the engine takes a retreat before melee from anyone, so the
    /// submit path makes sure it is the local seat's own unit -- and the
    /// decline its own side's, the resolution the attacker's.
    #[test]
    fn only_the_defender_submits_the_defenders_reaction() {
        use omdurman_net::{PlayerKey, Seat, SeatHolder};
        use omdurman_types::Player;
        let script = omdurman_rules::tactics::all_scripts()
            .into_iter()
            .find(|s| s.name == "retreat_before_melee")
            .expect("the §7.5 vignette");
        let mut gs = script.state;
        let omdurman_rules::tactics::ScriptStep::Legal { effect, .. } = &script.steps[0] else {
            panic!("the vignette opens with the declaration");
        };
        apply_effect(&mut gs, effect).unwrap();
        // An Anglo-Egyptian infantry attack on a Dervish camel unit.
        let camel = gs.pending_melee.as_ref().unwrap().attack.defenders[0];
        let (attacker, defender) = (PlayerKey(1), PlayerKey(2));
        let seats = vec![
            Seat {
                faction: Player::AngloEgyptian,
                scope: None,
                holder: SeatHolder::Human(attacker),
            },
            Seat {
                faction: Player::Dervish,
                scope: None,
                holder: SeatHolder::Human(defender),
            },
        ];
        let retreat = GameEvent::Effect(GameEffect::RetreatBeforeMelee {
            unit_id: camel,
            to: omdurman_types::HexCoord::new(33, 13),
        });
        let decline = GameEvent::Effect(GameEffect::DeclineRetreatBeforeMelee);
        let resolve = GameEvent::Effect(GameEffect::ResolveMelee);
        assert!(decision_is_local(&gs, &retreat, &seats, defender));
        assert!(decision_is_local(&gs, &decline, &seats, defender));
        assert!(!decision_is_local(&gs, &resolve, &seats, defender));
        assert!(!decision_is_local(&gs, &retreat, &seats, attacker));
        assert!(!decision_is_local(&gs, &decline, &seats, attacker));
        assert!(decision_is_local(&gs, &resolve, &seats, attacker));
        // A spectator decides nothing; an unbound session (no seats) anything.
        assert!(!decision_is_local(&gs, &retreat, &seats, PlayerKey(3)));
        assert!(decision_is_local(&gs, &retreat, &[], attacker));
        // Other events are left to the input sites' gates.
        assert!(decision_is_local(
            &gs,
            &GameEvent::Effect(GameEffect::AdvancePhase),
            &seats,
            PlayerKey(3)
        ));
    }

    #[test]
    fn dry_run_does_not_touch_the_live_state() {
        let gs = GameState::new(Scenario::Campaign);
        let before = format!("{:?}", gs.phase);
        let _ = dry_run(&gs, &[], &GameEvent::Effect(GameEffect::AdvancePhase));
        assert_eq!(format!("{:?}", gs.phase), before);
    }
}

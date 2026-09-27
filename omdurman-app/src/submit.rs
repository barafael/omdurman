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
use omdurman_rules::effects::{GameState, RuleError, apply_effect};

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
        | GameEvent::SeatCarved { .. } => Ok(()),
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

/// System-param bundle for [`submit_checked`]: the outbound queue plus the
/// dispatch queue that carries refusals. `Dispatches` is optional so headless
/// test apps without the dispatch plugin still submit.
#[derive(SystemParam)]
pub(crate) struct CheckedSubmit<'w> {
    pub pending: ResMut<'w, PendingEdits>,
    pub dispatches: Option<ResMut<'w, Dispatches>>,
}

impl CheckedSubmit<'_> {
    /// See [`submit_checked`].
    pub(crate) fn submit(&mut self, gs: &GameState, event: GameEvent) -> bool {
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

    #[test]
    fn dry_run_does_not_touch_the_live_state() {
        let gs = GameState::new(Scenario::Campaign);
        let before = format!("{:?}", gs.phase);
        let _ = dry_run(&gs, &[], &GameEvent::Effect(GameEffect::AdvancePhase));
        assert_eq!(format!("{:?}", gs.phase), before);
    }
}

//! Bevy [`Message`] types that decouple game-action producers (input systems,
//! network handler) from their consumers (state appliers, UI updaters).
//!
//! * **Outbound** -- [`LocalAction`] is emitted by local input systems
//!   (picker clicks, combat buttons) to request a game action.
//!   -> [`forward_local_actions`] bridges it into [`PendingEdits`] for the wire.
//!
//! * **Inbound** -- [`ObservationEvent`] is emitted for each engine
//!   observation produced by applying a sequenced game event.
//!   -> UI listeners (dispatch slips, combat cards) react without polling.

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use omdurman_rules::effects::Observation;

use crate::PendingEdits;

/// A game action initiated by the local player (unit placement, movement,
/// combat, map edit, ...). The [`forward_local_actions`] system stages it via
/// `PendingEdits::submit_game` (assigning the submission uid) for broadcast.
#[derive(Message, Clone)]
pub struct LocalAction {
    pub event: omdurman_net::GameEvent,
}

pub fn forward_local_actions(
    mut reader: MessageReader<LocalAction>,
    mut pending: ResMut<PendingEdits>,
) {
    for action in reader.read() {
        info!("forward_local_actions: bridging LocalAction to PendingEdits");
        pending.submit_game(action.event.clone());
    }
}

/// Staging buffer for [`Observation`]s drained from the rules engine's
/// [`GameState`](omdurman_rules::effects::GameState) after each `apply_effect`
/// call. A scheduled system drains this into [`ObservationEvent`] messages so
/// decoupled listeners (dispatch slips, sounds, VP animations) can react without
/// polling game state every frame.
#[derive(Resource, Default)]
pub struct PendingObservations(pub Vec<Observation>);

#[derive(Message, Clone)]
pub struct ObservationEvent {
    pub observation: Observation,
}

pub fn drain_observations(
    mut buffer: ResMut<PendingObservations>,
    mut writer: MessageWriter<ObservationEvent>,
) {
    for obs in buffer.0.drain(..) {
        log_observation(&obs);
        writer.write(ObservationEvent { observation: obs });
    }
}

/// Narrate each engine observation into the log so a game can be followed
/// from logs alone: every shot, melee, elimination, and breach states *what*
/// happened and *why* (roll, modifiers, band) in one line.
fn log_observation(obs: &Observation) {
    match obs {
        Observation::FireResolved {
            attack,
            roll,
            total_modifier,
            modified_roll,
            factor_row,
            result,
            eliminations,
            range,
            band,
            ..
        } => info!(
            firers = ?attack.firers,
            target = ?attack.target_hex,
            kind = ?attack.kind,
            roll = ?roll,
            modifier = total_modifier,
            modified = ?modified_roll,
            row = ?factor_row,
            range = ?range,
            band = band.as_deref().unwrap_or("?"),
            result = ?result,
            eliminated = eliminations.len(),
            "fire resolved"
        ),
        Observation::MeleeResolved {
            attack,
            attacker_roll,
            attacker_result,
            defender_roll,
            defender_result,
            attacker_losses,
            ..
        } => info!(
            attack = ?attack,
            attacker_roll = ?attacker_roll,
            attacker_result = ?attacker_result,
            defender_roll = ?defender_roll,
            defender_result = ?defender_result,
            attacker_losses = attacker_losses.len(),
            "melee resolved"
        ),
        Observation::UnitEliminated { id, cause, .. } => {
            info!(unit = ?id, %cause, "unit eliminated")
        }
        Observation::FortDestroyed { id, hex } => {
            info!(fort = ?id, hex = ?hex, "fort destroyed")
        }
        Observation::WallBreached {
            hexside,
            breached,
            row,
            ..
        } => info!(
            hexside = ?hexside,
            breached,
            row = ?row,
            "wall breach attempt resolved"
        ),
        Observation::LeaderKilled { id, by } => {
            info!(leader = ?id, ?by, "leader killed")
        }
        Observation::GordonEliminated { turn } => {
            info!(turn = ?turn, "GORDON has fallen")
        }
        Observation::DemolitionResolved {
            engineer_id,
            target,
            success,
        } => info!(
            engineer = ?engineer_id,
            target = ?target,
            success,
            "demolition resolved"
        ),
        Observation::VictoryScored {
            source,
            points,
            for_player,
        } => info!(
            source = ?source,
            points = ?points,
            player = ?for_player,
            "victory points awarded"
        ),
        other => info!(?other, "game observation"),
    }
}

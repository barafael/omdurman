//! Hex-click handler for river-mine and river-chain placement (§10.11, §10.21).
//!
//! During the Setup phase, the Dervish player may place up to two river mines on
//! Nile hexes, or a single river chain spanning up to four contiguous Nile hexes.
//! This system checks for a hex click and emits the corresponding `GameEffect`.

use bevy::prelude::*;

use crate::input::CombatClickCtx;
use crate::ui_plugin::OptionalRulePlacement;
use crate::{GameStateResource, PendingEdits};

/// Emits `PlaceMine` or `PlaceChain` effects when the Dervish player clicks a
/// Nile hex during Setup with the placement UI active.
pub(crate) fn handle_optional_rule_click(
    mut click: CombatClickCtx,
    game_state: Option<Res<GameStateResource>>,
    mut placement: ResMut<OptionalRulePlacement>,
    mut pending: ResMut<PendingEdits>,
) {
    let Some(gs) = game_state else { return };

    // Only during Setup.
    if !matches!(gs.0.phase, omdurman_rules::Phase::Setup) {
        return;
    }

    let clicked = click.clicked_hex();
    let Some(hex) = clicked else { return };

    // Check river-mine placement.
    if let Some(_target) = placement.pending_mine.take() {
        // Validate: must be a Nile hex.
        let is_nile = gs.0.board.terrain_at(hex).is_some_and(|t| t.is_nile());
        if !is_nile {
            placement.pending_mine = Some(hex); // let the player try again
            return;
        }

        pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
            omdurman_net::GameEvent::Effect(
                omdurman_rules::effects::GameEffect::PlaceMine { hex },
            ),
        ));
        return;
    }

    // Check river-chain placement.
    if placement.placing_chain {
        // Max 4 hexes.
        if placement.chain_hexes.len() >= 4 {
            return;
        }

        // Validate Nile hex.
        let is_nile = gs.0.board.terrain_at(hex).is_some_and(|t| t.is_nile());
        if !is_nile {
            return;
        }

        // Don't allow duplicates.
        if placement.chain_hexes.contains(&hex) {
            return;
        }

        // Check contiguity with the last hex (up to 2 hex distance along the river).
        if let Some(&last) = placement.chain_hexes.last() {
            let dist = hex.distance(last);
            if dist > 2 {
                return;
            }
        }

        placement.chain_hexes.push(hex);

        // If we've reached 4, auto-submit.
        if placement.chain_hexes.len() == 4 {
            pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
                omdurman_net::GameEvent::Effect(
                    omdurman_rules::effects::GameEffect::PlaceChain {
                        hexes: std::mem::take(&mut placement.chain_hexes),
                    },
                ),
            ));
            placement.placing_chain = false;
        }
    }
}

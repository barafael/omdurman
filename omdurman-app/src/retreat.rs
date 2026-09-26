//! Retreat before melee (§7.5) -- the defender's reaction.
//!
//! During the *attacker's* Melee phase, the **defending** player may pull a
//! threatened cavalry/camel unit two hexes back before the melee is resolved.
//! This is the non-active player's action, so it is gated on the local faction
//! being the *opponent* of the rules engine's active player (the attacker).
//!
//! Selecting an eligible unit highlights the legal two-hex retreat
//! destinations (empty, passable, within range); clicking one broadcasts a
//! [`GameEffect::RetreatBeforeMelee`]. The engine validates via
//! [`GameState::can_retreat_before_melee`].

use bevy::prelude::*;
use omdurman_net::GameEvent;
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{Phase, UnitId};
use omdurman_types::HexCoord;

use crate::GameStateResource;
use crate::board_click::RetreatClick;
use crate::peers::Peers;
use crate::picker::{PickerState, PlacedUnit, selected_unit_ids};

/// The retreat-eligible member of the current selection, if any. The
/// defender selects a tile (the unified combat selection model) or clicks the
/// threatened hex; the §7.5 retreat applies to whichever eligible
/// cavalry/camel counter rides in it.
pub(crate) fn selected_threatened_unit(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    gs: &GameState,
) -> Option<UnitId> {
    selected_unit_ids(state, placed_units)
        .into_iter()
        .find(|&unit| !valid_retreat_hexes(unit, gs).is_empty())
}

/// The first unit at `hex` that may retreat before the pending melee (§7.5),
/// as the engine judges it. Lets the defender -- who is not the phase player
/// and so cannot select through the picker -- pick the unit by clicking the
/// threatened hex.
pub(crate) fn retreat_candidate_at(gs: &GameState, hex: HexCoord) -> Option<UnitId> {
    gs.units
        .iter()
        .filter(|u| u.position == hex)
        .map(|u| u.id)
        .find(|&unit| !valid_retreat_hexes(unit, gs).is_empty())
}

/// Bundle of the read-only picker state + the placed-units query so
/// [`retreat_overlay_mesh`] stays under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct RetreatSelection<'w, 's> {
    pub state: Res<'w, PickerState>,
    pub placed_units: Query<'w, 's, (Entity, &'static PlacedUnit)>,
}

/// Two-hex retreat destinations the engine accepts for `unit` (§7.5: exactly
/// two hexes, empty, on-board, not the Nile, pending infantry melee on the
/// unit's hex, unit eligible and unmoved).
fn valid_retreat_hexes(unit: UnitId, gs: &GameState) -> Vec<HexCoord> {
    let Some(u) = gs.find_unit(unit) else {
        return Vec::new();
    };
    let mut out: Vec<HexCoord> = u
        .position
        .neighbors()
        .into_iter()
        .flat_map(HexCoord::neighbors)
        .filter(|h| u.position.distance(*h) == 2)
        .filter(|h| gs.can_retreat_before_melee(unit, *h).is_ok())
        .collect();
    out.sort_by_key(|h| (h.q, h.r));
    out.dedup();
    out
}

/// Highlight legal retreat destinations (orange) when the defender selects a
/// threatened cavalry/camel unit during the attacker's Melee phase.
#[derive(Component)]
pub struct RetreatTargetRing;

pub fn retreat_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    selection: RetreatSelection,
    game_state: Option<Res<GameStateResource>>,
    peers: Peers,
    existing: Query<Entity, With<RetreatTargetRing>>,
) {
    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing.iter());
    let RetreatSelection {
        state,
        placed_units,
    } = selection;
    let Some(gs) = game_state else { return };
    // The defender is the opponent of the active (attacking) player; an
    // unbound session may act (single-seat play/testing) — `may_act`'s rule.
    if !matches!(gs.0.phase, Phase::Melee) || !peers.may_act(gs.0.active_player.opponent()) {
        return;
    }
    let Some(unit) = selected_threatened_unit(&state, &placed_units, &gs.0) else {
        return;
    };
    for target in valid_retreat_hexes(unit, &gs.0) {
        rings.ring(RetreatTargetRing, target, 1.5, 1.0, &hex.assets.orange);
    }
}

/// The defender's §7.5 retreat-window click: on the threatened hex it selects
/// the unit that may retreat; on a legal destination (with that unit
/// selected) it broadcasts a `RetreatBeforeMelee` effect.
pub fn handle_retreat(
    mut clicks: bevy::ecs::message::MessageReader<RetreatClick>,
    mut state: ResMut<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    peers: Peers,
    mut submit: crate::submit::CheckedSubmit,
) {
    // Routed by `board_click::route_board_clicks` (the §7.5 retreat window).
    let Some(&RetreatClick(to)) = clicks.read().last() else {
        return;
    };
    let Some(gs) = game_state else { return };
    if !peers.may_act(gs.0.active_player.opponent()) {
        return;
    }
    if let Some(candidate) = retreat_candidate_at(&gs.0, to)
        && let Some((source, _)) = placed_units
            .iter()
            .find(|(_, p)| p.unit_id == Some(candidate))
    {
        *state = PickerState::Selected {
            source,
            start_coord: to,
            remaining_mp: 0,
            forced_stop: false,
        };
        return;
    }
    let Some(unit) = selected_threatened_unit(&state, &placed_units, &gs.0) else {
        return;
    };
    if gs.0.can_retreat_before_melee(unit, to).is_err() {
        return;
    }

    info!(?unit, to.q = to.q, to.r = to.r, "retreat before melee");
    submit.submit(
        &gs.0,
        GameEvent::Effect(GameEffect::RetreatBeforeMelee { unit_id: unit, to }),
    );
    *state = PickerState::Idle;
}

//! What on the board is news to the local player: the opponent's moves and
//! entries (a ring, and a sighting for the edge pointer), and every combat (a
//! trace or a melee mark).

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;
use omdurman_rules::UnitId;
use omdurman_rules::effects::Observation;
use omdurman_types::{HexCoord, Player};

use super::pointers::Sightings;
use super::{FxRequest, LiveApplied};

/// Whether `owner`'s actions are news to the local player: the other side's
/// (an opponent, the AI), or anyone's for a spectator. An unbound session
/// (no seats) plays both sides itself, so nothing is news.
fn is_foreign(owner: Option<Player>, peers: &crate::peers::Peers) -> bool {
    let Some(owner) = owner else { return false };
    match peers.local() {
        Some(local) => owner != local,
        None => peers.is_spectator(),
    }
}

fn owner_of(unit: UnitId) -> Option<Player> {
    omdurman_rules::unit_profiles::section_owner(unit.section_pos().0)
}

/// The opponent's moves and entries: a ring at each hex they ended on, and a
/// sighting for the edge pointer.
pub fn announce_live_changes(
    mut live: ResMut<LiveApplied>,
    peers: crate::peers::Peers,
    mut fx: MessageWriter<FxRequest>,
    mut sightings: ResMut<Sightings>,
) {
    if live.0.is_empty() {
        return;
    }
    let mut hexes: Vec<HexCoord> = Vec::new();
    for change in live.0.drain(..) {
        if change.setup || change.is_empty() {
            continue;
        }
        for (unit, hex) in change.moved.iter().chain(&change.placed) {
            if is_foreign(owner_of(*unit), &peers) && !hexes.contains(hex) {
                hexes.push(*hex);
            }
        }
    }
    for hex in hexes {
        fx.write(FxRequest::Attention { hex });
        sightings.see(hex);
    }
}

/// Combat on the board: a trace from each firing hex to the target (and a
/// finer one on to where a howitzer shell scattered), a mark between melee
/// opponents. The opponent's attacks are sightings too.
pub fn announce_combat(
    mut observations: MessageReader<crate::events::ObservationEvent>,
    game_state: Res<crate::GameStateResource>,
    peers: crate::peers::Peers,
    mut fx: MessageWriter<FxRequest>,
    mut sightings: ResMut<Sightings>,
) {
    for event in observations.read() {
        match &event.observation {
            Observation::FireResolved { attack, impact, .. } => {
                let firers = attack.all_firing_units();
                let aimed = attack.target_hex;
                for from in firing_hexes(&game_state.0, &firers) {
                    fx.write(FxRequest::Shot {
                        from,
                        to: aimed,
                        scatter: false,
                    });
                }
                let landed = impact.map_or(aimed, |(_, hex)| hex);
                if landed != aimed {
                    fx.write(FxRequest::Shot {
                        from: aimed,
                        to: landed,
                        scatter: true,
                    });
                }
                if is_foreign(firers.first().copied().and_then(owner_of), &peers) {
                    sightings.see(landed);
                }
            }
            Observation::MeleeResolved { attack, .. } => {
                fx.write(FxRequest::Clash {
                    attacker: attack.attacker_hex,
                    defender: attack.defender_hex,
                });
                let attacker_owner = game_state
                    .0
                    .units
                    .iter()
                    .find(|u| u.position == attack.attacker_hex)
                    .and_then(|u| owner_of(u.id))
                    .or(Some(game_state.0.active_player));
                if is_foreign(attacker_owner, &peers) {
                    sightings.see(attack.defender_hex);
                }
            }
            _ => {}
        }
    }
}

/// The distinct hexes `firers` stand on: a stacked combined attack (§6.14)
/// draws one trace, not one per unit. Fire moves no firer, so where they
/// stand now is where they fired from.
pub(super) fn firing_hexes(
    gs: &omdurman_rules::effects::GameState,
    firers: &[UnitId],
) -> Vec<HexCoord> {
    let mut hexes: Vec<HexCoord> = Vec::new();
    for id in firers {
        if let Some(unit) = gs.find_unit(*id)
            && !hexes.contains(&unit.position)
        {
            hexes.push(unit.position);
        }
    }
    hexes
}

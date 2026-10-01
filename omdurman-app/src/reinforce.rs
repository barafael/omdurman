//! Reinforcement entry guidance (§9.112/§9.113 Campaign).
//!
//! Placement itself flows through the ordinary unit picker: during a Movement
//! phase a picker click applies `PlaceReinforcements` (see
//! `placement.rs`/`picker.rs`), and the engine validates the counter against
//! the current turn's order of appearance. This module only *guides* the
//! interaction: it rings the annotated entrance hexes of the side about to
//! receive reinforcements (§9.112 west edge; §9.113 entrance area / north
//! Nile edge / Abu Alim hut) and reports how many counters may still enter
//! this turn.

use bevy::prelude::*;
use omdurman_rules::effects::GameState;
use omdurman_rules::{Phase, unit_id_for_section_pos};
use omdurman_types::{HexCoord, NamedArea, Player, Scenario};

use crate::GameStateResource;
use crate::peers::Peers;
use crate::picker::UnitPicker;

/// Registers the reinforcement-guidance systems.
pub struct ReinforcePlugin;

impl Plugin for ReinforcePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            reinforce_entry_overlay_mesh.run_if(crate::board_view_active),
        );
    }
}

/// Marker for an entry-edge highlight ring so it can be cleared each frame.
#[derive(Component)]
pub struct ReinforceEntryRing;

/// Whether reinforcement-entry guidance should show: the Campaign (the only
/// scenario with off-board arrivals, §9.112/§9.113), during a Movement
/// phase.
fn entry_window_open(gs: &GameState) -> bool {
    gs.scenario == Scenario::Campaign && matches!(gs.phase, Phase::Movement)
}

/// The entrance areas a side's reinforcements arrive through (§9.112/§9.113).
fn entrance_areas(player: Player) -> &'static [NamedArea] {
    match player {
        Player::Dervish => &[NamedArea::DervishWestEdge],
        Player::AngloEgyptian => &[
            NamedArea::AngloEgyptianEntrance,
            NamedArea::GunboatNorthEdge,
            NamedArea::AbuAlimHut,
        ],
    }
}

/// The annotated entrance hexes for the active player's side. Boards without
/// the annotation stay permissive (the engine accepts any legal hex), so an
/// empty result means "no rings".
fn entrance_hexes(gs: &GameState) -> Vec<HexCoord> {
    let mut out: Vec<HexCoord> = entrance_areas(gs.active_player)
        .iter()
        .flat_map(|area| gs.board.entrance_hexes(*area))
        .collect();
    out.sort_by_key(|h| (h.q, h.r));
    out.dedup();
    out
}

/// Highlight the annotated entrance hexes (green) while the local player's
/// side may bring reinforcements in (§9.112/§9.113).
pub fn reinforce_entry_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    game_state: Res<GameStateResource>,
    peers: Peers,
    existing: Query<Entity, With<ReinforceEntryRing>>,
) {
    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing.iter());
    let gs = game_state;
    if !entry_window_open(&gs.0) || !peers.may_act(gs.0.phase_player()) {
        return;
    }

    for target in entrance_hexes(&gs.0) {
        rings.ring(ReinforceEntryRing, target, 1.4, 1.0, &hex.assets.green);
    }
}

/// How many of the active side's unplaced counters may still enter this turn
/// (§9.112/§9.113): each counter the engine would accept on some hex of its
/// entrance area -- wave, quotas, stacking and all.
pub fn enterable_count(gs: &GameState, picker: &UnitPicker) -> usize {
    if gs.scenario != Scenario::Campaign {
        return 0;
    }
    picker
        .available
        .iter()
        .filter(|u| u.visible)
        .filter_map(|u| unit_id_for_section_pos(u.section_name, u.col as u8, u.row as u8))
        .filter_map(|id| Some((id, omdurman_rules::unit_profiles::profile_for_unit(id)?)))
        .filter(|(_, profile)| profile.identity.owner() == gs.active_player)
        .filter(|&(id, profile)| {
            gs.board
                .entrance_hexes(omdurman_rules::effects::entrance_area_for(&profile))
                .into_iter()
                .any(|position| {
                    gs.can_place_single_reinforcement(&omdurman_rules::UnitPlacement {
                        id,
                        position,
                        profile,
                        state: Default::default(),
                    })
                    .is_ok()
                })
        })
        .count()
}

/// The sidebar/banner reminder for the active player's reinforcement window,
/// or `None` when nothing may enter right now.
pub fn reinforcement_hint(gs: &GameState, picker: &UnitPicker) -> Option<String> {
    if !entry_window_open(gs) {
        return None;
    }
    let n = enterable_count(gs, picker);
    if n == 0 {
        return None;
    }
    Some(format!(
        "\u{2022} Reinforcements: {n} counter(s) may enter this turn (§9.112/§9.113) — drag them onto the green entrance hexes (or click one, then a hex)"
    ))
}

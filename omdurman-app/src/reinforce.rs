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
        app.add_systems(Update, reinforce_entry_overlay_mesh.run_if(crate::on_board));
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

/// The hexes counter `id` could enter on right now (§9.112/§9.113): its own
/// entrance area, filtered by the engine's full single-placement check --
/// wave, quotas, stacking and all. Empty when it may not enter this turn.
pub fn entry_hexes_for(gs: &GameState, id: omdurman_rules::UnitId) -> Vec<HexCoord> {
    if !entry_window_open(gs) {
        return Vec::new();
    }
    let Some(profile) = omdurman_rules::unit_profiles::profile_for_unit(id) else {
        return Vec::new();
    };
    if profile.identity.owner() != gs.active_player {
        return Vec::new();
    }
    gs.board
        .entrance_hexes(omdurman_rules::effects::entrance_area_for(&profile))
        .into_iter()
        .filter(|&position| {
            gs.can_place_single_reinforcement(&omdurman_rules::UnitPlacement {
                id,
                position,
                profile,
                state: Default::default(),
            })
            .is_ok()
        })
        .collect()
}

/// Whether counter `id` may enter this turn on some hex of its entrance area.
pub fn enterable(gs: &GameState, id: omdurman_rules::UnitId) -> bool {
    !entry_hexes_for(gs, id).is_empty()
}

/// This turn's order-of-appearance allowance for the side moving now
/// (§9.112/§9.113), as players read it: what may still come on.
pub fn arrival_allowance(gs: &GameState) -> Option<String> {
    if !entry_window_open(gs) {
        return None;
    }
    let turn = gs.current_turn.value();
    if gs.active_player == Player::Dervish {
        return (turn <= 3).then(|| "This turn's tribes enter on the west edge.".to_string());
    }
    if turn >= 4 {
        return Some("All remaining units may enter.".to_string());
    }
    let arrived: Vec<_> = gs
        .reinforcements_placed_this_turn
        .iter()
        .filter(|&&(player, _)| player == Player::AngloEgyptian)
        .filter_map(|&(_, id)| omdurman_rules::unit_profiles::profile_for_unit(id))
        .filter(|p| !matches!(p.kind, omdurman_types::UnitKind::BritishLeader { .. }))
        .collect();
    let boats = arrived.iter().filter(|p| p.kind.is_boat()).count();
    let land = arrived.len() - boats;
    Some(if turn == 1 {
        format!(
            "Gunboats {boats} of 3 \u{00b7} first wave: Friendlies, Egyptian Cavalry, \
             Horse Artillery, two Egyptian brigades"
        )
    } else {
        format!("Gunboats {boats} of 3 \u{00b7} land units {land} of 12")
    })
}

/// Ring the hexes reinforcements may enter on while the local player's side
/// may bring them in (§9.112/§9.113): the picked counter's own legal hexes,
/// or -- with nothing picked -- the entrance areas of every counter that may
/// still enter. The rings are rebuilt only when the set changes (or the
/// overlays were cleared).
#[allow(clippy::too_many_arguments)]
pub fn reinforce_entry_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    game_state: Res<GameStateResource>,
    peers: Peers,
    picker: (Res<UnitPicker>, Res<crate::picker::PickerState>),
    existing: Query<Entity, With<ReinforceEntryRing>>,
    mut last: Local<Option<Vec<HexCoord>>>,
    (generation, mut seen_generation): (Res<crate::picker::OverlayGeneration>, Local<u32>),
) {
    if generation.invalidates(&mut seen_generation) {
        *last = None;
    }
    let (picker, picker_state) = picker;
    let gs = &game_state.0;
    let picked = match &*picker_state {
        crate::picker::PickerState::Placing { unit_idx, .. } => picker
            .available
            .get(*unit_idx)
            .and_then(|u| unit_id_for_section_pos(u.section_name, u.col as u8, u.row as u8)),
        _ => None,
    };
    let mut targets = if !(entry_window_open(gs) && peers.may_act(gs.phase_player())) {
        Vec::new()
    } else if let Some(id) = picked {
        entry_hexes_for(gs, id)
    } else if any_enterable(gs, &picker) {
        entrance_hexes(gs)
    } else {
        Vec::new()
    };
    targets.sort_by_key(|h| (h.q, h.r));
    if last.as_ref() == Some(&targets) {
        return;
    }
    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing.iter());
    for &target in &targets {
        rings.ring(ReinforceEntryRing, target, 1.4, 1.0, &hex.assets.green);
    }
    *last = Some(targets);
}

/// Whether any unplaced counter of the moving side may enter this turn.
fn any_enterable(gs: &GameState, picker: &UnitPicker) -> bool {
    picker
        .available
        .iter()
        .filter_map(|u| unit_id_for_section_pos(u.section_name, u.col as u8, u.row as u8))
        .any(|id| enterable(gs, id))
}

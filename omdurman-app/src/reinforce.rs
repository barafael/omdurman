//! Reinforcement entry guidance (§9.112/§9.113 Campaign, §9.322 FoK turn 1).
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
use omdurman_hexmap::hex_world_pos;
use omdurman_rules::effects::GameState;
use omdurman_rules::reinforcements::{CampaignLeader, ReinforcementWave};
use omdurman_rules::{Phase, UnitIdentity, unit_id_for_section_pos};
use omdurman_types::{HexCoord, NamedArea, Player, Scenario};

use crate::GameStateResource;
use crate::peers::Peers;
use crate::picker::UnitPicker;

/// Registers the reinforcement-guidance systems.
pub struct ReinforcePlugin;

impl Plugin for ReinforcePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reinforce_entry_overlay_mesh);
    }
}

/// Marker for an entry-edge highlight ring so it can be cleared each frame.
#[derive(Component)]
pub struct ReinforceEntryRing;

/// Whether the local seat controls the currently active player (or no seat is
/// bound — editor / single-machine play).
fn local_controls_active(peers: &Peers, gs: &GameState) -> bool {
    match peers.local() {
        Some(player) => player == gs.active_player,
        None => !peers.any_assigned(),
    }
}

/// Whether reinforcement-entry guidance should show: a scenario with
/// off-board arrivals, during a Movement phase.
fn entry_window_open(gs: &GameState) -> bool {
    matches!(gs.scenario, Scenario::Campaign | Scenario::FallOfKhartoum)
        && matches!(gs.phase, Phase::Movement)
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
/// side may bring reinforcements in (§9.112/§9.113/§9.322).
pub fn reinforce_entry_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    game_state: Option<Res<GameStateResource>>,
    peers: Peers,
    existing: Query<Entity, With<ReinforceEntryRing>>,
) {
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = game_state else { return };
    if !entry_window_open(&gs.0) || !local_controls_active(&peers, &gs.0) {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for hex in entrance_hexes(&gs.0) {
        let pos = hex_world_pos(hex, origin, &overlay.params);
        commands.spawn((
            ReinforceEntryRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.green.clone()),
            Transform::from_xyz(pos.x, 1.4, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
}

/// Whether `identity` is admitted by the current turn's wave, ignoring the
/// land-unit cap (§9.112 by tribe/leader; §9.113 leaders listed).
fn wave_admits_identity(wave: &ReinforcementWave, identity: &UnitIdentity) -> bool {
    match identity {
        UnitIdentity::DervishTribal { tribe } => wave.tribes.contains(tribe),
        UnitIdentity::DervishLeader(leader) => wave
            .leaders
            .iter()
            .any(|l| matches!(l, CampaignLeader::Dervish(d) if d == leader)),
        UnitIdentity::AngloEgyptianLeader(leader) => wave
            .leaders
            .iter()
            .any(|l| matches!(l, CampaignLeader::British(b) if b == leader)),
        _ => false,
    }
}

/// How many of the active side's unplaced counters may still enter this turn
/// (§9.112/§9.113): wave membership (tribe/leader) plus the wave's remaining
/// land-unit cap for Anglo-Egyptian land arrivals. Gunboat quota and stacking
/// are left to the engine's authoritative check on the echo. `0` when no wave
/// exists for this turn (or in non-campaign scenarios, which guide via
/// `fok_entry` instead).
pub fn enterable_count(gs: &GameState, picker: &UnitPicker) -> usize {
    if gs.scenario != Scenario::Campaign {
        return 0;
    }
    let schedule = match gs.active_player {
        Player::Dervish => omdurman_rules::reinforcements::dervish_campaign_schedule(),
        Player::AngloEgyptian => omdurman_rules::reinforcements::anglo_egyptian_campaign_schedule(),
    };
    let Some(wave) = schedule.wave_for_turn(gs.current_turn.value()) else {
        return 0;
    };

    // Land units already entered this player-turn (§9.113 cap tracking).
    let land_entered = gs
        .reinforcements_placed_this_turn
        .iter()
        .filter(|(p, _)| *p == gs.active_player)
        .filter(|(_, id)| {
            omdurman_rules::unit_profiles::profile_for_unit(*id).is_some_and(|prof| {
                !matches!(prof.identity, UnitIdentity::AngloEgyptianLeader(_))
                    && !prof.kind.is_boat()
            })
        })
        .count();
    let cap_left = wave.unit_cap.map(|cap| cap.saturating_sub(land_entered));

    picker
        .available
        .iter()
        .filter(|u| u.visible)
        .filter_map(|u| {
            let id = unit_id_for_section_pos(u.section_name, u.col as u8, u.row as u8)?;
            omdurman_rules::unit_profiles::profile_for_unit(id)
        })
        .filter(|prof| prof.identity.owner() == gs.active_player)
        // Already on the board (entered an earlier wave): never again.
        .filter(|prof| !gs.units.iter().any(|u| u.profile.identity == prof.identity))
        .filter(|prof| match prof.identity.owner() {
            Player::Dervish => wave_admits_identity(wave, &prof.identity),
            Player::AngloEgyptian => match &prof.identity {
                UnitIdentity::AngloEgyptianLeader(_) => wave_admits_identity(wave, &prof.identity),
                _ if prof.kind.is_boat() => true,
                _ => cap_left.is_none_or(|left| left > 0),
            },
        })
        .count()
}

/// The sidebar/banner reminder for the active player's reinforcement window,
/// or `None` when nothing may enter right now.
pub fn reinforcement_hint(gs: &GameState, picker: &UnitPicker) -> Option<String> {
    if !entry_window_open(gs) || gs.scenario != Scenario::Campaign {
        return None;
    }
    let n = enterable_count(gs, picker);
    if n == 0 {
        return None;
    }
    Some(format!(
        "\u{2022} Reinforcements: {n} counter(s) may enter this turn (§9.112/§9.113) — drag them onto the green entrance hexes"
    ))
}

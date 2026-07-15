//! Guided Dervish entry for FALL OF KHARTOUM (§9.322).
//!
//! On turn 1 the Dervish player's units enter "through any hexes on the south
//! or east edge of the map." Placement itself flows through the ordinary unit
//! picker (`PlaceUnit`); this module only *guides* it, highlighting the legal
//! entry edge so the Dervish player can see where the §9.322 arrival is allowed.
//! It is shown only during FoK, turn 1, the Dervish movement phase, and to the
//! player controlling the Dervish faction.

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout, hex_world_pos};
use omdurman_net::NetState;
use omdurman_rules::effects::GameState;
use omdurman_rules::{GameTurnIndex, Phase};
use omdurman_types::{HexCoord, Player, Scenario};

use crate::render::{HexOverlay, HexRingAssets};
use crate::{GameStateResource, PlayerFactions};

/// Marker for an entry-edge highlight ring so it can be cleared each frame.
#[derive(Component)]
pub struct FokEntryRing;

/// Whether the local player controls the Dervish (or is an unbound sandbox seat).
fn local_is_dervish(factions: &PlayerFactions, net: &NetState) -> bool {
    match factions.local(net) {
        Some(player) => player == Player::Dervish,
        None => factions.by_peer.is_empty(),
    }
}

/// Whether the §9.322 entry guide should be shown for the current state: FoK,
/// turn 1, Dervish movement phase.
fn entry_window_open(gs: &GameState) -> bool {
    gs.scenario == Scenario::FallOfKhartoum
        && gs.current_turn == GameTurnIndex::new(1)
        && gs.active_player == Player::Dervish
        && matches!(gs.phase, Phase::Movement)
}

/// The legal §9.322 entry hexes: the south and east edge of the playable board.
/// "Edge" hexes are those on the board's maximum-`r` (south) or maximum-`q`
/// (east) extent -- the side from which the Dervish historically advanced on
/// Khartoum. Empty when no board is loaded.
pub fn entry_edge_hexes(game_map: &GameMap) -> Vec<HexCoord> {
    let max_q = game_map.hexes.keys().map(|h| h.q).max();
    let max_r = game_map.hexes.keys().map(|h| h.r).max();
    let (Some(max_q), Some(max_r)) = (max_q, max_r) else {
        return Vec::new();
    };
    let mut out: Vec<HexCoord> = game_map
        .hexes
        .iter()
        // South/east edge, but only land hexes -- the Dervish entry units are
        // land units, so a Nile edge hex is not a legal entry point (and a green
        // ring sitting on the river just looks like a leftover).
        .filter(|(h, data)| (h.q == max_q || h.r == max_r) && data.terrain.passable_by_land())
        .map(|(h, _)| *h)
        .collect();
    out.sort_by_key(|h| (h.q, h.r));
    out
}

/// Highlight the legal Dervish entry edge (green) during the FoK turn-1 Dervish
/// movement phase (§9.322).
pub fn fok_entry_overlay_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    game_state: Option<Res<GameStateResource>>,
    factions: Res<PlayerFactions>,
    net: Res<NetState>,
    existing: Query<Entity, With<FokEntryRing>>,
) {
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = game_state else { return };
    if !entry_window_open(&gs.0) || !local_is_dervish(&factions, &net) {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for hex in entry_edge_hexes(&game_map) {
        let pos = hex_world_pos(hex, origin, &overlay.params);
        commands.spawn((
            FokEntryRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.green.clone()),
            Transform::from_xyz(pos.x, 1.4, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
}

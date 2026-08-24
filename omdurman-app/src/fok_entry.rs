//! Guided Dervish entry for FALL OF KHARTOUM (§9.322).
//!
//! On turn 1 the Dervish player's units enter "through any hexes on the south
//! or east edge of the map." Placement itself flows through the ordinary unit
//! picker (`PlaceUnit`); this module only *guides* it, highlighting the legal
//! entry edge so the Dervish player can see where the §9.322 arrival is allowed.
//! It is shown only during FoK, turn 1, the Dervish movement phase, and to the
//! player controlling the Dervish faction.
//!
//! The FoK board is diamond-shaped: the south edge is the bottom row (no hex
//! at `r+1`) and the east edge is the diagonal of rightmost hexes per row
//! (no hex at `q+1`).

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, hex_world_pos};
use omdurman_rules::effects::GameState;
use omdurman_rules::{GameTurnIndex, Phase};
use omdurman_types::{HexCoord, Player, Scenario};

use crate::peers::Peers;
use crate::GameStateResource;

/// Marker for an entry-edge highlight ring so it can be cleared each frame.
#[derive(Component)]
pub struct FokEntryRing;

/// Whether the local player controls the Dervish (or is an unbound seat).
fn local_is_dervish(peers: &Peers) -> bool {
    match peers.local() {
        Some(player) => player == Player::Dervish,
        None => !peers.any_assigned(),
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
///
/// The FoK board is diamond-shaped. The "south edge" is the bottom row (no
/// hex at `r+1`); the "east edge" is the diagonal of rightmost hexes per
/// row (no hex at `q+1`). Empty when no board is loaded.
pub fn entry_edge_hexes(game_map: &GameMap) -> Vec<HexCoord> {
    let mut out: Vec<HexCoord> = game_map
        .hexes
        .keys()
        .filter(|h| {
            let on_south = !game_map.hexes.contains_key(&HexCoord::new(h.q, h.r + 1));
            let on_east = !game_map.hexes.contains_key(&HexCoord::new(h.q + 1, h.r));
            (on_south || on_east)
                && game_map
                    .hexes
                    .get(h)
                    .is_some_and(|d| d.terrain.passable_by_land())
        })
        .copied()
        .collect();
    out.sort_by_key(|h| (h.q, h.r));
    out
}

/// Highlight the legal Dervish entry edge (green) during the FoK turn-1 Dervish
/// movement phase (§9.322).
pub fn fok_entry_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    game_map: Res<GameMap>,
    game_state: Option<Res<GameStateResource>>,
    peers: Peers,
    existing: Query<Entity, With<FokEntryRing>>,
) {
    let crate::HexRender { assets, layout, overlay } = hex;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = game_state else { return };
    if !entry_window_open(&gs.0) || !local_is_dervish(&peers) {
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

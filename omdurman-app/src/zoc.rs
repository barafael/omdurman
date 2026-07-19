//! Zone-of-control overlay: yellow hex rings on every hex that lies in an
//! enemy ZOC from the local player's perspective (§5.41, §5.44).
//!
//! Toggled at runtime via [`ZocOverlay`]; the overlay system runs every frame
//! while the toggle is on, rebuilding the ring set from the live game state.

use std::collections::HashSet;

use bevy::prelude::*;
use omdurman_hexmap::{HexLayout, hex_world_pos};
use omdurman_net::NetState;
use omdurman_rules::{effects::GameState, Phase};
use omdurman_types::{HexCoord, HexsideKind, Player, UnitKind};

use crate::{
    GameStateResource, PlayerFactions,
    render::{HexOverlay, HexRingAssets},
};

// -- Marker + toggle ---------------------------------------------------------

#[derive(Component)]
pub(crate) struct ZocRing;

/// Runtime toggle for the ZOC overlay. Flipped by a toolbar button; the overlay
/// system reads this each frame.
#[derive(Resource)]
pub struct ZocOverlay {
    pub visible: bool,
}

impl Default for ZocOverlay {
    fn default() -> Self {
        Self { visible: false }
    }
}

// -- Overlay system ----------------------------------------------------------

/// Spawn yellow hex rings on every hex in enemy ZOC. Runs every frame while
/// the toggle is on, despawning and rebuilding from scratch.
pub fn zoc_overlay_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    toggle: Res<ZocOverlay>,
    game_state: Option<Res<GameStateResource>>,
    factions: Res<PlayerFactions>,
    net: Res<NetState>,
    existing: Query<Entity, With<ZocRing>>,
    mut last_zoc: Local<Option<HashSet<HexCoord>>>,
) {
    let existing: Vec<Entity> = existing.iter().collect();

    if !toggle.visible {
        if !existing.is_empty() {
            crate::ui::despawn_all(&mut commands, &existing);
            *last_zoc = None;
        }
        return;
    }

    let Some(gs) = game_state else { return };
    if matches!(gs.0.phase, Phase::Setup) {
        if !existing.is_empty() {
            crate::ui::despawn_all(&mut commands, &existing);
            *last_zoc = None;
        }
        return;
    }

    let my_player = factions
        .local(&net)
        .unwrap_or(Player::AngloEgyptian);
    let enemy = my_player.opponent();

    let zoc_hexes = compute_enemy_zoc(&gs.0, enemy, my_player);

    if last_zoc.as_ref() == Some(&zoc_hexes) {
        return;
    }

    crate::ui::despawn_all(&mut commands, &existing);

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for hex in &zoc_hexes {
        let pos = hex_world_pos(*hex, origin, &overlay.params);
        commands.spawn((
            ZocRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.yellow.clone()),
            Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }

    *last_zoc = Some(zoc_hexes);
}

/// Compute the set of hexes in enemy ZOC from the perspective of `my_player`.
///
/// Mirrors the logic in [`GameState::hex_in_enemy_zoc`] but returns the full
/// set of ZOC hexes rather than testing a single hex.
pub(crate) fn compute_enemy_zoc(gs: &GameState, enemy: Player, my_player: Player) -> HashSet<HexCoord> {
    // Use Infantry as the reference mover kind: land-unit ZOC is the
    // superset of what most units experience (gunboat-only ZOC is niche).
    let mover_kind = UnitKind::Infantry;
    let mut zoc = HashSet::new();

    for unit in &gs.units {
        // Only enemy units project ZOC relevant to us.
        if unit.profile.identity.owner() != enemy {
            continue;
        }
        // Check the core ZOC projection rules (disruption, kind, owner).
        if gs.unit_projects_zoc(unit, my_player, mover_kind).is_none() {
            continue;
        }
        for neighbor in unit.position.neighbors() {
            // §5.44: ZOC does not cross a khor/wall/Zariba hexside.
            if gs.board.hexside_is(unit.position, neighbor, HexsideKind::blocks_zoc) {
                continue;
            }
            // §5.44: ZOC does not extend into or out of a Nile hex (exception:
            // gunboats, already filtered by `unit_projects_zoc`).
            if gs.board.is_nile(unit.position) || gs.board.is_nile(neighbor) {
                continue;
            }
            zoc.insert(neighbor);
        }
    }

    zoc
}

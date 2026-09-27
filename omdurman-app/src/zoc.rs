//! Zone-of-control overlay: yellow hex rings on every hex that lies in an
//! enemy ZOC from the local player's perspective (§5.41, §5.44).
//!
//! Toggled at runtime via [`ZocOverlay`]; the overlay system runs every frame
//! while the toggle is on, rebuilding the ring set from the live game state.

use std::collections::HashSet;

use bevy::prelude::*;
use omdurman_rules::{Phase, effects::GameState};
use omdurman_types::{HexCoord, Player, UnitKind};

use crate::GameStateResource;
use crate::peers::Peers;

// -- Marker + toggle ---------------------------------------------------------

#[derive(Component)]
pub(crate) struct ZocRing;

/// Runtime toggle for the ZOC overlay. Flipped by a toolbar button; the overlay
/// system reads this each frame.
#[derive(Resource, Default)]
pub struct ZocOverlay {
    pub visible: bool,
}

// -- Overlay system ----------------------------------------------------------

#[allow(clippy::too_many_arguments)]
/// Spawn yellow hex rings on every hex in enemy ZOC. Runs every frame while
/// the toggle is on, despawning and rebuilding from scratch.
///
/// In the spectator/replay view there is no local player, so the overlay
/// shows *both* sides' zones: yellow rings for ZOC projected by the Dervish,
/// blue rings for ZOC projected by the Anglo-Egyptians.
pub fn zoc_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    toggle: Res<ZocOverlay>,
    game_state: Res<GameStateResource>,
    peers: Peers,
    app_state: Res<State<crate::AppState>>,
    existing: Query<Entity, With<ZocRing>>,
    mut last_zoc: Local<Option<HashSet<HexCoord>>>,
    (generation, mut seen_generation): (Res<crate::picker::OverlayGeneration>, Local<u32>),
) {
    if generation.invalidates(&mut seen_generation) {
        *last_zoc = None;
    }
    let assets = &hex.assets;
    let existing: Vec<Entity> = existing.iter().collect();

    if !toggle.visible {
        if !existing.is_empty() {
            crate::ui::despawn_all(&mut commands, &existing);
            *last_zoc = None;
        }
        return;
    }

    let gs = game_state;
    if matches!(gs.0.phase, Phase::Setup) {
        if !existing.is_empty() {
            crate::ui::despawn_all(&mut commands, &existing);
            *last_zoc = None;
        }
        return;
    }

    // Which side(s) project the drawn ZOC, and with what ring colour:
    // live game -> the enemy of the local player (yellow, as before);
    // spectator -> both sides (Dervish yellow, Anglo-Egyptian blue).
    let spectating = **app_state == crate::AppState::Spectating;
    let projecting: Vec<(Player, Handle<StandardMaterial>)> = if spectating {
        vec![
            (Player::Dervish, assets.yellow.clone()),
            (Player::AngloEgyptian, assets.blue.clone()),
        ]
    } else {
        let my_player = peers.local().unwrap_or(Player::AngloEgyptian);
        vec![(my_player.opponent(), assets.yellow.clone())]
    };

    // The union drives the rebuild cache; per-side rings are rebuilt from
    // scratch whenever the union changes.
    let mut union = HashSet::new();
    let mut spawns: Vec<(HexCoord, Handle<StandardMaterial>)> = Vec::new();
    for (side, material) in &projecting {
        for hex in compute_enemy_zoc(&gs.0, *side, side.opponent()) {
            union.insert(hex);
            spawns.push((hex, material.clone()));
        }
    }

    if last_zoc.as_ref() == Some(&union) {
        return;
    }

    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing);
    for (hex, material) in spawns {
        rings.ring(ZocRing, hex, 1.5, 1.0, &material);
    }

    *last_zoc = Some(union);
}

/// Compute the set of hexes in enemy ZOC from the perspective of `my_player`.
///
/// The union of the engine's per-unit [`GameState::zoc_hexes`] -- the same
/// §5.44 rules as [`GameState::hex_in_enemy_zoc`], as a set.
pub(crate) fn compute_enemy_zoc(
    gs: &GameState,
    enemy: Player,
    my_player: Player,
) -> HashSet<HexCoord> {
    // Use Infantry as the reference mover kind: land-unit ZOC is the
    // superset of what most units experience (gunboat-only ZOC is niche).
    let mover_kind = UnitKind::Infantry {
        fire: 0,
        melee: 0,
        movement: 0,
    };
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
        // §5.44 extent (khor, Nile, fort, hut/building, wall/gate
        // direction): the engine's own rule.
        zoc.extend(gs.zoc_hexes(unit, my_player, mover_kind));
    }

    zoc
}

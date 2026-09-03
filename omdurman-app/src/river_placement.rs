//! Hex-click handler for river-mine and river-chain placement (§10.11, §10.21).
//!
//! During the Setup phase, the Dervish player may place up to two river mines on
//! Nile hexes, or a single river chain spanning up to four contiguous Nile hexes.
//! This system checks for a hex click and emits the corresponding `GameEffect`.

use bevy::prelude::*;

use crate::input::CombatClickCtx;
use crate::ui_plugin::OptionalRulePlacement;
use crate::{GameStateResource, PendingEdits};
use omdurman_net::GameEvent;

/// Emits `PlaceMine` or `PlaceChain` effects when the Dervish player clicks a
/// Nile hex during Setup with the placement UI active. (Phase gate: the
/// `in_setup_phase` run condition on registration; see `ui_phase_state`.)
pub(crate) fn handle_optional_rule_click(
    mut click: CombatClickCtx,
    game_state: Option<Res<GameStateResource>>,
    mut placement: ResMut<OptionalRulePlacement>,
    mut pending: ResMut<PendingEdits>,
) {
    let Some(gs) = game_state else { return };

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

        pending.submit_game(GameEvent::Effect(
            omdurman_rules::effects::GameEffect::PlaceMine { hex },
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
            pending.submit_game(GameEvent::Effect(
                omdurman_rules::effects::GameEffect::PlaceChain {
                    hexes: std::mem::take(&mut placement.chain_hexes),
                },
            ));
            placement.placing_chain = false;
        }
    }
}

/// Board markers for placed river mines (§10.11) and the river chain
/// (§10.21). §10.11 says the mines are *secretly recorded*, so the overlay
/// is shown only to the Dervish seat (and unbound seats); the chain is a
/// Dervish obstruction, shown to the same audience.
pub(crate) fn mine_chain_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    game_state: Option<Res<GameStateResource>>,
    peers: crate::peers::Peers,
    existing: Query<Entity, With<MineChainMarker>>,
) {
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    let Some(gs) = game_state else { return };
    match peers.local() {
        Some(omdurman_types::Player::Dervish) | None => {}
        Some(_) => return,
    }
    if gs.0.mines.is_empty() && gs.0.chain.is_none() {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;

    // Mines: compact red rings on their Nile hexes.
    for mine in &gs.0.mines {
        let pos = omdurman_hexmap::hex_world_pos(mine.hex, origin, &overlay.params);
        commands.spawn((
            MineChainMarker,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.red.clone()),
            Transform::from_xyz(pos.x, 0.7, pos.z).with_scale(Vec3::splat(size * 0.35)),
            Visibility::Visible,
        ));
    }

    // Chain: grey bars spanning consecutive chain-hex centres.
    let Some(chain) = &gs.0.chain else { return };
    for pair in chain.hexes.windows(2) {
        let a = omdurman_hexmap::hex_world_pos(pair[0], origin, &overlay.params);
        let b = omdurman_hexmap::hex_world_pos(pair[1], origin, &overlay.params);
        let mid = (a + b) * 0.5;
        let len = a.distance(b).max(0.001);
        let dir = (b - a) / len;
        let angle = (-dir.z).atan2(dir.x);
        commands.spawn((
            MineChainMarker,
            Mesh3d(assets.unit_square.clone()),
            MeshMaterial3d(assets.gray.clone()),
            Transform::from_translation(Vec3::new(mid.x, 0.7, mid.z))
                .with_rotation(
                    Quat::from_rotation_y(angle)
                        * Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
                )
                .with_scale(Vec3::new(len, size * 0.12, 1.0)),
            Visibility::Visible,
        ));
    }
}

/// Marker component for mine/chain board markers.
#[derive(Component)]
pub(crate) struct MineChainMarker;

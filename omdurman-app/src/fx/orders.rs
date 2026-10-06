//! The local player's orders: a refused click flashes its hex, and a
//! committed move keeps faint dots along its route until its echo arrives,
//! so a sent order never looks ignored.

use std::collections::{HashMap, HashSet};

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use omdurman_hexmap::hex_world_pos;
use omdurman_net::GameEvent;
use omdurman_types::HexCoord;

use super::FxRequest;
use super::transient::{FX_HEIGHT, FxAssets};

/// A plotted leg the picker refused (unaffordable, blocked, over the stacking
/// cap) flashes where the player clicked: the click did register.
pub(super) fn flash_refused_legs(
    mut path: ResMut<crate::picker::MovementPath>,
    mut fx: MessageWriter<FxRequest>,
) {
    if path.refused.is_none() {
        return;
    }
    if let Some(hex) = path.refused.take() {
        fx.write(FxRequest::Refused { hex });
    }
}

/// A dot of a committed move's route, shown until the move's echo arrives.
#[derive(Component)]
pub(super) struct TrailDot;

/// When each pending move was first seen, and which ones have dots.
#[derive(Default)]
pub(super) struct TrailState {
    first_seen: HashMap<u64, f64>,
    shown: Vec<u64>,
}

/// How long a submission must wait for its echo before its dots show:
/// offline and on a fast link the echo beats it, and nothing flickers.
const TRAIL_DELAY_SECS: f64 = 0.25;

/// Faint dots along each committed move whose echo has not come back yet;
/// they go once the engine accepts it (and the counter glides along the same
/// route) or rejects it. Static: they only say "on its way".
pub(super) fn pending_order_trail(
    (time, pending): (Res<Time>, Res<crate::PendingEdits>),
    assets: Option<Res<FxAssets>>,
    board: crate::BoardGeometry,
    dots: Query<Entity, With<TrailDot>>,
    mut commands: Commands,
    mut activity: ResMut<crate::activity::Activity>,
    mut trail: Local<TrailState>,
) {
    let TrailState { first_seen, shown } = &mut *trail;
    let now = time.elapsed_secs_f64();
    let moves: Vec<(u64, &GameEvent)> = pending
        .unconfirmed
        .iter()
        .filter(|(_, ev)| matches!(ev, GameEvent::MoveUnit { .. }))
        .map(|(uid, ev)| (*uid, ev))
        .collect();
    let live: HashSet<u64> = moves.iter().map(|(uid, _)| *uid).collect();
    first_seen.retain(|uid, _| live.contains(uid));
    for (uid, _) in &moves {
        first_seen.entry(*uid).or_insert(now);
    }
    let due: Vec<u64> = moves
        .iter()
        .map(|(uid, _)| *uid)
        .filter(|uid| {
            first_seen
                .get(uid)
                .is_some_and(|t| now - t >= TRAIL_DELAY_SECS)
        })
        .collect();
    if due.len() < moves.len() {
        // Waiting out the delay: the dots must show on time.
        activity.keep_running();
    }
    if *shown == due && !board.layout.is_changed() && !board.overlay.is_changed() {
        return;
    }
    for entity in &dots {
        commands.entity(entity).despawn();
    }
    *shown = due.clone();
    let Some(assets) = assets.as_ref() else {
        return;
    };
    let origin = board.layout.adjusted_origin(&board.overlay.params);
    let size = board.overlay.params.hex_size;
    for (uid, event) in &moves {
        let GameEvent::MoveUnit {
            to_q, to_r, path, ..
        } = event
        else {
            continue;
        };
        if !due.contains(uid) {
            continue;
        }
        let route: Vec<HexCoord> = if path.is_empty() {
            vec![HexCoord::new(*to_q, *to_r)]
        } else {
            path.clone()
        };
        for hex in route {
            let p = hex_world_pos(hex, origin, &board.overlay.params);
            commands.spawn((
                TrailDot,
                Mesh3d(assets.fill.clone()),
                MeshMaterial3d(assets.trail.clone()),
                Transform::from_xyz(p.x, FX_HEIGHT, p.z).with_scale(Vec3::splat(size * 0.1)),
                Visibility::Visible,
            ));
        }
    }
}

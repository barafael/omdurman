use bevy::prelude::*;
use bevy_matchbox::prelude::{MatchboxSocket, PeerId};
use omdurman_net::{NetState, open_socket, Ephemeral, NetMsg};
use crate::{EditorMode, RtsCamera, CursorBroadcastTimer, CursorPositions, LocalFaction, PendingEdits, util};

/// Registers all networking-domain resources and systems: socket lifecycle,
/// message processing, cursor/lobby broadcast, game recording, and peer
/// management.
pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app
            // ── Resources ──────────────────────────────────────────────
            .insert_resource(NetState::default())
            .insert_resource(crate::PendingEdits::default())
            .insert_resource(crate::PendingIncoming::default())
            .insert_resource(crate::CursorPositions::default())
            .insert_resource(crate::CursorBroadcastTimer::default())
            .insert_resource(crate::PlayerFactions::default())
            .insert_resource(crate::LobbyChoices::default())
            .insert_resource(crate::LocalFaction::default())
            .insert_resource(crate::LobbyScenario::default())
            .insert_resource(crate::AppliedEvents::default())
            // ── Startup ────────────────────────────────────────────────
            .add_systems(Startup, (
                open_socket,
            ))
            // ── Update ─────────────────────────────────────────────────
            .add_systems(Update, (
                crate::handle_reconnect,
                crate::retry_snapshot_request.after(crate::handle_reconnect),
                crate::handle_socket.after(crate::handle_reconnect),
                crate::events::drain_applied_events.after(crate::handle_socket),
                crate::apply_ephemeral.after(crate::apply_pending_placement),
                crate::game_record::init_game_record.after(crate::handle_socket),
                crate::game_record::host_emit_annotations
                    .after(crate::game_record::init_game_record)
                    .before(crate::flush_pending),
                crate::game_record::flush_game_record.after(crate::handle_socket),
                send_player_info_on_connect.after(crate::handle_socket),
                prune_disconnected_peers.after(crate::handle_socket),
                broadcast_cursor,
                crate::broadcast_browser_selection,
                crate::flush_pending,
                crate::sync_lobby_appstate,
            ));
    }
}

pub(crate) fn broadcast_cursor(
    mut timer: ResMut<CursorBroadcastTimer>,
    time: Res<Time>,
    mode: Res<State<EditorMode>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    if !crate::map_mode_active(**mode) {
        return;
    }
    let Some(hit) = util::raycast_ground(&windows, &cameras) else {
        return;
    };
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };
    omdurman_net::broadcast_unreliable(
        &mut socket,
        &net.peers,
        &NetMsg::Ephemeral(Ephemeral::CursorPos { x: hit.x, y: hit.z }),
    );
}

/// Clean stale cursor positions and player info for disconnected peers.
pub(crate) fn prune_disconnected_peers(
    net: Res<NetState>,
    mut cursor_positions: ResMut<CursorPositions>,
    mut player_info: ResMut<crate::settings::PlayerInfoMap>,
) {
    let active: Vec<PeerId> = net.peers.to_vec();
    cursor_positions.current.retain(|&p, _| active.contains(&p));
    cursor_positions
        .previous
        .retain(|&p, _| active.contains(&p));
    cursor_positions
        .last_update
        .retain(|&p, _| active.contains(&p));
    cursor_positions.display.retain(|&p, _| active.contains(&p));
    player_info.peers.retain(|&p, _| active.contains(&p));
}

/// Send our PlayerInfo to every connected peer once.
pub(crate) fn send_player_info_on_connect(
    net: Res<NetState>,
    local: Res<crate::settings::LocalPlayerSettings>,
    local_faction: Res<LocalFaction>,
    mut pending: ResMut<PendingEdits>,
    mut notified: Local<Vec<PeerId>>,
) {
    for &peer in &net.peers {
        if !notified.contains(&peer) {
            notified.push(peer);
            let (r, g, b) = local.color_u8();
            pending.outgoing_targeted.push((
                NetMsg::Ephemeral(Ephemeral::PlayerInfo {
                    name: local.name.clone(),
                    color_r: r,
                    color_g: g,
                    color_b: b,
                }),
                peer,
            ));
            pending.outgoing_targeted.push((
                NetMsg::Ephemeral(Ephemeral::FactionChoice(local_faction.0)),
                peer,
            ));
        }
    }
}

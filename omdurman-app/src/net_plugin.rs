use crate::{CursorBroadcastTimer, CursorPositions, LobbyChoices, LocalFaction, PendingEdits, PendingIncoming, RtsCamera, util};
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_matchbox::prelude::{MatchboxSocket, PeerId};
use omdurman_net::{CH_RELIABLE, Ephemeral, NetMsg, NetState, enc_msg, open_socket};
use omdurman_types::{SectionName, SpriteRef};

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
            .add_systems(Startup, (open_socket,))
            // ── Update ─────────────────────────────────────────────────
            .add_systems(
                Update,
                (
                    crate::events::drain_applied_events.after(crate::net_socket::handle_socket),
                    apply_ephemeral.after(crate::apply_pending_placement),
                    crate::game_record::init_game_record.after(crate::net_socket::handle_socket),
                    crate::game_record::host_emit_annotations
                        .after(crate::game_record::init_game_record)
                        .before(flush_pending),
                    crate::game_record::flush_game_record.after(crate::net_socket::handle_socket),
                    send_player_info_on_connect.after(crate::net_socket::handle_socket),
                    prune_disconnected_peers.after(crate::net_socket::handle_socket),
                    broadcast_cursor.run_if(crate::map_mode_active_state),
                    broadcast_browser_selection,
                    flush_pending,
                ),
            );
    }
}

pub(crate) fn broadcast_cursor(
    mut timer: ResMut<CursorBroadcastTimer>,
    time: Res<Time>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
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

pub(crate) fn apply_ephemeral(
    mut incoming: ResMut<PendingIncoming>,
    mut player_info: ResMut<crate::settings::PlayerInfoMap>,
    mut cursor_positions: ResMut<CursorPositions>,
    mut event_viewer: Option<ResMut<crate::event_viewer::EventViewerState>>,
    mut lobby_choices: ResMut<LobbyChoices>,
    time: Res<Time>,
) {
    for (eph, peer) in incoming.ephemeral.drain(..) {
        match eph {
            Ephemeral::PlayerInfo {
                name,
                color_r,
                color_g,
                color_b,
            } => {
                player_info.peers.insert(
                    peer,
                    crate::settings::PeerPlayerInfo {
                        name,
                        color: egui::Color32::from_rgb(color_r, color_g, color_b),
                    },
                );
            }
            Ephemeral::CursorPos { x, y } => {
                let pos = Vec2::new(x, y);
                let prev = cursor_positions.current.get(&peer).copied().unwrap_or(pos);
                cursor_positions.previous.insert(peer, prev);
                cursor_positions.current.insert(peer, pos);
                cursor_positions
                    .last_update
                    .insert(peer, time.elapsed_secs_f64());
            }
            Ephemeral::EventViewerSelect(idx) => {
                if let Some(ref mut viewer) = event_viewer {
                    viewer.selected = if idx < 0 { None } else { Some(idx as usize) };
                }
            }
            Ephemeral::BrowserSelect { .. } => {}
            Ephemeral::FactionChoice(faction) => {
                lobby_choices.by_peer.insert(peer, faction);
            }
            Ephemeral::ScenarioChoice(scenario) => {
                lobby_choices.scenario = Some(scenario);
            }
        }
    }
}

pub(crate) fn broadcast_browser_selection(
    browser: Res<crate::browser::SpriteBrowser>,
    mut last: Local<Option<(SectionName, u32, u32)>>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    let current = browser
        .selected_sprite
        .as_ref()
        .map(|s| (s.section_name, s.col, s.row));
    if current == *last {
        return;
    }
    *last = current;
    let Some((section_name, col, row)) = current else {
        return;
    };
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };
    omdurman_net::broadcast_unreliable(
        &mut socket,
        &net.peers,
        &NetMsg::Ephemeral(Ephemeral::BrowserSelect {
            sprite: SpriteRef {
                section_name,
                col,
                row,
            },
        }),
    );
}

pub(crate) fn flush_pending(
    mut pending: ResMut<PendingEdits>,
    mut incoming: ResMut<PendingIncoming>,
    mut net: ResMut<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    if pending.outgoing_broadcast.is_empty() && pending.outgoing_targeted.is_empty() {
        return;
    }

    let i_sequence = net.is_host || net.peers.is_empty();
    let host = net.host_id();

    let staged: Vec<NetMsg> = std::mem::take(&mut pending.outgoing_broadcast);
    let mut to_broadcast: Vec<NetMsg> = Vec::new();
    let mut retained_broadcast: Vec<NetMsg> = Vec::new();

    let mut socket = socket_q.single_mut().ok();

    for msg in staged {
        match msg {
            NetMsg::Game(event) if i_sequence => {
                let seq = net.next_seq;
                net.next_seq += 1;
                let sequenced = NetMsg::Sequenced { seq, event };
                incoming.loopback.push(sequenced.clone());
                to_broadcast.push(sequenced);
            }
            NetMsg::Game(event) => {
                let submission = NetMsg::Game(event);
                let sent = match (host, socket.as_deref_mut()) {
                    (Some(host), Some(socket)) => socket
                        .channel_mut(CH_RELIABLE)
                        .try_send(enc_msg(&submission), host)
                        .inspect_err(|e| warn!(error = %e, "submit to host failed; will retry"))
                        .is_ok(),
                    _ => false,
                };
                if !sent {
                    retained_broadcast.push(submission);
                }
            }
            other => to_broadcast.push(other),
        }
    }

    let targeted: Vec<(NetMsg, PeerId)> = std::mem::take(&mut pending.outgoing_targeted);
    let mut retained_targeted: Vec<(NetMsg, PeerId)> = Vec::new();
    for (msg, peer) in targeted {
        let sent = match socket.as_deref_mut() {
            Some(socket) => socket
                .channel_mut(CH_RELIABLE)
                .try_send(enc_msg(&msg), peer)
                .inspect_err(|e| warn!(error = %e, "reliable targeted send failed; will retry"))
                .is_ok(),
            None => false,
        };
        if !sent {
            retained_targeted.push((msg, peer));
        }
    }

    for msg in to_broadcast {
        if net.peers.is_empty() {
            if !matches!(msg, NetMsg::Sequenced { .. }) {
                retained_broadcast.push(msg);
            }
            continue;
        }
        let Some(socket) = socket.as_deref_mut() else {
            retained_broadcast.push(msg);
            continue;
        };
        let encoded = enc_msg(&msg);
        let channel = socket.channel_mut(CH_RELIABLE);
        let mut all_ok = true;
        for &peer in &net.peers {
            if let Err(e) = channel.try_send(encoded.clone(), peer) {
                warn!(error = %e, "reliable broadcast send failed; will retry");
                all_ok = false;
            }
        }
        if !all_ok {
            retained_broadcast.push(msg);
        }
    }

    pending.outgoing_broadcast = retained_broadcast;
    pending.outgoing_targeted = retained_targeted;
}

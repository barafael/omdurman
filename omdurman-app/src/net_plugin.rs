use crate::{LobbyChoices, LocalFaction, util};
use crate::camera::RtsCamera;
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_matchbox::prelude::{MatchboxSocket, PeerId};
use omdurman_net::{
    CH_RELIABLE, Ephemeral, GameEvent, NetMsg, NetState, enc_msg, open_socket,
};
use omdurman_types::{Player, SectionName, SpriteRef};
use std::collections::HashMap;

// -- Net resources (moved from main.rs) -------------------------------------

/// Tracks whether the game has begun (set by the host's `StartGame`). Used by
/// the snapshot / host-failover paths in `net_socket`. The turn itself lives in
/// the rules engine (`GameState.active_player` / `phase`), advanced by the
/// `End Phase` button -- there is no separate app-level turn counter.
#[derive(Resource, Default)]
pub(crate) struct TurnState {
    pub game_started: bool,
}

/// Authoritative per-player faction binding, established by the host's
/// `GameEvent::StartGame` (§lobby). Keyed by `PeerId`; the local player's
/// faction is `factions.get(&net.my_id)`.
#[derive(Resource, Default)]
pub struct PlayerFactions {
    pub by_peer: HashMap<PeerId, Player>,
}

impl PlayerFactions {
    /// The faction the local peer commands, if assigned.
    pub fn local(&self, net: &NetState) -> Option<Player> {
        net.my_id.and_then(|id| self.by_peer.get(&id).copied())
    }

    /// Whether the local player may act right now: their faction is the rules
    /// engine's active player. Before any binding exists (no lobby) this returns
    /// `true` so the game stays playable. (§lobby)
    pub fn local_may_act(&self, net: &NetState, active: Player) -> bool {
        match self.local(net) {
            Some(mine) => mine == active,
            // No local faction: either an unbound session (empty binding -> may
            // drive both sides) or a spectator (non-empty binding, not in it ->
            // never acts). See `local_is_spectator`.
            None => self.by_peer.is_empty(),
        }
    }

    /// Whether the local peer is a spectator: a faction binding exists (the game
    /// started with assigned players) but this peer isn't in it, so it joined to
    /// watch only. A spectator may never place, move, or fight -- distinct from
    /// an unbound session (empty binding), which may drive both sides.
    pub fn local_is_spectator(&self, net: &NetState) -> bool {
        !self.by_peer.is_empty() && self.local(net).is_none()
    }

    /// Re-bind the local player's faction to its *current* `PeerId` after a
    /// reconnect. A dropped-and-reconnected peer is re-issued a fresh `PeerId`,
    /// so the binding recorded under its old id no longer matches `my_id` and
    /// [`Self::local`] returns `None` -- the player would silently become a
    /// spectator of their own game. If the local player still knows the faction
    /// it picked (`local_faction`, which is local state and survives the
    /// reconnect) and that faction is present in the binding under some other
    /// (now-stale) id, move it onto `my_id`. Returns `true` if a re-bind
    /// happened. Faction is the durable player identity here: there are exactly
    /// two playable sides, so reclaiming "my" faction is unambiguous.
    pub fn rebind_local_after_reconnect(
        &mut self,
        net: &NetState,
        local_faction: Option<Player>,
    ) -> bool {
        let (Some(my_id), Some(mine)) = (net.my_id, local_faction) else {
            return false;
        };
        // Already correctly bound -- nothing to do.
        if self.by_peer.get(&my_id) == Some(&mine) {
            return false;
        }
        // Find the stale id currently holding my faction and, importantly, make
        // sure that id is no longer a live peer (else we'd steal an active
        // player's binding). A stale id is one not present in `net.peers` and
        // not our own current id.
        let stale = self
            .by_peer
            .iter()
            .find(|(id, f)| {
                **f == mine && **id != my_id && !net.peers.contains(id) && net.my_id != Some(**id)
            })
            .map(|(id, _)| *id);
        if let Some(stale) = stale {
            self.by_peer.remove(&stale);
            self.by_peer.insert(my_id, mine);
            true
        } else {
            false
        }
    }
}

/// Holds remote cursor positions in world space (`Vec2(world.x, world.z)`,
/// i.e. the cursor's hit point on the ground plane). Each peer renders these
/// using their own camera so a pitched / panned / zoomed view stays consistent.
#[derive(Resource, Default)]
pub struct CursorPositions {
    pub current: HashMap<PeerId, Vec2>,
    pub previous: HashMap<PeerId, Vec2>,
    pub last_update: HashMap<PeerId, f64>,
    /// Per-frame exponentially-smoothed world-space position.
    pub display: HashMap<PeerId, Vec2>,
}

/// Throttle cursor-position broadcasts to ~10 Hz.
#[derive(Resource)]
pub(crate) struct CursorBroadcastTimer(Timer);

impl Default for CursorBroadcastTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

/// Frame-scoped staging buffer for reliable outbound messages.
///
/// Why a buffer at all if matchbox channels already queue? Two reasons:
/// (1) systems can stage messages without taking `&mut MatchboxSocket`, which
///     would conflict with other socket-using systems; (2) `flush_pending`
///     routes the host's own `NetMsg::Game` entries through `incoming.loopback`
///     (still unsequenced) so `handle_socket` sequences them through the same
///     arm as guest submissions -- a single serialization point. Recording
///     happens via `GameRecorder::push_event` on the `NetMsg::Sequenced` echo,
///     so the host records on echo exactly like every other peer.
///
/// Unreliable messages (cursors, ephemeral UI selections) bypass this and go
/// straight to the socket via `omdurman_net::broadcast_unreliable`.
#[derive(Resource, Default)]
pub struct PendingEdits {
    /// Reliable broadcast to all peers.
    pub outgoing_broadcast: Vec<NetMsg>,
    /// Reliable send to a single peer.
    pub outgoing_targeted: Vec<(NetMsg, PeerId)>,
}

#[derive(Resource, Default)]
pub struct PendingIncoming {
    /// `PlaceUnit` / `MoveUnit` events received live -- recorded by
    /// `apply_pending_placement` and applied to the world. Other game
    /// events are applied inline by `handle_socket`; these two are deferred
    /// because they need access to the picker + mesh/material asset pools.
    /// The `Option<u8>` is the pre-computed sender index.
    pub live: Vec<(GameEvent, PeerId, Option<u8>)>,
    /// Same kind of events but injected from a `GameHistory` replay --
    /// already in the canonical event log, so must NOT be re-recorded.
    pub replay: Vec<(GameEvent, PeerId)>,
    /// Ephemeral display messages buffered by `handle_socket` for
    /// `apply_pending_placement` to apply (cursor positions need access
    /// to the `Window` resource for normalisation, player info needs the
    /// `PlayerInfoMap` resource).
    pub ephemeral: Vec<(Ephemeral, PeerId)>,
    /// Host-only: `NetMsg::Sequenced` events the host just assigned a sequence
    /// number to, queued to be fed back through its own receive path so the
    /// host applies and records them in the same canonical order as everyone
    /// else. Drained at the top of `handle_socket` each frame.
    pub loopback: Vec<NetMsg>,
}

// -- NetPlugin --------------------------------------------------------------

/// Registers all networking-domain resources and systems: socket lifecycle,
/// message processing, cursor/lobby broadcast, game recording, and peer
/// management.
pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app
            // -- Resources ----------------------------------------------
            .insert_resource(NetState::default())
            .insert_resource(PendingEdits::default())
            .insert_resource(PendingIncoming::default())
            .insert_resource(CursorPositions::default())
            .insert_resource(CursorBroadcastTimer::default())
            .insert_resource(PlayerFactions::default())
            .insert_resource(crate::LobbyChoices::default())
            .insert_resource(crate::LocalFaction::default())
            .insert_resource(crate::LocalSpectator::default())
            .insert_resource(crate::LobbyScenario::default())
            .insert_resource(crate::AppliedEvents::default())
            .insert_resource(crate::events::PendingObservations::default())
            .insert_resource(TurnState::default())
            .insert_resource(crate::LobbyTab::default())
            // -- Startup ------------------------------------------------
            // Offline dev mode (OMDURMAN_OFFLINE): skip the matchbox socket and
            // self-host, so a single instance is authoritative and playable
            // without a signalling server (used for headless verification).
            .add_systems(Startup, open_socket.run_if(|| !offline_mode()))
            .add_systems(Startup, setup_offline.run_if(offline_mode))
            // -- Update -------------------------------------------------
            .add_systems(
                Update,
                (
                    crate::events::drain_applied_events.after(crate::net_socket::handle_socket),
                    crate::events::drain_observations.after(crate::net_socket::handle_socket),
                    apply_ephemeral.after(crate::apply_pending_placement),
                    crate::game_record::init_game_record.after(crate::net_socket::handle_socket),
                    crate::game_record::host_emit_annotations
                        .after(crate::game_record::init_game_record)
                        .before(flush_pending),
                    crate::game_record::flush_game_record.after(crate::net_socket::handle_socket),
                    send_player_info_on_connect.after(crate::net_socket::handle_socket),
                    rebind_faction_after_reconnect.after(crate::net_socket::handle_socket),
                    prune_disconnected_peers.after(crate::net_socket::handle_socket),
                    broadcast_cursor.run_if(crate::map_view_active),
                    broadcast_browser_selection,
                    flush_pending,
                ),
            );
    }
}

/// Dev offline mode: `OMDURMAN_OFFLINE` set to any value. Skips the matchbox
/// socket so a single instance runs authoritatively without a signalling
/// server -- used for headless screenshot verification of play-view features.
pub(crate) fn offline_mode() -> bool {
    std::env::var("OMDURMAN_OFFLINE").is_ok()
}

/// Make this instance a self-contained host when offline: assign a fixed local
/// `PeerId` and mark it host, so events flow through the host loopback/apply
/// path with no peers. No socket is spawned, so all `MatchboxSocket` queries
/// simply no-op.
fn setup_offline(mut net: ResMut<NetState>) {
    net.my_id = Some(PeerId(uuid::Uuid::nil()));
    net.is_host = true;
    info!("offline mode: self-hosting without a matchbox socket");
}

pub(crate) fn broadcast_cursor(
    mut timer: ResMut<CursorBroadcastTimer>,
    time: Res<Time>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    net: Res<NetState>,
    socket: Option<ResMut<MatchboxSocket>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    let Some(hit) = util::raycast_ground(&windows, &cameras) else {
        return;
    };
    let Some(mut socket) = socket else {
        return;
    };
    omdurman_net::broadcast_unreliable(
        &mut socket,
        &net.peers,
        &NetMsg::Ephemeral(Ephemeral::CursorPos { pos: [hit.x, hit.z] }),
    );
}

/// Re-attach the local player's faction to its current `PeerId` after a
/// reconnect (see [`PlayerFactions::rebind_local_after_reconnect`]).
/// Runs every frame but is a cheap no-op unless the local binding is actually
/// stale, so it self-heals the "reconnected as a spectator of my own game" bug.
pub(crate) fn rebind_faction_after_reconnect(
    net: Res<NetState>,
    local_faction: Res<LocalFaction>,
    mut factions: ResMut<PlayerFactions>,
) {
    // Only meaningful once a game has bound factions.
    if factions.by_peer.is_empty() {
        return;
    }
    if factions.rebind_local_after_reconnect(&net, local_faction.0) {
        info!("re-bound local faction to current PeerId after reconnect");
    }
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
    local_spectator: Res<crate::LocalSpectator>,
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
                    color: [r, g, b],
                }),
                peer,
            ));
            pending.outgoing_targeted.push((
                NetMsg::Ephemeral(Ephemeral::FactionChoice(local_faction.0)),
                peer,
            ));
            pending.outgoing_targeted.push((
                NetMsg::Ephemeral(Ephemeral::SpectatorChoice(local_spectator.0)),
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
                color: [cr, cg, cb],
            } => {
                player_info.peers.insert(
                    peer,
                    crate::settings::PeerPlayerInfo {
                        name,
                        color: egui::Color32::from_rgb(cr, cg, cb),
                    },
                );
            }
            Ephemeral::CursorPos { pos: [cx, cy] } => {
                let pos = Vec2::new(cx, cy);
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
            Ephemeral::SpectatorChoice(spectating) => {
                if spectating {
                    lobby_choices.spectators.insert(peer);
                } else {
                    lobby_choices.spectators.remove(&peer);
                }
            }
        }
    }
}

pub(crate) fn broadcast_browser_selection(
    browser: Res<crate::browser::SpriteBrowser>,
    mut last: Local<Option<(SectionName, u32, u32)>>,
    net: Res<NetState>,
    socket: Option<ResMut<MatchboxSocket>>,
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
    let Some(mut socket) = socket else {
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
    net: Res<NetState>,
    mut socket: Option<ResMut<MatchboxSocket>>,
) {
    if pending.outgoing_broadcast.is_empty() && pending.outgoing_targeted.is_empty() {
        return;
    }

    let broadcast_count = pending.outgoing_broadcast.len();
    let targeted_count = pending.outgoing_targeted.len();
    if broadcast_count > 0 || targeted_count > 0 {
        debug!(
            broadcast = broadcast_count,
            targeted = targeted_count,
            is_host = net.is_host,
            peers = net.peers.len(),
            "flushing pending outbound messages"
        );
    }

    let i_sequence = net.is_host || net.peers.is_empty();
    let host = net.host_id();

    let staged: Vec<NetMsg> = std::mem::take(&mut pending.outgoing_broadcast);
    let mut to_broadcast: Vec<NetMsg> = Vec::new();
    let mut retained_broadcast: Vec<NetMsg> = Vec::new();

    for msg in staged {
        match msg {
            NetMsg::Game(event) if i_sequence => {
                // The sequencer's *own* game events are not sequenced here.
                // They are looped back unsequenced so `handle_socket` assigns
                // their `seq` through the *same* arm that sequences guest
                // submissions -- a single serialization point. Assigning `seq`
                // in two systems made host-own vs guest-relayed ordering depend
                // on intra-frame system scheduling; routing both through
                // `handle_socket` removes that nondeterminism.
                incoming.loopback.push(NetMsg::Game(event));
            }
            NetMsg::Game(event) => {
                let submission = NetMsg::Game(event);
                let sent = match (host, enc_msg(&submission), socket.as_deref_mut()) {
                    (Some(host), Some(encoded), Some(socket)) => socket
                        .channel_mut(CH_RELIABLE)
                        .try_send(encoded, host)
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
        let sent = match (enc_msg(&msg), socket.as_deref_mut()) {
            (Some(encoded), Some(socket)) => socket
                .channel_mut(CH_RELIABLE)
                .try_send(encoded, peer)
                .inspect_err(|e| warn!(error = %e, "reliable targeted send failed; will retry"))
                .is_ok(),
            _ => false,
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
        let Some(encoded) = enc_msg(&msg) else {
            // Encoding failed (OOM) -- retain for a later retry rather than
            // emit an empty packet that WebRTC may silently drop.
            retained_broadcast.push(msg);
            continue;
        };
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

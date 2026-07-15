use crate::{
    AppState, GameStateParams, PendingEdits, PendingIncoming, ReconnectRoom, TurnState, browser,
    editor, game_apply, game_record, map_kind_for_scenario, picker,
    rebuild_state_to, render, units,
};
use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use omdurman_hexmap::GameMap;
use omdurman_net::{
    CH_RELIABLE, CH_UNRELIABLE, Control, Ephemeral, GameEvent, NetMsg, NetState, RoomId, decode,
};

#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct SocketContext<'w> {
    pub overlay: ResMut<'w, render::HexOverlay>,
    pub browser: ResMut<'w, browser::SpriteBrowser>,
    pub editor: ResMut<'w, editor::HexEditor>,
    pub incoming: ResMut<'w, PendingIncoming>,
    pub annotations: Option<ResMut<'w, browser::SpriteAnnotationsResource>>,
    pub viewer: ResMut<'w, units::UnitViewer>,
    pub recorder: ResMut<'w, game_record::GameRecorder>,
}

pub struct NetSocketPlugin;

impl Plugin for NetSocketPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_reconnect, retry_snapshot_request, handle_socket),
        );
    }
}

pub(crate) fn handle_reconnect(
    mut commands: Commands,
    reconnect: Option<ResMut<ReconnectRoom>>,
    mut net: ResMut<NetState>,
    mut turn: ResMut<TurnState>,
    mut pending: ResMut<PendingEdits>,
    mut incoming: ResMut<PendingIncoming>,
    mut recorder: ResMut<game_record::GameRecorder>,
    mut room: ResMut<RoomId>,
    mut next_state: ResMut<NextState<AppState>>,
    mut picker: ResMut<picker::UnitPicker>,
    mut picker_state: ResMut<picker::PickerState>,
    placed_unit_q: Query<Entity, With<picker::PlacedUnit>>,
    socket_q: Query<Entity, With<MatchboxSocket>>,
) {
    let Some(reconnect) = reconnect else { return };
    let new_room = reconnect.0.clone();

    if new_room.is_empty() {
        commands.remove_resource::<ReconnectRoom>();
        return;
    }

    info!(%new_room, "reconnecting");

    // -- despawn old socket --
    if let Ok(entity) = socket_q.single() {
        commands.entity(entity).despawn();
    }

    // -- reset state --
    *net = NetState::default();
    *turn = TurnState::default();
    pending.outgoing_broadcast.clear();
    pending.outgoing_targeted.clear();
    incoming.live.clear();
    incoming.replay.clear();
    incoming.ephemeral.clear();
    incoming.loopback.clear();
    *recorder = game_record::GameRecorder::default();

    // -- despawn placed units and restore full picker roster --
    for entity in &placed_unit_q {
        commands.entity(entity).despawn();
    }
    picker.reset_available();
    *picker_state = picker::PickerState::Idle;

    // -- update room id and URL --
    *room = RoomId::new(new_room.clone());

    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(history) = web_sys::window().unwrap().history() {
            let href = web_sys::window()
                .unwrap()
                .location()
                .href()
                .ok()
                .unwrap_or_default();
            if let Ok(url) = web_sys::Url::new(&href) {
                url.search_params().set("room", &new_room);
                let _ = history.replace_state_with_url(
                    &wasm_bindgen::JsValue::NULL,
                    "",
                    Some(&url.href()),
                );
            }
        }
    }

    // -- open new socket --
    commands.spawn(omdurman_net::build_socket(&new_room));

    // -- go back to connecting --
    next_state.set(AppState::Connecting);

    commands.remove_resource::<ReconnectRoom>();
}

pub(crate) fn retry_snapshot_request(
    time: Res<Time>,
    mut net: ResMut<NetState>,
    mut pending: ResMut<PendingEdits>,
) {
    if net.needs_snapshot {
        net.snapshot_retry_timer += time.delta_secs_f64();
        if net.snapshot_retry_timer > 2.0 {
            net.snapshot_retry_timer = 0.0;
            info!("guest: retrying snapshot request");
            pending
                .outgoing_broadcast
                .push(NetMsg::Control(Control::RequestSnapshot));
        }
    }
}

pub(crate) fn handle_socket(
    mut socket_q: Query<&mut MatchboxSocket>,
    mut net: ResMut<NetState>,
    mut pending: ResMut<PendingEdits>,
    mut turn: ResMut<TurnState>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mut game_map: ResMut<GameMap>,
    mut gsp: GameStateParams,
    mut ctx: SocketContext,
) {
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };

    let Ok(peer_updates) = socket.try_update_peers() else {
        return;
    };
    let mut peers_changed = false;
    let mut newly_connected: Vec<PeerId> = Vec::new();
    for (peer, peer_state) in peer_updates {
        match peer_state {
            PeerState::Connected if !net.peers.contains(&peer) => {
                net.peers.push(peer);
                newly_connected.push(peer);
                peers_changed = true;
                info!(%peer, "peer connected");
            }
            PeerState::Disconnected => {
                let before = net.peers.len();
                net.peers.retain(|&p| p != peer);
                peers_changed |= net.peers.len() != before;
                info!(%peer, "peer disconnected");
            }
            _ => {}
        }
    }

    // Track whether we just learned our own ID for the first time.
    let my_id_just_set = net.my_id.is_none() && socket.id().is_some();
    if my_id_just_set {
        net.my_id = socket.id();
    }
    if peers_changed || my_id_just_set {
        net.refresh_sorted();
    }

    if let Some(my_id) = net.my_id
        && (peers_changed || my_id_just_set)
    {
        let new_host_is_me = net.sorted_all().first() == Some(&my_id);
        let promoted = new_host_is_me && !net.is_host;
        if turn.game_started && promoted {
            // A freshly promoted host must resume the canonical sequence
            // numbering where the previous host left off. `next_seq` was only
            // ever incremented on whoever was host, so on a guest it is still 0;
            // adopting it as-is would re-issue sequence numbers that already
            // exist, and the receive-side dedup (`last_applied_seq`) would then
            // silently drop those *new* events -- a permanent desync. Every peer
            // tracks `last_applied_seq`, so it is the correct baseline: the next
            // seq to assign is one past the highest this peer has applied.
            net.next_seq = net.last_applied_seq.map_or(0, |s| s + 1);
            info!(
                next_seq = net.next_seq,
                "promoted to host after previous host disconnect; resumed sequence numbering"
            );
        }
        net.is_host = new_host_is_me;
    }

    // The lobby is entered voluntarily (via the mode picker), not
    // auto-triggered by peers appearing -- so a local editing/sandbox session
    // is never dragged into someone else's game.

    // Message processing runs in both Lobby and InGame: the lobby needs to
    // receive faction picks, the host's `StartGame`, and snapshot replies.
    // In `Spectating` the timeline owns the world (rebuilt from a record, no
    // live peer), so socket processing is suppressed.
    if matches!(*state.get(), AppState::Connecting | AppState::Spectating) {
        return;
    }

    let mut targeted: Vec<(NetMsg, PeerId)> = Vec::new();
    let mut sequenced_out: Vec<NetMsg> = Vec::new();

    // Host: proactively push the canonical record to any peer that just
    // connected while a game is in progress. This catches up both a fresh late
    // joiner and -- crucially -- a peer that dropped and reconnected during a
    // WebRTC blip (matchbox reconnects it automatically, but it silently missed
    // every `Sequenced` event sent while it was gone). The receiver only
    // replays a record that is ahead of its local state, so a peer that never
    // fell behind ignores it. This makes reconnection self-healing rather than
    // relying on the joiner noticing it is behind.
    if net.is_host
        && turn.game_started
        && !newly_connected.is_empty()
        && let Some(ref record) = ctx.recorder.record
        && !record.events.is_empty()
    {
        for peer in newly_connected {
            info!(%peer, "host: pushing game history to (re)connected peer");
            targeted.push((NetMsg::Control(Control::GameHistory(record.clone())), peer));
            if !net.snapshot_pending.contains(&peer) {
                net.snapshot_pending.push(peer);
            }
        }
    }

    let reliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_RELIABLE).receive();
    let unreliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_UNRELIABLE).receive();
    let is_host = net.is_host;

    // Host loopback: events the host sequenced for itself (below). They flow
    // through the identical apply path as remote `Sequenced` events so every
    // peer -- host included -- observes the same ordered stream. `my_id` is the
    // canonical "sender" for these.
    let my_id = net.my_id.unwrap_or(PeerId(uuid::Uuid::nil()));
    let loopback: Vec<(PeerId, NetMsg)> = ctx.incoming
        .loopback
        .drain(..)
        .map(|msg| (my_id, msg))
        .collect();

    let decoded = reliable
        .into_iter()
        .chain(unreliable)
        .filter_map(|(peer, raw)| match decode(&raw) {
            Some(msg) => Some((peer, msg)),
            None => {
                warn!("unknown message, ignoring");
                None
            }
        })
        .chain(loopback);

    for (peer, msg) in decoded {
        let sender_idx = net.sender_idx_or_recorded(peer);
        match msg {
            NetMsg::Game(ev) => {
                if !is_host {
                    // We received an unsequenced submission but we don't believe
                    // we are the host -- most likely a transient election
                    // disagreement right after a peer connect/disconnect (the
                    // sender's view of the lowest PeerId briefly differs from
                    // ours). Dropping it would silently lose real player input,
                    // so re-forward it to whoever *we* currently consider the
                    // host. If we are in fact the host, the two views reconcile
                    // within a frame or two and the resend reaches us.
                    match net.host_id() {
                        Some(host) => {
                            warn!(
                                "received unsequenced Game event but we are not host; re-forwarding to current host"
                            );
                            targeted.push((NetMsg::Game(ev), host));
                        }
                        None => {
                            warn!(
                                "received unsequenced Game event but we are not host and no host is known; retaining for retry"
                            );
                            // Bounce it back onto our own outgoing broadcast so
                            // `flush_pending` re-submits once a host is known.
                            pending.outgoing_broadcast.push(NetMsg::Game(ev));
                        }
                    }
                    continue;
                }
                let seq = net.next_seq;
                net.next_seq += 1;
                let sequenced = NetMsg::Sequenced { seq, event: ev };
                sequenced_out.push(sequenced.clone());
                // Push the echo onto our own loopback queue. It is *not* applied
                // here: this `for` loop is already iterating `decoded`, which was
                // chained from `loopback.drain(..)` *above*. The just-pushed echo
                // won't be seen until the next call to this system (next frame),
                // so the host applies its own sequenced events one frame later
                // than a guest that receives the broadcast `Sequenced` echo. This
                // is intentional -- it keeps every peer (host included) on the
                // identical apply-on-echo path instead of special-casing the host
                // to apply inline.
                ctx.incoming.loopback.push(sequenced);
            }
            NetMsg::Sequenced { seq, event: ev } => {
                // Apply each sequence number exactly once. The reliable channel
                // is ordered and `seq` is monotonic, so any `seq` at or below
                // the highest already applied is a duplicate delivery -- drop it
                // so its effect (and any state it mutates, e.g. mp_spent) is not
                // applied twice. (push_event already dedups *recording*; this
                // extends the same guarantee to *application*.)
                if net.last_applied_seq.is_some_and(|last| seq <= last) {
                    continue;
                }
                net.last_applied_seq = Some(seq);
                ctx.recorder.push_event(&ev, sender_idx, seq);
                match &ev {
                    GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } => {
                        ctx.incoming.live.push((ev, peer, sender_idx));
                    }
                    GameEvent::StartGame {
                        assignments,
                        scenario,
                    } => {
                        if *state.get() != AppState::Lobby {
                            info!(%scenario, "ignoring StartGame; not in lobby");
                        } else {
                            gsp.player_factions.by_peer.clear();
                            for (pid, faction) in assignments {
                                gsp.player_factions.by_peer.insert(*pid, *faction);
                            }
                            // `GameState::new` already sets the scenario's
                            // first-moving player (§9.113 A-E for Campaign,
                            // §9.212/§9.322 Dervish otherwise); do not override.
                            gsp.game_state.0 = omdurman_rules::effects::GameState::new(*scenario);
                            let map_kind = map_kind_for_scenario(*scenario);
                            // Attach the board to the engine state synchronously
                            // (same as the replay path), so movement costing /
                            // ZOC never see an empty board between StartGame and
                            // the deferred visual map load.
                            gsp.game_state.0.board =
                                omdurman_rules::board::BoardInfo::from_map_data(
                                    gsp.loaded_annotations.0.map(map_kind),
                                );
                            gsp.pending_map_load.0 = Some(map_kind);
                            // Switch the view to the game board, so play opens on
                            // the scenario's board rather than whatever screen
                            // preceded the lobby. (The board data loads from
                            // `pending_map_load`; the board picked follows the
                            // scenario via `sync_edit_board_to_mode`.)
                            gsp.next_app_mode.set(crate::AppMode::Game);
                            if !turn.game_started {
                                turn.game_started = true;
                                next_state.set(AppState::InGame);
                                info!(%scenario, "game started via host StartGame");
                                // A guest that wasn't assigned a faction is a
                                // spectator: request the full record from the host
                                // so it converges to every unit already placed,
                                // not just events seen after this point. (Playing
                                // guests are assigned and present from the start,
                                // so they don't need it.)
                                if !is_host
                                    && gsp.player_factions.local(&net).is_none()
                                    && !net.snapshot_applied
                                {
                                    net.needs_snapshot = true;
                                    net.snapshot_retry_timer = 0.0;
                                    if let Some(host) = net.host_id() {
                                        targeted.push((
                                            NetMsg::Control(Control::RequestSnapshot),
                                            host,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        let active_map = gsp.active_edit_map.0;
                        let mut apply_ctx = game_apply::GameApplyCtx {
                            game_map: &mut game_map,
                            overlay: &mut ctx.overlay,
                            editor: &mut ctx.editor,
                            annotations: ctx.annotations.as_deref_mut(),
                            viewer: &mut ctx.viewer,
                            commands: &mut commands,
                            game_state: Some(&mut gsp.game_state.0),
                            loaded_annotations: Some(&mut gsp.loaded_annotations),
                            active_map,
                        };
                        game_apply::apply_game_event(&ev, &mut apply_ctx);
                        gsp.applied_events.0.push((ev.clone(), seq));
                        // Drain observations produced by the effect (if any)
                        // into the pending buffer; `drain_observations` emits
                        // them as `ObservationEvent` messages for UI listeners.
                        for obs in gsp.game_state.0.drain_observations() {
                            gsp.pending_observations.0.push((obs, seq));
                        }
                    }
                }
            }
            NetMsg::Ephemeral(Ephemeral::BrowserSelect { sprite }) => {
                if let Some(si) = ctx.browser
                    .sections
                    .iter()
                    .position(|s| s.name == sprite.section_name)
                    && let Some(spi) = ctx.browser.sections[si]
                        .sprites
                        .iter()
                        .position(|s| s.col == sprite.col && s.row == sprite.row)
                {
                    let sprite = &ctx.browser.sections[si].sprites[spi];
                    ctx.browser.selected_sprite = Some(browser::SpriteSelection {
                        section: si,
                        sprite: spi,
                        section_name: ctx.browser.sections[si].name,
                        unit_name: ctx.browser.sections[si].name.display_name().to_string(),
                        col: sprite.col,
                        row: sprite.row,
                    });
                }
            }
            NetMsg::Ephemeral(eph) => {
                ctx.incoming.ephemeral.push((eph, peer));
            }
            NetMsg::Control(Control::RequestSnapshot) => {
                if !is_host {
                    continue;
                }
                info!("host: late joiner requested game history");
                if turn.game_started
                    && let Some(ref record) = ctx.recorder.record
                {
                    targeted.push((NetMsg::Control(Control::GameHistory(record.clone())), peer));
                    net.snapshot_pending.push(peer);
                }
            }
            NetMsg::Control(Control::SnapshotReceived) => {
                info!("host: late joiner acknowledged game history");
                net.snapshot_pending.retain(|&p| p != peer);
            }
            NetMsg::Control(Control::GameHistory(record)) => {
                // Accept a record only if it carries events we haven't applied
                // yet. This covers two cases with one rule:
                //   * fresh late joiner (`last_applied_seq == None`): always
                //     ahead, so replay it;
                //   * reconnecting peer that fell behind during a WebRTC blip:
                //     `snapshot_applied` is already `true` from its first join,
                //     but the host's record now has a higher max seq than we
                //     applied, so we resync to catch up.
                // A record whose highest seq we've already applied is a genuine
                // duplicate (two snapshots racing) -- ignore it.
                let record_max = record.events.iter().map(|e| e.seq).max();
                let ahead = match (record_max, net.last_applied_seq) {
                    (Some(hi), Some(applied)) => hi > applied,
                    (Some(_), None) => true,
                    (None, _) => false,
                };
                if !ahead {
                    info!("ignoring game history that is not ahead of local state");
                    continue;
                }
                net.snapshot_applied = true;
                net.needs_snapshot = false;
                net.snapshot_retry_timer = 0.0;
                info!(
                    "received game history ({} events), replaying to resync",
                    record.events.len()
                );
                targeted.push((NetMsg::Control(Control::SnapshotReceived), peer));
                ctx.recorder.install_history(record.clone());
                // The snapshot already includes every event up to the highest
                // recorded seq; mark them applied so a live `Sequenced` echo of
                // an event also present in the snapshot isn't applied a second
                // time (only seqs above the watermark are new to this joiner).
                net.last_applied_seq = record.events.iter().map(|e| e.seq).max();
                rebuild_state_to(
                    &record,
                    None, // late joiner: replay the whole log
                    &mut commands,
                    &mut game_map,
                    &mut ctx.overlay,
                    &mut ctx.editor,
                    ctx.annotations.as_deref_mut(),
                    &mut ctx.viewer,
                    &mut ctx.incoming.replay,
                    peer,
                    &mut gsp.game_state.0,
                    &mut gsp.player_factions,
                    &mut gsp.loaded_annotations,
                    &mut gsp.pending_map_load,
                );
            }
        }
    }
    for (msg, peer) in targeted {
        pending.outgoing_targeted.push((msg, peer));
    }
    for msg in sequenced_out {
        pending.outgoing_broadcast.push(msg);
    }
}

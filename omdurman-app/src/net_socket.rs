use crate::{
    AppState, GameStateParams, PendingEdits, PendingIncoming, ReconnectRoom, TurnState, browser,
    editor, game_apply, game_record, map_kind_for_scenario, parse_peer_id, picker, render,
    replay_game_history, units,
};
use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use omdurman_hexmap::GameMap;
use omdurman_net::{
    CH_RELIABLE, CH_UNRELIABLE, Control, Ephemeral, GameEvent, NetMsg, NetState, RoomId, decode,
};

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
    room.0.clone_from(&new_room);

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
    mut editor: ResMut<editor::HexEditor>,
    mut browser: ResMut<browser::SpriteBrowser>,
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<render::HexOverlay>,
    mut annotations: Option<ResMut<browser::SpriteAnnotationsResource>>,
    mut viewer: ResMut<units::UnitViewer>,
    mut incoming: ResMut<PendingIncoming>,
    mut recorder: ResMut<game_record::GameRecorder>,
    mut gsp: GameStateParams,
) {
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };

    let Ok(peer_updates) = socket.try_update_peers() else {
        return;
    };
    let mut peers_changed = false;
    for (peer, peer_state) in peer_updates {
        match peer_state {
            PeerState::Connected if !net.peers.contains(&peer) => {
                net.peers.push(peer);
                peers_changed = true;
            }
            PeerState::Disconnected => {
                let before = net.peers.len();
                net.peers.retain(|&p| p != peer);
                peers_changed |= net.peers.len() != before;
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
        if turn.game_started && new_host_is_me && !net.is_host {
            info!("promoted to host after previous host disconnect");
        }
        net.is_host = new_host_is_me;
    }

    // Lobby is entered voluntarily (via `EditorMode::Lobby`), not
    // auto-triggered by peers appearing -- so a local editing session is
    // never dragged into someone else's game. The mode->state transition
    // (and the guest snapshot request) lives in `sync_lobby_appstate`.

    // Message processing runs in both Lobby and InGame: the lobby needs to
    // receive faction picks, the host's `StartGame`, and snapshot replies.
    if *state.get() == AppState::Connecting {
        return;
    }

    let mut targeted: Vec<(NetMsg, PeerId)> = Vec::new();
    let mut sequenced_out: Vec<NetMsg> = Vec::new();
    let reliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_RELIABLE).receive();
    let unreliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_UNRELIABLE).receive();
    let is_host = net.is_host;

    // Host loopback: events the host sequenced for itself (below). They flow
    // through the identical apply path as remote `Sequenced` events so every
    // peer -- host included -- observes the same ordered stream. `my_id` is the
    // canonical "sender" for these.
    let my_id = net.my_id.unwrap_or(PeerId(uuid::Uuid::nil()));
    let loopback: Vec<(PeerId, NetMsg)> = incoming
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
        let sender_idx = net.sender_idx(peer);
        match msg {
            NetMsg::Game(ev) => {
                if !is_host {
                    warn!("received unsequenced Game event but we are not host; dropping");
                    continue;
                }
                let seq = net.next_seq;
                net.next_seq += 1;
                let sequenced = NetMsg::Sequenced { seq, event: ev };
                sequenced_out.push(sequenced.clone());
                incoming.loopback.push(sequenced);
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
                recorder.push_event(&ev, sender_idx, seq);
                match &ev {
                    GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } => {
                        incoming.live.push((ev, peer, sender_idx));
                    }
                    GameEvent::StartGame {
                        assignments,
                        scenario,
                    } => {
                        if *state.get() != AppState::Lobby {
                            info!(%scenario, "ignoring StartGame; not in lobby");
                        } else {
                            gsp.player_factions.by_peer.clear();
                            for (peer_str, faction) in assignments {
                                if let Some(pid) = parse_peer_id(peer_str) {
                                    gsp.player_factions.by_peer.insert(pid, *faction);
                                }
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
                            // Switch the view to the scenario's map mode, so the
                            // game opens on the selected board rather than
                            // whatever screen preceded the lobby. (The board data
                            // loads from `pending_map_load`; the camera / side
                            // panels follow `EditorMode`.)
                            gsp.next_editor_mode
                                .set(crate::editor_mode_for_map(map_kind));
                            if !turn.game_started {
                                turn.game_started = true;
                                next_state.set(AppState::InGame);
                                info!(%scenario, "game started via host StartGame");
                            }
                        }
                    }
                    _ => {
                        let active_map = gsp.active_edit_map.0;
                        let mut ctx = game_apply::GameApplyCtx {
                            game_map: &mut game_map,
                            overlay: &mut overlay,
                            editor: &mut editor,
                            annotations: annotations.as_deref_mut(),
                            viewer: &mut viewer,
                            commands: &mut commands,
                            game_state: Some(&mut gsp.game_state.0),
                            loaded_annotations: Some(&mut gsp.loaded_annotations),
                            active_map,
                        };
                        game_apply::apply_game_event(&ev, &mut ctx);
                        gsp.applied_events.0.push((ev.clone(), seq));
                    }
                }
            }
            NetMsg::Ephemeral(Ephemeral::BrowserSelect { sprite }) => {
                if let Some(si) = browser
                    .sections
                    .iter()
                    .position(|s| s.name == sprite.section_name)
                    && let Some(spi) = browser.sections[si]
                        .sprites
                        .iter()
                        .position(|s| s.col == sprite.col && s.row == sprite.row)
                {
                    let sprite = &browser.sections[si].sprites[spi];
                    browser.selected_sprite = Some(browser::SpriteSelection {
                        section: si,
                        sprite: spi,
                        section_name: browser.sections[si].name,
                        unit_name: browser.sections[si].name.display_name().to_string(),
                        col: sprite.col,
                        row: sprite.row,
                    });
                }
            }
            NetMsg::Ephemeral(eph) => {
                incoming.ephemeral.push((eph, peer));
            }
            NetMsg::Control(Control::RequestSnapshot) => {
                if !is_host {
                    continue;
                }
                info!("host: late joiner requested game history");
                if turn.game_started
                    && let Some(ref record) = recorder.record
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
                if net.snapshot_applied {
                    info!("ignoring duplicate game history");
                    continue;
                }
                net.snapshot_applied = true;
                net.needs_snapshot = false;
                net.snapshot_retry_timer = 0.0;
                info!(
                    "late joiner: received game history ({} events), replaying",
                    record.events.len()
                );
                targeted.push((NetMsg::Control(Control::SnapshotReceived), peer));
                recorder.install_history(record.clone());
                // The snapshot already includes every event up to the highest
                // recorded seq; mark them applied so a live `Sequenced` echo of
                // an event also present in the snapshot isn't applied a second
                // time (only seqs above the watermark are new to this joiner).
                net.last_applied_seq = record.events.iter().map(|e| e.seq).max();
                replay_game_history(
                    &record,
                    &mut commands,
                    &mut game_map,
                    &mut overlay,
                    &mut editor,
                    annotations.as_deref_mut(),
                    &mut viewer,
                    &mut incoming.replay,
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

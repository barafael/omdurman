//! Remember Gordon! Battle of Omdurman.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod browser;
mod camera;
mod dice;
mod editor;
mod event_viewer;
mod game_apply;
mod game_record;
mod picker;
mod render;
mod settings;
mod units;
mod util;

use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::tonemapping::Tonemapping,
    gizmos::config::{DefaultGizmoConfigGroup, GizmoConfigStore},
    input::{
        mouse::{MouseScrollUnit, MouseWheel},
        touch::Touches,
    },
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_matchbox::prelude::*;
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, load_annotations_from_str};
use omdurman_net::{
    CH_RELIABLE, CH_UNRELIABLE, Control, EditorMode, Ephemeral, GameEvent, GameRecord, GameRng,
    NetMsg, NetState, RoomId, decode, enc_msg, open_socket, room_id,
};
use omdurman_types::HexCoord;
use std::{borrow::Cow, collections::HashMap};

use crate::camera::{CameraDragState, CameraSettings, RtsCamera, RtsCameraState};

/// Set by settings_ui when the user clicks Host or Join.
/// The system `handle_reconnect` picks this up, disconnects from
/// the current room, and opens a new socket with the new room ID.
#[derive(Resource)]
pub struct ReconnectRoom(pub String);

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
struct CursorBroadcastTimer(Timer);

impl Default for CursorBroadcastTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

/// Frame-scoped staging buffer for reliable outbound messages.
///
/// Why a buffer at all if matchbox channels already queue? Two reasons:
/// (1) systems can stage messages without taking `&mut MatchboxSocket`, which
///     would conflict with other socket-using systems; (2) host-side
///     `record_host_events` reads `outgoing_broadcast` to append outgoing game
///     events into the canonical event log *before* they're flushed to the wire.
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
    /// `PlaceUnit` / `MoveUnit` events received live — recorded by
    /// `apply_pending_placement` and applied to the world. Other game
    /// events are applied inline by `handle_socket`; these two are deferred
    /// because they need access to the picker + mesh/material asset pools.
    /// The `u8` is the pre-computed sender index.
    pub live: Vec<(GameEvent, PeerId, u8)>,
    /// Same kind of events but injected from a `GameHistory` replay —
    /// already in the canonical event log, so must NOT be re-recorded.
    pub replay: Vec<(GameEvent, PeerId)>,
    /// Ephemeral display messages buffered by `handle_socket` for
    /// `apply_pending_placement` to apply (cursor positions need access
    /// to the `Window` resource for normalisation, player info needs the
    /// `PlayerInfoMap` resource).
    pub ephemeral: Vec<(Ephemeral, PeerId)>,
}

#[derive(Resource, Default)]
pub struct SidebarClip {
    pub right_sidebar: Option<egui::Rect>,
}

/// Debounce flag for `assets/annotations.ron` writes. Set by any editor /
/// browser / overlay system that mutates persisted state. The
/// `flush_annotations_to_disk` system writes the file after the dirty flag
/// has been set and no further change has arrived for one cooldown window.
#[derive(Resource, Default)]
pub struct AnnotationsDirty {
    pub dirty: bool,
    /// Seconds since the last setter touched `dirty`. Reset to 0 on every
    /// mark; the flush system writes once this exceeds [`ANNOTATIONS_FLUSH_SECS`].
    pub idle: f32,
}

impl AnnotationsDirty {
    pub fn mark(&mut self) {
        self.dirty = true;
        self.idle = 0.0;
    }
}

/// Time (seconds) the disk write waits for further edits before flushing.
const ANNOTATIONS_FLUSH_SECS: f32 = 0.5;

fn main() {
    let room = room_id();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(EguiPlugin::default())
        .init_state::<AppState>()
        .add_message::<DiceRollResult>()
        .insert_resource(RoomId(room))
        .insert_resource(NetState::default())
        .insert_resource(TurnState::default())
        .insert_resource(CameraSettings::default())
        .insert_resource(CameraDragState::default())
        .insert_resource(GameMap::default())
        .insert_resource(render::HexOverlay::default())
        .insert_resource(editor::HexEditor::default())
        .insert_resource(EditorMode::Normal)
        .insert_resource(units::UnitViewer::load_or_default())
        .insert_resource(browser::SpriteBrowser::new())
        .insert_resource(browser::SpriteMetaClipboard::default())
        .insert_resource(settings::SettingsOverlay::default())
        .insert_resource(settings::LocalPlayerSettings::default())
        .insert_resource(settings::PlayerInfoMap::default())
        .insert_resource(dice::DiceSimulator::default())
        .insert_resource(PendingEdits::default())
        .insert_resource(PendingIncoming::default())
        .insert_resource(SidebarClip::default())
        .insert_resource(AnnotationsDirty::default())
        .insert_resource(picker::UnitPicker::default())
        .insert_resource(picker::PickerState::default())
        .insert_resource(game_record::GameRecorder::default())
        .insert_resource(event_viewer::EventViewerState::default())
        .insert_resource(CursorPositions::default())
        .insert_resource(CursorBroadcastTimer::default())
        .insert_resource(HexLayout::calibrated(
            omdurman_types::Orientation::Pointy,
            Vec2::new(736.0, 420.0),
            omdurman_types::HexCoord::new(0, 0),
            Vec2::new(1178.0, 572.0),
            omdurman_types::HexCoord::new(5, -1),
        ))
        .add_systems(
            Startup,
            (
                setup_ui,
                open_socket,
                spawn_camera,
                spawn_ground,
                spawn_lights,
                render::spawn_map_plane,
                render::spawn_selection_marker,
                units::spawn_units_plane,
                browser::spawn_sprite_browser,
                picker::spawn_picker_assets,
                load_annotations,
                init_gizmo_config,
                configure_egui_touch,
            ),
        )
        .add_systems(
            Update,
            (
                setup_egui_fonts,
                (
                    camera_control,
                    render::draw_hex_debug,
                    render::update_selection_marker,
                    handle_mode_shortcuts,
                    editor::editor_terrain_keys,
                    editor::handle_hex_editor_click,
                    editor::draw_editor_highlight,
                    despawn_dice,
                    handle_reconnect,
                    retry_snapshot_request.after(handle_reconnect),
                    handle_socket.after(handle_reconnect),
                    apply_pending_placement.after(handle_socket),
                    handle_local_input.after(handle_socket),
                    update_status_text.after(handle_socket),
                    units::draw_unit_grids,
                    picker::placement_preview_gizmo,
                    picker::handle_picker_clicks,
                    picker::movement_overlay_gizmo,
                    picker::animate_unit_movement,
                    picker::cancel_placement,
                ),
                (
                    game_record::init_game_record.after(handle_socket),
                    game_record::host_emit_annotations
                        .after(game_record::init_game_record)
                        .before(flush_pending),
                    game_record::record_outgoing_broadcasts
                        .after(game_record::host_emit_annotations)
                        .before(flush_pending),
                    game_record::flush_game_record.after(game_record::record_outgoing_broadcasts),
                    send_player_info_on_connect.after(handle_socket),
                    prune_disconnected_peers.after(handle_socket),
                    broadcast_cursor,
                    broadcast_browser_selection,
                    flush_pending,
                    flush_annotations_to_disk,
                    sync_mode_visibilities,
                ),
                (
                    browser::scroll_sprite_browser,
                    browser::handle_sprite_clicks,
                    browser::update_sprite_selection_marker,
                    browser::navigate_sprite_selection,
                ),
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                cursor_overlay_ui,
                mode_toolbar,
                render::overlay_ui,
                editor::editor_ui,
                units::unit_grids_ui,
                units::unit_grid_labels,
                browser::sprite_meta_editor_ui,
                dice::dice_sim_ui,
                picker::unit_picker_ui,
                event_viewer::event_viewer_ui,
                settings::settings_ui,
            ),
        )
        .run();
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
enum AppState {
    #[default]
    Connecting,
    InGame,
}

#[derive(Resource, Default)]
struct TurnState {
    my_index: usize,
    current_turn: usize,
    pending_roll: Option<u32>,
    game_started: bool,
}

#[derive(Component)]
struct Dice {
    timer: Timer,
}

#[derive(Message, Debug)]
pub struct DiceRollResult {
    pub by_me: bool,
    pub data: u32,
}

#[derive(Component)]
struct StatusPane;

#[derive(Component)]
struct StatusText;

fn init_gizmo_config(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -0.01;
    config.line.width = 2.0;
}

fn setup_egui_fonts(mut contexts: EguiContexts, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
    ctx.add_font(FontInsert::new(
        "EBGaramond-Regular",
        egui::FontData::from_static(include_bytes!("../../assets/fonts/EBGaramond-Regular.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("Garamond".into()),
            priority: FontPriority::Highest,
        }],
    ));
    *done = true;
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut, unused_variables))]
fn configure_egui_touch(mut contexts: EguiContexts) {
    #[cfg(target_arch = "wasm32")]
    {
        let Ok(ctx) = contexts.ctx_mut() else { return };
        ctx.style_mut(|style| {
            style.spacing.interact_size = egui::vec2(40.0, 40.0);
            style.spacing.slider_width = 120.0;
        });
    }
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(14.0),
                left: Val::Px(14.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            StatusPane,
        ))
        .with_child((
            StatusText,
            Text::new("Connecting…"),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ));
}

fn handle_reconnect(
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

    // ── despawn old socket ──
    if let Ok(entity) = socket_q.single() {
        commands.entity(entity).despawn();
    }

    // ── reset state ──
    *net = NetState::default();
    *turn = TurnState::default();
    pending.outgoing_broadcast.clear();
    pending.outgoing_targeted.clear();
    incoming.live.clear();
    incoming.replay.clear();
    incoming.ephemeral.clear();
    *recorder = game_record::GameRecorder::default();

    // ── despawn placed units and restore full picker roster ──
    for entity in &placed_unit_q {
        commands.entity(entity).despawn();
    }
    picker.reset_available();
    *picker_state = picker::PickerState::Idle;

    // ── update room id and URL ──
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

    // ── open new socket ──
    commands.spawn(omdurman_net::build_socket(&new_room));

    // ── go back to connecting ──
    next_state.set(AppState::Connecting);

    commands.remove_resource::<ReconnectRoom>();
}

fn retry_snapshot_request(
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

fn handle_socket(
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
) {
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };

    let Ok(peer_updates) = socket.try_update_peers() else {
        return;
    };
    let mut new_peers: Vec<PeerId> = Vec::new();
    let mut peers_changed = false;
    for (peer, peer_state) in peer_updates {
        match peer_state {
            PeerState::Connected if !net.peers.contains(&peer) => {
                net.peers.push(peer);
                new_peers.push(peer);
                peers_changed = true;
            }
            PeerState::Disconnected => {
                let before = net.peers.len();
                net.peers.retain(|&p| p != peer);
                peers_changed |= net.peers.len() != before;
                // will be cleaned up by apply_pending_placement when it sees the empty entries
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

    if let Some(my_id) = net.my_id {
        if peers_changed && !net.peers.is_empty() {
            // Re-derive our position in the sorted list. If we're not present
            // (shouldn't happen — we should always include ourselves) keep the
            // old index so the game doesn't wedge.
            // Host election: the lowest-sorted PeerId is the host. Re-run on
            // every peer change so a guest gets promoted when the previous
            // host disconnects.
            let (my_index, new_host_is_me, total) = {
                let sorted = net.sorted_all();
                (
                    sorted
                        .iter()
                        .position(|&id| id == my_id)
                        .unwrap_or(turn.my_index),
                    sorted.first() == Some(&my_id),
                    sorted.len(),
                )
            };
            turn.my_index = my_index;
            if turn.game_started && new_host_is_me && !net.is_host {
                info!("promoted to host after previous host disconnect");
            }
            net.is_host = new_host_is_me;
            // Keep `current_turn` in range. If the active player dropped,
            // the new player at the same index (whoever shifted into that
            // slot in sorted order) gets their turn — close enough for a
            // casual wargame; the alternative is tracking turn-by-PeerId,
            // which is a bigger surgery.
            if total > 0 && turn.current_turn >= total {
                turn.current_turn %= total;
            }
        }

        if !turn.game_started && !net.peers.is_empty() {
            turn.game_started = true;
            turn.current_turn = 0;
            info!("game started, {} peers", net.peers.len());
            next_state.set(AppState::InGame);
            if !net.is_host {
                // Request game history from host.  If the game is brand-new the
                // host has no record yet and will ignore the request; if the game
                // is ongoing the host will reply with GameHistory.
                net.needs_snapshot = true;
                net.snapshot_retry_timer = 0.0;
                pending
                    .outgoing_broadcast
                    .push(NetMsg::Control(Control::RequestSnapshot));
            }
        } else if turn.game_started && net.is_host {
            for &p in &new_peers {
                if let Some(ref record) = recorder.record {
                    info!("host: sending game history to late joiner");
                    pending
                        .outgoing_targeted
                        .push((NetMsg::Control(Control::GameHistory(record.clone())), p));
                    net.snapshot_pending.push(p);
                }
            }
        }
    }

    if *state.get() != AppState::InGame {
        return;
    }

    let mut targeted: Vec<(NetMsg, PeerId)> = Vec::new();
    let reliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_RELIABLE).receive();
    let unreliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_UNRELIABLE).receive();
    let total_peers = net.sorted_all().len().max(1);
    for (peer, raw) in reliable.into_iter().chain(unreliable.into_iter()) {
        let Some(msg) = decode(&raw) else {
            warn!("unknown message, ignoring");
            continue;
        };
        let sender_idx = net.sender_idx(peer);
        match msg {
            NetMsg::Game(ev) => {
                recorder.push_event(&ev, sender_idx);
                match &ev {
                    // Placement needs picker + asset access; defer to
                    // `apply_pending_placement`.
                    GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } => {
                        incoming.live.push((ev, peer, sender_idx));
                    }
                    _ => {
                        // Action advances the turn here because `apply_game_event`
                        // is peer-agnostic by design — used by replay too, where
                        // the live peer count isn't meaningful.
                        if matches!(&ev, GameEvent::Action(_)) {
                            turn.current_turn = (turn.current_turn + 1) % total_peers;
                        }
                        let mut ctx = game_apply::GameApplyCtx {
                            game_map: &mut game_map,
                            overlay: &mut overlay,
                            editor: &mut editor,
                            annotations: annotations.as_deref_mut(),
                            viewer: &mut viewer,
                            commands: &mut commands,
                        };
                        game_apply::apply_game_event(&ev, &mut ctx);
                    }
                }
            }
            NetMsg::Ephemeral(Ephemeral::BrowserSelect {
                section_name,
                col,
                row,
            }) => {
                // Find the matching sprite in the local browser and apply
                // the selection so all peers share the same view in Units mode.
                if let Some(si) = browser.sections.iter().position(|s| s.name == section_name)
                    && let Some(spi) = browser.sections[si]
                        .sprites
                        .iter()
                        .position(|s| s.col == col && s.row == row)
                {
                    let sprite = &browser.sections[si].sprites[spi];
                    browser.selected_sprite = Some(browser::SpriteSelection {
                        section: si,
                        sprite: spi,
                        section_name: browser.sections[si].name.clone(),
                        unit_name: browser.sections[si].name.replace('_', " "),
                        col: sprite.col,
                        row: sprite.row,
                    });
                }
            }
            NetMsg::Ephemeral(eph) => {
                // Other ephemerals (CursorPos, PlayerInfo, EventViewerSelect)
                // need access to resources not available here — defer.
                incoming.ephemeral.push((eph, peer));
            }
            NetMsg::Control(Control::RequestSnapshot) => {
                info!("host: late joiner requested game history");
                // Any peer with a full record can serve as snapshot source.
                // The first responder wins; late joiners ignore duplicates
                // via `net.snapshot_applied`.
                if turn.game_started
                    && let Some(ref record) = recorder.record
                {
                    targeted.push((NetMsg::Control(Control::GameHistory(record.clone())), peer));
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
                // Install the host's record locally so the Event Viewer on
                // guests sees the full history, not just LoadAnnotations.
                recorder.record = Some(record.clone());
                replay_game_history(
                    &record,
                    &mut commands,
                    &mut game_map,
                    &mut overlay,
                    &mut editor,
                    annotations.as_deref_mut(),
                    &mut viewer,
                    &mut turn,
                    total_peers,
                    &mut incoming.replay,
                    peer,
                );
            }
        }
    }
    // queue targeted sends (flushed by flush_pending later)
    for (msg, peer) in targeted {
        pending.outgoing_targeted.push((msg, peer));
    }
}

fn apply_pending_placement(
    mut incoming: ResMut<PendingIncoming>,
    mut picker: ResMut<picker::UnitPicker>,
    layout: Res<HexLayout>,
    overlay: Res<render::HexOverlay>,
    game_map: Res<GameMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut placed_units: Query<(Entity, &mut picker::PlacedUnit)>,
    mut player_info: ResMut<settings::PlayerInfoMap>,
    mut recorder: ResMut<game_record::GameRecorder>,
    mut cursor_positions: ResMut<CursorPositions>,
    mut event_viewer: Option<ResMut<event_viewer::EventViewerState>>,
    time: Res<Time>,
    // Tracks entities spawned this invocation so MoveUnit can find units
    // placed in the same batch (e.g. during history replay) before Bevy
    // has flushed the deferred commands.
    // key: (section_name, col, row), value: (entity, is_boat)
    mut just_placed: Local<HashMap<(String, u32, u32), (Entity, bool)>>,
) {
    just_placed.clear();

    // Replay events come first and must NOT be re-recorded (they are already
    // in the canonical game log).  Live events follow and ARE recorded.
    // replay: already recorded, don't re-record; sender_idx not needed
    let replay_items: Vec<_> = incoming
        .replay
        .drain(..)
        .map(|(msg, peer)| (msg, peer, false, 0u8))
        .collect();
    // live: record with the pre-computed sender_idx stored in the queue
    let live_items: Vec<_> = incoming
        .live
        .drain(..)
        .map(|(msg, peer, idx)| (msg, peer, true, idx))
        .collect();

    for (event, _peer, should_record, sender_idx) in replay_items.into_iter().chain(live_items) {
        if should_record {
            recorder.push_event(&event, sender_idx);
        }

        match event {
            GameEvent::PlaceUnit {
                section_name,
                col,
                row,
                coord_q,
                coord_r,
                is_boat,
            } => {
                let coord = omdurman_types::HexCoord::new(coord_q, coord_r);
                if !game_map.hexes.contains_key(&coord) {
                    warn!(
                        coord_q,
                        coord_r, "ignoring inbound PlaceUnit for off-map coord"
                    );
                    continue;
                }
                if placed_units.iter().any(|(_, u)| {
                    u.section_name == section_name
                        && u.col == col
                        && u.row == row
                        && u.coord == coord
                }) {
                    continue;
                }
                let unit_idx = picker
                    .available
                    .iter()
                    .position(|u| u.section_name == section_name && u.col == col && u.row == row);
                if let Some(idx) = unit_idx {
                    let unit = picker.available.remove(idx);
                    let origin = crate::util::adjusted_origin(
                        &layout,
                        overlay.params.offset_x,
                        overlay.params.offset_y,
                    );
                    let pos = crate::util::hex_world_pos(coord, origin, &overlay.params);
                    let sprite_size = overlay.params.hex_size * 1.05;
                    let material = materials.add(StandardMaterial {
                        base_color_texture: Some(unit.handle.clone()),
                        unlit: true,
                        alpha_mode: AlphaMode::Mask(0.1),
                        ..default()
                    });
                    let entity = commands
                        .spawn((
                            picker::PlacedUnit {
                                coord,
                                section_name: section_name.clone(),
                                col,
                                row,
                                is_boat,
                            },
                            Mesh3d(meshes.add(Rectangle::new(sprite_size, sprite_size))),
                            MeshMaterial3d(material),
                            Transform::from_xyz(pos.x, 1.0, pos.z)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
                            Visibility::Visible,
                        ))
                        .id();
                    just_placed.insert((section_name, col, row), (entity, is_boat));
                }
            }
            GameEvent::MoveUnit {
                section_name,
                col,
                row,
                to_q,
                to_r,
            } => {
                let target = omdurman_types::HexCoord::new(to_q, to_r);
                if !game_map.hexes.contains_key(&target) {
                    warn!(to_q, to_r, "ignoring inbound MoveUnit to off-map coord");
                    continue;
                }
                let origin = crate::util::adjusted_origin(
                    &layout,
                    overlay.params.offset_x,
                    overlay.params.offset_y,
                );
                let pos = crate::util::hex_world_pos(target, origin, &overlay.params);
                let new_transform = Transform::from_xyz(pos.x, 1.0, pos.z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0));

                // Try the live world query first (normal gameplay path).
                let mut found = false;
                for (entity, mut placed) in placed_units.iter_mut() {
                    if placed.section_name == section_name && placed.col == col && placed.row == row
                    {
                        placed.coord = target;
                        commands.entity(entity).insert(new_transform);
                        commands
                            .entity(entity)
                            .remove::<picker::MovementAnimation>();
                        found = true;
                        break;
                    }
                }

                // Fall back to units placed earlier in this same batch
                // (replay path — Bevy commands are still deferred).
                if !found
                    && let Some(&(entity, is_boat)) =
                        just_placed.get(&(section_name.clone(), col, row))
                {
                    commands.entity(entity).insert(picker::PlacedUnit {
                        coord: target,
                        section_name: section_name.clone(),
                        col,
                        row,
                        is_boat,
                    });
                    commands.entity(entity).insert(new_transform);
                    // update the map so subsequent moves on the same unit work
                    just_placed.insert((section_name, col, row), (entity, is_boat));
                }
            }
            // Other GameEvent variants are applied inline by handle_socket /
            // replay_game_history — they shouldn't appear in the deferred
            // queues. Warn if one does so the misclassification is visible.
            other => warn!(?other, "non-placement GameEvent in placement queue"),
        }
    }

    // ── Ephemeral messages — completely outside event sourcing, never recorded ──
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
                    settings::PeerPlayerInfo {
                        name,
                        color: egui::Color32::from_rgb(color_r, color_g, color_b),
                    },
                );
            }
            Ephemeral::CursorPos { x, y } => {
                // Net cursor positions are world-space (x = world.x, y = world.z).
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
            // BrowserSelect is applied inline by handle_socket; it never
            // reaches `incoming.ephemeral` (the routing classifier sends
            // those directly into the live browser state).
            Ephemeral::BrowserSelect { .. } => {}
        }
    }
}

fn replay_game_history(
    record: &GameRecord,
    commands: &mut Commands,
    game_map: &mut GameMap,
    overlay: &mut render::HexOverlay,
    editor: &mut editor::HexEditor,
    annotations: Option<&mut browser::SpriteAnnotationsResource>,
    viewer: &mut units::UnitViewer,
    turn: &mut TurnState,
    total_peers: usize,
    replay: &mut Vec<(GameEvent, PeerId)>,
    history_peer: PeerId,
) {
    info!("replaying {} events from game history", record.events.len());

    // Reset RNG + clear map — the event stream is canonical so we rebuild
    // from a known state.
    commands.insert_resource(GameRng::from_seed(record.initial_state.seed));
    game_map.hexes.clear();

    let mut action_count = 0u32;
    let mut ctx = game_apply::GameApplyCtx {
        game_map,
        overlay,
        editor,
        annotations,
        viewer,
        commands,
    };
    for event in &record.events {
        match &event.payload {
            GameEvent::Action(_) => action_count += 1,
            GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } => {
                replay.push((event.payload.clone(), history_peer));
                continue;
            }
            _ => {}
        }
        game_apply::apply_game_event(&event.payload, &mut ctx);
    }

    // Restore the turn counter from the recorded action count.
    if total_peers > 0 {
        turn.current_turn = (action_count as usize) % total_peers;
    }
}

/// Broadcast the local sprite-browser selection whenever it changes, so
/// all peers share the same view in Units mode.
fn broadcast_browser_selection(
    browser: Res<browser::SpriteBrowser>,
    mut last: Local<Option<(String, u32, u32)>>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    let current = browser
        .selected_sprite
        .as_ref()
        .map(|s| (s.section_name.clone(), s.col, s.row));
    if current == *last {
        return;
    }
    *last = current.clone();
    // No broadcast on deselect — peers keep their own view.
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
            section_name,
            col,
            row,
        }),
    );
}

/// Remove cursors and player info for peers that are no longer connected.
fn prune_disconnected_peers(
    net: Res<NetState>,
    mut cursor_positions: ResMut<CursorPositions>,
    mut player_info: ResMut<settings::PlayerInfoMap>,
) {
    let active: Vec<PeerId> = net.peers.iter().copied().collect();
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
fn send_player_info_on_connect(
    net: Res<NetState>,
    local: Res<settings::LocalPlayerSettings>,
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
        }
    }
}

fn handle_mode_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut current: ResMut<EditorMode>,
    mut editor: ResMut<editor::HexEditor>,
    mut browser: ResMut<browser::SpriteBrowser>,
    game_map: Res<omdurman_map::GameMap>,
    mut contexts: EguiContexts,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
        return;
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }
    let new_mode = if keys.just_pressed(KeyCode::Digit1) {
        Some(EditorMode::Normal)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(EditorMode::Overlay)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(EditorMode::Editor)
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some(EditorMode::UnitSheet)
    } else if keys.just_pressed(KeyCode::Digit5) {
        Some(EditorMode::Units)
    } else if keys.just_pressed(KeyCode::Digit6) {
        Some(EditorMode::Dice)
    } else if keys.just_pressed(KeyCode::Digit7) {
        Some(EditorMode::EventViewer)
    } else {
        None
    };
    if let Some(m) = new_mode {
        apply_mode(m, &mut current, &mut editor, &mut browser, &game_map);
        info!(mode = ?m, "mode switch via keyboard shortcut");
    }
}

fn handle_local_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut turn: ResMut<TurnState>,
    net: Res<NetState>,
    rng_opt: Option<ResMut<GameRng>>,
    mut pending: ResMut<PendingEdits>,
    mut ev_action: MessageWriter<DiceRollResult>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
        return;
    }
    if turn.current_turn != turn.my_index {
        return;
    }
    if net.peers.is_empty() {
        return;
    }
    let Some(mut rng) = rng_opt else { return };
    let mut local_rng = rand::rng();

    if keys.just_pressed(KeyCode::Space) && turn.pending_roll.is_none() {
        let roll = rng.random_u32() % 10 + 1;
        info!(roll, "rolled");

        turn.pending_roll = Some(roll);

        let radius = 60.0;
        let height = 120.0;
        let throw_dir = Vec3::new(
            rand::RngExt::random_range(&mut local_rng, -1.0..1.0),
            0.0,
            rand::RngExt::random_range(&mut local_rng, -1.0..1.0),
        )
        .normalize_or_zero();
        let initial_spin = throw_dir.cross(Vec3::Y) * 3.0
            + Vec3::new(
                rand::RngExt::random_range(&mut local_rng, -0.75..0.75),
                rand::RngExt::random_range(&mut local_rng, -0.75..0.75),
                rand::RngExt::random_range(&mut local_rng, -0.75..0.75),
            );

        let collider_points = d10_collider_points(radius, height);
        commands.spawn((
            RigidBody::Dynamic,
            Collider::convex_hull(collider_points).unwrap(),
            Mass(1.0),
            GravityScale(30.0),
            Mesh3d(meshes.add(d10_mesh_colored(radius, height))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)).with_rotation(
                Quat::from_euler(
                    EulerRot::XYZ,
                    rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                    rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                    rand::RngExt::random_range(&mut local_rng, 0.0..core::f32::consts::TAU),
                ),
            ),
            LinearVelocity(throw_dir * 150.0 + Vec3::Y * 100.0),
            AngularVelocity(initial_spin),
            Restitution::new(0.3),
            Friction::new(0.8),
            Dice {
                timer: Timer::from_seconds(6.0, TimerMode::Once),
            },
        ));
    }

    if keys.just_pressed(KeyCode::Enter)
        && let Some(roll) = turn.pending_roll.take()
    {
        info!(roll, "sending action");

        pending
            .outgoing_broadcast
            .push(NetMsg::Game(GameEvent::Action(roll)));

        ev_action.write(DiceRollResult {
            by_me: true,
            data: roll,
        });
        let total = net.sorted_all().len().max(1);
        turn.current_turn = (turn.current_turn + 1) % total;
    }
}

fn update_status_text(
    state: Res<State<AppState>>,
    turn: Res<TurnState>,
    room: Res<RoomId>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    let new = match state.get() {
        AppState::Connecting => {
            Cow::Owned(format!("Waiting for players - share: ?room={}", room.0))
        }
        AppState::InGame if turn.current_turn == turn.my_index && turn.pending_roll.is_none() => {
            Cow::Borrowed("Your turn - SPACE to roll")
        }
        AppState::InGame if turn.current_turn == turn.my_index && turn.pending_roll.is_some() => {
            Cow::Borrowed("ENTER to confirm")
        }
        AppState::InGame => Cow::Owned(format!("Player {}'s turn...", turn.current_turn)),
    };
    if text.as_str() != new.as_ref() {
        *text = Text::new(new.into_owned());
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        RtsCamera,
        RtsCameraState::default(),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        Tonemapping::None,
    ));
}

fn apply_mode(
    mode: EditorMode,
    current: &mut EditorMode,
    editor: &mut editor::HexEditor,
    browser: &mut browser::SpriteBrowser,
    game_map: &omdurman_map::GameMap,
) {
    *current = mode;
    match mode {
        EditorMode::Normal => editor.selected = None,
        EditorMode::Editor => {
            let coord = HexCoord { q: 0, r: 0 };
            if let Some(data) = game_map.hexes.get(&coord) {
                editor.selected = Some(coord);
                editor.name = data.name.clone().unwrap_or_default();
                editor.terrain = data.terrain;
            }
        }
        EditorMode::EventViewer => {}
        EditorMode::Units => {
            if browser.selected_sprite.is_none()
                && let Some(section) = browser.sections.first()
                && let Some(sprite) = section.sprites.first()
            {
                browser.selected_sprite = Some(browser::SpriteSelection {
                    section: 0,
                    sprite: 0,
                    section_name: section.name.clone(),
                    unit_name: section.name.replace('_', " "),
                    col: sprite.col,
                    row: sprite.row,
                });
            }
        }
        _ => {}
    }
}

fn sync_mode_visibilities(
    mode: Res<EditorMode>,
    mut vis_set: ParamSet<(
        Query<&mut Visibility, With<units::UnitsPlane>>,
        Query<&mut Visibility, With<render::MapPlane>>,
        Query<&mut Visibility, With<browser::SpriteBrowserRoot>>,
        Query<&mut Visibility, With<StatusPane>>,
        Query<&mut Visibility, With<picker::PlacedUnit>>,
    )>,
) {
    if let Ok(mut vis) = vis_set.p0().single_mut() {
        *vis = if *mode == EditorMode::UnitSheet {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = vis_set.p1().single_mut() {
        *vis = if matches!(
            *mode,
            EditorMode::UnitSheet | EditorMode::Units | EditorMode::EventViewer
        ) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    if let Ok(mut vis) = vis_set.p2().single_mut() {
        *vis = if *mode == EditorMode::Units {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = vis_set.p3().single_mut() {
        *vis = if matches!(*mode, EditorMode::Normal) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut vis in vis_set.p4().iter_mut() {
        *vis = if *mode == EditorMode::Normal {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn mode_display_name(mode: EditorMode) -> &'static str {
    match mode {
        EditorMode::Normal => "Normal",
        EditorMode::Overlay => "Overlay",
        EditorMode::Editor => "Editor",
        EditorMode::UnitSheet => "Unit Sheet",
        EditorMode::Units => "Units",
        EditorMode::Dice => "Dice",
        EditorMode::EventViewer => "EventViewer",
    }
}

fn mode_toolbar(
    mut contexts: EguiContexts,
    mut current: ResMut<EditorMode>,
    mut editor: ResMut<editor::HexEditor>,
    mut browser: ResMut<browser::SpriteBrowser>,
    game_map: Res<omdurman_map::GameMap>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::Area::new(egui::Id::new("mode_toolbar"))
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_gray(45))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
                    let mut clicked = None;
                    let mode_label = mode_display_name(*current);
                    egui::ComboBox::from_id_salt("mode_selector")
                        .selected_text(mode_label)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(&mut *current, EditorMode::Normal, "Normal")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Normal);
                            }
                            if ui
                                .selectable_value(&mut *current, EditorMode::Overlay, "Overlay")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Overlay);
                            }
                            if ui
                                .selectable_value(&mut *current, EditorMode::Editor, "Editor")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Editor);
                            }
                            if ui
                                .selectable_value(
                                    &mut *current,
                                    EditorMode::UnitSheet,
                                    "Unit Sheet",
                                )
                                .clicked()
                            {
                                clicked = Some(EditorMode::UnitSheet);
                            }
                            if ui
                                .selectable_value(&mut *current, EditorMode::Units, "Units")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Units);
                            }
                            if ui
                                .selectable_value(&mut *current, EditorMode::Dice, "Dice")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Dice);
                            }
                            if ui
                                .selectable_value(
                                    &mut *current,
                                    EditorMode::EventViewer,
                                    "EventViewer",
                                )
                                .clicked()
                            {
                                clicked = Some(EditorMode::EventViewer);
                            }
                        });
                    if let Some(m) = clicked {
                        apply_mode(m, &mut current, &mut editor, &mut browser, &game_map);
                    }
                });
        });
}

fn cursor_overlay_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    mode: Res<EditorMode>,
    local: Res<settings::LocalPlayerSettings>,
    mut cursor_positions: ResMut<CursorPositions>,
    player_info: Res<settings::PlayerInfoMap>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
) {
    if !local.show_other_cursors || !map_mode_active(*mode) || cursor_positions.current.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };

    let now = time.elapsed_secs_f64();
    let dt = time.delta_secs();

    // Smoothing coefficient: higher = snappier, lower = smoother.
    // 6.0 gives a long, buttery glide that trails the true position noticeably.
    const SMOOTH: f32 = 6.0;
    let alpha = 1.0 - (-SMOOTH * dt).exp();

    let peers: Vec<_> = cursor_positions.current.keys().copied().collect();

    for peer in &peers {
        let pos = cursor_positions.current[peer];
        let t = match cursor_positions.last_update.get(peer) {
            Some(&last) if last > 0.0 => {
                let elapsed = now - last;
                // Normalise over the broadcast interval (0.1 s) so the lerp
                // completes right as the next packet is expected to arrive.
                (elapsed / 0.1).clamp(0.0, 1.0)
            }
            _ => 1.0,
        };
        let prev = cursor_positions.previous.get(peer).copied().unwrap_or(pos);
        // Smooth in world space so the screen path stays correct when the
        // local camera pans, zooms, or pitches.
        let target = prev.lerp(pos, t as f32);
        let display = cursor_positions.display.entry(*peer).or_insert(target);
        *display = display.lerp(target, alpha);
    }

    egui::Area::new(egui::Id::new("cursor_overlay"))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            for peer in &peers {
                let Some(&world_xz) = cursor_positions.display.get(peer) else {
                    continue;
                };
                let world = Vec3::new(world_xz.x, 0.0, world_xz.y);
                let Ok(viewport) = camera.world_to_viewport(cam_transform, world) else {
                    continue;
                };
                let screen = egui::pos2(viewport.x, viewport.y);

                let color = player_info
                    .peers
                    .get(peer)
                    .map(|p| p.color)
                    .unwrap_or(egui::Color32::WHITE);
                painter.circle_filled(screen, 5.0, color);
                let label = player_info
                    .peers
                    .get(peer)
                    .map(|p| p.name.as_str())
                    .unwrap_or("?");
                painter.text(
                    screen + egui::Vec2::new(8.0, -4.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(12.0),
                    color,
                );
            }
        });
}

fn broadcast_cursor(
    mut timer: ResMut<CursorBroadcastTimer>,
    time: Res<Time>,
    mode: Res<EditorMode>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    if !map_mode_active(*mode) {
        return;
    }
    let Some(hit) = util::raycast_ground(&windows, &cameras) else {
        return;
    };
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };
    // Send world-space ground-plane coordinates (x = world.x, y = world.z) so
    // peers see the cursor anchored to the map regardless of window size,
    // camera pan/zoom, or pitch.
    omdurman_net::broadcast_unreliable(
        &mut socket,
        &net.peers,
        &NetMsg::Ephemeral(Ephemeral::CursorPos { x: hit.x, y: hit.z }),
    );
}

/// True when the 3D map plane is what the user is looking at — i.e. modes that
/// keep `MapPlane` visible AND show terrain (excludes the dice simulator,
/// which floats UI over a non-map scene).
fn map_mode_active(mode: EditorMode) -> bool {
    matches!(
        mode,
        EditorMode::Normal | EditorMode::Overlay | EditorMode::Editor
    )
}

/// Persist `assets/annotations.ron` once the dirty flag has been idle for
/// `ANNOTATIONS_FLUSH_SECS`. Coalesces many per-keystroke / per-drag changes
/// into one disk write at the end of an edit burst. On WASM this is a no-op
/// because the underlying `save_annotations_to_file` already skips writes.
#[cfg(not(target_arch = "wasm32"))]
fn flush_annotations_to_disk(
    time: Res<Time>,
    mut dirty: ResMut<AnnotationsDirty>,
    game_map: Res<GameMap>,
    annotations: Option<Res<browser::SpriteAnnotationsResource>>,
) {
    if !dirty.dirty {
        return;
    }
    dirty.idle += time.delta_secs();
    if dirty.idle < ANNOTATIONS_FLUSH_SECS {
        return;
    }
    if let Some(ann) = annotations {
        omdurman_map::save_annotations_to_file(&game_map, &ann.0, editor::ANNOTATIONS_SAVE_PATH);
    }
    dirty.dirty = false;
    dirty.idle = 0.0;
}

#[cfg(target_arch = "wasm32")]
fn flush_annotations_to_disk(_dirty: ResMut<AnnotationsDirty>) {
    // No-op on WASM — annotations live in memory only.
}

fn flush_pending(
    mut pending: ResMut<PendingEdits>,
    net: Res<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    if pending.outgoing_broadcast.is_empty() && pending.outgoing_targeted.is_empty() {
        return;
    }
    let Ok(mut socket) = socket_q.single_mut() else {
        return;
    };

    // The matchbox channel is an `mpsc::UnboundedSender` internally — `try_send`
    // can only fail if the receiving socket task has been dropped, which means
    // the socket is dead and no retry will recover it. So we log and move on.
    let channel = socket.channel_mut(CH_RELIABLE);

    for (msg, peer) in pending.outgoing_targeted.drain(..) {
        if let Err(e) = channel.try_send(enc_msg(&msg), peer) {
            warn!(error = %e, "reliable targeted send failed; socket likely dead");
        }
    }

    for msg in pending.outgoing_broadcast.drain(..) {
        let encoded = enc_msg(&msg);
        for &peer in &net.peers {
            if let Err(e) = channel.try_send(encoded.clone(), peer) {
                warn!(error = %e, "reliable broadcast send failed; socket likely dead");
            }
        }
    }
}

fn camera_control(
    time: Res<Time>,
    settings: Res<CameraSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut scroll_events: MessageReader<MouseWheel>,
    mut drag_state: ResMut<CameraDragState>,
    windows: Query<&Window>,
    mut cam_q: Query<(&mut RtsCameraState, &mut Transform), With<RtsCamera>>,
    mode: Res<EditorMode>,
    mut contexts: EguiContexts,
    touches: Res<Touches>,
) {
    if matches!(*mode, EditorMode::Units | EditorMode::EventViewer) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((mut state, mut transform)) = cam_q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    // ── Right-click drag pan ──────────────────────────────────────────────
    let cursor_pos = windows.single().ok().and_then(|w| w.cursor_position());
    if !ctx.wants_pointer_input() {
        if buttons.just_pressed(MouseButton::Right) {
            drag_state.active = true;
            if let Some(pos) = cursor_pos {
                drag_state.last_cursor = pos;
            }
        } else if buttons.just_released(MouseButton::Right) {
            drag_state.active = false;
        }
    } else {
        drag_state.active = false;
    }

    if drag_state.active
        && let (Some(pos), false) = (cursor_pos, ctx.wants_pointer_input())
    {
        let delta = Vec2::new(
            pos.x - drag_state.last_cursor.x,
            pos.y - drag_state.last_cursor.y,
        );
        if delta.length_squared() > 0.0 {
            // Convert screen-space drag delta to world-space focus delta.
            // At distance 500 the scale is ~1 world unit per pixel, tuned by feel.
            let scale = (state.distance / 500.0) * 0.6;
            let fwd = Vec3::new(-state.yaw.sin(), 0.0, -state.yaw.cos());
            let right = Vec3::new(fwd.z, 0.0, -fwd.x);
            state.focus += fwd * delta.y * scale + right * delta.x * scale;
        }
        drag_state.last_cursor = pos;
    }

    // ── Arrow-key pan ────────────────────────────────────────────────────
    let mut pan = Vec2::ZERO;
    if !ctx.wants_keyboard_input() {
        if keys.pressed(KeyCode::ArrowUp) {
            pan.y += 1.0;
        }
        if keys.pressed(KeyCode::ArrowDown) {
            pan.y -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            pan.x -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            pan.x += 1.0;
        }
    }
    if pan != Vec2::ZERO {
        pan = pan.normalize() * settings.pan_speed * dt * (state.distance / 500.0).max(0.3);
        let fwd = Vec3::new(-state.yaw.sin(), 0.0, -state.yaw.cos());
        let right = Vec3::new(fwd.z, 0.0, -fwd.x);
        state.focus += fwd * pan.y + right * pan.x;
    }

    let mut zoom_ticks: f32 = 0.0;
    if !ctx.wants_pointer_input() {
        for ev in scroll_events.read() {
            let notch_scale = match ev.unit {
                MouseScrollUnit::Pixel => 0.01,
                MouseScrollUnit::Line => 1.0,
            };
            zoom_ticks += ev.y * notch_scale;
        }
    }
    if zoom_ticks != 0.0 {
        if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
            state.pitch =
                (state.pitch + zoom_ticks * 0.1).clamp(settings.min_pitch, settings.max_pitch);
        } else {
            let factor = 1.0 - zoom_ticks.clamp(-5.0, 5.0) * 0.06;
            state.distance =
                (state.distance * factor).clamp(settings.min_distance, settings.max_distance);
        }
    }

    let pitch_step = dt * 0.8;
    if !ctx.wants_keyboard_input() {
        if keys.pressed(KeyCode::PageUp) {
            state.pitch = (state.pitch + pitch_step).min(settings.max_pitch);
        }
        if keys.pressed(KeyCode::PageDown) {
            state.pitch = (state.pitch - pitch_step).max(settings.min_pitch);
        }
    }

    // ── Touch gestures (pinch zoom + two-finger pitch) ────────────────────
    if !ctx.wants_pointer_input() {
        let mut touches_iter = touches.iter();
        if let (Some(t0), Some(t1)) = (touches_iter.next(), touches_iter.next()) {
            // pinch zoom
            let prev_dist = t0.previous_position().distance(t1.previous_position());
            let cur_dist = t0.position().distance(t1.position());
            let pinch_delta = cur_dist - prev_dist;
            if pinch_delta != 0.0 {
                let factor = 1.0 - pinch_delta.clamp(-30.0, 30.0) * 0.02;
                state.distance =
                    (state.distance * factor).clamp(settings.min_distance, settings.max_distance);
            }
            // two-finger vertical drag → pitch
            let prev_mid_y = (t0.previous_position().y + t1.previous_position().y) * 0.5;
            let cur_mid_y = (t0.position().y + t1.position().y) * 0.5;
            let pitch_delta = cur_mid_y - prev_mid_y;
            if pitch_delta != 0.0 {
                state.pitch = (state.pitch - pitch_delta * 0.02)
                    .clamp(settings.min_pitch, settings.max_pitch);
            }
        }
    }

    let t = (settings.smoothing * dt).min(1.0);
    state.smooth_focus = state.smooth_focus.lerp(state.focus, t);
    state.smooth_distance = state.smooth_distance.lerp(state.distance, t);
    state.smooth_yaw = state.smooth_yaw.lerp(state.yaw, t);
    state.smooth_pitch = state.smooth_pitch.lerp(state.pitch, t);

    let hdist = state.smooth_distance * state.smooth_pitch.cos();
    let vert = state.smooth_distance * state.smooth_pitch.sin();
    let offset = Vec3::new(
        hdist * state.smooth_yaw.sin(),
        vert,
        hdist * state.smooth_yaw.cos(),
    );
    let eye = state.smooth_focus + offset;
    *transform = Transform::from_translation(eye).looking_at(state.smooth_focus, Vec3::Y);
}

fn spawn_ground(mut commands: Commands) {
    commands.spawn((RigidBody::Static, Collider::half_space(Vec3::Y)));
}

fn despawn_dice(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Dice)>) {
    for (entity, mut dice) in query.iter_mut() {
        dice.timer.tick(time.delta());
        if dice.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(50.0, 100.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            ..default()
        },
        Transform::from_xyz(-50.0, 50.0, -50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn load_annotations(
    mut commands: Commands,
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<render::HexOverlay>,
    mut current: ResMut<EditorMode>,
) {
    let ron_str = include_str!("../assets/annotations.ron");
    let annotations = load_annotations_from_str(ron_str, &mut game_map);
    overlay.params = game_map.overlay.clone();
    commands.insert_resource(browser::SpriteAnnotationsResource(annotations.sprites));
    *current = EditorMode::Normal;
}

pub fn d10_collider_points(radius: f32, height: f32) -> Vec<Vec3> {
    let n = 5;
    let mut points = vec![
        Vec3::new(0.0, height / 2.0, 0.0),
        Vec3::new(0.0, -height / 2.0, 0.0),
    ];
    for k in 0..n {
        let a = core::f32::consts::TAU * k as f32 / n as f32;
        points.push(Vec3::new(radius * a.cos(), 0.0, radius * a.sin()));
    }
    points
}

pub fn d10_mesh_colored(radius: f32, height: f32) -> Mesh {
    let n = 5;
    let top = [0.0, height / 2.0, 0.0];
    let bot = [0.0, -height / 2.0, 0.0];

    let mut ring = Vec::new();
    for k in 0..n {
        let a = core::f32::consts::TAU * k as f32 / n as f32;
        ring.push([radius * a.cos(), 0.0, radius * a.sin()]);
    }

    let pastel_colors: [[f32; 4]; 10] = [
        [0.95, 0.85, 0.65, 1.0],
        [0.95, 0.75, 0.45, 1.0],
        [0.95, 0.65, 0.25, 1.0],
        [0.90, 0.55, 0.20, 1.0],
        [0.95, 0.45, 0.15, 1.0],
        [0.90, 0.35, 0.10, 1.0],
        [0.85, 0.25, 0.10, 1.0],
        [0.80, 0.20, 0.10, 1.0],
        [1.00, 0.90, 0.60, 1.0],
        [0.85, 0.55, 0.20, 1.0],
    ];

    let mut positions = Vec::new();
    let mut colors = Vec::new();

    for k in 0..n {
        let a = ring[k];
        let b = ring[(k + 1) % n];
        positions.extend_from_slice(&[top, b, a]);
        let c = pastel_colors[k];
        colors.extend_from_slice(&[c, c, c]);
        positions.extend_from_slice(&[bot, a, b]);
        let c = pastel_colors[k + 5];
        colors.extend_from_slice(&[c, c, c]);
    }

    let indices: Vec<u32> = (0..positions.len() as u32).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.compute_normals();
    mesh
}

// ── Late-joiner sync tests ────────────────────────────────────────────────────

#[cfg(test)]
mod late_joiner_tests {
    use super::*;
    use bevy::ecs::world::CommandQueue;
    use chrono::Utc;
    use omdurman_net::{
        EditorMode, GameEvent, GameRecord, InitialGameState, RecordedEvent, new_seed,
    };
    use omdurman_types::{
        HexCoord, MapSection, OverlayParams, SpriteAnnotation, SpriteAnnotations, Terrain, TileInfo,
    };

    /// Build a minimal GameRecord from a list of events.
    fn make_record(events: Vec<GameEvent>) -> GameRecord {
        let events = events
            .into_iter()
            .enumerate()
            .map(|(i, payload)| RecordedEvent {
                utc: Utc::now(),
                sender_idx: 0,
                seq: i as u32,
                payload,
            })
            .collect();
        GameRecord {
            initial_state: InitialGameState { seed: new_seed() },
            events,
        }
    }

    /// Empty annotations file whose overlay is sized to cover every coord
    /// referenced by the test suite. Used to seed a map before MapEdit /
    /// placement tests so those events have on-map hexes to target.
    fn empty_annotations_file() -> omdurman_types::AnnotationsFile {
        // EvenR with width=64, height=32 starts at q≥0 on row 0 and covers
        // a wide enough range that every test coordinate (q∈[0,9], r∈[0,9])
        // lands inside `desired_hexes`.
        let overlay = OverlayParams {
            width: 64,
            height: 32,
            offset_variant: omdurman_types::OffsetVariant::EvenR,
            ..Default::default()
        };
        omdurman_types::AnnotationsFile {
            map: MapSection {
                tiles: std::collections::HashMap::new(),
            },
            overlay,
            sprites: SpriteAnnotations::default(),
        }
    }

    /// Run replay_game_history with sensible defaults and return the modified state.
    fn run_replay(
        record: &GameRecord,
        total_peers: usize,
    ) -> (
        GameMap,
        render::HexOverlay,
        EditorMode,
        editor::HexEditor,
        browser::SpriteBrowser,
        Option<browser::SpriteAnnotationsResource>,
        units::UnitViewer,
        TurnState,
        Vec<(GameEvent, PeerId)>,
    ) {
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let mut game_map = GameMap::default();
        let mut overlay = render::HexOverlay::default();
        let mut editor = editor::HexEditor::default();
        let browser_state = browser::SpriteBrowser {
            sections: vec![],
            selected_sprite: None,
        };
        let mut annotations = Some(browser::SpriteAnnotationsResource(
            SpriteAnnotations::default(),
        ));
        let mut viewer = units::UnitViewer {
            grids: vec![],
            grids_dirty: false,
        };
        let mut turn = TurnState::default();
        let mut incoming: Vec<(GameEvent, PeerId)> = vec![];
        let history_peer = PeerId(uuid::Uuid::nil());

        replay_game_history(
            record,
            &mut commands,
            &mut game_map,
            &mut overlay,
            &mut editor,
            annotations.as_mut(),
            &mut viewer,
            &mut turn,
            total_peers,
            &mut incoming,
            history_peer,
        );
        queue.apply(&mut world);

        (
            game_map,
            overlay,
            EditorMode::default(),
            editor,
            browser_state,
            annotations,
            viewer,
            turn,
            incoming,
        )
    }

    // ── map edit ──────────────────────────────────────────────────────────────

    #[test]
    fn test_map_edit_replayed() {
        // MapEdit only applies to on-map coords; seed the map first.
        let record = make_record(vec![
            GameEvent::LoadAnnotations(empty_annotations_file()),
            GameEvent::MapEdit {
                q: 1,
                r: 2,
                terrain: Terrain::Desert as u8,
                name: "Khartoum".into(),
            },
        ]);
        let (game_map, ..) = run_replay(&record, 2);
        let hex = game_map
            .hexes
            .get(&HexCoord::new(1, 2))
            .expect("hex not found");
        assert_eq!(hex.terrain, Terrain::Desert);
        assert_eq!(hex.name.as_deref(), Some("Khartoum"));
    }

    // ── load annotations rebuilds the map ────────────────────────────────────

    #[test]
    fn test_load_annotations_replayed() {
        use std::collections::HashMap;
        let mut tiles = HashMap::new();
        tiles.insert(
            (3, 4),
            TileInfo {
                terrain: Terrain::BlueNile,
                name: Some("Nile".into()),
            },
        );
        let ann_file = omdurman_types::AnnotationsFile {
            map: MapSection { tiles },
            overlay: OverlayParams::default(),
            sprites: SpriteAnnotations::default(),
        };
        let record = make_record(vec![GameEvent::LoadAnnotations(ann_file)]);
        let (game_map, ..) = run_replay(&record, 2);
        let hex = game_map
            .hexes
            .get(&HexCoord::new(3, 4))
            .expect("hex not found");
        assert_eq!(hex.terrain, Terrain::BlueNile);
        assert_eq!(hex.name.as_deref(), Some("Nile"));
    }

    // ── overlay update synced ────────────────────────────────────────────────

    #[test]
    fn test_overlay_update_replayed() {
        let mut params = OverlayParams::default();
        params.hex_size = 99.0;
        let record = make_record(vec![GameEvent::OverlayUpdate(params.clone())]);
        let (_, overlay, ..) = run_replay(&record, 2);
        assert_eq!(overlay.params.hex_size, 99.0);
    }

    // ── annotate sprite ──────────────────────────────────────────────────────

    #[test]
    fn test_annotate_sprite_replayed() {
        use omdurman_types::{Faction, SpriteColor};
        let ann = SpriteAnnotation {
            text: "Camel Corps".into(),
            faction: Faction::Dervish,
            color: SpriteColor::GreenRed,
            a: 0,
            b: 0,
            c: 0,
            is_boat: false,
            is_unit: true,
        };
        let record = make_record(vec![
            GameEvent::LoadAnnotations(empty_annotations_file()),
            GameEvent::AnnotateSprite {
                section_name: "infantry".into(),
                col: 0,
                row: 1,
                annotation: ann.clone(),
            },
        ]);
        let (_, _, _, _, _, annotations, ..) = run_replay(&record, 2);
        let ann_res = annotations.unwrap();
        let entry = ann_res.0.units["infantry"][&(0, 1)].clone();
        assert_eq!(entry.text, "Camel Corps");
    }

    // ── unit placement queued for apply_pending_placement ────────────────────

    #[test]
    fn test_place_unit_queued_in_incoming() {
        let record = make_record(vec![GameEvent::PlaceUnit {
            section_name: "cavalry".into(),
            col: 2,
            row: 3,
            coord_q: 5,
            coord_r: 6,
            is_boat: false,
        }]);
        let (.., incoming) = run_replay(&record, 2);
        assert_eq!(incoming.len(), 1);
        match &incoming[0].0 {
            GameEvent::PlaceUnit {
                section_name,
                col,
                row,
                coord_q,
                coord_r,
                is_boat,
            } => {
                assert_eq!(section_name, "cavalry");
                assert_eq!(*col, 2);
                assert_eq!(*row, 3);
                assert_eq!(*coord_q, 5);
                assert_eq!(*coord_r, 6);
                assert!(!is_boat);
            }
            other => panic!("expected PlaceUnit, got {other:?}"),
        }
    }

    // ── move unit queued ─────────────────────────────────────────────────────

    #[test]
    fn test_move_unit_queued_in_incoming() {
        let record = make_record(vec![
            GameEvent::PlaceUnit {
                section_name: "arty".into(),
                col: 0,
                row: 0,
                coord_q: 1,
                coord_r: 1,
                is_boat: false,
            },
            GameEvent::MoveUnit {
                section_name: "arty".into(),
                col: 0,
                row: 0,
                to_q: 7,
                to_r: 8,
            },
        ]);
        let (.., incoming) = run_replay(&record, 2);
        assert_eq!(incoming.len(), 2);
        match &incoming[1].0 {
            GameEvent::MoveUnit { to_q, to_r, .. } => {
                assert_eq!(*to_q, 7);
                assert_eq!(*to_r, 8);
            }
            other => panic!("expected MoveUnit, got {other:?}"),
        }
    }

    // ── turn counter ─────────────────────────────────────────────────────────

    #[test]
    fn test_turn_counter_restored() {
        // 3 actions with 2 total peers → (3 % 2) == 1
        let record = make_record(vec![
            GameEvent::Action(10),
            GameEvent::Action(5),
            GameEvent::Action(7),
        ]);
        let (.., turn, _) = run_replay(&record, 2);
        assert_eq!(turn.current_turn, 1);
    }

    #[test]
    fn test_turn_counter_zero_actions() {
        let record = make_record(vec![]);
        let (.., turn, _) = run_replay(&record, 2);
        assert_eq!(turn.current_turn, 0);
    }

    // ── show terrain overlay ─────────────────────────────────────────────────

    #[test]
    fn test_show_terrain_overlay_replayed() {
        let record = make_record(vec![GameEvent::ShowTerrainOverlay(true)]);
        let (_, _, _, editor, ..) = run_replay(&record, 2);
        assert!(editor.show_terrain_overlay);
    }

    // ── unit grids synced ────────────────────────────────────────────────────

    #[test]
    fn test_unit_grids_replayed() {
        use omdurman_types::UnitGrid;
        let grids = vec![UnitGrid {
            name: "test_section".into(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            cols: 4,
            rows: 2,
        }];
        let record = make_record(vec![GameEvent::UpdateUnitGrids(grids.clone())]);
        let (_, _, _, _, _, _, viewer, ..) = run_replay(&record, 2);
        assert_eq!(viewer.grids.len(), 1);
        assert_eq!(viewer.grids[0].name, "test_section");
    }

    // ── move after place in same batch ───────────────────────────────────────

    #[test]
    fn test_move_after_place_queued_in_order() {
        // PlaceUnit at (1,1) then MoveUnit to (7,8) — both in the same replay
        // batch.  The incoming queue must contain both events in order so that
        // apply_pending_placement can use the just_placed fallback map to apply
        // the move even though Bevy hasn't flushed the spawn command yet.
        let record = make_record(vec![
            GameEvent::PlaceUnit {
                section_name: "cavalry".into(),
                col: 0,
                row: 0,
                coord_q: 1,
                coord_r: 1,
                is_boat: false,
            },
            GameEvent::MoveUnit {
                section_name: "cavalry".into(),
                col: 0,
                row: 0,
                to_q: 7,
                to_r: 8,
            },
        ]);
        let (.., incoming) = run_replay(&record, 2);
        assert_eq!(
            incoming.len(),
            2,
            "both PlaceUnit and MoveUnit must be queued"
        );
        // PlaceUnit comes first
        assert!(matches!(
            &incoming[0].0,
            GameEvent::PlaceUnit {
                coord_q: 1,
                coord_r: 1,
                ..
            }
        ));
        // MoveUnit comes second, with the target coords
        assert!(matches!(
            &incoming[1].0,
            GameEvent::MoveUnit {
                to_q: 7,
                to_r: 8,
                ..
            }
        ));
    }

    // ── map is cleared before replay ────────────────────────────────────────

    #[test]
    fn test_map_cleared_before_replay() {
        // Pre-populate the map with a hex that is NOT in the record.
        // After replay it must be gone, and the new hex must be present.
        let record = make_record(vec![
            GameEvent::LoadAnnotations(empty_annotations_file()),
            GameEvent::MapEdit {
                q: 0,
                r: 0,
                terrain: Terrain::Desert as u8,
                name: "".into(),
            },
        ]);

        // Run with a pre-populated world by inserting a stale hex
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let mut game_map = GameMap::default();
        game_map.hexes.insert(
            HexCoord::new(99, 99),
            omdurman_types::HexData::new(Terrain::Shrubs, None),
        );

        let mut overlay = render::HexOverlay::default();
        let mut editor = editor::HexEditor::default();
        let mut annotations = Some(browser::SpriteAnnotationsResource(
            SpriteAnnotations::default(),
        ));
        let mut viewer = units::UnitViewer {
            grids: vec![],
            grids_dirty: false,
        };
        let mut turn = TurnState::default();
        let mut incoming: Vec<(GameEvent, PeerId)> = vec![];
        let history_peer = PeerId(uuid::Uuid::nil());

        replay_game_history(
            &record,
            &mut commands,
            &mut game_map,
            &mut overlay,
            &mut editor,
            annotations.as_mut(),
            &mut viewer,
            &mut turn,
            2,
            &mut incoming,
            history_peer,
        );

        assert!(
            !game_map.hexes.contains_key(&HexCoord::new(99, 99)),
            "stale hex must be cleared before replay"
        );
        assert!(game_map.hexes.contains_key(&HexCoord::new(0, 0)));
    }

    /// Make sure any pre-existing on-disk game record still parses against
    /// the current schema. Run only on native; on WASM there are no files.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_saved_games_still_load() {
        let games_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../games");
        let Ok(entries) = std::fs::read_dir(games_dir) else {
            // Directory may not exist on a fresh checkout — that's fine.
            return;
        };
        let mut found = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("ron") {
                continue;
            }
            let content = std::fs::read_to_string(&path).expect("read saved game");
            let rec: GameRecord = ron::from_str(&content)
                .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
            assert!(
                rec.events.iter().any(|e| matches!(
                    e.payload,
                    GameEvent::LoadAnnotations(_)
                        | GameEvent::Action(_)
                        | GameEvent::MapEdit { .. }
                )) || rec.events.is_empty(),
                "record {} has events but none of the expected variants",
                path.display()
            );
            found += 1;
        }
        if found > 0 {
            eprintln!("verified {found} saved game record(s)");
        }
    }
}

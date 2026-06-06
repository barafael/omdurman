//! Remember Gordon! Battle of Omdurman.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod browser;
mod camera;
mod dice;
mod editor;
mod event_viewer;
mod fire;
mod game_apply;
mod game_record;
mod game_ui;
mod lobby;
mod melee;
mod picker;
mod render;
mod resize_pump;
mod retreat;
mod settings;
mod unit_profiles;
mod units;
mod util;

use crate::browser::SpriteAnnotationsResource;
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
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_matchbox::prelude::*;
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, load_annotations_from_str};
use omdurman_net::{
    CH_RELIABLE, CH_UNRELIABLE, Control, EditorMode, Ephemeral, GameEvent, GameRecord, GameRng,
    NetMsg, NetState, RoomId, decode, enc_msg, open_socket, room_id,
};
use omdurman_rules::effects::GameState;
use omdurman_rules::{UnitId, UnitPlacement, UnitProfile, UnitState};
use omdurman_types::HexCoord;
use std::{borrow::Cow, collections::HashMap};

/// Bevy resource wrapper around the rules engine's game state.
#[derive(Resource)]
pub struct GameStateResource(pub GameState);

/// Bundles the rules-engine state with the per-player faction binding so
/// `handle_socket` stays under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
struct GameStateParams<'w> {
    game_state: ResMut<'w, GameStateResource>,
    player_factions: ResMut<'w, PlayerFactions>,
    /// In-memory two-board annotations file; the `StartGame`/`LoadAnnotations`
    /// handlers store into it, and `request_map_load` reads from it.
    loaded_annotations: ResMut<'w, LoadedAnnotations>,
    /// Set by the `StartGame` handler (and the editor's map toggle) to ask
    /// `apply_map_selection` to (re)load a board on the next frame (§dual-map).
    pending_map_load: ResMut<'w, PendingMapLoad>,
    /// Which board is currently live, so map-edit events apply to the right
    /// section (§dual-map).
    active_edit_map: Res<'w, ActiveEditMap>,
}

/// Read-only bundle for the "may the local player act now" check (§lobby),
/// kept as one `SystemParam` so action handlers stay under the param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct FactionGate<'w> {
    pub factions: Res<'w, PlayerFactions>,
    pub net: Res<'w, NetState>,
}

impl FactionGate<'_> {
    /// Whether the local player controls `active` this phase.
    pub fn may_act(&self, active: omdurman_rules::Player) -> bool {
        self.factions.local_may_act(&self.net, active)
    }
}

/// Bundle of the rules state + faction gate used by movement gating in the
/// picker, so `handle_picker_clicks` stays under the param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct MoveGate<'w> {
    pub game_state: Option<Res<'w, GameStateResource>>,
    pub gate: FactionGate<'w>,
}

/// Maps rules-engine [`UnitId`] to the Bevy [`Entity`] of its visual
/// representation, so effects (elimination, disruption, movement) can
/// update or despawn the right 3D entity.
#[derive(Resource, Default)]
pub struct UnitEntityMap(pub std::collections::HashMap<UnitId, Entity>);

/// Tracks which unit entity is currently selected by the local player.
#[derive(Resource, Default)]
pub struct SelectedUnit(pub Option<Entity>);

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
    /// Host-only: `NetMsg::Sequenced` events the host just assigned a sequence
    /// number to, queued to be fed back through its own receive path so the
    /// host applies and records them in the same canonical order as everyone
    /// else. Drained at the top of `handle_socket` each frame.
    pub loopback: Vec<NetMsg>,
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
        .add_plugins(resize_pump::ResizePumpPlugin)
        .init_state::<AppState>()
        .add_message::<DiceRollResult>()
        .insert_resource(RoomId(room))
        .insert_resource(NetState::default())
        .insert_resource(TurnState::default())
        .insert_resource(GameStateResource(GameState::new(
            omdurman_rules::Scenario::Campaign,
        )))
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
        .insert_resource(UnitEntityMap::default())
        .insert_resource(SelectedUnit::default())
        .insert_resource(event_viewer::EventViewerState::default())
        .insert_resource(CursorPositions::default())
        .insert_resource(CursorBroadcastTimer::default())
        .insert_resource(PlayerFactions::default())
        .insert_resource(LobbyChoices::default())
        .insert_resource(LocalFaction::default())
        .insert_resource(HoveredHex::default())
        .insert_resource(omdurman_hex::MapDims::default())
        .insert_resource(LoadedAnnotations::default())
        .insert_resource(ActiveEditMap::default())
        .insert_resource(PendingMapLoad::default())
        .insert_resource(LobbyScenario::default())
        .insert_resource(HexLayout::calibrated(
            omdurman_types::Orientation::Pointy,
            Vec2::new(736.0, 420.0),
            omdurman_types::HexCoord::new(0, 0),
            Vec2::new(1178.0, 572.0),
            omdurman_types::HexCoord::new(5, -1),
            omdurman_hex::IMG_W,
            omdurman_hex::IMG_H,
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
                    // Editor (terrain / hexside) input + gizmos.
                    (
                        editor::editor_terrain_keys,
                        editor::handle_hex_editor_click,
                        editor::handle_hexside_select,
                        editor::handle_hexside_keys,
                        editor::draw_editor_highlight,
                        editor::draw_hexsides,
                        editor::draw_hexside_hover,
                        editor::draw_excluded_hexes,
                        editor::draw_nile_flow_indicators,
                    ),
                    despawn_dice,
                    handle_reconnect,
                    retry_snapshot_request.after(handle_reconnect),
                    handle_socket.after(handle_reconnect),
                    apply_pending_placement.after(handle_socket),
                    apply_ephemeral.after(apply_pending_placement),
                    (
                        handle_local_input.after(handle_socket),
                        update_status_text.after(handle_socket),
                        update_hex_coord_display,
                        units::draw_unit_grids,
                        picker::placement_preview_gizmo,
                        fire::handle_fire_combat.before(picker::handle_picker_clicks),
                        melee::handle_melee_combat.before(picker::handle_picker_clicks),
                        melee::handle_advance_after_combat
                            .after(melee::handle_melee_combat)
                            .after(fire::handle_fire_combat)
                            .before(picker::handle_picker_clicks),
                        retreat::handle_retreat.before(picker::handle_picker_clicks),
                        picker::handle_picker_clicks,
                        picker::movement_overlay_gizmo,
                        fire::fire_target_overlay_gizmo,
                        melee::melee_target_overlay_gizmo,
                        retreat::retreat_overlay_gizmo,
                        picker::animate_unit_movement,
                        picker::sync_disrupted_visuals,
                        picker::cancel_placement,
                    ),
                ),
                (
                    game_record::init_game_record.after(handle_socket),
                    game_record::host_emit_annotations
                        .after(game_record::init_game_record)
                        .before(flush_pending),
                    // Recording happens on `Sequenced` receipt in `handle_socket`
                    // (the single canonical apply point), so `flush_game_record`
                    // only needs to run after the socket is processed.
                    game_record::flush_game_record.after(handle_socket),
                    send_player_info_on_connect.after(handle_socket),
                    prune_disconnected_peers.after(handle_socket),
                    broadcast_cursor,
                    broadcast_browser_selection,
                    flush_pending,
                    flush_annotations_to_disk,
                    sync_edit_board_to_mode,
                    sync_lobby_appstate,
                    apply_map_selection
                        .after(handle_socket)
                        .after(sync_edit_board_to_mode),
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
                editor::hexside_editor_ui,
                units::unit_grids_ui,
                units::unit_grid_labels,
                browser::sprite_meta_editor_ui,
                dice::dice_sim_ui,
                picker::unit_picker_ui,
                event_viewer::event_viewer_ui,
                settings::settings_ui,
                game_ui::game_state_ui,
                lobby::lobby_ui,
                melee::melee_reaction_ui,
            ),
        )
        .run();
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    /// Networking handshake while a guest fetches a snapshot. Reached only via
    /// the voluntarily-entered Lobby (see `EditorMode::Lobby`).
    Connecting,
    /// Pre-game lobby: peers are connected, players pick factions and see each
    /// other's names/colours/cursors. The host commits the assignment and
    /// starts the game (§lobby). Entered voluntarily from the mode dropdown.
    Lobby,
    /// Local or networked play/editing session. The app launches here as a
    /// local session and ignores peers until the lobby is voluntarily entered.
    #[default]
    InGame,
}

/// Authoritative per-player faction binding, established by the host's
/// `GameEvent::StartGame` (§lobby). Keyed by `PeerId`; the local player's
/// faction is `factions.get(&net.my_id)`.
#[derive(Resource, Default)]
pub struct PlayerFactions {
    pub by_peer: HashMap<PeerId, omdurman_rules::Player>,
}

impl PlayerFactions {
    /// The faction the local peer commands, if assigned.
    pub fn local(&self, net: &NetState) -> Option<omdurman_rules::Player> {
        net.my_id.and_then(|id| self.by_peer.get(&id).copied())
    }

    /// Whether the local player may act right now: their faction is the rules
    /// engine's active player. Before any binding exists (solo sandbox / no
    /// lobby) this returns `true` so the game stays playable. (§lobby)
    pub fn local_may_act(&self, net: &NetState, active: omdurman_rules::Player) -> bool {
        match self.local(net) {
            Some(mine) => mine == active,
            None => self.by_peer.is_empty(), // unbound sandbox → no restriction
        }
    }
}

/// Parse a `PeerId` from its string form (the canonical UUID text), as carried
/// in [`GameEvent::StartGame`]. Returns `None` for malformed input.
fn parse_peer_id(s: &str) -> Option<PeerId> {
    uuid::Uuid::parse_str(s).ok().map(PeerId)
}

/// Which board a scenario plays on: the Campaign game uses the strategic
/// campaign map; the Historical and Fall-of-Khartoum scenarios share the
/// tactical Fall-of-Khartoum map (§dual-map).
pub fn map_kind_for_scenario(scenario: omdurman_rules::Scenario) -> omdurman_types::MapKind {
    match scenario {
        omdurman_rules::Scenario::Campaign => omdurman_types::MapKind::Campaign,
        omdurman_rules::Scenario::Historical | omdurman_rules::Scenario::FallOfKhartoum => {
            omdurman_types::MapKind::FallOfKhartoum
        }
    }
}

/// The full two-board annotations file, kept in memory so map switches, edits,
/// and disk saves can address either board without re-reading from disk
/// (§dual-map). Seeded by `LoadAnnotations`; the active board's section is
/// rewritten from the live [`GameMap`] on save.
#[derive(Resource)]
pub struct LoadedAnnotations(pub omdurman_types::AnnotationsFile);

impl Default for LoadedAnnotations {
    fn default() -> Self {
        Self(omdurman_types::AnnotationsFile::empty())
    }
}

/// Which board the editor/overlay tools currently act on (§dual-map). Local to
/// each peer — calibration is a dev tool, not replicated state. Switching it
/// reloads the corresponding board into the live `GameMap`/overlay/layout.
#[derive(Resource, Default)]
pub struct ActiveEditMap(pub omdurman_types::MapKind);

/// A deferred request to (re)load a board into the live `GameMap`/`HexOverlay`/
/// `MapDims`/`HexLayout` and re-texture the map plane. Set by the `StartGame`
/// handler and the editor's map toggle; consumed by `apply_map_selection`,
/// which has the asset/material access those handlers lack (§dual-map).
#[derive(Resource, Default)]
pub struct PendingMapLoad(pub Option<omdurman_types::MapKind>);

/// Host's lobby scenario selection (§lobby), committed into
/// [`GameEvent::StartGame`]. Other peers see it as a live preview via
/// [`Ephemeral::ScenarioChoice`].
#[derive(Resource)]
pub struct LobbyScenario(pub omdurman_rules::Scenario);

impl Default for LobbyScenario {
    fn default() -> Self {
        Self(omdurman_rules::Scenario::Campaign)
    }
}

/// Live (pre-commit) lobby faction picks, keyed by `PeerId`. Populated from
/// `Ephemeral::FactionChoice` for display in the lobby; the local pick lives in
/// `LocalFaction`.
#[derive(Resource, Default)]
pub struct LobbyChoices {
    pub by_peer: HashMap<PeerId, Option<omdurman_rules::Player>>,
    /// Latest scenario broadcast by the host's lobby (live preview, §lobby).
    /// `None` until the host sends one; the committed value rides in
    /// [`GameEvent::StartGame`].
    pub scenario: Option<omdurman_rules::Scenario>,
}

/// The local player's current lobby faction pick (pre-commit).
#[derive(Resource, Default)]
pub struct LocalFaction(pub Option<omdurman_rules::Player>);

#[derive(Resource, Default)]
struct TurnState {
    my_index: usize,
    current_turn: usize,
    pending_roll: Option<u32>,
    game_started: bool,
    /// Set when the local player submits an `Action` and cleared when the
    /// host-sequenced echo of *any* `Action` is applied. Under host-relay the
    /// turn advances only on that echo (apply-on-echo, §ordering), so this
    /// guards against the player acting again during the round trip.
    action_in_flight: bool,
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

/// Written every frame by `render::update_selection_marker` with the hex
/// currently under the cursor (or `None` if no valid hex is hovered).
#[derive(Resource, Default)]
pub struct HoveredHex(pub Option<HexCoord>);

#[derive(Component)]
struct HexCoordLabel;

#[derive(Component)]
struct HexCoordPane;

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

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(14.0),
                right: Val::Px(14.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            HexCoordPane,
        ))
        .with_child((
            HexCoordLabel,
            Text::new(""),
            TextFont {
                font_size: 16.0,
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
    incoming.loopback.clear();
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
        if peers_changed || my_id_just_set {
            // Re-derive our position in the sorted list. If we're not present
            // (shouldn't happen — we should always include ourselves) keep the
            // old index so the game doesn't wedge.
            // Host election: the lowest-sorted PeerId is the host. Re-run on
            // every peer change (and once our own id is known) so a guest gets
            // promoted when the previous host disconnects, and a solo player —
            // whose sorted list is just `[my_id]` — elects itself host so it
            // can sequence its own events (§host-relay).
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

        // Lobby is entered voluntarily (via `EditorMode::Lobby`), not
        // auto-triggered by peers appearing — so a local editing session is
        // never dragged into someone else's game. The mode→state transition
        // (and the guest snapshot request) lives in `sync_lobby_appstate`.
    }

    // Message processing runs in both Lobby and InGame: the lobby needs to
    // receive faction picks, the host's `StartGame`, and snapshot replies.
    if *state.get() == AppState::Connecting {
        return;
    }

    let mut targeted: Vec<(NetMsg, PeerId)> = Vec::new();
    let mut sequenced_out: Vec<NetMsg> = Vec::new();
    let reliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_RELIABLE).receive();
    let unreliable: Vec<(PeerId, Box<[u8]>)> = socket.channel_mut(CH_UNRELIABLE).receive();
    let total_peers = net.sorted_all().len().max(1);
    let is_host = net.is_host;

    // Host loopback: events the host sequenced for itself (below). They flow
    // through the identical apply path as remote `Sequenced` events so every
    // peer — host included — observes the same ordered stream. `my_id` is the
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
            // Guest→host submission. The host assigns the next canonical
            // sequence number and rebroadcasts as `Sequenced`; the raw `Game`
            // form is never applied directly. A guest should never receive
            // this — if it does, the sender mistook us for the host (stale
            // host election); drop it so the originator retries.
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
            // Canonical, host-ordered event: the single apply + record point
            // for every peer (§ordering).
            NetMsg::Sequenced { seq, event: ev } => {
                recorder.push_event(&ev, sender_idx, seq);
                match &ev {
                    // Placement needs picker + asset access; defer to
                    // `apply_pending_placement`.
                    GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } => {
                        incoming.live.push((ev, peer, sender_idx));
                    }
                    // The host's faction commit: establish the binding and
                    // start the game on every peer (§lobby). Handled here (not
                    // in apply_game_event) because it needs net identity, the
                    // turn state, and the app-state transition.
                    GameEvent::StartGame {
                        assignments,
                        scenario,
                    } => {
                        // Only honour a start if we're actually in the lobby
                        // flow. A local editing session (the default) ignores a
                        // peer's StartGame so its board/state isn't swapped out
                        // from under the user (§lobby, voluntary entry).
                        if *state.get() != AppState::Lobby {
                            info!(%scenario, "ignoring StartGame; not in lobby");
                        } else {
                            gsp.player_factions.by_peer.clear();
                            for (peer_str, faction) in assignments {
                                if let Some(pid) = parse_peer_id(peer_str) {
                                    gsp.player_factions.by_peer.insert(pid, *faction);
                                }
                            }
                            // Seed the rules engine from the committed scenario,
                            // then make the Anglo-Egyptian player move first (§4).
                            gsp.game_state.0 = GameState::new(*scenario);
                            gsp.game_state.0.active_player = omdurman_rules::Player::AngloEgyptian;
                            // Load the board this scenario plays on (§dual-map).
                            gsp.pending_map_load.0 = Some(map_kind_for_scenario(*scenario));
                            if !turn.game_started {
                                turn.game_started = true;
                                turn.current_turn = 0;
                                next_state.set(AppState::InGame);
                                info!(%scenario, "game started via host StartGame");
                            }
                        }
                    }
                    _ => {
                        // Action advances the turn here because `apply_game_event`
                        // is peer-agnostic by design — used by replay too, where
                        // the live peer count isn't meaningful. This is the
                        // single turn-advance point (apply-on-echo), so clear
                        // any locally-flagged in-flight action too.
                        if matches!(&ev, GameEvent::Action(_)) {
                            turn.current_turn = (turn.current_turn + 1) % total_peers;
                            turn.action_in_flight = false;
                        }
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
                // Only the host answers, and only it — so a late joiner gets
                // exactly one copy of the log (§snapshot). A non-host that
                // receives this (stale host election on the sender) ignores it;
                // the guest's retry will reach the real host.
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
                // Install the host's record locally so the Event Viewer on
                // guests sees the full history, not just LoadAnnotations.
                recorder.install_history(record.clone());
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
                    &mut gsp.game_state.0,
                    &mut gsp.player_factions,
                    &mut gsp.loaded_annotations,
                    &mut gsp.pending_map_load,
                );
            }
        }
    }
    // queue targeted sends (flushed by flush_pending later)
    for (msg, peer) in targeted {
        pending.outgoing_targeted.push((msg, peer));
    }
    // queue host-sequenced game events for broadcast to every peer.
    for msg in sequenced_out {
        pending.outgoing_broadcast.push(msg);
    }
}

/// Look up a counter's authored [`SpriteAnnotation`] and build its rules
/// profile. Returns `None` if annotations aren't loaded yet, the counter has
/// no annotation, or its section name is unrecognised — in every case the
/// unit is placed visually but acquires no rules-engine `UnitId`.
fn profile_for(
    annotations: Option<&SpriteAnnotationsResource>,
    section_name: &str,
    col: u32,
    row: u32,
) -> Option<UnitProfile> {
    let annotation = annotations?
        .0
        .units
        .get(section_name)
        .and_then(|m| m.get(&(col, row)))?;
    unit_profiles::profile_from_annotation(section_name, col, annotation)
}

/// Route a unit move through the rules engine so it validates the move
/// (allowance, phase, ZOC, night-halving) and updates `unit.position`
/// authoritatively. The visual `GameEvent::MoveUnit` still animates the
/// sprite; this keeps the engine state in step. A rejected move is logged
/// (the sprite still moves for now — the app is not yet phase-gated) rather
/// than silently patching position.
fn apply_move_effect(state: &mut GameState, unit_id: UnitId, to: HexCoord) {
    let Some(unit) = state.find_unit(unit_id) else {
        warn!(?unit_id, "MoveUnit for unknown rules unit");
        return;
    };
    let cost = omdurman_rules::MovementPoints(unit.position.distance(to) as i16);
    let effect = omdurman_rules::effects::GameEffect::MoveUnit { unit_id, to, cost };
    if let Err(error) = omdurman_rules::effects::apply_effect(state, &effect) {
        warn!(%error, ?unit_id, to.q = to.q, to.r = to.r, "move rejected by rules engine");
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
    anim_query: Query<&picker::MovementAnimation>,
    mut game_state: Option<ResMut<GameStateResource>>,
    mut unit_map: ResMut<UnitEntityMap>,
    annotations: Option<Res<SpriteAnnotationsResource>>,
    // Tracks entities spawned this invocation so MoveUnit can find units
    // placed in the same batch (e.g. during history replay) before Bevy
    // has flushed the deferred commands.
    // key: (section_name, col, row), value: (entity, is_boat, unit_id)
    mut just_placed: Local<HashMap<(String, u32, u32), (Entity, bool, Option<UnitId>)>>,
) {
    just_placed.clear();

    // Replay events and live events are both already recorded — replay by the
    // canonical host log, live by `handle_socket` when the host-sequenced
    // event was applied. Do NOT re-record here.
    let replay_items: Vec<_> = incoming.replay.drain(..).map(|(msg, _peer)| msg).collect();
    let live_items: Vec<_> = incoming.live.drain(..).map(|(msg, _, _)| msg).collect();

    for event in replay_items.into_iter().chain(live_items) {
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
                // Local entity from handle_picker_clicks has unit_id: None;
                // allocate the rules-engine UnitId and update it in place.
                if let Some((entity, mut placed)) = placed_units.iter_mut().find(|(_, u)| {
                    u.unit_id.is_none()
                        && u.section_name == section_name
                        && u.col == col
                        && u.row == row
                        && u.coord == coord
                }) {
                    let profile: Option<UnitProfile> =
                        profile_for(annotations.as_deref(), &section_name, col, row);
                    let allocated = game_state.as_mut().and_then(|gs| {
                        let id = gs.0.alloc_unit_id();
                        let p = profile?;
                        gs.0.units.push(UnitPlacement {
                            id,
                            position: coord,
                            profile: p,
                            state: UnitState::default(),
                        });
                        Some(id)
                    });
                    placed.unit_id = allocated;
                    if let Some(id) = allocated {
                        unit_map.0.insert(id, entity);
                    }
                    continue;
                }
                let unit_idx = picker
                    .available
                    .iter()
                    .position(|u| u.section_name == section_name && u.col == col && u.row == row);
                if let Some(idx) = unit_idx {
                    let unit = picker.available.remove(idx);

                    // Allocate rules-engine UnitId and record placement in
                    // GameState so effect processing can refer to the unit.
                    let profile: Option<UnitProfile> =
                        profile_for(annotations.as_deref(), &section_name, col, row);
                    let allocated = game_state.as_mut().and_then(|gs| {
                        let id = gs.0.alloc_unit_id();
                        let p = profile?;
                        gs.0.units.push(UnitPlacement {
                            id,
                            position: coord,
                            profile: p,
                            state: UnitState::default(),
                        });
                        Some(id)
                    });

                    let origin = crate::util::adjusted_origin(
                        &layout,
                        overlay.params.offset_x,
                        overlay.params.offset_y,
                    );
                    let pos = crate::util::hex_world_pos(coord, origin, &overlay.params);
                    let entity = picker::spawn_placed_unit(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        unit.handle.clone(),
                        &overlay,
                        pos,
                        picker::PlacedUnit {
                            coord,
                            section_name: section_name.clone(),
                            col,
                            row,
                            is_boat,
                            unit_id: allocated,
                            disrupted: false,
                        },
                    );
                    if let Some(id) = allocated {
                        unit_map.0.insert(id, entity);
                    }
                    info!(
                        col,
                        row,
                        coord.q = coord.q,
                        coord.r = coord.r,
                        "applied placement"
                    );
                    just_placed.insert((section_name, col, row), (entity, is_boat, allocated));
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
                        // Route through the rules engine so it validates and
                        // owns the position update (see apply_move_effect).
                        if let Some(unit_id) = placed.unit_id
                            && let Some(ref mut gs) = game_state
                        {
                            apply_move_effect(&mut gs.0, unit_id, target);
                        }
                        // Don't snap if a local movement animation is already
                        // playing — let animate_unit_movement finish it.
                        if anim_query.get(entity).is_err() {
                            commands.entity(entity).insert(new_transform);
                            commands
                                .entity(entity)
                                .remove::<picker::MovementAnimation>();
                        }
                        found = true;
                        break;
                    }
                }

                // Fall back to units placed earlier in this same batch
                // (replay path — Bevy commands are still deferred).
                if !found
                    && let Some(&(entity, is_boat, unit_id)) =
                        just_placed.get(&(section_name.clone(), col, row))
                {
                    // Route through the rules engine (see apply_move_effect).
                    if let Some(uid) = unit_id
                        && let Some(ref mut gs) = game_state
                    {
                        apply_move_effect(&mut gs.0, uid, target);
                    }
                    commands.entity(entity).insert(picker::PlacedUnit {
                        coord: target,
                        section_name: section_name.clone(),
                        col,
                        row,
                        is_boat,
                        unit_id,
                        // Re-synced by `sync_disrupted_visuals` next frame.
                        disrupted: false,
                    });
                    info!(
                        col,
                        row,
                        to.q = target.q,
                        to.r = target.r,
                        "applied move (replay fallback)"
                    );
                    commands.entity(entity).insert(new_transform);
                    // update the map so subsequent moves on the same unit work
                    just_placed.insert((section_name, col, row), (entity, is_boat, unit_id));
                }
                if found {
                    info!(col, row, to.q = target.q, to.r = target.r, "applied move");
                }
            }
            // Other GameEvent variants are applied inline by handle_socket /
            // replay_game_history — they shouldn't appear in the deferred
            // queues. Warn if one does so the misclassification is visible.
            other => warn!(?other, "non-placement GameEvent in placement queue"),
        }
    }

    // ── Ephemeral messages handled by apply_ephemeral() — see below ──
}

/// Applies [`Ephemeral`] messages that were routed into `PendingIncoming`
/// by [`handle_socket`].  These are outside the event-sourcing record and
/// affect only local presentation (cursor positions, player info, etc.).
fn apply_ephemeral(
    mut incoming: ResMut<PendingIncoming>,
    mut player_info: ResMut<settings::PlayerInfoMap>,
    mut cursor_positions: ResMut<CursorPositions>,
    mut event_viewer: Option<ResMut<event_viewer::EventViewerState>>,
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
                    settings::PeerPlayerInfo {
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
    game_state: &mut GameState,
    player_factions: &mut PlayerFactions,
    loaded_annotations: &mut LoadedAnnotations,
    pending_map_load: &mut PendingMapLoad,
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
        game_state: Some(game_state),
        loaded_annotations: Some(loaded_annotations),
        // Replay rebuilds from the default board; `apply_map_selection` reloads
        // the scenario's board from the accumulated `LoadedAnnotations` after
        // replay completes (§dual-map).
        active_map: omdurman_types::MapKind::FallOfKhartoum,
    };
    for event in &record.events {
        match &event.payload {
            GameEvent::Action(_) => action_count += 1,
            GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } => {
                replay.push((event.payload.clone(), history_peer));
                continue;
            }
            // Reconstruct the faction binding for a late joiner from the
            // recorded host commit (§lobby); the engine state's active player
            // is also seeded so the replayed game is consistent.
            GameEvent::StartGame {
                assignments,
                scenario,
            } => {
                player_factions.by_peer.clear();
                for (peer_str, faction) in assignments {
                    if let Some(pid) = parse_peer_id(peer_str) {
                        player_factions.by_peer.insert(pid, *faction);
                    }
                }
                if let Some(gs) = ctx.game_state.as_deref_mut() {
                    *gs = GameState::new(*scenario);
                    gs.active_player = omdurman_rules::Player::AngloEgyptian;
                }
                // Defer the board (re)load until after replay completes, so the
                // late joiner lands on the scenario's map (§dual-map).
                pending_map_load.0 = Some(map_kind_for_scenario(*scenario));
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
            // Also send our current lobby faction pick so a newly-connected
            // peer sees it without waiting for us to re-click (§lobby).
            pending.outgoing_targeted.push((
                NetMsg::Ephemeral(Ephemeral::FactionChoice(local_faction.0)),
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
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
        return;
    }
    if turn.current_turn != turn.my_index {
        return;
    }
    // An action we already submitted hasn't been sequenced back yet — don't
    // let the player act again until the turn officially advances.
    if turn.action_in_flight {
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
        let tex = images.add(make_d10_texture());
        commands.spawn((
            RigidBody::Dynamic,
            Collider::convex_hull(collider_points).unwrap(),
            Mass(1.0),
            GravityScale(30.0),
            Mesh3d(meshes.add(d10_mesh_uv(radius, height))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(tex),
                unlit: true,
                alpha_mode: AlphaMode::Mask(0.5),
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
        // The turn advances when the host-sequenced echo of this `Action` is
        // applied (apply-on-echo). Flag the action as in-flight so the player
        // can't act again during the round trip.
        turn.action_in_flight = true;
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
        AppState::Lobby => Cow::Borrowed("Lobby — choose your faction"),
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

fn update_hex_coord_display(
    hovered: Res<HoveredHex>,
    mut query: Query<&mut Text, With<HexCoordLabel>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    let new = match hovered.0 {
        Some(coord) => format!("({}, {})", coord.q, coord.r),
        None => String::new(),
    };
    if text.as_str() != new {
        *text = Text::new(new);
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
        EditorMode::Editor | EditorMode::CampaignEditor => {
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
            EditorMode::UnitSheet | EditorMode::Units | EditorMode::EventViewer | EditorMode::Lobby
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
        EditorMode::CampaignOverlay => "Campaign Overlay",
        EditorMode::CampaignEditor => "Campaign Editor",
        EditorMode::Hexside => "Hexsides",
        EditorMode::CampaignHexside => "Campaign Hexsides",
        EditorMode::Lobby => "Lobby",
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
                                .selectable_value(&mut *current, EditorMode::Hexside, "Hexsides")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Hexside);
                            }
                            if ui
                                .selectable_value(
                                    &mut *current,
                                    EditorMode::CampaignOverlay,
                                    "Campaign Overlay",
                                )
                                .clicked()
                            {
                                clicked = Some(EditorMode::CampaignOverlay);
                            }
                            if ui
                                .selectable_value(
                                    &mut *current,
                                    EditorMode::CampaignEditor,
                                    "Campaign Editor",
                                )
                                .clicked()
                            {
                                clicked = Some(EditorMode::CampaignEditor);
                            }
                            if ui
                                .selectable_value(
                                    &mut *current,
                                    EditorMode::CampaignHexside,
                                    "Campaign Hexsides",
                                )
                                .clicked()
                            {
                                clicked = Some(EditorMode::CampaignHexside);
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
                            ui.separator();
                            if ui
                                .selectable_value(&mut *current, EditorMode::Lobby, "Lobby")
                                .clicked()
                            {
                                clicked = Some(EditorMode::Lobby);
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
    mode == EditorMode::Normal || mode.is_overlay() || mode.is_editor() || mode.is_hexside()
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
    loaded: Res<LoadedAnnotations>,
    active: Res<ActiveEditMap>,
) {
    if !dirty.dirty {
        return;
    }
    dirty.idle += time.delta_secs();
    if dirty.idle < ANNOTATIONS_FLUSH_SECS {
        return;
    }
    if let Some(ann) = annotations {
        // Write the whole two-board file, rewriting only the active board's
        // section so the other board's data is preserved (§dual-map).
        omdurman_map::save_annotations_to_file(
            &game_map,
            &ann.0,
            &loaded.0,
            active.0,
            editor::ANNOTATIONS_SAVE_PATH,
        );
    }
    dirty.dirty = false;
    dirty.idle = 0.0;
}

#[cfg(target_arch = "wasm32")]
fn flush_annotations_to_disk(_dirty: ResMut<AnnotationsDirty>) {
    // No-op on WASM — annotations live in memory only.
}

/// Flush staged reliable traffic onto the wire and route locally-originated
/// game events through the host-relay protocol (§ordering).
///
/// Routing of a staged `NetMsg::Game(event)` (a local submission) depends on
/// our role:
/// * **Host (or solo, i.e. no peers):** we *are* the sequencer. Assign the
///   next canonical `seq`, loop the resulting `Sequenced` back so we apply it
///   ourselves via `handle_socket`, and broadcast it to any peers. Because the
///   loopback always succeeds, the event is never lost even with zero peers.
/// * **Guest:** send the unsequenced `Game` to the host only. If the host is
///   unknown or the send fails, the message is **retained** for retry next
///   frame rather than dropped.
///
/// `NetMsg::Sequenced` entries are the host's already-ordered broadcasts; they
/// go to every peer (the host already looped its own copy back). Any other
/// staged reliable message is broadcast as-is.
///
/// Targeted and broadcast sends that fail are retained, so a transient socket
/// hiccup or a frame with a momentarily-empty peer list never silently drops
/// reliable traffic (#1/#2).
fn flush_pending(
    mut pending: ResMut<PendingEdits>,
    mut incoming: ResMut<PendingIncoming>,
    mut net: ResMut<NetState>,
    mut socket_q: Query<&mut MatchboxSocket>,
) {
    if pending.outgoing_broadcast.is_empty() && pending.outgoing_targeted.is_empty() {
        return;
    }

    // We are the sequencer when we're the elected host, or when we're alone
    // (no peers yet) — a solo player must sequence its own events locally.
    let i_sequence = net.is_host || net.peers.is_empty();
    let host = net.host_id();

    // First, route local game-event submissions. Host-sequenced events are
    // applied via loopback; guest submissions become targeted host sends. The
    // result is a flat list of wire messages still to broadcast, plus any
    // guest submissions that must be retained if the host send fails.
    let staged: Vec<NetMsg> = std::mem::take(&mut pending.outgoing_broadcast);
    let mut to_broadcast: Vec<NetMsg> = Vec::new();
    let mut retained_broadcast: Vec<NetMsg> = Vec::new();

    let mut socket = socket_q.single_mut().ok();

    for msg in staged {
        match msg {
            NetMsg::Game(event) if i_sequence => {
                // We order it ourselves: loop back for local application and
                // broadcast the canonical form to peers.
                let seq = net.next_seq;
                net.next_seq += 1;
                let sequenced = NetMsg::Sequenced { seq, event };
                incoming.loopback.push(sequenced.clone());
                to_broadcast.push(sequenced);
            }
            NetMsg::Game(event) => {
                // Guest: submit to the host for sequencing. Retain on failure.
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
            // Already-sequenced host broadcast, or any other reliable message:
            // broadcast to all peers below.
            other => to_broadcast.push(other),
        }
    }

    // Send targeted messages, retaining any that fail.
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

    // Broadcast remaining messages to every peer. If there are no peers, keep
    // them queued (rather than dropping) unless they were already looped back
    // locally — `Sequenced` events we produced are safe to drop here because
    // the loopback copy carries them; everything else is retained until a peer
    // exists to receive it.
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
    // Ctrl+arrows move the editor's hex selection (see `editor_terrain_keys`),
    // so plain arrows pan but Ctrl+arrows don't.
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let mut pan = Vec2::ZERO;
    if !ctx.wants_keyboard_input() && !ctrl {
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
    mut loaded: ResMut<LoadedAnnotations>,
    mut current: ResMut<EditorMode>,
) {
    let ron_str = include_str!("../assets/annotations.ron");
    // Startup default loads the Fall-of-Khartoum board; `StartGame` later swaps
    // to the scenario's board (§dual-map).
    let kind = omdurman_types::MapKind::FallOfKhartoum;
    let annotations = load_annotations_from_str(ron_str, kind, &mut game_map);
    overlay.params = game_map.overlay.clone();
    commands.insert_resource(browser::SpriteAnnotationsResource(
        annotations.map(kind).sprites.clone(),
    ));
    loaded.0 = annotations;
    *current = EditorMode::Normal;
}

/// Consume a [`PendingMapLoad`] request: reload the selected board into the live
/// `GameMap`/`HexOverlay`/`MapDims`/`HexLayout`, refresh the sprite annotations,
/// and re-size/re-texture the map plane (§dual-map).
///
/// Set by the `StartGame` handler (scenario → board) and the editor's active-map
/// toggle. Runs every frame but is a no-op unless a request is pending.
#[allow(clippy::too_many_arguments)]
fn apply_map_selection(
    mut pending: ResMut<PendingMapLoad>,
    loaded: Res<LoadedAnnotations>,
    mut active: ResMut<ActiveEditMap>,
    mut game_map: ResMut<GameMap>,
    mut overlay: ResMut<render::HexOverlay>,
    mut dims: ResMut<omdurman_hex::MapDims>,
    mut layout: ResMut<HexLayout>,
    mut annotations: Option<ResMut<browser::SpriteAnnotationsResource>>,
    mut commands: Commands,
    plane: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>), With<render::MapPlane>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Some(kind) = pending.0.take() else {
        return;
    };
    let map = loaded.0.map(kind);

    omdurman_map::load_map_data(map, &mut game_map);
    overlay.params = game_map.overlay.clone();
    *dims = omdurman_hex::MapDims {
        img_w: map.img_w,
        img_h: map.img_h,
    };
    *layout = HexLayout::calibrated(
        map.overlay.orientation,
        Vec2::new(map.calib.p1_px.0, map.calib.p1_px.1),
        omdurman_types::HexCoord::new(map.calib.p1_hex.0, map.calib.p1_hex.1),
        Vec2::new(map.calib.p2_px.0, map.calib.p2_px.1),
        omdurman_types::HexCoord::new(map.calib.p2_hex.0, map.calib.p2_hex.1),
        map.img_w,
        map.img_h,
    );
    if let Some(ref mut ann) = annotations {
        ann.0 = map.sprites.clone();
    } else {
        commands.insert_resource(browser::SpriteAnnotationsResource(map.sprites.clone()));
    }
    render::apply_map_data_to_plane(
        &plane,
        &mut meshes,
        &mut materials,
        &asset_server,
        &map.image,
        map.img_w,
        map.img_h,
    );
    active.0 = kind;
    info!(%kind, img_w = map.img_w, img_h = map.img_h, "loaded board");
}

/// Keep the active edit board in sync with the selected editor mode: entering a
/// board-editing mode (FoK or Campaign Overlay/Editor) requests a load of that
/// mode's board if it isn't already the active one (§dual-map). This is what
/// wires the "Campaign Overlay"/"Campaign Editor" dropdown entries (and the
/// Overlay/Editor entries) to the board they edit.
fn sync_edit_board_to_mode(
    mode: Res<EditorMode>,
    active: Res<ActiveEditMap>,
    mut pending: ResMut<PendingMapLoad>,
) {
    if !mode.is_changed() {
        return;
    }
    if let Some(board) = mode.edit_board()
        && board != active.0
        && pending.0.is_none()
    {
        pending.0 = Some(board);
    }
}

/// Drive the lobby `AppState` from the voluntarily-selected `EditorMode::Lobby`
/// (§lobby). Entering Lobby mode moves to `AppState::Lobby` and, for a guest,
/// requests a snapshot; leaving it returns to the local `InGame` session. The
/// game only leaves the lobby for real via the host's `StartGame`.
fn sync_lobby_appstate(
    mut mode: ResMut<EditorMode>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut net: ResMut<NetState>,
    mut pending: ResMut<PendingEdits>,
) {
    // The host's StartGame moves us to InGame while the mode is still Lobby —
    // drop back to Normal so the game board (not the lobby panel) is shown.
    if *state.get() == AppState::InGame && *mode == EditorMode::Lobby {
        *mode = EditorMode::Normal;
        return;
    }
    if !mode.is_changed() {
        return;
    }
    match (*mode, state.get()) {
        (EditorMode::Lobby, AppState::InGame) => {
            info!("entering lobby (voluntary)");
            next_state.set(AppState::Lobby);
            // A guest asks the host for the in-progress game history, if any.
            if !net.is_host && !net.peers.is_empty() {
                net.needs_snapshot = true;
                net.snapshot_retry_timer = 0.0;
                if let Some(host) = net.host_id() {
                    pending
                        .outgoing_targeted
                        .push((NetMsg::Control(Control::RequestSnapshot), host));
                }
            }
        }
        // Left Lobby mode without a game having started: back to local play.
        (m, AppState::Lobby) if m != EditorMode::Lobby => {
            next_state.set(AppState::InGame);
        }
        _ => {}
    }
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

pub fn d10_mesh_uv(radius: f32, height: f32) -> Mesh {
    let n = 5usize;
    let top = [0.0, height / 2.0, 0.0];
    let bot = [0.0, -height / 2.0, 0.0];

    let mut ring = Vec::new();
    for k in 0..n {
        let a = core::f32::consts::TAU * k as f32 / n as f32;
        ring.push([radius * a.cos(), 0.0, radius * a.sin()]);
    }

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let w = n as f32;

    // Top faces k=0..4 → tile index = k  (face number = k+1)
    for k in 0..n {
        let face = k as f32;
        let u0 = face / w;
        let u1 = (face + 1.0) / w;
        let uc = (face + 0.5) / w;

        positions.push(top);
        positions.push(ring[(k + 1) % n]);
        positions.push(ring[k]);

        uvs.push([uc, 0.0]); // top pole → top of image (digit head)
        uvs.push([u0, 1.0]); // ring[k+1] → bottom of image
        uvs.push([u1, 1.0]); // ring[k]
    }

    // Bottom faces j=0..4 → opposite top face is (j+3)%5
    // face number = 10 − (j+3)%5  → tile index = 9 − (j+3)%5
    for j in 0..n {
        let tile = 9 - (j + 3) % n;
        let u0 = tile as f32 / w;
        let u1 = (tile as f32 + 1.0) / w;
        let uc = (tile as f32 + 0.5) / w;

        positions.push(bot);
        positions.push(ring[j]);
        positions.push(ring[(j + 1) % n]);

        uvs.push([uc, 1.0]); // bot → bottom of image (digit feet)
        uvs.push([u0, 0.0]); // ring[j] → top of image
        uvs.push([u1, 0.0]); // ring[(j+1)%n] → top of image
    }

    let indices: Vec<u32> = (0..positions.len() as u32).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.compute_normals();
    mesh
}

// ── helpers for make_d10_texture ────────────────────────────────────────

/// Draw a 1‑px‑wide anti‑aliased line using a simple Bresenham‑style walk.
fn draw_line(data: &mut [u8], stride: u32, x0: u32, y0: u32, x1: u32, y1: u32, color: [u8; 4]) {
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = -(y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    loop {
        let idx = ((y as u32 * stride + x as u32) * 4) as usize;
        if idx + 3 < data.len() {
            data[idx..idx + 4].copy_from_slice(&color);
        }
        if x == x1 as i32 && y == y1 as i32 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Generate a 10‑tile texture atlas with digits 1…10 for the d10 faces.
pub fn make_d10_texture() -> Image {
    let tile_w = 64u32;
    let tile_h = 64u32;
    let w = tile_w * 10;
    let h = tile_h;

    // Pastel beige background (R=245, G=235, B=220)
    let mut data = vec![0u8; (w * h * 4) as usize];
    for px in data.chunks_exact_mut(4) {
        px.copy_from_slice(&[245, 235, 220, 255]);
    }

    // ── 1) Weak gray triangle outlines ─────────────────────────────────
    let gray = [180u8, 180, 180, 255];

    for tile in 0..10u32 {
        let ox = tile * tile_w;
        if tile < 5 {
            // Top face — apex at top-centre
            draw_line(&mut data, w, ox + 32, 0, ox + 0, 63, gray);
            draw_line(&mut data, w, ox + 32, 0, ox + 63, 63, gray);
            draw_line(&mut data, w, ox + 0, 63, ox + 63, 63, gray);
        } else {
            // Bottom face — apex at bottom-centre
            draw_line(&mut data, w, ox + 0, 0, ox + 63, 0, gray);
            draw_line(&mut data, w, ox + 0, 0, ox + 32, 63, gray);
            draw_line(&mut data, w, ox + 63, 0, ox + 32, 63, gray);
        }
    }

    // ── 2) Enlarged digit symbols ───────────────────────────────────────
    // 5 × 7 bitmap font for digits 0–9 (bit = filled pixel)
    let font: [[u8; 7]; 10] = [
        [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        [
            0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
        ],
        [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
    ];
    let scale = 5u32; // scale factor (5 × 7 → 25 × 35 px)
    let fw = 5u32;
    let fh = 7u32;
    let rw = fw * scale; // rendered width per char
    let rh = fh * scale; // rendered height

    for tile in 0..10u32 {
        let num = tile + 1;
        let s = num.to_string();
        let chars: Vec<_> = s.bytes().map(|b| (b - b'0') as usize).collect();
        let total_w = chars.len() as u32 * (rw + scale); // gap = scale
        let ox = tile * tile_w + (tile_w - total_w) / 2;
        let oy = (tile_h - rh) / 2;

        for (ci, &digit) in chars.iter().enumerate() {
            let bx = ox + ci as u32 * (rw + scale);
            for row in 0..fh {
                let bits = font[digit][row as usize];
                for col in 0..fw {
                    if bits & (1 << (4 - col)) != 0 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                let px = bx + col * scale + dx;
                                let py = oy + row * scale + dy;
                                let idx = ((py * w + px) * 4) as usize;
                                if idx + 3 < data.len() {
                                    data[idx] = 0;
                                    data[idx + 1] = 0;
                                    data[idx + 2] = 0;
                                    data[idx + 3] = 255;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Image {
        data: data.into(),
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..Default::default()
    }
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
        HexCoord, MapKind, OverlayParams, SpriteAnnotation, SpriteAnnotations, Terrain, TileInfo,
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
        let mut file = omdurman_types::AnnotationsFile::empty();
        file.fall_of_khartoum.overlay = overlay;
        file
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
            &mut GameStateResource(GameState::new(omdurman_rules::Scenario::Campaign)).0,
            &mut PlayerFactions::default(),
            &mut LoadedAnnotations::default(),
            &mut PendingMapLoad::default(),
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
                map: omdurman_types::MapKind::FallOfKhartoum,
                q: 1,
                r: 2,
                terrain: Terrain::Desert as u8,
                name: "Khartoum".into(),
                nile_flow: None,
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
        use std::collections::BTreeMap;
        let mut tiles = BTreeMap::new();
        tiles.insert(
            (3, 4),
            TileInfo {
                terrain: Terrain::BlueNile,
                name: Some("Nile".into()),
                nile_flow: None,
            },
        );
        let mut ann_file = omdurman_types::AnnotationsFile::empty();
        ann_file.fall_of_khartoum.tiles = tiles;
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
        let record = make_record(vec![GameEvent::OverlayUpdate(
            omdurman_types::MapKind::FallOfKhartoum,
            params.clone(),
        )]);
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
            kind: omdurman_types::UnitFormKind::Camel,
            brigade: omdurman_types::Brigade::None,
            fire: 0,
            melee: 0,
            movement: 0,
            movement_upstream: 0,
            movement_downstream: 0,
            is_boat: false,
            is_unit: true,
            fires_twice: false,
        };
        let record = make_record(vec![
            GameEvent::LoadAnnotations(empty_annotations_file()),
            GameEvent::AnnotateSprite {
                map: omdurman_types::MapKind::FallOfKhartoum,
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
                map: MapKind::FallOfKhartoum,
                q: 0,
                r: 0,
                terrain: Terrain::Desert as u8,
                name: "".into(),
                nile_flow: None,
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
            &mut GameStateResource(GameState::new(omdurman_rules::Scenario::Campaign)).0,
            &mut PlayerFactions::default(),
            &mut LoadedAnnotations::default(),
            &mut PendingMapLoad::default(),
        );

        assert!(
            !game_map.hexes.contains_key(&HexCoord::new(99, 99)),
            "stale hex must be cleared before replay"
        );
        assert!(game_map.hexes.contains_key(&HexCoord::new(0, 0)));
    }

    // ── scenario selects the board (§dual-map) ───────────────────────────────

    #[test]
    fn scenario_maps_to_board() {
        use omdurman_rules::Scenario;
        assert_eq!(map_kind_for_scenario(Scenario::Campaign), MapKind::Campaign);
        assert_eq!(
            map_kind_for_scenario(Scenario::Historical),
            MapKind::FallOfKhartoum
        );
        assert_eq!(
            map_kind_for_scenario(Scenario::FallOfKhartoum),
            MapKind::FallOfKhartoum
        );
    }

    /// A replayed `StartGame { scenario: Campaign }` must request the campaign
    /// board, and `LoadAnnotations` must keep both boards' data in
    /// `LoadedAnnotations` regardless of which board is live during replay.
    #[test]
    fn start_game_scenario_selects_board() {
        use omdurman_rules::{Player, Scenario};

        // Annotations carrying a distinctive tile on each board.
        let mut file = empty_annotations_file();
        file.campaign.tiles.insert(
            (7, 8),
            TileInfo {
                terrain: Terrain::Desert,
                name: Some("Omdurman".into()),
                nile_flow: None,
            },
        );

        let record = make_record(vec![
            GameEvent::LoadAnnotations(file),
            GameEvent::StartGame {
                assignments: vec![],
                scenario: Scenario::Campaign,
            },
        ]);

        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        let mut game_map = GameMap::default();
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
        let mut loaded = LoadedAnnotations::default();
        let mut pending_map = PendingMapLoad::default();
        let _ = Player::AngloEgyptian; // (faction binding unused here)

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
            PeerId(uuid::Uuid::nil()),
            &mut GameStateResource(GameState::new(Scenario::Campaign)).0,
            &mut PlayerFactions::default(),
            &mut loaded,
            &mut pending_map,
        );

        // StartGame requested the campaign board…
        assert_eq!(pending_map.0, Some(MapKind::Campaign));
        // …and both boards' data survived in the in-memory file.
        assert!(
            loaded.0.campaign.tiles.contains_key(&(7, 8)),
            "campaign tile preserved in LoadedAnnotations"
        );
        assert_eq!(loaded.0.fall_of_khartoum.image, "fall_of_khartoum_1885.png");
    }

    /// Make sure any pre-existing on-disk game record still parses against
    /// the current schema. Run only on native; on WASM there are no files.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_saved_games_still_load() {
        let games_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../games");
        let Ok(entries) = std::fs::read_dir(games_dir) else {
            return;
        };
        let mut found = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }
            let content = std::fs::read_to_string(&path).expect("read saved game");
            let mut lines = content.lines();
            // First line: {"seed": <n>}
            let header = lines
                .next()
                .unwrap_or_else(|| panic!("{}: empty file", path.display()));
            let seed: u64 = serde_json::from_str(header)
                .map(|v: serde_json::Value| {
                    v.get("seed")
                        .and_then(|s| s.as_u64())
                        .expect("missing seed")
                })
                .unwrap_or_else(|e| panic!("{}: bad header: {e}", path.display()));
            let mut events = Vec::new();
            for (i, line) in lines.enumerate() {
                let ev: RecordedEvent = serde_json::from_str(line)
                    .unwrap_or_else(|e| panic!("{}:{}: {e}", path.display(), i + 2));
                events.push(ev);
            }
            let rec = GameRecord {
                initial_state: InitialGameState { seed },
                events,
            };
            assert!(
                rec.events.iter().any(|e| matches!(
                    e.payload,
                    GameEvent::LoadAnnotations(_)
                        | GameEvent::Action(_)
                        | GameEvent::MapEdit { .. }
                        | GameEvent::PlaceUnit { .. }
                        | GameEvent::MoveUnit { .. }
                        | GameEvent::OverlayUpdate(..)
                        | GameEvent::UpdateUnitGrids(_)
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

    /// Run the game recording pipeline in isolation: create a JSONL file by
    /// starting the recorder, recording a PlaceUnit event the way
    /// `handle_socket` does on a host-sequenced receipt (`push_event` with a
    /// canonical seq), then flushing and reading back to verify it is present.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_jsonl_records_place_unit() {
        let tmp = tempfile::TempDir::new().unwrap();
        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Resources the pipeline needs
        app.insert_resource(game_record::GameRecorder::default());
        app.insert_resource(PendingEdits::default());
        app.insert_resource(PendingIncoming::default());
        app.insert_resource(NetState::default());
        app.insert_resource(TurnState::default());

        // Pipeline systems, run in order each frame
        app.add_systems(
            Update,
            (
                game_record::init_game_record,
                game_record::flush_game_record,
            )
                .chain(),
        );

        // Frame 1: init_game_record creates the recorder + seed file.
        app.update();

        // Record a PlaceUnit the way `handle_socket` does when it applies a
        // host-sequenced event: `push_event` with the canonical seq.
        app.world_mut()
            .resource_mut::<game_record::GameRecorder>()
            .push_event(
                &GameEvent::PlaceUnit {
                    section_name: "British_Infantry".into(),
                    col: 0,
                    row: 0,
                    coord_q: 0,
                    coord_r: 0,
                    is_boat: false,
                },
                0,
                0,
            );

        // Frame 2: flush_game_record appends the recorded event to the JSONL.
        app.update();

        // Restore CWD before reading / asserting (TempDir cleans up on drop).
        std::env::set_current_dir(&orig_cwd).unwrap();

        let games_dir = tmp.path().join("games");
        let mut jsonl_path = None;
        for entry in std::fs::read_dir(&games_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                jsonl_path = Some(path);
                break;
            }
        }
        let jsonl_path = jsonl_path.expect("no jsonl file found in games/");

        let content = std::fs::read_to_string(&jsonl_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert!(
            lines.len() >= 2,
            "expected >= 2 lines (seed + events), got {}",
            lines.len()
        );

        // Line 0: seed header.
        let seed_val: serde_json::Value =
            serde_json::from_str(lines[0]).expect("seed line must be valid JSON");
        assert!(
            seed_val.get("seed").and_then(|s| s.as_u64()).is_some(),
            "first line must contain seed"
        );

        // At least one line must contain a PlaceUnit payload.
        let has_place = lines[1..].iter().any(|l| l.contains("PlaceUnit"));
        assert!(has_place, "expected a PlaceUnit event in JSONL:\n{content}");
    }
}

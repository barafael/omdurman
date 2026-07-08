//! Remember Gordon! Battle of Omdurman.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod browser;
mod camera;
mod charts;
mod debug_capture;
mod dice;
mod editor;
mod event_viewer;
mod events;
mod fire;
mod fok_entry;
mod game_apply;
mod game_record;
mod lobby;
mod melee;
mod net_plugin;
mod net_socket;
mod overview;
mod picker;
mod render;
mod retreat;
mod sandbox;
mod scenario_setup;
mod settings;
mod splash;
mod theme;
mod timeline;
mod ui_plugin;
mod unit_profiles;
mod units;
mod util;

use crate::browser::SpriteAnnotationsResource;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::{EguiPlugin, egui};
use bevy_matchbox::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout};
use omdurman_net::{Ephemeral, GameEvent, GameRecord, NetMsg, NetState, RoomId, room_id};
use omdurman_rules::effects::GameState;
use omdurman_rules::{UnitId, UnitPlacement, UnitProfile, UnitState};
use omdurman_types::{HexCoord, SectionName};
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

/// Deterministic PRNG resource shared by every peer. Seeded from the
/// canonical game record so late joiners reproduce the same sequence.
#[derive(Resource)]
pub struct GameRng(ChaCha8Rng);

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
    pub fn random_u32(&mut self) -> u32 {
        self.0.random::<u32>()
    }
}

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
    /// Sequenced events applied this frame; drained by
    /// [`drain_applied_events`] into `GameEventApplied` messages.
    applied_events: ResMut<'w, AppliedEvents>,
    /// Set by the `StartGame` handler so the view switches to the game board
    /// (the board data loads via `pending_map_load`; the view follows `AppMode`).
    next_app_mode: ResMut<'w, NextState<AppMode>>,
}

/// Buffers sequenced game events that [`handle_socket`] has just applied, so a
/// scheduled system can drain them into [`events::GameEventApplied`] messages
/// for UI/game listeners without coupling to the socket handler directly.
#[derive(Resource, Default)]
pub struct AppliedEvents(pub Vec<(GameEvent, u32)>);

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

/// Bundle of the rules state + faction gate used by the picker's click handler,
/// so `handle_picker_clicks` stays under the param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct MoveGate<'w> {
    pub game_state: Option<Res<'w, GameStateResource>>,
    pub gate: FactionGate<'w>,
}

/// Tracks which unit entity is currently selected by the local player.
#[derive(Resource, Default)]
pub struct SelectedUnit(pub Option<Entity>);

use crate::camera::RtsCamera;

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
    /// `PlaceUnit` / `MoveUnit` events received live -- recorded by
    /// `apply_pending_placement` and applied to the world. Other game
    /// events are applied inline by `handle_socket`; these two are deferred
    /// because they need access to the picker + mesh/material asset pools.
    /// The `u8` is the pre-computed sender index.
    pub live: Vec<(GameEvent, PeerId, u8)>,
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

#[derive(Resource, Default)]
pub struct SidebarClip {
    pub right_sidebar: Option<egui::Rect>,
}

/// Debounce flag for `assets/annotations.ron` writes. Set by any editor /
/// browser / overlay system that mutates persisted state. The
/// `editor::flush_annotations_to_disk` writes the file after the dirty flag
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

fn main() {
    let room = room_id();

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // Start windowed but maximized (see `maximize_primary_window`).
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
    .add_plugins(camera::CameraPlugin)
    .add_plugins(omdurman_hexmap::HexMapPlugin)
    .add_plugins(editor::EditorPlugin)
    .add_plugins(render::RenderPlugin)
    .add_plugins(picker::GamePlugin)
    .add_plugins(ui_plugin::UiPlugin)
    .add_plugins(net_plugin::NetPlugin)
    .add_plugins(net_socket::NetSocketPlugin)
    .add_plugins(dice::DicePlugin)
    .add_plugins(splash::SplashPlugin)
    .add_plugins(sandbox::SandboxPlugin)
    .add_plugins(charts::ChartsPlugin)
    .add_plugins(debug_capture::DebugCapturePlugin)
    .init_state::<AppState>()
    .init_state::<AppMode>()
    .init_state::<EditorTab>()
    .add_message::<events::LocalAction>()
    .add_message::<events::GameEventApplied>()
    .configure_sets(
        Update,
        (
            EditorSet.run_if(in_state(AppMode::Editor).and(in_state(EditorTab::Terrain))),
            OverlaySet.run_if(in_state(AppMode::Editor).and(in_state(EditorTab::Overlay))),
            HexsideSet.run_if(in_state(AppMode::Editor).and(in_state(EditorTab::Hexside))),
            // Gameplay systems (picker, combat overlays, movement) run only on a
            // play view (Game or Sandbox) *and* while actually in a game -- never
            // in the lobby/connecting/editor.
            GameSet.run_if(
                in_state(AppState::InGame)
                    .and(in_state(AppMode::Game).or(in_state(AppMode::Sandbox))),
            ),
        ),
    )
    .insert_resource(RoomId(room))
    .insert_resource(TurnState::default())
    .insert_resource(GameStateResource(GameState::new(
        omdurman_rules::Scenario::Campaign,
    )))
    .insert_resource(game_record::GameRecorder::default())
    .insert_resource(SelectedUnit::default())
    .insert_resource(HoveredHex::default())
    .insert_resource(LoadedAnnotations::default())
    .insert_resource(ActiveEditMap::default())
    .insert_resource(EditorBoard::default())
    .insert_resource(PendingMapLoad::default())
    .insert_resource(GameTurn::default())
    .insert_resource(GamePhaseApp::default())
    .insert_resource(timeline::SpectatorTimeline::default())
    .insert_resource(LobbyTab::default())
    .insert_resource(HexLayout::calibrated(
        omdurman_types::Orientation::Pointy,
        Vec2::new(736.0, 420.0),
        omdurman_types::HexCoord::new(0, 0),
        Vec2::new(1178.0, 572.0),
        omdurman_types::HexCoord::new(5, -1),
        omdurman_hexmap::IMG_W,
        omdurman_hexmap::IMG_H,
    ))
    .add_systems(Startup, (spawn_ground, spawn_lights))
    .add_systems(
        Update,
        (
            events::forward_local_actions.before(net_plugin::flush_pending),
            sync_game_turn_phase.after(net_socket::handle_socket),
            // Timeline scrub: advance playback, then rebuild world state to
            // the cursor *before* apply_pending_placement drains the replay
            // queue it fills.
            timeline::advance_timeline_playback,
            timeline::apply_timeline_scrub
                .after(timeline::advance_timeline_playback)
                .before(apply_pending_placement),
        ),
    )
    .add_systems(
        bevy_egui::EguiPrimaryContextPass,
        (timeline::timeline_ui, timeline::exit_review_ui),
    )
    // The saved-games list is cached and refreshed on entering the lobby,
    // then rendered inside the lobby's "Saved games" sub-tab (native has
    // files on disk; the cache stays empty on wasm).
    .insert_resource(game_record::SavedGamesCache::default())
    .add_systems(
        OnEnter(AppState::Lobby),
        game_record::refresh_saved_games_on_lobby,
    );

    app.run();
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    Connecting,
    Lobby,
    #[default]
    InGame,
    /// Reviewing a recorded game (in-memory or loaded from disk) on the timeline
    /// scrubber, disconnected from any live socket (§spectator). The rules/map
    /// state is rebuilt from the record to the timeline cursor; live net systems
    /// are gated off in this state.
    Spectating,
}

/// Top-level app mode, chosen from the mode picker. Orthogonal to [`AppState`]
/// (which tracks the networking/game lifecycle: Connecting/Lobby/InGame/
/// Spectating). The picker shows three entries: **Lobby/Game** (whichever
/// `AppState` applies), **Sandbox**, and **Editor**.
///
/// - `Game`  — the live/networked game view (or the lobby, per `AppState`).
/// - `Sandbox` — a local, unbound single-seat session (drive both sides, free
///   placement); its board/scenario is chosen in the sandbox settings screen.
/// - `Editor` — the map/annotation editor; its sub-tools are [`EditorTab`]s and
///   its board is [`EditorBoard`].
///
/// Only `Editor` shows editor tooling; `Game` and `Sandbox` both show the play
/// board (unit picker, overview, gameplay overlays).
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppMode {
    #[default]
    Game,
    Sandbox,
    Editor,
}

impl AppMode {
    /// Whether this mode shows the playable board view (picker, overview,
    /// gameplay overlays, placed units): `Game` and `Sandbox`, not `Editor`.
    pub fn is_play(self) -> bool {
        matches!(self, AppMode::Game | AppMode::Sandbox)
    }
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Game => write!(f, "Game"),
            AppMode::Sandbox => write!(f, "Sandbox"),
            AppMode::Editor => write!(f, "Editor"),
        }
    }
}

/// The editor's sub-tool, selected via the editor's horizontal tab bar. Only
/// meaningful while [`AppMode::Editor`]. The board-specific tabs (Overlay,
/// Terrain, Hexside, Timing) act on the [`EditorBoard`]; the rest are
/// board-agnostic.
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EditorTab {
    /// Hex-grid alignment calibration over the map image.
    Overlay,
    /// Terrain painting / Nile flow / hex names / roads.
    #[default]
    Terrain,
    /// Hexside-feature (edge) editor.
    Hexside,
    /// Campaign turn-track bounding-box editor (Campaign board only).
    Timing,
    /// Sprite-sheet cutting-grid editor.
    UnitSheet,
    /// Sprite browser (cut counters).
    Sprites,
    /// Dice-roll physics tuning.
    Dice,
    /// Read-only recorded-event log viewer.
    EventViewer,
    /// Reference-chart sheet preview (the only editor tab that shows charts;
    /// charts are otherwise a play-view feature, hidden in the editor).
    Charts,
}

impl EditorTab {
    /// The board this tab edits, or `None` for board-agnostic tabs
    /// (Sprites/UnitSheet/Dice/EventViewer) that ignore the editor board pick.
    pub fn is_board_specific(self) -> bool {
        matches!(
            self,
            EditorTab::Overlay | EditorTab::Terrain | EditorTab::Hexside | EditorTab::Timing
        )
    }
    /// Whether this tab locks camera drag/zoom (sprite browser, event viewer).
    pub fn disables_camera(self) -> bool {
        matches!(self, EditorTab::Sprites | EditorTab::EventViewer)
    }
    /// Whether this tab shows no hex hover marker.
    pub fn hides_hex_hover(self) -> bool {
        matches!(
            self,
            EditorTab::Hexside | EditorTab::UnitSheet | EditorTab::EventViewer
        )
    }
    /// Whether the full-map plane is shown behind this tab.
    pub fn shows_map_plane(self) -> bool {
        !matches!(
            self,
            EditorTab::UnitSheet | EditorTab::Sprites | EditorTab::EventViewer | EditorTab::Charts
        )
    }
    pub fn label(self) -> &'static str {
        match self {
            EditorTab::Overlay => "Overlay",
            EditorTab::Terrain => "Terrain",
            EditorTab::Hexside => "Hexsides",
            EditorTab::Timing => "Timing",
            EditorTab::UnitSheet => "Unit sheet",
            EditorTab::Sprites => "Sprites",
            EditorTab::Dice => "Dice",
            EditorTab::EventViewer => "Events",
            EditorTab::Charts => "Charts",
        }
    }
    /// The tab bar, in display order.
    pub const ALL: [EditorTab; 9] = [
        EditorTab::Overlay,
        EditorTab::Terrain,
        EditorTab::Hexside,
        EditorTab::Timing,
        EditorTab::UnitSheet,
        EditorTab::Sprites,
        EditorTab::Dice,
        EditorTab::EventViewer,
        EditorTab::Charts,
    ];
}

/// Which board the editor tools currently act on, chosen by a scenario picker
/// (Fall of Khartoum / Historical / Campaign) in the editor's tab bar. Historical
/// and Campaign share the Campaign board (§9.1/§9.2), so the picker selects a
/// scenario and the board follows via [`map_kind_for_scenario`]. Local editor
/// state, not replicated.
#[derive(Resource)]
pub struct EditorBoard(pub omdurman_rules::Scenario);

impl Default for EditorBoard {
    fn default() -> Self {
        Self(omdurman_rules::Scenario::FallOfKhartoum)
    }
}

impl EditorBoard {
    pub fn map_kind(&self) -> omdurman_types::MapKind {
        map_kind_for_scenario(self.0)
    }
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
            // No local faction: either an unbound sandbox (empty binding -> may
            // drive both sides) or a spectator (non-empty binding, not in it ->
            // never acts). See `local_is_spectator`.
            None => self.by_peer.is_empty(),
        }
    }

    /// Whether the local peer is a spectator: a faction binding exists (the game
    /// started with assigned players) but this peer isn't in it, so it joined to
    /// watch only. A spectator may never place, move, or fight -- distinct from
    /// an unbound sandbox session (empty binding), which may drive both sides.
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
        local_faction: Option<omdurman_rules::Player>,
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

// -- System sets ----------------------------------------------------------

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EditorSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OverlaySet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexsideSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSet;

/// Parse a `PeerId` from its string form (the canonical UUID text), as carried
/// in [`GameEvent::StartGame`]. Returns `None` for malformed input.
fn parse_peer_id(s: &str) -> Option<PeerId> {
    uuid::Uuid::parse_str(s).ok().map(PeerId)
}

/// Which board a scenario plays on. Both the Campaign game (§9.1) and the
/// Historical scenario (§9.2) are the Battle of Omdurman fought on the main
/// Omdurman mapsheet -- they differ only in set-up, length, and victory, not
/// terrain -- so both use the campaign map (the lettered set-up hexes A/D/Y/K/S/O
/// of §9.212 live on it). Only the Fall-of-Khartoum bonus game (§9.3) uses the
/// separate tactical mini-map.
pub fn map_kind_for_scenario(scenario: omdurman_rules::Scenario) -> omdurman_types::MapKind {
    match scenario {
        omdurman_rules::Scenario::Campaign | omdurman_rules::Scenario::Historical => {
            omdurman_types::MapKind::Campaign
        }
        omdurman_rules::Scenario::FallOfKhartoum => omdurman_types::MapKind::FallOfKhartoum,
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

/// Keep the app-level [`GameTurn`] / [`GamePhaseApp`] resources in sync
/// with the rules engine's [`GameState`] after each frame's event processing.
pub(crate) fn sync_game_turn_phase(
    game_state: Option<Res<GameStateResource>>,
    mut game_turn: ResMut<GameTurn>,
    mut game_phase: ResMut<GamePhaseApp>,
) {
    let Some(state) = game_state else { return };
    let s = &state.0;
    **game_turn = s.current_turn.0;
    *game_phase = match s.phase {
        omdurman_rules::Phase::Setup => GamePhaseApp::Setup,
        omdurman_rules::Phase::Movement => GamePhaseApp::Movement,
        omdurman_rules::Phase::DefensiveFire(_) => GamePhaseApp::DefensiveFire,
        omdurman_rules::Phase::OffensiveFire(_) => GamePhaseApp::OffensiveFire,
        omdurman_rules::Phase::Melee => GamePhaseApp::Melee,
    };
}

/// Which board the editor/overlay tools currently act on (§dual-map). Local to
/// each peer -- calibration is a dev tool, not replicated state. Switching it
/// reloads the corresponding board into the live `GameMap`/overlay/layout.
#[derive(Resource, Default)]
pub struct ActiveEditMap(pub omdurman_types::MapKind);

/// A deferred request to (re)load a board into the live `GameMap`/`HexOverlay`/
/// `MapDims`/`HexLayout` and re-texture the map plane. Set by the `StartGame`
/// handler and the editor's map toggle; consumed by `apply_map_selection`,
/// which has the asset/material access those handlers lack (§dual-map).
#[derive(Resource, Default)]
pub struct PendingMapLoad(pub Option<omdurman_types::MapKind>);

/// Current game turn (1-based) for the campaign or scenario.
#[derive(Resource, Deref, DerefMut)]
pub struct GameTurn(pub u8);

impl Default for GameTurn {
    fn default() -> Self {
        Self(1)
    }
}

/// High-level phase within a single game turn, for the app UI.
/// The rules engine has its own more granular `Phase`; this is a lightweight
/// app-side mirror for the top-bar and turn advancement buttons.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GamePhaseApp {
    #[default]
    Setup,
    Movement,
    DefensiveFire,
    OffensiveFire,
    Melee,
}

impl std::fmt::Display for GamePhaseApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Setup => write!(f, "Setup"),
            Self::Movement => write!(f, "Movement"),
            Self::DefensiveFire => write!(f, "Defensive Fire"),
            Self::OffensiveFire => write!(f, "Offensive Fire"),
            Self::Melee => write!(f, "Melee"),
        }
    }
}

/// Which sub-tab the lobby screen is showing (§lobby). "Setup" is the faction /
/// scenario / start panel; "Saved games" is the review-a-game list (a saved-
/// games browser embedded in the lobby rather than a floating overlay).
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum LobbyTab {
    #[default]
    Setup,
    SavedGames,
}

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
    /// Peers who have toggled "Spectate" in the lobby (live preview). A
    /// spectator is never assigned a faction, so it shows as "spectating" in the
    /// roster and is ignored by the start-readiness check.
    pub spectators: std::collections::HashSet<PeerId>,
    /// Latest scenario broadcast by the host's lobby (live preview, §lobby).
    /// `None` until the host sends one; the committed value rides in
    /// [`GameEvent::StartGame`].
    pub scenario: Option<omdurman_rules::Scenario>,
}

/// The local player's current lobby faction pick (pre-commit).
#[derive(Resource, Default)]
pub struct LocalFaction(pub Option<omdurman_rules::Player>);

/// Whether the local player has chosen to spectate (join to watch, no faction).
/// Kept separate from [`LocalFaction`] so "spectating" is distinct from
/// "undecided". A spectator is never included in the `StartGame` assignments.
#[derive(Resource, Default)]
pub struct LocalSpectator(pub bool);

/// Tracks whether the game has begun (set by the host's `StartGame`). Used by
/// the snapshot / host-failover paths in `net_socket`. The turn itself lives in
/// the rules engine (`GameState.active_player` / `phase`), advanced by the
/// `End Phase` button -- there is no separate app-level turn counter.
#[derive(Resource, Default)]
pub(crate) struct TurnState {
    pub game_started: bool,
}

/// Written every frame by `render::update_selection_marker` with the hex
/// currently under the cursor (or `None` if no valid hex is hovered).
#[derive(Resource, Default)]
pub struct HoveredHex(pub Option<HexCoord>);

/// Camera drag/zoom is disabled only for the editor tabs that lock it (sprite
/// browser, event viewer). Every play view and the other editor tabs allow it.
pub(crate) fn camera_enabled(mode: Res<State<AppMode>>, tab: Res<State<EditorTab>>) -> bool {
    match **mode {
        AppMode::Editor => !tab.disables_camera(),
        _ => true,
    }
}

/// The hex hover marker is shown on the play board and on editor tabs that don't
/// suppress it (hexside/unit-sheet/event-viewer hide it).
pub(crate) fn hex_hover_visible(mode: Res<State<AppMode>>, tab: Res<State<EditorTab>>) -> bool {
    match **mode {
        AppMode::Editor => !tab.hides_hex_hover(),
        _ => true,
    }
}

/// Whether a hex-grid-bearing view is active (cursor broadcast / cursor overlay
/// gate): any play view, or an editor tab that shows the map plane.
pub(crate) fn map_view_active(mode: Res<State<AppMode>>, tab: Res<State<EditorTab>>) -> bool {
    match **mode {
        AppMode::Game | AppMode::Sandbox => true,
        AppMode::Editor => tab.shows_map_plane(),
    }
}

/// Look up a counter's authored [`SpriteAnnotation`] and build its rules
/// profile. Returns `None` if annotations aren't loaded yet, the counter has
/// no annotation, or its section name is unrecognised -- in every case the
/// unit is placed visually but acquires no rules-engine `UnitId`.
fn profile_for(
    annotations: Option<&SpriteAnnotationsResource>,
    section_name: SectionName,
    col: u32,
    row: u32,
) -> Option<UnitProfile> {
    let annotation = annotations?
        .0
        .units
        .get(&section_name)
        .and_then(|m| m.get(&(col, row)))?;
    unit_profiles::profile_from_annotation(section_name, col, row, annotation)
}

/// Route a unit move through the rules engine so it validates the move
/// (allowance, phase, ZOC, night-halving) and updates `unit.position`
/// authoritatively. Returns whether the engine *accepted* the move: the caller
/// must apply the visual update only on `true`, so a rejected move never moves
/// the sprite (the engine is authoritative over position).
#[must_use]
fn apply_move_effect(
    state: &mut GameState,
    unit_id: UnitId,
    to: HexCoord,
    path: &[HexCoord],
) -> bool {
    let Some(unit) = state.find_unit(unit_id) else {
        warn!(?unit_id, "MoveUnit for unknown rules unit");
        return false;
    };
    // When `path` is supplied (the hexes entered, ending at `to`) the engine
    // recomputes the true terrain/Nile cost and classifies gunboat up/downstream
    // steps from it; the straight-line `cost` is only the fallback for an empty
    // path (legacy records / sandbox).
    let cost = omdurman_rules::MovementPoints(unit.position.distance(to) as i16);
    let effect = omdurman_rules::effects::GameEffect::MoveUnit {
        unit_id,
        to,
        cost,
        path: path.to_vec(),
    };
    if let Err(error) = omdurman_rules::effects::apply_effect(state, &effect) {
        warn!(%error, ?unit_id, to.q = to.q, to.r = to.r, "move rejected by rules engine");
        return false;
    }
    true
}

/// Extend a unit's turn path with an accepted move. `path` is the sequence of
/// hexes *entered* this move (ending at `to`); when it is empty (legacy record /
/// sandbox) we fall back to a single hop straight to `to`. Each entered hex is
/// appended as its own step so multi-hex moves render as consecutive arrows.
fn record_move_path(
    paths: &mut picker::UnitPaths,
    unit_id: UnitId,
    from: HexCoord,
    path: &[HexCoord],
    to: HexCoord,
) {
    let mut prev = from;
    let steps: &[HexCoord] = if path.is_empty() { &[to] } else { path };
    for &step in steps {
        if step != prev {
            paths.record_step(unit_id, prev, step);
            prev = step;
        }
    }
}

pub(crate) fn apply_pending_placement(
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
    annotations: Option<Res<SpriteAnnotationsResource>>,
    // Per-unit movement paths this turn, extended on each accepted step so the
    // route each unit took is drawn as arrows until the turn ends.
    mut unit_paths: ResMut<picker::UnitPaths>,
    // Tracks entities spawned this invocation so MoveUnit can find units
    // placed in the same batch (e.g. during history replay) before Bevy
    // has flushed the deferred commands.
    // key: (section_name, col, row), value: (entity, is_boat, unit_id)
    mut just_placed: Local<HashMap<(SectionName, u32, u32), (Entity, bool, Option<UnitId>)>>,
) {
    just_placed.clear();

    // Replay events and live events are both already recorded -- replay by the
    // canonical host log, live by `handle_socket` when the host-sequenced
    // event was applied. Do NOT re-record here.
    let replay_items: Vec<_> = incoming.replay.drain(..).map(|(msg, _peer)| msg).collect();
    let live_items: Vec<_> = incoming.live.drain(..).map(|(msg, _, _)| msg).collect();

    for event in replay_items.into_iter().chain(live_items) {
        match event {
            GameEvent::PlaceUnit {
                sprite,
                coord_q,
                coord_r,
                is_boat,
            } => {
                let section_name = sprite.section_name;
                let col = sprite.col;
                let row = sprite.row;
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
                if let Some((_entity, mut placed)) = placed_units.iter_mut().find(|(_, u)| {
                    u.unit_id.is_none()
                        && u.section_name == section_name
                        && u.col == col
                        && u.row == row
                        && u.coord == coord
                }) {
                    let profile: Option<UnitProfile> =
                        profile_for(annotations.as_deref(), section_name, col, row);
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
                        profile_for(annotations.as_deref(), section_name, col, row);
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

                    let origin = omdurman_hexmap::adjusted_origin(
                        &layout,
                        overlay.params.offset_x,
                        overlay.params.offset_y,
                    );
                    let pos = omdurman_hexmap::hex_world_pos(coord, origin, &overlay.params);
                    let entity = picker::spawn_placed_unit(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        unit.handle.clone(),
                        &overlay,
                        pos,
                        picker::PlacedUnit {
                            coord,
                            section_name,
                            col,
                            row,
                            is_boat,
                            unit_id: allocated,
                            disrupted: false,
                        },
                    );
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
                sprite,
                to_q,
                to_r,
                path,
                ..
            } => {
                let section_name = sprite.section_name;
                let col = sprite.col;
                let row = sprite.row;
                info!(
                    ?section_name,
                    col, row, to_q, to_r, "apply_pending_placement: processing MoveUnit",
                );
                let target = omdurman_types::HexCoord::new(to_q, to_r);
                if !game_map.hexes.contains_key(&target) {
                    warn!(to_q, to_r, "ignoring inbound MoveUnit to off-map coord");
                    continue;
                }
                let origin = omdurman_hexmap::adjusted_origin(
                    &layout,
                    overlay.params.offset_x,
                    overlay.params.offset_y,
                );
                let pos = omdurman_hexmap::hex_world_pos(target, origin, &overlay.params);
                let new_transform = Transform::from_xyz(pos.x, 1.0, pos.z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0));

                // Try the live world query first (normal gameplay path).
                let mut found = false;
                for (entity, mut placed) in placed_units.iter_mut() {
                    if placed.section_name == section_name && placed.col == col && placed.row == row
                    {
                        info!(
                            ?section_name,
                            col, row, "apply_pending_placement: found entity for MoveUnit",
                        );
                        // The rules engine is authoritative: validate first and
                        // move the sprite only if the engine accepts. A rejected
                        // move leaves the counter where it was. (The picker's
                        // terrain-aware `cost` rides on the event; the engine
                        // recomputes from it.) Units without a rules id, or a
                        // sandbox with no game state, fall through as accepted.
                        let accepted = match (placed.unit_id, game_state.as_mut()) {
                            (Some(unit_id), Some(gs)) => {
                                apply_move_effect(&mut gs.0, unit_id, target, &path)
                            }
                            _ => true,
                        };
                        if accepted {
                            // Record the route before moving the counter, using
                            // the pre-move hex as this step's origin. Covers the
                            // interactive single-hop (`path == [target]`) and any
                            // multi-hop path carried on the event.
                            if let Some(uid) = placed.unit_id {
                                record_move_path(&mut unit_paths, uid, placed.coord, &path, target);
                            }
                            placed.coord = target;
                            // Don't snap if a local movement animation is already
                            // playing -- let animate_unit_movement finish it.
                            if anim_query.get(entity).is_err() {
                                commands.entity(entity).insert(new_transform);
                                commands
                                    .entity(entity)
                                    .remove::<picker::MovementAnimation>();
                            }
                        }
                        found = true;
                        break;
                    }
                }

                // Fall back to units placed earlier in this same batch
                // (replay path -- Bevy commands are still deferred).
                if !found
                    && let Some(&(entity, is_boat, unit_id)) =
                        just_placed.get(&(section_name, col, row))
                {
                    info!(
                        ?section_name,
                        col, row, "apply_pending_placement: MoveUnit fell back to just_placed",
                    );
                    // Route through the rules engine (see apply_move_effect).
                    // This batch-fallback is the replay path; the event is
                    // canonical history, so apply it visually regardless.
                    if let Some(uid) = unit_id
                        && let Some(ref mut gs) = game_state
                    {
                        // Capture the pre-move hex for the path before the effect
                        // updates it.
                        let from = gs.0.find_unit(uid).map(|u| u.position);
                        let _ = apply_move_effect(&mut gs.0, uid, target, &path);
                        if let Some(from) = from {
                            record_move_path(&mut unit_paths, uid, from, &path, target);
                        }
                    }
                    commands.entity(entity).insert(picker::PlacedUnit {
                        coord: target,
                        section_name,
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
                } else {
                    warn!(
                        ?section_name,
                        col, row, "apply_pending_placement: MoveUnit target entity not found",
                    );
                }
            }
            // Other GameEvent variants are applied inline by handle_socket /
            // rebuild_state_to -- they shouldn't appear in the deferred
            // queues. Warn if one does so the misclassification is visible.
            other => warn!(?other, "non-placement GameEvent in placement queue"),
        }
    }

    // -- Ephemeral messages handled by apply_ephemeral() -- see below --
}

/// Rebuild game + map state from the canonical event log, applying events
/// `0..=upto` (or all events when `upto` is `None`). The reset-from-seed + full
/// forward replay is the same mechanism the live late-joiner path uses; the
/// bounded form drives the spectator timeline scrubber (§spectator), which shows
/// the state as it was after event `upto`.
///
/// This rebuilds only the rules/map state and queues placement events into
/// `replay`; the caller is responsible for despawning any stale `PlacedUnit`
/// entities and clearing `UnitPaths`/`PickerState` before a *re-scrub* of an
/// already-populated world (the live path starts from an empty world, so it
/// needs no such reset).
fn rebuild_state_to(
    record: &GameRecord,
    upto: Option<usize>,
    commands: &mut Commands,
    game_map: &mut GameMap,
    overlay: &mut render::HexOverlay,
    editor: &mut editor::HexEditor,
    annotations: Option<&mut browser::SpriteAnnotationsResource>,
    viewer: &mut units::UnitViewer,
    replay: &mut Vec<(GameEvent, PeerId)>,
    history_peer: PeerId,
    game_state: &mut GameState,
    player_factions: &mut PlayerFactions,
    loaded_annotations: &mut LoadedAnnotations,
    pending_map_load: &mut PendingMapLoad,
) {
    let upto = upto.unwrap_or(record.events.len().saturating_sub(1));
    info!(
        upto,
        total = record.events.len(),
        "rebuilding state from log"
    );

    // Reset RNG + clear map -- the event stream is canonical so we rebuild
    // from a known state.
    commands.insert_resource(GameRng::from_seed(record.initial_state.seed));
    game_map.hexes.clear();

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
    let end = (upto + 1).min(record.events.len());
    for event in &record.events[..end] {
        match &event.payload {
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
                let map_kind = map_kind_for_scenario(*scenario);
                if let Some(gs) = ctx.game_state.as_deref_mut() {
                    // `GameState::new` sets the scenario's first-moving player
                    // (§9.113/§9.212/§9.322); do not override it.
                    *gs = GameState::new(*scenario);
                    // Attach the scenario's board to the engine state *now*, so
                    // the replayed MoveUnit/PlaceUnit events (queued into
                    // `incoming.replay` and applied later by
                    // `apply_pending_placement`) are costed by terrain and
                    // checked for ZOC/Nile against the same board the live game
                    // used. Deferring only the *visual* map load left those moves
                    // briefly validated against an empty board -- diverging from
                    // live, especially now that movement cost accumulates
                    // (mp_spent_this_turn).
                    if let Some(loaded) = ctx.loaded_annotations.as_deref() {
                        gs.board =
                            omdurman_rules::board::BoardInfo::from_map_data(loaded.0.map(map_kind));
                    }
                }
                // The *visual* board (map plane, overlay, camera) still loads
                // after replay completes, on the next frame (§dual-map).
                pending_map_load.0 = Some(map_kind);
                continue;
            }
            _ => {}
        }
        game_apply::apply_game_event(&event.payload, &mut ctx);
    }
}

fn spawn_ground(mut commands: Commands) {
    commands.spawn((RigidBody::Static, Collider::half_space(Vec3::Y)));
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

// -- Late-joiner sync tests ----------------------------------------------------

#[cfg(test)]
mod late_joiner_tests {
    use super::*;
    use bevy::ecs::world::CommandQueue;
    use chrono::Utc;
    use omdurman_net::{GameEvent, GameRecord, InitialGameState, RecordedEvent, new_seed};
    use omdurman_types::{
        HexCoord, MapKind, OverlayParams, SectionName, SpriteAnnotation, SpriteAnnotations,
        SpriteRef, Terrain, TileInfo,
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
        // EvenR with width=64, height=32 starts at q>=0 on row 0 and covers
        // a wide enough range that every test coordinate (q in [0,9], r in [0,9])
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

    /// Run rebuild_state_to (full replay) with sensible defaults and return the modified state.
    fn run_replay(
        record: &GameRecord,
        total_peers: usize,
    ) -> (
        GameMap,
        render::HexOverlay,
        AppMode,
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
            dirty_grids: std::collections::HashSet::new(),
        };
        let mut turn = TurnState::default();
        let mut incoming: Vec<(GameEvent, PeerId)> = vec![];
        let history_peer = PeerId(uuid::Uuid::nil());

        rebuild_state_to(
            record,
            None,
            &mut commands,
            &mut game_map,
            &mut overlay,
            &mut editor,
            annotations.as_mut(),
            &mut viewer,
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
            AppMode::default(),
            editor,
            browser_state,
            annotations,
            viewer,
            turn,
            incoming,
        )
    }

    // -- bounded rebuild (timeline scrub) -------------------------------------

    /// Rebuild to a bounded event index and return the resulting map (mirrors
    /// `run_replay` but exercises the `upto` scrub path used by the spectator
    /// timeline).
    fn run_replay_upto(record: &GameRecord, upto: usize) -> GameMap {
        let mut world = World::new();
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
            dirty_grids: std::collections::HashSet::new(),
        };
        let mut incoming: Vec<(GameEvent, PeerId)> = vec![];
        rebuild_state_to(
            record,
            Some(upto),
            &mut commands,
            &mut game_map,
            &mut overlay,
            &mut editor,
            annotations.as_mut(),
            &mut viewer,
            &mut incoming,
            PeerId(uuid::Uuid::nil()),
            &mut GameStateResource(GameState::new(omdurman_rules::Scenario::Campaign)).0,
            &mut PlayerFactions::default(),
            &mut LoadedAnnotations::default(),
            &mut PendingMapLoad::default(),
        );
        queue.apply(&mut world);
        game_map
    }

    #[test]
    fn scrub_applies_only_events_up_to_index() {
        // Seed the board, then two edits at distinct hexes on separate events.
        let mk_edit = |q, r, name: &str| GameEvent::MapEdit {
            map: omdurman_types::MapKind::FallOfKhartoum,
            q,
            r,
            terrain: Terrain::Rough as u8,
            name: name.into(),
            nile_flow: None,
            is_crossroad: false,
        };
        let record = make_record(vec![
            GameEvent::LoadAnnotations(Box::new(empty_annotations_file())), // idx 0
            mk_edit(1, 2, "first"),                                         // idx 1
            mk_edit(3, 4, "second"),                                        // idx 2
        ]);

        // Scrub to idx 1: only the first edit is applied.
        let at_1 = run_replay_upto(&record, 1);
        assert_eq!(
            at_1.hexes.get(&HexCoord::new(1, 2)).map(|h| h.terrain),
            Some(Terrain::Rough),
            "first edit should be present at idx 1"
        );
        assert!(
            at_1.hexes
                .get(&HexCoord::new(3, 4))
                .is_none_or(|h| h.terrain != Terrain::Rough),
            "second edit must NOT be present at idx 1"
        );

        // Scrub to idx 2: both edits are applied.
        let at_2 = run_replay_upto(&record, 2);
        assert_eq!(
            at_2.hexes.get(&HexCoord::new(3, 4)).map(|h| h.terrain),
            Some(Terrain::Rough),
            "second edit should be present at idx 2"
        );
    }

    // -- map edit --------------------------------------------------------------

    #[test]
    fn map_edit_replayed() {
        // MapEdit only applies to on-map coords; seed the map first.
        let record = make_record(vec![
            GameEvent::LoadAnnotations(Box::new(empty_annotations_file())),
            GameEvent::MapEdit {
                map: omdurman_types::MapKind::FallOfKhartoum,
                q: 1,
                r: 2,
                terrain: Terrain::Rough as u8,
                name: "Khartoum".into(),
                nile_flow: None,
                is_crossroad: false,
            },
        ]);
        let (game_map, ..) = run_replay(&record, 2);
        let hex = game_map
            .hexes
            .get(&HexCoord::new(1, 2))
            .expect("hex not found");
        assert_eq!(hex.terrain, Terrain::Rough);
        assert_eq!(hex.name.as_deref(), Some("Khartoum"));
    }

    // -- load annotations rebuilds the map ------------------------------------

    #[test]
    fn load_annotations_replayed() {
        use std::collections::BTreeMap;
        let mut tiles = BTreeMap::new();
        tiles.insert(
            (3, 4),
            TileInfo {
                terrain: Terrain::Nile,
                name: Some("Nile".into()),
                nile_flow: None,
                is_crossroad: false,
            },
        );
        let mut ann_file = omdurman_types::AnnotationsFile::empty();
        ann_file.fall_of_khartoum.tiles = tiles;
        let record = make_record(vec![GameEvent::LoadAnnotations(Box::new(ann_file))]);
        let (game_map, ..) = run_replay(&record, 2);
        let hex = game_map
            .hexes
            .get(&HexCoord::new(3, 4))
            .expect("hex not found");
        assert_eq!(hex.terrain, Terrain::Nile);
        assert_eq!(hex.name.as_deref(), Some("Nile"));
    }

    // -- overlay update synced ------------------------------------------------

    #[test]
    fn overlay_update_replayed() {
        let mut params = OverlayParams::default();
        params.hex_size = 99.0;
        let record = make_record(vec![GameEvent::OverlayUpdate(
            omdurman_types::MapKind::FallOfKhartoum,
            params.clone(),
        )]);
        let (_, overlay, ..) = run_replay(&record, 2);
        assert_eq!(overlay.params.hex_size, 99.0);
    }

    // -- annotate sprite ------------------------------------------------------

    #[test]
    fn annotate_sprite_replayed() {
        use omdurman_types::{DervishTribe, Faction, SpriteColor};
        let ann = SpriteAnnotation {
            text: "Camel Corps".into(),
            faction: Some(Faction::Dervish {
                tribe: DervishTribe::Baggara,
            }),
            color: SpriteColor::GreenRed,
            kind: omdurman_types::UnitFormKind::Camel,
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
            GameEvent::LoadAnnotations(Box::new(empty_annotations_file())),
            GameEvent::AnnotateSprite {
                sprite: SpriteRef {
                    section_name: SectionName::Baggara,
                    col: 0,
                    row: 1,
                },
                annotation: ann.clone(),
            },
        ]);
        let (_, _, _, _, _, annotations, ..) = run_replay(&record, 2);
        let ann_res = annotations.unwrap();
        let entry = ann_res.0.units[&SectionName::Baggara][&(0, 1)].clone();
        assert_eq!(entry.text, "Camel Corps");
    }

    // -- unit placement queued for apply_pending_placement --------------------

    #[test]
    fn place_unit_queued_in_incoming() {
        let record = make_record(vec![GameEvent::PlaceUnit {
            sprite: SpriteRef {
                section_name: SectionName::Baggara,
                col: 2,
                row: 3,
            },
            coord_q: 5,
            coord_r: 6,
            is_boat: false,
        }]);
        let (.., incoming) = run_replay(&record, 2);
        assert_eq!(incoming.len(), 1);
        match &incoming[0].0 {
            GameEvent::PlaceUnit {
                sprite,
                coord_q,
                coord_r,
                is_boat,
            } => {
                assert_eq!(sprite.section_name, SectionName::Baggara);
                assert_eq!(sprite.col, 2);
                assert_eq!(sprite.row, 3);
                assert_eq!(*coord_q, 5);
                assert_eq!(*coord_r, 6);
                assert!(!is_boat);
            }
            other => panic!("expected PlaceUnit, got {other:?}"),
        }
    }

    // -- move unit queued -----------------------------------------------------

    #[test]
    fn move_unit_queued_in_incoming() {
        let record = make_record(vec![
            GameEvent::PlaceUnit {
                sprite: SpriteRef {
                    section_name: SectionName::HadendowaForts,
                    col: 0,
                    row: 0,
                },
                coord_q: 1,
                coord_r: 1,
                is_boat: false,
            },
            GameEvent::MoveUnit {
                sprite: SpriteRef {
                    section_name: SectionName::HadendowaForts,
                    col: 0,
                    row: 0,
                },
                to_q: 7,
                to_r: 8,
                cost: 0,
                path: vec![],
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

    // -- show terrain overlay -------------------------------------------------

    #[test]
    fn show_terrain_overlay_replayed() {
        let record = make_record(vec![GameEvent::ShowTerrainOverlay(true)]);
        let (_, _, _, editor, ..) = run_replay(&record, 2);
        assert!(editor.show_terrain_overlay);
    }

    // -- unit grids synced ----------------------------------------------------

    #[test]
    fn unit_grids_replayed() {
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

    // -- move after place in same batch ---------------------------------------

    #[test]
    fn move_after_place_queued_in_order() {
        // PlaceUnit at (1,1) then MoveUnit to (7,8) -- both in the same replay
        // batch.  The incoming queue must contain both events in order so that
        // apply_pending_placement can use the just_placed fallback map to apply
        // the move even though Bevy hasn't flushed the spawn command yet.
        let record = make_record(vec![
            GameEvent::PlaceUnit {
                sprite: SpriteRef {
                    section_name: SectionName::Baggara,
                    col: 0,
                    row: 0,
                },
                coord_q: 1,
                coord_r: 1,
                is_boat: false,
            },
            GameEvent::MoveUnit {
                sprite: SpriteRef {
                    section_name: SectionName::Baggara,
                    col: 0,
                    row: 0,
                },
                to_q: 7,
                to_r: 8,
                cost: 0,
                path: vec![],
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

    // -- map is cleared before replay ----------------------------------------

    #[test]
    fn map_cleared_before_replay() {
        // Pre-populate the map with a hex that is NOT in the record.
        // After replay it must be gone, and the new hex must be present.
        let record = make_record(vec![
            GameEvent::LoadAnnotations(Box::new(empty_annotations_file())),
            GameEvent::MapEdit {
                map: MapKind::FallOfKhartoum,
                q: 0,
                r: 0,
                terrain: Terrain::Rough as u8,
                name: "".into(),
                nile_flow: None,
                is_crossroad: false,
            },
        ]);

        // Run with a pre-populated world by inserting a stale hex
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let mut game_map = GameMap::default();
        game_map.hexes.insert(
            HexCoord::new(99, 99),
            omdurman_types::HexData::new(Terrain::Swamp, None),
        );

        let mut overlay = render::HexOverlay::default();
        let mut editor = editor::HexEditor::default();
        let mut annotations = Some(browser::SpriteAnnotationsResource(
            SpriteAnnotations::default(),
        ));
        let mut viewer = units::UnitViewer {
            grids: vec![],
            grids_dirty: false,
            dirty_grids: std::collections::HashSet::new(),
        };
        let mut turn = TurnState::default();
        let mut incoming: Vec<(GameEvent, PeerId)> = vec![];
        let history_peer = PeerId(uuid::Uuid::nil());

        rebuild_state_to(
            &record,
            None,
            &mut commands,
            &mut game_map,
            &mut overlay,
            &mut editor,
            annotations.as_mut(),
            &mut viewer,
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

    // -- scenario selects the board (§dual-map) -------------------------------

    #[test]
    fn scenario_maps_to_board() {
        use omdurman_rules::Scenario;
        assert_eq!(map_kind_for_scenario(Scenario::Campaign), MapKind::Campaign);
        // The Historical scenario is the Battle of Omdurman on the main map.
        assert_eq!(
            map_kind_for_scenario(Scenario::Historical),
            MapKind::Campaign
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
                terrain: Terrain::Rough,
                name: Some("Omdurman".into()),
                nile_flow: None,
                is_crossroad: false,
            },
        );

        let record = make_record(vec![
            GameEvent::LoadAnnotations(Box::new(file)),
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
            dirty_grids: std::collections::HashSet::new(),
        };
        let mut turn = TurnState::default();
        let mut incoming: Vec<(GameEvent, PeerId)> = vec![];
        let mut loaded = LoadedAnnotations::default();
        let mut pending_map = PendingMapLoad::default();
        let _ = Player::AngloEgyptian; // (faction binding unused here)

        rebuild_state_to(
            &record,
            None,
            &mut commands,
            &mut game_map,
            &mut overlay,
            &mut editor,
            annotations.as_mut(),
            &mut viewer,
            &mut incoming,
            PeerId(uuid::Uuid::nil()),
            &mut GameStateResource(GameState::new(Scenario::Campaign)).0,
            &mut PlayerFactions::default(),
            &mut loaded,
            &mut pending_map,
        );

        // StartGame requested the campaign board...
        assert_eq!(pending_map.0, Some(MapKind::Campaign));
        // ...and both boards' data survived in the in-memory file.
        assert!(
            loaded.0.campaign.tiles.contains_key(&(7, 8)),
            "campaign tile preserved in LoadedAnnotations"
        );
        assert_eq!(
            loaded.0.fall_of_khartoum.image,
            "fall_of_khartoum_1885.webp"
        );
    }

    /// Make sure any pre-existing on-disk game record still parses against
    /// the current schema. Run only on native; on WASM there are no files.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn saved_games_still_load() {
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
                        | GameEvent::MapEdit { .. }
                        | GameEvent::PlaceUnit { .. }
                        | GameEvent::MoveUnit { .. }
                        | GameEvent::OverlayUpdate(..)
                        | GameEvent::UpdateUnitGrids(_)
                        | GameEvent::Effect(_)
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
    fn jsonl_records_place_unit() {
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
                    sprite: SpriteRef {
                        section_name: SectionName::BritishArmy,
                        col: 0,
                        row: 0,
                    },
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

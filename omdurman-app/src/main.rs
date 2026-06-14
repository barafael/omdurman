//! Remember Gordon! Battle of Omdurman.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod browser;
mod camera;
mod dice;
mod editor;
mod event_viewer;
mod events;
mod fire;
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
#[cfg(test)]
mod scenario_setup;
mod settings;
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
use omdurman_net::{
    Ephemeral, GameEvent, GameRecord, NetMsg,
    NetState, RoomId, room_id,
};
use omdurman_rules::effects::GameState;
use omdurman_rules::{UnitId, UnitPlacement, UnitProfile, UnitState};
use omdurman_types::{HexCoord, SectionName};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
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

/// Bundle of the rules state + faction gate used by movement gating in the
/// picker, so `handle_picker_clicks` stays under the param limit.
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

    App::new()
        .add_plugins(
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
        .init_state::<AppState>()
        .init_state::<EditorMode>()
        .add_message::<events::LocalAction>()
        .add_message::<events::GameEventApplied>()
        .configure_sets(
            Update,
            (
                EditorSet
                    .run_if(in_state(EditorMode::Editor).or(in_state(EditorMode::CampaignEditor))),
                OverlaySet.run_if(
                    in_state(EditorMode::Overlay).or(in_state(EditorMode::CampaignOverlay)),
                ),
                HexsideSet.run_if(
                    in_state(EditorMode::Hexside).or(in_state(EditorMode::CampaignHexside)),
                ),
                GameSet.run_if(in_state(EditorMode::FallOfKhartoumMap).or(in_state(EditorMode::CampaignMap))),
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
        .insert_resource(PendingMapLoad::default())
        .insert_resource(MapStateStore::default())
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
            ),
        )
        .run();
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    Connecting,
    Lobby,
    #[default]
    InGame,
}

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EditorMode {
    Overlay,
    Editor,
    UnitSheet,
    Units,
    Dice,
    EventViewer,
    CampaignOverlay,
    CampaignEditor,
    Hexside,
    CampaignHexside,
    Lobby,
    #[default]
    FallOfKhartoumMap,
    CampaignMap,
}

impl EditorMode {
    pub fn is_overlay(self) -> bool {
        matches!(self, EditorMode::Overlay | EditorMode::CampaignOverlay)
    }
    pub fn is_editor(self) -> bool {
        matches!(self, EditorMode::Editor | EditorMode::CampaignEditor)
    }
    pub fn is_hexside(self) -> bool {
        matches!(self, EditorMode::Hexside | EditorMode::CampaignHexside)
    }

    pub fn is_unit_sheet(self) -> bool {
        self == EditorMode::UnitSheet
    }
    pub fn is_units(self) -> bool {
        self == EditorMode::Units
    }
    pub fn is_dice(self) -> bool {
        self == EditorMode::Dice
    }
    pub fn is_event_viewer(self) -> bool {
        self == EditorMode::EventViewer
    }
    pub fn is_lobby(self) -> bool {
        self == EditorMode::Lobby
    }
    pub fn is_fall_of_khartoum_map(self) -> bool {
        self == EditorMode::FallOfKhartoumMap
    }
    pub fn is_campaign_map(self) -> bool {
        self == EditorMode::CampaignMap
    }
    pub fn is_map_mode(self) -> bool {
        matches!(self, EditorMode::FallOfKhartoumMap | EditorMode::CampaignMap)
    }
    /// Full-screen UI panels that overlay or replace the map view.
    pub fn shows_map_plane(self) -> bool {
        !matches!(
            self,
            EditorMode::UnitSheet | EditorMode::Units | EditorMode::EventViewer | EditorMode::Lobby
        )
    }
    /// Modes that lock camera drag/zoom (sprite browser, event viewer).
    pub fn disables_camera(self) -> bool {
        matches!(self, EditorMode::Units | EditorMode::EventViewer)
    }
    /// Modes that show no hex hover marker (unit sheet, event viewer, hexside editor).
    pub fn hides_hex_hover(self) -> bool {
        self.is_hexside() || matches!(self, EditorMode::UnitSheet | EditorMode::EventViewer)
    }
    pub fn edit_board(self) -> Option<omdurman_types::MapKind> {
        match self {
            EditorMode::Overlay | EditorMode::Editor | EditorMode::Hexside | EditorMode::FallOfKhartoumMap => {
                Some(omdurman_types::MapKind::FallOfKhartoum)
            }
            EditorMode::CampaignOverlay
            | EditorMode::CampaignEditor
            | EditorMode::CampaignHexside
            | EditorMode::CampaignMap => Some(omdurman_types::MapKind::Campaign),
            _ => None,
        }
    }
}

impl std::fmt::Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorMode::Overlay => write!(f, "Overlay"),
            EditorMode::Editor => write!(f, "Editor"),
            EditorMode::UnitSheet => write!(f, "Unit Sheet"),
            EditorMode::Units => write!(f, "Units"),
            EditorMode::Dice => write!(f, "Dice"),
            EditorMode::EventViewer => write!(f, "EventViewer"),
            EditorMode::CampaignOverlay => write!(f, "Campaign Overlay"),
            EditorMode::CampaignEditor => write!(f, "Campaign Editor"),
            EditorMode::Hexside => write!(f, "Hexsides"),
            EditorMode::CampaignHexside => write!(f, "Campaign Hexsides"),
            EditorMode::Lobby => write!(f, "Lobby"),
            EditorMode::FallOfKhartoumMap => write!(f, "Fall Of Khartoum Map"),
            EditorMode::CampaignMap => write!(f, "Campaign Map"),
        }
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
            None => self.by_peer.is_empty(), // unbound sandbox → no restriction
        }
    }
}

// ── System sets ──────────────────────────────────────────────────────────

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
/// Omdurman mapsheet — they differ only in set-up, length, and victory, not
/// terrain — so both use the campaign map (the lettered set-up hexes A/D/Y/K/S/O
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
pub(crate) struct TurnState {
    pub my_index: usize,
    pub current_turn: usize,
    pub pending_roll: Option<u32>,
    pub game_started: bool,
    /// Set when the local player submits an `Action` and cleared when the
    /// host-sequenced echo of *any* `Action` is applied. Under host-relay the
    /// turn advances only on that echo (apply-on-echo, §ordering), so this
    /// guards against the player acting again during the round trip.
    pub action_in_flight: bool,
}

/// Written every frame by `render::update_selection_marker` with the hex
/// currently under the cursor (or `None` if no valid hex is hovered).
#[derive(Resource, Default)]
pub struct HoveredHex(pub Option<HexCoord>);

pub(crate) fn camera_enabled(mode: Res<State<EditorMode>>) -> bool {
    !mode.disables_camera()
}

pub(crate) fn hex_hover_visible(mode: Res<State<EditorMode>>) -> bool {
    !mode.hides_hex_hover()
}

fn map_mode_active(mode: EditorMode) -> bool {
    mode.is_map_mode() || mode.is_overlay() || mode.is_editor() || mode.is_hexside()
}

pub(crate) fn map_mode_active_state(mode: Res<State<EditorMode>>) -> bool {
    map_mode_active(**mode)
}

/// Look up a counter's authored [`SpriteAnnotation`] and build its rules
/// profile. Returns `None` if annotations aren't loaded yet, the counter has
/// no annotation, or its section name is unrecognised — in every case the
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
    // Tracks entities spawned this invocation so MoveUnit can find units
    // placed in the same batch (e.g. during history replay) before Bevy
    // has flushed the deferred commands.
    // key: (section_name, col, row), value: (entity, is_boat, unit_id)
    mut just_placed: Local<HashMap<(SectionName, u32, u32), (Entity, bool, Option<UnitId>)>>,
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
                        just_placed.get(&(section_name, col, row))
                {
                    info!(
                        ?section_name,
                        col, row, "apply_pending_placement: MoveUnit fell back to just_placed",
                    );
                    // Route through the rules engine (see apply_move_effect).
                    if let Some(uid) = unit_id
                        && let Some(ref mut gs) = game_state
                    {
                        apply_move_effect(&mut gs.0, uid, target);
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
            // replay_game_history — they shouldn't appear in the deferred
            // queues. Warn if one does so the misclassification is visible.
            other => warn!(?other, "non-placement GameEvent in placement queue"),
        }
    }

    // ── Ephemeral messages handled by apply_ephemeral() — see below ──
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

#[derive(Resource, Default)]
pub struct MapStateStore {
    pub fall_of_khartoum_state: Option<GameState>,
    pub campaign_state: Option<GameState>,
    pub fall_of_khartoum_picker: Option<picker::UnitPicker>,
    pub campaign_picker: Option<picker::UnitPicker>,
}

impl MapStateStore {
    pub(crate) fn stash_current_as(
        &mut self,
        target: omdurman_types::MapKind,
        game_state: &GameStateResource,
        picker: &picker::UnitPicker,
    ) {
        let other_state = self.state_for(target);
        *other_state = Some(game_state.0.clone());
        let other_picker = self.picker_for(target);
        *other_picker = Some(picker.clone());
    }
    pub(crate) fn restore(
        &mut self,
        target: omdurman_types::MapKind,
        game_state: &mut GameStateResource,
        picker: &mut picker::UnitPicker,
    ) {
        if let Some(state) = self.state_for(target).take() {
            game_state.0 = state;
        }
        if let Some(stashed) = self.picker_for(target).take() {
            *picker = stashed;
        }
    }
    pub(crate) fn other(kind: omdurman_types::MapKind) -> omdurman_types::MapKind {
        match kind {
            omdurman_types::MapKind::FallOfKhartoum => omdurman_types::MapKind::Campaign,
            omdurman_types::MapKind::Campaign => omdurman_types::MapKind::FallOfKhartoum,
        }
    }
    pub(crate) fn state_for(
        &mut self,
        kind: omdurman_types::MapKind,
    ) -> &mut Option<GameState> {
        match kind {
            omdurman_types::MapKind::FallOfKhartoum => &mut self.fall_of_khartoum_state,
            omdurman_types::MapKind::Campaign => &mut self.campaign_state,
        }
    }
    pub(crate) fn picker_for(
        &mut self,
        kind: omdurman_types::MapKind,
    ) -> &mut Option<picker::UnitPicker> {
        match kind {
            omdurman_types::MapKind::FallOfKhartoum => &mut self.fall_of_khartoum_picker,
            omdurman_types::MapKind::Campaign => &mut self.campaign_picker,
        }
    }
}

// ── Late-joiner sync tests ────────────────────────────────────────────────────

#[cfg(test)]
mod late_joiner_tests {
    use super::*;
    use bevy::ecs::world::CommandQueue;
    use chrono::Utc;
    use omdurman_net::{GameEvent, GameRecord, InitialGameState, RecordedEvent, new_seed};
    use omdurman_types::{
        HexCoord, MapKind, OverlayParams, SectionName, SpriteAnnotation, SpriteAnnotations,
        Terrain, TileInfo,
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
            dirty_grids: std::collections::HashSet::new(),
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
    fn map_edit_replayed() {
        // MapEdit only applies to on-map coords; seed the map first.
        let record = make_record(vec![
            GameEvent::LoadAnnotations(Box::new(empty_annotations_file())),
            GameEvent::MapEdit {
                map: omdurman_types::MapKind::FallOfKhartoum,
                q: 1,
                r: 2,
                terrain: Terrain::Desert as u8,
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
        assert_eq!(hex.terrain, Terrain::Desert);
        assert_eq!(hex.name.as_deref(), Some("Khartoum"));
    }

    // ── load annotations rebuilds the map ────────────────────────────────────

    #[test]
    fn load_annotations_replayed() {
        use std::collections::BTreeMap;
        let mut tiles = BTreeMap::new();
        tiles.insert(
            (3, 4),
            TileInfo {
                terrain: Terrain::BlueNile,
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
        assert_eq!(hex.terrain, Terrain::BlueNile);
        assert_eq!(hex.name.as_deref(), Some("Nile"));
    }

    // ── overlay update synced ────────────────────────────────────────────────

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

    // ── annotate sprite ──────────────────────────────────────────────────────

    #[test]
    fn annotate_sprite_replayed() {
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
            GameEvent::LoadAnnotations(Box::new(empty_annotations_file())),
            GameEvent::AnnotateSprite {
                section_name: SectionName::Baggara,
                col: 0,
                row: 1,
                annotation: ann.clone(),
            },
        ]);
        let (_, _, _, _, _, annotations, ..) = run_replay(&record, 2);
        let ann_res = annotations.unwrap();
        let entry = ann_res.0.units[&SectionName::Baggara][&(0, 1)].clone();
        assert_eq!(entry.text, "Camel Corps");
    }

    // ── unit placement queued for apply_pending_placement ────────────────────

    #[test]
    fn place_unit_queued_in_incoming() {
        let record = make_record(vec![GameEvent::PlaceUnit {
            section_name: SectionName::Baggara,
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
                assert_eq!(*section_name, SectionName::Baggara);
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
    fn move_unit_queued_in_incoming() {
        let record = make_record(vec![
            GameEvent::PlaceUnit {
                section_name: SectionName::HadendowaForts,
                col: 0,
                row: 0,
                coord_q: 1,
                coord_r: 1,
                is_boat: false,
            },
            GameEvent::MoveUnit {
                section_name: SectionName::HadendowaForts,
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
    fn turn_counter_restored() {
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
    fn turn_counter_zero_actions() {
        let record = make_record(vec![]);
        let (.., turn, _) = run_replay(&record, 2);
        assert_eq!(turn.current_turn, 0);
    }

    // ── show terrain overlay ─────────────────────────────────────────────────

    #[test]
    fn show_terrain_overlay_replayed() {
        let record = make_record(vec![GameEvent::ShowTerrainOverlay(true)]);
        let (_, _, _, editor, ..) = run_replay(&record, 2);
        assert!(editor.show_terrain_overlay);
    }

    // ── unit grids synced ────────────────────────────────────────────────────

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

    // ── move after place in same batch ───────────────────────────────────────

    #[test]
    fn move_after_place_queued_in_order() {
        // PlaceUnit at (1,1) then MoveUnit to (7,8) — both in the same replay
        // batch.  The incoming queue must contain both events in order so that
        // apply_pending_placement can use the just_placed fallback map to apply
        // the move even though Bevy hasn't flushed the spawn command yet.
        let record = make_record(vec![
            GameEvent::PlaceUnit {
                section_name: SectionName::Baggara,
                col: 0,
                row: 0,
                coord_q: 1,
                coord_r: 1,
                is_boat: false,
            },
            GameEvent::MoveUnit {
                section_name: SectionName::Baggara,
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
    fn map_cleared_before_replay() {
        // Pre-populate the map with a hex that is NOT in the record.
        // After replay it must be gone, and the new hex must be present.
        let record = make_record(vec![
            GameEvent::LoadAnnotations(Box::new(empty_annotations_file())),
            GameEvent::MapEdit {
                map: MapKind::FallOfKhartoum,
                q: 0,
                r: 0,
                terrain: Terrain::Desert as u8,
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
            dirty_grids: std::collections::HashSet::new(),
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
                terrain: Terrain::Desert,
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
                        | GameEvent::Action(_)
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
                    section_name: SectionName::BritishArmy,
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

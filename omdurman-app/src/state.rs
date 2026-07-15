//! App-level state enums, game-state resources, and view-gating predicates.
//!
//! Collected here so [`crate::main`] stays focused on plugin wiring. The state
//! enums ([`AppState`], [`AppMode`], [`EditorTab`]) drive Bevy's state machine;
//! the resources wrap rules-engine state ([`GameStateResource`]), the
//! deterministic PRNG ([`GameRng`]), and various UI/lobby bindings. Everything
//! is re-exported at the crate root via `pub(crate) use state::*` so existing
//! `crate::Foo` paths continue to resolve.

use bevy::prelude::*;
use bevy_egui::egui;
use bevy_matchbox::prelude::PeerId;
use omdurman_net::GameEvent;
use omdurman_rules::effects::GameState;
use omdurman_rules::DieRoll;
use omdurman_types::{HexCoord, Player, Scenario};
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::{HashMap, HashSet};

// -- App state enums --------------------------------------------------------

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    #[default]
    Splash,
    Connecting,
    Lobby,
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
///   its board is [`crate::EditorBoard`].
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
    /// All top-level modes, in display order.
    pub const ALL: [AppMode; 3] = [AppMode::Game, AppMode::Sandbox, AppMode::Editor];

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
/// Terrain, Hexside, Timing) act on the [`crate::EditorBoard`]; the rest are
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
        matches!(
            self,
            EditorTab::Sprites | EditorTab::EventViewer | EditorTab::Charts
        )
    }
    /// Whether this tab shows no hex hover marker.
    pub fn hides_hex_hover(self) -> bool {
        matches!(
            self,
            EditorTab::Hexside | EditorTab::UnitSheet | EditorTab::EventViewer | EditorTab::Charts
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

    /// The lowercase env-var key for `OMDURMAN_START_TAB` matching.
    pub fn env_key(self) -> &'static str {
        match self {
            EditorTab::Overlay => "overlay",
            EditorTab::Terrain => "terrain",
            EditorTab::Hexside => "hexside",
            EditorTab::Timing => "timing",
            EditorTab::UnitSheet => "unitsheet",
            EditorTab::Sprites => "sprites",
            EditorTab::Dice => "dice",
            EditorTab::EventViewer => "events",
            EditorTab::Charts => "charts",
        }
    }
}

// -- System sets ------------------------------------------------------------

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EditorSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OverlaySet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexsideSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSet;

// -- Game-state resources ---------------------------------------------------

/// Deterministic PRNG resource shared by every peer. Seeded from the
/// canonical game record so late joiners reproduce the same sequence.
#[derive(Resource)]
pub struct GameRng(ChaCha8Rng);

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
    /// Roll a d10 (1..=10) as a validated [`DieRoll`]. The 1..=10 range is a
    /// closed subset of `DieRoll`'s valid domain, so the conversion never
    /// fails; consolidating it here keeps the modulo-10 + `unwrap` pattern in
    /// one place.
    pub fn roll_d10(&mut self) -> DieRoll {
        DieRoll::try_from(((self.0.random::<u32>() % 10) + 1) as u16).unwrap()
    }
}

/// Bevy resource wrapper around the rules engine's game state.
#[derive(Resource)]
pub struct GameStateResource(pub GameState);

/// Buffers sequenced game events that [`crate::net_socket::handle_socket`] has
/// just applied, so a scheduled system can drain them into
/// [`crate::events::GameEventApplied`] messages for UI/game listeners without
/// coupling to the socket handler directly.
#[derive(Resource, Default)]
pub struct AppliedEvents(pub Vec<(GameEvent, u32)>);

/// Tracks which unit entity is currently selected by the local player.
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct SelectedUnit(pub Option<Entity>);

/// Set by settings_ui when the user clicks Host or Join.
/// The system `handle_reconnect` picks this up, disconnects from
/// the current room, and opens a new socket with the new room ID.
#[derive(Resource)]
pub struct ReconnectRoom(pub String);

/// Written every frame by `render::update_selection_marker` with the hex
/// currently under the cursor (or `None` if no valid hex is hovered).
#[derive(Resource, Default)]
pub struct HoveredHex(pub Option<HexCoord>);

/// Current game turn (1-based) for the campaign or scenario.
#[derive(Resource, Deref, DerefMut)]
pub struct GameTurn(pub u8);

impl Default for GameTurn {
    fn default() -> Self {
        Self(1)
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
pub struct LobbyScenario(pub Scenario);

impl Default for LobbyScenario {
    fn default() -> Self {
        Self(Scenario::Campaign)
    }
}

/// Live (pre-commit) lobby faction picks, keyed by `PeerId`. Populated from
/// `Ephemeral::FactionChoice` for display in the lobby; the local pick lives in
/// `LocalFaction`.
#[derive(Resource, Default)]
pub struct LobbyChoices {
    pub by_peer: HashMap<PeerId, Option<Player>>,
    /// Peers who have toggled "Spectate" in the lobby (live preview). A
    /// spectator is never assigned a faction, so it shows as "spectating" in the
    /// roster and is ignored by the start-readiness check.
    pub spectators: HashSet<PeerId>,
    /// Latest scenario broadcast by the host's lobby (live preview, §lobby).
    /// `None` until the host sends one; the committed value rides in
    /// [`GameEvent::StartGame`].
    pub scenario: Option<Scenario>,
}

/// The local player's current lobby faction pick (pre-commit).
#[derive(Resource, Default)]
pub struct LocalFaction(pub Option<Player>);

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

#[derive(Resource, Default)]
pub struct SidebarClip {
    pub right_sidebar: Option<egui::Rect>,
}

// -- View-gating predicates -------------------------------------------------

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

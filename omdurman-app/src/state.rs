//! App-level state enums, game-state resources, and view-gating predicates.
//!
//! Collected here so [`crate::main`] stays focused on plugin wiring. The state
//! enums ([`AppState`], [`AppMode`], [`EditorTab`]) drive Bevy's state machine;
//! the resources wrap rules-engine state ([`GameStateResource`]), the
//! deterministic PRNG ([`GameRng`]), and view-gating predicates. Domain-specific
//! resources have been moved to their owning modules and are re-exported at the
//! crate root via `pub(crate) use` in [`crate::main`].

use bevy::prelude::*;
use omdurman_rules::effects::GameState;
use omdurman_rules::DieRoll;
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// -- App state enums --------------------------------------------------------

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    #[default]
    Splash,
    Lobby,
    InGame,
    /// Reviewing a recorded game (in-memory or loaded from disk) on the timeline
    /// scrubber, disconnected from any live socket (§spectator). The rules/map
    /// state is rebuilt from the record to the timeline cursor; live net systems
    /// are gated off in this state.
    Spectating,
}

/// Top-level app mode, chosen from the mode picker. Orthogonal to [`AppState`]
/// (which tracks the networking/game lifecycle: Lobby/InGame/Spectating). The
/// picker shows three entries: **Lobby/Game** (whichever `AppState` applies),
/// and **Editor**.
///
/// - `Game`  — the live/networked game view (or the lobby, per `AppState`).
/// - `Editor` — the map/annotation editor; its sub-tools are [`EditorTab`]s and
///   its board is [`crate::EditorBoard`].
///
/// Only `Editor` shows editor tooling; `Game` shows the play board (unit picker,
/// overview, gameplay overlays).
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppMode {
    /// Persistent main menu — the hub for mode selection. Entered from any mode
    /// via the M key, and on first load once the board texture is ready.
    #[default]
    Menu,
    /// Networked game setup: faction / scenario picks, player roster.
    Lobby,
    /// Active networked game.
    Game,
    /// Map / annotation editor.
    Editor,
}

impl AppMode {
    /// All top-level modes, in display order.
    pub const ALL: [AppMode; 4] = [
        AppMode::Menu,
        AppMode::Lobby,
        AppMode::Game,
        AppMode::Editor,
    ];

    /// Whether this mode shows the playable board view (picker, overview,
    /// gameplay overlays, placed units): `Game`, not `Menu`, `Lobby`, or
    /// `Editor`.
    pub fn is_play(self) -> bool {
        matches!(self, AppMode::Game)
    }
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Menu => write!(f, "Menu"),
            AppMode::Lobby => write!(f, "Lobby"),
            AppMode::Game => write!(f, "Game"),
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
    /// Read-only recorded-event log viewer.
    EventViewer,
    /// Reference-chart sheet preview (the only editor tab that shows charts;
    /// charts are otherwise a play-view feature, hidden in the editor).
    Charts,
}

impl EditorTab {
    /// The board this tab edits, or `None` for board-agnostic tabs
    /// (Sprites/UnitSheet/EventViewer) that ignore the editor board pick.
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
            EditorTab::EventViewer => "Events",
            EditorTab::Charts => "Charts",
        }
    }
    /// The tab bar, in display order.
    pub const ALL: [EditorTab; 8] = [
        EditorTab::Overlay,
        EditorTab::Terrain,
        EditorTab::Hexside,
        EditorTab::Timing,
        EditorTab::UnitSheet,
        EditorTab::Sprites,
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

/// Current game turn (1-based) for the campaign or scenario.
#[derive(Resource, Deref, DerefMut)]
pub struct GameTurn(pub u8);

impl Default for GameTurn {
    fn default() -> Self {
        Self(1)
    }
}

// -- View-gating predicates -------------------------------------------------

/// Camera drag/zoom is disabled only for the editor tabs that lock it (sprite
/// browser, event viewer). Every play view and the other editor tabs allow it.
pub(crate) fn camera_enabled(mode: Res<State<AppMode>>, tab: Res<State<EditorTab>>) -> bool {
    match **mode {
        AppMode::Editor => !tab.disables_camera(),
        AppMode::Menu => false,
        _ => true,
    }
}

/// The hex hover marker is shown on the play board and on editor tabs that don't
/// suppress it (hexside/unit-sheet/event-viewer hide it).
pub(crate) fn hex_hover_visible(mode: Res<State<AppMode>>, tab: Res<State<EditorTab>>) -> bool {
    match **mode {
        AppMode::Editor => !tab.hides_hex_hover(),
        AppMode::Menu => false,
        _ => true,
    }
}

/// Whether a hex-grid-bearing view is active (cursor broadcast / cursor overlay
/// gate): any play view, or an editor tab that shows the map plane.
pub(crate) fn map_view_active(mode: Res<State<AppMode>>, tab: Res<State<EditorTab>>) -> bool {
    match **mode {
        AppMode::Game => true,
        AppMode::Menu => false,
        AppMode::Editor => tab.shows_map_plane(),
        AppMode::Lobby => false,
    }
}

// -- Phase-condition predicates (for `.run_if`) ------------------------------

/// True during any fire-combat sub-phase (defensive or offensive, any kind).
pub(crate) fn in_fire_phase(game_state: Option<Res<GameStateResource>>) -> bool {
    game_state.is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::DefensiveFire(_) | omdurman_rules::Phase::OffensiveFire(_)))
}

/// True during offensive fire (the active player fires offensively).
pub(crate) fn in_offensive_fire(game_state: Option<Res<GameStateResource>>) -> bool {
    game_state.is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::OffensiveFire(_)))
}

/// True during defensive fire (the opponent fires defensively).
pub(crate) fn in_defensive_fire(game_state: Option<Res<GameStateResource>>) -> bool {
    game_state.is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::DefensiveFire(_)))
}

/// True during the movement phase.
pub(crate) fn in_movement(game_state: Option<Res<GameStateResource>>) -> bool {
    game_state.is_some_and(|gs| gs.0.phase == omdurman_rules::Phase::Movement)
}

/// True during the melee phase.
pub(crate) fn in_melee(game_state: Option<Res<GameStateResource>>) -> bool {
    game_state.is_some_and(|gs| gs.0.phase == omdurman_rules::Phase::Melee)
}

// -- Per-mode snapshot resources ----------------------------------------------

/// Serializable data for one placed unit, used to snapshot/restore placed units
/// across mode transitions without touching Bevy entities directly.
#[derive(Clone, Debug)]
pub struct PlacedUnitData {
    pub section_name: omdurman_types::SectionName,
    pub col: u32,
    pub row: u32,
    pub coord: omdurman_types::HexCoord,
    pub unit_id: Option<omdurman_rules::UnitId>,
    pub disrupted: bool,
    pub is_boat: bool,
}

/// Snapshot of the **Game** mode state. Saved when leaving Game (via M key)
/// and restored when re-entering. Keeps the game's rules state, faction
/// binding, turn counter, and placed-unit layout independent from other modes.
#[derive(Resource, Default)]
pub struct GameSnapshot {
    pub game_state: Option<omdurman_rules::effects::GameState>,
    pub factions: Vec<(bevy_matchbox::prelude::PeerId, omdurman_types::Player)>,
    pub game_turn: u8,
    pub placed_units: Vec<PlacedUnitData>,
    pub has_data: bool,
}

/// Snapshot of the **Lobby** mode state. Persists faction / scenario picks
/// and tab selection across menu round-trips.
#[derive(Resource, Default)]
pub struct LobbySnapshot {
    pub scenario: omdurman_types::Scenario,
    pub local_faction: Option<omdurman_types::Player>,
    pub local_spectator: bool,
    pub has_data: bool,
}

/// Snapshot of the **Editor** mode state. Persists the active board across
/// menu round-trips.
#[derive(Resource, Default)]
pub struct EditorSnapshot {
    pub board: omdurman_types::Scenario,
    pub has_data: bool,
}

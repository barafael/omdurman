//! App-level state enums, game-state resources, and view-gating predicates.
//!
//! Collected here so [`crate::main`] stays focused on plugin wiring. The state
//! enums ([`AppState`], [`AppMode`]) drive Bevy's state machine;
//! the resources wrap rules-engine state ([`GameStateResource`]), the
//! deterministic PRNG ([`GameRng`]), and view-gating predicates. Domain-specific
//! resources have been moved to their owning modules and are re-exported at the
//! crate root via `pub(crate) use` in [`crate::main`].

use bevy::prelude::*;
use omdurman_rules::effects::GameState;

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
/// (which tracks the networking/game lifecycle: Lobby/InGame/Spectating).
///
/// - `Game`  — the live/networked game view (or the lobby, per `AppState`).
///
/// `Game` shows the play board (unit picker, overview, gameplay overlays).
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
}

impl AppMode {
    /// All top-level modes, in display order.
    pub const ALL: [AppMode; 3] = [AppMode::Menu, AppMode::Lobby, AppMode::Game];

    /// Whether this mode shows the playable board view (picker, overview,
    /// gameplay overlays, placed units): `Game`, not `Menu` or `Lobby`.
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
        }
    }
}

// -- System sets ------------------------------------------------------------

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSet;

// -- Game-state resources ---------------------------------------------------

/// Bevy `Resource` wrapper around the engine's shared deterministic PRNG
/// ([`omdurman_rules::rng::GameRng`]). The dice-stream implementation itself
/// lives in the rules crate so the headless bot draws from the same code
/// (previously two hand-mirrored copies existed); this newtype only supplies
/// the `Resource` impl Bevy needs. `Deref`/`DerefMut` keep `roll_d10` etc.
/// working unchanged at every call site.
#[derive(Resource)]
pub struct GameRng(omdurman_rules::rng::GameRng);

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(omdurman_rules::rng::GameRng::from_seed(seed))
    }
}

impl std::ops::Deref for GameRng {
    type Target = omdurman_rules::rng::GameRng;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for GameRng {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

/// Camera drag/zoom is enabled everywhere but the menu.
pub(crate) fn camera_enabled(mode: Res<State<AppMode>>) -> bool {
    !matches!(**mode, AppMode::Menu)
}

/// The hex hover marker is shown on the play board (not the menu).
pub(crate) fn hex_hover_visible(mode: Res<State<AppMode>>) -> bool {
    !matches!(**mode, AppMode::Menu)
}

/// Whether a hex-grid-bearing view is active (cursor broadcast / cursor overlay
/// gate): the play view.
pub(crate) fn map_view_active(mode: Res<State<AppMode>>) -> bool {
    matches!(**mode, AppMode::Game)
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
    /// Command scopes bound at `StartGame` (§1.1 multi-player commands).
    pub commands: Vec<(bevy_matchbox::prelude::PeerId, omdurman_types::CommandScope)>,
    pub game_turn: u8,
    pub placed_units: Vec<PlacedUnitData>,
    pub has_data: bool,
}

/// Snapshot of the **Lobby** mode state. Persists faction / command /
/// scenario picks and tab selection across menu round-trips.
#[derive(Resource, Default)]
pub struct LobbySnapshot {
    pub scenario: omdurman_types::Scenario,
    pub local_faction: Option<omdurman_types::Player>,
    pub local_spectator: bool,
    pub local_command: Option<omdurman_types::CommandScope>,
    pub has_data: bool,
}

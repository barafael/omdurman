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

/// The board view of a live *or* reviewed game: `AppMode::Game` while
/// `InGame` or `Spectating`. Gate for the per-frame board markers, whose
/// entities `clear_gameplay_overlays` removes when this view is left.
pub(crate) fn board_view_active(mode: Res<State<AppMode>>, state: Res<State<AppState>>) -> bool {
    matches!(**mode, AppMode::Game) && matches!(**state, AppState::InGame | AppState::Spectating)
}

/// The live game's board view: `InGame` *and* `AppMode::Game`. The menu is
/// shown with `AppState::InGame` too, so in-game HUD / cards gate on this
/// rather than on the app state alone, or they would draw over the menu.
pub(crate) fn in_game_view(mode: Res<State<AppMode>>, state: Res<State<AppState>>) -> bool {
    matches!(**mode, AppMode::Game) && matches!(**state, AppState::InGame)
}

/// Whether there is a game to return to from the menu / lobby: a `StartGame`
/// was applied (live or via an installed history) or the record holds events.
pub(crate) fn game_in_progress(
    turn: &crate::TurnState,
    recorder: &crate::game_record::GameRecorder,
) -> bool {
    turn.game_started
        || recorder
            .record
            .as_ref()
            .is_some_and(|r| !r.events.is_empty())
}

/// Switch to the live game's board view. `AppMode` and `AppState` are
/// independent axes; the board view of a live game is `Game` + `InGame`, so
/// both are set here (leaving a `Lobby` app state behind would keep the lobby
/// UI drawn over the board). Shared by the menu and the toolbar buttons.
pub(crate) fn enter_game_view(
    next_mode: &mut NextState<AppMode>,
    next_state: &mut NextState<AppState>,
) {
    next_mode.set(AppMode::Game);
    next_state.set(AppState::InGame);
}

// -- Per-mode snapshot resources ----------------------------------------------

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

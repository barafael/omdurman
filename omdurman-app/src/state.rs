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

// -- The screen that is up ---------------------------------------------------

/// Which screen is up, derived from the two state axes: [`AppMode`] (what the
/// player picked: menu, lobby, game) and [`AppState`] (where the session is:
/// splash, lobby, in a game, reviewing one). Gate a system on the screen it
/// draws on, not on either axis alone: the menu is shown in every
/// `AppState`, and a lobby or game system gated on the state alone draws
/// over the menu.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Screen {
    /// The title screen: the start-up splash, or the menu over whatever
    /// session is running.
    Title,
    /// The lobby.
    Lobby,
    /// The board of the live game.
    Board,
    /// The board of a game under review on the timeline.
    Review,
}

impl ComputedStates for Screen {
    type SourceStates = (AppState, AppMode);

    fn compute((state, mode): (AppState, AppMode)) -> Option<Self> {
        match (state, mode) {
            (AppState::Splash, _) | (_, AppMode::Menu) => Some(Screen::Title),
            (AppState::Lobby, _) => Some(Screen::Lobby),
            (AppState::InGame, AppMode::Game) => Some(Screen::Board),
            (AppState::Spectating, AppMode::Game) => Some(Screen::Review),
            // (The lobby mode is entered together with the lobby state; a
            // frame between the two shows nothing.)
            (AppState::InGame | AppState::Spectating, AppMode::Lobby) => None,
        }
    }
}

/// A board is up: the live game's or a reviewed one's.
pub(crate) fn on_board(screen: Option<Res<State<Screen>>>) -> bool {
    screen.is_some_and(|screen| matches!(**screen, Screen::Board | Screen::Review))
}

/// Anything but the title screen is up (the camera, the hover marker).
pub(crate) fn off_title(screen: Option<Res<State<Screen>>>) -> bool {
    screen.is_none_or(|screen| **screen != Screen::Title)
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

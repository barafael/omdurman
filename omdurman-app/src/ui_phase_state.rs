//! Phase-level UI state machine for "Remember Gordon!"
//!
//! [`UiPhaseState`] is derived from the authoritative [`GameState`] each frame
//! and drives the right-sidebar action guide, the status-line phase label, and
//! the input-gating rings on the map.  Every variant carries the rulebook
//! section(s) that authorise the legal actions in that state so the UI can
//! deep-link players into the manual.
//!
//! The goal is one canonical source of truth for "what may the active player do
//! right now?" that the action panel, the game-control section, and the combat
//! click handlers all share.
//!
//! # Structure
//!
//! The flat 16-variant enum was restructured into a three-level hierarchy so
//! that orthogonal concerns are separate dimensions, not crossed variant names:
//!
//! * [`UiPhaseState`] — top-level: `Setup`, `Turn { active, night, phase }`,
//!   `GameOver`.
//! * [`PhaseKind`] — the phase within a turn: `Movement`, `DefensiveFire(_)`,
//!   `OffensiveFire(_)`, `Melee`.
//! * [`FireSubKind`] — direct vs. Maxim/howitzer fire.
//!
//! The `active` player and `night` flag are fields on `Turn`, not variant
//! dimensions, so [`firing_player`] branches explicitly on offensive vs.
//! defensive rather than relying on a naming convention.

use bevy::prelude::*;
use omdurman_rules::effects::GameState;
use omdurman_rules::{FireSubPhase, Phase};
use omdurman_types::{DayNight, Player};

// ---------------------------------------------------------------------------
// UiPhaseState + sub-enums
// ---------------------------------------------------------------------------

/// The fine-grained UI state derived from [`GameState`] each frame.
///
/// The `Turn` variant carries the *active player* (whose turn it is), the
/// *night flag*, and the *phase kind* as orthogonal fields — not as dimensions
/// folded into variant names.  This makes [`firing_player`] explicit: offensive
/// fire → active player fires; defensive fire → opponent fires.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiPhaseState {
    /// Pre-game deployment (§9.2/§9.3/§10). Both sides deploy concurrently;
    /// no turn/phase indicator shown.
    Setup,

    /// An active player's turn is in progress.
    Turn {
        /// Whose turn it is (the rules engine's `active_player`).
        active: Player,
        /// Whether night rules are in effect (§8.1): halved MP, halved fire
        /// ranges, no howitzer fire.
        night: bool,
        /// The phase within this turn.
        phase: PhaseKind,
    },

    /// The game has ended (§9.14/§9.24/§9.35).
    GameOver,
}

/// The phase within a player's turn (§4 sequence: Movement → Fire → Melee).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhaseKind {
    /// Movement phase (§5).
    Movement,
    /// Defensive fire — the *non-active* player fires back (§6.41/§6.42).
    DefensiveFire(FireSubKind),
    /// Offensive fire — the *active* player fires (§6.41/§6.42).
    OffensiveFire(FireSubKind),
    /// Melee phase (§7).
    Melee,
}

/// Direct fire vs. Maxim-second-fire-plus-howitzer (§6.41 vs §6.42).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FireSubKind {
    Direct,
    MaximHowitzer,
}

fn fire_sub_kind(sub: FireSubPhase) -> FireSubKind {
    match sub {
        FireSubPhase::DirectFire => FireSubKind::Direct,
        FireSubPhase::MaximSecondAndHowitzer => FireSubKind::MaximHowitzer,
    }
}

impl UiPhaseState {
    /// Derive the current UI phase state from the authoritative game state.
    ///
    /// This is a pure function — no mutation, no I/O — so it can be called
    /// every frame and the result cached cheaply.
    pub fn derive(gs: &GameState) -> Self {
        if gs.game_over {
            return Self::GameOver;
        }
        if gs.phase == Phase::Setup {
            return Self::Setup;
        }
        Self::Turn {
            active: gs.active_player,
            night: gs.day_night == DayNight::Night,
            phase: match gs.phase {
                Phase::Movement => PhaseKind::Movement,
                Phase::DefensiveFire(sub) => PhaseKind::DefensiveFire(fire_sub_kind(sub)),
                Phase::OffensiveFire(sub) => PhaseKind::OffensiveFire(fire_sub_kind(sub)),
                Phase::Melee => PhaseKind::Melee,
                Phase::Setup => unreachable!(),
            },
        }
    }

    // -- Accessors used by the UI layer ---------------------------------------

    /// The player who is *firing* in the current fire phase, or `None` if
    /// not in a fire phase.  During offensive fire the active player fires;
    /// during defensive fire the opponent of the active player fires (the
    /// non-active side shoots back §6.41).
    pub fn firing_player(self) -> Option<Player> {
        match self {
            Self::Turn {
                active,
                phase: PhaseKind::OffensiveFire(_),
                ..
            } => Some(active),
            Self::Turn {
                active,
                phase: PhaseKind::DefensiveFire(_),
                ..
            } => Some(active.opponent()),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Phase display labels
// ---------------------------------------------------------------------------

impl UiPhaseState {
    /// Short human-readable phase name for the status bar and game-control
    /// section.  Includes the sub-phase detail but not the player (that is
    /// rendered separately).
    pub fn phase_label(self) -> &'static str {
        match self {
            Self::Setup => "Setup",
            Self::GameOver => "Game Over",
            Self::Turn {
                phase: PhaseKind::Movement,
                night: false,
                ..
            } => "Movement",
            Self::Turn {
                phase: PhaseKind::Movement,
                night: true,
                ..
            } => "Movement (Night)",
            Self::Turn {
                phase: PhaseKind::DefensiveFire(FireSubKind::Direct),
                ..
            } => "Defensive Fire — Direct",
            Self::Turn {
                phase: PhaseKind::DefensiveFire(FireSubKind::MaximHowitzer),
                ..
            } => "Defensive Fire — Maxim/Howitzer",
            Self::Turn {
                phase: PhaseKind::OffensiveFire(FireSubKind::Direct),
                ..
            } => "Offensive Fire — Direct",
            Self::Turn {
                phase: PhaseKind::OffensiveFire(FireSubKind::MaximHowitzer),
                ..
            } => "Offensive Fire — Maxim/Howitzer",
            Self::Turn {
                phase: PhaseKind::Melee,
                ..
            } => "Melee",
        }
    }

    /// The rulebook section(s) that describe this phase.  Used as the primary
    /// deep-link target in the action guide header.
    pub fn rulebook_section(self) -> &'static str {
        match self {
            Self::Setup => "9.2",
            Self::Turn {
                phase: PhaseKind::Movement,
                ..
            } => "5",
            Self::Turn {
                phase:
                    PhaseKind::DefensiveFire(FireSubKind::Direct)
                    | PhaseKind::OffensiveFire(FireSubKind::Direct),
                ..
            } => "6.41",
            Self::Turn {
                phase:
                    PhaseKind::DefensiveFire(FireSubKind::MaximHowitzer)
                    | PhaseKind::OffensiveFire(FireSubKind::MaximHowitzer),
                ..
            } => "6.42",
            Self::Turn {
                phase: PhaseKind::Melee,
                ..
            } => "7",
            Self::GameOver => "9",
        }
    }

    /// Compact phase-sequence indicator: four labels with the current one
    /// bracketed and completed ones checked.  Used by the status bar.
    pub fn phase_sequence(self) -> String {
        let (m, d, o, me): (&str, &str, &str, &str) = match self {
            Self::Setup | Self::GameOver => return self.phase_label().to_string(),
            Self::Turn {
                phase: PhaseKind::Movement,
                ..
            } => ("[Mov]", "\u{2713} Def", "Off", "Melee"),
            Self::Turn {
                phase: PhaseKind::DefensiveFire(_),
                ..
            } => ("\u{2713} Mov", "[Def]", "Off", "Melee"),
            Self::Turn {
                phase: PhaseKind::OffensiveFire(_),
                ..
            } => ("\u{2713} Mov", "\u{2713} Def", "[Off]", "Melee"),
            Self::Turn {
                phase: PhaseKind::Melee,
                ..
            } => ("\u{2713} Mov", "\u{2713} Def", "\u{2713} Off", "[Melee]"),
        };
        format!("{m} > {d} > {o} > {me}")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_rules::effects::GameState;
    use omdurman_types::{DayNight, Player, Scenario};

    // -- derive ---------------------------------------------------------------

    #[test]
    fn setup_derives_as_setup() {
        let gs = GameState::new(Scenario::Campaign);
        assert_eq!(UiPhaseState::derive(&gs), UiPhaseState::Setup);
    }

    #[test]
    fn game_over_derives_correctly() {
        let mut gs = GameState::new(Scenario::Campaign);
        gs.game_over = true;
        assert_eq!(UiPhaseState::derive(&gs), UiPhaseState::GameOver);
    }

    #[test]
    fn derive_movement() {
        let mut gs = GameState::new(Scenario::Campaign);
        gs.phase = Phase::Movement;

        gs.active_player = Player::AngloEgyptian;
        gs.day_night = DayNight::Day;
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::Movement,
            }
        );

        gs.active_player = Player::Dervish;
        gs.day_night = DayNight::Night;
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::Dervish,
                night: true,
                phase: PhaseKind::Movement,
            }
        );
    }

    #[test]
    fn derive_fire_phases() {
        let mut gs = GameState::new(Scenario::Campaign);

        // AE offensive direct.
        gs.active_player = Player::AngloEgyptian;
        gs.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::OffensiveFire(FireSubKind::Direct),
            }
        );

        // AE offensive Maxim/howitzer.
        gs.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::OffensiveFire(FireSubKind::MaximHowitzer),
            }
        );

        // Dervish defensive direct (AE is active → Dervish defends).
        gs.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::DefensiveFire(FireSubKind::Direct),
            }
        );

        // Dervish active, AE defensive direct.
        gs.active_player = Player::Dervish;
        gs.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::Dervish,
                night: false,
                phase: PhaseKind::DefensiveFire(FireSubKind::Direct),
            }
        );

        // Dervish active, AE defensive Maxim/howitzer.
        gs.phase = Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer);
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::Dervish,
                night: false,
                phase: PhaseKind::DefensiveFire(FireSubKind::MaximHowitzer),
            }
        );

        // Dervish offensive direct.
        gs.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::Dervish,
                night: false,
                phase: PhaseKind::OffensiveFire(FireSubKind::Direct),
            }
        );
    }

    #[test]
    fn derive_melee() {
        let mut gs = GameState::new(Scenario::Campaign);
        gs.phase = Phase::Melee;

        gs.active_player = Player::AngloEgyptian;
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::Melee,
            }
        );

        gs.active_player = Player::Dervish;
        assert_eq!(
            UiPhaseState::derive(&gs),
            UiPhaseState::Turn {
                active: Player::Dervish,
                night: false,
                phase: PhaseKind::Melee,
            }
        );
    }

    // -- firing_player --------------------------------------------------------

    #[test]
    fn firing_player_is_none_for_non_fire_phases() {
        assert_eq!(UiPhaseState::Setup.firing_player(), None);
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::Movement,
            }
            .firing_player(),
            None
        );
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::Melee,
            }
            .firing_player(),
            None
        );
        assert_eq!(UiPhaseState::GameOver.firing_player(), None);
    }

    #[test]
    fn firing_player_offensive_returns_active() {
        // AE offensive fire: AE fires.
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::OffensiveFire(FireSubKind::Direct),
            }
            .firing_player(),
            Some(Player::AngloEgyptian)
        );
        // Dervish offensive fire: Dervish fires.
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::Dervish,
                night: false,
                phase: PhaseKind::OffensiveFire(FireSubKind::Direct),
            }
            .firing_player(),
            Some(Player::Dervish)
        );
    }

    #[test]
    fn firing_player_defensive_returns_opponent() {
        // AE is active, Dervish fires defensively.
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::DefensiveFire(FireSubKind::Direct),
            }
            .firing_player(),
            Some(Player::Dervish)
        );
        // Dervish is active, AE fires defensively.
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::Dervish,
                night: false,
                phase: PhaseKind::DefensiveFire(FireSubKind::Direct),
            }
            .firing_player(),
            Some(Player::AngloEgyptian)
        );
        // Maxim/howitzer defensive too.
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::Dervish,
                night: false,
                phase: PhaseKind::DefensiveFire(FireSubKind::MaximHowitzer),
            }
            .firing_player(),
            Some(Player::AngloEgyptian)
        );
    }

    // -- phase_label / rulebook_section / phase_sequence ---------------------

    #[test]
    fn phase_label_matches_expectations() {
        assert_eq!(UiPhaseState::Setup.phase_label(), "Setup");
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::Movement,
            }
            .phase_label(),
            "Movement"
        );
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: true,
                phase: PhaseKind::Movement,
            }
            .phase_label(),
            "Movement (Night)"
        );
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::OffensiveFire(FireSubKind::MaximHowitzer),
            }
            .phase_label(),
            "Offensive Fire — Maxim/Howitzer"
        );
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::Melee,
            }
            .phase_label(),
            "Melee"
        );
        assert_eq!(UiPhaseState::GameOver.phase_label(), "Game Over");
    }

    #[test]
    fn phase_sequence_for_movement() {
        let seq = UiPhaseState::Turn {
            active: Player::AngloEgyptian,
            night: false,
            phase: PhaseKind::Movement,
        }
        .phase_sequence();
        assert!(seq.contains("[Mov]"));
        assert!(seq.contains("Def"));
    }

    #[test]
    fn phase_sequence_for_melee() {
        let seq = UiPhaseState::Turn {
            active: Player::AngloEgyptian,
            night: false,
            phase: PhaseKind::Melee,
        }
        .phase_sequence();
        assert!(seq.contains("[Melee]"));
        assert!(seq.contains("\u{2713} Mov"));
    }

    #[test]
    fn rulebook_section_for_fire_subphases() {
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::OffensiveFire(FireSubKind::Direct),
            }
            .rulebook_section(),
            "6.41"
        );
        assert_eq!(
            UiPhaseState::Turn {
                active: Player::AngloEgyptian,
                night: false,
                phase: PhaseKind::OffensiveFire(FireSubKind::MaximHowitzer),
            }
            .rulebook_section(),
            "6.42"
        );
    }
}

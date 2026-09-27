//! Players, turn sequence phases, and scenario options.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 2) Players and turn sequence
// ---------------------------------------------------------------------------

/// The fine-grained phase within a player-turn (rulebook §4).
///
/// Fire-combat phase is broken down so that the legality of every fire is
/// statically checkable: e.g. a howitzer fire can only resolve inside the
/// `MaximSecondAndHowitzer` sub-phase, defensive fire only in `DefensiveFire`,
/// etc.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Phase {
    /// Pre-game deployment (§9.2/§9.3/§10): fixed units are placed, each side
    /// deploys its order of battle within its legal zone, and river
    /// mines/chain/zariba are laid. The game leaves `Setup` for the first
    /// player's `Movement` turn only once `setup_complete` holds. The initial
    /// phase of every scenario.
    #[default]
    Setup,
    Movement,
    DefensiveFire(FireSubPhase),
    OffensiveFire(FireSubPhase),
    Melee,
}

impl Phase {
    /// Top-level phase name for UI display (collapses sub-phases).
    pub fn top_level_name(self) -> &'static str {
        match self {
            Phase::Setup => "Setup",
            Phase::Movement => "Movement",
            Phase::DefensiveFire(_) => "Defensive Fire",
            Phase::OffensiveFire(_) => "Offensive Fire",
            Phase::Melee => "Melee",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FireSubPhase {
    /// Direct fire (§6.41). Both sides participate in this sub-phase.
    DirectFire,
    /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
    MaximSecondAndHowitzer,
}

// ---------------------------------------------------------------------------
// 3) Scenarios
// ---------------------------------------------------------------------------

/// Optional rules -- only legal in the campaign game, and at most one of the
/// two should be in play (rulebook §10).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OptionalRule {
    RiverMines,
    RiverChain,
}

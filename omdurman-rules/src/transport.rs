//! Friendlies transport across the Nile and the optional river
//! obstacles (mines, chain).

use serde::{Deserialize, Serialize};

use crate::{DieRoll, GameTurnIndex, UnitId};
use omdurman_types::HexCoord;

// ---------------------------------------------------------------------------
// 13) Loading / transport of the "Friendlies" brigade across the Nile
// ---------------------------------------------------------------------------

/// The action payload for `GameEffect::FriendliesTransport` -- what the
/// player does with a "Friendlies" unit and a gunboat (§5.21). Between the
/// two, the gunboat carries the unit wherever it moves.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FriendliesAction {
    /// Turn N: "a 'Friendlies' unit and any Anglo-Egyptian gunboat start
    /// their turn adjacent" -- the unit loads onto (stacks with) the gunboat.
    Load { unit: UnitId, gunboat: UnitId },
    /// Turn N+2 or later: the unit disembarks onto the west-bank hex `to`
    /// next to its gunboat, "paying the normal terrain cost for the first
    /// hex entered", and may move on normally.
    Disembark {
        unit: UnitId,
        gunboat: UnitId,
        to: HexCoord,
    },
}

/// A Friendlies transport under way (§5.21), stored on `GameState`.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransportState {
    /// `unit` has been aboard `gunboat` since turn `since` (the load turn,
    /// N): the gunboat may carry it on turn N+1, and it may disembark from
    /// turn N+2 ("on the Anglo-Egyptian player's third turn").
    Loaded {
        unit: UnitId,
        gunboat: UnitId,
        since: GameTurnIndex,
    },
}

// ---------------------------------------------------------------------------
// 14) Optional rules (mines and chain)
// ---------------------------------------------------------------------------

/// A mine resolution result (§10.12). The Dervish player rolls 1d10 when a
/// British gunboat enters a mined hex.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MineResult {
    /// Roll 1-4: no effect.
    NoEffect,
    /// Roll 5-7: engines lost; gunboat drifts two hexes per turn with the
    /// current for the rest of the game; guns/Maxims still work unless out
    /// of range.
    EnginesLost,
    /// Roll 8-10: gunboat sunk.
    Sunk,
}

impl MineResult {
    pub fn from_roll(roll: DieRoll) -> Self {
        use crate::DieRoll::*;
        match roll {
            One | Two | Three | Four => MineResult::NoEffect,
            Five | Six | Seven => MineResult::EnginesLost,
            Eight | Nine | Ten => MineResult::Sunk,
        }
    }
}

/// A river-mine placement record (§10.11). Two mines maximum, may not share
/// a hex, must be south of a given hexrow.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct MinePlacement {
    pub hex: HexCoord,
    pub triggered: bool,
}

/// A British gunboat stopped on a mine, awaiting the Dervish player's roll
/// (§10.12: "the Dervish player must order it to stop as it has struck a
/// mine. The Dervish player then resolves the effect").
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct StruckMine {
    pub gunboat: UnitId,
    pub hex: HexCoord,
}

/// A river-chain placement record (§10.21). Up to four contiguous river
/// hexes south of the Khor Shambat hexrow. Cleared by either: (a) an
/// infantry/cavalry unit spending a full turn adjacent on either bank, or
/// (b) artillery scoring 3+ on the Combat Results Table (§10.23).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChainPlacement {
    pub hexes: Vec<HexCoord>,
    pub sunk: bool,
}

//! Friendlies transport across the Nile and the optional river
//! obstacles (mines, chain).

use serde::{Deserialize, Serialize};

use crate::{DieRoll, UnitId};
use omdurman_types::HexCoord;

// ---------------------------------------------------------------------------
// 13) Loading / transport of the "Friendlies" brigade across the Nile
// ---------------------------------------------------------------------------

/// The action payload for `GameEffect::FriendliesTransport` -- what the
/// player wants to do with the Friendlies unit this turn (§5.21).
///
/// The manual does not cap how many Friendlies may load onto a single gunboat
/// (a hex has six neighbours, so multiple units can be adjacent).  The code
/// tracks each unit–gunboat pair independently.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FriendliesAction {
    /// Turn N (the load turn): unit and gunboat started adjacent; unit
    /// loads onto (stacks with) the gunboat.
    Load { unit: UnitId, gunboat: UnitId },
    /// Turn N+1: the gunboat may move to any Nile hex (`to`) adjacent to a
    /// west-bank hex.
    Cross {
        unit: UnitId,
        gunboat: UnitId,
        to: HexCoord,
    },
    /// Turn N+2: the unit may disembark, paying normal terrain cost for the
    /// first hex entered.
    Disembark { unit: UnitId, gunboat: UnitId },
}

/// The transport state stored on `GameState` (§5.21). Modelled as a state
/// machine so the engine can enforce that disembarking can only happen on the
/// third turn.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransportState {
    /// Turn N (the load turn): unit and gunboat started adjacent; unit
    /// loads onto (stacks with) the gunboat.
    Loaded { unit: UnitId, gunboat: UnitId },
    /// Turn N+1: the gunboat may move to any Nile hex (`to`) adjacent to a
    /// west-bank hex.
    Crossing {
        unit: UnitId,
        gunboat: UnitId,
        to: HexCoord,
    },
    /// Turn N+2: the unit may disembark, paying normal terrain cost for the
    /// first hex entered.
    ReadyToDisembark { unit: UnitId, gunboat: UnitId },
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

/// A river-chain placement record (§10.21). Up to four contiguous river
/// hexes south of the Khor Shambat hexrow. Cleared by either: (a) an
/// infantry/cavalry unit spending a full turn adjacent on either bank, or
/// (b) artillery scoring 3+ on the Combat Results Table (§10.23).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChainPlacement {
    pub hexes: Vec<HexCoord>,
    pub sunk: bool,
}

//! Scalar value types: `value_enum!` factor/allowance/die enums and the
//! tuple-struct wrappers (movement points, hex distance, victory points,
//! game-turn index), plus night-turn movement halving.

use serde::{Deserialize, Serialize};

use crate::combat_results_table::FireFactorRow;
use omdurman_types::{DayNight, Player};

// ---------------------------------------------------------------------------
// 1) Scalar wrapper types (tuple structs -- never type aliases)
// ---------------------------------------------------------------------------

value_enum! {
    /// A unit's fire-combat factor as printed on the counter (rulebook §6.11).
    /// Every possible value from the annotated counter set is a named variant.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
    pub enum FireFactor {
        One = 1,
        Three = 3,
        Four = 4,
        Five = 5,
        Six = 6,
        Eight = 8,
        Nine = 9,
        Ten = 10,
    }
}

impl FireFactor {
    /// Sum multiple fire factors and return the corresponding Combat Results Table row (rulebook §6.11).
    pub fn sum_to_row<'a>(factors: impl IntoIterator<Item = &'a FireFactor>) -> FireFactorRow {
        let total: u16 = factors.into_iter().map(|f| f.value()).sum();
        crate::combat_results_table::FireFactorRow::from_total(total)
    }
}

value_enum! {
    /// A unit's melee factor as printed on the counter (rulebook §7.1).
    /// Every possible value from the annotated counter set is a named variant.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
    pub enum MeleeFactor {
        One = 1,
        Three = 3,
        Five = 5,
        Six = 6,
        Seven = 7,
    }
}

impl MeleeFactor {
    /// Sum multiple melee factors (rulebook §7.1).
    pub fn sum<'a>(factors: impl IntoIterator<Item = &'a MeleeFactor>) -> u16 {
        factors.into_iter().map(|f| f.value()).sum()
    }
}

value_enum! {
    /// A unit's land movement allowance or a terrain-entry's movement cost
    /// (rulebook §5.11). Every possible value from the annotated counter set
    /// is a named variant.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum MovementAllowance {
        /// Immobile (forts, wrecked gunboats).
        Immobile = 0,
        One = 1,
        Two = 2,
        Three = 3,
        /// Intermediate value from night halving (not printed on any counter).
        Four = 4,
        /// Intermediate value from night halving (not printed on any counter).
        Five = 5,
        /// Intermediate value from night halving (not printed on any counter).
        Six = 6,
        Seven = 7,
        Eight = 8,
        Nine = 9,
        Ten = 10,
        Twelve = 12,
        Fifteen = 15,
        Sixteen = 16,
        Eighteen = 18,
    }
}

impl MovementAllowance {
    /// Night movement allowance = halved (round down) (rulebook §8.1, §5.11).
    pub fn halve(self) -> Self {
        let v = self.value() / 2;
        MovementAllowance::try_from(v).expect("halved value always a named variant")
    }
}

impl std::fmt::Display for MovementAllowance {
    /// Display as the numeric value of the movement allowance (rulebook §5.11).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

/// Movement points spent or remaining within a single phase (rulebook §5).
#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default,
)]
pub struct MovementPoints(pub(crate) i16);

impl std::fmt::Display for MovementPoints {
    /// Display as the number of movement points (rulebook §5.11).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl MovementPoints {
    pub fn new(value: i16) -> Self {
        Self(value)
    }
    pub fn value(self) -> i16 {
        self.0
    }
}

/// A distance measured in hexes (range to target, length of a retreat, ...)
/// (rulebook §6.22, §7.5).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct HexDistance(pub(crate) u16);

impl HexDistance {
    pub fn new(value: u16) -> Self {
        Self(value)
    }
    pub fn value(self) -> u16 {
        self.0
    }
}

value_enum! {
    /// A ten-sided die roll (1-10) as an exhaustive enum (rulebook §6, §7, §8, §10).
    ///
    /// Every legal die value is a named variant so that match arms are
    /// exhaustive at compile time.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
    pub enum DieRoll {
        One = 1,
        Two = 2,
        Three = 3,
        Four = 4,
        Five = 5,
        Six = 6,
        Seven = 7,
        Eight = 8,
        Nine = 9,
        Ten = 10,
    }
}

impl DieRoll {
    /// Apply a signed die-roll modifier, clamping to the legal 1-10 range
    /// (rulebook §6.24, §7.7). A method -- not an `Add<i16>` impl -- so the
    /// clamping is explicit at every call site instead of silent via `+`.
    pub fn apply_modifier(self, modifier: i16) -> DieRoll {
        // `saturating_add`, not `+`: `modifier` is an unconstrained `i16` on a
        // `pub` method, and `FireAttack::net_modifier` sums an unbounded list
        // of modifiers (one of which, `FireModifier::Terrain`, carries an
        // arbitrary `i16` off the wire). A plain add overflows for large
        // magnitudes -- Kani found it. Saturation then clamps to 1..=10, which
        // is the same answer for every in-range modifier.
        let v = (self.value() as i16).saturating_add(modifier).clamp(1, 10) as u16;
        // The clamp guarantees 1..=10 and `DieRoll` covers exactly that range,
        // so this branch is total and the fallback is unreachable.
        DieRoll::try_from(v).unwrap_or(DieRoll::Ten)
    }
}

/// Victory points (signed because they accumulate on either side of a ledger)
/// (rulebook §9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct VictoryPoints(pub(crate) i32);

impl VictoryPoints {
    pub fn new(value: i32) -> Self {
        Self(value)
    }
    pub fn value(self) -> i32 {
        self.0
    }
}

/// One-based Game Turn index (1, 2, ... up to the scenario length) (rulebook §4).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct GameTurnIndex(pub(crate) u8);

impl GameTurnIndex {
    pub fn new(value: u8) -> Self {
        Self(value)
    }
    pub fn value(self) -> u8 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// 16) Convenience: movement computation under night-turn halving
// ---------------------------------------------------------------------------

/// Apply night-turn movement halving for Anglo-Egyptian units (§8.1): all
/// Anglo-Egyptian movement allowances are halved (round down).
pub fn effective_movement_at_night(
    allowance: MovementAllowance,
    player: Player,
    day_night: DayNight,
) -> MovementAllowance {
    if day_night == DayNight::Night && player == Player::AngloEgyptian {
        allowance.halve()
    } else {
        allowance
    }
}

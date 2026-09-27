//! Fire and melee combat types: attacks, modifiers, results, and the
//! engineer demolition target.

use serde::{Deserialize, Serialize};

use crate::combat_results_table::FireFactorRow;
use crate::{Phase, UnitId};
use omdurman_types::{HexCoord, HexsideRef, Player};

// ---------------------------------------------------------------------------
// 9) Fire combat: attacks, modifiers, results
// ---------------------------------------------------------------------------

/// Every distinct die-roll modifier the rulebook recognises during a fire
/// attack. Encoding each as a variant means the engine cannot silently
/// double-apply a bonus and can audit any combat after the fact.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FireModifier {
    /// +1 to all Anglo-Egyptian *direct* fire (§6.24).
    AngloEgyptianDirectFire,
    /// +1 brigade integrity, applied only if all four battalions fire at
    /// the same enemy-occupied hex (§5.54, §6.24).
    BrigadeIntegrity,
    /// Negative modifier from the Terrain Effects Chart applied to the
    /// defender's hex (§6.23).
    Terrain(i16),
    /// -2 thorn-hedge defensive modifier (§9.231).
    ZaribaThornHedge,
    /// -4 trench defensive modifier (§9.232). Only applies vs. "entrenched"
    /// units (those Nile-side of the trench hexside).
    ZaribaTrenchEntrenched,
}

impl FireModifier {
    /// Return the numeric die-roll modifier for this bonus/penalty (rulebook §6.24, §5.54, §6.23, §9.231, §9.232).
    pub fn die_modifier(self) -> i16 {
        match self {
            FireModifier::AngloEgyptianDirectFire | FireModifier::BrigadeIntegrity => 1,
            FireModifier::Terrain(n) => n,
            FireModifier::ZaribaThornHedge => -2,
            FireModifier::ZaribaTrenchEntrenched => -4,
        }
    }
}

/// What kind of fire is being resolved -- direct fire, howitzer fire, or a
/// Maxim's second fire. The variant constrains which sub-phase the attack
/// may legally occur in.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FireKind {
    Direct,
    /// Howitzer fire (§6.64): range 4-10, ignores LOS, hit on impact roll
    /// 7-10, otherwise scatters per the Howitzer Fire Scattergram.
    Howitzer,
    /// A Maxim's second fire (§6.42) -- same as direct, but tagged so the
    /// engine can enforce "once in direct + once in second-fire = at most
    /// twice total" (§6.14).
    MaximSecondFire,
}

/// A fire attack as the rules engine sees it: who fires, at what hex, in
/// what sub-phase, with which kind of fire, with what total factor and what
/// modifiers (rulebook §6).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FireAttack {
    pub firing_player: Player,
    pub phase: Phase,
    pub kind: FireKind,
    pub firers: Vec<UnitId>,
    pub target_hex: HexCoord,
    /// Combat Results Table factor row (computed from summed unit fire factors before
    /// range-band application; the engine re-derives the effective row
    /// per-unit at resolution time).
    pub factor_row: FireFactorRow,
    pub modifiers: Vec<FireModifier>,
}

impl FireAttack {
    /// Sum of all fire modifiers applied to this attack (rulebook §6.24).
    pub fn net_modifier(&self) -> i16 {
        // Saturating fold rather than `sum()`: the modifier list is unbounded
        // and arrives over the network, so a plain sum can overflow `i16`.
        self.modifiers
            .iter()
            .map(|m| m.die_modifier())
            .fold(0i16, |acc, m| acc.saturating_add(m))
    }
}

/// A single row of the Combat Results Table, expressed as an enum (rulebook §6.22, §7.7).
/// Notation from the reference table at the foot of the manual:
///
/// * `D` -- half (round up) of units in the target hex disrupted
/// * `1`/`2`/`3`/`4`/`5` -- that many units in the target hex eliminated
/// * `--` -- no effect
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CombatResult {
    NoEffect,
    Disrupt,
    Eliminate(u8),
}

// ---------------------------------------------------------------------------
// 10) Melee combat
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MeleeModifier {
    /// +2 to all Dervish melee rolls (§7.7).
    DervishStandard,
    /// +1 to all Anglo-Egyptian melee rolls (§7.7).
    AngloEgyptianStandard,
    /// Inverted to -2 when Dervish units melee-attack across a trench into
    /// an entrenched defender (§9.232).
    DervishVsTrenchedDefender,
}

impl MeleeModifier {
    /// Return the numeric die-roll modifier for this melee bonus/penalty (rulebook §7.7, §9.232).
    pub fn die_modifier(self) -> i16 {
        match self {
            MeleeModifier::DervishStandard => 2,
            MeleeModifier::AngloEgyptianStandard => 1,
            MeleeModifier::DervishVsTrenchedDefender => -2,
        }
    }
}

/// A melee attack: simultaneous, both sides roll on the Combat Results Table (§7.3, §7.7).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MeleeAttack {
    pub attacker_player: Player,
    pub attacker_hex: HexCoord,
    pub defender_hex: HexCoord,
    pub attackers: Vec<UnitId>,
    pub defenders: Vec<UnitId>,
    pub attacker_modifiers: Vec<MeleeModifier>,
    pub defender_modifiers: Vec<MeleeModifier>,
}

// ---------------------------------------------------------------------------
// 11) Special engineer / demolition actions
// ---------------------------------------------------------------------------

/// The Royal Engineers' two demolition targets (§6.53). The Engineers spend
/// the entire turn adjacent to the target (no offensive fire or melee that
/// turn) and the target is removed at end-of-turn unless the Engineers were
/// disrupted or driven off.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DemolitionTarget {
    Fort(UnitId),
    WallHexside(HexsideRef),
}

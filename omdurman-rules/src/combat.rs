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
///
/// | Fire modifier                              | Die roll | Rule                              |
/// | :----------------------------------------- | -------: | :-------------------------------- |
/// | All Anglo-Egyptian direct fire attacks     |       +1 | §6.24                             |
/// | Anglo-Egyptian brigade integrity           |       +1 | §5.54, §6.24                      |
/// | Target hex is Huts / a named Building      |  −1 / −3 | §6.23, [`crate::terrain_chart`] |
/// | Fire crosses a crest / city wall hexside   |  −1 / −4 | §6.23, [`crate::terrain_chart`] |
/// | Target stacked inside a friendly fort      |       −3 | §6.54                             |
/// | Target behind the Zariba thorn hedge       |       −2 | §9.231                            |
/// | Entrenched target behind the Zariba trench |       −4 | §9.232                            |
///
/// The terrain, hexside and fort rows all travel as [`FireModifier::Terrain`];
/// the modified roll is clamped to 1–10 ([`crate::DieRoll::apply_modifier`]).
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
    /// §6.54/§6.62: at a hex holding an enemy fort, whether the fire is
    /// aimed at the fort itself (artillery only, destroyed on a result of 2
    /// or more with one of its occupants) rather than at the units stacked
    /// inside it (any weapon, the fort's −3 deducted from the roll).
    /// Meaningless elsewhere.
    #[serde(default)]
    pub at_fort: bool,
    /// Combat Results Table factor row (computed from summed unit fire factors before
    /// range-band application; the engine re-derives the effective row
    /// per-unit at resolution time).
    pub factor_row: FireFactorRow,
    pub modifiers: Vec<FireModifier>,
    /// Named gunboats whose Maxim guns join this attack (§2.32, §6.42): a
    /// second weapon on the counter, fired on the Maxims line, once in each
    /// fire subphase, independently of the gunboat's artillery/howitzer
    /// factor (which fires as one of `firers`). A gunboat may appear in both
    /// lists -- its two weapons combined at one hex (§6.14).
    #[serde(default)]
    pub gunboat_maxims: Vec<UnitId>,
}

/// Which of a counter's weapons fires (§2.3). Every armed counter has its
/// printed fire factor (`Main`); a new-type (named) gunboat also carries
/// Maxim guns beside its artillery -- the "6×2" of "5·6×2·12/18" (§2.32).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FireMount {
    /// The counter's printed fire factor: rifles, spears, a battery, a
    /// Maxim battery, a gunboat's artillery (fired as howitzer fire in the
    /// second subphase, §6.64).
    Main,
    /// A named gunboat's Maxim guns, fired on the Maxims line once in each
    /// fire subphase (§6.42).
    GunboatMaxims,
}

/// One weapon firing in an attack: a counter and which of its weapons
/// (§6.14 combines any number of them at one hex).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Shot {
    pub unit: UnitId,
    pub mount: FireMount,
}

impl FireAttack {
    /// The weapons firing in this attack: each of `firers` with its main
    /// weapon, then each gunboat in `gunboat_maxims` with its Maxims.
    pub fn shots(&self) -> Vec<Shot> {
        let main = self.firers.iter().map(|&unit| Shot {
            unit,
            mount: FireMount::Main,
        });
        let maxims = self.gunboat_maxims.iter().map(|&unit| Shot {
            unit,
            mount: FireMount::GunboatMaxims,
        });
        main.chain(maxims).collect()
    }

    /// Every unit firing in this attack, once each (a gunboat firing both
    /// weapons is listed once).
    pub fn all_firing_units(&self) -> Vec<UnitId> {
        let mut units: Vec<UnitId> = Vec::new();
        for shot in self.shots() {
            if !units.contains(&shot.unit) {
                units.push(shot.unit);
            }
        }
        units
    }

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

/// Which units a `D` result disrupts, drawn at random with the dice
/// (§CombatResults: "D* = ½ (round up) of the units in the target hex are
/// disrupted"; the rulebook does not say who picks them). The acting peer
/// rolls it into the effect like any die, so every peer disrupts the same
/// units on replay.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct DisruptionDraw(pub u32);

impl DisruptionDraw {
    /// The (up to) `n` of `candidates` this draw picks, in pick order. The
    /// draw is read as a mixed-radix number: each digit picks one of the
    /// candidates still left, so every ordered pick is reachable.
    pub fn pick(self, candidates: &[UnitId], n: usize) -> Vec<UnitId> {
        let mut left = candidates.to_vec();
        let mut rest = self.0;
        let mut picked = Vec::with_capacity(n.min(left.len()));
        while picked.len() < n && !left.is_empty() {
            let len = left.len() as u32;
            picked.push(left.remove((rest % len) as usize));
            rest /= len;
        }
        picked
    }

    /// Two independent draws out of one: a melee's two simultaneous results
    /// (§7.3) each pick their own victims, from the low and the high half.
    pub fn split(self) -> (Self, Self) {
        (Self(self.0 & 0xFFFF), Self(self.0 >> 16))
    }
}

// ---------------------------------------------------------------------------
// 10) Melee combat
// ---------------------------------------------------------------------------

/// Every distinct die-roll modifier the rulebook recognises during a melee
/// (§7.7). Each side rolls on the Combat Results Table with its own
/// modifier; the modified roll is clamped to 1–10.
///
/// | Melee modifier                                                    | Die roll | Rule   |
/// | :---------------------------------------------------------------- | -------: | :----- |
/// | Dervish units                                                     |       +2 | §7.7   |
/// | Anglo-Egyptian units                                              |       +1 | §7.7   |
/// | Anglo-Egyptian "Friendlies" (melee with the Dervish modifier)     |       +2 | §6.52  |
/// | Dervish attacking an entrenched defender across the Zariba trench |       −2 | §9.232 |
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MeleeModifier {
    /// +2 to all Dervish melee rolls (§7.7).
    DervishStandard,
    /// +1 to all Anglo-Egyptian melee rolls (§7.7).
    AngloEgyptianStandard,
    /// Inverted to -2 when Dervish units melee-attack across a trench into
    /// an entrenched defender (§9.232).
    DervishVsTrenchedDefender,
    /// +2: the Anglo-Egyptian "Friendlies" melee with the Dervish melee
    /// modifier (§6.52) -- in place of the +1 standard when every
    /// Anglo-Egyptian unit in the melee is a Friendlies unit.
    FriendliesStandard,
}

impl MeleeModifier {
    /// Return the numeric die-roll modifier for this melee bonus/penalty (rulebook §6.52, §7.7, §9.232).
    pub fn die_modifier(self) -> i16 {
        match self {
            MeleeModifier::DervishStandard => 2,
            MeleeModifier::AngloEgyptianStandard => 1,
            MeleeModifier::DervishVsTrenchedDefender => -2,
            MeleeModifier::FriendliesStandard => 2,
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

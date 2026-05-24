//! Rule-level types for "REMEMBER GORDON!" — The Battle of Omdurman.
//!
//! Every fact stated in the printed rulebook (Phoenix Enterprises, Ltd., 1982)
//! that affects a legal move, a legal stack, a fire/melee resolution, or a
//! victory tally is encoded here as an enum, a tuple struct, or a struct so
//! that the rules engine can statically prove which states are reachable.
//!
//! Tuple structs are used for every quantitative value (factors, points,
//! ranges, die rolls, turn indices) so that values are not accidentally
//! interchanged: a melee factor cannot be added to a movement allowance, a
//! die roll is not a fire factor, etc.

use serde::{Deserialize, Serialize};

use omdurman_types::{Faction, HexCoord};

pub mod effects;
pub mod tables;

// ---------------------------------------------------------------------------
// 1) Scalar wrapper types (tuple structs — never type aliases)
// ---------------------------------------------------------------------------

/// A unit's fire-combat factor as printed on the counter (rulebook §6.11).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FireFactor(pub u16);

/// A unit's melee factor as printed on the counter (rulebook §7.1).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MeleeFactor(pub u16);

/// A unit's movement allowance in movement points (rulebook §5.11).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MovementAllowance(pub u16);

/// Movement points spent or remaining within a single phase.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MovementPoints(pub i16);

/// A distance measured in hexes (range to target, length of a retreat, …).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct HexDistance(pub u16);

/// A raw ten-sided die roll in `1..=10`.
///
/// `DieRoll::new` is the only constructor; it clamps to the legal range so
/// that "less than 1 → 1, more than 10 → 10" (rulebook reference table) is
/// enforced at the type boundary.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DieRoll(u8);

impl DieRoll {
    pub fn new(raw: i16) -> Self {
        DieRoll(raw.clamp(1, 10) as u8)
    }
    pub fn get(self) -> u8 {
        self.0
    }
}

/// A die-roll modifier (positive or negative) accumulated during combat.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DieModifier(pub i16);

impl DieModifier {
    pub fn apply(self, roll: DieRoll) -> DieRoll {
        DieRoll::new(roll.0 as i16 + self.0)
    }
}

/// Victory points (signed because they accumulate on either side of a ledger).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct VictoryPoints(pub i32);

/// One-based Game Turn index (1, 2, … up to the scenario length).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct GameTurnIndex(pub u8);

/// One-based hex-row index (used to express set-up restrictions like
/// "south of the E–W hexrow in which the Khor Shambat empties into the Nile").
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct HexRow(pub i32);

// ---------------------------------------------------------------------------
// 2) Players and turn sequence
// ---------------------------------------------------------------------------

/// The two sides referenced everywhere in the rulebook. Distinct from
/// [`crate::Faction`] which also includes `Independent`; rule resolution
/// always picks between exactly these two.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum Player {
    AngloEgyptian,
    Dervish,
}

impl Player {
    pub fn opponent(self) -> Player {
        match self {
            Player::AngloEgyptian => Player::Dervish,
            Player::Dervish => Player::AngloEgyptian,
        }
    }
}

/// A game turn is either a day turn or a night turn; night turns halve all
/// Anglo-Egyptian movement and all fire ranges, and forbid howitzer fire
/// (rulebook §8.1).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DayNight {
    Day,
    Night,
}

/// Identifies the player-turn currently being resolved.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct PlayerTurn {
    pub turn: GameTurnIndex,
    pub day_night: DayNight,
    pub active: Player,
}

/// The fine-grained phase within a player-turn (rulebook §4).
///
/// Fire-combat phase is broken down so that the legality of every fire is
/// statically checkable: e.g. a howitzer fire can only resolve inside the
/// `MaximSecondAndHowitzer` sub-phase, defensive fire only in `DefensiveFire`,
/// etc.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Movement,
    DefensiveFire(FireSubPhase),
    OffensiveFire(FireSubPhase),
    Melee,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FireSubPhase {
    /// Direct fire (§6.41). Both sides participate in this sub-phase.
    DirectFire,
    /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
    MaximSecondAndHowitzer,
}

// ---------------------------------------------------------------------------
// 3) Scenarios
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
pub enum Scenario {
    /// 9.1 — 22 game turns, 6:00 am Sept 1 → 8:00 am Sept 3.
    Campaign,
    /// 9.2 — 4 game turns, 6:00 am → 12:00 noon Sept 2.
    Historical,
    /// 9.3 — variable length, see victory conditions.
    FallOfKhartoum,
}

/// Optional rules — only legal in the campaign game, and at most one of the
/// two should be in play (rulebook §10).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OptionalRule {
    RiverMines,
    RiverChain,
}

// ---------------------------------------------------------------------------
// 4) Unit identity — tribes, brigades, named leaders, classes
// ---------------------------------------------------------------------------

/// Dervish tribal/sub-faction identity. Drives the colour-based stacking
/// restriction (§5.52) and the leader→troops command match (§5.53).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum DervishTribe {
    Baggara,
    Jaalin,
    Danagla,
    Kehena,
    Degheim,
    Hadendowa,
    Mulazmin,
    Jehadia,
    /// The Khalifa's bodyguard (§9.111 — may enter the walled city).
    Taiasha,
    /// East-bank infantry (§9.111).
    IsaZachneih,
}

/// Anglo-Egyptian infantry brigades — designation printed on the counter
/// (§2.3, §5.54). The number is the brigade ordinal as printed, e.g. `2B`.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BrigadeId {
    pub number: u8,
    pub nationality: BrigadeNationality,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum BrigadeNationality {
    /// `xB` — British.
    British,
    /// `xE` — Egyptian.
    Egyptian,
    /// Sudanese brigade (e.g. Maxwell's XIII Sudanese).
    Sudanese,
    /// Native volunteer brigade — special rules apply (§6.52, §5.21).
    Friendlies,
}

/// Battalion ordinal within a brigade. Four battalions form one brigade and
/// brigade integrity requires all four stacked in one hex (§5.54).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BattalionOrdinal(pub u8);

/// Named Dervish leader (§9.112, §9.212). Drives both the colour-stacking
/// match (§5.53) and the historical-scenario set-up hex (§9.212).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum DervishLeader {
    /// "K" set-up hex; controls Taiasha.
    KhalifaAbdullah,
    /// "Y" set-up hex; Baggara/Jaalin command.
    Yakub,
    /// "S" set-up hex.
    Sherif,
    /// "A" set-up hex.
    AliWadHelu,
    /// "O" set-up hex; commands Hadendowa.
    OsmanDigna,
    /// "D" set-up hex; commands Mulazmin & Jehadia.
    SheikElDin,
}

/// Named Anglo-Egyptian leader (§6.51, §9.113). Movement factor only; needed
/// to claim the Mahdi's Tomb (§9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum BritishLeader {
    Kitchener,
    Gatacre,
    Hunter,
    /// Used only in FALL OF KHARTOUM (§9.32, §9.346).
    Gordon,
}

/// Named British gunboat. Five "named" gunboats have howitzer fire (§6.64);
/// "old" gunboats do not.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum GunboatId {
    /// One of the five new-type named gunboats with howitzer capability.
    Named(NamedGunboat),
    /// An old-style gunboat — no howitzer fire (§2.32).
    Old(OldGunboat),
    /// A Dervish gunboat (§9.111, §10.14).
    DervishGunboat(u8),
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum NamedGunboat {
    Sultan,
    Melik,
    Sheik,
    Fateh,
    Naser,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum OldGunboat {
    LordKitchener,
    Tamai,
    Metemmeh,
}

// ---------------------------------------------------------------------------
// 5) Unit kinds and weapons
// ---------------------------------------------------------------------------

/// What this unit *is* — drives every special-capability branch in the rules.
///
/// Notice that `Infantry`, `Cavalry`, `Camel`, and `DervishLeaderUnit` are the
/// only kinds that may *attack* in melee (§7.4) — this enum is what lets the
/// engine prove the constraint.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnitKind {
    /// Foot infantry. Includes Anglo-Egyptian infantry, "Friendlies",
    /// Royal Engineers, and Dervish foot tribes.
    Infantry,
    Cavalry,
    Camel,
    Artillery,
    Maxim,
    Gunboat,
    /// Permanent emplacement — may not move once placed (§5.25).
    Fort,
    /// Dervish leader: has fire/melee/movement factors and may melee attack.
    DervishLeaderUnit,
    /// Anglo-Egyptian leader: movement only (§6.51).
    BritishLeaderUnit,
}

impl UnitKind {
    /// Rulebook §7.4 — only infantry, cavalry, camel and Dervish leaders may
    /// melee *attack*. All others (except gunboats) may melee *defend* (§7.1).
    pub fn may_melee_attack(self) -> bool {
        matches!(
            self,
            UnitKind::Infantry | UnitKind::Cavalry | UnitKind::Camel | UnitKind::DervishLeaderUnit
        )
    }

    /// Gunboats neither attack nor are attacked in melee (§7.1).
    pub fn may_be_melee_attacked(self) -> bool {
        !matches!(self, UnitKind::Gunboat)
    }

    /// Cavalry and camel units may retreat two hexes from an infantry melee
    /// attack (§7.5).
    pub fn may_retreat_before_melee(self) -> bool {
        matches!(self, UnitKind::Cavalry | UnitKind::Camel)
    }
}

/// Weapon class — chooses which line of the Range Effects Table applies and
/// which special artillery rules (§6.6) are available. Spelled out as an
/// enum so a "spear" unit cannot accidentally fire on the "Howitzer" line.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum WeaponClass {
    /// Dervish spears and swords — no ranged fire at all.
    Melee,
    /// Rifles line. Anglo-Egyptian infantry, Dervish Jehadia/Danagla/Isa
    /// Zachneih, and the "Friendlies" all fire here (§2.31, §2.32, §6.52).
    Rifles,
    /// "Maxims" line; fires twice per turn (§6.42).
    Maxims,
    /// "Artillery" line. Used by Dervish artillery, forts, all gunboats
    /// (old + new), and Anglo-Egyptian artillery.
    Artillery,
    /// "Howitzer" line — only the five named British gunboats (§6.64).
    /// No howitzer fire allowed at night (§8.1, §6.64).
    Howitzer,
}

/// A range band on the Range Effects Table — how the printed fire factor is
/// multiplied at a given distance (§6.22).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RangeBand {
    Tripled,
    Doubled,
    Normal,
    Halved,
    OutOfRange,
}

impl RangeBand {
    /// Apply this band to a printed fire factor, rounding down per unit and
    /// never reducing below 1 by *halving* (§6.16).  `OutOfRange` returns 0.
    pub fn apply(self, factor: FireFactor) -> FireFactor {
        let raw = factor.0;
        let scaled = match self {
            RangeBand::Tripled => raw.saturating_mul(3),
            RangeBand::Doubled => raw.saturating_mul(2),
            RangeBand::Normal => raw,
            // halve, round down, floor at 1 (§6.16)
            RangeBand::Halved => (raw / 2).max(1),
            RangeBand::OutOfRange => 0,
        };
        FireFactor(scaled)
    }
}

/// Gunboats have two movement allowances — the smaller upstream and the
/// larger downstream (§5.24).  Combined movement is permitted but as soon as
/// the gunboat moves one hex upstream its upstream allowance caps the rest of
/// the turn.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct GunboatMovement {
    pub upstream: MovementAllowance,
    pub downstream: MovementAllowance,
}

// ---------------------------------------------------------------------------
// 6) Unit definition and runtime state
// ---------------------------------------------------------------------------

/// Stable identifier for a counter on the map. Opaque tuple-struct so it
/// can't be confused with a position or a brigade number.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UnitId(pub u32);

/// The owner-side identity of a unit: which faction, plus the optional
/// tribe / brigade / named-leader identity (whichever applies).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnitIdentity {
    DervishTribal {
        tribe: DervishTribe,
    },
    DervishLeader(DervishLeader),
    DervishArtillery,
    DervishFort,
    DervishGunboat(GunboatId),
    AngloEgyptianInfantry {
        brigade: BrigadeId,
        battalion: BattalionOrdinal,
    },
    AngloEgyptianCavalry,
    AngloEgyptianCamelCorps,
    AngloEgyptianArtillery,
    AngloEgyptianMaxim,
    AngloEgyptianGunboat(GunboatId),
    AngloEgyptianLeader(BritishLeader),
    /// The Royal Engineers (§6.53) — a *specific* unit, not a class, so we
    /// model it explicitly.
    RoyalEngineers,
}

impl UnitIdentity {
    pub fn owner(&self) -> Player {
        match self {
            UnitIdentity::DervishTribal { .. }
            | UnitIdentity::DervishLeader(_)
            | UnitIdentity::DervishArtillery
            | UnitIdentity::DervishFort
            | UnitIdentity::DervishGunboat(_) => Player::Dervish,
            _ => Player::AngloEgyptian,
        }
    }

    pub fn faction(&self) -> Faction {
        match self.owner() {
            Player::Dervish => Faction::Dervish,
            Player::AngloEgyptian => Faction::BritishEgyptian,
        }
    }

    /// "Friendlies" units obey several special rules (§5.21, §5.23, §6.52,
    /// §9.14 victory conditions).
    pub fn is_friendlies(&self) -> bool {
        matches!(
            self,
            UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    nationality: BrigadeNationality::Friendlies,
                    ..
                },
                ..
            }
        )
    }
}

/// The printed combat profile of a single counter. Optional factors are
/// `None` only where the rulebook leaves the value off the counter (e.g.
/// British leaders print only movement; gunboats print no melee value).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct UnitProfile {
    pub kind: UnitKind,
    pub identity: UnitIdentity,
    pub weapon: WeaponClass,
    pub fire: Option<FireFactor>,
    pub melee: Option<MeleeFactor>,
    pub movement: UnitMovement,
}

/// Movement allowance — uniform for land units, split for gunboats.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnitMovement {
    Land(MovementAllowance),
    Gunboat(GunboatMovement),
    /// Forts may not move once placed (§5.25).
    Immobile,
}

/// Volatile per-turn state of a unit — disrupted, loaded onto a gunboat,
/// constructing the Zariba, demolishing a target, etc.
///
/// Multiple state flags can be in effect at once (e.g. a unit may be both
/// loaded and disrupted), so `UnitState` is a struct of orthogonal fields
/// rather than one big enum.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct UnitState {
    /// Reference table: "Disrupted units: no ZOC; may not move; may not fire
    /// offensively or defensively; may not melee; are turned face up at the
    /// end of the owning player's turn."
    pub disrupted: bool,
    /// `Some(gunboat)` after a "Friendlies" unit loads onto a gunboat (§5.21).
    pub loaded_on: Option<UnitId>,
    /// Set while the unit is building Zariba hexsides — neither offensive
    /// fire nor melee allowed that turn (§5.3).
    pub constructing_zariba: bool,
    /// Set when the Royal Engineers are committed to a demolition this turn
    /// (§6.53) — neither offensive fire nor melee allowed that turn.
    pub demolishing: bool,
}

impl UnitState {
    /// A disrupted unit may not move, fire, or melee (reference notes).
    pub fn may_act(self) -> bool {
        !self.disrupted
    }

    /// A unit that began construction this turn may not fire offensively or
    /// melee (§5.3, §6.53).
    pub fn may_attack_this_turn(self) -> bool {
        !self.disrupted && !self.constructing_zariba && !self.demolishing
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct UnitPlacement {
    pub id: UnitId,
    pub position: HexCoord,
    pub profile: UnitProfile,
    pub state: UnitState,
}

// ---------------------------------------------------------------------------
// 7) Map topology — hexside kinds and terrain modifiers
// ---------------------------------------------------------------------------

/// Hex-side classifications referenced by the movement, line-of-sight, ZOC,
/// melee, and advance-after-combat rules.
///
/// Note: ordinary "clear" hexsides are represented by the *absence* of a
/// `HexsideKind` annotation in the game map, not by a variant here.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HexsideKind {
    /// City wall (Khartoum, walled-city of Omdurman). Blocks LOS, blocks
    /// movement except across gates/breaches (§5.23), blocks ZOC into the
    /// city across this hexside (§5.44), blocks melee (§7.2), blocks
    /// advance-after-combat (§6.82).
    Wall,
    /// Gate hexside in a wall. ZOC extends *out of* the walled city through
    /// gates but not into it (§5.44). Melee may be made through a gate
    /// (§7.2).
    Gate,
    /// Breach in a wall (placed when artillery or the Royal Engineers
    /// breach the wall — §6.63, §6.53). ZOC extends both ways; LOS no
    /// longer blocked across the hexside.
    Breach,
    /// Khor — gully/wadi. ZOCs do not extend across (§5.44); advance after
    /// combat may not cross (§6.82).
    Khor,
    /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
    ZaribaThornHedge,
    /// Historical-scenario trench segment of the Zariba (§9.232).
    ZaribaTrench,
}

impl HexsideKind {
    pub fn blocks_los_normally(self) -> bool {
        matches!(self, HexsideKind::Wall)
    }

    pub fn blocks_zoc_outbound(self) -> bool {
        // A wall blocks ZOC from outside into the walled-city, but ZOC does
        // leave the walled city across walls (§5.44). For outbound from a
        // walled-city hex, walls do not block.
        matches!(self, HexsideKind::Khor)
    }

    pub fn blocks_zoc_inbound(self) -> bool {
        matches!(
            self,
            HexsideKind::Khor | HexsideKind::Wall | HexsideKind::Gate
        )
    }

    pub fn blocks_melee(self) -> bool {
        matches!(self, HexsideKind::Wall | HexsideKind::ZaribaThornHedge)
    }

    pub fn blocks_advance_after_combat(self) -> bool {
        matches!(
            self,
            HexsideKind::Wall | HexsideKind::Khor | HexsideKind::ZaribaThornHedge
        )
    }
}

// ---------------------------------------------------------------------------
// 8) Zones of control, stacking, brigade integrity
// ---------------------------------------------------------------------------

/// Why a unit can or cannot exert/receive ZOC into a given adjacent hex.
/// Used by the engine when answering "is this hex in an enemy ZOC?".
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZocReason {
    /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
    /// leader (§5.41) projects ZOC into each of its six adjacent hexes.
    Normal,
    /// Gunboats project ZOC only against enemy gunboats (§5.41).
    GunboatVsGunboat,
    /// Forts project ZOC out of, but not into, an empty fort (§5.44, §6.54).
    Fort,
    /// Walled-city ZOC: extends out through walls and gates but not in,
    /// across a breach in both directions (§5.44).
    WalledCity,
    /// Zariba hexside ZOC behaviour in the historical scenario / when the
    /// Zariba is constructed (§5.44).
    Zariba,
}

/// Errors returned when a candidate stack would violate stacking rules.
#[derive(thiserror::Error, Clone, Copy, PartialEq, Eq, Debug)]
pub enum StackingError {
    /// "No more than four units may occupy a hex" (§5.51), excluding leaders
    /// and the gunboat exception.
    #[error("hex stack exceeds the four-unit limit")]
    OverLimit,
    /// "Gunboats may not stack with any other unit" (§5.51, exception §5.21).
    #[error("gunboats may not stack with non-gunboat units")]
    GunboatStack,
    /// "Units of different Dervish tribes may not stack together" (§5.52).
    #[error("Dervish units of different tribes may not stack")]
    DervishTribeMix,
    /// "If Dervish leaders elect to stack, they may only stack with units of
    /// their command (i.e. colour)" (§5.53).
    #[error("Dervish leader may only stack with units of their own command")]
    DervishLeaderCommandMismatch,
}

/// Brigade-integrity status of a stack (§5.54). Carries the brigade if the
/// stack contains all four battalions of a single Anglo-Egyptian brigade.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BrigadeIntegrity {
    None,
    Integrated(BrigadeId),
}

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
    /// defender's hex (§6.23). Carries the magnitude so the engine can show
    /// "-2 palm grove" etc. without an external table lookup.
    Terrain(DieModifier),
    /// −2 thorn-hedge defensive modifier (§9.231).
    ZaribaThornHedge,
    /// −4 trench defensive modifier (§9.232). Only applies vs. "entrenched"
    /// units (those Nile-side of the trench hexside).
    ZaribaTrenchEntrenched,
}

impl FireModifier {
    pub fn die_modifier(self) -> DieModifier {
        match self {
            FireModifier::AngloEgyptianDirectFire | FireModifier::BrigadeIntegrity => {
                DieModifier(1)
            }
            FireModifier::Terrain(m) => m,
            FireModifier::ZaribaThornHedge => DieModifier(-2),
            FireModifier::ZaribaTrenchEntrenched => DieModifier(-4),
        }
    }
}

/// What kind of fire is being resolved — direct fire, howitzer fire, or a
/// Maxim's second fire. The variant constrains which sub-phase the attack
/// may legally occur in.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FireKind {
    Direct,
    /// Howitzer fire (§6.64): range 4–10, ignores LOS, hit on impact roll
    /// 7–10, otherwise scatters per the Howitzer Fire Scattergram.
    Howitzer,
    /// A Maxim's second fire (§6.42) — same as direct, but tagged so the
    /// engine can enforce "once in direct + once in second-fire = at most
    /// twice total" (§6.14).
    MaximSecondFire,
}

/// A fire attack as the rules engine sees it: who fires, at what hex, in
/// what sub-phase, with which kind of fire, with what total factor and what
/// modifiers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FireAttack {
    pub firing_player: Player,
    pub phase: Phase,
    pub kind: FireKind,
    pub firers: Vec<UnitId>,
    pub target_hex: HexCoord,
    pub total_factor: FireFactor,
    pub modifiers: Vec<FireModifier>,
}

impl FireAttack {
    pub fn net_modifier(&self) -> DieModifier {
        DieModifier(self.modifiers.iter().map(|m| m.die_modifier().0).sum())
    }
}

/// A single row of the Combat Results Table, expressed as an enum.
/// Notation from the reference table at the foot of the manual:
///
/// * `D` — half (round up) of units in the target hex disrupted
/// * `1`/`2`/`3`/`4`/`5` — that many units in the target hex eliminated
/// * `—` — no effect
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CombatResult {
    NoEffect,
    Disrupt,
    Eliminate(u8),
}

/// Special artillery resolution results (§6.61–§6.63). These are separate
/// from the standard CombatResult because the thresholds differ.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArtillerySpecialResult {
    /// Sink a gunboat — requires CRT result ≥ 3 (§6.61).
    GunboatSunk,
    /// Eliminate a fort — requires CRT result ≥ 2 (§6.62). One occupant of
    /// the fort is also eliminated (§6.62).
    FortDestroyed,
    /// Breach a wall — requires CRT result ≥ 2 (§6.63). One enemy unit
    /// adjacent to the wall is eliminated.
    WallBreached,
    /// Otherwise: a miss.
    Miss,
}

/// Howitzer fire requires two die rolls: the CRT roll and the impact-hex
/// roll on the Howitzer Fire Scattergram (§6.64).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct HowitzerResolution {
    pub crt_roll: DieRoll,
    pub impact_roll: DieRoll,
}

impl HowitzerResolution {
    /// The designated target hex is hit on impact roll 7–10 (§6.64).
    pub fn hit_target_hex(self) -> bool {
        self.impact_roll.get() >= 7
    }
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
    /// Inverted to −2 when Dervish units melee-attack across a trench into
    /// an entrenched defender (§9.232).
    DervishVsTrenchedDefender,
}

impl MeleeModifier {
    pub fn die_modifier(self) -> DieModifier {
        match self {
            MeleeModifier::DervishStandard => DieModifier(2),
            MeleeModifier::AngloEgyptianStandard => DieModifier(1),
            MeleeModifier::DervishVsTrenchedDefender => DieModifier(-2),
        }
    }
}

/// A melee attack: simultaneous, both sides roll on the CRT (§7.3, §7.7).
#[derive(Serialize, Deserialize, Clone, Debug)]
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
// 11) Advance-after-combat eligibility
// ---------------------------------------------------------------------------

/// A unit is eligible to advance after combat if it participated in the
/// attack, was adjacent to the vacated hex, did not violate the "no
/// advance for artillery" / "no advance across wall (except gate/breach) /
/// no advance across khor" restrictions (§6.82, §7.6).
#[derive(thiserror::Error, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdvanceAfterCombatError {
    #[error("unit did not participate in the attack")]
    DidNotParticipate,
    #[error("unit was not adjacent to the vacated hex")]
    NotAdjacent,
    #[error("artillery may not advance after combat")]
    Artillery,
    #[error("may not advance across a wall hexside except at a gate or breach")]
    AcrossWall,
    #[error("may not advance after combat across a khor")]
    AcrossKhor,
    #[error("may not advance after combat across a thorn-hedge hexside")]
    AcrossZaribaThornHedge,
    #[error("only attacking units may advance after combat")]
    NotAttacker,
}

// ---------------------------------------------------------------------------
// 12) Special engineer / demolition actions
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

/// Reference to a specific hex-side on the map.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HexsideRef {
    pub a: HexCoord,
    pub b: HexCoord,
}

// ---------------------------------------------------------------------------
// 13) Loading / transport of the "Friendlies" brigade across the Nile
// ---------------------------------------------------------------------------

/// The three-step gunboat transport sequence for the "Friendlies" (§5.21).
/// Modelled as a state machine so the engine can enforce that disembarking
/// can only happen on the third turn.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FriendliesTransport {
    /// Turn N (the load turn): unit and gunboat started adjacent; unit
    /// loads onto (stacks with) the gunboat.
    Loaded { unit: UnitId, gunboat: UnitId },
    /// Turn N+1: the gunboat may move to any Nile hex adjacent to a
    /// west-bank hex.
    Crossing { unit: UnitId, gunboat: UnitId },
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
    /// Roll 1–4: no effect.
    NoEffect,
    /// Roll 5–7: engines lost; gunboat drifts two hexes per turn with the
    /// current for the rest of the game; guns/Maxims still work unless out
    /// of range.
    EnginesLost,
    /// Roll 8–10: gunboat sunk.
    Sunk,
}

impl MineResult {
    pub fn from_roll(roll: DieRoll) -> Self {
        match roll.get() {
            1..=4 => MineResult::NoEffect,
            5..=7 => MineResult::EnginesLost,
            _ => MineResult::Sunk,
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
/// (b) artillery scoring 3+ on the CRT (§10.23).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChainPlacement {
    pub hexes: Vec<HexCoord>,
    pub sunk: bool,
}

// ---------------------------------------------------------------------------
// 15) Victory ledger
// ---------------------------------------------------------------------------

/// Every distinct VP source the rulebook enumerates (§9.14). Each variant
/// carries its point value as a method so the table cannot drift between
/// the manual and the engine.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum VpSource {
    // ----- Anglo-Egyptian player receives:
    /// Mahdi's Tomb control at conclusion of play (§9.14).
    MahdisTomb,
    /// 1 pt — eliminating the Isa Zachneih unit (§9.14).
    IsaZachneihEliminated,
    /// 10 pts — eliminating the Khalifa Abdullah (§9.14).
    KhalifaEliminated,
    /// 1 pt — each Dervish unit eliminated (gunboats, artillery, other
    /// leaders included). Forts elimination is worth 0 pts (§9.14).
    DervishUnitEliminated,
    // ----- Dervish player receives:
    /// 10 pts — each British leader eliminated (§9.14).
    BritishLeaderEliminated,
    /// 10 pts — each British gunboat sunk (§9.14).
    BritishGunboatSunk,
    /// 1 pt — each "Friendlies" unit eliminated on the east bank (§9.14).
    FriendliesEastBankEliminated,
    /// 3 pts — each "Friendlies" unit eliminated on the west bank (§9.14).
    FriendliesWestBankEliminated,
    /// 3 pts — each Anglo-Egyptian land unit eliminated (§9.14).
    AngloEgyptianLandUnitEliminated,
}

impl VpSource {
    /// VP awarded to `who_scores()`.
    pub fn points(self) -> VictoryPoints {
        match self {
            VpSource::MahdisTomb => VictoryPoints(25),
            VpSource::IsaZachneihEliminated => VictoryPoints(1),
            VpSource::KhalifaEliminated => VictoryPoints(10),
            VpSource::DervishUnitEliminated => VictoryPoints(1),
            VpSource::BritishLeaderEliminated => VictoryPoints(10),
            VpSource::BritishGunboatSunk => VictoryPoints(10),
            VpSource::FriendliesEastBankEliminated => VictoryPoints(1),
            VpSource::FriendliesWestBankEliminated => VictoryPoints(3),
            VpSource::AngloEgyptianLandUnitEliminated => VictoryPoints(3),
        }
    }

    pub fn who_scores(self) -> Player {
        match self {
            VpSource::MahdisTomb
            | VpSource::IsaZachneihEliminated
            | VpSource::KhalifaEliminated
            | VpSource::DervishUnitEliminated => Player::AngloEgyptian,
            VpSource::BritishLeaderEliminated
            | VpSource::BritishGunboatSunk
            | VpSource::FriendliesEastBankEliminated
            | VpSource::FriendliesWestBankEliminated
            | VpSource::AngloEgyptianLandUnitEliminated => Player::Dervish,
        }
    }
}

/// Cumulative victory ledger for one scenario.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VictoryLedger {
    pub events: Vec<VpEvent>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct VpEvent {
    pub turn: GameTurnIndex,
    pub source: VpSource,
}

impl VictoryLedger {
    pub fn total_for(&self, player: Player) -> VictoryPoints {
        VictoryPoints(
            self.events
                .iter()
                .filter(|e| e.source.who_scores() == player)
                .map(|e| e.source.points().0)
                .sum(),
        )
    }

    /// Net superiority: positive = Anglo-Egyptian ahead, negative = Dervish ahead.
    pub fn superiority(&self) -> VictoryPoints {
        VictoryPoints(self.total_for(Player::AngloEgyptian).0 - self.total_for(Player::Dervish).0)
    }
}

/// Campaign-game victory levels (§9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CampaignVictoryLevel {
    Draw,
    Marginal(Player),
    Tactical(Player),
    Decisive(Player),
}

impl CampaignVictoryLevel {
    /// Assign a level from the net superiority (§9.14).
    pub fn from_superiority(s: VictoryPoints) -> Self {
        let net = s.0;
        // Positive → Anglo-Egyptian thresholds: 15/30/50
        // Negative → Dervish thresholds: 10/20/30 (rulebook §9.14 table)
        if net >= 50 {
            CampaignVictoryLevel::Decisive(Player::AngloEgyptian)
        } else if net >= 30 {
            CampaignVictoryLevel::Tactical(Player::AngloEgyptian)
        } else if net >= 15 {
            CampaignVictoryLevel::Marginal(Player::AngloEgyptian)
        } else if net >= 1 {
            // 1–14 = Draw for the Anglo-Egyptian side
            CampaignVictoryLevel::Draw
        } else if net >= -9 {
            // 1–9 Dervish superiority = Draw
            CampaignVictoryLevel::Draw
        } else if net >= -19 {
            CampaignVictoryLevel::Marginal(Player::Dervish)
        } else if net >= -29 {
            CampaignVictoryLevel::Tactical(Player::Dervish)
        } else {
            CampaignVictoryLevel::Decisive(Player::Dervish)
        }
    }
}

/// Historical-scenario victory levels (§9.24). Numeric so subtraction works
/// per the rulebook example ("decisive worth 5 minus strategic worth 4 = 1,
/// draw").
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum HistoricalVictoryLevel {
    Draw = 1,
    Marginal = 2,
    Tactical = 3,
    Strategic = 4,
    Decisive = 5,
}

// ---------------------------------------------------------------------------
// 16) Convenience: range computation under night-turn halving
// ---------------------------------------------------------------------------

/// Apply night-turn range halving (§8.1): "all fire ranges are halved for
/// both sides (round down, but range 1 stays range 1)."
pub fn effective_range_at_night(range: HexDistance) -> HexDistance {
    if range.0 <= 1 {
        range
    } else {
        HexDistance(range.0 / 2)
    }
}

/// Apply night-turn movement halving for Anglo-Egyptian units (§8.1): all
/// Anglo-Egyptian movement allowances are halved (round down).
pub fn effective_movement_at_night(
    allowance: MovementAllowance,
    player: Player,
    day_night: DayNight,
) -> MovementAllowance {
    if day_night == DayNight::Night && player == Player::AngloEgyptian {
        MovementAllowance(allowance.0 / 2)
    } else {
        allowance
    }
}

// ---------------------------------------------------------------------------
// Tests — every numeric rule above must round-trip a manual example.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_roll_clamps() {
        assert_eq!(DieRoll::new(-5).get(), 1);
        assert_eq!(DieRoll::new(0).get(), 1);
        assert_eq!(DieRoll::new(11).get(), 10);
        assert_eq!(DieRoll::new(7).get(), 7);
    }

    #[test]
    fn die_modifier_applies_and_clamps() {
        let r = DieRoll::new(5);
        assert_eq!(DieModifier(3).apply(r).get(), 8);
        assert_eq!(DieModifier(-9).apply(r).get(), 1);
        assert_eq!(DieModifier(99).apply(r).get(), 10);
    }

    #[test]
    fn range_band_halving_floors_at_one() {
        // §6.16: halving rounds down per unit but never below 1.
        assert_eq!(RangeBand::Halved.apply(FireFactor(1)).0, 1);
        assert_eq!(RangeBand::Halved.apply(FireFactor(9)).0, 4);
        assert_eq!(RangeBand::Halved.apply(FireFactor(0)).0, 1);
    }

    #[test]
    fn range_band_out_of_range_is_zero() {
        assert_eq!(RangeBand::OutOfRange.apply(FireFactor(9)).0, 0);
    }

    #[test]
    fn range_band_multipliers() {
        assert_eq!(RangeBand::Tripled.apply(FireFactor(4)).0, 12);
        assert_eq!(RangeBand::Doubled.apply(FireFactor(4)).0, 8);
        assert_eq!(RangeBand::Normal.apply(FireFactor(4)).0, 4);
    }

    #[test]
    fn night_range_halving_preserves_range_one() {
        // §8.1: range 1 stays range 1 at night.
        assert_eq!(effective_range_at_night(HexDistance(1)).0, 1);
        assert_eq!(effective_range_at_night(HexDistance(2)).0, 1);
        assert_eq!(effective_range_at_night(HexDistance(3)).0, 1);
        assert_eq!(effective_range_at_night(HexDistance(4)).0, 2);
        assert_eq!(effective_range_at_night(HexDistance(7)).0, 3);
    }

    #[test]
    fn night_movement_only_halves_anglo_egyptian() {
        let a = MovementAllowance(8);
        assert_eq!(
            effective_movement_at_night(a, Player::AngloEgyptian, DayNight::Night).0,
            4
        );
        assert_eq!(
            effective_movement_at_night(a, Player::Dervish, DayNight::Night).0,
            8
        );
        assert_eq!(
            effective_movement_at_night(a, Player::AngloEgyptian, DayNight::Day).0,
            8
        );
    }

    #[test]
    fn howitzer_target_hex_hit_band() {
        // §6.64: target hex is hit on impact roll 7–10.
        for roll in 1..=6 {
            assert!(
                !HowitzerResolution {
                    crt_roll: DieRoll::new(5),
                    impact_roll: DieRoll::new(roll),
                }
                .hit_target_hex()
            );
        }
        for roll in 7..=10 {
            assert!(
                HowitzerResolution {
                    crt_roll: DieRoll::new(5),
                    impact_roll: DieRoll::new(roll),
                }
                .hit_target_hex()
            );
        }
    }

    #[test]
    fn mine_result_from_roll() {
        // §10.12.
        assert_eq!(MineResult::from_roll(DieRoll::new(1)), MineResult::NoEffect);
        assert_eq!(MineResult::from_roll(DieRoll::new(4)), MineResult::NoEffect);
        assert_eq!(
            MineResult::from_roll(DieRoll::new(5)),
            MineResult::EnginesLost
        );
        assert_eq!(
            MineResult::from_roll(DieRoll::new(7)),
            MineResult::EnginesLost
        );
        assert_eq!(MineResult::from_roll(DieRoll::new(8)), MineResult::Sunk);
        assert_eq!(MineResult::from_roll(DieRoll::new(10)), MineResult::Sunk);
    }

    #[test]
    fn player_opponent_involutes() {
        assert_eq!(Player::AngloEgyptian.opponent(), Player::Dervish);
        assert_eq!(Player::Dervish.opponent(), Player::AngloEgyptian);
        assert_eq!(
            Player::AngloEgyptian.opponent().opponent(),
            Player::AngloEgyptian
        );
    }

    #[test]
    fn unit_kind_melee_capability() {
        // §7.4.
        assert!(UnitKind::Infantry.may_melee_attack());
        assert!(UnitKind::Cavalry.may_melee_attack());
        assert!(UnitKind::Camel.may_melee_attack());
        assert!(UnitKind::DervishLeaderUnit.may_melee_attack());
        assert!(!UnitKind::Artillery.may_melee_attack());
        assert!(!UnitKind::Maxim.may_melee_attack());
        assert!(!UnitKind::Gunboat.may_melee_attack());
        assert!(!UnitKind::Fort.may_melee_attack());
        assert!(!UnitKind::BritishLeaderUnit.may_melee_attack());

        // §7.1 — gunboats may not be melee attacked.
        assert!(!UnitKind::Gunboat.may_be_melee_attacked());
        assert!(UnitKind::Infantry.may_be_melee_attacked());
        assert!(UnitKind::Fort.may_be_melee_attacked());

        // §7.5.
        assert!(UnitKind::Cavalry.may_retreat_before_melee());
        assert!(UnitKind::Camel.may_retreat_before_melee());
        assert!(!UnitKind::Infantry.may_retreat_before_melee());
    }

    #[test]
    fn disrupted_unit_may_not_act() {
        let s = UnitState {
            disrupted: true,
            ..UnitState::default()
        };
        assert!(!s.may_act());
        assert!(!s.may_attack_this_turn());
    }

    #[test]
    fn constructing_unit_may_not_attack() {
        // §5.3: a unit constructing Zariba "may neither fire offensively nor
        // melee attack during the turn of construction."
        let s = UnitState {
            constructing_zariba: true,
            ..UnitState::default()
        };
        assert!(s.may_act());
        assert!(!s.may_attack_this_turn());
    }

    #[test]
    fn fire_modifiers_compose() {
        // §5.54 + §6.24: a brigade-integrity stack firing direct receives
        // both the +1 direct-fire and the +1 brigade-integrity bonuses.
        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: Phase::OffensiveFire(FireSubPhase::DirectFire),
            kind: FireKind::Direct,
            firers: vec![],
            target_hex: HexCoord::new(0, 0),
            total_factor: FireFactor(16),
            modifiers: vec![
                FireModifier::AngloEgyptianDirectFire,
                FireModifier::BrigadeIntegrity,
                FireModifier::Terrain(DieModifier(-2)),
            ],
        };
        assert_eq!(attack.net_modifier().0, 0);
    }

    #[test]
    fn vp_source_attributes() {
        assert_eq!(VpSource::KhalifaEliminated.points().0, 10);
        assert_eq!(
            VpSource::KhalifaEliminated.who_scores(),
            Player::AngloEgyptian
        );
        assert_eq!(VpSource::BritishLeaderEliminated.points().0, 10);
        assert_eq!(
            VpSource::BritishLeaderEliminated.who_scores(),
            Player::Dervish
        );
        assert_eq!(VpSource::MahdisTomb.points().0, 25);
        assert_eq!(VpSource::FriendliesWestBankEliminated.points().0, 3);
    }

    #[test]
    fn campaign_victory_levels() {
        // §9.14 thresholds.
        // Anglo-Egyptian:
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints(50)),
            CampaignVictoryLevel::Decisive(Player::AngloEgyptian)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints(30)),
            CampaignVictoryLevel::Tactical(Player::AngloEgyptian)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints(15)),
            CampaignVictoryLevel::Marginal(Player::AngloEgyptian)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints(5)),
            CampaignVictoryLevel::Draw
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints(0)),
            CampaignVictoryLevel::Draw
        ));
        // Dervish:
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints(-5)),
            CampaignVictoryLevel::Draw
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints(-15)),
            CampaignVictoryLevel::Marginal(Player::Dervish)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints(-25)),
            CampaignVictoryLevel::Tactical(Player::Dervish)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints(-40)),
            CampaignVictoryLevel::Decisive(Player::Dervish)
        ));
    }

    #[test]
    fn victory_ledger_accumulates() {
        let mut l = VictoryLedger::default();
        l.events.push(VpEvent {
            turn: GameTurnIndex(1),
            source: VpSource::KhalifaEliminated,
        });
        l.events.push(VpEvent {
            turn: GameTurnIndex(2),
            source: VpSource::DervishUnitEliminated,
        });
        l.events.push(VpEvent {
            turn: GameTurnIndex(2),
            source: VpSource::BritishGunboatSunk,
        });
        assert_eq!(l.total_for(Player::AngloEgyptian).0, 11);
        assert_eq!(l.total_for(Player::Dervish).0, 10);
        assert_eq!(l.superiority().0, 1);
    }

    #[test]
    fn unit_identity_owner_partitions_correctly() {
        let dervish = UnitIdentity::DervishTribal {
            tribe: DervishTribe::Hadendowa,
        };
        assert_eq!(dervish.owner(), Player::Dervish);

        let lancers = UnitIdentity::AngloEgyptianCavalry;
        assert_eq!(lancers.owner(), Player::AngloEgyptian);

        let friendlies = UnitIdentity::AngloEgyptianInfantry {
            brigade: BrigadeId {
                number: 1,
                nationality: BrigadeNationality::Friendlies,
            },
            battalion: BattalionOrdinal(1),
        };
        assert!(friendlies.is_friendlies());

        let british = UnitIdentity::AngloEgyptianInfantry {
            brigade: BrigadeId {
                number: 2,
                nationality: BrigadeNationality::British,
            },
            battalion: BattalionOrdinal(3),
        };
        assert!(!british.is_friendlies());
    }

    #[test]
    fn hexside_kind_classifies_blockers() {
        // §5.44 + §6.82 + §7.2.
        assert!(HexsideKind::Wall.blocks_los_normally());
        assert!(!HexsideKind::Gate.blocks_los_normally());
        assert!(HexsideKind::Wall.blocks_melee());
        assert!(!HexsideKind::Gate.blocks_melee());
        assert!(HexsideKind::Khor.blocks_advance_after_combat());
        assert!(!HexsideKind::Breach.blocks_advance_after_combat());
        assert!(HexsideKind::ZaribaThornHedge.blocks_melee());
    }
}

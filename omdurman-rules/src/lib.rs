//! Rule-level types for "REMEMBER GORDON!" -- The Battle of Omdurman.
//!
//! Every fact stated in the printed rulebook (Phoenix Enterprises, Ltd., 1982)
//! that affects a legal move, a legal stack, a fire/melee resolution, or a
//! victory tally is encoded here as an enum, a tuple struct, or a struct so
//! that the rules engine can statically prove which states are reachable.
//!
//! Enums are used for every quantitative value that has a fixed, annotated set
//! of possible values so that match arms are exhaustive at compile time.
//! Tuple structs remain only for values with an unbounded range (movement
//! points, hex distances, victory points, game-turn indices).

use serde::{Deserialize, Serialize};

use omdurman_types::{Faction, HexCoord};

pub mod combat_results_table;
pub mod effects;
pub mod howitzer_scatter;
pub mod los_table;
pub mod range_effects;
pub mod terrain_chart;
pub mod turn_track;
use crate::combat_results_table::FireFactorRow;

// ---------------------------------------------------------------------------
// 1) Scalar wrapper types (tuple structs -- never type aliases)
// ---------------------------------------------------------------------------

macro_rules! value_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $value:expr,)+
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $($(#[$variant_meta])* $variant,)+
        }

        impl $name {
            pub fn value(self) -> u16 {
                match self {
                    $(Self::$variant => $value,)+
                }
            }
        }

        impl TryFrom<u16> for $name {
            type Error = ();
            fn try_from(v: u16) -> Result<Self, ()> {
                match v {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(()),
                }
            }
        }
    };
}

value_enum! {
    /// A unit's fire-combat factor as printed on the counter (rulebook §6.11).
    /// Every possible value from the annotated counter set is a named variant.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
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
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
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
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MovementPoints(pub i16);

/// A distance measured in hexes (range to target, length of a retreat, ...)
/// (rulebook §6.22, §7.5).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct HexDistance(pub u16);

impl HexDistance {
    pub fn value(self) -> u16 {
        self.0
    }
}

value_enum! {
    /// A ten-sided die roll (1-10) as an exhaustive enum (rulebook §6, §7, §8, §10).
    ///
    /// Every legal die value is a named variant so that match arms are
    /// exhaustive at compile time.
    ///
    /// Every legal die value is a named variant so that match arms are
    /// exhaustive at compile time.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
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

impl std::ops::Add<i16> for DieRoll {
    type Output = DieRoll;
    fn add(self, rhs: i16) -> DieRoll {
        let v = (self.value() as i16 + rhs).clamp(1, 10) as u16;
        DieRoll::try_from(v).unwrap_or(DieRoll::Ten)
    }
}

/// A die-roll modifier from a single named source (rulebook §6.24, §7.7).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DieModifier {
    #[default]
    Zero,
    PlusOne,
    PlusTwo,
    MinusOne,
    MinusTwo,
    MinusThree,
    MinusFour,
}

impl DieModifier {
    /// Apply this modifier to a die roll (rulebook §6.24, §7.7).
    pub fn apply(self, roll: DieRoll) -> DieRoll {
        roll + match self {
            DieModifier::Zero => 0,
            DieModifier::PlusOne => 1,
            DieModifier::PlusTwo => 2,
            DieModifier::MinusOne => -1,
            DieModifier::MinusTwo => -2,
            DieModifier::MinusThree => -3,
            DieModifier::MinusFour => -4,
        }
    }
}

/// Victory points (signed because they accumulate on either side of a ledger)
/// (rulebook §9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct VictoryPoints(pub i32);

/// One-based Game Turn index (1, 2, ... up to the scenario length) (rulebook §4).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct GameTurnIndex(pub u8);

impl GameTurnIndex {
    pub fn value(self) -> u8 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// 2) Players and turn sequence
// ---------------------------------------------------------------------------

/// The two sides referenced everywhere in the rulebook (rulebook §2). Distinct
/// from [`crate::Faction`] which also includes `Independent`; rule resolution
/// always picks between exactly these two.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum Player {
    AngloEgyptian,
    Dervish,
}

impl Player {
    /// Return the opposing player (rulebook §2).
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

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, strum::Display)]
pub enum Scenario {
    /// 9.1 -- 22 game turns, 6:00 am Sept 1 -> 8:00 am Sept 3.
    #[default]
    Campaign,
    /// 9.2 -- 4 game turns, 6:00 am -> 12:00 noon Sept 2.
    Historical,
    /// 9.3 -- variable length, see victory conditions.
    FallOfKhartoum,
}

/// Optional rules -- only legal in the campaign game, and at most one of the
/// two should be in play (rulebook §10).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OptionalRule {
    RiverMines,
    RiverChain,
}

// ---------------------------------------------------------------------------
// 4) Unit identity -- tribes, brigades, named leaders, classes
// ---------------------------------------------------------------------------

/// Dervish tribal/sub-faction identity. Drives the colour-based stacking
/// restriction (§5.52) and the leader->troops command match (§5.53).
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
    /// The Khalifa's bodyguard (§9.111 -- may enter the walled city).
    Taiasha,
    /// East-bank infantry (§9.111).
    IsaZachneih,
}

/// Anglo-Egyptian infantry brigades -- designation printed on the counter
/// (§2.3, §5.54). The number is the brigade ordinal as printed, e.g. `2B`.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BrigadeId {
    pub number: u8,
    pub nationality: BrigadeNationality,
}

value_enum! {
    /// Battalion ordinal within a brigade. Four battalions form one brigade and
    /// brigade integrity requires all four stacked in one hex (§5.54).
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
    pub enum BattalionOrdinal {
        First = 1,
        Second = 2,
        Third = 3,
        Fourth = 4,
    }
}

impl BattalionOrdinal {
    pub fn index(self) -> usize {
        self.value() as usize - 1
    }
}

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

/// Named British gunboat (rulebook §6.64). Five "named" gunboats have howitzer
/// fire; "old" gunboats do not (rulebook §2.32).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum GunboatId {
    /// One of the five new-type named gunboats with howitzer capability.
    Named(NamedGunboat),
    /// An old-style gunboat -- no howitzer fire (§2.32).
    Old(OldGunboat),
    /// A Dervish gunboat (§9.111, §10.14).
    DervishGunboat(u8),
}

/// The five named gunboats with howitzer capability (rulebook §6.64, §2.32).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum NamedGunboat {
    Sultan,
    Melik,
    Sheik,
    Fateh,
    Naser,
}

/// Old-style gunboat -- no howitzer fire (rulebook §2.32).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum OldGunboat {
    LordKitchener,
    Tamai,
    Metemmeh,
}

// ---------------------------------------------------------------------------
// 5) Unit kinds and weapons
// ---------------------------------------------------------------------------

/// What this unit *is* -- drives every special-capability branch in the rules.
///
/// Notice that `Infantry`, `Cavalry`, `Camel`, and `DervishLeaderUnit` are the
/// only kinds that may *attack* in melee (§7.4) -- this enum is what lets the
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
    /// Permanent emplacement -- may not move once placed (§5.25).
    Fort,
    /// Dervish leader: has fire/melee/movement factors and may melee attack.
    DervishLeaderUnit,
    /// Anglo-Egyptian leader: movement only (§6.51).
    BritishLeaderUnit,
}

impl UnitKind {
    /// Rulebook §7.4 -- only infantry, cavalry, camel and Dervish leaders may
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

/// Weapon class -- chooses which line of the Range Effects Table applies and
/// which special artillery rules (§6.6) are available. Spelled out as an
/// enum so a "spear" unit cannot accidentally fire on the "Howitzer" line.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum WeaponClass {
    /// Dervish spears and swords -- no ranged fire at all.
    Melee,
    /// Rifles line. Anglo-Egyptian infantry, Dervish Jehadia/Danagla/Isa
    /// Zachneih, and the "Friendlies" all fire here (§2.31, §2.32, §6.52).
    Rifles,
    /// "Maxims" line; fires twice per turn (§6.42).
    Maxims,
    /// "Artillery" line. Used by Dervish artillery, forts, all gunboats
    /// (old + new), and Anglo-Egyptian artillery.
    Artillery,
    /// "Howitzer" line -- only the five named British gunboats (§6.64).
    /// No howitzer fire allowed at night (§8.1, §6.64).
    Howitzer,
}

/// Hex distance expressed as a range band on the firing tables (1-10 hexes)
/// (rulebook §6.22). Distances beyond 10 hexes are out of range for all weapons.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Range {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

/// A range band on the Range Effects Table -- how the printed fire factor is
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
    pub fn apply(self, raw: u16) -> u16 {
        match self {
            RangeBand::Tripled => raw.saturating_mul(3),
            RangeBand::Doubled => raw.saturating_mul(2),
            RangeBand::Normal => raw,
            // halve, round down, floor at 1 (§6.16)
            RangeBand::Halved => (raw / 2).max(1),
            RangeBand::OutOfRange => 0,
        }
    }

    /// Whether the target is within firing range (anything but `OutOfRange`).
    pub fn in_range(self) -> bool {
        !matches!(self, RangeBand::OutOfRange)
    }
}

/// Gunboats have two movement allowances -- the smaller upstream and the
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

mod unit_id;
pub use unit_id::*;

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
    /// The Royal Engineers (§6.53) -- a *specific* unit, not a class, so we
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

    /// The brigade designation, if this is an Anglo-Egyptian infantry unit
    /// (§5.54). `None` for every other identity.
    pub fn brigade(&self) -> Option<BrigadeId> {
        match self {
            UnitIdentity::AngloEgyptianInfantry { brigade, .. } => Some(*brigade),
            _ => None,
        }
    }

    /// The battalion ordinal within the brigade, if this is an Anglo-Egyptian
    /// infantry unit (§5.54). `None` for every other identity.
    pub fn battalion(&self) -> Option<BattalionOrdinal> {
        match self {
            UnitIdentity::AngloEgyptianInfantry { battalion, .. } => Some(*battalion),
            _ => None,
        }
    }
}

/// Whether a set of firing units forms a brigade with integrity (§5.54): all
/// four distinct battalions (1-4) of one Anglo-Egyptian brigade present. Used
/// to grant the +1 brigade-integrity direct-fire modifier when they all fire
/// at the same hex.
pub fn brigade_integrity(identities: &[UnitIdentity]) -> BrigadeIntegrity {
    let Some(brigade) = identities.first().and_then(|i| i.brigade()) else {
        return BrigadeIntegrity::None;
    };
    // Every firer must belong to the same brigade...
    if !identities.iter().all(|i| i.brigade() == Some(brigade)) {
        return BrigadeIntegrity::None;
    }
    // ...and all four battalion ordinals must be present.
    let mut seen = [false; 4];
    for id in identities {
        if let Some(b) = id.battalion() {
            seen[b.index()] = true;
        }
    }
    if seen.iter().all(|&b| b) {
        BrigadeIntegrity::Integrated(brigade)
    } else {
        BrigadeIntegrity::None
    }
}

/// The printed combat profile of a single counter (rulebook §2.3, §6.11, §7.1,
/// §5.11, §5.24). Optional factors are `None` only where the rulebook leaves the
/// value off the counter (e.g. British leaders print only movement; gunboats
/// print no melee value).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct UnitProfile {
    pub kind: UnitKind,
    pub identity: UnitIdentity,
    pub weapon: WeaponClass,
    pub fire: Option<FireFactor>,
    pub melee: Option<MeleeFactor>,
    pub movement: UnitMovement,
}

/// Movement allowance -- uniform for land units, split for gunboats (rulebook §5.11, §5.24, §5.25).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnitMovement {
    Land(MovementAllowance),
    Gunboat(GunboatMovement),
    /// Forts may not move once placed (§5.25).
    Immobile,
}

/// Volatile per-turn state of a unit -- disrupted, loaded onto a gunboat,
/// constructing the Zariba, demolishing a target, etc. (rulebook §5, §6).
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
    /// Set while the unit is building Zariba hexsides -- neither offensive
    /// fire nor melee allowed that turn (§5.3).
    pub constructing_zariba: bool,
    /// Set when the Royal Engineers are committed to a demolition this turn
    /// (§6.53) -- neither offensive fire nor melee allowed that turn.
    pub demolishing: bool,
}

impl UnitState {
    /// A disrupted unit may not move, fire, or melee (rulebook §5, reference notes).
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
// 7) Map topology -- hexside kinds and terrain modifiers
// ---------------------------------------------------------------------------

/// Hex-side classifications referenced by the movement, line-of-sight, ZOC,
/// melee, and advance-after-combat rules.
///
/// Note: ordinary "clear" hexsides are represented by the *absence* of a
/// `HexsideKind` annotation in the game map, not by a variant here.
// `HexsideKind` and `HexsideRef` are defined in `omdurman-types` so the map
// crate can store per-edge hexside data; re-exported here for the rules layer.
pub use omdurman_types::{BrigadeNationality, HexsideKind, HexsideRef};

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
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
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
        self.modifiers.iter().map(|m| m.die_modifier()).sum()
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

/// Howitzer fire requires two die rolls: the Combat Results Table roll and the impact-hex
/// roll on the Howitzer Fire Scattergram (§6.64).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct HowitzerResolution {
    pub combat_results_table_roll: DieRoll,
    pub impact_roll: DieRoll,
}

impl HowitzerResolution {
    /// The designated target hex is hit on impact roll 7-10 (§6.64).
    pub fn hit_target_hex(self) -> bool {
        use DieRoll::*;
        matches!(self.impact_roll, Seven | Eight | Nine | Ten)
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
        use DieRoll::*;
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
    /// 1 pt -- eliminating the Isa Zachneih unit (§9.14).
    IsaZachneihEliminated,
    /// 10 pts -- eliminating the Khalifa Abdullah (§9.14).
    KhalifaEliminated,
    /// 1 pt -- each Dervish unit eliminated (gunboats, artillery, other
    /// leaders included). Forts elimination is worth 0 pts (§9.14).
    DervishUnitEliminated,
    // ----- Dervish player receives:
    /// 10 pts -- each British leader eliminated (§9.14).
    BritishLeaderEliminated,
    /// 10 pts -- each British gunboat sunk (§9.14).
    BritishGunboatSunk,
    /// 1 pt -- each "Friendlies" unit eliminated on the east bank (§9.14).
    FriendliesEastBankEliminated,
    /// 3 pts -- each "Friendlies" unit eliminated on the west bank (§9.14).
    FriendliesWestBankEliminated,
    /// 3 pts -- each Anglo-Egyptian land unit eliminated (§9.14).
    AngloEgyptianLandUnitEliminated,
}

impl VpSource {
    /// VP awarded to `who_scores()` (rulebook §9.14).
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

    /// Which player receives these victory points (rulebook §9.14).
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

/// Cumulative victory ledger for one scenario (rulebook §9.14).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VictoryLedger {
    pub events: Vec<VpEvent>,
}

/// A single victory-point scoring event (rulebook §9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct VpEvent {
    pub turn: GameTurnIndex,
    pub source: VpSource,
}

impl VictoryLedger {
    /// Total victory points earned by a given player (rulebook §9.14).
    pub fn total_for(&self, player: Player) -> VictoryPoints {
        VictoryPoints(
            self.events
                .iter()
                .filter(|e| e.source.who_scores() == player)
                .map(|e| e.source.points().0)
                .sum(),
        )
    }

    /// Net superiority: positive = Anglo-Egyptian ahead, negative = Dervish ahead
    /// (rulebook §9.14).
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
        // Positive -> Anglo-Egyptian thresholds: 15/30/50
        // Negative -> Dervish thresholds: 10/20/30 (rulebook §9.14 table)
        if net >= 50 {
            CampaignVictoryLevel::Decisive(Player::AngloEgyptian)
        } else if net >= 30 {
            CampaignVictoryLevel::Tactical(Player::AngloEgyptian)
        } else if net >= 15 {
            CampaignVictoryLevel::Marginal(Player::AngloEgyptian)
        } else if net >= 1 {
            // 1-14 = Draw for the Anglo-Egyptian side
            CampaignVictoryLevel::Draw
        } else if net >= -9 {
            // 1-9 Dervish superiority = Draw
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
        allowance.halve()
    } else {
        allowance
    }
}

// ---------------------------------------------------------------------------
// Tests -- every numeric rule above must round-trip a manual example.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_roll_from_u8_clamps() {
        assert_eq!(DieRoll::try_from(0u16.clamp(1, 10)).unwrap_or(DieRoll::Ten).value(), 1);
        assert_eq!(DieRoll::try_from(11u16.clamp(1, 10)).unwrap_or(DieRoll::Ten).value(), 10);
        assert_eq!(DieRoll::try_from(7u16).unwrap(), DieRoll::Seven);
    }

    #[test]
    fn die_roll_add_clamps() {
        let r = DieRoll::Five;
        assert_eq!((r + 3).value(), 8);
        assert_eq!((r + (-9)).value(), 1);
        assert_eq!((r + 99).value(), 10);
    }

    #[test]
    fn die_modifier_value_and_apply() {
        assert_eq!(DieModifier::Zero.apply(DieRoll::Five), DieRoll::Five);
        assert_eq!(DieModifier::PlusOne.apply(DieRoll::Five), DieRoll::Six);
        assert_eq!(DieModifier::MinusThree.apply(DieRoll::Five), DieRoll::Two);
        assert_eq!(DieModifier::PlusTwo.apply(DieRoll::Five), DieRoll::Seven);
    }

    #[test]
    fn range_band_halving_floors_at_one() {
        // §6.16: halving rounds down per unit but never below 1.
        assert_eq!(RangeBand::Halved.apply(1), 1);
        assert_eq!(RangeBand::Halved.apply(9), 4);
        assert_eq!(RangeBand::Halved.apply(0), 1);
    }

    #[test]
    fn range_band_out_of_range_is_zero() {
        assert_eq!(RangeBand::OutOfRange.apply(9), 0);
    }

    #[test]
    fn range_band_multipliers() {
        assert_eq!(RangeBand::Tripled.apply(4), 12);
        assert_eq!(RangeBand::Doubled.apply(4), 8);
        assert_eq!(RangeBand::Normal.apply(4), 4);
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
    fn night_movement_halves_round_down() {
        // §8.1: movement halved (round down).
        assert_eq!(
            effective_movement_at_night(MovementAllowance::Three, Player::AngloEgyptian, DayNight::Night).value(),
            1
        );
        assert_eq!(
            effective_movement_at_night(MovementAllowance::Five, Player::AngloEgyptian, DayNight::Night).value(),
            2
        );
        assert_eq!(
            effective_movement_at_night(MovementAllowance::One, Player::AngloEgyptian, DayNight::Night).value(),
            0
        );
    }

    #[test]
    fn night_movement_only_halves_anglo_egyptian() {
        let a = MovementAllowance::Eight;
        assert_eq!(
            effective_movement_at_night(a, Player::AngloEgyptian, DayNight::Night).value(),
            4
        );
        assert_eq!(
            effective_movement_at_night(a, Player::Dervish, DayNight::Night).value(),
            8
        );
        assert_eq!(
            effective_movement_at_night(a, Player::AngloEgyptian, DayNight::Day).value(),
            8
        );
    }

    #[test]
    fn howitzer_target_hex_hit_band() {
        // §6.64: target hex is hit on impact roll 7-10.
        for roll in 1u8..=6 {
            assert!(
                !HowitzerResolution {
                    combat_results_table_roll: DieRoll::Five,
                    impact_roll: DieRoll::try_from(roll as u16).unwrap(),
                }
                .hit_target_hex()
            );
        }
        for roll in 7u8..=10 {
            assert!(
                HowitzerResolution {
                    combat_results_table_roll: DieRoll::Five,
                    impact_roll: DieRoll::try_from(roll as u16).unwrap(),
                }
                .hit_target_hex()
            );
        }
    }

    #[test]
    fn mine_result_from_roll() {
        // §10.12.
        assert_eq!(MineResult::from_roll(DieRoll::One), MineResult::NoEffect);
        assert_eq!(MineResult::from_roll(DieRoll::Four), MineResult::NoEffect);
        assert_eq!(
            MineResult::from_roll(DieRoll::Five),
            MineResult::EnginesLost
        );
        assert_eq!(
            MineResult::from_roll(DieRoll::Seven),
            MineResult::EnginesLost
        );
        assert_eq!(MineResult::from_roll(DieRoll::Eight), MineResult::Sunk);
        assert_eq!(MineResult::from_roll(DieRoll::Ten), MineResult::Sunk);
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

        // §7.1 -- gunboats may not be melee attacked.
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
            factor_row: FireFactorRow::Row16to20,
            modifiers: vec![
                FireModifier::AngloEgyptianDirectFire,
                FireModifier::BrigadeIntegrity,
                FireModifier::Terrain(-2),
            ],
        };
        assert_eq!(attack.net_modifier(), 0);
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
            battalion: BattalionOrdinal::First,
        };
        assert!(friendlies.is_friendlies());

        let british = UnitIdentity::AngloEgyptianInfantry {
            brigade: BrigadeId {
                number: 2,
                nationality: BrigadeNationality::British,
            },
            battalion: BattalionOrdinal::Third,
        };
        assert!(!british.is_friendlies());
    }

    #[test]
    fn hexside_kind_classifies_blockers() {
        // §5.44 + §6.82 + §7.2.
        assert!(HexsideKind::Wall.blocks_los());
        assert!(!HexsideKind::Gate.blocks_los());
        assert!(HexsideKind::Wall.blocks_melee());
        assert!(!HexsideKind::Gate.blocks_melee());
        assert!(HexsideKind::Khor.blocks_advance_after_combat());
        assert!(!HexsideKind::Breach.blocks_advance_after_combat());
        assert!(HexsideKind::ZaribaThornHedge.blocks_melee());
    }
}

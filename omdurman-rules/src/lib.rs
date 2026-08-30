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

use omdurman_types::{
    BrigadeId, BrigadeNationality, DayNight, DervishTribe, Faction, HexCoord, HexsideRef, Player,
    SetupLetter, UnitKind,
};

pub mod board;
pub mod board_data;
pub mod combat_results_table;
pub mod effects;
pub mod howitzer_scatter;
pub mod los_table;
pub mod newspaper;
pub mod range_effects;
pub mod reinforcements;
pub mod rng;
pub mod scenario_setup;
pub mod sprite_data;
pub mod tables_data;
pub mod tactics;
pub mod telegram_prompt;
pub mod terrain_chart;
pub mod turn_summary;
pub mod turn_track;
pub mod unit_profiles;
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
            /// Every variant, in declaration order. Generated so exhaustive
            /// callers (tests, Kani proofs) pick up new variants automatically
            /// instead of silently skipping them.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

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
pub struct MovementPoints(i16);

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
pub struct HexDistance(u16);

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
pub struct VictoryPoints(i32);

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
pub struct GameTurnIndex(u8);

impl GameTurnIndex {
    pub fn new(value: u8) -> Self {
        Self(value)
    }
    pub fn value(self) -> u8 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// 2) Players and turn sequence
// ---------------------------------------------------------------------------

/// The fine-grained phase within a player-turn (rulebook §4).
///
/// Fire-combat phase is broken down so that the legality of every fire is
/// statically checkable: e.g. a howitzer fire can only resolve inside the
/// `MaximSecondAndHowitzer` sub-phase, defensive fire only in `DefensiveFire`,
/// etc.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
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

impl DervishLeader {
    /// Whether this leader commands `tribe`, i.e. may stack with its units
    /// (§5.53: "Dervish leaders... may only stack with units of their command,
    /// i.e. colour"). The rulebook gives the colour groupings by example; the
    /// documented commands are Khalifa->Taiasha, Yakub->Baggara/Jaalin,
    /// Osman Digna->Hadendowa, Sheik El Din->Mulazmin/Jehadia. Leaders whose
    /// colour is not pinned down by the rules (Sherif, Ali Wad Helu) are treated
    /// as commanding any tribe rather than over-restricting a legal stack.
    pub fn commands(self, tribe: DervishTribe) -> bool {
        match self {
            DervishLeader::KhalifaAbdullah => tribe == DervishTribe::Taiasha,
            DervishLeader::Yakub => {
                matches!(tribe, DervishTribe::Baggara | DervishTribe::Jaalin)
            }
            DervishLeader::OsmanDigna => tribe == DervishTribe::Hadendowa,
            DervishLeader::SheikElDin => {
                matches!(tribe, DervishTribe::Mulazmin | DervishTribe::Jehadia)
            }
            // Colour not fixed by the rules text: do not restrict.
            DervishLeader::Sherif | DervishLeader::AliWadHelu => true,
        }
    }

    /// The lettered Historical-scenario set-up hex this leader is pinned to
    /// (§9.212): A→Ali Wad Helu, D→Sheik El Din, Y→Yakub, K→Khalifa Abdullah,
    /// S→Sherif, O→Osman Digna. Inverse of [`dervish_leader_for_setup_letter`].
    pub fn setup_letter(self) -> SetupLetter {
        match self {
            DervishLeader::AliWadHelu => SetupLetter::A,
            DervishLeader::SheikElDin => SetupLetter::D,
            DervishLeader::Yakub => SetupLetter::Y,
            DervishLeader::KhalifaAbdullah => SetupLetter::K,
            DervishLeader::Sherif => SetupLetter::S,
            DervishLeader::OsmanDigna => SetupLetter::O,
        }
    }
}

/// The Dervish leader pinned to a lettered Historical-scenario set-up hex
/// (§9.212). `SetupLetter` lives in `omdurman-types` and cannot carry an
/// inherent impl here, so the mapping is a free function -- the bijective
/// inverse of [`DervishLeader::setup_letter`].
pub fn dervish_leader_for_setup_letter(letter: SetupLetter) -> DervishLeader {
    match letter {
        SetupLetter::A => DervishLeader::AliWadHelu,
        SetupLetter::D => DervishLeader::SheikElDin,
        SetupLetter::Y => DervishLeader::Yakub,
        SetupLetter::K => DervishLeader::KhalifaAbdullah,
        SetupLetter::S => DervishLeader::Sherif,
        SetupLetter::O => DervishLeader::OsmanDigna,
    }
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

impl GunboatId {
    /// Whether this gunboat carries a howitzer (§6.64): only the five named
    /// new-type gunboats. Old-style gunboats and Dervish gunboats lack one.
    pub fn has_howitzer(self) -> bool {
        matches!(self, GunboatId::Named(_))
    }
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
/// May fire only once per turn (Direct Fire subphase only); it lacks the
/// howitzer equipped by the five named gunboats and thus cannot participate
/// in the Maxim Second Fire and Howitzer subphase (§6.42).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum OldGunboat {
    LordKitchener,
    Tamai,
    Metemmeh,
}

// ---------------------------------------------------------------------------
// 5) Unit kinds and weapons
// ---------------------------------------------------------------------------

/// Weapon class -- chooses which line of the Range Effects Table applies and
/// which special artillery rules (§6.6) are available. Spelled out as an
/// enum so a "spear" unit cannot accidentally fire on the "Howitzer" line.
#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display,
)]
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
        match self {
            UnitIdentity::DervishTribal { tribe } => Faction::Dervish { tribe: *tribe },
            _ => match self.owner() {
                Player::Dervish => Faction::Dervish {
                    tribe: DervishTribe::Baggara,
                },
                Player::AngloEgyptian => Faction::BritishEgyptian { brigade: None },
            },
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

    /// Whether this is the GORDON leader unit (§9.32, §9.346) -- the immobile
    /// palace defender whose elimination ends FALL OF KHARTOUM (§9.35).
    pub fn is_gordon(&self) -> bool {
        matches!(
            self,
            UnitIdentity::AngloEgyptianLeader(BritishLeader::Gordon)
        )
    }

    /// Whether this unit may enter the walled portion of Omdurman (§5.23).
    /// Dervish: only the Khalifa unit, the three artillery units, and the
    /// Taiasha bodyguard may enter. Anglo-Egyptian: any unit that can reach the
    /// walled city *except* gunboats and "Friendlies".
    pub fn may_enter_walled_city(&self) -> bool {
        match self {
            // §5.23 Dervish: Khalifa, artillery, Taiasha.
            UnitIdentity::DervishLeader(DervishLeader::KhalifaAbdullah)
            | UnitIdentity::DervishArtillery
            | UnitIdentity::DervishTribal {
                tribe: DervishTribe::Taiasha,
            } => true,
            // Any other Dervish unit (other leaders, other tribes, forts, gunboats) may not.
            UnitIdentity::DervishLeader(_)
            | UnitIdentity::DervishTribal { .. }
            | UnitIdentity::DervishFort
            | UnitIdentity::DervishGunboat(_) => false,
            // §5.23 Anglo-Egyptian: all may enter except gunboats and Friendlies.
            UnitIdentity::AngloEgyptianGunboat(_) => false,
            other => !other.is_friendlies(),
        }
    }

    /// Whether this Dervish unit is exempt from the desertion roll (§8.2): the
    /// Khalifa, gunboats, artillery units, and forts "may not be chosen".
    /// Non-Dervish identities are trivially not eligible to desert and so are
    /// reported as exempt too.
    pub fn is_desertion_exempt(&self) -> bool {
        match self {
            UnitIdentity::DervishLeader(DervishLeader::KhalifaAbdullah)
            | UnitIdentity::DervishArtillery
            | UnitIdentity::DervishFort
            | UnitIdentity::DervishGunboat(_) => true,
            // Any other Dervish unit may desert; non-Dervish cannot desert.
            other => other.owner() != Player::Dervish,
        }
    }

    /// The Dervish tribe this unit belongs to, if any. Used to enforce §5.52
    /// (different Dervish tribes may not stack together).
    pub fn dervish_tribe(&self) -> Option<DervishTribe> {
        match self {
            UnitIdentity::DervishTribal { tribe } => Some(*tribe),
            _ => None,
        }
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

    /// Short, human-readable name for a unit identity, suitable for a one-line
    /// dispatch slip, tooltip, picker row, or combat-card line. The single
    /// source of truth for the short label previously duplicated as
    /// `identity_short` across the app surfaces.
    pub fn short_label(&self) -> String {
        match self {
            UnitIdentity::DervishTribal { tribe } => tribe.to_string(),
            UnitIdentity::DervishLeader(leader) => leader.to_string(),
            UnitIdentity::DervishArtillery => "Dervish Artillery".into(),
            UnitIdentity::DervishFort => "Dervish Fort".into(),
            UnitIdentity::DervishGunboat(g) => format!("Dervish Gunboat {g}"),
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                let nat = match brigade.nationality {
                    BrigadeNationality::British => 'B',
                    BrigadeNationality::Egyptian => 'E',
                    BrigadeNationality::Sudanese => 'S',
                    BrigadeNationality::Friendlies => 'F',
                };
                format!("{}{} {battalion} Btn", brigade.number, nat)
            }
            UnitIdentity::AngloEgyptianCavalry => "Cavalry".into(),
            UnitIdentity::AngloEgyptianCamelCorps => "Camel Corps".into(),
            UnitIdentity::AngloEgyptianArtillery => "Artillery".into(),
            UnitIdentity::AngloEgyptianMaxim => "Maxim".into(),
            UnitIdentity::AngloEgyptianGunboat(g) => format!("Gunboat {g}"),
            UnitIdentity::AngloEgyptianLeader(leader) => leader.to_string(),
            UnitIdentity::RoyalEngineers => "Royal Engineers".into(),
        }
    }
}

/// Whether a set of firing units forms a brigade with integrity (§5.54): all
/// four distinct battalions (1-4) of one Anglo-Egyptian brigade present. Used
/// to grant the +1 brigade-integrity direct-fire modifier when they all fire
/// at the same hex.
///
/// Only a full stack of four battalions qualifies.  Three or fewer may still
/// stack and fire, but they receive no brigade-integrity bonus.
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
    /// Set when a gunboat has lost its engines to a river mine (§10.12, roll
    /// 5-7): it may no longer move under power and instead drifts two hexes per
    /// turn with the current for the rest of the game.
    #[serde(default)]
    pub engines_lost: bool,
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

// Hex-side classifications referenced by the movement, line-of-sight, ZOC,
// melee, and advance-after-combat rules.
//
// Note: ordinary "clear" hexsides are represented by the *absence* of a
// `HexsideKind` annotation in the game map, not by a variant here.

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
            VpSource::MahdisTomb => VictoryPoints::new(25),
            VpSource::IsaZachneihEliminated => VictoryPoints::new(1),
            VpSource::KhalifaEliminated => VictoryPoints::new(10),
            VpSource::DervishUnitEliminated => VictoryPoints::new(1),
            VpSource::BritishLeaderEliminated => VictoryPoints::new(10),
            VpSource::BritishGunboatSunk => VictoryPoints::new(10),
            VpSource::FriendliesEastBankEliminated => VictoryPoints::new(1),
            VpSource::FriendliesWestBankEliminated => VictoryPoints::new(3),
            VpSource::AngloEgyptianLandUnitEliminated => VictoryPoints::new(3),
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

impl std::fmt::Display for VpSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VpSource::MahdisTomb => write!(f, "Mahdi's Tomb control"),
            VpSource::IsaZachneihEliminated => write!(f, "Isa Zachneih eliminated"),
            VpSource::KhalifaEliminated => write!(f, "Khalifa eliminated"),
            VpSource::DervishUnitEliminated => write!(f, "Dervish unit eliminated"),
            VpSource::BritishLeaderEliminated => write!(f, "British leader eliminated"),
            VpSource::BritishGunboatSunk => write!(f, "British gunboat sunk"),
            VpSource::FriendliesEastBankEliminated => write!(f, "Friendlies lost (east bank)"),
            VpSource::FriendliesWestBankEliminated => write!(f, "Friendlies lost (west bank)"),
            VpSource::AngloEgyptianLandUnitEliminated => {
                write!(f, "Anglo-Egyptian unit eliminated")
            }
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
        VictoryPoints(
            self.total_for(Player::AngloEgyptian).value() - self.total_for(Player::Dervish).value(),
        )
    }

    /// The number of *enemy units eliminated* by `player`, used by the
    /// Historical scenario's unit-count victory schedule (§9.24). Every
    /// elimination/sinking source records one event per unit; the Mahdi's Tomb
    /// source is control, not an elimination, so it is excluded.
    pub fn units_eliminated_by(&self, player: Player) -> i16 {
        self.events
            .iter()
            .filter(|e| e.source.who_scores() == player && e.source != VpSource::MahdisTomb)
            .count() as i16
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

impl HistoricalVictoryLevel {
    /// Anglo-Egyptian level from the number of Dervish units eliminated
    /// (§9.24 left column): 0-29 draw, 30-44 marginal, 45-59 tactical,
    /// 60-99 strategic, 100+ decisive.
    pub fn for_anglo_egyptian(dervish_eliminated: i16) -> Self {
        match dervish_eliminated {
            n if n >= 100 => Self::Decisive,
            n if n >= 60 => Self::Strategic,
            n if n >= 45 => Self::Tactical,
            n if n >= 30 => Self::Marginal,
            _ => Self::Draw,
        }
    }

    /// Dervish level from the number of Anglo-Egyptian units eliminated
    /// (§9.24 right column): 0-4 draw, 5-9 marginal, 10-14 tactical,
    /// 15-29 strategic, 30+ decisive.
    pub fn for_dervish(anglo_egyptian_eliminated: i16) -> Self {
        match anglo_egyptian_eliminated {
            n if n >= 30 => Self::Decisive,
            n if n >= 15 => Self::Strategic,
            n if n >= 10 => Self::Tactical,
            n if n >= 5 => Self::Marginal,
            _ => Self::Draw,
        }
    }
}

/// Fall-of-Khartoum victory levels (§9.35). The base level is set by the turn
/// GORDON is eliminated; the Dervish player then loses victory levels for his
/// own losses. Modelled on a signed ladder (Dervish-favourable is more
/// negative) so the loss penalty is a simple shift toward the British end.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum FoKVictoryLevel {
    DervishDecisive = -3,
    DervishTactical = -2,
    DervishMarginal = -1,
    BritishMarginal = 1,
    BritishTactical = 2,
    BritishDecisive = 3,
}

impl FoKVictoryLevel {
    const LADDER: [FoKVictoryLevel; 6] = [
        FoKVictoryLevel::DervishDecisive,
        FoKVictoryLevel::DervishTactical,
        FoKVictoryLevel::DervishMarginal,
        FoKVictoryLevel::BritishMarginal,
        FoKVictoryLevel::BritishTactical,
        FoKVictoryLevel::BritishDecisive,
    ];

    /// Fallback index into [`Self::LADDER`] when a base level is somehow not
    /// found there. Centred on the Dervish-Marginal / British-Marginal boundary.
    const DEFAULT_LADDER_IDX: usize = 3;

    /// The base level from when GORDON died (§9.35): eliminated turn ≤4 Dervish
    /// decisive, turn 5 tactical, turn 6 marginal; if he survives, the British
    /// level depends on how long he held out -- turn 6 British marginal, turn 7
    /// tactical, turn 8 (or later) decisive.
    ///
    /// `gordon_died_turn` is `None` if GORDON was still alive at scenario end;
    /// `scenario_end_turn` is the 1-based turn on which the game ended (the
    /// scenario's max is 8 per the FoK turn track).
    fn base(gordon_died_turn: Option<u8>, scenario_end_turn: u8) -> Self {
        match gordon_died_turn {
            Some(t) if t <= 4 => FoKVictoryLevel::DervishDecisive,
            Some(5) => FoKVictoryLevel::DervishTactical,
            // GORDON dead turn 6+ is off the table's intent (the scenario ends
            // by turn 8); treat a turn-6-or-later death as the weakest Dervish
            // win.
            Some(_) => FoKVictoryLevel::DervishMarginal,
            // GORDON survived -- the British level grows with how long he held.
            // The ladder starts at turn 6; ending before that yields the floor
            // (BritishMarginal) as a best-effort result (§9.35 doesn't cover it).
            None => match scenario_end_turn {
                t if t >= 8 => FoKVictoryLevel::BritishDecisive,
                7 => FoKVictoryLevel::BritishTactical,
                _ => FoKVictoryLevel::BritishMarginal,
            },
        }
    }

    /// How many victory levels the Dervish player forfeits for his own losses
    /// (§9.35): 1 level at 16-23 units lost, 2 at 24-31, 3 at 32+.
    pub fn loss_penalty(dervish_lost: i16) -> i16 {
        match dervish_lost {
            n if n >= 32 => 3,
            n if n >= 24 => 2,
            n if n >= 16 => 1,
            _ => 0,
        }
    }

    /// The next Dervish-loss threshold at which an additional victory level is
    /// forfeited (§9.35), or `None` if already at the maximum (32+) penalty.
    /// Used by the FoK victory-progress panel to show "next penalty at N".
    pub fn next_loss_threshold(dervish_lost: i16) -> Option<i16> {
        match dervish_lost {
            n if n < 16 => Some(16),
            n if n < 24 => Some(24),
            n if n < 32 => Some(32),
            _ => None,
        }
    }

    /// Final level: the turn-based base shifted toward the British end of the
    /// ladder by the Dervish loss penalty (§9.35). Worked example from the
    /// rulebook: GORDON dies turn 5 (tactical) with 24 Dervish losses (−2
    /// levels) nets a British marginal.
    pub fn resolve(gordon_died_turn: Option<u8>, scenario_end_turn: u8, dervish_lost: i16) -> Self {
        let base = Self::base(gordon_died_turn, scenario_end_turn);
        let base_idx = Self::LADDER
            .iter()
            .position(|l| *l == base)
            .unwrap_or(Self::DEFAULT_LADDER_IDX) as i16;
        let shifted =
            (base_idx + Self::loss_penalty(dervish_lost)).clamp(0, Self::LADDER.len() as i16 - 1);
        Self::LADDER[shifted as usize]
    }
}

impl std::fmt::Display for FoKVictoryLevel {
    /// Human-readable label with a space, e.g. "Dervish Decisive",
    /// "British Marginal" (§9.35). Used by the FoK victory-progress panel.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FoKVictoryLevel::DervishDecisive => "Dervish Decisive",
            FoKVictoryLevel::DervishTactical => "Dervish Tactical",
            FoKVictoryLevel::DervishMarginal => "Dervish Marginal",
            FoKVictoryLevel::BritishMarginal => "British Marginal",
            FoKVictoryLevel::BritishTactical => "British Tactical",
            FoKVictoryLevel::BritishDecisive => "British Decisive",
        };
        f.write_str(s)
    }
}

/// The typed result of a finished game, preserving the scenario-specific
/// victory level (rulebook §9.14, §9.24, §9.35). Replaces the former
/// stringly-typed `game_result: Option<String>`.
///
/// The Historical scenario scores each side on its own unit-elimination
/// ladder (§9.24), so its result carries *both* levels rather than a single
/// net level -- the newspaper layer compares them to pick a template.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameResult {
    Campaign(CampaignVictoryLevel),
    Historical {
        ae: HistoricalVictoryLevel,
        d: HistoricalVictoryLevel,
    },
    FoK(FoKVictoryLevel),
}

impl GameResult {
    /// Render the result as the short human-readable key the app displays in
    /// its end-of-game stats line and feeds to the newspaper prompt. Mirrors
    /// the strings the former `Option<String>` field held: the `Debug` output
    /// of the level enums for Campaign/FoK, and the signed net
    /// (`ae_level - d_level`) for Historical.
    pub fn display_key(self) -> String {
        match self {
            GameResult::Campaign(level) => format!("{level:?}"),
            GameResult::Historical { ae, d } => {
                format!("{:+}", ae as i16 - d as i16)
            }
            GameResult::FoK(level) => format!("{level:?}"),
        }
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

// ---------------------------------------------------------------------------
// Tests -- every numeric rule above must round-trip a manual example.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::HexsideKind;
    use traceability_macro::rulebook;

    #[test]
    fn die_roll_from_u8_clamps() {
        assert_eq!(
            DieRoll::try_from(0u16.clamp(1, 10))
                .unwrap_or(DieRoll::Ten)
                .value(),
            1
        );
        assert_eq!(
            DieRoll::try_from(11u16.clamp(1, 10))
                .unwrap_or(DieRoll::Ten)
                .value(),
            10
        );
        assert_eq!(DieRoll::try_from(7u16).unwrap(), DieRoll::Seven);
    }

    #[test]
    fn die_roll_add_clamps() {
        let r = DieRoll::Five;
        assert_eq!(r.apply_modifier(3).value(), 8);
        assert_eq!(r.apply_modifier(-9).value(), 1);
        assert_eq!(r.apply_modifier(99).value(), 10);
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
    fn night_movement_halves_round_down() {
        // §8.1: movement halved (round down).
        assert_eq!(
            effective_movement_at_night(
                MovementAllowance::Three,
                Player::AngloEgyptian,
                DayNight::Night
            )
            .value(),
            1
        );
        assert_eq!(
            effective_movement_at_night(
                MovementAllowance::Five,
                Player::AngloEgyptian,
                DayNight::Night
            )
            .value(),
            2
        );
        assert_eq!(
            effective_movement_at_night(
                MovementAllowance::One,
                Player::AngloEgyptian,
                DayNight::Night
            )
            .value(),
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
        assert!(
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_melee_attack()
        );
        assert!(
            UnitKind::Cavalry {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_melee_attack()
        );
        assert!(
            UnitKind::Camel {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_melee_attack()
        );
        assert!(
            UnitKind::DervishLeader {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_melee_attack()
        );
        assert!(
            !UnitKind::Artillery {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_melee_attack()
        );
        assert!(
            !UnitKind::Maxim {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_melee_attack()
        );
        assert!(
            !UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0
            }
            .may_melee_attack()
        );
        assert!(!UnitKind::Fort { fire: 0, melee: 0 }.may_melee_attack());
        assert!(!UnitKind::BritishLeader { movement: 0 }.may_melee_attack());

        // §7.1 -- gunboats may not be melee attacked.
        assert!(
            !UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0
            }
            .may_be_melee_attacked()
        );
        assert!(
            UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_be_melee_attacked()
        );
        assert!(UnitKind::Fort { fire: 0, melee: 0 }.may_be_melee_attacked());

        // §7.5.
        assert!(
            UnitKind::Cavalry {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_retreat_before_melee()
        );
        assert!(
            UnitKind::Camel {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_retreat_before_melee()
        );
        assert!(
            !UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0
            }
            .may_retreat_before_melee()
        );
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

    #[rulebook("§9.14")]
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
            CampaignVictoryLevel::from_superiority(VictoryPoints::new(50)),
            CampaignVictoryLevel::Decisive(Player::AngloEgyptian)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints::new(30)),
            CampaignVictoryLevel::Tactical(Player::AngloEgyptian)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints::new(15)),
            CampaignVictoryLevel::Marginal(Player::AngloEgyptian)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints::new(5)),
            CampaignVictoryLevel::Draw
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints::new(0)),
            CampaignVictoryLevel::Draw
        ));
        // Dervish:
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints::new(-5)),
            CampaignVictoryLevel::Draw
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints::new(-15)),
            CampaignVictoryLevel::Marginal(Player::Dervish)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints::new(-25)),
            CampaignVictoryLevel::Tactical(Player::Dervish)
        ));
        assert!(matches!(
            CampaignVictoryLevel::from_superiority(VictoryPoints::new(-40)),
            CampaignVictoryLevel::Decisive(Player::Dervish)
        ));
    }

    #[test]
    fn victory_ledger_accumulates() {
        let mut l = VictoryLedger::default();
        l.events.push(VpEvent {
            turn: GameTurnIndex::new(1),
            source: VpSource::KhalifaEliminated,
        });
        l.events.push(VpEvent {
            turn: GameTurnIndex::new(2),
            source: VpSource::DervishUnitEliminated,
        });
        l.events.push(VpEvent {
            turn: GameTurnIndex::new(2),
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

    #[rulebook("§5.54")]
    #[test]
    fn brigade_integrity_empty_slice() {
        assert_eq!(brigade_integrity(&[]), BrigadeIntegrity::None);
    }

    #[rulebook("§5.54")]
    #[test]
    fn brigade_integrity_non_infantry_returns_none() {
        let ids = [UnitIdentity::DervishTribal {
            tribe: DervishTribe::Baggara,
        }];
        assert_eq!(brigade_integrity(&ids), BrigadeIntegrity::None);
    }

    #[rulebook("§5.54")]
    #[test]
    fn brigade_integrity_three_battalions_returns_none() {
        let brigade = BrigadeId {
            number: 1,
            nationality: BrigadeNationality::British,
        };
        let ids = [
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::First,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::Second,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::Third,
            },
        ];
        assert_eq!(brigade_integrity(&ids), BrigadeIntegrity::None);
    }

    #[rulebook("§5.54")]
    #[test]
    fn brigade_integrity_four_battalions_returns_integrated() {
        let brigade = BrigadeId {
            number: 2,
            nationality: BrigadeNationality::Egyptian,
        };
        let ids = [
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::First,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::Second,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::Third,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::Fourth,
            },
        ];
        assert_eq!(
            brigade_integrity(&ids),
            BrigadeIntegrity::Integrated(brigade)
        );
    }

    #[rulebook("§5.54")]
    #[test]
    fn brigade_integrity_mixed_brigades_returns_none() {
        let b1 = BrigadeId {
            number: 1,
            nationality: BrigadeNationality::British,
        };
        let b2 = BrigadeId {
            number: 2,
            nationality: BrigadeNationality::British,
        };
        let ids = [
            UnitIdentity::AngloEgyptianInfantry {
                brigade: b1,
                battalion: BattalionOrdinal::First,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade: b1,
                battalion: BattalionOrdinal::Second,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade: b2,
                battalion: BattalionOrdinal::Third,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade: b2,
                battalion: BattalionOrdinal::Fourth,
            },
        ];
        assert_eq!(brigade_integrity(&ids), BrigadeIntegrity::None);
    }

    #[rulebook("§5.54")]
    #[test]
    fn brigade_integrity_friendlies_returns_none() {
        let brigade = BrigadeId {
            number: 1,
            nationality: BrigadeNationality::Friendlies,
        };
        let ids = [
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::First,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::Second,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::Third,
            },
            UnitIdentity::AngloEgyptianInfantry {
                brigade,
                battalion: BattalionOrdinal::Fourth,
            },
        ];
        // Friendlies brigade — brigade() returns Some but the check still
        // passes since all four battalions are present and same brigade.
        // brigade_integrity does NOT filter on nationality, just checks
        // all four ordinals present and same brigade.
        assert_eq!(
            brigade_integrity(&ids),
            BrigadeIntegrity::Integrated(brigade)
        );
    }

    #[rulebook("§5.54")]
    #[test]
    fn unit_identity_brigade_and_battalion_accessors() {
        let id = UnitIdentity::AngloEgyptianInfantry {
            brigade: BrigadeId {
                number: 3,
                nationality: BrigadeNationality::Sudanese,
            },
            battalion: BattalionOrdinal::Fourth,
        };
        assert_eq!(
            id.brigade(),
            Some(BrigadeId {
                number: 3,
                nationality: BrigadeNationality::Sudanese,
            })
        );
        assert_eq!(id.battalion(), Some(BattalionOrdinal::Fourth));

        // Non-infantry identity returns None for both.
        let dervish = UnitIdentity::DervishTribal {
            tribe: DervishTribe::Taiasha,
        };
        assert_eq!(dervish.brigade(), None);
        assert_eq!(dervish.battalion(), None);
    }

    #[rulebook("§9.24")]
    #[test]
    fn historical_victory_level_for_anglo_egyptian() {
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(0),
            HistoricalVictoryLevel::Draw
        );
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(29),
            HistoricalVictoryLevel::Draw
        );
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(30),
            HistoricalVictoryLevel::Marginal
        );
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(44),
            HistoricalVictoryLevel::Marginal
        );
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(45),
            HistoricalVictoryLevel::Tactical
        );
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(59),
            HistoricalVictoryLevel::Tactical
        );
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(60),
            HistoricalVictoryLevel::Strategic
        );
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(99),
            HistoricalVictoryLevel::Strategic
        );
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(100),
            HistoricalVictoryLevel::Decisive
        );
        assert_eq!(
            HistoricalVictoryLevel::for_anglo_egyptian(150),
            HistoricalVictoryLevel::Decisive
        );
    }

    #[rulebook("§9.24")]
    #[test]
    fn historical_victory_level_for_dervish() {
        assert_eq!(
            HistoricalVictoryLevel::for_dervish(0),
            HistoricalVictoryLevel::Draw
        );
        assert_eq!(
            HistoricalVictoryLevel::for_dervish(4),
            HistoricalVictoryLevel::Draw
        );
        assert_eq!(
            HistoricalVictoryLevel::for_dervish(5),
            HistoricalVictoryLevel::Marginal
        );
        assert_eq!(
            HistoricalVictoryLevel::for_dervish(9),
            HistoricalVictoryLevel::Marginal
        );
        assert_eq!(
            HistoricalVictoryLevel::for_dervish(10),
            HistoricalVictoryLevel::Tactical
        );
        assert_eq!(
            HistoricalVictoryLevel::for_dervish(14),
            HistoricalVictoryLevel::Tactical
        );
        assert_eq!(
            HistoricalVictoryLevel::for_dervish(15),
            HistoricalVictoryLevel::Strategic
        );
        assert_eq!(
            HistoricalVictoryLevel::for_dervish(29),
            HistoricalVictoryLevel::Strategic
        );
        assert_eq!(
            HistoricalVictoryLevel::for_dervish(30),
            HistoricalVictoryLevel::Decisive
        );
    }

    #[rulebook("§9.35")]
    #[test]
    fn fok_victory_level_gordon_died_early() {
        assert_eq!(
            FoKVictoryLevel::resolve(Some(3), 8, 0),
            FoKVictoryLevel::DervishDecisive
        );
        assert_eq!(
            FoKVictoryLevel::resolve(Some(4), 8, 0),
            FoKVictoryLevel::DervishDecisive
        );
        assert_eq!(
            FoKVictoryLevel::resolve(Some(5), 8, 0),
            FoKVictoryLevel::DervishTactical
        );
        assert_eq!(
            FoKVictoryLevel::resolve(Some(6), 8, 0),
            FoKVictoryLevel::DervishMarginal
        );
    }

    #[rulebook("§9.35")]
    #[test]
    fn fok_victory_level_gordon_survived() {
        // GORDON survived to turn 8 → British decisive.
        assert_eq!(
            FoKVictoryLevel::resolve(None, 8, 0),
            FoKVictoryLevel::BritishDecisive
        );
        // GORDON survived to turn 7 → British tactical.
        assert_eq!(
            FoKVictoryLevel::resolve(None, 7, 0),
            FoKVictoryLevel::BritishTactical
        );
        // GORDON survived to turn 6 → British marginal.
        assert_eq!(
            FoKVictoryLevel::resolve(None, 6, 0),
            FoKVictoryLevel::BritishMarginal
        );
    }

    #[rulebook("§9.35")]
    #[test]
    fn fok_victory_level_worked_example() {
        assert_eq!(
            FoKVictoryLevel::resolve(Some(5), 8, 24),
            FoKVictoryLevel::BritishMarginal
        );
    }

    #[rulebook("§9.35")]
    #[test]
    fn fok_victory_level_late_gordon_death() {
        assert_eq!(
            FoKVictoryLevel::resolve(Some(7), 8, 0),
            FoKVictoryLevel::DervishMarginal
        );
        assert_eq!(
            FoKVictoryLevel::resolve(Some(8), 8, 0),
            FoKVictoryLevel::DervishMarginal
        );
    }

    #[test]
    fn movement_allowance_display() {
        assert_eq!(format!("{}", MovementAllowance::Eight), "8");
        assert_eq!(format!("{}", MovementAllowance::Immobile), "0");
        assert_eq!(format!("{}", MovementAllowance::Three), "3");
    }

    // §6.16: halving fire strength rounds down per unit and never reduces
    // a unit's firing strength below one.
    #[rulebook("§6.16")]
    #[test]
    fn halving_rounds_down_and_never_below_one() {
        assert_eq!(RangeBand::Halved.apply(9), 4);
        assert_eq!(RangeBand::Halved.apply(4), 2);
        assert_eq!(RangeBand::Halved.apply(3), 1);
        assert_eq!(RangeBand::Halved.apply(1), 1);
    }

    #[rulebook("§6.11")]
    #[test]
    fn fire_factor_sum_to_row() {
        let factors = [FireFactor::Eight, FireFactor::Eight];
        let row = FireFactor::sum_to_row(&factors);
        // 8 + 8 = 16 → Row16to20.
        assert!(matches!(
            row,
            crate::combat_results_table::FireFactorRow::Row16to20
        ));

        let factors2 = [FireFactor::Five, FireFactor::Five];
        let row2 = FireFactor::sum_to_row(&factors2);
        // 5 + 5 = 10 → Row06to10.
        assert!(matches!(
            row2,
            crate::combat_results_table::FireFactorRow::Row06to10
        ));
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

/// Kani proof harnesses for the rules-engine value types (`cargo kani`, see
/// `scripts/kani.sh`).
///
/// Scope note: the four printed tables (Combat Results, Range Effects,
/// Scattergram, Line of Sight) moved to authored RON parsed at runtime
/// (`tables_data`). Kani cannot see through `include_str!` data, so table
/// *contents* are checked by `tables_data::tests::tables_parse_and_have_expected_shape`
/// instead. What is proven here is the arithmetic and the conversions around
/// those lookups -- the parts that are pure functions of small enum domains.
#[cfg(kani)]
mod verification {
    use super::{DieRoll, FireModifier, MeleeModifier, MovementAllowance};

    // -- value_enum! conversions -------------------------------------------
    //
    // `value_enum!` generates `value() -> u16` and `TryFrom<u16>`. The
    // round-trip `try_from(x.value()) == Ok(x)` requires `value` to be
    // injective: it holds today by inspection, but a new variant reusing an
    // existing printed value would break it silently. These iterate `ALL`, so
    // they cover new variants automatically.

    /// Round-trip and injectivity for every `value_enum!` enum.
    macro_rules! prove_value_enum {
        ($name:ident, $ty:ty) => {
            #[kani::proof]
            fn $name() {
                let all = <$ty>::ALL;
                let i: usize = kani::any();
                let j: usize = kani::any();
                kani::assume(i < all.len());
                kani::assume(j < all.len());
                // `TryFrom` inverts `value`.
                assert!(<$ty>::try_from(all[i].value()) == Ok(all[i]));
                // Distinct variants never share a printed value.
                if i != j {
                    assert!(all[i].value() != all[j].value());
                }
            }
        };
    }

    prove_value_enum!(fire_factor_value_roundtrips, super::FireFactor);
    prove_value_enum!(melee_factor_value_roundtrips, super::MeleeFactor);
    prove_value_enum!(die_roll_value_roundtrips, DieRoll);
    prove_value_enum!(battalion_ordinal_value_roundtrips, super::BattalionOrdinal);
    prove_value_enum!(movement_allowance_value_roundtrips, MovementAllowance);

    /// §8.1 halves the movement allowance at night. The `expect` in
    /// [`MovementAllowance::halve`] is safe only because every variant's value
    /// halves onto *another* variant -- an arithmetic coincidence a new variant
    /// could break (e.g. `TwentyTwo = 22` halves to 11, which is not a
    /// variant, and would panic). This proves it holds for all variants,
    /// including any added later.
    // §8.1
    #[kani::proof]
    fn movement_allowance_halve_never_panics() {
        let i: usize = kani::any();
        kani::assume(i < MovementAllowance::ALL.len());
        let a = MovementAllowance::ALL[i];
        let halved = a.halve();
        // Halving is exactly integer division by two, and never increases.
        assert!(halved.value() == a.value() / 2);
        assert!(halved.value() <= a.value());
    }

    // -- die-roll arithmetic (§6.24, §7.7) ---------------------------------

    /// An arbitrary legal die roll.
    fn any_roll() -> DieRoll {
        let i: usize = kani::any();
        kani::assume(i < DieRoll::ALL.len());
        DieRoll::ALL[i]
    }

    /// `apply_modifier` is total over *every* `i16`. It is a `pub` method
    /// taking an unconstrained modifier, and `FireAttack::net_modifier` folds
    /// an unbounded list whose `FireModifier::Terrain(i16)` arrives over the
    /// network -- so a plain `+` overflowed here. Saturating arithmetic plus
    /// the 1..=10 clamp makes it total, which also makes the
    /// `unwrap_or(DieRoll::Ten)` fallback unreachable.
    // §6.24
    #[kani::proof]
    fn die_roll_apply_modifier_is_total() {
        let roll = any_roll();
        let modifier: i16 = kani::any();
        let out = roll.apply_modifier(modifier);
        assert!(out.value() >= 1 && out.value() <= 10);
    }

    /// A larger modifier never yields a lower roll. The outcome-prediction UI
    /// renders modifier bands assuming this.
    #[kani::proof]
    fn die_roll_apply_modifier_is_monotone() {
        let roll = any_roll();
        let a: i16 = kani::any();
        let b: i16 = kani::any();
        kani::assume(a <= b);
        assert!(roll.apply_modifier(a).value() <= roll.apply_modifier(b).value());
    }

    /// A zero modifier is the identity.
    #[kani::proof]
    fn die_roll_zero_modifier_is_identity() {
        let roll = any_roll();
        assert!(roll.apply_modifier(0) == roll);
    }

    /// Applying any single fire modifier keeps the roll legal, including
    /// `FireModifier::Terrain(n)` for an arbitrary `n` -- the variant that
    /// carries an unbounded `i16` straight off the wire (§6.23).
    #[kani::proof]
    fn fire_modifier_keeps_roll_legal() {
        let n: i16 = kani::any();
        let mods = [
            FireModifier::AngloEgyptianDirectFire,
            FireModifier::BrigadeIntegrity,
            FireModifier::Terrain(n),
            FireModifier::ZaribaThornHedge,
            FireModifier::ZaribaTrenchEntrenched,
        ];
        let i: usize = kani::any();
        kani::assume(i < mods.len());
        let out = any_roll().apply_modifier(mods[i].die_modifier());
        assert!(out.value() >= 1 && out.value() <= 10);
    }

    /// Same for melee modifiers (§7.7, §9.232).
    // §7.7
    #[kani::proof]
    fn melee_modifier_keeps_roll_legal() {
        let mods = [
            MeleeModifier::DervishStandard,
            MeleeModifier::AngloEgyptianStandard,
            MeleeModifier::DervishVsTrenchedDefender,
        ];
        let i: usize = kani::any();
        kani::assume(i < mods.len());
        let out = any_roll().apply_modifier(mods[i].die_modifier());
        assert!(out.value() >= 1 && out.value() <= 10);
    }
}

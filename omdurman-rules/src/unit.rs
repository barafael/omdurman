//! Unit identity (tribes, brigades, named leaders, gunboats), weapon
//! classes and range bands, unit profile/runtime state, and the ZOC,
//! stacking and brigade-integrity types.

use serde::{Deserialize, Serialize};

use crate::{FireFactor, MeleeFactor, MovementAllowance, UnitId};
use omdurman_types::{
    BrigadeId, BrigadeNationality, DervishTribe, Faction, HexCoord, Player, SetupLetter, UnitKind,
};

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
    /// "K" set-up hex; black: the Taiasha (and the artillery).
    #[strum(serialize = "Khalifa Abdullah")]
    KhalifaAbdullah,
    /// "Y" set-up hex; grey: Baggara and Jaalin.
    Yakub,
    /// "S" set-up hex; red: the Danagla.
    Sherif,
    /// "A" set-up hex; blue: Kehena and Degheim.
    #[strum(serialize = "Ali Wad Helu")]
    AliWadHelu,
    /// "O" set-up hex; white: the Hadendowa.
    #[strum(serialize = "Osman Digna")]
    OsmanDigna,
    /// "D" set-up hex; green: Mulazmin and Jehadia.
    #[strum(serialize = "Sheik El Din")]
    SheikElDin,
}

impl DervishLeader {
    /// Whether this leader commands `tribe`, i.e. may stack with its units
    /// (§5.53: "Dervish leaders... may only stack with units of their command,
    /// i.e. colour"). The command is the leader's counter colour: Khalifa
    /// black (Taiasha), Yakub grey (Baggara, Jaalin), Osman Digna white
    /// (Hadendowa), Sheik El Din green (Mulazmin, Jehadia), Sherif red
    /// (Danagla), Ali Wad Helu blue (Kehena, Degheim). Isa Zachneih, alone on
    /// the east bank (§9.111), serves under no leader.
    pub fn commands(self, tribe: DervishTribe) -> bool {
        DervishLeader::of_tribe(tribe) == Some(self)
    }

    /// The leader whose colour `tribe` wears (see [`Self::commands`]): the
    /// leader a Historical-scenario unit sets up near (§9.212).
    pub fn of_tribe(tribe: DervishTribe) -> Option<DervishLeader> {
        match tribe {
            DervishTribe::Taiasha => Some(DervishLeader::KhalifaAbdullah),
            DervishTribe::Baggara | DervishTribe::Jaalin => Some(DervishLeader::Yakub),
            DervishTribe::Hadendowa => Some(DervishLeader::OsmanDigna),
            DervishTribe::Mulazmin | DervishTribe::Jehadia => Some(DervishLeader::SheikElDin),
            DervishTribe::Danagla => Some(DervishLeader::Sherif),
            DervishTribe::Kehena | DervishTribe::Degheim => Some(DervishLeader::AliWadHelu),
            DervishTribe::IsaZachneih => None,
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
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GunboatId {
    /// One of the five new-type named gunboats with howitzer capability.
    Named(NamedGunboat),
    /// An old-style gunboat -- no howitzer fire (§2.32).
    Old(OldGunboat),
    /// A Dervish gunboat (§9.111, §10.14).
    DervishGunboat(u8),
}

/// The boat's name for players ("Sultan", "Tamai"; a Dervish boat by number),
/// not the variant ("Named").
impl std::fmt::Display for GunboatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GunboatId::Named(boat) => write!(f, "{boat}"),
            GunboatId::Old(boat) => write!(f, "{boat}"),
            GunboatId::DervishGunboat(n) => write!(f, "No. {n}"),
        }
    }
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
    #[strum(serialize = "Lord Kitchener")]
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

impl WeaponClass {
    /// All weapon classes in `tables_data` row order: `index()` into the
    /// authored range-effects rows matches this order (§6.22).
    pub const ALL: [WeaponClass; 5] = [
        WeaponClass::Melee,
        WeaponClass::Rifles,
        WeaponClass::Maxims,
        WeaponClass::Artillery,
        WeaponClass::Howitzer,
    ];

    /// Zero-based row index into the authored range-effects tables
    /// (`tables_data::{AE,DERVISH}_RANGE_EFFECTS`, §6.22).
    pub fn index(self) -> usize {
        match self {
            WeaponClass::Melee => 0,
            WeaponClass::Rifles => 1,
            WeaponClass::Maxims => 2,
            WeaponClass::Artillery => 3,
            WeaponClass::Howitzer => 4,
        }
    }
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

/// The §5.52 stacking group of a Dervish unit (see
/// [`UnitIdentity::dervish_stacking_group`]): tribal units group by tribe;
/// the Dervish artillery is its own group.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DervishStackingGroup {
    /// A tribal counter of this tribe (§5.52).
    Tribe(DervishTribe),
    /// The three Dervish artillery counters (§9.322) -- not a tribe, but not
    /// any tribe's unit either, so they stack only with each other (and with
    /// a leader whose §5.53 command allows).
    Artillery,
}

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
    /// A British fort: FALL OF KHARTOUM's Forts Makran and Buri (§9.321).
    AngloEgyptianFort,
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
            // (A fort never moves, §5.25.)
            UnitIdentity::AngloEgyptianGunboat(_) | UnitIdentity::AngloEgyptianFort => false,
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

    /// The §5.52 stacking group this Dervish unit belongs to, if any: tribal
    /// units group by tribe, and the Dervish artillery (§9.322's three guns)
    /// groups as its own "tribe" -- a gun is not a Hadendowa (or Kehena, or
    /// Mulazmin, ...) unit, so counters of different groups may not share a
    /// hex. Guns still stack with guns, and with the Khalifa (whose §5.53
    /// command check constrains tribal units only -- §5.23 groups the
    /// Khalifa, his artillery, and the Taiasha as the walled-city force).
    /// `None` for every unit the §5.52 law does not constrain: Dervish
    /// leaders, forts and gunboats, and all Anglo-Egyptian units.
    pub fn dervish_stacking_group(&self) -> Option<DervishStackingGroup> {
        match self {
            UnitIdentity::DervishTribal { tribe } => Some(DervishStackingGroup::Tribe(*tribe)),
            UnitIdentity::DervishArtillery => Some(DervishStackingGroup::Artillery),
            _ => None,
        }
    }

    /// The brigade printed on the counter, if this is an Anglo-Egyptian
    /// infantry unit (§5.54): a Sudanese battalion serves in an Egyptian
    /// brigade ([`BrigadeId::designation`]). `None` for every other identity.
    /// (The identity's own `brigade` field keeps the troop type, which the
    /// FALL OF KHARTOUM order of battle counts.)
    pub fn brigade(&self) -> Option<BrigadeId> {
        match self {
            UnitIdentity::AngloEgyptianInfantry { brigade, .. } => Some(brigade.designation()),
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
            UnitIdentity::AngloEgyptianFort => "British Fort".into(),
            UnitIdentity::DervishGunboat(g) => format!("Dervish Gunboat {g}"),
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                format!("{brigade} {battalion} Btn")
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
    // Brigades as printed (§5.54): 1E is II Egyptian plus three Sudanese
    // battalions (see `UnitIdentity::brigade`).
    let Some(brigade) = identities.first().and_then(UnitIdentity::brigade) else {
        return BrigadeIntegrity::None;
    };
    // §5.54 names only "British, Sudanese, and Egyptian infantry" -- the
    // "Friendlies" brigade never has brigade integrity.
    if brigade.nationality == BrigadeNationality::Friendlies {
        return BrigadeIntegrity::None;
    }
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
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
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
}

/// Errors returned when a candidate stack would violate stacking rules.
#[derive(thiserror::Error, Clone, Copy, PartialEq, Eq, Debug)]
pub enum StackingError {
    /// "No more than four units may occupy a hex" (§5.51), excluding leaders
    /// and the gunboat exception. Also covers the occupation corollary: a hex
    /// occupied by *enemy* units is never a legal stack (§7.1 -- engaging the
    /// enemy is what melee is for), except a lone Anglo-Egyptian leader
    /// (§6.51: a Dervish unit occupying his hex eliminates him).
    #[error("hex stack exceeds the four-unit limit [§5.51]")]
    OverLimit,
    /// "Gunboats may not stack with any other unit" (§5.51, exception §5.21).
    #[error("gunboats may not stack with non-gunboat units [§5.51]")]
    GunboatStack,
    /// "Units of different Dervish tribes may not stack together" (§5.52).
    /// The Dervish artillery (§9.322's three guns) is not a tribe but is not
    /// any tribe's unit either: it forms its own stacking group (see
    /// [`UnitIdentity::dervish_stacking_group`]).
    #[error("Dervish units of different tribes may not stack [§5.52]")]
    DervishTribeMix,
    /// A unit may never share a hex with enemy units (§5.51, §7.1); only the
    /// lone Anglo-Egyptian leader is exempt (§6.51).
    #[error("enemy units may not share a hex; melee, not movement, engages them [§5.51, §7.1]")]
    EnemyCohabitation,
    /// "If Dervish leaders elect to stack, they may only stack with units of
    /// their command (i.e. colour)" (§5.53).
    #[error("Dervish leader may only stack with units of their own command [§5.53]")]
    DervishLeaderCommandMismatch,
}

/// Brigade-integrity status of a stack (§5.54). Carries the brigade if the
/// stack contains all four battalions of a single Anglo-Egyptian brigade.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BrigadeIntegrity {
    None,
    Integrated(BrigadeId),
}

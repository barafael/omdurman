//! A palette of real counters for the proof harnesses: one of each shape the
//! rules treat differently (leaders, tribal and British infantry, cavalry,
//! artillery, Maxims, engineers, gunboats, forts, "Friendlies", GORDON).
//!
//! The profiles are literals so a harness can draw a unit, or stub
//! [`profile_for_unit`](crate::unit_profiles::profile_for_unit) with a
//! loop-free `match`, without symexing the roster tables. The test below
//! holds every literal to the real roster.

use crate::{
    BattalionOrdinal, BritishLeader, DervishLeader, FireFactor, GunboatId, GunboatMovement,
    MeleeFactor, MovementAllowance, NamedGunboat, UnitId, UnitIdentity, UnitMovement, UnitProfile,
    WeaponClass,
};
use omdurman_types::{BrigadeId, BrigadeNationality, DervishTribe, UnitKind};

/// Number of palette counters.
pub(crate) const PALETTE_LEN: usize = 14;

/// The palette's counters, `(id, profile)`, exactly as the roster prints them.
pub(crate) const PALETTE: [(UnitId, UnitProfile); PALETTE_LEN] = [
    (
        UnitId::AliWadHelu_0_0,
        UnitProfile {
            kind: UnitKind::DervishLeader {
                fire: 0,
                melee: 0,
                movement: 0,
            },
            identity: UnitIdentity::DervishLeader(DervishLeader::AliWadHelu),
            weapon: WeaponClass::Melee,
            fire: Some(FireFactor::One),
            melee: Some(MeleeFactor::One),
            movement: UnitMovement::Land(MovementAllowance::Fifteen),
        },
    ),
    (
        UnitId::AliWadHelu_0_1,
        UnitProfile {
            kind: UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            },
            identity: UnitIdentity::DervishTribal {
                tribe: DervishTribe::Kehena,
            },
            weapon: WeaponClass::Melee,
            fire: Some(FireFactor::Three),
            melee: Some(MeleeFactor::Six),
            movement: UnitMovement::Land(MovementAllowance::Nine),
        },
    ),
    (
        UnitId::KhalifaAbdullah_0_1,
        UnitProfile {
            kind: UnitKind::Artillery {
                fire: 0,
                melee: 0,
                movement: 0,
            },
            identity: UnitIdentity::DervishArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(FireFactor::Six),
            melee: Some(MeleeFactor::One),
            movement: UnitMovement::Land(MovementAllowance::Seven),
        },
    ),
    (
        UnitId::KhalifaAbdullah_1_0,
        UnitProfile {
            kind: UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0,
            },
            identity: UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1)),
            weapon: WeaponClass::Artillery,
            fire: Some(FireFactor::Four),
            melee: None,
            movement: UnitMovement::Gunboat(GunboatMovement {
                upstream: MovementAllowance::Ten,
                downstream: MovementAllowance::Sixteen,
            }),
        },
    ),
    (
        UnitId::Hadendowa_7_1,
        UnitProfile {
            kind: UnitKind::Fort { fire: 0, melee: 0 },
            identity: UnitIdentity::DervishFort,
            weapon: WeaponClass::Artillery,
            fire: Some(FireFactor::Four),
            melee: Some(MeleeFactor::One),
            movement: UnitMovement::Immobile,
        },
    ),
    (
        UnitId::BritishArmy_0_0,
        UnitProfile {
            kind: UnitKind::Cavalry {
                fire: 8,
                melee: 5,
                movement: 15,
            },
            identity: UnitIdentity::AngloEgyptianCavalry,
            weapon: WeaponClass::Rifles,
            fire: Some(FireFactor::Eight),
            melee: Some(MeleeFactor::Five),
            movement: UnitMovement::Land(MovementAllowance::Fifteen),
        },
    ),
    (
        UnitId::BritishArmy_0_1,
        UnitProfile {
            kind: UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::British,
                },
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(FireFactor::Ten),
            melee: Some(MeleeFactor::Five),
            movement: UnitMovement::Land(MovementAllowance::Eight),
        },
    ),
    (
        UnitId::BritishArmy_1_0,
        UnitProfile {
            kind: UnitKind::Infantry {
                fire: 5,
                melee: 3,
                movement: 8,
            },
            identity: UnitIdentity::RoyalEngineers,
            weapon: WeaponClass::Rifles,
            fire: Some(FireFactor::Five),
            melee: Some(MeleeFactor::Three),
            movement: UnitMovement::Land(MovementAllowance::Eight),
        },
    ),
    (
        UnitId::BritishArmy_2_0,
        UnitProfile {
            kind: UnitKind::Artillery {
                fire: 10,
                melee: 1,
                movement: 7,
            },
            identity: UnitIdentity::AngloEgyptianArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(FireFactor::Ten),
            melee: Some(MeleeFactor::One),
            movement: UnitMovement::Land(MovementAllowance::Seven),
        },
    ),
    (
        UnitId::BritishArmy_4_0,
        UnitProfile {
            kind: UnitKind::Maxim {
                fire: 6,
                melee: 1,
                movement: 12,
            },
            identity: UnitIdentity::AngloEgyptianMaxim,
            weapon: WeaponClass::Maxims,
            fire: Some(FireFactor::Six),
            melee: Some(MeleeFactor::One),
            movement: UnitMovement::Land(MovementAllowance::Twelve),
        },
    ),
    (
        UnitId::BritishBoats_3_0,
        UnitProfile {
            kind: UnitKind::Gunboat {
                fire: 0,
                upstream: 0,
                downstream: 0,
            },
            identity: UnitIdentity::AngloEgyptianGunboat(GunboatId::Named(NamedGunboat::Naser)),
            weapon: WeaponClass::Artillery,
            fire: Some(FireFactor::Five),
            melee: None,
            movement: UnitMovement::Gunboat(GunboatMovement {
                upstream: MovementAllowance::Twelve,
                downstream: MovementAllowance::Eighteen,
            }),
        },
    ),
    (
        UnitId::BritishBoats_3_1,
        UnitProfile {
            kind: UnitKind::BritishLeader { movement: 0 },
            identity: UnitIdentity::AngloEgyptianLeader(BritishLeader::Gordon),
            weapon: WeaponClass::Melee,
            fire: None,
            melee: None,
            movement: UnitMovement::Land(MovementAllowance::Immobile),
        },
    ),
    (
        UnitId::Kitchener_0_1,
        UnitProfile {
            kind: UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::Friendlies,
                },
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(FireFactor::Eight),
            melee: Some(MeleeFactor::Six),
            movement: UnitMovement::Land(MovementAllowance::Nine),
        },
    ),
    (
        UnitId::BritishForts_0_0,
        UnitProfile {
            kind: UnitKind::Fort { fire: 0, melee: 0 },
            identity: UnitIdentity::AngloEgyptianFort,
            weapon: WeaponClass::Artillery,
            fire: Some(FireFactor::Four),
            melee: Some(MeleeFactor::One),
            movement: UnitMovement::Immobile,
        },
    ),
];

/// [`profile_for_unit`](crate::unit_profiles::profile_for_unit) restricted to
/// the palette: the real profile for a palette counter, `None` for any other
/// id. A `match`, not a scan, so a proof pays no loop for it.
#[cfg(kani)]
pub(crate) fn palette_profile(id: UnitId) -> Option<UnitProfile> {
    let i = match id {
        UnitId::AliWadHelu_0_0 => 0,
        UnitId::AliWadHelu_0_1 => 1,
        UnitId::KhalifaAbdullah_0_1 => 2,
        UnitId::KhalifaAbdullah_1_0 => 3,
        UnitId::Hadendowa_7_1 => 4,
        UnitId::BritishArmy_0_0 => 5,
        UnitId::BritishArmy_0_1 => 6,
        UnitId::BritishArmy_1_0 => 7,
        UnitId::BritishArmy_2_0 => 8,
        UnitId::BritishArmy_4_0 => 9,
        UnitId::BritishBoats_3_0 => 10,
        UnitId::BritishBoats_3_1 => 11,
        UnitId::Kitchener_0_1 => 12,
        UnitId::BritishForts_0_0 => 13,
        _ => return None,
    };
    Some(PALETTE[i].1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit_profiles::profile_for_unit;

    #[test]
    fn palette_counters_are_the_printed_counters() {
        for (id, profile) in PALETTE {
            assert_eq!(profile_for_unit(id), Some(profile), "{id:?}");
        }
    }

    #[test]
    fn palette_ids_are_distinct() {
        for (i, (a, _)) in PALETTE.iter().enumerate() {
            assert!(PALETTE[..i].iter().all(|(b, _)| b != a), "{a:?}");
        }
    }
}

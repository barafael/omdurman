//! Build rules-engine [`UnitProfile`]s for placed counters.
//!
//! A counter is identified on the sprite sheet by `(section_name, col, row)`.
//! Two distinct kinds of information go into a profile:
//!
//! * **Identity / kind / weapon** -- *what the unit is* (a Baggara tribe, the
//!   Khalifa, a British brigade battalion, a gunboat). This is not printed on
//!   the counter in a machine-readable way, so it is mapped from the section
//!   name via [`identity_for_section`].
//! * **Numeric factors** -- fire / melee / movement. These *are* authored, in
//!   the [`SpriteAnnotation`] the Units-mode editor writes. We read them from
//!   there rather than inventing them.
//!
//! A counter with no annotation, or whose section name we don't recognise,
//! yields `None` -- callers must cope with that rather than receiving a
//! fabricated stand-in unit.

use crate::{
    BattalionOrdinal, BrigadeId, BrigadeNationality, BritishLeader, DervishLeader, DervishTribe,
    FireFactor, GunboatMovement, MeleeFactor, MovementAllowance, UnitIdentity, UnitKind,
    UnitMovement, UnitProfile, WeaponClass,
};
use omdurman_types::{Brigade, Faction, SectionName, SpriteAnnotation};

/// The fixed identity facts about a counter, independent of its printed
/// numeric factors. Weapon class and kind follow from the identity.
pub(crate) struct Classification {
    kind: UnitKind,
    identity: UnitIdentity,
    weapon: WeaponClass,
}

/// Build a [`UnitProfile`] from a counter's section/grid identity plus its
/// authored [`SpriteAnnotation`] stats.
///
/// Returns `None` when the section name is not recognised -- there is no
/// generic fallback unit, so an unmapped counter is surfaced rather than
/// silently becoming, say, British infantry.
pub fn profile_from_annotation(
    section_name: SectionName,
    col: u32,
    row: u32,
    annotation: &SpriteAnnotation,
) -> Option<UnitProfile> {
    let Classification {
        kind,
        identity,
        weapon,
    } = identity_for_section(section_name, col, row)?;

    // The brigade designation printed on the counter (e.g. 2B, 3E) is the
    // authoritative source for an infantry unit's brigade (rulebook §5.54);
    // when set it overrides the column-derived default from
    // `identity_for_section`.
    let annotation_brigade = match annotation.faction {
        Some(Faction::BritishEgyptian { brigade }) => brigade,
        _ => Brigade::None,
    };
    let identity = apply_brigade_designation(identity, annotation_brigade);

    Some(UnitProfile {
        kind,
        identity,
        weapon,
        fire: factor(annotation.fire).and_then(|v| FireFactor::try_from(v).ok()),
        melee: factor(annotation.melee).and_then(|v| MeleeFactor::try_from(v).ok()),
        movement: movement_from_annotation(kind, annotation),
    })
}

/// Override an Anglo-Egyptian infantry unit's brigade with the designation
/// picked on its counter, e.g. [`Brigade::B2`] -> 2nd British, [`Brigade::E3`]
/// -> 3rd Egyptian (rulebook §5.54). Non-infantry identities and
/// [`Brigade::None`] are returned unchanged.
fn apply_brigade_designation(identity: UnitIdentity, brigade: Brigade) -> UnitIdentity {
    let UnitIdentity::AngloEgyptianInfantry {
        brigade: _,
        battalion,
    } = identity
    else {
        return identity;
    };
    let Some((number, nationality)) = brigade.parts() else {
        return identity;
    };
    UnitIdentity::AngloEgyptianInfantry {
        brigade: BrigadeId {
            number,
            nationality,
        },
        battalion,
    }
}

/// A positive factor is a printed value; zero/negative means the counter
/// prints no value in that slot (e.g. British leaders print no fire factor).
fn factor(value: i32) -> Option<u16> {
    (value > 0).then_some(value as u16)
}

/// Movement allowance from the annotation. Boats carry split upstream /
/// downstream allowances; everything else is uniform land movement. Forts
/// are immobile regardless of any printed number.
fn movement_from_annotation(kind: UnitKind, a: &SpriteAnnotation) -> UnitMovement {
    if kind == UnitKind::Fort {
        return UnitMovement::Immobile;
    }
    if a.is_boat {
        UnitMovement::Gunboat(GunboatMovement {
            upstream: MovementAllowance::try_from(a.movement_upstream.max(0) as u16)
                .unwrap_or(MovementAllowance::Immobile),
            downstream: MovementAllowance::try_from(a.movement_downstream.max(0) as u16)
                .unwrap_or(MovementAllowance::Immobile),
        })
    } else {
        UnitMovement::Land(
            MovementAllowance::try_from(a.movement.max(0) as u16)
                .unwrap_or(MovementAllowance::Immobile),
        )
    }
}

/// Map a sprite-sheet section name (and column, for multi-brigade sheets) to
/// the unit's identity, kind, and weapon class. `None` for unrecognised
/// sections.
/// Which faction a sprite-sheet section belongs to, for grouping the unit
/// picker. Sections are single-faction (Dervish tribes/leaders/forts vs the
/// Anglo-Egyptian army/boats), so this is a section-level classification.
/// Returns `None` only for sections that map to no placeable unit.
pub fn section_owner(section_name: SectionName) -> Option<crate::Player> {
    use crate::Player;
    match section_name {
        SectionName::Taiasha
        | SectionName::KhalifaAbdullah
        | SectionName::Sherif
        | SectionName::AliWadHelu
        | SectionName::SheikElDin
        | SectionName::Yakub
        | SectionName::OsmanDigna
        | SectionName::Hadendowa
        | SectionName::Baggara
        | SectionName::Jehadia
        | SectionName::Mulazmin
        | SectionName::Kehena
        | SectionName::Degheim
        | SectionName::Danagla
        | SectionName::UpperJaalin
        | SectionName::LowerJaalin
        | SectionName::HadendowaForts => Some(Player::Dervish),
        SectionName::BritishArmy
        | SectionName::EgyptianArmy
        | SectionName::Kitchener
        | SectionName::BritishBoats => Some(Player::AngloEgyptian),
        // Duplicate Mulazmin print runs; not placed from the picker.
        SectionName::UpperGreen | SectionName::LowerGreen => None,
    }
}

pub(crate) fn identity_for_section(
    section_name: SectionName,
    col: u32,
    row: u32,
) -> Option<Classification> {
    let c = |kind, identity, weapon| {
        Some(Classification {
            kind,
            identity,
            weapon,
        })
    };

    // Two Dervish leaders share a sprite section with their tribal retinue
    // rather than having a section of their own: Yakub is the first counter of
    // the `upper_Jaalin` block, and Osman Digna is the second counter of the
    // `Hadendowa` block. Resolve those specific counters as leaders before the
    // section falls through to its tribal mapping below.
    match (section_name, col, row) {
        (SectionName::UpperJaalin, 0, 0) => return dervish_leader(DervishLeader::Yakub),
        (SectionName::Hadendowa, 1, 0) => return dervish_leader(DervishLeader::OsmanDigna),
        _ => {}
    }

    // The `British_Boats` block is a mixed section resolved by cell, like the
    // tribal-leader blocks above. Its layout (rulebook §6.64, §2.32, §9.32):
    //   row 0: BREECH x3 (markers, §6.63), then the 5 named new-type gunboats
    //   row 1: BREECH x3, GORDON (3,1), then the 4 old-style gunboats
    // The specific gunboat variant is cosmetic (only Named-vs-Old drives the
    // howitzer rule, §6.64), so cells are mapped to variants positionally.
    if section_name == SectionName::BritishBoats {
        return british_boats(col, row);
    }

    match section_name {
        // -- Dervish leaders ------------------------------------------
        SectionName::KhalifaAbdullah => dervish_leader(DervishLeader::KhalifaAbdullah),
        SectionName::Sherif => dervish_leader(DervishLeader::Sherif),
        SectionName::AliWadHelu => dervish_leader(DervishLeader::AliWadHelu),
        SectionName::SheikElDin => dervish_leader(DervishLeader::SheikElDin),
        SectionName::Yakub => dervish_leader(DervishLeader::Yakub),
        SectionName::OsmanDigna => dervish_leader(DervishLeader::OsmanDigna),

        // -- Dervish foot tribes --------------------------------------
        SectionName::Taiasha => dervish_tribe(DervishTribe::Taiasha),
        SectionName::Hadendowa => dervish_tribe(DervishTribe::Hadendowa),
        SectionName::Baggara => dervish_tribe(DervishTribe::Baggara),
        SectionName::Jehadia => dervish_tribe(DervishTribe::Jehadia),
        SectionName::Mulazmin => dervish_tribe(DervishTribe::Mulazmin),
        SectionName::Kehena => dervish_tribe(DervishTribe::Kehena),
        SectionName::Degheim => dervish_tribe(DervishTribe::Degheim),
        SectionName::Danagla => dervish_tribe(DervishTribe::Danagla),
        SectionName::UpperJaalin | SectionName::LowerJaalin => dervish_tribe(DervishTribe::Jaalin),

        // -- Dervish artillery ----------------------------------------
        SectionName::HadendowaForts => c(
            UnitKind::Fort,
            UnitIdentity::DervishFort,
            WeaponClass::Artillery,
        ),

        // -- Anglo-Egyptian infantry brigades -------------------------
        SectionName::BritishArmy => ae_infantry(BrigadeNationality::British, col),
        SectionName::EgyptianArmy => ae_infantry(BrigadeNationality::Egyptian, col),

        // -- Anglo-Egyptian leaders -----------------------------------
        SectionName::Kitchener => c(
            UnitKind::BritishLeaderUnit,
            UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener),
            WeaponClass::Melee,
        ),

        // `British_Boats` is resolved by cell above; the "green" sections are
        // duplicate Mulazmin print runs with their own sections, unused here.
        SectionName::UpperGreen | SectionName::LowerGreen | SectionName::BritishBoats => None,
    }
}

/// Resolve a counter in the `British_Boats` section (rulebook §6.64, §2.32,
/// §9.32). GORDON is the immobile palace leader of FALL OF KHARTOUM (§9.346);
/// the named gunboats have howitzer fire, the old ones do not. `BREECH` marker
/// cells (§6.63) and any unmapped cell return `None`.
fn british_boats(col: u32, row: u32) -> Option<Classification> {
    use crate::{GunboatId, NamedGunboat, OldGunboat};

    let gunboat = |id| {
        Some(Classification {
            kind: UnitKind::Gunboat,
            identity: UnitIdentity::AngloEgyptianGunboat(id),
            // All gunboats fire on the Artillery line (§2.32); named gunboats
            // additionally have howitzer fire (§6.64), which the engine selects
            // from the `HowitzerFire` action rather than the profile weapon.
            weapon: WeaponClass::Artillery,
        })
    };

    match (col, row) {
        // GORDON -- immobile (0-0-0) palace leader, FALL OF KHARTOUM (§9.346).
        (3, 1) => Some(Classification {
            kind: UnitKind::BritishLeaderUnit,
            identity: UnitIdentity::AngloEgyptianLeader(BritishLeader::Gordon),
            weapon: WeaponClass::Melee,
        }),
        // Named new-type gunboats (§6.64). The sheet's fifth boat is printed
        // "Abu Klea"; the `NamedGunboat` enum's spare variant is `Naser`.
        (4, 0) => gunboat(GunboatId::Named(NamedGunboat::Sultan)),
        (5, 0) => gunboat(GunboatId::Named(NamedGunboat::Sheik)),
        (6, 0) => gunboat(GunboatId::Named(NamedGunboat::Fateh)),
        (7, 0) => gunboat(GunboatId::Named(NamedGunboat::Melik)),
        (3, 0) => gunboat(GunboatId::Named(NamedGunboat::Naser)),
        // Old-style gunboats (§2.32) -- no howitzer. Four cells, three named
        // variants; the fourth reuses a variant (identity is cosmetic).
        (4, 1) => gunboat(GunboatId::Old(OldGunboat::LordKitchener)),
        (5, 1) => gunboat(GunboatId::Old(OldGunboat::Tamai)),
        (6, 1) => gunboat(GunboatId::Old(OldGunboat::Metemmeh)),
        (7, 1) => gunboat(GunboatId::Old(OldGunboat::LordKitchener)),
        // BREECH markers (§6.63) and anything else: not a placeable unit.
        _ => None,
    }
}

fn dervish_leader(leader: DervishLeader) -> Option<Classification> {
    Some(Classification {
        kind: UnitKind::DervishLeaderUnit,
        identity: UnitIdentity::DervishLeader(leader),
        weapon: WeaponClass::Melee,
    })
}

fn dervish_tribe(tribe: DervishTribe) -> Option<Classification> {
    Some(Classification {
        kind: UnitKind::Infantry,
        identity: UnitIdentity::DervishTribal { tribe },
        weapon: WeaponClass::Rifles,
    })
}

fn ae_infantry(nationality: BrigadeNationality, col: u32) -> Option<Classification> {
    // Each brigade occupies a block of four columns; the column within the
    // block is the battalion ordinal (1..=4).
    let number = (col / 4) as u8 + 1;
    let battalion = match (col % 4) as u8 + 1 {
        1 => BattalionOrdinal::First,
        2 => BattalionOrdinal::Second,
        3 => BattalionOrdinal::Third,
        _ => BattalionOrdinal::Fourth,
    };
    Some(Classification {
        kind: UnitKind::Infantry,
        identity: UnitIdentity::AngloEgyptianInfantry {
            brigade: BrigadeId {
                number,
                nationality,
            },
            battalion,
        },
        weapon: WeaponClass::Rifles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use SectionName;

    fn annotation(fire: i32, melee: i32, movement: i32) -> SpriteAnnotation {
        SpriteAnnotation {
            color: omdurman_types::SpriteColor::BlackWhite,
            faction: None,
            text: String::new(),
            kind: omdurman_types::UnitFormKind::Infantry,
            fire,
            melee,
            movement,
            movement_upstream: 0,
            movement_downstream: 0,
            is_boat: false,
            is_unit: true,
            fires_twice: false,
        }
    }

    // §6.63
    #[test]
    fn breech_marker_cell_returns_none() {
        // `British_Boats` (0,0) is a BREECH marker (§6.63), not a placeable
        // unit -- it must yield no profile even though the section is mapped.
        assert!(
            profile_from_annotation(SectionName::BritishBoats, 0, 0, &annotation(0, 0, 0))
                .is_none()
        );
    }

    // §9.346
    #[test]
    fn gordon_is_an_immobile_british_leader() {
        // GORDON is the 0-0-0 palace leader at British_Boats (3,1) (§9.346).
        let p = profile_from_annotation(SectionName::BritishBoats, 3, 1, &annotation(0, 0, 0))
            .expect("Gordon resolves");
        assert_eq!(p.kind, UnitKind::BritishLeaderUnit);
        assert!(matches!(
            p.identity,
            UnitIdentity::AngloEgyptianLeader(BritishLeader::Gordon)
        ));
        // A 0-movement land unit is `Land(Immobile)` (0 MP); `UnitMovement::Immobile`
        // is reserved for forts. Either way Gordon has no movement allowance; the
        // hard §9.346 "may not move" ban is enforced separately in the engine.
        assert_eq!(p.movement, UnitMovement::Land(MovementAllowance::Immobile));
    }

    // §6.64
    #[test]
    fn named_and_old_gunboats_resolve() {
        let boat = SpriteAnnotation {
            is_boat: true,
            movement_upstream: 12,
            movement_downstream: 18,
            ..annotation(5, 0, 0)
        };
        let named = profile_from_annotation(SectionName::BritishBoats, 4, 0, &boat)
            .expect("named gunboat resolves");
        assert_eq!(named.kind, UnitKind::Gunboat);
        assert!(matches!(
            named.identity,
            UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Named(_))
        ));
        let old = profile_from_annotation(SectionName::BritishBoats, 4, 1, &boat)
            .expect("old gunboat resolves");
        assert!(matches!(
            old.identity,
            UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(_))
        ));
    }

    // §5.54
    #[test]
    fn tribe_stats_come_from_annotation() {
        let p = profile_from_annotation(SectionName::Baggara, 0, 0, &annotation(4, 3, 7)).unwrap();
        assert_eq!(p.fire, Some(FireFactor::Four));
        assert_eq!(p.melee, Some(MeleeFactor::Three));
        assert_eq!(p.movement, UnitMovement::Land(MovementAllowance::Seven));
        assert!(matches!(p.identity, UnitIdentity::DervishTribal { .. }));
    }

    // §6.51
    #[test]
    fn zero_factor_is_none_not_zero() {
        // A British leader prints no fire factor; an annotation of 0 must
        // become `None`, not `FireFactor(0)`.
        let p =
            profile_from_annotation(SectionName::Kitchener, 0, 0, &annotation(0, 0, 6)).unwrap();
        assert_eq!(p.fire, None);
        assert_eq!(p.melee, None);
        assert_eq!(p.kind, UnitKind::BritishLeaderUnit);
    }

    // §5.24
    #[test]
    fn boat_annotation_yields_split_gunboat_movement() {
        let mut a = annotation(4, 0, 0);
        a.is_boat = true;
        a.movement_upstream = 3;
        a.movement_downstream = 7;
        // British_Army isn't a boat identity, but movement derivation is
        // driven purely by the annotation's is_boat flag.
        let p = profile_from_annotation(SectionName::BritishArmy, 0, 0, &a).unwrap();
        assert_eq!(
            p.movement,
            UnitMovement::Gunboat(GunboatMovement {
                upstream: MovementAllowance::Three,
                downstream: MovementAllowance::Seven,
            })
        );
    }

    // §5.54
    #[test]
    fn brigade_and_battalion_from_column() {
        // col 5 -> brigade 2 (5/4+1), battalion 2 (5%4+1)
        let p =
            profile_from_annotation(SectionName::BritishArmy, 5, 0, &annotation(4, 2, 6)).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 2);
                assert_eq!(brigade.nationality, BrigadeNationality::British);
                assert_eq!(battalion, BattalionOrdinal::Second);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    // §5.54
    #[test]
    fn printed_brigade_designation_overrides_column() {
        // §5.54: a 3E designation overrides the column-derived 2nd British.
        let mut a = annotation(4, 2, 6);
        a.faction = Some(Faction::BritishEgyptian {
            brigade: Brigade::E3,
        });
        let p = profile_from_annotation(SectionName::BritishArmy, 5, 0, &a).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 3);
                assert_eq!(brigade.nationality, BrigadeNationality::Egyptian);
                // Battalion (column-derived) is preserved.
                assert_eq!(battalion, BattalionOrdinal::Second);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    // §5.54
    #[test]
    fn brigade_none_keeps_column_derived_brigade() {
        // Brigade::None leaves the column-derived brigade untouched.
        let mut a = annotation(4, 2, 6);
        a.faction = Some(Faction::BritishEgyptian {
            brigade: Brigade::None,
        });
        let p = profile_from_annotation(SectionName::BritishArmy, 5, 0, &a).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, .. } => {
                assert_eq!(brigade.number, 2);
                assert_eq!(brigade.nationality, BrigadeNationality::British);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    // §5.54
    #[test]
    fn brigade_designation_ignored_for_non_infantry() {
        // A designation on a leader counter must not change its identity.
        let mut a = annotation(0, 0, 15);
        a.faction = Some(Faction::BritishEgyptian {
            brigade: Brigade::B2,
        });
        let p = profile_from_annotation(SectionName::Kitchener, 0, 0, &a).unwrap();
        assert!(matches!(p.identity, UnitIdentity::AngloEgyptianLeader(_)));
    }

    // §9.212
    #[test]
    fn embedded_leaders_resolve_from_their_host_section() {
        // Yakub is the (0,0) counter of the `upper_Jaalin` tribal block, and
        // Osman Digna is the (1,0) counter of the `Hadendowa` block -- neither
        // has a section of its own. They must resolve as leaders, while the
        // other counters in those sections stay tribal.
        let yakub =
            profile_from_annotation(SectionName::UpperJaalin, 0, 0, &annotation(1, 1, 6)).unwrap();
        assert_eq!(
            yakub.identity,
            UnitIdentity::DervishLeader(DervishLeader::Yakub)
        );
        assert_eq!(yakub.kind, UnitKind::DervishLeaderUnit);

        let osman =
            profile_from_annotation(SectionName::Hadendowa, 1, 0, &annotation(1, 1, 6)).unwrap();
        assert_eq!(
            osman.identity,
            UnitIdentity::DervishLeader(DervishLeader::OsmanDigna)
        );

        // A different counter in the same section is still a tribal unit.
        let jaalin =
            profile_from_annotation(SectionName::UpperJaalin, 1, 0, &annotation(1, 1, 6)).unwrap();
        assert!(matches!(
            jaalin.identity,
            UnitIdentity::DervishTribal { .. }
        ));
    }

    // §5.54
    #[test]
    fn ae_infantry_third_battalion_from_col_2() {
        // col=2: (2/4)+1=1 (brigade 1), (2%4)+1=3 → Third ordinal.
        let p =
            profile_from_annotation(SectionName::BritishArmy, 2, 0, &annotation(4, 2, 6)).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 1);
                assert_eq!(battalion, BattalionOrdinal::Third);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    // §5.54
    #[test]
    fn ae_infantry_fourth_battalion_from_col_3() {
        // col=3: (3/4)+1=1 (brigade 1), (3%4)+1=4 → Fourth ordinal.
        let p =
            profile_from_annotation(SectionName::BritishArmy, 3, 0, &annotation(4, 2, 6)).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 1);
                assert_eq!(battalion, BattalionOrdinal::Fourth);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    // §5.54
    #[test]
    fn ae_infantry_brigade_number_three_from_col_8() {
        // col=8: (8/4)+1=3 (brigade 3), (8%4)+1=1 → First ordinal.
        let p =
            profile_from_annotation(SectionName::BritishArmy, 8, 0, &annotation(4, 2, 6)).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 3);
                assert_eq!(battalion, BattalionOrdinal::First);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    // §5.54
    #[test]
    fn section_owner_dervish_sections() {
        assert_eq!(
            section_owner(SectionName::Taiasha),
            Some(crate::Player::Dervish)
        );
        assert_eq!(
            section_owner(SectionName::KhalifaAbdullah),
            Some(crate::Player::Dervish)
        );
        assert_eq!(
            section_owner(SectionName::Baggara),
            Some(crate::Player::Dervish)
        );
        assert_eq!(
            section_owner(SectionName::Hadendowa),
            Some(crate::Player::Dervish)
        );
        assert_eq!(
            section_owner(SectionName::HadendowaForts),
            Some(crate::Player::Dervish)
        );
    }

    // §5.54
    #[test]
    fn section_owner_anglo_egyptian_sections() {
        assert_eq!(
            section_owner(SectionName::BritishArmy),
            Some(crate::Player::AngloEgyptian)
        );
        assert_eq!(
            section_owner(SectionName::EgyptianArmy),
            Some(crate::Player::AngloEgyptian)
        );
        assert_eq!(
            section_owner(SectionName::Kitchener),
            Some(crate::Player::AngloEgyptian)
        );
        assert_eq!(
            section_owner(SectionName::BritishBoats),
            Some(crate::Player::AngloEgyptian)
        );
    }

    // §5.54
    #[test]
    fn section_owner_green_sections_return_none() {
        assert_eq!(section_owner(SectionName::UpperGreen), None);
        assert_eq!(section_owner(SectionName::LowerGreen), None);
    }

    // §5.24
    #[test]
    fn movement_from_annotation_fort_returns_immobile() {
        let a = annotation(0, 0, 6);
        let m = movement_from_annotation(UnitKind::Fort, &a);
        assert_eq!(m, UnitMovement::Immobile);
    }
}

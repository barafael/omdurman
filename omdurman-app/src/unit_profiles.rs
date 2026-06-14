//! Build rules-engine [`UnitProfile`]s for placed counters.
//!
//! A counter is identified on the sprite sheet by `(section_name, col, row)`.
//! Two distinct kinds of information go into a profile:
//!
//! * **Identity / kind / weapon** — *what the unit is* (a Baggara tribe, the
//!   Khalifa, a British brigade battalion, a gunboat). This is not printed on
//!   the counter in a machine-readable way, so it is mapped from the section
//!   name via [`identity_for_section`].
//! * **Numeric factors** — fire / melee / movement. These *are* authored, in
//!   the [`SpriteAnnotation`] the Units-mode editor writes. We read them from
//!   there rather than inventing them.
//!
//! A counter with no annotation, or whose section name we don't recognise,
//! yields `None` — callers must cope with that rather than receiving a
//! fabricated stand-in unit.

use omdurman_rules::{
    BattalionOrdinal, BrigadeId, BrigadeNationality, BritishLeader, DervishLeader, DervishTribe,
    FireFactor, GunboatMovement, MeleeFactor, MovementAllowance, UnitIdentity, UnitKind,
    UnitMovement, UnitProfile, WeaponClass,
};
use omdurman_types::{Brigade, SectionName, SpriteAnnotation};

/// The fixed identity facts about a counter, independent of its printed
/// numeric factors. Weapon class and kind follow from the identity.
struct Classification {
    kind: UnitKind,
    identity: UnitIdentity,
    weapon: WeaponClass,
}

/// Build a [`UnitProfile`] from a counter's section/grid identity plus its
/// authored [`SpriteAnnotation`] stats.
///
/// Returns `None` when the section name is not recognised — there is no
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
    let identity = apply_brigade_designation(identity, annotation.brigade);

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
/// picked on its counter, e.g. [`Brigade::B2`] → 2nd British, [`Brigade::E3`]
/// → 3rd Egyptian (rulebook §5.54). Non-infantry identities and
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
                .unwrap_or(MovementAllowance::Impassable),
            downstream: MovementAllowance::try_from(a.movement_downstream.max(0) as u16)
                .unwrap_or(MovementAllowance::Impassable),
        })
    } else {
        UnitMovement::Land(
            MovementAllowance::try_from(a.movement.max(0) as u16)
                .unwrap_or(MovementAllowance::Impassable),
        )
    }
}

/// Map a sprite-sheet section name (and column, for multi-brigade sheets) to
/// the unit's identity, kind, and weapon class. `None` for unrecognised
/// sections.
fn identity_for_section(section_name: SectionName, col: u32, row: u32) -> Option<Classification> {
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

    match section_name {
        // ── Dervish leaders ──────────────────────────────────────────
        SectionName::KhalifaAbdullah => dervish_leader(DervishLeader::KhalifaAbdullah),
        SectionName::Sherif => dervish_leader(DervishLeader::Sherif),
        SectionName::AliWadHelu => dervish_leader(DervishLeader::AliWadHelu),
        SectionName::SheikElDin => dervish_leader(DervishLeader::SheikElDin),
        SectionName::Yakub => dervish_leader(DervishLeader::Yakub),
        SectionName::OsmanDigna => dervish_leader(DervishLeader::OsmanDigna),

        // ── Dervish foot tribes ──────────────────────────────────────
        SectionName::Taiasha => dervish_tribe(DervishTribe::Taiasha),
        SectionName::Hadendowa => dervish_tribe(DervishTribe::Hadendowa),
        SectionName::Baggara => dervish_tribe(DervishTribe::Baggara),
        SectionName::Jehadia => dervish_tribe(DervishTribe::Jehadia),
        SectionName::Mulazmin => dervish_tribe(DervishTribe::Mulazmin),
        SectionName::Kehena => dervish_tribe(DervishTribe::Kehena),
        SectionName::Degheim => dervish_tribe(DervishTribe::Degheim),
        SectionName::Danagla => dervish_tribe(DervishTribe::Danagla),
        SectionName::UpperJaalin | SectionName::LowerJaalin => dervish_tribe(DervishTribe::Jaalin),

        // ── Dervish artillery ────────────────────────────────────────
        SectionName::HadendowaForts => c(
            UnitKind::Fort,
            UnitIdentity::DervishFort,
            WeaponClass::Artillery,
        ),

        // ── Anglo-Egyptian infantry brigades ─────────────────────────
        SectionName::BritishArmy => ae_infantry(BrigadeNationality::British, col),
        SectionName::EgyptianArmy => ae_infantry(BrigadeNationality::Egyptian, col),

        // ── Anglo-Egyptian leaders ───────────────────────────────────
        SectionName::Kitchener => c(
            UnitKind::BritishLeaderUnit,
            UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener),
            WeaponClass::Melee,
        ),

        SectionName::UpperGreen | SectionName::LowerGreen | SectionName::BritishBoats => None,
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
            brigade: Brigade::None,
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

    #[test]
    fn unknown_section_returns_none() {
        // There is no SectionName variant for "not_a_real_section", so we
        // test with a section that exists but maps to nothing.
        assert!(
            profile_from_annotation(SectionName::BritishBoats, 0, 0, &annotation(1, 1, 1))
                .is_none()
        );
    }

    #[test]
    fn tribe_stats_come_from_annotation() {
        let p = profile_from_annotation(SectionName::Baggara, 0, 0, &annotation(4, 3, 7)).unwrap();
        assert_eq!(p.fire, Some(FireFactor::Four));
        assert_eq!(p.melee, Some(MeleeFactor::Three));
        assert_eq!(p.movement, UnitMovement::Land(MovementAllowance::Seven));
        assert!(matches!(p.identity, UnitIdentity::DervishTribal { .. }));
    }

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

    #[test]
    fn brigade_and_battalion_from_column() {
        // col 5 → brigade 2 (5/4+1), battalion 2 (5%4+1)
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

    #[test]
    fn printed_brigade_designation_overrides_column() {
        // §5.54: a 3E designation overrides the column-derived 2nd British.
        let mut a = annotation(4, 2, 6);
        a.brigade = Brigade::E3;
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

    #[test]
    fn brigade_none_keeps_column_derived_brigade() {
        // Brigade::None leaves the column-derived brigade untouched.
        let mut a = annotation(4, 2, 6);
        a.brigade = Brigade::None;
        let p = profile_from_annotation(SectionName::BritishArmy, 5, 0, &a).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, .. } => {
                assert_eq!(brigade.number, 2);
                assert_eq!(brigade.nationality, BrigadeNationality::British);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    #[test]
    fn brigade_designation_ignored_for_non_infantry() {
        // A designation on a leader counter must not change its identity.
        let mut a = annotation(0, 0, 15);
        a.brigade = Brigade::B2;
        let p = profile_from_annotation(SectionName::Kitchener, 0, 0, &a).unwrap();
        assert!(matches!(p.identity, UnitIdentity::AngloEgyptianLeader(_)));
    }

    #[test]
    fn embedded_leaders_resolve_from_their_host_section() {
        // Yakub is the (0,0) counter of the `upper_Jaalin` tribal block, and
        // Osman Digna is the (1,0) counter of the `Hadendowa` block — neither
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
}

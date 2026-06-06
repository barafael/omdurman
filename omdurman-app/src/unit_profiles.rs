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
use omdurman_types::{Brigade, SpriteAnnotation};

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
    section_name: &str,
    col: u32,
    annotation: &SpriteAnnotation,
) -> Option<UnitProfile> {
    let Classification {
        kind,
        identity,
        weapon,
    } = identity_for_section(section_name, col)?;

    // The brigade designation printed on the counter (e.g. 2B, 3E) is the
    // authoritative source for an infantry unit's brigade (rulebook §5.54);
    // when set it overrides the column-derived default from
    // `identity_for_section`.
    let identity = apply_brigade_designation(identity, annotation.brigade);

    Some(UnitProfile {
        kind,
        identity,
        weapon,
        fire: factor(annotation.fire).map(FireFactor),
        melee: factor(annotation.melee).map(MeleeFactor),
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
            nationality: rules_nationality(nationality),
        },
        battalion,
    }
}

/// Map the annotation's [`Brigade`] nationality to the rules-engine
/// [`BrigadeNationality`]. (The annotation enum has no `Friendlies` — those
/// are modelled by section identity, not a printed brigade designation.)
fn rules_nationality(n: omdurman_types::BrigadeNationality) -> BrigadeNationality {
    match n {
        omdurman_types::BrigadeNationality::British => BrigadeNationality::British,
        omdurman_types::BrigadeNationality::Egyptian => BrigadeNationality::Egyptian,
        omdurman_types::BrigadeNationality::Sudanese => BrigadeNationality::Sudanese,
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
            upstream: MovementAllowance(a.movement_upstream.max(0) as u16),
            downstream: MovementAllowance(a.movement_downstream.max(0) as u16),
        })
    } else {
        UnitMovement::Land(MovementAllowance(a.movement.max(0) as u16))
    }
}

/// Map a sprite-sheet section name (and column, for multi-brigade sheets) to
/// the unit's identity, kind, and weapon class. `None` for unrecognised
/// sections.
fn identity_for_section(section_name: &str, col: u32) -> Option<Classification> {
    let c = |kind, identity, weapon| {
        Some(Classification {
            kind,
            identity,
            weapon,
        })
    };

    match section_name {
        // ── Dervish leaders ──────────────────────────────────────────
        "Khalifa_Abdullah" => dervish_leader(DervishLeader::KhalifaAbdullah),
        "Sherif" => dervish_leader(DervishLeader::Sherif),
        "Ali_Wad_Helu" => dervish_leader(DervishLeader::AliWadHelu),
        "Sheik_El_Din" => dervish_leader(DervishLeader::SheikElDin),
        "Yakub" => dervish_leader(DervishLeader::Yakub),
        "Osman_Digna" => dervish_leader(DervishLeader::OsmanDigna),

        // ── Dervish foot tribes ──────────────────────────────────────
        "Taiasha" => dervish_tribe(DervishTribe::Taiasha),
        "Hadendowa" => dervish_tribe(DervishTribe::Hadendowa),
        "Baggara" => dervish_tribe(DervishTribe::Baggara),
        "Jehadia" => dervish_tribe(DervishTribe::Jehadia),
        "Mulazmin" => dervish_tribe(DervishTribe::Mulazmin),
        "Kehena" => dervish_tribe(DervishTribe::Kehena),
        "Degheim" => dervish_tribe(DervishTribe::Degheim),
        "Danagla" => dervish_tribe(DervishTribe::Danagla),
        "upper_Jaalin" | "lower_Jaalin" => dervish_tribe(DervishTribe::Jaalin),

        // ── Dervish artillery ────────────────────────────────────────
        "Hadendowa_Guns" => c(
            UnitKind::Artillery,
            UnitIdentity::DervishArtillery,
            WeaponClass::Artillery,
        ),

        // ── Anglo-Egyptian infantry brigades ─────────────────────────
        "British_Army" => ae_infantry(BrigadeNationality::British, col),
        "Egyptian_Army" => ae_infantry(BrigadeNationality::Egyptian, col),

        // ── Anglo-Egyptian leaders ───────────────────────────────────
        "Kitchener" => c(
            UnitKind::BritishLeaderUnit,
            UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener),
            WeaponClass::Melee,
        ),

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
    let battalion = BattalionOrdinal((col % 4) as u8 + 1);
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

    fn annotation(fire: i32, melee: i32, movement: i32) -> SpriteAnnotation {
        SpriteAnnotation {
            color: omdurman_types::SpriteColor::BlackWhite,
            faction: omdurman_types::Faction::Independent,
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
        assert!(profile_from_annotation("not_a_real_section", 0, &annotation(1, 1, 1)).is_none());
    }

    #[test]
    fn tribe_stats_come_from_annotation() {
        let p = profile_from_annotation("Baggara", 0, &annotation(2, 3, 6)).unwrap();
        assert_eq!(p.fire, Some(FireFactor(2)));
        assert_eq!(p.melee, Some(MeleeFactor(3)));
        assert_eq!(p.movement, UnitMovement::Land(MovementAllowance(6)));
        assert!(matches!(p.identity, UnitIdentity::DervishTribal { .. }));
    }

    #[test]
    fn zero_factor_is_none_not_zero() {
        // A British leader prints no fire factor; an annotation of 0 must
        // become `None`, not `FireFactor(0)`.
        let p = profile_from_annotation("Kitchener", 0, &annotation(0, 0, 6)).unwrap();
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
        let p = profile_from_annotation("British_Army", 0, &a).unwrap();
        assert_eq!(
            p.movement,
            UnitMovement::Gunboat(GunboatMovement {
                upstream: MovementAllowance(3),
                downstream: MovementAllowance(7),
            })
        );
    }

    #[test]
    fn brigade_and_battalion_from_column() {
        // col 5 → brigade 2 (5/4+1), battalion 2 (5%4+1)
        let p = profile_from_annotation("British_Army", 5, &annotation(4, 2, 6)).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 2);
                assert_eq!(brigade.nationality, BrigadeNationality::British);
                assert_eq!(battalion, BattalionOrdinal(2));
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    #[test]
    fn printed_brigade_designation_overrides_column() {
        // §5.54: a 3E designation overrides the column-derived 2nd British.
        let mut a = annotation(4, 2, 6);
        a.brigade = Brigade::E3;
        let p = profile_from_annotation("British_Army", 5, &a).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 3);
                assert_eq!(brigade.nationality, BrigadeNationality::Egyptian);
                // Battalion (column-derived) is preserved.
                assert_eq!(battalion, BattalionOrdinal(2));
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    #[test]
    fn brigade_none_keeps_column_derived_brigade() {
        // Brigade::None leaves the column-derived brigade untouched.
        let mut a = annotation(4, 2, 6);
        a.brigade = Brigade::None;
        let p = profile_from_annotation("British_Army", 5, &a).unwrap();
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
        let p = profile_from_annotation("Kitchener", 0, &a).unwrap();
        assert!(matches!(p.identity, UnitIdentity::AngloEgyptianLeader(_)));
    }
}

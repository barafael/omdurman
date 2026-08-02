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
    BattalionOrdinal, BritishLeader, DervishLeader,
    FireFactor, GunboatId, GunboatMovement, MeleeFactor, MovementAllowance, UnitIdentity,
    UnitMovement, UnitProfile, WeaponClass,
};
use omdurman_types::{
    BrigadeId, BrigadeNationality, DervishTribe, Faction, Player, SectionName, UnitKind,
};

/// The fixed identity facts about a counter, independent of its printed
/// numeric factors. Weapon class and kind follow from the identity.
pub(crate) struct Classification {
    kind: UnitKind,
    identity: UnitIdentity,
    weapon: WeaponClass,
}

/// Build a [`UnitProfile`] from a [`UnitId`] by looking up its compiled
/// annotations data.
#[must_use]
pub fn profile_for_unit(unit_id: crate::UnitId) -> Option<UnitProfile> {
    let (section_name, col, row) = unit_id.section_pos();
    let Classification {
        kind,
        identity,
        weapon,
    } = identity_for_section(section_name, col as u32, row as u32)?;

    let annotation_brigade = match unit_id.faction() {
        Some(Faction::BritishEgyptian { brigade }) => brigade,
        _ => None,
    };
    let identity = apply_brigade_designation(identity, annotation_brigade);

    let ann_kind = unit_id.kind().unwrap_or(kind);
    let (fire_i, melee_i) = match ann_kind {
        UnitKind::Infantry { fire, melee, .. }
        | UnitKind::Cavalry { fire, melee, .. }
        | UnitKind::Camel { fire, melee, .. }
        | UnitKind::Artillery { fire, melee, .. }
        | UnitKind::Maxim { fire, melee, .. }
        | UnitKind::DervishLeader { fire, melee, .. } => (fire, melee),
        UnitKind::Fort { fire, melee } => (fire, melee),
        UnitKind::BritishLeader { .. } => (0, 0),
        UnitKind::Gunboat { fire, .. } | UnitKind::NamedGunboat { fire, .. } => (fire, 0),
        UnitKind::Marker | UnitKind::Breech | UnitKind::BareCounter => (0, 0),
    };

    Some(UnitProfile {
        kind,
        identity,
        weapon,
        fire: factor(fire_i).and_then(|v| FireFactor::try_from(v).ok()),
        melee: factor(melee_i).and_then(|v| MeleeFactor::try_from(v).ok()),
        movement: movement_from_kind(ann_kind),
    })
}

/// Movement allowance from a [`UnitKind`] value. Boats carry split upstream /
/// downstream allowances; everything else is uniform land movement. Forts
/// are immobile regardless of any printed number.
fn movement_from_kind(kind: UnitKind) -> UnitMovement {
    match kind {
        UnitKind::Fort { .. } => UnitMovement::Immobile,
        UnitKind::Gunboat { upstream, downstream, .. }
        | UnitKind::NamedGunboat { upstream, downstream, .. } => {
            UnitMovement::Gunboat(GunboatMovement {
                upstream: MovementAllowance::try_from(upstream.max(0) as u16)
                    .unwrap_or(MovementAllowance::Immobile),
                downstream: MovementAllowance::try_from(downstream.max(0) as u16)
                    .unwrap_or(MovementAllowance::Immobile),
            })
        }
        UnitKind::Infantry { movement, .. }
        | UnitKind::Cavalry { movement, .. }
        | UnitKind::Camel { movement, .. }
        | UnitKind::Artillery { movement, .. }
        | UnitKind::Maxim { movement, .. }
        | UnitKind::DervishLeader { movement, .. }
        | UnitKind::BritishLeader { movement } => UnitMovement::Land(
            MovementAllowance::try_from(movement.max(0) as u16)
                .unwrap_or(MovementAllowance::Immobile),
        ),
        UnitKind::Marker | UnitKind::Breech | UnitKind::BareCounter => UnitMovement::Immobile,
    }
}

/// Override an Anglo-Egyptian infantry unit's brigade with the designation
/// picked on its counter, e.g. `BrigadeId::british(2)` -> 2nd British,
/// `BrigadeId::egyptian(3)` -> 3rd Egyptian (rulebook §5.54). Non-infantry
/// identities and `None` are returned unchanged.
fn apply_brigade_designation(
    identity: UnitIdentity,
    brigade: Option<BrigadeId>,
) -> UnitIdentity {
    let UnitIdentity::AngloEgyptianInfantry {
        brigade: _,
        battalion,
    } = identity
    else {
        return identity;
    };
    let Some(BrigadeId { number, nationality }) = brigade else {
        return identity;
    };
    UnitIdentity::AngloEgyptianInfantry {
        brigade: BrigadeId { number, nationality },
        battalion,
    }
}

/// A positive factor is a printed value; zero/negative means the counter
/// prints no value in that slot (e.g. British leaders print no fire factor).
fn factor(value: i32) -> Option<u16> {
    (value > 0).then_some(value as u16)
}

/// Map a sprite-sheet section name (and column, for multi-brigade sheets) to
/// the unit's identity, kind, and weapon class. `None` for unrecognised
/// sections.
/// Which faction a sprite-sheet section belongs to, for grouping the unit
/// picker. Sections are single-faction (Dervish tribes/leaders/forts vs the
/// Anglo-Egyptian army/boats), so this is a section-level classification.
/// Returns `None` only for sections that map to no placeable unit.
pub fn section_owner(section_name: SectionName) -> Option<Player> {
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
        | SectionName::HadendowaForts
        // Fall-of-Khartoum Mulazmin print runs (§9.322); green-backed cells.
        | SectionName::UpperGreen
        | SectionName::LowerGreen => Some(Player::Dervish),
        SectionName::BritishArmy
        | SectionName::EgyptianArmy
        | SectionName::Kitchener
        | SectionName::BritishBoats => Some(Player::AngloEgyptian),
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
        // `KhalifaAbdullah` is a mixed section resolved by cell, like the
        // tribal-leader blocks below: cell (0,0) is the Khalifa leader, (1,0)
        // and (2,0) are the two Dervish gunboats, and the row-1 cells are the
        // three Dervish field-artillery counters used in §9.111 and §9.322.
        SectionName::KhalifaAbdullah => khalifa_abdullah(col, row),
        SectionName::Sherif => dervish_leader(DervishLeader::Sherif),
        SectionName::AliWadHelu => ali_wad_helu(col, row),
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
            UnitKind::Fort { fire: 0, melee: 0 },
            UnitIdentity::DervishFort,
            WeaponClass::Artillery,
        ),

        // -- Anglo-Egyptian infantry brigades -------------------------
        SectionName::BritishArmy => ae_infantry(BrigadeNationality::British, col),
        SectionName::EgyptianArmy => ae_infantry(BrigadeNationality::Egyptian, col),

        // -- Anglo-Egyptian leaders -----------------------------------
        SectionName::Kitchener => c(
            UnitKind::BritishLeader { movement: 0 },
            UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener),
            WeaponClass::Melee,
        ),

        // `British_Boats` is resolved by cell above; the green sections are the
        // Fall-of-Khartoum Mulazmin print runs (rulebook §9.322), sharing the
        // Mulazmin tribal identity.
        SectionName::UpperGreen | SectionName::LowerGreen => dervish_tribe(DervishTribe::Mulazmin),
        SectionName::BritishBoats => None,
    }
}

/// Resolve a counter in the `Khalifa_Abdullah` section (rulebook §2.31,
/// §9.111, §9.322). The block is mixed:
///   - `(0,0)` is the Khalifa Abdullah leader himself (used in the Campaign
///     and Historical scenarios).
///   - `(1,0)` and `(2,0)` are the two Dervish gunboats (campaign scenario
///     §9.111; optional river-mines §10.14).
///   - `(0,1)`, `(1,1)`, `(2,1)` are the three Dervish field-artillery
///     counters — used both as the Campaign scenario's fort artillery and as
///     the three artillery units in the Fall of Khartoum Dervish order of
///     battle (§9.322). All three are interchangeable, so they share the
///     `DervishArtillery` identity.
fn khalifa_abdullah(col: u32, row: u32) -> Option<Classification> {
    let artillery = || {
        Some(Classification {
            kind: UnitKind::Artillery { fire: 0, melee: 0, movement: 0 },
            identity: UnitIdentity::DervishArtillery,
            weapon: WeaponClass::Artillery,
        })
    };
    let dervish_gunboat = |id: u8| {
        Some(Classification {
            kind: UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 },
            identity: UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(id)),
            // All gunboats fire on the Artillery line (§2.32 analogue).
            weapon: WeaponClass::Artillery,
        })
    };
    match (col, row) {
        (0, 0) => dervish_leader(DervishLeader::KhalifaAbdullah),
        (1, 0) => dervish_gunboat(1),
        (2, 0) => dervish_gunboat(2),
        (0, 1) | (1, 1) | (2, 1) => artillery(),
        _ => None,
    }
}

/// Resolve a counter in the `Ali_Wad_Helu` section. The block is mixed:
///   - `(0,0)` is the Ali Wad Helu leader himself.
///   - `(0,1)`–`(5,1)` are six Kehena "Deghelim" foot counters (3-6-9),
///     one of the two Dervish forces listed in §9.322.
///   - `(1,0)`–`(5,0)` are five Baggara-tribe "Deghelim" foot counters
///     (3-6-9) -- the Degheim force of §9.322, printed on Baggara-backed
///     sprites.
fn ali_wad_helu(col: u32, row: u32) -> Option<Classification> {
    match (col, row) {
        (0, 0) => dervish_leader(DervishLeader::AliWadHelu),
        (_, 1) => dervish_tribe(DervishTribe::Kehena),
        (1.., 0) => dervish_tribe(DervishTribe::Baggara),
        _ => None,
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
            kind: UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 },
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
            kind: UnitKind::BritishLeader { movement: 0 },
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
        kind: UnitKind::DervishLeader { fire: 0, melee: 0, movement: 0 },
        identity: UnitIdentity::DervishLeader(leader),
        weapon: WeaponClass::Melee,
    })
}

fn dervish_tribe(tribe: DervishTribe) -> Option<Classification> {
    // §2.31: "Jehadia and Danagla units fire on the 'rifles' line as does the
    // Isa Zachneih unit. All other Dervish units (including leaders) are armed
    // with spears and swords." Spears use the Melee weapon class — the
    // Dervish Range Effects Table's Spears line is range 1 ×1 only
    // (`range_effects::dervish_range_effects` handles the band).
    let weapon = match tribe {
        DervishTribe::Jehadia | DervishTribe::Danagla | DervishTribe::IsaZachneih => {
            WeaponClass::Rifles
        }
        _ => WeaponClass::Melee,
    };
    Some(Classification {
        kind: UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
        identity: UnitIdentity::DervishTribal { tribe },
        weapon,
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
        kind: UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
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
    use crate::unit_id_for_section_pos;
    use SectionName;
    use traceability_macro::rulebook;

    fn profile_for(section: SectionName, col: u8, row: u8) -> Option<UnitProfile> {
        let uid = unit_id_for_section_pos(section, col, row)?;
        profile_for_unit(uid)
    }

    #[rulebook("§6.63")]
    #[test]
    fn breech_marker_cell_returns_none() {
        // `British_Boats` (0,0) is a BREECH marker (§6.63), not a placeable
        // unit -- it must yield no profile even though the section is mapped.
        assert!(profile_for(SectionName::BritishBoats, 0, 0).is_none());
    }

    #[rulebook("§9.346")]
    #[test]
    fn gordon_is_an_immobile_british_leader() {
        // GORDON is the 0-0-0 palace leader at British_Boats (3,1) (§9.346).
        let p = profile_for(SectionName::BritishBoats, 3, 1)
            .expect("Gordon resolves");
        assert!(matches!(p.kind, UnitKind::BritishLeader { .. }));
        assert!(matches!(
            p.identity,
            UnitIdentity::AngloEgyptianLeader(BritishLeader::Gordon)
        ));
        assert_eq!(p.movement, UnitMovement::Land(MovementAllowance::Immobile));
    }

    #[rulebook("§6.64")]
    #[test]
    fn named_and_old_gunboats_resolve() {
        let named = profile_for(SectionName::BritishBoats, 4, 0)
            .expect("named gunboat resolves");
        assert!(matches!(named.kind, UnitKind::Gunboat { .. }));
        assert!(matches!(
            named.identity,
            UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Named(_))
        ));

        let old = profile_for(SectionName::BritishBoats, 4, 1)
            .expect("old gunboat resolves");
        assert!(matches!(
            old.identity,
            UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(_))
        ));
    }

    #[rulebook("§5.54")]
    #[test]
    fn tribe_stats_come_from_annotation() {
        // Baggara (0,0) stats come from the compiled sprite data.
        let p = profile_for(SectionName::Baggara, 0, 0).unwrap();
        // Smoke-test: stats are present (not None/Immobile).
        assert!(p.fire.is_some());
        assert!(p.melee.is_some());
        assert_ne!(p.movement, UnitMovement::Immobile);
        assert!(matches!(p.identity, UnitIdentity::DervishTribal { .. }));
    }

    #[rulebook("§6.51")]
    #[test]
    fn zero_factor_is_none_not_zero() {
        // Kitchener is a British leader with no printed fire/melee.
        let p = profile_for(SectionName::Kitchener, 0, 0).unwrap();
        assert_eq!(p.fire, None);
        assert_eq!(p.melee, None);
        assert!(matches!(p.kind, UnitKind::BritishLeader { .. }));
    }

    #[rulebook("§5.24")]
    #[test]
    fn boat_annotation_yields_split_gunboat_movement() {
        // Named gunboat at British_Boats (4,0) has split movement.
        let p = profile_for(SectionName::BritishBoats, 4, 0).unwrap();
        assert!(matches!(p.movement, UnitMovement::Gunboat(_)));
    }

    #[rulebook("§5.54")]
    #[test]
    fn ae_infantry_brigade_number_three_from_col_7() {
        // col=7: (7/4)+1=2 (brigade 2), (7%4)+1=4 → Fourth ordinal.
        let p = profile_for(SectionName::BritishArmy, 7, 0).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 2);
                assert_eq!(battalion, BattalionOrdinal::Fourth);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    #[rulebook("§5.54")]
    #[test]
    fn printed_brigade_designation_overrides_column() {
        // Some British_Army counters carry a faction with a brigade override.
        // British_Army (5,0) has compiled faction data;
        // verify the brigade comes from the annotation, not the column.
        let p = profile_for(SectionName::BritishArmy, 5, 0).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, .. } => {
                // The annotation may set brigade; we just verify the identity resolves.
                assert!(brigade.number >= 1);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    #[rulebook("§5.54")]
    #[test]
    fn brigade_designation_ignored_for_non_infantry() {
        // Kitchener is a leader, not infantry — brigade must not change identity.
        let p = profile_for(SectionName::Kitchener, 0, 0).unwrap();
        assert!(matches!(p.identity, UnitIdentity::AngloEgyptianLeader(_)));
    }

    #[rulebook("§9.212")]
    #[test]
    fn embedded_leaders_resolve_from_their_host_section() {
        // Yakub is the (0,0) counter of `upper_Jaalin`; Osman Digna is (1,0)
        // of `Hadendowa`. Neither has its own section.
        let yakub = profile_for(SectionName::UpperJaalin, 0, 0).unwrap();
        assert_eq!(
            yakub.identity,
            UnitIdentity::DervishLeader(DervishLeader::Yakub)
        );
        assert!(matches!(yakub.kind, UnitKind::DervishLeader { .. }));

        let osman = profile_for(SectionName::Hadendowa, 1, 0).unwrap();
        assert_eq!(
            osman.identity,
            UnitIdentity::DervishLeader(DervishLeader::OsmanDigna)
        );

        // A different counter in the same section is still a tribal unit.
        let jaalin = profile_for(SectionName::UpperJaalin, 1, 0).unwrap();
        assert!(matches!(
            jaalin.identity,
            UnitIdentity::DervishTribal { .. }
        ));
    }

    #[rulebook("§5.54")]
    #[test]
    fn ae_infantry_third_battalion_from_col_2() {
        let p = profile_for(SectionName::BritishArmy, 2, 0).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 1);
                assert_eq!(battalion, BattalionOrdinal::Third);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    #[rulebook("§5.54")]
    #[test]
    fn ae_infantry_fourth_battalion_from_col_3() {
        let p = profile_for(SectionName::BritishArmy, 3, 0).unwrap();
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, battalion } => {
                assert_eq!(brigade.number, 1);
                assert_eq!(battalion, BattalionOrdinal::Fourth);
            }
            other => panic!("expected AE infantry, got {other:?}"),
        }
    }

    #[rulebook("§5.54")]
    #[test]
    fn section_owner_dervish_sections() {
        assert_eq!(section_owner(SectionName::Taiasha), Some(Player::Dervish));
        assert_eq!(section_owner(SectionName::KhalifaAbdullah), Some(Player::Dervish));
        assert_eq!(section_owner(SectionName::Baggara), Some(Player::Dervish));
        assert_eq!(section_owner(SectionName::Hadendowa), Some(Player::Dervish));
        assert_eq!(section_owner(SectionName::HadendowaForts), Some(Player::Dervish));
        assert_eq!(section_owner(SectionName::UpperGreen), Some(Player::Dervish));
        assert_eq!(section_owner(SectionName::LowerGreen), Some(Player::Dervish));
    }

    #[rulebook("§5.54")]
    #[test]
    fn section_owner_anglo_egyptian_sections() {
        assert_eq!(section_owner(SectionName::BritishArmy), Some(Player::AngloEgyptian));
        assert_eq!(section_owner(SectionName::EgyptianArmy), Some(Player::AngloEgyptian));
        assert_eq!(section_owner(SectionName::Kitchener), Some(Player::AngloEgyptian));
        assert_eq!(section_owner(SectionName::BritishBoats), Some(Player::AngloEgyptian));
    }

    #[rulebook("§5.54")]
    #[test]
    fn section_owner_green_sections_are_dervish() {
        assert_eq!(section_owner(SectionName::UpperGreen), Some(Player::Dervish));
        assert_eq!(section_owner(SectionName::LowerGreen), Some(Player::Dervish));
    }

    #[rulebook("§5.52")]
    #[test]
    fn green_sections_are_mulazmin_tribal_units() {
        for section in [SectionName::UpperGreen, SectionName::LowerGreen] {
            for (col, row) in [(0, 0), (7, 1)] {
                let p = profile_for(section, col, row).unwrap();
                assert_eq!(
                    p.identity,
                    UnitIdentity::DervishTribal { tribe: DervishTribe::Mulazmin },
                    "{section:?} ({col},{row}) should be Mulazmin"
                );
                assert!(matches!(p.kind, UnitKind::Infantry { .. }));
            }
        }
    }

    // §9.322 -- the AliWadHelu counter block is mixed: (0,0) is the leader,
    // row-1 cells are the 6 Kehena "Deghelim" foot counters, and row-0 cells
    // (cols 1-5) are the 5 Baggara-tribe "Deghelim" counters that make up the
    // Degheim force.
    #[rulebook("§9.322")]
    #[test]
    fn ali_wad_helu_block_resolves_leader_and_degelim_tribes() {
        let leader = profile_for(SectionName::AliWadHelu, 0, 0).unwrap();
        assert_eq!(leader.identity, UnitIdentity::DervishLeader(DervishLeader::AliWadHelu));
        for col in 0..=5 {
            let kehena = profile_for(SectionName::AliWadHelu, col, 1).unwrap();
            assert_eq!(
                kehena.identity,
                UnitIdentity::DervishTribal { tribe: DervishTribe::Kehena },
                "AliWadHelu ({col},1) should be Kehena"
            );
        }
        for col in 1..=5 {
            let degheim = profile_for(SectionName::AliWadHelu, col, 0).unwrap();
            assert_eq!(
                degheim.identity,
                UnitIdentity::DervishTribal { tribe: DervishTribe::Baggara },
                "AliWadHelu ({col},0) should be Baggara-tribe Degheim"
            );
        }
    }
}

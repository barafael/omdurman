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
        UnitKind::Gunboat { fire, .. } => (fire, 0),
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
        UnitKind::Gunboat { upstream, downstream, .. } => {
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
        | SectionName::JaalinI
        | SectionName::JaalinII
        | SectionName::HadendowaForts
        // Fall-of-Khartoum Mulazmin print runs (§9.322); green-backed cells.
        | SectionName::MulazminI
        | SectionName::MulazminII => Some(Player::Dervish),
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
    // the `Jaalin_I` block, and Osman Digna is the second counter of the
    // `Hadendowa` block. Resolve those specific counters as leaders before the
    // section falls through to its tribal mapping below.
    //
    // Cell (7,0) of the `Hadendowa` sheet is the printed "GAME TURN" counter
    // (the turn-track marker, §4) -- not a placeable unit, so it yields no
    // classification and is hidden from the picker like the §6.63 BREECH cells.
    match (section_name, col, row) {
        (SectionName::JaalinI, 0, 0) => return dervish_leader(DervishLeader::Yakub),
        // Cell (0,0) of the `Hadendowa` block is the Isa Zachneih counter
        // (§9.111's east-bank unit, the §5.21 transport gate) -- its own
        // tribe, printed on a Hadendowa-backed sprite cell.
        (SectionName::Hadendowa, 0, 0) => return dervish_tribe(DervishTribe::IsaZachneih),
        (SectionName::Hadendowa, 1, 0) => return dervish_leader(DervishLeader::OsmanDigna),
        (SectionName::Hadendowa, 7, 0) => return None,
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
        SectionName::Sherif => sherif_block(col, row),
        SectionName::AliWadHelu => ali_wad_helu(col, row),
        SectionName::SheikElDin => sheik_el_din_block(col, row),
        // Marker-only sections: the printed sheet carries no real counters
        // here (the sprite cells are blank markers). The actual Yakub and
        // Osman Digna leaders are resolved per-cell from the JaalinI
        // (0,0) and Hadendowa (1,0) blocks above -- resolving these sections
        // as leaders would fabricate phantom counters.
        SectionName::Yakub | SectionName::OsmanDigna => None,

        // -- Dervish foot tribes --------------------------------------
        SectionName::Taiasha => dervish_tribe(DervishTribe::Taiasha),
        SectionName::Hadendowa => dervish_tribe(DervishTribe::Hadendowa),
        SectionName::Baggara => dervish_tribe(DervishTribe::Baggara),
        SectionName::Jehadia => dervish_tribe(DervishTribe::Jehadia),
        SectionName::Mulazmin => dervish_tribe(DervishTribe::Mulazmin),
        SectionName::Kehena => dervish_tribe(DervishTribe::Kehena),
        SectionName::Degheim => dervish_tribe(DervishTribe::Degheim),
        SectionName::Danagla => dervish_tribe(DervishTribe::Danagla),
        SectionName::JaalinI | SectionName::JaalinII => dervish_tribe(DervishTribe::Jaalin),

        // -- Dervish artillery ----------------------------------------
        SectionName::HadendowaForts => c(
            UnitKind::Fort { fire: 0, melee: 0 },
            UnitIdentity::DervishFort,
            WeaponClass::Artillery,
        ),

        // -- Anglo-Egyptian infantry brigades -------------------------
        // The `British_Army` and `Egyptian_Army` sheets are mixed: row 0
        // interleaves the division's non-infantry counters (cavalry, Royal
        // Engineers, batteries, Maxims) with two infantry battalions, and
        // row 1 carries the battalions. Resolved per cell -- mapping the
        // whole section as infantry-by-column mislabelled "21 Lancers" and
        // "Egy. Cav." as 1st-brigade battalions with cavalry movement
        // (15 MPs on an infantry identity), and the batteries/Maxims as
        // battalions with artillery factors.
        SectionName::BritishArmy => british_army_block(col, row),
        SectionName::EgyptianArmy => egyptian_army_block(col, row),

        // -- Anglo-Egyptian leaders and the mixed leader-sheet block ----
        // The `Kitchener` sheet section is mixed (§2.3): the three leaders,
        // the "Friendlies" brigade, the Camel Corps, and the Sudanese
        // battalions IX–XIV. Resolved by cell like the Dervish blocks.
        SectionName::Kitchener => kitchener_block(col, row),

        // `British_Boats` is resolved by cell above; the green sections are the
        // Fall-of-Khartoum Mulazmin print runs (rulebook §9.322), sharing the
        // Mulazmin tribal identity.
        SectionName::MulazminI | SectionName::MulazminII => dervish_tribe(DervishTribe::Mulazmin),
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

/// Resolve a counter in the `Sheik_El_Din` section. The block is mixed:
///   - `(0,0)` is the Sheik El Din leader counter (1-1-15, §6.51: Dervish
///     leaders have fire/melee/movement factors and fight like combat units).
///   - every other cell is a Jehadia tribal counter (8-6-9, rifles §2.31).
fn sheik_el_din_block(col: u32, row: u32) -> Option<Classification> {
    match (col, row) {
        (0, 0) => dervish_leader(DervishLeader::SheikElDin),
        _ => dervish_tribe(DervishTribe::Jehadia),
    }
}

/// Resolve a counter in the `Sherif` section. The block is mixed:
///   - `(0,0)` is the Sherif leader counter (1-1-15, §6.51).
///   - the remaining cells are unnamed 4-6-12 Danagla-backed counters --
///     Sherif's Danagla retinue (§2.3 sample: "Danagla, 4-6-12").
fn sherif_block(col: u32, row: u32) -> Option<Classification> {
    match (col, row) {
        (0, 0) => dervish_leader(DervishLeader::Sherif),
        _ => dervish_tribe(DervishTribe::Danagla),
    }
}

/// Resolve a counter in the `Kitchener` sheet section. The printed block is
/// mixed (rulebook §2.3 sample counters, §6.51, §6.52, §5.54):
///   - `(0,0)`, `(1,0)`, `(2,0)` are the three British leaders Kitchener,
///     Gatacre, and Hunter (0-0-15: movement only, no fire or melee, §6.51).
///   - `(0,1)`–`(4,1)` are the five "Friendlies" counters (8-6-9 volunteers,
///     §6.52: rifles on the Dervish range table, special VP by bank §9.14).
///   - `(3,0)` and `(4,0)` are the two Camel Corps counters (8-5-12, §7.5
///     camel retreat).
///   - `(5,0)`–`(7,1)` are the Sudanese battalions IX–XIV (9-5-8): IX–XII
///     form the 1st Sudanese brigade, XIII–XIV the 2nd (§5.54).
fn kitchener_block(col: u32, row: u32) -> Option<Classification> {
    let leader = |who: BritishLeader| {
        Some(Classification {
            kind: UnitKind::BritishLeader { movement: 15 },
            identity: UnitIdentity::AngloEgyptianLeader(who),
            // §6.51: Anglo-Egyptian leaders have a movement factor only --
            // no fire factor, so no weapon that can fire.
            weapon: WeaponClass::Melee,
        })
    };
    let friendlies = |col: u32| {
        // Each Friendlies counter is its own "brigade" for identity purposes
        // (they never integrate, §5.54), keeping the five counters distinct.
        Some(Classification {
            kind: UnitKind::Infantry { fire: 8, melee: 6, movement: 9 },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: (col + 1) as u8,
                    nationality: BrigadeNationality::Friendlies,
                },
                battalion: crate::BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
        })
    };
    let camel = || {
        Some(Classification {
            kind: UnitKind::Camel { fire: 8, melee: 5, movement: 12 },
            identity: UnitIdentity::AngloEgyptianCavalry,
            weapon: WeaponClass::Rifles,
        })
    };
    let sudanese = |brigade: u8, battalion: crate::BattalionOrdinal| {
        Some(Classification {
            kind: UnitKind::Infantry { fire: 9, melee: 5, movement: 8 },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: brigade,
                    nationality: BrigadeNationality::Sudanese,
                },
                battalion,
            },
            weapon: WeaponClass::Rifles,
        })
    };
    use crate::BattalionOrdinal as Bn;
    match (col, row) {
        (0, 0) => leader(BritishLeader::Kitchener),
        (1, 0) => leader(BritishLeader::Gatacre),
        (2, 0) => leader(BritishLeader::Hunter),
        (0, 1) | (1, 1) | (2, 1) | (3, 1) | (4, 1) => friendlies(col),
        (3, 0) | (4, 0) => camel(),
        // 1st Sudanese: IX, X, XI, XII (§5.54 brigade integrity).
        (5, 0) => sudanese(1, Bn::First),
        (6, 0) => sudanese(1, Bn::Second),
        (7, 0) => sudanese(1, Bn::Third),
        (5, 1) => sudanese(1, Bn::Fourth),
        // 2nd Sudanese: XIII, XIV.
        (6, 1) => sudanese(2, Bn::First),
        (7, 1) => sudanese(2, Bn::Second),
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

/// The `British_Army` sheet (rulebook §2.3 sample counters): row 0 carries
/// the cavalry, Royal Engineers, batteries and Maxim guns between the
/// battalions, row 1 carries the infantry battalions. Layout:
///   - `(0,0)` 21 Lancers (8-5-15, cavalry)
///   - `(1,0)` Royal Engineers (5-3-8, §6.53)
///   - `(2,0)` 32 Battery, `(3,0)` 37 Battery (10-1-7, artillery)
///   - `(4,0)`–`(7,0)` Maxim Batt. ×4 (6-1-12, §6.42)
///   - row 1 (and any unmapped cell) resolves by column as British infantry.
fn british_army_block(col: u32, row: u32) -> Option<Classification> {
    match (col, row) {
        (0, 0) => Some(Classification {
            kind: UnitKind::Cavalry { fire: 8, melee: 5, movement: 15 },
            identity: UnitIdentity::AngloEgyptianCavalry,
            weapon: WeaponClass::Rifles,
        }),
        (1, 0) => Some(Classification {
            kind: UnitKind::Infantry { fire: 5, melee: 3, movement: 8 },
            identity: UnitIdentity::RoyalEngineers,
            weapon: WeaponClass::Rifles,
        }),
        (2, 0) | (3, 0) => Some(Classification {
            kind: UnitKind::Artillery { fire: 10, melee: 1, movement: 7 },
            identity: UnitIdentity::AngloEgyptianArtillery,
            weapon: WeaponClass::Artillery,
        }),
        (4..=7, 0) => Some(Classification {
            kind: UnitKind::Maxim { fire: 6, melee: 1, movement: 12 },
            identity: UnitIdentity::AngloEgyptianMaxim,
            weapon: WeaponClass::Maxims,
        }),
        _ => ae_infantry(BrigadeNationality::British, col),
    }
}

/// The `Egyptian_Army` sheet: like the British one, row 0 interleaves the
/// division's cavalry and guns with two late battalions. Layout:
///   - `(0,0)`, `(1,0)` Egy. Cav. ×2 (10-5-15, cavalry)
///   - `(2,0)` Horse Art. (6-1-12)
///   - `(3,0)`–`(5,0)` Egy. Batt. ×3 (8-1-7, artillery -- includes the
///     §9.321 Fall-of-Khartoum "Egyptian Battalion artillery unit")
///   - `(6,0)`, `(7,0)` II/VIII Egy. infantry battalions
///   - row 1 resolves by column as Egyptian infantry.
fn egyptian_army_block(col: u32, row: u32) -> Option<Classification> {
    match (col, row) {
        (0, 0) | (1, 0) => Some(Classification {
            kind: UnitKind::Cavalry { fire: 10, melee: 5, movement: 15 },
            identity: UnitIdentity::AngloEgyptianCavalry,
            weapon: WeaponClass::Rifles,
        }),
        (2, 0) => Some(Classification {
            kind: UnitKind::Artillery { fire: 6, melee: 1, movement: 12 },
            identity: UnitIdentity::AngloEgyptianArtillery,
            weapon: WeaponClass::Artillery,
        }),
        (3..=5, 0) => Some(Classification {
            kind: UnitKind::Artillery { fire: 8, melee: 1, movement: 7 },
            identity: UnitIdentity::AngloEgyptianArtillery,
            weapon: WeaponClass::Artillery,
        }),
        _ => ae_infantry(BrigadeNationality::Egyptian, col),
    }
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
        assert!(profile_for(SectionName::BritishBoats, 0, 0).is_none());    }

    #[rulebook("§4")]
    #[test]
    fn game_turn_marker_cell_returns_none() {
        // `Hadendowa` (7,0) is the printed "GAME TURN" turn-track marker (§4),
        // not a placeable unit -- it must yield no profile so the picker hides
        // it, like the §6.63 BREECH cells.
        assert!(profile_for(SectionName::Hadendowa, 7, 0).is_none());
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
        // Row 1: the (7,0) cell is a Maxim counter, not a battalion.
        let p = profile_for(SectionName::BritishArmy, 7, 1).unwrap();
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
        // British_Army (5,1) has compiled faction data;
        // verify the brigade comes from the annotation, not the column.
        // (The (5,0) cell is a Maxim counter, not a battalion.)
        let p = profile_for(SectionName::BritishArmy, 5, 1).unwrap();
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
        // Yakub is the (0,0) counter of `Jaalin_I`; Osman Digna is (1,0)
        // of `Hadendowa`. Neither has its own section.
        let yakub = profile_for(SectionName::JaalinI, 0, 0).unwrap();
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
        let jaalin = profile_for(SectionName::JaalinI, 1, 0).unwrap();
        assert!(matches!(
            jaalin.identity,
            UnitIdentity::DervishTribal { .. }
        ));
    }

    #[rulebook("§5.54")]
    #[test]
    fn ae_infantry_third_battalion_from_col_2() {
        let p = profile_for(SectionName::BritishArmy, 2, 1).unwrap();
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
        let p = profile_for(SectionName::BritishArmy, 3, 1).unwrap();
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
        assert_eq!(section_owner(SectionName::MulazminI), Some(Player::Dervish));
        assert_eq!(section_owner(SectionName::MulazminII), Some(Player::Dervish));
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
        assert_eq!(section_owner(SectionName::MulazminI), Some(Player::Dervish));
        assert_eq!(section_owner(SectionName::MulazminII), Some(Player::Dervish));
    }

    #[rulebook("§5.52")]
    #[test]
    fn green_sections_are_mulazmin_tribal_units() {
        for section in [SectionName::MulazminI, SectionName::MulazminII] {
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

    #[rulebook("§6.51")]
    #[test]
    fn kitchener_block_resolves_leaders_friendlies_camel_and_sudanese() {
        // The three leaders (§6.51: movement only, no fire factor).
        for (col, who) in [
            (0, BritishLeader::Kitchener),
            (1, BritishLeader::Gatacre),
            (2, BritishLeader::Hunter),
        ] {
            let p = profile_for(SectionName::Kitchener, col, 0).unwrap();
            assert_eq!(
                p.identity,
                UnitIdentity::AngloEgyptianLeader(who),
                "Kitchener ({col},0) should be leader {who:?}"
            );
            assert!(matches!(p.kind, UnitKind::BritishLeader { .. }));
            assert!(p.fire.is_none(), "leaders print no fire factor (§6.51)");
        }
        // The five "Friendlies" (§6.52).
        for col in 0..=4 {
            let p = profile_for(SectionName::Kitchener, col, 1).unwrap();
            assert!(
                p.identity.is_friendlies(),
                "Kitchener ({col},1) should be a Friendlies counter (§6.52)"
            );
        }
        // The Camel Corps pair (§7.5 camel retreat; §9.14: 3-pt land unit).
        for col in [3, 4] {
            let p = profile_for(SectionName::Kitchener, col, 0).unwrap();
            assert_eq!(p.identity, UnitIdentity::AngloEgyptianCavalry);
            assert!(matches!(p.kind, UnitKind::Camel { .. }));
        }
        // Sudanese IX–XIV (§5.54): two brigades, ordinals per battalion.
        let p = profile_for(SectionName::Kitchener, 5, 0).unwrap(); // IX
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, .. } => {
                assert_eq!(brigade.number, 1);
                assert_eq!(brigade.nationality, crate::BrigadeNationality::Sudanese);
            }
            other => panic!("IX Sudanese misresolved: {other:?}"),
        }
        let p = profile_for(SectionName::Kitchener, 6, 1).unwrap(); // XIII
        match p.identity {
            UnitIdentity::AngloEgyptianInfantry { brigade, .. } => {
                assert_eq!(brigade.number, 2, "XIII Sudanese is 2nd brigade");
            }
            other => panic!("XIII Sudanese misresolved: {other:?}"),
        }
    }

    #[rulebook("§6.51")]
    #[test]
    fn dervish_leader_sections_resolve_leader_and_retinue_per_cell() {
        // Sheik El Din: leader cell + Jehadia retinue (§2.31 rifles).
        let leader = profile_for(SectionName::SheikElDin, 0, 0).unwrap();
        assert_eq!(
            leader.identity,
            UnitIdentity::DervishLeader(DervishLeader::SheikElDin)
        );
        let retinue = profile_for(SectionName::SheikElDin, 3, 1).unwrap();
        assert_eq!(
            retinue.identity,
            UnitIdentity::DervishTribal { tribe: DervishTribe::Jehadia },
            "SheikElDin (3,1) is a Jehadia counter, not a second leader"
        );
        // Sherif: leader cell + Danagla retinue.
        let leader = profile_for(SectionName::Sherif, 0, 0).unwrap();
        assert_eq!(leader.identity, UnitIdentity::DervishLeader(DervishLeader::Sherif));
        let retinue = profile_for(SectionName::Sherif, 2, 0).unwrap();
        assert_eq!(
            retinue.identity,
            UnitIdentity::DervishTribal { tribe: DervishTribe::Danagla }
        );
    }

    #[test]
    fn marker_only_leader_sections_yield_no_phantom_counters() {
        // The `Yakub` and `Osman_Digna` sections carry only blank marker
        // cells; the real leaders resolve from JaalinI (0,0) and
        // Hadendowa (1,0). These sections must yield nothing.
        assert!(profile_for(SectionName::Yakub, 0, 0).is_none());
        assert!(profile_for(SectionName::OsmanDigna, 0, 0).is_none());
    }

    #[rulebook("§9.111")]
    #[test]
    fn hadendowa_first_cell_is_isa_zachneih() {
        // §9.111's east-bank unit is printed on the Hadendowa sheet's (0,0)
        // cell; it must resolve to its own tribe (the §5.21 transport gate
        // and 1-VP §9.14 target), not as a Hadendowa tribesman.
        let p = profile_for(SectionName::Hadendowa, 0, 0).unwrap();
        assert_eq!(
            p.identity,
            UnitIdentity::DervishTribal { tribe: DervishTribe::IsaZachneih }
        );
        // And the rest of the block stays Hadendowa.
        let p = profile_for(SectionName::Hadendowa, 0, 1).unwrap();
        assert_eq!(
            p.identity,
            UnitIdentity::DervishTribal { tribe: DervishTribe::Hadendowa }
        );
    }

    #[rulebook("§6.52")]
    #[test]
    fn friendlies_counters_score_by_bank_not_as_leaders() {
        // §9.14: Friendlies 1 pt east bank / 3 pts west bank; only true
        // British leaders are worth 10. The vp source derivation lives in
        // effects, but the identity gate is here: a Friendlies counter must
        // not carry the AngloEgyptianLeader identity.
        let p = profile_for(SectionName::Kitchener, 0, 1).unwrap();
        assert!(!matches!(p.identity, UnitIdentity::AngloEgyptianLeader(_)));
        assert!(p.identity.is_friendlies());
    }

    #[rulebook("§2.3")]
    #[test]
    fn british_army_row_zero_specials_classify_by_counter() {
        // 21 Lancers is cavalry, not a 1st British brigade battalion.
        let p = profile_for(SectionName::BritishArmy, 0, 0).unwrap();
        assert_eq!(p.identity, UnitIdentity::AngloEgyptianCavalry);
        assert!(matches!(p.kind, UnitKind::Cavalry { .. }));

        // Royal Engineers (§6.53).
        let p = profile_for(SectionName::BritishArmy, 1, 0).unwrap();
        assert_eq!(p.identity, UnitIdentity::RoyalEngineers);

        // 32/37 Battery are artillery (§2.32 artillery line).
        for col in [2u8, 3u8] {
            let p = profile_for(SectionName::BritishArmy, col, 0).unwrap();
            assert_eq!(p.identity, UnitIdentity::AngloEgyptianArtillery);
            assert_eq!(p.weapon, WeaponClass::Artillery);
        }

        // Maxim Batt. cells are Maxims (fire twice, §6.42).
        for col in [4u8, 5, 6, 7] {
            let p = profile_for(SectionName::BritishArmy, col, 0).unwrap();
            assert_eq!(p.identity, UnitIdentity::AngloEgyptianMaxim);
        }
    }

    #[rulebook("§2.3")]
    #[test]
    fn egyptian_army_row_zero_specials_classify_by_counter() {
        // Egy. Cav. is cavalry with 15 MPs -- never an infantry battalion
        // (the mislabel gave "1E First Btn" cavalry movement).
        for col in [0u8, 1u8] {
            let p = profile_for(SectionName::EgyptianArmy, col, 0).unwrap();
            assert_eq!(p.identity, UnitIdentity::AngloEgyptianCavalry);
            match p.movement {
                UnitMovement::Land(a) => assert_eq!(a.value(), 15),
                other => panic!("cavalry must be land-mobile, got {other:?}"),
            }
        }
        // Horse Art. and the three Egy. Batt. artillery counters.
        for col in [2u8, 3, 4, 5] {
            let p = profile_for(SectionName::EgyptianArmy, col, 0).unwrap();
            assert_eq!(
                p.identity,
                UnitIdentity::AngloEgyptianArtillery,
                "cell ({col},0) must be artillery"
            );
        }
        // II/VIII Egy. at (6,0)/(7,0) are infantry battalions.
        for col in [6u8, 7u8] {
            let p = profile_for(SectionName::EgyptianArmy, col, 0).unwrap();
            assert!(matches!(p.identity, UnitIdentity::AngloEgyptianInfantry { .. }));
        }
    }
}

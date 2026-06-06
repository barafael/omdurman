//! Map picker (section_name, col, row) triples to rules-engine [`UnitProfile`].
//!
//! The picker lists counters by section name (tribe/brigade/gunboat) and
//! grid position on the sprite sheet.  This module translates those into
//! the rule types the effect processor needs.

use omdurman_rules::{
    BattalionOrdinal, BrigadeId, BrigadeNationality, BritishLeader, DervishLeader, DervishTribe,
    FireFactor, MeleeFactor, MovementAllowance, UnitIdentity, UnitKind, UnitMovement, UnitProfile,
    WeaponClass,
};

/// Build a [`UnitProfile`] for a counter identified by its sprite-sheet
/// section name and grid position.
pub fn profile_from_picker(section_name: &str, col: u32, _row: u32) -> Option<UnitProfile> {
    match section_name {
        // ── Dervish leaders ──────────────────────────────────────────
        "Khalifa_Abdullah" => Some(leader_profile(
            UnitIdentity::DervishLeader(DervishLeader::KhalifaAbdullah),
        )),
        "Sherif" => Some(leader_profile(
            UnitIdentity::DervishLeader(DervishLeader::Sherif),
        )),
        "Ali_Wad_Helu" => Some(leader_profile(
            UnitIdentity::DervishLeader(DervishLeader::AliWadHelu),
        )),
        "Sheik_El_Din" => Some(leader_profile(
            UnitIdentity::DervishLeader(DervishLeader::SheikElDin),
        )),
        "Yakub" => Some(leader_profile(
            UnitIdentity::DervishLeader(DervishLeader::Yakub),
        )),
        "Osman_Digna" => Some(leader_profile(
            UnitIdentity::DervishLeader(DervishLeader::OsmanDigna),
        )),

        // ── Dervish tribes ────────────────────────────────────────────
        "Taiasha" => Some(tribal_profile(DervishTribe::Taiasha)),
        "Hadendowa" => Some(tribal_profile(DervishTribe::Hadendowa)),
        "Baggara" => Some(tribal_profile(DervishTribe::Baggara)),
        "Jehadia" => Some(tribal_profile(DervishTribe::Jehadia)),
        "Mulazmin" => Some(tribal_profile(DervishTribe::Mulazmin)),
        "Kehena" => Some(tribal_profile(DervishTribe::Kehena)),
        "Degheim" => Some(tribal_profile(DervishTribe::Degheim)),
        "Danagla" => Some(tribal_profile(DervishTribe::Danagla)),
        "upper_Jaalin" | "lower_Jaalin" => {
            Some(tribal_profile(DervishTribe::Jaalin))
        }
        "upper_green" | "lower_green" => {
            // Green sections may be IsaZachneih or other east-bank infantry.
            Some(tribal_profile(DervishTribe::IsaZachneih))
        }

        // ── Dervish artillery ─────────────────────────────────────────
        "Hadendowa_Guns" => Some(UnitProfile {
            kind: UnitKind::Artillery,
            identity: UnitIdentity::DervishArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(FireFactor(4)),
            melee: Some(MeleeFactor(1)),
            movement: UnitMovement::Land(MovementAllowance(4)),
        }),

        // ── Anglo-Egyptian infantry brigades ──────────────────────────
        "British_Army" => Some(ae_infantry_profile(
            BrigadeId {
                number: brigade_from_col(col),
                nationality: BrigadeNationality::British,
            },
            BattalionOrdinal((col % 4) as u8 + 1),
        )),
        "Egyptian_Army" => Some(ae_infantry_profile(
            BrigadeId {
                number: brigade_from_col(col),
                nationality: BrigadeNationality::Egyptian,
            },
            BattalionOrdinal((col % 4) as u8 + 1),
        )),

        // ── A-E cavalry, camel corps ──────────────────────────────────
        "British_Boats" => Some(UnitProfile {
            kind: UnitKind::Cavalry,
            identity: UnitIdentity::AngloEgyptianCavalry,
            weapon: WeaponClass::Rifles,
            fire: Some(FireFactor(3)),
            melee: Some(MeleeFactor(3)),
            movement: UnitMovement::Land(MovementAllowance(8)),
        }),

        // ── A-E artillery ─────────────────────────────────────────────
        "Kitchener" => Some(leader_profile(UnitIdentity::AngloEgyptianLeader(
            BritishLeader::Kitchener,
        ))),

        // ── Unique units ──────────────────────────────────────────────
        _ => {
            // For unknown sections (e.g. sections not yet in the mapping),
            // return a generic infantry profile so the game stays playable.
            Some(UnitProfile {
                kind: UnitKind::Infantry,
                identity: UnitIdentity::AngloEgyptianInfantry {
                    brigade: BrigadeId {
                        number: 1,
                        nationality: BrigadeNationality::British,
                    },
                    battalion: BattalionOrdinal(1),
                },
                weapon: WeaponClass::Rifles,
                fire: Some(FireFactor(4)),
                melee: Some(MeleeFactor(2)),
                movement: UnitMovement::Land(MovementAllowance(6)),
            })
        }
    }
}

/// Determine brigade number from sprite column position.
fn brigade_from_col(col: u32) -> u8 {
    match col {
        0..=3 => 1,
        4..=7 => 2,
        8..=11 => 3,
        _ => 1,
    }
}

fn tribal_profile(tribe: DervishTribe) -> UnitProfile {
    UnitProfile {
        kind: UnitKind::Infantry,
        identity: UnitIdentity::DervishTribal { tribe },
        weapon: WeaponClass::Rifles,
        fire: Some(FireFactor(2)),
        melee: Some(MeleeFactor(3)),
        movement: UnitMovement::Land(MovementAllowance(6)),
    }
}

fn leader_profile(identity: UnitIdentity) -> UnitProfile {
    UnitProfile {
        kind: UnitKind::DervishLeaderUnit,
        identity,
        weapon: WeaponClass::Melee,
        fire: None,
        melee: Some(MeleeFactor(2)),
        movement: UnitMovement::Land(MovementAllowance(6)),
    }
}

fn ae_infantry_profile(brigade: BrigadeId, _battalion: BattalionOrdinal) -> UnitProfile {
    UnitProfile {
        kind: UnitKind::Infantry,
        identity: UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal(1),
        },
        weapon: WeaponClass::Rifles,
        fire: Some(FireFactor(4)),
        melee: Some(MeleeFactor(2)),
        movement: UnitMovement::Land(MovementAllowance(6)),
    }
}

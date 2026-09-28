//! Sprite annotation data per `UnitId` position.
//! Methods on `UnitId` delegate here.

use omdurman_types::{Faction, UnitKind};

/// Static data for a single sprite.
#[derive(Copy, Clone, Debug)]
pub struct SpriteData {
    pub faction: Option<Faction>,
    pub kind: Option<UnitKind>,
    pub text: &'static str,
}

/// Lookup sprite data by `(SectionName, col, row)`.
/// Returns `None` for sections/positions without a defined sprite.
pub fn sprite_data_for(
    section: omdurman_types::SectionName,
    col: u8,
    row: u8,
) -> Option<SpriteData> {
    match (section, col, row) {
        (omdurman_types::SectionName::BritishBoats, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 0,
            }),
            text: "Gen. Gordon",
        }),
        (omdurman_types::SectionName::AliWadHelu, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 1,
                melee: 1,
                movement: 15,
            }),
            text: "Ali Wad Helu",
        }),
        (omdurman_types::SectionName::AliWadHelu, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Kehena,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Kehena,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Kehena,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Kehena,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Kehena,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::AliWadHelu, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Kehena,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Deghelim",
        }),
        (omdurman_types::SectionName::Baggara, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::Baggara, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 15,
            }),
            text: "Baggara",
        }),
        (omdurman_types::SectionName::BritishBoats, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 5,
                upstream: 12,
                downstream: 18,
            }),
            text: "Abu Klea",
        }),
        (omdurman_types::SectionName::BritishBoats, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 5,
                upstream: 12,
                downstream: 18,
            }),
            text: "Sultan",
        }),
        (omdurman_types::SectionName::BritishBoats, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 4,
                upstream: 10,
                downstream: 16,
            }),
            text: "Gunboat",
        }),
        (omdurman_types::SectionName::BritishBoats, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 5,
                upstream: 12,
                downstream: 18,
            }),
            text: "Sheik",
        }),
        (omdurman_types::SectionName::BritishBoats, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 4,
                upstream: 10,
                downstream: 16,
            }),
            text: "Gunboat",
        }),
        (omdurman_types::SectionName::BritishBoats, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 5,
                upstream: 12,
                downstream: 18,
            }),
            text: "Fateh",
        }),
        (omdurman_types::SectionName::BritishBoats, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 4,
                upstream: 10,
                downstream: 16,
            }),
            text: "Gunboat",
        }),
        (omdurman_types::SectionName::BritishBoats, 7, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 5,
                upstream: 12,
                downstream: 18,
            }),
            text: "Melik",
        }),
        (omdurman_types::SectionName::BritishBoats, 7, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 4,
                upstream: 10,
                downstream: 16,
            }),
            text: "Gunboat",
        }),
        (omdurman_types::SectionName::BritishArmy, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 5,
                movement: 15,
            }),
            text: "21 Lancers",
        }),
        (omdurman_types::SectionName::BritishArmy, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 8,
            }),
            text: "Cameron II.",
        }),
        (omdurman_types::SectionName::BritishArmy, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 5,
                melee: 3,
                movement: 8,
            }),
            text: "Royal Eng.",
        }),
        (omdurman_types::SectionName::BritishArmy, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 8,
            }),
            text: "Seaforth II.",
        }),
        (omdurman_types::SectionName::BritishArmy, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 1,
                movement: 7,
            }),
            text: "32 Battery",
        }),
        (omdurman_types::SectionName::BritishArmy, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 8,
            }),
            text: "Lincolnshire",
        }),
        (omdurman_types::SectionName::BritishArmy, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 1,
                movement: 7,
            }),
            text: "37 Battery",
        }),
        (omdurman_types::SectionName::BritishArmy, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 8,
            }),
            text: "Warwicksh.",
        }),
        (omdurman_types::SectionName::BritishArmy, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 6,
                melee: 1,
                movement: 12,
            }),
            text: "Maxim Batt.",
        }),
        (omdurman_types::SectionName::BritishArmy, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 8,
            }),
            text: "Rifle Brig.",
        }),
        (omdurman_types::SectionName::BritishArmy, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 6,
                melee: 1,
                movement: 12,
            }),
            text: "Maxim Batt.",
        }),
        (omdurman_types::SectionName::BritishArmy, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 8,
            }),
            text: "Gren. Grds.",
        }),
        (omdurman_types::SectionName::BritishArmy, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 6,
                melee: 1,
                movement: 12,
            }),
            text: "Maxim Batt.",
        }),
        (omdurman_types::SectionName::BritishArmy, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 8,
            }),
            text: "Lancas. Fus.",
        }),
        (omdurman_types::SectionName::BritishArmy, 7, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 6,
                melee: 1,
                movement: 12,
            }),
            text: "Maxim Batt.",
        }),
        (omdurman_types::SectionName::BritishArmy, 7, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 8,
            }),
            text: "Northn. Fus.",
        }),
        (omdurman_types::SectionName::Danagla, 0, 0) => Some(SpriteData {
            faction: None,
            kind: Some(::omdurman_types::UnitKind::Marker),
            text: "",
        }),
        (omdurman_types::SectionName::Degheim, 0, 0) => Some(SpriteData {
            faction: None,
            kind: Some(::omdurman_types::UnitKind::Marker),
            text: "",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 15,
            }),
            text: "Egy. Cav.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "III Egy.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 10,
                melee: 5,
                movement: 15,
            }),
            text: "Egy. Cav.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "IV Egy.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 6,
                melee: 1,
                movement: 12,
            }),
            text: "Horse Art.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "VII Egy.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 1,
                movement: 7,
            }),
            text: "Egy. Batt.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "XV Egy.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 1,
                movement: 7,
            }),
            text: "Egy. Batt.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "I Egy.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 1,
                movement: 7,
            }),
            text: "Egy. Batt.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "V Egy.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "II Egy.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "VI Egy.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 7, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "VIII Egy.",
        }),
        (omdurman_types::SectionName::EgyptianArmy, 7, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "XVI Egy.",
        }),
        (omdurman_types::SectionName::Hadendowa, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::IsaZachneih,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Isa Zachneih",
        }),
        (omdurman_types::SectionName::Hadendowa, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 1,
                melee: 1,
                movement: 15,
            }),
            text: "Osman Digna",
        }),
        (omdurman_types::SectionName::Hadendowa, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 7,
                movement: 9,
            }),
            text: "Hadendowa",
        }),
        (omdurman_types::SectionName::Hadendowa, 7, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Marker),
            text: "GAME TURN",
        }),
        (omdurman_types::SectionName::Hadendowa, 7, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 7, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        (omdurman_types::SectionName::HadendowaForts, 7, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Hadendowa,
            }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "Hadendowa Fort",
        }),
        // FALL OF KHARTOUM's printed British forts (§9.321), "4-1-0 -3" on
        // the map like the North Fort.
        (omdurman_types::SectionName::BritishForts, 0 | 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Fort { fire: 4, melee: 1 }),
            text: "British Fort",
        }),
        (omdurman_types::SectionName::Jehadia, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Jehadia, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Kehena, 0, 0) => Some(SpriteData {
            faction: None,
            kind: Some(::omdurman_types::UnitKind::Marker),
            text: "",
        }),
        (omdurman_types::SectionName::KhalifaAbdullah, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 1,
                melee: 1,
                movement: 15,
            }),
            text: "Khalifa Abdullah",
        }),
        (omdurman_types::SectionName::KhalifaAbdullah, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 6,
                melee: 1,
                movement: 7,
            }),
            text: "",
        }),
        (omdurman_types::SectionName::KhalifaAbdullah, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 4,
                upstream: 10,
                downstream: 16,
            }),
            text: "Gunboat",
        }),
        (omdurman_types::SectionName::KhalifaAbdullah, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 6,
                melee: 1,
                movement: 7,
            }),
            text: "",
        }),
        (omdurman_types::SectionName::KhalifaAbdullah, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Gunboat {
                fire: 4,
                upstream: 10,
                downstream: 16,
            }),
            text: "Gunboat",
        }),
        (omdurman_types::SectionName::KhalifaAbdullah, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 6,
                melee: 1,
                movement: 7,
            }),
            text: "",
        }),
        (omdurman_types::SectionName::Kitchener, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 15,
            }),
            text: "Lord Kitchener Sirdar",
        }),
        (omdurman_types::SectionName::Kitchener, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "\"Friendlies\"",
        }),
        (omdurman_types::SectionName::Kitchener, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 15,
            }),
            text: "Gen. Gatacre Brit. Div.",
        }),
        (omdurman_types::SectionName::Kitchener, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "\"Friendlies\"",
        }),
        (omdurman_types::SectionName::Kitchener, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 0,
                melee: 0,
                movement: 15,
            }),
            text: "Gen. Hunter Egy. Div.",
        }),
        (omdurman_types::SectionName::Kitchener, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "\"Friendlies\"",
        }),
        (omdurman_types::SectionName::Kitchener, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 5,
                movement: 12,
            }),
            text: "Camel Corps",
        }),
        (omdurman_types::SectionName::Kitchener, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "\"Friendlies\"",
        }),
        (omdurman_types::SectionName::Kitchener, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 5,
                movement: 12,
            }),
            text: "Camel Corps",
        }),
        (omdurman_types::SectionName::Kitchener, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "\"Friendlies\"",
        }),
        (omdurman_types::SectionName::Kitchener, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "IX. Sud.",
        }),
        (omdurman_types::SectionName::Kitchener, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "XII. Sud.",
        }),
        (omdurman_types::SectionName::Kitchener, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "X. Sud.",
        }),
        (omdurman_types::SectionName::Kitchener, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "XIII. Sud.",
        }),
        (omdurman_types::SectionName::Kitchener, 7, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "XI. Sud.",
        }),
        (omdurman_types::SectionName::Kitchener, 7, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::BritishEgyptian { brigade: None }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 9,
                melee: 5,
                movement: 8,
            }),
            text: "XIV. Sud.",
        }),
        (omdurman_types::SectionName::JaalinII, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinII, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        // FoK green print-run Mulazmin counters. Color GreenRed per the
        // annotations RON; text "Mulazmin" is a documented deviation from
        // the RON (which records no printed text) so the picker can label them.
        (omdurman_types::SectionName::MulazminII, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 7, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminII, 7, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::Mulazmin, 0, 0) => Some(SpriteData {
            faction: None,
            kind: Some(::omdurman_types::UnitKind::Marker),
            text: "",
        }),
        (omdurman_types::SectionName::OsmanDigna, 0, 0) => Some(SpriteData {
            faction: None,
            kind: Some(::omdurman_types::UnitKind::Marker),
            text: "",
        }),
        (omdurman_types::SectionName::SheikElDin, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 1,
                melee: 1,
                movement: 15,
            }),
            text: "Sheik El Din",
        }),
        (omdurman_types::SectionName::SheikElDin, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::SheikElDin, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jehadia,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 9,
            }),
            text: "Jehadia",
        }),
        (omdurman_types::SectionName::Sherif, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Danagla,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 1,
                melee: 1,
                movement: 15,
            }),
            text: "Sherif",
        }),
        (omdurman_types::SectionName::Sherif, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Danagla,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 4,
                melee: 6,
                movement: 12,
            }),
            text: "",
        }),
        (omdurman_types::SectionName::Sherif, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Danagla,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 4,
                melee: 6,
                movement: 12,
            }),
            text: "",
        }),
        (omdurman_types::SectionName::Sherif, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Danagla,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 4,
                melee: 6,
                movement: 12,
            }),
            text: "",
        }),
        (omdurman_types::SectionName::Sherif, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Danagla,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 4,
                melee: 6,
                movement: 12,
            }),
            text: "",
        }),
        (omdurman_types::SectionName::Taiasha, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::Taiasha, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Taiasha,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Taiasha",
        }),
        (omdurman_types::SectionName::JaalinI, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 1,
                melee: 1,
                movement: 15,
            }),
            text: "Yakub",
        }),
        (omdurman_types::SectionName::JaalinI, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Baggara,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::JaalinI, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Jaalin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 12,
            }),
            text: "Jaalin",
        }),
        (omdurman_types::SectionName::MulazminI, 0, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 0, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 1, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 1, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 2, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 2, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 3, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 3, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 4, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 4, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 5, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 5, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 6, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 6, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 7, 0) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::MulazminI, 7, 1) => Some(SpriteData {
            faction: Some(::omdurman_types::Faction::Dervish {
                tribe: ::omdurman_types::DervishTribe::Mulazmin,
            }),
            kind: Some(::omdurman_types::UnitKind::Infantry {
                fire: 3,
                melee: 6,
                movement: 9,
            }),
            text: "Mulazmin",
        }),
        (omdurman_types::SectionName::Yakub, 0, 0) => Some(SpriteData {
            faction: None,
            kind: Some(::omdurman_types::UnitKind::Marker),
            text: "",
        }),
        _ => None,
    }
}

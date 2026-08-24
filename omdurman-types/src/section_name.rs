use serde::{Deserialize, Serialize};

/// A sprite-sheet section identifier such as `Hadendowa_Forts`.
///
/// Sections are the top-level grouping of unit counters on the sprite sheet
/// (e.g. all Hadendowa tribal units, British Army brigades, etc.). Each
/// section occupies a rectangular grid of cells identified by `(col, row)`.
///
/// Serialises to/from the underscore-separated string format used throughout
/// the project (e.g. `HadendowaForts` <-> `"Hadendowa_Forts"`).
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
    Serialize,
    Deserialize,
)]
pub enum SectionName {
    #[strum(serialize = "Taiasha")]
    #[serde(rename = "Taiasha")]
    Taiasha,
    #[strum(serialize = "Mulazmin_I")]
    #[serde(rename = "Mulazmin_I")]
    MulazminI,
    #[strum(serialize = "Khalifa_Abdullah")]
    #[serde(rename = "Khalifa_Abdullah")]
    KhalifaAbdullah,
    #[strum(serialize = "Sherif")]
    #[serde(rename = "Sherif")]
    Sherif,
    #[strum(serialize = "Mulazmin_II")]
    #[serde(rename = "Mulazmin_II")]
    MulazminII,
    #[strum(serialize = "Jaalin_I")]
    #[serde(rename = "Jaalin_I")]
    JaalinI,
    #[strum(serialize = "Hadendowa")]
    #[serde(rename = "Hadendowa")]
    Hadendowa,
    #[strum(serialize = "Jaalin_II")]
    #[serde(rename = "Jaalin_II")]
    JaalinII,
    #[strum(serialize = "Hadendowa_Forts")]
    #[serde(rename = "Hadendowa_Forts")]
    HadendowaForts,
    #[strum(serialize = "Baggara")]
    #[serde(rename = "Baggara")]
    Baggara,
    #[strum(serialize = "British_Boats")]
    #[serde(rename = "British_Boats")]
    BritishBoats,
    #[strum(serialize = "Ali_Wad_Helu")]
    #[serde(rename = "Ali_Wad_Helu")]
    AliWadHelu,
    #[strum(serialize = "British_Army")]
    #[serde(rename = "British_Army")]
    BritishArmy,
    #[strum(serialize = "Sheik_El_Din")]
    #[serde(rename = "Sheik_El_Din")]
    SheikElDin,
    #[strum(serialize = "Kitchener")]
    #[serde(rename = "Kitchener")]
    Kitchener,
    #[strum(serialize = "Jehadia")]
    #[serde(rename = "Jehadia")]
    Jehadia,
    #[strum(serialize = "Egyptian_Army")]
    #[serde(rename = "Egyptian_Army")]
    EgyptianArmy,
    #[strum(serialize = "Mulazmin")]
    #[serde(rename = "Mulazmin")]
    Mulazmin,
    #[strum(serialize = "Kehena")]
    #[serde(rename = "Kehena")]
    Kehena,
    #[strum(serialize = "Degheim")]
    #[serde(rename = "Degheim")]
    Degheim,
    #[strum(serialize = "Danagla")]
    #[serde(rename = "Danagla")]
    Danagla,
    #[strum(serialize = "Yakub")]
    #[serde(rename = "Yakub")]
    Yakub,
    #[strum(serialize = "Osman_Digna")]
    #[serde(rename = "Osman_Digna")]
    OsmanDigna,
}

impl SectionName {
    /// Human-readable display name (e.g. `HadendowaForts` -> `"Hadendowa Forts"`).
    pub fn display_name(self) -> &'static str {
        match self {
            SectionName::Taiasha => "Taiasha",
            // The "upper/lower green" counter-sheet sections are both simply
            // Mulazmin units; the "upper/lower Jaalin" sections are both Jaalin.
            SectionName::MulazminI => "Mulazmin I",
            SectionName::KhalifaAbdullah => "Khalifa Abdullah",
            SectionName::Sherif => "Sherif",
            SectionName::MulazminII => "Mulazmin II",
            SectionName::JaalinI => "Jaalin I",
            SectionName::Hadendowa => "Hadendowa",
            SectionName::JaalinII => "Jaalin II",
            SectionName::HadendowaForts => "Hadendowa Forts",
            SectionName::Baggara => "Baggara",
            SectionName::BritishBoats => "British Boats",
            SectionName::AliWadHelu => "Ali Wad Helu",
            SectionName::BritishArmy => "British Army",
            SectionName::SheikElDin => "Sheik El Din",
            SectionName::Kitchener => "Kitchener",
            SectionName::Jehadia => "Jehadia",
            SectionName::EgyptianArmy => "Egyptian Army",
            SectionName::Mulazmin => "Mulazmin",
            SectionName::Kehena => "Kehena",
            SectionName::Degheim => "Degheim",
            SectionName::Danagla => "Danagla",
            SectionName::Yakub => "Yakub",
            SectionName::OsmanDigna => "Osman Digna",
        }
    }
}

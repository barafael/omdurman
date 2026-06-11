use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// A sprite-sheet section identifier such as `Hadendowa_Forts`.
///
/// Sections are the top-level grouping of unit counters on the sprite sheet
/// (e.g. all Hadendowa tribal units, British Army brigades, etc.). Each
/// section occupies a rectangular grid of cells identified by `(col, row)`.
///
/// Serialises to/from the underscore-separated string format used throughout
/// the project (e.g. `HadendowaForts` ↔ `"Hadendowa_Forts"`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SectionName {
    Taiasha,
    UpperGreen,
    KhalifaAbdullah,
    Sherif,
    LowerGreen,
    UpperJaalin,
    Hadendowa,
    LowerJaalin,
    HadendowaForts,
    Baggara,
    BritishBoats,
    AliWadHelu,
    BritishArmy,
    SheikElDin,
    Kitchener,
    Jehadia,
    EgyptianArmy,
    Mulazmin,
    Kehena,
    Degheim,
    Danagla,
    Yakub,
    OsmanDigna,
}

impl SectionName {
    /// Human-readable display name (e.g. `HadendowaForts` → `"Hadendowa Forts"`).
    pub fn display_name(self) -> &'static str {
        match self {
            SectionName::Taiasha => "Taiasha",
            SectionName::UpperGreen => "upper green",
            SectionName::KhalifaAbdullah => "Khalifa Abdullah",
            SectionName::Sherif => "Sherif",
            SectionName::LowerGreen => "lower green",
            SectionName::UpperJaalin => "upper Jaalin",
            SectionName::Hadendowa => "Hadendowa",
            SectionName::LowerJaalin => "lower Jaalin",
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

    /// All known section names in an arbitrary-but-stable order.
    pub const ALL: &'static [SectionName] = &[
        SectionName::Taiasha,
        SectionName::UpperGreen,
        SectionName::KhalifaAbdullah,
        SectionName::Sherif,
        SectionName::LowerGreen,
        SectionName::UpperJaalin,
        SectionName::Hadendowa,
        SectionName::LowerJaalin,
        SectionName::HadendowaForts,
        SectionName::Baggara,
        SectionName::BritishBoats,
        SectionName::AliWadHelu,
        SectionName::BritishArmy,
        SectionName::SheikElDin,
        SectionName::Kitchener,
        SectionName::Jehadia,
        SectionName::EgyptianArmy,
        SectionName::Mulazmin,
        SectionName::Kehena,
        SectionName::Degheim,
        SectionName::Danagla,
        SectionName::Yakub,
        SectionName::OsmanDigna,
    ];
}

/// Display produces the underscore-separated string used in annotations.ron
/// and throughout the codebase (e.g. `HadendowaForts` → `"Hadendowa_Forts"`).
impl fmt::Display for SectionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectionName::Taiasha => write!(f, "Taiasha"),
            SectionName::UpperGreen => write!(f, "upper_green"),
            SectionName::KhalifaAbdullah => write!(f, "Khalifa_Abdullah"),
            SectionName::Sherif => write!(f, "Sherif"),
            SectionName::LowerGreen => write!(f, "lower_green"),
            SectionName::UpperJaalin => write!(f, "upper_Jaalin"),
            SectionName::Hadendowa => write!(f, "Hadendowa"),
            SectionName::LowerJaalin => write!(f, "lower_Jaalin"),
            SectionName::HadendowaForts => write!(f, "Hadendowa_Forts"),
            SectionName::Baggara => write!(f, "Baggara"),
            SectionName::BritishBoats => write!(f, "British_Boats"),
            SectionName::AliWadHelu => write!(f, "Ali_Wad_Helu"),
            SectionName::BritishArmy => write!(f, "British_Army"),
            SectionName::SheikElDin => write!(f, "Sheik_El_Din"),
            SectionName::Kitchener => write!(f, "Kitchener"),
            SectionName::Jehadia => write!(f, "Jehadia"),
            SectionName::EgyptianArmy => write!(f, "Egyptian_Army"),
            SectionName::Mulazmin => write!(f, "Mulazmin"),
            SectionName::Kehena => write!(f, "Kehena"),
            SectionName::Degheim => write!(f, "Degheim"),
            SectionName::Danagla => write!(f, "Danagla"),
            SectionName::Yakub => write!(f, "Yakub"),
            SectionName::OsmanDigna => write!(f, "Osman_Digna"),
        }
    }
}

/// Parses the underscore-separated string form. Returns an error on unknown
/// names.
#[derive(Debug)]
pub struct ParseSectionNameError(pub String);

impl fmt::Display for ParseSectionNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown section name: `{}`", self.0)
    }
}

impl std::error::Error for ParseSectionNameError {}

impl Serialize for SectionName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SectionName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SectionNameVisitor;
        impl<'de> serde::de::Visitor<'de> for SectionNameVisitor {
            type Value = SectionName;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a section name string")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<SectionName, E> {
                v.parse().map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_any(SectionNameVisitor)
    }
}

impl FromStr for SectionName {
    type Err = ParseSectionNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Taiasha" => Ok(SectionName::Taiasha),
            "upper_green" => Ok(SectionName::UpperGreen),
            "Khalifa_Abdullah" => Ok(SectionName::KhalifaAbdullah),
            "Sherif" => Ok(SectionName::Sherif),
            "lower_green" => Ok(SectionName::LowerGreen),
            "upper_Jaalin" => Ok(SectionName::UpperJaalin),
            "Hadendowa" => Ok(SectionName::Hadendowa),
            "lower_Jaalin" => Ok(SectionName::LowerJaalin),
            "Hadendowa_Forts" => Ok(SectionName::HadendowaForts),
            "Baggara" => Ok(SectionName::Baggara),
            "British_Boats" => Ok(SectionName::BritishBoats),
            "Ali_Wad_Helu" => Ok(SectionName::AliWadHelu),
            "British_Army" => Ok(SectionName::BritishArmy),
            "Sheik_El_Din" => Ok(SectionName::SheikElDin),
            "Kitchener" => Ok(SectionName::Kitchener),
            "Jehadia" => Ok(SectionName::Jehadia),
            "Egyptian_Army" => Ok(SectionName::EgyptianArmy),
            "Mulazmin" => Ok(SectionName::Mulazmin),
            "Kehena" => Ok(SectionName::Kehena),
            "Degheim" => Ok(SectionName::Degheim),
            "Danagla" => Ok(SectionName::Danagla),
            "Yakub" => Ok(SectionName::Yakub),
            "Osman_Digna" => Ok(SectionName::OsmanDigna),
            other => Err(ParseSectionNameError(other.to_owned())),
        }
    }
}

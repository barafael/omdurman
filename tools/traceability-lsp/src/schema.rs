//! Serde schema for `docs/traceability.toml` and shared constants.

use serde::Deserialize;

/// Section keys that are not numbered rulebook sections (printed charts,
/// credits) and so are exempt from the "must exist in the OCR manual" check.
/// Those whose printed text is transcribed in the manual under a
/// `### Key) Title` heading (`§CombatResults`, `§Disrupted`, `§TurnTrack`)
/// are indexed like numbered sections and their clause is checked verbatim;
/// the others (`§CRT` is the table's cells) describe their chart instead.
pub const PSEUDO_SECTIONS: &[&str] = &[
    "§Credits",
    "§Reference",
    "§CRT",
    "§CombatResults",
    "§Disrupted",
    "§TurnTrack",
];

/// Whether `key` (without the `§`) names a pseudo-section, i.e. may anchor a
/// `### Key) Title` heading in the manual.
pub fn is_pseudo_key(key: &str) -> bool {
    PSEUDO_SECTIONS
        .iter()
        .any(|s| s.strip_prefix('§') == Some(key))
}

/// The `[[mapping]]` entries of the matrix.
#[derive(Deserialize, Clone, Debug)]
pub struct Traceability {
    #[serde(rename = "mapping")]
    pub mappings: Vec<Mapping>,
}

/// One rulebook <-> implementation mapping.
#[derive(Deserialize, Clone, Debug)]
pub struct Mapping {
    pub section: String,
    pub title: String,
    pub status: String,
    #[serde(rename = "impl", default)]
    pub impls: Vec<ImplSite>,
    /// Optional free-form caveat / simplification note. Does not affect any
    /// check; purely informational for the generated PDF report.
    #[serde(default)]
    pub _note: Option<String>,
    /// Test functions that exercise this mapping's implementation.
    #[serde(default)]
    pub tests: Vec<String>,
    /// Kani proof harnesses that *prove* this mapping over their bounded input
    /// domain (`cargo kani`, see `scripts/kani.sh`). Kept separate from
    /// `tests` so the report can distinguish "proven" from "tested": a proof
    /// covers its whole domain, a test covers the cases it enumerates.
    #[serde(default)]
    pub proofs: Vec<String>,
    /// The manual's own words for the rule an `implemented` section's code
    /// enforces, quoted verbatim (checked against the OCR manual text).
    #[serde(default)]
    pub clause: Option<String>,
    /// The one test or proof whose job is that clause; it must be listed in
    /// `tests` or `proofs`.
    #[serde(default)]
    pub witness: Option<String>,
    /// Where the implementation deliberately departs from the clause, and why.
    #[serde(default)]
    pub approximation: Option<String>,
}

/// A single `[[mapping.impl]]` site: a file and a symbol in it. There is no
/// line number; `resolve` locates the symbol's definition.
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ImplSite {
    pub file: String,
    pub symbol: String,
}

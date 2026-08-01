//! Serde schema for `docs/traceability.toml` and shared constants.

use serde::Deserialize;

/// Section numbers that are not real rulebook sections (chart references,
/// credits) and so are exempt from the "must exist in the OCR manual" check.
pub const PSEUDO_SECTIONS: &[&str] = &["§Credits", "§Reference", "§CRT"];

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
}

/// A single `[[mapping.impl]]` site. `line` is 1-based and may drift; the
/// `resolve` module re-locates symbols robustly rather than trusting it.
#[derive(Deserialize, Clone, Debug)]
pub struct ImplSite {
    pub file: String,
    pub line: u32,
    pub symbol: String,
}

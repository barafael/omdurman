//! The LSP-facing graph over the traceability matrix.
//!
//! Builds an in-memory index connecting requirements (`docs/traceability.toml`),
//! manual sections, `§N` citations in source, resolved impl sites, and
//! annotated tests — all with byte-exact positions for navigation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::checks::read_traceability;
use crate::manual::{self, ManualSection};
use crate::resolve::resolve_symbol;
use crate::scan::{Citation, collect_rs_files, section_refs};
use crate::tests::{TestEntry, scan_test_entries};
use crate::{manual_path, traceability_path};

/// One `[[mapping]]` row, ready for navigation.
#[derive(Debug, Clone)]
pub struct Requirement {
    pub section: String,
    pub title: String,
    pub status: String,
    pub note: Option<String>,
    pub impls: Vec<crate::ImplSite>,
    pub tests: Vec<String>,
}

/// An impl site resolved to a concrete file/line/column.
#[derive(Debug, Clone)]
pub struct ResolvedImpl {
    pub section: String,
    pub symbol: String,
    pub file: PathBuf,
    pub line: usize,
    pub byte_col: usize,
}

/// The full navigable graph for one workspace snapshot.
#[derive(Debug)]
pub struct TraceIndex {
    pub root: PathBuf,
    pub requirements: Vec<Requirement>,
    pub manual_sections: Vec<ManualSection>,
    pub citations: Vec<Citation>,
    pub test_entries: Vec<TestEntry>,
    pub resolved_impls: Vec<ResolvedImpl>,
    /// Absolute path -> current text (disk content, possibly overlaid by an
    /// open editor document).
    pub file_texts: HashMap<PathBuf, String>,
}

impl TraceIndex {
    /// The absolute path of the OCR manual.
    pub fn manual_path(&self) -> PathBuf {
        crate::manual_path()
    }

    /// Build the index from disk, applying `overlays` (open, possibly edited
    /// documents) on top. Overlays are keyed by absolute path.
    pub fn build(root: &Path, overlays: &HashMap<PathBuf, String>) -> TraceIndex {
        let requirements: Vec<Requirement> = read_traceability(&traceability_path())
            .map(|t| {
                t.mappings
                    .into_iter()
                    .map(|m| Requirement {
                        section: m.section,
                        title: m.title,
                        status: m.status,
                        note: m._note,
                        impls: m.impls,
                        tests: m.tests,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let manual_sections = manual::index_manual(&manual_path());
        let test_entries = scan_test_entries(root);

        // Collect every relevant file text (disk + overlay): source, plus the
        // traceability matrix and OCR manual (for TOML/manual navigation).
        let mut file_texts: HashMap<PathBuf, String> = HashMap::new();
        let mut citations: Vec<Citation> = Vec::new();
        let mut walk = Vec::new();
        collect_rs_files(root, &mut walk);
        walk.push(traceability_path());
        walk.push(manual_path());
        for path in &walk {
            if file_texts.contains_key(path) {
                continue;
            }
            if let Some(text) = overlays.get(path) {
                file_texts.insert(path.clone(), text.clone());
            } else if let Ok(text) = std::fs::read_to_string(path) {
                file_texts.insert(path.clone(), text);
            }
        }
        for (path, text) in &file_texts {
            citations.extend(collect_citations_from_text(path, text));
        }

        // Resolve impl sites drift-resiliently.
        let mut resolved_impls: Vec<ResolvedImpl> = Vec::new();
        for req in &requirements {
            for imp in &req.impls {
                let file = root.join(&imp.file);
                let resolved = resolve_symbol(&file, &imp.symbol);
                resolved_impls.push(ResolvedImpl {
                    section: req.section.clone(),
                    symbol: imp.symbol.clone(),
                    file,
                    line: resolved.line,
                    byte_col: resolved.byte_col,
                });
            }
        }

        TraceIndex {
            root: root.to_path_buf(),
            requirements,
            manual_sections,
            citations,
            test_entries,
            resolved_impls,
            file_texts,
        }
    }

    /// The requirement for a section number, if mapped.
    pub fn requirement(&self, section: &str) -> Option<&Requirement> {
        self.requirements.iter().find(|r| r.section == section)
    }

    /// The manual anchor for a section number (accepts `§N` or `N`).
    pub fn manual(&self, section: &str) -> Option<&ManualSection> {
        let num = section.trim_start_matches('§');
        self.manual_sections.iter().find(|s| s.num == num)
    }

    /// Impl sites for a section.
    pub fn impls_for<'a>(&'a self, section: &str) -> impl Iterator<Item = &'a ResolvedImpl> {
        self.resolved_impls
            .iter()
            .filter(move |r| r.section == section)
    }

    /// Citations of a section across source.
    pub fn citations_for<'a>(&'a self, section: &str) -> impl Iterator<Item = &'a Citation> {
        self.citations.iter().filter(move |c| c.section == section)
    }

    /// Annotated tests covering a section.
    pub fn tests_for<'a>(&'a self, section: &str) -> impl Iterator<Item = &'a TestEntry> {
        self.test_entries
            .iter()
            .filter(move |t| t.sections.contains(section))
    }

    /// Which impl-site symbol sits at `path`/`line`/`byte_col`? Returns
    /// `(section, symbol)`.
    pub fn impl_symbol_at(
        &self,
        path: &Path,
        line: usize,
        byte_col: usize,
    ) -> Option<(String, String)> {
        let text = self.file_texts.get(path)?;
        self.resolved_impls
            .iter()
            .filter(|r| r.file == path && r.line == line)
            .find(|r| match text.lines().nth(line.saturating_sub(1)) {
                Some(line_str) => {
                    let key = r.symbol.rsplit("::").next().unwrap_or(&r.symbol);
                    let end = (r.byte_col + key.len()).min(line_str.len());
                    byte_col >= r.byte_col && byte_col <= end
                }
                None => false,
            })
            .map(|r| (r.section.clone(), r.symbol.clone()))
    }
}

fn collect_citations_from_text(path: &Path, text: &str) -> Vec<Citation> {
    text.lines()
        .enumerate()
        .flat_map(|(line_idx, line)| {
            section_refs(line).map(move |(byte_col, _, section)| Citation {
                section,
                file: path.to_path_buf(),
                line: line_idx + 1,
                byte_col,
            })
        })
        .collect()
}

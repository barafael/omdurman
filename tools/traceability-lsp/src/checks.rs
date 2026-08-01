//! The traceability checks.
//!
//! This is the single source of truth shared by `cargo test` (via
//! `omdurman-rules/tests/traceability.rs`) and the LSP's live diagnostics.
//! Failure strings are kept stable so the two surfaces agree byte-for-byte.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use crate::manual;
use crate::resolve::LINE_WINDOW;
use crate::scan::collect_section_refs;
use crate::schema::{PSEUDO_SECTIONS, Traceability};
use crate::tests::collect_test_annotations;
use crate::{manual_path, traceability_path};

/// Result of the four-way matrix check.
#[derive(Debug, Default)]
pub struct MatrixReport {
    pub failures: Vec<String>,
    pub num_mappings: usize,
    pub num_impls: usize,
    pub num_source_files: usize,
}

/// Result of the test-coverage bijectivity check.
#[derive(Debug, Default)]
pub struct CoverageReport {
    pub failures: Vec<String>,
    pub num_tests: usize,
    pub num_sections: usize,
}

/// A warning-level gap: an `implemented` mapping with no annotated tests.
#[derive(Debug, Clone)]
pub struct GapIssue {
    pub section: String,
    pub title: String,
}

/// Parse `docs/traceability.toml`.
pub fn read_traceability(path: &Path) -> Result<Traceability, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    toml::from_str(&content).map_err(|e| format!("invalid TOML in {}: {}", path.display(), e))
}

/// Check 1: `implemented` entries have valid impl sites; other statuses have
/// none; every impl symbol resolves (with drift detection). Plus the matrix
/// cross-checks 2-4:
///   2. every `§N` citation in source has a `[[mapping]]`
///   3. every (non-pseudo) mapping section exists in the OCR manual
///   4. every cited symbol is compiler-anchored in `traceability_paths.rs`
pub fn check_matrix(root: &Path) -> MatrixReport {
    let mut report = MatrixReport::default();
    let table = match read_traceability(&traceability_path()) {
        Ok(t) => t,
        Err(e) => {
            report.failures.push(e);
            return report;
        }
    };
    report.num_mappings = table.mappings.len();

    let mut all_impls: Vec<(String, u32, String)> = Vec::new();

    for m in &table.mappings {
        match m.status.as_str() {
            "implemented" => {
                if m.impls.is_empty() {
                    report.failures.push(format!(
                        "{} \"{}\" is 'implemented' but has no [[impl]] entries",
                        m.section, m.title
                    ));
                }
                for imp in &m.impls {
                    all_impls.push((imp.file.clone(), imp.line, imp.symbol.clone()));
                }
            }
            "descriptive" | "implicit" | "out-of-scope" => {
                if !m.impls.is_empty() {
                    report.failures.push(format!(
                        "{} \"{}\" is '{}' but has [[impl]] entries (should have none)",
                        m.section, m.title, m.status
                    ));
                }
            }
            other => {
                report.failures.push(format!(
                    "{} \"{}\" has unknown status '{}'",
                    m.section, m.title, other
                ));
            }
        }
    }

    // ---- Check 1b: impl sites exist and the symbol is (nearly) there -------
    report.num_impls = all_impls.len();
    for (file, line, symbol) in &all_impls {
        let full_path = root.join(file);
        if !full_path.exists() {
            report.failures.push(format!("impl file does not exist: {file}"));
            continue;
        }
        let content = fs::read_to_string(&full_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();
        let search_key = symbol.rsplit("::").next().unwrap_or(symbol);
        let cited = (*line as usize).saturating_sub(1);
        let lo = cited.saturating_sub(LINE_WINDOW);
        let hi = (cited + LINE_WINDOW + 1).min(lines.len());
        let near = lines
            .get(lo..hi)
            .is_some_and(|w| w.iter().any(|l| l.contains(search_key)));
        if !near {
            let anywhere = content.contains(search_key);
            let here = lines
                .get(cited)
                .map(|l| l.trim().chars().take(80).collect::<String>())
                .unwrap_or_else(|| "out of range".to_string());
            let why = if anywhere {
                "line has drifted (symbol exists elsewhere in the file)"
            } else {
                "symbol not found in file"
            };
            report.failures.push(format!(
                "{file}:{line}: '{symbol}' (key '{search_key}') -- {why} (line {line}: {here:?})",
            ));
        }
    }

    // ---- Check 2: every § citation in source has a mapping entry ----------
    let all_section_refs = collect_section_refs(root);
    report.num_source_files = all_section_refs.len();

    let mapped_sections: BTreeSet<&str> =
        table.mappings.iter().map(|m| m.section.as_str()).collect();

    for (src_path, refs) in &all_section_refs {
        for r in refs {
            if mapped_sections.contains(r.as_str()) {
                continue;
            }
            let is_covered_by_specific = !r.contains('.')
                && mapped_sections
                    .iter()
                    .any(|m| m.starts_with(r.as_str()) && *m != r);
            if !is_covered_by_specific {
                report
                    .failures
                    .push(format!("{src_path} cites {r} which has no [[mapping]] entry"));
            }
        }
    }

    // ---- Check 3: every mapping section exists in the OCR manual ----------
    let manual_path = manual_path();
    if manual_path.exists() {
        let manual_text = fs::read_to_string(&manual_path).unwrap_or_default();

        for m in &table.mappings {
            if PSEUDO_SECTIONS.contains(&m.section.as_str()) {
                continue;
            }
            let num = m.section.trim_start_matches('§');
            let variants = [
                format!("### {})", num),
                format!("### {}.", num),
                format!("## {})", num),
                format!("## {} ", num),
                format!("**{}**)", num),
                format!("**{}", num),
                format!("\n{})", num),
            ];
            let found = variants.iter().any(|v| manual_text.contains(v.as_str()));
            if !found {
                report.failures.push(format!(
                    "{} \"{}\" not found in OCR manual (searched variants of '{}')",
                    m.section, m.title, num
                ));
            }
        }
    }

    // ---- Check 4: every cited symbol is compiler-anchored -----------------
    let paths_file = fs::read_to_string(root.join("omdurman-rules/tests/traceability_paths.rs"))
        .unwrap_or_default();
    for (_, _, symbol) in &all_impls {
        let key = symbol.rsplit("::").next().unwrap_or(symbol);
        if !paths_file.contains(key) {
            report.failures.push(format!(
                "symbol '{symbol}' (key '{key}') is not anchored in traceability_paths.rs \
                 -- add a compiler-checked reference there"
            ));
        }
    }

    report
}

/// Validate that the `tests = [...]` field in each `[[mapping]]` is bijective
/// with the `#[rulebook]` / `// §` annotations in source.
pub fn check_coverage(root: &Path) -> CoverageReport {
    let mut report = CoverageReport::default();
    let table = match read_traceability(&traceability_path()) {
        Ok(t) => t,
        Err(e) => {
            report.failures.push(e);
            return report;
        }
    };

    let actual = collect_test_annotations(root);
    report.num_tests = actual.len();

    let mut failures: Vec<String> = Vec::new();

    // Check 1: every entry in TOML tests arrays is a real annotated test
    for m in &table.mappings {
        for test_name in &m.tests {
            match actual.get(test_name.as_str()) {
                None => {
                    failures.push(format!(
                        "{}: tests array lists '{}' but no such #[test] fn found in source",
                        m.section, test_name
                    ));
                }
                Some(sections) => {
                    if !sections.contains(&m.section) {
                        failures.push(format!(
                            "{}: tests array lists '{}' but that test's annotations are {:?} (does not include {})",
                            m.section, test_name, sections, m.section
                        ));
                    }
                }
            }
        }
    }

    // Check 2: every annotated test is listed in the correct section's tests array
    let mut toml_tests_by_section: HashMap<String, BTreeSet<String>> = HashMap::new();
    for m in &table.mappings {
        for test_name in &m.tests {
            toml_tests_by_section
                .entry(m.section.clone())
                .or_default()
                .insert(test_name.clone());
        }
    }

    for (test_name, sections) in &actual {
        for section in sections {
            let listed = toml_tests_by_section
                .get(section)
                .map(|s| s.contains(test_name))
                .unwrap_or(false);
            if !listed {
                failures.push(format!(
                    "test '{}' has annotation {section} but is not listed in the tests array of [[mapping]] section = \"{section}\"",
                    test_name
                ));
            }
        }
    }

    report.num_sections = toml_tests_by_section.len();
    report.failures = failures;
    report
}

/// Warning-level gap: `implemented` mappings with no annotated tests. This is
/// deliberately NOT a hard check — it surfaces coverage nudges in the editor
/// without failing `cargo test`.
pub fn check_semantic_gap(root: &Path) -> Vec<GapIssue> {
    let table = match read_traceability(&traceability_path()) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let actual = collect_test_annotations(root);

    let mut issues = Vec::new();
    for m in &table.mappings {
        if m.status != "implemented" {
            continue;
        }
        let has_test = m.tests.iter().any(|t| {
            actual
                .get(t.as_str())
                .is_some_and(|s| s.contains(&m.section))
        });
        if !has_test {
            issues.push(GapIssue {
                section: m.section.clone(),
                title: m.title.clone(),
            });
        }
    }
    issues
}

/// Write `target/traceability_generated.toml` listing every annotated test
/// with its full path and sections.
pub fn write_generated_toml(root: &Path) -> std::io::Result<std::path::PathBuf> {
    let mut all = crate::tests::collect_test_annotations_full(root);

    let mut lines: Vec<String> = Vec::new();
    lines.push("# Generated by #[rulebook] attributes — do not edit.".into());
    lines.push("# Run: cargo test -p omdurman-rules --test traceability".into());
    lines.push(String::new());

    let mut sorted: Vec<_> = all.drain().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    for (full_path, sections) in &sorted {
        lines.push("[[test]]".into());
        lines.push(format!("full_path = \"{full_path}\""));
        let section_strs: Vec<String> = sections.iter().map(|s| format!("\"{s}\"")).collect();
        lines.push(format!("sections = [{}]", section_strs.join(", ")));
        lines.push(String::new());
    }

    let out_path = root.join("target/traceability_generated.toml");
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, lines.join("\n"))?;
    Ok(out_path)
}

/// Build a manual-section index for the OCR manual (kept here so both the
/// checks and the LSP share the same parsing).
pub fn manual_sections() -> Vec<manual::ManualSection> {
    manual::index_manual(&manual_path())
}

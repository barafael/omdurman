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
use crate::tests::{EntryKind, collect_annotations_of_kind, collect_test_annotations};
use crate::{manual_path, traceability_path};

/// Result of the six-way matrix check.
#[derive(Debug, Default)]
pub struct MatrixReport {
    pub failures: Vec<String>,
    pub num_mappings: usize,
    pub num_impls: usize,
    pub num_source_files: usize,
    pub num_manual_sections: usize,
}

/// Result of the test-coverage bijectivity check.
#[derive(Debug, Default)]
pub struct CoverageReport {
    pub failures: Vec<String>,
    pub num_tests: usize,
    /// Annotated Kani proof harnesses found in source.
    pub num_proofs: usize,
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
    let content = fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    toml::from_str(&content).map_err(|e| format!("invalid TOML in {}: {}", path.display(), e))
}

/// Check 1: `implemented` entries have valid impl sites; other statuses have
/// none; every impl symbol resolves (with drift detection). Plus the matrix
/// cross-checks 2-6:
///   2. every `§N` citation in source has a `[[mapping]]`
///   3. every (non-pseudo) mapping section exists in the OCR manual
///   4. every cited symbol is compiler-anchored in `traceability_paths.rs`
///   5. every manual section has a `[[mapping]]` (manual -> matrix direction)
///   6. every anchor in `traceability_paths.rs` is cited by the matrix
///      (paths -> matrix direction), so the two files cannot drift apart
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
            report
                .failures
                .push(format!("impl file does not exist: {file}"));
            continue;
        }
        let content = fs::read_to_string(&full_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();
        let search_key = symbol.rsplit("::").next().unwrap_or(symbol);
        let cited = (*line as usize).saturating_sub(1);
        let lo = cited.saturating_sub(LINE_WINDOW);
        let hi = (cited + LINE_WINDOW + 1).min(lines.len());
        // Match only the *code* part of each line: a symbol that survives only
        // in a `//` / `///` comment near the cited line must not satisfy the
        // anchor check. (Heuristic: `//` inside a string literal also truncates
        // -- acceptable, since cited symbols are Rust identifiers.)
        let near = lines.get(lo..hi).is_some_and(|w| {
            w.iter()
                .any(|l| l.split("//").next().unwrap_or(l).contains(search_key))
        });
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
                report.failures.push(format!(
                    "{src_path} cites {r} which has no [[mapping]] entry"
                ));
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

        // ---- Check 5: every manual section has a [[mapping]] entry --------
        // (the manual -> matrix direction; Check 3 is matrix -> manual).
        let manual_secs = crate::manual::index_manual(&manual_path);
        report.num_manual_sections = manual_secs.len();
        for s in &manual_secs {
            let key = format!("§{}", s.num);
            if !mapped_sections.contains(key.as_str()) {
                report.failures.push(format!(
                    "manual section §{} \"{}\" has no [[mapping]] entry -- add one \
                     (\"descriptive\" for container headings)",
                    s.num, s.title
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

    // ---- Check 6: every anchor is cited by the matrix ---------------------
    // (the paths -> matrix direction; Check 4 is matrix -> paths).
    let toml_symbol_keys: std::collections::BTreeSet<String> = all_impls
        .iter()
        .map(|(_, _, s)| s.rsplit("::").next().unwrap_or(s).to_string())
        .collect();
    let anchors = anchors_from_paths_file(&paths_file);
    // Owning-type `use` items that only exist so `let _ = Type::member;`
    // anchors compile are scaffolding, not citations of their own.
    let method_owner_prefixes: std::collections::BTreeSet<String> = anchors
        .iter()
        .filter(|a| !a.is_use_item)
        .filter_map(|a| a.owner.clone())
        .collect();
    for a in &anchors {
        let cited = toml_symbol_keys.contains(&a.member)
            || PATHS_ANCHOR_ALLOWLIST.contains(&a.member.as_str());
        let scaffold = a.is_use_item && method_owner_prefixes.contains(&a.member);
        if cited || scaffold {
            continue;
        }
        report.failures.push(format!(
            "traceability_paths.rs anchors '{}' but no [[mapping.impl]] cites that \
             symbol -- remove the anchor or add/correct the mapping",
            a.member
        ));
    }

    report
}

/// Identifiers that appear in `traceability_paths.rs` anchor forms but are
/// test-scaffolding rather than cited symbols (`None`/`Ok` markers, `std`
/// iterator plumbing in method-path references).
const PATHS_ANCHOR_ALLOWLIST: &[&str] = &["None", "Some", "Ok", "Err", "std", "iter", "empty"];

/// One anchor statement extracted from `traceability_paths.rs`.
struct AnchorRef {
    /// `use` items are satisfied either by a matrix citation or by being an
    /// owning-type prefix of a method/field anchor (`use ... BoardInfo;`
    /// exists so `let _ = BoardInfo::is_walled_city;` compiles).
    is_use_item: bool,
    /// The referenced member/identifier itself (`is_walled_city`, `BoardInfo`
    /// for a type import). This is what must be cited by the matrix.
    member: String,
    /// For method anchors, the owning type (`BoardInfo` of
    /// `BoardInfo::is_walled_city`); scaffolding for the member reference.
    owner: Option<String>,
}

/// Extract every anchor from `traceability_paths.rs`.
///
/// Recognises the documented reference forms:
///   * `use path::to::{A, B};` / `use path::To::Type;` (may span lines)
///   * `let _ = Type::method;` / `let _ = x.field;` / `let _ = (a, b);` /
///     `let _ = CONST;`
fn anchors_from_paths_file(source: &str) -> Vec<AnchorRef> {
    let mut out = Vec::new();
    let mut joined = String::new(); // accumulates multi-line `use ...;`
    let mut in_use = false;
    for raw in source.lines() {
        let line = raw.trim();
        let line = line.split("//").next().unwrap_or(line).trim();
        if line.is_empty() {
            continue;
        }
        if in_use {
            joined.push(' ');
            joined.push_str(line);
            if line.ends_with(';') {
                in_use = false;
                if let Some(a) = parse_use(&joined) {
                    out.push(a);
                }
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("use ") {
            if line.ends_with(';') && !rest.contains('{') {
                if let Some(a) = parse_use(line) {
                    out.push(a);
                }
            } else if rest.contains('{') {
                in_use = !line.ends_with(';');
                joined = line.to_string();
                if !in_use && let Some(a) = parse_use(&joined) {
                    out.push(a);
                }
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("let _ =") {
            let rest = rest.trim_end().trim_end_matches(';').trim();
            for fragment in rest.split(['(', ')', ',']) {
                let mut frag = fragment.trim();
                // Drop generic/call tails: `empty::<&T>`, `Vec<T>`.
                frag = frag.split('<').next().unwrap_or(frag).trim();
                // Field form `x.field` -> `field`.
                frag = frag.rsplit('.').next().unwrap_or(frag).trim();
                let tail = frag.rsplit("::").next().unwrap_or(frag).trim();
                if tail.is_empty()
                    || tail == "_"
                    || !tail
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                {
                    continue;
                }
                // Owning type: the segment before the member (`Type::member`),
                // if the fragment is a multi-segment path.
                let owner = frag
                    .rsplit_once("::")
                    .map(|(prefix, _)| prefix.rsplit("::").next().unwrap_or(prefix).to_string());
                out.push(AnchorRef {
                    is_use_item: false,
                    member: tail.to_string(),
                    owner,
                });
            }
        }
    }
    out
}

/// Parse a complete `use ...;` statement into an [`AnchorRef`] of items.
fn parse_use(stmt: &str) -> Option<AnchorRef> {
    let rest = stmt.trim().strip_prefix("use ")?.trim_end();
    let rest = rest.trim_end().trim_end_matches(';').trim();
    let mut candidates = Vec::new();
    let mut push = |item: &str| {
        let item = item.trim();
        if item.is_empty() || item == "self" {
            return;
        }
        let name = match item.rsplit_once(" as ") {
            Some((_, alias)) => alias.trim(),
            None => item,
        };
        if name == "_" || name.is_empty() {
            return;
        }
        let name = name.rsplit("::").next().unwrap_or(name).trim();
        if !candidates.iter().any(|c: &String| c == name) {
            candidates.push(name.to_string());
        }
    };
    if let Some(brace) = rest.find('{') {
        if rest.ends_with('}') {
            // Expand (possibly nested) brace lists: we only care about final
            // identifiers, so drop inner braces before splitting.
            let inner: String = rest[brace + 1..rest.len() - 1]
                .chars()
                .filter(|c| *c != '{' && *c != '}')
                .collect();
            for item in inner.split(',') {
                push(item);
            }
        } else {
            return None;
        }
    } else {
        push(rest);
    }
    if candidates.is_empty() {
        None
    } else {
        Some(AnchorRef {
            is_use_item: true,
            member: candidates.remove(0),
            owner: None,
        })
    }
}

/// Validate that the `tests = [...]` field in each `[[mapping]]` is bijective
/// with the `#[rulebook]` / `// §` annotations in source.
///
/// This is a *listing* check: it proves the TOML and the source annotations
/// agree, that every listed test exists as a real (non-`#[ignore]`d) `#[test]`
/// fn, and that annotations match sections. It cannot prove that a test's
/// body actually exercises the mapped rule -- that judgement lives in the
/// annotation itself.
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

    // Checks 3 and 4: the same bijection for Kani proof harnesses, in their own
    // `proofs = [...]` namespace. A `#[rulebook]`-annotated `#[kani::proof]`
    // would otherwise be invisible -- silently uncounted rather than flagged.
    let proofs = collect_annotations_of_kind(root, EntryKind::Proof);
    report.num_proofs = proofs.len();

    for m in &table.mappings {
        for name in &m.proofs {
            match proofs.get(name.as_str()) {
                None => failures.push(format!(
                    "{}: proofs array lists '{}' but no such #[kani::proof] fn found in source",
                    m.section, name
                )),
                Some(sections) => {
                    if !sections.contains(&m.section) {
                        failures.push(format!(
                            "{}: proofs array lists '{}' but that proof's annotations are {:?} (does not include {})",
                            m.section, name, sections, m.section
                        ));
                    }
                }
            }
        }
    }

    let mut toml_proofs_by_section: HashMap<String, BTreeSet<String>> = HashMap::new();
    for m in &table.mappings {
        for name in &m.proofs {
            toml_proofs_by_section
                .entry(m.section.clone())
                .or_default()
                .insert(name.clone());
        }
    }
    for (name, sections) in &proofs {
        for section in sections {
            let listed = toml_proofs_by_section
                .get(section)
                .map(|s| s.contains(name))
                .unwrap_or(false);
            if !listed {
                failures.push(format!(
                    "proof '{}' has annotation {section} but is not listed in the proofs array of [[mapping]] section = \"{section}\"",
                    name
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

/// Build a manual-section index for the OCR manual (kept here so both the
/// checks and the LSP share the same parsing).
pub fn manual_sections() -> Vec<manual::ManualSection> {
    manual::index_manual(&manual_path())
}

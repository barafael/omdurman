//! Test annotation collection.
//!
//! One style counts: a `#[rulebook("§6.22")]` attribute (or the qualified
//! `#[traceability_macro::rulebook(...)]`) on a `#[test]` fn or a Kani proof
//! harness. A `§` in a comment is a citation, never coverage.
//!
//! `scan_test_entries` source-scans the workspace and records file/line so
//! navigation and code lens can point at the test.
//! The coverage check (`collect_test_annotations`) is a thin aggregation
//! over the same scan, so it does not depend on a prior build having
//! populated `target/rulebook_entries.jsonl`.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::scan::collect_rs_files;

/// Collect all annotated tests for the coverage check, keyed by
/// `crate::module::fn_name` (the file path as module path).
///
/// Keys are fully qualified so same-named test fns in different files can
/// never merge in the coverage map: the TOML `tests = [...]` arrays must list
/// the qualified name. Source-scans every relevant crate for `#[rulebook]`
/// attributes. This used to
/// load `target/rulebook_entries.jsonl` (written by the `#[rulebook]`
/// proc-macro during `cfg(test)` builds of `omdurman-rules`), but that made
/// the traceability test fragile: running `cargo test -p omdurman-rules --test
/// traceability` alone left the jsonl empty because the `cfg(test)` modules
/// in `omdurman-rules/src/*` were never compiled, so every `tests` entry in
/// the TOML failed with "no such #[test] fn found in source". Source scanning
/// is deterministic and independent of which test binary was built last.
pub fn collect_test_annotations(root: &Path) -> HashMap<String, BTreeSet<String>> {
    collect_annotations_of_kind(root, EntryKind::Test)
}

/// Annotated entries of one kind only, keyed by `module_prefix::fn_name` (the
/// fully qualified form the TOML `tests`/`proofs` arrays use).
///
/// The coverage check keeps tests and Kani proofs in separate namespaces
/// (`tests = [...]` vs `proofs = [...]`), so it filters by kind.
pub fn collect_annotations_of_kind(
    root: &Path,
    want: EntryKind,
) -> HashMap<String, BTreeSet<String>> {
    let mut result: HashMap<String, BTreeSet<String>> = HashMap::new();
    for entry in scan_test_entries(root) {
        if entry.kind != want {
            continue;
        }
        result
            .entry(qualified_name(root, &entry))
            .or_default()
            .extend(entry.sections);
    }
    result
}

/// `module_prefix::fn_name` for an entry, e.g.
/// `omdurman-rules::src::effects::sink_chain_is_atomic`.
fn qualified_name(root: &Path, entry: &TestEntry) -> String {
    let relative = entry
        .file
        .strip_prefix(root)
        .unwrap_or(&entry.file)
        .display()
        .to_string()
        .replace('\\', "/");
    let module_prefix = relative.trim_end_matches(".rs").replace('/', "::");
    format!("{module_prefix}::{}", entry.name)
}

/// Like `collect_test_annotations`, but keys by `module_prefix::fn_name`.
pub fn collect_test_annotations_full(root: &Path) -> HashMap<String, BTreeSet<String>> {
    let mut result: HashMap<String, BTreeSet<String>> = HashMap::new();
    for entry in scan_test_entries(root) {
        let relative = entry
            .file
            .strip_prefix(root)
            .unwrap_or(&entry.file)
            .display()
            .to_string()
            .replace('\\', "/");
        let module_prefix = relative.trim_end_matches(".rs").replace('/', "::");
        let full_path = format!("{module_prefix}::{}", entry.name);
        result.entry(full_path).or_default().extend(entry.sections);
    }
    result
}

/// Whether an annotated entry is an ordinary test or a Kani proof harness.
///
/// Proofs are tracked separately from tests so the matrix can report
/// "proven" distinctly from "tested": a proof covers its whole input domain,
/// a test covers the cases it happens to enumerate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Test,
    Proof,
}

/// A located annotated test, used by the LSP for navigation and code lens.
#[derive(Debug, Clone)]
pub struct TestEntry {
    pub name: String,
    /// Test vs Kani proof harness.
    pub kind: EntryKind,
    pub sections: BTreeSet<String>,
    /// Absolute path to the file containing the test.
    pub file: PathBuf,
    /// 1-based line of the `#[rulebook]` attr / `#[test]` marker.
    pub line: usize,
}

/// Source-scan the workspace for `#[rulebook]`-annotated tests and proofs,
/// recording locations. Uses disk contents only; does not require the jsonl to be fresh.
pub fn scan_test_entries(root: &Path) -> Vec<TestEntry> {
    let mut out: Vec<TestEntry> = Vec::new();
    for dir in [
        root.join("omdurman-rules/src"),
        root.join("omdurman-rules/tests"),
        root.join("omdurman-app/src"),
        root.join("omdurman-app/tests"),
        root.join("omdurman-net/src"),
        root.join("omdurman-types/src"),
        root.join("omdurman-hexmap/src"),
    ] {
        if !dir.exists() {
            continue;
        }
        let mut walk = Vec::new();
        collect_rs_files(&dir, &mut walk);
        for path in &walk {
            scan_file_test_entries(path, &mut out);
        }
    }
    out
}

fn scan_file_test_entries(path: &Path, out: &mut Vec<TestEntry>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        // `#[rulebook("§...")]` (or the qualified `traceability_macro::`
        // form) on a `#[test]` fn or a Kani proof -- the only annotation.
        // A `§` in a comment is a citation (it must name a mapped section),
        // never coverage.
        let trimmed = line.trim();
        let Some(attr) = trimmed
            .strip_prefix("#[rulebook(")
            .or_else(|| trimmed.strip_prefix("#[traceability_macro::rulebook("))
        else {
            continue;
        };
        // The attribute tail is `)]` after the last argument; trim it so
        // `#[rulebook("§4")]` -> `"§4"` rather than `"§4")]`.
        let attr = attr.trim_end().trim_end_matches(")]");
        let sections: BTreeSet<String> = attr
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim().to_string())
            .filter(|s| s.starts_with('§'))
            .collect();
        if sections.is_empty() {
            continue;
        }
        // It only counts on an actual `#[test]` fn or proof harness -- never
        // a helper -- and not on an `#[ignore]`d one.
        if let Some((fn_line, name, kind)) = locate_test(&lines, i + 1) {
            out.push(TestEntry {
                name,
                kind,
                sections,
                file: path.to_path_buf(),
                line: fn_line,
            });
        }
    }
}

/// Starting the scan `after` the `#[rulebook]` line (0-based), find the
/// `fn name` line among the attributes (and comments) that follow. Returns
/// the 1-based fn line, or `None` when no `#[test]` / Kani proof attribute
/// (`#[kani::proof]` or `#[cfg_attr(kani, kani::proof)]`) precedes the fn,
/// or the fn is `#[ignore]`d: an ignored test is not coverage.
fn locate_test(lines: &[&str], after: usize) -> Option<(usize, String, EntryKind)> {
    let mut kind = None;
    let mut ignored = false;
    for (k, line) in lines.iter().enumerate().skip(after).take(12) {
        let trimmed = line.trim();
        if trimmed.starts_with("#[ignore") {
            ignored = true;
        } else if trimmed == "#[test]" {
            kind.get_or_insert(EntryKind::Test);
        } else if trimmed.starts_with("#[kani::proof")
            || trimmed.starts_with("#[cfg_attr(kani, kani::proof")
        {
            // A Kani harness counts as coverage the same way a test does: it
            // proves the cited section over its whole bounded input domain.
            kind = Some(EntryKind::Proof);
        } else if trimmed.starts_with("#[") || trimmed.starts_with("//") {
            continue;
        } else {
            // The fn line ends the attribute run; anything else is no test.
            let rest = trimmed.strip_prefix("fn ")?;
            if ignored {
                return None;
            }
            let name = rest
                .split(['(', '<'])
                .next()
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some((k + 1, name, kind?));
        }
    }
    None
}

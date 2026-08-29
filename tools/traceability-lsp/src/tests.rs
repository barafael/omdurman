//! Test annotation collection.
//!
//! Two styles exist in the codebase:
//!   * `#[rulebook("§6.22")]` attributes above `#[test]` fns (omdurman-rules)
//!   * `// §X.Y` comment blocks above `#[test]` fns (omdurman-app)
//!
//! `scan_test_entries` source-scans both styles across the workspace and
//! records file/line so navigation and code lens can point at the test.
//! The coverage check (`collect_test_annotations`) is a thin aggregation
//! over the same scan, so it does not depend on a prior build having
//! populated `target/rulebook_entries.jsonl`.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::scan::{collect_rs_files, extract_section_refs_from_str};

/// Collect all annotated tests for the coverage check, keyed by
/// `crate::module::fn_name` (the file path as module path).
///
/// Keys are fully qualified so same-named test fns in different files can
/// never merge in the coverage map: the TOML `tests = [...]` arrays must list
/// the qualified name. Source-scans every relevant crate for both annotation
/// styles (`#[rulebook("§...")]` attributes and `// §` comments). This used to
/// load `target/rulebook_entries.jsonl` (written by the `#[rulebook]`
/// proc-macro during `cfg(test)` builds of `omdurman-rules`), but that made
/// the traceability test fragile: running `cargo test -p omdurman-rules --test
/// traceability` alone left the jsonl empty because the `cfg(test)` modules
/// in `omdurman-rules/src/*` were never compiled, so every `tests` entry in
/// the TOML failed with "no such #[test] fn found in source". Source scanning
/// is deterministic and independent of which test binary was built last.
pub fn collect_test_annotations(root: &Path) -> HashMap<String, BTreeSet<String>> {
    collect_test_annotations_full(root)
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

/// A located annotated test, used by the LSP for navigation and code lens.
#[derive(Debug, Clone)]
pub struct TestEntry {
    pub name: String,
    pub sections: BTreeSet<String>,
    /// Absolute path to the file containing the test.
    pub file: PathBuf,
    /// 1-based line of the `#[rulebook]` attr / `#[test]` marker.
    pub line: usize,
}

/// Source-scan the workspace for annotated tests in either style, recording
/// locations. Uses disk contents only; does not require the jsonl to be fresh.
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
        // Style 1: #[rulebook("§...")] above #[test] fn.
        if let Some(attr) = line.trim().strip_prefix("#[rulebook(") {
            // The attribute tail is `)]` after the last argument; trim it so
            // `#[rulebook("§4")]` -> `"§4"` rather than `"§4")]`. Without this
            // the section ended up as `§4")]` and never matched the TOML's `§4`.
            let attr = attr.trim_end().trim_end_matches(")]");
            let sections: BTreeSet<String> = attr
                .split(',')
                .filter_map(|s| {
                    let s = s.trim().trim_matches('"').trim().to_string();
                    if s.starts_with('§') && !s.is_empty() {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            if sections.is_empty() {
                continue;
            }
            // A `#[rulebook]` attribute only counts when it annotates an
            // actual `#[test]` fn -- never a helper. `#[ignore]`d tests are
            // excluded: an ignored test is not coverage.
            if let Some((fn_line, name)) = locate_test(&lines, i + 1, true) {
                out.push(TestEntry {
                    name,
                    sections,
                    file: path.to_path_buf(),
                    line: fn_line,
                });
            }
            continue;
        }

        // Style 2: // § comments above #[test].
        if !line.trim().starts_with("#[test]") {
            continue;
        }
        let mut sections = BTreeSet::new();
        let mut ignored_above = false;
        let mut j = i;
        while j > 0 {
            j -= 1;
            let prev = lines[j].trim();
            if prev.is_empty() {
                break;
            }
            if let Some(rest) = prev.strip_prefix("//") {
                extract_section_refs_from_str(rest.trim(), &mut sections);
            } else if prev.starts_with("#[") {
                if prev.starts_with("#[ignore") {
                    ignored_above = true;
                }
                // Other attributes (e.g. #[should_panic]) don't break the run.
            } else {
                break;
            }
        }
        if sections.is_empty() {
            continue;
        }
        // `#[ignore]` above or between `#[test]` and the fn excludes the test
        // (locate_test already returns None for a `#[ignore]` below).
        if let Some((fn_line, name)) = locate_test(&lines, i + 1, false)
            && !ignored_above
        {
            out.push(TestEntry {
                name,
                sections,
                file: path.to_path_buf(),
                line: fn_line,
            });
        }
    }
}

/// Starting the scan `after` the `#[rulebook]`/`#[test]` marker line (0-based),
/// find the `fn name` line within a few lines. Returns the 1-based fn line.
///
/// `require_test_attr` (style 1): a `#[test]` line must appear between the
/// `#[rulebook]` attribute and the fn, so annotated helpers never count as
/// tests. Returns `None` for `#[ignore]`d fns: an ignored test is not coverage.
fn locate_test(lines: &[&str], after: usize, require_test_attr: bool) -> Option<(usize, String)> {
    let mut seen_test = !require_test_attr;
    let mut ignored = false;
    for (k, line) in lines.iter().enumerate().skip(after).take(4) {
        let trimmed = line.trim();
        if trimmed.starts_with("#[ignore") {
            ignored = true;
            continue;
        }
        if trimmed == "#[test]" {
            seen_test = true;
            continue;
        }
        if trimmed.starts_with("#[") {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            if !seen_test || ignored {
                return None;
            }
            let name = rest.split('(').next().unwrap_or(rest).trim().to_string();
            return Some((k + 1, name));
        }
        return None;
    }
    None
}

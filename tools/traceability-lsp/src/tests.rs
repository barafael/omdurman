//! Test annotation collection.
//!
//! Two styles exist in the codebase:
//!   * `#[rulebook("§6.22")]` attributes above `#[test]` fns (omdurman-rules,
//!     written to `target/rulebook_entries.jsonl` by the proc-macro on
//!     `cfg(test)` builds)
//!   * `// §X.Y` comment blocks above `#[test]` fns (omdurman-app)
//!
//! The checks use the same data source as the original rules test
//! (jsonl + app source scan). The LSP's `TestEntry` scan is richer: it
//! source-scans both styles across the workspace and records file/line so
//! navigation and code lens can point at the test.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::scan::{collect_rs_files, extract_section_refs_from_str};

/// Load `target/rulebook_entries.jsonl` written by the `#[rulebook]`
/// proc-macro. Returns `false` if the file is missing.
pub fn load_rulebook_jsonl(
    root: &Path,
    result: &mut HashMap<String, BTreeSet<String>>,
) -> bool {
    let path = root.join("target/rulebook_entries.jsonl");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(test_name) = entry["test_name"].as_str().map(str::to_string) else {
            continue;
        };
        let sections: BTreeSet<String> = entry["sections"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        result.entry(test_name).or_default().extend(sections);
    }
    true
}

/// Scan a single file for `// §` comment annotations above `#[test]` fns.
pub fn collect_source_annotations(path: &Path, result: &mut HashMap<String, BTreeSet<String>>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if !line.trim().starts_with("#[test]") {
            continue;
        }
        let mut sections = BTreeSet::new();
        let mut j = i;
        while j > 0 {
            j -= 1;
            let prev = lines[j].trim();
            if prev.is_empty() {
                break;
            }
            if let Some(rest) = prev.strip_prefix("//") {
                extract_section_refs_from_str(rest.trim(), &mut sections);
            } else {
                break;
            }
        }
        if sections.is_empty() {
            continue;
        }
        let fn_line_idx = (i + 1..=std::cmp::min(i + 5, lines.len() - 1))
            .find(|&k| lines[k].trim().starts_with("fn "));
        if let Some(k) = fn_line_idx {
            if let Some(name) = lines[k].trim().strip_prefix("fn ") {
                let name = name.split('(').next().unwrap_or(name).trim();
                result.entry(name.to_string()).or_default().extend(sections);
            }
        }
    }
}

/// Collect all annotated tests for the coverage check:
/// `omdurman-rules` from the jsonl, `omdurman-app` by source scan.
pub fn collect_test_annotations(root: &Path) -> HashMap<String, BTreeSet<String>> {
    let mut result: HashMap<String, BTreeSet<String>> = HashMap::new();
    load_rulebook_jsonl(root, &mut result);

    let app_dir = root.join("omdurman-app/src");
    if app_dir.exists() {
        let mut walk = Vec::new();
        collect_rs_files(&app_dir, &mut walk);
        for path in &walk {
            collect_source_annotations(path, &mut result);
        }
    }

    result
}

/// Like `collect_test_annotations`, but keys by `module_prefix::fn_name`.
pub fn collect_test_annotations_full(root: &Path) -> HashMap<String, BTreeSet<String>> {
    let mut result: HashMap<String, BTreeSet<String>> = HashMap::new();
    load_rulebook_jsonl(root, &mut result);

    let app_dir = root.join("omdurman-app/src");
    if app_dir.exists() {
        let mut walk = Vec::new();
        collect_rs_files(&app_dir, &mut walk);
        for path in &walk {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(path)
                .display()
                .to_string()
                .replace('\\', "/");
            let module_prefix = relative.trim_end_matches(".rs").replace('/', "::");
            collect_source_annotations_full(path, &module_prefix, &mut result);
        }
    }

    result
}

/// Like `collect_source_annotations`, but keys by `module_prefix::fn_name`.
fn collect_source_annotations_full(
    path: &Path,
    module_prefix: &str,
    result: &mut HashMap<String, BTreeSet<String>>,
) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if !line.trim().starts_with("#[test]") {
            continue;
        }
        let mut sections = BTreeSet::new();
        let mut j = i;
        while j > 0 {
            j -= 1;
            let prev = lines[j].trim();
            if prev.is_empty() {
                break;
            }
            if let Some(rest) = prev.strip_prefix("//") {
                extract_section_refs_from_str(rest.trim(), &mut sections);
            } else {
                break;
            }
        }
        if sections.is_empty() {
            continue;
        }
        let fn_line_idx = (i + 1..=std::cmp::min(i + 5, lines.len() - 1))
            .find(|&k| lines[k].trim().starts_with("fn "));
        if let Some(k) = fn_line_idx {
            if let Some(name) = lines[k].trim().strip_prefix("fn ") {
                let name = name.split('(').next().unwrap_or(name).trim();
                let full_path = format!("{module_prefix}::{name}");
                result.entry(full_path).or_default().extend(sections);
            }
        }
    }
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
            scan_file_test_entries(&path, &mut out);
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
            if let Some((fn_line, name)) = find_test_fn(&lines, i + 1) {
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
        let mut j = i;
        while j > 0 {
            j -= 1;
            let prev = lines[j].trim();
            if prev.is_empty() {
                break;
            }
            if let Some(rest) = prev.strip_prefix("//") {
                extract_section_refs_from_str(rest.trim(), &mut sections);
            } else {
                break;
            }
        }
        if sections.is_empty() {
            continue;
        }
        if let Some((fn_line, name)) = find_test_fn(&lines, i + 1) {
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
fn find_test_fn(lines: &[&str], after: usize) -> Option<(usize, String)> {
    for k in after..std::cmp::min(after + 3, lines.len()) {
        let trimmed = lines[k].trim();
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            let name = rest.split('(').next().unwrap_or(rest).trim().to_string();
            return Some((k + 1, name));
        }
    }
    None
}

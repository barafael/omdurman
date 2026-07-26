//! Validate the bijective rulebook <-> implementation traceability matrix.
//!
//! Four checks:
//!   1. Every `implemented` entry has at least one `impl` child, and each
//!      `impl` file:line exists with the declared symbol (searched file-wide).
//!   2. Every `§N` citation in `.rs` source files has a corresponding
//!      `[[mapping]]` in the TOML.
//!   3. Every non-pseudo `[[mapping]]` section number exists in the OCR
//!      rulebook.
//!   4. Every cited symbol is compiler-anchored in `traceability_paths.rs`.
//!
//! A separate test (`test_coverage_mapping_is_bijective`) validates that the
//! `tests = [...]` field in each mapping is bijective with the `#[rulebook]`
//! annotations (collected via `inventory` for `omdurman-rules`, source-scanned
//! for `omdurman-app`).
//!
//! A third test (`generate_traceability_toml`) writes a generated TOML file to
//! `target/traceability_generated.toml` listing all annotated tests with their
//! full paths and sections.
//!
//! Run: `cargo test -p omdurman-rules --test traceability`

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// TOML schema (serde)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct Traceability {
    #[serde(rename = "mapping")]
    mappings: Vec<Mapping>,
}

#[derive(serde::Deserialize)]
struct Mapping {
    section: String,
    title: String,
    status: String,
    #[serde(rename = "impl", default)]
    impls: Vec<ImplSite>,
    /// Optional free-form caveat / simplification note (Tier 4). Does not
    /// affect any check; purely informational for the generated PDF report.
    #[serde(default)]
    _note: Option<String>,
    /// Test functions that exercise this mapping's implementation.
    #[serde(default)]
    tests: Vec<String>,
}

#[derive(serde::Deserialize)]
struct ImplSite {
    file: String,
    line: u32,
    symbol: String,
}

// ---------------------------------------------------------------------------
// Pseudo-sections are not real rulebook section numbers
// ---------------------------------------------------------------------------

const PSEUDO_SECTIONS: &[&str] = &["§Credits", "§Reference", "§CRT"];

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn workspace_root() -> PathBuf {
    let this = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    this.parent().unwrap().to_path_buf()
}

fn traceability_path() -> PathBuf {
    workspace_root().join("docs/traceability.toml")
}

// ---------------------------------------------------------------------------
// Test entry point
// ---------------------------------------------------------------------------

#[test]
fn traceability_matrix_is_bijective() {
    let toml_content =
        fs::read_to_string(traceability_path()).expect("docs/traceability.toml not found");
    let table: Traceability =
        toml::from_str(&toml_content).expect("invalid TOML in traceability.toml");

    let mut failures: Vec<String> = Vec::new();

    // ---- Check 1: implemented entries have valid impl sites ---------------
    let mut all_impls: Vec<(&str, u32, &str)> = Vec::new();

    for m in &table.mappings {
        match m.status.as_str() {
            "implemented" => {
                if m.impls.is_empty() {
                    failures.push(format!(
                        "{} \"{}\" is 'implemented' but has no [[impl]] entries",
                        m.section, m.title
                    ));
                }
                for imp in &m.impls {
                    all_impls.push((&imp.file, imp.line, &imp.symbol));
                }
            }
            "descriptive" | "implicit" | "out-of-scope" => {
                if !m.impls.is_empty() {
                    failures.push(format!(
                        "{} \"{}\" is '{}' but has [[impl]] entries (should have none)",
                        m.section, m.title, m.status
                    ));
                }
            }
            other => {
                failures.push(format!(
                    "{} \"{}\" has unknown status '{}'",
                    m.section, m.title, other
                ));
            }
        }
    }

    const LINE_WINDOW: usize = 8;
    let root = workspace_root();
    for &(file, line, symbol) in &all_impls {
        let full_path = root.join(file);
        if !full_path.exists() {
            failures.push(format!("impl file does not exist: {file}"));
            continue;
        }
        let content = fs::read_to_string(&full_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();
        let search_key = symbol.rsplit("::").next().unwrap_or(symbol);
        let cited = (line as usize).saturating_sub(1);
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
            failures.push(format!(
                "{file}:{line}: '{symbol}' (key '{search_key}') -- {why} (line {line}: {here:?})",
            ));
        }
    }

    // ---- Check 2: every § citation in source has a mapping entry ----------
    let all_section_refs = collect_section_refs(&root);

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
                failures.push(format!(
                    "{src_path} cites {r} which has no [[mapping]] entry"
                ));
            }
        }
    }

    // ---- Check 3: every mapping section exists in the OCR manual ----------
    let manual_root = root.join("Boardgame - Remember_Gordon/Boardgame - Remember_Gordon/Manual");
    let manual_path = manual_root.join("RememberGordonManual.md");

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
                failures.push(format!(
                    "{} \"{}\" not found in OCR manual (searched variants of '{}')",
                    m.section, m.title, num
                ));
            }
        }
    } else {
        eprintln!("  [!] OCR manual not found at {manual_path:?} -- skipping §3 check");
    }

    // ---- Check 4: every cited symbol is compiler-anchored -----------------
    let paths_file = fs::read_to_string(root.join("omdurman-rules/tests/traceability_paths.rs"))
        .unwrap_or_default();
    for &(_, _, symbol) in &all_impls {
        let key = symbol.rsplit("::").next().unwrap_or(symbol);
        if !paths_file.contains(key) {
            failures.push(format!(
                "symbol '{symbol}' (key '{key}') is not anchored in traceability_paths.rs \
                 -- add a compiler-checked reference there"
            ));
        }
    }

    // ---- Report -----------------------------------------------------------
    if failures.is_empty() {
        eprintln!(
            "traceability OK: {} mappings, {} impl sites, {} source files with § refs checked",
            table.mappings.len(),
            all_impls.len(),
            all_section_refs.len(),
        );
    } else {
        eprintln!("\n=== TRACEABILITY MATRIX ISSUES ===\n");
        for f in &failures {
            eprintln!("  [ ] {f}");
        }
        eprintln!(
            "\n{} failure(s) -- fix the TOML or the code to restore bijectivity.\n",
            failures.len()
        );
        panic!("traceability matrix check failed");
    }
}

// ---------------------------------------------------------------------------
// Test-coverage bijectivity check
// ---------------------------------------------------------------------------

/// Validate that the `tests = [...]` field in each [[mapping]] is bijective
/// with the `#[rulebook]` annotations in source code.
///
/// `omdurman-rules` tests are collected via `inventory` (from the
/// `#[rulebook]` proc-macro). `omdurman-app` tests are collected via source
/// scanning (they still use `// §` comments).
///
/// 1. Every test name listed in a mapping's `tests` must exist as an annotated
///    test, and that test's sections must include the mapping's section.
/// 2. Every annotated test must be listed in the `tests` array of each of its
///    sections' `[[mapping]]` entries.
#[test]
fn test_coverage_mapping_is_bijective() {
    let toml_content =
        fs::read_to_string(traceability_path()).expect("docs/traceability.toml not found");
    let table: Traceability =
        toml::from_str(&toml_content).expect("invalid TOML in traceability.toml");

    let root = workspace_root();
    let actual = collect_test_annotations(&root);

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

    if failures.is_empty() {
        eprintln!(
            "test coverage bijectivity OK: {} test functions annotated across {} sections",
            actual.len(),
            toml_tests_by_section.len(),
        );
    } else {
        eprintln!("\n=== TEST COVERAGE BIJECTIVITY ISSUES ===\n");
        for f in &failures {
            eprintln!("  [ ] {f}");
        }
        eprintln!(
            "\n{} failure(s) -- fix the TOML tests arrays or the #[rulebook] annotations.\n",
            failures.len()
        );
        panic!("test coverage bijectivity check failed");
    }
}

// ---------------------------------------------------------------------------
// Generated TOML output
// ---------------------------------------------------------------------------

/// Write `target/traceability_generated.toml` listing every annotated test
/// with its full path and sections.  Run this test to refresh the file:
///
/// ```sh
/// cargo test -p omdurman-rules --test traceability -- generate_traceability_toml
/// ```
#[test]
fn generate_traceability_toml() {
    let root = workspace_root();
    let all = collect_test_annotations_full(&root);

    let mut lines: Vec<String> = Vec::new();
    lines.push("# Generated by #[rulebook] attributes — do not edit.".into());
    lines.push("# Run: cargo test -p omdurman-rules --test traceability".into());
    lines.push(String::new());

    let mut sorted: Vec<_> = all.into_iter().collect();
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
        fs::create_dir_all(parent).ok();
    }
    fs::write(&out_path, lines.join("\n")).expect("failed to write generated TOML");
    eprintln!("wrote {}", out_path.display());
}

// ---------------------------------------------------------------------------
// Annotation collection
// ---------------------------------------------------------------------------

/// Collect all annotated tests.
/// - `omdurman-rules`: from `target/rulebook_entries.jsonl` (written by `#[rulebook]` proc-macro)
/// - `omdurman-app/src/`: `// §...` comments before `#[test]` (source-scanned)
///
/// Returns `fn_name -> BTreeSet<section>`.
fn collect_test_annotations(root: &Path) -> HashMap<String, BTreeSet<String>> {
    let mut result: HashMap<String, BTreeSet<String>> = HashMap::new();

    // 1. JSONL file from #[rulebook] proc-macro (omdurman-rules tests)
    load_rulebook_jsonl(root, &mut result);

    // 2. Source scan — omdurman-app tests with // § comments
    let app_dir = root.join("omdurman-app/src");
    if app_dir.exists() {
        let mut walk = Vec::new();
        collect_rs_files(&app_dir, &mut walk, root);
        for path in &walk {
            collect_source_annotations(path, &mut result);
        }
    }

    result
}

/// Like `collect_test_annotations`, but returns `full_path -> BTreeSet<section>`
/// for the generated TOML file.
fn collect_test_annotations_full(root: &Path) -> HashMap<String, BTreeSet<String>> {
    let mut result: HashMap<String, BTreeSet<String>> = HashMap::new();

    // 1. JSONL file from #[rulebook] proc-macro (use fn name as key, same as bijectivity)
    load_rulebook_jsonl(root, &mut result);

    // 2. Source scan — omdurman-app (use file-based path as approximation)
    let app_dir = root.join("omdurman-app/src");
    if app_dir.exists() {
        let mut walk = Vec::new();
        collect_rs_files(&app_dir, &mut walk, root);
        for path in &walk {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(path)
                .display()
                .to_string()
                .replace('\\', "/");
            let module_prefix = relative
                .trim_end_matches(".rs")
                .replace('/', "::");
            collect_source_annotations_full(path, &module_prefix, &mut result);
        }
    }

    result
}

/// Load `target/rulebook_entries.jsonl` written by the `#[rulebook]` proc-macro.
/// Each line is a JSON object with `"test_name"` and `"sections"` fields.
fn load_rulebook_jsonl(root: &Path, result: &mut HashMap<String, BTreeSet<String>>) {
    let path = root.join("target/rulebook_entries.jsonl");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            panic!(
                "Could not read {} — run `cargo test -p omdurman-rules --lib` first \
                 to generate it (the #[rulebook] proc-macro writes this file during \
                 compilation with cfg(test)): {}",
                path.display(),
                e,
            );
        }
    };

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let entry: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("Invalid JSONL at line {}: {} — {}", line_num + 1, line, e));
        let test_name = entry["test_name"]
            .as_str()
            .unwrap_or_else(|| panic!("Missing test_name at line {}", line_num + 1))
            .to_string();
        let sections: BTreeSet<String> = entry["sections"]
            .as_array()
            .unwrap_or_else(|| panic!("Missing sections at line {}", line_num + 1))
            .iter()
            .map(|v| v.as_str().expect("section is not a string").to_string())
            .collect();
        result.entry(test_name).or_default().extend(sections);
    }
}

/// Scan a single file for `// §` comments before `#[test]` functions.
/// Appends to `result` keyed by fn name.
fn collect_source_annotations(path: &Path, result: &mut HashMap<String, BTreeSet<String>>) {
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
                result
                    .entry(name.to_string())
                    .or_default()
                    .extend(sections);
            }
        }
    }
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

/// Extract `§N.M` references from a comment body string.
fn extract_section_refs_from_str(body: &str, out: &mut BTreeSet<String>) {
    let mut search_start = 0;
    while let Some(pos) = body[search_start..].find('§') {
        let abs_pos = search_start + pos;
        let after = &body[abs_pos + '§'.len_utf8()..];
        let section_num: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '.')
            .collect();
        let clean = section_num.trim_end_matches('x').trim_end_matches('.');
        if !clean.is_empty() {
            out.insert(format!("§{}", clean));
        }
        search_start = abs_pos + '§'.len_utf8();
    }
}

// ---------------------------------------------------------------------------
// Source-code § ref scanner
// ---------------------------------------------------------------------------

fn collect_section_refs(root: &Path) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    let mut walk = Vec::new();
    collect_rs_files(root, &mut walk, root);

    let exclude = [
        "omdurman-rules/tests/traceability.rs",
        "omdurman-rules/tests/traceability_paths.rs",
    ];

    for path in &walk {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();
        let normalized = relative.replace('\\', "/");
        if exclude.iter().any(|e| normalized.ends_with(e)) {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut refs = BTreeSet::new();

        for line in content.lines() {
            let mut search_start = 0;
            while let Some(pos) = line[search_start..].find('§') {
                let abs_pos = search_start + pos;
                let after = &line[abs_pos + '§'.len_utf8()..];
                let section_num: String = after
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '.')
                    .collect();
                let clean = section_num.trim_end_matches('x').trim_end_matches('.');
                if !clean.is_empty()
                    && (clean.starts_with(|c: char| c.is_ascii_digit()) || clean == "x")
                {
                    refs.insert(format!("§{}", clean));
                }
                search_start = abs_pos + '§'.len_utf8();
            }
        }

        if !refs.is_empty() {
            result.insert(relative, refs.into_iter().collect());
        }
    }

    result
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>, _root: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name != "target" && name != ".git" {
                    collect_rs_files(&path, out, _root);
                }
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
}

//! Validate the bijective rulebook <-> implementation traceability matrix.
//!
//! Three checks:
//!   1. Every `implemented` entry has at least one `impl` child, and each
//!      `impl` file:line exists with the declared symbol (searched file-wide).
//!   2. Every `§N` citation in `.rs` source files has a corresponding
//!      `[[mapping]]` in the TOML.
//!   3. Every non-pseudo `[[mapping]]` section number exists in the OCR
//!      rulebook.
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

    // Verify each impl site: symbol must appear *somewhere* in the file.
    // The symbol is the "last segment" (e.g. "UnitKind::may_melee_attack"
    // searches for "may_melee_attack" since the full path won't appear).
    let root = workspace_root();
    for &(file, line, symbol) in &all_impls {
        let full_path = root.join(file);
        if !full_path.exists() {
            failures.push(format!("impl file does not exist: {file}"));
            continue;
        }
        let content = fs::read_to_string(&full_path).unwrap_or_default();
        // Use the last segment of the symbol for matching
        let search_key = symbol.rsplit("::").next().unwrap_or(symbol);
        if !content.contains(search_key) {
            let source_line = content.lines().nth((line as usize).saturating_sub(1));
            let context = match source_line {
                Some(l) => format!(
                    " (line {}: {:?})",
                    line,
                    l.trim().chars().take(80).collect::<String>()
                ),
                None => format!(" (line {}: out of range)", line),
            };
            failures.push(format!(
                "{file}: symbol '{symbol}' (searched for '{search_key}') not found{context}",
            ));
        }
    }

    // ---- Check 2: every § citation in source has a mapping entry ----------
    let all_section_refs = collect_section_refs(&root);

    // Build a lookup: "§X.Y" -> entry, and also collect prefixes like "§6" so
    // we can tell if a generic "§6" is covered by more specific entries.
    let mapped_sections: BTreeSet<&str> = table.mappings.iter().map(|m| m.section.as_str()).collect();

    for (src_path, refs) in &all_section_refs {
        for r in refs {
            if mapped_sections.contains(r.as_str()) {
                continue;
            }
            // Generic ref like "§5" is covered by "§5.11" etc.
            let is_covered_by_specific =
                !r.contains('.') && mapped_sections.iter().any(|m| m.starts_with(r.as_str()) && *m != r);
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
// Source-code § ref scanner
// ---------------------------------------------------------------------------

fn collect_section_refs(root: &Path) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    let mut walk = Vec::new();
    collect_rs_files(root, &mut walk, root);

    let exclude = ["omdurman-rules/tests/traceability.rs"];

    for path in &walk {
        let relative = path.strip_prefix(root).unwrap_or(path).display().to_string();
        if exclude.iter().any(|e| relative.ends_with(e)) {
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
                // Strip a trailing period or "x" wildcard (e.g. "§7.5." -> "§7.5", "§6.2x" -> "§6.2")
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

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>, root: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name != "target" && name != ".git" {
                    collect_rs_files(&path, out, root);
                }
            } else if path.extension().map_or(false, |e| e == "rs") {
                out.push(path);
            }
        }
    }
}

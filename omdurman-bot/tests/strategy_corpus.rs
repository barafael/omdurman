//! Validate the strategy doctrine corpus (`docs/strategy/*.md`) the
//! commanders' doctrine is distilled from: every section-number citation in
//! the corpus resolves to a `[[mapping]]` in `docs/traceability.toml` — the
//! advice must be grounded in the manual, and the mapping table is the single
//! source of truth for which sections exist.
//!
//! Citations are checked by section *number* only (prefix match on the TOML
//! `section` field), so section-level entries satisfy references to
//! sub-sections and vice versa. The '§' marker is built at runtime so this
//! test file never contains the literal character (the traceability source
//! scan keys on it).

const SECTION_MARKER: &str = "\u{a7}";

/// The corpus files, in reading order.
const CORPUS_FILES: &[&str] = &[
    "common_doctrine.md",
    "anglo_egyptian_doctrine.md",
    "dervish_doctrine.md",
    "fall_of_khartoum_doctrine.md",
];

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("workspace root")
}

fn read_corpus(name: &str) -> String {
    let path = workspace_root().join("docs").join("strategy").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {name}: {e}"))
}

/// All mapped section numbers from `docs/traceability.toml` (without the '§').
fn mapped_sections() -> Vec<String> {
    let toml = std::fs::read_to_string(workspace_root().join("docs/traceability.toml"))
        .expect("read traceability.toml");
    toml.lines()
        .filter_map(|l| l.trim().strip_prefix("section = \""))
        .filter_map(|l| l.strip_suffix('"'))
        .map(|s| s.trim_start_matches(SECTION_MARKER).to_string())
        .collect()
}

/// Extract `(line, section_number)` citations from a corpus file.
fn corpus_citations(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let mut rest = line;
        while let Some(pos) = rest.find(SECTION_MARKER) {
            let after = &rest[pos + SECTION_MARKER.len()..];
            let num: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '.')
                .collect();
            let clean = num.trim_end_matches('x').trim_end_matches('.');
            if !clean.is_empty() {
                out.push((line_no + 1, clean.to_string()));
            }
            rest = after;
        }
    }
    out
}

/// Does `mapped` cover `cite`? (e.g. mapped "6.24" covers cite "6.2", and
/// mapped "6" covers cite "6.13".)
fn covers(mapped: &str, cite: &str) -> bool {
    mapped == cite || mapped.starts_with(cite) || cite.starts_with(mapped)
}

#[test]
fn every_corpus_citation_exists_in_traceability() {
    let mapped = mapped_sections();
    let mut failures = Vec::new();
    for name in CORPUS_FILES {
        let text = read_corpus(name);
        for (line_no, cite) in corpus_citations(&text) {
            let has = mapped.iter().any(|m| covers(m, &cite));
            if !has {
                failures.push(format!(
                    "{name}:{line_no} cites un-mapped {SECTION_MARKER}{cite}"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "corpus citations not in traceability.toml:\n  {}",
        failures.join("\n  ")
    );
}

#[test]
fn corpus_files_exist_and_are_substantial() {
    for name in CORPUS_FILES {
        assert!(
            !read_corpus(name).trim().is_empty(),
            "corpus file {name} is empty"
        );
    }
    let total: usize = CORPUS_FILES.iter().map(|n| read_corpus(n).len()).sum();
    assert!(total > 10_000, "corpus is thin: {total} chars");
}

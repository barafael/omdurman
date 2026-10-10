//! Source scanning: `.rs` file walking, `§N` citation extraction (with
//! positions for navigation), and the aggregate `§` reference map used by the
//! checks.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Recursively collect `.rs` files under `dir`, skipping `target`/`.git`.
pub fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name != "target" && name != ".git" {
                    collect_rs_files(&path, out);
                }
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
}

/// Every `§N.M` reference in `text`, as `(start, end, section)`.
///
/// `start` is the byte offset of the `§`, `end` the byte offset just past the
/// raw token (the alphanumerics and `.` that follow it), and `section` the
/// token with trailing junk normalised away (`§10.11x`, `§6.3.` -> `§10.11`,
/// `§6.3`). A `§` with nothing left after normalising yields nothing.
pub fn section_refs(text: &str) -> impl Iterator<Item = (usize, usize, String)> + '_ {
    text.match_indices('§').filter_map(move |(start, sign)| {
        let after = &text[start + sign.len()..];
        let raw_len = after
            .find(|c: char| !(c.is_alphanumeric() || c == '.'))
            .unwrap_or(after.len());
        let clean = after[..raw_len].trim_end_matches('x').trim_end_matches('.');
        (!clean.is_empty()).then(|| (start, start + sign.len() + raw_len, format!("§{clean}")))
    })
}

/// Extract `§N.M` references from a string, normalising trailing junk
/// (`§10.11x`, `§6.3.` -> `§10.11`, `§6.3`).
pub fn extract_section_refs_from_str(body: &str, out: &mut BTreeSet<String>) {
    out.extend(section_refs(body).map(|(_, _, section)| section));
}

/// A `§N` citation at a precise position in a source file (1-based line,
/// 0-based byte column).
#[derive(Debug, Clone)]
pub struct Citation {
    pub section: String,
    pub file: PathBuf,
    pub line: usize,
    pub byte_col: usize,
}

/// Aggregate `§` references across every `.rs` file under `root` except the
/// traceability test files themselves.
///
/// Returns `relative_path -> Vec<section>` (sorted). This is the check-2 data.
pub fn collect_section_refs(root: &Path) -> std::collections::HashMap<String, Vec<String>> {
    let mut result: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut walk = Vec::new();
    collect_rs_files(root, &mut walk);

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
        // The traceability test files and this crate's own sources (which cite
        // `§N` in doc prose) must not be counted as rule citations.
        if exclude.iter().any(|e| normalized.ends_with(e))
            || normalized.starts_with("tools/traceability-lsp/")
        {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut refs = BTreeSet::new();
        for line in content.lines() {
            extract_section_refs_from_str(line, &mut refs);
        }
        refs.retain(|r| {
            let clean = r.trim_start_matches('§');
            clean.starts_with(|c: char| c.is_ascii_digit()) || clean == "x"
        });
        if !refs.is_empty() {
            result.insert(relative, refs.into_iter().collect());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::section_refs;

    #[test]
    fn section_refs_normalise_and_span_the_raw_token() {
        let refs: Vec<_> = section_refs("see §10.11x, § and §6.3.").collect();
        assert_eq!(
            refs,
            vec![(4, 12, "§10.11".to_string()), (21, 27, "§6.3".to_string())]
        );
    }
}

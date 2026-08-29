//! Drift-resilient resolution of `[[mapping.impl]]` symbols to concrete
//! locations in source files.
//!
//! The TOML `line` field is known to drift (a rename/edit shifts the symbol
//! while the anchor stays). We never trust it blindly: we search the cited
//! file for the symbol and prefer the occurrence nearest to the declared line,
//! falling back to the first file-wide occurrence. Callers can compare the
//! resolved line with the declared line to emit a drift diagnostic.

use std::fs;
use std::path::{Path, PathBuf};

/// Window (in lines, either side) around the declared line in which a symbol
/// match is considered "at the anchor".
pub const LINE_WINDOW: usize = 8;

/// Result of resolving a symbol within a file.
#[derive(Debug, Clone)]
pub struct Resolved {
    pub file: PathBuf,
    pub line: usize,
    pub byte_col: usize,
    /// `true` if the resolved line is within `LINE_WINDOW` of the declared
    /// TOML line (i.e. the anchor is not stale).
    pub within_window: bool,
    /// Whether the symbol was found at all.
    pub found: bool,
}

/// Search `file` for `symbol`, preferring occurrences near `declared_line`
/// (1-based). The symbol may be a full path like `effects::apply_river_mine`;
/// we match on its final `::`-segment.
pub fn resolve_symbol(file: &Path, declared_line: u32, symbol: &str) -> Resolved {
    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(_) => {
            return Resolved {
                file: file.to_path_buf(),
                line: declared_line as usize,
                byte_col: 0,
                within_window: false,
                found: false,
            };
        }
    };
    let lines: Vec<&str> = content.lines().collect();
    let key = symbol.rsplit("::").next().unwrap_or(symbol);

    // Collect all matching (line, col) pairs, in line order.
    let mut matches: Vec<(usize, usize)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some(col) = line.find(key) {
            matches.push((i + 1, col));
        }
    }

    let Some(&(first_line, first_col)) = matches.first() else {
        return Resolved {
            file: file.to_path_buf(),
            line: declared_line as usize,
            byte_col: 0,
            within_window: false,
            found: false,
        };
    };

    let cited = declared_line as usize;
    let best = matches
        .iter()
        .min_by_key(|(l, _)| l.abs_diff(cited))
        .copied()
        .unwrap_or((first_line, first_col));
    let (best_line, best_col) = best;

    let within_window = best_line.abs_diff(cited) <= LINE_WINDOW;
    // Prefer the in-window occurrence over a file-wide one for navigation.
    let (line, col) = if within_window {
        (best_line, best_col)
    } else {
        // A stray match far from the anchor: use it but flag as drifted.
        (first_line, first_col)
    };

    Resolved {
        file: file.to_path_buf(),
        line,
        byte_col: col,
        within_window,
        found: true,
    }
}

/// The range of a whole `symbol` occurrence starting at `line`/`byte_col`
/// (end exclusive). Assumes `file` text is already known to the caller via
/// `text`; the symbol may appear as `key` (last path segment).
pub fn symbol_range(text: &str, line: usize, byte_col: usize, symbol: &str) -> (usize, usize) {
    let key = symbol.rsplit("::").next().unwrap_or(symbol);
    let line_str = text.lines().nth(line.saturating_sub(1)).unwrap_or_default();
    let start = byte_col.min(line_str.len());
    let end = (start + key.len()).min(line_str.len());
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_symbol_and_detects_drift() {
        let dir = std::env::temp_dir().join("traceability-lsp-resolve-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("file.rs");
        std::fs::write(
            &path,
            "fn other() {}\nfn apply_river_mine() {}\nfn apply_demolition() {}\n",
        )
        .unwrap();

        let near = resolve_symbol(&path, 2, "apply_river_mine");
        assert!(near.found);
        assert!(near.within_window);
        assert_eq!(near.line, 2);

        let drifted = resolve_symbol(&path, 99, "apply_river_mine");
        assert!(drifted.found);
        assert!(!drifted.within_window);
        assert_eq!(drifted.line, 2);

        let missing = resolve_symbol(&path, 1, "does_not_exist");
        assert!(!missing.found);
        std::fs::remove_dir_all(&dir).ok();
    }
}

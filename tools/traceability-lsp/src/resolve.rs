//! Resolution of `[[mapping.impl]]` symbols to locations in source files.
//!
//! The matrix names a file and a symbol, never a line: the line is found
//! here, at the symbol's *definition* (`fn`, `struct`, `enum`, `trait`,
//! `type`, `const`, `static`, `mod`, `macro_rules!`, an enum variant or a
//! struct field), falling back to its first use in code. Comments never
//! count. Moving code therefore never touches the matrix; renames are caught
//! by the compiler anchors in `omdurman-rules/tests/traceability_paths.rs`.

use std::fs;
use std::path::{Path, PathBuf};

/// Result of resolving a symbol within a file.
#[derive(Debug, Clone)]
pub struct Resolved {
    pub file: PathBuf,
    /// 1-based line (1 when not found).
    pub line: usize,
    pub byte_col: usize,
    /// Whether the symbol occurs in the file's code at all.
    pub found: bool,
}

/// Resolve `symbol` (a full path like `effects::apply_river_mine` is matched
/// on its last `::` segment) in `file`.
pub fn resolve_symbol(file: &Path, symbol: &str) -> Resolved {
    let located = fs::read_to_string(file)
        .ok()
        .and_then(|text| locate_symbol(&text, symbol));
    match located {
        Some((line, byte_col)) => Resolved {
            file: file.to_path_buf(),
            line,
            byte_col,
            found: true,
        },
        None => Resolved {
            file: file.to_path_buf(),
            line: 1,
            byte_col: 0,
            found: false,
        },
    }
}

/// `(1-based line, byte column)` of `symbol`'s definition in `text`, else of
/// its first whole-word occurrence in code; `None` if it only appears in
/// comments or not at all.
pub fn locate_symbol(text: &str, symbol: &str) -> Option<(usize, usize)> {
    let key = symbol.rsplit("::").next().unwrap_or(symbol);
    let mut first_use = None;
    let mut first_member = None;
    for (i, line) in text.lines().enumerate() {
        let code = line.split("//").next().unwrap_or(line);
        for col in word_occurrences(code, key) {
            match definition_kind(code, col, key) {
                Some(Definition::Item) => return Some((i + 1, col)),
                Some(Definition::Member) => {
                    first_member.get_or_insert((i + 1, col));
                }
                None => {
                    first_use.get_or_insert((i + 1, col));
                }
            }
        }
    }
    first_member.or(first_use)
}

enum Definition {
    /// `fn key`, `struct key`, ...: certainly the definition.
    Item,
    /// `key(..),` / `key: T` / `key = ..` opening a line: a variant or field
    /// (or, rarely, a statement -- hence second choice).
    Member,
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Byte columns of whole-word occurrences of `key` in `code`.
fn word_occurrences<'a>(code: &'a str, key: &'a str) -> impl Iterator<Item = usize> + 'a {
    code.match_indices(key)
        .map(|(col, _)| col)
        .filter(move |&col| {
            let before = code[..col].chars().next_back();
            let after = code[col + key.len()..].chars().next();
            !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
        })
}

fn definition_kind(code: &str, col: usize, key: &str) -> Option<Definition> {
    const KEYWORDS: [&str; 10] = [
        "fn",
        "struct",
        "enum",
        "trait",
        "type",
        "const",
        "static",
        "mod",
        "union",
        "macro_rules!",
    ];
    let before = code[..col].trim_end();
    for keyword in KEYWORDS {
        if let Some(prefix) = before.strip_suffix(keyword)
            && !prefix.chars().next_back().is_some_and(is_ident)
        {
            return Some(Definition::Item);
        }
    }
    let lead = before.trim_start();
    let opens_line =
        lead.is_empty() || lead == "pub" || (lead.starts_with("pub(") && lead.ends_with(')'));
    let after = code[col + key.len()..].trim_start();
    let member_shape = after.is_empty()
        || after.starts_with(['(', '{', ','])
        || (after.starts_with(':') && !after.starts_with("::"))
        || (after.starts_with('=') && !after.starts_with("==") && !after.starts_with("=>"));
    (opens_line && member_shape).then_some(Definition::Member)
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
    fn finds_the_definition_not_the_first_use() {
        let text = "// apply_river_mine in a comment\nfn other() { apply_river_mine(); }\n\
                    pub(crate) fn apply_river_mine() {}\n";
        assert_eq!(
            locate_symbol(text, "effects::apply_river_mine"),
            Some((3, 14))
        );
    }

    #[test]
    fn finds_variants_and_fields() {
        let text = "enum E {\n    Wall,\n    Gate(u8),\n}\nstruct S {\n    pub loaded_on: u8,\n}\n";
        assert_eq!(locate_symbol(text, "Gate"), Some((3, 4)));
        assert_eq!(locate_symbol(text, "loaded_on"), Some((6, 8)));
    }

    #[test]
    fn comments_and_partial_words_do_not_count() {
        let text = "// Wall\nfn walled() {}\n";
        assert_eq!(locate_symbol(text, "Wall"), None);
        assert_eq!(locate_symbol(text, "wall"), None);
    }
}

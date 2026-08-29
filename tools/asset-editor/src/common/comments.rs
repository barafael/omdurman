//! RON comment scanner and serde preprocessor.
//!
//! `ron` 0.8 only accepts *quoted* keys for string-keyed maps (bare
//! identifiers such as `Row01to05:` fail with `ExpectedString`), and tuple
//! keys such as `(Ground, Rough):` are unsupported entirely. The files on
//! disk use both. Rather than rewriting the files, [`scan`] produces:
//!
//! 1. `header` — the comment block before the root value, verbatim.
//! 2. `comments` — every other comment, keyed by a structural address
//!    (e.g. `cells/(Ground, Rough)/[0]`) so serializers can re-emit each
//!    comment exactly where it was found.
//! 3. `quoted` — a whitespace-normalized rebuild of the document with all
//!    map keys converted to quoted strings, directly deserializable into
//!    `IndexMap<String, _>` models.
//!
//! Addresses use `/` separators; map entries contribute their (normalized)
//! key text, sequence elements contribute `[i]`. Structural keys inside
//! tuple-struct values (e.g. `Infantry(fire: 3)`) get their own segments.

use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Scan {
    pub header: String,
    pub comments: BTreeMap<String, String>,
    pub quoted: String,
}

/// Normalize whitespace inside a map-key address segment: `(Ground,  Rough)`
/// and `(Ground, Rough)` must produce the same address on both the scanner
/// and the serializer side.
pub fn norm_key(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_ws = false;
    for ch in raw.trim().chars() {
        if ch.is_whitespace() {
            if !in_ws {
                out.push(' ');
            }
            in_ws = true;
        } else {
            out.push(ch);
            in_ws = false;
        }
    }
    out
}

/// Join address segments: `join("cells", "(Ground, Rough)")`.
pub fn addr(parent: &str, segment: &str) -> String {
    if parent.is_empty() {
        norm_key(segment)
    } else {
        format!("{parent}/{}", norm_key(segment))
    }
}

/// Address of element `i` inside `parent`.
pub fn elem(parent: &str, i: usize) -> String {
    if parent.is_empty() {
        format!("[{i}]")
    } else {
        format!("{parent}/[{i}]")
    }
}

// ── Scanner ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Delim {
    Brace,
    Bracket,
    Paren,
}

struct Frame {
    delim: Delim,
    path: String,
    index: usize,
}

struct Scanner {
    stack: Vec<Frame>,
    pending: Vec<String>,
    out: Scan,
    /// Address of the most recent key, awaiting its value (used so a value
    /// container opened right after `key:` inherits the key's address).
    key_addr: Option<String>,
    root_closed: bool,
    /// Length of `out.quoted` when the current line started; lets the
    /// comment branch tell full-line from trailing comments.
    line_quoted_start: usize,
}

pub fn scan(text: &str) -> Scan {
    let mut sc = Scanner {
        stack: Vec::new(),
        pending: Vec::new(),
        out: Scan::default(),
        key_addr: None,
        root_closed: false,
        line_quoted_start: 0,
    };
    sc.run(text);
    sc.out
}

impl Scanner {
    fn parent(&self) -> Option<&Frame> {
        self.stack.last()
    }

    fn in_brace(&self) -> bool {
        self.parent().is_some_and(|f| f.delim == Delim::Brace)
    }

    fn flush(&mut self, address: &str) {
        if !self.pending.is_empty() {
            let text = self.pending.join("\n");
            self.out.comments.insert(address.to_string(), text);
            self.pending.clear();
        }
    }

    /// Address for the next value element of the current parent (bumps the
    /// element counter).
    fn next_elem_addr(&mut self) -> String {
        match self.stack.last_mut() {
            Some(frame) => {
                let a = elem(&frame.path, frame.index);
                frame.index += 1;
                a
            }
            None => String::new(),
        }
    }

    fn push_frame(&mut self, delim: Delim, path: String) {
        self.stack.push(Frame {
            delim,
            path,
            index: 0,
        });
    }

    fn run(&mut self, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        let mut line_start = 0usize;

        macro_rules! skip_ws_inline {
            ($j:expr) => {
                while $j < chars.len() && chars[$j].is_whitespace() {
                    $j += 1;
                }
            };
        }

        while i < chars.len() {
            let c = chars[i];
            match c {
                '\n' => {
                    self.out.quoted.push('\n');
                    i += 1;
                    line_start = i;
                    self.line_quoted_start = self.out.quoted.len();
                }
                ' ' | '\t' | '\r' => {
                    i += 1;
                }
                '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                    // Comment to end of line. If code was already emitted on
                    // this line it is a trailing comment (kept from the `//`);
                    // otherwise a full-line comment (kept with indentation).
                    let trailing = self.out.quoted.len() > self.line_quoted_start;
                    let start = if trailing { i } else { line_start };
                    let end = chars[i..]
                        .iter()
                        .position(|&c| c == '\n')
                        .map(|p| i + p)
                        .unwrap_or(chars.len());
                    let line: String = chars[start..end].iter().collect();
                    let line = line.trim_end().to_string();
                    if self.stack.is_empty() && !self.root_closed {
                        if !self.out.header.is_empty() {
                            self.out.header.push('\n');
                        }
                        self.out.header.push_str(&line);
                    } else {
                        self.pending.push(line);
                    }
                    i = end;
                }
                '"' => {
                    let (raw, next) = scan_string(&chars, i);
                    // Peek: key or value?
                    let mut j = next;
                    skip_ws_inline!(j);
                    let is_key = j < chars.len() && chars[j] == ':';
                    if is_key {
                        let a = addr(
                            &self.parent().map(|f| f.path.clone()).unwrap_or_default(),
                            &raw,
                        );
                        self.flush(&a);
                        self.key_addr = Some(a);
                        self.quote_key(&raw[1..raw.len() - 1]);
                        self.out.quoted.push_str(" : ");
                        i = j + 1;
                    } else {
                        self.out.quoted.push_str(&raw);
                        self.take_value();
                        i = next;
                    }
                }
                '(' => {
                    // Either a tuple map key `(A, B):` or a value container.
                    let (close, next) = match_balanced(&chars, i);
                    let mut j = next;
                    skip_ws_inline!(j);
                    if j < chars.len() && chars[j] == ':' {
                        // Tuple key. The quoted form keeps the parens inside
                        // the string so model keys match `cell_key()` output.
                        let inner: String = chars[i + 1..close].iter().collect();
                        let parent = self.parent().map(|f| f.path.clone()).unwrap_or_default();
                        let a = addr(&parent, &format!("({inner})"));
                        self.flush(&a);
                        self.key_addr = Some(a.clone());
                        self.out.quoted.push_str("\"(");
                        self.out.quoted.push_str(&norm_key(&inner));
                        self.out.quoted.push_str(")\"");
                        self.out.quoted.push_str(" : ");
                        i = j + 1;
                    } else {
                        // Value: tuple / tuple-struct / struct.
                        let a = self.take_value_addr();
                        self.push_frame(Delim::Paren, a);
                        self.out.quoted.push('(');
                        i += 1;
                    }
                }
                '[' | '{' => {
                    let delim = if c == '[' {
                        Delim::Bracket
                    } else {
                        Delim::Brace
                    };
                    let path = match self.key_addr.take() {
                        Some(a) => a,
                        None => self.next_elem_addr(),
                    };
                    self.flush(&path);
                    self.push_frame(delim, path);
                    self.out.quoted.push(c);
                    i += 1;
                }
                ')' | ']' | '}' => {
                    let frame = self.stack.pop();
                    if let Some(f) = frame {
                        self.flush(&format!("{}/__close__", f.path));
                        if self.stack.is_empty() {
                            self.root_closed = true;
                        }
                    }
                    self.key_addr = None;
                    self.out.quoted.push(c);
                    i += 1;
                }
                ',' => {
                    self.out.quoted.push_str(", ");
                    i += 1;
                }
                c if c.is_alphabetic() || c == '_' => {
                    let mut j = i;
                    while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                        j += 1;
                    }
                    let ident: String = chars[i..j].iter().collect();
                    let mut k = j;
                    skip_ws_inline!(k);
                    if k < chars.len() && chars[k] == ':' {
                        // Map/struct key.
                        let parent = self.parent().map(|f| f.path.clone()).unwrap_or_default();
                        let a = addr(&parent, &ident);
                        self.flush(&a);
                        self.key_addr = Some(a);
                        if self.in_brace() {
                            self.quote_key(&ident);
                        } else {
                            self.out.quoted.push_str(&ident);
                        }
                        self.out.quoted.push_str(" : ");
                        i = k + 1;
                    } else if k < chars.len() && chars[k] == '(' {
                        // Named tuple-struct / enum-with-payload: `Section(`,
                        // `Infantry(`, `Eliminate(`, `Some(`. The paren is
                        // part of this value: one element.
                        let a = self.take_value_addr();
                        self.push_frame(Delim::Paren, a);
                        self.out.quoted.push_str(&ident);
                        self.out.quoted.push('(');
                        i = k + 1;
                    } else {
                        self.out.quoted.push_str(&ident);
                        self.take_value();
                        i = j;
                    }
                }
                c if c.is_ascii_digit() || c == '-' || c == '+' || c == '.' => {
                    let mut j = i;
                    while j < chars.len()
                        && (chars[j].is_ascii_digit()
                            || matches!(chars[j], '-' | '+' | '.' | 'e' | 'E'))
                    {
                        j += 1;
                    }
                    let num: String = chars[i..j].iter().collect();
                    self.out.quoted.push_str(&num);
                    self.take_value();
                    i = j;
                }
                _ => {
                    self.out.quoted.push(c);
                    i += 1;
                }
            }
        }

        // Comments after the root close.
        if !self.pending.is_empty() {
            let text = self.pending.join("\n");
            self.out.comments.insert("__trailing__".to_string(), text);
        }
    }

    fn quote_key(&mut self, ident: &str) {
        self.out.quoted.push('"');
        self.out.quoted.push_str(ident);
        self.out.quoted.push('"');
    }

    /// Consume a pending key address for a scalar value (no container). If
    /// there was no pending key the value is a sequence element: bump the
    /// counter and attach any pending comments to its address.
    fn take_value(&mut self) {
        if self.key_addr.take().is_none() {
            let a = self.next_elem_addr();
            self.flush(&a);
        }
    }

    /// Address for a value that opens a paren container.
    fn take_value_addr(&mut self) -> String {
        match self.key_addr.take() {
            Some(a) => a,
            None => {
                let a = self.next_elem_addr();
                self.flush(&a);
                a
            }
        }
    }
}

fn scan_string(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start + 1;
    let mut raw = String::from("\"");
    while i < chars.len() {
        raw.push(chars[i]);
        match chars[i] {
            '\\' if i + 1 < chars.len() => {
                raw.push(chars[i + 1]);
                i += 2;
                continue;
            }
            '"' => return (raw, i + 1),
            _ => {}
        }
        i += 1;
    }
    (raw, i)
}

/// Find the index of the `)` matching the `(` at `start` and the next index
/// after it. String- and comment-aware.
fn match_balanced(chars: &[char], start: usize) -> (usize, usize) {
    let mut depth = 0i32;
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            '"' => {
                let (_raw, next) = scan_string(chars, i);
                i = next;
                continue;
            }
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return (i, i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    (chars.len(), chars.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNITS: &str = "\
// header A
// header B
[
    // ── banner ──
    Section(
        \"Alpha\",
        [
            (\"Alpha_0_0\", Dervish, Infantry(fire: 3, melee: 6, movement: 9), (Black, White), Some(\"Alpha\")),
        ],
    ),
]
";

    #[test]
    fn units_addresses() {
        let s = scan(UNITS);
        assert_eq!(s.header, "// header A\n// header B");
        // Full-line comments keep their original indentation so serializers
        // can re-emit them byte-identically.
        assert_eq!(
            s.comments.get("[0]").map(String::as_str),
            Some("    // ── banner ──")
        );
        assert!(s.quoted.contains("[\n\nSection("));
        assert!(s.quoted.contains("(\"Alpha_0_0\", Dervish"));
    }

    #[test]
    fn brace_keys_are_quoted() {
        let s = scan("{ Row01to05: [NoEffect, Disrupt], Row41Plus: [] }");
        assert!(s.quoted.contains("\"Row01to05\" :"));
        assert!(s.quoted.contains("\"Row41Plus\" :"));
    }

    #[test]
    fn struct_fields_are_not_quoted() {
        let s = scan("(levels: {Ground: [Clear]}, cells: 1)");
        // `levels` is a struct field: unquoted. `Ground` is a map key: quoted.
        assert!(s.quoted.contains("levels :"));
        assert!(!s.quoted.contains("\"levels\""));
        assert!(s.quoted.contains("\"Ground\" :"));
    }

    #[test]
    fn tuple_keys_become_quoted_strings() {
        let s = scan("cells: { (Ground, Rough): [ (Units, []) ], (Rough, Rough): [] }");
        assert!(s.quoted.contains("\"(Ground, Rough)\" :"));
        assert!(s.quoted.contains("\"(Rough, Rough)\" :"));
    }

    #[test]
    fn comment_inside_map_value_list() {
        let text = "\
(
    cells: {
        (Ground, Rough): [
            // B: note.
            (Units, [CloserToFirer]),
        ],
    },
)
";
        let s = scan(text);
        assert_eq!(
            s.comments
                .get("cells/(Ground, Rough)/[0]")
                .map(String::as_str),
            Some("            // B: note.")
        );
    }

    #[test]
    fn nested_element_addresses() {
        let text = "[ A, B, (C) ]";
        let s = scan(text);
        assert!(s.comments.is_empty());
        assert_eq!(norm_key("  Ground,   Rough "), "Ground, Rough");
        assert_eq!(addr("cells", "(Ground, Rough)"), "cells/(Ground, Rough)");
        assert_eq!(elem("x", 3), "x/[3]");
    }

    #[test]
    fn some_and_enums_pass_through() {
        let s = scan("(a: Some(\"x\"), b: Eliminate(2), c: None)");
        assert!(s.quoted.contains("Some(\"x\")"));
        assert!(s.quoted.contains("Eliminate(2)"));
        assert!(s.quoted.contains("None"));
    }

    #[test]
    fn seq_scalars_bump_element_counter() {
        let text = "\
[
    Center,
    Left,
    // before third
    Right,
]
";
        let s = scan(text);
        assert_eq!(
            s.comments.get("[2]").map(String::as_str),
            Some("    // before third")
        );
    }
}

#[cfg(test)]
mod dump {
    use super::*;
    #[test]
    fn dump_quoted() {
        for name in [
            "combat_results_table.ron",
            "range_effects_table.ron",
            "los_table.ron",
            "order_of_appearance.ron",
        ] {
            let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../Boardgame - Remember_Gordon/tables")
                .join(name);
            let text = std::fs::read_to_string(&p).unwrap();
            let s = scan(&text);
            println!("==== {name} ====\n{}\n----", s.quoted);
        }
    }
}

#[cfg(test)]
mod dump2 {
    use super::*;
    #[test]
    fn minimal() {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../Boardgame - Remember_Gordon/tables/range_effects_table.ron"),
        )
        .unwrap();
        let s = scan(&text);
        let esc: String = s
            .quoted
            .chars()
            .take(80)
            .map(|c| {
                if c == '\n' {
                    "\\n".to_string()
                } else {
                    c.to_string()
                }
            })
            .collect();
        println!("MINIMAL: [{}]", esc);
        println!("COMMENTS: {:?}", s.comments);
        println!("HEADER: {:?}", s.header);
    }
}

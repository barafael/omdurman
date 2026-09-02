use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// TOML schema
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct Traceability {
    #[serde(rename = "mapping")]
    mappings: Vec<Mapping>,
}

#[derive(serde::Deserialize, Clone, Default)]
struct Mapping {
    section: String,
    title: String,
    status: String,
    #[serde(default)]
    page: Option<String>,
    #[serde(default)]
    tests: Vec<String>,
    /// Kani proof harnesses proving this mapping (see `scripts/kani.sh`).
    #[serde(default)]
    proofs: Vec<String>,
    #[serde(rename = "impl", default)]
    impls: Vec<ImplSite>,
}

#[derive(serde::Deserialize, Clone)]
struct ImplSite {
    file: String,
    line: u32,
    symbol: String,
}

// ---------------------------------------------------------------------------
// Manual section text extraction
// ---------------------------------------------------------------------------

fn parse_manual_sections(path: &Path) -> HashMap<String, String> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();

    #[derive(Clone)]
    struct Heading {
        number: String,
        line_idx: usize,
        inline: String,
    }

    let mut headings: Vec<Heading> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some((number, inline)) = parse_heading(trimmed) {
            headings.push(Heading {
                number,
                line_idx: i,
                inline,
            });
        }
    }

    let mut sections: HashMap<String, String> = HashMap::new();
    for (h_idx, heading) in headings.iter().enumerate() {
        let next_line_idx = headings
            .get(h_idx + 1)
            .map(|h| h.line_idx)
            .unwrap_or(lines.len());

        let mut text = String::new();
        if !heading.inline.is_empty() {
            text.push_str(heading.inline.trim());
            text.push('\n');
        }

        for l in &lines[(heading.line_idx + 1)..next_line_idx] {
            if l.trim() == "---" {
                continue;
            }
            text.push_str(l);
            text.push('\n');
        }

        let cleaned = text.trim().to_string();
        if !cleaned.is_empty() {
            sections.insert(heading.number.clone(), cleaned);
        }
    }

    for heading in &headings {
        sections.entry(heading.number.clone()).or_default();
    }

    sections
}

fn parse_heading(line: &str) -> Option<(String, String)> {
    if let Some(rest) = line.strip_prefix("**") {
        if let Some(pos) = rest.find(")**") {
            let num_part = &rest[..pos];
            if is_section_number(num_part) {
                let inline = rest[pos + 3..].trim().to_string();
                return Some((num_part.to_string(), inline));
            }
        }
    } else if line.starts_with('#') {
        let level = line.bytes().take_while(|&b| b == b'#').count();
        let after = line[level..].trim();
        if let Some(pos) = after.find(')') {
            let num_part = &after[..pos];
            if is_section_number(num_part) {
                let inline = after[pos + 1..].trim().to_string();
                return Some((num_part.to_string(), inline));
            }
        }
    }
    None
}

fn is_section_number(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_digit() || c == '.')
        && !s.starts_with('.')
        && !s.ends_with('.')
}

// ---------------------------------------------------------------------------
// Code snippet extraction (with line numbers)
// ---------------------------------------------------------------------------

fn extract_snippet_lines(path: &Path, target_line: u32, context: usize) -> Vec<(u32, String)> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }
    let idx = (target_line as usize).saturating_sub(1);
    let start = idx.saturating_sub(context);
    let end = std::cmp::min(idx + context + 1, lines.len());
    lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, l)| ((start + i + 1) as u32, l.to_string()))
        .collect()
}

fn file_extension(path: &str) -> &str {
    if let Some(pos) = path.rfind('.') {
        &path[pos + 1..]
    } else {
        ""
    }
}

// ---------------------------------------------------------------------------
// Typst content escaping
// ---------------------------------------------------------------------------

fn typst_content(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '#' => out.push_str("\\#"),
            '*' => out.push_str("\\*"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            c => out.push(c),
        }
    }
    out
}

/// Per-paragraph smart-quote substitution state machine.
///
/// Mirrors Typst's `SmartQuoter` (crates/typst-layout/src/inline/collect.rs,
/// crates/typst-library/src/text/smartquote.rs). Typst only makes quotes smart
/// in *markup*; the template renders manual text from strings, so the Rust side
/// must pre-substitute curly quotes to reproduce the markup path byte-for-byte.
/// A fresh quoter is used per paragraph (markup resets the state at each
/// paragraph boundary) and the `before` char is the last non-ignorable
/// character of the paragraph stream (including the substituted quotes).
struct SmartQuoter {
    /// The amount of quotes that have been opened.
    depth: u8,
    /// Each bit indicates whether the quote at this nesting depth is a double.
    kinds: u32,
}

impl SmartQuoter {
    fn new() -> Self {
        Self { depth: 0, kinds: 0 }
    }

    /// The kind of the most recently opened quote, if any.
    fn top(&self) -> Option<bool> {
        self.depth
            .checked_sub(1)
            .map(|i| (self.kinds >> i) & 1 == 1)
    }

    fn push(&mut self, double: bool) {
        if self.depth < 32 {
            self.kinds |= (double as u32) << self.depth;
            self.depth += 1;
        }
    }

    fn pop(&mut self) {
        self.depth -= 1;
        self.kinds &= (1 << self.depth) - 1;
    }

    /// Determine which smart quote to substitute given this quoter's nesting
    /// state and the character immediately preceding the quote. The English
    /// quote set (`‘ ’ “ ”`) is the fallback Typst uses for the default
    /// language.
    fn quote(&mut self, before: Option<char>, double: bool) -> &'static str {
        let opened = self.top();
        let before = before.unwrap_or(' ');

        // After a number, and without a quote of this kind open, produce a
        // prime (e.g. `188'4` → `188′4`).
        if before.is_numeric() && opened != Some(double) {
            return if double { "″" } else { "′" };
        }

        // A single quote after an alphabetic char (with no single quote open)
        // is an apostrophe.
        if !double && opened != Some(false) && (before.is_alphabetic() || before == '\u{FFFC}') {
            return "’";
        }

        // Close the most recently opened quotation of this kind unless the
        // preceding char suggests a nested quotation.
        if opened == Some(double)
            && !before.is_whitespace()
            && !is_newline(before)
            && !is_opening_bracket(before)
        {
            self.pop();
            return if double { "”" } else { "’" };
        }

        // Otherwise open a new quotation.
        self.push(double);
        if double { "“" } else { "‘" }
    }
}

/// Whether the character is a line break, matching Typst's `is_newline`
/// (crates/typst-syntax/src/lexer.rs).
fn is_newline(c: char) -> bool {
    matches!(
        c,
        '\n' | '\x0B' | '\x0C' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}'
    )
}

/// Whether the character is an opening bracket, parenthesis, or brace
/// (crates/typst-library/src/text/smartquote.rs).
fn is_opening_bracket(c: char) -> bool {
    matches!(c, '(' | '{' | '[')
}

/// The most common Unicode `Default_Ignorable_Code_Point`s, used when Typst
/// computes the character preceding a quote (crates/typst-library/src/text/
/// mod.rs). The manual contains none; this mirrors the look-back faithfully.
fn is_default_ignorable(c: char) -> bool {
    matches!(c,
        '\u{00AD}'          // SOFT HYPHEN
        | '\u{034F}'        // COMBINING GRAPHEME JOINER
        | '\u{061C}'        // ARABIC LETTER MARK
        | '\u{180B}'..='\u{180F}' // MONGOLIAN FREE VARIATION SELECTORS
        | '\u{200B}'        // ZERO WIDTH SPACE
        | '\u{200C}'..='\u{200F}' // ZERO WIDTH JOINER..RIGHT-TO-LEFT MARK
        | '\u{202A}'..='\u{202E}' // BIDI EMBEDDINGS
        | '\u{2060}'..='\u{2064}' // WORD JOINER..INVISIBLE PLUS
        | '\u{2066}'..='\u{206F}' // BIDI ISOLATES
        | '\u{FEFF}'        // ZERO WIDTH NO-BREAK SPACE
        | '\u{FE00}'..='\u{FE0F}' // VARIATION SELECTORS
        | '\u{E0000}'..='\u{E0FFF}' // TAGS
    )
}

/// Substitute smart quotes in a single paragraph of text. Quote substitution
/// must run over the *whole* paragraph (state carries across soft line breaks
/// inside a paragraph, and the `before` char is the last character emitted),
/// but each paragraph starts with a fresh quoter.
fn smart_quotes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut quoter = SmartQuoter::new();
    let mut before: Option<char> = None;

    for c in s.chars() {
        match c {
            '"' => {
                let quote = quoter.quote(before, true);
                out.push_str(quote);
                before = quote.chars().last();
            }
            '\'' => {
                let quote = quoter.quote(before, false);
                out.push_str(quote);
                before = quote.chars().last();
            }
            c => {
                out.push(c);
                if !is_default_ignorable(c) {
                    before = Some(c);
                }
            }
        }
    }

    out
}

/// Convert hyphen runs to their proper dashes for string-literal text.
///
/// In Typst *markup*, `--` smart-renders as an en-dash and `---` as an
/// em-dash; but inside a string argument to `#heading(...)` / `#text(...)`
/// the smart conversion is skipped and `--` renders literally as two hyphens.
/// Headings and the subtitle are emitted as string args, so normalize `--` →
/// `–` (en-dash) and `---` → `—` (em-dash) here to match what the author of
/// `--` expected to see. Runs longer than three hyphens are kept verbatim.
/// Code snippets (`#raw`) are never passed through this function.
fn dashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '-' {
            let start = i;
            while i < chars.len() && chars[i] == '-' {
                i += 1;
            }
            match i - start {
                1 => out.push('-'),
                2 => out.push('\u{2013}'), // –
                3 => out.push('\u{2014}'), // —
                n => out.push_str(&"-".repeat(n)),
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashes_normalizes_hyphen_runs() {
        assert_eq!(dashes("a--b"), "a–b");
        assert_eq!(dashes("a---b"), "a—b");
        assert_eq!(dashes("a--b---c-d"), "a–b—c-d");
        assert_eq!(dashes("----"), "----", "4+ hyphens stay verbatim");
        assert_eq!(dashes("§1 -- Introduction"), "§1 – Introduction");
        assert_eq!(
            dashes("Dervish set up -- deployment zones"),
            "Dervish set up – deployment zones"
        );
        assert_eq!(dashes("no dashes here"), "no dashes here");
    }

    #[test]
    fn dashes_keeps_unicode_intact() {
        assert_eq!(dashes("⇌ — §5.11"), "⇌ — §5.11");
    }

    #[test]
    fn smart_quotes_double_and_single() {
        assert_eq!(
            smart_quotes("\"1/2\""),
            "“1/2”",
            "open double wins over prime"
        );
        assert_eq!(smart_quotes("\"hello world\""), "“hello world”");
        assert_eq!(smart_quotes("188'4"), "188′4", "prime after digit");
        assert_eq!(smart_quotes("6.'64"), "6.‘64", "open after punctuation");
        assert_eq!(smart_quotes("Great Britain's"), "Great Britain’s");
        assert_eq!(smart_quotes("1820's"), "1820′s");
        assert_eq!(smart_quotes("\"a\" and 'b'"), "“a” and ‘b’");
    }

    #[test]
    fn smart_quotes_paragraph_boundaries() {
        assert_eq!(
            smart_quotes("a\n\"b"),
            "a\n“b",
            "state carries across soft break"
        );
        assert_eq!(
            smart_quotes("a\n\n\"b"),
            "a\n\n“b",
            "fresh quoter per paragraph"
        );
    }

    #[test]
    fn smart_quotes_after_opening_bracket_stays_open() {
        assert_eq!(
            smart_quotes("( \"x\""),
            "( “x”",
            "closing after bracket is normal"
        );
    }

    #[test]
    fn smart_quotes_does_not_touch_plain_text() {
        assert_eq!(
            smart_quotes("no quotes here — §5.11"),
            "no quotes here — §5.11"
        );
    }

    /// The committed `data.json` must match a fresh regeneration from
    /// `docs/traceability.toml` + the OCR manual. A stale artifact fails here;
    /// fix by re-running the generator and committing both outputs:
    ///
    /// ```sh
    /// cargo run -p traceability-typst -- docs/traceability.toml \
    ///     traceability.typ tools/traceability-typst/data.json
    /// ```
    #[test]
    fn committed_data_json_is_fresh() {
        let root = workspace_root();
        let toml_content =
            fs::read_to_string(root.join("docs/traceability.toml")).expect("read toml");
        let table: Traceability = toml::from_str(&toml_content).expect("parse toml");
        let manual_path = root
            .join("Boardgame - Remember_Gordon/Manual")
            .join("RememberGordonManual.md");
        let manual_sections = parse_manual_sections(&manual_path);
        let data = build_data(&table, &manual_sections, &root);
        let fresh = serde_json::to_string_pretty(&data).expect("serialize data JSON");

        let committed_path = root.join("tools/traceability-typst/data.json");
        let committed = fs::read_to_string(&committed_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", committed_path.display()));
        assert_eq!(
            fresh.trim_end(),
            committed.trim_end(),
            "{} is stale -- regenerate with `cargo run -p traceability-typst -- \
             docs/traceability.toml traceability.typ tools/traceability-typst/data.json` \
             and commit it",
            committed_path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// Section reference extraction + cross-reference linking
// ---------------------------------------------------------------------------

fn section_label(section: &str) -> String {
    let num = section.trim_start_matches('§');
    format!("sect-{}", num.replace('.', "-"))
}

/// Scan `text` for `N.M` patterns that match a known rulebook section.
/// Returns (byte_start, byte_end, section_number) tuples in order.
fn find_section_refs(text: &str, known_sections: &BTreeSet<String>) -> Vec<(usize, usize, String)> {
    let mut refs = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            let num = &text[start..i];
            if num.contains('.') && known_sections.contains(&format!("§{num}")) {
                refs.push((start, i, num.to_string()));
            }
        } else {
            i += 1;
        }
    }

    refs
}

/// Escape text for Typst, turning known § references into internal links.
fn typst_content_with_links(s: &str, known_sections: &BTreeSet<String>) -> String {
    let refs = find_section_refs(s, known_sections);

    let mut out = String::new();
    let mut last_end = 0;

    for (start, end, section_num) in refs {
        let plain = &s[last_end..start];
        out.push_str(&typst_content(plain));

        let label = section_label(&format!("§{section_num}"));
        out.push_str(&format!("#link(<{label}>)[{section_num}]"));

        last_end = end;
    }

    out.push_str(&typst_content(&s[last_end..]));
    out
}

/// Build a "See also: §X, §Y" line from the § refs found in `manual_text`.
fn see_also_links(
    current_section: &str,
    manual_text: &str,
    known_sections: &BTreeSet<String>,
) -> Option<String> {
    let refs = find_section_refs(manual_text, known_sections);
    let unique: BTreeSet<String> = refs.into_iter().map(|(_, _, s)| s).collect();

    let mut links: Vec<String> = Vec::new();
    for num in &unique {
        let full = format!("§{num}");
        if full != current_section {
            let label = section_label(&full);
            links.push(format!("#link(<{label}>)[§{num}]"));
        }
    }

    if links.is_empty() {
        None
    } else {
        Some(format!(
            "#text(size: 8.5pt, fill: luma(120), style: \"italic\")[See also: {}]",
            links.join(", ")
        ))
    }
}

// ---------------------------------------------------------------------------
// Chapter grouping helpers
// ---------------------------------------------------------------------------

const PSEUDO_SECTIONS: &[&str] = &["§Credits", "§Reference", "§CRT"];

fn chapter_key(section: &str) -> String {
    if PSEUDO_SECTIONS.contains(&section) {
        return section.to_string();
    }
    let num = section.trim_start_matches('§');
    let chapter = num.split('.').next().unwrap_or(num);
    format!("§{chapter}")
}

fn sort_key(section: &str) -> Vec<u32> {
    section
        .trim_start_matches('§')
        .split('.')
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

fn chapter_title(key: &str) -> String {
    match key {
        "§1" => "§1 -- Introduction".into(),
        "§2" => "§2 -- Game Components".into(),
        "§3" => "§3 -- Getting Started".into(),
        "§4" => "§4 -- Turn Sequence".into(),
        "§5" => "§5 -- Movement Phase".into(),
        "§6" => "§6 -- Fire Combat Phase".into(),
        "§7" => "§7 -- Melee Phase".into(),
        "§8" => "§8 -- Night Game Turns".into(),
        "§9" => "§9 -- The Scenarios".into(),
        "§10" => "§10 -- Optional Rules".into(),
        "§11" => "§11 -- Historical Notes".into(),
        "§Credits" => "Credits".into(),
        "§Reference" => "Reference -- Charts and Tables".into(),
        "§CRT" => "Combat Results Table (shared reference)".into(),
        _ => key.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Typst preamble
// ---------------------------------------------------------------------------

fn generate_preamble(root: &Path) -> String {
    let root_str = root.to_string_lossy().replace('\\', "/");
    format!(
        r##"#set page(paper: "a4", margin: (top: 2cm, bottom: 2cm, left: 2.5cm, right: 2cm))
#set text(font: ("EB Garamond", "Libertinus Serif", "DejaVu Serif"), size: 10pt)
#set par(justify: true, leading: 0.5em)
#set heading(numbering: none)

#show raw.where(block: true): set text(font: ("DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"), size: 7.5pt)
#show raw.where(block: false): set text(font: ("DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"), size: 8.5pt)
#show raw.where(block: true): block.with(fill: luma(248), inset: 0.4em, radius: 2pt)

#show heading.where(level: 1): it => {{
  pagebreak()
  v(1em)
  block(stroke: (left: 3pt + luma(80)), inset: (top: 0.2em, bottom: 0.2em, left: 0.6em, right: 0.3em), it)
}}

#show heading.where(level: 2): it => {{
  v(0.6em)
  it
}}

#let status-tag(status) = {{
  let (bg, fg) = if status == "implemented" {{
    (green.transparentize(70%), green.darken(30%))
  }} else if status == "descriptive" {{
    (blue.transparentize(70%), blue.darken(30%))
  }} else if status == "implicit" {{
    (yellow.transparentize(70%), yellow.darken(40%))
  }} else {{
    (luma(85), luma(40))
  }};
  box(
    fill: bg, inset: (left: 0.4em, right: 0.4em, top: 0.1em, bottom: 0.1em),
    radius: 3pt, text(fill: fg, size: 8pt, weight: "bold", status)
  )
}}

#let root = "{root_str}"

#let vscode-link(rel, line) = {{
  let abs = root + "/" + rel
  link("vscode://file/" + abs + ":" + str(line))[
    #text(size: 9pt, fill: blue.darken(20%), rel + ":" + str(line))
  ]
}}

#let github-link(rel, line) = {{
  let url = "https://github.com/barafael/omdurman/blob/HEAD/" + rel + "#L" + str(line)
  link(url)[
    #text(size: 8pt, fill: luma(100), "GH:" + rel + ":" + str(line))
  ]
}}

#let progress-bar(done, total) = {{
  let filled = "█" * done
  let empty = "░" * (total - done)
  text(font: ("DejaVu Sans Mono", "Liberation Mono"), size: 8pt)[
    #text(fill: green.darken(20%))[#filled]#text(fill: luma(180))[#empty] #done/#total implemented
  ]
}}
"##,
    )
}

// ---------------------------------------------------------------------------
// Typst document generation
// ---------------------------------------------------------------------------

const COLLAPSE_THRESHOLD: usize = 500;

fn generate_typst(
    table: &Traceability,
    manual_sections: &HashMap<String, String>,
    root: &Path,
) -> String {
    let mut out = String::new();

    let known_sections: BTreeSet<String> =
        table.mappings.iter().map(|m| m.section.clone()).collect();

    // -- Preamble -----------------------------------------------------------
    out.push_str(&generate_preamble(root));

    // -- Title block --------------------------------------------------------
    out.push_str("#align(center, text(size: 18pt, weight: \"bold\", \"Traceability Matrix\"))\n");
    out.push_str("#align(center, text(size: 10pt, \"REMEMBER GORDON! – Rulebook ⇌ Implementation Mapping\"))\n");
    out.push_str(
        "#align(center, text(size: 9pt, fill: luma(120), \"Generated from `docs/traceability.toml`\"))\n",
    );
    out.push_str("#v(2em)\n");

    // -- Overview ----------------------------------------------------------
    let total_by_status = {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for m in &table.mappings {
            *counts.entry(m.status.clone()).or_default() += 1;
        }
        counts
    };
    let total_impl_sites: usize = table.mappings.iter().map(|m| m.impls.len()).sum();

    let imp = total_by_status.get("implemented").copied().unwrap_or(0);
    let desc = total_by_status.get("descriptive").copied().unwrap_or(0);
    let impli = total_by_status.get("implicit").copied().unwrap_or(0);
    let oos = total_by_status.get("out-of-scope").copied().unwrap_or(0);

    out.push_str("#heading(level: 1, \"Overview\") <sect-overview>\n");
    out.push_str("#v(0.3em)\n");
    out.push_str("#table(\n");
    out.push_str("  columns: (1fr, 1fr, 1fr, 1fr),\n");
    out.push_str("  stroke: 0.4pt + luma(190),\n");
    out.push_str("  [*Implemented*], [*Descriptive*], [*Implicit*], [*Out-of-scope*],\n");
    out.push_str(&format!(
        "  [#text(fill: green.darken(20%))[{imp}]], [#text(fill: blue.darken(20%))[{desc}]], [#text(fill: yellow.darken(30%))[{impli}]], [{oos}],\n"
    ));
    out.push_str(")\n");
    out.push_str(&format!(
        "#v(0.3em)\n#text(size: 9pt)[Total mappings: {} · Total impl sites: {}]\n",
        table.mappings.len(),
        total_impl_sites
    ));
    out.push_str("#v(1em)\n");

    // -- Table of Contents --------------------------------------------------
    out.push_str("#outline(title: [Table of Contents])\n");
    out.push_str("#pagebreak()\n");

    // -- Group mappings by chapter ------------------------------------------
    let mut chapter_groups: HashMap<String, Vec<&Mapping>> = HashMap::new();
    let mut chapter_order: Vec<String> = Vec::new();

    for m in &table.mappings {
        let key = chapter_key(&m.section);
        if !chapter_groups.contains_key(&key) {
            chapter_order.push(key.clone());
        }
        chapter_groups.entry(key).or_default().push(m);
    }

    for key in &chapter_order {
        let mappings = &chapter_groups[key];
        let title = chapter_title(key);

        // Per-chapter progress bar
        let total = mappings.len();
        let done = mappings
            .iter()
            .filter(|m| m.status == "implemented")
            .count();
        out.push_str(&format!("#progress-bar({done}, {total})\n"));

        // Chapter heading with label
        let label = section_label(key);
        out.push_str(&format!(
            "#heading(level: 1, \"{}\") <{label}>\n",
            dashes(&title)
        ));

        // Sort sub-sections
        let mut sorted = mappings.clone();
        sorted.sort_by_key(|a| sort_key(&a.section));

        for m in sorted {
            let section_num = m.section.trim_start_matches('§');
            let manual_text = manual_sections
                .get(section_num)
                .map(|s| s.as_str())
                .unwrap_or("");

            // Section heading with label (skip label when it would duplicate the chapter heading)
            let heading_text = dashes(&format!("{} -- {}", m.section, m.title));
            if m.section == *key {
                out.push_str(&format!("#heading(level: 2, \"{heading_text}\")\n"));
            } else {
                let sect_label = section_label(&m.section);
                out.push_str(&format!(
                    "#heading(level: 2, \"{heading_text}\") <{sect_label}>\n"
                ));
            }

            // Status tag
            out.push_str(&format!("#status-tag(\"{}\")\n", m.status));

            // Page number (feature 10: emit "unknown" when missing)
            match &m.page {
                Some(page) => {
                    out.push_str(&format!(
                        "#linebreak()\n#text(size: 8.5pt, fill: luma(120))[manual page {}]\n",
                        page
                    ));
                }
                None => {
                    out.push_str("#linebreak()\n#text(size: 8.5pt, fill: luma(120), style: \"italic\")[manual page unknown]\n");
                }
            }
            out.push_str("#v(0.3em)\n");

            // Rule text from manual (with cross-reference links, possibly collapsed)
            if !manual_text.is_empty() {
                let escaped = typst_content_with_links(manual_text, &known_sections);
                let is_long = manual_text.len() > COLLAPSE_THRESHOLD;

                if is_long {
                    out.push_str(&format!(
                        "#stack(\n  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[{escaped}]],\n  align(right, text(size: 8pt, fill: luma(120), style: \"italic\")[(see manual for full text)])\n)\n"
                    ));
                } else {
                    out.push_str(&format!(
                        "#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[{escaped}]]\n"
                    ));
                }
                out.push_str("#v(0.5em)\n");
            }

            // "See also" cross-references
            if !manual_text.is_empty()
                && let Some(see_also) = see_also_links(&m.section, manual_text, &known_sections)
            {
                out.push_str(&see_also);
                out.push_str("\n#v(0.3em)\n");
            }

            // Implementation sites (with GitHub links, line numbers, highlighted symbols)
            if !m.impls.is_empty() {
                out.push_str("#table(\n");
                out.push_str("  columns: (1.2fr, 1.8fr, 5fr),\n");
                out.push_str("  stroke: 0.4pt + luma(190),\n");
                out.push_str("  [*File*], [*Symbol*], [*Code Snippet*],\n");

                for imp in &m.impls {
                    let file_path = root.join(&imp.file);
                    let snippet_lines = extract_snippet_lines(&file_path, imp.line, 2);
                    let ext = file_extension(&imp.file);

                    // Build snippet with line numbers
                    let snippet: String = snippet_lines
                        .iter()
                        .map(|(num, line)| format!("{:>3} │ {}", num, line))
                        .collect::<Vec<_>>()
                        .join("\n");

                    let snippet_esc = snippet.replace('\\', "\\\\").replace('"', "\\\"");

                    // File cell with both VS Code and GitHub links
                    out.push_str(&format!(
                        "  [#vscode-link(\"{}\", {}) \\ #github-link(\"{}\", {})],",
                        imp.file, imp.line, imp.file, imp.line
                    ));

                    // Symbol cell with highlight, linked to GitHub
                    out.push_str(&format!(
                        "  [#link(\"https://github.com/barafael/omdurman/blob/HEAD/{}#L{}\")[#highlight(fill: yellow.transparentize(70%))[#text(weight: \"bold\")[{}]]]],",
                        imp.file, imp.line, imp.symbol
                    ));

                    if snippet.is_empty() {
                        out.push_str(" [],\n");
                    } else {
                        out.push_str(&format!(
                            " [#raw(\"{}\", block: true, lang: \"{}\")],\n",
                            snippet_esc, ext
                        ));
                    }
                }

                out.push_str(")\n");
                out.push_str("#v(0.5em)\n");
            }

            // Kani proof coverage list. Rendered before the test list and in
            // blue rather than green: a proof covers its whole bounded input
            // domain, a test covers the cases it enumerates. Must stay in step
            // with the `#list`/`#enum` template path (see CLAUDE.md).
            if !m.proofs.is_empty() {
                let proof_tags: Vec<String> = m
                    .proofs
                    .iter()
                    .map(|t| {
                        format!(
                            "#box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: \"bold\")[{t}]]"
                        )
                    })
                    .collect();
                out.push_str(&format!(
                    "#text(size: 9pt, fill: luma(80))[Proven by: {}]
",
                    proof_tags.join(" ")
                ));
                out.push_str(
                    "#v(0.3em)
",
                );
            }

            // Test coverage list
            if !m.tests.is_empty() {
                let test_tags: Vec<String> = m
                    .tests
                    .iter()
                    .map(|t| {
                        format!(
                            "#box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: \"bold\")[{t}]]"
                        )
                    })
                    .collect();
                out.push_str(&format!(
                    "#text(size: 9pt, fill: luma(80))[Covered by tests: {}]\n",
                    test_tags.join(" ")
                ));
                out.push_str("#v(0.3em)\n");
            }
        }
    }

    // -- Symbol index (feature 8) -------------------------------------------
    let mut symbol_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for m in &table.mappings {
        for imp in &m.impls {
            let key = imp
                .symbol
                .rsplit("::")
                .next()
                .unwrap_or(&imp.symbol)
                .to_string();
            symbol_index
                .entry(key)
                .or_default()
                .insert(m.section.clone());
        }
    }

    if !symbol_index.is_empty() {
        out.push_str("#heading(level: 1, \"Appendix: Symbol Index\") <sect-symbol-index>\n");
        out.push_str("#v(0.5em)\n");
        out.push_str("#table(\n");
        out.push_str("  columns: (2fr, 5fr),\n");
        out.push_str("  stroke: 0.4pt + luma(190),\n");
        out.push_str("  [*Symbol*], [*Sections*],\n");

        for (symbol, sections) in &symbol_index {
            let section_links: Vec<String> = sections
                .iter()
                .map(|s| {
                    let label = section_label(s);
                    format!("#link(<{label}>)[{s}]")
                })
                .collect();

            out.push_str(&format!(
                "  [#text(weight: \"bold\", size: 9pt)[{symbol}]], [{}],\n",
                section_links.join(", ")
            ));
        }

        out.push_str(")\n");
    }

    out
}

// ---------------------------------------------------------------------------
// Data-driven JSON generation (spike: data-in / template-out split)
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct DataDocument {
    root: String,
    total_mappings: usize,
    total_impl_sites: usize,
    status_counts: BTreeMap<String, usize>,
    chapters: Vec<ChapterData>,
    symbol_index: Vec<SymbolIndexEntry>,
}

#[derive(serde::Serialize)]
struct ChapterData {
    key: String,
    title: String,
    done: usize,
    total: usize,
    sections: Vec<SectionData>,
}

#[derive(serde::Serialize)]
struct SectionData {
    section: String,
    heading: String,
    status: String,
    page: Option<String>,
    manual: Vec<ManualBlock>,
    collapsed: bool,
    see_also: Vec<String>,
    impls: Vec<ImplData>,
    tests: Vec<String>,
    proofs: Vec<String>,
}

/// One tokenized piece of manual prose: plain text, a `§` reference, or inline
/// code (from backtick spans).
#[derive(serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Segment {
    Text { text: String },
    Ref { rref: String },
    Raw { text: String },
}

/// A structural block of manual prose. Markup lists render as real `list()`
/// / `enum()` blocks so the template reproduces the `•`/`1.` markers the
/// markup path produced automatically.
///
/// `loose` mirrors Typst's loose/tight list distinction: a blank line between
/// any two items (or before an item's nested content) makes the whole list
/// non-tight, adding ~0.65em between every item. `blank_before` records
/// whether a blank line preceded this block in the source — that is the exact
/// signal for whether the old markup path had a blank line between this list
/// and the paragraph before it (which renders as an extra `#parbreak()`).
#[derive(serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ManualBlock {
    Paragraph {
        segments: Vec<Segment>,
    },
    List {
        items: Vec<ListItem>,
        loose: bool,
        blank_before: bool,
    },
    Enum {
        items: Vec<ListItem>,
        loose: bool,
        blank_before: bool,
    },
}

#[derive(serde::Serialize)]
struct ListItem {
    blocks: Vec<ManualBlock>,
}

#[derive(serde::Serialize)]
struct ImplData {
    file: String,
    line: u32,
    symbol: String,
    snippet: String,
    ext: String,
}

#[derive(serde::Serialize)]
struct SymbolIndexEntry {
    symbol: String,
    sections: Vec<String>,
}

/// Split a paragraph of manual prose into segments: plain text, `§` references
/// (rendered as `N.M` in both paths), and inline code (backtick spans).
/// `smart_quotes` is applied to the whole paragraph first — Typst makes quotes
/// smart only in markup, so the string-based template must get the same
/// substitution here. Backticks are split out after that so their content is
/// never quote/dash-processed, matching markup's inline `raw` handling.
fn segmentize(text: &str, known_sections: &BTreeSet<String>) -> Vec<Segment> {
    let text = dashes(text);
    let text = smart_quotes(&text);

    let positions: Vec<usize> = text.match_indices('`').map(|(i, _)| i).collect();
    if positions.len() < 2 || !positions.len().is_multiple_of(2) {
        // Lone/unbalanced backticks fall through as plain text.
        return ref_segments(text, known_sections);
    }

    let mut segments: Vec<Segment> = Vec::new();
    let mut last = 0;
    for pair in positions.chunks(2) {
        let start = pair[0];
        let end = pair[1];
        if start > last {
            segments.extend(ref_segments(text[last..start].to_string(), known_sections));
        }
        segments.push(Segment::Raw {
            text: text[start + 1..end].to_string(),
        });
        last = end + 1;
    }
    if last < text.len() {
        segments.extend(ref_segments(text[last..].to_string(), known_sections));
    }
    segments
}

/// Split plain text into `Text` / `Ref` segments.
fn ref_segments(text: String, known_sections: &BTreeSet<String>) -> Vec<Segment> {
    let refs = find_section_refs(&text, known_sections);
    if refs.is_empty() {
        return vec![Segment::Text { text }];
    }

    let mut segments = Vec::new();
    let mut last_end = 0;
    for (start, end, num) in refs {
        if start > last_end {
            segments.push(Segment::Text {
                text: text[last_end..start].to_string(),
            });
        }
        segments.push(Segment::Ref { rref: num });
        last_end = end;
    }
    if last_end < text.len() {
        segments.push(Segment::Text {
            text: text[last_end..].to_string(),
        });
    }
    segments
}

/// Raw structural blocks (paragraph strings + lists) before segmentizing.
/// Lists carry their `loose` status; `blank_before` is only meaningful for
/// top-level blocks and is set by `parse_manual_blocks`.
enum RawBlock {
    Paragraph(String),
    List(Vec<RawItem>, bool, bool),
    Enum(Vec<RawItem>, bool, bool),
}

struct RawItem {
    blocks: Vec<RawBlock>,
}

/// The kind of a list marker: `- ` (unordered) or `N. ` (ordered).
#[derive(Clone, Copy, PartialEq, Eq)]
enum ListKind {
    List,
    Enum,
}

/// Whether `line` (with leading whitespace already stripped) opens a list item.
fn list_marker(line: &str) -> Option<ListKind> {
    if line.starts_with("- ") {
        return Some(ListKind::List);
    }
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i > 0 && bytes.get(i) == Some(&b'.') && bytes.get(i + 1) == Some(&b' ') {
        return Some(ListKind::Enum);
    }
    None
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

/// Parse a run of list items at `indent` (all with `kind` markers) starting at
/// `i`. Returns the items, the index of the first line after the list, and
/// whether the list is "loose". Typst's markup parser makes a list non-tight
/// (loose) when a blank line appears between two items or between an item and
/// its nested content; the whole list then gets paragraph spacing (~0.65em)
/// between every item instead of just the tight leading gutter.
fn parse_list(
    lines: &[&str],
    i: usize,
    indent: usize,
    kind: ListKind,
) -> (Vec<RawItem>, usize, bool) {
    let mut items = Vec::new();
    let mut loose = false;
    let mut i = i;
    while i < lines.len() {
        let line = lines[i];
        if is_blank(line) {
            i += 1;
            continue;
        }
        let li = line.len() - line.trim_start().len();
        let rest = line.trim_start();
        if li < indent || list_marker(rest) != Some(kind) {
            break;
        }
        // Item text, plus any continuation / nested-list content.
        let text = item_text(rest, kind);
        let mut blocks: Vec<RawBlock> = vec![RawBlock::Paragraph(text)];
        let mut idx = i + 1;
        let mut has_nested = false;
        let mut pending_blank = false;
        while idx < lines.len() {
            let line = lines[idx];
            if is_blank(line) {
                pending_blank = true;
                idx += 1;
                continue;
            }
            let li2 = line.len() - line.trim_start().len();
            let rest2 = line.trim_start();
            if li2 <= indent {
                // Next sibling item or end of the run. A blank line between
                // two sibling items makes the list loose.
                if pending_blank && li2 == indent && list_marker(rest2) == Some(kind) {
                    loose = true;
                }
                break;
            }
            // Blank line before nested content also makes the list loose
            // (Typst treats the item body as separate paragraphs).
            if pending_blank {
                loose = true;
            }
            pending_blank = false;
            if let Some(kind2) = list_marker(rest2) {
                let (sub, next, loose_sub) = parse_list(lines, idx, li2, kind2);
                blocks.push(if kind2 == ListKind::List {
                    RawBlock::List(sub, loose_sub, false)
                } else {
                    RawBlock::Enum(sub, loose_sub, false)
                });
                idx = next;
                has_nested = true;
            } else {
                // Indented prose: continuation of the item's paragraph, or a
                // new paragraph after a nested list (mirrors Typst markup).
                // Join with a SPACE, not a newline: a `\n` would render as a
                // hard line break, while the old markup path soft-breaks.
                let prose = rest2.trim_end().to_string();
                if !has_nested {
                    match blocks.first_mut() {
                        Some(RawBlock::Paragraph(p)) => {
                            p.push(' ');
                            p.push_str(&prose);
                        }
                        _ => blocks.push(RawBlock::Paragraph(prose)),
                    }
                } else if let Some(RawBlock::Paragraph(p)) = blocks.last_mut() {
                    p.push(' ');
                    p.push_str(&prose);
                } else {
                    blocks.push(RawBlock::Paragraph(prose));
                }
                idx += 1;
            }
        }
        items.push(RawItem { blocks });
        i = idx;
    }
    (items, i, loose)
}

/// The text of a list item line (after the `- ` / `N. ` marker).
fn item_text(rest: &str, kind: ListKind) -> String {
    if kind == ListKind::List {
        rest[2..].to_string()
    } else {
        // `N. text` → text after the marker.
        let mut i = 0;
        let bytes = rest.as_bytes();
        while bytes[i].is_ascii_digit() {
            i += 1;
        }
        rest[i + 2..].to_string()
    }
}

/// Parse the manual section text into structural blocks (paragraphs, bullet
/// lists, ordered lists) mirroring how Typst markup parses the same text
/// inside `#quote(block: true)[...]`. This is the string-literal counterpart to
/// the old generator's markup emission: quotes are smart-converted, `- ` / `N.`
/// prefixes become real lists, and paragraph flow is preserved.
fn parse_manual_blocks(text: &str, known_sections: &BTreeSet<String>) -> Vec<ManualBlock> {
    let lines: Vec<&str> = text.lines().collect();
    let mut blocks: Vec<RawBlock> = Vec::new();
    let mut i = 0;
    let mut saw_blank = false;

    while i < lines.len() {
        let line = lines[i];
        if is_blank(line) {
            saw_blank = true;
            i += 1;
            continue;
        }
        let rest = line.trim_start();
        if let Some(kind) = list_marker(rest) {
            let (items, next, loose) = parse_list(&lines, i, 0, kind);
            blocks.push(if kind == ListKind::List {
                RawBlock::List(items, loose, saw_blank)
            } else {
                RawBlock::Enum(items, loose, saw_blank)
            });
            saw_blank = false;
            i = next;
        } else {
            // Paragraph: contiguous prose lines (leading/trailing whitespace
            // ignored), terminated by a blank line or a top-level list marker.
            let mut para = String::new();
            while i < lines.len() {
                let line = lines[i];
                if is_blank(line) {
                    break;
                }
                let rest = line.trim_start().trim_end();
                if list_marker(rest).is_some() {
                    break;
                }
                if !para.is_empty() {
                    para.push(' ');
                }
                para.push_str(rest);
                i += 1;
            }
            blocks.push(RawBlock::Paragraph(para));
            saw_blank = false;
        }
    }

    blocks
        .into_iter()
        .map(|b| match b {
            RawBlock::Paragraph(text) => ManualBlock::Paragraph {
                segments: segmentize(&text, known_sections),
            },
            RawBlock::List(items, loose, blank_before) => ManualBlock::List {
                items: items
                    .into_iter()
                    .map(|it| ListItem {
                        blocks: raw_blocks_to_manual(it.blocks, known_sections),
                    })
                    .collect(),
                loose,
                blank_before,
            },
            RawBlock::Enum(items, loose, blank_before) => ManualBlock::Enum {
                items: items
                    .into_iter()
                    .map(|it| ListItem {
                        blocks: raw_blocks_to_manual(it.blocks, known_sections),
                    })
                    .collect(),
                loose,
                blank_before,
            },
        })
        .collect()
}

fn raw_blocks_to_manual(
    blocks: Vec<RawBlock>,
    known_sections: &BTreeSet<String>,
) -> Vec<ManualBlock> {
    blocks
        .into_iter()
        .map(|b| match b {
            RawBlock::Paragraph(text) => ManualBlock::Paragraph {
                segments: segmentize(&text, known_sections),
            },
            // Nested blocks are always "attached" in the old markup (never
            // separated by a blank line from their parent item), so their
            // `blank_before` is always false; the parbreak rule is disabled
            // for them anyway via `attach: true` in the template.
            RawBlock::List(items, loose, _) => ManualBlock::List {
                items: items
                    .into_iter()
                    .map(|it| ListItem {
                        blocks: raw_blocks_to_manual(it.blocks, known_sections),
                    })
                    .collect(),
                loose,
                blank_before: false,
            },
            RawBlock::Enum(items, loose, _) => ManualBlock::Enum {
                items: items
                    .into_iter()
                    .map(|it| ListItem {
                        blocks: raw_blocks_to_manual(it.blocks, known_sections),
                    })
                    .collect(),
                loose,
                blank_before: false,
            },
        })
        .collect()
}

/// "See also: §X, §Y" target list, minus the section itself.
fn see_also_list(
    current_section: &str,
    manual_text: &str,
    known_sections: &BTreeSet<String>,
) -> Vec<String> {
    let refs = find_section_refs(manual_text, known_sections);
    let unique: BTreeSet<String> = refs.into_iter().map(|(_, _, s)| s).collect();

    unique
        .into_iter()
        .filter(|num| format!("§{num}") != current_section)
        .map(|num| format!("§{num}"))
        .collect()
}

fn build_impl(imp: &ImplSite, root: &Path) -> ImplData {
    let file_path = root.join(&imp.file);
    let snippet_lines = extract_snippet_lines(&file_path, imp.line, 2);
    let snippet: String = snippet_lines
        .iter()
        .map(|(num, line)| format!("{:>3} │ {}", num, line))
        .collect::<Vec<_>>()
        .join("\n");

    ImplData {
        file: imp.file.clone(),
        line: imp.line,
        symbol: imp.symbol.clone(),
        snippet,
        ext: file_extension(&imp.file).to_string(),
    }
}

/// Extract all rulebook data from the TOML + manual into a JSON-serializable
/// document. Mirror of `generate_typst`: any change to the data shape must be
/// mirrored in `traceability-template.typ`.
fn build_data(
    table: &Traceability,
    manual_sections: &HashMap<String, String>,
    root: &Path,
) -> DataDocument {
    let known_sections: BTreeSet<String> =
        table.mappings.iter().map(|m| m.section.clone()).collect();

    let mut chapter_groups: HashMap<String, Vec<&Mapping>> = HashMap::new();
    let mut chapter_order: Vec<String> = Vec::new();
    for m in &table.mappings {
        let key = chapter_key(&m.section);
        if !chapter_groups.contains_key(&key) {
            chapter_order.push(key.clone());
        }
        chapter_groups.entry(key).or_default().push(m);
    }

    let mut chapters: Vec<ChapterData> = Vec::new();
    for key in &chapter_order {
        let mappings = &chapter_groups[key];
        let done = mappings
            .iter()
            .filter(|m| m.status == "implemented")
            .count();

        let mut sorted = mappings.clone();
        sorted.sort_by_key(|a| sort_key(&a.section));

        let sections = sorted
            .into_iter()
            .map(|m| {
                let section_num = m.section.trim_start_matches('§');
                let manual_text = manual_sections
                    .get(section_num)
                    .map(String::as_str)
                    .unwrap_or("");

                SectionData {
                    section: m.section.clone(),
                    heading: dashes(&format!("{} -- {}", m.section, m.title)),
                    status: m.status.clone(),
                    page: m.page.clone(),
                    manual: parse_manual_blocks(manual_text, &known_sections),
                    collapsed: manual_text.len() > COLLAPSE_THRESHOLD,
                    see_also: see_also_list(&m.section, manual_text, &known_sections),
                    impls: m.impls.iter().map(|imp| build_impl(imp, root)).collect(),
                    tests: m.tests.clone(),
                    proofs: m.proofs.clone(),
                }
            })
            .collect();

        chapters.push(ChapterData {
            key: key.clone(),
            title: dashes(&chapter_title(key)),
            done,
            total: mappings.len(),
            sections,
        });
    }

    let mut symbol_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for m in &table.mappings {
        for imp in &m.impls {
            let key = imp
                .symbol
                .rsplit("::")
                .next()
                .unwrap_or(&imp.symbol)
                .to_string();
            symbol_index
                .entry(key)
                .or_default()
                .insert(m.section.clone());
        }
    }
    let symbol_index: Vec<SymbolIndexEntry> = symbol_index
        .into_iter()
        .map(|(symbol, sections)| SymbolIndexEntry {
            symbol,
            sections: sections.into_iter().collect(),
        })
        .collect();

    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();
    for m in &table.mappings {
        *status_counts.entry(m.status.clone()).or_default() += 1;
    }

    DataDocument {
        root: root.to_string_lossy().replace('\\', "/"),
        total_mappings: table.mappings.len(),
        total_impl_sites: table.mappings.iter().map(|m| m.impls.len()).sum(),
        status_counts,
        chapters,
        symbol_index,
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = workspace_root();

    let toml_path = if args.len() > 1 {
        root.join(&args[1])
    } else {
        root.join("docs/traceability.toml")
    };

    let output_path: Option<PathBuf> = if args.len() > 2 {
        Some(root.join(&args[2]))
    } else {
        None
    };

    // Optional 3rd positional arg: write the data document as JSON (the
    // input for `traceability-template.typ`).
    let json_output: Option<PathBuf> = if args.len() > 3 {
        Some(root.join(&args[3]))
    } else {
        None
    };

    // Read TOML
    let toml_content = fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", toml_path.display()));
    let table: Traceability = toml::from_str(&toml_content).expect("Invalid traceability.toml");

    // Read and parse manual
    let manual_root = root.join("Boardgame - Remember_Gordon/Manual");
    let manual_path = manual_root.join("RememberGordonManual.md");
    let manual_sections = parse_manual_sections(&manual_path);

    // Generate Typst
    let output = generate_typst(&table, &manual_sections, &root);

    // Write output
    if let Some(out_path) = output_path {
        fs::write(&out_path, &output)
            .unwrap_or_else(|e| panic!("Cannot write {}: {e}", out_path.display()));
        eprintln!("Wrote {}", out_path.display());
    } else {
        println!("{output}");
    }

    // Write data JSON (spike path)
    if let Some(json_path) = json_output {
        let data = build_data(&table, &manual_sections, &root);
        let json = serde_json::to_string_pretty(&data).expect("serialize data JSON");
        fs::write(&json_path, &json)
            .unwrap_or_else(|e| panic!("Cannot write {}: {e}", json_path.display()));
        eprintln!("Wrote {}", json_path.display());
    }
}

fn workspace_root() -> PathBuf {
    let this = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    this.parent().unwrap().parent().unwrap().to_path_buf()
}

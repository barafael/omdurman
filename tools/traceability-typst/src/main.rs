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
    out.push_str("#align(center, text(size: 10pt, \"REMEMBER GORDON! -- Rulebook ⇌ Implementation Mapping\"))\n");
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
        out.push_str(&format!("#heading(level: 1, \"{title}\") <{label}>\n"));

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
            if m.section == *key {
                out.push_str(&format!(
                    "#heading(level: 2, \"{} -- {}\")\n",
                    m.section, m.title
                ));
            } else {
                let sect_label = section_label(&m.section);
                out.push_str(&format!(
                    "#heading(level: 2, \"{} -- {}\") <{}>\n",
                    m.section, m.title, sect_label
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

    // Read TOML
    let toml_content = fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", toml_path.display()));
    let table: Traceability = toml::from_str(&toml_content).expect("Invalid traceability.toml");

    // Read and parse manual
    let manual_root = root.join("omdurman-app/assets");
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
}

fn workspace_root() -> PathBuf {
    let this = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    this.parent().unwrap().parent().unwrap().to_path_buf()
}

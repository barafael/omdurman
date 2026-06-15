use std::collections::HashMap;
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
    /// Optional original printed page number (e.g. 5, "5-6").
    #[serde(default)]
    page: Option<String>,
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

        for line_idx in (heading.line_idx + 1)..next_line_idx {
            let l = lines[line_idx];
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
    if line.starts_with("**") {
        let rest = &line[2..];
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
    if s.is_empty() {
        return false;
    }
    s.chars().all(|c| c.is_ascii_digit() || c == '.')
        && !s.starts_with('.')
        && !s.ends_with('.')
}

// ---------------------------------------------------------------------------
// Code snippet extraction
// ---------------------------------------------------------------------------

fn extract_snippet(path: &Path, target_line: u32, context: usize) -> String {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }
    let idx = (target_line as usize).saturating_sub(1);
    let start = idx.saturating_sub(context);
    let end = std::cmp::min(idx + context + 1, lines.len());
    lines[start..end].join("\n")
}

fn file_extension(path: &str) -> &str {
    if let Some(pos) = path.rfind('.') {
        &path[pos + 1..]
    } else {
        ""
    }
}

// ---------------------------------------------------------------------------
// Typst content escaping helpers
// ---------------------------------------------------------------------------

/// Escape text for use inside a Typst content block `[...]`.
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
// Typst document generation
// ---------------------------------------------------------------------------

fn generate_preamble(root: &Path) -> String {
    let root_str = root.to_string_lossy().replace('\\', "/");
    format!(
        r#"#set page(paper: "a4", margin: (top: 2cm, bottom: 2cm, left: 2.5cm, right: 2cm))
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
"#,
    )
}

fn generate_typst(table: &Traceability, manual_sections: &HashMap<String, String>, root: &Path) -> String {
    let mut out = String::new();

    // Preamble
    out.push_str(&generate_preamble(root));

    // Title block
    out.push_str(
        "#align(center, text(size: 18pt, weight: \"bold\", \"Traceability Matrix\"))\n",
    );
    out.push_str("#align(center, text(size: 10pt, \"REMEMBER GORDON! -- Rulebook ⇌ Implementation Mapping\"))\n");
    out.push_str(
        "#align(center, text(size: 9pt, fill: luma(120), \"Generated from `docs/traceability.toml`\"))\n",
    );
    out.push_str("#v(2em)\n");

    // Group mappings by chapter
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

        // Chapter heading
        out.push_str(&format!("#heading(level: 1, \"{title}\")\n"));

        // Sort sub-sections
        let mut sorted = mappings.clone();
        sorted.sort_by(|a, b| sort_key(&a.section).cmp(&sort_key(&b.section)));

        for m in sorted {
            let section_num = m.section.trim_start_matches('§');
            let manual_text = manual_sections
                .get(section_num)
                .map(|s| s.as_str())
                .unwrap_or("");

            // Section heading
            out.push_str(&format!(
                "#heading(level: 2, \"{} -- {}\")\n",
                m.section,
                m.title
            ));

            // Status tag
            out.push_str(&format!("#status-tag(\"{}\")\n", m.status));

            // Page number
            if let Some(page) = &m.page {
                out.push_str(&format!("#linebreak()\n#text(size: 8.5pt, fill: luma(120))[manual page {}]\n", page));
            }
            out.push_str("#v(0.3em)\n");

            // Rule text from manual
            if !manual_text.is_empty() {
                let escaped = typst_content(manual_text);
                out.push_str(&format!(
                    "#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[{escaped}]]\n"
                ));
                out.push_str("#v(0.5em)\n");
            }

            // Implementation sites
            if !m.impls.is_empty() {
                out.push_str("#table(\n");
                out.push_str("  columns: (1fr, 3fr, 4.5fr),\n");
                out.push_str("  stroke: 0.4pt + luma(190),\n");
                out.push_str("  [*File*], [*Symbol*], [*Code Snippet*],\n");

                for imp in &m.impls {
                    let file_path = root.join(&imp.file);
                    let snippet = extract_snippet(&file_path, imp.line, 2);
                    let ext = file_extension(&imp.file);

                    // Only escape backslash and double-quote in the raw snippet
                    let snippet_esc = snippet
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"");

                    out.push_str(&format!(
                        "  [#vscode-link(\"{}\", {})], [#text[{}]],",
                        imp.file, imp.line, imp.symbol
                    ));

                    if snippet.is_empty() {
                        out.push_str(" [],\n");
                    } else {
                        out.push_str(&format!(
                            " [#raw(\"{snippet_esc}\", block: true, lang: \"{ext}\")],\n"
                        ));
                    }
                }

                out.push_str(")\n");
                out.push_str("#v(0.5em)\n");
            }
        }
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
    let toml_content =
        fs::read_to_string(&toml_path).unwrap_or_else(|e| panic!("Cannot read {}: {e}", toml_path.display()));
    let table: Traceability = toml::from_str(&toml_content).expect("Invalid traceability.toml");

    // Read and parse manual
    let manual_root = root.join("Boardgame - Remember_Gordon/Boardgame - Remember_Gordon/Manual");
    let manual_path = manual_root.join("RememberGordonManual.md");
    let manual_sections = parse_manual_sections(&manual_path);

    // Generate Typst
    let output = generate_typst(&table, &manual_sections, &root);

    // Write output
    if let Some(out_path) = output_path {
        fs::write(&out_path, &output).unwrap_or_else(|e| panic!("Cannot write {}: {e}", out_path.display()));
        eprintln!("Wrote {}", out_path.display());
    } else {
        println!("{output}");
    }
}

fn workspace_root() -> PathBuf {
    let this = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // tools/traceability-typst -> tools -> workspace root
    this.parent().unwrap().parent().unwrap().to_path_buf()
}

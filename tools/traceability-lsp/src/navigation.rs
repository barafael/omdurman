//! Navigation handlers: hover, definition, references, implementation, code lens.

use lsp_types::{
    request::GotoImplementationResponse, CodeLens, CodeLensParams, Command, GotoDefinitionResponse,
    Hover, HoverContents, HoverParams, Location, MarkupContent, MarkupKind, Position, ReferenceParams,
};

use crate::lsp_util::{full_line, path_to_uri, range, section_token_at, uri_to_path};
use traceability_lsp::{TraceIndex, traceability_path};

/// The requirement (section) referenced by the document position, if any.
fn section_at(index: &TraceIndex, path: &std::path::Path, pos: &Position) -> Option<String> {
    let text = index.file_texts.get(path)?;
    let line = pos.line as usize + 1;
    let line_str = text.lines().nth(line.saturating_sub(1))?;
    let byte_col = byte_offset_from_utf16(line_str, pos.character as usize);

    // 1. A `§N` token on the line (works in .rs, .toml, .md, prose).
    if let Some(section) = section_token_at(line_str, byte_col)
        && index.requirement(&section).is_some() {
            return Some(section);
        }

    // 2. A manual section anchor (header / bold lead).
    if path == index.manual_path()
        && let Some(ms) = index.manual_sections.iter().find(|s| {
            pos.line as usize + 1 >= s.start_line && (pos.line as usize) < s.end_line
        }) {
            let section = format!("§{}", ms.num);
            if index.requirement(&section).is_some() {
                return Some(section);
            }
        }

    None
}

/// Convert a UTF-16 column back to a byte offset within a line.
fn byte_offset_from_utf16(line: &str, col16: usize) -> usize {
    let mut count = 0usize;
    for (i, c) in line.char_indices() {
        if count >= col16 {
            return i;
        }
        count += c.len_utf16();
    }
    line.len()
}

pub fn hover(index: &TraceIndex, params: &HoverParams) -> Option<Hover> {
    let path = uri_to_path(&params.text_document_position_params.text_document.uri)?;
    let text = index.file_texts.get(&path)?;
    let pos = params.text_document_position_params.position;
    let line = pos.line as usize + 1;
    let line_str = text.lines().nth(line.saturating_sub(1))?;
    let byte_col = byte_offset_from_utf16(line_str, pos.character as usize);

    // On an impl site symbol: "implements §N".
    if let Some((section, symbol)) = index.impl_symbol_at(&path, line, byte_col) {
        if let Some(req) = index.requirement(&section) {
            let value = format!(
                "**implements {} — {}**\n\n{}",
                req.section,
                req.title,
                impl_summary(index, &section)
            );
            return Some(hover_of(&value));
        }
        let _ = symbol;
    }

    // On a test function: which sections it covers.
    if let Some(t) = index
        .test_entries
        .iter()
        .find(|t| t.file == path && t.line == line)
    {
        let sections: Vec<&str> = t.sections.iter().map(String::as_str).collect();
        let value = format!("**test `{}`** covers {}", t.name, sections.join(", "));
        return Some(hover_of(&value));
    }

    // On a `§N` reference (any file): requirement card.
    if let Some(section) = section_at(index, &path, &pos)
        && let Some(req) = index.requirement(&section) {
            let value = requirement_card(index, req);
            return Some(hover_of(&value));
        }

    // On a manual section header with no mapping: still show the header.
    if path == index.manual_path()
        && let Some(ms) = index.manual_sections.iter().find(|s| s.start_line == line) {
            let value = format!("**§{} — {}**\n\n(no mapping in traceability.toml)", ms.num, ms.title);
            return Some(hover_of(&value));
        }

    None
}

fn hover_of(value: &str) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: value.to_string(),
        }),
        range: None,
    }
}

fn requirement_card(index: &TraceIndex, req: &traceability_lsp::Requirement) -> String {
    let mut out = format!(
        "**{} — {}**  \nstatus: `{}`",
        req.section, req.title, req.status
    );
    if let Some(note) = &req.note {
        out.push_str(&format!("\n\n> {note}"));
    }
    out.push('\n');
    out.push_str(&impl_summary(index, &req.section));
    out
}

fn impl_summary(index: &TraceIndex, section: &str) -> String {
    let impls: Vec<_> = index.impls_for(section).collect();
    let tests: Vec<_> = index.tests_for(section).collect();
    let mut out = String::new();
    out.push_str(&format!(
        "\n\n{} impl site(s), {} annotated test(s)",
        impls.len(),
        tests.len()
    ));
    for imp in &impls {
        let rel = imp.file.strip_prefix(&index.root).unwrap_or(&imp.file);
        out.push_str(&format!("\n- `{}` at `{}:{}`", imp.symbol, rel.display(), imp.line));
    }
    for t in tests.iter().take(10) {
        let rel = t.file.strip_prefix(&index.root).unwrap_or(&t.file);
        out.push_str(&format!("\n- test `{}` ({})", t.name, rel.display()));
    }
    out
}

pub fn definition(index: &TraceIndex, params: &lsp_types::GotoDefinitionParams) -> Option<GotoDefinitionResponse> {
    let path = uri_to_path(&params.text_document_position_params.text_document.uri)?;
    let text = index.file_texts.get(&path)?;
    let pos = params.text_document_position_params.position;
    let line = pos.line as usize + 1;
    let line_str = text.lines().nth(line.saturating_sub(1))?;

    // `symbol = "..."` in the TOML -> goto the source impl location.
    if path.ends_with("traceability.toml")
        && let Some((section, symbol)) = symbol_in_toml_line(line_str) {
            let _ = section;
            if let Some(loc) = impl_location(index, &symbol) {
                return Some(GotoDefinitionResponse::Scalar(loc));
            }
        }

    let section = section_at(index, &path, &pos)?;
    definition_for_section(index, &section)
}

/// Where is a requirement *defined*? The manual anchor, or the TOML mapping as
/// a fallback (e.g. pseudo-sections like `§CRT` have no manual text).
pub fn definition_for_section(index: &TraceIndex, section: &str) -> Option<GotoDefinitionResponse> {
    if let Some(ms) = index.manual(section) {
        let text = index.file_texts.get(&index.manual_path())?;
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: path_to_uri(&index.manual_path()),
            range: full_line(text, ms.start_line),
        }));
    }
    toml_mapping_location(index, section).map(GotoDefinitionResponse::Scalar)
}

fn impl_location(index: &TraceIndex, symbol: &str) -> Option<Location> {
    index
        .resolved_impls
        .iter()
        .find(|r| r.symbol == symbol || r.symbol.ends_with(&format!("::{symbol}")))
        .and_then(|r| {
            let text = index.file_texts.get(&r.file)?;
            let key = r.symbol.rsplit("::").next().unwrap_or(&r.symbol);
            Some(Location {
                uri: path_to_uri(&r.file),
                range: range(text, r.line, r.byte_col, r.byte_col + key.len()),
            })
        })
}

pub fn references(index: &TraceIndex, params: &ReferenceParams) -> Vec<Location> {
    let Some(path) = uri_to_path(&params.text_document_position.text_document.uri) else {
        return Vec::new();
    };
    let Some(section) = section_at(index, &path, &params.text_document_position.position) else {
        return Vec::new();
    };
    references_for_section(index, &section)
}

pub fn references_for_section(index: &TraceIndex, section: &str) -> Vec<Location> {
    let mut out: Vec<Location> = Vec::new();

    for c in index.citations_for(section) {
        if let Some(text) = index.file_texts.get(&c.file) {
            out.push(Location {
                uri: path_to_uri(&c.file),
                range: range(text, c.line, c.byte_col, c.byte_col + section.len()),
            });
        }
    }
    for r in index.impls_for(section) {
        if let Some(text) = index.file_texts.get(&r.file) {
            let key = r.symbol.rsplit("::").next().unwrap_or(&r.symbol);
            out.push(Location {
                uri: path_to_uri(&r.file),
                range: range(text, r.line, r.byte_col, r.byte_col + key.len()),
            });
        }
    }
    for t in index.tests_for(section) {
        if let Some(text) = index.file_texts.get(&t.file) {
            out.push(Location {
                uri: path_to_uri(&t.file),
                range: full_line(text, t.line),
            });
        }
    }
    if let Some(loc) = toml_mapping_location(index, section) {
        out.push(loc);
    }

    out
}

pub fn implementation(
    index: &TraceIndex,
    params: &lsp_types::request::GotoImplementationParams,
) -> Option<GotoImplementationResponse> {
    let path = uri_to_path(&params.text_document_position_params.text_document.uri)?;
    let section = section_at(index, &path, &params.text_document_position_params.position)?;
    let locs: Vec<Location> = index
        .impls_for(&section)
        .filter_map(|r| {
            let text = index.file_texts.get(&r.file)?;
            let key = r.symbol.rsplit("::").next().unwrap_or(&r.symbol);
            Some(Location {
                uri: path_to_uri(&r.file),
                range: range(text, r.line, r.byte_col, r.byte_col + key.len()),
            })
        })
        .collect();
    match locs.len() {
        0 => None,
        1 => Some(GotoImplementationResponse::Scalar(locs.into_iter().next().unwrap())),
        _ => Some(GotoImplementationResponse::Array(locs)),
    }
}

pub fn code_lens(index: &TraceIndex, params: &CodeLensParams) -> Vec<CodeLens> {
    let Some(path) = uri_to_path(&params.text_document.uri) else {
        return Vec::new();
    };
    let Some(text) = index.file_texts.get(&path) else {
        return Vec::new();
    };
    let mut out: Vec<CodeLens> = Vec::new();

    // Impl-site symbols: "§6.53 ✓ implemented (2 tests)".
    for r in index.resolved_impls.iter().filter(|r| r.file == path) {
        if let Some(req) = index.requirement(&r.section) {
            let key = r.symbol.rsplit("::").next().unwrap_or(&r.symbol);
            let tests = index.tests_for(&r.section).count();
            let title = format!(
                "§{} — {} [{} · {} test{}]",
                req.section.trim_start_matches('§'),
                req.title,
                req.status,
                tests,
                if tests == 1 { "" } else { "s" },
            );
            out.push(CodeLens {
                range: range(text, r.line, r.byte_col, r.byte_col + key.len()),
                command: Some(Command {
                    title,
                    command: String::new(),
                    arguments: None,
                }),
                data: None,
            });
        }
    }

    // Annotated tests: "covers §10.11, §10.21".
    for t in index.test_entries.iter().filter(|t| t.file == path) {
        let sections: Vec<String> = t
            .sections
            .iter()
            .map(|s| s.trim_start_matches('§').to_string())
            .collect();
        let title = format!("covers {}", sections.join(", "));
        out.push(CodeLens {
            range: full_line(text, t.line),
            command: Some(Command {
                title,
                command: String::new(),
                arguments: None,
            }),
            data: None,
        });
    }

    out
}

/// The `section = "§N"` / `symbol = "x"` pair on a TOML line, if present.
fn symbol_in_toml_line(line: &str) -> Option<(String, String)> {
    let section = if let Some(start) = line.find("section = ") {
        let rest = &line[start + "section = ".len()..];
        let s = rest.trim().trim_start_matches('"').split('"').next().unwrap_or("");
        if s.starts_with('§') {
            Some(s.to_string())
        } else {
            None
        }
    } else {
        None
    };
    let symbol = if let Some(start) = line.find("symbol = ") {
        let rest = &line[start + "symbol = ".len()..];
        Some(rest.trim().trim_start_matches('"').split('"').next().unwrap_or("").to_string())
    } else {
        None
    };
    match (section, symbol) {
        (Some(s), Some(sym)) if !sym.is_empty() => Some((s, sym)),
        _ => None,
    }
}

/// Locate the `section = "§N"` line of a mapping in the TOML.
pub fn toml_mapping_location(index: &TraceIndex, section: &str) -> Option<Location> {
    let toml_path = traceability_path();
    let text = index.file_texts.get(&toml_path)?;
    let needle = format!("section = \"{section}\"");
    let line_idx = text.lines().position(|l| l.contains(&needle))?;
    Some(Location {
        uri: path_to_uri(&toml_path),
        range: full_line(text, line_idx + 1),
    })
}

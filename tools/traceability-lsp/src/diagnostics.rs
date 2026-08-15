//! Per-file diagnostics, derived from the same `checks` that `cargo test` runs
//! so the editor and the test suite always agree. Semantic-gap issues are the
//! only warnings: the test contract treats them as informational.

use lsp_types::{Diagnostic, DiagnosticSeverity};

use crate::lsp_util::{full_line, range};
use traceability_lsp::checks::{check_coverage, check_matrix, check_semantic_gap};
use traceability_lsp::{TraceIndex, manual_path, traceability_path};

/// One `[[mapping]]` block located in the TOML text.
#[derive(Debug, Clone)]
struct TomlBlock {
    section: String,
    section_line: usize,
    impls: Vec<ImplLine>,
}

/// One `[[mapping.impl]]` entry located in the TOML text.
#[derive(Debug, Clone)]
struct ImplLine {
    declared: u32,
    file_line: usize,
    symbol_line: usize,
    file: String,
    symbol: String,
}

/// Locate every `[[mapping]]` block in the TOML text with its inner lines.
fn toml_blocks(text: &str) -> Vec<TomlBlock> {
    let mut blocks: Vec<TomlBlock> = Vec::new();
    let mut cur: Option<TomlBlock> = None;
    // (file, declared_line, symbol) for the current [[mapping.impl]].
    let mut imp: Option<ImplLine> = None;

    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        let trimmed = line.trim();

        if trimmed == "[[mapping]]" {
            if let Some(b) = cur.take() {
                blocks.push(b);
            }
            cur = Some(TomlBlock {
                section: String::new(),
                section_line: line_no,
                impls: Vec::new(),
            });
            continue;
        }
        let Some(b) = cur.as_mut() else { continue };

        if trimmed == "[[mapping.impl]]" {
            imp = Some(ImplLine {
                declared: 0,
                file_line: line_no,
                symbol_line: line_no,
                file: String::new(),
                symbol: String::new(),
            });
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("section = ") {
            b.section = quoted(rest);
            b.section_line = line_no;
            continue;
        }

        if let Some(cur_imp) = imp.as_mut() {
            if let Some(rest) = trimmed.strip_prefix("file = ") {
                cur_imp.file = quoted(rest);
                cur_imp.file_line = line_no;
            } else if let Some(rest) = trimmed.strip_prefix("line = ") {
                cur_imp.declared = rest.trim().parse().unwrap_or(0);
            } else if let Some(rest) = trimmed.strip_prefix("symbol = ") {
                cur_imp.symbol = quoted(rest);
                cur_imp.symbol_line = line_no;
                b.impls.push(cur_imp.clone());
                imp = None;
            }
            continue;
        }
    }
    if let Some(b) = cur.take() {
        blocks.push(b);
    }
    blocks
}

fn quoted(s: &str) -> String {
    s.trim()
        .trim_start_matches('"')
        .split('"')
        .next()
        .unwrap_or("")
        .to_string()
}

/// The classified failures the checks can produce.
enum Failure {
    NoImpls(String),
    ImplsOnNonImplemented(String, String),
    UnknownStatus(String, String),
    ManualMissing(String),
    ImplFileMissing(String),
    ImplDrifted { file: String, line: u32, symbol: String },
    ImplMissing { file: String, line: u32, symbol: String },
    NotAnchored(String),
    OrphanCitation { src_path: String, section: String },
    TestsUnknown { section: String, test: String },
    TestsMismatch { section: String, test: String },
    TestNotListed { test: String, section: String },
}

fn parse_failure(s: &str) -> Option<Failure> {
    if let Some(rest) = s.strip_prefix("impl file does not exist: ") {
        return Some(Failure::ImplFileMissing(rest.to_string()));
    }

    // `{file}:{line}: '{symbol}' (key '...') -- {why} (line ...)`
    if let Some((head, tail)) = s.split_once(" -- ")
        && let Some((symbol_head, _)) = head.split_once(" (key '")
            && let Some((file_line, sym)) = symbol_head.split_once(": '") {
                let symbol = sym.split('\'').next().unwrap_or("").to_string();
                if let Some((file, line)) = file_line.rsplit_once(':')
                    && let Ok(line) = line.trim().parse::<u32>() {
                        if tail.starts_with("line has drifted") {
                            return Some(Failure::ImplDrifted {
                                file: file.to_string(),
                                line,
                                symbol,
                            });
                        }
                        if tail.starts_with("symbol not found") {
                            return Some(Failure::ImplMissing {
                                file: file.to_string(),
                                line,
                                symbol,
                            });
                        }
                    }
            }

    // `{src_path} cites {section} which has no [[mapping]] entry`
    if let Some((src_path, rest)) = s.split_once(" cites ")
        && let Some(section) = rest.strip_suffix(" which has no [[mapping]] entry") {
            return Some(Failure::OrphanCitation {
                src_path: src_path.to_string(),
                section: section.to_string(),
            });
        }

    // `test '{test}' has annotation {section} but is not listed ...`
    if let Some(rest) = s.strip_prefix("test '")
        && let Some((test, rest)) = rest.split_once('\'')
            && let Some(section) = rest.strip_prefix(" has annotation ")
                && let Some(section) = section.split(" but is not listed").next() {
                    return Some(Failure::TestNotListed {
                        test: test.to_string(),
                        section: section.to_string(),
                    });
                }

    // `{section}: tests array lists '{test}' but ...`
    if let Some((section, rest)) = s.split_once(": tests array lists '") {
        let test = rest.split('\'').next().unwrap_or("").to_string();
        if s.contains(" but no such #[test] fn found") {
            return Some(Failure::TestsUnknown {
                section: section.to_string(),
                test,
            });
        }
        if s.contains(" but that test's annotations are ") {
            return Some(Failure::TestsMismatch {
                section: section.to_string(),
                test,
            });
        }
    }

    // `symbol '{symbol}' (key '...') is not anchored ...`
    if let Some(rest) = s.strip_prefix("symbol '")
        && let Some(symbol) = rest.split('\'').next()
            && s.contains(" is not anchored in traceability_paths.rs") {
                return Some(Failure::NotAnchored(symbol.to_string()));
            }

    // Section-scoped status failures: `{section} "{title}" ...`
    let section = s.split_whitespace().next().unwrap_or("").to_string();
    if !section.is_empty() && section.starts_with('§') {
        if s.contains(" is 'implemented' but has no [[impl]] entries") {
            return Some(Failure::NoImpls(section));
        }
        if s.contains(" but has [[impl]] entries (should have none)")
            && let Some(start) = s.find(" is '") {
                let status = s[start + 4..].split('\'').next().unwrap_or("").to_string();
                return Some(Failure::ImplsOnNonImplemented(section, status));
            }
        if let Some(start) = s.find(" has unknown status '") {
            let status = s[start + " has unknown status '".len()..]
                .split('\'')
                .next()
                .unwrap_or("")
                .to_string();
            return Some(Failure::UnknownStatus(section, status));
        }
        if s.contains(" not found in OCR manual ") {
            return Some(Failure::ManualMissing(section));
        }
    }

    None
}

fn block_for<'a>(blocks: &'a [TomlBlock], section: &str) -> Option<&'a TomlBlock> {
    blocks.iter().find(|b| b.section == section)
}

fn find_impl<'a>(blocks: &'a [TomlBlock], file: &str, line: u32, symbol: &str) -> Option<&'a ImplLine> {
    blocks
        .iter()
        .flat_map(|b| b.impls.iter())
        .find(|i| i.file == file && i.declared == line && i.symbol == symbol)
}

fn find_impl_by_symbol<'a>(blocks: &'a [TomlBlock], symbol: &str) -> Option<&'a ImplLine> {
    blocks.iter().flat_map(|b| b.impls.iter()).find(|i| i.symbol == symbol)
}

fn find_impl_by_file<'a>(blocks: &'a [TomlBlock], file: &str) -> Option<&'a ImplLine> {
    blocks.iter().flat_map(|b| b.impls.iter()).find(|i| i.file == file)
}

/// Diagnostics for one document (absolute path). Non-relevant files yield an
/// empty list.
pub fn diagnostics_for(index: &TraceIndex, path: &std::path::Path) -> Vec<Diagnostic> {
    if path == traceability_path().as_path() {
        return toml_diagnostics(index);
    }
    if path == manual_path().as_path() {
        return Vec::new();
    }
    if path.extension().is_some_and(|e| e == "rs") {
        return rs_diagnostics(index, path);
    }
    Vec::new()
}

fn rs_diagnostics(index: &TraceIndex, path: &std::path::Path) -> Vec<Diagnostic> {
    let mut out: Vec<Diagnostic> = Vec::new();
    let Some(text) = index.file_texts.get(path) else {
        return out;
    };
    let rel = path
        .strip_prefix(&index.root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
    let root = &index.root;

    let matrix = check_matrix(root);
    let coverage = check_coverage(root);
    let gaps = check_semantic_gap(root);

    for f in &matrix.failures {
        match parse_failure(f) {
            Some(Failure::ImplDrifted { file, line, .. })
            | Some(Failure::ImplMissing { file, line, .. })
                if file == rel =>
            {
                out.push(Diagnostic::new(
                    full_line(text, line as usize),
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    Some("traceability".into()),
                    "impl anchor in docs/traceability.toml does not match this location".to_string(),
                    None,
                    None,
                ));
            }
            Some(Failure::OrphanCitation { src_path, section }) if src_path == rel => {
                for c in index.citations_for(&section).filter(|c| {
                    c.file
                        .strip_prefix(root)
                        .unwrap_or(&c.file)
                        .to_string_lossy()
                        == rel
                }) {
                    out.push(Diagnostic::new(
                        range(text, c.line, c.byte_col, c.byte_col + section.len()),
                        Some(DiagnosticSeverity::ERROR),
                        None,
                        Some("traceability".into()),
                        format!("{section} has no [[mapping]] entry in docs/traceability.toml"),
                        None,
                        None,
                    ));
                }
            }
            _ => {}
        }
    }

    for f in &coverage.failures {
        if let Some(Failure::TestNotListed { test, section }) = parse_failure(f)
            && let Some(t) = index
                .test_entries
                .iter()
                .find(|t| t.name == test && t.file == path)
            {
                out.push(Diagnostic::new(
                    full_line(text, t.line),
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    Some("traceability".into()),
                    format!("annotation {section} is not listed in the tests array of [[mapping]] section = \"{section}\""),
                    None,
                    None,
                ));
            }
    }

    // Warning-only: implemented mappings with no annotated tests.
    for gap in &gaps {
        for r in index.impls_for(&gap.section).filter(|r| r.file == path) {
            let key = r.symbol.rsplit("::").next().unwrap_or(&r.symbol);
            out.push(Diagnostic::new(
                range(text, r.line, r.byte_col, r.byte_col + key.len()),
                Some(DiagnosticSeverity::WARNING),
                None,
                Some("traceability".into()),
                format!(
                    "{} \"{}\" is implemented but has no annotated test — add a #[rulebook] test",
                    gap.section, gap.title
                ),
                None,
                None,
            ));
        }
    }

    out
}

fn toml_diagnostics(index: &TraceIndex) -> Vec<Diagnostic> {
    let mut out: Vec<Diagnostic> = Vec::new();
    let Some(text) = index.file_texts.get(&traceability_path()) else {
        return out;
    };
    let root = &index.root;
    let blocks = toml_blocks(text);

    let matrix = check_matrix(root);
    let coverage = check_coverage(root);
    let gaps = check_semantic_gap(root);

    for f in &matrix.failures {
        match parse_failure(f) {
            Some(Failure::NoImpls(section)) => {
                if let Some(b) = block_for(&blocks, &section) {
                    out.push(err(text, b.section_line, format!("{section} is 'implemented' but has no [[impl]] entries")));
                }
            }
            Some(Failure::ImplsOnNonImplemented(section, status)) => {
                if let Some(b) = block_for(&blocks, &section) {
                    out.push(err(text, b.section_line, format!("{section} is '{status}' but has [[impl]] entries (should have none)")));
                }
            }
            Some(Failure::UnknownStatus(section, status)) => {
                if let Some(b) = block_for(&blocks, &section) {
                    out.push(err(text, b.section_line, format!("{section} has unknown status '{status}'")));
                }
            }
            Some(Failure::ManualMissing(section)) => {
                if let Some(b) = block_for(&blocks, &section) {
                    out.push(err(text, b.section_line, format!("{section} not found in OCR manual")));
                }
            }
            Some(Failure::ImplFileMissing(file)) => {
                if let Some(imp) = find_impl_by_file(&blocks, &file) {
                    out.push(err(text, imp.file_line, format!("impl file does not exist: {file}")));
                }
            }
            Some(Failure::ImplDrifted { file, line, symbol }) => {
                if let Some(imp) = find_impl(&blocks, &file, line, &symbol) {
                    out.push(err(text, imp.symbol_line, format!("'{symbol}' — line has drifted (declared line {line})")));
                }
            }
            Some(Failure::ImplMissing { file, line, symbol }) => {
                if let Some(imp) = find_impl(&blocks, &file, line, &symbol) {
                    out.push(err(text, imp.symbol_line, format!("'{symbol}' — symbol not found in file (declared line {line})")));
                }
            }
            Some(Failure::NotAnchored(symbol)) => {
                if let Some(imp) = find_impl_by_symbol(&blocks, &symbol) {
                    out.push(err(text, imp.symbol_line, format!("'{symbol}' is not anchored in traceability_paths.rs — add a compiler-checked reference there")));
                }
            }
            _ => {}
        }
    }

    for f in &coverage.failures {
        match parse_failure(f) {
            Some(Failure::TestsUnknown { section, test }) => {
                if let Some(b) = block_for(&blocks, &section) {
                    out.push(err(text, b.section_line, format!("tests array lists '{test}' but no such #[test] fn found in source")));
                }
            }
            Some(Failure::TestsMismatch { section, test }) => {
                if let Some(b) = block_for(&blocks, &section) {
                    out.push(err(text, b.section_line, format!("tests array lists '{test}' but that test's annotations do not include {section}")));
                }
            }
            _ => {}
        }
    }

    // Warning-only semantic gap.
    for gap in &gaps {
        if let Some(b) = block_for(&blocks, &gap.section) {
            out.push(Diagnostic::new(
                full_line(text, b.section_line),
                Some(DiagnosticSeverity::WARNING),
                None,
                Some("traceability".into()),
                format!(
                    "{} \"{}\" is implemented but has no annotated test — add a #[rulebook] test",
                    gap.section, gap.title
                ),
                None,
                None,
            ));
        }
    }

    out
}

fn err(text: &str, line: usize, message: String) -> Diagnostic {
    Diagnostic::new(
        full_line(text, line),
        Some(DiagnosticSeverity::ERROR),
        None,
        Some("traceability".into()),
        message,
        None,
        None,
    )
}

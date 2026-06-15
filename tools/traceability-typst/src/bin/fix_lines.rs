//! One-off maintenance tool: rewrite the `line = N` field of every
//! `[[mapping.impl]]` entry in `docs/traceability.toml` so it points at the
//! line where the cited `symbol` is actually defined.
//!
//! The traceability test only uses `line` for navigational accuracy (existence
//! is proven by the `traceability_paths.rs` compile-check), but a correct line
//! makes the matrix clickable. Run from the workspace root:
//!
//!     cargo run -p traceability-typst --bin fix_lines
//!
//! Idempotent: re-running after a code edit re-syncs the line numbers.

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Score how strongly a source line *defines* `key` (higher = more likely the
/// definition, as opposed to a use site). Returns `None` if `key` is absent.
fn definition_score(line: &str, key: &str) -> Option<i32> {
    if !line.contains(key) {
        return None;
    }
    let t = line.trim_start();
    let mut score = 0;
    // Definition keywords immediately preceding the key.
    for (kw, pts) in [
        ("pub fn ", 100),
        ("fn ", 90),
        ("pub struct ", 100),
        ("struct ", 90),
        ("pub enum ", 100),
        ("enum ", 90),
        ("pub const ", 100),
        ("const ", 90),
        ("pub type ", 100),
        ("type ", 80),
        ("pub ", 40), // field: `pub name: T`
    ] {
        if t.starts_with(&format!("{kw}{key}")) {
            score += pts;
        }
    }
    // A bare enum variant / field line: `Key,` or `Key {` or `Key(`.
    if t.starts_with(key) && t[key.len()..].starts_with([',', ' ', '(', '{', ':']) {
        score += 60;
    }
    // Penalise obvious doc comments / string mentions.
    if t.starts_with("//") || t.starts_with("///") {
        score -= 50;
    }
    Some(score)
}

/// Find the 1-based line in `source` that best defines `symbol`'s last segment,
/// preferring the match nearest the currently-cited `hint` line on ties.
fn best_line(source: &str, symbol: &str, hint: usize) -> Option<usize> {
    let key = symbol.rsplit("::").next().unwrap_or(symbol);
    source
        .lines()
        .enumerate()
        .filter_map(|(i, l)| definition_score(l, key).map(|s| (i + 1, s)))
        .max_by_key(|&(lineno, score)| {
            // Maximise score, then minimise distance from the hint.
            let dist = (lineno as i64 - hint as i64).abs();
            (score, -dist)
        })
        .map(|(lineno, _)| lineno)
}

fn main() {
    let root = workspace_root();
    let toml_path = root.join("docs/traceability.toml");
    let text = fs::read_to_string(&toml_path).expect("read traceability.toml");
    let lines: Vec<&str> = text.lines().collect();

    // A small per-block state machine: within an `[[mapping.impl]]` block, find
    // the `file`, the `line` (to rewrite), and the `symbol`.
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut cur_file: Option<String> = None;
    let mut cur_line_out_idx: Option<usize> = None;
    let mut cur_hint: usize = 0;
    let mut fixed = 0usize;

    for &line in &lines {
        let trimmed = line.trim();
        if trimmed == "[[mapping.impl]]" {
            cur_file = None;
            cur_line_out_idx = None;
            cur_hint = 0;
            out.push(line.to_string());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("file = ") {
            cur_file = Some(rest.trim_matches('"').to_string());
            out.push(line.to_string());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("line = ") {
            cur_hint = rest.trim().parse().unwrap_or(0);
            cur_line_out_idx = Some(out.len());
            out.push(line.to_string());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("symbol = ") {
            let symbol = rest.trim_matches('"').to_string();
            out.push(line.to_string());
            if let (Some(file), Some(line_idx)) = (&cur_file, cur_line_out_idx) {
                let src_path = root.join(file);
                if let Ok(src) = fs::read_to_string(&src_path)
                    && let Some(n) = best_line(&src, &symbol, cur_hint)
                {
                    let new = format!("line = {n}");
                    if out[line_idx] != new {
                        out[line_idx] = new;
                        fixed += 1;
                    }
                }
            }
            continue;
        }
        out.push(line.to_string());
    }

    let mut joined = out.join("\n");
    if text.ends_with('\n') {
        joined.push('\n');
    }
    fs::write(&toml_path, joined).expect("write traceability.toml");
    println!("fix_lines: updated {fixed} line numbers in {toml_path:?}");
}

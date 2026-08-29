//! Conversions between the index's byte/1-based positions and LSP's
//! 0-based / UTF-16 positions.

use lsp_types::{Position, Range, Uri};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Percent-encode a filesystem path into a `file://` URI.
pub fn path_to_uri(path: &Path) -> Uri {
    let s = path.to_string_lossy();
    let encoded = encode_uri_path(&s);
    let uri_str = if encoded.starts_with('/') {
        format!("file://{encoded}")
    } else {
        format!("file:///{encoded}")
    };
    Uri::from_str(&uri_str).expect("valid file URI")
}

/// Decode a `file://` URI back to a filesystem path.
pub fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    let s = uri.as_str();
    let rest = s.strip_prefix("file://")?;
    // A `file://` URI may carry a host (`file://localhost/path`) or none
    // (`file:///path`); both resolve to a local absolute path, so the host is
    // dropped either way.
    let rest = if let Some((_host, path)) = rest.split_once('/') {
        format!("/{path}")
    } else {
        String::new()
    };
    if rest.is_empty() {
        return None;
    }
    Some(PathBuf::from(decode_percent(&rest)))
}

fn encode_uri_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                out.push(*b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn decode_percent(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2]))
        {
            out.push((h << 4 | l) as char);
            i += 3;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Convert a byte column within a line to a UTF-16 code-unit column.
pub fn utf16_col(line: &str, byte_col: usize) -> u32 {
    let clamped = byte_col.min(line.len());
    line[..clamped].encode_utf16().count() as u32
}

/// Range spanning a byte span on a line (end exclusive in bytes).
pub fn range(text: &str, line_1based: usize, byte_start: usize, byte_end: usize) -> Range {
    let line_str = text
        .lines()
        .nth(line_1based.saturating_sub(1))
        .unwrap_or_default();
    let start = utf16_col(line_str, byte_start);
    let end = utf16_col(line_str, byte_end.max(byte_start));
    Range::new(
        Position::new(line_1based.saturating_sub(1) as u32, start),
        Position::new(line_1based.saturating_sub(1) as u32, end),
    )
}

/// Range covering an entire line.
pub fn full_line(text: &str, line_1based: usize) -> Range {
    let line_str = text
        .lines()
        .nth(line_1based.saturating_sub(1))
        .unwrap_or_default();
    Range::new(
        Position::new(line_1based.saturating_sub(1) as u32, 0),
        Position::new(
            line_1based.saturating_sub(1) as u32,
            line_str.encode_utf16().count() as u32,
        ),
    )
}

/// Extract a `§N.M` token spanning byte `col` on a line, if any.
pub fn section_token_at(line: &str, col: usize) -> Option<String> {
    let mut search = 0;
    while let Some(pos) = line[search..].find('§') {
        let abs = search + pos;
        let after = &line[abs + '§'.len_utf8()..];
        let num: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '.')
            .collect();
        let clean = num.trim_end_matches('x').trim_end_matches('.');
        let end_byte = abs + '§'.len_utf8() + num.len();
        if !clean.is_empty() && col >= abs && col <= end_byte {
            return Some(format!("§{clean}"));
        }
        search = end_byte;
    }
    None
}

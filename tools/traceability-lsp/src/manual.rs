//! Index the OCR rulebook markdown: section anchors (headers and bold
//! paragraph leads) with line ranges, for "go to definition" of a `§N`.

use std::fs;
use std::path::Path;

/// A located section in the manual. `start_line` is the anchor line (1-based);
/// `end_line` is inclusive and points at the last line before the next anchor.
#[derive(Debug, Clone)]
pub struct ManualSection {
    /// Section number, e.g. `"6.53"`.
    pub num: String,
    /// Heading or paragraph lead title.
    pub title: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Extract the numeric section number from a manual heading or bold lead,
/// e.g. `### 9.1) The Campaign Game` -> `9.1`, `**9.111)** ...` -> `9.111`.
fn section_number(heading: &str) -> Option<String> {
    let mut s = heading.trim();
    s = s.trim_start_matches('#').trim_start();
    s = s.strip_prefix("**").unwrap_or(s);
    let num = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .trim_end_matches('.')
        .to_string();
    if num.is_empty() || !num.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(num)
}

/// The human-readable title after the section number, e.g.
/// `### 9.1) The Campaign Game` -> `The Campaign Game`.
fn section_title(line: &str) -> String {
    line.trim_start_matches('#')
        .trim_start()
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
        .trim_start_matches(|c: char| c == ')' || c == '*' || c == ' ')
        .trim_start()
        .to_string()
}

/// Build the section index for a manual file.
///
/// Recognises both `#`-header sections (`### 9.1) The Campaign Game`) and
/// inline bold paragraph leads (`**9.111)** Dervish player sets up first...`),
/// which the OCR manual uses for sub-sub-sections. Plain-numbered lines (e.g.
/// table-of-contents entries) are not anchors.
pub fn index_manual(path: &Path) -> Vec<ManualSection> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let lines: Vec<&str> = content.lines().collect();

    // (start_line, num, title) for every anchor.
    let mut anchors: Vec<(usize, String, String)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_header = trimmed.starts_with('#') && trimmed.starts_with("##");
        if is_header {
            if let Some(num) = section_number(trimmed) {
                anchors.push((i + 1, num, section_title(trimmed)));
                continue;
            }
        }
        if trimmed.starts_with("**") {
            if let Some(num) = section_number(trimmed) {
                anchors.push((i + 1, num, section_title(trimmed)));
            }
        }
    }

    let mut out = Vec::with_capacity(anchors.len());
    for (idx, (start_line, num, title)) in anchors.iter().enumerate() {
        let end_line = anchors
            .get(idx + 1)
            .map(|(s, _, _)| s.saturating_sub(1))
            .unwrap_or(lines.len());
        out.push(ManualSection {
            num: num.clone(),
            title: title.clone(),
            start_line: *start_line,
            end_line,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_header_and_bold_leads() {
        let dir = std::env::temp_dir().join("traceability-lsp-manual-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("manual.md");
        let body = concat!(
            "# Title\n",
            "\n",
            "## 9) The Scenarios\n",
            "\n",
            "### 9.1) The Campaign Game\n",
            "\n",
            "**9.111)** Dervish player sets up first.\n",
            "\n",
            "**9.112)** Dervish reinforcements.\n",
        );
        std::fs::write(&path, body).unwrap();

        let sections = index_manual(&path);
        let nums: Vec<&str> = sections.iter().map(|s| s.num.as_str()).collect();
        assert_eq!(nums, vec!["9", "9.1", "9.111", "9.112"]);
        assert_eq!(sections[2].start_line, 7);
        assert_eq!(sections[2].end_line, 8);
        std::fs::remove_dir_all(&dir).ok();
    }
}

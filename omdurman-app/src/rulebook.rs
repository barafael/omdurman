//! The rulebook: `RememberGordonManual.md` parsed into a §-keyed section tree
//! and rendered inside the chart sheet's Rulebook tab. Searchable, with a
//! collapsible section index and §-deep-link support (scroll + brief spotlight).
//!
//! The manual is the single source of truth (shipped in `assets/`); its own `N)`
//! / `N.M)` headings are the section anchors -- there is no runtime dependency on
//! `traceability.toml`.

use bevy::prelude::*;
use bevy_egui::egui;

/// The manual, embedded at build time so it ships with the binary and the wasm
/// bundle without a separate fetch.
const MANUAL_MD: &str = include_str!("../assets/RememberGordonManual.md");

/// One parsed section: its § number (e.g. "5" or "5.4"), heading title, depth
/// (1 = `## N)`, 2 = `### N.M)`), and body lines (until the next heading).
#[derive(Clone)]
pub struct Section {
    pub number: String,
    pub title: String,
    pub depth: u8,
    pub body: String,
}

/// The parsed manual, plus the rulebook tab's own view state (search + a pending
/// scroll-to-section deep link).
#[derive(Resource)]
pub struct Rulebook {
    pub sections: Vec<Section>,
    pub search: String,
    /// A section number to scroll to (and briefly spotlight) next frame.
    pub scroll_to: Option<String>,
    /// The section currently spotlighted, with seconds of highlight left.
    pub flash: Option<(String, f32)>,
}

impl Default for Rulebook {
    fn default() -> Self {
        Self {
            sections: parse_manual(MANUAL_MD),
            search: String::new(),
            scroll_to: None,
            flash: None,
        }
    }
}

/// Parse `## N) Title` / `### N.M) Title` headings into a flat section list,
/// each carrying the body text up to the next heading. The table-of-contents
/// block (before the first numbered `##` heading) is skipped.
fn parse_manual(md: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current: Option<Section> = None;

    for line in md.lines() {
        if let Some((depth, number, title)) = parse_heading(line) {
            if let Some(sec) = current.take() {
                sections.push(sec);
            }
            current = Some(Section {
                number,
                title,
                depth,
                body: String::new(),
            });
        } else if let Some(sec) = current.as_mut() {
            sec.body.push_str(line);
            sec.body.push('\n');
        }
        // Lines before the first numbered heading (title, ToC) are dropped.
    }
    if let Some(sec) = current.take() {
        sections.push(sec);
    }
    sections
}

/// Parse a heading line into `(depth, number, title)` if it is a numbered
/// rulebook heading: `## N) Title` (depth 1) or `### N.M) Title` (depth 2).
/// Returns `None` for the document title (`# ...`) and non-numbered headings.
fn parse_heading(line: &str) -> Option<(u8, String, String)> {
    let (hashes, rest) = if let Some(r) = line.strip_prefix("### ") {
        (2u8, r)
    } else if let Some(r) = line.strip_prefix("## ") {
        (1u8, r)
    } else {
        return None;
    };
    // Expect "<number>) <title>", where number is digits and dots.
    let (number, title) = rest.split_once(')')?;
    let number = number.trim();
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    Some((hashes, number.to_string(), title.trim().to_string()))
}

/// Request the rulebook scroll to (and briefly spotlight) a section number.
/// Called by the chart-sheet deep-link path when a `§` reference is followed.
pub fn request_section(rulebook: &mut Rulebook, number: &str) {
    rulebook.scroll_to = Some(number.to_string());
    rulebook.flash = Some((number.to_string(), 2.0));
}

/// Render the rulebook tab: a left section index + search, and the scrollable
/// body on the right. Returns a section number if the user clicked a `[§N]`
/// cross-reference link, so the caller can re-target.
pub fn draw_rulebook(ui: &mut egui::Ui, rulebook: &mut Rulebook, dt: f32) -> Option<String> {
    let mut clicked_ref: Option<String> = None;

    // Decay the flash spotlight.
    if let Some((_, ref mut secs)) = rulebook.flash {
        *secs -= dt;
        if *secs <= 0.0 {
            rulebook.flash = None;
        }
    }

    egui::SidePanel::left("rulebook_index")
        .default_width(190.0)
        .show_inside(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut rulebook.search)
                    .hint_text("search…")
                    .desired_width(f32::INFINITY),
            );
            ui.separator();
            let needle = rulebook.search.to_lowercase();
            egui::ScrollArea::vertical()
                .id_salt("rulebook_toc")
                .show(ui, |ui| {
                    for sec in &rulebook.sections {
                        // When searching, only show sections that match by
                        // number, title, or body.
                        if !needle.is_empty()
                            && !sec.number.contains(&needle)
                            && !sec.title.to_lowercase().contains(&needle)
                            && !sec.body.to_lowercase().contains(&needle)
                        {
                            continue;
                        }
                        let indent = if sec.depth >= 2 { "   " } else { "" };
                        let label = format!("{indent}{} {}", sec.number, sec.title);
                        if ui.link(label).clicked() {
                            rulebook.scroll_to = Some(sec.number.clone());
                            rulebook.flash = Some((sec.number.clone(), 2.0));
                        }
                    }
                });
        });

    let scroll_to = rulebook.scroll_to.take();
    let flash = rulebook.flash.clone();
    let needle = rulebook.search.to_lowercase();

    egui::ScrollArea::vertical()
        .id_salt("rulebook_body")
        .auto_shrink(false)
        .show(ui, |ui| {
            for sec in &rulebook.sections {
                if !needle.is_empty()
                    && !sec.number.contains(&needle)
                    && !sec.title.to_lowercase().contains(&needle)
                    && !sec.body.to_lowercase().contains(&needle)
                {
                    continue;
                }

                let heading = egui::RichText::new(format!("{}  {}", sec.number, sec.title))
                    .size(if sec.depth >= 2 { 15.0 } else { 18.0 })
                    .strong();
                let resp = ui.label(heading);

                // Deep-link / index scroll target: scroll this heading into view
                // and, if it is the flashed section, tint its background.
                if scroll_to.as_deref() == Some(sec.number.as_str()) {
                    resp.scroll_to_me(Some(egui::Align::TOP));
                }
                if let Some((ref n, secs)) = flash
                    && n == &sec.number
                {
                    let a = (secs.clamp(0.0, 1.0) * 90.0) as u8;
                    ui.painter().rect_filled(
                        resp.rect.expand2(egui::vec2(4.0, 2.0)),
                        2.0,
                        egui::Color32::from_rgba_unmultiplied(0x8f, 0xc5, 0xd7, a),
                    );
                }

                if let Some(r) = render_body(ui, &sec.body) {
                    clicked_ref = Some(r);
                }
                ui.add_space(10.0);
            }
        });

    // Keep repainting while a flash is animating.
    if rulebook.flash.is_some() {
        ui.ctx().request_repaint();
    }
    clicked_ref
}

/// Render a section's body text, turning inline `§N` / `§N.M` references into
/// clickable links. Returns a section number if one was clicked.
fn render_body(ui: &mut egui::Ui, body: &str) -> Option<String> {
    let mut clicked: Option<String> = None;
    for para in body.split("\n\n") {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for tok in split_refs(para) {
                match tok {
                    RefTok::Text(t) => {
                        ui.label(t);
                    }
                    RefTok::Ref(number) => {
                        if ui.link(format!("§{number}")).clicked() {
                            clicked = Some(number.to_string());
                        }
                    }
                }
            }
        });
    }
    clicked
}

enum RefTok<'a> {
    Text(&'a str),
    Ref(&'a str),
}

/// Split text on `§N` / `§N.M` references, yielding interleaved text and refs.
fn split_refs(text: &str) -> Vec<RefTok<'_>> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(pos) = rest.find('§') {
        if pos > 0 {
            out.push(RefTok::Text(&rest[..pos]));
        }
        let after = &rest[pos + '§'.len_utf8()..];
        let num_len = after
            .char_indices()
            .take_while(|(_, c)| c.is_ascii_digit() || *c == '.')
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        if num_len == 0 {
            // Lone § with no number: keep as text so it is not lost.
            out.push(RefTok::Text(&rest[pos..pos + '§'.len_utf8()]));
            rest = after;
        } else {
            out.push(RefTok::Ref(&after[..num_len]));
            rest = &after[num_len..];
        }
    }
    if !rest.is_empty() {
        out.push(RefTok::Text(rest));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numbered_headings() {
        let secs = parse_manual(MANUAL_MD);
        assert!(!secs.is_empty(), "manual should parse into sections");
        // A well-known section exists.
        assert!(
            secs.iter().any(|s| s.number == "5"),
            "movement section 5 present"
        );
        assert!(
            secs.iter().any(|s| s.number.contains('.')),
            "subsections present"
        );
    }

    #[test]
    fn heading_parse_rejects_non_numbered() {
        assert!(parse_heading("# REMEMBER GORDON!").is_none());
        assert!(parse_heading("## Rules of Play — Table of Contents").is_none());
        assert_eq!(
            parse_heading("## 5) Movement Phase"),
            Some((1, "5".to_string(), "Movement Phase".to_string()))
        );
        assert_eq!(
            parse_heading("### 5.4) Zones of Control"),
            Some((2, "5.4".to_string(), "Zones of Control".to_string()))
        );
    }

    #[test]
    fn splits_section_refs() {
        let toks = split_refs("see §5.26 and §6 here");
        let refs: Vec<&str> = toks
            .iter()
            .filter_map(|t| match t {
                RefTok::Ref(n) => Some(*n),
                RefTok::Text(_) => None,
            })
            .collect();
        assert_eq!(refs, vec!["5.26", "6"]);
    }
}

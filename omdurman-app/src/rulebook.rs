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

impl Rulebook {
    /// Look up a section's short title by its `§` number (e.g. `"5.26"` ->
    /// `"Units stop on entering enemy ZOC"`). Returns `None` when the section
    /// isn't in the parsed manual -- callers should fall back to a bare `§N`.
    ///
    /// Used by UI surfaces (dispatch slips, combat cards, tooltips) so a
    /// citation reads as `§5.26 Units stop on entering enemy ZOC` rather than
    /// an opaque `§5.26` -- closing the gap between a player who has not read
    /// the manual and the rule the engine just enforced.
    pub fn title_of(&self, number: &str) -> Option<&str> {
        self.sections
            .iter()
            .find(|s| s.number == number)
            .map(|s| s.title.as_str())
    }

    /// Render `text` with inline `§N` references as clickable links that deep-
    /// link into the Rulebook tab, and any non-§ text as plain labels. Each
    /// reference is annotated with its section title when one is known
    /// (`§5.41 Zones of Control`), so a reader sees the rule's name without
    /// leaving the current view.
    ///
    /// Returns the section number of any `§` reference the user clicked, so
    /// the caller can re-target the rulebook tab via [`request_section`].
    pub fn render_refs(&self, ui: &mut egui::Ui, text: &str) -> Option<String> {
        let title_for = |num: &str| self.title_of(num).map(str::to_owned);
        render_refs_with(ui, text, title_for)
    }

    /// Like [`Rulebook::render_refs`] but every `§` reference is rendered as a
    /// standalone clickable chip (used in lists / footers where each citation
    /// is on its own line). Returns the clicked section, if any.
    ///
    /// Kept as a public helper even though no caller currently uses it: footer
    /// citation strips are a natural fit for combat cards / dispatch slips and
    /// will likely land there once the patterns settle.
    #[allow(dead_code)]
    pub fn render_ref_chips(&self, ui: &mut egui::Ui, numbers: &[&str]) -> Option<String> {
        let mut clicked = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (i, num) in numbers.iter().enumerate() {
                if i > 0 {
                    ui.label(" ");
                }
                let title = self.title_of(num);
                let label = if let Some(t) = title {
                    format!("§{num} {t}")
                } else {
                    format!("§{num}")
                };
                if ui.link(label).clicked() {
                    clicked = Some((*num).to_string());
                }
            }
        });
        clicked
    }
}

/// A piece of text that is either a literal run or a `§N.M` section reference
/// (the latter rendered as a deep link into the Rulebook tab). Public so the
/// dispatch system, combat card, and tooltips can share one tokenizer.
pub enum RefTok<'a> {
    Text(&'a str),
    Ref(&'a str),
}

/// Split `text` into literal runs and `§N` / `§N.M` section references. Shared
/// by every UI surface that turns a body of text with rule citations into
/// clickable deep links (dispatch slips, combat resolution cards, tooltips).
pub fn split_refs(text: &str) -> Vec<RefTok<'_>> {
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

/// Render `text` with `§N` references as deep links, annotating each reference
/// with `title_for(number)` when one is available. Returns the clicked section.
/// Free function so callers without a [`Rulebook`] handy can still render with
/// a no-op title lookup.
pub fn render_refs_with<F: Fn(&str) -> Option<String>>(
    ui: &mut egui::Ui,
    text: &str,
    title_for: F,
) -> Option<String> {
    let mut clicked: Option<String> = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for tok in split_refs(text) {
            match tok {
                RefTok::Text(t) => {
                    ui.label(t);
                }
                RefTok::Ref(number) => {
                    let title = title_for(number);
                    let label = if let Some(t) = title {
                        format!("§{number} {t}")
                    } else {
                        format!("§{number}")
                    };
                    if ui.link(label).clicked() {
                        clicked = Some(number.to_string());
                    }
                }
            }
        }
    });
    clicked
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
        // The body renderer in the Rulebook tab itself annotates references
        // with their section title via the no-title fallback (`§N` alone),
        // because the user is already reading the manual -- a long chip would
        // be redundant. External callers (dispatch, combat card, tooltips)
        // use [`Rulebook::render_refs`] for the titled-chip form.
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

    #[test]
    fn title_of_finds_known_sections() {
        let rb = Rulebook::default();
        // Section 5 is the manual's Movement Phase -- a stable, well-known anchor.
        let title = rb.title_of("5").expect("section 5 exists");
        assert!(
            title.to_lowercase().contains("movement"),
            "section 5 title should mention movement, got {title}"
        );
        // An unknown section returns None (callers fall back to bare `§N`).
        assert!(rb.title_of("999.999").is_none());
    }
}

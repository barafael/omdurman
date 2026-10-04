//! The rulebook: `RememberGordonManual.md` parsed into a §-keyed section tree
//! and rendered inside the chart sheet's Rulebook tab. Searchable, with a
//! collapsible section index and §-deep-link support (scroll + brief spotlight).
//!
//! The manual is the single source of truth (shipped in the `Boardgame -
//! Remember_Gordon/Manual/` directory); its own `N)` / `N.M)` headings are the
//! section anchors -- there is no runtime dependency on `traceability.toml`.

use bevy::prelude::*;
use bevy_egui::egui;

/// The manual, embedded at build time so it ships with the binary and the wasm
/// bundle without a separate fetch.
const MANUAL_MD: &str =
    include_str!("../../Boardgame - Remember_Gordon/Manual/RememberGordonManual.md");

/// One parsed section: its § number (e.g. "5" or "5.4"), heading title, depth
/// (1 = `## N)`, 2 = `### N.M)`, 3 = `#### N.M.K)`), and body lines (until the
/// next heading). A numbered *paragraph* (`**6.63)** ...`) is its own section
/// too, so a `§6.63` link lands on it: `paragraph` is set, its body keeps the
/// bold number (it renders exactly as before, with no extra heading), and its
/// title is the paragraph's own bold title or its opening words.
#[derive(Clone)]
pub struct Section {
    pub number: String,
    pub title: String,
    pub depth: u8,
    pub body: String,
    pub paragraph: bool,
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
    /// Which sections match the search, for the needle they were computed
    /// for: lowercasing every section body twice a frame for an unchanged
    /// search is the tab's main per-frame cost.
    matches: Option<(String, Vec<bool>)>,
}

/// The manual's sections, parsed once. Shared by the Rulebook tab and every
/// `§` link's hover title ([`section_title`]), which needs no resource.
static SECTIONS: std::sync::LazyLock<Vec<Section>> =
    std::sync::LazyLock::new(|| parse_manual(MANUAL_MD));

/// The title of manual section `number` (`"6.63"` -> `"..."`), if it exists.
pub fn section_title(number: &str) -> Option<&'static str> {
    SECTIONS
        .iter()
        .find(|s| s.number == number)
        .map(|s| s.title.as_str())
}

/// egui-memory slot for a `§` link clicked this frame: any widget can ask
/// for the manual with only its `Ui` in hand, and the chart sheet
/// ([`take_requested_section`]) opens the Rulebook tab there.
fn requested_section_id() -> egui::Id {
    egui::Id::new("rulebook_requested_section")
}

/// Ask for the manual to open at section `number` (see [`ref_link`]).
pub fn request_open(ctx: &egui::Context, number: &str) {
    ctx.data_mut(|d| d.insert_temp(requested_section_id(), number.to_string()));
}

/// The section a `§` link asked for since the last call, if any.
pub fn take_requested_section(ctx: &egui::Context) -> Option<String> {
    ctx.data_mut(|d| d.remove_temp::<String>(requested_section_id()))
}

/// A `§N` link: hovering shows the section's title, a click opens the manual
/// (the chart sheet's Rulebook tab) scrolled to it.
pub fn ref_link(ui: &mut egui::Ui, number: &str, size: f32) -> egui::Response {
    let response = ui.add(
        egui::Label::new(
            egui::RichText::new(format!("§{number}"))
                .size(size)
                .underline()
                .color(ui.visuals().hyperlink_color),
        )
        .sense(egui::Sense::click()),
    );
    let response = match section_title(number) {
        Some(title) => response.on_hover_text(format!("§{number} {title} — open the rulebook")),
        None => response,
    }
    .on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.clicked() {
        request_open(ui.ctx(), number);
    }
    response
}

/// Render `text` in `color` at `size`, with every `§N` in it as a
/// [`ref_link`]: the one way to show a rule citation in the UI, so that each
/// opens the manual at its section.
pub fn refs_label(ui: &mut egui::Ui, text: &str, color: egui::Color32, size: f32) {
    refs_rich(ui, text, size, |t| t.color(color));
}

/// [`refs_label`] with a free text style (`|t| t.strong().color(..)`) for the
/// runs between the links.
pub fn refs_rich(
    ui: &mut egui::Ui,
    text: &str,
    size: f32,
    style: impl Fn(egui::RichText) -> egui::RichText,
) {
    if !text.contains('§') {
        ui.label(style(egui::RichText::new(text).size(size)));
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for tok in split_refs(text) {
            match tok {
                RefTok::Text(t) => {
                    ui.label(style(egui::RichText::new(t).size(size)));
                }
                RefTok::Ref(number) => {
                    ref_link(ui, number, size);
                }
            }
        }
    });
}

impl Default for Rulebook {
    fn default() -> Self {
        Self {
            sections: SECTIONS.clone(),
            search: String::new(),
            scroll_to: None,
            flash: None,
            matches: None,
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
                paragraph: false,
            });
        } else if current.is_some()
            && let Some((number, title)) = parse_paragraph(line)
        {
            if let Some(sec) = current.take() {
                sections.push(sec);
            }
            current = Some(Section {
                number,
                title,
                depth: 4,
                body: format!("{line}\n"),
                paragraph: true,
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
    let (hashes, rest) = if let Some(r) = line.strip_prefix("#### ") {
        (3u8, r)
    } else if let Some(r) = line.strip_prefix("### ") {
        (2u8, r)
    } else {
        let r = line.strip_prefix("## ")?;
        (1u8, r)
    };
    // Expect "<number>) <title>", where number is digits and dots.
    let (number, title) = rest.split_once(')')?;
    let number = number.trim();
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    Some((hashes, number.to_string(), title.trim().to_string()))
}

/// Parse a numbered paragraph line -- `**6.63)** Only artillery ...` or
/// `**6.51) Leader Units:**` -- into `(number, title)`. The title is the bold
/// one when present, else the paragraph's first words.
fn parse_paragraph(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("**")?;
    let (number, after) = rest.split_once(')')?;
    if !number.contains('.') || !number.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let (bold_title, text) = after.split_once("**")?;
    let bold_title = bold_title.trim().trim_end_matches(':').trim();
    let title = if bold_title.is_empty() {
        let words: Vec<&str> = text.split_whitespace().take(8).collect();
        let more = text.split_whitespace().count() > words.len();
        format!("{}{}", words.join(" "), if more { "\u{2026}" } else { "" })
    } else {
        bold_title.to_string()
    };
    Some((number.to_string(), title))
}

/// Request the rulebook scroll to (and briefly spotlight) a section number.
/// Called by the chart-sheet deep-link path when a `§` reference is followed.
pub fn request_section(rulebook: &mut Rulebook, number: &str) {
    rulebook.scroll_to = Some(number.to_string());
    rulebook.flash = Some((number.to_string(), 2.0));
}

impl Rulebook {
    /// Per section, whether it matches the (lowercased) search `needle` by
    /// number, title or body -- every section for an empty search. Cached
    /// until the needle changes.
    fn search_matches(&mut self, needle: &str) -> Vec<bool> {
        if let Some((cached, matches)) = &self.matches
            && cached == needle
        {
            return matches.clone();
        }
        let matches: Vec<bool> = self
            .sections
            .iter()
            .map(|sec| {
                needle.is_empty()
                    || sec.number.contains(needle)
                    || sec.title.to_lowercase().contains(needle)
                    || sec.body.to_lowercase().contains(needle)
            })
            .collect();
        self.matches = Some((needle.to_string(), matches.clone()));
        matches
    }

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

    /// Bare `§N` links, each section's opening words on hover -- compact
    /// enough for a card, where a full excerpt per reference outweighed the
    /// result itself.
    pub fn render_ref_links(&self, ui: &mut egui::Ui, numbers: &[&str]) -> Option<String> {
        let mut clicked = None;
        ui.horizontal_wrapped(|ui| {
            for num in numbers {
                let link = ui.link(egui::RichText::new(format!("§{num}")).size(11.0));
                let link = match self.title_of(num) {
                    Some(title) => link.on_hover_text(title),
                    None => link,
                };
                if link.clicked() {
                    clicked = Some((*num).to_string());
                }
            }
        });
        clicked
    }
}

/// Render `text` with every `§N` reference as plain text, annotated with its
/// section title when one is known — no click sensing. For surfaces that must
/// never claim the pointer (the board hover tooltip): an interactive widget
/// there would feed `EguiPointerOverUi` and nil the board-plane hover under
/// the cursor, and a click would fall through to the board beneath.
pub fn render_refs_plain(ui: &mut egui::Ui, text: &str, rulebook: Option<&Rulebook>) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for tok in split_refs(text) {
            match tok {
                RefTok::Text(t) => {
                    ui.label(t);
                }
                RefTok::Ref(number) => {
                    let label = match rulebook.and_then(|r| r.title_of(number)) {
                        Some(t) => format!("§{number} {t}"),
                        None => format!("§{number}"),
                    };
                    ui.label(label);
                }
            }
        }
    });
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

    egui::Panel::left("rulebook_index")
        .default_size(190.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut rulebook.search)
                    .hint_text("search…")
                    .desired_width(f32::INFINITY),
            );
            ui.separator();
            let needle = rulebook.search.to_lowercase();
            let shown = rulebook.search_matches(&needle);
            egui::ScrollArea::vertical()
                .id_salt("rulebook_toc")
                .show(ui, |ui| {
                    for (sec, &matches) in rulebook.sections.iter().zip(&shown) {
                        // Numbered paragraphs join the index only in a search
                        // (it would otherwise list every rule).
                        if sec.paragraph && needle.is_empty() {
                            continue;
                        }
                        // When searching, only show sections that match by
                        // number, title, or body.
                        if !matches {
                            continue;
                        }
                        let indent = match sec.depth {
                            1 => "",
                            2 => "   ",
                            _ => "      ",
                        };
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
    let shown = rulebook.search_matches(&needle);

    egui::ScrollArea::vertical()
        .id_salt("rulebook_body")
        .auto_shrink(false)
        .show(ui, |ui| {
            for (sec, &matches) in rulebook.sections.iter().zip(&shown) {
                if !matches {
                    continue;
                }

                // A numbered paragraph shows as before (its body opens with
                // its bold number); a heading gets its title line.
                let resp = if sec.paragraph {
                    let body = ui.scope(|ui| render_body(ui, &sec.body));
                    if let Some(r) = body.inner {
                        clicked_ref = Some(r);
                    }
                    body.response
                } else {
                    let heading = egui::RichText::new(format!("{}  {}", sec.number, sec.title))
                        .size(match sec.depth {
                            1 => 18.0,
                            2 => 15.0,
                            _ => 14.0,
                        })
                        .strong();
                    ui.label(heading)
                };

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
                        crate::ui::palette::with_alpha(crate::ui::palette::SEARCH_HIT, a),
                    );
                }

                if !sec.paragraph
                    && let Some(r) = render_body(ui, &sec.body)
                {
                    clicked_ref = Some(r);
                }
                ui.add_space(if sec.paragraph { 4.0 } else { 10.0 });
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
            // Markdown `**bold**` (the `**6.63)**` paragraph numbers, the
            // "**Sample Dervish Units**" lead-ins) may span a § link.
            let mut bold = false;
            for tok in split_refs(para) {
                match tok {
                    RefTok::Text(t) => {
                        for (strong, run) in bold_runs(t, &mut bold) {
                            let text = egui::RichText::new(run);
                            ui.label(if strong { text.strong() } else { text });
                        }
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

/// Split a run of manual text at its markdown `**` delimiters into
/// `(bold, text)` pieces, dropping the delimiters. `bold` is the state on
/// entry -- a bold span may continue from an earlier run of the same
/// paragraph (across a § link) -- and is left as the state on exit.
fn bold_runs<'a>(text: &'a str, bold: &mut bool) -> Vec<(bool, &'a str)> {
    let mut out = Vec::new();
    for (i, piece) in text.split("**").enumerate() {
        if i > 0 {
            *bold = !*bold;
        }
        if !piece.is_empty() {
            out.push((*bold, piece));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every § link hands its section to the chart sheet through egui memory;
    // the sheet takes it exactly once (and opens the Rulebook tab).
    #[test]
    fn a_clicked_reference_reaches_the_sheet_once() {
        let ctx = egui::Context::default();
        request_open(&ctx, "6.63");
        assert_eq!(take_requested_section(&ctx).as_deref(), Some("6.63"));
        assert_eq!(take_requested_section(&ctx), None);
    }

    // Citations mostly name numbered paragraphs (6.63, 9.346) and ####
    // headings (9.35): each is a section a link can land on. (Only ## / ###
    // headings used to be, so most links opened the manual and went nowhere.)
    // The numbers carry no section sign: this test covers the link targets,
    // not those rules.
    #[test]
    fn link_targets_include_paragraphs_and_deep_headings() {
        assert_eq!(section_title("6.51"), Some("Leader Units"));
        assert_eq!(section_title("9.35"), Some("Victory Conditions"));
        assert!(section_title("6.63").is_some_and(|t| t.starts_with("Only artillery may fire")));
        assert!(section_title("9.346").is_some());
        assert!(section_title("99.9").is_none());
    }

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
    fn bold_markers_become_bold_runs_not_asterisks() {
        let mut bold = false;
        assert_eq!(
            bold_runs("**5.11)** The movement allowances", &mut bold),
            vec![(true, "5.11)"), (false, " The movement allowances")]
        );
        assert!(!bold);
        // A bold span left open runs on into the next run (past a § link).
        let mut bold = false;
        assert_eq!(bold_runs("**see ", &mut bold), vec![(true, "see ")]);
        assert!(bold);
        assert_eq!(
            bold_runs(" below** then", &mut bold),
            vec![(true, " below"), (false, " then")]
        );
        assert!(!bold);
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

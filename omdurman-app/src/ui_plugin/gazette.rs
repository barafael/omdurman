//! The end-of-game front page, drawn loosely after the Victorian papers in
//! `newspaper-reference/` (The Graphic, the Illustrated London News, the New
//! York Times of 1890): a blackletter masthead over a double rule, the
//! issue / date / price line, a banner headline, the lead article with
//! stacked decks and a dateline beside a column of late telegrams, and the
//! other news of the day in narrow justified columns below.
use super::*;
use omdurman_rules::press::gazette::{Article, FrontPage};

/// Aged newsprint and its ink.
const PAPER: egui::Color32 = egui::Color32::from_rgb(234, 224, 197);
const INK: egui::Color32 = egui::Color32::from_rgb(30, 26, 20);
const FADED: egui::Color32 = egui::Color32::from_rgb(88, 76, 58);

fn font(family: &str, size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name(family.into()))
}

fn body(size: f32) -> egui::FontId {
    font("OldStandard", size)
}

fn bold(size: f32) -> egui::FontId {
    font("OldStandardBold", size)
}

fn italic(size: f32) -> egui::FontId {
    font("OldStandardItalic", size)
}

/// A justified paragraph, `lead` (a dateline) set in bold before it.
fn paragraph(ui: &mut egui::Ui, lead: Option<&str>, text: &str, size: f32) {
    let mut job = egui::text::LayoutJob {
        justify: true,
        ..Default::default()
    };
    job.wrap.max_width = ui.available_width();
    if let Some(lead) = lead {
        job.append(
            &format!("{lead} "),
            0.0,
            egui::TextFormat::simple(bold(size), INK),
        );
    }
    // A paragraph indent, as the columns of the day.
    let indent = if lead.is_some() { 0.0 } else { size * 1.2 };
    job.append(text, indent, egui::TextFormat::simple(body(size), INK));
    ui.label(job);
    ui.add_space(size * 0.35);
}

/// A centred line in the given font.
fn centred(ui: &mut egui::Ui, text: &str, font: egui::FontId, color: egui::Color32) {
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(text).font(font).color(color));
    });
}

/// A horizontal rule across the available width; `double` draws two.
fn rule(ui: &mut egui::Ui, width: f32, double: bool) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), if double { 6.0 } else { 3.0 }),
        egui::Sense::hover(),
    );
    let stroke = egui::Stroke::new(width, INK);
    ui.painter().hline(rect.x_range(), rect.top() + 1.0, stroke);
    if double {
        ui.painter().hline(
            rect.x_range(),
            rect.bottom() - 1.0,
            egui::Stroke::new(0.8, INK),
        );
    }
}

/// A short centred rule between the decks of a head.
fn short_rule(ui: &mut egui::Ui) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 11.0), egui::Sense::hover());
    let mid = rect.center().x;
    ui.painter().hline(
        (mid - 30.0)..=(mid + 30.0),
        rect.center().y,
        egui::Stroke::new(0.8, INK),
    );
}

/// A small article: centred small-caps head and justified paragraphs.
fn feature(ui: &mut egui::Ui, article: &Article) {
    centred(ui, &article.head, bold(13.0), INK);
    short_rule(ui);
    for text in &article.paragraphs {
        paragraph(ui, None, text, 12.5);
    }
    ui.add_space(6.0);
}

/// The whole front page at `width` points.
pub(crate) fn draw_front_page(ui: &mut egui::Ui, page: &FrontPage, width: f32) {
    egui::Frame::new()
        .fill(PAPER)
        .stroke(egui::Stroke::new(1.0, FADED))
        .inner_margin(egui::Margin::symmetric(28, 18))
        .shadow(egui::epaint::Shadow {
            offset: [6, 8],
            blur: 18,
            spread: 0,
            color: egui::Color32::from_black_alpha(140),
        })
        .show(ui, |ui| {
            ui.set_width(width - 56.0);
            ui.spacing_mut().item_spacing.y = 2.0;

            // -- masthead --------------------------------------------------
            centred(ui, &page.masthead, font("Masthead", 58.0), INK);
            centred(ui, "PUBLISHED DAILY IN LONDON", italic(11.0), FADED);
            ui.add_space(4.0);
            rule(ui, 2.0, true);
            ui.horizontal(|ui| {
                let third = ui.available_width() / 3.0;
                for (text, align) in [
                    (page.issue.as_str(), egui::Align::Min),
                    (page.date.as_str(), egui::Align::Center),
                    (page.price.as_str(), egui::Align::Max),
                ] {
                    ui.allocate_ui_with_layout(
                        egui::vec2(third, 18.0),
                        egui::Layout::top_down(align),
                        |ui| {
                            ui.set_min_width(third);
                            ui.label(
                                egui::RichText::new(text.to_uppercase())
                                    .font(body(12.0))
                                    .color(INK),
                            );
                        },
                    );
                }
            });
            rule(ui, 1.0, true);
            ui.add_space(6.0);

            // -- banner headline -------------------------------------------
            centred(ui, &page.headline, bold(34.0), INK);
            ui.add_space(4.0);
            rule(ui, 1.0, false);
            ui.add_space(6.0);

            // -- lead article beside the late telegrams --------------------
            let full = ui.available_width();
            let gutter = 18.0;
            let lead_w = (full - gutter) * 0.66;
            let side_w = full - gutter - lead_w;
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(lead_w, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(lead_w);
                        let lead = &page.lead;
                        centred(ui, &lead.head, bold(22.0), INK);
                        for deck in &lead.decks {
                            short_rule(ui);
                            centred(ui, &deck.to_uppercase(), body(13.0), INK);
                        }
                        short_rule(ui);
                        ui.add_space(4.0);
                        for (i, text) in lead.paragraphs.iter().enumerate() {
                            let dateline = if i == 0 {
                                lead.dateline.as_deref()
                            } else {
                                None
                            };
                            paragraph(ui, dateline, text, 14.5);
                        }
                        if !page.chronicle.is_empty() {
                            ui.add_space(6.0);
                            centred(ui, "THE COURSE OF THE BATTLE", bold(15.0), INK);
                            short_rule(ui);
                            for (head, text) in &page.chronicle {
                                let stop = if head.ends_with('.') { "" } else { "." };
                                paragraph(ui, Some(&format!("{head}{stop}\u{2014}")), text, 13.5);
                            }
                        }
                    },
                );
                // A column rule.
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(gutter, ui.min_rect().height().max(200.0)),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .vline(rect.center().x, rect.y_range(), egui::Stroke::new(0.8, INK));
                ui.allocate_ui_with_layout(
                    egui::vec2(side_w, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(side_w);
                        centred(ui, "LATE TELEGRAMS", bold(15.0), INK);
                        short_rule(ui);
                        for (head, text) in &page.telegrams {
                            ui.label(egui::RichText::new(head).font(italic(12.0)).color(FADED));
                            ui.label(egui::RichText::new(text).font(body(12.5)).color(INK));
                            ui.add_space(5.0);
                        }
                        if !page.roll_of_honour.is_empty() {
                            ui.add_space(4.0);
                            rule(ui, 0.8, false);
                            centred(ui, "ROLL OF HONOUR", bold(13.0), INK);
                            short_rule(ui);
                            for name in &page.roll_of_honour {
                                centred(ui, name, body(12.5), INK);
                            }
                        }
                    },
                );
            });

            // -- the other news, in three columns --------------------------
            ui.add_space(6.0);
            rule(ui, 1.5, true);
            ui.add_space(4.0);
            ui.columns(3, |columns| {
                for (i, article) in page.features.iter().enumerate() {
                    feature(&mut columns[i % 3], article);
                }
                // The advertisements close the last column.
                let ads = &mut columns[2];
                for advert in &page.adverts {
                    egui::Frame::new()
                        .stroke(egui::Stroke::new(1.0, INK))
                        .inner_margin(egui::Margin::same(6))
                        .show(ads, |ui| {
                            ui.set_width(ui.available_width());
                            centred(ui, &advert.head, bold(12.5), INK);
                            for line in &advert.lines {
                                centred(ui, line, italic(11.5), INK);
                            }
                        });
                    ads.add_space(6.0);
                }
            });
        });
}

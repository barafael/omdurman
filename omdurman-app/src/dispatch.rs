//! Period-dispatch system messages (§decision 10). Rejections, combat results,
//! and other terse notices render as a small "field telegraph" slip: a paper
//! card with a 2px ink border and a letter-spaced small-caps header. The flavour
//! lives only in the frame and header line; the body stays dry and factual, with
//! any rulebook `§` reference rendered as a deep link into the Rulebook tab.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::theme;

/// One queued dispatch slip.
pub struct Dispatch {
    /// Small-caps header line, e.g. "DISPATCH" or "FIELD TELEGRAPH".
    pub header: String,
    /// Dry, factual body. Any `§N` reference in it becomes a rulebook link.
    pub body: String,
    /// Seconds this slip has been shown (for fade-out + expiry).
    pub age: f32,
}

/// The live dispatch queue. Newest slips stack at the bottom-left; each expires
/// after [`DISPATCH_TTL`] seconds.
#[derive(Resource, Default)]
pub struct Dispatches {
    pub slips: Vec<Dispatch>,
}

impl Dispatches {
    /// Queue a dispatch. `header` is the small-caps frame label; `body` is the
    /// factual message (may contain `§N` references).
    pub fn push(&mut self, header: impl Into<String>, body: impl Into<String>) {
        self.slips.push(Dispatch {
            header: header.into(),
            body: body.into(),
            age: 0.0,
        });
        // Cap the backlog so a burst can't pile up indefinitely.
        const MAX: usize = 5;
        let len = self.slips.len();
        if len > MAX {
            self.slips.drain(0..len - MAX);
        }
    }
}

const DISPATCH_TTL: f32 = 6.0;
const DISPATCH_FADE: f32 = 1.0;

pub struct DispatchPlugin;

impl Plugin for DispatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Dispatches>()
            .add_systems(EguiPrimaryContextPass, draw_dispatches);
        // Dev: keep demo slips on screen for headless verification
        // (OMDURMAN_DISPATCH=1) by re-seeding whenever the queue empties.
        if std::env::var("OMDURMAN_DISPATCH").is_ok() {
            app.add_systems(Update, |mut d: ResMut<Dispatches>| {
                if d.slips.is_empty() {
                    d.push("Field Telegraph", "Fire refused — no line of sight (§6.3).");
                    d.push("Dispatch", "Move rejected — zone of control (§5.4).");
                }
            });
        }
    }
}

fn draw_dispatches(
    mut contexts: EguiContexts,
    mut dispatches: ResMut<Dispatches>,
    time: Res<Time>,
    mut rulebook: ResMut<crate::rulebook::Rulebook>,
) {
    // Age and expire.
    let dt = time.delta_secs();
    for slip in &mut dispatches.slips {
        slip.age += dt;
    }
    dispatches.slips.retain(|s| s.age < DISPATCH_TTL);
    if dispatches.slips.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut clicked_section: Option<String> = None;

    egui::Area::new(egui::Id::new("dispatch_slips"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(14.0, -48.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.set_max_width(320.0);
            // Oldest on top, newest at the bottom (nearest the corner).
            for slip in &dispatches.slips {
                let fade = ((DISPATCH_TTL - slip.age) / DISPATCH_FADE).clamp(0.0, 1.0);
                if let Some(sec) = draw_slip(ui, slip, fade) {
                    clicked_section = Some(sec);
                }
                ui.add_space(6.0);
            }
        });

    if let Some(sec) = clicked_section {
        crate::rulebook::request_section(&mut rulebook, &sec);
    }
    ctx.request_repaint(); // keep the fade animating
}

/// Draw one slip; returns a section number if the player clicked a `§` link.
fn draw_slip(ui: &mut egui::Ui, slip: &Dispatch, fade: f32) -> Option<String> {
    let a = |c: egui::Color32| c.gamma_multiply(fade);
    let mut clicked = None;

    egui::Frame::new()
        .fill(a(theme::PAPER_CHART))
        .stroke(egui::Stroke::new(2.0, a(theme::INK)))
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.set_max_width(300.0);
            // Header: letter-spaced small caps, faint ink.
            let header: String = slip
                .header
                .to_uppercase()
                .chars()
                .flat_map(|c| [c, '\u{2009}']) // thin space between glyphs
                .collect();
            ui.label(
                egui::RichText::new(header)
                    .color(a(theme::INK_FAINT))
                    .size(11.0)
                    .strong(),
            );
            ui.add_space(2.0);
            // Body: dry text with §N references as rulebook links.
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for tok in split_refs(&slip.body) {
                    match tok {
                        Tok::Text(t) => {
                            ui.label(egui::RichText::new(t).color(a(theme::INK)).size(14.0));
                        }
                        Tok::Ref(n) => {
                            if ui
                                .add(
                                    egui::Label::new(
                                        egui::RichText::new(format!("§{n}"))
                                            .color(a(theme::INK))
                                            .size(14.0)
                                            .underline(),
                                    )
                                    .sense(egui::Sense::click()),
                                )
                                .clicked()
                            {
                                clicked = Some(n.to_string());
                            }
                        }
                    }
                }
            });
        });
    clicked
}

enum Tok<'a> {
    Text(&'a str),
    Ref(&'a str),
}

/// Split on `§N` / `§N.M` references (shared shape with the rulebook body).
fn split_refs(text: &str) -> Vec<Tok<'_>> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(pos) = rest.find('§') {
        if pos > 0 {
            out.push(Tok::Text(&rest[..pos]));
        }
        let after = &rest[pos + '§'.len_utf8()..];
        let num_len = after
            .char_indices()
            .take_while(|(_, c)| c.is_ascii_digit() || *c == '.')
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        if num_len == 0 {
            out.push(Tok::Text(&rest[pos..pos + '§'.len_utf8()]));
            rest = after;
        } else {
            out.push(Tok::Ref(&after[..num_len]));
            rest = &after[num_len..];
        }
    }
    if !rest.is_empty() {
        out.push(Tok::Text(rest));
    }
    out
}

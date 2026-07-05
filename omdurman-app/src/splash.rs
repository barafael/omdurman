//! Loading / start menu: a full-screen panel shown at app start with the game
//! title and a randomly-picked war-and-peace epigraph, while the (slow, ~30 MB)
//! board textures decode and upload in the background.
//!
//! The panel stays up until the startup Fall-of-Khartoum board texture reports
//! [`LoadState::Loaded`] (see [`crate::render::spawn_map_plane`]); at that point
//! it shows three entry buttons — Lobby, Fall of Khartoum Map, Campaign Map —
//! and dismisses once the player picks one, navigating to that destination.

use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use rand::RngExt;

use crate::{AppState, EditorMode};

/// The curated quote pool, embedded at build time. Lives in `assets/quotes.md`
/// so it ships with the app and stays hand-curatable; parsed once on startup.
const QUOTES_MD: &str = include_str!("../assets/quotes.md");

/// One epigraph: the quote text and its attribution.
#[derive(Clone)]
struct Quote {
    text: String,
    attribution: String,
}

/// Start-menu state. Present until the player picks a destination, then the
/// resource is removed and the panel stops drawing.
#[derive(Resource)]
struct Splash {
    quote: Option<Quote>,
    /// Set true once the startup board texture has finished loading; gates the
    /// entry buttons (before that the panel just shows the quote + "Loading…").
    loaded: bool,
}

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Splash {
            quote: pick_quote(),
            loaded: false,
        })
        .add_systems(Update, update_loaded)
        .add_systems(EguiPrimaryContextPass, splash_ui);
    }
}

/// Parse `quotes.md` into quote blocks. Blocks are separated by a line that is
/// exactly `---`; within a block the quote is the `> ` line and the attribution
/// is the `— ` line. Everything before the first `---` is preamble and ignored.
fn parse_quotes(md: &str) -> Vec<Quote> {
    md.split("\n---\n")
        .skip(1) // drop the heading/format preamble above the first separator
        .filter_map(|block| {
            let mut text: Option<String> = None;
            let mut attribution: Option<String> = None;
            for line in block.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("> ") {
                    text = Some(rest.trim().to_string());
                } else if let Some(rest) = line.strip_prefix("— ") {
                    attribution = Some(rest.trim().to_string());
                }
            }
            match (text, attribution) {
                (Some(text), attribution) => Some(Quote {
                    text,
                    attribution: attribution.unwrap_or_default(),
                }),
                _ => None,
            }
        })
        .collect()
}

/// Build a [`egui::text::LayoutJob`] from text with `*...*` markdown emphasis,
/// rendering the emphasized runs italic (and the rest at `base_italic`). Used so
/// `*The Histories*` in `quotes.md` shows as italic rather than literal asterisks.
///
/// Splitting on `*` makes every odd-indexed segment the emphasized one; a stray
/// unmatched `*` just leaves its trailing segment un-emphasized, which is fine.
fn emphasis_job(
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    base_italic: bool,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (i, segment) in text.split('*').enumerate() {
        if segment.is_empty() {
            continue;
        }
        let italic = if i % 2 == 1 { true } else { base_italic };
        job.append(
            segment,
            0.0,
            egui::TextFormat {
                font_id: font.clone(),
                color,
                italics: italic,
                ..Default::default()
            },
        );
    }
    job
}

/// Pick one quote at random from the pool, or `None` if the pool is empty.
///
/// Uses the thread RNG (`rand::rng()`, matching [`crate::settings`]) rather than
/// the seeded game PRNG — the splash pick is cosmetic and must not perturb the
/// deterministic game sequence. The `getrandom` wasm_js backend (Cargo.toml)
/// makes this work on the web build too.
fn pick_quote() -> Option<Quote> {
    let quotes = parse_quotes(QUOTES_MD);
    if quotes.is_empty() {
        warn!("splash: no quotes parsed from assets/quotes.md");
        return None;
    }
    let idx = rand::rng().random_range(0..quotes.len());
    quotes.into_iter().nth(idx)
}

/// Flip the [`Splash::loaded`] flag once the startup board texture has finished
/// decoding — that's the cue to reveal the entry buttons. The panel is *not*
/// dismissed here; the player dismisses it by picking a destination in
/// `splash_ui`. If the cache/handle isn't present yet we simply keep waiting.
fn update_loaded(
    asset_server: Res<AssetServer>,
    cache: Option<Res<crate::render::MapTextureCache>>,
    splash: Option<ResMut<Splash>>,
) {
    let Some(mut splash) = splash else { return };
    if splash.loaded {
        return;
    }
    let loaded = cache
        .and_then(|cache| cache.0.get("fall_of_khartoum_1885.webp").cloned())
        .map(|handle| matches!(asset_server.load_state(&handle), LoadState::Loaded))
        .unwrap_or(false);
    if loaded {
        splash.loaded = true;
    }
}

/// Draw the full-screen splash. No-op once the [`Splash`] resource is gone.
///
/// Drawn as a foreground [`egui::Area`] painting an opaque rect over the entire
/// screen, rather than a `CentralPanel` — a `CentralPanel` only fills the space
/// left by the map-mode `SidePanel`s (the unit-overview sidebar is up from the
/// first frame, see `EditorMode::FallOfKhartoumMap` default), so those would
/// show through at the edges. A full-screen foreground area covers them.
fn splash_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    splash: Option<Res<Splash>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_editor: ResMut<NextState<EditorMode>>,
) {
    let Some(splash) = splash else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // A destination the player picked this frame, applied after the UI closure.
    let mut chosen: Option<Destination> = None;

    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("splash_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            // Opaque backdrop over the whole window, hiding any chrome beneath.
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_gray(16));

            // One font family (EB Garamond serif, registered in
            // `ui_plugin::setup_egui_fonts`) and a tight three-step size scale
            // throughout, so the screen reads as one typographic system.
            let serif =
                |size: f32| egui::FontId::new(size, egui::FontFamily::Name("Garamond".into()));
            const TITLE: f32 = 52.0;
            const QUOTE: f32 = 34.0;
            const SMALL: f32 = 22.0;

            ui.allocate_ui_with_layout(
                screen.size(),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(screen.height() * 0.26);
                    ui.label(
                        egui::RichText::new("REMEMBER GORDON!")
                            .font(serif(TITLE))
                            .color(egui::Color32::from_rgb(214, 178, 106)),
                    );
                    ui.add_space(56.0);

                    if let Some(quote) = &splash.quote {
                        ui.set_max_width((screen.width() * 0.7).min(820.0));
                        // Quote body is italic throughout; `*...*` runs stay
                        // italic too (no visible toggle), so we just wrap it.
                        ui.label(emphasis_job(
                            &format!("\u{201c}{}\u{201d}", quote.text),
                            serif(QUOTE),
                            egui::Color32::from_gray(228),
                            true,
                        ));
                        if !quote.attribution.is_empty() {
                            ui.add_space(16.0);
                            // Attribution is upright; `*Title*` runs render italic.
                            ui.label(emphasis_job(
                                &format!("\u{2014} {}", quote.attribution),
                                serif(SMALL),
                                egui::Color32::from_gray(160),
                                false,
                            ));
                        }
                    }

                    ui.add_space(56.0);
                    if !splash.loaded {
                        ui.label(
                            egui::RichText::new("Loading\u{2026}")
                                .font(serif(SMALL))
                                .color(egui::Color32::from_gray(120)),
                        );
                    } else {
                        // Entry buttons, revealed once the board texture is ready.
                        let button = |ui: &mut egui::Ui, label: &str| {
                            ui.add_sized(
                                [300.0, 44.0],
                                egui::Button::new(
                                    egui::RichText::new(label)
                                        .font(serif(SMALL))
                                        .color(egui::Color32::from_gray(230)),
                                ),
                            )
                            .clicked()
                        };
                        if button(ui, "Lobby") {
                            chosen = Some(Destination::Lobby);
                        }
                        ui.add_space(12.0);
                        if button(ui, "Fall Of Khartoum Map") {
                            chosen = Some(Destination::Map(EditorMode::FallOfKhartoumMap));
                        }
                        ui.add_space(12.0);
                        if button(ui, "Campaign Map") {
                            chosen = Some(Destination::Map(EditorMode::CampaignMap));
                        }
                    }
                },
            );
        });

    // Apply the pick and dismiss. Map switches drive the board load via
    // `editor::sync_edit_board_to_mode` (which sets `PendingMapLoad` on a mode
    // change), mirroring the mode toolbar; the lobby is a plain state switch.
    if let Some(dest) = chosen {
        match dest {
            Destination::Lobby => {
                info!("splash: entering lobby");
                next_app_state.set(AppState::Lobby);
            }
            Destination::Map(mode) => {
                info!(?mode, "splash: entering map");
                next_editor.set(mode);
            }
        }
        commands.remove_resource::<Splash>();
    }
}

/// Where a start-menu button sends the player.
enum Destination {
    Lobby,
    Map(EditorMode),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_quotes_parse() {
        let quotes = parse_quotes(QUOTES_MD);
        assert!(
            !quotes.is_empty(),
            "assets/quotes.md should contain at least one quote block"
        );
        for q in &quotes {
            assert!(!q.text.is_empty(), "every quote must have text");
        }
    }

    #[test]
    fn emphasis_marks_star_runs_italic() {
        let font = egui::FontId::proportional(16.0);
        let color = egui::Color32::WHITE;
        let job = emphasis_job("plain *italic* plain", font, color, false);
        // Three sections: "plain ", "italic", " plain".
        assert_eq!(job.sections.len(), 3);
        assert!(!job.sections[0].format.italics);
        assert!(
            job.sections[1].format.italics,
            "the *starred* run is italic"
        );
        assert!(!job.sections[2].format.italics);
        // The literal asterisks are consumed, not rendered.
        assert!(!job.text.contains('*'));
    }

    #[test]
    fn preamble_is_ignored() {
        let md = "# Heading\n\nnotes here\n> not a real quote (above first separator)\n---\n> Real quote.\n— Someone\n";
        let quotes = parse_quotes(md);
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].text, "Real quote.");
        assert_eq!(quotes[0].attribution, "Someone");
    }

    #[test]
    fn attribution_optional() {
        let md = "preamble\n---\n> A quote with no attribution.\n";
        let quotes = parse_quotes(md);
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].attribution, "");
    }
}

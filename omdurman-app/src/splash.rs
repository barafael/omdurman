//! Loading / start menu: a full-screen panel shown at app start with the game
//! title and a randomly-picked war-and-peace epigraph, while the (slow, ~30 MB)
//! board textures decode and upload in the background.
//!
//! Once loaded the panel transitions to [`AppMode::Menu`] — the persistent hub
//! for mode selection. Pressing **M** from any mode returns here. The menu
//! overlay is semi-transparent over play views (Game) and opaque over
//! full-screen UIs (Lobby/Editor).

use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{AppMode, AppState, GameSnapshot};

/// The curated quote pool, embedded at build time. Lives in `assets/quotes.md`
/// so it ships with the app and stays hand-curatable; parsed once on startup.
const QUOTES_MD: &str = include_str!("../assets/quotes.md");

/// One epigraph: the quote text and its attribution.
#[derive(Clone)]
pub(crate) struct Quote {
    text: String,
    attribution: String,
}

/// Data held by the splash screen while it is active. The "start screen is up"
/// signal is now the `AppState::Splash` state variant; this resource only
/// carries the quote and the loaded flag.
#[derive(Resource)]
pub(crate) struct SplashData {
    pub quote: Option<Quote>,
    /// Set true once the startup board texture has finished loading; gates the
    /// entry buttons (before that the panel just shows the quote + "Loading…").
    pub loaded: bool,
}

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SplashData {
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
/// rendering the emphasized runs italic (and the rest at `base_italic`).
fn emphasis_job(
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    base_italic: bool,
    wrap_width: f32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob {
        halign: egui::Align::Center,
        wrap: egui::text::TextWrapping {
            max_width: wrap_width,
            break_anywhere: false,
            overflow_character: None,
            ..Default::default()
        },
        ..Default::default()
    };
    let italic_font = egui::FontId::new(font.size, egui::FontFamily::Name("GaramondItalic".into()));
    for (i, segment) in text.split('*').enumerate() {
        if segment.is_empty() {
            continue;
        }
        let italic = if i % 2 == 1 { true } else { base_italic };
        job.append(
            segment,
            0.0,
            egui::TextFormat {
                font_id: if italic {
                    italic_font.clone()
                } else {
                    font.clone()
                },
                color,
                ..Default::default()
            },
        );
    }
    job
}

/// Pick one quote at random from the pool, or `None` if the pool is empty.
fn pick_quote() -> Option<Quote> {
    let quotes = parse_quotes(QUOTES_MD);
    if quotes.is_empty() {
        warn!("splash: no quotes parsed from assets/quotes.md");
        return None;
    }
    use rand::seq::IndexedRandom;
    quotes.choose(&mut rand::rng()).cloned()
}

/// Flip the [`SplashData::loaded`] flag once the startup board texture has
/// finished decoding. On first load completion, transitions to
/// [`AppMode::Menu`] (the persistent hub).
#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
fn update_loaded(
    asset_server: Res<AssetServer>,
    cache: Option<Res<crate::render::MapTextureCache>>,
    splash_data: Option<ResMut<SplashData>>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_app_mode: ResMut<NextState<AppMode>>,
    mut timeline: ResMut<crate::timeline::SpectatorTimeline>,
) {
    if *app_state.get() != AppState::Splash {
        return;
    }
    let Some(mut splash_data) = splash_data else {
        return;
    };
    if splash_data.loaded {
        return;
    }
    let loaded = cache
        .and_then(|cache| cache.0.get("fall_of_khartoum_1885.webp").cloned())
        .map(|handle| matches!(asset_server.load_state(&handle), LoadState::Loaded))
        .unwrap_or(false);
    if loaded {
        splash_data.loaded = true;
        // Dev affordance: open a recorded game straight into the spectator
        // timeline (native only; pairs with OMDURMAN_OFFLINE for reviewing
        // bot playthroughs without touching the lobby).
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(path) = std::env::var("OMDURMAN_REPLAY")
            && !path.is_empty()
        {
            match crate::game_record::load_record_from_jsonl(&path) {
                Ok(record) => {
                    info!(
                        %path,
                        events = record.events.len(),
                        "splash: opening replay (OMDURMAN_REPLAY)"
                    );
                    timeline.open(record, path);
                    // OMDURMAN_REPLAY_PLAY=1: start playback from the first
                    // event instead of parking on the last (verification aid
                    // for the scrub path). OMDURMAN_REPLAY_AT=<idx>: park on
                    // a specific event (verification aid for per-event
                    // visuals -- fire tracers, movement animation).
                    if std::env::var("OMDURMAN_REPLAY_PLAY").is_ok() {
                        timeline.cursor = 0;
                        timeline.playing = true;
                    } else if let Some(at) = std::env::var("OMDURMAN_REPLAY_AT")
                        .ok()
                        .and_then(|s| s.parse::<usize>().ok())
                    {
                        timeline.cursor = at;
                    }
                    next_app_state.set(AppState::Spectating);
                    return;
                }
                Err(error) => {
                    warn!(%error, %path, "splash: OMDURMAN_REPLAY failed to load; entering menu");
                }
            }
        }
        // Dev affordance: skip the start menu straight into a mode.
        if let Some(mode) = std::env::var("OMDURMAN_START_MODE").ok().and_then(|s| {
            AppMode::ALL
                .iter()
                .find(|m| m.to_string().eq_ignore_ascii_case(&s))
                .copied()
        }) {
            info!(?mode, "splash: auto-entering mode (OMDURMAN_START_MODE)");
            next_app_mode.set(mode);
            // Each AppMode pairs with a specific AppState; the restore hooks no
            // longer drive AppState, so set it here. Lobby is the only one that
            // isn't InGame.
            next_app_state.set(match mode {
                AppMode::Lobby => AppState::Lobby,
                _ => AppState::InGame,
            });
        } else {
            // Normal path: enter the persistent menu.
            info!("splash: entering menu");
            next_app_mode.set(AppMode::Menu);
            next_app_state.set(AppState::InGame);
        }
    }
}

/// Draw the full-screen splash / persistent menu.
///
/// Runs during `AppState::Splash` (initial load) and whenever
/// `AppMode::Menu` is active (returning via M key). During initial load the
/// background is opaque; on return it is semi-transparent so the game board
/// shows through.
fn splash_ui(
    mut contexts: EguiContexts,
    splash_data: Option<Res<SplashData>>,
    app_state: Res<State<AppState>>,
    mode: Res<State<AppMode>>,
    game_snapshot: Res<GameSnapshot>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_app_mode: ResMut<NextState<AppMode>>,
) {
    // Show during initial splash OR while in Menu mode.
    let is_splash = *app_state.get() == AppState::Splash;
    let is_menu = *mode.get() == AppMode::Menu;
    if !is_splash && !is_menu {
        return;
    }
    let Some(splash_data) = splash_data else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // A destination the player picked this frame, applied after the UI closure.
    let mut chosen: Option<Destination> = None;

    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("splash_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            // Opaque backdrop during initial load; semi-transparent when
            // returning to menu from a mode (so the board shows through).
            let bg_alpha = if is_splash { 255u8 } else { 200u8 };
            ui.painter().rect_filled(
                screen,
                0.0,
                egui::Color32::from_rgba_premultiplied(16, 16, 16, bg_alpha),
            );

            // One font family (Merriweather serif, registered in
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

                    if let Some(quote) = &splash_data.quote {
                        // Shared wrap width for the quote block. The job wraps
                        // itself at this width (word boundaries only), so the ui
                        // container must be at least this wide or it would clip.
                        let wrap_w = (screen.width() * 0.7).min(820.0);
                        ui.set_max_width(wrap_w);
                        // Quote body is italic throughout; `*...*` runs stay
                        // italic too (no visible toggle), so we just wrap it.
                        ui.label(emphasis_job(
                            &format!("\u{201c}{}\u{201d}", quote.text),
                            serif(QUOTE),
                            egui::Color32::from_gray(228),
                            true,
                            wrap_w,
                        ));
                        if !quote.attribution.is_empty() {
                            ui.add_space(16.0);
                            // Attribution is upright; `*Title*` runs render italic.
                            ui.label(emphasis_job(
                                &format!("\u{2014} {}", quote.attribution),
                                serif(SMALL),
                                egui::Color32::from_gray(160),
                                false,
                                wrap_w,
                            ));
                        }
                    }

                    ui.add_space(56.0);
                    if !splash_data.loaded {
                        ui.label(
                            egui::RichText::new("Loading\u{2026}")
                                .font(serif(SMALL))
                                .color(egui::Color32::from_gray(120)),
                        );
                    } else {
                        // Entry buttons, revealed once the board texture is ready.
                        // The splash is an art-directed dark title card, not paper
                        // chrome, so it keeps its own dark button visuals rather
                        // than inheriting the global paper skin (which would paint
                        // cream fills that wash out this screen's light-grey text).
                        {
                            let w = &mut ui.visuals_mut().widgets;
                            w.inactive.weak_bg_fill = egui::Color32::from_gray(32);
                            w.inactive.bg_fill = egui::Color32::from_gray(32);
                            w.inactive.bg_stroke =
                                egui::Stroke::new(1.0_f32, egui::Color32::from_gray(90));
                            w.hovered.weak_bg_fill = egui::Color32::from_gray(52);
                            w.hovered.bg_fill = egui::Color32::from_gray(52);
                            w.hovered.bg_stroke =
                                egui::Stroke::new(1.0_f32, egui::Color32::from_gray(150));
                            w.active.weak_bg_fill = egui::Color32::from_gray(70);
                            w.active.bg_fill = egui::Color32::from_gray(70);
                            w.active.bg_stroke =
                                egui::Stroke::new(1.0_f32, egui::Color32::from_gray(180));
                            for state in [&mut w.inactive, &mut w.hovered, &mut w.active] {
                                state.corner_radius = egui::CornerRadius::same(2);
                            }
                        }
                        let button = |ui: &mut egui::Ui, label: &str, enabled: bool| {
                            ui.add_enabled(
                                enabled,
                                egui::Button::new(
                                    egui::RichText::new(label).font(serif(SMALL)).color(
                                        egui::Color32::from_gray(if enabled { 230 } else { 100 }),
                                    ),
                                )
                                .min_size(egui::vec2(300.0, 44.0)),
                            )
                        };

                        if button(ui, "Lobby", true).clicked() {
                            chosen = Some(Destination::Lobby);
                        }
                        ui.add_space(12.0);
                        let game_enabled = game_snapshot.has_data;
                        let game_resp = button(ui, "Game", game_enabled);
                        if game_enabled && game_resp.clicked() {
                            chosen = Some(Destination::Mode(AppMode::Game));
                        } else if !game_enabled {
                            game_resp.on_disabled_hover_text(
                                "No game in progress — start one from the Lobby",
                            );
                        }
                    }
                },
            );
        });

    // Apply the pick and dismiss.
    if let Some(dest) = chosen {
        match dest {
            Destination::Lobby => {
                info!("menu: entering lobby");
                next_app_state.set(AppState::Lobby);
                next_app_mode.set(AppMode::Lobby);
            }
            Destination::Mode(mode) => {
                info!(?mode, "menu: entering mode");
                next_app_state.set(AppState::InGame);
                next_app_mode.set(mode);
            }
        }
    }
}

/// Where a start-menu button sends the player.
enum Destination {
    Lobby,
    Mode(AppMode),
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
        let job = emphasis_job("plain *italic* plain", font, color, false, 800.0);
        assert_eq!(job.sections.len(), 3);
        let italic_family = egui::FontFamily::Name("GaramondItalic".into());
        assert_ne!(job.sections[0].format.font_id.family, italic_family);
        assert_eq!(
            job.sections[1].format.font_id.family, italic_family,
            "the *starred* run uses the italic font family"
        );
        assert_ne!(job.sections[2].format.font_id.family, italic_family);
        assert!(job.sections.iter().all(|s| !s.format.italics));
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

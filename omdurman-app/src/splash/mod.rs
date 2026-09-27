//! Loading / start menu: a full-screen panel shown at app start with the game
//! title and a randomly-picked war-and-peace epigraph, while the (slow, ~30 MB)
//! board textures decode and upload in the background.
//!
//! Once loaded the panel transitions to [`AppMode::Menu`] — the persistent hub
//! for mode selection. Pressing **M** from any mode returns here. The menu
//! overlay is semi-transparent over play views (Game) and opaque over
//! full-screen UIs (Lobby/Editor).
//!
//! Layout (design 2a): the menu column on the dark left third, and on the
//! right two thirds a period map of the battlefield ([`map`]) that pans and
//! tilts slowly, fading into the backdrop. Narrow windows fall back to one
//! centred column over the full-bleed map. Every look-and-feel number is in
//! [`params`].

mod map;
mod params;

use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{AppMode, AppState};
pub(crate) use map::SplashMaps;
use params::*;

/// The curated quote pool, embedded at build time. Lives in `assets/quotes.md`
/// so it ships with the app and stays hand-curatable; parsed once on startup.
const QUOTES_MD: &str = include_str!("../../assets/quotes.md");

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
        .insert_resource(SplashMaps::new())
        .add_systems(Startup, map::load_splash_maps)
        .add_systems(
            Update,
            (
                update_loaded,
                map::prepare_splash_maps,
                map::animate_splash_maps,
            ),
        )
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
/// rendering the emphasized runs italic (and the rest at `base_italic`),
/// wrapped at `wrap_width` and aligned by `halign`.
fn emphasis_job(
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    base_italic: bool,
    wrap_width: f32,
    halign: egui::Align,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob {
        halign,
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
#[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
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
/// composition is opaque; on return it is semi-transparent so the game board
/// shows through.
#[allow(clippy::too_many_arguments)]
fn splash_ui(
    mut contexts: EguiContexts,
    splash_data: Option<Res<SplashData>>,
    maps: Res<SplashMaps>,
    app_state: Res<State<AppState>>,
    mode: Res<State<AppMode>>,
    progress: (Res<crate::TurnState>, Res<crate::game_record::GameRecorder>),
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

    // The map moves (`map::animate_splash_maps`): keep repainting.
    if maps.is_animating() {
        ctx.request_repaint();
    }

    let game_enabled = crate::game_in_progress(&progress.0, &progress.1);
    // A destination the player picked this frame, applied after the UI closure.
    let mut chosen: Option<Destination> = None;

    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("splash_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            // Fullscreen blocker: register the whole splash with egui's
            // hit-testing so map input (hover/click) never leaks through the
            // painted backdrop's blank areas.
            ui.interact(
                screen,
                egui::Id::new("splash_blocker"),
                egui::Sense::click(),
            );
            // Opaque during initial load and whenever there is no game behind
            // it; semi-transparent when returning to the menu from a game (so
            // the board shows through). It applies to the whole composition,
            // map included.
            let bg_alpha = if is_splash || !game_enabled {
                255u8
            } else {
                200u8
            };
            let narrow = screen.width() < NARROW_BREAKPOINT;
            match maps.texture(maps.show.current) {
                Some(_) => paint_map_backdrop(ui, screen, &maps, narrow, bg_alpha),
                None => {
                    ui.painter().rect_filled(
                        screen,
                        0.0,
                        crate::ui::palette::with_alpha_premultiplied(
                            crate::ui::palette::SPLASH_BACKDROP,
                            bg_alpha,
                        ),
                    );
                }
            }
            chosen = if narrow {
                centred_column(ui, screen, &splash_data, game_enabled)
            } else {
                menu_column(ui, screen, &splash_data, game_enabled)
            };
        });

    // Apply the pick and dismiss.
    if let Some(dest) = chosen {
        match dest {
            Destination::Lobby => {
                info!("menu: entering lobby");
                next_app_state.set(AppState::Lobby);
                next_app_mode.set(AppMode::Lobby);
            }
            Destination::Game => {
                info!("menu: entering game");
                crate::enter_game_view(&mut next_app_mode, &mut next_app_state);
            }
        }
    }
}

/// Paint the backdrop with the map: the showing map (crossfading into the next
/// one in turn), the backdrop fade over it (horizontal, or a uniform scrim on
/// narrow windows), the top and bottom scrims, and the credits. `bg_alpha` is
/// the composition's group opacity.
fn paint_map_backdrop(
    ui: &egui::Ui,
    screen: egui::Rect,
    maps: &SplashMaps,
    narrow: bool,
    bg_alpha: u8,
) {
    let backdrop = crate::ui::palette::SPLASH_BACKDROP;
    let composite = f32::from(bg_alpha) / 255.0;
    let region = if narrow {
        screen
    } else {
        map::map_region(screen)
    };
    // The backdrop's opacity over the map at a screen position: the fade is
    // drawn on top as its own layer, and the map is drawn at the opacity that
    // makes the pair composite as one group (`map::map_alpha`).
    let fade_at = |pos: egui::Pos2| {
        if narrow {
            NARROW_SCRIM_ALPHA
        } else {
            map::css_overlay_alpha(map::stops_at(
                &FADE_STOPS,
                (pos.x - screen.left()) / screen.width(),
            ))
        }
    };
    let painter = ui.painter();
    map::paint_maps(
        painter,
        region,
        maps,
        map::MapVariant::Title,
        backdrop,
        composite,
        fade_at,
    );
    // The fade spans the whole width: left of the map region it is the solid
    // backdrop of the menu column.
    if narrow {
        painter.rect_filled(
            screen,
            0.0,
            map::solid(backdrop, NARROW_SCRIM_ALPHA * composite),
        );
    } else {
        painter.add(map::gradient_mesh(
            screen,
            &FADE_STOPS,
            egui::Direction::LeftToRight,
            backdrop,
            composite,
        ));
    }
    painter.add(map::gradient_mesh(
        screen,
        &SCRIM_STOPS,
        egui::Direction::TopDown,
        backdrop,
        composite,
    ));
    paint_credits(painter, screen, maps, composite);
}

/// The lobby's floating panel for a UI column `column_w` points wide: centred,
/// [`LOBBY_PANEL_PAD`] wider than the column on each side and
/// [`LOBBY_PANEL_H_REL`] of the screen's height, never larger than the screen.
pub(crate) fn lobby_panel_rect(screen: egui::Rect, column_w: f32) -> egui::Rect {
    let size = egui::vec2(
        column_w + 2.0 * LOBBY_PANEL_PAD,
        screen.height() * LOBBY_PANEL_H_REL,
    );
    egui::Rect::from_center_size(screen.center(), size.min(screen.size()))
}

/// Where the lobby's UI goes inside its floating panel.
pub(crate) fn lobby_panel_content(panel: egui::Rect) -> egui::Rect {
    panel.shrink(LOBBY_PANEL_PAD)
}

/// The lobby's background: the showing map full-screen, more blurred and
/// moving more slowly than on the title screen, under a light scrim, and the
/// opaque dark `panel` that holds the lobby's UI, floating in a soft glow of
/// its own colour that fades it into the map. Before the maps are ready the screen behind
/// the panel is the plain dark backdrop.
pub(crate) fn paint_lobby_backdrop(
    painter: &egui::Painter,
    screen: egui::Rect,
    panel: egui::Rect,
    maps: &SplashMaps,
) {
    let dark = crate::ui::palette::NEUTRAL_BG;
    let radius = egui::CornerRadius::same(LOBBY_PANEL_RADIUS);
    let has_map = maps.texture(maps.show.current).is_some();
    if has_map {
        map::paint_maps(
            painter,
            screen,
            maps,
            map::MapVariant::Lobby,
            dark,
            1.0,
            |_| 0.0,
        );
        painter.rect_filled(screen, 0.0, map::solid(dark, LOBBY_SCRIM_ALPHA));
        painter.add(map::ring_gradient_mesh(
            panel,
            f32::from(LOBBY_PANEL_RADIUS),
            LOBBY_GLOW_REL * screen.height(),
            &LOBBY_GLOW_STOPS,
            dark,
        ));
    } else {
        painter.rect_filled(screen, 0.0, dark);
    }
    painter.rect(
        panel,
        radius,
        dark,
        egui::Stroke::new(1.0, crate::ui::palette::LOBBY_PANEL_BORDER),
        egui::StrokeKind::Inside,
    );
    if has_map {
        paint_credits(painter, screen, maps, 1.0);
    }
}

/// The showing map's credit, bottom-right. During a crossfade the credits hand
/// over in sequence -- the old one out in the first half, the new one in over
/// the second -- so two lines of different lengths never overlap.
fn paint_credits(painter: &egui::Painter, screen: egui::Rect, maps: &SplashMaps, opacity: f32) {
    let opacity = opacity * maps.show.fade_in();
    let fade = maps.show.fade().filter(|&(index, _)| maps.is_ready(index));
    let progress = fade.map_or(0.0, |(_, progress)| progress);
    let handover = |from: f32| ((progress - from) * 2.0).clamp(0.0, 1.0);
    paint_credit(
        painter,
        screen,
        MAPS[maps.show.current].credit,
        opacity * (1.0 - handover(0.0)),
    );
    if let Some((index, _)) = fade {
        paint_credit(painter, screen, MAPS[index].credit, opacity * handover(0.5));
    }
}

/// A map credit, bottom-right, at `opacity` (0..1), over a soft shadow.
fn paint_credit(painter: &egui::Painter, screen: egui::Rect, text: &str, opacity: f32) {
    if opacity <= 0.0 {
        return;
    }
    let color = map::solid(crate::ui::palette::SPLASH_CREDIT, opacity);
    let credit = painter.layout_no_wrap(
        text.to_string(),
        egui::FontId::new(CREDIT_SIZE, egui::FontFamily::Name("GaramondItalic".into())),
        color,
    );
    let pos = screen.max - CREDIT_MARGIN - credit.size();
    // A soft shadow (egui has none): dark copies on two rings plus the centre,
    // each faint enough that the overlapping core reaches the peak opacity.
    let rings = [0.0, 0.5, 1.0].map(|f| f * CREDIT_SHADOW_RADIUS);
    let copies = 1 + 2 * 8;
    let per_copy = 1.0 - (1.0 - CREDIT_SHADOW_ALPHA * opacity).powf(1.0 / copies as f32);
    let shade = map::solid(egui::Color32::BLACK, per_copy);
    for (ring, radius) in rings.into_iter().enumerate() {
        let steps = if ring == 0 { 1 } else { 8 };
        for step in 0..steps {
            let angle = std::f32::consts::TAU * step as f32 / steps as f32;
            let offset = CREDIT_SHADOW_OFFSET + radius * egui::vec2(angle.cos(), angle.sin());
            painter.galley_with_override_text_color(pos + offset, credit.clone(), shade);
        }
    }
    painter.galley(pos, credit, color);
}

/// The serif family (Merriweather, registered in `ui_plugin::fonts`) at `size`.
fn serif(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name("Garamond".into()))
}

/// The dark title-card button visuals. The splash is art-directed, not paper
/// chrome, so it keeps its own dark buttons rather than inheriting the global
/// paper skin (whose cream fills would wash out the light-grey text).
fn apply_splash_button_visuals(ui: &mut egui::Ui) {
    let w = &mut ui.visuals_mut().widgets;
    w.inactive.weak_bg_fill = crate::ui::palette::NEUTRAL_FILL;
    w.inactive.bg_fill = crate::ui::palette::NEUTRAL_FILL;
    w.inactive.bg_stroke = egui::Stroke::new(1.0_f32, crate::ui::palette::NEUTRAL_BORDER);
    w.hovered.weak_bg_fill = crate::ui::palette::NEUTRAL_FILL_RAISED;
    w.hovered.bg_fill = crate::ui::palette::NEUTRAL_FILL_RAISED;
    w.hovered.bg_stroke = egui::Stroke::new(1.0_f32, crate::ui::palette::NEUTRAL_BORDER_HOVER);
    w.active.weak_bg_fill = crate::ui::palette::NEUTRAL_FILL_PRESSED;
    w.active.bg_fill = crate::ui::palette::NEUTRAL_FILL_PRESSED;
    w.active.bg_stroke = egui::Stroke::new(1.0_f32, crate::ui::palette::NEUTRAL_BORDER_ACTIVE);
    for state in [&mut w.inactive, &mut w.hovered, &mut w.active] {
        state.corner_radius = egui::CornerRadius::same(BUTTON_RADIUS);
    }
}

/// The "Game" button's hover text while no game is in progress.
const NO_GAME_HINT: &str = "No game in progress — start one from the Lobby";

/// The menu column of the wide layout: kicker, two-line title, quote,
/// attribution and the entry buttons, left-aligned in the left third and
/// vertically centred on their measured height.
fn menu_column(
    ui: &mut egui::Ui,
    screen: egui::Rect,
    splash_data: &SplashData,
    game_enabled: bool,
) -> Option<Destination> {
    let painter = ui.painter().clone();
    let mut kicker = egui::text::LayoutJob::default();
    kicker.append(
        KICKER,
        0.0,
        egui::TextFormat {
            font_id: serif(KICKER_SIZE),
            color: crate::ui::palette::SPLASH_KICKER,
            extra_letter_spacing: KICKER_SIZE * KICKER_TRACKING,
            ..Default::default()
        },
    );
    let kicker = painter.layout_job(kicker);
    let mut title = egui::text::LayoutJob::default();
    title.append(
        "REMEMBER\nGORDON!",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::new(TITLE_SIZE, egui::FontFamily::Name("GaramondBold".into())),
            color: crate::ui::palette::SPLASH_TITLE,
            line_height: Some(TITLE_SIZE * TITLE_LINE_HEIGHT),
            ..Default::default()
        },
    );
    let title = painter.layout_job(title);
    let quote = splash_data.quote.as_ref().map(|quote| {
        let mut job = emphasis_job(
            &format!("\u{201c}{}\u{201d}", quote.text),
            serif(QUOTE_SIZE),
            crate::ui::palette::TEXT_STRONG,
            true,
            QUOTE_WRAP,
            egui::Align::LEFT,
        );
        for section in &mut job.sections {
            section.format.line_height = Some(QUOTE_SIZE * QUOTE_LINE_HEIGHT);
        }
        painter.layout_job(job)
    });
    let attribution = splash_data
        .quote
        .as_ref()
        .filter(|quote| !quote.attribution.is_empty())
        .map(|quote| {
            // Upright; `*Title*` runs render italic.
            painter.layout_job(emphasis_job(
                &format!("\u{2014} {}", quote.attribution),
                serif(ATTR_SIZE),
                crate::ui::palette::TEXT_MUTED,
                false,
                QUOTE_WRAP,
                egui::Align::LEFT,
            ))
        });
    let loading = painter.layout_no_wrap(
        "Loading\u{2026}".to_string(),
        serif(BUTTON_TEXT),
        crate::ui::palette::TEXT_FAINT,
    );

    // Measure the stack once (some quotes are long) and centre it. The
    // buttons' height is reserved while "Loading…" stands in for them, so the
    // column does not jump when they appear.
    let buttons_h = (2.0 * BUTTON_SIZE.y + GAP_BUTTONS).max(loading.size().y);
    let mut stack_h = kicker.size().y + GAP_KICKER_TITLE + title.size().y;
    if let Some(quote) = &quote {
        stack_h += GAP_TITLE_QUOTE + quote.size().y;
    }
    if let Some(attribution) = &attribution {
        stack_h += GAP_QUOTE_ATTR + attribution.size().y;
    }
    stack_h += GAP_ATTR_BUTTONS + buttons_h;

    let column = egui::Rect::from_min_size(
        egui::pos2(screen.left() + COL_LEFT_REL * screen.width(), screen.top()),
        egui::vec2(COL_W_REL * screen.width(), screen.height()),
    );
    let mut chosen = None;
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(column)
            .layout(egui::Layout::top_down(egui::Align::LEFT)),
        |ui| {
            ui.spacing_mut().item_spacing.y = 0.0; // only the explicit gaps
            ui.add_space(((screen.height() - stack_h) / 2.0).max(0.0));
            ui.label(kicker);
            ui.add_space(GAP_KICKER_TITLE);
            ui.label(title);
            if let Some(quote) = quote {
                ui.add_space(GAP_TITLE_QUOTE);
                ui.label(quote);
            }
            if let Some(attribution) = attribution {
                ui.add_space(GAP_QUOTE_ATTR);
                ui.label(attribution);
            }
            ui.add_space(GAP_ATTR_BUTTONS);
            if !splash_data.loaded {
                ui.label(loading);
                return;
            }
            apply_splash_button_visuals(ui);
            if menu_button(ui, "Lobby", true).clicked() {
                chosen = Some(Destination::Lobby);
            }
            ui.add_space(GAP_BUTTONS);
            let game = menu_button(ui, "Game", game_enabled);
            if game_enabled && game.clicked() {
                chosen = Some(Destination::Game);
            } else if !game_enabled {
                game.on_disabled_hover_text(NO_GAME_HINT);
            }
        },
    );
    chosen
}

/// A menu-column button with its label left-aligned (egui buttons centre
/// theirs): painted by hand inside an allocated [`BUTTON_SIZE`] rect, with the
/// splash button visuals when enabled.
fn menu_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled_ui(enabled, |ui| {
        let (rect, response) = ui.allocate_exact_size(BUTTON_SIZE, egui::Sense::click());
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
        if ui.is_rect_visible(rect) {
            let (fill, stroke, text) = if enabled {
                let visuals = ui.style().interact(&response);
                (
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    crate::ui::palette::TEXT_STRONG,
                )
            } else {
                (
                    crate::ui::palette::SPLASH_BUTTON_DISABLED_FILL,
                    egui::Stroke::new(1.0, crate::ui::palette::SPLASH_BUTTON_DISABLED_BORDER),
                    crate::ui::palette::TEXT_DISABLED,
                )
            };
            let painter = ui.painter();
            painter.rect(
                rect,
                egui::CornerRadius::same(BUTTON_RADIUS),
                fill,
                stroke,
                egui::StrokeKind::Inside,
            );
            let galley = painter.layout_no_wrap(label.to_string(), serif(BUTTON_TEXT), text);
            let pos = egui::pos2(
                rect.left() + BUTTON_PAD_X,
                rect.center().y - galley.size().y / 2.0,
            );
            painter.galley(pos, galley, text);
        }
        response
    })
    .inner
}

/// The narrow-window layout: one centred column (title, quote, attribution,
/// buttons) over the full-bleed map.
fn centred_column(
    ui: &mut egui::Ui,
    screen: egui::Rect,
    splash_data: &SplashData,
    game_enabled: bool,
) -> Option<Destination> {
    let mut chosen = None;
    ui.allocate_ui_with_layout(
        screen.size(),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.add_space(screen.height() * NARROW_TOP_REL);
            ui.label(
                egui::RichText::new("REMEMBER GORDON!")
                    .font(serif(NARROW_TITLE_SIZE))
                    .color(crate::ui::palette::SPLASH_TITLE),
            );
            ui.add_space(NARROW_GAP_LARGE);

            if let Some(quote) = &splash_data.quote {
                // Shared wrap width for the quote block. The job wraps itself
                // at this width (word boundaries only), so the ui container
                // must be at least this wide or it would clip.
                let wrap_w = (screen.width() * NARROW_WRAP_REL).min(NARROW_WRAP_MAX);
                ui.set_max_width(wrap_w);
                // Quote body is italic throughout; `*...*` runs stay italic
                // too (no visible toggle), so we just wrap it.
                ui.label(emphasis_job(
                    &format!("\u{201c}{}\u{201d}", quote.text),
                    serif(NARROW_QUOTE_SIZE),
                    crate::ui::palette::TEXT_STRONG,
                    true,
                    wrap_w,
                    egui::Align::Center,
                ));
                if !quote.attribution.is_empty() {
                    ui.add_space(NARROW_GAP_ATTR);
                    // Attribution is upright; `*Title*` runs render italic.
                    ui.label(emphasis_job(
                        &format!("\u{2014} {}", quote.attribution),
                        serif(NARROW_SMALL_SIZE),
                        crate::ui::palette::TEXT_MUTED,
                        false,
                        wrap_w,
                        egui::Align::Center,
                    ));
                }
            }

            ui.add_space(NARROW_GAP_LARGE);
            if !splash_data.loaded {
                ui.label(
                    egui::RichText::new("Loading\u{2026}")
                        .font(serif(NARROW_SMALL_SIZE))
                        .color(crate::ui::palette::TEXT_FAINT),
                );
                return;
            }
            // Entry buttons, revealed once the board texture is ready.
            apply_splash_button_visuals(ui);
            let button = |ui: &mut egui::Ui, label: &str, enabled: bool| {
                ui.add_enabled(
                    enabled,
                    egui::Button::new(
                        egui::RichText::new(label)
                            .font(serif(NARROW_SMALL_SIZE))
                            .color(if enabled {
                                crate::ui::palette::TEXT_STRONG
                            } else {
                                crate::ui::palette::TEXT_DISABLED
                            }),
                    )
                    .min_size(BUTTON_SIZE),
                )
            };
            if button(ui, "Lobby", true).clicked() {
                chosen = Some(Destination::Lobby);
            }
            ui.add_space(NARROW_GAP_BUTTONS);
            let game_resp = button(ui, "Game", game_enabled);
            if game_enabled && game_resp.clicked() {
                chosen = Some(Destination::Game);
            } else if !game_enabled {
                game_resp.on_disabled_hover_text(NO_GAME_HINT);
            }
        },
    );
    chosen
}

/// Where a start-menu button sends the player.
enum Destination {
    Lobby,
    Game,
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
        let job = emphasis_job(
            "plain *italic* plain",
            font,
            color,
            false,
            800.0,
            egui::Align::Center,
        );
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

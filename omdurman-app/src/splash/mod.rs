//! Loading / start menu: a full-screen screen shown at app start with the game
//! title and a randomly-picked war-and-peace epigraph, while the (slow, ~30 MB)
//! board textures decode and upload in the background.
//!
//! Once loaded the screen transitions to [`AppMode::Menu`] — the persistent
//! hub for mode selection. Pressing **M** from any mode returns here. The menu
//! is semi-transparent over a game in progress and opaque otherwise.
//!
//! Layout (design 2a): the menu column on the dark left third, and on the
//! right two thirds a period map of the battlefield ([`map`]) that pans and
//! tilts slowly, fading into the backdrop. Narrow windows fall back to one
//! centred column over the full-bleed map. The lobby shows the same maps
//! behind its panel. Both are native Bevy UI ([`screen`]) over a GPU backdrop
//! ([`backdrop`]); every look-and-feel number is in [`params`].

mod backdrop;
mod map;
mod params;
mod screen;
mod tuning;

use bevy::asset::{LoadState, load_internal_asset};
use bevy::prelude::*;
use bevy::ui_render::prelude::UiMaterialPlugin;
use bevy_egui::egui;

use crate::{AppMode, AppState};
use backdrop::BackdropMaterial;
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
    /// entry buttons (before that the screen just shows the quote + "Loading…").
    pub loaded: bool,
}

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            backdrop::BACKDROP_SHADER,
            "backdrop.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins(UiMaterialPlugin::<BackdropMaterial>::default())
            .insert_resource(SplashData {
                quote: pick_quote(),
                loaded: false,
            })
            .insert_resource(SplashMaps::from_env())
            .init_resource::<tuning::SplashTuning>()
            .add_systems(
                Startup,
                (
                    map::load_splash_maps,
                    (screen::load_fonts, screen::spawn_splash_screen).chain(),
                    tuning::spawn_tuning_pane,
                ),
            )
            .add_systems(
                Update,
                (
                    update_loaded,
                    map::prepare_splash_maps,
                    map::animate_splash_maps,
                    screen::menu_buttons,
                    screen::sync_splash_screen,
                    screen::sync_credits,
                    tuning::show_tuning_pane,
                    tuning::apply_tuned_fps,
                )
                    .chain(),
            );
    }
}

/// A palette colour (the palette is shared with the egui screens).
fn palette_color(color: egui::Color32) -> Color {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    Color::srgba_u8(r, g, b, a)
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

/// Pick one quote at random from the pool, or `None` if the pool is empty.
/// `OMDURMAN_SPLASH_QUOTE=<index>` pins one (dev affordance, with
/// `OMDURMAN_SPLASH_FREEZE`, for screenshots that compare across builds).
fn pick_quote() -> Option<Quote> {
    let quotes = parse_quotes(QUOTES_MD);
    if quotes.is_empty() {
        warn!("splash: no quotes parsed from assets/quotes.md");
        return None;
    }
    let pinned = std::env::var("OMDURMAN_SPLASH_QUOTE")
        .ok()
        .and_then(|index| quotes.get(index.trim().parse::<usize>().ok()?));
    use rand::seq::IndexedRandom;
    pinned.or_else(|| quotes.choose(&mut rand::rng())).cloned()
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

/// The lobby's floating panel for a UI column `column_w` points wide on a
/// `screen` that large: centred, [`LOBBY_PANEL_PAD`] wider than the column on
/// each side and [`LOBBY_PANEL_H_REL`] of the screen's height, never larger
/// than the screen.
pub(crate) fn lobby_panel_rect(screen: Vec2, column_w: f32) -> Rect {
    let size = Vec2::new(
        column_w + 2.0 * LOBBY_PANEL_PAD,
        screen.y * LOBBY_PANEL_H_REL,
    )
    .min(screen);
    Rect::from_center_size(screen / 2.0, size)
}

/// Where the lobby's UI goes inside its floating panel.
pub(crate) fn lobby_panel_content(panel: Rect) -> Rect {
    panel.inflate(-LOBBY_PANEL_PAD)
}

/// Where a start-menu button sends the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

//! The reference sheets: a slide-in overlay card holding the game's coarse
//! chart scans (combat results table, terrain effects, campaign timing, order
//! of appearance) plus the rulebook. It is a card laid *on* the table -- an
//! `egui::Area` that slides over the right edge of the board, so the board never
//! reflows. When closed a slim "CHARTS" tab peeks at the right edge.
//!
//! This module owns the shell (tabs, zoom/pan, hotkey `C`); the Rulebook tab
//! is rendered by [`crate::rulebook`].

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiUserTextures, egui};

/// The chart tabs, in printed index order. `Rulebook` is text (see the rulebook
/// pass); the rest are scan textures.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChartTab {
    Crt,
    Terrain,
    Timing,
    Arrivals,
    Rulebook,
}

impl ChartTab {
    const ALL: [ChartTab; 5] = [
        ChartTab::Crt,
        ChartTab::Terrain,
        ChartTab::Timing,
        ChartTab::Arrivals,
        ChartTab::Rulebook,
    ];

    fn label(self) -> &'static str {
        match self {
            ChartTab::Crt => "CRT",
            ChartTab::Terrain => "Terrain",
            ChartTab::Timing => "Timing",
            ChartTab::Arrivals => "Arrivals",
            ChartTab::Rulebook => "Rulebook",
        }
    }

    /// The scan asset for a texture tab, or `None` for the text rulebook.
    fn asset_path(self) -> Option<&'static str> {
        match self {
            ChartTab::Crt => Some("charts/combat_results_table.webp"),
            ChartTab::Terrain => Some("charts/terrain_effects_chart.webp"),
            ChartTab::Timing => Some("charts/campaign_timing.webp"),
            ChartTab::Arrivals => Some("charts/order_of_appearance.webp"),
            ChartTab::Rulebook => None,
        }
    }

    /// Whether this sheet belongs to `scenario` (`None`: no game running --
    /// every sheet is offered). The campaign turn record (Timing) covers the
    /// Omdurman scenarios, not FALL OF KHARTOUM (its turn track is the §9.33
    /// strip in Game control); the order of appearance (Arrivals) is the
    /// Campaign game's alone (§9.112/§9.113).
    fn applies_to(self, scenario: Option<omdurman_types::Scenario>) -> bool {
        use omdurman_types::Scenario;
        match (self, scenario) {
            (_, None) => true,
            (ChartTab::Timing, Some(s)) => s != Scenario::FallOfKhartoum,
            (ChartTab::Arrivals, Some(s)) => s == Scenario::Campaign,
            _ => true,
        }
    }
}

/// A loaded scan: the Bevy image handle and, once registered with egui, its
/// texture id and pixel size. Registration is deferred until the asset finishes
/// loading (its size is unknown before then).
struct ChartTexture {
    handle: Handle<Image>,
    egui_id: Option<egui::TextureId>,
    size: Option<egui::Vec2>,
}

/// Per-tab pan/zoom, so switching tabs preserves each sheet's framing.
#[derive(Clone, Copy)]
struct View {
    /// Zoom multiplier over fit-to-width. 1.0 == fit width.
    zoom: f32,
    /// Pan offset in points from the fitted top-left.
    pan: egui::Vec2,
}

impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
        }
    }
}

#[derive(Resource)]
pub struct ChartSheet {
    open: bool,
    active: ChartTab,
    /// Scan textures keyed by tab (rulebook has no entry).
    textures: Vec<(ChartTab, ChartTexture)>,
    views: [(ChartTab, View); 5],
}

impl ChartSheet {
    /// Whether the sheet is open (it takes Esc before the board does).
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    fn texture_mut(&mut self, tab: ChartTab) -> Option<&mut ChartTexture> {
        self.textures
            .iter_mut()
            .find(|(t, _)| *t == tab)
            .map(|(_, tex)| tex)
    }

    fn view_mut(&mut self, tab: ChartTab) -> &mut View {
        &mut self
            .views
            .iter_mut()
            .find(|(t, _)| *t == tab)
            .expect("every tab has a view")
            .1
    }
}

pub struct ChartsPlugin;

impl Plugin for ChartsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::rulebook::Rulebook>()
            .add_systems(Startup, load_chart_textures)
            // Texture registration touches `EguiUserTextures`, which the egui
            // context pass also accesses internally -- doing both in one system
            // that holds `EguiContexts` is a conflicting `ResMut` borrow (B0002).
            // Register in a plain `Update` system, render in the egui pass.
            .add_systems(Update, register_chart_textures)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    open_rulebook_on_request,
                    chart_sheet_ui.run_if(charts_visible),
                )
                    .chain(),
            );
    }
}

/// A `§` link anywhere in the UI opens the manual at its section: take the
/// link clicked since last frame ([`crate::rulebook::request_open`]) and
/// open the sheet on the Rulebook tab for any pending deep link -- a closed
/// sheet used to swallow them. Runs everywhere the sheet may show, and in
/// the lobby (whose optional rules cite §10).
fn open_rulebook_on_request(
    mut contexts: EguiContexts,
    mut sheet: Option<ResMut<ChartSheet>>,
    mut rulebook: ResMut<crate::rulebook::Rulebook>,
) {
    if let Ok(ctx) = contexts.ctx_mut()
        && let Some(number) = crate::rulebook::take_requested_section(ctx)
    {
        crate::rulebook::request_section(&mut rulebook, &number);
    }
    if rulebook.scroll_to.is_some()
        && let Some(sheet) = sheet.as_mut()
    {
        sheet.open = true;
        sheet.active = ChartTab::Rulebook;
    }
}

/// Where the chart sheet may appear:
///   * play map views -- Game, while actually in a game or reviewing
///     a recording (not the lobby / connecting screen).
///
/// It is hidden everywhere else, and while the start screen is up (the default
/// mode/state would otherwise let it draw beneath the splash).
fn charts_visible(
    mode: Res<State<crate::AppMode>>,
    app_state: Res<State<crate::AppState>>,
    sheet: Option<Res<ChartSheet>>,
) -> bool {
    if *app_state.get() == crate::AppState::Splash {
        return false;
    }
    match **mode {
        crate::AppMode::Game => matches!(
            **app_state,
            crate::AppState::InGame | crate::AppState::Spectating
        ),
        // The lobby shows the sheet only while a § link has it open (no peek
        // tab over the lobby).
        crate::AppMode::Lobby => sheet.is_some_and(|s| s.open),
        crate::AppMode::Menu => false,
    }
}

/// Kick off loading the scan assets and register the resource. Textures are
/// registered with egui lazily (once loaded) in `chart_sheet_ui`.
fn load_chart_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    let textures = ChartTab::ALL
        .into_iter()
        .filter_map(|tab| {
            tab.asset_path().map(|path| {
                (
                    tab,
                    ChartTexture {
                        handle: asset_server.load(path),
                        egui_id: None,
                        size: None,
                    },
                )
            })
        })
        .collect();
    // Dev: start opened on a given tab for headless screenshots
    // (OMDURMAN_CHARTS=crt|terrain|timing|arrivals|rulebook). Inert otherwise.
    let (open, active) = match std::env::var("OMDURMAN_CHARTS").ok().as_deref() {
        Some("crt") => (true, ChartTab::Crt),
        Some("terrain") => (true, ChartTab::Terrain),
        Some("timing") => (true, ChartTab::Timing),
        Some("arrivals") => (true, ChartTab::Arrivals),
        Some("rulebook") => (true, ChartTab::Rulebook),
        _ => (false, ChartTab::Crt),
    };
    commands.insert_resource(ChartSheet {
        open,
        active,
        textures,
        views: ChartTab::ALL.map(|t| (t, View::default())),
    });
}

/// Slim tab width when the sheet is closed; open sheet width fraction of window.
const PEEK_W: f32 = 28.0;
const OPEN_FRAC: f32 = 0.40;
const OPEN_MIN_W: f32 = 480.0;

/// Register newly-loaded scan textures with egui once their pixel size is
/// known. Runs outside the egui context pass (see the plugin note on B0002).
fn register_chart_textures(
    sheet: Option<ResMut<ChartSheet>>,
    mut user_textures: ResMut<EguiUserTextures>,
    images: Res<Assets<Image>>,
) {
    let Some(mut sheet) = sheet else { return };
    for (_, tex) in sheet.textures.iter_mut() {
        if tex.egui_id.is_none()
            && let Some(image) = images.get(&tex.handle)
        {
            let dims = image.size();
            tex.size = Some(egui::vec2(dims.x as f32, dims.y as f32));
            tex.egui_id = Some(
                user_textures.add_image(bevy_egui::EguiTextureHandle::Strong(tex.handle.clone())),
            );
        }
    }
}

/// Bundle of the top-level mode plus the keyboard input so [`chart_sheet_ui`]
/// stays under clippy's argument limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct ChartView<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    /// Typing into an egui field (e.g. the rulebook search) must not toggle
    /// the sheet — see [`crate::hotkeys::keyboard_free`].
    focus: Res<'w, crate::hotkeys::EguiKeyboardFocus>,
    /// The running game, whose scenario decides which sheets are offered.
    game_state: Option<Res<'w, crate::GameStateResource>>,
}

pub(crate) fn chart_sheet_ui(
    mut contexts: EguiContexts,
    mut sheet: Option<ResMut<ChartSheet>>,
    view: ChartView,
    mut rulebook: ResMut<crate::rulebook::Rulebook>,
    time: Res<Time>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let ChartView {
        keys,
        focus,
        game_state,
    } = view;
    let scenario = game_state.as_deref().map(|gs| gs.0.scenario);
    let Some(sheet) = sheet.as_mut() else { return };
    // A sheet this scenario doesn't use (the tab left open from another
    // game) falls back to the CRT.
    if !sheet.active.applies_to(scenario) {
        sheet.active = ChartTab::Crt;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Hotkey: C toggles, Esc closes (Esc goes to an open sheet before the
    // board's cancel; see `hotkeys::command_for_keys`). Both are ignored
    // while a text field has the keyboard.
    let keys_free = !focus.0;
    if keys_free && keys.just_pressed(KeyCode::KeyC) {
        sheet.open = !sheet.open;
    }
    if keys_free && sheet.open && keys.just_pressed(KeyCode::Escape) {
        sheet.open = false;
    }

    let screen = ctx.content_rect();
    // Anchor to the window's right edge, below the top bar (the sheet used to
    // run from y=0 under the toolbar). The sheet and its peek tab sit at the
    // right edge and never overlap the left rail.
    let right = ctx.content_rect().right().min(screen.max.x);

    // Don't lay anything out until there is a sane amount of room. Early frames
    // (before the window is maximized) report a tiny rect; constraining against
    // that produced a bad initial state.
    if right - screen.min.x < OPEN_MIN_W + PEEK_W || screen.height() < 2.0 {
        return;
    }
    let open_w = (screen.width() * OPEN_FRAC).max(OPEN_MIN_W);

    // Card left edge. Closed -> only PEEK_W shows past the right edge; open ->
    // the full card is on-screen. Positioned directly (no slide animation for
    // now: driving `animate_value_with_time` + `request_repaint` every frame
    // spun the render loop and froze the window before the first stable frame).
    let x = if sheet.open {
        right - open_w
    } else {
        right - PEEK_W
    };

    let card = egui::Rect::from_min_max(
        egui::pos2(x, screen.min.y + layout.top_bar_height),
        egui::pos2(right, screen.max.y),
    );
    // Right-anchored cards (combat card, optional-rule setup, desertion)
    // shift left by this so they clear the sheet / peek tab.
    layout.right_inset = right - x;

    egui::Area::new(egui::Id::new("chart_sheet"))
        .order(egui::Order::Foreground)
        .fixed_pos(card.min)
        .constrain_to(card)
        .show(ctx, |ui| {
            ui.set_clip_rect(card);
            // An Area sizes to its content by default -- an unbounded width lets
            // the scan blow up to its native size. Pin the ui to the card.
            ui.set_width(card.width());
            ui.set_max_width(card.width());
            ui.set_height(card.height());
            // Card-sized blocker: clicks/hovers on the sheet must not fall
            // through to the map.
            ui.interact(
                card,
                egui::Id::new("chart_sheet_blocker"),
                egui::Sense::click(),
            );

            // Card-on-table look: a hard offset shadow so it reads as sitting on
            // the board. (One deliberate shadow; the rest of the chrome has none.)
            let shadow = egui::Rect::from_min_max(
                card.min + egui::vec2(-4.0, 4.0),
                egui::pos2(card.min.x, card.max.y) + egui::vec2(-4.0, 4.0),
            );
            ui.painter()
                .rect_filled(shadow, 0.0, crate::ui::palette::DROP_SHADOW);

            const MARGIN: f32 = 8.0;
            egui::Frame::new()
                .fill(crate::ui::palette::NEUTRAL_BG)
                .stroke(egui::Stroke::new(
                    2.0_f32,
                    crate::ui::palette::NEUTRAL_BORDER,
                ))
                .inner_margin(egui::Margin::same(MARGIN as i8))
                .show(ui, |ui| {
                    // Fill the card minus the frame's own margins on both sides;
                    // using the full card size here pushed content 2*MARGIN wider
                    // than the card, clipping the right edge (the close button).
                    ui.set_min_size(card.size() - egui::vec2(2.0 * MARGIN, 2.0 * MARGIN));
                    if sheet.open {
                        draw_open_sheet(ui, sheet, &mut rulebook, time.delta_secs(), scenario);
                    } else {
                        draw_peek_tab(ui, sheet);
                    }
                });
        });
}

/// Paint `text` vertically (one character per line) centred at `pos`.
fn vertical_label(ui: &egui::Ui, pos: egui::Pos2, text: &str, font: egui::FontId) {
    let vertical: String = text
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    ui.painter().text(
        pos,
        egui::Align2::CENTER_CENTER,
        vertical,
        font,
        crate::ui::palette::TEXT_STRONG,
    );
}

/// The closed state: a slim vertical "CHARTS" strip that toggles the sheet.
/// Filled brighter than the board so it reads as a clickable index tab, and
/// clickable over its whole area (not just the glyphs).
fn draw_peek_tab(ui: &mut egui::Ui, sheet: &mut ChartSheet) {
    let rect = ui.max_rect();
    let resp = ui.allocate_rect(rect, egui::Sense::click());
    let fill = if resp.hovered() {
        crate::ui::palette::NEUTRAL_FILL_PRESSED
    } else {
        crate::ui::palette::NEUTRAL_FILL_RAISED
    };
    ui.painter().rect_filled(rect, 0.0, fill);

    // egui has no vertical text; stack the glyphs down the strip.
    vertical_label(ui, rect.center(), "CHARTS", egui::FontId::monospace(13.0));
    if resp.clicked() {
        sheet.open = true;
    }
}

/// The open state: index tabs across the top, then the active tab's content.
fn draw_open_sheet(
    ui: &mut egui::Ui,
    sheet: &mut ChartSheet,
    rulebook: &mut crate::rulebook::Rulebook,
    dt: f32,
    scenario: Option<omdurman_types::Scenario>,
) {
    ui.horizontal(|ui| {
        for tab in ChartTab::ALL.into_iter().filter(|t| t.applies_to(scenario)) {
            if ui
                .add(egui::Button::selectable(sheet.active == tab, tab.label()))
                .clicked()
            {
                sheet.active = tab;
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("hide").clicked() {
                sheet.open = false;
            }
        });
    });
    ui.separator();

    let active = sheet.active;
    if active == ChartTab::Rulebook {
        // A clicked §-reference re-targets the rulebook to that section.
        if let Some(number) = crate::rulebook::draw_rulebook(ui, rulebook, dt) {
            crate::rulebook::request_section(rulebook, &number);
        }
        return;
    }

    // Scan tab: show the texture fit-to-width with scroll-zoom and drag-pan.
    let (tex_id, tex_size) = match sheet.texture_mut(active) {
        Some(ChartTexture {
            egui_id: Some(id),
            size: Some(size),
            ..
        }) => (*id, *size),
        _ => {
            ui.label("Loading…");
            return;
        }
    };

    let avail = ui.available_size();
    // Fit-to-width base scale, times the per-tab zoom.
    let view = *sheet.view_mut(active);
    let base = (avail.x / tex_size.x).max(0.01);
    let scale = base * view.zoom;
    let draw_size = tex_size * scale;

    let (rect, resp) = ui.allocate_exact_size(avail, egui::Sense::click_and_drag());
    ui.set_clip_rect(rect);

    // Scroll to zoom (about the cursor), drag to pan, double-click resets.
    let mut view = view;
    if resp.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            view.zoom = (view.zoom * (1.0 + scroll * 0.001)).clamp(1.0, 6.0);
        }
    }
    if resp.dragged() {
        view.pan += resp.drag_delta();
    }
    if resp.double_clicked() {
        view = View::default();
    }
    *sheet.view_mut(active) = view;

    let top_left = rect.min + view.pan;
    let image_rect = egui::Rect::from_min_size(top_left, draw_size);
    egui::Image::new(egui::load::SizedTexture::new(tex_id, draw_size)).paint_at(ui, image_rect);
}

#[cfg(test)]
mod tests {
    use super::ChartTab;
    use omdurman_types::Scenario;

    /// The campaign turn record and order of appearance are not offered in
    /// FALL OF KHARTOUM (its turn track is in Game control); the order of
    /// appearance is the Campaign game's alone.
    #[test]
    fn sheets_follow_the_scenario() {
        let tabs = |s| {
            ChartTab::ALL
                .into_iter()
                .filter(|t| t.applies_to(s))
                .map(ChartTab::label)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            tabs(Some(Scenario::FallOfKhartoum)),
            ["CRT", "Terrain", "Rulebook"]
        );
        assert_eq!(
            tabs(Some(Scenario::Historical)),
            ["CRT", "Terrain", "Timing", "Rulebook"]
        );
        assert_eq!(tabs(Some(Scenario::Campaign)).len(), 5);
        assert_eq!(tabs(None).len(), 5);
    }
}

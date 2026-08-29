//! The reference sheets: a slide-in overlay card holding the game's coarse
//! chart scans (combat results table, terrain effects, campaign timing, order
//! of appearance) plus the rulebook. It is a card laid *on* the table -- an
//! `egui::Area` that slides over the right edge of the board, so the board never
//! reflows. When closed a slim "CHARTS" tab peeks at the right edge.
//!
//! This module owns the shell (tabs, zoom/pan, hotkey `C`), the spotlight-dim
//! highlight (§decision 4), the in-app box calibrator on the editor Charts tab,
//! and the gentle `ChartSheetRequest` staging (§decision 3). The Rulebook tab is
//! rendered by [`crate::rulebook`]. The docked turn-track strip is still to come
//! (it needs the timing scan calibrated first).

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

    /// Stable id used as the key of the chart-scan table index, or `None`
    /// for the text rulebook (which has no calibrated boxes).
    fn band_id(self) -> Option<&'static str> {
        match self {
            ChartTab::Crt => Some("crt"),
            ChartTab::Terrain => Some("terrain"),
            ChartTab::Timing => Some("timing"),
            ChartTab::Arrivals => Some("arrivals"),
            ChartTab::Rulebook => None,
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

/// A spotlight target within one chart: a table plus an optional row and/or
/// column to keep bright. When both are set, their intersection cell is the
/// brightest cut-out (and the full row + full column are also lit); a lone row
/// or column lights that whole band. Indices are into the table's code-defined
/// `rows`/`cols`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ChartHighlight {
    pub chart: ChartTab,
    pub table: usize,
    pub row: Option<usize>,
    pub col: Option<usize>,
}

/// A contextual request to draw attention to a chart region (fire declared,
/// unit hovered, turn changed, ...). Handling is *gentle* (§decision 3): the
/// sheet never opens itself. If closed, the peek tab pulses and the tab +
/// highlight are staged; opening within the staging window lands on that tab
/// with the highlight applied. If already open, the target tab gets a small
/// tick instead. Any system may send this; `charts.rs` is the only consumer.
#[derive(Message, Clone, Copy)]
pub struct ChartSheetRequest {
    pub tab: ChartTab,
    pub highlight: Option<ChartHighlight>,
}

/// How long a staged request stays live after arriving (§decision 3: ~10 s).
const STAGE_SECS: f32 = 10.0;

#[derive(Resource)]
pub struct ChartSheet {
    open: bool,
    active: ChartTab,
    /// Scan textures keyed by tab (rulebook has no entry).
    textures: Vec<(ChartTab, ChartTexture)>,
    views: [(ChartTab, View); 5],
    /// Active spotlight, if any -- dims the scan except the lit region.
    highlight: Option<ChartHighlight>,
    /// A staged contextual request (§decision 3): the tab+highlight to apply
    /// when the player opens the sheet, and the seconds left before it expires.
    staged: Option<(ChartTab, Option<ChartHighlight>, f32)>,
    /// Seconds left on the peek-tab attention pulse (counts down to 0).
    pulse: f32,
}

impl ChartSheet {
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

    /// Open the sheet, consuming any live staged request so the player lands on
    /// the staged tab with its highlight already applied (§decision 3).
    fn open_and_consume_stage(&mut self) {
        self.open = true;
        self.pulse = 0.0;
        if let Some((tab, hl, _)) = self.staged.take() {
            self.active = tab;
            self.highlight = hl;
        }
    }
}

pub struct ChartsPlugin;

impl Plugin for ChartsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::rulebook::Rulebook>()
            .add_message::<ChartSheetRequest>()
            .add_systems(Startup, load_chart_textures)
            // Texture registration touches `EguiUserTextures`, which the egui
            // context pass also accesses internally -- doing both in one system
            // that holds `EguiContexts` is a conflicting `ResMut` borrow (B0002).
            // Register in a plain `Update` system, render in the egui pass.
            .add_systems(Update, (register_chart_textures, handle_chart_requests))
            .add_systems(
                EguiPrimaryContextPass,
                chart_sheet_ui.run_if(charts_visible),
            );
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
) -> bool {
    if *app_state.get() == crate::AppState::Splash {
        return false;
    }
    match **mode {
        crate::AppMode::Game => matches!(
            **app_state,
            crate::AppState::InGame | crate::AppState::Spectating
        ),
        crate::AppMode::Menu | crate::AppMode::Lobby => false,
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
    // Dev: seed a demo spotlight for headless verification
    // (OMDURMAN_CHARTS_HL=row,col on the active chart's table 0). Inert otherwise.
    let highlight = std::env::var("OMDURMAN_CHARTS_HL").ok().map(|s| {
        let mut parts = s.split(',');
        let row = parts.next().and_then(|p| p.trim().parse::<usize>().ok());
        let col = parts.next().and_then(|p| p.trim().parse::<usize>().ok());
        ChartHighlight {
            chart: active,
            table: 0,
            row,
            col,
        }
    });
    commands.insert_resource(ChartSheet {
        open,
        active,
        textures,
        views: ChartTab::ALL.map(|t| (t, View::default())),
        highlight,
        staged: None,
        pulse: 0.0,
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

/// Consume [`ChartSheetRequest`]s the gentle way (§decision 3). The sheet never
/// opens itself: if it is closed, the latest request stages its tab+highlight
/// and starts the peek-tab pulse; if it is already open, the request switches to
/// that tab and applies the highlight immediately (the player is already
/// looking). Staged requests and the pulse decay over time.
fn handle_chart_requests(
    time: Res<Time>,
    mut reader: MessageReader<ChartSheetRequest>,
    sheet: Option<ResMut<ChartSheet>>,
) {
    let Some(mut sheet) = sheet else {
        reader.clear();
        return;
    };
    let dt = time.delta_secs();

    // Decay the pulse and expire a stale staged request.
    sheet.pulse = (sheet.pulse - dt).max(0.0);
    if let Some((_, _, ref mut secs)) = sheet.staged {
        *secs -= dt;
        if *secs <= 0.0 {
            sheet.staged = None;
        }
    }

    for req in reader.read() {
        if sheet.open {
            // Already open: the player is looking, so switch + apply directly.
            sheet.active = req.tab;
            sheet.highlight = req.highlight;
        } else {
            // Closed: stage it and pulse the peek tab. Don't open.
            sheet.staged = Some((req.tab, req.highlight, STAGE_SECS));
            sheet.pulse = 2.4; // ~3 slow pulses (see draw_peek_tab)
        }
    }
}

/// Bundle of the top-level mode plus the keyboard input so [`chart_sheet_ui`]
/// stays under clippy's argument limit.
#[derive(bevy::ecs::system::SystemParam)]
struct ChartView<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
}

fn chart_sheet_ui(
    mut contexts: EguiContexts,
    mut sheet: Option<ResMut<ChartSheet>>,
    view: ChartView,
    mut rulebook: ResMut<crate::rulebook::Rulebook>,
    time: Res<Time>,
) {
    let ChartView { keys } = view;
    let Some(sheet) = sheet.as_mut() else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Hotkey: C toggles, Esc closes.
    if keys.just_pressed(KeyCode::KeyC) {
        if sheet.open {
            sheet.open = false;
        } else {
            sheet.open_and_consume_stage();
        }
    }
    if sheet.open && keys.just_pressed(KeyCode::Escape) {
        sheet.open = false;
    }

    let screen = ctx.content_rect();
    // Anchor to the right edge of the remaining content -- i.e. the window's
    // right edge, clear of the play mode's left sidebar (that Panel has
    // already reserved its space by now, so `content_rect` excludes it). The
    // sheet and its peek tab sit at the right edge and never overlap it.
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

    let card =
        egui::Rect::from_min_max(egui::pos2(x, screen.min.y), egui::pos2(right, screen.max.y));

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

            // Card-on-table look: a hard offset shadow so it reads as sitting on
            // the board. (One deliberate shadow; the rest of the chrome has none.)
            let shadow = egui::Rect::from_min_max(
                card.min + egui::vec2(-4.0, 4.0),
                egui::pos2(card.min.x, card.max.y) + egui::vec2(-4.0, 4.0),
            );
            ui.painter()
                .rect_filled(shadow, 0.0, egui::Color32::from_black_alpha(46));

            const MARGIN: f32 = 8.0;
            egui::Frame::new()
                .fill(egui::Color32::from_gray(28))
                .stroke(egui::Stroke::new(2.0_f32, egui::Color32::from_gray(90)))
                .inner_margin(egui::Margin::same(MARGIN as i8))
                .show(ui, |ui| {
                    // Fill the card minus the frame's own margins on both sides;
                    // using the full card size here pushed content 2*MARGIN wider
                    // than the card, clipping the right edge (the close button).
                    ui.set_min_size(card.size() - egui::vec2(2.0 * MARGIN, 2.0 * MARGIN));
                    if sheet.open {
                        // Resolve the active chart's calibrated boxes up front
                        // (an owned Vec) so the spotlight can use them without
                        // contending for `loaded`.
                        let active_boxes = sheet
                            .active
                            .band_id()
                            .map(resolved_boxes)
                            .unwrap_or_default();
                        draw_open_sheet(ui, sheet, &active_boxes, &mut rulebook, time.delta_secs());
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
        egui::Color32::from_gray(220),
    );
}

/// The closed state: a slim vertical "CHARTS" strip that toggles the sheet.
/// Filled brighter than the board so it reads as a clickable index tab, and
/// clickable over its whole area (not just the glyphs).
fn draw_peek_tab(ui: &mut egui::Ui, sheet: &mut ChartSheet) {
    let rect = ui.max_rect();
    let resp = ui.allocate_rect(rect, egui::Sense::click());
    let fill = if resp.hovered() {
        egui::Color32::from_gray(64)
    } else {
        egui::Color32::from_gray(48)
    };
    ui.painter().rect_filled(rect, 0.0, fill);

    // Gentle attention pulse (§decision 3): while `pulse` is live, run a slow
    // teal edge-stripe in and out (~0.8 s per cycle) so a staged request is
    // noticeable without opening the sheet. Keep the frame repainting so the
    // animation actually advances.
    if sheet.pulse > 0.0 {
        ui.ctx().request_repaint();
        let phase = (sheet.pulse * std::f32::consts::TAU / 0.8).sin() * 0.5 + 0.5;
        let a = (phase * 200.0) as u8;
        let stripe =
            egui::Rect::from_min_max(rect.min, egui::pos2(rect.left() + 3.0, rect.bottom()));
        ui.painter().rect_filled(
            stripe,
            0.0,
            egui::Color32::from_rgba_unmultiplied(0x8f, 0xc5, 0xd7, a),
        );
    }

    // egui has no vertical text; stack the glyphs down the strip.
    vertical_label(ui, rect.center(), "CHARTS", egui::FontId::monospace(13.0));
    if resp.clicked() {
        sheet.open_and_consume_stage();
    }
}

/// The open state: index tabs across the top, then the active tab's content.
fn draw_open_sheet(
    ui: &mut egui::Ui,
    sheet: &mut ChartSheet,
    active_boxes: &[omdurman_types::ChartBox],
    rulebook: &mut crate::rulebook::Rulebook,
    dt: f32,
) {
    ui.horizontal(|ui| {
        for tab in ChartTab::ALL {
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

    if let Some(hl) = sheet.highlight {
        // Play: spotlight-dim the active region if the highlight targets this
        // chart (§decision 4 -- dim everything else, no coloured boxes).
        if hl.chart == active
            && let Some(band_id) = active.band_id()
        {
            draw_spotlight(ui, image_rect, band_id, hl, active_boxes);
        }
    }
}

/// Spotlight-dim: darken the whole scan with a translucent ink scrim, leaving
/// the highlighted row / column / cell at full brightness. egui has no even-odd
/// fill, so the scrim is tiled as rects *around* the lit cut-out rather than
/// punched through (§decision 4). The intersection cell of a row+col highlight
/// gets no extra tint; the surrounding row and column are lit a touch dimmer.
fn draw_spotlight(
    ui: &egui::Ui,
    image_rect: egui::Rect,
    chart: &str,
    hl: ChartHighlight,
    boxes: &[omdurman_types::ChartBox],
) {
    let layout = chart_layout(chart);
    let (Some(t), Some(b)) = (layout.get(hl.table), boxes.get(hl.table)) else {
        return;
    };
    let grid = box_grid_rect(box_outer_rect(image_rect, b), b);
    let (nr, nc) = (t.rows.len().max(1), t.cols.len().max(1));

    // The lit region: the union of the highlighted row band and column band,
    // clipped to the grid. With both set, that is a plus-shape (full row + full
    // column); with one set, the whole band; with neither, nothing to light.
    let row_band = hl.row.map(|r| {
        egui::Rect::from_min_max(
            egui::pos2(grid.left(), cell_rect(grid, nr, nc, r, 0).top()),
            egui::pos2(grid.right(), cell_rect(grid, nr, nc, r, 0).bottom()),
        )
    });
    let col_band = hl.col.map(|c| {
        egui::Rect::from_min_max(
            egui::pos2(cell_rect(grid, nr, nc, 0, c).left(), grid.top()),
            egui::pos2(cell_rect(grid, nr, nc, 0, c).right(), grid.bottom()),
        )
    });

    let scrim = egui::Color32::from_black_alpha(150);
    let painter = ui.painter_at(image_rect);

    // Paint the scrim everywhere, then "erase" the lit bands by leaving them
    // uncovered: tile up to four scrim rects around the union bounding rect and,
    // when both bands are present, re-cover the two off-axis corners so only the
    // plus-shape stays bright.
    let lit = match (row_band, col_band) {
        (Some(r), Some(c)) => r.union(c),
        (Some(r), None) => r,
        (None, Some(c)) => c,
        (None, None) => {
            // No cell resolved -- just leave the scan untinted.
            return;
        }
    };
    // Four bands around `lit` (relative to the whole image, so off-table area
    // dims too).
    let full = image_rect;
    for r in [
        egui::Rect::from_min_max(full.min, egui::pos2(full.right(), lit.top())), // above
        egui::Rect::from_min_max(egui::pos2(full.left(), lit.bottom()), full.max), // below
        egui::Rect::from_min_max(
            egui::pos2(full.left(), lit.top()),
            egui::pos2(lit.left(), lit.bottom()),
        ), // left
        egui::Rect::from_min_max(
            egui::pos2(lit.right(), lit.top()),
            egui::pos2(full.right(), lit.bottom()),
        ), // right
    ] {
        if r.width() > 0.0 && r.height() > 0.0 {
            painter.rect_filled(r, 0.0, scrim);
        }
    }
    // With both a row and a column, `lit` is their bounding rect (a filled
    // square), but only the plus-shape should stay bright. Re-cover the four
    // off-axis quadrants left inside `lit`.
    if let (Some(rb), Some(cb)) = (row_band, col_band) {
        for r in [
            egui::Rect::from_min_max(lit.min, egui::pos2(cb.left(), rb.top())), // TL
            egui::Rect::from_min_max(
                egui::pos2(cb.right(), lit.top()),
                egui::pos2(lit.right(), rb.top()),
            ), // TR
            egui::Rect::from_min_max(
                egui::pos2(lit.left(), rb.bottom()),
                egui::pos2(cb.left(), lit.bottom()),
            ), // BL
            egui::Rect::from_min_max(egui::pos2(cb.right(), rb.bottom()), lit.max), // BR
        ] {
            if r.width() > 0.0 && r.height() > 0.0 {
                painter.rect_filled(r, 0.0, scrim);
            }
        }
    }
}

/// The fixed structure of one table on a chart scan, inferred from the printed
/// scan: its display name, the cell labels down its rows and across its columns
/// (which also give the grid dimensions), and a rough default box so it starts
/// roughly in place. Only the *box* is calibrated/persisted; this structure is
/// code.
#[derive(Clone, Copy)]
struct TableLayout {
    rows: &'static [&'static str],
    cols: &'static [&'static str],
    default_box: omdurman_types::ChartBox,
}

const fn rough(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label_w: f32,
    header_h: f32,
) -> omdurman_types::ChartBox {
    omdurman_types::ChartBox {
        x,
        y,
        w,
        h,
        label_w,
        header_h,
    }
}

/// The fixed table layouts per chart, read off the printed scans. The calibrator
/// only nudges each table's box to line up with the scan; the counts and labels
/// never change, so the user never adds or removes tables.
///
/// Every field is `'static` (string literals + plain `f32`s via `const fn`
/// `rough`), so the layouts live in `static` arrays and `chart_layout` hands
/// out `'static` slices -- no heap allocation per chart lookup.
static CRT_LAYOUT: [TableLayout; 3] = [
    TableLayout {
        rows: &[
            "1-5", "6-10", "11-15", "16-20", "21-25", "26-30", "31-35", "36-40", "41+",
        ],
        cols: &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"],
        // Lower-left block, with a left label column + header rows.
        default_box: rough(0.02, 0.55, 0.60, 0.42, 0.10, 0.16),
    },
    TableLayout {
        rows: &["Spears", "Rifles", "Artillery"],
        cols: &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"],
        default_box: rough(0.20, 0.02, 0.78, 0.22, 0.16, 0.30),
    },
    TableLayout {
        rows: &["Rifles", "Maxims", "Artillery", "Howitzer"],
        cols: &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"],
        default_box: rough(0.20, 0.24, 0.78, 0.28, 0.16, 0.0),
    },
];

static TERRAIN_LAYOUT: [TableLayout; 1] = [TableLayout {
    rows: &["Move cost", "Combat"],
    cols: &[
        "Clear",
        "Rough",
        "Trees",
        "Swamp",
        "Nile",
        "Hilltop",
        "Huts",
        "Building",
        "Road",
        "Khor",
        "Crest",
        "City Wall",
        "Zariba",
    ],
    default_box: rough(0.0, 0.0, 1.0, 1.0, 0.14, 0.40),
}];

fn chart_layout(chart: &str) -> &'static [TableLayout] {
    match chart {
        "crt" => &CRT_LAYOUT,
        "terrain" => &TERRAIN_LAYOUT,
        // "timing" intentionally has no tables here: the turn track is already
        // calibrated on the campaign map (CampaignTurnTrack, the Timing editor
        // tab), and it does not apply to the Fall-of-Khartoum board. Re-doing it
        // in the chart calibrator would duplicate that existing annotation.
        //
        // "arrivals" intentionally has no tables here: the order-of-appearance
        // scan is shown as a static reference image. Reinforcement arrival is
        // enforced by `apply_place_reinforcements` (§9.112/§9.113) keyed on
        // `Turn`/`Scenario`, not by spotlighting a sub-table during play.
        _ => &[],
    }
}

/// Resolve the boxes to draw for `chart`: the fixed `default_box` from each
/// `TableLayout`. With the on-disk calibrator dissolved, the printed-scan
/// geometry *is* the only geometry.
fn resolved_boxes(chart: &str) -> Vec<omdurman_types::ChartBox> {
    chart_layout(chart).iter().map(|t| t.default_box).collect()
}

/// The outer box rect for `b` mapped into `image_rect` (whole-scan space).
fn box_outer_rect(image_rect: egui::Rect, b: &omdurman_types::ChartBox) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(
            image_rect.left() + b.x * image_rect.width(),
            image_rect.top() + b.y * image_rect.height(),
        ),
        egui::vec2(b.w * image_rect.width(), b.h * image_rect.height()),
    )
}

/// The data-grid rect (box minus its label column and header rows).
fn box_grid_rect(outer: egui::Rect, b: &omdurman_types::ChartBox) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(
            outer.left() + b.label_w * outer.width(),
            outer.top() + b.header_h * outer.height(),
        ),
        egui::vec2(
            outer.width() * (1.0 - b.label_w),
            outer.height() * (1.0 - b.header_h),
        ),
    )
}

/// The rect of data cell `(r, c)` within an evenly-divided `rows`×`cols` grid.
fn cell_rect(grid: egui::Rect, rows: usize, cols: usize, r: usize, c: usize) -> egui::Rect {
    let cw = grid.width() / cols.max(1) as f32;
    let ch = grid.height() / rows.max(1) as f32;
    egui::Rect::from_min_size(
        egui::pos2(grid.left() + c as f32 * cw, grid.top() + r as f32 * ch),
        egui::vec2(cw, ch),
    )
}

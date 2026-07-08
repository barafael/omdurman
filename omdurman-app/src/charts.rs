//! The reference sheets: a slide-in overlay card holding the game's coarse
//! chart scans (combat results table, terrain effects, campaign timing, order
//! of appearance) plus the rulebook. It is a card laid *on* the table -- an
//! `egui::Area` that slides over the right edge of the board, so the board never
//! reflows. When closed a slim "CHARTS" tab peeks at the right edge.
//!
//! This module owns the shell (slide, tabs, zoom/pan, hotkey). Spotlight-dim
//! highlighting, the docked turn-track strip, and contextual `ChartSheetRequest`
//! staging land in later passes; the rulebook tab is a placeholder for now.

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

    /// Stable id used as the key in `AnnotationsFile::chart_bands`, or `None`
    /// for the text rulebook (which has no bands).
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

/// Which band the calibrator is editing on the current chart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BandAxis {
    Row,
    Col,
}

/// Editor-only calibration state for the chart spotlight bands. Lives on the
/// dedicated editor Charts tab; edits `LoadedAnnotations.0.chart_bands` and
/// marks annotations dirty so the normal debounced flush persists them.
#[derive(Resource, Default)]
pub struct ChartCalibrator {
    /// The band currently selected for editing (axis + index), if any.
    selected: Option<(BandAxis, usize)>,
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
        app.init_resource::<ChartCalibrator>()
            .add_systems(Startup, load_chart_textures)
            // Texture registration touches `EguiUserTextures`, which the egui
            // context pass also accesses internally -- doing both in one system
            // that holds `EguiContexts` is a conflicting `ResMut` borrow (B0002).
            // Register in a plain `Update` system, render in the egui pass.
            .add_systems(Update, register_chart_textures)
            .add_systems(
                EguiPrimaryContextPass,
                chart_sheet_ui.run_if(charts_visible),
            );
    }
}

/// Where the chart sheet may appear:
///   * play map views -- Game or Sandbox, while actually in a game or reviewing
///     a recording (not the lobby / connecting screen);
///   * the editor's dedicated `Charts` tab, for previewing the sheet.
/// It is hidden everywhere else in the editor (charts are a play-view feature)
/// and while the start screen is up (the default mode/state would otherwise let
/// it draw beneath the splash).
fn charts_visible(
    mode: Res<State<crate::AppMode>>,
    tab: Res<State<crate::EditorTab>>,
    app_state: Res<State<crate::AppState>>,
    splash: Option<Res<crate::splash::Splash>>,
) -> bool {
    if splash.is_some() {
        return false;
    }
    match **mode {
        crate::AppMode::Game | crate::AppMode::Sandbox => matches!(
            **app_state,
            crate::AppState::InGame | crate::AppState::Spectating
        ),
        crate::AppMode::Editor => **tab == crate::EditorTab::Charts,
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

fn chart_sheet_ui(
    mut contexts: EguiContexts,
    mut sheet: Option<ResMut<ChartSheet>>,
    mode: Res<State<crate::AppMode>>,
    tab: Res<State<crate::EditorTab>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut calibrator: ResMut<ChartCalibrator>,
    mut loaded: ResMut<crate::LoadedAnnotations>,
    mut dirty: ResMut<crate::AnnotationsDirty>,
) {
    let Some(sheet) = sheet.as_mut() else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // The dedicated editor Charts tab exists to view/calibrate the sheet, so it
    // is always shown open there; the peek/toggle behaviour is for play views.
    let calibrating = **mode == crate::AppMode::Editor && **tab == crate::EditorTab::Charts;
    let force_open = calibrating;
    if force_open {
        sheet.open = true;
    }

    // Dev: seed a couple of demo bands so the overlay is visible in a headless
    // screenshot (OMDURMAN_CHARTS_SEED=1). Inert otherwise; runs once.
    if calibrating
        && std::env::var("OMDURMAN_CHARTS_SEED").is_ok()
        && let Some(band_id) = sheet.active.band_id()
    {
        let axes = loaded.0.chart_bands.axes_mut(band_id);
        if axes.rows.is_empty() && axes.cols.is_empty() {
            axes.rows.push(omdurman_types::ChartBand {
                name: "1-5".into(),
                start: 0.55,
                extent: 0.05,
            });
            axes.cols.push(omdurman_types::ChartBand {
                name: "4".into(),
                start: 0.30,
                extent: 0.06,
            });
            calibrator.selected = Some((BandAxis::Col, 0));
        }
    }

    // In calibration mode, a left side panel lists the active chart's bands.
    if calibrating && let Some(band_id) = sheet.active.band_id() {
        calibrator_panel(ctx, &mut calibrator, &mut loaded, &mut dirty, band_id);
    }

    // Hotkey: C toggles, Esc closes (not on the dedicated editor tab).
    if !force_open {
        if keys.just_pressed(KeyCode::KeyC) {
            sheet.open = !sheet.open;
        }
        if sheet.open && keys.just_pressed(KeyCode::Escape) {
            sheet.open = false;
        }
    }

    let screen = ctx.content_rect();
    // Anchor to the right *content* edge, which is the left edge of the play
    // mode's game-control sidebar (that SidePanel has already reserved its space
    // by now, so `available_rect` excludes it). The sheet and its peek tab sit
    // just left of that sidebar and never overlap it. Falls back to the window
    // edge when no sidebar is present.
    let right = ctx.available_rect().right().min(screen.max.x);

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
                .stroke(egui::Stroke::new(2.0, egui::Color32::from_gray(90)))
                .inner_margin(egui::Margin::same(MARGIN as i8))
                .show(ui, |ui| {
                    // Fill the card minus the frame's own margins on both sides;
                    // using the full card size here pushed content 2*MARGIN wider
                    // than the card, clipping the right edge (the close button).
                    ui.set_min_size(card.size() - egui::vec2(2.0 * MARGIN, 2.0 * MARGIN));
                    if sheet.open {
                        let calib = calibrating.then(|| CalibCtx {
                            calibrator: &mut calibrator,
                            loaded: &mut loaded,
                        });
                        draw_open_sheet(ui, sheet, calib);
                    } else {
                        draw_peek_tab(ui, sheet);
                    }
                });
        });
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
    // egui has no vertical text; stack the glyphs down the strip.
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "C\nH\nA\nR\nT\nS",
        egui::FontId::monospace(13.0),
        egui::Color32::from_gray(220),
    );
    if resp.clicked() {
        sheet.open = true;
    }
}

/// Mutable calibration context handed to `draw_open_sheet` on the editor Charts
/// tab, so the scan view can draw the spotlight bands over the scan. (Band
/// *editing* happens in `calibrator_panel`; this overlay is read-only for now.)
struct CalibCtx<'a> {
    calibrator: &'a mut ChartCalibrator,
    loaded: &'a mut crate::LoadedAnnotations,
}

/// The open state: index tabs across the top, then the active tab's content.
/// `calib` present == the editor Charts tab, which overlays editable bands.
fn draw_open_sheet(ui: &mut egui::Ui, sheet: &mut ChartSheet, mut calib: Option<CalibCtx<'_>>) {
    ui.horizontal(|ui| {
        for tab in ChartTab::ALL {
            if ui
                .add(egui::Button::selectable(sheet.active == tab, tab.label()))
                .clicked()
            {
                sheet.active = tab;
            }
        }
        // No hide button while calibrating: the editor Charts tab is always open.
        if calib.is_none() {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("hide").clicked() {
                    sheet.open = false;
                }
            });
        }
    });
    ui.separator();

    let active = sheet.active;
    if active == ChartTab::Rulebook {
        ui.label("Rulebook — coming in a later pass.");
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

    // Overlay the calibration bands on top of the scan, mapped from normalized
    // coords into `image_rect`. The selected band is drawn brighter.
    if let (Some(calib), Some(band_id)) = (calib.as_mut(), active.band_id()) {
        draw_band_overlay(ui, image_rect, calib, band_id);
    }
}

/// Draw the row/column bands over `image_rect` (which spans the whole scan).
/// Row bands are horizontal strips (full width); column bands are vertical
/// strips (full height). The calibrator's selected band is highlighted.
fn draw_band_overlay(
    ui: &egui::Ui,
    image_rect: egui::Rect,
    calib: &mut CalibCtx<'_>,
    band_id: &str,
) {
    let painter = ui.painter_at(image_rect);
    let axes = calib.loaded.0.chart_bands.axes_mut(band_id);
    let selected = calib.calibrator.selected;

    let row_col = egui::Color32::from_rgba_unmultiplied(0x8f, 0xc5, 0xd7, 60); // teal wash
    let col_col = egui::Color32::from_rgba_unmultiplied(0xba, 0xa7, 0xa6, 60); // rose wash
    let sel_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
    let dim_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(160));

    for (i, band) in axes.rows.iter().enumerate() {
        let y0 = image_rect.top() + band.start * image_rect.height();
        let y1 = y0 + band.extent * image_rect.height();
        let r = egui::Rect::from_min_max(
            egui::pos2(image_rect.left(), y0),
            egui::pos2(image_rect.right(), y1),
        );
        let is_sel = selected == Some((BandAxis::Row, i));
        painter.rect_filled(r, 0.0, row_col);
        painter.rect_stroke(
            r,
            0.0,
            if is_sel { sel_stroke } else { dim_stroke },
            egui::StrokeKind::Inside,
        );
    }
    for (i, band) in axes.cols.iter().enumerate() {
        let x0 = image_rect.left() + band.start * image_rect.width();
        let x1 = x0 + band.extent * image_rect.width();
        let r = egui::Rect::from_min_max(
            egui::pos2(x0, image_rect.top()),
            egui::pos2(x1, image_rect.bottom()),
        );
        let is_sel = selected == Some((BandAxis::Col, i));
        painter.rect_filled(r, 0.0, col_col);
        painter.rect_stroke(
            r,
            0.0,
            if is_sel { sel_stroke } else { dim_stroke },
            egui::StrokeKind::Inside,
        );
    }
}

/// Left side panel (editor Charts tab): list/add/select/edit/delete the active
/// chart's row and column bands. Edits `LoadedAnnotations` and marks the
/// annotations dirty so the normal debounced flush persists them.
fn calibrator_panel(
    ctx: &egui::Context,
    calibrator: &mut ChartCalibrator,
    loaded: &mut crate::LoadedAnnotations,
    dirty: &mut crate::AnnotationsDirty,
    band_id: &str,
) {
    egui::SidePanel::left("chart_calibrator")
        .default_width(230.0)
        .show(ctx, |ui| {
            ui.heading("Chart bands");
            ui.label(format!("chart: {band_id}"));
            ui.separator();

            let axes = loaded.0.chart_bands.axes_mut(band_id);
            let mut changed = false;

            for (axis, label) in [(BandAxis::Row, "Rows"), (BandAxis::Col, "Columns")] {
                ui.horizontal(|ui| {
                    ui.strong(label);
                    if ui.small_button("+ add").clicked() {
                        let bands = match axis {
                            BandAxis::Row => &mut axes.rows,
                            BandAxis::Col => &mut axes.cols,
                        };
                        bands.push(omdurman_types::ChartBand {
                            name: format!("{}", bands.len() + 1),
                            start: 0.4,
                            extent: 0.1,
                        });
                        calibrator.selected = Some((axis, bands.len() - 1));
                        changed = true;
                    }
                });

                let bands = match axis {
                    BandAxis::Row => &mut axes.rows,
                    BandAxis::Col => &mut axes.cols,
                };
                let mut delete: Option<usize> = None;
                for i in 0..bands.len() {
                    let is_sel = calibrator.selected == Some((axis, i));
                    ui.horizontal(|ui| {
                        if ui.selectable_label(is_sel, &bands[i].name).clicked() {
                            calibrator.selected = Some((axis, i));
                        }
                        if ui.small_button("✕").clicked() {
                            delete = Some(i);
                        }
                    });
                    if is_sel {
                        ui.indent(("band_edit", axis as u8, i), |ui| {
                            let b = &mut bands[i];
                            if ui.text_edit_singleline(&mut b.name).changed() {
                                changed = true;
                            }
                            changed |= ui
                                .add(
                                    egui::Slider::new(&mut b.start, 0.0..=1.0)
                                        .text("start")
                                        .fixed_decimals(3),
                                )
                                .changed();
                            changed |= ui
                                .add(
                                    egui::Slider::new(&mut b.extent, 0.001..=1.0)
                                        .text("extent")
                                        .fixed_decimals(3),
                                )
                                .changed();
                        });
                    }
                }
                if let Some(i) = delete {
                    bands.remove(i);
                    calibrator.selected = None;
                    changed = true;
                }
                ui.separator();
            }

            if changed {
                dirty.mark();
            }
        });
}

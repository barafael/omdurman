//! Original-scan reference view: renders the manual / mapsheet scans a
//! table was transcribed from, below the active editor.

use std::path::{Path, PathBuf};

use egui::{Color32, RichText, TextureHandle, TextureOptions};

use crate::common::TableKind;

/// The scan(s) a table was transcribed from, resolved relative to
/// `tables_dir` (i.e. `Boardgame - Remember_Gordon/tables/`).
///
/// - Units:      photo of the die-cut counter sheets
/// - CRT/Range:  the extracted tables sheet (range effects top, CRT middle,
///   scattergram lower right)
/// - Scatter:    the hand-cropped scattergram hex diagram
/// - LOS:        rulebook back page (LOS table + special LOS notes)
/// - Appearance: the Campaign Game Order of Appearance card
pub fn scans_for(kind: TableKind, tables_dir: &Path) -> Vec<PathBuf> {
    let Some(base) = tables_dir.parent() else {
        return Vec::new();
    };
    let manual = |name: &str| base.join("Manual/Elements").join(name);
    match kind {
        TableKind::Units => vec![base.join("Units/units_photo.png")],
        TableKind::Crt | TableKind::Range => vec![manual("CombatResultsTable.jpg")],
        TableKind::Scatter => vec![manual("scattergram.png")],
        TableKind::Los => vec![manual("Manual_11.jpg")],
        TableKind::Appearance => vec![manual("CampaignGameOrdnerOfAppearance.jpg")],
    }
}

/// Caches one texture set per table and draws it with fit-width / zoom.
pub struct ReferenceView {
    /// Joined path list the current textures were loaded from.
    loaded_key: Option<String>,
    textures: Vec<(PathBuf, Result<TextureHandle, String>)>,
    fit_width: bool,
    zoom: f32,
}

impl ReferenceView {
    pub fn new() -> Self {
        ReferenceView {
            loaded_key: None,
            textures: Vec::new(),
            fit_width: true,
            zoom: 1.0,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, paths: &[PathBuf]) {
        self.ensure_loaded(ctx, paths);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Original (scan)").strong());
            for (path, res) in &self.textures {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                match res {
                    Ok(_) => {
                        ui.label(RichText::new(name).color(Color32::GRAY))
                            .on_hover_text(path.display().to_string());
                    }
                    Err(e) => {
                        ui.label(RichText::new(format!("{name}: {e}")).color(Color32::RED))
                            .on_hover_text(path.display().to_string());
                    }
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.fit_width {
                    if ui.button("1:1 zoom").clicked() {
                        self.fit_width = false;
                        self.zoom = 1.0;
                    }
                } else {
                    if ui.button("Fit width").clicked() {
                        self.fit_width = true;
                    }
                    ui.add(
                        egui::DragValue::new(&mut self.zoom)
                            .range(0.05..=4.0)
                            .speed(0.05)
                            .fixed_decimals(2)
                            .suffix("x"),
                    );
                }
            });
        });

        let fit_width = self.fit_width;
        let zoom = self.zoom;
        let paint = |ui: &mut egui::Ui| {
            for (_, res) in &self.textures {
                let Ok(tex) = res else { continue };
                let native = tex.size_vec2();
                let scale = if fit_width {
                    ui.available_width() / native.x
                } else {
                    zoom
                };
                let size = egui::Vec2::new(native.x * scale, native.y * scale);
                ui.add(egui::Image::from_texture(tex).fit_to_exact_size(size));
            }
        };
        if fit_width {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, paint);
        } else {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, paint);
        }
    }

    fn ensure_loaded(&mut self, ctx: &egui::Context, paths: &[PathBuf]) {
        let key = paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("|");
        if self.loaded_key.as_deref() == Some(&key) {
            return;
        }
        log::debug!("loading reference scans: {key}");
        self.textures = paths.iter().map(|p| (p.clone(), load(ctx, p))).collect();
        self.loaded_key = Some(key);
    }
}

fn load(ctx: &egui::Context, path: &Path) -> Result<TextureHandle, String> {
    let img = image::open(path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    let id = format!("reference_scan::{}", path.display());
    let started = std::time::Instant::now();
    let tex = ctx.load_texture(id, image, TextureOptions::default());
    log::info!(
        "loaded reference scan {} ({}x{}) in {:?}",
        path.display(),
        size[0],
        size[1],
        started.elapsed()
    );
    Ok(tex)
}

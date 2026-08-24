//! Shared infrastructure for the table editors.

pub mod command;
pub mod comments;
pub mod sprites;

use std::path::{Path, PathBuf};

use eframe::egui;

/// Parse a table file into its model: comments are scanned out and map keys
/// are quoted so serde can handle them.
pub fn parse_table<T: serde::de::DeserializeOwned>(path: &Path) -> Result<(T, comments::Scan), EditorError> {
    let started = std::time::Instant::now();
    log::debug!("parsing {} as {}", path.display(), std::any::type_name::<T>());
    let text = std::fs::read_to_string(path)?;
    let scan = comments::scan(&text);
    let doc: T = ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&scan.quoted)
        .map_err(|e| {
            log::error!("failed to parse {}: {e}", path.display());
            EditorError::Parse(e.to_string())
        })?;
    log::debug!(
        "parsed {} ({} bytes) in {:?}",
        path.display(),
        text.len(),
        started.elapsed()
    );
    Ok((doc, scan))
}

/// Atomically replace `path` (write to a sibling temp file, then rename).
pub fn save_atomic(path: &Path, contents: &str) -> Result<(), EditorError> {
    let tmp = path.with_extension("ron.tmp");
    if let Err(e) = std::fs::write(&tmp, contents).and_then(|()| std::fs::rename(&tmp, path)) {
        log::error!("failed to save {}: {e}", path.display());
        return Err(e.into());
    }
    log::info!("saved {} ({} bytes)", path.display(), contents.len());
    Ok(())
}

#[derive(Debug)]
pub enum EditorError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for EditorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorError::Io(e) => write!(f, "I/O error: {e}"),
            EditorError::Parse(e) => write!(f, "parse error: {e}"),
        }
    }
}

impl From<std::io::Error> for EditorError {
    fn from(e: std::io::Error) -> Self {
        EditorError::Io(e)
    }
}

// ── Table registry ──────────────────────────────────────────────────────

/// The six editable tables in `Boardgame - Remember_Gordon/tables/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TableKind {
    Units,
    Crt,
    Scatter,
    Los,
    Range,
    Appearance,
}

impl TableKind {
    pub const ALL: [TableKind; 6] = [
        TableKind::Units,
        TableKind::Crt,
        TableKind::Scatter,
        TableKind::Los,
        TableKind::Range,
        TableKind::Appearance,
    ];

    pub fn file_name(self) -> &'static str {
        match self {
            TableKind::Units => "units.ron",
            TableKind::Crt => "combat_results_table.ron",
            TableKind::Scatter => "howitzer_scattergram.ron",
            TableKind::Los => "los_table.ron",
            TableKind::Range => "range_effects_table.ron",
            TableKind::Appearance => "order_of_appearance.ron",
        }
    }

    pub fn display(self) -> &'static str {
        match self {
            TableKind::Units => "Units",
            TableKind::Crt => "Combat Results",
            TableKind::Scatter => "Howitzer Scatter",
            TableKind::Los => "Line of Sight",
            TableKind::Range => "Range Effects",
            TableKind::Appearance => "Order of Appearance",
        }
    }

    pub fn path(self, tables_dir: &Path) -> PathBuf {
        tables_dir.join(self.file_name())
    }
}

/// One engine cross-check finding.
#[derive(Clone, Debug)]
pub struct CheckResult {
    pub severity: Severity,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
    /// The `.ron` table and the compiled engine disagree.
    Mismatch,
    /// They agree only at a coarser level, or the comparison is partial.
    Info,
    /// They agree.
    Ok,
}

/// Behavior every table editor implements; the shell drives save/undo/redo
/// and switching.
pub trait TableEditor {
    fn dirty(&self) -> bool;
    fn save(&mut self) -> Result<(), String>;
    fn undo(&mut self);
    fn redo(&mut self);
    fn can_undo(&self) -> bool;
    fn can_redo(&self) -> bool;
    /// Draw the editor into the shell's central area.
    fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui);
    /// Compare the table against the compiled rules engine.
    fn engine_check(&self) -> Vec<CheckResult>;
}

// ── Shared widgets ──────────────────────────────────────────────────────

/// A compact dropdown cell: shows `current`, opens a popup with `options`.
/// Returns the index of the chosen option.
pub fn dropdown_cell(
    ui: &mut egui::Ui,
    id: egui::Id,
    current: &str,
    options: &[(String, String)], // (value, display)
) -> Option<usize> {
    let display = options
        .iter()
        .find(|(v, _)| v == current)
        .map(|(_, d)| d.as_str())
        .unwrap_or(current);
    let button = ui.add_sized(
        [ui.available_width(), 18.0],
        egui::Button::new(egui::RichText::new(display).small()),
    );
    let mut picked = None;
    if button.clicked() {
        ui.memory_mut(|m| m.toggle_popup(id));
    }
    egui::popup_below_widget(ui, id, &button, egui::PopupCloseBehavior::CloseOnClick, |ui| {
        ui.set_min_width(72.0);
        for (i, (_, label)) in options.iter().enumerate() {
            if ui
                .selectable_label(label.as_str() == display, label.as_str())
                .clicked()
            {
                picked = Some(i);
                ui.memory_mut(|m| m.close_popup());
            }
        }
    });
    picked
}

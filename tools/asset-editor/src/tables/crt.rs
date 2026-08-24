//! `combat_results_table.ron` — 9 fire-factor bands × 10 modified rolls.

use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, RichText};
use indexmap::IndexMap;

use crate::common::command::{EditorCommand, History};
use crate::common::{parse_table, save_atomic, CheckResult, EditorError, Severity, TableEditor};

pub const ROLLS: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CrtCell {
    NoEffect,
    Disrupt,
    Eliminate(u8),
}

impl CrtCell {
    pub const OPTIONS: [CrtCell; 7] = [
        CrtCell::NoEffect,
        CrtCell::Disrupt,
        CrtCell::Eliminate(1),
        CrtCell::Eliminate(2),
        CrtCell::Eliminate(3),
        CrtCell::Eliminate(4),
        CrtCell::Eliminate(5),
    ];

    pub fn label(&self) -> String {
        match self {
            CrtCell::NoEffect => "–".into(),
            CrtCell::Disrupt => "D".into(),
            CrtCell::Eliminate(n) => format!("E{n}"),
        }
    }

    fn ron_literal(&self) -> String {
        match self {
            CrtCell::NoEffect => "NoEffect".into(),
            CrtCell::Disrupt => "Disrupt".into(),
            CrtCell::Eliminate(n) => format!("Eliminate({n})"),
        }
    }

    /// Ordering used by the monotonicity sanity check (higher = harsher).
    fn severity(&self) -> u8 {
        match self {
            CrtCell::NoEffect => 0,
            CrtCell::Disrupt => 1,
            CrtCell::Eliminate(n) => 2 + n.min(&5) * 2,
        }
    }
}

/// A band's ten roll outcomes. A `Vec` rather than `[CrtCell; ROLLS]`
/// because serde deserializes arrays via `deserialize_tuple`, which RON
/// requires parenthesized — the file uses bracketed sequences.
pub type CrtRow = Vec<CrtCell>;

#[derive(Clone, Debug, Default)]
pub struct CrtDoc {
    pub header: String,
    pub comments: std::collections::BTreeMap<String, String>,
    pub rows: IndexMap<String, CrtRow>,
}

impl CrtDoc {
    pub fn to_ron_string(&self) -> String {
        let mut out = String::new();
        if !self.header.is_empty() {
            out.push_str(&self.header);
            out.push('\n');
        }
        out.push_str("{\n");
        for (name, row) in &self.rows {
            if let Some(c) = self.comments.get(name.as_str()) {
                out.push_str(c);
                out.push('\n');
            }
            out.push_str(&format!("    {name}: ["));
            let cells: Vec<String> = row.iter().map(|c| c.ron_literal()).collect();
            out.push_str(&cells.join(", "));
            out.push_str("],\n");
        }
        out.push_str("}\n");
        out
    }
}

// ── Commands ────────────────────────────────────────────────────────────

/// Rows are addressed by index so undo survives renames.
#[derive(Clone, Debug)]
enum Cmd {
    SetCell { row: usize, col: usize, old: CrtCell, new: CrtCell },
    RenameRow { row: usize, old: String, new: String },
    AddRow { name: String },
    DeleteRow { index: usize, name: String, row: CrtRow },
}

impl EditorCommand for Cmd {
    fn coalesce_key(&self) -> Option<String> {
        match self {
            Cmd::SetCell { row, col, .. } => Some(format!("cell/{row}/{col}")),
            Cmd::RenameRow { row, .. } => Some(format!("rename/{row}")),
            _ => None,
        }
    }

    fn merge(&mut self, next: &Self) -> bool {
        match (self, next) {
            (Cmd::SetCell { new, .. }, Cmd::SetCell { new: n, .. }) => {
                *new = *n;
                true
            }
            (Cmd::RenameRow { new, .. }, Cmd::RenameRow { new: n, .. }) => {
                *new = n.clone();
                true
            }
            _ => false,
        }
    }
}

impl Cmd {
    fn apply(&self, doc: &mut CrtDoc) {
        match self {
            Cmd::SetCell { row, col, new, .. } => {
                if let Some((_, r)) = doc.rows.get_index_mut(*row) {
                    r[*col] = *new;
                }
            }
            Cmd::RenameRow { row, new, .. } => rename_row(doc, *row, new.clone()),
            Cmd::AddRow { name } => {
                doc.rows.insert(name.clone(), vec![CrtCell::NoEffect; ROLLS]);
            }
            Cmd::DeleteRow { index, name, .. } => {
                doc.rows.shift_remove(name);
                doc.comments.remove(name);
                debug_assert_eq!(doc.rows.get_index(*index), None);
            }
        }
    }

    fn revert(&self, doc: &mut CrtDoc) {
        match self {
            Cmd::SetCell { row, col, old, .. } => {
                if let Some((_, r)) = doc.rows.get_index_mut(*row) {
                    r[*col] = *old;
                }
            }
            Cmd::RenameRow { row, old, .. } => rename_row(doc, *row, old.clone()),
            Cmd::AddRow { name } => {
                if let Some(i) = doc.rows.get_index_of(name) {
                    doc.rows.shift_remove_index(i);
                }
            }
            Cmd::DeleteRow { index, name, row } => {
                let mut pairs: Vec<(String, CrtRow)> = doc.rows.drain(..).collect();
                pairs.insert(*index, (name.clone(), row.clone()));
                doc.rows.extend(pairs);
            }
        }
    }
}

/// Rename the band at `row`, preserving order and moving any comment.
fn rename_row(doc: &mut CrtDoc, row: usize, new_name: String) {
    let Some((old, _)) = doc.rows.get_index(row) else {
        return;
    };
    let old = old.clone();
    if let Some(c) = doc.comments.remove(&old) {
        doc.comments.insert(new_name.clone(), c);
    }
    let mut pairs: Vec<(String, CrtRow)> = doc.rows.drain(..).collect();
    if let Some(pair) = pairs.get_mut(row) {
        pair.0 = new_name;
    }
    doc.rows = pairs.into_iter().collect();
}

// ── Editor ──────────────────────────────────────────────────────────────

pub struct CrtEditor {
    path: PathBuf,
    doc: CrtDoc,
    dirty: bool,
    history: History<Cmd>,
    new_row_name: String,
    error: Option<String>,
}

impl CrtEditor {
    pub fn open(path: PathBuf) -> Self {
        let (doc, error) = match load(&path) {
            Ok(doc) => (doc, None),
            Err(e) => (CrtDoc::default(), Some(format!("failed to load: {e}"))),
        };
        CrtEditor {
            path,
            doc,
            dirty: false,
            history: History::new(),
            new_row_name: String::new(),
            error,
        }
    }

    fn run(&mut self, cmd: Cmd) {
        cmd.apply(&mut self.doc);
        self.history.record(cmd);
        self.dirty = true;
    }
}

fn load(path: &std::path::Path) -> Result<CrtDoc, EditorError> {
    let (rows, scan): (IndexMap<String, CrtRow>, _) = parse_table(path)?;
    Ok(CrtDoc {
        header: scan.header,
        comments: scan.comments,
        rows,
    })
}

impl TableEditor for CrtEditor {
    fn dirty(&self) -> bool {
        self.dirty
    }

    fn save(&mut self) -> Result<(), String> {
        save_atomic(&self.path, &self.doc.to_ron_string()).map_err(|e| e.to_string())?;
        self.dirty = false;
        self.error = None;
        Ok(())
    }

    fn undo(&mut self) {
        if let Some(cmd) = self.history.undo() {
            cmd.revert(&mut self.doc);
            self.dirty = true;
        }
    }

    fn redo(&mut self) {
        if let Some(cmd) = self.history.redo() {
            cmd.apply(&mut self.doc);
            self.dirty = true;
        }
    }

    fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(err) = &self.error {
            ui.label(RichText::new(err).color(Color32::RED));
            return;
        }
        ui.heading("Combat Results Table (§6.22)");
        ui.label(
            RichText::new("total fire factor band × modified d10 roll")
                .weak(),
        );
        ui.separator();

        let mut cmds: Vec<Cmd> = Vec::new();
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("crt_grid")
                .num_columns(ROLLS + 2)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("");
                    for roll in 1..=ROLLS {
                        ui.strong(RichText::new(format!("{roll}")).small())
                            .on_hover_text(format!("modified die roll {roll}"));
                    }
                    ui.strong("band");
                    ui.end_row();

                    let rows: Vec<(String, CrtRow)> = self
                        .doc
                        .rows
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    for (row_idx, (name, row)) in rows.into_iter().enumerate() {
                        let mut name_edit = name.clone();
                        ui.add(
                            egui::TextEdit::singleline(&mut name_edit)
                                .desired_width(90.0)
                                .clip_text(true),
                        );
                        if name_edit != name && !name_edit.is_empty() {
                            cmds.push(Cmd::RenameRow {
                                row: row_idx,
                                old: name.clone(),
                                new: name_edit,
                            });
                        }

                        let mut non_monotonic = false;
                        for (col, cell) in row.iter().enumerate() {
                            if col > 0 && cell.severity() < row[col - 1].severity() {
                                non_monotonic = true;
                            }
                            let id = egui::Id::new(("crt_cell", &name, col));
                            let options: Vec<(String, String)> = CrtCell::OPTIONS
                                .iter()
                                .map(|c| {
                                    (
                                        match c {
                                            CrtCell::NoEffect => "NoEffect".into(),
                                            CrtCell::Disrupt => "Disrupt".into(),
                                            CrtCell::Eliminate(n) => {
                                                format!("Eliminate{n}")
                                            }
                                        },
                                        c.label(),
                                    )
                                })
                                .collect();
                            let current = match cell {
                                CrtCell::NoEffect => "NoEffect".to_string(),
                                CrtCell::Disrupt => "Disrupt".to_string(),
                                CrtCell::Eliminate(n) => format!("Eliminate{n}"),
                            };
                            let cell = *cell;
                            if let Some(pick) = crate::common::dropdown_cell(
                                ui,
                                id,
                                &current,
                                &options,
                            ) {
                                cmds.push(Cmd::SetCell {
                                    row: row_idx,
                                    col,
                                    old: cell,
                                    new: CrtCell::OPTIONS[pick],
                                });
                            }
                        }

                        let warn = non_monotonic
                            .then(|| RichText::new("⚠").color(Color32::YELLOW))
                            .unwrap_or_else(|| RichText::new(""));
                        ui.label(warn).on_hover_text(
                            "band is less severe at a higher roll — check for typos",
                        );
                        ui.end_row();
                    }
                });
        });

        for cmd in cmds {
            self.run(cmd);
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("new band:");
            ui.text_edit_singleline(&mut self.new_row_name);
            if ui.button("+ add").clicked() && !self.new_row_name.is_empty() {
                let name = self.new_row_name.clone();
                self.run(Cmd::AddRow { name });
                self.new_row_name.clear();
            }
        });
        ui.horizontal(|ui| {
            let deletable: Vec<(usize, String)> = self
                .doc
                .rows
                .iter()
                .enumerate()
                .map(|(i, (k, _))| (i, k.clone()))
                .collect();
            for (index, name) in deletable {
                if ui
                    .small_button(format!("del {name}"))
                    .on_hover_text("delete band (undoable)")
                    .clicked()
                {
                    let row = self.doc.rows[&name].clone();
                    self.run(Cmd::DeleteRow { index, name, row });
                }
            }
        });
    }

    fn engine_check(&self) -> Vec<CheckResult> {
        use omdurman_rules::combat_results_table::{
            combat_results_table, FireFactorRow,
        };
        use omdurman_rules::DieRoll;

        let roll_names = [
            "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten",
        ];
        let mut results = Vec::new();
        let mut mismatches = 0;
        for (name, row) in &self.doc.rows {
            let Ok(band) = ron::from_str::<FireFactorRow>(name) else {
                results.push(CheckResult {
                    severity: Severity::Mismatch,
                    message: format!("band {name:?} is not a FireFactorRow variant"),
                });
                continue;
            };
            for (i, cell) in row.iter().enumerate() {
                let roll: DieRoll = ron::from_str(roll_names[i]).unwrap();
                let engine = combat_results_table(band, roll);
                let same = match (cell, engine) {
                    (CrtCell::NoEffect, omdurman_rules::CombatResult::NoEffect) => true,
                    (CrtCell::Disrupt, omdurman_rules::CombatResult::Disrupt) => true,
                    (CrtCell::Eliminate(a), omdurman_rules::CombatResult::Eliminate(b)) => *a == b,
                    _ => false,
                };
                if !same {
                    mismatches += 1;
                    results.push(CheckResult {
                        severity: Severity::Mismatch,
                        message: format!(
                            "{name}, roll {}: table says {}, engine says {:?}",
                            i + 1,
                            cell.label(),
                            engine,
                        ),
                    });
                }
            }
        }
        if mismatches == 0 && results.is_empty() {
            results.push(CheckResult {
                severity: Severity::Ok,
                message: format!(
                    "all {} bands × 10 rolls match combat_results_table()",
                    self.doc.rows.len()
                ),
            });
        }
        results
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn real_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../Boardgame - Remember_Gordon/tables/combat_results_table.ron")
    }

    #[test]
    fn real_file_round_trip_is_byte_identical() {
        let path = real_path();
        let original = std::fs::read_to_string(&path).unwrap();
        let doc = load(&path).unwrap();
        assert_eq!(doc.rows.len(), 9);
        assert!(doc.header.starts_with("// Combat Results Table (§6.22)."));
        let out = doc.to_ron_string();
        assert_eq!(out, original, "serialization must be byte-identical");
    }

    #[test]
    fn cell_literals() {
        assert_eq!(CrtCell::Eliminate(3).ron_literal(), "Eliminate(3)");
        assert_eq!(CrtCell::NoEffect.label(), "–");
    }

    #[test]
    fn set_cell_undo() {
        let mut ed = CrtEditor::open(real_path());
        let name = ed.doc.rows.keys().next().unwrap().clone();
        let old = ed.doc.rows[&name][0];
        ed.run(Cmd::SetCell {
            row: 0,
            col: 0,
            old,
            new: CrtCell::Eliminate(5),
        });
        assert_eq!(ed.doc.rows[&name][0], CrtCell::Eliminate(5));
        ed.undo();
        assert_eq!(ed.doc.rows[&name][0], old);
        ed.redo();
        assert_eq!(ed.doc.rows[&name][0], CrtCell::Eliminate(5));
    }
}

//! `howitzer_scattergram.ron` — impact direction by d10 roll.

use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, RichText};

use crate::common::command::{EditorCommand, History};
use crate::common::{parse_table, save_atomic, CheckResult, EditorError, Severity, TableEditor, TableKind};

pub const ROLLS: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    UpperLeft,
    UpperRight,
    Right,
    LowerRight,
    LowerLeft,
    Left,
    Center,
}

impl Direction {
    pub const ALL: [Direction; 7] = [
        Direction::UpperLeft,
        Direction::UpperRight,
        Direction::Right,
        Direction::LowerRight,
        Direction::LowerLeft,
        Direction::Left,
        Direction::Center,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Direction::UpperLeft => "UpperLeft",
            Direction::UpperRight => "UpperRight",
            Direction::Right => "Right",
            Direction::LowerRight => "LowerRight",
            Direction::LowerLeft => "LowerLeft",
            Direction::Left => "Left",
            Direction::Center => "Center",
        }
    }

    pub fn short(&self) -> &'static str {
        match self {
            Direction::UpperLeft => "↖",
            Direction::UpperRight => "↗",
            Direction::Right => "→",
            Direction::LowerRight => "↘",
            Direction::LowerLeft => "↙",
            Direction::Left => "←",
            Direction::Center => "◎",
        }
    }

    /// Position in the 3×3 compass rose (row, col).
    fn rose_pos(&self) -> (usize, usize) {
        match self {
            Direction::UpperLeft => (0, 0),
            Direction::UpperRight => (0, 2),
            Direction::Right => (1, 2),
            Direction::LowerRight => (2, 2),
            Direction::LowerLeft => (2, 0),
            Direction::Left => (1, 0),
            Direction::Center => (1, 1),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScatterDoc {
    pub header: String,
    pub comments: std::collections::BTreeMap<String, String>,
    /// `rolls[i]` is the impact direction for die roll `i + 1`.
    pub rolls: Vec<Direction>,
}

impl ScatterDoc {
    pub fn to_ron_string(&self) -> String {
        let mut out = String::new();
        if !self.header.is_empty() {
            out.push_str(&self.header);
            out.push('\n');
        }
        out.push_str("[\n");
        for (i, dir) in self.rolls.iter().enumerate() {
            if let Some(c) = self
                .comments
                .get(&crate::common::comments::elem("", i))
            {
                out.push_str(c);
                out.push('\n');
            }
            out.push_str(&format!("    {},\n", dir.name()));
        }
        out.push_str("]\n");
        out
    }
}

// ── Commands ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Cmd {
    SetRoll { roll: usize, old: Direction, new: Direction },
    SetLen { old: Vec<Direction>, new: Vec<Direction> },
}

impl EditorCommand for Cmd {
    fn label(&self) -> &'static str {
        match self {
            Cmd::SetRoll { .. } => "set roll",
            Cmd::SetLen { .. } => "resize",
        }
    }

    fn coalesce_key(&self) -> Option<String> {
        match self {
            Cmd::SetRoll { roll, .. } => Some(format!("roll/{roll}")),
            Cmd::SetLen { .. } => None,
        }
    }

    fn merge(&mut self, next: &Self) -> bool {
        match (self, next) {
            (Cmd::SetRoll { new, .. }, Cmd::SetRoll { new: n, .. }) => {
                *new = *n;
                true
            }
            _ => false,
        }
    }
}

impl Cmd {
    fn apply(&self, doc: &mut ScatterDoc) {
        match self {
            Cmd::SetRoll { roll, new, .. } => {
                if let Some(d) = doc.rolls.get_mut(*roll) {
                    *d = *new;
                }
            }
            Cmd::SetLen { new, .. } => doc.rolls = new.clone(),
        }
    }

    fn revert(&self, doc: &mut ScatterDoc) {
        match self {
            Cmd::SetRoll { roll, old, .. } => {
                if let Some(d) = doc.rolls.get_mut(*roll) {
                    *d = *old;
                }
            }
            Cmd::SetLen { old, .. } => doc.rolls = old.clone(),
        }
    }
}

// ── Editor ──────────────────────────────────────────────────────────────

pub struct ScatterEditor {
    path: PathBuf,
    doc: ScatterDoc,
    dirty: bool,
    history: History<Cmd>,
    error: Option<String>,
}

impl ScatterEditor {
    pub fn open(path: PathBuf) -> Self {
        let (doc, error) = match load(&path) {
            Ok(doc) => (doc, None),
            Err(e) => (ScatterDoc::default(), Some(format!("failed to load: {e}"))),
        };
        ScatterEditor {
            path,
            doc,
            dirty: false,
            history: History::new(),
            error,
        }
    }

    fn run(&mut self, cmd: Cmd) {
        cmd.apply(&mut self.doc);
        self.history.record(cmd);
        self.dirty = true;
    }
}

fn load(path: &std::path::Path) -> Result<ScatterDoc, EditorError> {
    let (rolls, scan): (Vec<Direction>, _) = parse_table(path)?;
    Ok(ScatterDoc {
        header: scan.header,
        comments: scan.comments,
        rolls,
    })
}

impl TableEditor for ScatterEditor {
    fn kind(&self) -> TableKind {
        TableKind::Scatter
    }

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
        ui.heading("Howitzer Scattergram (§6.64)");
        ui.label(RichText::new("impact direction per second (impact) d10 roll").weak());
        ui.separator();

        // Compass rose: which rolls land where.
        ui.horizontal(|ui| {
            egui::Grid::new("rose")
                .num_columns(3)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    for r in 0..3 {
                        for c in 0..3 {
                            let hits: Vec<String> = self
                                .doc
                                .rolls
                                .iter()
                                .enumerate()
                                .filter(|(_, d)| d.rose_pos() == (r, c))
                                .map(|(i, _)| format!("{}", i + 1))
                                .collect();
                            let any = Direction::ALL
                                .iter()
                                .find(|d| d.rose_pos() == (r, c));
                            let (sym, name) = match any {
                                Some(d) => (d.short(), d.name()),
                                None => ("·", ""),
                            };
                            let text = if hits.is_empty() {
                                RichText::new(sym).weak().size(20.0)
                            } else {
                                RichText::new(format!("{sym} {}", hits.join(",")))
                                    .size(14.0)
                                    .color(Color32::LIGHT_BLUE)
                            };
                            let resp = egui::Frame::NONE
                                .fill(Color32::from_rgb(30, 30, 30))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::Vec2::new(64.0, 36.0));
                                    ui.centered_and_justified(|ui| ui.label(text));
                                })
                                .response;
                            resp.on_hover_text(name);
                        }
                        ui.end_row();
                    }
                });

            ui.vertical(|ui| {
                ui.label("rolls:");
                let mut cmds: Vec<Cmd> = Vec::new();
                egui::Grid::new("roll_strip")
                    .num_columns(5)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        for (i, dir) in self.doc.rolls.iter().enumerate() {
                            let id = egui::Id::new(("scatter_roll", i));
                            let options: Vec<(String, String)> = Direction::ALL
                                .iter()
                                .map(|d| (d.name().to_string(), d.short().to_string()))
                                .collect();
                            if let Some(pick) = crate::common::dropdown_cell(
                                ui,
                                id,
                                dir.name(),
                                &options,
                            ) {
                                let old = *dir;
                                let new = Direction::ALL[pick];
                                cmds.push(Cmd::SetRoll { roll: i, old, new });
                            }
                            if (i + 1) % 5 == 0 {
                                ui.end_row();
                            }
                        }
                    });
                for cmd in cmds {
                    self.run(cmd);
                }
            });
        });

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("array length: {}", self.doc.rolls.len()));
            if ui.button("−").clicked() && self.doc.rolls.len() > 1 {
                let old = self.doc.rolls.clone();
                let mut new = old.clone();
                new.pop();
                self.run(Cmd::SetLen { old, new });
            }
            if ui.button("+").clicked() && self.doc.rolls.len() < 20 {
                let old = self.doc.rolls.clone();
                let mut new = old.clone();
                new.push(Direction::Center);
                self.run(Cmd::SetLen { old, new });
            }
            if self.doc.rolls.len() != ROLLS {
                ui.label(
                    RichText::new(format!("⚠ expected {ROLLS} rolls (d10)"))
                        .color(Color32::YELLOW),
                );
            }
        });
    }

    fn engine_check(&self) -> Vec<CheckResult> {
        use omdurman_rules::howitzer_scatter::{howitzer_scatter, ScatterDirection};
        use omdurman_rules::DieRoll;

        let roll_names = [
            "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten",
        ];
        let mut results = Vec::new();
        let mut mismatches = 0;
        for (i, dir) in self.doc.rolls.iter().enumerate() {
            if i >= ROLLS {
                break;
            }
            let roll: DieRoll = ron::from_str(roll_names[i]).unwrap();
            let engine = howitzer_scatter(roll);
            // Coarse comparison: the table knows precise directions, the
            // engine bucket: OnTarget vs scattered.
            let table_on_target = *dir == Direction::Center;
            let engine_on_target = engine == ScatterDirection::OnTarget;
            if table_on_target != engine_on_target {
                mismatches += 1;
                results.push(CheckResult {
                    severity: Severity::Mismatch,
                    message: format!(
                        "roll {}: table says {} ({:?}), engine says {:?}",
                        i + 1,
                        dir.short(),
                        dir,
                        engine,
                    ),
                });
            }
        }
        if self.doc.rolls.len() != ROLLS {
            results.push(CheckResult {
                severity: Severity::Mismatch,
                message: format!(
                    "table has {} rolls, engine expects a d10 (10)",
                    self.doc.rolls.len()
                ),
            });
        }
        if mismatches == 0 && results.is_empty() {
            results.push(CheckResult {
                severity: Severity::Ok,
                message: "on-target/scatter pattern matches howitzer_scatter()".into(),
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
            .join("../../Boardgame - Remember_Gordon/tables/howitzer_scattergram.ron")
    }

    #[test]
    fn real_file_round_trip_is_byte_identical() {
        let path = real_path();
        let original = std::fs::read_to_string(&path).unwrap();
        let doc = load(&path).unwrap();
        assert_eq!(doc.rolls.len(), 10);
        assert_eq!(doc.rolls[6], Direction::Center);
        assert!(doc.header.contains("§6.64"));
        assert_eq!(doc.to_ron_string(), original);
    }

    #[test]
    fn set_roll_undo() {
        let mut ed = ScatterEditor::open(real_path());
        let old = ed.doc.rolls[0];
        ed.run(Cmd::SetRoll {
            roll: 0,
            old,
            new: Direction::Center,
        });
        assert_eq!(ed.doc.rolls[0], Direction::Center);
        ed.undo();
        assert_eq!(ed.doc.rolls[0], old);
    }
}

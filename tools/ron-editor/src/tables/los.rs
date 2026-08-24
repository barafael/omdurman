//! `los_table.ron` — line-of-sight levels, blocking rules, details, notes.
//!
//! Stringly-typed on purpose: the editor offers known names as dropdowns but
//! tolerates unknown ones so hand-made extensions survive a round-trip.

use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, RichText};
use indexmap::IndexMap;

use crate::common::command::{EditorCommand, History};
use crate::common::comments;
use crate::common::{parse_table, save_atomic, CheckResult, EditorError, Severity, TableEditor, TableKind};

pub const LEVELS: [&str; 3] = ["Ground", "Rough", "Hilltop"];
pub const TERRAINS: [&str; 7] = [
    "Clear", "Swamp", "Nile", "Huts", "Building", "Rough", "Hilltop",
];
pub const FEATURES: [&str; 7] = ["Units", "Huts", "Wall", "Rough", "Crest", "Trees", "Hilltop"];
pub const CONDITIONS: [&str; 8] = [
    "MoreThanTwo",
    "CrestAdjacency",
    "CloserToFirer",
    "CloserToTarget",
    "AdjSameLevelFirer",
    "AdjSameLevelTarget",
    "NotAtLowerLevel",
    "HilltopOnly",
];

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FeatureRule(
    pub String,
    pub Vec<String>,
);

impl FeatureRule {
    pub fn feature(&self) -> &str {
        &self.0
    }
    pub fn conditions(&self) -> &[String] {
        &self.1
    }
}

#[derive(Clone, Debug, Default)]
pub struct LosDoc {
    pub header: String,
    pub comments: std::collections::BTreeMap<String, String>,
    /// level name → terrain names at that level.
    pub levels: IndexMap<String, Vec<String>>,
    /// `"(Firer, Target)"` → feature rules.
    pub cells: IndexMap<String, Vec<FeatureRule>>,
    pub details: IndexMap<String, String>,
    pub notes: IndexMap<String, String>,
}

/// `(firer, target)` → the map key used on disk.
pub fn cell_key(firer: &str, target: &str) -> String {
    format!("({firer}, {target})")
}

impl LosDoc {
    pub fn to_ron_string(&self) -> String {
        let mut out = String::new();
        if !self.header.is_empty() {
            out.push_str(&self.header);
            out.push('\n');
        }
        out.push_str("(\n");
        out.push_str("    levels: {\n");
        for (level, terrains) in &self.levels {
            let addr = comments::addr("levels", level);
            if let Some(c) = self.comments.get(&addr) {
                out.push_str(c);
                out.push('\n');
            }
            out.push_str(&format!("        {level}: [{}],\n", terrains.join(", ")));
        }
        out.push_str("    },\n");
        out.push_str("    cells: {\n");
        for (key, rules) in &self.cells {
            let addr = comments::addr("cells", key);
            if let Some(c) = self.comments.get(&addr) {
                out.push_str(c);
                out.push('\n');
            }
            out.push_str(&format!("        {key}: [\n"));
            for (i, rule) in rules.iter().enumerate() {
                let addr = comments::elem(&comments::addr("cells", key), i);
                if let Some(c) = self.comments.get(&addr) {
                    out.push_str(c);
                    out.push('\n');
                }
                let conds = if rule.conditions().is_empty() {
                    String::new()
                } else {
                    format!(", [{}]", rule.conditions().join(", "))
                };
                out.push_str(&format!(
                    "            ({},{}),\n",
                    rule.feature(),
                    conds
                ));
            }
            out.push_str("        ],\n");
        }
        out.push_str("    },\n");
        out.push_str("    details: {\n");
        for (k, v) in &self.details {
            out.push_str(&format!("        {k}: {},\n", quote(v)));
        }
        out.push_str("    },\n");
        out.push_str("    notes: {\n");
        for (k, v) in &self.notes {
            out.push_str(&format!("        {k}: {},\n", quote(v)));
        }
        out.push_str("    },\n");
        out.push_str(")\n");
        out
    }
}

fn quote(s: &str) -> String {
    format!("{:?}", s)
}

// ── Commands ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Cmd {
    SetTerrains { level: String, old: Vec<String>, new: Vec<String> },
    SetRules { key: String, old: Vec<FeatureRule>, new: Vec<FeatureRule> },
    SetDetail { key: String, old: String, new: String },
    SetNote { key: String, old: String, new: String },
}

impl EditorCommand for Cmd {
    fn label(&self) -> &'static str {
        match self {
            Cmd::SetTerrains { .. } => "edit level terrains",
            Cmd::SetRules { .. } => "edit blocking rules",
            Cmd::SetDetail { .. } => "edit detail",
            Cmd::SetNote { .. } => "edit note",
        }
    }

    fn coalesce_key(&self) -> Option<String> {
        match self {
            Cmd::SetTerrains { level, .. } => Some(format!("levels/{level}")),
            Cmd::SetRules { key, .. } => Some(format!("cells/{key}")),
            Cmd::SetDetail { key, .. } => Some(format!("details/{key}")),
            Cmd::SetNote { key, .. } => Some(format!("notes/{key}")),
        }
    }

    fn merge(&mut self, next: &Self) -> bool {
        match (self, next) {
            (Cmd::SetTerrains { new, .. }, Cmd::SetTerrains { new: n, .. }) => {
                *new = n.clone();
                true
            }
            (Cmd::SetRules { new, .. }, Cmd::SetRules { new: n, .. }) => {
                *new = n.clone();
                true
            }
            (Cmd::SetDetail { new, .. }, Cmd::SetDetail { new: n, .. }) => {
                *new = n.clone();
                true
            }
            (Cmd::SetNote { new, .. }, Cmd::SetNote { new: n, .. }) => {
                *new = n.clone();
                true
            }
            _ => false,
        }
    }
}

impl Cmd {
    fn apply(&self, doc: &mut LosDoc) {
        match self {
            Cmd::SetTerrains { level, new, .. } => {
                doc.levels.insert(level.clone(), new.clone());
            }
            Cmd::SetRules { key, new, .. } => {
                doc.cells.insert(key.clone(), new.clone());
            }
            Cmd::SetDetail { key, new, .. } => {
                doc.details.insert(key.clone(), new.clone());
            }
            Cmd::SetNote { key, new, .. } => {
                doc.notes.insert(key.clone(), new.clone());
            }
        }
    }

    fn revert(&self, doc: &mut LosDoc) {
        match self {
            Cmd::SetTerrains { level, old, .. } => {
                doc.levels.insert(level.clone(), old.clone());
            }
            Cmd::SetRules { key, old, .. } => {
                doc.cells.insert(key.clone(), old.clone());
            }
            Cmd::SetDetail { key, old, .. } => {
                doc.details.insert(key.clone(), old.clone());
            }
            Cmd::SetNote { key, old, .. } => {
                doc.notes.insert(key.clone(), old.clone());
            }
        }
    }
}

// ── Deserialization shape ───────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct RawDoc {
    levels: IndexMap<String, Vec<String>>,
    cells: IndexMap<String, Vec<FeatureRule>>,
    details: IndexMap<String, String>,
    notes: IndexMap<String, String>,
}

// ── Editor ──────────────────────────────────────────────────────────────

pub struct LosEditor {
    path: PathBuf,
    doc: LosDoc,
    selected: (usize, usize), // (firer level idx, target level idx)
    dirty: bool,
    history: History<Cmd>,
    error: Option<String>,
}

impl LosEditor {
    pub fn open(path: PathBuf) -> Self {
        let (doc, error) = match load(&path) {
            Ok(doc) => (doc, None),
            Err(e) => (LosDoc::default(), Some(format!("failed to load: {e}"))),
        };
        LosEditor {
            path,
            doc,
            selected: (0, 0),
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

fn load(path: &std::path::Path) -> Result<LosDoc, EditorError> {
    let (raw, scan): (RawDoc, _) = parse_table(path)?;
    Ok(LosDoc {
        header: scan.header,
        comments: scan.comments,
        levels: raw.levels,
        cells: raw.cells,
        details: raw.details,
        notes: raw.notes,
    })
}

impl TableEditor for LosEditor {
    fn kind(&self) -> TableKind {
        TableKind::Los
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
        ui.heading("Line of Sight Table (§6.3)");
        ui.separator();

        self.show_levels(ui);
        ui.separator();
        self.show_matrix(ui);
        ui.separator();
        self.show_selected_rules(ui);
        ui.separator();
        self.show_details_notes(ui);
    }

    fn engine_check(&self) -> Vec<CheckResult> {
        use omdurman_rules::los_table::{los_level, LosLevel};
        use omdurman_types::Terrain;

        let mut results = Vec::new();
        let mut mismatches = 0;
        // levels: every known terrain's engine level matches the map.
        for (level_name, terrains) in &self.doc.levels {
            for terrain_name in terrains {
                let Ok(terrain) = ron::from_str::<Terrain>(terrain_name) else {
                    results.push(CheckResult {
                        severity: Severity::Mismatch,
                        message: format!(
                            "terrain {terrain_name:?} under {level_name:?} is not a Terrain variant"
                        ),
                    });
                    continue;
                };
                let engine = los_level(terrain);
                let table_level = match level_name.as_str() {
                    "Ground" => LosLevel::Ground,
                    "Rough" => LosLevel::Rough,
                    "Hilltop" => LosLevel::Hilltop,
                    other => {
                        results.push(CheckResult {
                            severity: Severity::Mismatch,
                            message: format!("unknown level {other:?}"),
                        });
                        continue;
                    }
                };
                if engine != table_level {
                    mismatches += 1;
                    results.push(CheckResult {
                        severity: Severity::Mismatch,
                        message: format!(
                            "terrain {terrain_name}: table puts it at {level_name}, engine says {engine:?}"
                        ),
                    });
                }
            }
        }
        // Terrain variants missing from the table.
        for terrain_name in TERRAINS {
            let listed = self
                .doc
                .levels
                .values()
                .any(|v| v.iter().any(|t| t == terrain_name));
            if !listed {
                results.push(CheckResult {
                    severity: Severity::Info,
                    message: format!("terrain {terrain_name} is not listed in `levels`"),
                });
            }
        }
        results.push(CheckResult {
            severity: Severity::Info,
            message: format!(
                "{} cells × feature rules are not compared against blocking_rules() yet",
                self.doc.cells.len()
            ),
        });
        if mismatches == 0 {
            results.insert(
                0,
                CheckResult {
                    severity: Severity::Ok,
                    message: "all listed terrains match los_level()".into(),
                },
            );
        }
        results
    }
}

impl LosEditor {
    fn show_levels(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("levels — terrain → LOS level").strong());
        let mut cmds: Vec<Cmd> = Vec::new();
        for level in LEVELS {
            ui.horizontal(|ui| {
                ui.label(format!("{level:>8}:"));
                let current = self.doc.levels.get(level).cloned().unwrap_or_default();
                let mut new = current.clone();
                for t in TERRAINS {
                    let active = current.iter().any(|x| x == t);
                    let text = if active {
                        RichText::new(t).color(Color32::LIGHT_BLUE)
                    } else {
                        RichText::new(t).weak()
                    };
                    if ui
                        .selectable_label(active, text.small())
                        .on_hover_text(format!(
                            "toggle {t} at {level} level"
                        ))
                        .clicked()
                    {
                        if active {
                            new.retain(|x| x != t);
                        } else {
                            new.push(t.to_string());
                        }
                    }
                }
                if new != current {
                    cmds.push(Cmd::SetTerrains {
                        level: level.to_string(),
                        old: current,
                        new,
                    });
                }
            });
        }
        for cmd in cmds {
            self.run(cmd);
        }
    }

    fn show_matrix(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("cells — features that block (firer ↓, target →)").strong());
        egui::Grid::new("los_matrix")
            .num_columns(LEVELS.len() + 1)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                ui.strong("");
                for target in LEVELS {
                    ui.strong(target);
                }
                ui.end_row();
                for (fi, firer) in LEVELS.iter().enumerate() {
                    ui.strong(*firer);
                    for (ti, target) in LEVELS.iter().enumerate() {
                        let key = cell_key(firer, target);
                        let rules = self.doc.cells.get(&key);
                        let count = rules.map(Vec::len).unwrap_or(0);
                        let selected = self.selected == (fi, ti);
                        let text = if count == 0 {
                            RichText::new("∅").weak()
                        } else {
                            RichText::new(format!("{count}")).color(Color32::LIGHT_BLUE)
                        };
                        let frame = if selected {
                            egui::Frame::NONE.stroke(egui::Stroke::new(2.0_f32, Color32::YELLOW))
                        } else {
                            egui::Frame::NONE
                        };
                        let resp = frame
                            .show(ui, |ui| {
                                ui.set_min_size(egui::Vec2::new(48.0, 24.0));
                                ui.centered_and_justified(|ui| ui.label(text));
                            })
                            .response;
                        if resp.clicked() {
                            self.selected = (fi, ti);
                        }
                        resp.on_hover_text(format!("{count} blocking features"));
                    }
                    ui.end_row();
                }
            });
    }

    fn show_selected_rules(&mut self, ui: &mut egui::Ui) {
        let (fi, ti) = self.selected;
        let key = cell_key(LEVELS[fi], LEVELS[ti]);
        ui.label(
            RichText::new(format!("rules for {key} — a feature blocks only if ALL its conditions hold"))
                .strong(),
        );

        let mut new_rules = self.doc.cells.get(&key).cloned().unwrap_or_default();
        let original = new_rules.clone();
        let mut structural = false;

        let mut remove: Option<usize> = None;
        for (ri, rule) in new_rules.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // Feature dropdown.
                let id = egui::Id::new(("los_feature", &key, ri));
                let options: Vec<(String, String)> = FEATURES
                    .iter()
                    .map(|f| (f.to_string(), f.to_string()))
                    .collect();
                let current = rule.0.clone();
                if let Some(pick) =
                    crate::common::dropdown_cell(ui, id, &current, &options)
                {
                    rule.0 = options[pick].0.clone();
                    structural = true;
                }

                // Condition chips.
                for c in CONDITIONS {
                    let active = rule.1.iter().any(|x| x == c);
                    let text = if active {
                        RichText::new(c).small().color(Color32::LIGHT_BLUE)
                    } else {
                        RichText::new(c).small().weak()
                    };
                    if ui.selectable_label(active, text).clicked() {
                        if active {
                            rule.1.retain(|x| x != c);
                        } else {
                            rule.1.push(c.to_string());
                        }
                        structural = true;
                    }
                }
                if ui.small_button("✕").clicked() {
                    remove = Some(ri);
                }
            });
        }
        if let Some(ri) = remove {
            new_rules.remove(ri);
            structural = true;
        }
        ui.horizontal(|ui| {
            if ui.button("+ add feature").clicked() {
                new_rules.push(FeatureRule("Units".into(), Vec::new()));
                structural = true;
            }
        });

        if structural || new_rules != original {
            self.run(Cmd::SetRules {
                key: key.clone(),
                old: original,
                new: new_rules,
            });
        }
    }

    fn show_details_notes(&mut self, ui: &mut egui::Ui) {
        let mut cmds: Vec<Cmd> = Vec::new();

        egui::CollapsingHeader::new("details (condition explanations)")
            .default_open(false)
            .show(ui, |ui| {
                let entries: Vec<(String, String)> = self
                    .doc
                    .details
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                for (key, value) in entries {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&key).monospace().strong(),
                        );
                        let mut edit = value.clone();
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut edit)
                                .desired_width(ui.available_width()),
                        );
                        if resp.changed() && edit != value {
                            cmds.push(Cmd::SetDetail {
                                key,
                                old: value,
                                new: edit,
                            });
                        }
                    });
                }
            });

        egui::CollapsingHeader::new("notes A–F")
            .default_open(false)
            .show(ui, |ui| {
                let entries: Vec<(String, String)> = self
                    .doc
                    .notes
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                for (key, value) in entries {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&key).monospace().strong());
                        let mut edit = value.clone();
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut edit)
                                .desired_width(ui.available_width()),
                        );
                        if resp.changed() && edit != value {
                            cmds.push(Cmd::SetNote {
                                key,
                                old: value,
                                new: edit,
                            });
                        }
                    });
                }
            });

        for cmd in cmds {
            self.run(cmd);
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn real_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../Boardgame - Remember_Gordon/tables/los_table.ron")
    }

    #[test]
    fn real_file_round_trip_is_byte_identical() {
        let path = real_path();
        let original = std::fs::read_to_string(&path).unwrap();
        let doc = load(&path).unwrap();
        assert_eq!(doc.levels.len(), 3);
        assert_eq!(doc.cells.len(), 9);
        assert_eq!(doc.details.len(), 9);
        assert_eq!(doc.notes.len(), 6);

        // The two inline comments survive at their exact addresses.
        assert_eq!(
            doc.comments
                .get("cells/(Ground, Rough)/[0]")
                .map(String::as_str),
            Some("            // B: walled-city units count as Rough targets.")
        );
        assert_eq!(
            doc.comments
                .get("cells/(Rough, Rough)/[0]")
                .map(String::as_str),
            Some("            // B: walled-city units count as Rough targets.")
        );
        assert_eq!(doc.to_ron_string(), original);
    }

    #[test]
    fn cell_key_format() {
        assert_eq!(cell_key("Ground", "Rough"), "(Ground, Rough)");
    }

    #[test]
    fn set_rules_undo() {
        let mut ed = LosEditor::open(real_path());
        let key = cell_key("Ground", "Ground");
        let old = ed.doc.cells[&key].clone();
        let mut new = old.clone();
        new.push(FeatureRule("Trees".into(), vec!["MoreThanTwo".into()]));
        ed.run(Cmd::SetRules { key: key.clone(), old, new });
        assert_eq!(ed.doc.cells[&key].len(), 6);
        ed.undo();
        assert_eq!(ed.doc.cells[&key].len(), 5);
    }
}

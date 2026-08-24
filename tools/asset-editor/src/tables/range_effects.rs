//! `range_effects_table.ron` — fire multipliers by faction × weapon × range.

use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, RichText};
use indexmap::IndexMap;

use crate::common::command::{EditorCommand, History};
use crate::common::{parse_table, save_atomic, CheckResult, EditorError, Severity, TableEditor};

pub const RANGES: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Effect {
    Normal,
    Doubled,
    Tripled,
    Halved,
    OutOfRange,
}

impl Effect {
    pub const ALL: [Effect; 5] = [
        Effect::Normal,
        Effect::Doubled,
        Effect::Tripled,
        Effect::Halved,
        Effect::OutOfRange,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Effect::Normal => "×1",
            Effect::Doubled => "×2",
            Effect::Tripled => "×3",
            Effect::Halved => "½",
            Effect::OutOfRange => "—",
        }
    }

    fn ron_literal(&self) -> &'static str {
        match self {
            Effect::Normal => "Normal",
            Effect::Doubled => "Doubled",
            Effect::Tripled => "Tripled",
            Effect::Halved => "Halved",
            Effect::OutOfRange => "OutOfRange",
        }
    }
}

/// A weapon's ten range-band outcomes. A `Vec` rather than
/// `[Effect; RANGES]` because serde deserializes arrays via
/// `deserialize_tuple`, which RON requires parenthesized — the file uses
/// bracketed sequences.
type Row = Vec<Effect>;

#[derive(Clone, Debug, Default)]
pub struct RangeDoc {
    pub header: String,
    pub comments: std::collections::BTreeMap<String, String>,
    pub dervish: IndexMap<String, Row>,
    pub anglo: IndexMap<String, Row>,
}

impl RangeDoc {
    fn faction(&self, which: Faction) -> &IndexMap<String, Row> {
        match which {
            Faction::Dervish => &self.dervish,
            Faction::AngloEgyptian => &self.anglo,
        }
    }

    fn faction_mut(&mut self, which: Faction) -> &mut IndexMap<String, Row> {
        match which {
            Faction::Dervish => &mut self.dervish,
            Faction::AngloEgyptian => &mut self.anglo,
        }
    }

    pub fn to_ron_string(&self) -> String {
        let mut out = String::new();
        if !self.header.is_empty() {
            out.push_str(&self.header);
            out.push('\n');
        }
        out.push_str("(\n");
        for (ron_name, table) in [("Dervish", &self.dervish), ("AngloEgyptian", &self.anglo)] {
            if let Some(c) = self.comments.get(ron_name) {
                out.push_str(c);
                out.push('\n');
            }
            out.push_str(&format!("    {ron_name}: {{\n"));
            for (weapon, row) in table {
                let addr = format!("{ron_name}/{weapon}");
                if let Some(c) = self.comments.get(&addr) {
                    out.push_str(c);
                    out.push('\n');
                }
                let cells: Vec<&str> = row.iter().map(|e| e.ron_literal()).collect();
                out.push_str(&format!("        {weapon}: [{}],\n", cells.join(", ")));
            }
            out.push_str("    },\n");
        }
        out.push_str(")\n");
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    Dervish,
    AngloEgyptian,
}

impl Faction {
    fn label(&self) -> &'static str {
        match self {
            Faction::Dervish => "Dervish",
            Faction::AngloEgyptian => "Anglo-Egyptian",
        }
    }
}

// ── Deserialization shape ───────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct RawDoc {
    #[serde(rename = "Dervish")]
    dervish: IndexMap<String, Row>,
    #[serde(rename = "AngloEgyptian")]
    anglo_egyptian: IndexMap<String, Row>,
}

// ── Commands ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Cmd {
    SetCell { faction: Faction, weapon_idx: usize, col: usize, old: Effect, new: Effect },
    RenameWeapon { faction: Faction, weapon_idx: usize, old: String, new: String },
    AddWeapon { faction: Faction, name: String },
    DeleteWeapon { faction: Faction, index: usize, name: String, row: Row },
}

impl EditorCommand for Cmd {
    fn coalesce_key(&self) -> Option<String> {
        match self {
            Cmd::SetCell { faction, weapon_idx, col, .. } => {
                Some(format!("cell/{faction:?}/{weapon_idx}/{col}"))
            }
            Cmd::RenameWeapon { faction, weapon_idx, .. } => {
                Some(format!("rename/{faction:?}/{weapon_idx}"))
            }
            _ => None,
        }
    }

    fn merge(&mut self, next: &Self) -> bool {
        match (self, next) {
            (Cmd::SetCell { new, .. }, Cmd::SetCell { new: n, .. }) => {
                *new = *n;
                true
            }
            (Cmd::RenameWeapon { new, .. }, Cmd::RenameWeapon { new: n, .. }) => {
                *new = n.clone();
                true
            }
            _ => false,
        }
    }
}

impl Cmd {
    fn apply(&self, doc: &mut RangeDoc) {
        match self {
            Cmd::SetCell { faction, weapon_idx, col, new, .. } => {
                if let Some((_, row)) = doc.faction_mut(*faction).get_index_mut(*weapon_idx) {
                    row[*col] = *new;
                }
            }
            Cmd::RenameWeapon { faction, weapon_idx, new, .. } => {
                rename_weapon(doc, *faction, *weapon_idx, new.clone());
            }
            Cmd::AddWeapon { faction, name } => {
                doc.faction_mut(*faction)
                    .insert(name.clone(), vec![Effect::OutOfRange; RANGES]);
            }
            Cmd::DeleteWeapon { faction, name, .. } => {
                doc.faction_mut(*faction).shift_remove(name);
            }
        }
    }

    fn revert(&self, doc: &mut RangeDoc) {
        match self {
            Cmd::SetCell { faction, weapon_idx, col, old, .. } => {
                if let Some((_, row)) = doc.faction_mut(*faction).get_index_mut(*weapon_idx) {
                    row[*col] = *old;
                }
            }
            Cmd::RenameWeapon { faction, weapon_idx, old, .. } => {
                rename_weapon(doc, *faction, *weapon_idx, old.clone());
            }
            Cmd::AddWeapon { faction, name } => {
                if let Some(i) = doc.faction_mut(*faction).get_index_of(name) {
                    doc.faction_mut(*faction).shift_remove_index(i);
                }
            }
            Cmd::DeleteWeapon { faction, index, name, row } => {
                let mut pairs: Vec<(String, Row)> =
                    doc.faction_mut(*faction).drain(..).collect();
                pairs.insert(*index, (name.clone(), row.clone()));
                doc.faction_mut(*faction).extend(pairs);
            }
        }
    }
}

fn faction_ron_name(f: Faction) -> &'static str {
    match f {
        Faction::Dervish => "Dervish",
        Faction::AngloEgyptian => "AngloEgyptian",
    }
}

/// Rename the weapon at `weapon_idx`, preserving order and moving any
/// comment (indexmap keys are immutable, so the map is rebuilt).
fn rename_weapon(doc: &mut RangeDoc, faction: Faction, weapon_idx: usize, new: String) {
    let Some((old, _)) = doc.faction(faction).get_index(weapon_idx) else {
        return;
    };
    let old = old.clone();
    let comment = doc
        .comments
        .remove(&format!("{}/{old}", faction_ron_name(faction)));
    let mut pairs: Vec<(String, Row)> = doc.faction_mut(faction).drain(..).collect();
    if let Some(pair) = pairs.get_mut(weapon_idx) {
        pair.0 = new.clone();
    }
    doc.faction_mut(faction).extend(pairs);
    if let Some(c) = comment {
        doc.comments
            .insert(format!("{}/{new}", faction_ron_name(faction)), c);
    }
}

// ── Editor ──────────────────────────────────────────────────────────────

pub struct RangeEditor {
    path: PathBuf,
    doc: RangeDoc,
    tab: Faction,
    new_weapon: String,
    dirty: bool,
    history: History<Cmd>,
    error: Option<String>,
}

impl RangeEditor {
    pub fn open(path: PathBuf) -> Self {
        let (doc, error) = match load(&path) {
            Ok(doc) => (doc, None),
            Err(e) => (RangeDoc::default(), Some(format!("failed to load: {e}"))),
        };
        RangeEditor {
            path,
            doc,
            tab: Faction::Dervish,
            new_weapon: String::new(),
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

fn load(path: &std::path::Path) -> Result<RangeDoc, EditorError> {
    let (raw, scan): (RawDoc, _) = parse_table(path)?;
    Ok(RangeDoc {
        header: scan.header,
        comments: scan.comments,
        dervish: raw.dervish,
        anglo: raw.anglo_egyptian,
    })
}

impl TableEditor for RangeEditor {
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
        ui.heading("Range Effects Table (§6.22)");
        ui.separator();

        ui.horizontal(|ui| {
            for f in [Faction::Dervish, Faction::AngloEgyptian] {
                if ui
                    .selectable_label(self.tab == f, f.label())
                    .clicked()
                {
                    self.tab = f;
                }
            }
        });
        ui.separator();

        let faction = self.tab;
        let mut cmds: Vec<Cmd> = Vec::new();
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("range_grid")
                .num_columns(RANGES + 1)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("weapon");
                    for r in 1..=RANGES {
                        ui.strong(RichText::new(format!("{r}")).small())
                            .on_hover_text(format!("hex distance {r}"));
                    }
                    ui.end_row();

                    let rows: Vec<(String, Row)> = self
                        .doc
                        .faction(faction)
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    for (wi, (weapon, row)) in rows.into_iter().enumerate() {
                        let mut name_edit = weapon.clone();
                        ui.add(
                            egui::TextEdit::singleline(&mut name_edit)
                                .desired_width(80.0)
                                .clip_text(true),
                        );
                        if name_edit != weapon && !name_edit.is_empty() {
                            cmds.push(Cmd::RenameWeapon {
                                faction,
                                weapon_idx: wi,
                                old: weapon.clone(),
                                new: name_edit,
                            });
                        }
                        for (col, eff) in row.iter().enumerate() {
                            let id = egui::Id::new(("range_cell", faction, weapon.as_str(), col));
                            let options: Vec<(String, String)> = Effect::ALL
                                .iter()
                                .map(|e| (format!("{e:?}"), e.label().to_string()))
                                .collect();
                            let current = format!("{eff:?}");
                            if let Some(pick) =
                                crate::common::dropdown_cell(ui, id, &current, &options)
                            {
                                cmds.push(Cmd::SetCell {
                                    faction,
                                    weapon_idx: wi,
                                    col,
                                    old: *eff,
                                    new: Effect::ALL[pick],
                                });
                            }
                        }
                        ui.end_row();
                    }
                });
        });

        for cmd in cmds {
            self.run(cmd);
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_weapon);
            if ui.button("+ add weapon").clicked() && !self.new_weapon.is_empty() {
                let name = self.new_weapon.clone();
                self.run(Cmd::AddWeapon { faction, name });
                self.new_weapon.clear();
            }
            let deletable: Vec<(usize, String)> = self
                .doc
                .faction(faction)
                .iter()
                .enumerate()
                .map(|(i, (k, _))| (i, k.clone()))
                .collect();
            for (index, weapon) in deletable {
                if ui.small_button(format!("del {weapon}")).clicked() {
                    let row = self.doc.faction(faction)[&weapon].clone();
                    self.run(Cmd::DeleteWeapon { faction, index, name: weapon, row });
                }
            }
        });
    }

    fn engine_check(&self) -> Vec<CheckResult> {
        use omdurman_rules::range_effects::{
            ae_range_effects, dervish_range_effects,
        };
        use omdurman_rules::{HexDistance, RangeBand, WeaponClass};

        let mut results = Vec::new();
        let mut mismatches = 0;
        for (faction, table) in [
            (Faction::Dervish, &self.doc.dervish),
            (Faction::AngloEgyptian, &self.doc.anglo),
        ] {
            for (weapon_name, row) in table {
                let Ok(weapon) = ron::from_str::<WeaponClass>(weapon_name) else {
                    results.push(CheckResult {
                        severity: Severity::Mismatch,
                        message: format!(
                            "{faction:?} weapon {weapon_name:?} is not a WeaponClass variant"
                        ),
                    });
                    continue;
                };
                for (i, eff) in row.iter().enumerate() {
                    let distance = HexDistance::new(i as u16 + 1);
                    let engine = match faction {
                        Faction::Dervish => dervish_range_effects(weapon, distance),
                        Faction::AngloEgyptian => ae_range_effects(weapon, distance),
                    };
                    let table_band = match eff {
                        Effect::Normal => RangeBand::Normal,
                        Effect::Doubled => RangeBand::Doubled,
                        Effect::Tripled => RangeBand::Tripled,
                        Effect::Halved => RangeBand::Halved,
                        Effect::OutOfRange => RangeBand::OutOfRange,
                    };
                    if table_band != engine {
                        mismatches += 1;
                        results.push(CheckResult {
                            severity: Severity::Mismatch,
                            message: format!(
                                "{faction:?} {weapon_name} at range {}: table {eff:?}, engine {engine:?}",
                                i + 1
                            ),
                        });
                    }
                }
            }
        }
        if !self.doc.anglo.contains_key("Melee") {
            results.push(CheckResult {
                severity: Severity::Info,
                message: "AE Melee is not tabled; the engine treats it as adjacent-only"
                    .into(),
            });
        }
        if mismatches == 0 {
            results.insert(
                0,
                CheckResult {
                    severity: Severity::Ok,
                    message: "all tabled weapon × range cells match the engine".into(),
                },
            );
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
            .join("../../Boardgame - Remember_Gordon/tables/range_effects_table.ron")
    }

    #[test]
    fn real_file_round_trip_is_byte_identical() {
        let path = real_path();
        let original = std::fs::read_to_string(&path).unwrap();
        let doc = load(&path).unwrap();
        assert_eq!(doc.dervish.len(), 5);
        assert_eq!(doc.anglo.len(), 4);
        // The two inline comments sit on specific weapon rows.
        assert!(doc
            .comments
            .get("Dervish/Melee")
            .is_some_and(|c| c.contains("Spears")));
        assert!(doc
            .comments
            .get("Dervish/Maxims")
            .is_some_and(|c| c.contains("archived txt")));
        assert_eq!(doc.to_ron_string(), original);
    }

    #[test]
    fn set_cell_undo() {
        let mut ed = RangeEditor::open(real_path());
        let old = ed.doc.dervish[0][0];
        ed.run(Cmd::SetCell {
            faction: Faction::Dervish,
            weapon_idx: 0,
            col: 0,
            old,
            new: Effect::Tripled,
        });
        assert_eq!(ed.doc.dervish[0][0], Effect::Tripled);
        ed.undo();
        assert_eq!(ed.doc.dervish[0][0], old);
    }
}

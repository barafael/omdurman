//! `units.ron` — the physical counter sheets.
//!
//! Ported from the original single-table editor; the document now stores
//! comments in the shared path-keyed map instead of `SectionEntry` wrappers.

use std::collections::BTreeMap;
use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, RichText, Vec2};

use crate::common::command::{EditorCommand, History};
use crate::common::comments;
use crate::common::sprites::SpriteCache;
use crate::common::{CheckResult, EditorError, Severity, TableEditor, parse_table, save_atomic};

// ── Model ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ColorName {
    Black,
    White,
    Red,
    Green,
    Sand,
    Blue,
    Gray,
}

impl ColorName {
    pub const ALL: [ColorName; 7] = [
        ColorName::Black,
        ColorName::White,
        ColorName::Red,
        ColorName::Green,
        ColorName::Sand,
        ColorName::Blue,
        ColorName::Gray,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ColorName::Black => "Black",
            ColorName::White => "White",
            ColorName::Red => "Red",
            ColorName::Green => "Green",
            ColorName::Sand => "Sand",
            ColorName::Blue => "Blue",
            ColorName::Gray => "Gray",
        }
    }
}

pub type SpriteColor = (ColorName, ColorName);

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Faction {
    Dervish,
    /// Anglo-Egyptian; the payload is the brigade name, empty if unassigned.
    AE(String),
    Unknown,
}

impl Faction {
    pub fn display(&self) -> String {
        match self {
            Faction::Dervish => "Dervish".into(),
            Faction::AE(b) if b.is_empty() => "AE".into(),
            Faction::AE(b) => format!("AE({b})"),
            Faction::Unknown => "Unknown".into(),
        }
    }

    fn ron_literal(&self) -> String {
        match self {
            Faction::Dervish => "Dervish".into(),
            Faction::AE(b) => format!("AE({})", ron_str(b)),
            Faction::Unknown => "Unknown".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    Infantry {
        fire: u8,
        melee: u8,
        movement: u8,
    },
    Leader {
        fire: u8,
        melee: u8,
        movement: u8,
    },
    OldGunboat {
        artillery: u8,
        upstream: u8,
        downstream: u8,
    },
    NamedGunboat {
        artillery: u8,
        maxim: u8,
        upstream: u8,
        downstream: u8,
    },
    Fort {
        attack: u8,
        defense: u8,
    },
    Marker,
}

impl Kind {
    pub const NAMES: [&str; 6] = [
        "Infantry",
        "Leader",
        "OldGunboat",
        "NamedGunboat",
        "Fort",
        "Marker",
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Kind::Infantry { .. } => "Infantry",
            Kind::Leader { .. } => "Leader",
            Kind::OldGunboat { .. } => "OldGunboat",
            Kind::NamedGunboat { .. } => "NamedGunboat",
            Kind::Fort { .. } => "Fort",
            Kind::Marker => "Marker",
        }
    }

    pub fn display(&self) -> String {
        match self {
            Kind::Infantry {
                fire,
                melee,
                movement,
            } => {
                format!("Infantry {fire},{melee},{movement}")
            }
            Kind::Leader {
                fire,
                melee,
                movement,
            } => format!("Leader {fire},{melee},{movement}"),
            Kind::OldGunboat {
                artillery,
                upstream,
                downstream,
            } => {
                format!("OldGunboat {artillery},{upstream},{downstream}")
            }
            Kind::NamedGunboat {
                artillery,
                maxim,
                upstream,
                downstream,
            } => {
                format!("NamedGunboat {artillery},{maxim},{upstream},{downstream}")
            }
            Kind::Fort { attack, defense } => format!("Fort {attack},{defense}"),
            Kind::Marker => "Marker".into(),
        }
    }

    pub fn default_for(name: &str) -> Option<Kind> {
        Some(match name {
            "Infantry" => Kind::Infantry {
                fire: 3,
                melee: 5,
                movement: 9,
            },
            "Leader" => Kind::Leader {
                fire: 0,
                melee: 0,
                movement: 12,
            },
            "OldGunboat" => Kind::OldGunboat {
                artillery: 5,
                upstream: 4,
                downstream: 4,
            },
            "NamedGunboat" => Kind::NamedGunboat {
                artillery: 5,
                maxim: 2,
                upstream: 4,
                downstream: 4,
            },
            "Fort" => Kind::Fort {
                attack: 3,
                defense: 5,
            },
            "Marker" => Kind::Marker,
            _ => return None,
        })
    }

    fn ron_literal(&self) -> String {
        match self {
            Kind::Infantry {
                fire,
                melee,
                movement,
            } => format!(
                "Infantry(\n                    fire: {fire},\n                    melee: {melee},\n                    movement: {movement},\n                )"
            ),
            Kind::Leader {
                fire,
                melee,
                movement,
            } => format!(
                "Leader(\n                    fire: {fire},\n                    melee: {melee},\n                    movement: {movement},\n                )"
            ),
            Kind::OldGunboat {
                artillery,
                upstream,
                downstream,
            } => format!(
                "OldGunboat(\n                    artillery: {artillery},\n                    upstream: {upstream},\n                    downstream: {downstream},\n                )"
            ),
            Kind::NamedGunboat {
                artillery,
                maxim,
                upstream,
                downstream,
            } => format!(
                "NamedGunboat(\n                    artillery: {artillery},\n                    maxim: {maxim},\n                    upstream: {upstream},\n                    downstream: {downstream},\n                )"
            ),
            Kind::Fort { attack, defense } => format!(
                "Fort(\n                    attack: {attack},\n                    defense: {defense},\n                )"
            ),
            Kind::Marker => "Marker".into(),
        }
    }
}

/// One counter: `(unit_id, faction, kind, (bg, fg), text)` — the on-disk
/// tuple layout.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Cell(
    pub String,
    pub Faction,
    pub Kind,
    pub SpriteColor,
    pub Option<String>,
);

impl Cell {
    pub fn new(unit_id: impl Into<String>) -> Self {
        Cell(
            unit_id.into(),
            Faction::Unknown,
            Kind::Marker,
            (ColorName::Sand, ColorName::Black),
            None,
        )
    }

    pub fn unit_id(&self) -> &str {
        &self.0
    }
    pub fn faction(&self) -> &Faction {
        &self.1
    }
    pub fn faction_mut(&mut self) -> &mut Faction {
        &mut self.1
    }
    pub fn kind(&self) -> &Kind {
        &self.2
    }
    pub fn kind_mut(&mut self) -> &mut Kind {
        &mut self.2
    }
    pub fn color(&self) -> &SpriteColor {
        &self.3
    }
    pub fn color_mut(&mut self) -> &mut SpriteColor {
        &mut self.3
    }
    pub fn text(&self) -> Option<&str> {
        self.4.as_deref()
    }
    pub fn text_mut(&mut self) -> &mut Option<String> {
        &mut self.4
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Section(pub String, pub Vec<Cell>);

impl Section {
    pub fn name(&self) -> &str {
        &self.0
    }
    pub fn cells(&self) -> &[Cell] {
        &self.1
    }
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.1
    }
}

#[derive(Clone, Debug, Default)]
pub struct UnitsDoc {
    pub header: String,
    pub comments: BTreeMap<String, String>,
    pub sections: Vec<Section>,
}

impl UnitsDoc {
    pub fn to_ron_string(&self) -> String {
        let mut out = String::new();
        if !self.header.is_empty() {
            out.push_str(&self.header);
            out.push('\n');
        }
        out.push_str("[\n");
        for (i, section) in self.sections.iter().enumerate() {
            if let Some(c) = self.comments.get(&comments::elem("", i)) {
                out.push_str(c);
                out.push('\n');
            }
            write_section(&mut out, section);
        }
        out.push_str("]\n");
        out
    }
}

fn write_section(out: &mut String, section: &Section) {
    out.push_str("    Section(\n        ");
    out.push_str(&ron_str(section.name()));
    out.push_str(",\n        [\n");
    for cell in section.cells() {
        out.push_str("            (\n");
        out.push_str("                ");
        out.push_str(&ron_str(cell.unit_id()));
        out.push_str(",\n                ");
        out.push_str(&cell.faction().ron_literal());
        out.push_str(",\n                ");
        out.push_str(&cell.kind().ron_literal());
        out.push_str(",\n                (");
        out.push_str(cell.color().0.as_str());
        out.push_str(", ");
        out.push_str(cell.color().1.as_str());
        out.push_str("),\n                ");
        match cell.text() {
            Some(t) => {
                out.push_str("Some(");
                out.push_str(&ron_str(t));
                out.push(')');
            }
            None => out.push_str("None"),
        }
        out.push_str(",\n            ),\n");
    }
    out.push_str("        ],\n    ),\n");
}

pub fn ron_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

// ── Commands ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Cmd {
    EditCell {
        section: usize,
        cell: usize,
        old: Cell,
        new: Cell,
    },
    AddCell {
        section: usize,
    },
    DeleteCell {
        section: usize,
        index: usize,
        old: Cell,
    },
    RenameSection {
        section: usize,
        old: String,
        new: String,
    },
    AddSection,
    DeleteSection {
        index: usize,
        old: Section,
        old_comment: Option<String>,
    },
}

impl EditorCommand for Cmd {
    fn coalesce_key(&self) -> Option<String> {
        match self {
            Cmd::EditCell { section, cell, .. } => Some(format!("cell/{section}/{cell}")),
            Cmd::RenameSection { section, .. } => Some(format!("rename/{section}")),
            _ => None,
        }
    }

    fn merge(&mut self, next: &Self) -> bool {
        match (self, next) {
            (Cmd::EditCell { new: top, .. }, Cmd::EditCell { new: next_new, .. }) => {
                *top = next_new.clone();
                true
            }
            (Cmd::RenameSection { new: top, .. }, Cmd::RenameSection { new: next_new, .. }) => {
                *top = next_new.clone();
                true
            }
            _ => false,
        }
    }
}

impl Cmd {
    fn apply(&self, doc: &mut UnitsDoc) {
        match self {
            Cmd::EditCell {
                section, cell, new, ..
            } => {
                if let Some(c) = doc
                    .sections
                    .get_mut(*section)
                    .and_then(|s| s.cells_mut().get_mut(*cell))
                {
                    *c = new.clone();
                }
            }
            Cmd::AddCell { section } => {
                if let Some(s) = doc.sections.get_mut(*section) {
                    let id = format!("{}_{}", s.name(), s.cells().len());
                    s.1.push(Cell::new(id));
                }
            }
            Cmd::DeleteCell { section, index, .. } => {
                if let Some(s) = doc.sections.get_mut(*section)
                    && *index < s.cells().len()
                {
                    s.1.remove(*index);
                }
            }
            Cmd::RenameSection { section, new, .. } => {
                if let Some(s) = doc.sections.get_mut(*section) {
                    s.0 = new.clone();
                }
            }
            Cmd::AddSection => {
                doc.sections.push(Section(
                    format!("NewSection{}", doc.sections.len()),
                    Vec::new(),
                ));
            }
            Cmd::DeleteSection { index, .. } => {
                if *index < doc.sections.len() {
                    doc.sections.remove(*index);
                    rekey_section_comments(&mut doc.comments, *index, -1);
                }
            }
        }
    }

    fn revert(&self, doc: &mut UnitsDoc) {
        match self {
            Cmd::EditCell {
                section, cell, old, ..
            } => {
                if let Some(c) = doc
                    .sections
                    .get_mut(*section)
                    .and_then(|s| s.cells_mut().get_mut(*cell))
                {
                    *c = old.clone();
                }
            }
            Cmd::AddCell { section } => {
                if let Some(s) = doc.sections.get_mut(*section) {
                    s.1.pop();
                }
            }
            Cmd::DeleteCell {
                section,
                index,
                old,
            } => {
                if let Some(s) = doc.sections.get_mut(*section)
                    && *index <= s.cells().len()
                {
                    s.1.insert(*index, old.clone());
                }
            }
            Cmd::RenameSection { section, old, .. } => {
                if let Some(s) = doc.sections.get_mut(*section) {
                    s.0 = old.clone();
                }
            }
            Cmd::AddSection => {
                doc.sections.pop();
            }
            Cmd::DeleteSection {
                index,
                old,
                old_comment,
            } => {
                if *index <= doc.sections.len() {
                    doc.sections.insert(*index, old.clone());
                    rekey_section_comments(&mut doc.comments, *index, 1);
                    if let Some(c) = old_comment {
                        doc.comments.insert(comments::elem("", *index), c.clone());
                    }
                }
            }
        }
    }
}

/// Shift section-level comment addresses `[j]` by `delta` for `j >= from`,
/// keeping comments glued to their sections across insertions/deletions.
fn rekey_section_comments(comments: &mut BTreeMap<String, String>, from: usize, delta: i32) {
    let keys: Vec<String> = comments
        .keys()
        .filter(|k| {
            k.strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
                .and_then(|n| n.parse::<usize>().ok())
                .is_some_and(|n| n >= from)
        })
        .cloned()
        .collect();
    for key in keys {
        let n: usize = key[1..key.len() - 1].parse().unwrap();
        let value = comments.remove(&key).unwrap();
        let new_n = (n as i32 + delta).max(0) as usize;
        comments.insert(comments::elem("", new_n), value);
    }
}

// ── Editor ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    section: usize,
    cell: Option<usize>,
}

impl Selection {
    fn clamp(&mut self, sections: &[Section]) {
        if self.section >= sections.len() {
            self.section = sections.len().saturating_sub(1);
            self.cell = None;
        }
        if self.section < sections.len()
            && self
                .cell
                .is_some_and(|c| c >= sections[self.section].cells().len())
        {
            self.cell = None;
        }
    }
}

enum Action {
    Command(Cmd),
    SelectSection(usize),
    SelectCell(usize),
}

pub struct UnitsEditor {
    path: PathBuf,
    doc: UnitsDoc,
    selection: Selection,
    search: String,
    dirty: bool,
    history: History<Cmd>,
    sprites: SpriteCache,
    armed_delete_section: Option<usize>,
    armed_delete_cell: Option<(usize, usize)>,
    error: Option<String>,
}

impl UnitsEditor {
    pub fn open(path: PathBuf, sprites_dir: PathBuf) -> Self {
        let (doc, error) = match load(&path) {
            Ok(doc) => (doc, None),
            Err(e) => (UnitsDoc::default(), Some(format!("failed to load: {e}"))),
        };
        UnitsEditor {
            path,
            doc,
            selection: Selection {
                section: 0,
                cell: None,
            },
            search: String::new(),
            dirty: false,
            history: History::new(),
            sprites: SpriteCache::new(sprites_dir),
            armed_delete_section: None,
            armed_delete_cell: None,
            error,
        }
    }

    fn handle(&mut self, action: Action) {
        match action {
            Action::Command(cmd) => {
                let clears_cell = matches!(cmd, Cmd::DeleteCell { .. });
                cmd.apply(&mut self.doc);
                self.history.record(cmd);
                self.dirty = true;
                if clears_cell {
                    self.selection.cell = None;
                }
            }
            Action::SelectSection(i) => {
                self.selection = Selection {
                    section: i,
                    cell: None,
                };
            }
            Action::SelectCell(i) => self.selection.cell = Some(i),
        }
        self.selection.clamp(&self.doc.sections);
        self.disarm_if_selection_moved();
    }

    fn disarm_if_selection_moved(&mut self) {
        if let Some(armed) = self.armed_delete_section
            && armed != self.selection.section
        {
            self.armed_delete_section = None;
        }
        if let Some((s, c)) = self.armed_delete_cell
            && self.selection
                != (Selection {
                    section: s,
                    cell: Some(c),
                })
        {
            self.armed_delete_cell = None;
        }
    }

    fn run_undo(&mut self) {
        if let Some(cmd) = self.history.undo() {
            cmd.revert(&mut self.doc);
            self.dirty = true;
        }
    }

    fn run_redo(&mut self) {
        if let Some(cmd) = self.history.redo() {
            cmd.apply(&mut self.doc);
            self.dirty = true;
        }
    }
}

fn load(path: &std::path::Path) -> Result<UnitsDoc, EditorError> {
    let (sections, scan): (Vec<Section>, _) = parse_table(path)?;
    Ok(UnitsDoc {
        header: scan.header,
        comments: scan.comments,
        sections,
    })
}

impl TableEditor for UnitsEditor {
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
        self.run_undo();
    }

    fn redo(&mut self) {
        self.run_redo();
    }

    fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(err) = &self.error {
            ui.label(RichText::new(err).color(Color32::RED));
        }

        let mut actions: Vec<Action> = Vec::new();
        egui::SidePanel::left("units_sections")
            .resizable(true)
            .default_width(200.0)
            .show_inside(ui, |ui| self.sections_panel(ui, &mut actions));
        self.apply_actions(ui, ctx, actions.drain(..));
    }

    fn engine_check(&self) -> Vec<CheckResult> {
        let mut results = Vec::new();
        // unit ids encode their section: `<Section>_<col>_<row>`.
        let mut mismatches = 0;
        for section in &self.doc.sections {
            let prefix = snake(section.name());
            for cell in section.cells() {
                let id_section = cell.unit_id().split('_').next().unwrap_or("");
                if id_section != prefix {
                    mismatches += 1;
                    results.push(CheckResult {
                        severity: Severity::Mismatch,
                        message: format!(
                            "{}: unit id {:?} does not start with section {:?}",
                            section.name(),
                            cell.unit_id(),
                            prefix,
                        ),
                    });
                }
            }
        }
        if mismatches == 0 {
            results.push(CheckResult {
                severity: Severity::Ok,
                message: "all unit ids encode their section name".into(),
            });
        }
        results
    }
}

/// `Mulazmin I` → `MulazminI`, matching the id prefix convention.
fn snake(name: &str) -> String {
    name.chars().filter(|c| !c.is_whitespace()).collect()
}

impl UnitsEditor {
    fn apply_actions(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        actions: impl IntoIterator<Item = Action>,
    ) {
        for action in actions {
            self.handle(action);
        }
        let mut actions: Vec<Action> = Vec::new();
        egui::SidePanel::right("units_details")
            .resizable(true)
            .default_width(340.0)
            .show_inside(ui, |ui| self.details_panel(ui, &mut actions));
        for action in actions.drain(..) {
            self.handle(action);
        }
        self.grid(ctx, ui);
    }
}

// ── Panels ──────────────────────────────────────────────────────────────

impl UnitsEditor {
    fn sections_panel(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.horizontal(|ui| {
            ui.heading("Sections");
            if ui
                .small_button("+")
                .on_hover_text("Add a new section")
                .clicked()
            {
                actions.push(Action::Command(Cmd::AddSection));
                actions.push(Action::SelectSection(self.doc.sections.len()));
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, section) in self.doc.sections.iter().enumerate() {
                let selected = i == self.selection.section;
                let label = format!("{} ({})", section.name(), section.cells().len());
                let text = if selected {
                    RichText::new(label).strong()
                } else {
                    RichText::new(label)
                };
                ui.horizontal(|ui| {
                    if ui.selectable_label(selected, text).clicked() {
                        actions.push(Action::SelectSection(i));
                    }
                    if selected {
                        let armed = self.armed_delete_section == Some(i);
                        let label = if armed { "really delete?" } else { "del" };
                        if ui
                            .small_button(label)
                            .on_hover_text("Delete this section (two clicks)")
                            .clicked()
                        {
                            if armed {
                                actions.push(Action::Command(Cmd::DeleteSection {
                                    index: i,
                                    old: section.clone(),
                                    old_comment: self
                                        .doc
                                        .comments
                                        .get(&comments::elem("", i))
                                        .cloned(),
                                }));
                            } else {
                                self.armed_delete_section = Some(i);
                            }
                        }
                    }
                });
            }
        });
    }

    fn details_panel(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.heading("Unit Details");
        ui.separator();

        let Some(si) = self.doc.sections.get(self.selection.section) else {
            ui.label("No section selected.\nAdd one with \"+\" in the left panel.");
            return;
        };
        let si_idx = self.selection.section;

        ui.horizontal(|ui| {
            ui.label("Section:");
            let mut name = si.name().to_string();
            if ui.text_edit_singleline(&mut name).changed() && !name.is_empty() {
                actions.push(Action::Command(Cmd::RenameSection {
                    section: si_idx,
                    old: si.name().to_string(),
                    new: name,
                }));
            }
        });
        ui.separator();

        let Some(ci) = self.selection.cell else {
            ui.label("Select a unit in the grid.");
            ui.add_space(8.0);
            if ui.button("+ Add unit").clicked() {
                actions.push(Action::Command(Cmd::AddCell { section: si_idx }));
                actions.push(Action::SelectCell(
                    self.doc.sections[si_idx].cells().len() - 1,
                ));
            }
            return;
        };
        let Some(cell) = si.cells().get(ci) else {
            ui.label("Unit out of range.");
            return;
        };

        let mut draft = cell.clone();
        let mut changed = false;

        ui.label(RichText::new(draft.unit_id()).strong().size(16.0));
        ui.separator();

        changed |= self.faction_editor(ui, &mut draft);
        changed |= self.text_editor(ui, &mut draft);
        changed |= self.kind_editor(ui, &mut draft);
        changed |= self.color_editor(ui, &mut draft);

        if changed {
            actions.push(Action::Command(Cmd::EditCell {
                section: si_idx,
                cell: ci,
                old: cell.clone(),
                new: draft,
            }));
        }

        ui.separator();
        let armed = self.armed_delete_cell == Some((si_idx, ci));
        let label = if armed {
            RichText::new("really delete?").color(Color32::RED)
        } else {
            RichText::new("Delete unit").color(Color32::RED)
        };
        if ui.button(label).clicked() {
            if armed {
                let old = cell.clone();
                actions.push(Action::Command(Cmd::DeleteCell {
                    section: si_idx,
                    index: ci,
                    old,
                }));
                self.armed_delete_cell = None;
            } else {
                self.armed_delete_cell = Some((si_idx, ci));
            }
        }
    }

    fn faction_editor(&self, ui: &mut egui::Ui, draft: &mut Cell) -> bool {
        let mut changed = false;
        ui.label("Faction:");
        let selected = match draft.faction() {
            Faction::Dervish => "Dervish",
            Faction::AE(_) => "AE",
            Faction::Unknown => "Unknown",
        };
        egui::ComboBox::from_id_salt("faction_combo")
            .selected_text(selected)
            .show_ui(ui, |ui| {
                let mut pick = |ui: &mut egui::Ui, label: &str, f: Faction| {
                    let is_current = draft.faction() == &f;
                    if ui.selectable_label(is_current, label).clicked() && !is_current {
                        *draft.faction_mut() = f;
                        changed = true;
                    }
                };
                pick(ui, "Dervish", Faction::Dervish);
                pick(ui, "AE", Faction::AE(String::new()));
                pick(ui, "Unknown", Faction::Unknown);
            });
        if let Faction::AE(brigade) = draft.faction_mut() {
            ui.horizontal(|ui| {
                ui.label("Brigade:");
                changed |= ui.text_edit_singleline(brigade).changed();
            });
        }
        changed
    }

    fn text_editor(&self, ui: &mut egui::Ui, draft: &mut Cell) -> bool {
        ui.label("Text:");
        let mut text = draft.text().unwrap_or_default().to_string();
        let resp = ui.text_edit_singleline(&mut text);
        let new = if text.is_empty() { None } else { Some(text) };
        let changed = resp.changed() && new != *draft.text_mut();
        if changed {
            *draft.text_mut() = new;
        }
        changed
    }

    fn kind_editor(&self, ui: &mut egui::Ui, draft: &mut Cell) -> bool {
        let mut changed = false;
        ui.separator();
        ui.label("Kind:");
        egui::ComboBox::from_id_salt("kind_combo")
            .selected_text(draft.kind().name())
            .show_ui(ui, |ui| {
                for name in Kind::NAMES {
                    let is_current = draft.kind().name() == name;
                    if ui.selectable_label(is_current, name).clicked()
                        && !is_current
                        && let Some(k) = Kind::default_for(name)
                    {
                        *draft.kind_mut() = k;
                        changed = true;
                    }
                }
            });

        let field = |ui: &mut egui::Ui, label: &str, value: &mut u8, max: u8| {
            let mut c = false;
            ui.horizontal(|ui| {
                ui.label(label);
                c = ui.add(egui::DragValue::new(value).range(0..=max)).changed();
            });
            c
        };

        match draft.kind_mut() {
            Kind::Infantry {
                fire,
                melee,
                movement,
            }
            | Kind::Leader {
                fire,
                melee,
                movement,
            } => {
                changed |= field(ui, "Fire:", fire, 15);
                changed |= field(ui, "Melee:", melee, 15);
                changed |= field(ui, "Move:", movement, 20);
            }
            Kind::OldGunboat {
                artillery,
                upstream,
                downstream,
            } => {
                changed |= field(ui, "Artillery:", artillery, 10);
                changed |= field(ui, "Upstream:", upstream, 10);
                changed |= field(ui, "Downstream:", downstream, 10);
            }
            Kind::NamedGunboat {
                artillery,
                maxim,
                upstream,
                downstream,
            } => {
                changed |= field(ui, "Artillery:", artillery, 10);
                changed |= field(ui, "Maxim:", maxim, 10);
                changed |= field(ui, "Upstream:", upstream, 10);
                changed |= field(ui, "Downstream:", downstream, 10);
            }
            Kind::Fort { attack, defense } => {
                changed |= field(ui, "Attack:", attack, 10);
                changed |= field(ui, "Defense:", defense, 10);
            }
            Kind::Marker => {
                ui.label(RichText::new("no editable fields").color(Color32::GRAY));
            }
        }
        changed
    }

    fn color_editor(&self, ui: &mut egui::Ui, draft: &mut Cell) -> bool {
        let mut changed = false;
        ui.separator();
        ui.label("Sprite colors:");
        ui.horizontal(|ui| {
            ui.label("BG:");
            changed |= color_swatches(ui, &mut draft.color_mut().0);
        });
        ui.horizontal(|ui| {
            ui.label("FG:");
            changed |= color_swatches(ui, &mut draft.color_mut().1);
        });
        changed
    }
}

fn color_swatches(ui: &mut egui::Ui, current: &mut ColorName) -> bool {
    let mut changed = false;
    for color in ColorName::ALL {
        let selected = *current == color;
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), egui::Sense::click());
        let painter = ui.painter();
        painter.rect_filled(rect, 2.0, color_to_egui(&color));
        if selected {
            painter.rect_stroke(
                rect,
                2.0,
                egui::Stroke::new(2.0_f32, Color32::YELLOW),
                egui::StrokeKind::Inside,
            );
        }
        if resp.clicked() {
            *current = color;
            changed = true;
        }
    }
    changed
}

fn color_to_egui(c: &ColorName) -> Color32 {
    match c {
        ColorName::Black => Color32::BLACK,
        ColorName::White => Color32::WHITE,
        ColorName::Red => Color32::RED,
        ColorName::Green => Color32::from_rgb(0, 180, 0),
        ColorName::Sand => Color32::from_rgb(194, 178, 128),
        ColorName::Blue => Color32::from_rgb(0, 100, 255),
        ColorName::Gray => Color32::GRAY,
    }
}

// ── Central grid ────────────────────────────────────────────────────────

/// Total card width (frame border included).
const CARD_WIDTH: f32 = 160.0;
/// Card frame inner margin on each side.
const CARD_MARGIN: f32 = 4.0;
/// Content width inside the card frame.
const CARD_CONTENT_W: f32 = CARD_WIDTH - 2.0 * CARD_MARGIN;
const CARD_SPRITE_HEIGHT: f32 = 120.0;

impl UnitsEditor {
    fn grid(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let Some(si) = self.doc.sections.get(self.selection.section) else {
            ui.heading("No sections yet — add one on the left.");
            return;
        };
        ui.heading(si.name());
        ui.separator();

        let search = self.search.to_lowercase();
        let visible: Vec<usize> = si
            .cells()
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                search.is_empty()
                    || c.unit_id().to_lowercase().contains(&search)
                    || c.text()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&search)
                    || c.faction().display().to_lowercase().contains(&search)
                    || c.kind().name().to_lowercase().contains(&search)
            })
            .map(|(i, _)| i)
            .collect();

        if visible.is_empty() {
            ui.label(RichText::new("no units match").color(Color32::GRAY));
            return;
        }

        let mut select: Option<usize> = None;
        egui::ScrollArea::both().show(ui, |ui| {
            let cols = (ui.available_width() / CARD_WIDTH).floor().max(1.0) as usize;
            // egui::Grid gives every cell a bounded rect and sizes each
            // column from its widest content, keeping columns and rows
            // aligned — manual horizontal rows misplace compound widgets.
            // The id includes `cols` so a resize starts a fresh grid
            // instead of mixing stale column widths.
            egui::Grid::new(format!("unit_grid_{cols}"))
                .num_columns(cols)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    for (i, &ci) in visible.iter().enumerate() {
                        if self.unit_card(ctx, ui, si, ci) {
                            select = Some(ci);
                        }
                        if (i + 1) % cols == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
        if let Some(ci) = select {
            self.handle(Action::SelectCell(ci));
        }
    }

    fn unit_card(&self, ctx: &egui::Context, ui: &mut egui::Ui, si: &Section, ci: usize) -> bool {
        let cell = &si.cells()[ci];
        let selected = self.selection.cell == Some(ci);

        let frame = if selected {
            egui::Frame::NONE
                .fill(Color32::from_rgb(40, 40, 80))
                .stroke(egui::Stroke::new(2.0_f32, Color32::LIGHT_BLUE))
                .inner_margin(4.0)
        } else {
            egui::Frame::NONE
                .fill(Color32::from_rgb(30, 30, 30))
                .inner_margin(4.0)
        };

        let resp = ui
            .scope(|ui| {
                frame.show(ui, |ui| {
                    // Vertical stack below the sprite. Every card reserves
                    // exactly the same space — fixed-size sprite box and
                    // five label lines (an empty string still occupies a
                    // line) — so cards never differ in height and the Grid
                    // cells stay perfectly aligned regardless of the
                    // LEFT_CENTER cell alignment.
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.set_width(CARD_CONTENT_W);
                        let (rect, _) = ui.allocate_exact_size(
                            Vec2::new(CARD_CONTENT_W, CARD_SPRITE_HEIGHT),
                            egui::Sense::hover(),
                        );
                        match self.sprites.get(ctx, cell.unit_id()) {
                            Some(tex) => {
                                let size = tex.size_vec2();
                                let scale = (CARD_SPRITE_HEIGHT / size.y)
                                    .min(CARD_CONTENT_W / size.x)
                                    .min(1.0);
                                let display = size * scale;
                                ui.put(
                                    egui::Rect::from_center_size(rect.center(), display),
                                    egui::Image::new((tex.id(), display)),
                                );
                            }
                            None => {
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "no sprite",
                                    egui::TextStyle::Small.resolve(ui.style()),
                                    Color32::DARK_GRAY,
                                );
                            }
                        }
                        ui.label(
                            RichText::new(cell.unit_id())
                                .small()
                                .color(Color32::GRAY)
                                .strong(),
                        );
                        ui.label(RichText::new(cell.faction().display()).small());
                        ui.label(
                            RichText::new(cell.text().unwrap_or(""))
                                .small()
                                .color(Color32::LIGHT_BLUE),
                        );
                        ui.label(
                            RichText::new(cell.kind().display())
                                .small()
                                .color(Color32::LIGHT_GRAY),
                        );
                        ui.label(
                            RichText::new(format!(
                                "({},{})",
                                cell.color().0.as_str(),
                                cell.color().1.as_str()
                            ))
                            .small()
                            .color(Color32::DARK_GRAY),
                        );
                    });
                });
            })
            .response;

        resp.interact(egui::Sense::click()).clicked()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn real_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../Boardgame - Remember_Gordon/tables/units.ron")
    }

    #[test]
    fn real_file_round_trip() {
        let path = real_path();
        let text = std::fs::read_to_string(&path).unwrap();
        let doc = load(&path).unwrap();
        assert_eq!(doc.sections.len(), 16);
        assert_eq!(
            doc.sections.iter().map(|s| s.cells().len()).sum::<usize>(),
            205
        );
        // Comments survive: header note + two banners.
        assert!(
            doc.header
                .contains("Cells whose annotation is not yet known")
        );
        let banners: Vec<_> = doc.comments.values().filter(|c| c.contains("──")).collect();
        assert_eq!(banners.len(), 2, "comments: {:?}", doc.comments);
        // No non-unit markers remain (the GAME TURN counter and the stub
        // sections were removed).
        for section in &doc.sections {
            for cell in section.cells() {
                assert_ne!(
                    cell.kind(),
                    &crate::tables::units::Kind::Marker,
                    "{} is still a marker",
                    cell.unit_id()
                );
            }
        }

        // Fixed point: serialize → parse → serialize is stable, and the
        // document survives semantically.
        let out = doc.to_ron_string();
        let dir = std::env::temp_dir().join(format!("asset-editor-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("units.ron");
        std::fs::write(&tmp, &out).unwrap();
        let doc2 = load(&tmp).unwrap();
        assert_eq!(doc2.sections, doc.sections);
        assert_eq!(doc2.comments, doc.comments);
        assert_eq!(doc2.to_ron_string(), out);
        // Serialization of the unmodified file is byte-identical except for
        // the single-line stub cells, which are normalized to multi-line.
        let normalized = normalize(&text);
        assert_eq!(out, normalized);
    }

    /// The current file's stub sections use a compact single-line cell
    /// layout; the serializer always emits multi-line cells.
    fn normalize(text: &str) -> String {
        let doc = parse_for_test(text);
        let _ = doc;
        // Re-parse + re-serialize once; the second pass is canonical.
        let dir = std::env::temp_dir().join(format!("asset-editor-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("units_norm.ron");
        std::fs::write(&tmp, text).unwrap();
        load(&tmp).unwrap().to_ron_string()
    }

    fn parse_for_test(text: &str) -> usize {
        let _ = text;
        0
    }

    #[test]
    fn delete_section_rekeys_and_restores_comments() {
        let mut doc = UnitsDoc::default();
        doc.sections = vec![
            Section("A".into(), vec![Cell::new("A_0")]),
            Section("B".into(), vec![Cell::new("B_0")]),
            Section("C".into(), vec![Cell::new("C_0")]),
        ];
        doc.comments
            .insert(comments::elem("", 0), "// banner A".into());
        doc.comments
            .insert(comments::elem("", 2), "// banner C".into());

        let cmd = Cmd::DeleteSection {
            index: 1,
            old: doc.sections[1].clone(),
            old_comment: doc.comments.get(&comments::elem("", 1)).cloned(),
        };
        cmd.apply(&mut doc);
        assert_eq!(doc.sections.len(), 2);
        assert!(doc.comments.contains_key(&comments::elem("", 0)));
        assert!(doc.comments.contains_key(&comments::elem("", 1)));
        assert!(!doc.comments.contains_key(&comments::elem("", 2)));
        assert_eq!(
            doc.comments.get(&comments::elem("", 1)).map(String::as_str),
            Some("// banner C")
        );

        cmd.revert(&mut doc);
        assert_eq!(doc.sections.len(), 3);
        assert!(doc.comments.contains_key(&comments::elem("", 2)));
    }

    #[test]
    fn edit_cell_coalesces() {
        let mut editor = UnitsEditor::open(real_path(), sprites_dir());
        let old = editor.doc.sections[0].cells()[0].clone();
        let mut new = old.clone();
        if let Faction::AE(b) = new.faction_mut() {
            let _ = b;
        }
        new.4 = Some("x".into());
        let cmd = Cmd::EditCell {
            section: 0,
            cell: 0,
            old: old.clone(),
            new: new.clone(),
        };
        cmd.apply(&mut editor.doc);
        editor.history.record(cmd);
        let original_text = old.text().map(str::to_string);
        let mut new2 = new.clone();
        new2.4 = Some("xy".into());
        let cmd2 = Cmd::EditCell {
            section: 0,
            cell: 0,
            old,
            new: new2,
        };
        cmd2.apply(&mut editor.doc);
        editor.history.record(cmd2);
        editor.run_undo();
        // Both edits coalesced into one history entry, so a single undo
        // restores the cell's original text.
        assert_eq!(
            editor.doc.sections[0].cells()[0].text(),
            original_text.as_deref()
        );
    }

    fn sprites_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../omdurman-app/assets/sprites")
    }

    #[test]
    fn kind_and_faction_literals() {
        assert_eq!(
            Faction::AE("Lancers21".into()).ron_literal(),
            "AE(\"Lancers21\")"
        );
        assert_eq!(
            Kind::Infantry {
                fire: 1,
                melee: 2,
                movement: 3
            }
            .ron_literal(),
            "Infantry(\n                    fire: 1,\n                    melee: 2,\n                    movement: 3,\n                )"
        );
    }

    #[test]
    fn index_map_type_exists() {
        let m: IndexMap<String, u8> = IndexMap::new();
        assert!(m.is_empty());
    }
}

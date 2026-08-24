//! `order_of_appearance.ron` — Campaign reinforcement schedule (§9.112/§9.113).

use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, RichText};
use indexmap::IndexMap;

use crate::common::command::{EditorCommand, History};
use crate::common::{parse_table, save_atomic, CheckResult, EditorError, Severity, TableEditor};

pub const LEADERS: [&str; 5] = [
    "Yakub",
    "Sherif",
    "AliWadHelu",
    "OsmanDigna",
    "SheikElDin",
];

// ── Model ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AeWave {
    pub turn: u8,
    pub time: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gunboats: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub friendlies: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub land: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub land_any: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_remaining: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DervishSection {
    pub leader: String,
    pub units: IndexMap<String, u16>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DervishWave {
    pub turn: u8,
    pub time: String,
    pub sections: Vec<DervishSection>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppearDoc {
    pub header: String,
    pub comments: std::collections::BTreeMap<String, String>,
    pub ae: Vec<AeWave>,
    pub dervish: Vec<DervishWave>,
}

impl AppearDoc {
    pub fn to_ron_string(&self) -> String {
        let mut out = String::new();
        if !self.header.is_empty() {
            out.push_str(&self.header);
            out.push('\n');
        }
        out.push_str("(\n");
        if let Some(c) = self.comments.get("AngloEgyptian") {
            out.push_str(c);
            out.push('\n');
        }
        out.push_str("    AngloEgyptian: [\n");
        for wave in &self.ae {
            out.push_str(&ae_wave_line(wave, self));
        }
        out.push_str("    ],\n");
        if let Some(c) = self.comments.get("Dervish") {
            out.push_str(c);
            out.push('\n');
        }
        out.push_str("    Dervish: [\n");
        for wave in &self.dervish {
            out.push_str(&dervish_wave_lines(wave));
        }
        out.push_str("    ],\n");
        out.push_str(")\n");
        out
    }
}

fn ae_wave_line(wave: &AeWave, doc: &AppearDoc) -> String {
    // Waves with a `land:` list wrap it onto its own lines.
    match &wave.land {
        None => {
            let mut s = format!(
                "        (turn: {}, time: {}, gunboats: {}, friendlies: {}, land_any: {}, all_remaining: {}),\n",
                wave.turn,
                quote(&wave.time),
                opt_u8(wave.gunboats),
                opt_u8(wave.friendlies),
                opt_u16(wave.land_any),
                opt_bool(wave.all_remaining),
            );
            s = s.replace(", gunboats: None", "");
            s = s.replace(", friendlies: None", "");
            s = s.replace(", land_any: None", "");
            s = s.replace(", all_remaining: None", "");
            s
        }
        Some(land) => {
            let _ = doc;
            let mut s = format!(
                "        (turn: {}, time: {}, gunboats: {}, friendlies: {}, land: [\n",
                wave.turn,
                quote(&wave.time),
                opt_u8(wave.gunboats),
                opt_u8(wave.friendlies),
            );
            s = s.replace(", gunboats: None", "");
            s = s.replace(", friendlies: None", "");
            for item in land {
                s.push_str(&format!("            {},\n", quote(item)));
            }
            s.push_str("        ]),\n");
            s
        }
    }
}

fn dervish_wave_lines(wave: &DervishWave) -> String {
    if wave.sections.is_empty() {
        return format!(
            "        (turn: {}, time: {}, sections: []),\n",
            wave.turn,
            quote(&wave.time),
        );
    }
    let mut s = format!(
        "        (turn: {}, time: {}, sections: [\n",
        wave.turn,
        quote(&wave.time),
    );
    for sec in &wave.sections {
        let units: Vec<String> = sec
            .units
            .iter()
            .map(|(k, v)| format!("\"{k}\": {v}"))
            .collect();
        s.push_str(&format!(
            "            (leader: {}, units: {{ {} }}),\n",
            quote(&sec.leader),
            units.join(", ")
        ));
    }
    s.push_str("        ]),\n");
    s
}

fn opt_u8(v: Option<u8>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "None".into())
}
fn opt_u16(v: Option<u16>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "None".into())
}
fn opt_bool(v: Option<bool>) -> String {
    v.map(|b| b.to_string()).unwrap_or_else(|| "None".into())
}
fn quote(s: &str) -> String {
    format!("{s:?}")
}

// ── Deserialization shape ───────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct RawDoc {
    #[serde(rename = "AngloEgyptian")]
    anglo_egyptian: Vec<AeWave>,
    #[serde(rename = "Dervish")]
    dervish: Vec<DervishWave>,
}

// ── Commands ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Cmd {
    SetAe { index: usize, old: AeWave, new: AeWave },
    SetDervish { index: usize, old: DervishWave, new: DervishWave },
    AddAe,
    AddDervish,
    DeleteAe { index: usize, old: AeWave },
    DeleteDervish { index: usize, old: DervishWave },
}

impl EditorCommand for Cmd {
    fn coalesce_key(&self) -> Option<String> {
        match self {
            Cmd::SetAe { index, .. } => Some(format!("ae/{index}")),
            Cmd::SetDervish { index, .. } => Some(format!("dervish/{index}")),
            _ => None,
        }
    }

    fn merge(&mut self, next: &Self) -> bool {
        match (self, next) {
            (Cmd::SetAe { new, .. }, Cmd::SetAe { new: n, .. }) => {
                *new = n.clone();
                true
            }
            (Cmd::SetDervish { new, .. }, Cmd::SetDervish { new: n, .. }) => {
                *new = n.clone();
                true
            }
            _ => false,
        }
    }
}

impl Cmd {
    fn apply(&self, doc: &mut AppearDoc) {
        match self {
            Cmd::SetAe { index, new, .. } => {
                if let Some(w) = doc.ae.get_mut(*index) {
                    *w = new.clone();
                }
            }
            Cmd::SetDervish { index, new, .. } => {
                if let Some(w) = doc.dervish.get_mut(*index) {
                    *w = new.clone();
                }
            }
            Cmd::AddAe => doc.ae.push(AeWave {
                turn: (doc.ae.len() + 1) as u8,
                time: String::new(),
                ..Default::default()
            }),
            Cmd::AddDervish => doc.dervish.push(DervishWave {
                turn: (doc.dervish.len() + 1) as u8,
                time: String::new(),
                sections: Vec::new(),
            }),
            Cmd::DeleteAe { index, .. } => {
                if *index < doc.ae.len() {
                    doc.ae.remove(*index);
                }
            }
            Cmd::DeleteDervish { index, .. } => {
                if *index < doc.dervish.len() {
                    doc.dervish.remove(*index);
                }
            }
        }
    }

    fn revert(&self, doc: &mut AppearDoc) {
        match self {
            Cmd::SetAe { index, old, .. } => {
                if let Some(w) = doc.ae.get_mut(*index) {
                    *w = old.clone();
                }
            }
            Cmd::SetDervish { index, old, .. } => {
                if let Some(w) = doc.dervish.get_mut(*index) {
                    *w = old.clone();
                }
            }
            Cmd::AddAe => {
                doc.ae.pop();
            }
            Cmd::AddDervish => {
                doc.dervish.pop();
            }
            Cmd::DeleteAe { index, old } => {
                if *index <= doc.ae.len() {
                    doc.ae.insert(*index, old.clone());
                }
            }
            Cmd::DeleteDervish { index, old } => {
                if *index <= doc.dervish.len() {
                    doc.dervish.insert(*index, old.clone());
                }
            }
        }
    }
}

// ── Editor ──────────────────────────────────────────────────────────────

pub struct AppearEditor {
    path: PathBuf,
    doc: AppearDoc,
    dirty: bool,
    history: History<Cmd>,
    error: Option<String>,
}

impl AppearEditor {
    pub fn open(path: PathBuf) -> Self {
        let (doc, error) = match load(&path) {
            Ok(doc) => (doc, None),
            Err(e) => (AppearDoc::default(), Some(format!("failed to load: {e}"))),
        };
        AppearEditor {
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

fn load(path: &std::path::Path) -> Result<AppearDoc, EditorError> {
    let (raw, scan): (RawDoc, _) = parse_table(path)?;
    Ok(AppearDoc {
        header: scan.header,
        comments: scan.comments,
        ae: raw.anglo_egyptian,
        dervish: raw.dervish,
    })
}

impl TableEditor for AppearEditor {
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
        ui.heading("Campaign Order of Appearance (§9.112/§9.113)");
        ui.separator();

        let mut cmds: Vec<Cmd> = Vec::new();
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.ae_section(ui, &mut cmds);
            ui.separator();
            self.dervish_section(ui, &mut cmds);
        });
        for cmd in cmds {
            self.run(cmd);
        }
        self.warnings(ui);
    }

    fn engine_check(&self) -> Vec<CheckResult> {
        use omdurman_rules::reinforcements::{
            anglo_egyptian_campaign_schedule, dervish_campaign_schedule,
        };

        let mut results = Vec::new();
        let mut check_side = |waves: &[u8], engine_turns: &[u8], name: &str| {
            if waves == engine_turns {
                results.push(CheckResult {
                    severity: Severity::Ok,
                    message: format!("{name} wave turns match the engine schedule"),
                });
            } else {
                results.push(CheckResult {
                    severity: Severity::Mismatch,
                    message: format!(
                        "{name} wave turns: table {waves:?}, engine {engine_turns:?}"
                    ),
                });
            }
        };

        let ae_schedule = anglo_egyptian_campaign_schedule();
        let d_schedule = dervish_campaign_schedule();
        check_side(
            &self.doc.ae.iter().map(|w| w.turn).collect::<Vec<_>>(),
            &ae_schedule.waves.iter().map(|w| w.turn).collect::<Vec<_>>(),
            "AE",
        );
        check_side(
            &self.doc.dervish.iter().map(|w| w.turn).collect::<Vec<_>>(),
            &d_schedule.waves.iter().map(|w| w.turn).collect::<Vec<_>>(),
            "Dervish",
        );

        // Gunboat counts per turn.
        let engine_gunboats: Vec<(u8, usize)> = ae_schedule
            .waves
            .iter()
            .filter_map(|w| w.unit_cap.map(|c| (w.turn, c)))
            .collect();
        let _ = engine_gunboats;
        results.push(CheckResult {
            severity: Severity::Info,
            message: "unit caps and section compositions are not compared cell-by-cell yet"
                .into(),
        });
        results
    }
}

impl AppearEditor {
    fn ae_section(&mut self, ui: &mut egui::Ui, cmds: &mut Vec<Cmd>) {
        ui.label(RichText::new("Anglo-Egyptian (§9.113)").strong());
        let waves: Vec<AeWave> = self.doc.ae.clone();
        for (index, wave) in waves.iter().enumerate() {
            let mut draft = wave.clone();
            egui::Frame::NONE
                .fill(Color32::from_rgb(30, 30, 30))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(format!("turn {}", draft.turn));
                        let mut turn = draft.turn;
                        if ui
                            .add(egui::DragValue::new(&mut turn).range(1..=20))
                            .changed()
                        {
                            draft.turn = turn;
                        }
                        ui.label("time:");
                        ui.text_edit_singleline(&mut draft.time);

                        optional_u8(ui, "gunboats", &mut draft.gunboats);
                        optional_u8(ui, "friendlies", &mut draft.friendlies);
                        if ui
                            .checkbox(
                                draft.all_remaining.get_or_insert(false),
                                "all remaining",
                            )
                            .changed()
                        {
                            // checkbox toggle already wrote through.
                        } else if draft.all_remaining == Some(false) {
                            draft.all_remaining = None;
                        }
                        if ui.small_button("✕").clicked() {
                            cmds.push(Cmd::DeleteAe {
                                index,
                                old: wave.clone(),
                            });
                        }
                    });

                    // land list
                    let land = draft.land.get_or_insert_with(Vec::new);
                    ui.horizontal(|ui| {
                        ui.label("land:");
                        if ui.small_button("+").clicked() {
                            land.push(String::new());
                        }
                    });
                    let mut remove: Option<usize> = None;
                    for (li, item) in land.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.small_button("✕").clicked() {
                                remove = Some(li);
                            }
                            ui.add(
                                egui::TextEdit::singleline(item)
                                    .desired_width(360.0)
                                    .clip_text(true),
                            );
                        });
                    }
                    if let Some(li) = remove {
                        land.remove(li);
                    }
                    if land.is_empty() {
                        draft.land = None;
                    }

                    // land_any only when there is no explicit list
                    if draft.land.is_none() {
                        let mut any = draft.land_any.unwrap_or(0);
                        let mut set = draft.land_any.is_some();
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut set, "land_any").changed() {
                                draft.land_any = set.then_some(any);
                            }
                            ui.add_enabled(
                                set,
                                egui::DragValue::new(&mut any).range(0..=60),
                            );
                            if set {
                                draft.land_any = Some(any);
                            }
                        });
                    }
                });
            if draft != *wave {
                cmds.push(Cmd::SetAe {
                    index,
                    old: wave.clone(),
                    new: draft,
                });
            }
        }
        if ui.button("+ add AE wave").clicked() {
            cmds.push(Cmd::AddAe);
        }
    }

    fn dervish_section(&mut self, ui: &mut egui::Ui, cmds: &mut Vec<Cmd>) {
        ui.label(RichText::new("Dervish (§9.112)").strong());
        let waves: Vec<DervishWave> = self.doc.dervish.clone();
        for (index, wave) in waves.iter().enumerate() {
            let mut draft = wave.clone();
            egui::Frame::NONE
                .fill(Color32::from_rgb(30, 30, 30))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(format!("turn {}", draft.turn));
                        let mut turn = draft.turn;
                        if ui
                            .add(egui::DragValue::new(&mut turn).range(1..=20))
                            .changed()
                        {
                            draft.turn = turn;
                        }
                        ui.label("time:");
                        ui.text_edit_singleline(&mut draft.time);
                        if ui.small_button("✕").clicked() {
                            cmds.push(Cmd::DeleteDervish {
                                index,
                                old: wave.clone(),
                            });
                        }
                    });

                    let mut remove: Option<usize> = None;
                    for (si, sec) in draft.sections.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            let id = egui::Id::new(("appear_leader", index, si));
                            let options: Vec<(String, String)> = LEADERS
                                .iter()
                                .map(|l| (l.to_string(), l.to_string()))
                                .collect();
                            if let Some(pick) =
                                crate::common::dropdown_cell(ui, id, &sec.leader, &options)
                            {
                                sec.leader = options[pick].0.clone();
                            }
                            let mut entries: Vec<(String, u16)> = sec
                                .units
                                .iter()
                                .map(|(k, v)| (k.clone(), *v))
                                .collect();
                            let mut drop: Option<usize> = None;
                            for (ui_i, (tribe, count)) in entries.iter_mut().enumerate() {
                                ui.label(tribe.as_str());
                                let mut c = *count;
                                if ui
                                    .add(egui::DragValue::new(&mut c).range(0..=60))
                                    .changed()
                                {
                                    *count = c;
                                }
                                if ui.small_button("✕").clicked() {
                                    drop = Some(ui_i);
                                }
                            }
                            if let Some(di) = drop {
                                entries.remove(di);
                                sec.units = entries.into_iter().collect();
                            } else {
                                sec.units = entries.into_iter().collect();
                            }
                            if ui.small_button("+ tribe").clicked() {
                                sec.units.insert("NewTribe".into(), 0);
                            }
                            if ui.small_button("✕ section").clicked() {
                                remove = Some(si);
                            }
                        });
                    }
                    if let Some(si) = remove {
                        draft.sections.remove(si);
                    }
                    if ui.button("+ add section").clicked() {
                        draft
                            .sections
                            .push(DervishSection { leader: "Yakub".into(), units: IndexMap::new() });
                    }
                });
            if draft != *wave {
                cmds.push(Cmd::SetDervish {
                    index,
                    old: wave.clone(),
                    new: draft,
                });
            }
        }
        if ui.button("+ add Dervish wave").clicked() {
            cmds.push(Cmd::AddDervish);
        }
    }

    fn warnings(&self, ui: &mut egui::Ui) {
        for (name, turns) in [
            ("AE", self.doc.ae.iter().map(|w| w.turn).collect::<Vec<_>>()),
            ("Dervish", self.doc.dervish.iter().map(|w| w.turn).collect::<Vec<_>>()),
        ] {
            let mut sorted = turns.clone();
            sorted.sort_unstable();
            if sorted != turns {
                ui.label(
                    RichText::new(format!("⚠ {name} turns are not in ascending order"))
                        .color(Color32::YELLOW),
                );
            }
            if sorted.windows(2).any(|w| w[0] == w[1]) {
                ui.label(
                    RichText::new(format!("⚠ {name} has duplicate turns")).color(Color32::YELLOW),
                );
            }
        }
    }
}

fn optional_u8(ui: &mut egui::Ui, label: &str, value: &mut Option<u8>) {
    let mut set = value.is_some();
    let mut v = value.unwrap_or(0);
    ui.horizontal(|ui| {
        if ui.checkbox(&mut set, label).changed() {
            *value = set.then_some(v);
        }
        ui.add_enabled(set, egui::DragValue::new(&mut v).range(0..=30));
        if set {
            *value = Some(v);
        }
    });
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn real_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../Boardgame - Remember_Gordon/tables/order_of_appearance.ron")
    }

    #[test]
    fn real_file_round_trip_is_byte_identical() {
        let path = real_path();
        let original = std::fs::read_to_string(&path).unwrap();
        let doc = load(&path).unwrap();
        assert_eq!(doc.ae.len(), 4);
        assert_eq!(doc.dervish.len(), 4);
        assert_eq!(doc.ae[0].gunboats, Some(3));
        assert_eq!(doc.ae[0].friendlies, Some(5));
        assert_eq!(doc.ae[0].land.as_ref().map(Vec::len), Some(3));
        assert_eq!(doc.ae[1].land_any, Some(12));
        assert_eq!(doc.ae[3].all_remaining, Some(true));
        assert_eq!(doc.dervish[0].sections[0].leader, "Yakub");
        assert_eq!(doc.dervish[0].sections[0].units["Baggara"], 12);
        assert!(doc.header.contains("§9.112"));
        assert_eq!(doc.to_ron_string(), original);
    }

    #[test]
    fn set_wave_undo() {
        let mut ed = AppearEditor::open(real_path());
        let old = ed.doc.ae[1].clone();
        let mut new = old.clone();
        new.land_any = Some(20);
        ed.run(Cmd::SetAe { index: 1, old, new });
        assert_eq!(ed.doc.ae[1].land_any, Some(20));
        ed.undo();
        assert_eq!(ed.doc.ae[1].land_any, Some(12));
    }
}

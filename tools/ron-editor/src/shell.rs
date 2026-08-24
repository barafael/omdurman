//! Application shell: table switcher, top bar, save/undo/redo routing,
//! unsaved-changes dialog, engine-check window.

use std::path::PathBuf;
use std::time::Instant;

use eframe::egui;
use egui::{Color32, RichText};

use crate::common::{CheckResult, Severity, TableEditor, TableKind};
use crate::tables::Editors;

pub struct Shell {
    tables_dir: PathBuf,
    active: TableKind,
    editors: Editors,
    /// Switch requested while the active editor is dirty; needs a decision.
    pending_switch: Option<TableKind>,
    status: Option<(String, Instant)>,
    error: Option<String>,
    show_engine_check: bool,
}

impl Shell {
    pub fn new(tables_dir: PathBuf, sprites_dir: PathBuf, initial: TableKind) -> Self {
        Shell {
            editors: Editors::new(tables_dir.clone(), sprites_dir),
            tables_dir,
            active: initial,
            pending_switch: None,
            status: None,
            error: None,
            show_engine_check: false,
        }
    }

    fn note(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), Instant::now()));
    }

    fn request_switch(&mut self, kind: TableKind) {
        if kind == self.active && self.pending_switch.is_none() {
            return;
        }
        if self.editors.get(self.active).dirty() {
            self.pending_switch = Some(kind);
        } else {
            self.active = kind;
        }
    }

    fn save_active(&mut self) {
        let result = self.editors.get(self.active).save();
        match result {
            Ok(()) => {
                self.error = None;
                self.note(format!("saved {}", self.active.file_name()));
            }
            Err(e) => self.error = Some(format!("save failed: {e}")),
        }
    }
}

impl eframe::App for Shell {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.shortcuts(ctx);

        // Snapshot editor state so panels don't hold a borrow of self.
        let (dirty, can_undo, can_redo) = {
            let editor = self.editors.get(self.active);
            (editor.dirty(), editor.can_undo(), editor.can_redo())
        };

        let mut do_save = false;
        let mut do_undo = false;
        let mut do_redo = false;

        egui::TopBottomPanel::top("shell_top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save (Ctrl+S)").clicked() {
                    do_save = true;
                }
                ui.add_enabled_ui(can_undo, |ui| {
                    if ui.button("Undo (Ctrl+Z)").clicked() {
                        do_undo = true;
                    }
                });
                ui.add_enabled_ui(can_redo, |ui| {
                    if ui.button("Redo (Ctrl+Shift+Z)").clicked() {
                        do_redo = true;
                    }
                });
                if dirty {
                    ui.label(RichText::new("*modified*").color(Color32::YELLOW));
                }
                ui.separator();
                let mut check = self.show_engine_check;
                if ui
                    .selectable_label(check, "Engine check")
                    .on_hover_text("compare this table against the compiled rules engine")
                    .clicked()
                {
                    check = !check;
                }
                self.show_engine_check = check;

                if let Some((msg, at)) = &self.status
                    && at.elapsed().as_secs() < 3
                {
                    ui.label(RichText::new(msg).color(Color32::LIGHT_GREEN));
                }
                if let Some(err) = &self.error {
                    ui.label(RichText::new(err).color(Color32::RED));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "tables/{}",
                            self.active.file_name()
                        ))
                        .color(Color32::GRAY),
                    );
                });
            });
        });

        if do_save {
            self.save_active();
        }
        if do_undo {
            self.editors.get(self.active).undo();
        }
        if do_redo {
            self.editors.get(self.active).redo();
        }

        let dirty_flags: Vec<(TableKind, bool)> = TableKind::ALL
            .into_iter()
            .map(|k| (k, self.editors.get(k).dirty()))
            .collect();
        let mut switch_to: Option<TableKind> = None;
        egui::SidePanel::left("shell_tables")
            .resizable(false)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Tables");
                ui.separator();
                for (kind, kind_dirty) in dirty_flags {
                    let selected = kind == self.active;
                    let label = if kind_dirty {
                        format!("* {}", kind.display())
                    } else {
                        kind.display().to_string()
                    };
                    let text = if selected {
                        RichText::new(label).strong()
                    } else {
                        RichText::new(label)
                    };
                    if ui.selectable_label(selected, text).clicked() {
                        switch_to = Some(kind);
                    }
                }
            });
        if let Some(kind) = switch_to {
            self.request_switch(kind);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let editor = self.editors.get(self.active);
            editor.show(ctx, ui);
        });

        if self.show_engine_check {
            self.engine_check_window(ctx);
        }
        self.switch_dialog(ctx);
    }
}

impl Shell {
    fn shortcuts(&mut self, ctx: &egui::Context) {
        let pressed = |key, ctrl, shift| {
            ctx.input(|i| {
                i.key_pressed(key) && i.modifiers.ctrl == ctrl && i.modifiers.shift == shift
            })
        };
        if pressed(egui::Key::S, true, false) {
            self.save_active();
        }
        if pressed(egui::Key::Z, true, false) {
            self.editors.get(self.active).undo();
        }
        if pressed(egui::Key::Z, true, true) || pressed(egui::Key::Y, true, false) {
            self.editors.get(self.active).redo();
        }
    }

    fn engine_check_window(&mut self, ctx: &egui::Context) {
        let results = self.editors.get(self.active).engine_check();
        egui::Window::new(format!("Engine check — {}", self.active.display()))
            .open(&mut self.show_engine_check)
            .default_width(480.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for result in &results {
                        let (color, tag) = match result.severity {
                            Severity::Mismatch => (Color32::RED, "MISMATCH"),
                            Severity::Info => (Color32::YELLOW, "info"),
                            Severity::Ok => (Color32::LIGHT_GREEN, "ok"),
                        };
                        ui.label(
                            RichText::new(format!("[{tag}] {}", result.message))
                                .color(color),
                        );
                    }
                });
            });
    }

    fn switch_dialog(&mut self, ctx: &egui::Context) {
        let Some(target) = self.pending_switch else {
            return;
        };
        let mut decision: Option<Decision> = None;
        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!(
                    "{} has unsaved changes. Save before switching to {}?",
                    self.active.file_name(),
                    target.display()
                ));
                ui.horizontal(|ui| {
                    if ui.button("Save and switch").clicked() {
                        decision = Some(Decision::SaveAndSwitch);
                    }
                    if ui.button("Discard and switch").clicked() {
                        decision = Some(Decision::DiscardAndSwitch);
                    }
                    if ui.button("Cancel").clicked() {
                        decision = Some(Decision::Cancel);
                    }
                });
            });
        match decision {
            Some(Decision::SaveAndSwitch) => {
                self.save_active();
                self.pending_switch = None;
                self.active = target;
            }
            Some(Decision::DiscardAndSwitch) => {
                self.pending_switch = None;
                // Reload the editor from disk to drop in-memory edits.
                self.editors.drop_editor(self.active);
                self.active = target;
            }
            Some(Decision::Cancel) => self.pending_switch = None,
            None => {}
        }
    }
}

enum Decision {
    SaveAndSwitch,
    DiscardAndSwitch,
    Cancel,
}

// ── Entry point helpers ─────────────────────────────────────────────────

/// Resolve the `--file/-f <name>` argument to a table kind.
pub fn kind_from_arg(arg: &str) -> Option<TableKind> {
    let name = std::path::Path::new(arg)
        .file_name()?
        .to_string_lossy()
        .into_owned();
    TableKind::ALL.into_iter().find(|k| k.file_name() == name)
}

//! The six `tables/*.ron` editors.

pub mod appearance;
pub mod crt;
pub mod los;
pub mod range_effects;
pub mod scatter;
pub mod units;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::common::{TableEditor, TableKind};

/// Lazily-opened editors, one per table.
pub struct Editors {
    tables_dir: PathBuf,
    sprites_dir: PathBuf,
    open: HashMap<TableKind, Box<dyn TableEditor>>,
}

impl Editors {
    pub fn new(tables_dir: PathBuf, sprites_dir: PathBuf) -> Self {
        Editors {
            tables_dir,
            sprites_dir,
            open: HashMap::new(),
        }
    }

    pub fn get(&mut self, kind: TableKind) -> &mut Box<dyn TableEditor> {
        self.open.entry(kind).or_insert_with(|| {
            log::debug!("opening editor for {}", kind.file_name());
            match kind {
                TableKind::Units => Box::new(units::UnitsEditor::open(
                    kind.path(&self.tables_dir),
                    self.sprites_dir.clone(),
                )),
                TableKind::Crt => Box::new(crt::CrtEditor::open(kind.path(&self.tables_dir))),
                TableKind::Scatter => {
                    Box::new(scatter::ScatterEditor::open(kind.path(&self.tables_dir)))
                }
                TableKind::Los => Box::new(los::LosEditor::open(kind.path(&self.tables_dir))),
                TableKind::Range => {
                    Box::new(range_effects::RangeEditor::open(kind.path(&self.tables_dir)))
                }
                TableKind::Appearance => {
                    Box::new(appearance::AppearEditor::open(kind.path(&self.tables_dir)))
                }
            }
        })
    }

    /// Drop a cached editor (used to discard in-memory edits).
    pub fn drop_editor(&mut self, kind: TableKind) {
        log::debug!("dropping editor for {} (in-memory edits discarded)", kind.file_name());
        self.open.remove(&kind);
    }
}

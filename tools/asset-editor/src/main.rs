//! Standalone egui editor for the RON tables in
//! `Boardgame - Remember_Gordon/tables/`.
//!
//! Logging goes through the `log` facade; `env_logger` reads `RUST_LOG`
//! (default `info`, e.g. `RUST_LOG=debug cargo run -p asset-editor`).

mod common;
mod shell;
mod tables;

use std::path::PathBuf;

use common::TableKind;

const WINDOW_SIZE: [f32; 2] = [1500.0, 950.0];

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let initial = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .find_map(|a| shell::kind_from_arg(&a))
        .unwrap_or(TableKind::Units);

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tables_dir = base.join("Boardgame - Remember_Gordon/tables");
    let sprites_dir = base.join("omdurman-app/assets/sprites");

    log::info!(
        "asset-editor starting: initial table {}, tables dir {tables_dir:?}, sprites dir {sprites_dir:?}",
        initial.file_name()
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(WINDOW_SIZE),
        ..Default::default()
    };
    eframe::run_native(
        "tables/*.ron editor",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(shell::Shell::new(
                tables_dir,
                sprites_dir,
                initial,
            )))
        }),
    )
}

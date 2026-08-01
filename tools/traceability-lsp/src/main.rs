//! `traceability-lsp` — an LSP server mapping rulebook requirements to
//! implementation sites.
//!
//! Attach it to `.rs`, `docs/traceability.toml`, and the manual markdown for:
//!   * hover / go-to-definition / references / implementation between `§N`
//!     citations, the traceability matrix, the manual, and Rust symbols
//!   * code lens ("§6.53 implemented — 2 tests", "covers §10.11")
//!   * live diagnostics, sharing the same checks that `cargo test -p
//!     omdurman-rules --test traceability` runs
//!
//! Run standalone: `cargo run -p traceability-lsp`

mod diagnostics;
mod lsp_util;
mod navigation;
mod server;

fn main() {
    match server::run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("traceability-lsp: {e}");
            std::process::exit(1);
        }
    }
}

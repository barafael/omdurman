//! Shared traceability index + checks for the `traceability-lsp` server.
//!
//! This crate is the single source of truth for the rulebook <-> code
//! traceability graph: `omdurman-rules/tests/traceability.rs` runs the same
//! `checks` here that the LSP surfaces as live diagnostics, so the test and the
//! editor cannot disagree.
//!
//! The graph connects four artifacts (see `docs/traceability.toml`):
//!   * `docs/traceability.toml` — the bijective `[[mapping]]` matrix
//!   * the OCR manual markdown — requirement *definition* sites
//!   * `.rs` source files — `§N` citations and `[[mapping.impl]]` symbols
//!   * `#[rulebook]` / `// §` test annotations — behavior proofs

pub mod checks;
pub mod index;
pub mod manual;
pub mod resolve;
pub mod scan;
pub mod schema;
pub mod tests;

pub use index::{Requirement, TraceIndex};
pub use schema::{ImplSite, Mapping, PSEUDO_SECTIONS, Traceability};

use std::path::PathBuf;

/// Locate the workspace root (the directory whose `Cargo.toml` declares a
/// `[workspace]` table) by walking up from this crate's manifest dir.
pub fn workspace_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.exists()
            && let Ok(text) = std::fs::read_to_string(&manifest)
            && text.contains("[workspace]")
        {
            return dir;
        }
        if !dir.pop() {
            panic!(
                "could not find a workspace root above {}",
                env!("CARGO_MANIFEST_DIR")
            );
        }
    }
}

/// Canonical location of the traceability matrix.
pub fn traceability_path() -> PathBuf {
    workspace_root().join("docs/traceability.toml")
}

/// Canonical location of the OCR rulebook markdown — the same file the app
/// embeds (`omdurman-app/src/rulebook.rs` `include_str!`s it), so editor and
/// game reference the identical transcription.
pub fn manual_path() -> PathBuf {
    workspace_root().join("Boardgame - Remember_Gordon/Manual/RememberGordonManual.md")
}

//! Strategy doctrine loader: reads the checked-in corpus under
//! `docs/strategy/*.md` and assembles the per-side LLM brief.
//!
//! The corpus is a set of standalone markdown files of tactical advice, each
//! item citing the rulebook section it relies on (validated against
//! `docs/traceability.toml` by `tests/strategy_corpus.rs`). A side's brief is
//! the common doctrine plus its faction file (plus the Fall-of-Khartoum file
//! for that scenario). The brief is prepended to the advisor system prompt via
//! [`AgentStrategy::LlmAdvised`].

use std::fs;
use std::path::{Path, PathBuf};

use omdurman_types::{Player, Scenario};

/// Where the doctrine corpus lives, relative to the workspace root.
fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("strategy")
}

fn read(name: &str) -> Option<String> {
    fs::read_to_string(corpus_dir().join(name)).ok()
}

/// Assemble the strategic brief for `player` in `scenario` from the corpus.
/// Missing files degrade gracefully (empty sections are skipped), so the bot
/// still works on a partial checkout.
pub fn doctrine_brief(player: Player, scenario: Scenario) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(common) = read("common_doctrine.md") {
        parts.push(common);
    }

    let faction = match player {
        Player::AngloEgyptian => "anglo_egyptian_doctrine.md",
        Player::Dervish => "dervish_doctrine.md",
    };
    if let Some(text) = read(faction) {
        parts.push(text);
    }

    if matches!(scenario, Scenario::FallOfKhartoum)
        && let Some(fok) = read("fall_of_khartoum_doctrine.md")
    {
        parts.push(fok);
    }

    parts.join("\n\n---\n\n")
}

/// The list of corpus files that should exist, in load order.
pub fn corpus_files() -> &'static [&'static str] {
    &[
        "common_doctrine.md",
        "anglo_egyptian_doctrine.md",
        "dervish_doctrine.md",
        "fall_of_khartoum_doctrine.md",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_corpus_files_exist_and_are_nonempty() {
        for name in corpus_files() {
            let text = read(name).unwrap_or_else(|| panic!("missing corpus file {name}"));
            assert!(!text.trim().is_empty(), "corpus file {name} is empty");
        }
    }

    #[test]
    fn briefs_are_faction_specific() {
        let ae = doctrine_brief(Player::AngloEgyptian, Scenario::Campaign);
        let derv = doctrine_brief(Player::Dervish, Scenario::Campaign);
        assert!(ae.contains("Anglo-Egyptian"));
        assert!(derv.contains("Dervish"));
        assert_ne!(ae, derv);
    }

    #[test]
    fn fok_brief_includes_the_scenario_delta() {
        let fok = doctrine_brief(Player::Dervish, Scenario::FallOfKhartoum);
        assert!(fok.contains("GORDON"));
    }
}

//! Architecture guard: inside the engine's `effects/` module, the mutable
//! world is read through the `GameState` lens — `hexside_effective`,
//! `chain_covers`, `is_zariba_entrenched` — never straight off the static
//! `BoardInfo`.
//!
//! Terrain, landmark and Nile reads are exempt: that data is immutable
//! scenario input. Hexsides are not: wall breaches (§6.53/§6.63) and
//! constructed zariba (§5.3/§9.231) exist only as game state, so a direct
//! `board.hexsides` read silently misses them — the exact bug class this
//! split exists to prevent.

use std::path::PathBuf;

/// `file name -> compact patterns this file may use`. `state.rs` hosts the
/// lens itself (`hexside_effective`, `breach_wall` and
/// `is_printed_zariba_side` read `board.hexside_between` by design);
/// `tests.rs` asserts board *staticness* and builds authored fixtures.
const EXEMPT: &[(&str, &[&str])] = &[
    ("state.rs", &[".board.hexside_between"]),
    ("tests.rs", &[".board.hexsides", ".board.hexside_between"]),
];

/// Compact (whitespace-stripped) spellings of game-time map reads that must
/// go through the lens.
const FORBIDDEN: &[&str] = &[
    ".board.hexsides",
    ".board.hexside_between",
    ".board.is_zariba_entrenched",
    ".board.has_zariba_thorn_hedge",
];

/// Every `.rs` file under `dir`, recursively (the validators live in
/// `effects/state/`).
fn sources(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("effects/ module directory") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn effects_read_the_world_through_the_lens() {
    let effects_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/effects");
    let mut files = Vec::new();
    sources(&effects_dir, &mut files);
    let mut violations = Vec::new();
    for path in files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let content = std::fs::read_to_string(&path).expect("read effects source");
        let exempted: &[&str] = EXEMPT
            .iter()
            .find(|(file, _)| *file == name)
            .map(|(_, patterns)| *patterns)
            .unwrap_or(&[]);
        // The whole file, whitespace stripped, with each character's line:
        // a method chain split over lines still matches.
        let (compact, lines): (String, Vec<usize>) = content
            .lines()
            .enumerate()
            .flat_map(|(n, line)| line.chars().map(move |c| (c, n + 1)))
            .filter(|(c, _)| !c.is_whitespace())
            .unzip();
        for pattern in FORBIDDEN.iter().filter(|p| !exempted.contains(p)) {
            let compact_pattern: String = pattern.chars().filter(|c| !c.is_whitespace()).collect();
            for (offset, _) in compact.match_indices(&compact_pattern) {
                let line = lines[compact[..offset].chars().count()];
                violations.push(format!(
                    "{}:{line}: direct board read — use the GameState lens (`hexside_effective` et al.)",
                    path.strip_prefix(&effects_dir).unwrap_or(&path).display()
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "game-time board reads bypassing the lens:\n{}",
        violations.join("\n")
    );
}

//! Architecture guard: inside the engine's `effects/` module, the mutable
//! world is read through the `GameState` lens — `hexside_effective`,
//! `mine_at`, `chain_covers`, `is_zariba_entrenched`,
//! `has_zariba_thorn_hedge` — never straight off the static `BoardInfo`.
//!
//! Terrain, landmark and Nile reads are exempt: that data is immutable
//! scenario input. Hexsides are not: wall breaches (§6.53/§6.63) and
//! constructed zariba (§5.3/§9.231) exist only as game state, so a direct
//! `board.hexsides` read silently misses them — the exact bug class this
//! split exists to prevent.

use std::path::PathBuf;

/// `file name -> compact patterns this file may use`. `state.rs` hosts the
/// lens itself (`hexside_effective`, `breach_wall` and
/// `zariba_entry_surcharge` read `board.hexside_between` by design);
/// `tests.rs` asserts board *staticness* and builds authored fixtures.
const EXEMPT: &[(&str, &[&str])] = &[
    ("state.rs", &[".board.hexside_between"]),
    (
        "tests.rs",
        &[
            ".board.hexsides",
            ".board.hexside_between",
            ".board.hexside_is",
        ],
    ),
];

/// Compact (whitespace-stripped) spellings of game-time map reads that must
/// go through the lens.
const FORBIDDEN: &[&str] = &[
    ".board.hexsides",
    ".board.hexside_between",
    ".board.hexside_is",
    ".board.is_zariba_entrenched",
    ".board.has_zariba_thorn_hedge",
    ".board.zariba_entry_surcharge",
];

#[test]
fn effects_read_the_world_through_the_lens() {
    let effects_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/effects");
    let mut violations = Vec::new();
    for entry in std::fs::read_dir(&effects_dir).expect("effects/ module directory") {
        let path = entry.expect("dir entry").path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let content = std::fs::read_to_string(&path).expect("read effects source");
        let exempted: &[&str] = EXEMPT
            .iter()
            .find(|(file, _)| *file == name)
            .map(|(_, patterns)| *patterns)
            .unwrap_or(&[]);
        for pattern in FORBIDDEN {
            if exempted.contains(pattern) {
                continue;
            }
            let compact: String = pattern.chars().filter(|c| !c.is_whitespace()).collect();
            for (lineno, line) in content.lines().enumerate() {
                let stripped: String = line.chars().filter(|c| !c.is_whitespace()).collect();
                if stripped.contains(&compact) {
                    violations.push(format!(
                        "{}:{}: direct board read — use the GameState lens (`hexside_effective` et al.)",
                        name,
                        lineno + 1
                    ));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "game-time board reads bypassing the lens:\n{}",
        violations.join("\n")
    );
}

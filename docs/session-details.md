# Session details

## 2026-09-02 — Traceability semantic-status cleanup (§5 + §9 family)

- `disrupted_unit_cannot_fire` re-annotated `§5` → `§6.41` (Direct Fire Subphase); §5 is now a clean `descriptive` container with no tests.
- `out-of-scope` → `descriptive` for containers with implemented children: §9.3, §9.11, §9.21, §9.32, §9.34.
- §9.31 (Bonus game map) `out-of-scope` → `implemented`: tests `scenario_maps_to_board` + `start_game_scenario_selects_board` (app), impl `fall_of_khartoum_map_data` (`omdurman-rules/src/board_data.rs:37`, already anchored; duplicate impl symbol with §9.342 is fine).
- Container tests re-homed to leaves: `campaign_has_no_fixed_placements` → §9.111 (`omdurman-app/src/scenario_setup.rs`), `scenario_maps_to_board` + `start_game_scenario_selects_board` → §9.31 (`omdurman-app/src/tests.rs`), `remove_deployed_unit_happy_path` → §9.321 (single annotation now). §9.1/§9.2/§9.3 carry zero tests.
- §9.321 note corrected: GORDON auto-placed at Palace; North Fort is Dervish-placed per §9.344.
- `artillery_sinks_gunboat_only_on_three_plus` given `#[rulebook("§6.61")]` + registered in §6.61 tests.
- `fix_lines` resynced 2 line fields; regenerated `traceability.typ` + `tools/traceability-typst/data.json`.
- Green: traceability gate (3), traceability_paths (6), full `cargo test -p omdurman-rules` (374 lib + integration), traceability-typst (7, incl. data.json freshness). LSP diagnostics were stale/noisy throughout; cargo gate authoritative.

## 2026-09-02 — Tables → const statics + Kani proofs over them

- **Architecture (Option A)**: the four rules tables (CRT, range effects, scattergram, LOS) moved from `include_str!` + `OnceLock` + runtime `ron::from_str` (Kani-unmodelable) to `static` const arrays in `tables_data.rs`, transcribed from the RON under `Boardgame - Remember_Gordon/tables/`. RON remains the asset-editor source of truth; 4 new `#[cfg(test)]` parity tests parse the RON and fail cell-by-cell on drift.
- **Consumers**: `band_at`/`max_day_range`/`combat_results_table`/`howitzer_scatter`/`blocking_rules`/`los_level` are plain array indexing; runtime panic paths (missing CRT row / LOS cell, short scattergram) now compile-time bounds. `BlockingRule.1` is `&'static [LosCondition]`; added `WeaponClass::ALL` + `index()`; `max_day_range` rewritten as plain index loop (the `iter().enumerate().rev()` chain was a CBMC unroll bomb).
- **7 new Kani proofs, all SUCCESSFUL** (registered in `traceability.toml` proofs arrays): §6.22 ×2 (`range_effects_are_out_of_range_outside_the_printed_ten_hex_distance_window`, `max_day_range_is_the_last_in_range_hex`), §8.1 ×2 (night cap halved/≥1, night gating of day band), §CRT ×2 (Eliminate bounds, monotone in roll), §6.64 ×1 (scatter Center iff roll ≥ 7).
- **Earlier in session (pre-Option-A)**: 6 proofs over `RangeBand`/CRT-halving/scatter geometry (§6.16 ×3, §CRT, §6.64 ×2) in lib.rs/effects.rs verification modules, all green.
- **Traceability green 3/3**: new proofs registered; pre-existing §7.5 mismatch fixed by rewording an incidental doc citation in `advance_phase_is_atomic` (guard assumed, not proven); `fix_lines` + `traceability.typ`/`data.json`/PDF regenerated.
- **Verification**: `cargo test --workspace` 39/39 binaries green (377 rules lib tests); wasm32 checks pass for rules + app (CI gate intact); CLAUDE.md `tables_data` description updated.

## Session: Kani proofs green + CI workflow (2026-09-02)

**Outcome:** all 63 Kani harnesses now verify `SUCCESS` (the `apply_effect` atomicity proofs were stuck at `UNDETERMINED`); CI workflow added; docs updated. All gates green: fmt, clippy `-D warnings`, `cargo test --workspace` (39 suites), traceability, Kani, `trunk build --release` (wasm). Nothing committed.

- **Root cause of UNDETERMINED**: `BoardInfo`'s six `IndexMap`/`IndexSet` fields defaulted to `RandomState` → `getrandom` `syscall` on the live `BoardInfo::default()` path in every `GameState::new` (not "dead ron/panic paths" as previously documented). Fixed with deterministic `BuildHasherDefault<DefaultHasher>` (`omdurman-rules/src/board.rs`); `rand`'s `getrandom` features disabled in rules (dice are hand-seeded).
- **Harness tractability** (measured): concrete unit counts (symbolic `take(n)` unrolls forever), phase pinning where property lives on one arm (symbolic 7-arm match ≈ 10× steps), stubs `-Z stubbing` (`end_player_turn`, `advance_phase`, `apply_melee_combat`), `--features kani` gates out 4 `debug!` sites. Worst formula 2.8M → <450k SSA steps. Empty `kani = []` feature added to types crate so combined runs pass one flag.
- **False alarm caught**: unwind-bound truncation produced a counterexample concrete playback couldn't reproduce; harness restructured, warning documented (bounds can fail *un*loudly — prefer concrete trip counts).
- **CI**: new `.github/workflows/ci.yml` — fmt/clippy/test (3-OS matrix), Kani (ubuntu, `model-checking/kani-github-action`), traceability gates. ~20 pre-existing clippy warnings fixed to make `-D warnings` real.
- **Docs**: architecture.md §9 + "CI runs no tests" refreshed; CLAUDE.md documents CI + script defaults; traceability re-synced (`fix_lines` ×5 lines, §7.5 `proofs` array gained `advance_phase_is_atomic`, regenerated typ/data.json/PDF).
- **Files touched this session**: `omdurman-rules/src/board.rs`, `omdurman-rules/Cargo.toml` (rand features + `kani` feature), `omdurman-types/Cargo.toml`, `omdurman-rules/src/effects.rs` (harnesses + module docs), `effects/dispatch.rs` (cfg-gated debug!), `scripts/kani.sh`, `.github/workflows/ci.yml`, `docs/architecture.md`, `CLAUDE.md`, `docs/traceability.toml`, `traceability.typ`, `tools/traceability-typst/data.json`, `traceability.pdf`, clippy fixes across app/bot/tools/tests. Note: working tree also carries the other session's uncommitted edits; `cargo fmt` normalized formatting in them too.


## 2026-09-02 — FoK 2-player playability audit

**Outcome:** full Fall-of-Khartoum flow traced event-by-event through the engine, verified by 4 instrumented bot playthroughs (864–1190 effects each), and one blocking UI gap found + fixed. All gates green (fmt, clippy `-D warnings`, 40 test suites incl. the new `omdurman-bot/tests/playability.rs`, wasm build).

- **Flow**: Setup (2 fixed placements auto-emitted; deploy 17 AE + 49 Dervish under OOB caps; exit via both `ConfirmSetupReady`s or `AdvancePhase` once `setup_complete`) → per player-turn Movement → DefFire(Direct) → [Dervish turn: DefFire(Maxim) AE second fire → OffFire(Direct)] / [AE turn: OffFire(Direct) → OffFire(Maxim/Howitzer)] → Melee → `end_player_turn` (demolitions resolve, disruption recovery, trackers cleared, turn snapshot) → 8 turns (T1–T2 night 2am/4am, T3–T8 day) → `finish_game` → `FoKVictoryLevel` ladder (Gordon turn ≤4/5/6+ vs survival + Dervish-loss penalty). Gordon dies on Palace occupation or leader overrun → instant game end. FoK has no reinforcements/desertion by design.
- **Instrumented runs**: `cargo run -p omdurman-bot -- play fok <seed> <random|aggressive|dervish-agg|ae-agg> 30 <log>` — all games terminate with proper `FoK(...)` results (DervishDecisive when Gordon falls T2–3; BritishDecisive on survival). Effect totals: MoveUnit 2319, DeployUnit 264, **ArtilleryBreachWall 211**, AdvancePhase 193, FireCombat 49, DeclareMelee/ResolveMelee 15/15, AdvanceAfterCombat 7, ConfirmSetupReady 5.
- **Blocking gap fixed**: `ArtilleryBreachWall` had **no UI caller** (bot-only) — the attacker's standard §6.63 way into the walled city. Added `artillery_breach_ui` card (`omdurman-app/src/ui_plugin.rs`): select artillery/howitzer in a fire phase → wall-hexside buttons filtered by `can_fire_at_wall` (in-range first, night halving honored) → d10 pre-rolled → `GameEffect::ArtilleryBreachWall`. Registered under the fire-phase run conditions.
- **UI plausibility verdict**: every effect emitted in real FoK play now has an active path (deploy/re-pickup via picker + `PlaceUnit`/`RemoveUnit` events, Ready/Begin-battle, End Phase per faction, fire allocation + Execute All with per-subphase reset, melee declare/resolve/retreat/advance-after-combat, FoK victory panel). Effects deliberately UI-less are all out of FoK scope (Campaign optional rules §10, PlaceReinforcements, MeleeCombat legacy, DriftGunboat known gap).
- **Regression lock**: `omdurman-bot/tests/playability.rs` — plays FoK seeds 7/42/2026, asserts every emitted variant is in a documented `UI_SUPPORTED`/`UI_EXCLUDED` allowlist (unknown variants fail until wired), setup handoff completes, core action set exercised.

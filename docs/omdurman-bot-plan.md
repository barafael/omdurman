# `omdurman-bot` — Headless AI Playthrough Crate

## Goal

A simple AI that plays full games of *Remember Gordon!* headlessly (no Bevy render
loop), logging every move as a replayable `GameRecord` trace so a downstream agent
can verify rule correctness. Two play strategies: **uniform-random** (fast, broad
coverage) and **LLM-advised** (per-turn, narrated with rulebook citations).
Verification via **proptest** invariants now; **LLM trace review** later.

## Game-AI crate survey (conclusion: depend on none)

| Crate | Verdict |
|---|---|
| `mcts` (zxqfl, 2019) | Reject — abandonware, `rand 0.4`, Rust 2015 |
| `arboriter-mcts` (0.3.0, 2025) | Marginal — modern MCTS, but `rand 0.8` mismatch + branching factor too high for full-game search. *Future option for constrained sub-problems only.* |
| `minimax`, `rival`, `turk`, `game-solver`, `mtdf` | Reject — minimax/negamax; branching factor infeasible |
| `board-game-traits` | Marginal — trait inspiration only; engine already has cleaner effect architecture |
| `big-brain`, `emergent`, `telic`, `oxyde`, `bevy-yoetz` | Reject — real-time NPC utility-AI, wrong paradigm |

**No external game-AI crate earns a dependency.** Reasons: (1) branching factor is
too high for exhaustive search; (2) goal is *exploration* not *winning* — MCTS
narrows, we want to broaden; (3) engine already has a clean deterministic
effect-based forward model.

The right tools: the existing `rand 0.10` (uniform play) + `proptest 1` (invariant
verification) + the engine's own `can_*` predicates (move generation).

## Decisions locked

- **Depend on `omdurman-net`** — reuse `GameEvent` / `GameRecord` natively (traces
  are byte-compatible with the app's `SpectatorTimeline`).
- **LLM transport → into `omdurman-net`** — shared by app + bot.
- **LLM granularity → per-turn** — one prompt per player-turn; LLM returns a plan
  + reasoning + updated cache.
- **Play strategy → uniform-random** core; `LlmAdvised` layered on top.
- **LLM cache → 500 KB persistent scratchpad** threaded turn-to-turn.
- **Verification → proptest now + LLM review later.**

## Architectural caveats

**Bevy transitive dep (pragmatic for now):** `omdurman-net` mixes pure event types
(`GameEvent`, `GameRecord`) with Bevy-coupled socket glue (`MatchboxSocket`).
Depending on it pulls Bevy into the bot's compile graph, but the bot never spawns
an `App` — it's headless at runtime. *Deferred follow-up:* split into
`omdurman-net-core` (pure types + LLM transport, no Bevy) + `omdurman-net`
(matchbox), if compile time bites.

**reqwest on WASM:** `reqwest 0.13` + `rustls` does not build for `wasm32`. The LLM
module in `omdurman-net` is `#[cfg(not(target_arch = "wasm32"))]`-gated so CI's
`trunk build --release` stays green. The bot is native-only.

---

## Phase 1 — Refactor: shared LLM transport into `omdurman-net`

**New: `omdurman-net/src/llm.rs`**
- Move from `omdurman-app/src/llm.rs`: `LlmConfig`, `LlmError`,
  `request_completion`, `ChatRequest`, `ChatMessage`, `ChatResponse`.
- Whole module `#[cfg(not(target_arch = "wasm32"))]`.
- `omdurman-net/Cargo.toml` gains under
  `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`:
  `reqwest = { version = "0.13", default-features = false, features = ["json", "rustls"] }`

**Slimmed: `omdurman-app/src/llm.rs`**
- Keeps `PendingCompletions`, `CompletionTag`, `spawn_completion` (Bevy-specific).
- `pub use omdurman_net::llm::{LlmConfig, LlmError, request_completion};` so
  `telegram.rs` / `newspaper.rs` call sites are unchanged.

---

## Phase 2 — New crate `omdurman-bot`

**`omdurman-bot/Cargo.toml`**
```toml
[dependencies]
omdurman-rules  = { path = "../omdurman-rules" }
omdurman-types  = { path = "../omdurman-types" }
omdurman-net    = { path = "../omdurman-net" }
rand            = { workspace = true }
rand_chacha     = { workspace = true }
serde           = { workspace = true }
serde_json      = "1"
[dev-dependencies]
proptest = "1"
```
Add `"omdurman-bot"` to workspace `members`.

### `src/rng.rs` (~30 lines)
Mirror of `GameRng` so the same seed produces the same dice sequence:
```rust
pub struct BotRng(ChaCha8Rng);
impl BotRng {
    pub fn from_seed(u64) -> Self;
    pub fn roll_d10(&mut self) -> DieRoll;
    pub fn roll_d6(&mut self) -> u8;
    pub fn choose<'a, T>(&mut self, s: &'a [T]) -> Option<&'a T>;
}
```

### `src/oob.rs` (~150 lines) — Setup OOB reconstructor
- `pub fn scenario_oob(scenario) -> Vec<UnitPlacement>` — builds the full
  deployable force list from `Scenario::sections_for_picker()` ×
  `unit_id_for_section_pos(col, row)`, ported headlessly from the logic currently
  tangled in `picker.rs`.
- Filters out fixed auto-placements (GORDON, North Fort) emitted separately by
  `scenario_setup::build_setup_plan`.
- **Test:** count matches `GameState::setup_target(player)` for each scenario.

### `src/actions.rs` (~450 lines) — the core move generator
```rust
pub fn legal_actions(state: &GameState, rng: &mut BotRng) -> Vec<GameEffect>;
```
Dispatches on `state.phase`:

| Phase | Enumerates via |
|---|---|
| **Setup** | Each unplaced OOB unit → probe in-zone hexes (`in_deployment_zone` + `check_stacking`) → `DeployUnit`. Append `ConfirmSetupReady` when `setup_target_met`. |
| **Movement** | Per owned unit: BFS reachable hexes ≤ MP allowance (`can_move_unit_to`); gunboat Nile-steps (`can_move_gunboat`) incl. Nile-mouth. `ConstructZariba` / `Demolition` / `RecoverUnit` / `FriendliesTransport` where `can_*` allow. |
| **Fire** | Per unfired friendly unit: `can_fire_at` over enemy hexes → group co-stacked firers (port `build_fire_attack`) → `FireCombat` / `HowitzerFire` / `ArtilleryBreachWall` with pre-rolled dice. `AdvanceAfterCombat` for eligible units. |
| **Melee** | `can_melee` over neighbours → `DeclareMelee`; if pending → `ResolveMelee` (clone-and-try); `RetreatBeforeMelee`; `AdvanceAfterCombat`. |
| **Any** | Always append `AdvancePhase`. `DervishDesertion` (clone-and-try when due). |

For the **8 predicate-less variants** (`ResolveMelee`, `FriendliesTransport`,
`RiverMine`, `SinkChain`, etc.): clone-and-try —
`let mut s = state.clone(); if apply_effect(&mut s, &e).is_ok()`. Sound; cheap
enough.

### `src/playthrough.rs` (~200 lines)
```rust
pub struct PlayConfig {
    pub max_actions_per_phase: usize,   // anti-stall, default 200
    pub max_turns: u8,                  // hard ceiling, default scenario_len + 4
}
pub enum PlayStrategy {
    Random,
    LlmAdvised { config: omdurman_net::llm::LlmConfig },
}
pub struct PlayResult {
    pub events: Vec<omdurman_net::GameEvent>,
    pub llm_annotations: Vec<LlmAnnotation>,
    pub final_cache: Option<String>,        // Some only in LlmAdvised mode
    pub seed: u64,
    pub final_state: GameState,
    pub variant_coverage: Vec<&'static str>,
}
pub fn playthrough(scenario, seed, cfg, strategy) -> PlayResult;
```
Loop: build state with compiled board (`fall_of_khartoum_map_data()` /
`campaign_map_data()` → `BoardInfo::from_map_data`), enumerate → pick → apply →
push `GameEvent::Effect` → drain observations. Anti-stall: force `AdvancePhase`
after `max_actions_per_phase`; hard ceiling at `max_turns`.

### `src/llm.rs` (~250 lines) — per-turn advisor + persistent cache

**The cache** — a 500 KB persistent scratchpad threaded turn-to-turn:
```rust
const MAX_CACHE_BYTES: usize = 512_000;

#[derive(Default, Clone)]
pub struct LlmCache(pub String);

impl LlmCache {
    pub fn truncate_to_cap(&mut self) {
        if self.0.len() > MAX_CACHE_BYTES {
            let cut = self.0.char_indices()
                .take_while(|(i, _)| *i <= MAX_CACHE_BYTES)
                .last().map(|(i, _)| i).unwrap_or(MAX_CACHE_BYTES);
            self.0.truncate(cut);
            self.0.push_str("\n…[cache truncated at 500 KB]");
        }
    }
}
```

**Response protocol (tagged format — robust for large free-form text):**
```
CACHE:
<updated notes for next turn — what the model wants to remember>

PLAN:
[3, 7, 12]      // indices into the legal_actions list

REASONING:
- 3: fire at (q,r) — §6.24 A-E direct bonus applies
- 7: move Mulazmin toward Palace — §9.322 entry edge
```
The bot parses the three tagged sections: `CACHE` → stored (then
`.truncate_to_cap()`), `PLAN` → applied in order, `REASONING` → logged as
`LlmAnnotation`s. Missing/malformed section → fall back to random for that turn,
keep cache as-is.

**Prompt inclusion (next turn):**
```
=== NOTES FROM PREVIOUS TURNS (≤500 KB) ===
{cache}
=== END NOTES ===
```
with a system-prompt instruction: "Update your notes each turn. Use them to track
the game's evolution. Cite rulebook sections. The notes are your only memory
between turns."

`playthrough.rs` holds a single `Option<LlmCache>` and threads it into the
per-turn call. `final_cache` ships with `PlayResult` so a downstream review agent
sees what the LLM "remembered" by game end.

### `src/invariants.rs` (~200 lines) — predicates asserted after every `apply_effect`
1. Every unit position is a valid board hex.
2. `check_stacking` passes for every occupied hex (§5.51).
3. No land unit on Nile / no boat on land (§5.22).
4. `units_fired_this_phase` references only extant units.
5. `mp_spent_this_turn` ≤ allowance for every unit.
6. `game_over` is monotonic.
7. Dervish-tribal stacking rule (§5.52): no hex mixes two tribes.

---

## Phase 3 — Tests

| File | What it proves |
|---|---|
| `tests/determinism.rs` | Same seed → byte-identical `events` across two runs. |
| `tests/coverage.rs` | Over N=200 games per scenario, every `GameEffect` variant that *can* appear *does* appear. |
| `tests/termination.rs` | Every playthrough reaches `game_over` within `max_turns × 8 × max_actions_per_phase` actions. |
| `tests/invariants.rs` | `proptest!` over random seeds: all 7 invariants hold after every effect. Automatic shrinking → minimal violating traces, persisted to `proptest-regressions/`. |

---

## Phase 4 — Follow-ups (not in this cut)

- **In-game bot:** Bevy system in `omdurman-app` that calls `legal_actions` and
  pushes picks through `PendingEdits::outgoing_broadcast` when a seat is
  bot-flagged. Requires a "bot" lobby binding.
- **LLM trace review:** `--review <jsonl>` subcommand feeding each turn's events +
  observations to the LLM, asking it to flag rulebook violations (mirrors the
  existing telegram/newspaper flow).
- **Net-crate split** if bot compile time is painful.
- **`arboriter-mcts`** for a constrained sub-problem (e.g. optimal
  fire-allocation) if a "playing well" bot is ever wanted.

---

## Effort

| Piece | Lines | Risk |
|---|---|---|
| LLM refactor into omdurman-net | ~80 moved + cfg gates | Low — mechanical |
| `rng.rs` | ~30 | None |
| `oob.rs` | ~150 | **Medium** — porting tangled picker logic; test against `setup_target` |
| `actions.rs` | ~450 | **Highest** — breadth; 8 clone-and-try effects; movement BFS |
| `playthrough.rs` | ~200 | Low |
| `llm.rs` (per-turn advisor + cache) | ~250 | Medium — prompt engineering + tagged parser + truncation |
| `invariants.rs` | ~200 | Low |
| Tests | ~300 | Low |
| **Total** | **~1660** | |

The move generator (`actions.rs` + `oob.rs`) is ~60% of the effort and the only
real risk area.

# `omdurman-bot` + Tactics Suite — Manual

User manual for the headless rule-verification stack added on top of
`c576dcf` ("Add FoK victory-progress panel…"). Everything described here is
**uncommitted working-tree work**: the per-side "three-agent" architecture for
`omdurman-bot` (two players + an offline observer), the strategy doctrine
corpus, the `omdurman-bot-cli` binary, and the deterministic tactics vignette
suite in `omdurman-rules`.

The bot itself was first committed in `d72f639` ("Add omdurman-bot headless
playthrough crate…", 2026-08-06) with a *single* `PlayStrategy` and no game
log; this document covers everything layered on since. §-references are to the
Phoenix Enterprises (1982) manual.

---

## 1. What is new, and where it came from

| Since | Commit | Contents |
|---|---|---|
| `d72f639` (2026-08-06) | bot baseline | `actions.rs`, `invariants.rs`, `llm.rs`, `oob.rs`, `playthrough.rs` (single strategy, single cache), `rng.rs`, first tests; `docs/omdurman-bot-plan.md` |
| `c576dcf` (2026-08-06) | last commit | app-side FoK victory-progress panel; **no bot changes** |
| *working tree* | **this manual** | per-side agents, game log, offline observer, doctrine corpus, CLI, tactics suite |

The design was spelled out in `docs/omdurman-bot-three-agent-design.md`
("design — not yet implemented") and `docs/omdurman-bot-plan.md` Phase 4
("LLM trace review"); this work implements it (with the deviations in §8).

The stale `stash@{0}` ("Workspace un-crates-ification") is an *older,
abandoned* crate restructure and is unrelated to this work.

### New files (untracked)

```
omdurman-bot/src/agent.rs       AgentStrategy + Agents (per-side config)
omdurman-bot/src/describe.rs    describe_effect / describe_observation / describe_turn_event
omdurman-bot/src/doctrine.rs    strategy-doctrine brief loader
omdurman-bot/src/log.rs         GameLog (the observer's ground truth)
omdurman-bot/src/main.rs        omdurman-bot-cli (play / review / run / tactics)
omdurman-bot/src/observer.rs    offline LLM rules audit (chunked review)
omdurman-bot/tests/head_to_head.rs
omdurman-bot/tests/log_format.rs
omdurman-bot/tests/observer.rs
omdurman-bot/tests/strategy_corpus.rs
omdurman-rules/src/tactics.rs   tactics DSL + 22 vignettes
omdurman-rules/tests/tactics.rs vignette runner
docs/omdurman-bot-three-agent-design.md
docs/rules_crib_sheet.md
docs/strategy/                  doctrine corpus (4 files + README)
```

### Modified files

`omdurman-bot/src/{lib.rs, llm.rs, playthrough.rs}`, `omdurman-bot/Cargo.toml`
(adds `[[bin]] omdurman-bot-cli`, `futures`), the four existing bot tests
(`coverage`, `determinism`, `invariants`, `termination` — all switched from
`PlayStrategy` to `Agents`), and `omdurman-rules/src/lib.rs` (adds
`pub mod tactics;`).

---

## 2. Quick start

```shell
# Tactics vignette suite (rules crate) — 22 scripts, deterministic
cargo test -p omdurman-rules --test tactics

# Whole bot test suite (head-to-head, log format, observer, corpus, invariants…)
cargo test -p omdurman-bot

# CLI: run everything offline
cargo run -p omdurman-bot --bin omdurman-bot-cli -- tactics
cargo run -p omdurman-bot --bin omdurman-bot-cli -- play Campaign 123 random 30
cargo run -p omdurman-bot --bin omdurman-bot-cli -- review game.log findings
cargo run -p omdurman-bot --bin omdurman-bot-cli -- run run.json
```

`play` writes `game.log` (the human-readable log) plus prints a run summary;
`review` writes `findings.md` + `findings.json`; `tactics` prints one
`PASS/FAIL` line per vignette and exits 1 on any failure. The LLM paths need an
API key (see `omdurman-net::llm::LlmConfig`); `random` needs nothing.

---

## 3. Architecture: two players, one observer

```
  Agent AE ──┐
             ├──> rules engine (apply_effect) ──> GameLog ──> Observer ──> findings.md / .json
  Agent Dervish ─┘
   (own strategy, own cache, own brief)      (byte-stable, §-cited)   (offline LLM pass)
```

The engine remains authoritative: the driver never bypasses `apply_effect`,
and the log is a *derived side-channel* (observations are drained via
`std::mem::take`), so the `GameEvent` trace stays byte-for-byte deterministic.

### 3.1 Per-side agents — `src/agent.rs`

`AgentStrategy` is either `Random` (uniform over `actions::legal_actions`) or
`LlmAdvised { config, brief }`. `Agents { ae, dervish }` holds one per faction;
`Agents::random()` is the default. Each LLM side owns its **own 500 KB
`LlmCache`** threaded turn-to-turn, and the persona brief is prepended to the
system prompt. `advise_turn` (in `src/llm.rs`) gained a `side: Player` and
`brief: &str` parameter; the existing prompt builder already frames
Friendly/Enemy from `active_player`, so one code path serves both sides.

The playthrough loop (`src/playthrough.rs`):
`playthrough(scenario, seed, cfg, agents) -> PlayResult` dispatches on
`state.active_player`, refreshes an LLM plan once per player-turn (per side),
and validates every planned index against the *current* candidate list before
applying. On a `Random` side the LLM is never called.

`PlayResult` now carries: `events` (replayable trace), `log` (the `GameLog`),
`llm_annotations`, `ae_final_cache` / `dervish_final_cache`, `seed`,
`final_state`, `variant_coverage`, `actions_taken`, `observations_total`.

### 3.2 Game log — `src/log.rs` + `src/describe.rs`

One plain-text file that must "give enough context" on its own (no live engine
access). Rendered by `GameLog::render()`, **byte-identical for a given seed**
(tested). Structure:

```
GAME LOG — Remember Gordon! (The Battle of Omdurman)
scenario:        campaign
seed:            0x9e3779b97f4a7c15
agents:          ae=random dervish=llm(<brief>)   # or "random"
rules_version:   Manual §1–§10

[0] T1 Setup AngloEgyptian  DeployUnit …
      → UnitEliminated: …  [event 0]             # indented, §-cited observation
[23] T2 Movement Dervish  MoveUnit Mulazmin … → …
[reasoning, Dervish T2] 3: advance toward Palace — §9.322
=== Turn 2 complete (…Day) — 5 fire, 1 melee, 2 eliminations; VP AE 4 / Dervish 0 ===
    - 1st Yorkshires moved …
=== GAME OVER ===  result: …
victory: AE 12 / Dervish 3
```

- `describe_effect(effect, state)` renders a `GameEffect` as one log line,
  naming units via `profile_for_unit(...).short_label()`, printing hexes as
  `(q,r)`, paths as ordered routes, and spelling out pre-rolled dice so an
  observer can re-derive CRT lookups and MP arithmetic.
- `describe_observation(obs)` renders engine `Observation`s; the `paragraphs`
  field is printed verbatim, because the engine is the authoritative source of
  § citations.
- `describe_turn_event(ev)` renders the structured per-turn records in the
  `=== Turn N complete ===` block, alongside the running victory ledger.

The full `GameEffect` match surface is covered (no `Debug`-fallback regression)
by `tests/head_to_head.rs::describe_effect_renders_real_trace_effects`, which
replays a real Campaign playthrough and renders every applied effect.

### 3.3 Offline observer — `src/observer.rs`

`review(log, config, completion, crib) -> ObserverReport` feeds the log to the
LLM **turn by turn**, carrying a running notes/findings cache between chunks
(the same `CACHE:`/tagged-response pattern as the players). Chunking happens at
the `=== Turn N complete ===` markers (`chunk_log`), so a game of thousands of
events fits in a prompt budget.

The model is told to respond exactly:

```
CACHE:
<working notes / open threads>
FINDINGS:
- severity|seq|§section|explanation
SUMMARY:
<closing assessment>
```

`Severity` ∈ {Critical, Error, Warning, Info}; malformed lines are dropped,
and a failed/malformed chunk keeps the previous cache and continues
(graceful degradation). `Completion` is a small trait so tests run on a canned
transport; `ReqwestCompletion` wraps `omdurman_net::llm::request_completion`.
With no API key, `review` returns an empty report and a "skipped" summary.

Findings are **advisory**. The deterministic gating stays on the three-layer
stack: (1) engine `can_*`/`apply_effect` predicates → (2) `invariants::check_all`
after every effect → (3) this observer for rule *misapplications* the
invariants can't encode (wrong CRT row, missed modifier, phase-order slip,
FoK deltas).

### 3.4 Rules context — `docs/rules_crib_sheet.md`

A curated, checked-in summary of the manual (turn/phase sequence, terrain costs,
fire/melee CRTs, stacking, ZOC, zariba, reinforcements/desertion, transport,
optional rules, VP, FoK deltas). Attached to the observer's **first chunk**
only. Because it is a tracked file, its drift from the manual is itself
auditable.

---

## 4. Doctrine corpus — `src/doctrine.rs` + `docs/strategy/`

`docs/strategy/` holds standalone tactical advice for the LLM agents:

| File | Scope |
|---|---|
| `common_doctrine.md` | system-wide doctrine (both sides) |
| `anglo_egyptian_doctrine.md` | Kitchener playbook |
| `dervish_doctrine.md` | Khalifa playbook |
| `fall_of_khartoum_doctrine.md` | FoK scenario deltas |

`doctrine_brief(player, scenario)` concatenates common + faction (+ FoK file in
the Fall-of-Khartoum scenario) as the side's `brief`, which
`AgentStrategy::LlmAdvised` prepends to the system prompt. Every item cites the
rulebook sections it rests on; `tests/strategy_corpus.rs` verifies that every
`§N.NN` citation resolves to a `[[mapping]]` in `docs/traceability.toml`, that
every file loads non-empty through the same loader the agents use, and that the
corpus is substantial (>10k chars). Missing files degrade gracefully, so the
bot still works on a partial checkout.

---

## 5. CLI — `omdurman-bot-cli`

Binary `omdurman-bot-cli` (declared via `[[bin]]` in `omdurman-bot/Cargo.toml`);
the lib holds all logic so tests exercise it without spawning a process.

```
omdurman-bot-cli play   [scenario] [seed] [strategy] [max_turns] [log_file]
omdurman-bot-cli review [log_file] [findings_prefix]
omdurman-bot-cli run    [run.json]
omdurman-bot-cli tactics
```

- **play** — scenario ∈ `Campaign` (default), `FallOfKhartoum`/`fok`,
  `Historical`. `seed` defaults to system RNG. `strategy`: `random` (both
  sides), `llm` (both sides LLM-advised), `ae` (AE LLM / Dervish random),
  `dervish` (Dervish LLM / AE random). Writes `game.log` and prints a summary.
- **review** — reads a log, runs the observer against
  `docs/rules_crib_sheet.md`, writes `{prefix}.md` + `{prefix}.json`.
- **run** — one JSON spec: `{scenario, seed?, ae_strategy, dervish_strategy,
  max_turns?, output_log?, output_findings?, review?}`; play, optionally
  review, in one invocation.
- **tactics** — see §6.

The exit code is 0 on success and 1 if any tactics script fails (2 for unknown
subcommands), so the CLI is CI-usable.

---

## 6. Tactics suite — `omdurman-rules/src/tactics.rs`

A living, human-readable regression suite for the rules engine. A **tactics
script** is a hand-built `GameState` plus an ordered list of steps:

- `ScriptStep::Legal { note, effect }` — `apply_effect` must return `Ok`.
- `ScriptStep::Illegal { note, probe, effect }` — must return `Err` matching
  the `Probe` (`Any` or a `matched(label, closure)` predicate).
- `ScriptStep::Assert { note, predicate }` — a closure over the state.

`run_step(&mut GameState, &ScriptStep) -> Option<String>` returns the first
failure message. `all_scripts()` returns 22 vignettes in rulebook order. Both
the unit runner (`omdurman-rules/tests/tactics.rs`) and the CLI `tactics`
subcommand replay every script from a fresh clone of its initial state and
report the first misbehaving step. Pre-rolled dice ride inside the effects, so
every script is fully deterministic.

| Script | Citation | Exercises |
|---|---|---|
| `movement_allowance` | §4, §5.11, §5.12 | allowance cap + no MP carryover |
| `walled_city_entry_artillery` | §5.23 | Dervish artillery may enter the walled city |
| `walled_city_entry_denied` | §5.23 | Baggara denied; wall-hexside blocking |
| `gunboat_river_move` | §5.22, §5.24 | Nile movement, upstream/downstream caps |
| `artillery_sinks_gunboat` | §6.22, §6.61 | only artillery may sink gunboats |
| `artillery_destroys_fort` | §6.22, §6.62 | only artillery may destroy forts |
| `maxim_second_fire` | §6.14, §6.42 | Maxim fires in Direct + Second-fire subphase |
| `howitzer_on_target` | §6.22, §6.64 | range-4 hit, impact roll 10 |
| `howitzer_scatter_miss` | §6.64 | impact roll 2 scatters off-target |
| `no_howitzer_at_night` | §6.64, §8.1 | howitzer fire forbidden after dark |
| `retreat_before_melee` | §7.5, §7.7 | camel retreats 2 hexes, once per turn |
| `infantry_cannot_retreat` | §7.5 | infantry has no retreat |
| `melee_edges` | §7.1, §7.2, §7.4 | adjacency, wall/breach, gunboat immunity |
| `artillery_may_not_melee` | §7.4 | artillery is not a melee attacker |
| `advance_after_combat` | §6.82, §7.6, §7.7 | mandatory Dervish advance, restrictions |
| `phase_sequence` | §4 | Movement → Offensive Fire → Defensive Fire → Melee |
| `zone_of_control` | §5.26, §5.43 | ZOC stops movement through enemy ZOC |
| `stacking_limits` | §5.51, §5.52 | four-unit cap; Dervish tribe-mix ban |
| `gordon_immobile` | §9.346 | GORDON fixed at the Palace |
| `disrupted_unit_inert` | §5 | disrupted unit cannot act / projects no ZOC |
| `wrong_owner_cannot_fire` | §6.11 | off-turn player cannot fire |
| `out_of_range` | §6.22 | range-band halving rejects out-of-range fire |

Helper builders (`campaign_state`, `fall_of_khartoum_state`, `place`,
`alloc_synthetic`, `ae_infantry`, `dervish_camel`, `ae_maxim`, `ae_artillery`,
`ae_howitzer`, plus `fire_attack`/`melee_attack`) stand in for real counters
where the engine does not model a distinct `UnitId` (there is no
`UnitId::BareCounter`; synthetic units come from `GameState::alloc_unit_id`).

### Traceability caution

`omdurman-rules` is scanned for `§N` citations by the traceability test
(`cargo test -p omdurman-rules --test traceability`), which asserts each one
appears in a `[[mapping]]`. The vignette files carry citations in
`TacticsScript::new(name, "§…", …)` strings (all mapped), and the runner uses
`"\u{a7}"` at runtime — do **not** introduce a literal `§` in
`omdurman-rules/tests/tactics.rs`.

---

## 7. Tests (new + re-pointed)

| Test | Proves |
|---|---|
| `omdurman-rules/tests/tactics.rs` | all 22 vignettes replay clean from a fresh clone |
| `omdurman-bot/tests/head_to_head.rs` | both random agents play; caches capped; per-side identity; every real effect renders |
| `omdurman-bot/tests/log_format.rs` | header/footer, per-event lines, observations drained 1:1, turn boundaries, observer round-trip, byte-stable log |
| `omdurman-bot/tests/observer.rs` | tagged FINDINGS parsing, cross-chunk aggregation, graceful no-key / malformed degradation |
| `omdurman-bot/tests/strategy_corpus.rs` | every corpus citation maps in `traceability.toml`; briefs load per side/scenario; corpus >10k chars |
| `omdurman-bot/tests/{coverage,determinism,invariants,termination}.rs` | re-pointed from `PlayStrategy` to `Agents::random()` (behaviour unchanged) |

---

## 8. Cross-check: design doc vs implementation

`docs/omdurman-bot-three-agent-design.md` (dated 2026-08-08) is the blueprint;
the working tree implements it with these deviations:

- **CLI shape.** Design: long flags + `--out-dir`. Implemented: positional
  args (`play Campaign 123 random 30`), binary named `omdurman-bot-cli`.
- **No binary `game.record`.** The design's postcard `GameRecord` companion was
  not built; the text `game.log` is the only output.
- **`review` signature.** Design had `ReviewContext`; implemented as
  `review(log, config, &Completion, crib)`.
- **`tests/describe.rs` dropped.** Describe coverage was folded into
  `tests/head_to_head.rs`.
- **Added beyond the design.** The doctrine corpus (+ `doctrine.rs` + corpus
  test), the `tactics` subcommand, and the entire tactics suite (§6) postdate
  the design doc and are not mentioned in it.

---

## 9. Commands you can run right now (all verified green)

```shell
cargo test -p omdurman-rules --test tactics            # 22/22 vignettes
cargo test -p omdurman-bot                              # all bot suites
cargo test -p omdurman-rules --test traceability        # § mapping bijection intact
cargo run  -p omdurman-bot --bin omdurman-bot-cli -- tactics   # PASS ×22, exit 0
cargo check --workspace                                 # clean
```

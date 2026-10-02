# `omdurman-bot` + Tactics Suite — Manual

How to use the headless rule-verification stack: the `omdurman-bot` crate
(agents, playthrough driver, game log, audits, LLM observer), its CLI
`omdurman-bot-cli`, and the deterministic tactics vignette suite in
`omdurman-rules`. §-references are to the Phoenix Enterprises (1982) manual.

---

## 1. Overview

```
  Agent AE ─────┐                                        ┌─> audit (deterministic)   ─> report, exit 1 on Error
                ├─> rules engine ─> GameLog (text) ──────┤
  Agent Dervish ┘   (apply_effect)                       └─> review (LLM observer)   ─> findings.md / .json
                         │
                         └──────> events.jsonl (replay record) ─> audit-record, app replay viewer
```

- **Two independent agents**, one per faction, each with its own strategy
  (and, for LLM sides, its own cache and brief).
- **The engine is authoritative.** The driver never bypasses `apply_effect`.
  Engine observations are drained into the log as a side-channel, so the
  `GameEvent` trace stays byte-for-byte deterministic for a given seed.
- **Two artifacts per game:** a human/LLM-readable text log and an
  app-compatible `events.jsonl` replay record.
- **Verification layers**, from hard to advisory:
  1. engine validation (`apply_effect` / `can_*`);
  2. hard invariants (`invariants::check_all_with_tribal`) after every effect,
     in the proptest and adversarial test suites;
  3. deterministic scanners over the log (`audit`) and the record
     (`audit-record`);
  4. the LLM observer (`review`), whose findings are **advisory only**: rule
     *misapplications* the invariants can't encode (wrong CRT row, missed
     modifier, phase-order slip, FoK deltas).

The library also builds for `wasm32`: the app embeds the historical
commanders (§4.4) as its in-game AI. Only the CLI's Tokio runtime and `.env`
loading are native-only.

---

## 2. Quick start

Run from the repository root, so the pinned toolchain (`rust-toolchain.toml`)
applies and the relative `games/` output lands in the repo (it is gitignored).

```shell
# Tactics vignette suite (rules crate) -- 25 scripts, deterministic
cargo test -p omdurman-rules --test tactics

# Whole bot test suite
cargo test -p omdurman-bot

# CLI
cargo run -p omdurman-bot --bin omdurman-bot-cli -- tactics
cargo run -p omdurman-bot --bin omdurman-bot-cli -- play Campaign 123 random 30
cargo run -p omdurman-bot --bin omdurman-bot-cli -- play FallOfKhartoum 777 commanders
cargo run -p omdurman-bot --bin omdurman-bot-cli -- audit game.log
cargo run -p omdurman-bot --bin omdurman-bot-cli -- audit-record games/game_bot_<ts>/events.jsonl
cargo run -p omdurman-bot --bin omdurman-bot-cli -- review game.log findings
cargo run -p omdurman-bot --bin omdurman-bot-cli -- run run.json
```

Everything except the LLM paths runs offline. The LLM paths (`llm`-based
presets, `review`) need an API key; the CLI loads `.env` at startup:

| Env var | Default |
|---|---|
| `LLM_API_KEY` (falls back to `OPENAI_API_KEY`) | none |
| `LLM_BASE_URL` | `https://api.openai.com/v1` |
| `LLM_MODEL` | `gpt-4o-mini` |

Without a key, an LLM side plays the aggressive heuristic (§4.2) for every
pick, and `review` returns an empty report with a "review skipped" summary.

---

## 3. CLI — `omdurman-bot-cli`

Declared as a `[[bin]]` in `omdurman-bot/Cargo.toml` (`src/main.rs`). All
arguments are positional; the lib holds the logic, so tests exercise it
without spawning a process.

```
omdurman-bot-cli play         [scenario] [seed] [strategy] [max_turns] [log_file]
omdurman-bot-cli review       [log_file] [findings_prefix]
omdurman-bot-cli audit        [log_file]
omdurman-bot-cli audit-record <events.jsonl>
omdurman-bot-cli run          [run.json]
omdurman-bot-cli tactics
omdurman-bot-cli help | -h | --help      (also printed with no arguments)
```

Exit codes: `0` success; `1` on a failed tactics script, an `audit` Error, or
an `audit-record` violation; `2` for an unknown subcommand or `audit-record`
without a path.

### 3.1 `play`

- **scenario** (case-insensitive): `fok` / `fallofkhartoum` /
  `fall_of_khartoum` → Fall of Khartoum; `historical` → Historical; anything
  else (default) → Campaign.
- **seed**: decimal `u64`; missing or unparsable → drawn from the system RNG
  (printed in the summary as hex).
- **strategy**: one of the presets below (exact, lowercase). An unknown name
  silently falls back to `random`.
- **max_turns**: turn ceiling (default 30); the game also stops at game over.
- **log_file**: default `game.log`. The arguments are positional, so setting
  it requires all the preceding ones.

| Preset | Anglo-Egyptian | Dervish | Notes |
|---|---|---|---|
| `random` (default) | Random | Random | fastest, broadest coverage |
| `llm` | LLM + doctrine | LLM + doctrine | brief = `doctrine_brief` per side |
| `ae` | LLM + doctrine | Random | |
| `dervish` | Random | LLM + doctrine | |
| `aggressive`, `agg` | Aggressive | Aggressive | |
| `ae-agg` | Aggressive | Random | |
| `dervish-agg`, `agg-dervish` | Random | Aggressive | historical swarm on GORDON (§9.346) |
| `commanders`, `kitchener-vs-khalifa`, `kitchener_vs_khalifa` | Kitchener | Khalifa | tuning match-up; same code as the in-app AI |
| `ae-kitchener` | Kitchener | Random | |
| `dervish-khalifa` | Random | Khalifa | |
| `storm`, `dervish-storm` | Random | LLM + `storm_brief` | all-out assault on the Palace |
| `siege`, `fortress` | LLM + `fortress_brief` | LLM + `horde_brief` | |
| `laststand`, `drama`, `final` | LLM + `defender_brief` | LLM + `besieger_brief` | in FoK also sets a keep-out zone: the Dervish may not end a move within 2 hexes of the Palace before turn 5 |

The scripted briefs (`storm_brief` etc., in `src/doctrine.rs`) are the side's
normal doctrine brief plus appended override orders.

**Outputs:** the text log (§5), plus a replay record
`games/game_bot_<UTC timestamp>/events.jsonl` relative to the working
directory: a `{"seed":N}` header line, then one JSON `RecordedEvent` per line
(the app's own record format). Open it in the app via Lobby → Saved games,
or check it with `audit-record`. A one-line summary (scenario, seed, turns,
events, observations) is printed.

### 3.2 `run`

One JSON spec (default `run.json`) to play and optionally review in one go:

```json
{
  "scenario": "fok",
  "seed": 777,
  "ae_strategy": "kitchener",
  "dervish_strategy": "llm",
  "max_turns": 8,
  "output_log": "game.log",
  "output_findings": "findings",
  "review": true
}
```

| Key | Required | Meaning |
|---|---|---|
| `scenario` | yes | same names as `play` |
| `ae_strategy`, `dervish_strategy` | yes | per-side name, case-insensitive: `random` / `rand` / `""`, `aggressive` / `agg`, `kitchener`, `khalifa`, `llm` / `llm-advised` / `llm_advised` (brief = `doctrine_brief` for that side). Unknown → warning, then Random. |
| `seed` | no | `u64`; default from the system RNG |
| `max_turns` | no | default 30 |
| `output_log` | no | default `game.log` |
| `output_findings` | no | findings prefix, default `findings` |
| `review` | no | run the observer afterwards (default `false`) |

The preset names from §3.1 are not accepted here; compose the two sides
explicitly. Outputs are the same as `play`, plus the findings files when
`review` is true.

### 3.3 `review`

Reads a log (default `game.log`), runs the LLM observer (§6) with the rules
crib sheet `docs/rules_crib_sheet.md`, and writes `{prefix}.md` and
`{prefix}.json` (default prefix `findings`). The crib-sheet path is baked in
at compile time from the crate directory; a missing file silently yields an
empty crib sheet.

### 3.4 `audit`

Deterministic scanners over a rendered log (`src/audit.rs`). Each finding
is an **Error** (a rule the engine must enforce was demonstrably broken) or a
**Warning** (usually a violation, but the log alone cannot exclude a legal
explanation). Exit 1 on any Error.

- §6.82/§7.6: every advance needs a combat-opened vacated-hex window (Error).
- §6.14: fired-at-once per unit and subphase (Warning: stacked occupants,
  mid-phase turnover and the §6.42 subphase reset can explain it).
- §9.111 / §9.211 / §9.212: setup force composition and not-in-play units (Error).
- GORDON uniqueness and scenario presence (Error).
- §8.2: desertion count and exemptions (Error).
- §9.112 / §9.113: reinforcement schedule: wave membership, turn window,
  three-gunboat quota (Error/Warning).
- Board-state reconstruction: §5.51/§5.52 stacking, §7.1 enemy
  cohabitation, §5.11 MP arithmetic.

Run it over a fixed-seed matrix after any rules-engine change.

### 3.5 `audit-record`

Replays an `events.jsonl` record through `apply_effect` from its
`StartGame` and reports, per effect: a rejection on replay (nondeterminism),
a stacking-invariant violation (§5.51–§5.53), and a wall trespass (§5.23)
by a one- or two-hex `MoveUnit` / `RetreatBeforeMelee` / `AdvanceAfterCombat`
(a breached wall is legal, §6.63). Exit 1 on any violation. Works on the
app's own saved games too.

### 3.6 `tactics`

Replays every tactics vignette (§8) from a fresh clone of its state and
prints `PASS  <name> [<citation>]` or `FAIL … step N (<note>) -- <reason>`,
then `all 25 tactics scripts passed`. Exit 1 on any failure.

---

## 4. Agents — `src/agent.rs`

`Agents { ae, dervish }` holds one `AgentStrategy` per faction
(`Agents::random()` is the default). The driver
`playthrough(scenario, seed, cfg, agents) -> PlayResult` (async,
`src/playthrough.rs`) enumerates candidates with `actions::legal_actions`
(or `legal_actions_deep_setup`, with per-hex deployment options, when a
commander plays) and asks the side that owns the phase to pick: the active
player, except in Defensive Fire, where the non-moving player fires. The
mandatory arrivals (the §8.2 desertion roll, reinforcement waves) are forced
through regardless of strategy.

`PlayConfig` holds `max_actions_per_phase` (anti-stall, default 200),
`max_turns` (default 30) and an optional `keep_out` pacing zone (used by the
`laststand` preset; it only filters candidates and never touches the engine).
`PlayResult` carries `events`, `log`, `llm_annotations`,
`ae_final_cache` / `dervish_final_cache`, `seed`, `final_state`,
`variant_coverage`, `actions_taken` and `observations_total`.

### 4.1 `Random`

Uniform over the candidate list, drawn from the seeded `BotRng` (which wraps
the engine's `GameRng`). Fastest; broadest raw coverage.

### 4.2 `Aggressive` — `src/aggressive.rs`

Greedy: scores every candidate and takes the best (ties broken by `BotRng`).
Melee over advance-after-combat over fire over wall breaching; movement is
scored by progress toward the objective (the Palace for the Dervish in Fall
of Khartoum, otherwise the nearest enemy); never retreats (§7.5); ends the
phase only when nothing better remains.

### 4.3 `LlmAdvised { config, brief }` — `src/llm.rs`

- Once per side-turn, at the start of that side's Movement phase, the
  driver calls `advise_turn` with the side, its brief (prepended to the
  system prompt), the state, the indexed candidate list and the side's own
  500 KB `LlmCache`.
- The returned indices are resolved into concrete actions against that
  candidate list; out-of-range indices and `AdvancePhase` entries are
  dropped. Each later pick takes the first plan entry that matches a current
  candidate *by intent* (ignoring pre-rolled dice); stale entries are
  dropped with a `[note, …]` log line.
- When no plan entry matches (or there is no plan: no key, an API error, an
  unparsable reply), the pick falls back to the aggressive heuristic.
- The reply's `cache` overwrites the side's cache; each reasoning string is
  logged as a `[reasoning, …]` line and kept as an `LlmAnnotation`.

The wire format is specified in `docs/llm-response-protocol.md`. The brief
for the `llm`, `ae`, `dervish` presets and for `run.json` comes from the
doctrine corpus (§7).

### 4.4 `Commander(Kitchener | Khalifa)` — `src/commanders.rs`

The two historical commanders, with scenario-adaptive doctrine distilled
from `docs/strategy/` (no LLM). **Kitchener** (Anglo-Egyptian): massed fire,
brigade integrity, Maxim second fire, counter-battery, hold the wall/gate
line and the Palace ring in Fall of Khartoum, and the Mahdi's Tomb axis in
the Campaign. **Khalifa** (Dervish): in Fall of Khartoum a race to kill
GORDON (close under night cover, mass by tribe, breach, storm), in the
Campaign waves, ZOC screens and a guarded Khalifa. During Setup,
`commanders::pick_setup` scores each deployment by its owner's doctrine.

**In-app AI.** `omdurman-app/src/bot_player.rs` drives AI seats with the
same `Commander::pick` API (`Commander::for_player`: Kitchener for the
Anglo-Egyptian, Khalifa for the Dervish), over the bot's own candidate
generator (`actions::legal_actions` / `legal_actions_deep_setup`) and
`BotRng`. The headless `commanders` preset runs the identical code path, so
tuning sessions and in-app games agree.

**Fire planning — `src/fire_plan.rs`.** The enumerator offers one attack per
firing stack and target, but a hex may be fired at only once per phase
(§6.14). In every fire phase the commanders merge those per-stack shots into
combined attacks (`combine_fire_attacks`): each weapon group (a stack's main
weapons of one kind, a named gunboat's Maxims) goes to the target where it
adds the most expected value — the CRT row, the mandatory and terrain/fort/
hexside modifiers, gunboat 3+ / fort 2+ thresholds, the howitzer's 40% on
target, all weighted by §9.14 unit values. Melee is scored the same way
(`melee_value`: expected VP inflicted minus expected VP lost, both sides
rolling at once, §7.3/§7.7). Planning never reads the dice already embedded
in a candidate.

**Fire lanes and paths — `src/threat.rs`.** `fire_reaching` sums the enemy
fire that can reach a hex (§6.22 bands, §8.1 night ranges, terrain LOS),
cached per enemy layout; `melee_reaching` the spears that can reach it next
turn; `path_cost` is a Dijkstra movement-point field (walls open only at
gates and breaches). Off the FoK walls the Khalifa stages out of the
Maxims' lanes and crosses them only into contact; Kitchener kites at rifle
range (his 5 hexes against their 4, artillery 8 against the forts' 7),
closes on the field army before the forts, shelters leaders with the
safest stack, and dashes for the Mahdi's Tomb only once the city is
cleared. The Khalifa garrisons the Tomb with his Taiasha and chooses §8.2
deserters from the disrupted and the units in the fire lanes
(`commanders::choose_deserters`, also used by the app).

**Measuring strength — `src/arena.rs`, `tests/arena.rs`.** `arena::play`
plays a whole game through the app's decision path with either the live
commanders (`Version::Current`) or the frozen pre-tuning ones
(`src/baseline.rs`, `Version::Baseline`) on each side. The `#[ignore]`d
harness plays every pairing on the same seeds and prints wins, the signed
result (`ae_score`: Campaign VP superiority, Historical net level x10, FoK
level), losses and turns:

```sh
cargo test --release -p omdurman-bot --test arena -- --ignored --nocapture
ARENA_SEEDS=20 ARENA_SCENARIOS=fok,historical,campaign ARENA_VERBOSE=1 ...
ARENA_TRACE=1 ARENA_GAME=campaign:1000:current:baseline \
  cargo test --release -p omdurman-bot --test arena_trace -- --ignored --nocapture
```

---

## 5. Game log — `src/log.rs` + `src/describe.rs`

One plain-text file that gives an auditor enough context on its own (no live
engine access). Rendered by `GameLog::render()`; byte-identical for a given
seed (tested).

```
GAME LOG — Remember Gordon! (The Battle of Omdurman)
scenario:        campaign
seed:            0x7b
agents:          ae=random dervish=llm(<brief>)

[0] T1 Setup Dervish  DeployUnit …
[41] T1 Movement AngloEgyptian  MoveUnit <unit>: (q,r) → (q,r) (1 MP) mp 1/8 via [(q,r) → (q,r)]
[52] T1 Offensive Fire AngloEgyptian  <fire attack …> [roll 7]
      → UnitEliminated: <unit> <cause> [<VP source>]  [event 52]
[reasoning, Dervish T2] - 3: <reason> (§9.112)
[note, T2] plan entry no longer legal in Movement: … -- dropped
=== Turn 1 complete (<time>, Day) — 5 fire, 1 melee, 2 eliminations, 0 advances, 0 retreats, 12 reinforcements; VP AE 4 / Dervish 0 ===
    - <per-turn event record>

=== GAME OVER ===  result: <result>
victory: AE 12 / Dervish 3
```

- **Event lines** `[seq] T<turn> <phase> <actor>  <action>`: `seq` is the index
  in the event trace; the phase is the top-level name (`Setup`, `Movement`,
  `Defensive Fire`, `Offensive Fire`, `Melee`); the actor is the side that
  acted (the non-moving side in Defensive Fire, the firer or melee attacker
  otherwise).
- `describe_effect` names units via their profile labels, prints hexes as
  `(q,r)` and paths as ordered routes, shows cumulative MP against the
  allowance for moves, and spells out pre-rolled dice, so an auditor can
  re-derive CRT lookups and MP arithmetic.
- **Observation lines** (`→ … [event seq]`) come from the engine's
  `Observation`s via `describe_observation`; § citations are the engine's.
- **Turn boundary**: a count summary with the running VP ledger, then one
  line per turn-event record (`describe_turn_event`).
- **`[reasoning, …]`** lines appear only for LLM sides; **`[note, …]`** lines
  are driver annotations (dropped plan entries, rejected picks).

---

## 6. Offline observer — `src/observer.rs`

`review(log, config, completion, crib) -> ObserverReport` feeds the log to the
LLM **turn by turn**: `chunk_log` splits it at the `=== Turn N complete ===`
markers, every chunk carries the log header, the crib sheet goes with the
first chunk only, and a running cache is carried between chunks. Each reply
is one JSON object (`ReviewResponse`, enforced via `response_format`);
malformed findings are dropped individually, a failed or malformed chunk
keeps the previous cache, and findings are de-duplicated on
`(severity, seq, section)`. `Completion` is a small trait so tests run on a
canned transport; `ReqwestCompletion` wraps
`omdurman_net::llm::request_completion`.

`ObserverReport` holds the findings (`Severity` ∈ Critical, Error, Warning,
Info), the last non-empty summary, `turns_audited` and `events_audited`. The
full reply contract is in `docs/llm-response-protocol.md`.

The crib sheet `docs/rules_crib_sheet.md` is a checked-in summary of the
manual. The observer is told to cite only sections it contains, so keep it
accurate and complete: its errors become the auditor's errors.

---

## 7. Doctrine corpus — `src/doctrine.rs` + `docs/strategy/`

`doctrine_brief(player, scenario)` concatenates `common_doctrine.md`, the
faction file (`anglo_egyptian_doctrine.md` / `dervish_doctrine.md`) and, in
Fall of Khartoum, `fall_of_khartoum_doctrine.md`. The files are read at run
time from the source tree (a path baked in at compile time); missing files
are skipped. Only LLM sides use the brief. See `docs/strategy/README.md` for
the format and `tests/strategy_corpus.rs` for the citation check.

---

## 8. Tactics suite — `omdurman-rules/src/tactics.rs`

A human-readable regression suite for the rules engine. A **tactics script**
(`TacticsScript`) is a hand-built `GameState` plus ordered steps:

- `ScriptStep::Legal { note, effect }`: `apply_effect` must return `Ok`.
- `ScriptStep::Illegal { note, probe, effect }`: must return `Err` matching
  the `Probe` (`Probe::Any(label)`, or `Probe::matched(label, predicate)` on
  the `RuleError`).
- `ScriptStep::Assert { note, predicate }`: a closure over the state.

`run_step(&mut GameState, &ScriptStep) -> Option<String>` returns the first
failure. `all_scripts()` returns the 25 vignettes below, in order. The runner
(`omdurman-rules/tests/tactics.rs`) and the CLI `tactics` subcommand replay
each from a fresh clone of its state. Pre-rolled dice ride inside the
effects, so every script is deterministic.

| Script | Citation | Exercises |
|---|---|---|
| `movement_allowance` | §4, §5.11, §5.12 | allowance cap, MP spent tracked |
| `walled_city_entry_artillery` | §5.23 | Dervish artillery may enter the walled city |
| `walled_city_entry_denied` | §5.23 | other tribes denied; a wall hexside blocks movement |
| `gunboat_river_move` | §5.22, §5.24 | Nile movement; gunboats never leave the river |
| `artillery_sinks_gunboat` | §6.22, §6.61 | only artillery sinks gunboats, Eliminate(3)+ |
| `artillery_destroys_fort` | §6.22, §6.62 | artillery destroys forts, Eliminate(2)+ |
| `maxim_second_fire` | §6.14, §6.42 | Maxim fires in Direct and again in the second subphase |
| `howitzer_on_target` | §6.22, §6.64 | impact roll 7–10 hits the designated hex |
| `howitzer_scatter_miss` | §6.64 | impact roll below 7 scatters |
| `no_howitzer_at_night` | §6.64, §8.1 | no howitzer fire at night |
| `retreat_before_melee` | §7.5, §7.7 | cavalry/camel retreat two hexes, once per turn |
| `infantry_cannot_retreat` | §7.5 | only cavalry and camels retreat |
| `melee_edges` | §7.1, §7.2, §7.4 | adjacency, wall blocking, melee-capable attackers |
| `artillery_may_not_melee` | §7.4 | artillery defends but never attacks |
| `advance_after_combat` | §6.82, §7.6, §7.7 | adjacent non-artillery attacker advances |
| `advance_requires_vacated_hex` | §6.82, §7.6 | an empty-but-never-vacated hex is not a target |
| `advance_requires_participation` | §6.82, §7.6 | only combat participants may advance |
| `phase_sequence` | §4 | Movement → Defensive Fire → Offensive Fire (Direct, then Maxim/howitzer) → Melee, then the other side's Movement |
| `zone_of_control` | §5.26, §5.43 | enter an enemy ZOC and stop; no passing through |
| `stacking_limits` | §5.51 | four non-leader units per hex |
| `stacking_tribe_mix` | §5.52 | Dervish tribes may not share a hex |
| `gordon_immobile` | §9.346 | GORDON may not move |
| `disrupted_unit_inert` | §5 | a disrupted unit may not move, fire or melee |
| `wrong_owner_cannot_fire` | §6.11 | only the phasing side fires offensively |
| `out_of_range` | §6.22 | targets beyond the range band are rejected |

Private helpers (`campaign_state`, `fall_of_khartoum_state`, `place`,
`alloc_synthetic`, `ae_infantry`, `dervish_camel`, `ae_maxim`,
`ae_artillery`, `ae_howitzer`, `fire_attack`, `melee_attack`) build the
states; synthetic units come from `GameState::alloc_unit_id`.

**Traceability.** The traceability test scans every workspace `.rs` file for
`§N` citations, including the `TacticsScript::new(name, "§…", …)` strings
and doc comments, and each must map to a `[[mapping]]` in
`docs/traceability.toml`. A bare `§` not followed by a section number (such
as the runner's `"§{}"` format string) is ignored.

---

## 9. Tests

| Test | Proves |
|---|---|
| `omdurman-rules/tests/tactics.rs` | all 25 vignettes replay clean |
| `omdurman-bot/tests/determinism.rs` | same seed → byte-identical event trace |
| `omdurman-bot/tests/coverage.rs` | the common `GameEffect` variants all occur over a batch of games |
| `omdurman-bot/tests/termination.rs` | every playthrough ends within the caps |
| `omdurman-bot/tests/invariants.rs` | proptest: invariants hold after every effect on a replayed trace; `game_over` is monotonic |
| `omdurman-bot/tests/adversarial.rs` | `apply_effect` rejects illegal effects: directed regressions, a fire-sweep proptest, a mutation fuzzer (reject or keep invariants, never panic) |
| `omdurman-bot/tests/head_to_head.rs` | both agents play, caches capped, per-side identity, every real effect renders via `describe_effect` |
| `omdurman-bot/tests/log_format.rs` | header/footer, event lines, observations, turn boundaries, byte-stable log, observer round-trip |
| `omdurman-bot/tests/observer.rs` | JSON findings parsing, cross-chunk aggregation, malformed-item and no-key degradation |
| `omdurman-bot/tests/strategy_corpus.rs` | corpus citations map in `traceability.toml`; briefs load per side and scenario; corpus > 10k chars |
| `omdurman-bot/tests/playability.rs` | every effect variant the bot emits in Fall of Khartoum has a UI path in the app |

---

## 10. Design notes

The three-agent design (two players plus an offline observer) was
implemented with these deliberate differences from the original plan:

- **CLI:** positional arguments and a binary named `omdurman-bot-cli`, not
  long flags with `--out-dir`. `run.json` is an input spec, not an output
  manifest.
- **Replay record:** a JSON `events.jsonl` in the app's record format, not a
  postcard `game.record`.
- **`review` signature:** `review(log, config, &Completion, crib)` instead of
  a `ReviewContext`.
- **Describe coverage** lives in `tests/head_to_head.rs`, not a separate
  `tests/describe.rs`.
- **Added later:** the doctrine corpus, the scripted briefs, the `Aggressive`
  and `Commander` strategies, the deterministic `audit` / `audit-record`
  scanners, and the tactics suite.

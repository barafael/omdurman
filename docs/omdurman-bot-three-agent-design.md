# `omdurman-bot` — Three-Agent Rule-Verification Design

Status: **design — not yet implemented**. Review before coding.

## 1. Purpose

Two agents play *Remember Gordon!* against each other (Anglo-Egyptian vs
Dervish); a third agent observes. The goal is **not** to train the agents or to
produce strong play — it is to **validate that the game implements the rules
correctly**. The observer reads the game log and flags rulebook violations.

This is the "LLM trace review" item in `docs/omdurman-bot-plan.md` (Phase 4),
plus the missing per-side agent split.

## 2. What exists today

- `omdurman-bot::playthrough` runs one driver loop with a **single**
  `PlayStrategy` (`Random` or `LlmAdvised`) applied to whichever player is
  active, sharing a **single** `LlmCache` across both sides. No per-side
  identity or memory.
- The playthrough records only `GameEvent::Effect(...)` events. It never
  drains `GameState::observations` / `turn_events`, so the trace carries *what
  was attempted* but not *what the engine ruled happened* (eliminations,
  combat results, VP, breaches, § citations).
- The engine *does* surface exactly the context an auditor needs after every
  `apply_effect`, as `Observation`s (already carrying `paragraphs` §-citations):
  `FireResolved`, `MeleeResolved`, `UnitEliminated`, `WallBreached`,
  `DemolitionResolved`, `LeaderKilled`, `VictoryScored`, `GordonEliminated`,
  `FortDestroyed`, `FriendliesDisembarked`.
- `omdurman-net::llm` provides the native-only `request_completion` transport
  and `LlmConfig`.

## 3. Design overview

```
  Agent AE ──┐
             ├──> engine (apply_effect) ──> per-step log  ──> Observer ──> findings.md
  Agent Dervish ─┘
        (each: own strategy, own cache, own persona)      (offline review pass)
```

- **Players:** two independent agents, configured **per side**. Each has its
  own strategy (`Random` or `LlmAdvised`), its own 500 KB `LlmCache`, and its
  own persona system-prompt.
- **Log:** a single text file with a header, one line per effect (turn, phase,
  acting side, human-readable action, pre-rolled dice, resulting observations
  with § citations), turn-boundary summaries, and interleaved agent reasoning.
  This is the observer's ground truth and must "give enough context" on its
  own — no live engine access.
- **Observer:** offline pass over the log. Because a game is thousands of
  events, it reviews the log in **turn-sized chunks**, carrying a running
  notes/findings cache between chunks (same tagged-response pattern as the
  players' cache), and emits a structured findings report.

## 4. Per-side agents

New `src/agent.rs`:

```rust
/// What an agent is allowed to be. Owned separately by each side.
pub enum AgentStrategy {
    Random,
    LlmAdvised {
        config: omdurman_net::llm::LlmConfig,
        /// Short strategic brief prepended to the system prompt, e.g.
        /// "You command the Dervish. Your infantry must close to melee while
        /// the Khalifa survives; you win by British losses."
        brief: String,
    },
}

/// One strategy per faction.
pub struct Agents {
    pub ae: AgentStrategy,
    pub dervish: AgentStrategy,
}
```

Changes to `playthrough.rs`:

- `playthrough(scenario, seed, cfg, agents)` takes `Agents` instead of a single
  `PlayStrategy`. In the loop, dispatch on `state.active_player`:
  - `Random` side → `rng.choose(&candidates)` (unchanged).
  - `LlmAdvised` side → per-side `LlmCache`, refreshed once per player-turn
    with a persona-prefixed system prompt. `advise_turn` gains a `brief: &str`
    parameter; the existing `build_prompt` already frames Friendly/Enemy from
    `active_player`, so one code path serves both sides.
- After each successful `apply_effect`, **drain** `state.observations`
  (`std::mem::take`) into the log stream. Observations are a derived
  side-channel (regenerated identically on replay), so draining never changes
  the `GameEvent` trace and byte-for-byte determinism of `events` is preserved.
- At each turn boundary, drain `state.turn_events` / emit the `TurnSummary`.
- `PlayResult` grows:
  - `ae_final_cache` / `dervish_final_cache: Option<String>`,
  - `log: GameLog` (see §5) instead of the bare `events` only (`events` stays,
    it is the replayable record),
  - `observations_total: usize` (stats: how many eliminations, combat
    resolutions, etc. the observer can audit).

## 5. The log — "enough context"

New `src/log.rs`. One plain-text file, LLM-readable, plus a machine-readable
`GameRecord` (postcard/serde) companion for the app's timeline.

### Header

```
GAME LOG — Remember Gordon! (The Battle of Omdurman)
scenario:        campaign
seed:            0x9e3779b97f4a7c15
agents:          ae=llm("1st Yorkshires")  dervish=random
rules_version:   Manual §1–§10
```

### Per-event line

```
[seq] T{turn} {Phase} {AngloEgyptian|Dervish}  {action}  => {observations}
```

Concretely (`src/describe.rs` — one `describe_effect` / `describe_observation`
pair):

```
[0231] T4 Fire AngloEgyptian  1st Yorkshire (4,-1) fires 8 factors at Khalifa (3,2)
       => roll 4 +2 (terrain+brigade) = 6 -> Disrupted; 0 eliminated [§6.22 §6.24 §6.42]
[0232] T4 Fire AngloEgyptian  2nd Yorkshire Maxim second fire at Dervish spear (3,3)
       => roll 7 -> Eliminate; 1 Dervish eliminated [§6.42]
[0233] T4 Movement Dervish  Mulazmin (2,4) -> (3,4) via [(2,4),(3,4)] (1 MP)
       => (no observations)
[0240] T4 AdvancePhase  -> turn 5 complete (see turn summary)
```

The dice travel in the effect and are spelled out (`roll 4 +2 = 6`), so the
observer can re-derive CRT lookups and movement-cost arithmetic from the log
alone.

### Turn boundary

```
=== Turn 4 complete ===
combat: 3 fire, 1 melee | eliminations: 2 AE, 1 Dervish | VP: AE 4, Dervish 0
```

### Agent reasoning

Interleaved where an LLM agent moved, tagged with its side, from the existing
`LlmAnnotation`s:

```
[reasoning, Dervish T5] 3: advance Mulazmin toward Palace — §9.322 entry edge
```

### Formatting rules

- `describe_effect` names units via `profile_for_unit(id)` →
  `UnitIdentity` (fall back to `Debug`), prints hexes as `(q,r)` and paths as
  the ordered route so MP can be re-verified against the terrain chart.
- `describe_observation` prints the `paragraphs` list verbatim — the engine is
  the authoritative source of § citations.
- Lines are stable: same seed → byte-identical log (tested).

## 6. The observer (`src/observer.rs`)

Offline review pass over the log file.

```rust
pub struct Finding {
    pub severity: Severity,        // Critical | Error | Warning | Info
    pub seq: usize,                // event sequence number
    pub section: Option<String>,   // cited §
    pub explanation: String,
}
pub struct ObserverReport {
    pub findings: Vec<Finding>,
    pub summary: String,           // the LLM's closing assessment
    pub turns_audited: usize,
    pub events_audited: usize,
}
pub async fn review(
    log: &str,                     // the full game log text
    ctx: &ReviewContext,           // rules crib sheet + manual excerpts (see §6.2)
    completion: &impl Completion,  // mockable (§6.3)
) -> ObserverReport;
```

### 6.1 Chunking — the game is too big for one prompt

Split the log into per-turn chunks (a turn is a few hundred lines at most).
Feed the LLM sequentially:

```
=== CHUNK i/total ===
{rules crib sheet (first chunk only)}
=== RUNNING CONTEXT FROM PREVIOUS CHUNKS ===
{observer cache: findings so far + open questions, ≤ 500 KB}
=== LOG, TURN {t} ===
{...}

Respond with one JSON object (enforced by `response_format: json_object`):
```json
{
  "cache": "<your working notes / open threads>",
  "findings": [
    {"severity": "warning", "seq": 12, "section": "5.24",
     "explanation": "gunboat may have exceeded upstream allowance"}
  ]
}
```

The `ReviewResponse` / `Finding` JSON protocol mirrors the players' advisor
(`bot::llm::PlanResponse`), so both speak the same serde machinery — and
`#[serde(default)]` gives the same degrade semantics: missing or malformed
sections → keep previous cache, log a warning, continue; malformed individual
findings are dropped while well-formed siblings survive.

### 6.2 Rules context for the observer

The observer cannot hold the whole manual. Provide a curated crib sheet
(`docs/rules_crib_sheet.md`, seeded from
`Boardgame - Remember_Gordon/Manual/RememberGordonManual.md`): movement-cost
chart, fire/melee CRT rows, stacking rules, the day/night and turn track,
victory conditions, FoK deltas. The LLM is told: "Where the log contradicts a
crib-sheet rule, flag it with the § citation; where you are unsure, use
`Warning` and say what's ambiguous." The crib sheet is a checked-in file, so
its drift from the manual is itself auditable.

### 6.3 Mockable completion (tests without network)

`request_completion` is a concrete free function; the observer takes a small
trait instead:

```rust
#[async_trait] // or futures::Future in a generic bound
pub trait Completion {
    async fn complete(&self, cfg: &LlmConfig, system: &str, user: &str, max_tokens: u32)
        -> Result<String, LlmError>;
}
```

Real impl wraps `omdurman_net::llm::request_completion`; tests use a canned
impl returning a fixed JSON `ReviewResponse` to assert the parser, chunking, and
report aggregation without a network call.

### 6.4 Findings report

Two outputs from one `ObserverReport`:

- `findings.md` — human/PR-readable: header stats (`turns_audited`,
  `events_audited`, counts per severity) then one section per finding
  (severity, seq, §, explanation, turn).
- `findings.json` — the `Vec<Finding>` serialized, so a downstream step (CI
  gate, triage) can process it.

### 6.5 Relationship to the deterministic invariants

The observer is the **top layer** of a three-layer verification stack, not a
replacement for anything:

| Layer | Mechanic | Catches |
|---|---|---|
| 1. Engine predicates | `can_*` / `apply_effect` validation | Illegal moves at the input gate |
| 2. Hard invariants (`bot::invariants`, proptest) | 7 predicates after every effect | Illegal *states* (stacking, Nile boats, MP over-spend, …) |
| 3. LLM observer | reads log vs crib sheet | Rule *misapplications* the invariants don't encode (wrong CRT row, wrong modifier, missed VP, phase-order slip, FoK-only deltas) |

Because layer 3 is an LLM, its findings are **advisory**: they surface
suspicions for a human (or a follow-up) to triage. Automated gating stays on
layers 1–2 + determinism. The value of the observer is breadth — it reviews
*every* line of long games that invariants can't express.

## 7. CLI

Add a thin `src/main.rs` binary (`omdurman-bot` already exists as a lib; a
`[[bin]]` section goes in its `Cargo.toml`).

```
omdurman-bot play  --scenario campaign|fok [--seed N] \
                   --ae random|llm[:"brief"] --dervish random|llm[:"brief"] \
                   [--max-turns N] --out-dir logs/run_01
    # writes logs/run_01/game.log, logs/run_01/game.record, logs/run_01/run.json

omdurman-bot review --log logs/run_01/game.log [--crib docs/rules_crib_sheet.md] \
                    --out-dir logs/run_01
    # writes logs/run_01/findings.md + findings.json

omdurman-bot run    --... (play then review)  # the "two agents + observer" invocation
```

`run.json` carries header metadata (seed, scenario, agent config, caches) so a
review run is fully reproducible. The bin stays a thin wrapper; all logic lives
in the lib so tests exercise it without spawning a process.

## 8. File-by-file changes

| File | Change |
|---|---|
| `src/agent.rs` | **new** — `AgentStrategy`, `Agents` |
| `src/describe.rs` | **new** — `describe_effect`, `describe_observation`, unit-name helper |
| `src/log.rs` | **new** — `GameLog`, per-event line writer, turn-boundary writer |
| `src/observer.rs` | **new** — `Completion` trait, chunked `review`, `Finding`, `ObserverReport`, JSON parser (`ReviewResponse`, sharing `bot::llm::strip_json_fence`) |
| `src/playthrough.rs` | per-side dispatch, per-side caches, drain observations/turn_events, build `GameLog`; extend `PlayResult` |
| `src/llm.rs` | `advise_turn` gains `brief`; expose/refactor the tagged parser for reuse |
| `src/main.rs` | **new** — `play` / `review` / `run` subcommands |
| `Cargo.toml` | add `[[bin]]`; nothing else (deps already present) |
| `docs/rules_crib_sheet.md` | **new** — observer's rules context |

## 9. Tests

| Test | Proves |
|---|---|
| `tests/head_to_head.rs` | Two `Random` agents: deterministic (same seed → identical `events` **and** identical `game.log`); per-side caches are separate (an LLM-briefed AE never leaks into the Dervish cache). |
| `tests/log_format.rs` | Log has header, one line per effect, observations with § paragraphs present, turn-boundary summaries, reasoning interleaving; byte-stable across runs. |
| `tests/observer.rs` | With a canned `Completion`: chunk boundaries are respected (all turns audited), tagged response parsed, findings aggregated into `ObserverReport`, malformed responses degrade gracefully. |
| `tests/describe.rs` | `describe_effect` for a representative sample of the 26 variants produces non-trivial text containing the unit names and hexes (guards against a `Debug`-fallback regression). |

## 10. Phasing

| Phase | Content | Risk |
|---|---|---|
| A | `describe.rs` + `log.rs` + drain observations (log becomes rich; no agent change) | Low–medium (26 effect variants to describe) |
| B | Per-side agents in `playthrough` (+ `agent.rs`, per-side caches) | Low (dispatch + cache plumbing) |
| C | `observer.rs` + `Completion` trait + crib sheet + report writer | Medium (chunking + prompt engineering) |
| D | `main.rs` CLI + tests | Low |

Order deliberately puts the log first: the observer's prompt quality is bounded
by log quality, and Phase A is independently testable.

## 11. Open decisions

- **Crib sheet scope.** One curated file vs feeding the full manual in chunks
  to the observer. Proposal: crib sheet now, manual-as-context later if
  findings show it's needed.
- **Severity → CI.** Keep observer findings advisory, or fail CI on
  `Critical`/`Error` findings? Proposal: advisory initially; add the gate once
  the false-positive rate is known.
- **Observer per turn vs per N events.** Proposal: per-turn (natural
  boundaries, matches `TurnComplete`). Revisit if a turn is too large.

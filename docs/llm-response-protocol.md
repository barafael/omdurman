# LLM response protocol

The structured-reply contract between this codebase and the LLM. It defines
*one* transport (OpenAI-compatible chat completions), *two* reply shapes (the
per-turn planner and the offline observer), and the conventions both
consumers share.

Consumers:

| Consumer | Rust type | Caller |
|---|---|---|
| Per-turn strategy advisor | `PlanResponse` | `omdurman_bot::llm::advise_turn` |
| Offline rules auditor | `ReviewResponse` | `omdurman_bot::observer::review` |

Flavour-text calls (telegrams, newspapers in `omdurman-app`) are **not** part of
this protocol: they request plain prose and never set `response_format`.

---

## 1. Transport

`request_completion` (`omdurman-net/src/llm.rs`) issues a single
chat-completions request:

- `POST {base_url}/chat/completions` with `Authorization: Bearer {api_key}`
  (90 s request timeout, 15 s connect timeout).
- Request body:

  ```json
  {
    "model": "gpt-4o-mini",
    "messages": [
      {"role": "system", "content": "<system prompt>"},
      {"role": "user",   "content": "<user prompt>"}
    ],
    "max_tokens": 6000,
    "temperature": 0.7,
    "response_format": {"type": "json_object"}
  }
  ```

- `max_tokens` is chosen per call: **6000** for the planner (2000 truncated
  long replies mid-string), **2000** for each observer chunk. `temperature`
  is fixed at 0.7.
- `response_format` is omitted unless the caller opts in via
  `LlmConfig::with_json_object()`. Both protocol consumers do; prose callers
  do not.
- The reply is read from `choices[0].message.content` (a missing content is
  an empty string). A non-2xx status is `LlmError::Api`.
- The transport is native-only. On wasm, `request_completion` is a stub that
  returns `NoApiKey`. The app calls `request_completion_blocking` (the same
  request on a dedicated current-thread runtime); the bot calls the async
  form.

### Configuration

`LlmConfig` is built from environment variables (`LlmConfig::default`):

| Env var | Default |
|---|---|
| `LLM_API_KEY` (falls back to `OPENAI_API_KEY`) | none |
| `LLM_BASE_URL` | `https://api.openai.com/v1` |
| `LLM_MODEL` | `gpt-4o-mini` |

Empty values count as unset. On wasm the key is always withheld. With no
key, neither consumer touches the network: the planner returns no plan and
the observer returns an empty report with a "review skipped" summary.

---

## 2. Shared conventions

1. **One JSON object, nothing else.** The reply must be a single top-level
   object with no surrounding prose and no code fence. The system prompt says
   so; `response_format: json_object` asks the endpoint to honour it; and
   `strip_json_fence` (`omdurman_bot::llm`, crate-private) strips one stray
   ` ```json ` / ` ``` ` wrapper as a last resort.
2. **Every field defaults.** All `PlanResponse` and `ReviewResponse` fields
   are `#[serde(default)]`, and a reply that fails to parse becomes the
   all-default value (with a warning on stderr). What the consumer does with
   defaults differs (§3.3, §4).
3. **Cite the rulebook.** Reasoning and findings carry `§N` citations
   (`N` without the `§` in structured fields). The observer is told to cite
   only sections that appear in its crib sheet and never to invent numbers.
4. **The cache is the model's only memory.** The `cache` string is threaded
   turn-to-turn (planner, one cache per side) or chunk-to-chunk (observer)
   and hard-capped at `MAX_CACHE_BYTES` (512 000 bytes) on a char boundary by
   `LlmCache::truncate_to_cap`, which appends a
   `…[cache truncated at 500 KB]` marker.

---

## 3. Planner reply — `PlanResponse`

### 3.1 When it is asked

For an `AgentStrategy::LlmAdvised` side, once per side-turn: when that
side's Movement phase begins (a new turn or a side change). It is not
re-queried when the plan runs out mid-turn.

The system prompt names the side and the reply shape, followed by
`Your brief: <brief>` when the side has one (the doctrine corpus or a
scripted brief). The user prompt is:

```
=== NOTES FROM PREVIOUS TURNS ===      (only if the cache is non-empty)
<cache>
=== END NOTES ===

Scenario: <scenario>
Turn: <n>  Phase: <phase>  Player: <side>

Friendly units:
  <identity> at (q,r)
Enemy units:
  <identity> at (q,r)

Legal actions (<N> total):
  [0] <GameEffect, Debug-formatted>
  [1] …
```

### 3.2 Reply

```json
{
  "cache": "updated notes for next turn — what the model wants to remember",
  "plan": [3, 7, 12],
  "reasoning": [
    "- 3: fire at (q,r) — §6.24 direct fire bonus applies",
    "- 7: move Mulazmin toward Palace — §9.322 entry edge"
  ]
}
```

| Field | Type | Semantics |
|---|---|---|
| `cache` | string | The side's new scratchpad. Replaces the previous cache (then capped). |
| `plan` | array of int | Indices into the legal-action list of the prompt, in the order to play them. |
| `reasoning` | array of string | Free-form notes, one per planned action by convention. |

Rust type: `omdurman_bot::llm::PlanResponse`.

### 3.3 What the driver does with it

- **Cache:** `advise_turn` assigns `cache` to the side's `LlmCache`
  **unconditionally** whenever a reply arrives. A reply without a `cache`
  field, or one that fails to parse, therefore empties the side's cache.
  A transport error or a missing key leaves the cache untouched. (Whether
  the planner should keep the previous cache instead is tracked in
  `docs/open-issues.md`.)
- **Plan:** the indices are resolved against the candidate list the prompt
  showed, turning them into concrete actions. Out-of-range indices and
  `AdvancePhase` entries are dropped; the driver ends phases itself. The
  candidate list is re-enumerated after every applied action, so each later
  pick takes the first plan entry that matches a current candidate *by
  intent* (`same_intent`, ignoring pre-rolled dice). Entries that no longer
  match are dropped with a `[note, T<turn>] plan entry no longer legal …`
  log line.
- **Fallback:** when the plan is empty or exhausted, or no entry matches,
  the pick falls back to the aggressive heuristic
  (`omdurman_bot::aggressive::pick`), not to a random move. This also covers
  a missing key, an API error and an unparsable reply.
- **Reasoning:** each string is logged as a
  `[reasoning, <side> T<turn>] <text>` line and kept as an `LlmAnnotation`.

---

## 4. Observer reply — `ReviewResponse`

Sent once per turn-sized log chunk (see Chunking below).

```json
{
  "cache": "<working notes / open threads>",
  "findings": [
    {"severity": "warning", "seq": 12, "section": "5.24",
     "explanation": "gunboat may have exceeded upstream allowance"},
    {"severity": "error", "seq": 34, "section": "6.24",
     "explanation": "fire modifier not applied to CRT roll"}
  ],
  "summary": "<one-paragraph closing assessment>"
}
```

| Field | Type | Semantics |
|---|---|---|
| `cache` | string | Running notes carried to the next chunk. Empty or missing → the previous cache is kept. |
| `findings` | array of finding objects | Rule violations / suspicions. Omit or empty for a clean chunk. |
| `summary` | string | Closing assessment; the last non-empty one wins. |

### Finding object

| Field | Type | Semantics |
|---|---|---|
| `severity` | string | One of `critical` \| `error` \| `warning` \| `info` (case-insensitive). |
| `seq` | int | Sequence number of the log event the finding refers to. |
| `section` | string, optional | Rulebook section number, **without** the `§` prefix. |
| `explanation` | string | What contradicts the rulebook (defaults to empty). |

Malformed **individual** findings are dropped while well-formed siblings
survive: `ReviewResponse` keeps `findings` as raw JSON values and converts
each one separately (`ReviewResponse::into_parts`). A chunk whose request
fails, or whose reply does not parse, keeps the previous cache and
contributes nothing.

Findings are de-duplicated across chunks on `(severity, seq, section)`: the
model may re-flag an issue it carried in `cache`, and the report lists it
once.

### Chunking

A full game is too large for one prompt, so the observer feeds the log
**turn by turn**. The log is split at `=== Turn N complete ===` markers
(`chunk_log`). Each chunk's user prompt carries:

```
=== REVIEW CHUNK {i}/{total} ===
=== GAME HEADER ===            (every chunk; the log header block)
=== RULES CRIB SHEET ===       (first chunk only)
=== RUNNING CONTEXT FROM PREVIOUS CHUNKS ===   (the cache, or "(none)")
=== LOG TURN ===
```

The system prompt also explains the log line format (event, observation,
`[reasoning, …]` and `[note, …]` lines) and a few rules of thumb (advance
after combat only into an engine-marked vacated hex; one fire per unit per
subphase, Maxims again in the second subphase).

The result is an `ObserverReport`: the findings, the summary,
`turns_audited` and `events_audited`. Findings are **advisory**: the
engine's validation and the deterministic invariants and audits remain the
only gate.

Rust types: `omdurman_bot::observer::ReviewResponse` (private);
`omdurman_bot::observer::Finding` (public, serde round-trip).

---

## 5. Calling the transport

Both consumers construct `LlmConfig` from the environment, then opt in to
JSON:

```rust
let config = LlmConfig::default();
let json_config = config.clone().with_json_object();

// Planner (native async), inside advise_turn:
let response = request_completion(&json_config, &system, &user, 6000).await?;
let plan: PlanResponse = serde_json::from_str(strip_json_fence(&response))?;

// Observer goes through the `Completion` trait, so tests inject canned
// responses; `ReqwestCompletion` wraps `request_completion`. `review`
// applies `with_json_object()` itself and asks for 2000 tokens per chunk.
let report = review(log, &config, &ReqwestCompletion, crib).await;
```

`Completion` (`omdurman_bot::observer`) is the seam that keeps the observer
testable without a network call; the transport itself
(`omdurman_net::llm::request_completion`) is what protocol consumers share
with the app's prose flavour text.

---

## 6. Source of truth

- Transport + `LlmConfig` + `ResponseFormat`: `omdurman-net/src/llm.rs`
- `PlanResponse` + `strip_json_fence` + `advise_turn`: `omdurman-bot/src/llm.rs`
- Plan resolution, intent matching and fallback: `omdurman-bot/src/playthrough.rs`
- `ReviewResponse` + `Finding` + chunking: `omdurman-bot/src/observer.rs`
- Protocol tests: `omdurman-bot/tests/observer.rs`,
  `omdurman-bot/src/observer.rs` (unit tests), `omdurman-bot/tests/head_to_head.rs`

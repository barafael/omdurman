# LLM response protocol

The structured-reply contract between this codebase and the LLM. It defines
*one* transport (OpenAI-compatible chat completions), *two* reply shapes (the
per-turn planner and the offline observer), and the shared conventions both
consumers live by.

Protocol versions:

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

- `POST {base_url}/chat/completions` with `Authorization: Bearer {api_key}`.
- Request body (native build):

  ```json
  {
    "model": "gpt-4o-mini",
    "messages": [
      {"role": "system", "content": "<system prompt>"},
      {"role": "user",   "content": "<user prompt>"}
    ],
    "max_tokens": 2000,
    "temperature": 0.7,
    "response_format": {"type": "json_object"}
  }
  ```

- `response_format` is omitted unless the caller opts in via
  `LlmConfig::with_json_object()` — both protocol consumers do; prose callers
  do not.
- Replies are expected as `choices[0].message.content`, a single string.
- Native-only transport; the wasm stub always returns `NoApiKey`.

### Configuration

`LlmConfig` is built from environment variables (`LlmConfig::default`):

| Env var | Default |
|---|---|
| `LLM_API_KEY` (falls back to `OPENAI_API_KEY`) | none |
| `LLM_BASE_URL` | `https://api.openai.com/v1` |
| `LLM_MODEL` | `gpt-4o-mini` |

With no key, every consumer degrades deterministically (empty plan, skipped
review) and never touches the network.

---

## 2. Shared conventions

1. **One JSON object, nothing else.** The reply must be a single top-level
   object with no surrounding prose and no code fence. The system prompt says
   so; `response_format: json_object` makes the endpoint honour it; and
   `strip_json_fence` (`omdurman_bot::llm`) tolerates the occasional stray
   ` ```json ` wrapper as a last resort.
2. **Degrade, don't crash.** Every field in every schema is
   `#[serde(default)]`. A missing or malformed field (or a whole malformed
   reply) yields the default value and the caller falls back:
   - empty `plan` → random move for that turn;
   - empty `cache` → previous scratchpad is kept;
   - empty `summary` → previous summary is kept.
3. **Cite the rulebook.** Reasoning and findings carry `§N` citations
   (`N` without the `§` in structured fields). The observer is told to cite
   only sections that exist in its crib sheet and never invent numbers.
4. **The cache is the model's only memory.** The `cache` string is threaded
   turn-to-turn / chunk-to-chunk and hard-capped at 500 KB on a char boundary
   (`LlmCache::truncate_to_cap`), appending a `…[cache truncated at 500 KB]`
   marker.

---

## 3. Planner reply — `PlanResponse`

Sent in `AgentStrategy::LlmAdvised` mode, once per player-turn.

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
| `cache` | string | Updated scratchpad; replaces the previous cache (then capped). |
| `plan` | array of int | Indices into the enumerated legal-action list, applied in order. |
| `reasoning` | array of string | One reason per planned action; logged as `LlmAnnotation`s. |

The user prompt lists the current state and every legal action by index; the
model must pick from that list only. An out-of-range index is the caller's
problem to filter — the schema itself carries no bounds.

Rust type: `omdurman_bot::llm::PlanResponse`
(`#[serde(default)]` on all three fields).

---

## 4. Observer reply — `ReviewResponse`

Sent once per turn-sized log chunk (see below for chunking).

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
| `cache` | string | Running notes carried between chunks. |
| `findings` | array of finding objects | Rule violations / suspicions. Omit or empty for a clean chunk. |
| `summary` | string | Closing assessment; the last non-empty one wins. |

### Finding object

| Field | Type | Semantics |
|---|---|---|
| `severity` | string | One of `critical` \| `error` \| `warning` \| `info` (case-insensitive). |
| `seq` | int | Sequence number of the log event the finding refers to. |
| `section` | string, optional | Rulebook section number, **without** the `§` prefix. |
| `explanation` | string | What contradicts the rulebook. |

Malformed **individual** findings are dropped while well-formed siblings
survive: `ReviewResponse` keeps `findings` as raw JSON values and converts
each one separately (`ReviewResponse::into_parts`). A whole malformed chunk
keeps the previous cache and contributes nothing.

Findings are de-duplicated across chunks on `(severity, seq, section)` — the
model may re-flag the same issue after carrying it in `cache`; the report
lists it once.

### Chunking

A full game is too large for one prompt, so the observer feeds the log
**turn by turn**. The log is split at `=== Turn N complete ===` markers
(`chunk_log`). Each chunk's user prompt carries:

```
=== REVIEW CHUNK {i}/{total} ===
=== GAME HEADER ===            (every chunk)
=== RULES CRIB SHEET ===       (first chunk only)
=== RUNNING CONTEXT FROM PREVIOUS CHUNKS ===   (the cache, or "(none)")
=== LOG TURN ===
```

The result is an `ObserverReport`: findings plus a summary, `turns_audited`,
and `events_audited` counts. Findings are **advisory** — deterministic
invariants and the engine's `can_*` validation remain the only gate.

Rust type: `omdurman_bot::observer::ReviewResponse` (private);
`omdurman_bot::observer::Finding` (public, serde round-trip).

---

## 5. Calling the transport

Both consumers construct `LlmConfig` from env, then opt in to JSON:

```rust
let config = LlmConfig::default();
let json_config = config.clone().with_json_object();

// Planner (native async):
let response = request_completion(&json_config, &system, &user, 2000).await?;
let plan: PlanResponse = serde_json::from_str(strip_json_fence(&response))?;

// Observer goes through the `Completion` trait, so tests inject canned
// responses; `ReqwestCompletion` wraps `request_completion`.
let report = review(log, &config, &ReqwestCompletion, crib).await;
```

`Completion` (`omdurman_bot::observer`) is the seam that keeps the observer
testable without a network call; the transport itself
(`omdurman_net::llm::request_completion`) is what protocol consumers share
with the app's prose flavour text.

---

## 6. Source of truth

- Transport + `LlmConfig` + `ResponseFormat`: `omdurman-net/src/llm.rs`
- Shared parser + `PlanResponse` + `strip_json_fence`: `omdurman-bot/src/llm.rs`
- `ReviewResponse` + `Finding` + chunking: `omdurman-bot/src/observer.rs`
- Protocol tests: `omdurman-bot/tests/observer.rs`,
  `omdurman-bot/src/observer.rs` (unit tests), `omdurman-bot/tests/head_to_head.rs`

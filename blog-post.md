# LLM-assisted Implementation Approaches for a Board Game

Notes on `Omdurman`, a Bevy implementation of *Remember Gordon! — The Battle of Omdurman* (1982).

This game is a turn-based strategy game originally made in paper. Two very different factions battle it out over the Sudanese desert cities, and the Nile.
The game is defined by its manual and the maps, unit counters, and charts that came with it.

The implementation of the game is a webrtc-based multiplayer bevy+egui application which runs natively or in the browser.

The game implementation relies on a deterministic rules engine (randomness for dice via initial seed). It enabled interesting implementation approaches.
A game replay is a recorded sequence of `GameEffect`s, each applied to the game state in order.
This has interesting consequences:

* A spectator can scrub through the game like a piece of music.
* A new-joiner gets the full history (although this is not needed for a deterministic game, it is valuable for cheat detection).
* A game replay can be inspected later for rule deviations _which the engine permitted_.

## 1. Machine-checked Traceability

Rulebook §-sections are mapped to implementation and test sites bijectively, enforced at four independent levels:

- **Compile time:** a dedicated test crate references every _cited_ symbol as `use path as _;` (renaming breaks build).
- **Test time:** bijectivity + citation coverage checks; `#[rulebook("§6.3")]` macro makes tests self-annotate their covered sections into a jsonl the checker reads.
- **Live editor:** an LSP server gives diagnostics/hover/definitions over `.rs`, `.toml`, and the OCR'd manual — same `checks` as the test, so editor and CI always agree.
- **Generated artifact:** a typst pipeline regenerates `traceability.pdf`; `fix_lines` re-syncs stale line numbers.

## 2. LLM confined to render/observe

The LLM is a *derived presentation layer*, never a participant.

- The engine accumulates structured `TurnSummary`s, which are emitted as `GameEvent::TurnComplete` — recorded and replayed like any other event.
- Telegrams and newspapers for player delectation are generated from that deterministic summary, not just from LLM reasoning; the newspaper template is selected from the typed game result.
- The log drives the GUI, spectator replay, bot, telegram, and newspaper.

## 3. Three-agent rule verification

Two independent per-faction agents play head-to-head; a third audits.
The aim of the playing agents is not just to win, but to explore the whole game.

- **Layers:** (1) engine `can_*` predicates reject illegal moves, (2) hard invariants + proptest assert legal states after every effect, (3) an LLM observer reads the whole log against the rulebook for misapplications the invariants can't express (wrong Combat-Results-Table row, missed modifier, phase-order slip).
- **Advisory LLM, deterministic arbiter.** Automated gating never depends on the LLM; findings are surfaced for humans. Probabilistic tools add breadth, never authority.

The point of this is _not_ to train agents; it is to audit the game codebase and to find rule loopholes.

### Agent memory 

- **Scratchpad memory.** Each LLM side owns a 500 KB cache. This serves as inter-turn memory.
- **JSON response protocol.** Every LLM reply is a single JSON object — typed structs (`PlanResponse`, `ReviewResponse`) with `#[serde(default)]` degrade on malformed output, and `response_format: json_object` at the transport constrains the provider. One shared fence-stripper tolerates stray code fences; no ad-hoc line parser.
- **Chunked audit.** A game is too big for one prompt, so the observer reviews turn-sized chunks, carrying running notes between them and deduping findings across chunks.
- **Self-sufficient log.** Effects are rendered as prose designed for LLM consumption — dice spelled out, CRT rows, MP arithmetic, engine-authoritative § citations — so the auditor can re-derive rule outcomes without engine access.

## 4. Grounded reference corpus

- The OCR'd manual is the canonical asset; printed tables are transcribed as one module each.
- A curated crib sheet and per-side strategy doctrine files are checked-in; every § citation in them must resolve against the traceability matrix — the LLM's reference corpus is itself versioned and machine-verified.
- Observers are told to cite only crib-sheet sections and never invent numbers.

## 5. Spec as executable vignettes

- 24 "tactics scripts" — hand-built states plus ordered `Legal` / `Illegal` / `Assert` steps with pre-rolled dice — replay deterministically as both a regression suite and the spec the move generator is validated against.

## 6. Testability & offline operation

- The LLM transport is behind a mockable `Completion` trait; the entire observer pipeline runs on canned responses.
- Env-driven config (`LLM_API_KEY` / `LLM_BASE_URL` / `LLM_MODEL`) shared across app and bot; no-key runs skip cleanly. Every LLM path has a deterministic fallback — the whole stack runs with zero API access.

---

*The lesson: put agents on a deterministic, replayable substrate; confine them to render/observe roles; let them challenge the code at breadth while deterministic tools hold the gate.*

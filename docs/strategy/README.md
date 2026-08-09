# Strategy Doctrine Corpus — Remember Gordon!

Checked-in strategic/tactical doctrine files for the `omdurman-bot` playing
agents. Each file holds standalone advice for one side (or the shared game
system). Every piece of advice cites the rulebook section(s) it relies on, and
every citation resolves to a `[[mapping]]` in `docs/traceability.toml` — so the
corpus is machine-checked against the manual, not hand-waved.

## Files

| File | Scope |
|---|---|
| `common_doctrine.md` | System-wide doctrine: fire/melee/advance/ZOC/stacking trade-offs, relevant to both sides |
| `anglo_egyptian_doctrine.md` | Anglo-Egyptian (Kitchener) doctrine |
| `dervish_doctrine.md` | Dervish (Khalifa) doctrine |
| `fall_of_khartoum_doctrine.md` | Fall-of-Khartoum scenario deltas, both sides |

## Format

Each entry is one item: a headline, the advice (with concrete numbers where the
manual gives them), and a trailing `— §N.NN, §N.NN` citation list. The format is
stable so a test can extract and validate every `§N.NN` against the traceability
matrix.

## How it is used

`omdurman-bot` loads the side-appropriate file(s) as the LLM advisor's `brief`
for that side (`AgentStrategy::LlmAdvised { brief, .. }`), prepended to the
system prompt. Random agents ignore the corpus.

## Validation

`omdurman-bot/tests/strategy_corpus.rs` asserts:

1. Every `§N.NN` cited in the corpus exists in `docs/traceability.toml`.
2. Each file loads non-empty through the same loader the agents use.

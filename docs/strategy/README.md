# Strategy Doctrine Corpus — Remember Gordon!

Checked-in strategic/tactical doctrine for the `omdurman-bot` LLM agents.
Each file holds standalone advice for one side (or the shared game system),
and every piece of advice cites the rulebook section(s) it relies on.

## Files

| File | Scope |
|---|---|
| `common_doctrine.md` | System-wide doctrine: fire/melee/advance/ZOC/stacking/terrain trade-offs, relevant to both sides |
| `anglo_egyptian_doctrine.md` | Anglo-Egyptian (Kitchener) doctrine |
| `dervish_doctrine.md` | Dervish (Khalifa) doctrine |
| `fall_of_khartoum_doctrine.md` | Fall-of-Khartoum scenario deltas, both sides |

## Format

Each entry is one numbered item: a bold headline, the advice (with concrete
numbers where the manual gives them), and a trailing `— §N.NN, §N.NN`
citation list. Keep the `§N` form so the corpus test can extract every
citation.

## How it is used

`omdurman-bot/src/doctrine.rs` reads the files **at run time** with
`fs::read_to_string`, from `docs/strategy/` located via the crate's
`CARGO_MANIFEST_DIR` (a path baked in at compile time, so the binary expects
the source tree). They are not `include_str!`'d, so an edit takes effect on
the next run without a rebuild. A missing file is skipped.

- `doctrine_brief(player, scenario)` joins `common_doctrine.md`, the
  faction file and, in Fall of Khartoum, `fall_of_khartoum_doctrine.md`.
  It is the brief of an `AgentStrategy::LlmAdvised` side (the CLI's `llm`,
  `ae`, `dervish` presets and `run.json` `llm` sides), prepended to the
  advisor's system prompt.
- The scripted briefs in `doctrine.rs` (`storm_brief`, `fortress_brief`,
  `horde_brief`, `defender_brief`, `besieger_brief`) are the side's
  `doctrine_brief` plus appended override orders for the `storm`, `siege` and
  `laststand` presets. Their text lives in the Rust source, not here.
- Random, Aggressive and Commander agents do not read the corpus. The
  commanders' scoring (`omdurman-bot/src/commanders.rs`) was distilled from
  it, so keep the two consistent by hand.

## Validation

`omdurman-bot/tests/strategy_corpus.rs` checks the four corpus files (not
this README):

1. Every `§N` citation matches a `section` in `docs/traceability.toml`. The
   match is by prefix in either direction, so a mapped `§6.24` satisfies a
   cited `§6.2` and a mapped `§6` satisfies a cited `§6.13`. It proves that a
   cited section exists, not that the advice is right.
2. Every side/scenario brief loads non-empty through `doctrine_brief`.
3. The corpus is substantial (more than 10 000 characters).

The rules content itself is checked only by review against the manual
(`Boardgame - Remember_Gordon/Manual/RememberGordonManual.md`) and the
transcribed tables under `Boardgame - Remember_Gordon/tables/`.

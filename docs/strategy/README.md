# Strategy Doctrine Corpus — Remember Gordon!

Checked-in strategic/tactical doctrine for Remember Gordon!, written for
human readers and as the reference the bot's commanders were tuned against.
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

No code reads the corpus at run time. The commanders' scoring
(`omdurman-bot/src/commanders.rs`) was distilled from it, so keep the two
consistent by hand.

## Validation

`omdurman-bot/tests/strategy_corpus.rs` checks the four corpus files (not
this README):

1. Every `§N` citation matches a `section` in `docs/traceability.toml`. The
   match is by prefix in either direction, so a mapped `§6.24` satisfies a
   cited `§6.2` and a mapped `§6` satisfies a cited `§6.13`. It proves that a
   cited section exists, not that the advice is right.
2. The corpus files exist and are substantial (more than 10 000 characters
   in all).

The rules content itself is checked only by review against the manual
(`Boardgame - Remember_Gordon/Manual/RememberGordonManual.md`) and the
transcribed tables under `Boardgame - Remember_Gordon/tables/`.

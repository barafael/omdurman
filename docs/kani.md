# Kani in the Omdurman codebase: what the proofs taught us

*A field manual for the Rust model checker in this repo — how it is wired in,
what it is good for, and what it costs you.*

---

## Why a model checker in a boardgame port

This workspace re-implements a 1982 wargame whose rules are mostly *small,
total functions over closed domains*: a table lookup here, a modifier sum
there, a predicate over an enum with six variants. That is exactly the shape
of code [Kani](https://github.com/model-checking/kani) proves cheaply. A unit
test samples the domain; a Kani harness closes it. "The defence modifier is
never positive" is one `assert` over a symbolic `Terrain` — and it stays true
when someone adds a seventh terrain kind, because the `match` arms are
generated and the proof picks up the new variant automatically.

The proofs live in `#[cfg(kani)] mod verification` blocks next to the code
they constrain. Today: 93 harnesses across `omdurman-types` and
`omdurman-rules`, all verifying except one resource-blocked outlier (below).

## Running it

```sh
./scripts/kani.sh -p omdurman-types -p omdurman-rules     # the suite
KANI_JOBS=8 ./scripts/kani.sh -p omdurman-types -p omdurman-rules
./scripts/kani.sh -p omdurman-rules --harness verification::melee_factor_sum_is_total
```

The script bakes in the two non-negotiables:

- `-Z stubbing` — enables `#[kani::stub]`, the escape hatch for heavy call
  graphs (§ below).
- `--features kani` — compiles the engine's `debug!` call sites out. Tracing
  format machinery otherwise dominates the SAT instance.

`KANI_JOBS=N` verifies harnesses in parallel. Artifacts go to
`/tmp/kani-target` (`KANI_TARGET_DIR`), never the host `target/`.

CI does **not** run the suite: GitHub runners kept killing the job with
shutdown signals (exit 143). The suite is gated to `workflow_dispatch`; the
local run is the authoritative check. Budget minutes, not seconds, for the
full suite — and RAM, not CPU, is the binding constraint.

## The annotation contract

Proofs are traceability citizens. Every harness carries a `// §N` line above
`#[kani::proof]` — **not** `#[rulebook(...)]`, because the proof modules are
`cfg(kani)` on the lib, where dev-dependencies (and the proc-macro) do not
exist. The fully-qualified harness name must appear in the `proofs = [...]`
array of the matching `[[mapping]]` in `docs/traceability.toml`, and the
mapping is bijective in both directions: an annotated harness not listed in
the TOML fails the build, and so does a listed harness whose annotation went
missing. Scoping matters as much as existence: a proof that a modifier *clamps
to 1..=10* does not prove that the modifier *is +2*. Name harnesses after the
property, not the function.

## Lessons the proofs taught us

**Prove the data too.** The engine's rules tables are `static` consts
(`tables_data.rs`), transcribed from RON and parity-checked cell-by-cell.
That was a Kani decision first: a runtime RON parse is unmodelable (CBMC
unrolls the parser and UTF-8 validation on every proof), while a `static`
array is plain data a proof can close over the whole input domain. The
Terrain Effects Chart proofs (`terrain_chart.rs`) hold the movement column to
1..=3 and the defence column to −3..0 for *every* terrain kind, road state,
and Nile flow — bounds the unit tests only sample.

**A proof is exactly as true as its encoded constants.** A harness named
`movement_column_matches_the_printed_chart` proves the column matches *the
constants encoded in the harness*. If the constants are a mis-transcription
of the paper chart, the proof will happily verify the wrong table forever.
Proofs close domains; they cannot close the gap between the model and the
world. That gap belongs to the transcription tests — and to whoever reads the
scanned chart.

**Stub the cascade, not the rule.** Some properties sit behind heavy-but-
property-neutral machinery: `advance_phase_is_atomic` does not care how melee
resolves, so it stubs `apply_melee_combat` with `Ok(())`. The house rules for
stubs: the stub must be *extensionally exact for every input the harness can
reach* (not merely convenient), the reason goes in the doc comment, and the
property under proof must not mention the stubbed behaviour. Precedents live
in `effects.rs`; the `score_elimination` harness stubs `BoardInfo::bank_of`
with `None` precisely because its board is empty, on which the real method
provably returns `None` everywhere.

**Keep proof states minimal.** `GameState::new(Scenario::Campaign)` symexes
the entire campaign order of battle — a roster no victory-ledger property
needs. Hence `GameState::kani_minimal()`: a `cfg(kani)`-gated, field-complete
literal with empty ledgers. It is deliberately *not* derived `Default` (a
default game state is a production footgun) and deliberately field-complete
(adding a `GameState` field breaks the proof build loudly instead of drifting
silently). Build the smallest state the property can be stated over.

**Weak proofs still pass.** A harness asserting only `result.is_ok()` proves
almost nothing but is invisible in the suite — the same trap as a vacuous
unit test. When auditing, ask of every harness: which *clause* of the rule
does this property pin, and would it fail on a plausible wrong
implementation? Harnesses that only prove totality or clamping are support
lemmas; cite them as such.

## When a proof OOMs

OOM is a resource report, not a verdict. Diagnosis playbook, learned the hard
way on `score_elimination_records_exactly_what_it_scores`:

1. **Watch the pipeline.** Run under a loop that samples `ps` (or read the
   log): our case showed ~125 s symex, 1.1M assignments, 42k VCCs after
   slicing, then death in the propositional reduction at ~13.5 GB RSS. Peak
   RSS just above the machine is a different problem than a runaway symex.
2. **Bisect with scratch harnesses.** Copy the harness body under an
   unannotated name (`z_scratch_*` — no `// §N`, so the traceability checker
   ignores it), and strip pieces: state construction only; property function
   only; concrete inputs instead of `kani::any()`. Our outlier OOM'd *fully
   concrete* at unwind 2 — proof the cost was inherent call-graph size, not
   the roster, the solver inputs, or loop unwinding.
3. **Shrink what you can, document what you can't.** Roster-free states and
   exact stubs shrank our outlier but could not get it under ~13 GB. That is
   a legitimate end state: keep the improvements, record the memory floor in
   the harness doc comment, and let the harness run on a machine that fits.
   `--cbmc-args --slice-formula` (needs `-Z unstable-options`) is worth one
   try; external SMT solvers (`--smtlib-solver`) would trade time for memory
   but none are installed here.

Never tune unwind bounds below what the Vec pushes and scans actually need —
an unwind failure is a proof you no longer have.

## Checklist: adding a proof

1. State the property as a biconditional or an exact bound over the whole
   domain — not a sample.
2. `// §N` above `#[kani::proof]`, matching an `implemented` mapping's
   section; add the fully-qualified name to that mapping's `proofs` array.
3. Build the minimal state (`kani_minimal()`, or a smaller local helper);
   stub heavy cascades only with an exactness argument in the doc comment.
4. Run the harness alone, then the suite; note the runtime if notable.
5. Regenerate the traceability artifacts (`fix_lines`, then the typst tool)
   if you moved cited code — the PDF renders proofs in blue above the tests.

The suite is the proof of the proofs: run `./scripts/kani.sh -p
omdurman-types -p omdurman-rules` before you trust the word "verified".

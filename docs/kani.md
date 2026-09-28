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
they constrain, plus the crate-level `omdurman-rules/src/verification.rs`.
Today the everyday suite runs 90 harnesses (23 in `omdurman-types`, 67 in
`omdurman-rules`, counting the `prove_value_enum!` template in
`verification.rs` as the five harnesses it expands to), all of which verify
on a big machine. The four in `quantifier_experiment.rs` sit behind the
`kani-quantifiers` feature that the script never enables, and the expensive
tier (below) adds 51 more behind `kani-expensive`.

## Running it

```sh
./scripts/kani.sh -p omdurman-types -p omdurman-rules     # the suite
KANI_JOBS=8 ./scripts/kani.sh -p omdurman-types -p omdurman-rules
./scripts/kani.sh -p omdurman-rules --harness verification::die_roll_apply_modifier_is_total
```

The script bakes in the two non-negotiables:

- `-Z stubbing` — enables `#[kani::stub]`, the escape hatch for heavy call
  graphs (§ below).
- `--features kani` — compiles the engine's `debug!` call sites out. Tracing
  format machinery otherwise dominates the SAT instance.

`KANI_JOBS=N` verifies harnesses in parallel. Artifacts go to `target/kani`
(`KANI_TARGET_DIR`; `/tmp/kani-target` inside WSL), apart from the host
build. Not `/tmp` on Linux: where it is a tmpfs, the multi-GB proof build sits
in the RAM the solver needs.

CI does **not** run the suite: GitHub runners kept killing the job with
shutdown signals (exit 143). The suite is gated to `workflow_dispatch`; the
local run is the authoritative check. Budget minutes, not seconds, for the
full suite — and RAM, not CPU, is the binding constraint.

## The annotation contract

Proofs are traceability citizens. A harness that pins a rule clause carries a
`// §N` line above `#[kani::proof]` — **not** `#[rulebook(...)]`, because the
proof modules are `cfg(kani)` on the lib, where dev-dependencies (and the
proc-macro) do not exist. The fully-qualified harness name must appear in the
`proofs = [...]` array of the matching `[[mapping]]` in
`docs/traceability.toml`, and the mapping is bijective in both directions: an
annotated harness not listed in the TOML fails the build, and so does a listed
harness whose annotation went missing. The gate only sees annotated harnesses:
22 suite harnesses carry no `// §N` and are not tracked (support lemmas such
as `distance_is_symmetric`, `game_over_is_absorbing`, `sink_chain_is_atomic`;
see [open-issues.md](open-issues.md)). The scanner walks upward from the
attribute and stops at the first blank line, so a `§` comment separated from
`#[kani::proof]` by a blank line does not count. Scoping matters as much as
existence: a proof that a modifier *clamps to 1..=10* does not prove that the
modifier *is +2*. Name harnesses after the property, not the function.

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
property-neutral machinery: `resolve_melee_is_atomic` does not care how melee
resolves, so it stubs `resolve_melee_combat` with a no-op;
`advance_phase_is_atomic` stubs the `end_player_turn` cascade, and
`setup_ready_latches_are_monotonic` stubs `advance_phase`. The house rules for
stubs: the stub must be *extensionally exact for every input the harness can
reach* (not merely convenient), the reason goes in the doc comment, and the
property under proof must not mention the stubbed behaviour. Those three
precedents live in `effects.rs`; in `effects/expensive.rs`, the harnesses
stub `BoardInfo::bank_of` with `None` precisely because their board is
empty, on which the real method provably returns `None` everywhere.

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

## Making a harness cheaper without weakening it

Measured on this suite (September 2026); each is safe because it changes what
the solver has to *carry*, never what it has to *prove*:

- **Discard the state instead of dropping it.** A harness that lets its
  `GameState` fall out of scope makes CBMC walk the drop glue of every ledger —
  `Observation` and `TurnEventRecord` carry `Vec<String>`s, each unrolled to
  the unwind bound. End with `state.kani_discard()` (a `mem::forget`): every
  assertion has already run, the engine has no `unsafe` (`forbid`) and no
  `Drop` impl of its own, and Kani does not check leaks. `sink_chain_is_atomic`
  went from 1.31M steps / 187 s to 233k / 24 s.
- **Use the tightest unwind bound that verifies.** Unwinding assertions are on,
  so a bound that is too low fails loudly ("unwinding assertion loop N") — it
  cannot silently prove less. A bound that verifies is a complete proof. Most
  loops in the state harnesses scan two or three units, yet the whole set used
  `unwind(14)`, and every loop whose trip count the solver cannot fold (a
  slice over a merged heap pointer) was unrolled 14 times.
  `river_mine_sinking_removes_the_gunboat` went from unfinished after 20 min to
  130 s at `unwind(4)`.
- **No recursion on proof paths.** `eliminate_unit` used to recurse for a sunk
  gunboat's passenger; CBMC unrolled the recursion to the bound with the whole
  scoring path at every level. The cascade is one level deep by the rules (a
  passenger is never a gunboat), so it is now a loop over a helper.
- **Run heavy harnesses alone.** Parallel jobs share RAM; three of our "OOMs"
  were two big harnesses side by side. Size `KANI_JOBS` by memory, not cores.

What did *not* help: pre-reserving ledger capacity, `--arrays-uf-always`, and
splitting a symbolic `bool` into two concrete runs.

**Enumerate a tiny domain instead of proving it.** Two harnesses never
finished even on a 230 GB machine, and neither needed a model checker:
`score_elimination_records_exactly_what_it_scores` had one symbolic `bool`
(reading an `Observation` back out of the ledger is what exploded the
propositional reduction; its length checks alone verified in 5 s), and
`turn_labels_agree_with_the_rule_bearing_track` one symbolic `u8` (it formats
the label text, and Kani symexes all of `fmt`). Running every value of a
2- or 256-value domain in a `#[test]` proves the same property -- the same
panics, overflow checks on in test builds, no UB without `unsafe` -- in
milliseconds. Both are exhaustive tests now. Before writing a harness, count
the domain: if a loop can walk it, a test is the proof.

## The expensive tier

`omdurman-rules/src/effects/expensive.rs` and the B-tree proofs in
`omdurman-types/src/net_seq.rs` hold the proofs worth having but too big for
the everyday suite. Each state harness takes longer than 20 minutes on a
desktop; budget an hour or more and tens of GB per job.

```sh
KANI_EXPENSIVE=1 ./run-kani.sh                          # everyday suite + tier
./scripts/kani.sh -p omdurman-rules --features kani-expensive --harness expensive::
```

What they prove, for every input they can build:

- **Every effect kind** (one harness each): `apply_effect` never panics or
  overflows; a rejected effect leaves the state untouched (every rule field
  compared, the logs by length); an accepted one keeps the global invariants
  (legal stacks, unique ids, no eliminated unit back, no tracker naming a
  unit that left).
- **Movement legality**: an accepted move ends on `to`, steps hex by
  adjacent hex from where the unit stood, never passes a hex next to an
  enemy projecting a zone of control, and spends within its allowance --
  stated apart from the engine's planner.
- **Effect pairs** (melee then phase change, fire then phase change, melee
  declared then resolved, two moves): the invariants after every step.
- **Wire format**: every effect kind round-trips through postcard; decoding
  any 16 bytes never panics; an exhaustive `kind_of` match fails to compile
  when a new `GameEffect` variant lacks a wire proof.
- **Sequencing** (`net_seq`): identity dedup accepts each uid exactly once;
  the reorder buffer hands out exactly the contiguous run from the
  watermark, in order, the latest delivery winning.

The state is three distinct real counters from the palette
(`omdurman-rules/src/proof_palette.rs`, held to the roster by a unit test)
on a 5x5 window of the engine's empty, rule-neutral board, in every
scenario, phase, side and turn, with the per-kind extras (a declared melee,
a mine, the chain, a passenger...). On the empty board the accepted halves
of `ConstructZariba`, `ArtilleryBreachWall`, `Demolition` and
`FriendliesTransport` are unreachable (they need printed sides, walls, forts
or Nile hexes); their `cover!`s report it, and those kinds are proven for
the rejection half only.

**The same harnesses run under `cargo test`** as a randomized test: the
Kani API is swapped for a seeded RNG, a failed `assume` rejects the sample,
and half the samples are shaped like a client's inputs (the engine's own
builders) so the accepted half gets exercised too. 20 000 samples per
harness by default, `EXPENSIVE_SAMPLES=500000` for a deep run (35 s in
release; nothing found). It is the everyday guard; the proofs are the
exhaustive one.

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

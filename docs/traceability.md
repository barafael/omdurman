# Traceability in the Omdurman codebase: an index, and the evidence behind it

*How this repo pairs a 1982 rulebook with a Rust engine, what the checks
prove, and what they cannot.*

---

## What it is

`docs/traceability.toml` is a **rulebook index**: one `[[mapping]]` per
section of the manual, naming the code that implements it, the tests and
proofs that exercise it, and -- for every implemented section -- the manual's
own words for the rule the code enforces (`clause`) and the one test or proof
whose job that rule is (`witness`).

It is an index, not a verdict. An index can be perfectly consistent while the
code is wrong: in September 2026 an audit against the manual fixed fifteen
sections that were all marked `implemented`, with one to eleven tests each.
The structural checks keep the index from rotting; the **evidence** that the
code follows the rules comes from two further layers -- clauses with
witnesses, and the mutation gate -- and from reading the manual.

## The artifacts

| Artifact | Role |
|---|---|
| `Boardgame - Remember_Gordon/Manual/RememberGordonManual.md` | The requirement: OCR-corrected transcription of the printed rulebook. |
| `docs/traceability.toml` | The index: per section `status`, `[[mapping.impl]]` sites (`file`, `symbol`), `tests`, `proofs`, and for implemented sections `clause`, `witness` and an optional `approximation`. |
| `omdurman-rules/tests/traceability_paths.rs` | Compiler anchors: every cited symbol as a real `use`/item path -- a rename breaks the build. |
| `tools/traceability-lsp` | The checks (`checks.rs`), shared by `cargo test` and the editor LSP; the annotation scanner; the symbol resolver; the `mutation-gate` binary. |
| `tools/traceability-typst` | The report generator (`traceability.typ`, `data.json`, compiled with `typst`). Not committed: CI builds it, Pages publishes it at `https://barafael.github.io/omdurman/traceability.pdf`. |
| `traceability_macro` | `#[rulebook("§N")]`, the one annotation that counts. |

## Layer 1: the index (structural checks)

`cargo test -p omdurman-rules --test traceability`; the LSP shows the same
failures live.

1. **Impl sites are real.** `implemented` mappings list `[[impl]]` entries;
   other statuses must not. Each symbol must occur in the cited file's code
   (comments do not count). There are no line numbers: the resolver finds the
   symbol's definition (`fn`, `struct`, variant, field ...) when the report is
   built, so moving code never touches the index.
2. **Every citation is mapped**, both ways: every `§N` in Rust source names a
   mapped section, every mapping names a manual section, and every manual
   section has a mapping (containers are `descriptive`).
3. **Every cited symbol is compiler-anchored** in `traceability_paths.rs`,
   and every anchor is cited.
4. **Coverage is a hard gate.** Every implemented mapping lists at least one
   test that exists, is not `#[ignore]`d, and carries `#[rulebook("§N")]` for
   the section. `tests` and `proofs` are bijective with the attributes in
   source. Only the attribute counts: a `§` in a comment is a citation, never
   coverage, so rewording a doc comment cannot change coverage.

## Layer 2: clauses and witnesses (reviewable evidence)

Every implemented section states **which rule** it enforces and **which test**
proves it:

```toml
clause = "If Dervish leaders elect to stack, however, they may only stack with units of their command (i.e. color)."
witness = "omdurman-rules::src::effects::tests::dervish_leader_stacks_only_with_command_colour"
```

- The clause must be **verbatim** in the manual section's text (whitespace,
  emphasis and quote/dash styles normalised). A paraphrase that drifts from
  the rulebook fails. Pseudo-sections without manual text (the Combat Results
  Table, `§CRT`) describe their chart instead.
- The witness must be one of the section's listed tests or proofs.
- `approximation` says where the code deliberately departs from the clause
  and why. The report shows it in orange under the clause.

The report prints clause, witness and approximation above each section's code,
so a reviewer reads the rule next to the test that claims it. What the check
cannot do is read the witness: whether it asserts the clause is the reviewer's
call. Witnesses that only cover part of their clause are listed in
[open-issues.md](open-issues.md) as tests to write.

## Layer 3: the mutation gate (machine-checked evidence)

`cargo run -p traceability-lsp --bin mutation-gate`; the CI job
`mutation-gate` runs it on every push and pull request with `--in-diff`.

For every engine function an implemented section cites, cargo-mutants mutates
it (flips comparisons, replaces return values, deletes branches) and runs
**only the engine tests of the sections that cite it**. A mutant those tests
all miss means the cited code can change without any rulebook test noticing.
On a change, only mutants on changed lines are tested: new and edited rule
code must be pinned by its sections' tests, and old code is checked as it is
touched. A cited function with mutants but no engine test fails too.

Genuinely equivalent mutants (no rule-visible behaviour can tell them apart)
go in `.cargo/mutants.toml`, each with a comment saying why. A mutant no test
kills is not equivalent; write the test.

## Statuses and notes

- `implemented` -- code, tests, a clause and a witness.
- `descriptive` -- narrative or container heading; no impls, tests or clause.
- `implicit` -- a real convention that needs no enforcement; the `note` argues
  the reading.
- `out-of-scope` -- physical components, printed-table scans, setup fluff.

`note` carries other caveats; unlike `approximation` it is not in the report.

## Where the guarantees end

- **Witnesses are chosen, not checked.** The gate proves a section's tests
  pin its cited code; whether the witness asserts the clause is read by a
  person.
- **Uncited code is ungated.** The mutation gate only mutates functions the
  index cites; a rule implemented somewhere the index does not point is
  invisible to it.
- **Self-referential proofs** prove the constants they encode (see
  `docs/kani.md`); a mis-transcribed table stays green.
- **The manual is an OCR transcription.** "Verbatim" means verbatim against
  it, not against the printed page.

The remedy stays the audit: read the manual sentence, the cited code, the
witness body. The index makes that audit cheap; the clauses make its
conclusions visible; the mutation gate keeps them from silently eroding.

## Workflow

**Adding a rule:** cite the section in a comment at the implementation
(`(rulebook §6.11)`); add the `[[mapping]]` with `status = "implemented"`,
the impl sites, a verbatim `clause`, a `witness`, and at least one
`#[rulebook]`-annotated test; anchor the symbols in `traceability_paths.rs`.

**Renaming a symbol:** update the `symbol` in the TOML *and* the anchor in
`traceability_paths.rs`.

**Moving code:** nothing to do.

**Building the report locally:**

```sh
cargo run -p traceability-typst -- \
    docs/traceability.toml traceability.typ tools/traceability-typst/data.json
typst compile traceability.typ traceability.pdf
```

**Running the mutation gate locally** (copies the tree; `--in-place` is for
CI's throwaway checkout):

```sh
cargo run -p traceability-lsp --bin mutation-gate -- --section §5.51
git diff --no-ext-diff origin/main > my.diff
cargo run -p traceability-lsp --bin mutation-gate -- --in-diff my.diff
```

One caveat for editors: the LSP's diagnostics are only as good as its
workspace root -- unrooted, it reports every test as "not found". The cargo
test is the authority.

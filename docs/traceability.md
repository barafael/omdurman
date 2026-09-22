# Traceability in the Omdurman codebase: a matrix that cannot rot

*How this repo keeps a 1982 rulebook and a Rust engine provably paired — and
where the guarantees end.*

---

## The problem

This engine claims to be a faithful transcription of a printed rulebook.
Claims rot. Comments drift, tests get renamed, symbols move, sections get
reworded, and six months later nobody can say which line of code implements
which sentence of the manual — or, worse, everybody *believes* they can.
This repo's answer is a machine-checked matrix, enforced on every `cargo
test` run, that makes every direction of drift a build failure.

## The artifacts

| Artifact | Role |
|---|---|
| `Boardgame - Remember_Gordon/Manual/RememberGordonManual.md` | The requirement: OCR-corrected transcription of the printed rulebook. |
| `docs/traceability.toml` | The matrix: one `[[mapping]]` per manual section, with `[[mapping.impl]]` sites (file, line, symbol), `tests`, `proofs`, `status`, and optional `note`/`page`. |
| `omdurman-rules/tests/traceability_paths.rs` | Compiler anchors: every cited symbol referenced as a real `use`/item path — a rename breaks the build, not a string search. |
| `tools/traceability-lsp` | The check library (`checks.rs`) shared by the cargo test and the editor LSP, plus the annotation scanner. |
| `tools/traceability-typst` | `fix_lines` (line re-sync) and the PDF generator (`traceability.typ`, `data.json`, rendered with the `typst` CLI). |
| `traceability_macro` | The `#[rulebook("§N")]` proc-macro annotating tests. |

## The six structural checks

`cargo test -p omdurman-rules --test traceability` runs all of them; the LSP
runs the same code live. Failure strings are byte-identical in both.

1. **Impl sites are real.** `implemented` mappings must list `[[impl]]`
   entries; other statuses must not. Each symbol must appear in *code* (the
   comment part of a line is stripped) within 8 lines of the cited `line`.
2. **Every citation is mapped.** Every `§N` mention in Rust source has a
   `[[mapping]]` (a bare `§5` is covered by its `§5.x` children).
3. **Matrix → manual.** Every mapping section exists in the manual text.
4. **Every cited symbol is compiler-anchored** in `traceability_paths.rs`.
5. **Manual → matrix.** Every manual section has a mapping — container
   headings become `status = "descriptive"`.
6. **Anchors → matrix.** Every anchor in the paths file is cited by some
   `[[impl]]` (owning-type imports and a small `Some/Ok/Err/std` allowlist
   excepted). The two files cannot drift apart.

On top of the structure sits **coverage as a hard gate**: every
`implemented` mapping must list at least one test that (a) exists, (b) is
not `#[ignore]`d, and (c) carries the section's annotation. Tests and proofs
are bijective with their annotations *in both directions*: an annotated test
missing from the TOML fails, and so does a listed test whose annotation was
removed.

## Annotation mechanics

- Tests: `#[rulebook("§6.24", "§5.54")]` directly above `#[test]`. Multiple
  sections are fine; the test must then be listed under each.
- Kani proofs: `// §N` above `#[kani::proof]` — never `#[rulebook]`, because
  the `cfg(kani)` proof modules live on the lib where dev-dependencies (and
  the proc-macro) are unavailable. `///` doc comments count too: the scanner
  reads any `//`-prefixed line (including `///`) walking upward from the
  attribute until a blank line.
- `#[ignore]`d tests are not coverage and are excluded from the bijection.
- The scanner keys on trailing identifiers, so `UnitKind::may_melee_attack`
  matches on `may_melee_attack`; qualify with the owning type for readability.

## Statuses and the honesty valve

- `implemented` — code exists and is test-covered.
- `descriptive` — narrative or pure container heading; must not carry impls
  or tests (the gates enforce this).
- `implicit` — a real convention that needs no enforcement (e.g. "fire is
  voluntary" in a purely reactive engine); the `note` argues the reading.
- `out-of-scope` — physical components, printed-table scans, setup fluff.

The `note` field is the honesty valve. When only part of a manual section is
engine-enforced — the Zariba's no-fire/no-melee ban lives in the app UI, the
fort's −3 fire defence is not implemented at all — the note says so in the
mapping itself. A reader of the PDF sees the gap next to the green checkmark.
Undocumented partial implementation is the one failure mode this system does
not catch on its own.

## Where the guarantees end

The gates prove *structure*: that symbols exist, lines point somewhere real,
tests carry the right annotations, the mapping covers the manual. They cannot
prove *meaning*. Three failure classes survive every green check:

1. **Annotation-deep tests.** A test annotated §7.3 that asserts only
   `is_ok()` would pass on a sequential implementation of a simultaneous
   rule. The annotation is a claim; only the assertions honour it.
2. **Self-referential proofs.** A harness named
   `movement_column_matches_the_printed_chart` proves the chart matches *the
   constants encoded in the harness*. If those constants mis-transcribe the
   paper, the proof is green forever (see `docs/kani.md`).
3. **Data divergence.** The manual `.md` has no Terrain Effects Chart
   transcription, so the engine's chart is invisible to the matrix-to-manual
   check; a wrong table verified the wrong rules until someone read the scan.

The remedy is a four-way audit: for each mapping, read the manual sentence,
the cited code, the whole test body, and the proof property, and judge
whether each would fail on a plausible wrong implementation. The matrix makes
that audit cheap by handing you the shortlist; it cannot do the audit for
you.

## Workflow

**Adding a rule:** cite the section in a comment at the implementation
(`(rulebook §6.11)`), add the `[[mapping]]` with at least one
`#[rulebook]`-annotated test, anchor the symbol in `traceability_paths.rs`.
Container headings get `descriptive` and nothing else.

**Renaming a symbol:** update the `symbol` in the TOML *and* the anchor in
`traceability_paths.rs` — miss either and the build fails.

**Moving code:** run `cargo run -p traceability-typst --bin fix_lines` to
re-sync `line` fields (it scores definition sites, preferring the nearest
match to the old line).

**Regenerating the PDF artifacts** (commit with the TOML; a stale
`data.json` fails the freshness gate):

```sh
cargo run -p traceability-typst --bin traceability-typst -- \
    docs/traceability.toml traceability.typ tools/traceability-typst/data.json
typst compile traceability.typ traceability.pdf
```

One caveat for editors: the traceability LSP's diagnostics are only as good
as its workspace root — unrooted, it reports every test as "not found" and
every listed test as unlisted. The cargo test is the authority; do not chase
ghosts from a badly-rooted editor session.

## What the matrix buys

A rename is caught by the compiler, a moved symbol by the line check, a
deleted test by the coverage gate, an orphaned annotation by the bijection,
a dropped manual section by the reverse mapping. That is the whole surface of
*structural* rot, closed. The semantic layer — does the code actually *do*
what §N says, and does the test actually *pin* it — remains a human
discipline, exercised by audit and encoded honestly in `note`s. The matrix
cannot make the engine faithful; it makes unfaithfulness visible.

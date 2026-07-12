# Instructions

## Project

Rust/Bevy implementation of *Remember Gordon! — The Battle of Omdurman* (Phoenix Enterprises, 1982).
Runs natively and as a WASM web app (deployed to GitHub Pages via Trunk).
Networking is peer-to-peer via `bevy_matchbox` (WebRTC + a `wss://` signalling server).

## Common commands

Build / run native:

```shell
cargo run -p omdurman-app
```

Build / serve the WASM web build:

```shell
trunk serve
trunk build --release
```

Run a single rules test or a single test by name:

```shell
cargo test -p omdurman-rules
cargo test -p omdurman-rules $test_name_substring
```

The rulebook <-> code traceability check (see "Traceability" below):

```shell
cargo test -p omdurman-rules --test traceability
```

Regenerate `traceability.pdf` (requires the `typst` CLI):

```shell
cargo run -p traceability-typst
```

CI builds with `trunk build --release` for the `wasm32-unknown-unknown` target — keep that working
when changing dependencies.
The signalling server URL is bakeable via the `MATCHBOX_SERVER` env var at build time (see `omdurman-net/src/lib.rs`).

## Workspace layout

Five workspace crates plus one tool, all sharing `edition = "2024"`:

- **`omdurman-types`** — leaf crate, no Bevy. Pure serde types shared by everything else (`HexCoord`,
  `SectionName`, `AnnotationsFile`, hexside/Nile/overlay types, `Faction`, `Brigade`). Must stay
  dependency-light so both the rules engine and the net layer can depend on it.
- **`omdurman-rules`** — the rules engine. No Bevy. Defines `GameState`, `GameEffect`, and
  `apply_effect`: every legal mutation flows through `effects::apply_effect`. Effects carry
  pre-rolled dice so the same effect applied on every peer yields the same state. Submodules:
  `board`, `effects`, `combat_results_table`, `howitzer_scatter`, `los_table`, `range_effects`,
  `terrain_chart`, `turn_track`, `unit_id`. Most rulebook constants live as `value_enum!` enums in
  `lib.rs` so match arms are exhaustive at compile time.
- **`omdurman-hexmap`** — Bevy plugin (`HexMapPlugin`) for the hex grid: `GameMap`, `HexLayout`,
  `MapDims`, world-space conversion. `HexLayout` must be inserted manually with calibration data.
- **`omdurman-net`** — net glue. Defines `NetMsg`, `GameEvent`, `GameRecord` (event log),
  `InitialGameState`, and `room_id()`. `GameEvent` variants are the *only* messages recorded into
  the canonical event log and replayed for late joiners — adding a variant here automatically
  participates in recording/replay.
- **`omdurman-app`** — the Bevy binary (`omdurman`). Owns rendering, input, egui UI, dice physics
  (avian3d), camera, networking glue, and editor tools. Entry point: `omdurman-app/src/main.rs`.
- **`tools/traceability-typst`** — regenerates the traceability PDF from `docs/traceability.toml`.

## Architecture: event-sourced, peer-to-peer, host-relayed

The system is a deterministic event-sourced engine over a peer-to-peer mesh:

1. **Rules engine is authoritative.** Game mutations are `GameEffect`s; `apply_effect` validates
   and mutates `GameState`. Dice are rolled *before* the effect is constructed and embedded in it,
   so re-applying the effect on any peer reproduces the same state.
2. **Host relays for global ordering.** A peer wishing to act sends an unsequenced event; the host
   assigns a sequence number and broadcasts it (`NetMsg::Sequenced`). Every peer (including the
   host) applies events *only* when they receive the sequenced echo — `apply-on-echo`. The host's
   `loopback` queue in `PendingIncoming` feeds its own outgoing sequenced events through the same
   receive path so it doesn't apply them twice or apply them out of order.
3. **Canonical event log.** `GameRecord` records every `GameEvent` in order. Late joiners are sent
   the record and replay it to converge to current state. `PendingIncoming.replay` flags events
   that came from a replay so they aren't re-recorded.
4. **Outbound staging.** `PendingEdits` buffers reliable broadcasts and targeted sends so multiple
   systems can stage messages without contending for `&mut MatchboxSocket`. The host routes its own
   outgoing game events through `incoming.loopback` as unsequenced `NetMsg::Game`, so `handle_socket`
   sequences them through the same arm as guest submissions (single serialization point). Recording
   happens via `GameRecorder::push_event` on the `NetMsg::Sequenced` echo — the host records on echo
   exactly like every other peer, preserving the apply-on-echo invariant.
   Unreliable traffic (cursor positions, ephemeral selections) bypasses staging.
5. **PRNG is shared and seeded.** `GameRng(ChaCha8Rng)` is seeded from the seed in `InitialGameState`
   so late joiners produce the same sequence on replay.

## Architecture: dual-map (campaign + Fall-of-Khartoum)

The game uses two boards, switched by `MapKind` (`Campaign`, `FallOfKhartoum`).
Many net events carry an explicit `map: MapKind` so an edit applies to the right board,
and `ActiveEditMap` tracks which one is live for editor input.
`PendingMapLoad` is set by the `StartGame` handler (and the editor's map toggle) to (re)load a board on the next frame.

## Annotations file (`assets/annotations.ron`)

`AnnotationsFile` (in `omdurman-types`) is the persisted map/sprite state.
Critically: **unit sprite annotations are top-level on `AnnotationsFile`, not per-board**
— there is one global sprites block.
Edits are flushed by `editor::flush_annotations_to_disk` on an idle debounce (`AnnotationsDirty.idle` exceeds `ANNOTATIONS_FLUSH_SECS`).
Never discard unstaged `annotations.ron` edits — they're real synced map state; if invalid, migrate rather than drop.

## Mode switching (UI)

The top-level `AppMode` (`Game`, `Sandbox`, `Editor`) is selected via the `mode_toolbar`
egui top panel in `ui_plugin.rs`; there are no keyboard shortcuts for mode switching.
Editor vs game behaviour is gated on the active mode, not on a build flag.

## Traceability

`docs/traceability.toml` is the bijective rulebook ↔ code mapping.
Two invariants enforced by `cargo test -p omdurman-rules --test traceability`:

- Every `[[mapping]]` with `status = "implemented"` must list at least one `[[mapping.impl]]`
  (`file`, `line`, `symbol`), and the file must contain that symbol.
- Every `§N` citation in Rust source must appear in at least one mapping.

When adding code that implements a new rulebook section, cite the section in a comment
(`(rulebook §6.11)`) *and* add the matching `[[mapping]]` entry.
When renaming a symbol, update its `symbol` field in `traceability.toml` or the test will fail.

## Conventions to preserve

- The rules engine uses `value_enum!` (defined at the top of `omdurman-rules/src/lib.rs`) for any
  quantitative value with a fixed annotated set of possibilities. Match arms then stay
  exhaustive — prefer extending `value_enum!` over adding an `_ =>` arm.
- `omdurman-types` and `omdurman-rules` have no Bevy dependency. Keep it that way; Bevy lives in
  `omdurman-app` (and `omdurman-hexmap` for its plugin shim).
- `GameEvent` is the only enum whose variants get recorded and replayed. Adding a network message
  that should *not* persist (cursor pings, selection hints) belongs in `Ephemeral`, not
  `GameEvent`.
- New game mutations must go through a new `GameEffect` variant + `apply_effect` arm so they
  participate in determinism, replay, and host-relay ordering.

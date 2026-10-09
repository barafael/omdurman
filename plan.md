# Audit findings & plan — 2026-10-07 (revised)

Scope pass over hangs / busy loops / crashes / performance, plus live sessions
driving the native app. Revised after a sceptical review: every cited
location was re-checked against the tree; the diagnoses in §1 were not
confirmed and are marked so; the work packages are reordered by evidence.

---

## 0. Ground rules for this plan

- **The tree carries ~1000 lines of uncommitted WIP** across ~30 files, and
  the plan's line numbers refer to that tree, not to `main`. Two of the
  plan's "existing" building blocks are WIP: `WriteRetry` (game_record.rs)
  and the AI `engine_accepts` / `stuck` handling (bot_player.rs). WP0 lands
  the WIP first; nothing else starts before it.
- **Another Claude session owns the WIP** (confirmed 2026-10-07 evening):
  two themes, (1) runtime-failure / busy-loop hardening plus the camera
  tilt, (2) fixes from a review of the title-screen commit decdd96
  (`Screen` computed state, splash menu buttons via picking observers,
  bevy_feathers removed). It reports: builds, clippy clean, 227 app tests
  pass; still doing the wasm check, a live check of the menu buttons and
  tuning pane, and a final review. Nothing in this plan touches the tree
  before that session reports the WIP committable.
- This repo configures an external diff tool. Any `--in-diff` run or diff
  saved for review needs `git diff --no-ext-diff`.
- `target/debug` fingerprints were corrupted once by a two-toolchain fight
  (`cargo test` on 1.97.1 while the workspace pins 1.98.0). If builds behave
  oddly, `cargo clean -p <crate>`.

---

## 1. The input artifact: measured, not explained

An instrumented build (`egui_ctx.input(...)` logging) showed, with the
driver's synthetic input:

- egui laid the UI out at ~1536x852 logical px while screenshots and
  xdotool worked in 3072x1704, i.e. the *app* saw a scale factor of 2.0.
- Synthetic (xdotool/XTest) motion events landed at the right logical
  position; synthetic clicks reached egui at the raw physical coordinates
  (click at (2000,800) -> egui pointer (2000,800), outside the viewport).
- Real clicks worked.

What this does **not** establish:

- That "the desktop changed to 2.0". The compositor config
  (`~/.config/kwinoutputconfig.json`) lists outputs at 1, 1.25, 1.45 and
  1.5, none at 2.0; kdeglobals says 1.5. The 2.0 the app sees is the app's
  winit/Wayland path (integer buffer scale, fractional-scale negotiation,
  or the instrumentation reading the wrong window), not the desktop. The
  run-omdurman skill's 1.5 calibration for xdotool/screenshot space may
  still be right. Do not recalibrate the skill on this evidence.
- The click mechanism. The session is Wayland (`XDG_SESSION_TYPE`), so
  xdotool reaches the app through XWayland; a Wayland button event carries
  no position. "Clicks arrive at physical coordinates" is one measurement,
  with no mechanism, and the driver already keeps a measured `warp_scale`
  cache per state dir that a changed output leaves stale.
- That *every* failed-click observation on 2026-10-07 was this. Some may
  have been; the generalisation is not warranted.

To close: reproduce on a clean build with (a) the real pointer, (b) xdotool
clicking at the *logical* coordinates in physical space (if clicks are taken
unscaled, that should hit), (c) a fresh driver state dir. Record
`window.scale_factor()` and the compositor's reported scale in the same log
line. Only then touch the skill.

### 1.1 The red lobby toast: unreproduced

Several sessions showed a persistent bottom-left FIELD TELEGRAPH slip:

    Start Battle failed: no host is elected yet (self-hosting without a
    signalling server? Try reconnecting from the lobby.)
    no peer id for host

This text exists nowhere: not in the source, not in git history (`git log
-S`), not in the stashes, not in the current WIP, not in any binary left
under `target/`. The mechanism hypothesised earlier is contradicted by the
code: the lobby's Start Battle button calls `PendingEdits::submit_game`
directly (lobby.rs ~832), which returns a uid and cannot refuse; the only
synchronous refusal path is `submit::submit_checked`, headed "Order
Refused", and no `RuleError` mentions hosts. The audit session built
instrumented binaries and then ran `cargo clean -p` -- those deleted
binaries are the one place the byte-grep could not look. Most likely the
toast came from an experimental build of that session.

To close: on a **clean** build (`git stash` the WIP or build from `main`),
real mouse, offline (`OMDURMAN_OFFLINE=1`), click Start Battle in a lobby
with no faction/AI selected, and with the recipe in 1.2. If the toast does
not appear, close the item. Only if it does, add the backtrace log in
`Dispatches::push` (dispatch.rs) and find the composer.

### 1.2 Correct lobby recipe (human clicks)

Looks-selected is not selected: the faction row, the scenario, and the AI
commander tick each need an explicit click before Start Battle enables
(`all_players_ready_with_ai`, lobby.rs). Order: faction -> AI commander ->
scenario -> Start Battle. Working offline Campaign games exist on record
(`games/game_2026-10-06T20-51-49-*`, `games/game_2026-10-06T21-09-47-*`:
human Anglo-Egyptian + AI Dervish, 40 events each, setup completes with
`ConfirmSetupReady`), so offline Start Battle is not broken.

---

## 2. Hangs / busy loops / crashes — audit results (verified)

Clean. No busy-waits (reactive pacing in `activity.rs`; retransmit 0.5 s;
snapshot requests 2 s -> 15 s once patient (WIP); `RecentUids` 4096 and
`ReorderBuffer` 1024 caps; once-per-condition reporting). No blocking I/O,
threads, or `block_on` in systems. All `loop {}` sites terminate
(seat_arbiter.rs fixed-point drains a queue; Dijkstras cost-pruned).

| # | Finding | Where | Severity |
|---|---------|-------|----------|
| H1 | `bot_player_act` early-return gates are silent; with `Seats` empty the AI no-ops with zero logging, indistinguishable from a hung AI. The WIP adds a once-per-streak warn for the *stuck* case only. | bot_player.rs `bot_player_act` | observability |
| H2 | Playthrough livelock cap is 500k iterations x full `legal_actions` -- a pathological loop spins for hours, not forever | omdurman-bot/src/playthrough.rs `MAX_DRIVER_ITERATIONS` | low (headless) |
| H3 | Latent panic: `candidates[pick]` on an empty list (all current callers guarantee non-empty) | aggressive.rs `pick`; commanders.rs `rank_*` | latent |
| H4 | `reconnect_attempts(None)` -- unlimited signalling reconnect, deliberate | omdurman-net/src/lib.rs | by design |
| H5 | App exits during sessions were the user closing windows; no panics in any log | -- | none |

---

## 3. Work packages

Ordered by evidence. Gates per commit: `cargo fmt --check`, `cargo clippy
--workspace --all-targets -- -D warnings`, `cargo test --workspace`
(includes the traceability test). The mutation gate is hard-coded to
`omdurman-rules`; no WP below touches that crate, so it does not apply.
No touched symbol is cited in `docs/traceability.toml`.

### WP0 — Land the WIP (first; several commits, by theme)
Themes in the tree, each its own commit once clippy/tests pass:
1. `Screen` computed state replacing the five view predicates
   (state.rs, main.rs, every `run_if` site, splash/screen.rs, fx/mod.rs,
   ui_plugin/mod.rs, picker/mod.rs, render.rs, camera.rs, reinforce.rs,
   hover_tooltip.rs, combat_card.rs).
2. Splash menu buttons via picking observers + material write guard
   (splash/*). **Check the observers are registered** (`add_observer` for
   `press_/release_/click_menu_button`, `end_menu_button_drag`,
   `cancel_menu_button`) -- the diff adds the functions, splash/mod.rs
   must wire them.
3. Net: solo-room session evidence (`SOLO_ROOM_SECS`), `rejoining`,
   signalling timeout -> offline fallback (`SIGNALLING_TIMEOUT_SECS`),
   snapshot-request backoff, loopback dedup, re-announce after reconnect,
   once-per-peer undecodable warning (omdurman-net/src/lib.rs,
   net_socket.rs, net_plugin.rs, CLAUDE.md §5).
4. Idle-pacing fixes: bot in-flight spin bound + stuck handling,
   combat-card repaint only while fading, placement grace frames,
   `SeatPresence` change detection, camera ease snap, stacked-unit lerp
   snap (bot_player.rs, combat_card.rs, placed.rs, seats.rs,
   omdurman-board-ui/src/camera.rs).
5. Game-record write backoff (`WriteRetry`, game_record.rs).
6. Small fixes: event-viewer UTF-8 highlighter + test, orphaned
   selection/hover rings, sidebar offer recompute on seat change, camera
   fit tilt.

### Progress log (2026-10-07, evening)
- WP0 done: the WIP landed as 5667431 (title-screen review fixes) and
  2a90944 (hardening); plus ee09523 (pan variation) from the other session.
- WP1 done: `OMDURMAN_FRAME_STATS=<secs>` (6040eab, 78ce848). Measured, debug
  build, 3072x1704, offline solo Campaign:
  - idle lobby: 15 fps (ambient pan), 4.3 ms/frame main world
  - AI set-up: 60 fps, every frame asked for, 4-8 ms, spikes to 290 ms
  - board, set-up waiting on the human, pointer still: a first reading
    of 60 fps with `busy=0` turned out to be external. **Closed** by two
    attributed runs through the in-app input path: unfocused 2 fps,
    focused 7-10 fps, each with `busy=0 egui=0 input=0 motion=0` -- the
    designed reactive waits (500 ms / 100 ms). No idle leak; the
    attribution stays in the stats for the next time it is seen.
  - bot CLI baseline (debug): Campaign 10 turns, 3258 events, 10.0-11.0 s
    (~3 ms/event); Fall of Khartoum 3 turns, 429 events, 1.4 s.
  - per event the bot calls `find_unit` ~1,200 and `units_in_hex` ~4,900
    times (3.8 M / 15.9 M over the Campaign game), each a full scan and,
    for `units_in_hex`, a Vec allocation. The WP4 hot spot is confirmed.
- WP2 done (9b12a7f), WP3 done (d31e381), WP5 done (022f8cd).
- WP4, re-targeted by callgrind (no perf on this machine; valgrind is):
  the plan's `units_in_hex` guess was *not* the top cost. Fall of
  Khartoum: 70% under `setup_actions` -- `sort_by_key` recomputing
  `placement_preference` (42 board lookups) per comparison, per unit.
  Campaign (2 turns): 52% the same, then `plan_move` 33%, `threat::key`
  (SipHash over every shooter per query) 11%, SipHash itself 16% of all
  instructions (the engine's board `IndexMap`s). Done, records
  byte-identical to the baseline runs, bot suites green:
  - 99937c1 cached sort keys + one preference pass per enumeration:
    FoK 1.37 s -> 0.68 s, Campaign 14.0 -> 12.5 s (A/B)
  - ad87af8 exact threat-cache key: Campaign 12.5 -> 8.0 s
  - c94bb2b board maps on `FxHasher` (rules crate, deterministic, no OS
    RNG; workspace tests, wasm32 and three board Kani harnesses checked):
    FoK 0.68 -> 0.39 s, Campaign 8.0 -> 6.9 s
  Remaining from the profile, not done: `plan_move` / `move_memory`
  (BTreeMap entry churn), `sort_dedup_hexes` merges in movement
  enumeration (~10%), `hex_in_enemy_zoc` (6%), `can_deploy_unit` (6%).
  A per-decision unit index is still plausible but no longer the first
  thing to do.
- The driver can now deliver input without xdotool warps: the hex probe
  serves `<probe>.input` (one step per frame), the driver falls back to it
  when a warp is ignored (`OMDURMAN_INPUT=probe` to use it outright). The
  warp failure was the user's pointer resting on a native Wayland window,
  which XWayland cannot warp out of -- not a scale factor.
- The §1 click claim is refuted: xdotool clicks landed in egui on a clean
  build (faction row, AI tick, Start Battle, Ready all took). The driver
  refuses input when the game is not the active window; that, not scale,
  blocked the later runs. §1.1 (the toast) remains unreproduced.

### WP1 — Measure before optimising
- App: a frame-time readout behind an env var (`OMDURMAN_FRAME_STATS=1`:
  `FrameTimeDiagnosticsPlugin` + a periodic `info!` of avg/p95 frame time
  and frame count per second), so "per frame" claims get numbers. Capture:
  idle lobby, idle board, AI setup phase, fire phase with 5+ allocations,
  event viewer open with a 500-event record.
- Bot: `time cargo run -q -p omdurman-bot --bin omdurman-bot-cli -- play
  Campaign 123 commanders 10` and FallOfKhartoum; one arena seed. Record
  the numbers in the commit message of every later WP.

### WP2 — Stop rebuilding allocation arrows every frame
`fire_allocation_arrows` (fire_allocation.rs) despawns/respawns all
`AllocationArrow` entities per frame with no change guard, unlike its
siblings in fire.rs (`inputs_moved`). Add a `Local` last-drawn key
(allocation len + committed + phase + attack endpoints) and rebuild only
when it changes. The one app-side item with a plausible visible cost
(entity + mesh churn); WP1 numbers before/after.

### WP3 — Artifact-writer backoff (robustness, not perf)
`save_telegram_artifacts` (telegram.rs) and `save_newspaper_artifact`
(newspaper.rs) retry every frame on a persistent write failure: full-file
rewrite + `warn!` per frame. Port the landed `WriteRetry` (WP0.5). Test:
same pattern as game_record's retry tests. Costs nothing in the normal
case; it is about not flooding the log when the disk is full.

### WP4 — Bot-side unit index (test-suite and arena speed; behaviour-sensitive)
`GameState::find_unit` / `units_in_hex` (effects/state.rs) are O(units)
scans, multiplied through every bot helper. **In the app the AI is
cooldown-bound** (0.4 s per action; measured 2.3 events/s live), so this
does not change game pace. It speeds up the bot test suite, the CLI and
the arena, which is worth having, stated as such.
- Keep `GameState` untouched. Build a `UnitIndex` once per
  `next_ai_action` and thread it through `plan_move`, `plan_fire`,
  `position_value`, `threat::reaching`/`best_shot_from`,
  `melee_reaching`, ZOC checks.
- **Determinism constraint:** the index must never be iterated, only
  looked up -- or use `BTreeMap`. A `HashMap` iterates in per-instance
  random order and would make the AI's choices run-dependent.
- Also: `threat.rs` `key()` hashes all units per `reaching()` call --
  compute once per decision. `PATHS` FIFO-32 cache -> LRU or sized to goal
  count. `deploy_hex_options` prefilter by deployment area.
  `victim_to_remove` clones the state per victim.
- **Not** the Fall-of-Khartoum replan scoping in `move_memory.rs`
  unless the arena shows it strength-neutral: it changes AI behaviour.
Gates: the whole bot suite (determinism, no_oscillation, playability,
termination, invariants, adversarial, head_to_head, strategy_corpus)
**and** the arena at one seed (`#[ignore]`d, minutes) -- mandatory for
this WP, with before/after strength in the commit message.

### WP5 — AI observability (H1)
One-time `info!` when an AI gate blocks for > N seconds (which gate, which
state), and make the probe state line distinguish "no StartGame applied
yet" from "in setup".

### Hygiene (optional, after the above)
- `placement_marker_color` (picker/overlays.rs) writes the material handle
  every frame -- compare-guard (`set_if_neq` discipline).
- Event viewer (dev-only): cache row strings keyed by (events len, last
  seq); cache the RON highlight by (selected idx, seq).

### Dropped, with reasons
- *Single validation per AI action* (old WP4): saves one `GameState` clone
  per 0.4 s in the app; `pick_validated` already clones per candidate, so
  the hot cost is upstream. Refactors WIP that has not landed.
- *Per-frame record scan in Setup* (old WP6): a reverse `find` over a few
  hundred events, during Setup only. Microseconds.
- *zoc.rs / los.rs collect-before-early-return* (old WP7): the early
  returns *use* the collected list to despawn the overlays; moving it
  breaks them. The saving is one small allocation.
- `ACT_COOLDOWN_SECS` pacing: a product decision, not a perf fix.
- Engine-side `GameState` indexing: serialization + Kani risk; WP4 covers
  the bot-side win.

---

## 4. Session facts for the record

- AI setup pacing from a real record: 39 deployments + `ConfirmSetupReady`
  in 17.5 s (~2.3 ev/s), i.e. the cooldown ceiling. Headless
  `omdurman-bot-cli play Campaign 123 commanders` finishes 3 turns / 516
  events in seconds -- the engine+bot are not the in-game bottleneck.
- Working offline games exist (see 1.2).
- Two app exits during sessions were the user closing windows; no panics.

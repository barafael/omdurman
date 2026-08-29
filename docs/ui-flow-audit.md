# UI Flow Audit & Improvement Plan

Audit of `docs/ui-interaction-flows.md` (F1–F26) against the current
`omdurman-app` implementation, plus a prioritized plan to close the gaps.
Evidence is `file:line` as of this audit; line numbers will drift.

Headline: **the rules engine is far ahead of the UI.** Most "missing" flows
have complete `GameEffect` support that no app code invokes. A handful of flows
are impossible as specified without engine or net-protocol changes; a few are
outright rules bugs where the engine deviates from the manual.

Legend: ✅ implemented · 🟡 partial · ❌ missing · ⛔ impossible as specified
(engine/net blocker) · 🐛 engine deviation from the rulebook

---

## 1. Verdicts

| Flow | Verdict | One-line status |
|---|---|---|
| F1 Campaign Dervish deployment | 🟡 | Generic picker + single-color zone tint; **no per-class zones/groups/counters** (engine explicitly delegates zone semantics to the UI, `effects.rs:1150-1156`, UI never implements them) |
| F2 Historical deployment | ❌ | Leaders auto-placed by host (`scenario_setup.rs:58-101`); player can pick them up and re-place anywhere; **no lettered-hex snap, no 3-hex halo, no setup-time LOS validator** — §9.212 unenforced |
| F3 FoK garrison deployment | ✅ | Engine zone exact (`effects.rs:1219-1260`); only GORDON padlock badge missing (suppressed in setup, `fok_panel.rs:157,172`) |
| F4 River mines | ⛔🟡 | Placement UI exists (`river_placement.rs:15-46`) but **no band check, no secrecy** (broadcast + recorded, visible to opponent), **no map marker**, Dervish-only is UI-side only |
| F5 River chain | ⛔🟡 | Placement exists (`river_placement.rs:48-87`) but contiguity is loose (`distance <= 2`), 1-hex chains allowed, **no secrecy, no chain sprite, and `SinkChain` has zero UI path** (bot-only) |
| F6 Basic move | ✅ | BFS overlay, ZOC stop rings, per-leg costs, path pinning, stacking hints all present (`picker.rs`); minor: no pulse animation, stacking preview textual |
| F7 Gunboat move | 🟡 | Two pools shown as text; **local budget uses `max(up,down)` ignoring the sticky upstream cap** (`picker.rs:127-132`); no upstream-lock toast; **no Nile flow arrows rendered** |
| F8 Reinforcement entry | ❌ | **Engine complete** (`PlaceReinforcements`, waves, quotas, entry MP) — **no app code constructs or visualizes it**; FoK entry highlight exists but is unreachable via the picker post-setup |
| F9 Construct Zariba | 🟡🐛 | Panel gated to **RoyalEngineers only** (rulebook: any A-E infantry); all six sides offered regardless of adjacency; **no origin tracking** (§5.3 "begins AND ends adjacent"); **no hexside rendering**; **fire/melee lock never enforced** |
| F10 RE demolition | 🟡 | Flow works end-to-end incl. deferred resolution + toasts; missing on-map badge; same unenforced fire/melee lock as F9 |
| F11 Friendlies transport | 🐛 | **Cross ignores destination** (sends gunboat's own hex, `ui_plugin.rs:1022-1029`); **`ReadyToDisembark` is a dead-end** (Disembark re-offered then guaranteed-rejected; mission never clears → no second mission ever) |
| F12 Desertion roll | 🟡 | Roll→N→select→confirm works (`desertion.rs`); ineligible units hidden not greyed, **not faction-gated** (A-E sees the Dervish panel), no no-VP footnote |
| F13 LOS probe | 🟡 | Hover-origin green/red ring partition (`los.rs`); **no two-hex ray, no blocked-terrain naming in probe, ignores units** (`|_| None`), no entrenched/howitzer/breach annotations |
| F14 Brigade-integrity attack | 🟡 | Preview has factors, band text, integrity +1 line, CRT row, outcome bands; **no per-band target colors, no per-unit checkboxes** (whole stack always bundled) |
| F15 Stack split warning | ❌ | Split made structurally impossible (`fire_allocation.rs:78-83`) and never explained |
| F16 Allocate-then-resolve | 🟡 | Tray + Execute All exist; **no reorder/edit; target "fired-at-once" not pre-checked** → silent echo rejections (`game_apply.rs:74-75` warn only) |
| F17 Defensive fire interjection | 🟡 | Mechanics work for the non-active player; **no prompt, no Pass, defender can't End Phase** (`ui_plugin.rs:468,530`) |
| F18 Maxim second fire | ❌ | Subphase routing fine; **no fired pips** — `units_fired_this_phase` never read (TODO at `render.rs:149-150`) |
| F19 Howitzer fire | 🟡🐛 | Two-dice broadcast works; **no 4–10 ring, no scatter animation, no impact hex surfaced; engine spares friendlies** (`effects.rs:3586`) contradicting §6.64; no combine checkbox (engine treats as independent) |
| F20 Artillery special targets | 🟡/❌ | Gunboat/fort: silent filtering + post-hoc slips, no threshold tooltips; **wall-breach UI entirely absent** (`can_fire_at_wall` + `ArtilleryBreachWall` unused by app; no breach markers rendered) |
| F21 Melee planning | 🟡 | Preview/declare/react/resolve works; orange rings not red, **no wall 🚫/gate-breach glow on the adjacency ring**, fort −3 not surfaced, error reasons swallowed (`melee.rs:114-126`) |
| F22 Retreat before melee | 🟡 | Full retreat flow in `retreat.rs`; **UI threat-gate mismatches engine** (rings before any declared melee); no once-per-turn pip; no retreat-path drawing; **"attack the retreating unit?" has no engine support at all** |
| F23 Advance after combat | 🟡 | Click-to-advance + rings + engine windows; **no per-unit "Advance?" chips; Dervish mandatory advance auto-fills and is silently discarded** (`mandatory_advance` dropped at `combat_card.rs:205`) |
| F24 Mahdi's Tomb tracker | ❌ | Engine scoring complete (`effects.rs:3093-3124`); **no UI at all** beyond a post-hoc VP row |
| F25 Victory panel | 🟡 | Live ledger + FoK projection + newspaper modal exist; **game-over screen omits the arithmetic** (superiority table, Historical level subtraction, alternative-decisive announcement); zero-count rows hidden; no Campaign/Historical live projection |
| F26 Unit inspector | 🟡 | Hover tooltip + selected-unit panel + charts sheet (CRT/TEC/Timing/Arrivals/Rulebook); **no right-click card; no has-fired/loaded/drifting pips; no LOS Table or Scattergram chart; `ChartSheetRequest` has zero producers** (deep links dead) |

Cross-cutting: phase banner + night badge + "your turn" framing ✅ (but the
sequence indicator collapses the two fire subphases into one rung,
`ui_phase_state.rs:187-201`); victory modal not shown while Spectating;
spectators are mislabelled as the defender in the melee window (`melee.rs:193`).

**Engine support with zero UI consumers (build these, don't redesign them):**

| Engine item | UI status |
|---|---|
| `GameEffect::PlaceReinforcements` + entrance areas + wave quotas (`effects.rs:4272-4395`, `reinforcements.rs`) | unused by app |
| `GameEffect::ArtilleryBreachWall` + `can_fire_at_wall` (`effects.rs:1999-2101, 307-311`) | bot-only |
| `GameEffect::SinkChain` (`effects.rs:5132-5143`) | bot-only |
| `GameEffect::DriftGunboat` (`effects.rs:5051-5083`) | never emitted |
| `GameState::zariba_hexsides` | never rendered |
| `GameState::units_fired_this_phase` / `units_fired_at_this_phase` | never read |
| `GameState::gunboats_upstream_this_turn`, `zoc_stopped_this_turn` | never read (UI re-derives, wrongly) |
| `board.entrance_hexes(NamedArea)`, `board.flow_at` | never drawn |
| `Observation::WallBreached`/`MeleeResolved.mandatory_advance`/`HowitzerResolution` | text-only or dropped |
| `ChartSheetRequest` staging (`charts.rs:103-136`) | no producers |
| `DervishLeader::setup_letter` (`rules/lib.rs:375-397`) | app duplicates the table by hand |

---

## 2. Impossible-as-specified & rules bugs (engine/net changes required)

These cannot be fixed purely in the UI; they are prerequisites for the flows.

1. **Secrecy of mines/chain (F4/F5).** All mutations are recorded, host-sequenced
   `GameEvent`s replayed to every peer (`omdurman-net/src/lib.rs:23-100`), and
   `PlaceMine`/`PlaceChain` store plaintext hexes in the shared `GameState`.
   True secrecy needs a hidden-information protocol (e.g. host-staged secrets
   revealed on trigger, or an encrypted/commit-reveal `Control` exchange).
   Cheaper interim: UI never renders opponent placements + event-viewer
   redaction — honest against the UI, not against a memory-inspecting peer.
2. **Howitzer friendly-occupied impact (F19).** `resolve_fire_attack` collects
   only *opponent* units at the impact hex (`effects.rs:3586`); §6.64 says
   results must apply even into friendly-occupied hexes. Engine bug + UI surfacing.
3. **Constructing/demolishing units may still fire & melee (F9/F10).**
   `constructing_zariba`/`demolishing` are set but never consulted by
   `can_fire_at`/`can_melee`. Engine enforcement + UI exclusion needed.
4. **Zariba "begins AND ends the turn adjacent" (F9).** No origin tracking
   exists; `ConstructZariba` builds immediately on click. Needs an
   intent-then-validate-at-turn-end model (like demolitions already do).
5. **Entrenched units don't block LOS (§9.232, F13).** `los_table.rs` has no
   trench exception at all. Engine LOS gap, then probe/fire annotations.
6. **"Attack the retreating unit?" (F22, §7.5).** No engine support; retreats
   simply remove the unit from the pending attack's target set.
7. **Chain placement legality (F5).** 1-hex chains accepted; contiguity is
   `distance <= 2` (allows gaps); no "south of Khor Shambat row" band for
   mines or chain anywhere. Tighten `can_place_chain`/`can_place_mine` +
   board data (needs a board landmark/row for the Khor Shambat mouth).
8. **Friendlies transport state machine (F11).** `Cross` destination is
   meaningless (gunboat's own hex), `ReadyToDisembark` can never complete, and
   the mission never clears — no second ferry is possible. Engine redesign of
   the `TransportState` transitions + real destination pick in UI.
9. **Zariba construction by any A-E infantry (F9).** UI gates to RE only;
   §5.3 says any A-E infantry. UI fix once 4 lands (engine already accepts
   any `unit_ids`).

---

## 3. Improvement plan

Ordered by priority; each item names the flow(s), the change, and where.
P0 = correctness (wrong behavior today) · P1 = missing capabilities with
engine support ready · P2 = interaction depth/feedback · P3 = protocol/polish.

### P0 — Correctness

| # | Task | Files | Flows |
|---|---|---|---|
| 0.1 | Enforce construct/demolish locks in `can_fire_at`/`can_melee`; exclude those units from target rings & allocation | `omdurman-rules/src/effects.rs` (fire ~1786, melee ~2110), `omdurman-app/src/fire.rs`, `melee.rs` | F9, F10 |
| 0.2 | Howitzer applies CRT to *all* units at impact hex (§6.64) + tests | `effects.rs:3586`, `resolve_fire_attack` | F19 |
| 0.3 | Fix Friendlies transport: real `Cross` destination (west-bank-adjacent pick), complete `ReadyToDisembark` (disembark→move/cost, clear mission), allow second mission; UI destination click + tracker | `effects.rs:4950-5041`, `omdurman-app/src/ui_plugin.rs:917-1050` | F11 |
| 0.4 | Zariba: intent model — declare at click, validate begin+end adjacency at `end_player_turn`, build then; open to any A-E infantry; render `zariba_hexsides` on the map plane | `effects.rs:4447-4460`, `ui_plugin.rs:1092-1215`, `picker.rs`/`render.rs` | F9 |
| 0.5 | Tighten chain placement: ≥2 contiguous hexes (exact adjacency), band check vs Khor Shambat row (add board datum); same band check for mines | `river_placement.rs`, `effects.rs:1395-1435`, board data | F4, F5 |
| 0.6 | Desertion panel: faction-gate to Dervish, grey (not hide) exempt units, add no-VP footnote; surface engine rejection as slip | `desertion.rs`, `game_apply.rs` | F12 |
| 0.7 | Retreat threat-gate: only offer retreat rings while a *declared* melee targets the unit's hex; draw the 2-hex path; surface `RetreatBlockedByWall` | `retreat.rs:55-127` | F22 |
| 0.8 | Gunboat budget: read `gunboats_upstream_this_turn` and cap the overlay at the sticky allowance; surface `GunboatUpstreamCap` rejection as a slip, not a log line | `picker.rs:127-132`, `dispatch.rs` | F7 |
| 0.9 | Surface all engine rejections as dispatch slips (replace the `warn!`-only path in `apply_game_event`) | `game_apply.rs:68-96` | all |
| 0.10 | Spectators: never show defender retreat prompt; show victory modal in Spectating at game end | `melee.rs:193`, `ui_plugin.rs:124` | F21, F25 |

### P1 — Missing capabilities (engine ready, build the UI)

| # | Task | Files | Flows |
|---|---|---|---|
| 1.1 | **Reinforcement entry UI**: per-turn tray with quota meter (12 land units; leaders exempt checklist "deploy by end of turn 4"); highlight `entrance_hexes` per identity with entry-cost tooltips; emit `PlaceReinforcements` | new `reinforcements_ui.rs` + `picker.rs` hook | F8 |
| 1.2 | **Wall-breach fire UI**: wall hexsides as artillery targets in the fire flow (reuse `can_fire_at_wall` for range/LOS preview), allocate `ArtilleryBreachWall`, render `HexsideKind::Breach` markers on the live board | `fire.rs`, `fire_allocation.rs`, `render.rs` | F20 |
| 1.3 | **Chain-sink UI**: artillery-vs-chain allocation (CRT 3+ → `SinkChain`); land-cutting: mark unit "cutting chain" (needs small engine state or reuse a marker), auto-emit `SinkChain` when the condition holds at turn end; chain sprite rendering | `fire_allocation.rs`, `actions_panel.rs`, `render.rs`, small `effects.rs` addition | F5 |
| 1.4 | **Mahdi's Tomb tracker**: persistent panel — holder, live capture-requirements checklist (leader + non-Friendlies combat unit, both undisrupted), anchored map marker at the Tomb hex | new section in `fok_panel.rs`-style module + `picker.rs` | F24 |
| 1.5 | **Setup LOS validator (Historical)**: reuse `los_path_analysis` to sweep placed Dervish units vs all A-E units; red-flag exposed hexes; gate Ready | `placement.rs`/new `setup_validation.rs` | F2 |
| 1.6 | **Historical leader letters**: replace the hand table with `DervishLeader::setup_letter()`; clicking an unplaced leader highlights its lettered hex and snaps placement; block re-pickup of correctly placed leaders | `scenario_setup.rs`, `picker.rs` | F2 |
| 1.7 | **LOS Table + Scattergram chart tabs**: add scans or render the LOS table from `los_table.rs`; scattergram hex-diagram from `howitzer_scatter.rs`; wire `ChartSheetRequest` producers from fire/melee preview modifier lines and LOS probe | `charts.rs`, producers in `fire.rs`/`melee.rs`/`los.rs` | F13, F26 |
| 1.8 | **Mine/chain map markers** (own-view only pending P3 secrecy) + mine-resolution prompt/roll animation on trigger | `render.rs`, `river_placement.rs` | F4, F5 |

### P2 — Interaction depth & feedback

| # | Task | Files | Flows |
|---|---|---|---|
| 2.1 | Fire allocation: per-unit checkboxes in the preview (partial-stack attacks — engine already accepts subsets); split-integrity warning naming the brigade; per-unit "already allocated" state | `fire.rs:528-535`, `fire_allocation.rs` | F14, F15 |
| 2.2 | Tray: drag-to-reorder resolution order; edit = remove+re-click is fine, but keep allocations across subphase resets visibly | `fire_allocation.rs:162-214` | F16 |
| 2.3 | Target pre-checks in UI: "already fired at" rings/tooltip from `units_fired_at_this_phase`; fired pips from `units_fired_this_phase` (generalize `ActedMarker` per its own TODO) | `fire.rs`, `render.rs:149-174` | F16, F18, F26 |
| 2.4 | Defensive-fire interjection: prompt panel for the non-active player when attacks are possible; explicit **Pass** button (defender can end their window) | `ui_plugin.rs:468-535`, new prompt in `fire.rs` | F17 |
| 2.5 | Howitzer UX: 4–10 annulus overlay on gunboat selection; post-resolution target→impact arrow + impact-hex line in card/slip; friendly-impact warning; combine-with-Maxim checkbox (post-hoc combined roll or documented deviation) | `fire.rs`, `fire_allocation.rs:252-268`, `combat_card.rs` | F19 |
| 2.6 | LOS probe upgrade: click-origin ray to hovered hex with blocked-feature naming (reuse `LosStepResult::Blocked`), unit-blocking toggle, howitzer/entrenchment/breach annotations once 2.5/§9.232 land | `los.rs:89-130` | F13 |
| 2.7 | Per-band target coloring (×3/×2/×1/×½) for legal fire targets; red rings for melee | `fire.rs:116-124`, `melee.rs:71` | F14, F21 |
| 2.8 | Melee ring annotations: 🚫 on wall-blocked hexsides, glow on gate/breach; fort −3 fire note in previews; surface `can_melee` rejection reasons on hover | `melee.rs:29-76, 370-577` | F21 |
| 2.9 | Advance chips: per-eligible-unit floating "Advance into [hex]?" chips (not just selected unit); display engine's `mandatory_advance` list when Dervish auto-advances | `melee.rs:264-304`, `combat_card.rs:205`, `dispatch.rs` | F23 |
| 2.10 | Status pips in inspector: loaded-on, engines-lost/drifting, constructing, demolishing, retreated, has-fired (after 2.3) | `actions_panel.rs`, `render.rs` | F26 |
| 2.11 | Nile flow arrows overlay from `board.flow_at`; upstream-lock toast on first upstream step | `hexmap` overlay + `picker.rs` | F7 |
| 2.12 | Zariba build preview: highlight candidate hexsides before committing; per-class setup groups/counters and per-class zone tints for Campaign (encode §9.111 zones as board data or UI tables) | `ui_plugin.rs:1092-1215`, `picker.rs` | F1, F9 |
| 2.13 | 5-rung sequence indicator (split Direct / Maxim+Howitzer); "Your fire window" framing during defensive fire | `ui_phase_state.rs:186-203`, `phase_banner.rs` | cross-cutting |

### P3 — Protocol & polish

| # | Task | Files | Flows |
|---|---|---|---|
| 3.1 | Hidden-placement protocol for mines/chain (host-staged secrets, reveal on trigger/`SinkChain`); until then redact opponent rows in event viewer | `omdurman-net`, `net_plugin.rs`, `event_viewer.rs` | F4, F5 |
| 3.2 | §7.5 "attack the retreating unit?" window (engine: allow newly-adjacent declared attacks on the retreating unit before resolution) | `effects.rs` melee block | F22 |
| 3.3 | Entrenched-don't-block LOS in `los_table` + probe/preview notes (§9.232) | `los_table.rs`, `los.rs` | F13 |
| 3.4 | Game-over arithmetic screen: superiority table, Historical level subtraction, alternative-decisive conditions, Campaign/Historical live "if the game ended now" projection (mirror the FoK panel) | `newspaper.rs`, `ui_plugin.rs:786-911` | F25 |
| 3.5 | Victory ledger: always show all §9.14 rows incl. zero counts; per-row linkage to units | `ui_plugin.rs:574-652` | F25 |
| 3.6 | Selection pulse animation; scattergram hop animation; zariba build animation (satisfying but last) | `render.rs` | F6, F19, F9 |

### Suggested execution order

1. **P0 batch** (0.1, 0.2, 0.3, 0.6, 0.7, 0.8, 0.9, 0.10) — small, high-value
   correctness; 0.4/0.5 are the larger engine items.
2. **P1 quick wins**: 1.4 (Tomb), 1.6 (letters), 1.5 (setup LOS) — contained;
   then 1.2 (wall breach) and 1.1 (reinforcements) as the two big features;
   1.3 chain-sink after 0.5; 1.7/1.8 whenever asset scans are available.
3. **P2** in file-clusters: fire cluster (2.1–2.5), map cluster (2.6, 2.7,
   2.11, 2.12), markers cluster (2.3, 2.10), prompts cluster (2.4, 2.9, 2.13).
4. **P3** last; 3.1 only if secrecy is deemed a real requirement (it changes
   the net protocol and replay model).

Every UI change above stays inside the existing architecture: interactions
emit `GameEffect`s through the host-relay path; nothing new is recorded unless
it adds a `GameEffect` variant (only 0.4's intent model, 1.3's chain-cutting
marker, and 3.1/3.2 touch the engine's effect/state surface).

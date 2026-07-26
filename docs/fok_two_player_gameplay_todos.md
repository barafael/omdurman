# Fall of Khartoum — Two-Player Gameplay TODO Plan

Generated from `fok_two_player_gameplay.md`. Grouped by priority.

---

## Tier 1 — Bugs (break gameplay or allow incorrect actions)

| # | Line | TODO | Finding | Fix | Status |
|---|------|------|---------|-----|--------|
| 1 | 56 | BREECH marker shows as placeable unit | `annotations.ron` marks BREECH cells with `kind: Some(Infantry)` → `is_unit()` returns true. | Add `Breech` kind (playable) and `BareCounter` kind (non-playable) to `UnitKind`. Filter `BareCounter` from picker. | **DONE** |
| 2 | 76 | AE player can place in North Fort hex | North Fort hex `(4,1)` is `Terrain::Building`, which matches `is_garrison_terrain` for AE. | Exclude North Fort from AE deployment zone in `effects.rs`. | **DONE** |
| 3 | 85 | Dervish gunboats can't be placed | Dervish gunboats need Nile. Deployment zone is south/east edge. Only 2 Nile hexes on south edge. | Research in manual. In doubt, allow entire western map edge (all Nile). | **DONE** |
| 4 | 80 | Can't place on eastern map edge | East edge (q=25) has valid land hexes but user reports can't place. | Don't fix annotations. Find issue in game logic. | **DONE** |
| 5 | 69 | AE player can place Dervish units | In unbound sessions, `restrict_to` is `None`. Deploy check uses unit's `section_owner`. | No solo session (game requires two peers). Fix architecturally. | **DEFERRED** |

## Tier 2 — UX issues (confusing or frustrating during setup)

| # | Line | TODO | Finding | Fix | Status |
|---|------|------|---------|-----|--------|
| 6 | 96 | Auto-pickup next unit after placement | After placing, `PickerState::Idle` — player must manually pick next unit. | Right-click deselect already exists. Add default-on "auto-place next" checkbox. | **DONE** |
| 7 | 97 | Re-pickup / re-place before ready | No way to undo a placement during setup. | Add left-click handler on placed units to pick back up (without triggering movement). | **DONE** |
| 8 | 66 | Stacking UI during setup | No visual feedback for stacking limits. | Show up-to-4 unit counters next to each other in hex. Enlarge hex tile by 2x on hover for easier interaction. | **PENDING** |
| 9 | 95 | No feedback for invalid placement zone | Green preview ring shows even for out-of-zone hexes. | In `placement_preview_mesh()`, show red ring when `!in_deployment_zone()`. | **DONE** |
| 10 | 93 | Show placement zones in green | Existing overlay draws brown rings. | Change overlay fill to semi-transparent green during `Phase::Setup`. | **DONE** |
| 11 | 67 | Can't place in trees | Trees aren't in deployment zone. This is correct. | Drop — not placing on trees is fine. | **DROPPED** |

## Tier 3 — Scenario setup automation

| # | Line | TODO | Finding | Fix | Status |
|---|------|------|---------|-----|--------|
| 12 | 49 | Auto "setup scenario" | Gordon and North Fort ARE auto-placed via `FALL_OF_KHARTOUM_SETUP`, but North Fort is NOT placed via the button. Also requires host clicking button. | Fix North Fort placement. Auto-trigger when entering setup phase. | **DONE** |
| 13 | 87 | Dervish can't place all 49 units (stacking) | Stacking not working in placement mode — unit can't be dropped on already occupied hex. | Fix stacking in placement mode. | **DONE** |
| 14 | 77 | Dervish non-unit color counters | Some Dervish "units" are just color markers. | Fix with finding 1 (BareCounter kind). | **DONE** |

## Tier 4 — Scenario filtering

| # | Line | TODO | Finding | Fix | Status |
|---|------|------|---------|-----|--------|
| 15 | 89 | Only show units available in the scenario | All 100+ counters from all scenarios appear in picker. | Hardcode with enums. Use `SectionName`-per-scenario lookup. | **DONE** |

## Tier 5 — Doc corrections

| # | Line | TODO | Fix | Status |
|---|------|------|-----|--------|
| 16 | 20 | Confirm Dervish Range Effects Table for both sides | Confirmed: `range_band_for()` returns `dervish_range_effects()` for FoK. Update doc. | **DONE** |
| 17 | 53 | "Adjacent to any wall hexside" clarification | Correct per §9.321. Add parenthetical. | **DONE** |
| 18 | 59 | FoK doesn't have a walled city | Rewrite section. | **DONE** |
| 19 | 64 | "No engineers in FoK?" | Keep question for later. | **DEFERRED** |
| 20 | 65 | "Only old gunboats allowed" | Named gunboats NOT available in FoK. Update doc and code. | **DONE** |

## Tier 6 — Features (deferred)

| # | Line | TODO | Notes | Status |
|---|------|------|-------|--------|
| 21 | 91 | Single Player Mode | Large feature. Defer. | **DEFERRED** |
| 22 | 92 | Multi Player Mode → lobby | Defer to splash redesign. | **DEFERRED** |
| 23 | 57 | Unit sprite editor too thin | Visual regression. File separately. | **DEFERRED** |
| 24 | 55 | Weird hexes during setup | Need repro. Investigate when testing. | **DEFERRED** |

---

## Execution order

1. **UnitKind variants** — Add `Breech` and `BareCounter` to `UnitKind` (fixes #1, #14)
2. **Deployment zone fixes** — North Fort exclusion (#2), Dervish gunboats (#3), eastern edge (#4)
3. **Scenario filtering** — Hardcode `SectionName`-per-scenario enums (#15)
4. **FoK setup automation** — Fix North Fort, auto-trigger setup (#12)
5. **Stacking in placement** — Fix occupied hex drop (#13)
6. **Placement UX** — Auto-place next (#6), re-pickup (#7), zone feedback (#9, #10)
7. **Stacking UI** — Show counters in hex, enlarge on hover (#8)
8. **Named gunboats** — Remove from FoK (#20)
9. **Doc corrections** — Update walkthrough text (#16, #17, #18)

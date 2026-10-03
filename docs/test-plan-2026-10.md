# Test plan — October 2026 changes

Manual checks for the changes on `fix/victory-points-and-docs` (commits
`faa9c06`, `71e2696`, `a12bea2`). Each item: set-up, steps, expected
result. The automated suites (`cargo test --workspace`, the traceability
tests, the mutation gate) already cover the engine side; these checks cover
what only the running game shows.

**Wire format changed** (new fields on the fire and melee effects): every
peer must run this build, and game records from older builds no longer
replay.

## Start a solo game

```shell
OMDURMAN_OFFLINE=1 cargo run -p omdurman-app
```

Splash → Lobby. Pick a faction and a scenario, tick the AI commander for
the other side, Start Battle. (Agents: the `run-omdurman` skill drives the
same flow; see item 20.)

## Lobby and set-up

1. **Offline lobby lists you.** Open the lobby offline.
   - Expect: your name, "(you) [host]", under Players; after picking a
     faction and ticking the other side's AI, Start Battle enables.
     (Before: nobody was listed and Start Battle stayed grey.)
2. **AI deploys first in the Campaign.** Campaign, you Anglo-Egyptian,
   AI Khalifa.
   - Expect: the AI places all 38 Dervish set-up units (Khalifa in a palace
     hex, Taiasha and artillery in the walled city, 17 forts, Isa Zachneih,
     2 gunboats), confirms Ready, and turn 1 opens on your movement.
     (Before: "Waiting on Dervish" forever.)
3. **Set-up is the deployer's turn.** Campaign, you Dervish, AI Kitchener.
   - The top bar reads "▶ Set-up: you deploy (Dervish)" during set-up.
   - Place a Taiasha in the walled city, click it on the board, press Del
     (or "Return to tray"): it goes back to the counter tray.
     (Before: deployed counters could not be selected or returned, because
     the Anglo-Egyptians move first.)
4. **Drag-and-drop placement.** During any set-up, or a Campaign movement
   phase with reinforcements:
   - Drag a counter from the sidebar onto a legal hex (green deployment
     zone / green entrance hexes) and release: it is placed.
   - Drag one and release over the sidebar: nothing is placed.
   - Click a counter, then click a hex: still places it.
   - The reinforcement hint reads "… drag them onto the green entrance
     hexes (or click one, then a hex)".
   (Before: a drag never placed anything.)

## Fire (Historical, you Anglo-Egyptian, AI Khalifa)

Deploy the gunboats on the Nile hexes next to the Zariba (e.g. Sultan at
(34,12)), the British brigades four to a hex, Ready. Fire comes in the
Dervish turn's Anglo-Egyptian defensive fire once Dervish stacks close in.

5. **Gunboat panel.** Click a named gunboat (Sultan, Sheik, Fateh, Melik,
   Naser).
   - Expect the stats line to end in "… + Maxims 6×2" after the artillery
     weapon (fire 5, 12 up / 18 down MP).
6. **Two weapons per named gunboat** (§2.32).
   - Direct Fire subphase: select the gunboat, click enemy hex A → tray
     row "Gunboat Sultan (5) → …"; click enemy hex B → "Gunboat Sultan
     Maxims (6) → …", both "+1 Anglo-Egyptian direct fire".
   - Resolve: two combat cards, the second naming "Gunboat Sultan Maxims".
   - End phase → Maxim/Howitzer subphase: the same gunboat fires its
     howitzer at a hex 4–10 away ("Howitzer · … · no modifiers") and its
     Maxims again ("Maxim 2nd · … · net die +1").
   - A third Maxims shot in the same subphase is refused.
7. **Maxims-only preview.** Second subphase, gunboat selected, hover an
   enemy hex 1–3 away (too close for the howitzer).
   - Expect a fire preview for the Maxims (factor 6), not nothing.
8. **Maxim battery second fire gets +1** (§6.24).
   - Second subphase, Maxim batteries: tray shows "Maxim 2nd · net die
     +1"; a howitzer shot shows no +1.
9. **Combined fire.** Double-click a British brigade's hex (all four
   battalions) and click the hex the gunboat's Maxims already target.
   - Expect one combined row listing the four battalions and "Gunboat …
     Maxims (6)", net die +2 (+1 direct fire, +1 brigade integrity).
10. **Random disruption** (§CombatResults).
    - A `D` result on a 4-unit stack disrupts exactly 2 — not necessarily
      the first two counters of the stack; a second `D` on the same stack
      falls on the still-undisrupted ones.

## Melee

11. **Quiet tile selection.** Melee phase: double-click your own hex to
    select the tile.
    - Expect: no "melee refused" line in the log, no dispatch. Clicking
      an adjacent enemy then declares the melee as before; a melee across
      a Zariba thorn hedge is refused with a "Melee refused" dispatch.

## Line of sight (§6.3)

12. **Hilltop → ground, footnote 4.** Campaign, you Dervish; in a movement
    phase enter a reinforcement on entrance hex (11,21) (next to the summit
    (12,22)). Toggle the LOS overlay, hover (12,22).
    - Expect: (10,19) and (10,18) are clear (no label) — the unit is
      nearer the firer; (11,20) is blocked, labelled "units" — the unit is
      halfway.

## Night and desertion (§8.2)

13. **Desertion rounds up.** Campaign, you Dervish, AI Kitchener; play or
    End-phase (E) to turn 9 (the first NIGHT box), Dervish movement.
    - The desertion panel opens; its Roll table reads 1→2, 2→3, 3→5,
      4→6, 5→8, 6→9, 7→11, 8→12, 9→14, 10→15.
    - "End phase" stays locked until the desertion is confirmed.
    - The count is capped by the eligible units (the Khalifa, artillery,
      gunboats and forts are never offered).
    - After confirming, the units are gone and the score is unchanged.

## Victory

14. **Fall of Khartoum loss penalty** (§9.35). Spectate with both factions
    AI, Fall of Khartoum.
    - At the end the panel shows Gordon's fate, the Dervish losses and
      the level. If Gordon fell by turn 6, the losses shift the level
      toward the British, possibly across into a British win; if he
      survived, the British level is the turn-based one, never enlarged.
15. **West-bank decisive** (§9.14) is judged only at the end of the
    Campaign and is covered by
    `the_west_bank_decisive_needs_units_that_entered_it`; no manual check.

## Data and documents

16. **Rulebook viewer**, §2.3: the sample Anglo-Egyptian counters list
    "an unheaded leader counter (LORD KITCHENER — Sirdar —, 0·0·15)" and
    "Old Gunboat (GUNBOAT, 4·10/16)".
17. **Traceability report** — build it (see `CLAUDE.md`) and check the
    new chapters "Explanation of Combat Results", "Disrupted Units" and
    "Turn Record Track", each with its quoted clause and witness.

## Tooling

18. **Mutation gate on a local diff.**

    ```shell
    git diff --no-ext-diff main > /tmp/change.diff
    cargo run -p traceability-lsp --bin mutation-gate -- --in-diff /tmp/change.diff --list
    ```

    - Expect a non-empty plan (before: "0 mutant(s)" whenever git wrote
      `c/`/`w/` mnemonic prefixes). Without `--list` it runs; exit 0.
19. **Saved games** — `cargo test -p omdurman-app saved_games_still_load`
    passes with a record that holds only `StartGame` in `games/`.
20. **`run-omdurman` skill** (XWayland desktop with `xdotool` and
    ImageMagick). Follow `.claude/skills/run-omdurman/SKILL.md` from the
    top, without improvising:
    - `launch Lobby` opens the lobby; `shot lobby` writes
      `/tmp/omdurman-play/lobby.png`.
    - The lobby clicks start a Campaign against the AI Khalifa; `wait`
      returns on "phase=Movement active=AngloEgyptian"; `state` prints
      both sides.
    - With another window active, every input command exits 2 with
      "refusing input: active window is …".
    - `launch` survives the calling shell ending (it is detached).
    - Note any step that needed improvising, and fix the skill.

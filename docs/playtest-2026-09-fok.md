# Play-test notebook — Fall of Khartoum, two native windows (2026-09-26)

*A dated notebook, with follow-ups from 2026-09-27. Items still open are
tracked in [open-issues.md](open-issues.md); the tags in the lists below give
each item's status.*

Two native windows under XWayland, driven by mouse and keyboard (xdotool) and
read back from in-app screenshots.

- Window A: Dervish, and host.
- Window B: Anglo-Egyptian.
- Connection: the live fly.io signalling server.

Played from the lobby through setup and turns 1–8 to the victory screen:

- The early turns were full turns of movement, fire allocation, melee and advance.
- The later turns were mostly fast-forwarded with the new `E` hotkey. The
  Dervish mounted two palace assaults on turn 5.
- Result: British Decisive (Gordon survived; 23 Dervish losses).

Both peers were relaunched about 15 times mid-game. Each relaunch doubled as a
reconnect and replay test. After every relaunch the peers' rules states were
diffed and they agreed.

Test support that was added, and is inert unless enabled:

- `OMDURMAN_HEX_PROBE=<path>` writes every hex's screen position and the rules
  state twice a second.
- Creating `<path>.shot` takes an in-app screenshot. Screenshots taken from
  outside are stale under XWayland.

## Bugs found and fixed

### Rules engine (correctness)

1. **The CRT's numbered results also disrupted half the survivors.**
   - The printed CRT key says: "# = That many units in the target hex are eliminated".
   - Only `D` disrupts. The extra disruption made every volley and melee far
     harsher.
   - It also blocked the mandatory Dervish advance (§7.6), because disrupted
     units may not advance.
   - Fixed, and covered by test `numbered_result_eliminates_without_disrupting_survivors`.
2. **GORDON could be shot dead.**
   - Casualties were taken in list order, and Gordon listed first. So a lucky
     Dervish "Eliminate 1" on the palace ended the game on turn 2.
   - §9.346 says only a Dervish unit entering the palace kills him.
   - Now Anglo-Egyptian leaders never absorb CRT results. Gordon also survives
     the orphan-leader rule, since only entry into the palace kills him.
   - A lone leader no longer counts as "defenders remain", so the mandatory
     advance into the palace, which kills Gordon, still happens.
   - Test: `fire_never_kills_gordon`. The existing melee test only passed
     *because of* the old bug.
3. **Dead units could come back.**
   - An eliminated counter reappeared in the picker, and the engine accepted it
     as a fresh reinforcement. Board presence was the only check.
   - It could also be placed outside the owner's turn.
   - Now `GameState::eliminated` exists, `RuleError::UnitEliminated` refuses
     re-entry, and the picker hides eliminated counters and counts them against
     the FoK caps.
   - Non-setup placement is gated on the owner's turn and on the pause.
   - Test: `eliminated_unit_cannot_reenter_play`.
4. **Fire at a hex that was already targeted was refused.**
   - Firing at an already-targeted hex opened a second attack, which the engine
     refused (§6.14: a hex is fired at once per phase).
   - The app's builder required all firers to stand in one hex, so fire from
     several hexes could never combine.
   - New `combine_fire_attacks` merges the new group into the existing attack.
     "Already allocated" is now checked per unit.
   - Test: `fire_from_two_hexes_combines_into_one_attack`.

### Networking

5. **Relaunching the host process lost the game.**
   - The relaunched process re-won the election with an empty record, and
     nothing asked the guests for theirs.
   - Now a guest offers its record to a host that has just (re)connected. A host
     that has applied nothing installs it.
   - Verified live many times.
6. **A lost first `PlayerInfo` left the game falsely paused for the whole session.**
   - Messages arriving before the peer entity existed were dropped.
   - They are now deferred to the next frame, and peer-entity sync runs after the
     socket.
7. **Every submission was retransmitted in the frame it was sent.**
   - The retransmit clock kept counting while idle, doubling traffic.
   - Test: `fresh_submission_is_not_resent_in_its_own_frame`.

### App and UI

8. **Group moves were truncated.**
   - An 8-hex stack move moved one hex.
   - The commit charged units against their post-plot MP instead of their budget
     at selection.
9. **The plotted path ignored walls.** A path could be plotted through Khartoum's
   wall; the engine refused it only on confirm.
10. **Tap-to-click placement did nothing.**
    - A press and release in one frame did nothing: touchpads, fast clicks.
    - A refused placement was silent and dropped the unit. Now an "Order Refused"
      slip explains why and the unit stays in hand.
11. **The camera could be lost.**
    - Panning lost the board for good, with no fit command.
    - Now: focus clamped to the board, `Home` fits the board, and the board
      auto-fits on load, on entering the game and on window resize.
12. **Switching the firing unit needed Esc first** in the fire and melee phases.
13. **The status line was hidden under the sidebar.**
    - The status line sat under the sidebar, and the hex readout under the charts
      tab.
    - The telegram card covered the status line and the fire tray's Resolve button.
14. **Out-of-range and other fire refusals were silent.** Only LOS was explained.
15. **Internal IDs leaked into player-facing text.**
    - Turn summaries and telegrams used internal IDs such as "MulazminII_5_1" and
      raw coordinates.
    - `Player` displayed as "AngloEgyptian".
    - The gazette showed "FallOfKhartoum" and "DervishDecisive", and was dated
      "September 1898" for an 1885 battle.
    - The telegram model was told it was at Omdurman in 1898. It invented Kitchener
      and the 21st Lancers, and the prompt did not forbid inventing events.
16. **Too many slips per kill.**
    - Every kill produced two slips: a Casualty Report plus a "Victory Points
      (§9.14)" slip.
    - In FoK there are no VPs at all.
17. **No End Phase hotkey.** `E` now ends the phase, is listed in Keys, and is shown
    on the button.

## Open: bugs and questionable rules

- *(fixed: 75ff4ab)* **Road movement (unverified; the printed TEC is not
  transcribed).** A hex costs 1 MP if *any* road touches it, whatever direction
  the unit enters from. Entering a building hex costs 1 MP this way.
- *(partly fixed: 75ff4ab removed the fort-wall rings; the pocket north of the
  Blue Nile remains and gets no warning → open-issues.md)* **FoK deployment
  traps.**
  - (20,4) is enclosed by the Nile and the North Fort wall; units deployed there
    can never move.
  - The Hogali pocket (18,0)–(19,3) is cut off by the Blue Nile.
  - Both are legal under §9.322 but there is no warning; 20 Dervish units sat
    out the whole game.
- *(open → open-issues.md)* **§9.322 says the Dervish *enter* on turn 1.** The
  app deploys them onto edge hexes during Setup, so the first hex is free.
- *(fixed: 75ff4ab)* **Setup Actions in FoK are wrong.**
  - They show "§9.2 The Historical Scenario" twice.
  - They offer river mines and the chain, which are §10 optional Campaign rules
    that are not enabled.
  - The Dervish action list offers "Construct zariba" and "Load/disembark
    Friendlies".
- *(fixed: 9865f09)* **The setup banner is wrong.** It says "Dervish Turn (you)"
  during simultaneous deployment, but §9.321 has the British set up first.
- *(fixed: 75ff4ab)* **Game-over screen.**
  - It shows "Turn 8 · Movement · Dervish Turn (you)", as if turn 8 were unplayed.
  - The actions list still offers moves and End phase.
- *(fixed: 75ff4ab)* **The gazette contradicts itself.** The LLM wrote "Gordon
  saved" and then "Gordon fell". The newspaper prompt does not state whether
  Gordon lived.
- *(open → open-issues.md)* **The Melee card says "Defenders may retreat"** even
  when no defender can (infantry against an infantry melee).
- *(open → open-issues.md)* **Advance slip wording.**
  - After a Dervish melee the slip says "may advance". §7.6 says MUST, and the
    engine does advance automatically.
  - End Phase (or `E`) with an A-E advance-after-combat still possible gives no
    reminder.
- *(fixed: 75ff4ab)* **Night Maxim/Howitzer sub-phase.** At night in FoK the A-E
  have no Maxims and no howitzers, but must still click through this empty
  sub-phase. This happens twice per turn.
- *(open → open-issues.md)* **Kani.** `eliminate_unit` now pushes to
  `eliminated`. The river-mine and elimination harnesses have not been
  re-verified.

## Redundant UI

- *(not re-checked)* The Unit list lists "Kehena (1x)" and "Degheim (1x)" once
  per counter instead of aggregating, and itemises every A-E battalion. It gets
  long.
- *(eased: 0963c1b, one slip per kill; not re-checked)* Fire-resolution slips
  stack up. Before the fix, 8 slips for one volley covered the tray.
- *(open → open-issues.md)* "Review allocations (0 pending)" is shown in phases
  where nothing can be allocated, such as setup and movement.
- *(not re-checked)* In the lobby, the AI rows show "(claimed by a player)"
  twice.

## Missing UI

- *(not a bug: the overlay exists; cheapest-first since 75ff4ab)* No
  reachable-hex highlight when a unit is selected.
- *(open → open-issues.md)* "No legal route" when clicking an unreachable hex is
  only logged.
- *(probably open → open-issues.md)* No zoom-to-cursor: the wheel zooms around
  the centre of the screen.
- *(not re-checked)* Random player colours clash; both players got yellow in one
  session.
- *(probably open → open-issues.md)* Names are re-randomised on relaunch even
  though the player key persists.
- *(open → open-issues.md)* No "advance available" hint at End Phase.

## Good UI

- The fire preview card shows range band, night halving, the FoK table rule
  (§9.343), LOS, per-firer factors, the net modifier, the CRT row and the odds.
- Fire and melee result cards give rolls, modifiers, results, losses and rule
  links.
- Click-to-route (new): one click plots the cheapest legal path, with per-hex
  costs and Confirm/Undo/Cancel.
- The pause card ("Waiting for X — claimable in 57s") and silent seat reclaim on
  relaunch work.
- Precise refusal slips, for example "illegal stack … four-unit limit [§5.51]".
- Game Control for FoK shows Gordon's status, the Dervish loss penalty
  thresholds, the projected result and the turn track.
- The London Gazette end screen is atmospheric, and the day/night board tint
  reads well.

## Hard for me (and likely for players)

- Finding buttons that move: the sidebar shifts when the picker empties and the
  melee card moves with the banner.
- Knowing *why* nothing happened. Most of the silent refusals are now explained;
  route planning still is not.
- Deployment choices whose consequences only show turns later (the dead-end
  pockets).
- Right mouse means both pan (drag) and cancel (click).

## Follow-up (2026-09-27): Terrain Effects Chart, dead ends, open items

### Terrain Effects Chart

The chart was transcribed from `Manual/Elements/TerrainEffectsChart.jpg` into
`tables/terrain_effects_chart.ron`. A test checks the engine against it cell by
cell.

**Fixed:**
- **Road.** The "Road: 1" rate now applies only to a step *along* a road link. Any
  hex a road touched used to cost 1, so every road-side building was a 1 MP hex.
- **Hexside movement costs.** Khor +5, Crest +1 and City Wall "+1: may only cross
  at gate or breech" were all missing. A breach costs like a gate.
- **Khor melee.** "May not melee across" was missing; `blocks_melee` now includes
  the khor hexsides.
- **Hexside fire modifiers.** Crest −1 and City Wall −4 were missing. They now
  apply when a unit's line of fire enters the target hex across the hexside. When
  a combined attack comes in over different hexsides, the most protective one
  counts. Howitzer shells ignore it.
- **One step-cost function.** The engine, plot, preview, tooltip, path labels and
  bot now share a single rule. There were four slightly different copies before;
  the path-label copy looked up roads on the wrong hex.
- **Reachable-hex overlay.** It exists, so my earlier "missing" note was wrong. But
  it was a first-visit breadth-first search, which under-reports range with mixed
  costs. It now searches cheapest-first.

### Dead ends and the forts

- **One real pocket, as the manual implies.** Hogali, the North Fort and (20,4)
  lie north of the Blue Nile, and §5.22 bars land units from the Nile. Units
  entering there (legal under §9.322, "any hexes on the south or east edge") can
  never reach Khartoum; they can only garrison the North Fort, whose guns the
  Dervish may fire (§9.344).
- **Data bug that made it look like more.** All three FoK forts (Makran, Buri,
  North Fort) were ringed by 18 *city-wall* hexsides. That sealed any garrison in
  for good. It also blocked the melee and artillery fire that §6.54 allows against
  forts, and cut (20,4) off from the North Fort.
  - The rings are removed.
  - Test: `fok_forts_are_not_walled_in`.

### Directional ZOC (§5.44), fixed

The directional cases were documented as "left to the caller", but no caller
implemented them. ZOC now:
- does not extend into huts or buildings, but does extend out of them;
- does not extend into forts;
- crosses a wall or gate only outward from the walled city.

`GameState::zoc_extends` is the single rule; the app's overlay uses it. Most of
Khartoum is Building terrain, so before this fix ZOC stopped Dervish units
throughout the city.

In FoK, the engine's "walled city" is the 17-hex building block. It is not the
area behind the southern rampart. So the rampart still blocks ZOC both ways there.
Defining "inside" for that wall remains open.

### §9.322 (confirmed)

Text: "Dervish player moves first: enters turn one through any hexes on the south or
east edge of the map."

The app places the Dervish on those edge hexes during Setup instead. Gameplay
effects:
- **A free first hex.** Entering would cost the edge hex's MP. The entry edge is
  25 clear hexes (1 MP), plus two hut hexes and one building hex (3 MP): (18,1),
  (22,9) and (19,3).
- **Information order.** §9.321 has the British set up *first*. Deployment is
  simultaneous in the app, so the Anglo-Egyptian player can watch the Dervish mass
  and adjust the garrison. The rules do not allow that.
- **Stacking.** Setup forces legal four-unit stacks on the edge hexes. Entering
  units only pass through them.

### Also fixed

- **Setup Actions list.**
  - It now cites the scenario's own set-up rules: §9.11, §9.21 or §9.32.
  - River mines and the chain appear only when those optional rules are enabled.
  - Zariba building and the Friendlies' crossing appear only on the Campaign
    Anglo-Egyptian turn.
- **Game-over screen.**
  - The banner and Game Control read "Turn N · (final) · Game over — <result>".
  - The Actions list offers nothing.
  - Every result has a readable label, including the Campaign and Historical ones.
  - The newspaper prompt now states Gordon's fate as a fact.
- **FoK Maxim/Howitzer sub-phase removed.** There are no Maxims or named gunboats
  in the FoK order of battle (§9.321). The §6.42 advance "bridge" is disabled with
  it, so windows don't leak into Melee.
- **Status line.** It could still render under the rail: its inset ran before the
  layout ledger was filled in the frame. It now runs in `Last`.

### New finding

- In offline mode (`OMDURMAN_OFFLINE`) the local player never appears in the lobby
  roster, so Start stays disabled unless both sides are AI.

### Forts, checked against the manual

- **No fort walls exist in the rules.**
  - §6.54: "There is no additional movement point cost to enter or leave a
    friendly fort."
  - Forts may be destroyed by infantry melee (§6.54b / §7.6), but §7.2 forbids
    melee across a wall hexside. A walled ring would contradict that.
  - §5.44: ZOC extends *out of* a fort.
  - So removing the rings conforms to the rules.
- **Artillery never breaches a fort.** §6.63 limits breaching to "a wall hexside
  of Khartoum or the walled city of Omdurman". Artillery instead destroys the fort
  itself on a result of 2+ (§6.62).
- **The rings had been hiding a rule gap.** §6.54: "Players may not occupy an
  enemy fort". The engine only knew forts as counters, so the rings were what kept
  Dervish units out of Makran and Buri.
  - Makran and Buri are now the printed British forts (§9.321;
    `GameState::printed_fort_owner`) for every occupy/advance/retreat check. The
    North Fort remains its Dervish fort counter (§9.344).
  - Test: `printed_fok_forts_are_british_forts`.
- **Still open.** The printed "4-1-0 −3" factors of Makran and Buri are not
  modelled: their guns can't fire and they can't be destroyed. Their hex terrain
  gives the −3.

### Wall breach casualty (§6.63), fixed

This was found in an AI-vs-AI log.
- **Who can be the casualty.** The breach victim was any enemy unit *next to*
  either hex of the breached side. It could have been GORDON, and the loss was
  labelled "demolition (§6.53)".
- **Fix.** The victim now stands in one of the two hexes, is never an
  Anglo-Egyptian leader, and gets the cause "caught in the wall breach (§6.63)".
- **Royal Engineers.** They must now stand at the wall (in one of its hexes) to
  breach it.
- Test: `breach_casualty_stands_at_the_wall`.

### AI note

In the AI-vs-AI FoK game, the AI's Anglo-Egyptian deployment left GORDON alone in
the palace and marched the garrison south-east. A Dervish unit walked into the
palace on turn 2. That is legal (§9.346), just weak play.

## Follow-up 2: sequential set-up and the telegram overlay

### Set-up order is enforced

- §9.111: "Dervish player sets up first". §9.211 and §9.321: the Anglo-Egyptian
  or British player sets up first.
- The second side may deploy only once the first has confirmed Ready. A confirmed
  deployment is final. `RuleError::SetupOrder` explains a refusal.
- The scenario's own fixed counters (GORDON, the North Fort, the Historical
  leaders) are exempt. They now live in the rules crate (`fixed_placements`).
- The banner shows who is deploying ("Setup — British Deploys First"). The waiting
  side's Ready is disabled, with the reason shown.
- An unbound single-seat session gets "<side> deployment done", then "Begin
  battle".
- The bot deploys side by side in the same order.

### The telegram is now a centered modal

- It appears when a game turn completes, over a dimmed board, and play waits
  until it is dismissed: a click anywhere, Continue, Enter, Space or Esc. The
  focused button holds back the hotkeys, including `E`.
- Only the latest turn's telegram is shown. A replay backlog, or the game over,
  skips it.
- Without a flavour model it now reports the turn's actual events instead of "the
  situation develops".

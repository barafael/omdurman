# UI States

Every visual state of the game client, with the actions available to the player
and the feedback the UI gives. States are listed hierarchically: app-level
`AppState`, mode-level `AppMode`, then `UiPhaseState` for in-game phases.

---

## 1. AppState::Splash — Loading screen

**When:** On first launch while board assets load.

**Actions:**
- None (loading is automatic).

**Visual feedback:**
- Centred loading bar / spinner.
- Once assets finish, auto-transitions to `AppMode::Menu`.

---

## 2. AppMode::Menu — Main menu hub

**When:** On first load (after splash), or when the player presses M during a
game or editor session.

**Actions:**
- **"Host Game"** — create a new room (becomes host).
- **"Join Game"** — enter a room code to join an existing room.
- **"Editor"** — enter the map/annotation editor (last-used board).
- **"Review saved game"** — browse saved game files, load one as spectator.

**Visual feedback:**
- Full-screen centred card with the game title and the four buttons.
- No map, no hex grid, no units. Background is dark / decorative.
- Room-code entry sub-dialog on "Join".

---

## 3. AppState::Lobby / AppMode::Lobby — Pre-game lobby

**When:** After connecting to a room, before the game starts.

**Actions:**
- **Pick a faction** (Anglo-Egyptian or Dervish) or **toggle Spectate**.
- **Pick scenario** (host only): Campaign, Historical, Fall of Khartoum.
- **Edit display name** in the local-player settings field.
- **Start Game** (host only, once both factions are claimed by non-spectating peers).
- **Back** — disconnect and return to menu.
- **Saved Games tab** — browse, load, or delete replay files.

**Visual feedback:**
- Roster of connected peers with their colour, name, and chosen faction.
- Real-time faction-pick previews from other peers (ephemeral).
- Scenario dropdown (host only).
- "Start" button disabled until both factions are taken.
- Live cursor overlay of other peers' mouse positions on the (empty) map plane.
- In the Saved Games tab: scrollable file list with timestamps, click to load.

---

## 4. UiPhaseState::Setup — Deployment

**When:** Scenario started; each player places their units.

**Actions:**
- **Place a unit:** click a unit in the sidebar unit picker, then click a hex
  in your deployment zone.
- **Remove a placed unit:** right-click it or drag it back to the picker.
- **Select optional rules** (Dervish host only, campaign): toggle River Mines
  or River Chain, then click hexes to place them. Placement is visible only to
  the Dervish player (§10.11, §10.21 — secretly recorded).
- Zariba hexsides are pre-printed on the map for the Historical scenario
  (§9.23). The Anglo-Egyptian player deploys inside the Zariba's 13 hexes.
- **Confirm Ready** — locks your deployment. Game starts when both players ready.

**Visual feedback:**
- Map shows each player's deployment zone highlighted (tinted overlay).
- Unit picker sidebar: scrollable grid of unit sprites grouped by tribe/brigade,
  with remaining unplaced counts.
- As a unit is placed, the counter sprite appears on the hex.
- Hexes in the deployment zone glow green when a unit is selected in the picker.
- Invalid hexes (wrong zone, blocking terrain) are red-highlighted on hover.
- "Ready" toggle button — once toggled, your units lock; an icon shows your
  opponent's ready status.
- Dervish-only: optional-rule checkboxes; mine/chain placement mode (visible only to Dervish player).
- "N" night indicator (FoK starts at night).

---

## 5. PhaseKind::Movement

**When:** Active player moves units.

### Visual baseline (both players):

- **Phase banner:** "Movement" with a check-mark sequence showing
  `[Mov] > Def > Off > Melee`.
- **Night badge** (if night): "Movement (Night) — AE movement halved".
- **Active-player indicator:** whose turn it is (colour-coded).
- **Turn counter:** "Game Turn 3 — 8:00 AM".

### Actions:

**Select a unit:** left-click on a friendly unit counter.

- Selected unit highlights with a pulsing ring.
- Valid destination hexes overlay with a dot pattern.
- Invalid hexes (out of MP, blocked terrain, enemy ZOC interior, wall hexside,
  off-map, stacking full) show a red X on hover.
- Right-click clears selection.

**Move the selected unit along a path:** click a valid destination hex, or
click intermediate hexes to build a path manually.

- Path appears as a trail of coloured dots or arrows.
- MP cost displayed next to the cursor / on the unit card.
- ZOC-entry hexes pulse red (unit must stop there).
- Remaining MP shown on the unit card.
- Terrain cost labelled per hex on hover ("Rough: 2 MP").

**End Movement Phase:** click "End Phase" button.

### Additional actions — Anglo-Egyptian only:

- **Construct Zariba:** with an infantry unit selected that begins AND ends the
  phase adjacent to a zariba hexside, click "Build Zariba" in the action guide.
  - A "Building" marker appears on the unit's counter (may not fire/melee this turn).
  - Zariba hexside sprite animates from incomplete to complete.
- **Transport Friendlies:** select a gunboat and a friendly unit on an adjacent
  hex, click "Load" in the action guide (turn 1). On turn 2, move the gunboat.
  On turn 3, disembark the Friendlies on a Nile-adjacent hex.
- **Royal Engineers demolition:** with an RE unit adjacent to a fort or wall
  hexside and not firing/meleeing, the action guide shows "Demolish". Click it.
  - If RE survives undisrupted to end of turn: fort sprite is removed or a
    Breach marker appears on the wall hexside.

### Additional actions — Dervish only:

- **Desertion roll** (§8.2, campaign, first night turn only, once per game):
  button appears in the action guide during the movement phase. Click to roll
  1d10; 1½× the result (round down) = number of deserting units. The Dervish
  player chooses which units to remove (Khalifa, gunboats, artillery, and forts
  may not be chosen). No VP awarded for deserters.

---

## 6. PhaseKind::DefensiveFire(FireSubKind::Direct) — Defensive direct fire

**When:** Non-active player fires at the active player's units.

### Visual baseline:

- Phase banner: "Defensive Fire — Direct".
- Sequence indicator: `✓ Mov > [Def] > Off > Melee`.
- **Firing-player indicator:** "[Dervish] fires defensively" (the non-active player).
- **Night badge** (if night): "Night — all ranges halved (§8.1)".

### Actions:

1. **Select a firing unit** (any unit of the *firing player* that is in range,
   subject to artillery-only restrictions below).
   - In-range enemies in LOS highlight in orange. Out-of-range / no-LOS targets
     are dimmed.
   - A line-of-sight ray is drawn to the hovered target hex (blocks shown in
     brown if blocked).
2. **Select a target hex** (must contain enemy units in range and LOS).
    - A preview panel opens: fire factors, range band multiplier, total effective
      FF, terrain DRM, predicted CRT band, and brigade-integrity bonus (+1) if
      applicable (§5.54).
    - Select which friendly firing units to commit (per §6.14, players may
      combine fire voluntarily). Units committed to the same target hex are
      combined into a single attack. The preview shows the combined total.
3. **Resolve the attack:** click the "Fire" button in the preview panel.
   - Result toast: "Eliminate 2" / "Disrupt" / "No effect".
   - Eliminated counters slide off; disrupted counters show a "D" marker.
4. **Repeat** for any other eligible firing unit/hex combination.
5. **End phase:** click "End Phase" button.

### Note
- Howitzer fire *not* available here (that's the Maxim/Howitzer sub-phase).
- No advance after combat in defensive fire (§6.7).
- **Artillery-only targets (§6.61–6.63):** Only artillery-class units are
  eligible to target gunboats, forts, or wall hexsides. Non-artillery units
  are filtered from the firing-unit list when these targets are selected.
- **Friendlies (§6.52):** The Friendlies brigade fires on the Dervish Range
  Effects Table (Rifles line) and melees with the Dervish +2 modifier. Their
  preview calculations use these values.

---

## 7. PhaseKind::DefensiveFire(FireSubKind::MaximHowitzer) — Defensive Maxim/Howitzer

**When:** Anglo-Egyptian only (their Maxims fire a second time defensively, or
howitzers fire).

### Visual baseline:

- Phase banner: "Defensive Fire — Maxim/Howitzer".
- Firing-player indicator: "Anglo-Egyptian fires defensively (Maxim/Howitzer)".

### Actions — Maxims:

- Select any Maxim unit that hasn't fired yet this phase.
- Same target-selection and resolution as Direct Fire.
- Maxims that fired in Direct Fire may fire again; those that skipped Direct
  may still fire once here (§6.42).

### Actions — Howitzers (named gunboats only, day only (§8.1), range 4–10):

1. Select a named gunboat. Valid target hexes (4–10 hexes away, any terrain)
   are highlighted in purple (LOS ignored).
2. Click a target hex. Preview panel shows: fire factor, no terrain or LOS mods.
3. Click "Fire Howitzer". Two dice are rolled sequentially:
   - **CRT die:** determines the result (eliminate/disrupt/—).
   - **Impact die:** 7–10 = target hex hit; 1–6 = scatter one hex in the
     direction per the scattergram (shown as an arrow overlay).
   - The CRT result is applied to the actual impact hex (even if friendly).
   - If the howitzer hits the intended hex, Maxim fire from the same/same hex
     may be combined.
4. Result toast shows both rolls and the final impact hex.

---

## 8. PhaseKind::OffensiveFire(FireSubKind::Direct) — Offensive direct fire

**When:** Active player fires offensively.

### Visual baseline:

- Phase banner: "Offensive Fire — Direct".
- Sequence indicator: `✓ Mov > ✓ Def > [Off] > Melee`.
- **Firing-player indicator:** "[Anglo-Egyptian] fires offensively".
- **Night badge** (if night): "Night — all ranges halved (§8.1)".

### Actions:

Same as Defensive direct fire (select friendly firing units → select target →
resolve), **plus**:

- **Artillery-only targets (§6.61–6.63):** Same restrictions as defensive fire.
- **Friendlies (§6.52):** Same special combat rules as defensive fire.

- **Advance after combat (§6.82):** If the target hex is vacated (all units
  eliminated), adjacent participating units that are not artillery and can
  cross the hexside may advance. A toast + button appears: "Advance into
  [hex]?"  Click to move the unit. If multiple units participated, each may
  advance individually (up to stacking limit).
  - Cannot advance across wall hexsides (except gate/breach), or across khors.
  - Artillery cannot advance.

---

## 9. PhaseKind::OffensiveFire(FireSubKind::MaximHowitzer) — Offensive Maxim/Howitzer

**When:** Anglo-Egyptian active player's second fire sub-phase.

### Visual baseline:

- Phase banner: "Offensive Fire — Maxim/Howitzer".
- Sequence: `✓ Mov > ✓ Def > ✓ Off(Direct) > [Off(Maxim)]`
- **Night badge** (if night): "Night — howitzer fire prohibited (§8.1)".

### Actions:

Identical to Defensive Maxim/Howitzer (Maxim second fire + howitzer scatter),
with the same advance-after-combat option as Offensive Direct Fire.

Dervish players **skip** this sub-phase (no Maxims or howitzers).

---

## 10. PhaseKind::Melee — Melee combat

**When:** Both players resolve melee attacks simultaneously.

### Visual baseline:

- Phase banner: "Melee".
- Sequence: `✓ Mov > ✓ Def > ✓ Off > [Melee]`.

### Actions:

**Declare a melee attack:**

1. **Select an attacking unit** (infantry, cavalry, camel, Dervish leaders only;
   A-E leaders may not melee attack) that is **adjacent** to an enemy unit.
2. Adjacent enemy hexes highlight in red. Wall hexsides show a blocked icon.
   Gate and breach hexsides are passable (highlighted).
3. **Click the target hex.** Both sides' melee factors are summed and displayed
   in the preview panel. Dervish melee DRM +2, A-E +1 shown.
4. **Click "Resolve Melee".** Both sides' dice animate simultaneously.
   - Result toasts appear for both attacker and defender.
   - Losses are deducted from meleeing units first (§7.7).
    - If either side has cavalry/camel units defending, those may **retreat**
      before resolution: a "Retreat 2 hexes?" prompt appears (§7.5).
    - **Forts (§6.54):** A defending unit inside a fort applies −3 to the
      attacker's die roll (shown in the modifier breakdown). Forts may not
      melee attack. Players may not occupy an enemy fort nor advance after
      combat into one.

**After melee resolution:**

- **Advance after melee (§7.6):** If the defender's hex was vacated:
  - Dervish **must** advance (auto-prompt; click destination unit).
  - Anglo-Egyptian **may** advance (optional button).
  - Only attacking units that survived may advance. Stacking limit enforced.

---

## 11. UiPhaseState::GameOver — Victory / Defeat screen

**When:** Scenario end condition met.

**Actions:**
- **View final map** (read-only, no interaction).
- **View casualty summary** — clickable list of eliminated units per side.
- **Save replay** — writes the game record to disk.
- **Return to menu** — goes to `AppMode::Menu`.

**Visual feedback:**
- Overlay with victory level, VP tally, and a breakdown of how each side scored.
- FoK: shows GORDON death turn + Dervish loss penalties.
- "Victory" vs "Defeat" banner with player colour.
- Timeline scrubber disabled (game is over, not spectating).

---

## 12. AppState::Spectating — Replay / review mode

**When:** A saved game is loaded, or joining as spectator mid-game.

**Actions:**
- **Timeline scrubber** at the bottom: drag to any past event; state is rebuilt
  from the event record up to that point.
- **Play / Pause** — auto-advance through the event log at a configurable speed.
- **Skip to next turn** button.
- **Exit review** — back to menu (or rejoin live if connected).
- No game actions (unit movement, fire, etc.) — view-only.

**Visual feedback:**
- Map state reflects whatever event the timeline cursor is at.
- Counter positions, disruption markers, phase label, turn counter all update as
  you scrub.
- Event list panel on the right: scrollable sequence of every `GameEvent` with
  timestamps, expandable for details (e.g. "Dervish Movement: unit X moved from
  A to B, cost 3 MP").

---

## 13. AppMode::Editor — Map / annotation editor

**When:** Editor mode selected from the menu.

### Visual baseline:

- Map plane shown (the chosen board: Campaign or Fall of Khartoum).
- Hex grid overlay if the Overlay tab is calibrated.
- Horizontal tab bar at the top with 8 tabs.

### Editor tabs:

#### 13a. Overlay Tab
- **Adjust hex-grid alignment** over the map image: drag rotation, scale, offset.
- All calibration happens via numeric fields and drag handles.
- **Visual feedback:** grid changes in real time; hex coordinates shown as labels.

#### 13b. Terrain Tab
- **Paint terrain types** on hexes (Clear, Building, Rough, Sand, Nile, etc.).
- **Paint Nile flow direction** per hex.
- **Set hex names** from the rulebook.
- **Place roads** between hexes.
- **Visual feedback:** terrain colour-coded per hex; click to assign; a legend shows current palette.

#### 13c. Hexside Tab
- **Paint hexside features:** Wall, Gate, Breach, Khor, Zariba, NilEdge, etc.
- Click a hexside edge to toggle a feature on/off.
- **Visual feedback:** edges highlight on hover; placed features appear as coloured lines.

#### 13d. Timing Tab (Campaign board only)
- **Place turn-track bounding-box** — drag a rectangle over the turn track area of the map image.
- Used to auto-recognise the printed turn track.

#### 13e. Unit Sheet Tab
- **Adjust sprite-cutting grid** — the column/row divisions on the unit sprite sheet.
- Drag lines to align with counter borders.

#### 13f. Sprites Tab
- **Browse cut counters** from the sprite sheet.
- **Assign metadata** (UnitId, section name, brigade) to each sprite slot.
- **Visual feedback:** grid of all cut sprites; clicking one opens its metadata editor panel.

#### 13g. Event Viewer Tab
- **Browse recorded game events** (read-only). Same as spectator event log.
- Filter by event type, turn, or player.

#### 13h. Charts Tab
- **Reference charts** — the Combat Results Table, Range Effects Tables, Terrain
  Effects Chart, Howitzer Scattergram displayed as images.
- Read-only within the editor; pan/zoom only.

---

## Cross-cutting: always-visible UI elements

Throughout all non-menu, non-splash states:

- **Top toolbar:** mode picker (Game / Editor) and current phase/time display.
- **Cursor overlay:** other peers' cursor positions (coloured dots with names).
- **Chat panel** (collapsible): text messages broadcast to all peers.
- **Connection status:** indicator dot (green/ yellow/ red) and room code.
- **Map controls:** zoom (scroll wheel), pan (middle-click drag), reset view button.
- **Night indicator:** crescent-moon icon and "Night: ranges halved" note during
  night turns, visible in all phases.

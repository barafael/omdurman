# Playtest 2026-10 — two-instance UI issues (Fall of Khartoum + Campaign)

Dated record of issues found during a supervised two-window play session on
2026-10-04/05. One session, two native instances driven by real mouse/keyboard
input (`xdotool` via the run-omdurman driver), tiled side by side on a KDE
Wayland desktop, networked peer-to-peer through the deployed matchbox server.

- **Game 1** — *Fall of Khartoum*, both sides human: room `omd-review-fok1`,
  window A "Ember Panther" (Anglo-Egyptian, slot 7), window B "Quick Leopard"
  (Dervish, host, slot 8). Played to turn 3 (A-E movement), full deployment of
  both armies (19 + 49 units), night turns 1–2, defensive fire, one melee.
- **Game 2** — *Campaign*, A-E human vs AI Dervish: room `omd-camp-game2`,
  window A host ("Nimble Dingo", slot 7), window B spectator ("Water
  Panther"). Played to turn 3 (A-E movement); AI deployed 38 initial units and
  received waves to 106; A-E placed 3 leaders + 3 battalions.

Open items here could be folded into `open-issues.md` by a maintainer; this
file stays as the dated record. Everything below was observed through the UI
by real input; where the engine's behaviour is only inferred (not verified in
code), the text says so.

Severity legend: **High** = cost minutes of playtime or blocks a flow;
**Medium** = confusion, wrong mental model, or repeated avoidable friction;
**Low** = cosmetic or single-instance annoyance.

---

## A. Deployment & reinforcement UI

### A1. The deployment rail reflows under the cursor — High

**Component:** set-up / reinforcement rail (`omdurman-app`, lobby of the board
UI; the sidebar listing groups of counters to place).

**Description.** Every placement is an effect that must round-trip (apply on
echo). When the echo applies, the rail re-lays out: the placed chip's slot
empties, counts change, and in some cases rows shift position. A player (or a
driver) who locates a chip visually and then clicks is racing a moving target.
Empty groups keep their full slot grid (see A5), so row *heights* stay, but the
re-anchoring after each echo was enough to invalidate positions captured a
second earlier.

**Observed incidents (game 1 unless noted).**

- Aiming for 3E Brigade chip #1 at rail (30, 665) picked nothing
  ("Pick a counter above, then a highlighted hex" — no chip in hand); two
  clicks in a row failed at the same verified-looking spot.
- A click intended for 3E actually picked a 1E chip and placed it at
  (12, 6) — the rows had shifted between the screenshot and the click. The
  placed unit was only discovered later in the probe state
  (`Kitchener_5_0 … brigade 1 … at 12 6`).
- The Artillery chip pick failed at both (30, 1155) and (30, 860) across two
  strip-screenshots taken ~30 s apart; the row heights differed each time.
- One aimed artillery click instead expanded the "Forces on the map" tree —
  the click landed on the header of the collapsed list that sits below the
  groups because the rail had grown.
- Dervish side: the Kehena chip pick failed at (25, 420) and (25, 422); the
  true position was (25, 490) — verified by a 1:1 pixel strip only.
- Campaign: after Kitchener and Hunter were placed, Gatacre's chip moved from
  slot 2 to slot 1; a click at the old position hit an empty slot. Later, the
  last 1B chip moved between (92, 462) and (85, 472) between two reads.

**Impact.** This was the single largest source of failed input in the whole
session — dozens of mis-clicks across two deployments (~70 placed units). It
also makes scripted/assisted play brittle.

**Suggestion.** Pin each group to a fixed anchor (sticky headers) and animate
re-layout, or freeze rail layout for a grace window (e.g. 750 ms) after an
echo applies. Alternatively keep the "in hand" chip rendered at the pointer
rather than at its slot, so the click target for *placing* never depends on
the rail at all — only the initial pick would.

### A2. "Then pick the next of the group" (auto-next) is inconsistent — Medium

**Description.** With the auto-next checkbox ticked, most placements did hand
the next chip of the same group to the player (Friendlies ×4, 3E ×3, Degheim
×5, Mulazmin ×32 chained cleanly). But several chains broke for no reason
visible to the player:

- 2E Brigade: after placing #1 at (10, 6), no chip was in hand ("Pick a
  counter above…" in Next step); the second hex click of the chain was
  silently wasted.
- Campaign 1B Brigade: after #2 placed at (27, 0), #3 was not in hand; the
  next rail click picked nothing.

The distinguishing condition is not visible from the player's seat (suspect:
the chain only fires when the placement echo applies before the next rail
interaction, or only when the *first* slot of the group is the one that
emptied — unverified).

**Expected.** Auto-next is deterministic: after every successful placement
from a group with remaining chips, the next chip is in hand, and the Next step
line says so ("Next: 1B Second Btn — click a highlighted hex").

### A3. Illegal target clicks bounce silently — High

**Description.** With a counter in hand, a left click on a hex that is not a
legal placement does nothing visible: no message, no flash, no shake. The
counter sometimes returns to the rail and sometimes stays in hand — which of
the two happened is not signalled either. The only legality feedback in the
whole UI is the thin green/red hex outline while hovering, and the ghost
preview of the held chip on legal hexes.

**Observed refusals (all silent).**

- FoK A-E: 2E #2 rejected at (13, 7) and (8, 5) — hexes outside the walled
  city's building-hex set. Legal set discovered only by hovering around with
  the counter in hand.
- FoK Dervish: Degheim cavalry rejected at (5, 6), (6, 7), (7, 7), (7, 8)
  and (8, 9). The legal zone turned out to be the south-or-east map edge of
  §9.322; the hover ring at (8, 9) finally showed red = illegal. Cost: ~10
  blind clicks plus a full-board ghost hunt.
- FoK Dervish artillery: rejected at nineteen south/east-edge hexes —
  including empty-looking ones — until (13, 15), a *free* south-edge hex,
  accepted it. The inferable rule (guns need an unoccupied hex, or their own
  stacking class) is stated nowhere; cf. A4.
- Campaign: the last 1B battalion was rejected at (27, 0) and (25, 0) — hexes
  holding a single unit each, while (24, 0) and (26, 0) had each accepted a
  second unit earlier in the same turn. From the UI alone it is impossible to
  tell whether the blocker is stacking, a leader-stacking class, a stale
  in-hand state, or something else. (FoK by contrast accepted 3 Mulazmin on
  (22, 15) — the stacking rule visibly differs per scenario and is never
  stated; cf. A4.)

**Impact.** Deployment of 49 Dervish units and the Campaign reinforcement
waves were slow, and every refusal cost a diagnosis loop of screenshot →
hover-probe → retry. A first-time player would plausibly conclude the game is
buggy rather than that they violated an unstated rule.

**Expected / suggestion.** The engine already produces typed errors for these
paths (`RuleError::OutsideDeploymentZone(hex)`, `OutsideEntranceArea`,
stacking validation in `effects/state/setup.rs`). Surface them in the same
"Field Telegram" style used for fire refusals ("Place refused — (8, 5) is
outside the Dervish entry area (§9.322)"). One consistent bounce behaviour
(suggest: keep the counter in hand, show the toast) would fix both halves of
the problem.

### A4. Stacking limits are scenario-dependent and never stated — High

**Description / observed.** FoK: (22, 15) accepted three Mulazmin (2 then a
3rd); Fort Buri held fort + battalion; (24, 0)/(26, 0) in the Campaign each
accepted two units — and then near-identical second-unit placements elsewhere
were refused (see A3). `open-issues.md` already records that §5.51's four-unit
limit has open questions. Whatever the engine enforces per scenario, the UI
never states the limit for the scenario at hand, and refusals carry no reason
(A3).

**Suggestion.** Print the effective stacking rule in the Next step panel
during set-up ("Stacking: up to 3 units per hex (§5.51)"), and include the
violated limit in the refusal toast.

### A5. Exhausted groups keep rendering empty dashed slots — Medium

**Description.** When a group's chips are all placed, the group header stays
with a grid of empty dashed boxes: Leaders (0), Forts (0), Friendlies (0),
2E/3E (0), and most dramatically Mulazmin (0) — eleven dashed chips in a 4-wide
grid. They look like faint interactive buttons (one of my mis-clicks landed in
an empty Leaders slot while aiming for the occupied middle slot — the real
chip had reflowed one slot over, cf. A1). In the FoK rail, Gordon — pre-placed
on the map at (13, 5) — still leaves a dashed slot in the Leaders row even
though he was never "yours to deploy".

**Suggestion.** Collapse exhausted groups to their header line
("Friendlies (0)"), or drop the group entirely once empty and at 0 count.
Never render a dashed slot for a fixed/pre-placed counter.

### A6. Campaign gunboats: quota says 3, rail shows 9 — Low

**Description.** The reinforcement header reads "Gunboats 0 of 3 · land units
0 of 12" while the Gunboats group renders nine chips (Abu Klea, Sarra, Sheika,
Faid, Zafir, El-Te…, plus three generic GUNBOAT chips). Presumably only three
may enter this turn (§9.113) and the rest are later waves; the over-quota
chips give no visual signal and would (presumably) refuse placement like A3.
FoK's deployment rail has no such quota text at all — the two scenarios render
the same concept differently.

**Suggestion.** Render only the chips the current wave allows; grey-badge the
deferred ones ("turn 4") rather than showing them as placeable.

### A7. Campaign reinforcement rail doesn't surface the turn-4 leader deadline — Low

**Description.** §9.113 requires Kitchener, Gatacre and Hunter in play by the
end of turn 4 (the engine even refuses to end the phase until then, per the
run-omdurman skill notes), but the Campaign rail shows only "Reinforcements —
Gunboats 0 of 3 · land units 0 of 12". The deadline is invisible unless the
player opens the rulebook. FoK's Ready gate, by contrast, warns in plain text
("Deploy your forces before confirming to ready.").

**Suggestion.** A rail line like "Leaders must enter by turn 4 (3 remaining)"
with the §9.113 link.

---

## B. Targeting & combat UI

### B1. Melee adjacency is invisible on the board — Medium

**Description / observed.** In FoK turn 2, fire from the Fort Buri stack at
(20, 9) to (21, 9) succeeded as "range 1 hex"; a melee was declarable from
(21, 10) against (20, 9); but a Mulazmin at (21, 8) reported "0 adjacent
target hex(es)" and a fire attempt from the same hex reported "target (20, 9)
out of range from (21, 8)". All three are consistent with hex-neighbour
geometry ((21, 9) and (21, 10) neighbour (20, 9); (21, 8) does not), so the
engine is presumably right — but the board offers no adjacency highlight. The
only feedback is a *count* in the sidebar ("0 adjacent target hex(es)"), and
the counter looks adjacent on the map.

**Related.** With the (21, 8) stack selected, the "Melee" action in the
Selected unit card was greyed with no reason. Units that had spent MP showed
"Move 9 MP (3 spent)" — is greying caused by the spent MP, the phase, or the
adjacency? Indistinguishable.

**Suggestion.** In the melee phase, outline legal target hexes on the board
when a stack is selected (deployment already outlines legal placement hexes —
reuse that). Make the fire refusal distinguish "not adjacent" from "out of
range", and print the greyed-out reason inline in the card.

### B2. Movement "out of reach" feedback exists only in the hover tooltip — Medium

**Description / observed.** Plotting a Kehena move from (12, 15) to (11, 12)
at night: the hover tooltip correctly said "(11, 12) · Clear — Move cost 1 MP
(§5.11…) … Out of reach; the cheaper MP, 8 left (§5.11…)" — the khor crossings
en route cost extra MP. But clicking the destination did nothing at all (no
banner, no telegram), and nothing on the board explains that the *path*
(khors) is what breaks the move rather than the destination.

**Related.** The Selected unit card prints the unmodified "Move 9 MP" while
the night rules halve it; only the sidebar's global "Night rules (§6)" block
explains. The card should show the effective allowance.

**Suggestion.** On an unreachable destination click, toast the hover text
("Out of reach: 8 MP needed, 4 available (night)"), and flash the first
blocking hex. Show effective MP in the unit card.

### B3. The Field Telegram swallows the first interaction after a transition — Medium

**Description / observed.** In FoK's Defensive Fire the telegram banner sat
top-right while I double-clicked the fort, clicked the target and pressed
Enter — all silently swallowed until `Return` dismissed it (confirmed by
before/after screenshots: same lobby-like frame, "…telegram (Enter to read
on)" in the bar). In the Campaign a telegram appears at each turn rollover and
keys are dead until it is read. The banner form is easy to miss; the read
state is not reflected anywhere else.

**Suggestion.** Either dim the board under the telegram (unmissable), or
buffer input for a second after it closes. The `ff` helper's
"press Return first" workaround in the skill doc exists precisely because of
this.

### B4. Two different staging/resolving metaphors for fire and melee — Low

**Description.** Fire uses a bottom tray with per-attack cards and "Resolve N
attacks (Enter)"; melee uses a floating card top-left ("Dervish melee on hex
(20, 9) — resolve when ready") with its own "Resolve Melee" button. Both work;
having two patterns for the same stage-then-resolve concept adds recall cost.

---

## C. Lobby & identity

### C1. Player names do not persist across restarts — Medium

**Description / observed.** The same player key (slot 7) presented as
"Ember Panther", then "Wandering Otter", then "Nimble Dingo" across three
launches. Seat reclaim works (the seat and the Verified badge came back), but
the identity shown to co-players is a stranger every relaunch. Web
`sessionStorage` presumably has the same hole across browser restarts.

**Suggestion.** Persist the generated/chosen name next to the key in
`player_key_store` and reuse it.

### C2. Guests can click scenario buttons the host will override — Low

**Description / observed.** The label says "The host chooses the scenario",
but the Campaign / Historical / Fall of Khartoum buttons stay enabled for a
guest — and the splash art even switches when clicked (the Dervish print
appeared for the guest after clicking Fall of Khartoum). The pick is
cosmetic; the host's choice won.

**Suggestion.** Disable the buttons for non-hosts with a tooltip; keep the
art change if it is considered a preview, but don't leave a control that
lies.

### C3. The Start Battle gate doesn't say which condition is unmet — Medium

**Description / observed.** Campaign: the Dervish faction splits into two AI
commands ("AI — the Mahdi (Dervish)", "AI — Khalifa (Dervish)"). After
ticking only the Mahdi, Start Battle stayed grey with the same generic text
it shows when *nothing* is chosen. It went green only after the Khalifa row
was ticked as well; separately, the guest (a connected player holding no
seat) also kept it grey until switched to Spectate. Three different blockers,
one message.

**Suggestion.** Enumerate the unmet conditions in the greyed state
("Dervish: the Khalifa command is unassigned · Water Panther is neither
seated nor spectating"). The predicate lives in
`all_players_ready_with_ai(roster, &effective_ai)` — it already knows.

Related cosmetic: after the AI ticks, the AI-held seat renders as a *player
row* ("AI · Khalifa … Dervish") whose checkbox-like swatch invites the same
click as the AI Commanders box above it — two adjacent click targets for
related concepts.

### C4. The crossed-swords glyph on Start Battle renders as "✕" — Low

**Description.** `\u{2694}` (⚔) has no coverage in the shipped font and
renders as a thin ✕-shaped box, reading as "close" or "disabled" on the one
button whose enabled state matters most (see also the disabled-state
confusion above).

**Suggestion.** Inline SVG icon or bundle a glyph fallback.

---

## D. Presentation & copy

### D1. The probe state line reports a live-looking game before any game exists — Medium (bit the documented tooling)

**Description / observed.** `probe.state`'s first line reads
`T turn=GameTurnIndex(1) phase=Setup active=Dervish` even in the lobby before
StartGame — it is the engine's default state. A driver `wait 'phase='` matched
instantly and I misdiagnosed "the game has started" twice; the log
(`game started via host StartGame scenario=…`) was the only reliable signal.

**Suggestion.** Prefix the state line with `no-game` until a record/StartGame
exists, or expose `started=true|false` as its own field.

### D2. Stale screenshot frames at mode transitions — Low (tooling)

**Description.** The in-app screenshot (`probe.png`) lagged the lobby→game
switch by at least a frame: shots taken after `phase=Setup` appeared still
showed the Lobby. Combined with D1 this produced two false "it didn't start"
diagnoses. A frame/sequence number alongside the PNG would let the driver
detect staleness.

### D3. Minor copy/cosmetic notes — Low

- Peer cursor labels ("Quick Leopard", "Water Panther") linger at their last
  position indefinitely; a spectator's parked cursor read as a unit on the
  city for several minutes. Fading idle cursors would help.
- Counter stat orders differ by unit type (gunboats "4-1/3s", infantry
  "9-5-8", leaders "0-0-1/5"); the Selected unit card decodes them well, but
  a one-line legend in the Keys/Charts panel would save squinting at the
  scanned counters.
- The Campaign AI command rows mix fiction registers: the Mahdi and the
  Khalifa both appear as separate Dervish commands (fine per the game's OOB),
but nothing tells the player they are two halves of one faction that must
*both* be handed over (cf. C3).

---

## E. Environment & tooling notes (not app bugs)

These cost real session time and belong in the record for the next driver
user; they are XWayland/KDE/driver issues, not omdurman defects.

### E1. Both instances spawned perfectly stacked; KWin ignored xdotool move/resize

New windows opened at (2048, 24) sized 3072×1704 on a 3072-wide screen —
exactly overlapping. `xdotool windowmove`/`windowsize` (separate and combined
calls, with the window activated first) were silently ignored — suspected
maximized-state. The first session's identical calls had worked, so this is
flaky, not deterministic. Symptoms while stacked: hover events reached the top
window but button presses at verified coordinates did nothing; driver clicks
aimed at screen x > 3072 (most of the top window's body) silently missed;
one `moveto` failed with "pointer warp ignored by the compositor" and left the
pointer at x = 3325.

**Working fix** — KWin DBus scripting, keyed on the window PIDs:

```bash
pidA=$(xdotool getwindowpid $(cat /tmp/omdurman-a/win))
pidB=$(xdotool getwindowpid $(cat /tmp/omdurman-b/win))
cat > /tmp/kwin-tile.js <<EOF
for (const w of workspace.windowList()) {
  if (w.pid == $pidA) { w.frameGeometry = { x: 0,    y: 24, width: 1536, height: 1704 }; }
  if (w.pid == $pidB) { w.frameGeometry = { x: 1536, y: 24, width: 1536, height: 1704 }; }
}
EOF
ID=$(qdbus org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript /tmp/kwin-tile.js)
qdbus org.kde.KWin /Scripting/Script$ID org.kde.kwin.Script.run
```

Worth adding to the run-omdurman skill's gotchas alongside the ydotoold note.

### E2. Probe pixel scale is "physical framebuffer", which moves under you

`probe.png` is the window's physical framebuffer: at a 1536×1704 physical
window it is 3072×1704 (2× window px); after the relaunch placed a
3072×1704 window it was 1×; after the KWin re-tile it became 1536×1680 (1×)
— and it lags resizes by a frame or two. The skill's "probe pixels are
physical window pixels" holds, but every screenshot *reading* in between was
displayed to me at yet another scale, and several rounds of mis-aimed clicks
came from mixing those scales. A `magick identify` of `probe.png` against
`xdotool getwindowgeometry` before every coordinate-derivation step is the
reliable discipline; a driver `check-scale` command would be better.

### E3. Symmetric "promoted to host" log lines on join — Low (observation)

Both peers logged `promoted to host; resumed sequence numbering next_seq=0
game_started=false` (A while alone, B on joining ~9 s later). The
stabilization gate held — game 1 started from B as host, game 2 from A, no
divergence — but two live hosts in one room each announcing promotion in the
log reads alarming in retrospect. A distinct "subordinating to peer host"
line when a second host appears would make failover auditable.

---

## What worked (for balance)

Lobby matchmaking, seat persistence + Verified badges, host-only AI/scenario
controls, the start gate (once its message is fixed, C3), spectator mode, and
late-join-by-replay all behaved across three process restarts and a host
migration. Set-up legality, night rules, LOS through the Buri huts, the CRT
eliminations, the mutual-destruction melee, reinforcement waves to 106 units,
and the AI's multi-wave turns all matched expectations with zero state
divergence between the two windows. The issues above are concentrated in
input *feedback* (A3/A4/B1/B2) and rail *stability* (A1/A2) — the engine
underneath did everything asked of it.

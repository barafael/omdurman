# A short Fall-of-Khartoum game — two local instances

A step-by-step script for playing a brief Fall of Khartoum (FoK) game across two
native instances on one machine, exercising the FoK rules added in the
`fok-full-playability` work. Read the **Assumptions** first — several are worth
cross-checking against your build, and one (the two turn systems) is important.

---

## Assumptions (cross-check these)

1. **Native, two instances, one machine.** You run two `omdurman` binaries that
   connect peer-to-peer over the configured matchbox signalling server
   (`wss://omdurman-matchbox.fly.dev` unless `MATCHBOX_SERVER` was set at build
   time). **This needs network access to that server** — the two local processes
   do not connect purely offline; they rendezvous through the signalling server,
   then talk WebRTC. If you have no connectivity to it, the lobby never populates.
2. **Room = first CLI argument**, default `"dev-room"` (`room_id()` in
   `omdurman-net`). Two instances launched with the *same* arg (or both with no
   arg) join the same room and find each other.
3. **Host = lowest peer id**, decided automatically once both are connected
   (`net.is_host`). You can't choose which instance is host; whichever wins the
   sort shows the scenario picker and the **Start Battle** button. The script
   says "on the host instance" / "on the guest instance" accordingly — look at
   which window shows the gold `[host]` tag.
4. **Both factions must be represented to start.** The lobby's Start button is
   disabled until one player is Anglo-Egyptian and one is Dervish
   (`all_players_ready`). So in a 2-player game the two instances take opposite
   sides.
5. **FoK turn order: the Dervish move first** (`GameState::new`, §9.322). The
   rules engine starts in turn 1, Movement phase, `active_player = Dervish`,
   day/night = Night (§9.341).
6. **One turn system.** The turn lives in the rules engine (`GameState`):
   `active_player` + `phase`. It gates movement / fire / melee and advances
   **only** via the **End Phase** button (`GameEffect::AdvancePhase`); the
   bottom-left status text and the faction gate both read it. There is no
   key-driven roll step — all dice are pre-rolled into effects by the engine.
   (The old SPACE-to-roll / ENTER-to-confirm round-robin has been removed.)
7. **The map must load before placement.** Selecting the scenario and starting
   loads the FoK board; placement is only meaningful once you see the
   Fall-of-Khartoum map (the small Khartoum mini-map, not the big campaign map).
8. **"Short game" = a Dervish rush to the Palace.** The fastest legal conclusion
   is the Dervish reaching GORDON's Palace hex, which ends the game immediately
   (§9.346/§9.35). This script aims for that rather than a full multi-turn battle.
9. **Set-up beyond GORDON is manual.** The **Set up scenario** button only
   auto-places the one unambiguous counter — GORDON in the Palace. Every other
   counter (the British garrison, the Dervish attackers) is dragged from the
   left-hand picker by hand. The script places only a handful, not the full
   historical rosters, to keep it short.
10. **Faction gating is active.** Each instance can only move/fire/melee with the
    side it picked, and only when the rules engine says it's that side's phase.
    If clicks "do nothing", check the HUD's `Active:` field and that you're on
    the right instance.

---

## 0. Build once

```shell
cargo build -p omdurman-app
```

## 1. Launch two local instances

Two terminals (or run the first in the background). Same room, so they pair up:

```shell
# terminal 1
cargo run -p omdurman-app -- dev-room
```

```shell
# terminal 2
cargo run -p omdurman-app -- dev-room
```

(`-- dev-room` is the room arg; omit it and both default to `dev-room` anyway —
passing it explicitly just makes the pairing obvious.)

Each window starts at **Connecting…** ("Waiting for players — share: ?room=dev-room"
in the status bar). When the WebRTC handshake completes, both flip to the
**Lobby**. If they stay on Connecting, re-read assumption 1 (signalling-server
reachability).

## 2. Lobby — pick sides and scenario

1. Identify the **host** window: in the *Players* list it carries the gold
   `[host]` tag.
2. **On the host instance:** under *Scenario*, click **Fall of Khartoum**. Guests
   see this as a read-only preview.
3. Pick factions so the two sides are covered:
   - **On one instance**, click **Dervish** under *Faction:*.
   - **On the other**, click **Anglo-Egyptian**.
   Each player row should now show its faction in light green and nobody
   "undecided".
4. **On the host instance**, click **⚔ Start Battle** (enabled once both factions
   are present). Both windows switch to the game view and load the FoK map.

> Tip: it's least confusing if you make the **host the Anglo-Egyptian** (it owns
> GORDON and the set-up button feels natural there), and the **guest the
> Dervish** (the attacker who will end the game). The script assumes this; swap
> "host/guest" below if you chose the other way.

## 3. Anglo-Egyptian set-up (on the A-E instance)

1. Confirm the HUD (top-right) shows the FoK state: `Turn 1 … Night`, and an
   `Active:` field. (It will read `Active: Dervish` — the Dervish move first —
   but set-up placement is **not** turn-gated, so you can still place A-E units.)
2. Click **Set up scenario** in the HUD. This broadcasts GORDON's placement; the
   **GEN. GORDON** counter (0·0·15-style leader, immobile) appears on the
   **Palace** hex on both screens. Hover the button first to see "Place 1
   fixed-hex unit(s)" — if it instead lists GORDON as *unresolved*, the board's
   Palace tile name didn't resolve (see Troubleshooting).
3. Place a couple of defenders by hand from the left **picker** sidebar:
   - Click a British/Sudanese infantry counter in the sidebar to pick it up
     (enters *Placing* mode).
   - Left-click a building/hut hex inside Khartoum near the Palace to drop it.
   - Repeat for one more infantry counter on an adjacent hex.
   (Two defenders is enough for a short game. Historically §9.321 has a larger
   garrison; we're keeping it minimal.)

   Optionally place a **Dervish fort** counter on the **North Fort** hex — §9.344
   has the Dervish hold it, and the engine now forbids A-E units from entering an
   enemy fort (§6.54). Skippable for a short game.

## 4. Dervish entry (on the Dervish instance)

1. The HUD shows `Active: Dervish`, Movement phase. Because it's **turn 1, FoK,
   Dervish movement**, the legal §9.322 entry edge is highlighted **green** along
   the south/east edge of the map.
2. From the picker, place 2–3 Dervish attacker counters (e.g. Mulazmin / Baggara)
   onto **green-highlighted south/east edge hexes**. (Placement isn't hard-gated
   to the edge — the highlight is guidance — so place them there to play
   faithfully.)

## 5. Dervish advance toward the Palace (Dervish instance, Movement phase)

1. Left-click one of your Dervish units to **select** it (only works on the
   Dervish instance, since it's the Dervish's phase). Adjacent legal destination
   hexes highlight green; out-of-range-this-turn hexes show gray.
2. Left-click an **adjacent** green hex to step toward the Palace. The counter
   slides over. Movement is one hex per click; click again (re-selecting if
   needed) to keep advancing. A rejected step won't move the counter (the engine
   is authoritative now).
3. Walk your lead unit hex-by-hex until it is **adjacent to the Palace hex**.
   (Depending on where the edge entry was, this may take the rest of this
   movement phase; that's fine — you can continue next turn.)

## 6. Cycle phases with End Phase

The rules engine advances only when someone clicks **End Phase** (top-right HUD).
The FoK phase order per player turn is:

```
Movement → Defensive Fire (Direct) → Defensive Fire (Maxim/Howitzer)
        → Offensive Fire (Direct) → Offensive Fire (Maxim/Howitzer) → Melee
        → (end of player turn; active player switches)
```

- During **fire** phases, the firing side selects a unit (left-click), sees red
  target hexes, and left-clicks a target to resolve. *Offensive* fire is the
  active player's; *defensive* fire is the opponent's (the engine flips it).
  Note §9.343: **both** sides use the Dervish Range Effects Table in FoK.
- For a short game you can click **End Phase** through the fire phases without
  firing, to reach the next Dervish movement phase quickly.
- Watch the **bottom-left log panel**: it shows the engine's recent results
  (moves, any disruptions/eliminations).

Keep ending phases until it is again the **Dervish Movement** phase.

## 7. Take the Palace — end the game

1. On the **Dervish instance**, in a Dervish Movement phase, select the unit you
   parked next to the Palace.
2. Left-click the **Palace hex** (the hex holding GORDON). The Dervish unit
   advances onto it.
3. §9.346 fires: **GORDON is eliminated**, his counter despawns, the engine
   records the turn, and the game ends. A centered **GAME OVER** modal appears on
   both instances showing the §9.35 verdict (e.g. a Dervish-decisive / -tactical
   level depending on the turn, minus any Dervish-loss penalty — for this short
   game, no losses, so the raw turn-based level).

That's a complete FoK game: set-up → guided Dervish entry → advance → victory
screen, with every FoK special rule (§9.343–§9.346, §9.35) enforced by the engine.

---

## What each step demonstrates (mapping to the rules)

| Step | Rule exercised |
|---|---|
| 3.2 Set up scenario → GORDON in Palace | §9.321/§9.346 fixed placement |
| 3.3 (optional) North Fort fort, A-E can't enter it | §9.344, §6.54 |
| 4 Green entry edge, turn-1 Dervish | §9.322 guided entry |
| 5 Hex-by-hex move, rejected steps don't animate | turn-gated, engine-authoritative movement |
| 6 End Phase cycle; both sides on Dervish fire table | §4 phase order, §9.343 |
| 7 Dervish onto Palace → GORDON dies, modal | §9.346 + §9.35 victory level |

---

## Troubleshooting / cross-check points

- **Stuck on "Connecting…"** — the two instances can't reach the signalling
  server (assumption 1). Confirm connectivity or a reachable `MATCHBOX_SERVER`.
- **No `[host]`/Start button anywhere** — only one instance connected; the lobby
  needs both peers.
- **Start Battle disabled** — both factions aren't picked yet (assumption 4).
- **"Set up scenario" lists GORDON as unresolved** — the FoK board's Palace tile
  isn't named "Palace" (the engine/app resolve the landmark from the tile name).
  Cross-check `assets/annotations.ron` has a `name: Some("Palace")` tile on the
  `fall_of_khartoum` board.
- **Clicking a unit does nothing** — you're on the wrong instance for the current
  phase, or it isn't that side's phase. Check the HUD `Active:` field; movement
  only works for the side the engine says is active.
- **No GAME OVER modal after taking the Palace** — confirm the unit that entered
  was Dervish-owned and the hex is the Palace landmark; the modal reads
  `game_over` set by the §9.346 trigger.

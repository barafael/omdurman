# Play-test notebook — two instances, mouse-driven (2026-10-05)

A = /tmp/omdurman-a (guest, slot 7), B = /tmp/omdurman-b (host, slot 8).

## Round 1
- Launch: both windows 2048x1256, same room; B elected host, A sees "Waiting for the host to start". OK.
- FoK set-up (A=British, B=Dervish/host). Illegal placements tried by click and refused: gunboat on land, 5th unit on a hex, Kehena onto a Degheim hex. OK (§5.51/§5.52).
- UI nit: remote player's cursor label sits on top of the Ready button (both windows same size → the idle pointer of A is drawn over B's rail).
- **F1 (rules, §5.44)**: Mulazmin walked (17,15)→(17,12)→through Kalakla gate→(16,11) in one move. (17,12) is adjacent to British 1B at (17,11) across a *wall* hexside. §5.44: ZOC extends from a walled-city hex outward across a wall, and out of (not into) the city across a gate. Engine: `zoc_extends_across` uses `board.is_walled_city`, which in Khartoum is only Palace+contiguous Building hexes (wall has a washed-away gap → no flood-fill) → rampart is ZOC-opaque in both directions in FoK. `Board::is_inside_of_wall` (palace-distance) already exists for the LOS note; ZOC should use it.
- Q1 (rules reading, no change): LOS table prints "Wall (b)" in Ground→Ground, Ground→Rough and Rough→Ground. Engine reads it literally → a rampart unit (rough level per note b) cannot fire at a ground unit at the foot of the wall and vice versa; only Rough↔Rough (gunboats, other ramparts, rough hills) fire across a wall at −4. Seen in play: 1B at (17,11) on the rampart had exactly 1 target (the Mulazmin that came in through the gate), none of the 12 Dervish under the wall. Plausible but harsh; alternative reading "(b)" = "except the rampart you stand on". Left as is.
- U1 (UI): the fire-allocation tray re-opens on every staged attack and covers the lower-middle board (here: the whole south wall where the fight is). A click on a hex under it is swallowed (my first two Sudanese attacks silently did not stage).
- **F2 (rules, §6.23/§6.3)**: Fort Buri stack (20,9) → Mulazmin at Buri gate (22,10), range 2 on a hexside-tie ray. LOS is granted (only the path via (21,9)→gate is open; the other, via (21,10)→wall, is blocked for ground firers) but the tray charges "−4 hexside": `target_hexside_fire_modifier` reads only the primary `HexLine`, not the path the LOS walk accepted. Either LOS is blocked or the wall was not crossed — never both.
- §6.63 breach by Dervish artillery at night (range 1, doubled): breached, 1B on the rampart eliminated. OK. Breach resolves instantly from a list (not via the tray) — fine. Breach marker on the map is nearly invisible at default zoom.
- §6.14 combined fire from 3 hexes (10 units, 30 factors) staged as one attack; §6.82 advance offered to all participants; tribe mix on advance refused; advance through gate OK.
- §9.346 Gordon: "Move 0 MP", cannot move. OK.
- **F3 (candidate, §9.345)**: the off-board White↔Blue crossing is pinned to two labelled hexes (1,0)↔(16,1). (16,1) "Blue Nile Mouth" is not a map-edge hex (the Blue Nile leaves the north edge at (5,0)/(6,0) and (14,0)/(15,0)). Manual: any move "from the White Nile to the Blue Nile ... off-board". At night (halved 5 up) the 6-MP crossing is impossible — correct.
- P1 (presentation): end-of-turn-1 telegram says "FIRE DESTROYED SIX ENEMY BANDS AT MESSALAMIA GATE" — the six Dervish losses were spread over Kalakla gate (1), the west gap (2), Buri gate (1) and Messalamia gate (2).
- Night fire: Buri artillery reached range 3 (Dervish artillery line max 7 → 3), rifles dropped out at range 3 (max 4 → 2). OK §8.1/§9.343.
- Turn 2 day: melee Kehena×2 (12, +2) vs 3E in a building (5, +1): both rolls, simultaneous losses, mandatory Dervish advance of the survivor (§7.3/§7.6/§7.7). No terrain modifier in melee. OK.
- **F4 (UI, §9.345)**: gunboat on the Blue Nile Mouth (16,1), full allowance (10 up), click on White Nile Mouth (1,0): "ORDER REFUSED — No route to (1,0): zones of control, enemy units or impassable terrain…". The hover tooltip advertises "Nile-mouth crossing — 6 MP flat (§9.345)" but the route planner never offers the off-board step, so the rule is unreachable by a human player (the bot can do it).
- Disrupted Kehena melee-attacked: attacker rolls, defender gets no roll. Correct per the Disrupted Units note ("may not melee"), but the combat card cites "no melee factor — no roll (§6.51)" — wrong reason/citation for a disrupted defender (P2).
- Turn 3: fire at GORDON alone in the Palace: allowed, "Eliminate 1", nobody lost (leader immune to fire, §6.51/§9.346). OK. Card shows "Eliminate 1" with no explanation that the leader is immune (nit).
- Melee declared against GORDON alone: attacker rolls (Eliminate 2), card says "lost: Gordon", mandatory advance, game over → Dervish Decisive (turn 3, 14 Dervish losses < 16 → no penalty). Outcome defensible ("as advance after combat"), though strictly the melee result cannot hurt him — a "no effect" roll would have left him alive although the Dervish could simply have walked in.
- Gazette + telegrams render; totals right (14 Dervish, 7 A-E incl. Gordon).
- FoK game 1 over after 3 turns. Fix candidates so far: F1, F2, F3, F4; presentation P1, P2; UI U1.

## Round 1b — Campaign (A = Anglo-Egyptian, B = Dervish/host), river mines on
- Set-up refusals verified by click: Taiasha outside the walled city, Khalifa on the Tomb hex (only Palace/Grounds), Isa Zachneih north of El Debeba, fort north of Khor Shambat, fort on the east bank north of Halfaya, gunboat off the south edge, second gunboat in a hex, mine north of the khor mouth row, second mine in the same hex. All OK (§9.111, §10.11).
- Q2: two forts may be set up in the same hex (did it at (39,34)). Nothing in the manual forbids it (≤4 units), though it is an odd reading of "fort". Left.
- Nit: the refusal for a 2nd gunboat in a hex reads "gunboats may not stack with non-gunboat units".
- A-E turn 1 arrivals: third Egyptian-Division brigade, British brigade, 4th gunboat all refused/dimmed (§9.113). Friendlies: 4 fit on Abu Alim (8 MP entry → 1 MP left), the 5th greys out until one steps off, then is offered again. OK.
- Driver note (not the game): a bad cached `warp_scale` made sidebar clicks fail after a focus blip; deleting the cache fixed it.
- Turn 2: Isa Zachneih defensive fire killed a Friendlies unit on the east bank → score "A-E 0 – 1 Dervish" (§9.14, 1 pt). OK.
- Ending a fire subphase with staged attacks asks "Discard 1 attack & end phase? / Keep them". Good.
- **F5 (rules, §6.42)**: named gunboat in the Maxim/Howitzer subphase: howitzer at Isa Zachneih (range 5) + its Maxims' second fire at the same hex stage as two attacks. Howitzer landed on target (impact 9) → the Maxim attack was then refused: "unit Isa Zachneih has already been fired at this phase (§6.14)" and the Maxim second fire was lost (sub-phase closed). §6.42: "Howitzer fire may be combined with Maxim fire, but only if the howitzer fire impacts in the intended hex" — the engine forbids exactly the case the manual allows (and whichever of the two resolves second is refused).
- Q3 (reading): §6.14 "(exceptions: Maxim guns and gunboats — see 6.4)" is read as "Maxim and gunboat *targets* may be fired at repeatedly" (test `gunboat_and_maxim_may_be_fired_at_repeatedly`). "see 6.4" suggests the exception is about those units *firing* twice. Deliberate; left.
- P3: display names vs engine identities differ: unit `Named(Naser)` shows as "Gunboat Abu Klea", `Old(LordKitchener)` as "Gunboat El Teb" (FoK: "Steamer Bordein/Talahawiyeh"). Cosmetic, but the log/debug identity and the counter disagree.
- Turn 2 melee: 3 Friendlies (18, Dervish +2 since the whole side is Friendlies) vs Isa Zachneih: both lose one; Isa gone → A-E 1 VP, transport unlocked (§5.21, §6.52, §9.14).
- Turn 3: two stacked forts fire 8 at a gunboat at range 2: "Disrupt" = miss, gunboat not disrupted (§6.61). Gunboat artillery ×2 at range 2 on the forts: Eliminate(1) = miss (needs 2+, §6.62). OK.
- Turn 4: gunboat steams onto the mined hex (41,35): stopped there (asked for (41,36)), mine rolled automatically by the host: 2 = no effect. Rules OK (§10.12). U2: neither window showed any card/toast about the mine — the boat just silently stops short (only the log has "MineResolved").
- §9.113 leaders gate: End Phase disabled on turn 4 with Hunter not yet in play, reason shown. OK.
- §5.21 load: offered when a Friendlies unit and gunboat began the turn adjacent, after Isa Zachneih fell; carrier may not move on the loading turn; second Friendlies not offered while one is aboard (documented approximation). OK. U3: clicking the loaded gunboat's hex selects the passenger; trying to sail gives "(40,20) is the Nile: only gunboats move on the river" rather than "aboard / gunboat sails next turn".
- §5.3 Zariba: "Build the Zariba here" appears only for a *single* selected battalion (not when the stack is double-clicked — U4). Builder cannot move afterwards; at end of the A-E player turn the two hexsides of its hex turned green (built). OK.
- Stopping round 1 here (turn 4 of the Campaign) to fix.

### Round 1 fix list
1. F1 §5.44 ZOC across Khartoum's rampart (engine).
2. F2 §6.23 wall −4 charged on a hexside-tie ray whose LOS used the gate (engine).
3. F5 §6.42 howitzer + Maxim second fire on the same hex (engine).
4. F4 §9.345 Nile-mouth crossing unreachable from the UI (app).
5. P2 wrong reason on the melee card for a disrupted defender (app).
6. P1 telegram attributes all fire kills to one place (press).
7. U2 mine strike gives no on-screen notice.
Not changed (questions for Rafael): Q1 Wall(b) reading, Q2 two forts in a hex, Q3 §6.14 exception reading, F3 Blue Nile Mouth hex (16,1) is not on a map edge.
- (U2 retracted: the app does post a "River Mine" slip — `dispatch.rs` — I looked too late and it had faded.)

### Round 1 fixes made
- F1: `BoardInfo::palace_steps` (BFS from the Palace, stopped by wall hexsides) decides the city side of each Khartoum wall/gate hexside; ZOC uses it. The old "nearer the Palace" test tied on 10 of 36 hexsides (bastions) — also sharpened §6.3 note b there. Tests: `khartoum_rampart_projects_zoc_outward_only`, `every_khartoum_wall_hexside_has_one_city_side`.
- F2: `target_hexside_fire_modifier` follows the LOS-clear side of a hexside-tie ray (`los_entry_hexes`). Test: `fire_through_a_gate_beside_a_wall_is_not_charged_the_wall`.
- F5: on-target howitzer shell tracked apart (`units_shelled_this_phase`) so Maxim second fire may hit the same hex. Test: `howitzer_on_target_combines_with_maxim_second_fire`. Documented as an approximation (two rolls, not one combined).
- F4: gunboat on a Nile mouth can click the other mouth: plots the 6-MP off-board leg (§9.345).
- P2: melee card wording. P1: loss counts are only pinned to a place when most of them fell there.
- P4 (new, found reading the telegram code): compass bearings in the press used `q + r/2` for "east"; the grid is `q − r/2` → "south-east of Omdurman" for a hex due south. Fixed + test `bearings_follow_the_map`.
- `cargo test --workspace`, clippy -D warnings, fmt: green.

## Round 2 — replay after fixes
- F1 verified: Mulazmin (17,15) → (16,11) via the Kalakla gate now "out of reach: cheapest route costs 18 MP" — (17,12) is in the rampart battalion's ZOC and must be stopped in. Nit N1: the refusal blames "Rough and Swamp cost 3 MP a hex" when the detour is caused by a ZOC stop.
- F2 verified: Fort Buri stack → (22,10): tray now "net die +1" (no −4 hexside).
- ZOC stop shown on the unit card ("in enemy ZOC — may withdraw next Movement phase"), further move refused. OK §5.43.
- F4 verified: gunboat on Blue Nile Mouth, click White Nile Mouth → "1 step, 6 MP total", confirm, boat is at (1,0) (turn 2, day).
- Q4 (reading): fire at units inside Fort Buri: "net die −6" = Building −3 (the printed "Fort Buri" hex is Building terrain) + fort counter −3 (§6.54). Same at Fort Makran / North Fort. By the letter both apply; but in FoK the printed building *is* the fort, so this looks like a double count. Left for Rafael.
- Melee across a wall hexside: 0 adjacent targets offered (§7.2). OK. Route refusal next to Fort Buri says "the city wall is in the way" when the real stop is the fort's ZOC (N1 again).
- Melee vs Fort Buri (fort 1 + 1B 5 + artillery 1 = 7 defending, §6.54/§7.4): attacker D result → 2 of 3 "units" disrupted, and the draw picked **the fort itself** and the battalion. Q5: can a fort be disrupted? Fire can't (§6.62 "any other result is a miss"), but in melee the fort counter is one of "the units in the target hex" and gets inverted → it then has no ZOC and cannot fire (vs §5.44 "ZOCs do extend out of a fort (even if unoccupied)"). Left for Rafael; I'd exclude forts (and count only real units for the ½).
- FoK replay stopped at turn 3; switching to the Campaign to verify F5 and try §7.5.

### Round 2b — Campaign replay (A host = Anglo-Egyptian, B = Dervish)
- F5 in play: howitzer + the same boat's Maxim second fire both staged on Isa Zachneih; this time the shell scattered (impact 3) and both resolved. The on-target case that failed in round 1 is pinned by the new unit test (dice cannot be forced by clicks).
- Khor Shambat: camel stack entering at (8,14) asked to ride to (19,14): "cheapest route costs 16 MP" = 11 clear + 5 for the khor hexside. OK (TEC Khor +5). N2: the toast says "this move has 12 MP left", the hover tooltip for the same click "11 left".
- §7.5 in play: Egyptian brigade moves next to 4 Danagla camels (ZOC stop), declares melee; the Dervish window gets "You may retreat threatened cavalry/camel…", hexes two away are outlined; each camel retreats individually (4 clicks, one retreat per unit); with the hex empty the attacker's card says "The defenders withdrew (§7.5): resolving ends the attack" → "the melee lapses, with no roll and no forced advance". OK.
- N2 explained: hover tooltip's route cache was keyed by (unit, hex, path, unit count, phase) — not by turn — so it showed last turn's "11 left". Fixed (`route_stamp`). N1: refusal text now also names the khor and ZOC stops.

### Round 2 result
No new engine defects confirmed; two rule-reading questions (Q4 fort −3 on top of building −3, Q5 fort disrupted in melee), two UI nits fixed (N1, N2).

## Round 3 — short smoke after the round-2 fixes
- N1 wording seen in play: "(16,11) is out of reach: the cheapest route costs 18 MP, this move has 8 MP left (… no route passes through an enemy zone of control …)". Punctuation tidied afterwards.
- Two steamers combine 8 at the North Fort, range 2: +1 A-E, −3 building → E1 = miss (needs 2+, §6.62). North Fort at a steamer: E2 = miss (needs 3+, §6.61). OK.
- Telegram after turn 1: "FIRE DESTROYED ONE ENEMY BAND AT BURI GATE … ENEMY MAIN BODY A MILE SOUTH-WEST OF KALAKLA GATE" — place and bearing now right (the body stood on the south edge, west of the gate).
- Final: `cargo fmt`, `clippy -D warnings`, `cargo test --workspace` (856 passed), traceability test, mutation gate on the diff: all green. Not run: Kani, `trunk build --release`.

## Open questions for Rafael (not changed)
- Q1 LOS table "Wall (b)": rampart units cannot fire at ground-level units at the foot of the wall (and vice versa); only rough↔rough crosses a wall.
- Q2 two forts may be set up in one hex.
- Q3 §6.14 "(exceptions: Maxim guns and gunboats — see 6.4)" read as targets that may be fired at repeatedly.
- Q4 units inside Fort Buri / Makran / North Fort get −6 (building −3 + fort −3).
- Q5 a fort can be disrupted by a melee "D" result (then no ZOC, no fire for a turn).
- F3 §9.345: the "Blue Nile Mouth" hex (16,1) is not on a map edge; the crossing is tied to two labelled hexes.
- UI: U1 allocation tray covers the board and swallows clicks under it; U3 loaded gunboat's hex selects the passenger; U4 "Build the Zariba" only for a single selected battalion; breach marker hard to see; remote cursor label drawn over the Ready button.

## Round 4 — Rafael's rulings applied (same day)
1. Fort −3 no longer adds to the hex's own terrain modifier (garrison of Fort Buri: −3, seen in play).
2. Fort disrupted by a melee "D": left as is (ruled correct).
3. LOS "Wall (b)" reading: left as is.
4. §9.345 mouths moved to where the rivers leave the map: White (1,0), Blue (5,0)/(6,0); (16,1) no longer takes part. Crossing (6,0)→(1,0) made by click; the bot offers it too.
5. One fort per hex (`StackingError::FortStack`); refused in Campaign set-up in play.
6. §6.14 exception is about firers: gunboats and Maxims are fired at once per subphase like everyone else.
UI: staged-fire list moved into the left rail (nothing floats over the board); a click on a loaded gunboat's hex selects the gunboat in the Movement phase; "Build the Zariba here (N battalions)" for a selected stack; breach drawn as a wide pale band with a red core, and it now appears at once (new bars were spawned hidden until the next state change — that was the real cause of the "invisible" marker); remote cursors are not drawn over this window's rail/top bar and vanish 2 s after the peer's pointer leaves its board.
Checks: fmt, clippy -D warnings, 858 tests, traceability, mutation gate on the diff — green. Not run: Kani, `trunk build --release`.

## Round 5 — the branches never exercised (solo and two-instance games)
- Kani standard tier (10 min/harness cap): 89/90; the failure, `max_day_range_is_the_last_in_range_hex`, predates this work (a weapon line a faction's table does not print has no in-range hex) — proof corrected, 90/90.
- §6.14 re-read with Rafael's question: the exception "(Maxim guns and gunboats — see 6.4)" points at §6.4, where those units fire a second time and "may fire at enemy units fired at in Direct Fire Subphase". So nothing stops the Dervish hitting a gunboat: they fire at it once per fire phase with as many guns as they can combine (§6.14), and the bot's fire planner already merges per-hex candidates into combined attacks.
- **H1 (board data, §9.231/§9.232)**: Historical game, me as A-E vs AI Khalifa. The AI's Jehadia shot the 1st Egyptian Brigade out of (33,12) at "−2 thorn hedge". Checked against the map and the Terrain Effects Chart legend (trench = solid line + dashed parapet, hedge = line of crosses): the Zariba's northern stretch is the *trench*, the southern the *hedge* — the board file had them swapped. Fixed in `campaign.ron`; the two entrances are now `ZaribaTrenchEnd` (north) and `ZaribaThornHedgeEnd` (south, hedge rules: no melee/advance across, −2). Replayed: fire at (33,11) −4, melee into it at −2 instead of +2.
- §5.54 in play: 4th Egyptian Brigade, all four at one hex → "+1 A-E, +1 brigade integrity"; one battalion alone → +1 only. OK.
- U5: staging one battalion and then double-clicking its stack at the same target was refused whole ("already allocated"); the rest of the stack now joins.
- §9.24: both games ended after the noon turn with the right levels (A-E Draw / Dervish Marginal → net Draw; then Draw/Draw); Gazette totals right.
- Dervish may walk into an empty Zariba hex through the southern entrance (+2 MP) on turn 1 — legal (§9.233); Osman Digna did, alone, and was shot.
- Campaign with the river chain (two instances, me on both sides), played to the end of turn 22:
  - **C1 (§10.22)**: a British gunboat was *refused* entry to a chained hex with the text "entered a chained hex and must stop". Manual: she enters, stops, and may not cross. Now: a move plotted past the chain ends on it (she does not know where it lies), no further that turn, and from the chain only back the way she came until it is sunk. Seen in play: slip "has run onto a chain … at (41,36)", onward step refused, back step accepted.
  - **C2 (§10.21)**: a 4-hex chain could be laid in mid-stream where the Nile is 8 wide (bars nothing). Now it must run bank to bank (an island counts). The click-by-click placement checks the partial line without the far bank (my first version of the check broke it — caught in play).
  - §10.23: artillery at the chain, 15 factors: rolls 7 and 6 → E2, "it holds" (needs 3); infantry (the Royal Engineers) a full unmoved turn on the bank → "Troops holding the bank have sunk the chain". OK.
  - §8.1 night (turns 9–10): A-E movement halved (RE 4 MP, gunboat 9 downstream refused at 10), artillery range capped, "no howitzer fire at night". OK.
  - §8.2 desertion: roll 1 → 2 units; list offers only eligible units (no Khalifa, guns, forts, boats); End Phase blocked until done; no VP. OK.
  - **C3 (§5.21 × §8.2)**: the Isa Zachneih *deserted* — the Friendlies' transport then never unlocks (the flag is set only on elimination). Now unlocked whenever the Isa Zachneih is no longer on the map.
  - §6.53: Royal Engineers beside fort (39,34): commit in Movement, may not move, may not fire at the fort ("only artillery"), fort removed at the end of the A-E turn; no VP for it. OK.
  - §9.12/§9.14: game ends after turn 22; A-E 0 – 25 Dervish (tomb) → Dervish Tactical. OK.
  - Gazette: "THE ENEMY LOSES NONE BANDS" → "No Bands"; "Our guns destroyed one enemy fort" for an engineers' demolition → "We have destroyed…".
- Small leftovers: melee card on a lone leader now says he fell to the attackers entering the hex whatever the dice (§6.51) — the engine always did that; gunboat identities renamed to their counters (`AbuKlea`, `ElTeb`).
- Two-instance FoK after the above: eight separate attacks staged — the rail list scrolls and stays legible (section refs shortened to "§6.24" to wrap less). Breach marker verified appearing at once on the shot. Guest killed mid-game and relaunched into the same room: rebound to its seat, identical state hash on both peers before and after, moved a unit afterwards. The host kept playing meanwhile.
- Solo Campaign vs the AI Khalifa: the Isa Zachneih fell to Friendlies melee on turn 2; load (turn 4, button now names the boat), carry (turn 5, a sail on the loading turn refused), disembark on the west bank (turn 6, three hexes offered) and the first west-bank step all worked (§5.21).
- Bot: `plan_fire` now lets weapons too weak to matter alone join a target that massed fire makes worth shooting at (two forts combine on a gunboat instead of neither firing). Test `forts_combine_their_fire_on_a_gunboat`.
- Kani (standard tier, 10-min cap): omdurman-types 23/23, omdurman-rules 67/67 — one harness ran CBMC out of memory with three solvers in parallel and verified alone (194 s).
- `trunk build --release`: builds (one dead-code warning on wasm, `page_text`, now allowed there). AI vs AI Campaign (seed 0xb, Kitchener vs Khalifa, 22 turns): the Dervish fire planner massed six forts (26 factors) on gunboat Sheik and sank her on a 7; of 27 Dervish fire attacks, 20 combined two or more weapons.

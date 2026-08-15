# Common Doctrine — Remember Gordon!

System-wide tactical doctrine for both sides. These rules of thumb hold for
whichever faction you command; side-specific priorities live in the other
files. Each entry cites the manual section(s) it is grounded in.

## Fire combat

1. **Mass fire, don't scatter it.** A unit's fire factor is unitary and may not
   be split between hexes (§6.13), but co-stacked units may combine their
   factors into one attack (§6.14). One big attack pushes the CRT factor column
   up and is more likely to Disrupt or Eliminate than several small ones.
   — §6.13, §6.14

2. **Halving rounds per unit, floor, minimum 1.** When fire factors are halved
   (range bands §6.22), each unit's factors halve independently, rounded down,
   never below 1 (§6.16). Four battalions of 9 fire a combined 16 factors when
   halved, not 18. Plan attacks using the halved column, not the printed total.
   — §6.16, §6.22

3. **Combine with co-stacked firers before splitting stacks.** You may divide a
   stack to fire at different hexes (§6.15), but the same units can also all
   fire at one hex for a combined attack. If you want both a high factor column
   and multi-target coverage, use *different* stacks for the two jobs.
   — §6.14, §6.15

4. **Target terrain, not just numbers.** Apply the defender's terrain modifier
   (§6.23) to your net die roll before cross-indexing the CRT. Firing at a
   hex that reduces your roll below the threshold you need wastes a unit's
   once-per-phase fire (§6.14). Look for the hex with the least defensive
   benefit.
   — §6.23, §6.14

5. **Check LOS before you commit.** Direct fire requires a LOS path per the
   Line of Sight Table (§6.21, §6.3). Only howitzers ignore LOS (§6.64). A
   blocked target is a wasted attack — reposition or pick another hex.
   — §6.21, §6.3, §6.64

6. **A unit fires once per phase, Maxims/gunboats excepted.** In a given fire
   combat phase a combat unit may fire once and be fired at once, except Maxims
   and gunboats (§6.14). Spend the attack deliberately — there is no second
   try this phase for ordinary units.
   — §6.14, §6.42

7. **Allocate first, then resolve.** In Direct Fire you must allocate all your
   attacks before resolving any (§6.41). Order your resolutions so that an
   Eliminate opens a hex for advance-after-combat before you resolve the shots
   that depend on it — but remember attacks are fixed at allocation.
   — §6.41

8. **No advance after defensive fire.** There is no advance after combat
   resulting from defensive fire (§6.7). When you are the non-moving player,
   defensive fire is pure attrition — never plan an advance off the back of it.
   — §6.7

## Melee

9. **Melee is simultaneous — a dead defender still rolls.** Both sides roll,
   and losses from one side do not cancel the other's roll (§7.3). Committing
   to melee risks taking a hit even when you will win. Weigh that against the
   Dervish +2 / Anglo-Egyptian +1 modifiers (§7.7).
   — §7.3, §7.7

10. **Only infantry/cavalry/camel/Dervish leaders attack in melee.** All units
    except gunboats may defend (§7.4). Artillery and leaders can be *used* as
    defence but are not melee attackers — don't build a melee plan around them.
    — §7.4, §7.1

11. **Mandatory advance for the Dervish after a winning melee.** If a melee
    eliminates all defenders, every surviving eligible Dervish unit *must*
    advance into the vacated hex (§7.6). Use it to pour through a gap — but be
    ready for it to be enforced, and don't stack-block your own follow-through
    (§5.51). The Anglo-Egyptian may advance if desired (§7.6).
    — §7.6, §5.51

12. **Advance-after-combat is restricted — and perishable: take it.** Only
    units that took part and were adjacent may advance; artillery may never
    advance; never across a wall hexside (except a gate or breach) or across
    a khor (§6.82). A breach or a gate is a strategic corridor — hold it or
    close it.
    In the legal-action list the move appears as
    `AdvanceAfterCombat { unit_id, to }` — that is *your* unit stepping into
    a hex your combat just emptied. The window evaporates at the end of the
    phase (only the Maxim/howitzer subphase bridge keeps it open, §6.42): an
    advance you skip is ground you paid casualties for and never took. After
    a fire attack empties the target hex, look for your eligible firers'
    `AdvanceAfterCombat` candidates *in that same phase* (§6.82) — closing
    onto a vacated hex moves you without spending movement points.
    — §6.82, §6.63, §6.42

13. **Take the advance — it is a listed action, not an automatic effect.**
    When offensive fire or a melee empties an enemy hex, an
    `AdvanceAfterCombat` action appears in your legal list for the units that
    fought (§6.82, §7.6). The window closes at the end of the phase, so take
    it *this phase* or lose it. Advancing gains the hex, keeps the momentum,
    and (for the Dervish) is often mandatory anyway. If you see a vacated hex
    in front of a participant, strongly prefer advancing over ending the phase.
    — §6.82, §7.6

## Zones of control

14. **ZOC stops movement cold.** A unit must stop immediately when it enters an
    enemy ZOC (§5.26, §5.43). There is no movement cost to enter or leave a ZOC
    (§5.42) but you cannot keep moving through one — plan two turns to slip a
    screen, not one.
    — §5.26, §5.43, §5.42

15. **ZOC terrain gaps are real.** ZOCs do not extend across a khor, into a
    fort, or into a walled-city hex across a wall; they extend both ways across
    a breach, and out of (not into) a fort or a walled city across a gate
    (§5.44). When a fortress line or wall blocks ZOC, the breach is the only
    two-way gap — defend it or exploit it.
    — §5.44

16. **Disrupted units exert no ZOC.** A disrupted unit has no ZOC (§5.41), so a
    front of disrupted units is a sieve. Disrupt the enemy's screening line
    before your breakthrough — and keep your own screen undisrupted.
    — §5.41

## Stacking

17. **Four combat units per hex, leaders free.** Max four units per hex except
    leaders (free stacking) and gunboats (alone, §5.21 aside) (§5.51).
    Overstacking at end of movement or during combat is illegal — compute the
    stack before you move the fifth unit in.
    — §5.51

18. **Dervish tribes never mix; AE brigades must.** Dervish units of different
    tribes may not stack together (§5.52), and Dervish leaders may only stack
    with their own command (§5.53). Anglo-Egyptian brigades get +1 for
    integrity only when all four battalions of one brigade share a hex *and*
    fire at one hex (§5.54). Keep those constraints in mind when shaping stacks.
    — §5.52, §5.53, §5.54

## Movement

19. **Terrain costs are per hex entered, roads cheapen clear/rough/trees.**
    Movement costs follow the Terrain Effects Chart (§5.11): Clear 1, Rough 1,
    Trees 2, Swamp 2, Hilltop 1, Huts 2, Building 2; Nile is impassable to land
    units (§5.22). Roads reduce Clear/Rough/Trees/Hilltop to 1. Count the route
    before moving so you don't waste a unit's allowance (§5.13 — no carryover).
    — §5.11, §5.22, §5.13

20. **Gunboats: upstream vs downstream.** A gunboat's two allowances are
    upstream (smaller) and downstream (larger) (§5.24). Moving even one hex
    upstream caps the whole turn at the upstream allowance — never start a turn
    upstream-bound if the down-current leg is the urgent one.
    — §5.24

21. **MP don't carry over.** Unused movement points are lost at the end of the
    turn and cannot be transferred (§5.13). Spend them or lose them — but an
    over-extended unit can be caught exposed in the enemy's turn.
    — §5.13

## Night turns (§8)

22. **Night halves AE movement, halves ranges, bans howitzer.** At night:
    Anglo-Egyptian movement halved (round down), no AE howitzer fire, all fire
    ranges halved for both sides (min 1, so range 1 stays 1) (§8.1). Force
    contact at close range and don't rely on long guns.
    — §8.1

## Leaders (§6.51)

23. **AE leaders die alone.** An Anglo-Egyptian leader is eliminated if alone in
    a hex when a Dervish unit occupies or passes through it, or if all units he
    is stacked with are eliminated (§6.51). Keep a bodyguard with the leader who
    must reach the Mahdi's Tomb (§9.14).
    — §6.51, §9.14

## Victory priorities (§9.14)

24. **VP are asymmetric — play to your table.** The AE scores the Mahdi's Tomb
    (25), the Khalifa (10), Isa Zachneih (1), and 1 per Dervish unit eliminated;
    the Dervish scores British leaders (10), gunboats (10), and AE land units
    (3) (§9.14). Every unit trade should be read against these numbers, not raw
    body count.
    — §9.14

// ══════════════════════════════════════════════════════════════
//  REMEMBER GORDON! — The Battle of Omdurman
//  Rules of Play — period-authentic recreation
//  Phoenix Enterprises, Ltd. © 1982
// ══════════════════════════════════════════════════════════════

// ── Page setup ──────────────────────────────────────────────
#set page(
  paper: "us-letter",
  margin: (top: 0.42in, bottom: 0.48in, left: 0.48in, right: 0.48in),
  numbering: none,
)

// ── Typography ───────────────────────────────────────────────
#set text(
  font: "Overpass",
  size: 8.8pt,
  lang: "en",
  hyphenate: true,
)
#set par(leading: 0.42em, spacing: 0.55em)

// ── Colours ──────────────────────────────────────────────────
#let ink    = rgb("#1a1208")
#let paper  = rgb("#f4edd8")
#let tinted = rgb("#e8dfc0")
#let ruled  = rgb("#9a8870")

// ── Typographic helpers ──────────────────────────────────────
#let hrule(weight: 0.6pt) = line(length: 100%, stroke: weight + ink)
#let thick-rule = line(length: 100%, stroke: 2pt + ink)
#let thin-rule  = line(length: 100%, stroke: 0.5pt + ruled)

// Section head: bold caps spaced
#let sec(body) = {
  set text(size: 8.8pt, weight: "bold")
  upper(body)
}

// Sub-section head: bold italic small
#let subsec(body) = {
  set text(size: 8.8pt, weight: "bold")
  body
}

// Rule-paragraph: numbered wargame rule
#let rule-par(num, body) = {
  block(spacing: 2.2pt,
    par(hanging-indent: 0pt,
      [#text(weight: "bold")[#num)] #body]
    )
  )
}

// ── Counter box: tiny hex-game counter ───────────────────────
#let counter-box(name: none, nums: "", dark: false, width: 30pt) = {
  let bg = if dark { ink } else { tinted }
  let fg = if dark { paper } else { ink }
  box(
    width: width, height: width,
    fill: bg,
    stroke: 1.2pt + ink,
    inset: 2pt,
    align(center + horizon,
      stack(spacing: 1pt,
        if name != none { text(size: 5.5pt, fill: fg, weight: "bold", align(center, name)) },
        text(size: 7.5pt, fill: fg, weight: "bold", nums),
      )
    )
  )
}

// ── Three-column layout helper ────────────────────────────────
// Typst columns() works across page; we use it directly.

// ══════════════════════════════════════════════════════════════
//  PAGE 1
// ══════════════════════════════════════════════════════════════

// ── Masthead ─────────────────────────────────────────────────
#align(center)[
  #text(font: "Tanach", size: 86pt, weight: "bold", tracking: 2pt)[
    REMEMBER GORDON!
  ]
  #v(-6pt)
  #text(font: "Tanach", size: 64pt, weight: "bold", tracking: 1pt)[
    The Battle of Omdurman
  ]
]
#v(-16pt)
#thick-rule
#v(20pt)

// ── Table of Contents (3-col) ────────────────────────────────
#set text(size: 8.2pt)
#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 14pt,
  // Col 1
  [
    *1) Introduction*\
    #h(8pt)1.1) General Comments\
    #h(8pt)1.2) Game Scale\
    *2) Game Components*\
    #h(8pt)2.1) The Game Maps\
    #h(8pt)2.2) Play Aids\
    #h(8pt)2.3) The Units\
    #h(8pt)2.4) Game Parts Inventory\
    *3) Getting Started*\
    *4) Turn Sequence*
  ],
  // Col 2
  [
    *5) Movement Phase*\
    #h(8pt)5.1) General Rules\
    #h(8pt)5.2) Movement Restrictions\
    #h(8pt)5.3) Constructing "The Zariba"\
    #h(8pt)5.4) Zones of Control\
    #h(8pt)5.5) Stacking\
    *6) Fire Combat Phase*\
    #h(8pt)6.1) General Rules\
    #h(8pt)6.2) How to Have Combat\
    #h(8pt)6.3) Line of Sight Table\
    #h(8pt)6.4) Fire Combat Sequence\
    #h(8pt)6.5) Special Unit Capabilities\
    #h(8pt)6.6) Special Artillery Capabilities\
    #h(8pt)6.7) Defensive Fire\
    #h(8pt)6.8) Offensive Fire
  ],
  // Col 3
  [
    *7) Melee Phase*\
    \
    *8) Night Game Turns*\
    \
    *9) The Scenarios*\
    #h(8pt)9.1) The Campaign Game\
    #h(8pt)9.2) The Historical Scenario\
    #h(8pt)9.3) Bonus Game: FALL OF KHARTOUM\
    \
    *10) Optional Rules*\
    \
    *11) Historical Notes*
  ],
)
#set text(size: 8.8pt)
#v(3pt)
#hrule()
#v(3pt)

// ══════════════════════════════════════════════════════════════
//  BODY — three-column
// ══════════════════════════════════════════════════════════════
#columns(3, gutter: 12pt)[

// ── §1 Introduction ──────────────────────────────────────────
#sec[1) Introduction:]
#v(2pt)

#subsec[1.1) General Comments:]

"REMEMBER GORDON!" --- THE BATTLE OF OMDURMAN is a simulation of the final battle in Great Britain's two-year campaign to reassert her presence in the Sudan (1896--1898). Fought September 2nd, 1898, Omdurman finally broke the back of the fanatical Dervish rebellion and gained Britain a million square miles of desolate territory and two million impoverished subjects. With two players, one assumes the role of Herbert Kitchener, Sirdar (CIC) of the Anglo-Egyptian army; the other player becomes the Khalifa, Abdullah the Taiasha, absolute ruler of the Dervishes. The game is also suited for multi-player participation, with each player assuming command of one or more Dervish tribes or Anglo-Egyptian brigades.

While "REMEMBER GORDON!" --- THE BATTLE OF OMDURMAN is not, strictly speaking, a beginner's game, the mechanics of play should be familiar to players of modest experience. It is suggested that the bonus game, FALL OF KHARTOUM, and the historical scenario be played first to familiarize players with the game system prior to embarking on the full campaign game.

The designer would also like to point out that English spelling of Arabic names, places, and words is a process of transliteration rather than translation. Spellings thus tend to vary widely according to the source, author, and date of publication.

#v(3pt)
#subsec[1.2) Game Scale:]

Each hexagon of the mapsheet represents approximately 400--440 yards of real terrain and each day turn is the equivalent of two hours of real time. Each counter of infantry and cavalry represents between 400 and 700 men, and each of the gunboats present at the battle has its own counter. The upper echelon of command is represented by individual leader counters for the Anglo-Egyptian force; and leaders plus their retinues for the Dervish army.

#v(5pt)
#hrule()
#v(3pt)

// ── §2 Game Components ───────────────────────────────────────
#sec[2) Game Components:]
#v(2pt)

#subsec[2.1) The Game Maps:]

The Omdurman battle map represents approximately 100 square miles of real territory and includes the area north of Omdurman in which the historical battle took place as well as the dominant terrain features that influenced the course of the battle. Note that the mapsheet also contains the Turn Record Track, Turn Sequence, and Terrain Effects Chart at the top; and the Combat Tables and Howitzer Fire Scattergram in the lower right corner. The large letters "A", "D", "Y", etc. are set-up hexes for the historical scenario only (9.2) and should be ignored in the campaign game. Similarly, the hexsides of the Zariba exist only in the historical scenario and should be considered clear terrain in the campaign game. Note, however, that the Anglo-Egyptian player may "construct" the Zariba in the campaign game if desired (see 5.3). All full hexes of the Omdurman game map are playable, including the seven hexes of the Howitzer Fire Scattergram.

The mini-map for the bonus game, FALL OF KHARTOUM, represents that city as it appeared in 1885. The portion of wall conspicuous by its absence represents the area washed away by the receding White Nile after the flood. Players will note that the north edge of the Khartoum mini-map abuts the middle portion of the Omdurman map south edge. After Khartoum fell, it was destroyed by the Mahdi's troops and lay in ruins in 1898.

#v(3pt)
#subsec[2.2) Play Aids:]

Certain charts and tables are needed to play the game. The Terrain Effects Chart lists all terrain found on the mapsheet and the effect of each type on movement and combat. The Combat Tables describe the range effects on various weapon types and includes the Combat Results Table. Also note the Line of Sight Table on the back of this rulebook. It tells players when certain terrain types block line of sight, thus preventing direct fire attacks on enemy units. Players should become familiar with these various charts and tables prior to the beginning of play.

#v(3pt)
#subsec[2.3) The Units:]

*2.31)* Dervish artillery, gunboats, and forts fire on the "artillery" line of the Dervish Range Effects Table; Jehadia and Danagla units fire on the "rifles" line as does the Isa Zachneih unit. All other Dervish units (including leaders) are armed with spears and swords.

#v(4pt)
#text(size: 7.5pt, style: "italic", weight: "bold")[-Sample Dervish Units:]
#v(3pt)

// Counter diagrams - Dervish
#block(width: 100%)[
  #set text(size: 6.5pt)
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr),
    column-gutter: 3pt,
    row-gutter: 2pt,
    align: center,
    // Row 1: boxes
    counter-box(name: [Isa\ Zachneih], nums: "28·6·9", dark: true, width: 28pt),
    box(width: 28pt, height: 28pt, fill: paper, stroke: 1.5pt + ink, inset: 2pt,
      align(center + horizon,
        stack(spacing: 1pt,
          text(size: 5pt, weight: "bold")[OSMAN\ DIGNA],
          text(size: 6.5pt, weight: "bold")[1·1·15],
        )
      )
    ),
    counter-box(name: [Taiasha], nums: "3·6·9", dark: true, width: 28pt),
    box(width: 28pt, height: 28pt, fill: tinted, stroke: 1.2pt + ink, inset: 2pt,
      align(center + horizon,
        stack(spacing: 1pt,
          text(size: 5pt, weight: "bold")[Danagla],
          text(size: 6.5pt, weight: "bold")[4·6·12],
        )
      )
    ),
    box(width: 28pt, height: 28pt, fill: tinted, stroke: 1.5pt + ink, inset: 2pt,
      align(center + horizon,
        stack(spacing: 1pt,
          text(size: 7pt, weight: "bold")[⊕],
          text(size: 6.5pt, weight: "bold")[4·1·0],
        )
      )
    ),
    // Row 2: labels
    text(size: 6pt)[Combat],
    text(size: 6pt)[Melee],
    text(size: 6pt)[Movement],
    text(size: 6pt)[Tribe],
    text(size: 6pt)[Fort],
  )
]

#v(4pt)
*2.32)* All Anglo-Egyptian units (except gunboats, Maxims, artillery, and leaders) are armed with rifles. Maxims fire on the "Maxims" line of the Anglo-Egyptian Range Effects Table, and artillery and old gunboats fire on the "Artillery" line. New type (named) gunboats may fire on the "Howitzer", "Artillery", and "Maxims" lines of the Range Effects Table. (See 6.52 for the fire capabilities of the "Friendlies".)

#v(4pt)
#text(size: 7.5pt, style: "italic", weight: "bold")[-Sample Anglo-Egyptian Units:]
#v(3pt)

// Counter diagrams - Anglo-Egyptian (green tint)
#let aeg-green = rgb("#c8d8a0")
#let aeg-stroke = rgb("#3a5a20")
#block(width: 100%)[
  #set text(size: 6.5pt)
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr),
    column-gutter: 3pt,
    row-gutter: 2pt,
    align: center,
    box(width: 28pt, height: 28pt, fill: aeg-green, stroke: 1.2pt + aeg-stroke, inset: 2pt,
      align(center + horizon, stack(spacing: 1pt,
        text(size: 5pt, weight: "bold")[21 Lancers],
        text(size: 6.5pt, weight: "bold")[8·5·15],
      ))
    ),
    box(width: 28pt, height: 28pt, fill: aeg-green, stroke: 1.2pt + aeg-stroke, inset: 2pt,
      align(center + horizon, stack(spacing: 1pt,
        text(size: 5pt, weight: "bold")[32 Battery],
        text(size: 6.5pt, weight: "bold")[10·1·7],
      ))
    ),
    box(width: 28pt, height: 28pt, fill: aeg-green, stroke: 1.2pt + aeg-stroke, inset: 2pt,
      align(center + horizon, stack(spacing: 1pt,
        text(size: 4.5pt, weight: "bold")[LORD\ KITCHENER\ Sirdar],
        text(size: 6.5pt, weight: "bold")[0·0·15],
      ))
    ),
    box(width: 28pt, height: 28pt, fill: rgb("#a0b8d0"), stroke: 1.2pt + rgb("#1a3a5a"), inset: 2pt,
      align(center + horizon, stack(spacing: 1pt,
        text(size: 4.5pt, weight: "bold")[GUNBOAT],
        text(size: 6pt, weight: "bold")[4·#super[10]/#sub[16]],
      ))
    ),
    box(width: 28pt, height: 28pt, fill: aeg-green, stroke: 1.2pt + aeg-stroke, inset: 1.5pt,
      align(center + horizon, stack(spacing: 1pt,
        text(size: 4.5pt, weight: "bold")[2B  31],
        text(size: 6.5pt, weight: "bold")[9·5·8],
      ))
    ),
    // labels
    text(size: 6pt)[Cavalry],
    text(size: 6pt)[Artillery],
    text(size: 6pt)[Leader],
    text(size: 6pt)[Old Gunboat],
    text(size: 6pt)[Infantry],
  )
  #v(3pt)
  #grid(
    columns: (1fr, 1fr),
    column-gutter: 3pt,
    row-gutter: 2pt,
    align: center,
    box(width: 28pt, height: 28pt, fill: aeg-green, stroke: 1.2pt + aeg-stroke, inset: 2pt,
      align(center + horizon, stack(spacing: 1pt,
        text(size: 4.5pt, weight: "bold")[Maxim B-1],
        text(size: 6.5pt, weight: "bold")[6·1·12],
      ))
    ),
    box(width: 28pt, height: 28pt, fill: rgb("#a0b8d0"), stroke: 1.2pt + rgb("#1a3a5a"), inset: 2pt,
      align(center + horizon, stack(spacing: 1pt,
        text(size: 5pt, weight: "bold")[Sultan],
        text(size: 6pt, weight: "bold")[5·6½·#super[12]/#sub[18]],
      ))
    ),
    text(size: 6pt)[Maxim Guns\ #text(size: 5.5pt)[(fire twice/turn)]],
    text(size: 6pt)[New Gunboat\ #text(size: 5.5pt)[(Arty + Howitzer)]],
  )
]

#v(5pt)
#subsec[2.4) Game Parts Inventory:]

Your complete copy of "REMEMBER GORDON!" --- THE BATTLE OF OMDURMAN includes:

#set list(marker: "—", indent: 6pt, body-indent: 4pt)
- One 22×28 Battle of Omdurman mapsheet
- One 8½×11 bonus game: FALL OF KHARTOUM mapsheet
- One Rules Booklet
- One die-cut Unit Counter Sheet
- One Campaign Game Order of Appearance Card
- One ten-sided die
- One game box
#set list(marker: "•")

#v(5pt)
#hrule()
#v(3pt)

// ── §3 Getting Started ───────────────────────────────────────
#sec[3) Getting Started:]
#v(2pt)

Spread out the mapsheet on a table. It should lie flat if you backfold it against the scored lines. The Dervish player should sit next to the west edge of the map and the Anglo-Egyptian player opposite him on the east edge. Read through the rules once, looking over the various charts as they are referred to in the various sections. Next, select a scenario and punch out only those unit counters needed to play. Later on, the rest of the unit counters should be punched out, sorted and stored by unit type.

#v(5pt)
#hrule()
#v(3pt)

// ── §4 Turn Sequence ─────────────────────────────────────────
#sec[4) Turn Sequence:]
#v(2pt)

"REMEMBER GORDON!" --- THE BATTLE OF OMDURMAN is played in "Game Turns", each of which has two "Player Turns". The player moving first will vary according to the scenario being played. In the Campaign Game, for example, the Anglo-Egyptian player moves first.

#v(3pt)
#subsec[A) Anglo-Egyptian Player Turn:]
#v(1pt)

#set enum(numbering: "1)", indent: 8pt, body-indent: 4pt)
+ Anglo-Egyptian Movement Phase
+ Fire Combat Phase\
  #h(12pt)a) Dervish Defensive Fire\
  #h(12pt)b) Anglo-Egyptian Offensive Fire\
  #h(24pt)1) Direct Fire Subphase\
  #h(24pt)2) Maxim Second Fire and Howitzer Fire Subphase
+ Anglo-Egyptian Melee Attacks
#set enum(numbering: "1.")

#v(3pt)
#subsec[B) Dervish Player Turn:]
#v(1pt)

#set enum(numbering: "1)", indent: 8pt, body-indent: 4pt)
+ Dervish Movement Phase
+ Fire Combat Phase\
  #h(12pt)a) Anglo-Egyptian Defensive Fire\
  #h(24pt)1) Direct Fire Subphase\
  #h(24pt)2) Maxim Second Fire and Howitzer Fire Subphase\
  #h(12pt)b) Dervish Offensive Fire
+ Dervish Melee Attacks
#set enum(numbering: "1.")

#v(2pt)
*C)* After both players have completed their "Player Turns", advance the "Game Turn" marker to the next hour. Continue in this manner, alternating turns, until the end of the scenario being played.

#v(5pt)
#hrule()
#v(3pt)

// ── §5 Movement Phase ────────────────────────────────────────
#sec[5) Movement Phase:]
#v(2pt)

#subsec[5.1) General Rules:]
#v(1pt)

#rule-par("5.11", [The movement allowances of the various unit types are printed directly on the units (see 2.3). A unit may move up to this printed movement allowance, paying varying costs for different terrain types (see the Terrain Effects Chart).])

#rule-par("5.12", [A player may move as many or as few of his units as desired during each movement phase, limited only by the units' movement allowance, the terrain costs paid in moving from hex to hex, and enemy zones of control (see 5.4).])

#rule-par("5.13", [A unit may never accumulate movement points from turn to turn, nor may a unit transfer unused movement points to other units. A unit's unused movement points in any given turn are considered lost.])

#v(3pt)
#subsec[5.2) Movement Restrictions:]
#v(1pt)

#rule-par("5.21", [In general, naval transport missions are not allowed, i.e. gunboats may not carry any land units. The sole exception is that the Anglo-Egyptian player may transport the surviving units of the "Friendlies" brigade from the east bank of the Nile to the west bank after, and only after, the Dervish east bank unit (Isa Zachneih) has been eliminated. The transport is accomplished in the following sequence: a) on any turn that a "Friendlies" unit and any Anglo-Egyptian gunboat start their turn adjacent, that unit may load onto (i.e. stack with) the gunboat; b) during the Anglo-Egyptian player's next turn the gunboat may move to any Nile hex adjacent to a west bank hex (up to the gunboat's movement allowance); c) on the Anglo-Egyptian player's third turn the "Friendlies" unit may disembark and move normally, paying the normal terrain cost for the first hex entered. The gunboat may also move normally that turn.])

#rule-par("5.22", [With the exception of 5.21, land units may never enter a Nile River hex. Only gunboats may enter and move along Nile River hexes.])

#rule-par("5.23", [Only certain units may enter the walled portion of Omdurman. For the Dervish player these are the Khalifa unit, the three Dervish artillery units, and the Taiasha units (the Khalifa's bodyguard). Any Anglo-Egyptian units that can get to the walled city may enter it (except gunboats and "Friendlies"). Units entering and/or exiting the walled city may only do so through a gate or breach hexside.])

#rule-par("5.24", [Note that gunboats have two movement allowances separated by a slash, e.g. 10/16. The smaller number is the movement allowance when moving upstream, i.e. against the current (the direction of the current is indicated by arrows in the Nile). The larger number is the movement allowance when moving downstream, i.e. with the current. Gunboats may combine movement in both directions, but if they move even one hex upstream, their upstream movement allowance is their maximum movement allowance for that turn, and may not be exceeded.])

#rule-par("5.25", [Dervish forts may not move in any way once placed.])

#rule-par("5.26", [Units must stop their movement immediately upon entering an enemy zone of control (see 5.4).])

#v(3pt)
#subsec[5.3) Constructing the Zariba:]

The Zariba trench and thorn hedge hexsides are built and in place in the historical scenario only. These hexsides are considered clear terrain in the campaign game. The Anglo-Egyptian player may, however, find it useful to construct this defensive position during the campaign game. The Zariba hexsides may only be built in their position as displayed on the mapsheet. Construction procedure is as follows: any Anglo-Egyptian infantry unit that begins and ends the Anglo-Egyptian player turn adjacent to (and on the Nile side of) Zariba hexsides has constructed all Zariba hexsides to which he is adjacent. The constructing unit may neither fire offensively nor melee attack during the turn of construction. Use a blank counter to denote units constructing Zariba hexsides. See 9.23 for defensive benefits and movement restrictions of Zariba hexsides.

#v(3pt)
#subsec[5.4) Zone of Control:]
#v(1pt)

#rule-par("5.41", [All units except Anglo-Egyptian leaders exert a zone of control (hereafter called a ZOC) into their six adjacent hexes (exception: Gunboats exert a ZOC only against enemy gunboats). Disrupted units have no ZOC.])

#rule-par("5.42", [There is no movement point cost to enter or leave an enemy ZOC.])

#rule-par("5.43", [All units must stop when they enter an enemy ZOC and may move no further that turn. In their next movement phase they may withdraw or, if desired, move directly into another enemy ZOC.])

#rule-par("5.44", [ZOCs do not extend into or out of a Nile River hex (exception: Gunboats, see 5.41). ZOCs do not extend across a khor, into a fort, or into a hex inside the walled city across a wall hexside. ZOCs do extend out of a fort (even if unoccupied), and from a walled city hex into an adjacent non-walled city hex across a wall hexside. ZOCs also extend out of (but not into) a walled city hex across a gate hexside. ZOCs extend both ways across a breach hexside. ZOCs also extend out of, but not into, a hut or building hex. In the historical scenario ZOCs extend out of, but not into, the Zariba across a Zariba hexside (also in the campaign game if the Zariba is constructed).])

#v(3pt)
#subsec[5.5) Stacking:]
#v(1pt)

#rule-par("5.51", [No more than four units may occupy a hex, with the exception of leaders and gunboats. All leader units are free stacking, i.e. they may stack in addition to the four-unit-per-hex stacking limitation. Gunboats may not stack with any other unit (Exception: 5.21). Players may move through friendly units at no additional cost in movement points. The stacking limitation applies only at the end of the movement phase and during combat.])

#rule-par("5.52", [The units of different Dervish tribes may not stack together, even if they are the same color (e.g. although both are green, Mulazmin and Jehadia units may not stack with each other).])

#rule-par("5.53", [Leader units are not required to stack. If Dervish leaders elect to stack, however, they may only stack with units of their command (i.e. color). For example, Sheik El Din may only stack with Mulazmins or Jehadias.])

#rule-par("5.54", [*Anglo-Egyptian Brigade Integrity:* All British, Sudanese, and Egyptian infantry units have their brigade designation printed in the upper right corner (e.g. "2B" = 2nd British Brigade; "3E" = 3rd Egyptian Brigade, etc.). In any combat phase in which all four infantry battalions belonging to any Anglo-Egyptian infantry brigade are stacked in the same hex they are said to have brigade integrity. Stacks having brigade integrity receive a +1 modifier to their fire combat die roll provided they all fire at the same enemy-occupied hex. This modifier is in addition to the normal +1 bonus given to all Anglo-Egyptian direct fire attacks (see 6.24).])

#v(5pt)
#hrule()
#v(3pt)

// ── §6 Fire Combat Phase ─────────────────────────────────────
#sec[6) Fire Combat Phase:]
#v(2pt)

#subsec[6.1) General Rules:]
#v(1pt)

#rule-par("6.11", [The fire combat factor of the various unit types is printed directly on the units and is a numerical expression of the unit's fire strength.])

#rule-par("6.12", [Fire combat is always voluntary. A unit is never required to fire at enemy units merely because they are in range or adjacent.])

#rule-par("6.13", [If a unit elects to fire its fire combat factor at an enemy unit that fire combat factor is unitary. A unit's fire combat factor may not be divided up to fire at enemy units on different hexes.])

#rule-par("6.14", [Players may combine fire during fire combat phase, i.e. they may fire at an enemy-occupied hex with as many friendly units as may legally do so, combining all of their fire combat factors into one attack. Note that in any given fire combat phase, however, a combat unit may only fire once and may only be fired at once (exceptions: Maxim guns and gunboats --- see 6.4).])

#rule-par("6.15", [Players may also divide a stack of units in order to fire at different enemy-occupied hexes. Anglo-Egyptian infantry units having brigade integrity, however, do not receive their +1 direct fire modifier unless they all fire at the same enemy-occupied hex (see 5.54).])

#rule-par("6.16", [When halving fire combat strength, always round down each individual unit. For example, an Egyptian brigade of four battalions, each having a printed strength of 9 fire factors, will fire a total of 16 factors when halved. However, a unit's firing strength is never reduced below one by halving.])

#v(3pt)
#subsec[6.2) How To Have Fire Combat:]
#v(1pt)

#rule-par("6.21", [When combat units wish to fire at enemy units, first check the Line of Sight Table to be sure the firing unit can see the target hex (exception: howitzer fire, see 6.64).])

#rule-par("6.22", [Next consult the Range Effects Table to see if the firing unit's fire combat factor is tripled, doubled, normal, halved, or if the target hex is out of range. Add up the total number of fire combat factors firing at the enemy-occupied hex.])

#rule-par("6.23", [Next check the Terrain Effects Chart to see if the enemy-occupied hex fired upon contains any terrain which gives the enemy units in that hex a defensive benefit. If so, apply this negative modifier to the roll of the ten-sided die and cross-index your net die roll on the Combat Results Table with the number of combat factors firing.])

#rule-par("6.24", [All Anglo-Egyptian direct fire attacks receive a +1 modifier to their die roll as an accuracy bonus. In addition, any stack of Anglo-Egyptian infantry having brigade integrity (see 5.54) receives a +1 modifier to their die roll if all four fire at the same enemy-occupied hex. These modifiers are cumulative.])

#v(3pt)
#subsec[6.3) Line of Sight Table:]

This table is located on the back of this rulebook and should be self-explanatory. Locate the terrain type the firing unit is in and cross-index it with the terrain type the target unit is in. Terrain types in the intersecting box block line of sight, with exceptions as footnoted. Also study the "Special LOS Notes" given and remember that (with the exception of howitzer fire --- see 6.64) you can't fire at anything you can't see!

#v(3pt)
#subsec[6.4) Fire Combat Sequence:]

The sequence of fire combat resolution is the same for both defensive and offensive fire combat. During the Dervish player turn the Anglo-Egyptian player executes 6.41 AND 6.42 as defensive fire, after which the Dervish player executes 6.41 as offensive fire. During the Anglo-Egyptian player turn the Dervish player executes 6.41 as defensive fire, after which the Anglo-Egyptian player executes 6.41 AND 6.42 as offensive fire.

#rule-par("6.41", [*Direct Fire Subphase (Dervish and Anglo-Egyptian players):* The firing player must first allocate all of his fire attacks, combining his units' direct fire combat factors in any manner he wishes. After all fire has been allocated, the firing player then resolves his attacks in any order he wishes.])

#rule-par("6.42", [*Maxim Second Fire and Howitzer Fire Subphase (Anglo-Egyptian player only!):* Anglo-Egyptian named gunboats may now fire their artillery factor as howitzer fire (see 6.64) and all Maxim guns may fire a second time. Once again, first allocate all fires, then resolve combat in any order desired. Howitzer fire may be combined with Maxim fire, but only if the howitzer fire impacts in the intended hex (see 6.64). If any Maxim guns did not fire during the Direct Fire Subphase (6.41), they may still only fire once in the Maxim and Howitzer Subphase (6.42). Units firing in this subphase may fire at enemy units fired at in Direct Fire Subphase.])

#v(3pt)
#subsec[6.5) Special Unit Capabilities:]
#v(2pt)
#subsec[6.51) Leader Units:]

Dervish leader units have a fire factor, a melee factor, and a movement factor. They may thus attack, melee, and be eliminated like any other combat unit. Their special benefit is that they stack free.

Anglo-Egyptian leaders have a movement factor only. They are eliminated if a) they are alone in a hex when a Dervish unit occupies or passes through that hex, or b) if all of the combat units a leader is stacked with are eliminated in fire combat or melee. The special function of Anglo-Egyptian leaders is that at least one must survive to occupy the Mahdi's tomb hex if it is to be taken from the Dervish player (see 9.14).

#v(3pt)
#subsec[6.52) Anglo-Egyptian "Friendlies" Brigade:]

These units represent native volunteers in the Anglo-Egyptian army. They fire rifles on the Dervish Range Effects Table and melee with the Dervish melee modifier. They may not enter the walled city of Omdurman (see 5.23). They may be transferred to the west bank (see 5.21).

#v(3pt)
#subsec[6.53) Anglo-Egyptian Royal Engineers (Royal Eng. 5-3-8):]

In addition to normal combat and melee capabilities, this unit may breach a wall hexside or destroy a fort. The procedure is as follows: The Royal Engineers must move adjacent to a fort or a wall hexside and end their movement adjacent. They may neither fire offensively nor melee attack in the ensuing combat phase. If the Royal Engineers remain adjacent to their target and undisrupted at the end of the Anglo-Egyptian player turn, the target is destroyed. Remove a destroyed fort or place a breach marker adjacent to a breached wall hexside. See 6.62 and 6.63 for the effects on adjacent enemy units. The Royal Engineers may perform demolitions while stacked with other Anglo-Egyptian units.

#v(3pt)
#subsec[6.54) Forts:]

The artillery factor of a fort may be fired normally by the owning player, even if it is not stacked with a friendly unit. The melee value of a fort is defensive only, i.e. forts may not melee attack. The −3 defensive value is deducted from the die roll of enemy fire attacks on friendly units stacked inside the fort. Players may not occupy an enemy fort nor advance after combat into an unoccupied enemy fort. There is no additional movement point cost to enter or leave a friendly fort. Forts may be destroyed by: a) artillery fire (see 6.62), b) infantry melee attack (see 7.6), or c) the Royal Engineers (see 6.53). Forts have a ZOC even if unoccupied (see 5.44).

#v(3pt)
#subsec[6.6) Special Artillery Capabilities:]
#v(1pt)

#rule-par("6.61", [Only artillery may fire at gunboats. A result of 3 or more on the combat results table is required to sink a gunboat. Any other result is a miss.])

#rule-par("6.62", [Only artillery may fire at forts. A result of 2 or more on the combat results table is required to eliminate a fort. Any other result is a miss. If the fort contains any enemy units at the instant it is destroyed, one unit is eliminated with the fort.])

#rule-par("6.63", [Only artillery may fire to breach a wall hexside of Khartoum or the walled city of Omdurman. A result of 2 or more on the combat results table is required to breach a wall. Any other result is a miss. The effect of the breach is to negate the wall hexside for line of sight purposes. Place a "BREACH" marker in an adjacent hex so that the arrow points to the breached hexside. If any enemy units are adjacent to the wall hexside at the instant it is breached, one enemy unit is eliminated.])

#rule-par("6.64", [*Howitzer fire:* Five units in the game have howitzer fire capability. These are the five named British gunboats. They may fire their artillery factor as direct fire during the Direct Fire Subphase (see 4 and 6.41) and may then fire the same artillery factor as howitzer fire during the Maxim Second Fire and Howitzer Subphase (see 4 and 6.42). Exception: no howitzer fire is allowed during night game turns. To fire howitzer fire, select any target hex between 4 and 10 hexes from the firing gunboat (ignoring the Line of Sight Table) and roll the ten-sided die twice. The first die roll is the Combat Results Table die roll and the second roll is the impact hex die roll. Refer to the Howitzer Fire Scattergram on the mapsheet for the impact hex. The designated target hex is hit on a roll of 7--10. Once a howitzer fire die roll has been made the results must take effect, even if the fire scatters into a friendly-occupied hex.])

#v(3pt)
#subsec[6.7) Defensive Fire:]

In Defensive Fire phase, all of the non-moving player's units may fire at any of the moving player's units in range, within the limitations imposed by the rules of combat (see 6.1 to 6.6). There is no advance after combat as a result of defensive fires.

#v(3pt)
#subsec[6.8) Offensive Fire:]
#v(1pt)

#rule-par("6.81", [During Offensive Fire phase, the moving player may fire with all of his units capable of firing, up to their maximum range, within the limitations imposed by the rules of combat.])

#rule-par("6.82", [If an enemy-occupied hex is vacated as a result of offensive fire, friendly units may advance after combat into the vacated hex. To be eligible to advance, the friendly units must have participated in the attack and must have been adjacent to the vacated hex. Note that artillery may not advance, nor may units advance across a wall hexside (except at a gate or breach). Units may never advance after combat across a khor.])

#v(5pt)
#hrule()
#v(3pt)

// ── §7 Melee Phase ───────────────────────────────────────────
#sec[7) Melee Phase:]
#v(1pt)

#rule-par("7.1", [The melee strength of all units is printed on the counter. Note that gunboats have no melee strength. Gunboats may neither melee attack nor be melee attacked.])

#rule-par("7.2", [Melee simulates the hand-to-hand fighting of the period. Units may melee attack adjacent enemy units only. Units may not melee attack across a wall hexside, but may melee attack through a gate or breach hexside.])

#rule-par("7.3", [Melee combat is considered simultaneous, so that units eliminated by melee attacks still get a melee combat die roll.])

#rule-par("7.4", [Only infantry, cavalry, camel units, and Dervish leaders may melee attack. All units (except gunboats --- see 7.1) may melee defend.])

#rule-par("7.5", [Cavalry and camel units may retreat two hexes from an infantry melee attack. Note, however, that only one retreat per unit per turn is permitted. Thus, if their retreat places them adjacent to enemy units whose melee attacks have not yet been resolved, those enemy units may elect to attack the retreating unit(s).])

#rule-par("7.6", [If a melee attack eliminates all of the defenders in an adjacent hex, the Dervish player MUST advance into the vacated hex. To be eligible to advance, the Dervish units must have been adjacent to the vacated hex and participated in the melee attack that eliminated the defenders. All surviving eligible Dervish units MUST advance, up to the stacking limit. The Anglo-Egyptian player may advance if desired. Note that only attacking units may advance.])

#rule-par("7.7", [To resolve melee, both the attacker and the defender roll on the Combat Results Table and apply the applicable melee modifier to their die roll. The Dervish player receives a +2 melee modifier, the Anglo-Egyptian player receives a +1 melee modifier. No terrain modifiers are applied to melee combat (Exception: Zariba hexsides in the historical scenario and the campaign game, if constructed --- see 9.23). Melee losses must be taken from meleeing units first!])

#v(5pt)
#hrule()
#v(3pt)

// ── §8 Night Game Turns ──────────────────────────────────────
#sec[8) Night Game Turns:]
#v(1pt)

#rule-par("8.1", [*The effects of night game turns are:* a) all Anglo-Egyptian movement allowances are halved (round down), b) there is no Anglo-Egyptian howitzer fire, and c) all fire ranges are halved for both sides (round down, but range 1 stays range 1). Range effects on fire combat are the same as during day game turns. For example, an Anglo-Egyptian infantry unit firing at night will be doubled at range 1, normal at range 2, and may not fire at range 3 or greater.])

#rule-par("8.2", [*Dervish Desertion Roll:* Once each campaign game, during the first night turn of the game, the Dervish player rolls one die to see how many of his units desert. The roll is made during the movement phase and the number of deserting Dervish units is equal to 1½ times the roll of one die. The Dervish player may choose which units desert by merely removing them from the mapsheet. The KHALIFA unit, gunboats, artillery units, and forts are the only Dervish units that may not be chosen. No victory points are awarded to the Anglo-Egyptian player for deserting Dervishes.])

#v(5pt)
#hrule()
#v(3pt)

// ── §9 The Scenarios ─────────────────────────────────────────
#sec[9) The Scenarios:]
#v(2pt)

#subsec[9.1) The Campaign Game:]
#v(2pt)
#subsec[9.11) Set Up:]
#v(1pt)

#rule-par("9.111", [Dervish player sets up first, moves second.\
— Isa Zachneih infantry unit: anywhere on the east bank, in or south of El Debeba.\
— KHALIFA ABDULLAH: in the walled city of Omdurman, in either palace hex.\
— 3 artillery units, and all Taiasha units: anywhere in the walled city of Omdurman.\
— 17 forts: anywhere on the mapsheet south of the Khor Shambat on the west bank, and/or south of all Halfaya hut hexes on the east bank and Nile River islands.\
— 2 gunboats: any south edge Nile River hexes.])

#rule-par("9.112", [*Dervish reinforcements:* all reinforcements enter on the west edge of the mapsheet, south of the Khor Shambat. Each unit pays the terrain cost of the hex through which it enters, no matter how many units enter through that hex.\
Turn 1) all Baggara, Jaalin, Danagla, Kehena, and Degheim units, and their leaders: YAKUB, SHERIF, and ALI WAD HELU.\
Turn 2) all Hadendowa units and their leader, OSMAN DIGNA.\
Turn 3) all Mulazmin and Jehadia units and their leader, SHEIK EL DIN.])

#rule-par("9.113", [The Anglo-Egyptian player moves first. There are no Anglo-Egyptian units on the mapsheet at start. The GORDON unit is not used in this scenario.\
— The leader units KITCHENER, GATACRE, and HUNTER may enter anytime during the first four game turns and do not count against the 12 units per turn limit. All three leaders must be in play by the end of turn four!\
— All gunboats enter through any north edge Nile River hex, paying one movement point for the first hex entered. The "Friendlies" brigade enters through the Abu Alim hut hex on the east bank, paying eight movement points per unit. All other Anglo-Egyptian units enter through the west bank "ANGLO-EGYPTIAN ENTRANCE AREA", each unit paying one movement point to enter the mapsheet.\
Turn 1) Any three gunboats; "Friendlies" brigade; Egyptian Cavalry; Horse Artillery; and two infantry brigades from the Egyptian Division.\
Turn 2) Any three gunboats plus any twelve land units.\
Turn 3) Any three gunboats plus any twelve land units.\
Turn 4) All remaining Anglo-Egyptian units.])

#v(2pt)
*9.12) Scenario Length:* 6:00 am, Sept. 1 through 8:00 am, Sept. 3, 22 Game Turns.

*9.13) Special Rules:* None.

#v(3pt)
#subsec[9.14) Victory Conditions:]

The Mahdi's Tomb in Omdurman was not only the tallest structure in the entire Sudan in 1898, it was also a Dervish holy shrine. Its loss or destruction would be a severe blow to the Mahdist cause. It is accordingly assigned 25 victory points which are awarded to the player who controls it at the conclusion of play. The Dervish player controls it at the start of play. As a tactical note, the Anglo-Egyptian player will find a decisive victory almost impossible unless he takes the Mahdi's Tomb from the Dervish player. To take the Tomb hex, it must be occupied by one British leader plus any one non-"Friendlies" Anglo-Egyptian combat unit (both undisrupted) at the conclusion of play.

*Additional victory points are awarded as follows:*

#v(2pt)
#set text(size: 8pt)
#grid(
  columns: (1fr, 1fr),
  column-gutter: 6pt,
  [
    *Dervish Player receives:*\
    10 pts: each British leader eliminated\
    10 pts: each British gunboat sunk\
    #h(4pt)1 pt: each "Friendlies" unit eliminated on the east bank side\
    #h(4pt)3 pts: each "Friendlies" unit eliminated on the west bank (see 5.21)\
    #h(4pt)3 pts: each Anglo-Egyptian land unit eliminated.
  ],
  [
    *Anglo-Egyptian Player receives:*\
    No pts: eliminating forts\
    #h(4pt)1 pt: eliminating Isa Zachneih unit\
    10 pts: eliminating KHALIFA ABDULLAH\
    #h(4pt)1 pt: each Dervish unit eliminated, including gunboats, artillery and all other leaders.
  ],
)
#set text(size: 8.8pt)

#v(4pt)
*At the conclusion of play, victory points are totaled and victory levels are assigned according to the following schedule:*

#v(2pt)
// Victory table
#set text(size: 7.5pt)
#table(
  columns: (1fr, auto, 1fr),
  stroke: 0.5pt + ink,
  fill: (col, row) => if row == 0 { rgb("#d8ceac") } else if calc.even(row) { rgb("#ede5cb") } else { paper },
  inset: (x: 4pt, y: 2.5pt),
  align: (left, center, left),
  table.header(
    [*Dervish Player*], [*Victory Level*], [*Anglo-Egyptian Player*],
  ),
  [30+ point superiority], [Decisive], [50+ point superiority],
  [20--29 point superiority], [Tactical], [30--49 point superiority],
  [10--19 point superiority], [Marginal], [15--29 point superiority],
  [1--9 point superiority], [Draw], [1--14 point superiority],
)
#set text(size: 8.8pt)

#v(3pt)
Alternatively, a decisive victory is awarded to the Anglo-Egyptian player if he eliminates every Dervish unit in play (including gunboats and forts). A decisive victory may be awarded the Dervish player if he eliminates all Anglo-Egyptian units on the west bank (excluding gunboats).

#v(5pt)
#hrule()
#v(3pt)

// ── §9.2 Historical Scenario ─────────────────────────────────
#subsec[9.2) The Historical Scenario:]

Players should note that the historical scenario is an exercise in futility for the Dervish player. It is, however, an interesting demonstration of the absolute imbecility of the Khalifa's generalship and vividly shows the superiority of entrenched firepower over traditional tribal arms in the colonial period.

#v(3pt)
#subsec[9.21) Set Up:]
#v(1pt)

#rule-par("9.211", [The Anglo-Egyptian player sets up first, and moves second:\
— Not in play: GORDON leader unit, "Friendlies" brigade.\
— Gunboats start in any Nile River hexes adjacent to the Zariba.\
— Camel Corps, Egyptian Cavalry, and Horse Artillery start in the village of Kerreri hut hexes.\
— All remaining Anglo-Egyptian units set up in the 13 hexes of the Zariba.])

#rule-par("9.212", [The Dervish player sets up second, and moves first.\
— Not in play: Isa Zachneih, gunboats, and forts.\
— All Dervish units must be set up out of the line of sight of all Anglo-Egyptian units.\
— Dervish leaders start on the lettered hexes: A: Ali Wad Helu · D: Sheik El Din · Y: Yakub · K: Khalifa Abdullah · S: Sherif · O: Osman Digna\
— All remaining Dervish units set up within three hexes of their leader as identified by color (e.g. all green units set up within three hexes of Sheik El Din).])

#v(2pt)
*9.22) Scenario Length:* 6:00 am, September 2 through 12:00 noon, September 2. Four game turns.

#v(3pt)
#subsec[9.23) Special Rule: "The Zariba"]
#v(1pt)

#rule-par("9.231", [*Thorn hedge hexsides:* −2 to die roll on all Dervish fire attacks; may not melee across in either direction; may not advance after combat across in either direction.])

#rule-par("9.232", [*Trench hexsides:* −4 to die roll on all Dervish fire attacks vs. entrenched units only; −2 (instead of +2) melee modifier to Dervish units melee attacking an entrenched unit; entrenched units may be fired "over" in both directions (i.e. they do not block line of sight); units are considered "entrenched" if they are directly adjacent to (and on the Nile River side of) a trench hexside.])

#rule-par("9.233", [Units may only enter and/or leave the Zariba via the two end hexsides that connect to the Nile River, paying +2 movement points to cross (Exception: advance after combat across an entrenched hexside).])

#v(3pt)
#subsec[9.24) Victory Conditions:]

Victory Levels are based solely on eliminating enemy units while conserving your own force as much as possible.

#v(2pt)
#set text(size: 7.5pt)
#table(
  columns: (1fr, auto, 1fr),
  stroke: 0.5pt + ink,
  fill: (col, row) => if row == 0 { rgb("#d8ceac") } else if calc.even(row) { rgb("#ede5cb") } else { paper },
  inset: (x: 4pt, y: 2.5pt),
  align: (left, center, left),
  table.header(
    [*Anglo-Egyptian Player*], [*Victory Level*], [*Dervish Player*],
  ),
  [Eliminate 100+ Dervish units], [5 --- DECISIVE], [Eliminate 30+ Anglo-Egyptian units],
  [Eliminate 60--99 Dervish units], [4 --- STRATEGIC], [Eliminate 15--29 Anglo-Egyptian units],
  [Eliminate 45--59 Dervish units], [3 --- TACTICAL], [Eliminate 10--14 Anglo-Egyptian units],
  [Eliminate 30--44 Dervish units], [2 --- MARGINAL], [Eliminate 5--9 Anglo-Egyptian units],
  [Eliminate 0--29 Dervish units], [1 --- DRAW], [Eliminate 0--4 Anglo-Egyptian units],
)
#set text(size: 8.8pt)

#v(3pt)
The lower value victory level is then subtracted from the higher level to determine a player's net victory. For example, if the Anglo-Egyptian player eliminates 104 Dervish units (decisive victory) but loses 18 units doing it (Dervish Strategic), the Anglo-Egyptian player only nets out with a draw (decisive worth 5 minus strategic worth 4 = 1, draw).

#v(5pt)
#hrule()
#v(3pt)

// ── §9.3 Bonus Game ──────────────────────────────────────────
#subsec[9.3) Bonus Game: FALL OF KHARTOUM scenario:]
#v(1pt)

*9.31)* Only the small FALL OF KHARTOUM scenario map is used for this game.

#v(3pt)
#subsec[9.32) Set up:]
#v(1pt)

#rule-par("9.321", [The British player sets up first, moves second: — General GORDON leader unit in the palace. — Two old style (unnamed) gunboats in any Nile River hexes. — Set up in any building or hut hexes of Khartoum, Forts Makran and/or Buri, and/or adjacent to any wall hex: one Egyptian Battalion artillery unit; two British infantry units (represents Caucasian troops); three Egyptian infantry units (represents Cairo "Bazouks"); four Sudan infantry units (represents Sudanese blacks); four "Friendlies" units (represents the Shaggyeh).])

#rule-par("9.322", [Dervish player moves first: enters turn one through any hexes on the south or east edge of the map. — 32 Mulazmin units (represents combined forces of Wad El Nejumi, Abu Girgeh, and Sheik El Obeid); 2 Hadendowa; 6 Kehena; 5 Degheim (represents Mahdi's combined west bank forces); 3 Dervish artillery units.])

#v(2pt)
*9.33) Scenario Length:* Variable, see victory conditions (9.35). Rarely lasts five turns.

#v(3pt)
#subsec[9.34) Special Rules:]
#v(1pt)

#rule-par("9.341", [Turn 1 is always a night turn (see 8.1).])
#rule-par("9.342", [All hexes are playable, including hexes showing up half or less.])
#rule-par("9.343", [Both players must use the Dervish Range Effects Table.])
#rule-par("9.344", [The Dervish player controls the "North Fort" and may fire its guns.])
#rule-par("9.345", [The British gunboats may move from the White Nile to the Blue Nile and vice-versa at an off-board movement cost of six "upstream" movement points.])
#rule-par("9.346", [The GORDON leader unit starts in the palace and may not move during the scenario. He may only be eliminated by a Dervish unit passing through or occupying the palace hex (as normal movement or as advance after combat).])

#v(3pt)
#subsec[9.35) Victory Conditions:]

Victory is determined by how many turns it takes the Dervish player to eliminate the GORDON leader unit and how many Dervish units are eliminated:

— Dervish decisive: eliminate GORDON turn four or sooner.\
— Dervish tactical: eliminate GORDON turn five.\
— Dervish marginal: eliminate GORDON turn six.\
— British marginal: GORDON survives end of turn six.\
— British tactical: GORDON survives end of turn seven.\
— British decisive: GORDON survives end of turn eight.

The Dervish player then loses one victory level if he has lost 16--23 units, two victory levels if he has lost 24--31 units, and three victory levels if he has lost 32 units or more. Thus, for example, a Dervish tactical victory becomes a British marginal victory if the Dervish player eliminates GORDON on turn five, but loses 24 Dervish units doing it!

#v(5pt)
#hrule()
#v(3pt)

// ── §10 Optional Rules ───────────────────────────────────────
#sec[10) Optional Rules (Campaign game only):]
#v(2pt)

It is suggested that the most intriguing employment of the following two options is to permit the Dervish player to have either one or the other, but the Anglo-Egyptian player doesn't know which one until he stumbles onto it. Players are advised that the employment of both optionals in the same game is not recommended.

#v(3pt)
#subsec[10.1) River Mines:]

The Khalifa twice tried (unsuccessfully) to submerge a powerful mine in the Nile to sink or damage British gunboats. This option assumes that both attempts were successful.

#rule-par("10.11", [Prior to the commencement of play the Dervish player secretly records two Nile River hexes to be mined (the mines may not both be placed in the same hex). These hexes must be south of the E--W hexrow in which the Khor Shambat empties into the Nile.])

#rule-par("10.12", [When a British gunboat enters a mined hex, the Dervish player must order it to stop as it has struck a mine. The Dervish player then resolves the effect of the mine's blast by rolling the ten-sided die: 1--4: No effect; 5--7: Gunboat damaged, lost use of its engines and must drift two hexes per turn (with the current) for the rest of the game. No effect on guns or Maxims unless they drift out of range; 8--10: Gunboat sunk!])

#rule-par("10.13", [After both mines have been rolled for, no more are available.])
#rule-par("10.14", [The Dervish player's gunboats may pass through the mined hexes with no ill effect (he knows where the mines are).])

#v(3pt)
#subsec[10.2) River Chain:]

The Khalifa also tried (also unsuccessfully) to string a heavy chain across the Nile to stop or slow down the British gunboats. This option assumes the chain was emplaced.

#rule-par("10.21", [Prior to the commencement of play the Dervish player secretly records a line of river hexes (not exceeding four hexes long) across which the chain is strung. The hexes must be south of the E--W hexrow in which the Khor Shambat empties into the Nile.])

#rule-par("10.22", [When a British gunboat enters a "chained" river hex it must stop and may move no further that turn.])

#rule-par("10.23", [No gunboats (British or Dervish) may cross the chain until it has been sunk by the British player. He may sink the chain by a) having an infantry or cavalry unit spend one complete turn on either riverbank adjacent to a "chained" river hex, or b) firing at the chain with artillery and achieving a 3 or more on the Combat Results Table.])

#v(5pt)
#hrule()
#v(3pt)

// ── §11 Historical Notes ─────────────────────────────────────
#sec[11) Historical Notes:]
#v(2pt)

In 1881 Mohammed Ahmed Ibn Al-Sayid Abdullah, the son of an obscure carpenter in the hinterlands of the Sudan, proclaimed himself the "Mahdi" --- the Messiah of the Islamic faith. His timing was propitious indeed. Since the early 1820's a corrupt Egypt, with the Sultan of Turkey's blessing, had incessantly raped the Sudan, taking ivory and flooding the slave markets with some half million captured Sudanese blacks. By 1880, nearly 40,000 Egyptian troops occupied outposts scattered throughout the Sudan, enforcing Egypt's hold on this lucrative ivory and slave trade and squeezing the native population dry through vicious and corrupt tax officials. All was controlled from Khartoum via the office of Governor General of the Sudan. The title had been held by a succession of individuals, including General Charles Gordon, whose appointment was an attempt to reinstate some rudimentary justice in the Sudan after France and Britain assumed joint political control of a bankrupt Egypt.

By 1881, however, Gordon's term had expired and a new Governor General, again corrupt and incompetent, attempted to deal with the Mahdi. Declining to come to terms with the representatives of Egypt's "benevolent civilization", the Mahdi butchered an armed force dispatched to arrest him in October, 1881. Three months later, the Dervishes (members of a fundamentalist sect following the Mahdi) again ambushed and slaughtered a punitive force of 1400 Egyptian troops sent against him. The effect of these two actions on the native Sudanese was electrifying and they flocked by the thousands to join his holy war and cast out their oppressors.

Egypt, in the meantime, was attempting to throw off Turkish rule and Britain, fearing a revolution and loss of Christian lives, ordered the Mediterranean Squadron to Alexandria in May, 1882. When Turkey refused to intervene, British Marines and Bluejackets went ashore and restored order in Alexandria. Britain next sent General Sir Garnet Wolseley to deal with the rebellious Egyptian army who still controlled Cairo and most of the Egyptian countryside. By mid-September Wolseley had subdued Egypt, winning the battles of Mahsama and Tel-el-Kabir. Thus, by the end of 1882, Britain unwillingly assumed responsibility for Egypt, protecting her communication lines to India in the bargain.

The Sudan, however, was another matter. In England, prime minister William Gladstone was opposed to any activity which would take British troops outside Egypt's borders. But London was very far away and the simple fact of the matter was that Egyptian security was dependent on a subjugated Sudan. Accordingly, the Egyptian army was reorganized along European lines under British officers and undertook its first major effort under General William Hicks, better known as Hicks Pasha, in February of 1883.

The Mahdi, in the meantime, was taking advantage of the situation in Egypt to expand his influence in the Sudan. Each success brought more recruits and the rebellion grew. He crushed an Egyptian force sent against him from Khartoum in March, 1882, and butchered another expedition in January, 1883.

Hicks Pasha marched his Egyptian army to Khartoum and, after a brief rest, moved out again on June 26th, 1883. After some four months of marching and several minor engagements, Hicks and his army met their end on November 4th at Kashgeil, about 225 miles southwest of Khartoum. The Mahdi's horde attacked on the 3rd and 4th and finally broke the square, the slaughter itself taking until the 5th to complete. Next into the fray was Valentine Baker Pasha, who led another Egyptian expedition into the eastern Sudan via the Red Sea in early 1884. It was hacked to pieces early in February when one of the Mahdi's Emirs, Osman Digna, again broke the square with his Hadendowa troops, the notorious "Fuzzy-Wuzzies".

With Khartoum itself now menaced, London finally reacted and ordered General Sir Gerald Graham into the Sudan with a detachment from the British Army of Occupation in Egypt. On February 29th he engaged a portion of Osman Digna's forces at El-Teb, near Suakim in the eastern Sudan, and won by a narrow margin when his square formation held. Seeking to expand on this victory, General Graham ordered Osman Digna and his chiefs to disperse their forces and surrender themselves. When they refused, the British expedition again marched against the Dervishes on March 12th. This time, however, the "Fuzzy-Wuzzies" broke the square, a British square. Although the broken square rallied and the Dervishes were finally beaten off, it was another narrow victory. The Mahdi still ruled the vastness of the Sudan with the few remaining Anglo-Egyptian garrisons like tiny islands in a hostile ocean. Eyes on both sides now turned toward Khartoum.

However distasteful to his politics, prime minister Gladstone was now forced to take some action on behalf of the troops and civilians in the Sudan. Abhorring the cost of a major imperial expedition, the decision was made to evacuate and one man was sent to accomplish it, General Sir Charles Gordon. Upon arrival at Khartoum he again assumed the role of Governor General of the Sudan and announced to the startled population (who had expected an army) that he came without troops, but with God on his side. Supremely self-confident, he showed no intention of evacuating the city and instead set about reinforcing the defenses and recruiting native volunteers. Unimpressed with Gordon's offers of reconciliation, the Mahdi responded by investing Khartoum on March 12th, 1884. The siege was, however, only effective on land, as Gordon's little gunboats continued to steam up and down the Nile transferring women, children and wounded to Berber, north of the sixth cataract. In Khartoum itself, Gordon took personal charge of everything, imposing a rationing system, printing his own paper money and awarding his own medals.

When Berber fell to the Mahdi's troops in May of 1884, Khartoum's isolation was virtually complete, and yet it continued to hold out. By August the public outcry in England and the British press compelled Gladstone to take further action for the relief of General Gordon and the Sudan. The action took the form of an expeditionary force under Sir Garnet Wolseley, who arrived in Egypt September 9th and had the relief force under way by October 5th.

Progress was unfortunately slow. So slow that by December Wolseley had only progressed some 150 miles to the third cataract. Beyond lay the Mahdi's Dervish-infested territory and three more cataracts before the column would be anywhere near Khartoum, whose time was running out. A desert strike force of 1800 men was thus detached to move overland and set out early in January. It was attacked on the 17th near Abu Klea and disaster was narrowly averted when the Dervishes again broke a British square but were unable to exploit because the baggage animals were packed tightly in the center. On the 19th, the Dervishes struck again at Abu Kru but were repulsed, and the strike force proceeded without further incident to the Nile.

Due to casualties, command of the strike force had passed to a Colonel Wilson, a staff officer with little combat experience. Accordingly, when four of Gordon's steamers reached him on January 21st, he declined to embark his troops, instead feeling they needed a three day rest to recover and build a defensive position.

In Khartoum, meanwhile, the garrison became daily more weakened by hunger and fatigue. If Gordon's disinclination to evacuate seems strange, then even stranger was the Mahdi's apparent reluctance to apply the coup de grace to the city. Even after the inevitable end became painfully obvious, he continued to offer Gordon honorable surrender terms, safe passage, and other concessions. Gordon, however, remained adamant. He had apparently prepared himself a martyr's place in history and would not be dissuaded from it except by the total capitulation of the Mahdi and his followers. Then the Mahdi was informed that the relief expedition was within a few days of Khartoum and decided the garrison must be taken at once. Thus it was that in the pre-dawn hours of January 25th, 1885, some 20,000 Dervishes poured through a gap in Khartoum's outer defenses where the receding White Nile had eroded away a section of wall. The garrison was slaughtered, Gordon among them (FALL OF KHARTOUM scenario --- 9.3). Three days later (Col. Wilson's three days of rest?) the steamers carrying the advance guard of the strike force came within sight of Khartoum. Seeing only smoking ruins, they turned around and steamed back downstream to bring the news to the main body. Queen Victoria voiced the feelings of the nation when she recorded in her diary: "The government alone is to blame".

The relief column withdrew back into Egypt, and the fall of Khartoum thus effectively eliminated Britain's presence in the Sudan for the next eleven years, leaving that vast hinterland to the Mahdist empire. The Mahdi died in June of 1885 and was succeeded by the Khalifa, Abdullah the Taiasha, a chief of the Baggaras. The Khalifa made Omdurman his capital and expanded it from a few mud huts in 1885 to a vast, sprawling fifteen square mile urban slum by 1898. It housed the Dervishes' holiest shrine, the Mahdi's Tomb, as well as the palace and other structures in a walled city within a city.

By 1896 the spread of Mahdism led to British concern for the security of Egypt. In a move ostensibly made to take pressure off an Italian outpost on the Abyssinian border, London ordered an expedition into Dervish territory in the northern Sudan. It was led by General Herbert Kitchener, Sirdar (commander) of the Egyptian army. Kitchener had been a major in the Khartoum relief expedition and had never forgotten the rage and shame he felt when that force withdrew without attacking the Mahdi's army. An obsession to avenge Gordon's death stayed with him over the intervening years, so that he welcomed the instructions to move on the Sudan. To free himself from total dependence on the Nile for transportation, the Sudan Military Railroad was planned and overland construction begun. By July of 1896, Kitchener was underway. Progress was slow but steady, with the army halting periodically for the railway to catch up. Following infrequent skirmishing with the Dervishes, Kitchener's Egyptian Division under General Hunter re-occupied Berber in July of 1897. The balance of that year was spent reorganizing and re-supplying the army while again waiting for the railway to catch up.

If 1897 was the year of consolidation and organization, 1898 was the year in which those efforts bore fruit. Reinforced with a British brigade, the Sirdar's army was again on the move in March, 1898. After fighting three minor engagements during March and early April, the army (now the Anglo-Egyptian army) found itself confronted by a large Dervish force under Mahmud, one of the Khalifa's few remaining competent generals. Mahmud had entrenched his force inside a circular defensive zariba of camel thorn, with his back on the dry bed of the river Atbara, a strong defensive position. Mahmud, however, had not taken the new British heavy artillery into account and, after an hour and a half of heavy bombardment, the Sirdar's army went in, led by the Cameron Highlanders. Forty-five minutes later 3,000 Dervishes were dead at a loss to Kitchener of 80 men killed, and Mahmud was a prisoner. The way to Omdurman was open!

By mid-April the railroad had reached the Nile below Berber, bringing with it the new shallow draft gunboats designed specifically for river campaigns. The sections of these new iron monsters were assembled and floated in the Nile. One hundred and forty feet long by twenty-four feet wide and drawing only thirty-nine inches of water, they were formidable concentrations of firepower with their 12 pounders, 6 pounders, and Maxim guns on the upper deck, and 4 inch howitzers on the gun deck. By August 17th all was in readiness and, reinforced with a second British brigade, Kitchener marched steadily south, arriving at the little mud village of Kerreri on September 1st (CAMPAIGN GAME scenario --- 9.1).

The Khalifa, Abdullah the Taiasha, in the meantime, had not been idle. Throughout the Spring and Summer of 1898, the Sudan experienced a hectic and frantic mobilization as the leading Emirs of the empire gathered the faithful to the Jihad, or holy war. Estimates of the response vary widely, but it seems likely that some 60--70,000 warriors answered the call and assembled on the plains of Kerreri, north of Omdurman. To guard the approaches to the city, seventeen forts were constructed and armed with old artillery pieces. The few guns available, old Remingtons and brass muzzle loaders using home-made cartridges, were issued to the Jehadia (commanded by the Khalifa's son, Sheik El Din) and the Danagla. The rest of the troops carried swords and spears.

Dawn of September 2nd saw the Sirdar and his Anglo-Egyptian army positioned inside a rough semi-circular formation protected by a zariba of thorn hedge and trenches. His back and flanks were on the Nile and guarded by the gunboats. At dawn the cavalry had gone out, but by 6:30 they were back in. Then they came --- the Dervishes in their thousands and tens of thousands pouring over the ridges of the Jebel Surgham and the Kerreri Hills (HISTORICAL scenario --- 9.2).

By 11:30 the battle was virtually over. 10,000 Dervishes dead --- 20,000 wounded, over ¼ of whom would die unattended in the blazing sun during the next two days --- 5,000 prisoners --- all at a cost of just over 400 killed and wounded in the Sirdar's army. The rest of the story is known to the most casual student of the battle: the 21st Lancers win their first battle streamer and three Victoria Crosses in one of history's last great knee to knee cavalry charges --- Maxwell and the XIII Sudanese first to enter the city --- 30,000 captured cooks and concubines for whom Kitchener declared he had no use in either capacity --- the unused Gatling guns and Nordenfeldts found in the Khalifa's arsenal --- the repulsive battlefield with its several hundred acres of suffering wounded and bloating corpses piled around the flags of their dead Emirs --- 30,000 Dervish survivors of the battle melted away in the desert, never to rise again. Rarely in modern history has an army and a civilization been so thoroughly crushed, consuming the efforts of half a generation. Fifty-eight years later, Britain would withdraw permanently from Egypt and the Anglo-Egyptian Sudan.

Two days after the battle, September 4th, 1898, Kitchener held a memorial service for General Sir Charles Gordon in front of the ruins of the Governor General's palace in Khartoum. He described it in moving phrases in a letter to Queen Victoria, who recorded in her diary: "Surely now he is avenged".

] // end columns()

// ══════════════════════════════════════════════════════════════
//  REFERENCE TABLES PAGE (after body)
// ══════════════════════════════════════════════════════════════
#thick-rule
#v(4pt)

// Two-column reference section
#columns(2, gutter: 14pt)[

// ── Range Effects Table ──────────────────────────────────────
#block(stroke: 0.5pt + ink, inset: 0pt, width: 100%)[
  // Side box for artillery-only notes and melee modifiers
  #grid(
    columns: (auto, 1fr),
    // Left: artillery rules + melee mods
    block(
      fill: rgb("#e0d6b8"),
      inset: (x: 4pt, y: 3pt),
      height: 100%,
      [
        #set text(size: 7pt)
        *ARTILLERY\ ONLY* #text(size: 6pt)[(see 6.6)]\
        #v(2pt)
        To sink a\ gunboat: *3+*\
        To breach a\ wall hexside: *2+*\
        To destroy\ a fort: *2+*\
        #v(4pt)
        *MELEE\ MODIFIERS:*\
        Dervish: *+2*\
        Anglo-Egypt: *+1*
      ]
    ),
    // Right: range table
    block(inset: 0pt, width: 100%)[
      #set text(size: 7pt)
      #table(
        columns: (auto, auto, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr),
        stroke: 0.4pt + ink,
        fill: (col, row) => {
          if row == 0 { rgb("#d8ceac") }
          else if col == 0 or col == 1 { rgb("#e5dbbf") }
          else { paper }
        },
        inset: (x: 2pt, y: 1.5pt),
        align: center,
        // Header
        table.cell(colspan: 2)[*RANGE IN HEXES*],
        [*1*],[*2*],[*3*],[*4*],[*5*],[*6*],[*7*],[*8*],[*9*],[*10*],
        // Dervish divider
        table.cell(colspan: 12, fill: rgb("#c8b880"))[
          #align(center, text(size: 6.5pt, weight: "bold")[DERVISH])
        ],
        [Spears],[],[x1],[],[],[],[—],[],[],[],[],[],
        [Rifles],[],[x1],[],[x½],[],[],[—],[],[],[],[],
        [Artillery],[],[x2],[],[x1],[],[],[x½],[],[],[—],[],
        // Anglo-Egyptian divider
        table.cell(colspan: 12, fill: rgb("#b8a868"))[
          #align(center, text(size: 6.5pt, weight: "bold")[ANGLO-EGYPTIAN])
        ],
        [Rifles],[],[x2],[x1],[],[x½],[],[—],[],[],[],[],
        [Maxims],[],[x2],[x1],[],[x½],[],[—],[],[],[],[],
        [Artillery],[],[x3],[x2],[],[x1],[],[],[x½],[],[—],[],
        [Howitzer],[],[—],[],[],[],[x½],[],[],[],[],[],
      )
    ]
  )
]

#v(6pt)

// ── Combat Results Table ─────────────────────────────────────
#align(center, text(size: 9pt, weight: "bold", tracking: 1pt)[COMBAT RESULTS TABLE])
#v(1pt)
#align(center, text(size: 7.5pt, style: "italic")[--- Die Roll ---])
#v(2pt)

#set text(size: 7.5pt)
#table(
  columns: (auto, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, auto),
  stroke: 0.5pt + ink,
  fill: (col, row) => {
    if row == 0 { rgb("#d8ceac") }
    else if col == 0 or col == 11 { rgb("#ddd4b0") }
    else { paper }
  },
  inset: (x: 2.5pt, y: 2pt),
  align: center,
  // Header
  [*Total\ Combat\ Factors\ Firing*],
  [*1*],[*2*],[*3*],[*4*],[*5*],[*6*],[*7*],[*8*],[*9*],[*10*],
  [*Total\ Combat\ Factors\ Firing*],
  // Rows
  [*1--5*],  [—],[—],[—],[D],[D],[1],[1],[1],[2],[2], [*1--5*],
  [*6--10*], [—],[—],[D],[D],[1],[1],[1],[2],[2],[2], [*6--10*],
  [*11--15*],[—],[D],[D],[1],[1],[1],[2],[2],[2],[3], [*11--15*],
  [*16--20*],[D],[D],[1],[1],[1],[2],[2],[2],[3],[3], [*16--20*],
  [*21--25*],[D],[1],[1],[1],[2],[2],[2],[3],[3],[3], [*21--25*],
  [*26--30*],[1],[1],[1],[2],[2],[2],[3],[3],[3],[4], [*26--30*],
  [*31--35*],[1],[1],[2],[2],[2],[3],[3],[3],[4],[4], [*31--35*],
  [*36--40*],[1],[2],[2],[2],[3],[3],[3],[4],[4],[4], [*36--40*],
  [*41+*],   [2],[2],[2],[3],[3],[3],[4],[4],[4],[5], [*41+*],
)
#set text(size: 8.8pt)
#v(2pt)
#set text(size: 7pt)
*D = ½ of units disrupted (round up).* Modified die rolls of less than 1 treated as 1, more than 10 treated as 10.\
+1: All Anglo-Egyptian Direct Fire Attacks #h(1fr) +1: Anglo-Egyptian Brigade Integrity
#set text(size: 8.8pt)

#v(6pt)

// ── Explanation of Combat Results + Disrupted Units ──────────
#grid(
  columns: (1fr, 1fr),
  column-gutter: 8pt,
  block(stroke: 0.5pt + ink, inset: (x: 5pt, y: 4pt))[
    #set text(size: 7.5pt)
    *EXPLANATION OF COMBAT RESULTS:*
    #v(2pt)
    — = miss, no effect\
    D\* = ½ (round up) of the units in the target hex are disrupted (inverted).\
    \# = That many units in the target hex are eliminated, i.e. removed from play.
  ],
  block(stroke: 0.5pt + ink, inset: (x: 5pt, y: 4pt))[
    #set text(size: 7.5pt)
    *\*DISRUPTED UNITS:* Have no ZOC; may not move; may not fire offensively or defensively; may not melee; are turned face up at the end of the owning player's turn.
  ],
)

#colbreak()

// ── Line of Sight Table ──────────────────────────────────────
#align(center)[
  #line(length: 100%, stroke: 1pt + ink)
  #v(2pt)
  #text(size: 9pt, weight: "bold", tracking: 0.5pt)[6.3) LINE OF SIGHT TABLE]
  #v(1pt)
  #text(size: 7.5pt, style: "italic")[---Target Unit's Terrain---]
  #v(2pt)
  #line(length: 100%, stroke: 0.5pt + ink)
]
#v(3pt)

#set text(size: 7pt)
#table(
  columns: (auto, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr),
  stroke: 0.5pt + ink,
  fill: (col, row) => {
    if row == 0 or row == 1 { rgb("#d8ceac") }
    else if col == 0 { rgb("#e5dbbf") }
    else { paper }
  },
  inset: (x: 2pt, y: 2pt),
  align: center,
  // Header row 1
  table.cell(rowspan: 2, align: center + horizon)[
    #text(size: 6.5pt, style: "italic")[Terrain types in boxes block LOS!\ _(see footnotes)_]
  ],
  table.cell(colspan: 2)[*GROUND (c)*],
  table.cell(colspan: 2)[*ROUGH (b)*],
  table.cell(colspan: 2)[*HILLTOP*],
  // Header row 2
  [Units\ Huts (1)\ Wall (b)], [Rough\ Trees (1)],
  [Units (3,6)\ Huts (1,3)\ Wall (b)], [Crest (2)\ Trees (1)\ Hilltop],
  [Units (3)\ Huts (1,3)], [Crest (3)\ Hilltop],
  // Data rows
  [*GROUND (c)*],
  [], [],
  [Units (4,5)\ Huts (1,4)\ Wall (b)], [Crest (2)\ Trees (1)\ Hilltop],
  [Units (3)\ Hilltop], [Crest (2,3)],
  [*ROUGH (b)*],
  [Units (4,5)\ Huts (1,4)\ Wall (b)], [Crest (2)\ Trees (1)\ Hilltop],
  [Units (7)\ Hilltop], [Crest (2)],
  [Units (3)\ Hilltop], [Crest (2,3)],
  [*HILLTOP*],
  [Units (4)\ Huts (1,4)], [Crest (4)\ Hilltop],
  [Units (4)\ Hilltop], [Crest (2,4)],
  table.cell(colspan: 2)[Units on a\ Hilltop],
)
#set text(size: 8.8pt)

#v(4pt)
#grid(
  columns: (1fr, 1fr),
  column-gutter: 8pt,
  [
    #set text(size: 7pt)
    *Footnotes:*\
    1) If fire through more than two.\
    2) Not blocked if firing units and/or target units are adjacent to all crest hexsides fired through.\
    3) If closer to firing unit, or halfway between.\
    4) If closer to target unit, or half way between.\
    5) If adjacent to, and at same level as, firing unit.\
    6) If adjacent to, and at same level as, target unit.\
    7) LOS not blocked if at lower level.
  ],
  [
    #set text(size: 7pt)
    *Special LOS Notes:*\
    a) Gunboats and forts never block LOS.\
    b) Gunboats and units inside a walled city adjacent to a wall hexside are considered at rough level for LOS purposes.\
    c) Forts are considered at ground level for LOS purposes.\
    d) Units may fire down, i.e. along the length of, one wall hexside.\
    e) Firing along the length of a crest hexside has the same effect on LOS as firing through a crest hexside.\
    f) Terrain types are considered to fill their entire hex for LOS purposes.
  ]
)

#v(6pt)

// ── Terrain Effects Chart ─────────────────────────────────────
#thick-rule
#v(3pt)
#text(size: 8pt, weight: "bold", tracking: 0.5pt)[TERRAIN EFFECTS CHART]
#v(2pt)

#set text(size: 6.5pt)
#table(
  columns: (auto, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr),
  stroke: 0.4pt + ink,
  fill: (col, row) => {
    if row == 0 { rgb("#d8ceac") }
    else if col == 0 { rgb("#e5dbbf") }
    else { paper }
  },
  inset: (x: 2pt, y: 1.5pt),
  align: center,
  [*Terrain\ Type*],
  [*Clear*],[*Rough*],[*Trees*],[*Swamp*],[*Nile*],[*Hilltop*],[*Huts*],[*Building*],[*Road*],[*Khor*],[*Crest*],[*City\ Wall*],[*Thorn\ Hedge*],[*Trench*],
  [*Movement\ Point Cost*],
  [1],[3],[1],[3],[Gunboats only: 1],[1],[3],[3],[1],[+5],[+1],[+1\ gate/breach only],[Hist. scenario only],[Hist. scenario only],
  [*Effect on\ Combat*],
  [None],[None],[None],[None],[None],[None],[−1 to attacker],[−3 to attacker],[Per other terrain in hex],[May not melee across],[−1 to attacker],[−4 attacker;\ see LOS],[−2 Dervish fire;\ no melee across],[−4 Dervish fire (entr.);\ −2 melee mod.],
)
#set text(size: 8.8pt)

#v(6pt)

// ── Campaign Game Order of Appearance ────────────────────────
#line(length: 100%, stroke: 2pt + ink)
#v(2pt)
#align(center, text(size: 9pt, weight: "bold", tracking: 1pt, upper[Campaign Game Order of Appearance]))
#v(2pt)
#line(length: 100%, stroke: 1pt + ink)
#v(2pt)

#set text(size: 7pt)
#table(
  columns: (1fr, auto, 1fr, 1fr, 1fr),
  stroke: 0.5pt + ink,
  fill: (col, row) => {
    if row == 0 { rgb("#c8b880") }
    else if col == 2 { rgb("#e8dfc0") }
    else if calc.even(row) { rgb("#ede5cb") } else { paper }
  },
  inset: (x: 4pt, y: 3pt),
  align: (left, center, left, left, left),
  // Header
  table.cell(colspan: 2, align: center)[*Anglo-Egyptian*],
  [*Turn*],
  table.cell(colspan: 2, align: center)[*Dervish* #text(size: 6pt)[(see 9.111 for set-up units)]],
  // Turn 1
  [3 Gunboats; "Friendlies" Brigade (5 units); Egyptian Cavalry & Horse Artillery],
  [Two Infantry Brigades from Egyptian Division (8 units)],
  [*TURN 1*\ 6:00 am\ Sept 1],
  [*YAKUB*\ 12 Baggara\ 25 Jaalin],
  [*SHERIF*\ 4 Danagla\ *ALI WAD HELU*\ 6 Kehena  5 Degheim],
  // Turn 2
  [3 Gunboats],
  [12 Land Units],
  [*TURN 2*\ 8:00 am],
  table.cell(colspan: 2)[*OSMAN DIGNA*\ 12 Hadendowa],
  // Turn 3
  [3 Gunboats],
  [12 Land Units],
  [*TURN 3*\ 10:00 am],
  table.cell(colspan: 2)[*SHEIK EL DIN*\ 32 Mulazmin   24 Jehadia],
  // Turn 4
  table.cell(colspan: 2, align: center)[All Remaining Anglo-Egyptian Units],
  [*TURN 4*\ 12:00 noon],
  table.cell(colspan: 2, fill: rgb("#d8ceac"), align: center)[
    #text(font: "Overpass", size: 10pt, weight: "bold")[Remember Gordon!]
    #v(2pt)
    #text(size: 6pt)[Copyright 1982 © Phoenix Enterprises, Ltd.]
  ],
)
#set text(size: 8.8pt)

] // end columns()

// ── Credits ──────────────────────────────────────────────────
#v(4pt)
#thick-rule
#v(4pt)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 16pt,
  block(stroke: 0.5pt + ink, inset: (x: 7pt, y: 5pt))[
    #set text(size: 8pt)
    #align(center, text(size: 9pt, weight: "bold", tracking: 1pt)[CREDITS])
    #v(3pt)
    *Game Design:* Peter Bertram\
    *Development:* Peter Bertram and Fred Chatham\
    *Graphic Arts:* Mike Williford, Graphics Unlimited\
    *Rules Editing:* Randall Mac Innis\
    *Components Design:* Peter Bertram\
    *Box Art:* George I. Parrish Jr.\
    *Production Coordinator:* Fred Chatham\
    *Printed By:* Seiz Printing Inc.\
    *Playtesters:* Martin Davisson, Dave Ferguson, Ron Glass, Randall Mac Innis, Henry Robinette, Michael Sincavage
  ],
  block(stroke: 0.5pt + ink, inset: (x: 7pt, y: 5pt))[
    #set text(size: 8pt)
    Questions concerning the rules will be answered if they are a) phrased to be answered "yes" or "no", and b) accompanied by a stamped, self-addressed envelope. General comments about the game are always welcome.
    #v(6pt)
    Address all correspondence to:\
    #h(12pt)Phoenix Enterprises, Ltd.\
    #h(12pt)P.O. Box 81192\
    #h(12pt)Chamblee, Ga. 30366
  ],
)

#v(6pt)
#align(center, text(size: 7.5pt, style: "italic")[Copyright 1982 © Phoenix Enterprises, Ltd.])

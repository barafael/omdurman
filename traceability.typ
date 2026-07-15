#set page(paper: "a4", margin: (top: 2cm, bottom: 2cm, left: 2.5cm, right: 2cm))
#set text(font: ("EB Garamond", "Libertinus Serif", "DejaVu Serif"), size: 10pt)
#set par(justify: true, leading: 0.5em)
#set heading(numbering: none)

#show raw.where(block: true): set text(font: ("DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"), size: 7.5pt)
#show raw.where(block: false): set text(font: ("DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"), size: 8.5pt)
#show raw.where(block: true): block.with(fill: luma(248), inset: 0.4em, radius: 2pt)

#show heading.where(level: 1): it => {
  pagebreak()
  v(1em)
  block(stroke: (left: 3pt + luma(80)), inset: (top: 0.2em, bottom: 0.2em, left: 0.6em, right: 0.3em), it)
}

#show heading.where(level: 2): it => {
  v(0.6em)
  it
}

#let status-tag(status) = {
  let (bg, fg) = if status == "implemented" {
    (green.transparentize(70%), green.darken(30%))
  } else if status == "descriptive" {
    (blue.transparentize(70%), blue.darken(30%))
  } else if status == "implicit" {
    (yellow.transparentize(70%), yellow.darken(40%))
  } else {
    (luma(85), luma(40))
  };
  box(
    fill: bg, inset: (left: 0.4em, right: 0.4em, top: 0.1em, bottom: 0.1em),
    radius: 3pt, text(fill: fg, size: 8pt, weight: "bold", status)
  )
}

#let root = "/home/rafael/omdurman-old"

#let vscode-link(rel, line) = {
  let abs = root + "/" + rel
  link("vscode://file/" + abs + ":" + str(line))[
    #text(size: 9pt, fill: blue.darken(20%), rel + ":" + str(line))
  ]
}

#let github-link(rel, line) = {
  let url = "https://github.com/barafael/omdurman/blob/HEAD/" + rel + "#L" + str(line)
  link(url)[
    #text(size: 8pt, fill: luma(100), "GH:" + rel + ":" + str(line))
  ]
}

#let progress-bar(done, total) = {
  let filled = "█" * done
  let empty = "░" * (total - done)
  text(font: ("DejaVu Sans Mono", "Liberation Mono"), size: 8pt)[
    #text(fill: green.darken(20%))[#filled]#text(fill: luma(180))[#empty] #done/#total implemented
  ]
}
#align(center, text(size: 18pt, weight: "bold", "Traceability Matrix"))
#align(center, text(size: 10pt, "REMEMBER GORDON! -- Rulebook ⇌ Implementation Mapping"))
#align(center, text(size: 9pt, fill: luma(120), "Generated from `docs/traceability.toml`"))
#v(2em)
#heading(level: 1, "Overview") <sect-overview>
#v(0.3em)
#table(
  columns: (1fr, 1fr, 1fr, 1fr),
  stroke: 0.4pt + luma(190),
  [*Implemented*], [*Descriptive*], [*Implicit*], [*Out-of-scope*],
  [#text(fill: green.darken(20%))[76]], [#text(fill: blue.darken(20%))[10]], [#text(fill: yellow.darken(30%))[6]], [16],
)
#v(0.3em)
#text(size: 9pt)[Total mappings: 108 · Total impl sites: 223]
#v(1em)
#outline(title: [Table of Contents])
#pagebreak()
#progress-bar(0, 2)
#heading(level: 1, "§1 -- Introduction") <sect-1>
#heading(level: 2, "§1.1 -- General Comments") <sect-1-1>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120))[manual page 1]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[General Comments

"REMEMBER GORDON!" — THE BATTLE OF OMDURMAN is a simulation of the final battle in Great Britain's two-year campaign to reassert her presence in the Sudan (1896–1898). Fought September 2nd, 1898, Omdurman finally broke the back of the fanatical Dervish rebellion and gained Britain a million square miles of desolate territory and two million impoverished subjects. With two players, one assumes the role of Herbert Kitchener, Sirdar (CIC) of the Anglo-Egyptian army; the other player becomes the Khalifa, Abdullah the Taiasha, absolute ruler of the Dervishes. The game is also suited for multi-player participation, with each player assuming command of one or more Dervish tribes or Anglo-Egyptian brigades.

While "REMEMBER GORDON!" — THE BATTLE OF OMDURMAN is not, strictly speaking, a beginner's game, the mechanics of play should be familiar to players of modest experience. It is suggested that the bonus game, FALL OF KHARTOUM, and the historical scenario be played first to familiarize players with the game system prior to embarking on the full campaign game.

The designer would also like to point out that English spelling of Arabic names, places, and words is a process of transliteration rather than translation. Spellings thus tend to vary widely accordingly to the source, author, and date of publication.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#heading(level: 2, "§1.2 -- Game Scale") <sect-1-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Game Scale

Each hexagon of the mapsheet represents approximately 400–440 yards of real terrain and each day turn is the equivalent of two hours of real time. Each counter of infantry and cavalry represents between 400 and 700 men, and each of the gunboats present at the battle has its own counter. The upper echelon of command is represented by individual leader counters for the Anglo-Egyptian force; and leaders plus their retinues for the Dervish army.]]
#v(0.5em)
#progress-bar(3, 6)
#heading(level: 1, "§2 -- Game Components") <sect-2>
#heading(level: 2, "§2.1 -- The Game Maps") <sect-2-1>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Game Maps

The Omdurman battle map represents approximately 100 square miles of real territory and includes the area north of Omdurman in which the historical battle took place as well as the dominant terrain features that influenced the course of the battle. Note that the mapsheet also contains the Turn Record Track, Turn Sequence, and Terrain Effects Chart at the top; and the Combat Tables and Howitzer Fire Scattergram in the lower right corner. The large letters "A", "D", "Y", etc. are set-up hexes for the historical scenario only (#link(<sect-9-2>)[9.2]) and should be ignored in the campaign game. Similarly, the hexsides of the Zariba exist only in the historical scenario and should be considered clear terrain in the campaign game. Note, however, that the Anglo-Egyptian player may "construct" the Zariba in the campaign game if desired (see #link(<sect-5-3>)[5.3]). All full hexes of the Omdurman game map are playable, including the seven hexes of the Howitzer Fire Scattergram.

The mini-map for the bonus game, FALL OF KHARTOUM, represents that city as it appeared in 1885. The portion of wall conspicuous by its absence represents the area washed away by the receding White Nile after the flood. Players will note that the north edge of the Khartoum mini-map abuts the middle portion of the Omdurman map south edge. After Khartoum fell, it was destroyed by the Mahdi's troops and lay in ruins in 1898.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-3>)[§5.3], #link(<sect-9-2>)[§9.2]]
#v(0.3em)
#heading(level: 2, "§2.2 -- Play Aids") <sect-2-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Play Aids

Certain charts and tables are needed to play the game. The Terrain Effects Chart lists all terrain found on the mapsheet and the effect of each type on movement and combat. The Combat Tables describe the range effects on various weapon types and includes the Combat Results Table. Also note the Line of Sight Table on the back of this rulebook. It tells players when certain terrain types block line of sight, thus preventing direct fire attacks on enemy units. Players should become familiar with these various charts and tables prior to the beginning of play.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#heading(level: 2, "§2.3 -- The Units") <sect-2-3>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Units]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 716) \ #github-link("omdurman-types/src/lib.rs", 716)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L716")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind]]]], [#raw("714 │     strum::EnumIter,
715 │ )]
716 │ pub enum UnitKind {
717 │     /// Foot infantry. Includes Anglo-Egyptian infantry, \"Friendlies\",
718 │     /// Royal Engineers, and Dervish foot tribes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 670) \ #github-link("omdurman-rules/src/lib.rs", 670)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L670")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitProfile]]]], [#raw("668 │ /// print no melee value).
669 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
670 │ pub struct UnitProfile {
671 │     pub kind: UnitKind,
672 │     pub identity: UnitIdentity,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 327) \ #github-link("omdurman-rules/src/lib.rs", 327)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L327")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeId]]]], [#raw("325 │ /// (§2.3, §5.54). The number is the brigade ordinal as printed, e.g. `2B`.
326 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
327 │ pub struct BrigadeId {
328 │     pub number: u8,
329 │     pub nationality: BrigadeNationality,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 892) \ #github-link("omdurman-types/src/lib.rs", 892)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L892")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[SpriteAnnotation]]]], [#raw("890 │ /// movement allowance (§5.24); leaders print movement only (§6.51).
891 │ #[derive(Serialize, Deserialize, Clone, Debug)]
892 │ pub struct SpriteAnnotation {
893 │     /// Command/tribe colour. A real game indicator: Dervish leaders may only
894 │     /// stack with units of their own colour, and different tribes may not", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§2.4 -- Game Parts Inventory") <sect-2-4>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Game Parts Inventory

Your complete copy of "REMEMBER GORDON!" — THE BATTLE OF OMDURMAN includes:

- One 22 × 28 Battle of Omdurman mapsheet
- One 8½ × 11 bonus game: FALL OF KHARTOUM mapsheet
- One Rules Booklet
- One die-cut Unit Counter Sheet
- One Campaign Game Order of Appearance Card
- One ten-sided die
- One game box]]
#v(0.5em)
#heading(level: 2, "§2.31 -- Dervish weapon types") <sect-2-31>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Dervish artillery, gunboats, and forts fire on the "artillery" line of the Dervish Range Effects Table; Jehadia and Danagla units fire on the "rifles" line as does the Isa Zachneih unit. All other Dervish units (including leaders) are armed with spears and swords.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 446) \ #github-link("omdurman-rules/src/lib.rs", 446)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L446")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("444 │ /// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
445 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
446 │ pub enum WeaponClass {
447 │     /// Dervish spears and swords -- no ranged fire at all.
448 │     Melee,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 890) \ #github-link("omdurman-rules/src/lib.rs", 890)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L890")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Howitzer]]]], [#raw("888 │ /// roll on the Howitzer Fire Scattergram (§6.64).
889 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
890 │ pub struct HowitzerResolution {
891 │     pub combat_results_table_roll: DieRoll,
892 │     pub impact_roll: DieRoll,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§2.32 -- Anglo-Egyptian weapon types") <sect-2-32>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[All Anglo-Egyptian units (except gunboats, Maxims, artillery, and leaders) are armed with rifles. Maxims fire on the "Maxims" line of the Anglo-Egyptian Range Effects Table, and artillery and old gunboats fire on the "Artillery" line. New type (named) gunboats may fire on the "Howitzer", "Artillery", and "Maxims" lines of the Range Effects Table. (See #link(<sect-6-52>)[6.52] for the fire capabilities of the "Friendlies".)

\*\*Sample Dervish Units\*\* (printed on counters): combat unit (Combat / Melee / Movement values, plus Tribe identifier); Leader (e.g. OSMAN DIGNA); Camel unit (e.g. Danagla, 4-6-12); Fort.

\*\*Sample Anglo-Egyptian Units\*\* (printed on counters): Cavalry (e.g. 21 Lancers); Artillery (e.g. 32 Battery); Old Gunboat (e.g. LORD KITCHENER, 0-0-15); New Gunboat — named (e.g. Sultan, with artillery and howitzer factor, plus movement downstream / movement upstream values); Maxim Guns (fire twice per turn); Infantry (Fire Combat Factor / Melee / Movement, plus Battalion ID and Brigade ID — e.g. "2B" = 2nd British Brigade, "3E" = 3rd Egyptian Brigade).]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-6-52>)[§6.52]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 406) \ #github-link("omdurman-rules/src/lib.rs", 406)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L406")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId]]]], [#raw("404 │ /// fire; \"old\" gunboats do not (rulebook §2.32).
405 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
406 │ pub enum GunboatId {
407 │     /// One of the five new-type named gunboats with howitzer capability.
408 │     Named(NamedGunboat),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 417) \ #github-link("omdurman-rules/src/lib.rs", 417)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L417")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[NamedGunboat]]]], [#raw("415 │ /// The five named gunboats with howitzer capability (rulebook §6.64, §2.32).
416 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
417 │ pub enum NamedGunboat {
418 │     Sultan,
419 │     Melik,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 430) \ #github-link("omdurman-rules/src/lib.rs", 430)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L430")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OldGunboat]]]], [#raw("428 │ /// in the Maxim Second Fire and Howitzer subphase (§6.42).
429 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
430 │ pub enum OldGunboat {
431 │     LordKitchener,
432 │     Tamai,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 430) \ #github-link("omdurman-rules/src/lib.rs", 430)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L430")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId::Old]]]], [#raw("428 │ /// in the Maxim Second Fire and Howitzer subphase (§6.42).
429 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
430 │ pub enum OldGunboat {
431 │     LordKitchener,
432 │     Tamai,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 412) \ #github-link("omdurman-rules/src/lib.rs", 412)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L412")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId::DervishGunboat]]]], [#raw("410 │     Old(OldGunboat),
411 │     /// A Dervish gunboat (§9.111, §10.14).
412 │     DervishGunboat(u8),
413 │ }
414 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#progress-bar(0, 1)
#heading(level: 1, "§3 -- Getting Started") <sect-3>
#heading(level: 2, "§3 -- Getting Started")
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Getting Started

Spread out the mapsheet on a table. It should lie flat if you backfold it against the scored lines. The Dervish player should sit next to the west edge of the map and the Anglo-Egyptian player opposite him on the east edge. Read through the rules once, looking over the various charts as they are referred to in the various sections. Next, select a scenario and punch out only those unit counters needed to play. Later on, the rest of the unit counters should be punched out, sorted and stored by unit type.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#progress-bar(1, 1)
#heading(level: 1, "§4 -- Turn Sequence") <sect-4>
#heading(level: 2, "§4 -- Turn Sequence")
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Turn Sequence

"REMEMBER GORDON!" — THE BATTLE OF OMDURMAN is played in "Game Turns", each of which has two "Player Turns". The player moving first will vary according to the scenario being played. In the Campaign Game, for example, the Anglo-Egyptian player moves first.

\#\#\# A) Anglo-Egyptian Player Turn:

1. Anglo-Egyptian Movement Phase
2. Fire Combat Phase
   a. Dervish Defensive Fire
   b. Anglo-Egyptian Offensive Fire
      1. Direct Fire Subphase
      2. Maxim Second Fire and Howitzer Fire Subphase
3. Anglo-Egyptian Melee Attacks

\#\#\# B) Dervish Player Turn:

1. Dervish Movement Phase
2. Fire Combat Phase
   a. Anglo-Egyptian Defensive Fire
      1. Direct Fire Subphase
      2. Maxim Second Fire and Howitzer Fire Subphase
   b. Dervish Offensive Fire
3. Dervish Melee Attacks

\*\*C)\*\* After both players have completed their "Player Turns", advance the "Game Turn" marker to the next hour. Continue in this manner, alternating turns, until the end of the scenario being played.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 234) \ #github-link("omdurman-rules/src/lib.rs", 234)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L234")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTurnIndex]]]], [#raw("232 │ /// One-based Game Turn index (1, 2, ... up to the scenario length) (rulebook §4).
233 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
234 │ pub struct GameTurnIndex(pub u8);
235 │ 
236 │ impl GameTurnIndex {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 266) \ #github-link("omdurman-rules/src/lib.rs", 266)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L266")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Phase]]]], [#raw("264 │ /// etc.
265 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
266 │ pub enum Phase {
267 │     /// Pre-game deployment (§9.2/§9.3/§10): fixed units are placed, each side
268 │     /// deploys its order of battle within its legal zone, and river", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 466) \ #github-link("omdurman-rules/src/effects.rs", 466)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L466")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState]]]], [#raw("464 │ /// All mutable state of a game in progress (rulebook §4).
465 │ #[derive(Serialize, Deserialize, Clone, Debug)]
466 │ pub struct GameState {
467 │     pub scenario: Scenario,
468 │     pub current_turn: GameTurnIndex,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 566) \ #github-link("omdurman-rules/src/effects.rs", 566)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L566")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState::new]]]], [#raw("564 │ impl GameState {
565 │     /// Create a fresh game state for a given scenario (rulebook §4).
566 │     pub fn new(scenario: Scenario) -> Self {
567 │         let first = scenario_turn(scenario, GameTurnIndex(1));
568 │         // First player to *move* per scenario: Campaign -- Anglo-Egyptian moves", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 40) \ #github-link("omdurman-rules/src/effects.rs", 40)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L40")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvancePhase]]]], [#raw(" 38 │     // -- Turn / phase flow ------------------------------------------------
 39 │     /// Advance to the next phase (or next player-turn if melee is done) (rulebook §4).
 40 │     AdvancePhase,
 41 │ 
 42 │     // -- Movement ----------------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1630) \ #github-link("omdurman-rules/src/effects.rs", 1630)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1630")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[advance_phase]]]], [#raw("1628 │ 
1629 │ /// Advance the game state to the next phase (rulebook §4).
1630 │ pub fn advance_phase(state: &mut GameState) -> Result<(), RuleError> {
1631 │     let old_phase = state.phase;
1632 │     debug!(old_phase = ?old_phase, active_player = ?state.active_player, \"advance_phase\");", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1702) \ #github-link("omdurman-rules/src/effects.rs", 1702)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1702")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[end_player_turn]]]], [#raw("1700 │ 
1701 │ /// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
1702 │ pub fn end_player_turn(state: &mut GameState) -> Result<(), RuleError> {
1703 │     debug!(
1704 │         old_player = ?state.active_player,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 237) \ #github-link("omdurman-rules/src/lib.rs", 237)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L237")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTurnIndex::value]]]], [#raw("235 │ 
236 │ impl GameTurnIndex {
237 │     pub fn value(self) -> u8 {
238 │         self.0
239 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 558) \ #github-link("omdurman-rules/src/effects.rs", 558)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L558")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[PendingMelee]]]], [#raw("556 │ /// resolution after the reaction window is deterministic and host-ordered (rulebook §7.5).
557 │ #[derive(Serialize, Deserialize, Clone, Debug)]
558 │ pub struct PendingMelee {
559 │     pub attack: MeleeAttack,
560 │     pub attacker_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1449) \ #github-link("omdurman-rules/src/effects.rs", 1449)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1449")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("1447 │     /// does not extend into or out of a Nile hex. With no board loaded these
1448 │     /// reduce to the plain adjacency rule.
1449 │     pub fn hex_in_enemy_zoc(
1450 │         &self,
1451 │         hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 849) \ #github-link("omdurman-rules/src/effects.rs", 849)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L849")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit]]]], [#raw("847 │     /// the same `RuleError` the `MoveUnit` effect would on rejection. Lets the
848 │     /// UI gate input without mutating or duplicating the rules.
849 │     pub fn can_move_unit(&self, unit_id: UnitId, cost: MovementPoints) -> Result<(), RuleError> {
850 │         self.can_move_unit_to(unit_id, None, cost)
851 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[both_ready_auto_advances_out_of_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_combat_wrong_phase_rejected]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[new_game_starts_in_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[scenario_turn_dispatches_correctly]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_advances_through_phases]]]
#v(0.3em)
#progress-bar(17, 19)
#heading(level: 1, "§5 -- Movement Phase") <sect-5>
#heading(level: 2, "§5 -- Movement Phase (general)")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Movement Phase]]
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[disrupted_unit_cannot_fire]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[disrupted_unit_may_not_act]]]
#v(0.3em)
#heading(level: 2, "§5.3 -- Constructing the Zariba") <sect-5-3>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Constructing the Zariba

The Zariba trench and thorn hedge hexsides are built and in place in the historical scenario only. These hexsides are considered clear terrain in the campaign game. The Anglo-Egyptian player may, however, find it useful to construct this defensive position during the campaign game. The Zariba hexsides may only be built in their position as displayed on the mapsheet. Construction procedure is as follows: any Anglo-Egyptian infantry unit that begins and ends the Anglo-Egyptian player turn adjacent to (and on the Nile side of) Zariba hexsides has constructed all Zariba hexsides to which he is adjacent. The constructing unit may neither fire offensively nor melee attack during the turn of construction. Use a blank counter to denote units constructing Zariba hexsides. See #link(<sect-9-23>)[9.23] for defensive benefits and movement restrictions of Zariba hexsides.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-9-23>)[§9.23]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 1541) \ #github-link("omdurman-rules/src/lib.rs", 1541)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1541")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[constructing_zariba]]]], [#raw("1539 │         // melee attack during the turn of construction.\"
1540 │         let s = UnitState {
1541 │             constructing_zariba: true,
1542 │             ..UnitState::default()
1543 │         };", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 109) \ #github-link("omdurman-rules/src/effects.rs", 109)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L109")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ConstructZariba]]]], [#raw("107 │ 
108 │     /// Begin constructing a Zariba hexside (rulebook §5.3).
109 │     ConstructZariba {
110 │         unit_ids: Vec<UnitId>,
111 │         hexside: HexsideRef,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2849) \ #github-link("omdurman-rules/src/effects.rs", 2849)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2849")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_construct_zariba]]]], [#raw("2847 │ 
2848 │ /// Mark a set of units as constructing a Zariba hexside (rulebook §5.3).
2849 │ pub fn apply_construct_zariba(
2850 │     state: &mut GameState,
2851 │     unit_ids: &[UnitId],", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 723) \ #github-link("omdurman-rules/src/lib.rs", 723)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L723")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitState::may_attack_this_turn]]]], [#raw("721 │     /// A unit that began construction this turn may not fire offensively or
722 │     /// melee (§5.3, §6.53).
723 │     pub fn may_attack_this_turn(self) -> bool {
724 │         !self.disrupted && !self.constructing_zariba && !self.demolishing
725 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.11 -- Movement allowances printed on units") <sect-5-11>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The movement allowances of the various unit types are printed directly on the units (see #link(<sect-2-3>)[2.3]). A unit may move up to this printed movement allowance, paying varying costs for different terrain types (see the Terrain Effects Chart).]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-2-3>)[§2.3]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 115) \ #github-link("omdurman-rules/src/lib.rs", 115)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L115")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementAllowance]]]], [#raw("113 │     /// is a named variant.
114 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
115 │     pub enum MovementAllowance {
116 │         /// Immobile (forts, wrecked gunboats).
117 │         Immobile = 0,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 681) \ #github-link("omdurman-rules/src/lib.rs", 681)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L681")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitMovement]]]], [#raw("679 │ /// Movement allowance -- uniform for land units, split for gunboats (rulebook §5.11, §5.24, §5.25).
680 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
681 │ pub enum UnitMovement {
682 │     Land(MovementAllowance),
683 │     Gunboat(GunboatMovement),", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 297) \ #github-link("omdurman-types/src/lib.rs", 297)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L297")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDirection]]]], [#raw("295 │ /// (`+q`, `+q+r`, `+r`, `-q`, `-q-r`, `-r` for pointy-top hexes) (rulebook §5.11, §5.24).
296 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
297 │ pub enum HexDirection {
298 │     #[default]
299 │     East = 0,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 155) \ #github-link("omdurman-rules/src/lib.rs", 155)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L155")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementPoints]]]], [#raw("153 │ /// Movement points spent or remaining within a single phase (rulebook §5).
154 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
155 │ pub struct MovementPoints(pub i16);
156 │ 
157 │ /// A distance measured in hexes (range to target, length of a retreat, ...)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 21) \ #github-link("omdurman-rules/src/terrain_chart.rs", 21)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/terrain_chart.rs#L21")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[terrain_effects_chart]]]], [#raw(" 19 │ ///
 20 │ /// Source: printed Terrain Effects Chart on the mapsheet.
 21 │ pub fn terrain_effects_chart(terrain: Terrain) -> TerrainEntry {
 22 │     match terrain {
 23 │         Terrain::Clear { .. } => TerrainEntry {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 65) \ #github-link("omdurman-rules/src/terrain_chart.rs", 65)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/terrain_chart.rs#L65")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[movement_cost]]]], [#raw(" 63 │ /// Convenience: get the movement cost for a terrain type (rulebook §5.11, Terrain Effects Chart).
 64 │ /// Returns `None` for impassable terrain (Nile).
 65 │ pub fn movement_cost(terrain: Terrain) -> Option<MovementAllowance> {
 66 │     terrain_effects_chart(terrain).movement_cost
 67 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 73) \ #github-link("omdurman-rules/src/terrain_chart.rs", 73)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/terrain_chart.rs#L73")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[movement_cost_with_road]]]], [#raw(" 71 │ /// underlying terrain; without a road it's the terrain's own cost. The road is
 72 │ /// a movement overlay only -- combat/LOS still use the underlying terrain.
 73 │ pub fn movement_cost_with_road(terrain: Terrain, road: bool) -> Option<MovementAllowance> {
 74 │     if road {
 75 │         Some(MovementAllowance::One)", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 427) \ #github-link("omdurman-types/src/lib.rs", 427)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L427")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::passable_by_land]]]], [#raw("425 │ 
426 │     /// Whether this terrain may be entered by land units (rulebook §5.11).
427 │     pub fn passable_by_land(self) -> bool {
428 │         !self.is_nile()
429 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 488) \ #github-link("omdurman-types/src/lib.rs", 488)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L488")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexData::is_crossroad]]]], [#raw("486 │ 
487 │     /// Whether roads converge at this hex's centre.
488 │     pub fn is_crossroad(self) -> bool {
489 │         matches!(self.road(), Road::Crossroad)
490 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 488) \ #github-link("omdurman-types/src/lib.rs", 488)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L488")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TileInfo::is_crossroad]]]], [#raw("486 │ 
487 │     /// Whether roads converge at this hex's centre.
488 │     pub fn is_crossroad(self) -> bool {
489 │         matches!(self.road(), Road::Crossroad)
490 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-hexmap/src/map.rs", 150) \ #github-link("omdurman-hexmap/src/map.rs", 150)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-hexmap/src/map.rs#L150")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameMap::roads]]]], [#raw("148 │         tiles,
149 │         hexsides,
150 │         roads,
151 │         excluded: game_map.excluded.iter().map(|c| (c.q, c.r)).collect(),
152 │         overlay: game_map.overlay.clone(),", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[clear_terrain_no_bonus]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[nile_is_impassable]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[rough_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[swamp_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[hilltop_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[huts_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[movement_cost_convenience_matches_chart]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[movement_cost_with_road_always_one]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[land_unit_may_not_enter_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[movement_cost_without_road_matches_terrain]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[movement_cost_for_uses_terrain]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[movement_cost_for_road_costs_one]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[road_gives_crossroad]]]
#v(0.3em)
#heading(level: 2, "§5.12 -- Move up to allowance, hex by hex (cumulative MP cap)") <sect-5-12>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[A player may move as many or as few of his units as desired during each movement phase, limited only by the units' movement allowance, the terrain costs paid in moving from hex to hex, and enemy zones of control (see 5.4).]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1274) \ #github-link("omdurman-rules/src/effects.rs", 1274)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1274")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[mp_spent]]]], [#raw("1272 │ 
1273 │     /// Movement points `unit_id` has already spent this turn (§5.11/§5.12).
1274 │     pub fn mp_spent(&self, unit_id: UnitId) -> i16 {
1275 │         self.mp_spent_this_turn
1276 │             .iter()", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.13 -- No MP accumulation between turns") <sect-5-13>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[A unit may never accumulate movement points from turn to turn, nor may a unit transfer unused movement points to other units. A unit's unused movement points in any given turn are considered lost.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1702) \ #github-link("omdurman-rules/src/effects.rs", 1702)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1702")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[end_player_turn]]]], [#raw("1700 │ 
1701 │ /// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
1702 │ pub fn end_player_turn(state: &mut GameState) -> Result<(), RuleError> {
1703 │     debug!(
1704 │         old_player = ?state.active_player,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.21 -- Friendlies transport via gunboat") <sect-5-21>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[In general, naval transport missions are not allowed, i.e. gunboats may not carry any land units. The sole exception is that the Anglo-Egyptian player may transport the surviving units of the "Friendlies" brigade from the east bank of the Nile to the west bank after, and only after, the Dervish east bank unit (Isa Zachneih) has been eliminated. The transport is accomplished in the following sequence:
a) on any turn that a "Friendlies" unit and any Anglo-Egyptian gunboat start their turn adjacent, that unit may load onto (i.e. stack with) the gunboat;
b) during the Anglo-Egyptian player's next turn the gunboat may move to any Nile hex adjacent to a west bank hex (up to the gunboat's movement allowance);
c) on the Anglo-Egyptian player's third turn the "Friendlies" unit may disembark and move normally, paying the normal terrain cost for the first hex entered. The gunboat may also move normally that turn.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 580) \ #github-link("omdurman-rules/src/lib.rs", 580)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L580")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_friendlies]]]], [#raw("578 │     /// \"Friendlies\" units obey several special rules (§5.21, §5.23, §6.52,
579 │     /// §9.14 victory conditions).
580 │     pub fn is_friendlies(&self) -> bool {
581 │         matches!(
582 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 701) \ #github-link("omdurman-rules/src/lib.rs", 701)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L701")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[loaded_on]]]], [#raw("699 │     pub disrupted: bool,
700 │     /// `Some(gunboat)` after a \"Friendlies\" unit loads onto a gunboat (§5.21).
701 │     pub loaded_on: Option<UnitId>,
702 │     /// Set while the unit is building Zariba hexsides -- neither offensive
703 │     /// fire nor melee allowed that turn (§5.3).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 136) \ #github-link("omdurman-rules/src/effects.rs", 136)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L136")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FriendliesTransport]]]], [#raw("134 │ 
135 │     /// Load/disembark the \"Friendlies\" brigade via gunboat (rulebook §5.21).
136 │     FriendliesTransport(crate::FriendliesTransport),
137 │ 
138 │     // -- Optional rules ----------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 3124) \ #github-link("omdurman-rules/src/effects.rs", 3124)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3124")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_friendlies_transport]]]], [#raw("3122 │ ///     transport mission ends and the unit is freed (a disembarking `MoveUnit`
3123 │ ///     should follow, costed by terrain).
3124 │ pub fn apply_friendlies_transport(
3125 │     state: &mut GameState,
3126 │     action: FriendliesTransport,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 967) \ #github-link("omdurman-rules/src/lib.rs", 967)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L967")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FriendliesTransport]]]], [#raw("965 │ /// tracks each unit–gunboat pair independently.
966 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
967 │ pub enum FriendliesTransport {
968 │     /// Turn N (the load turn): unit and gunboat started adjacent; unit
969 │     /// loads onto (stacks with) the gunboat.", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.22 -- Land units may never enter a Nile River hex") <sect-5-22>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[With the exception of #link(<sect-5-21>)[5.21], land units may never enter a Nile River hex. Only gunboats may enter and move along Nile River hexes.]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-21>)[§5.21]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 866) \ #github-link("omdurman-rules/src/effects.rs", 866)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L866")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw("864 │     ///
865 │     /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
866 │     pub fn can_move_unit_to(
867 │         &self,
868 │         unit_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.23 -- Walled city entry restrictions") <sect-5-23>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only certain units may enter the walled portion of Omdurman. For the Dervish player these are the Khalifa unit, the three Dervish artillery units, and the Taiasha units (the Khalifa's bodyguard). Any Anglo-Egyptian units that can get to the walled city may enter it (except gunboats and "Friendlies"). Units entering and/or exiting the walled city may only do so through a gate or breach hexside.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 157) \ #github-link("omdurman-types/src/lib.rs", 157)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L157")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideRef]]]], [#raw("155 │ /// data by [`HexsideRef`].
156 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
157 │ pub struct HexsideRef {
158 │     pub a: HexCoord,
159 │     pub b: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 192) \ #github-link("omdurman-types/src/lib.rs", 192)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L192")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideKind]]]], [#raw("190 │     strum::EnumIter,
191 │ )]
192 │ pub enum HexsideKind {
193 │     /// City wall (Khartoum, walled city of Omdurman). Blocks LOS, blocks
194 │     /// movement except across gates/breaches (§5.23), blocks ZOC into the city", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 256) \ #github-link("omdurman-types/src/lib.rs", 256)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L256")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_movement]]]], [#raw("254 │     /// Whether land movement may *not* cross this side (§5.23). Walls block
255 │     /// movement except at gates/breaches.
256 │     pub fn blocks_movement(self) -> bool {
257 │         matches!(self, HexsideKind::Wall)
258 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2639) \ #github-link("omdurman-rules/src/effects.rs", 2639)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2639")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_retreat_before_melee]]]], [#raw("2637 │     /// two hexes away and empty. (Does not verify the attacker is infantry --
2638 │     /// the caller offers the retreat only in response to one.)
2639 │     pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2640 │         let unit = self
2641 │             .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_move_rejects_wall_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_move_allows_gate_hexside]]]
#v(0.3em)
#heading(level: 2, "§5.24 -- Gunboat upstream/downstream movement") <sect-5-24>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Note that gunboats have two movement allowances separated by a slash, e.g. 10/16. The smaller number is the movement allowance when moving upstream, i.e. against the current (the direction of the current is indicated by arrows in the Nile). The larger number is the movement allowance when moving downstream, i.e. with the current. Gunboats may combine movement in both directions, but if they move even one hex upstream, their upstream movement allowance is their maximum movement allowance for that turn, and may not be exceeded.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 514) \ #github-link("omdurman-rules/src/lib.rs", 514)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L514")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatMovement]]]], [#raw("512 │ /// the turn.
513 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
514 │ pub struct GunboatMovement {
515 │     pub upstream: MovementAllowance,
516 │     pub downstream: MovementAllowance,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 755) \ #github-link("omdurman-types/src/lib.rs", 755)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L755")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_boat]]]], [#raw("753 │ 
754 │     /// Gunboats use the split upstream/downstream movement allowance (§5.24).
755 │     pub fn is_boat(self) -> bool {
756 │         matches!(self, UnitKind::Gunboat)
757 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[boat_annotation_yields_split_gunboat_movement]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[movement_from_annotation_fort_returns_immobile]]]
#v(0.3em)
#heading(level: 2, "§5.25 -- Dervish forts may not move") <sect-5-25>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Dervish forts may not move in any way once placed.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 117) \ #github-link("omdurman-rules/src/lib.rs", 117)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L117")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Immobile]]]], [#raw("115 │     pub enum MovementAllowance {
116 │         /// Immobile (forts, wrecked gunboats).
117 │         Immobile = 0,
118 │         One = 1,
119 │         Two = 2,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 726) \ #github-link("omdurman-types/src/lib.rs", 726)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L726")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Fort]]]], [#raw("724 │     Gunboat,
725 │     /// Permanent emplacement -- may not move once placed (§5.25).
726 │     Fort,
727 │     /// Dervish leader: has fire/melee/movement factors and may melee attack.
728 │     DervishLeaderUnit,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 685) \ #github-link("omdurman-rules/src/lib.rs", 685)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L685")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitMovement::Immobile]]]], [#raw("683 │     Gunboat(GunboatMovement),
684 │     /// Forts may not move once placed (§5.25).
685 │     Immobile,
686 │ }
687 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.26 -- Units stop on entering enemy ZOC") <sect-5-26>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Units must stop their movement immediately upon entering an enemy zone of control (see 5.4).]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 866) \ #github-link("omdurman-rules/src/effects.rs", 866)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L866")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw("864 │     ///
865 │     /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
866 │     pub fn can_move_unit_to(
867 │         &self,
868 │         unit_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.41 -- All units except AE leaders exert ZOC") <sect-5-41>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[All units except Anglo-Egyptian leaders exert a zone of control (hereafter called a ZOC) into their six adjacent hexes (exception: Gunboats exert a ZOC only against enemy gunboats). Disrupted units have no ZOC.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 756) \ #github-link("omdurman-rules/src/lib.rs", 756)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L756")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("754 │ /// Used by the engine when answering \"is this hex in an enemy ZOC?\".
755 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
756 │ pub enum ZocReason {
757 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
758 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1414) \ #github-link("omdurman-rules/src/effects.rs", 1414)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1414")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[unit_projects_zoc]]]], [#raw("1412 │     /// §5.44) need the game map, which the engine does not hold; the app layers
1413 │     /// those on top. This is the position/kind/disruption core of the rule.
1414 │     pub fn unit_projects_zoc(
1415 │         &self,
1416 │         unit: &UnitPlacement,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.42 -- No MP cost to enter/leave enemy ZOC") <sect-5-42>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[There is no movement point cost to enter or leave an enemy ZOC.]]
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[entering_enemy_zoc_costs_no_extra_mp]]]
#v(0.3em)
#heading(level: 2, "§5.43 -- Units stop when entering enemy ZOC") <sect-5-43>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[All units must stop when they enter an enemy ZOC and may move no further that turn. In their next movement phase they may withdraw or, if desired, move directly into another enemy ZOC.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 866) \ #github-link("omdurman-rules/src/effects.rs", 866)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L866")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw("864 │     ///
865 │     /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
866 │     pub fn can_move_unit_to(
867 │         &self,
868 │         unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1449) \ #github-link("omdurman-rules/src/effects.rs", 1449)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1449")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("1447 │     /// does not extend into or out of a Nile hex. With no board loaded these
1448 │     /// reduce to the plain adjacency rule.
1449 │     pub fn hex_in_enemy_zoc(
1450 │         &self,
1451 │         hex: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.44 -- ZOC limitations (walls, khor, fort, Nile, Zariba)") <sect-5-44>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[ZOCs do not extend into or out of a Nile River hex (exception: Gunboats, see #link(<sect-5-41>)[5.41]). ZOCs do not extend across a khor, into a fort, or into a hex inside the walled city across a wall hexside. ZOCs do extend out of a fort (even if unoccupied), and from a walled city hex into an adjacent non-walled-city hex across a wall hexside. ZOCs also extend out of (but not into) a walled city hex across a gate hexside. ZOCs extend both ways across a breach hexside. ZOCs also extend out of, but not into, a hut or building hex. In the historical scenario ZOCs extend out of, but not into, the Zariba across a Zariba hexside (also in the campaign game if the Zariba is constructed).]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-41>)[§5.41]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 756) \ #github-link("omdurman-rules/src/lib.rs", 756)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L756")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("754 │ /// Used by the engine when answering \"is this hex in an enemy ZOC?\".
755 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
756 │ pub enum ZocReason {
757 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
758 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 197) \ #github-link("omdurman-types/src/lib.rs", 197)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L197")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Wall]]]], [#raw("195 │     /// (§5.44), blocks melee (§7.2), blocks advance-after-combat (§6.82).
196 │     #[default]
197 │     Wall,
198 │     /// Gate hexside in a wall. ZOC extends *out of* the walled city through
199 │     /// gates but not into it (§5.44). Melee may be made through a gate (§7.2).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 206) \ #github-link("omdurman-types/src/lib.rs", 206)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L206")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Khor]]]], [#raw("204 │     /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
205 │     /// combat may not cross (§6.82).
206 │     Khor,
207 │     /// Crest line. Blocks LOS unless the firer is on the higher side
208 │     /// (§6.3 note 7).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 769) \ #github-link("omdurman-rules/src/lib.rs", 769)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L769")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason::Zariba]]]], [#raw("767 │     /// Zariba hexside ZOC behaviour in the historical scenario / when the
768 │     /// Zariba is constructed (§5.44).
769 │     Zariba,
770 │ }
771 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1414) \ #github-link("omdurman-rules/src/effects.rs", 1414)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1414")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[unit_projects_zoc]]]], [#raw("1412 │     /// §5.44) need the game map, which the engine does not hold; the app layers
1413 │     /// those on top. This is the position/kind/disruption core of the rule.
1414 │     pub fn unit_projects_zoc(
1415 │         &self,
1416 │         unit: &UnitPlacement,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.51 -- Stacking limit (4 units + leaders, gunboats isolated)") <sect-5-51>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[No more than four units may occupy a hex, with the exception of leaders and gunboats. All leader units are free stacking, i.e. they may stack in addition to the four-unit-per-hex stacking limitation. Gunboats may not stack with any other unit (Exception: #link(<sect-5-21>)[5.21]). Players may move through friendly units at no additional cost in movement points. The stacking limitation applies only at the end of the movement phase and during combat.]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-21>)[§5.21]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 778) \ #github-link("omdurman-rules/src/lib.rs", 778)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L778")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OverLimit]]]], [#raw("776 │     /// and the gunboat exception.
777 │     #[error(\"hex stack exceeds the four-unit limit\")]
778 │     OverLimit,
779 │     /// \"Gunboats may not stack with any other unit\" (§5.51, exception §5.21).
780 │     #[error(\"gunboats may not stack with non-gunboat units\")]", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 781) \ #github-link("omdurman-rules/src/lib.rs", 781)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L781")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatStack]]]], [#raw("779 │     /// \"Gunboats may not stack with any other unit\" (§5.51, exception §5.21).
780 │     #[error(\"gunboats may not stack with non-gunboat units\")]
781 │     GunboatStack,
782 │     /// \"Units of different Dervish tribes may not stack together\" (§5.52).
783 │     #[error(\"Dervish units of different tribes may not stack\")]", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[stacking_over_limit_rejected]]]
#v(0.3em)
#heading(level: 2, "§5.52 -- Different Dervish tribes may not stack together") <sect-5-52>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The units of different Dervish tribes may not stack together, even if they are the same color (e.g. although both are green, Mulazmin and Jehadia units may not stack with each other).]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 784) \ #github-link("omdurman-rules/src/lib.rs", 784)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L784")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishTribeMix]]]], [#raw("782 │     /// \"Units of different Dervish tribes may not stack together\" (§5.52).
783 │     #[error(\"Dervish units of different tribes may not stack\")]
784 │     DervishTribeMix,
785 │     /// \"If Dervish leaders elect to stack, they may only stack with units of
786 │     /// their command (i.e. colour)\" (§5.53).", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.53 -- Leader stacking with command colour only") <sect-5-53>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Leader units are not required to stack. If Dervish leaders elect to stack, however, they may only stack with units of their command (i.e. color). For example, Sheik El Din may only stack with Mulazmins or Jehadias.

\*\*#link(<sect-5-54>)[5.54]) Anglo-Egyptian Brigade Integrity:\*\* All British, Sudanese, and Egyptian infantry units have their brigade designation printed in the upper right corner (e.g. "2B" = 2nd British Brigade; "3E" = 3rd Egyptian Brigade, etc.). In any combat phase in which all four infantry battalions belonging to any Anglo-Egyptian infantry brigade are stacked in the same hex they are said to have brigade integrity. Stacks having brigade integrity receive a +1 modifier to their fire combat die roll provided they all fire at the same enemy occupied hex. This modifier is in addition to the normal +1 bonus given to all Anglo-Egyptian direct fire attacks (see #link(<sect-6-24>)[6.24]).]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-54>)[§5.54], #link(<sect-6-24>)[§6.24]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 788) \ #github-link("omdurman-rules/src/lib.rs", 788)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L788")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishLeaderCommandMismatch]]]], [#raw("786 │     /// their command (i.e. colour)\" (§5.53).
787 │     #[error(\"Dervish leader may only stack with units of their own command\")]
788 │     DervishLeaderCommandMismatch,
789 │ }
790 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.54 -- Anglo-Egyptian Brigade Integrity") <sect-5-54>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 794) \ #github-link("omdurman-rules/src/lib.rs", 794)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L794")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeIntegrity]]]], [#raw("792 │ /// stack contains all four battalions of a single Anglo-Egyptian brigade.
793 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
794 │ pub enum BrigadeIntegrity {
795 │     None,
796 │     Integrated(BrigadeId),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 643) \ #github-link("omdurman-rules/src/lib.rs", 643)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L643")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[brigade_integrity]]]], [#raw("641 │ /// Only a full stack of four battalions qualifies.  Three or fewer may still
642 │ /// stack and fire, but they receive no brigade-integrity bonus.
643 │ pub fn brigade_integrity(identities: &[UnitIdentity]) -> BrigadeIntegrity {
644 │     let Some(brigade) = identities.first().and_then(|i| i.brigade()) else {
645 │         return BrigadeIntegrity::None;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 336) \ #github-link("omdurman-rules/src/lib.rs", 336)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L336")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BattalionOrdinal]]]], [#raw("334 │     /// brigade integrity requires all four stacked in one hex (§5.54).
335 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
336 │     pub enum BattalionOrdinal {
337 │         First = 1,
338 │         Second = 2,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 832) \ #github-link("omdurman-types/src/lib.rs", 832)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L832")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Brigade]]]], [#raw("830 │     Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default, strum::EnumIter,
831 │ )]
832 │ pub enum Brigade {
833 │     #[default]
834 │     None,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[brigade_and_battalion_from_column]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[brigade_designation_ignored_for_non_infantry]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[brigade_none_keeps_column_derived_brigade]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[printed_brigade_designation_overrides_column]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[tribe_stats_come_from_annotation]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[section_owner_anglo_egyptian_sections]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[section_owner_green_sections_return_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[brigade_integrity_four_battalions_returns_integrated]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_infantry_fourth_battalion_from_col_3]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[brigade_integrity_empty_slice]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[brigade_integrity_friendlies_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[section_owner_dervish_sections]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[brigade_integrity_three_battalions_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[unit_identity_brigade_and_battalion_accessors]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_infantry_brigade_number_three_from_col_8]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[brigade_integrity_non_infantry_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[brigade_integrity_mixed_brigades_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_infantry_third_battalion_from_col_2]]]
#v(0.3em)
#progress-bar(21, 25)
#heading(level: 1, "§6 -- Fire Combat Phase") <sect-6>
#heading(level: 2, "§6.3 -- Line of Sight Table") <sect-6-3>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Line of Sight Table

This table is located on the back of this rulebook and should be self-explanatory. Locate the terrain type the firing unit is in and cross-index it with the terrain type the target unit is in. Terrain types in the intersecting box block line of sight, with exceptions as footnoted. Also study the "Special LOS Notes" given and remember that (with the exception of howitzer fire — see #link(<sect-6-64>)[6.64]) you can't fire at anything you can't see!]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-6-64>)[§6.64]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/los_table.rs", 3) \ #github-link("omdurman-rules/src/los_table.rs", 3)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L3")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosFirerTerrain]]]], [#raw("  1 │ /// Terrain type of the *firing* unit's hex for LOS purposes (rulebook §6.3).
  2 │ #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
  3 │ pub enum LosFirerTerrain {
  4 │     Ground,
  5 │     Rough,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 11) \ #github-link("omdurman-rules/src/los_table.rs", 11)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L11")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosTargetTerrain]]]], [#raw("  9 │ /// Terrain type of the *target* unit's hex for LOS purposes (rulebook §6.3).
 10 │ #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
 11 │ pub enum LosTargetTerrain {
 12 │     Ground,
 13 │     /// Units in the hex (including friendly -- LOS is blocked if the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 27) \ #github-link("omdurman-rules/src/los_table.rs", 27)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L27")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosResult]]]], [#raw(" 25 │ /// Whether LOS is blocked (rulebook §6.3).
 26 │ #[derive(Clone, Copy, PartialEq, Eq, Debug)]
 27 │ pub enum LosResult {
 28 │     Clear,
 29 │     Blocked,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 37) \ #github-link("omdurman-rules/src/los_table.rs", 37)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L37")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[los_table]]]], [#raw(" 35 │ /// If the cell says \"Blocks\", LOS is blocked; otherwise it is clear
 36 │ /// (subject to the special notes below).
 37 │ pub fn los_table(firer: LosFirerTerrain, target: LosTargetTerrain) -> LosResult {
 38 │     use LosFirerTerrain as F;
 39 │     use LosResult::*;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 158) \ #github-link("omdurman-rules/src/los_table.rs", 158)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L158")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosSpecialNote]]]], [#raw("156 │ /// 7. Crest hexsides block LOS unless the firer is on the higher side
157 │ ///    of the crest.
158 │ pub enum LosSpecialNote {
159 │     MaxTwoTreeHutHexes,
160 │     HilltopToHilltop,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 91) \ #github-link("omdurman-rules/src/los_table.rs", 91)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L91")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_los]]]], [#raw(" 89 │ /// [`GameState`](crate::effects::GameState)) so the engine can validate LOS
 90 │ /// without reaching into the app/Bevy layer.
 91 │ pub fn has_los(
 92 │     board: &crate::board::BoardInfo,
 93 │     from: omdurman_types::HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 233) \ #github-link("omdurman-types/src/lib.rs", 233)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L233")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_los]]]], [#raw("231 │     /// Whether this hexside blocks line of sight across it (§6.3). Crest is
232 │     /// directional and handled by the caller; here it is treated as blocking.
233 │     pub fn blocks_los(self) -> bool {
234 │         matches!(self, HexsideKind::Wall | HexsideKind::Crest)
235 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 433) \ #github-link("omdurman-types/src/lib.rs", 433)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L433")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::blocks_los]]]], [#raw("431 │     /// Whether an intervening hex of this terrain unconditionally blocks line
432 │     /// of sight (§6.3).
433 │     pub fn blocks_los(self) -> bool {
434 │         matches!(self, Terrain::Huts { .. } | Terrain::Building { .. })
435 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[los_wall_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[los_ground_to_ground_clear]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[los_hilltop_to_huts_blocked]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ground_firer_covers_all_targets]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[rough_firer_covers_all_targets]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[hilltop_firer_covers_all_targets]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[all_24_cells_exercised]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[los_every_cell_matches_the_table]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_empty_board_is_clear]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_wall_hexside_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_crest_hexside_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_gate_hexside_passes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_breach_hexside_passes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_huts_intervening_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_building_intervening_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_two_tree_hexes_pass]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_three_tree_hexes_block]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_hilltop_sees_over_intervening_terrain]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_hilltop_still_blocked_by_wall]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_howitzer_bypasses]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[has_los_adjacent_clear]]]
#v(0.3em)
#heading(level: 2, "§6.6 -- Special Artillery Capabilities") <sect-6-6>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Artillery Capabilities]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 446) \ #github-link("omdurman-rules/src/lib.rs", 446)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L446")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("444 │ /// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
445 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
446 │ pub enum WeaponClass {
447 │     /// Dervish spears and swords -- no ranged fire at all.
448 │     Melee,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.7 -- Defensive Fire") <sect-6-7>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Defensive Fire

In Defensive Fire phase, all of the non-moving player's units may fire at any of the moving player's units in range, within the limitations imposed by the rules of combat (see 6.1 to #link(<sect-6-6>)[6.6]). There is no advance after combat as a result of defensive fires.]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-6-6>)[§6.6]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 2684) \ #github-link("omdurman-rules/src/effects.rs", 2684)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2684")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("2682 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
2683 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
2684 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2685 │         let unit = self
2686 │             .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[no_advance_after_defensive_fire]]]
#v(0.3em)
#heading(level: 2, "§6.11 -- Fire combat factor printed on units") <sect-6-11>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The fire combat factor of the various unit types is printed directly on the units and is a numerical expression of the unit's fire strength.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 70) \ #github-link("omdurman-rules/src/lib.rs", 70)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L70")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireFactor]]]], [#raw(" 68 │     /// Every possible value from the annotated counter set is a named variant.
 69 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
 70 │     pub enum FireFactor {
 71 │         One = 1,
 72 │         Three = 3,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_factor_sum_to_row]]]
#v(0.3em)
#heading(level: 2, "§6.12 -- Fire combat is always voluntary") <sect-6-12>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Fire combat is always voluntary. A unit is never required to fire at enemy units merely because they are in range or adjacent.]]
#v(0.5em)
#heading(level: 2, "§6.13 -- Fire factor is unitary (may not be divided)") <sect-6-13>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[If a unit elects to fire, its fire combat factor at an enemy unit, that fire combat factor is unitary. A unit's fire combat factor may not be divided up to fire at enemy units on different hexes.]]
#v(0.5em)
#heading(level: 2, "§6.14 -- Players may combine fire factors into one attack") <sect-6-14>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Players may combine fire during fire combat phase, i.e. they may fire at an enemy-occupied hex with as many friendly units as may legally do so, combining all of their fire combat factors into one attack. Note that in any given fire combat phase, however, a combat unit may only fire once and may only be fired at once (exceptions: Maxim guns and gunboats — see 6.4).]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 84) \ #github-link("omdurman-rules/src/lib.rs", 84)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L84")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[sum_to_row]]]], [#raw(" 82 │ impl FireFactor {
 83 │     /// Sum multiple fire factors and return the corresponding Combat Results Table row (rulebook §6.11).
 84 │     pub fn sum_to_row<'a>(factors: impl IntoIterator<Item = &'a FireFactor>) -> FireFactorRow {
 85 │         let total: u16 = factors.into_iter().map(|f| f.value()).sum();
 86 │         crate::combat_results_table::FireFactorRow::from_total(total)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.15 -- May divide a stack to fire at different hexes") <sect-6-15>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Players may also divide a stack of units in order to fire at different enemy-occupied hexes. Anglo-Egyptian infantry units having brigade integrity, however, do not receive their +1 direct fire modifier unless they all fire at the same enemy-occupied hex (see #link(<sect-5-54>)[5.54]).]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-54>)[§5.54]]
#v(0.3em)
#heading(level: 2, "§6.16 -- Halving fire strength rounds down, minimum 1") <sect-6-16>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[When halving fire combat strength, always round down each individual unit. For example, an Egyptian brigade of four battalions, each having a printed strength of 9 fire factors, will fire a total of 16 factors when halved. However, a unit's firing strength is never reduced below one by halving.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 481) \ #github-link("omdurman-rules/src/lib.rs", 481)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L481")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RangeBand]]]], [#raw("479 │ /// multiplied at a given distance (§6.22).
480 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
481 │ pub enum RangeBand {
482 │     Tripled,
483 │     Doubled,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.21 -- First check LOS before firing") <sect-6-21>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[When combat units wish to fire at enemy units, first check the Line of Sight Table to be sure the firing unit can see the target hex (exception: howitzer fire, see #link(<sect-6-64>)[6.64]).]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-6-64>)[§6.64]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/los_table.rs", 37) \ #github-link("omdurman-rules/src/los_table.rs", 37)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L37")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[los_table]]]], [#raw(" 35 │ /// If the cell says \"Blocks\", LOS is blocked; otherwise it is clear
 36 │ /// (subject to the special notes below).
 37 │ pub fn los_table(firer: LosFirerTerrain, target: LosTargetTerrain) -> LosResult {
 38 │     use LosFirerTerrain as F;
 39 │     use LosResult::*;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 91) \ #github-link("omdurman-rules/src/los_table.rs", 91)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L91")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_los]]]], [#raw(" 89 │ /// [`GameState`](crate::effects::GameState)) so the engine can validate LOS
 90 │ /// without reaching into the app/Bevy layer.
 91 │ pub fn has_los(
 92 │     board: &crate::board::BoardInfo,
 93 │     from: omdurman_types::HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_fire_at_rejects_blocked_los]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_fire_at_allows_clear_los]]]
#v(0.3em)
#heading(level: 2, "§6.22 -- Consult Range Effects Table") <sect-6-22>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Next consult the Range Effects Table to see if the firing unit's fire combat factor is tripled, doubled, normal, halved, or if the target hex is out of range. Add up the total number of fire combat factors firing at the enemy-occupied hex.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/range_effects.rs", 23) \ #github-link("omdurman-rules/src/range_effects.rs", 23)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/range_effects.rs#L23")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ae_range_effects]]]], [#raw(" 21 │ /// Look up the range band for an Anglo-Egyptian weapon (§6.22, §6.24).
 22 │ /// Distances > 10 are out of range for all weapons.
 23 │ pub fn ae_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
 24 │     if distance.value() > 10 {
 25 │         return RangeBand::OutOfRange;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/range_effects.rs", 61) \ #github-link("omdurman-rules/src/range_effects.rs", 61)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/range_effects.rs#L61")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_range_effects]]]], [#raw(" 59 │ /// Look up the range band for a Dervish weapon (§6.22).
 60 │ /// Distances > 10 are out of range for all weapons.
 61 │ pub fn dervish_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
 62 │     if distance.value() > 10 {
 63 │         return RangeBand::OutOfRange;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 481) \ #github-link("omdurman-rules/src/lib.rs", 481)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L481")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Range]]]], [#raw("479 │ /// multiplied at a given distance (§6.22).
480 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
481 │ pub enum RangeBand {
482 │     Tripled,
483 │     Doubled,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 481) \ #github-link("omdurman-rules/src/lib.rs", 481)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L481")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RangeBand]]]], [#raw("479 │ /// multiplied at a given distance (§6.22).
480 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
481 │ pub enum RangeBand {
482 │     Tripled,
483 │     Doubled,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 160) \ #github-link("omdurman-rules/src/lib.rs", 160)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L160")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDistance]]]], [#raw("158 │ /// (rulebook §6.22, §7.5).
159 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
160 │ pub struct HexDistance(pub u16);
161 │ 
162 │ impl HexDistance {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_rifles_doubled_at_range_1]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_rifles_halved_at_range_4]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_howitzer_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_rifles_shorter_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[melee_only_range_1]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_range_effects_artillery_full]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_range_effects_maxims_match_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_range_effects_distance_over_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_range_effects_howitzer_halved_4_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_maxims_and_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_melee]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_distance_over_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_combat_eliminates_target]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[max_day_range_all_combos]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[range_effects_every_cell_ae_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[range_effects_every_cell_ae_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[range_effects_every_cell_ae_maxims]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[range_effects_every_cell_ae_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[range_effects_every_cell_dervish_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[range_effects_every_cell_dervish_maxims_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[range_effects_every_cell_dervish_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[range_effects_every_cell_dervish_spears]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_fire_at_gates_phase_range_and_player]]]
#v(0.3em)
#heading(level: 2, "§6.23 -- Terrain defensive modifier") <sect-6-23>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Next check the Terrain Effects Chart to see if the enemy-occupied hex fired upon contains any terrain which gives the enemy units in that hex a defensive benefit. If so, apply this negative modifier to the roll of the ten-sided die and cross-index your net die roll on the Combat Results Table with the number of combat factors firing.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 59) \ #github-link("omdurman-rules/src/terrain_chart.rs", 59)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/terrain_chart.rs#L59")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[defense_modifier]]]], [#raw(" 57 │ 
 58 │ /// Convenience: get the defense modifier for a terrain type (rulebook §6.23, Terrain Effects Chart).
 59 │ pub fn defense_modifier(terrain: Terrain) -> i16 {
 60 │     terrain_effects_chart(terrain).defense_modifier
 61 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 815) \ #github-link("omdurman-rules/src/lib.rs", 815)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L815")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain]]]], [#raw("813 │     /// Negative modifier from the Terrain Effects Chart applied to the
814 │     /// defender's hex (§6.23).
815 │     Terrain(i16),
816 │     /// -2 thorn-hedge defensive modifier (§9.231).
817 │     ZaribaThornHedge,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 233) \ #github-link("omdurman-types/src/lib.rs", 233)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L233")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::blocks_los]]]], [#raw("231 │     /// Whether this hexside blocks line of sight across it (§6.3). Crest is
232 │     /// directional and handled by the caller; here it is treated as blocking.
233 │     pub fn blocks_los(self) -> bool {
234 │         matches!(self, HexsideKind::Wall | HexsideKind::Crest)
235 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 439) \ #github-link("omdurman-types/src/lib.rs", 439)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L439")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::is_los_trees]]]], [#raw("437 │     /// Whether this terrain counts as \"trees\" for the LOS palm-grove rule
438 │     /// (§6.3 note 1).
439 │     pub fn is_los_trees(self) -> bool {
440 │         matches!(self, Terrain::Trees { .. })
441 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[clear_terrain_no_bonus]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[building_gives_minus_3]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[palm_grove_gives_minus_2]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[rough_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[swamp_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[hilltop_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[huts_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[defense_modifier_convenience_matches_chart]]]
#v(0.3em)
#heading(level: 2, "§6.24 -- Anglo-Egyptian direct fire accuracy bonus and brigade integrity") <sect-6-24>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[All Anglo-Egyptian direct fire attacks receive a +1 modifier to their die roll as an accuracy bonus. In addition, any stack of Anglo-Egyptian infantry having brigade integrity (see #link(<sect-5-54>)[5.54]) receives a +1 modifier to their die roll if all four fire at the same enemy-occupied hex. These modifiers are cumulative.]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-54>)[§5.54]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 809) \ #github-link("omdurman-rules/src/lib.rs", 809)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L809")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AngloEgyptianDirectFire]]]], [#raw("807 │ pub enum FireModifier {
808 │     /// +1 to all Anglo-Egyptian *direct* fire (§6.24).
809 │     AngloEgyptianDirectFire,
810 │     /// +1 brigade integrity, applied only if all four battalions fire at
811 │     /// the same enemy-occupied hex (§5.54, §6.24).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 794) \ #github-link("omdurman-rules/src/lib.rs", 794)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L794")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeIntegrity]]]], [#raw("792 │ /// stack contains all four battalions of a single Anglo-Egyptian brigade.
793 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
794 │ pub enum BrigadeIntegrity {
795 │     None,
796 │     Integrated(BrigadeId),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 825) \ #github-link("omdurman-rules/src/lib.rs", 825)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L825")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[die_modifier]]]], [#raw("823 │ impl FireModifier {
824 │     /// Return the numeric die-roll modifier for this bonus/penalty (rulebook §6.24, §5.54, §6.23, §9.231, §9.232).
825 │     pub fn die_modifier(self) -> i16 {
826 │         match self {
827 │             FireModifier::AngloEgyptianDirectFire | FireModifier::BrigadeIntegrity => 1,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.41 -- Direct Fire Subphase") <sect-6-41>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 296) \ #github-link("omdurman-rules/src/lib.rs", 296)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L296")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DirectFire]]]], [#raw("294 │ pub enum FireSubPhase {
295 │     /// Direct fire (§6.41). Both sides participate in this sub-phase.
296 │     DirectFire,
297 │     /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
298 │     MaximSecondAndHowitzer,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.42 -- Maxim Second Fire and Howitzer Fire Subphase") <sect-6-42>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 298) \ #github-link("omdurman-rules/src/lib.rs", 298)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L298")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MaximSecondAndHowitzer]]]], [#raw("296 │     DirectFire,
297 │     /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
298 │     MaximSecondAndHowitzer,
299 │ }
300 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 768) \ #github-link("omdurman-types/src/lib.rs", 768)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L768")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[fires_twice]]]], [#raw("766 │     /// again in the Maxim Second Fire Subphase (rulebook §6.42). The counter
767 │     /// is marked \"x2\" in the editor to surface this.
768 │     pub fn fires_twice(self) -> bool {
769 │         matches!(self, UnitKind::Maxim)
770 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_on_target_7_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_scatters_below_7]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_short_on_5_6]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_long_on_3_4]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_left_right_on_1_2]]]
#v(0.3em)
#heading(level: 2, "§6.51 -- Leader Units") <sect-6-51>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 395) \ #github-link("omdurman-rules/src/lib.rs", 395)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L395")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BritishLeader]]]], [#raw("393 │ /// to claim the Mahdi's Tomb (§9.14).
394 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
395 │ pub enum BritishLeader {
396 │     Kitchener,
397 │     Gatacre,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 730) \ #github-link("omdurman-types/src/lib.rs", 730)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L730")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BritishLeaderUnit]]]], [#raw("728 │     DervishLeaderUnit,
729 │     /// Anglo-Egyptian leader: movement only (§6.51).
730 │     BritishLeaderUnit,
731 │ }
732 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 761) \ #github-link("omdurman-types/src/lib.rs", 761)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L761")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_combat_factors]]]], [#raw("759 │     /// British leaders print a movement factor only (§6.51); other kinds carry
760 │     /// fire and/or melee factors.
761 │     pub fn has_combat_factors(self) -> bool {
762 │         !matches!(self, UnitKind::BritishLeaderUnit)
763 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[zero_factor_is_none_not_zero]]]
#v(0.3em)
#heading(level: 2, "§6.52 -- Anglo-Egyptian Friendlies Brigade") <sect-6-52>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 580) \ #github-link("omdurman-rules/src/lib.rs", 580)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L580")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_friendlies]]]], [#raw("578 │     /// \"Friendlies\" units obey several special rules (§5.21, §5.23, §6.52,
579 │     /// §9.14 victory conditions).
580 │     pub fn is_friendlies(&self) -> bool {
581 │         matches!(
582 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 807) \ #github-link("omdurman-types/src/lib.rs", 807)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L807")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Friendlies]]]], [#raw("805 │     /// Native volunteer brigade -- the Shaggyeh (§6.52). Do not receive
806 │     /// brigade integrity (§5.54 enumerates only British/Egyptian/Sudanese).
807 │     Friendlies,
808 │ }
809 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.53 -- Royal Engineers demolition") <sect-6-53>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 549) \ #github-link("omdurman-rules/src/lib.rs", 549)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L549")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RoyalEngineers]]]], [#raw("547 │     /// The Royal Engineers (§6.53) -- a *specific* unit, not a class, so we
548 │     /// model it explicitly.
549 │     RoyalEngineers,
550 │ }
551 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 707) \ #github-link("omdurman-rules/src/lib.rs", 707)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L707")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[demolishing]]]], [#raw("705 │     /// Set when the Royal Engineers are committed to a demolition this turn
706 │     /// (§6.53) -- neither offensive fire nor melee allowed that turn.
707 │     pub demolishing: bool,
708 │     /// Set when a gunboat has lost its engines to a river mine (§10.12, roll
709 │     /// 5-7): it may no longer move under power and instead drifts two hexes per", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 115) \ #github-link("omdurman-rules/src/effects.rs", 115)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L115")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Demolition]]]], [#raw("113 │ 
114 │     /// Royal Engineers demolition (rulebook §6.53).
115 │     Demolition {
116 │         unit_id: UnitId,
117 │         target: DemolitionTarget,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2869) \ #github-link("omdurman-rules/src/effects.rs", 2869)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2869")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_demolition]]]], [#raw("2867 │ /// resolution happens at end of turn via [`apply_resolve_demolition`], which
2868 │ /// checks the engineer is still adjacent and undisrupted.
2869 │ pub fn apply_demolition(
2870 │     state: &mut GameState,
2871 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 950) \ #github-link("omdurman-rules/src/lib.rs", 950)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L950")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DemolitionTarget]]]], [#raw("948 │ /// disrupted or driven off.
949 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
950 │ pub enum DemolitionTarget {
951 │     Fort(UnitId),
952 │     WallHexside(HexsideRef),", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[demolition_cancelled_when_engineer_disrupted]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[demolition_cancelled_when_engineer_moved_away]]]
#v(0.3em)
#heading(level: 2, "§6.54 -- Forts") <sect-6-54>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 756) \ #github-link("omdurman-rules/src/lib.rs", 756)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L756")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("754 │ /// Used by the engine when answering \"is this hex in an enemy ZOC?\".
755 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
756 │ pub enum ZocReason {
757 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
758 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 726) \ #github-link("omdurman-types/src/lib.rs", 726)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L726")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Fort]]]], [#raw("724 │     Gunboat,
725 │     /// Permanent emplacement -- may not move once placed (§5.25).
726 │     Fort,
727 │     /// Dervish leader: has fire/melee/movement factors and may melee attack.
728 │     DervishLeaderUnit,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 717) \ #github-link("omdurman-rules/src/lib.rs", 717)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L717")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitState::may_act]]]], [#raw("715 │ impl UnitState {
716 │     /// A disrupted unit may not move, fire, or melee (rulebook §5, reference notes).
717 │     pub fn may_act(self) -> bool {
718 │         !self.disrupted
719 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 695) \ #github-link("omdurman-rules/src/lib.rs", 695)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L695")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitState]]]], [#raw("693 │ /// rather than one big enum.
694 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
695 │ pub struct UnitState {
696 │     /// Reference table: \"Disrupted units: no ZOC; may not move; may not fire
697 │     /// offensively or defensively; may not melee; are turned face up at the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 854) \ #github-link("omdurman-rules/src/lib.rs", 854)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L854")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireAttack]]]], [#raw("852 │ /// modifiers (rulebook §6).
853 │ #[derive(Serialize, Deserialize, Clone, Debug)]
854 │ pub struct FireAttack {
855 │     pub firing_player: Player,
856 │     pub phase: Phase,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 869) \ #github-link("omdurman-rules/src/lib.rs", 869)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L869")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireAttack::net_modifier]]]], [#raw("867 │ impl FireAttack {
868 │     /// Sum of all fire modifiers applied to this attack (rulebook §6.24).
869 │     pub fn net_modifier(&self) -> i16 {
870 │         self.modifiers.iter().map(|m| m.die_modifier()).sum()
871 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.61 -- Only artillery may fire at gunboats; 3+ to sink") <sect-6-61>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only artillery may fire at gunboats. A result of 3 or more on the combat results table is required to sink a gunboat. Any other result is a miss.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 446) \ #github-link("omdurman-rules/src/lib.rs", 446)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L446")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("444 │ /// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
445 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
446 │ pub enum WeaponClass {
447 │     /// Dervish spears and swords -- no ranged fire at all.
448 │     Melee,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[rifles_may_not_sink_a_gunboat]]]
#v(0.3em)
#heading(level: 2, "§6.62 -- Only artillery may fire at forts; 2+ to destroy") <sect-6-62>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only artillery may fire at forts. A result of 2 or more on the combat results table is required to eliminate a fort. Any other result is a miss. If the fort contains any enemy units at the instant it is destroyed, one unit is eliminated with the fort.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 446) \ #github-link("omdurman-rules/src/lib.rs", 446)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L446")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("444 │ /// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
445 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
446 │ pub enum WeaponClass {
447 │     /// Dervish spears and swords -- no ranged fire at all.
448 │     Melee,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.63 -- Only artillery may breach wall hexsides; 2+ required") <sect-6-63>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only artillery may fire to breach a wall hexside of Khartoum or the walled city of Omdurman. A result of 2 or more on the combat results table is required to breach a wall. Any other result is a miss. The effect of the breach is to negate the wall hexside for line of sight purposes. Place a "BREACH" marker in an adjacent hex so that the arrow points to the breached hexside. If any enemy units are adjacent to the wall hexside at the instant it is breached, one enemy unit is eliminated.

\*\*#link(<sect-6-64>)[6.64]) Howitzer fire:\*\*
Five units in the game have howitzer fire capability. These are the five named British gunboats. They may fire their artillery factor as direct fire during the Direct Fire Subphase (see 4 and #link(<sect-6-41>)[6.41]) and may then fire the same artillery factor as howitzer fire during the Maxim Second Fire and Howitzer Subphase (see 4 and #link(<sect-6-42>)[6.42]). Exception: no howitzer fire is allowed during night game turns. To fire howitzer fire, select any target hex between 4 and 10 hexes from the firing gunboat (ignoring the Line of Sight Table) and roll the ten-sided die twice. The first die roll is the Combat Results Table die roll and the second roll is the impact hex die roll. Refer to the Howitzer Fire Scattergram on the mapsheet for the impact hex. The designated target hex is hit on a roll of 7–10. Once a howitzer fire die roll has been made the results must take effect, even if the fire scatters into a friendly-occupied hex.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-6-41>)[§6.41], #link(<sect-6-42>)[§6.42], #link(<sect-6-64>)[§6.64]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 203) \ #github-link("omdurman-types/src/lib.rs", 203)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L203")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Breach]]]], [#raw("201 │     /// Breach in a wall (artillery/§6.63 or Royal Engineers/§6.53). ZOC both
202 │     /// ways; LOS no longer blocked across the hexside.
203 │     Breach,
204 │     /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
205 │     /// combat may not cross (§6.82).", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[breech_marker_cell_returns_none]]]
#v(0.3em)
#heading(level: 2, "§6.64 -- Howitzer fire") <sect-6-64>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/howitzer_scatter.rs", 6) \ #github-link("omdurman-rules/src/howitzer_scatter.rs", 6)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/howitzer_scatter.rs#L6")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ScatterDirection]]]], [#raw("  4 │ /// (§6.64). The caller maps these to hex-grid offsets.
  5 │ #[derive(Clone, Copy, PartialEq, Eq, Debug)]
  6 │ pub enum ScatterDirection {
  7 │     /// Roll 7-10: hit the target hex.
  8 │     OnTarget,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/howitzer_scatter.rs", 28) \ #github-link("omdurman-rules/src/howitzer_scatter.rs", 28)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/howitzer_scatter.rs#L28")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[howitzer_scatter]]]], [#raw(" 26 │ /// | 3-4  | [`ScatterDirection::Long`] (upstream) |
 27 │ /// | 1-2  | [`ScatterDirection::LeftRight`] |
 28 │ pub fn howitzer_scatter(impact_roll: DieRoll) -> ScatterDirection {
 29 │     use DieRoll::*;
 30 │     match impact_roll {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 406) \ #github-link("omdurman-rules/src/lib.rs", 406)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L406")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId]]]], [#raw("404 │ /// fire; \"old\" gunboats do not (rulebook §2.32).
405 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
406 │ pub enum GunboatId {
407 │     /// One of the five new-type named gunboats with howitzer capability.
408 │     Named(NamedGunboat),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 64) \ #github-link("omdurman-rules/src/effects.rs", 64)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L64")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HowitzerFire]]]], [#raw(" 62 │ 
 63 │     /// Resolve a howitzer bombardment (two rolls: Combat Results Table + impact scatter) (rulebook §6.64).
 64 │     HowitzerFire {
 65 │         attack: FireAttack,
 66 │         combat_results_table_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2082) \ #github-link("omdurman-rules/src/effects.rs", 2082)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2082")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_howitzer_fire]]]], [#raw("2080 │ 
2081 │ /// Validate and apply a howitzer fire attack (scatter path) (rulebook §6.64).
2082 │ pub fn apply_howitzer_fire(
2083 │     state: &mut GameState,
2084 │     attack: &FireAttack,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 890) \ #github-link("omdurman-rules/src/lib.rs", 890)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L890")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HowitzerResolution]]]], [#raw("888 │ /// roll on the Howitzer Fire Scattergram (§6.64).
889 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
890 │ pub struct HowitzerResolution {
891 │     pub combat_results_table_roll: DieRoll,
892 │     pub impact_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 897) \ #github-link("omdurman-rules/src/lib.rs", 897)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L897")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HowitzerResolution::hit_target_hex]]]], [#raw("895 │ impl HowitzerResolution {
896 │     /// The designated target hex is hit on impact roll 7-10 (§6.64).
897 │     pub fn hit_target_hex(self) -> bool {
898 │         use DieRoll::*;
899 │         matches!(self.impact_roll, Seven | Eight | Nine | Ten)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1084) \ #github-link("omdurman-rules/src/effects.rs", 1084)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1084")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_fire_at]]]], [#raw("1082 │     /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
1083 │     /// (Howitzer fire ignores LOS entirely -- §6.64.)
1084 │     pub fn can_fire_at(
1085 │         &self,
1086 │         firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_on_target_7_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_scatters_below_7]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_short_on_5_6]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_long_on_3_4]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_left_right_on_1_2]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[named_and_old_gunboats_resolve]]]
#v(0.3em)
#heading(level: 2, "§6.81 -- Moving player may fire with all capable units") <sect-6-81>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[During Offensive Fire phase, the moving player may fire with all of his units capable of firing, up to their maximum range, within the limitations imposed by the rules of combat.]]
#v(0.5em)
#heading(level: 2, "§6.82 -- Advance after combat (offensive fire)") <sect-6-82>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[If an enemy-occupied hex is vacated as a result of offensive fire, friendly units may advance after combat into the vacated hex. To be eligible to advance, the friendly units must have participated in the attack and must have been adjacent to the vacated hex. Note that artillery may not advance, nor may units advance across a wall hexside (except at a gate or breach). Units may never advance after combat across a khor.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 102) \ #github-link("omdurman-rules/src/effects.rs", 102)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L102")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw("100 │     /// after fire, §7.6 after melee). Eligible units are adjacent attackers
101 │     /// that are not artillery; the target hex must be empty of enemies.
102 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
103 │ 
104 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2807) \ #github-link("omdurman-rules/src/effects.rs", 2807)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2807")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_advance_after_combat]]]], [#raw("2805 │ 
2806 │ /// Apply an advance-after-combat for a unit (rulebook §6.82, §7.6).
2807 │ pub fn apply_advance_after_combat(
2808 │     state: &mut GameState,
2809 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 102) \ #github-link("omdurman-rules/src/effects.rs", 102)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L102")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw("100 │     /// after fire, §7.6 after melee). Eligible units are adjacent attackers
101 │     /// that are not artillery; the target hex must be empty of enemies.
102 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
103 │ 
104 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 244) \ #github-link("omdurman-types/src/lib.rs", 244)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L244")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_advance_after_combat]]]], [#raw("242 │ 
243 │     /// Whether advance-after-combat may *not* cross this side (§6.82, §7.6).
244 │     pub fn blocks_advance_after_combat(self) -> bool {
245 │         matches!(
246 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2684) \ #github-link("omdurman-rules/src/effects.rs", 2684)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2684")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("2682 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
2683 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
2684 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2685 │         let unit = self
2686 │             .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_advance_after_combat_rejects_wall_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_advance_after_combat_rejects_khor_hexside]]]
#v(0.3em)
#progress-bar(7, 7)
#heading(level: 1, "§7 -- Melee Phase") <sect-7>
#heading(level: 2, "§7.1 -- Melee strength printed on counter") <sect-7-1>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The melee strength of all units is printed on the counter. Note that gunboats have no melee strength. Gunboats may neither melee attack nor be melee attacked.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 94) \ #github-link("omdurman-rules/src/lib.rs", 94)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L94")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeFactor]]]], [#raw(" 92 │     /// Every possible value from the annotated counter set is a named variant.
 93 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
 94 │     pub enum MeleeFactor {
 95 │         One = 1,
 96 │         Three = 3,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 105) \ #github-link("omdurman-rules/src/lib.rs", 105)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L105")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeFactor::sum]]]], [#raw("103 │ impl MeleeFactor {
104 │     /// Sum multiple melee factors (rulebook §7.1).
105 │     pub fn sum<'a>(factors: impl IntoIterator<Item = &'a MeleeFactor>) -> u16 {
106 │         factors.into_iter().map(|f| f.value()).sum()
107 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 744) \ #github-link("omdurman-types/src/lib.rs", 744)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L744")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_be_melee_attacked]]]], [#raw("742 │ 
743 │     /// Gunboats neither attack nor are attacked in melee (§7.1).
744 │     pub fn may_be_melee_attacked(self) -> bool {
745 │         !matches!(self, UnitKind::Gunboat)
746 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 744) \ #github-link("omdurman-types/src/lib.rs", 744)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L744")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind::may_be_melee_attacked]]]], [#raw("742 │ 
743 │     /// Gunboats neither attack nor are attacked in melee (§7.1).
744 │     pub fn may_be_melee_attacked(self) -> bool {
745 │         !matches!(self, UnitKind::Gunboat)
746 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.2 -- Melee adjacent only, not across wall hexsides") <sect-7-2>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Melee simulates the hand-to-hand fighting of the period. Units may melee attack adjacent enemy units only. Units may not melee attack across a wall hexside, but may melee attack through a gate or breach hexside.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 239) \ #github-link("omdurman-types/src/lib.rs", 239)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L239")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_melee]]]], [#raw("237 │     /// Whether melee may *not* be made across this side (§7.2). Gates and
238 │     /// breaches are passable to melee.
239 │     pub fn blocks_melee(self) -> bool {
240 │         matches!(self, HexsideKind::Wall | HexsideKind::ZaribaThornHedge)
241 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1224) \ #github-link("omdurman-rules/src/effects.rs", 1224)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1224")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_melee]]]], [#raw("1222 │     /// that may be melee-attacked (gunboats may not -- §7.1), and no wall or
1223 │     /// thorn-hedge hexside blocks the attack (§7.2).
1224 │     pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
1225 │         let unit = self
1226 │             .find_unit(attacker)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_melee_gates_phase_adjacency_and_kind]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_melee_rejects_wall_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_melee_rejects_thorn_hedge_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[can_melee_allows_gate_hexside]]]
#v(0.3em)
#heading(level: 2, "§7.3 -- Simultaneous melee combat") <sect-7-3>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Melee combat is considered simultaneous, so that units eliminated by melee attacks still get a melee combat die roll.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 74) \ #github-link("omdurman-rules/src/effects.rs", 74)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L74")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeCombat]]]], [#raw(" 72 │     /// Used for an immediate resolution with no reaction window (and as the
 73 │     /// resolution primitive in tests).
 74 │     MeleeCombat {
 75 │         attack: MeleeAttack,
 76 │         attacker_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2356) \ #github-link("omdurman-rules/src/effects.rs", 2356)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2356")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_melee_combat]]]], [#raw("2354 │ 
2355 │ /// Apply a simultaneous melee combat between two adjacent hexes (rulebook §7).
2356 │ pub fn apply_melee_combat(
2357 │     state: &mut GameState,
2358 │     attack: &MeleeAttack,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.4 -- Who may melee attack / defend") <sect-7-4>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only infantry, cavalry, camel units, and Dervish leaders may melee attack. All units (except gunboats — see #link(<sect-7-1>)[7.1]) may melee defend.]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-7-1>)[§7.1]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 736) \ #github-link("omdurman-types/src/lib.rs", 736)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L736")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_melee_attack]]]], [#raw("734 │     /// Rulebook §7.4 -- only infantry, cavalry, camel and Dervish leaders may
735 │     /// melee *attack*. All others (except gunboats) may melee *defend* (§7.1).
736 │     pub fn may_melee_attack(self) -> bool {
737 │         matches!(
738 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 716) \ #github-link("omdurman-types/src/lib.rs", 716)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L716")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind]]]], [#raw("714 │     strum::EnumIter,
715 │ )]
716 │ pub enum UnitKind {
717 │     /// Foot infantry. Includes Anglo-Egyptian infantry, \"Friendlies\",
718 │     /// Royal Engineers, and Dervish foot tribes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 322) \ #github-link("omdurman-rules/src/lib.rs", 322)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L322")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishTribe]]]], [#raw("320 │ // ---------------------------------------------------------------------------
321 │ 
322 │ pub use omdurman_types::DervishTribe;
323 │ 
324 │ /// Anglo-Egyptian infantry brigades -- designation printed on the counter", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1224) \ #github-link("omdurman-rules/src/effects.rs", 1224)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1224")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_melee]]]], [#raw("1222 │     /// that may be melee-attacked (gunboats may not -- §7.1), and no wall or
1223 │     /// thorn-hedge hexside blocks the attack (§7.2).
1224 │     pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
1225 │         let unit = self
1226 │             .find_unit(attacker)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.5 -- Cavalry/camel retreat before melee") <sect-7-5>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Cavalry and camel units may retreat two hexes from an infantry melee attack. Note, however, that only one retreat per unit per turn is permitted. Thus, if their retreat places them adjacent to enemy units whose melee attacks have not yet been resolved, those enemy units may elect to attack the retreating unit(s).]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 97) \ #github-link("omdurman-rules/src/effects.rs", 97)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L97")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RetreatBeforeMelee]]]], [#raw(" 95 │     /// melee attack, *before* it is resolved (§7.5). One retreat per unit per
 96 │     /// turn. (rulebook §7.5).
 97 │     RetreatBeforeMelee { unit_id: UnitId, to: HexCoord },
 98 │ 
 99 │     /// An attacking unit advances into a hex vacated by combat (rulebook §6.82", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2783) \ #github-link("omdurman-rules/src/effects.rs", 2783)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2783")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_retreat_before_melee]]]], [#raw("2781 │ 
2782 │ /// Apply a retreat-before-melee for a cavalry/camel unit (rulebook §7.5).
2783 │ pub fn apply_retreat_before_melee(
2784 │     state: &mut GameState,
2785 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 750) \ #github-link("omdurman-types/src/lib.rs", 750)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L750")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_retreat_before_melee]]]], [#raw("748 │     /// Cavalry and camel units may retreat two hexes from an infantry melee
749 │     /// attack (§7.5).
750 │     pub fn may_retreat_before_melee(self) -> bool {
751 │         matches!(self, UnitKind::Cavalry | UnitKind::Camel)
752 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 160) \ #github-link("omdurman-rules/src/lib.rs", 160)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L160")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDistance]]]], [#raw("158 │ /// (rulebook §6.22, §7.5).
159 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
160 │ pub struct HexDistance(pub u16);
161 │ 
162 │ impl HexDistance {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2639) \ #github-link("omdurman-rules/src/effects.rs", 2639)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2639")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_retreat_before_melee]]]], [#raw("2637 │     /// two hexes away and empty. (Does not verify the attacker is infantry --
2638 │     /// the caller offers the retreat only in response to one.)
2639 │     pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2640 │         let unit = self
2641 │             .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[retreat_before_melee_only_cavalry_two_hexes]]]
#v(0.3em)
#heading(level: 2, "§7.6 -- Advance after melee") <sect-7-6>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[If a melee attack eliminates all of the defenders in an adjacent hex, the Dervish player MUST advance into the vacated hex. To be eligible to advance, the Dervish units must have been adjacent to the vacated hex and participated in the melee attack that eliminated the defenders. All surviving eligible Dervish units MUST advance, up to the stacking limit. The Anglo-Egyptian player may advance if desired. Note that only attacking units may advance.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 102) \ #github-link("omdurman-rules/src/effects.rs", 102)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L102")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw("100 │     /// after fire, §7.6 after melee). Eligible units are adjacent attackers
101 │     /// that are not artillery; the target hex must be empty of enemies.
102 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
103 │ 
104 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2807) \ #github-link("omdurman-rules/src/effects.rs", 2807)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2807")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_advance_after_combat]]]], [#raw("2805 │ 
2806 │ /// Apply an advance-after-combat for a unit (rulebook §6.82, §7.6).
2807 │ pub fn apply_advance_after_combat(
2808 │     state: &mut GameState,
2809 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2684) \ #github-link("omdurman-rules/src/effects.rs", 2684)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2684")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("2682 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
2683 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
2684 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2685 │         let unit = self
2686 │             .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.7 -- Melee modifiers") <sect-7-7>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[To resolve melee, both the attacker and the defender roll on the Combat Results Table and apply the applicable melee modifier to their die roll. The Dervish player receives a +2 melee modifier, the Anglo-Egyptian player receives a +1 melee modifier. No terrain modifiers are applied to melee combat (Exception: Zariba hexsides in the historical scenario and the campaign game, if constructed — see #link(<sect-9-23>)[9.23]). Melee losses must be taken from meleeing units first!]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-9-23>)[§9.23]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 908) \ #github-link("omdurman-rules/src/lib.rs", 908)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L908")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier]]]], [#raw("906 │ 
907 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
908 │ pub enum MeleeModifier {
909 │     /// +2 to all Dervish melee rolls (§7.7).
910 │     DervishStandard,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 201) \ #github-link("omdurman-rules/src/lib.rs", 201)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L201")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DieModifier]]]], [#raw("199 │ /// A die-roll modifier from a single named source (rulebook §6.24, §7.7).
200 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
201 │ pub enum DieModifier {
202 │     #[default]
203 │     Zero,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 931) \ #github-link("omdurman-rules/src/lib.rs", 931)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L931")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeAttack]]]], [#raw("929 │ /// A melee attack: simultaneous, both sides roll on the Combat Results Table (§7.3, §7.7).
930 │ #[derive(Serialize, Deserialize, Clone, Debug)]
931 │ pub struct MeleeAttack {
932 │     pub attacker_player: Player,
933 │     pub attacker_hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 912) \ #github-link("omdurman-rules/src/lib.rs", 912)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L912")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::AngloEgyptianStandard]]]], [#raw("910 │     DervishStandard,
911 │     /// +1 to all Anglo-Egyptian melee rolls (§7.7).
912 │     AngloEgyptianStandard,
913 │     /// Inverted to -2 when Dervish units melee-attack across a trench into
914 │     /// an entrenched defender (§9.232).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 915) \ #github-link("omdurman-rules/src/lib.rs", 915)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L915")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::DervishVsTrenchedDefender]]]], [#raw("913 │     /// Inverted to -2 when Dervish units melee-attack across a trench into
914 │     /// an entrenched defender (§9.232).
915 │     DervishVsTrenchedDefender,
916 │ }
917 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 910) \ #github-link("omdurman-rules/src/lib.rs", 910)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L910")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::DervishStandard]]]], [#raw("908 │ pub enum MeleeModifier {
909 │     /// +2 to all Dervish melee rolls (§7.7).
910 │     DervishStandard,
911 │     /// +1 to all Anglo-Egyptian melee rolls (§7.7).
912 │     AngloEgyptianStandard,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[melee_resolves_simultaneously]]]
#v(0.3em)
#progress-bar(2, 2)
#heading(level: 1, "§8 -- Night Game Turns") <sect-8>
#heading(level: 2, "§8.1 -- Night effects") <sect-8-1>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The effects of night game turns are: a) all Anglo-Egyptian movement allowances are halved (round down), b) there is no Anglo-Egyptian howitzer fire, and c) all fire ranges are halved for both sides (round down, but range 1 stays range 1). Range effects on fire combat are the same as during day game turns. For example, an Anglo-Egyptian infantry unit firing at night will be doubled at range 1, normal at range 2, and may not fire at range 3 or greater.

\*\*#link(<sect-8-2>)[8.2]) Dervish Desertion Roll:\*\* Once each campaign game, during the first night turn of the game, the Dervish player rolls one die to see how many of his units desert. The roll is made during the movement phase and the number of deserting Dervish units is equal to 1½ times the roll of one die. The Dervish player may choose which units desert by merely removing them from the mapsheet. The KHALIFA unit, gunboats, artillery units, and forts are the only Dervish units that may not be chosen. No victory points are awarded to the Anglo-Egyptian player for deserting Dervishes.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-8-2>)[§8.2]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 140) \ #github-link("omdurman-rules/src/lib.rs", 140)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L140")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementAllowance::halve]]]], [#raw("138 │ impl MovementAllowance {
139 │     /// Night movement allowance = halved (round down) (rulebook §8.1, §5.11).
140 │     pub fn halve(self) -> Self {
141 │         let v = self.value() / 2;
142 │         MovementAllowance::try_from(v).expect(\"halved value always a named variant\")", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 677) \ #github-link("omdurman-types/src/lib.rs", 677)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L677")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DayNight]]]], [#raw("675 │ /// (rulebook §8.1).
676 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
677 │ pub enum DayNight {
678 │     Day,
679 │     Night,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1309) \ #github-link("omdurman-rules/src/lib.rs", 1309)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1309")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[effective_range_at_night]]]], [#raw("1307 │ /// Apply night-turn range halving (§8.1): \"all fire ranges are halved for
1308 │ /// both sides (round down, but range 1 stays range 1).\"
1309 │ pub fn effective_range_at_night(range: HexDistance) -> HexDistance {
1310 │     if range.0 <= 1 {
1311 │         range", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1319) \ #github-link("omdurman-rules/src/lib.rs", 1319)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1319")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[effective_movement_at_night]]]], [#raw("1317 │ /// Apply night-turn movement halving for Anglo-Egyptian units (§8.1): all
1318 │ /// Anglo-Egyptian movement allowances are halved (round down).
1319 │ pub fn effective_movement_at_night(
1320 │     allowance: MovementAllowance,
1321 │     player: Player,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[night_max_ranges]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[night_max_ranges_remaining]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_rifle_at_night_matches_rulebook_example]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[max_day_range_all_combos]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[night_movement_overlay_allowance_halved]]]
#v(0.3em)
#heading(level: 2, "§8.2 -- Dervish Desertion Roll") <sect-8-2>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 130) \ #github-link("omdurman-rules/src/effects.rs", 130)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L130")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishDesertion]]]], [#raw("128 │     /// the effect. The Khalifa, gunboats, artillery, and forts may not be
129 │     /// chosen.
130 │     DervishDesertion {
131 │         roll: DieRoll,
132 │         deserters: Vec<UnitId>,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 62) \ #github-link("omdurman-rules/src/turn_track.rs", 62)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L62")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishDesertion]]]], [#raw(" 60 │     None,
 61 │     /// Dervish desertion roll (§8.2) -- occurs on the first night turn.
 62 │     DervishDesertion,
 63 │     /// Dervish reinforcements are available.
 64 │     DervishReinforcements,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 59) \ #github-link("omdurman-rules/src/turn_track.rs", 59)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L59")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnEvent]]]], [#raw(" 57 │ /// Special events that occur on specific turns (rulebook §8.2, §9.112, §9.113).
 58 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 59 │ pub enum TurnEvent {
 60 │     None,
 61 │     /// Dervish desertion roll (§8.2) -- occurs on the first night turn.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[desertion_count_is_floor_one_and_a_half]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[desertion_on_first_night]]]
#v(0.3em)
#progress-bar(17, 31)
#heading(level: 1, "§9 -- The Scenarios") <sect-9>
#heading(level: 2, "§9.1 -- The Campaign Game") <sect-9-1>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Campaign Game]]
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[campaign_has_no_fixed_placements]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[scenario_maps_to_board]]]
#v(0.3em)
#heading(level: 2, "§9.2 -- The Historical Scenario") <sect-9-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Historical Scenario

Players should note that the historical scenario is an exercise in futility for the Dervish player. It is, however, an interesting demonstration of the absolute imbecility of the Khalifa's generalship and vividly shows the superiority of entrenched firepower over traditional tribal arms in the colonial period.]]
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[start_game_scenario_selects_board]]]
#v(0.3em)
#heading(level: 2, "§9.3 -- Bonus Game: Fall of Khartoum") <sect-9-3>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Bonus Game: FALL OF KHARTOUM scenario]]
#v(0.5em)
#heading(level: 2, "§9.11 -- Set Up (Campaign)") <sect-9-11>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Set Up]]
#v(0.5em)
#heading(level: 2, "§9.12 -- Scenario Length (Campaign)") <sect-9-12>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Scenario Length

6:00 am, Sept. 1 through 8:00 am, Sept. 3, 22 Game Turns.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 10) \ #github-link("omdurman-rules/src/turn_track.rs", 10)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L10")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTime]]]], [#raw("  8 │ /// starts at one of these twelve times.
  9 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 10 │ pub enum GameTime {
 11 │     SixAM,
 12 │     EightAM,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 46) \ #github-link("omdurman-rules/src/turn_track.rs", 46)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L46")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnEntry]]]], [#raw(" 44 │ /// A single entry on the Turn Record Track (rulebook §9.12, §9.22).
 45 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 46 │ pub struct TurnEntry {
 47 │     /// 1-based turn number.
 48 │     pub turn: u8,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 94) \ #github-link("omdurman-rules/src/turn_track.rs", 94)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L94")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CAMPAIGN_TURN_TRACK]]]], [#raw(" 92 │ /// is turn 9, which carries the once-per-game Dervish Desertion Roll (§8.2) --
 93 │ /// the printed track prints \"Dervish Desertion Roll / NIGHT\" on that cell.
 94 │ pub const CAMPAIGN_TURN_TRACK: [TurnEntry; 22] = [
 95 │     // Row 1, left->right: Sept 1, 6 am -> 8 pm, then the first NIGHT.
 96 │     entry(1, SixAM, Day, TurnEvent::None),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 240) \ #github-link("omdurman-rules/src/turn_track.rs", 240)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L240")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnLabel]]]], [#raw("238 │ /// The track is a 9 × 3 grid with a snake layout.
239 │ #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
240 │ pub enum TurnLabel {
241 │     /// A cell that has no printed label (unused position in the 9×3 grid).
242 │     Blank,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 301) \ #github-link("omdurman-rules/src/turn_track.rs", 301)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L301")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[turn_marker_pixel]]]], [#raw("299 │ ///
300 │ /// Rows 0 and 1 use all 9 columns; row 2 uses only columns 0–3.
301 │ pub fn turn_marker_pixel(track: &omdurman_types::CampaignTurnTrack, turn: u8) -> (f32, f32) {
302 │     let cell_w = track.w / 9.0;
303 │     let cell_h = track.h / 3.0;", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[campaign_track_22_turns]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[desertion_on_first_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[campaign_track_label_and_day_night_agree]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[game_time_display_all_variants]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_label_display]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_label_out_of_range_is_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_marker_pixel_row_0_left_to_right]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_marker_pixel_row_1_right_to_left]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_marker_pixel_rows_are_stacked]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[game_over_after_campaign_turns]]]
#v(0.3em)
#heading(level: 2, "§9.13 -- Special Rules (Campaign)") <sect-9-13>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Rules

None.]]
#v(0.5em)
#heading(level: 2, "§9.14 -- Victory Conditions (Campaign)") <sect-9-14>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Victory Conditions

The Mahdi's Tomb in Omdurman was not only the tallest structure in the entire Sudan in 1898, it was also a Dervish holy shrine. Its loss or destruction would be a severe blow to the Mahdist cause. It is accordingly assigned 25 victory points which are awarded to the player who controls it at the conclusion of play. The Dervish player controls it at the start of play. As a tactical note, the Anglo-Egyptian player will find a decisive victory almost impossible unless he takes the Mahdi's Tomb from the Dervish player. To take the Tomb hex, it must be occupied by one British leader plus any one non-"Friendlies" Anglo-Egyptian combat unit (both undisrupted) at the conclusion of play.

Additional victory points are awarded as follows:

\*\*Dervish Player receives:\*\*
- 10 pts: each British leader eliminated
- 10 pts: each British gunboat sunk
- 1 pt: each "Friendlies" unit eliminated on the east bank side
- 3 pts: each "Friendlies" unit eliminated on the west bank (see #link(<sect-5-21>)[5.21])
- 3 pts: each Anglo-Egyptian land unit eliminated.

\*\*Anglo-Egyptian Player receives:\*\*
- No pts: eliminating forts
- 1 pt: eliminating Isa Zachneih unit
- 10 pts: eliminating KHALIFA ABDULLAH
- 1 pt: each Dervish unit eliminated, including gunboats, artillery and all other leaders.

At the conclusion of play, victory points are totaled and victory levels are assigned according to the following schedule:

| Victory Level | Dervish Player superiority | Anglo-Egyptian Player superiority |
|---|---|---|
| Decisive | 30+ points | 50+ points |
| Tactical | 20–29 points | 30–49 points |
| Marginal | 10–19 points | 15–29 points |
| Draw | 1–9 points | 1–14 points |

Alternatively, a decisive victory is awarded to the Anglo-Egyptian player if he eliminates every Dervish unit in play (including gunboats and forts). A decisive victory may be awarded the Dervish player if he eliminates all Anglo-Egyptian units on the west bank (excluding gunboats).]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-21>)[§5.21]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 1034) \ #github-link("omdurman-rules/src/lib.rs", 1034)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1034")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource]]]], [#raw("1032 │ /// the manual and the engine.
1033 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
1034 │ pub enum VpSource {
1035 │     // ----- Anglo-Egyptian player receives:
1036 │     /// Mahdi's Tomb control at conclusion of play (§9.14).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1060) \ #github-link("omdurman-rules/src/lib.rs", 1060)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1060")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource::points]]]], [#raw("1058 │ impl VpSource {
1059 │     /// VP awarded to `who_scores()` (rulebook §9.14).
1060 │     pub fn points(self) -> VictoryPoints {
1061 │         match self {
1062 │             VpSource::MahdisTomb => VictoryPoints(25),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1075) \ #github-link("omdurman-rules/src/lib.rs", 1075)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1075")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource::who_scores]]]], [#raw("1073 │ 
1074 │     /// Which player receives these victory points (rulebook §9.14).
1075 │     pub fn who_scores(self) -> Player {
1076 │         match self {
1077 │             VpSource::MahdisTomb", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1110) \ #github-link("omdurman-rules/src/lib.rs", 1110)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1110")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger]]]], [#raw("1108 │ /// Cumulative victory ledger for one scenario (rulebook §9.14).
1109 │ #[derive(Serialize, Deserialize, Clone, Debug, Default)]
1110 │ pub struct VictoryLedger {
1111 │     pub events: Vec<VpEvent>,
1112 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1116) \ #github-link("omdurman-rules/src/lib.rs", 1116)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1116")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpEvent]]]], [#raw("1114 │ /// A single victory-point scoring event (rulebook §9.14).
1115 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
1116 │ pub struct VpEvent {
1117 │     pub turn: GameTurnIndex,
1118 │     pub source: VpSource,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1123) \ #github-link("omdurman-rules/src/lib.rs", 1123)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1123")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger::total_for]]]], [#raw("1121 │ impl VictoryLedger {
1122 │     /// Total victory points earned by a given player (rulebook §9.14).
1123 │     pub fn total_for(&self, player: Player) -> VictoryPoints {
1124 │         VictoryPoints(
1125 │             self.events", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1135) \ #github-link("omdurman-rules/src/lib.rs", 1135)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1135")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger::superiority]]]], [#raw("1133 │     /// Net superiority: positive = Anglo-Egyptian ahead, negative = Dervish ahead
1134 │     /// (rulebook §9.14).
1135 │     pub fn superiority(&self) -> VictoryPoints {
1136 │         VictoryPoints(self.total_for(Player::AngloEgyptian).0 - self.total_for(Player::Dervish).0)
1137 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1153) \ #github-link("omdurman-rules/src/lib.rs", 1153)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1153")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CampaignVictoryLevel]]]], [#raw("1151 │ /// Campaign-game victory levels (§9.14).
1152 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
1153 │ pub enum CampaignVictoryLevel {
1154 │     Draw,
1155 │     Marginal(Player),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1162) \ #github-link("omdurman-rules/src/lib.rs", 1162)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1162")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CampaignVictoryLevel::from_superiority]]]], [#raw("1160 │ impl CampaignVictoryLevel {
1161 │     /// Assign a level from the net superiority (§9.14).
1162 │     pub fn from_superiority(s: VictoryPoints) -> Self {
1163 │         let net = s.0;
1164 │         // Positive -> Anglo-Egyptian thresholds: 15/30/50", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 3557) \ #github-link("omdurman-rules/src/effects.rs", 3557)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3557")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[score_elimination]]]], [#raw("3555 │ 
3556 │ /// Score victory points for eliminating a unit (rulebook §9.14).
3557 │ pub fn score_elimination(state: &mut GameState, unit_id: UnitId, _owner: Player) {
3558 │     if let Some(unit) = state.find_unit(unit_id) {
3559 │         let identity = unit.profile.identity;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 230) \ #github-link("omdurman-rules/src/lib.rs", 230)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L230")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryPoints]]]], [#raw("228 │ /// (rulebook §9.14).
229 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
230 │ pub struct VictoryPoints(pub i32);
231 │ 
232 │ /// One-based Game Turn index (1, 2, ... up to the scenario length) (rulebook §4).", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[friendlies_bank_scores_by_side]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[mahdis_tomb_not_scored_without_a_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[mahdis_tomb_scores_for_anglo_egyptian_when_held]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[vp_source_attributes]]]
#v(0.3em)
#heading(level: 2, "§9.21 -- Set Up (Historical)") <sect-9-21>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Set Up]]
#v(0.5em)
#heading(level: 2, "§9.22 -- Scenario Length (Historical)") <sect-9-22>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Scenario Length

6:00 am, September 2 through 12:00 noon, September 2. Four game turns.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 131) \ #github-link("omdurman-rules/src/turn_track.rs", 131)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L131")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HISTORICAL_TURN_TRACK]]]], [#raw("129 │ 
130 │ /// Historical scenario track (§9.22 -- 4 turns, Sept 2 6:00 am -> 12:00 pm).
131 │ pub const HISTORICAL_TURN_TRACK: [TurnEntry; 4] = [
132 │     TurnEntry {
133 │         turn: 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[historical_turn_all_four_turns]]]
#v(0.3em)
#heading(level: 2, "§9.23 -- Special Rule: The Zariba") <sect-9-23>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Rule: "The Zariba"

\*\*#link(<sect-9-231>)[9.231]) Thorn hedge hexsides:\*\* −2 to die roll on all Dervish fire attacks; may not melee across in either direction; may not advance after combat across in either direction.

\*\*#link(<sect-9-232>)[9.232]) Trench hexsides:\*\* −4 to die roll on all Dervish fire attacks vs. entrenched units only; −2 (instead of +2) melee modifier to Dervish units melee attacking an entrenched unit; entrenched units may be fired "over" in both directions (i.e. they do not block line of sight); units are considered "entrenched" if they are directly adjacent to (and on the Nile River side of) a trench hexside.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-9-231>)[§9.231], #link(<sect-9-232>)[§9.232]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 817) \ #github-link("omdurman-rules/src/lib.rs", 817)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L817")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("815 │     Terrain(i16),
816 │     /// -2 thorn-hedge defensive modifier (§9.231).
817 │     ZaribaThornHedge,
818 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
819 │     /// units (those Nile-side of the trench hexside).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 820) \ #github-link("omdurman-rules/src/lib.rs", 820)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L820")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrenchEntrenched]]]], [#raw("818 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
819 │     /// units (those Nile-side of the trench hexside).
820 │     ZaribaTrenchEntrenched,
821 │ }
822 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 211) \ #github-link("omdurman-types/src/lib.rs", 211)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L211")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("209 │     Crest,
210 │     /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
211 │     ZaribaThornHedge,
212 │     /// Historical-scenario trench segment of the Zariba (§9.232).
213 │     ZaribaTrench,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 213) \ #github-link("omdurman-types/src/lib.rs", 213)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L213")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrench]]]], [#raw("211 │     ZaribaThornHedge,
212 │     /// Historical-scenario trench segment of the Zariba (§9.232).
213 │     ZaribaTrench,
214 │     /// One of the two end hexsides of a Zariba trench segment that connect to
215 │     /// the Nile River (§9.233).  Units may only enter/leave the Zariba via", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.24 -- Victory Conditions (Historical)") <sect-9-24>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Victory Conditions

Victory Levels are based solely on eliminating enemy units while conserving your own force as much as possible.

| Victory Level | Anglo-Egyptian Player (Dervish units eliminated) | Dervish Player (Anglo-Egyptian units eliminated) |
|---|---|---|
| 5 — DECISIVE | 100+ | 30+ |
| 4 — STRATEGIC | 60–99 | 15–29 |
| 3 — TACTICAL | 45–59 | 10–14 |
| 2 — MARGINAL | 30–44 | 5–9 |
| 1 — DRAW | 0–29 | 0–4 |

The lower value victory level is then subtracted from the higher level to determine a player's net victory. For example, if the Anglo-Egyptian player eliminates 104 Dervish units (decisive victory) but loses 18 units doing it (Dervish Strategic), the Anglo-Egyptian player only nets out with a draw (decisive worth 5 minus strategic worth 4 = 1, draw).]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 1192) \ #github-link("omdurman-rules/src/lib.rs", 1192)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1192")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HistoricalVictoryLevel]]]], [#raw("1190 │ /// draw\").
1191 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
1192 │ pub enum HistoricalVictoryLevel {
1193 │     Draw = 1,
1194 │     Marginal = 2,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[historical_victory_level_for_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[historical_victory_level_for_anglo_egyptian]]]
#v(0.3em)
#heading(level: 2, "§9.31 -- Bonus game map") <sect-9-31>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only the small FALL OF KHARTOUM scenario map is used for this game.]]
#v(0.5em)
#heading(level: 2, "§9.32 -- Set Up (Bonus)") <sect-9-32>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Set Up]]
#v(0.5em)
#heading(level: 2, "§9.33 -- Scenario Length (Bonus)") <sect-9-33>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Scenario Length

Variable, see victory conditions (#link(<sect-9-35>)[9.35]). Rarely lasts five turns.]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-9-35>)[§9.35]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 171) \ #github-link("omdurman-rules/src/turn_track.rs", 171)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L171")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_TURN_TRACK]]]], [#raw("169 │ /// (the rulebook fixes none); only `day_night` is rule-bearing (night halves
170 │ /// Anglo-Egyptian movement and ranges and bars howitzer fire, §8.1).
171 │ pub const FALL_OF_KHARTOUM_TURN_TRACK: [TurnEntry; 8] = [
172 │     TurnEntry {
173 │         turn: 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fall_of_khartoum_turn_one_is_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fall_of_khartoum_turns_3_to_8_are_day]]]
#v(0.3em)
#heading(level: 2, "§9.34 -- Special Rules (Bonus)") <sect-9-34>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Rules]]
#v(0.5em)
#heading(level: 2, "§9.35 -- Victory Conditions (Bonus)") <sect-9-35>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Victory Conditions

Victory is determined by how many turns it takes the Dervish player to eliminate the GORDON leader unit and how many Dervish units are eliminated:

- Dervish decisive: eliminate GORDON turn four or sooner.
- Dervish tactical: eliminate GORDON turn five.
- Dervish marginal: eliminate GORDON turn six.
- British marginal: GORDON survives end of turn six.
- British tactical: GORDON survives end of turn seven.
- British decisive: GORDON survives end of turn eight.

The Dervish player then loses one victory level if he has lost 16–23 units, two victory levels if he has lost 24–31 units, and three victory levels if he has lost 32 units or more. Thus, for example, a Dervish tactical victory becomes a British marginal victory if the Dervish player eliminates GORDON on turn five, but loses 24 Dervish units doing it!]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 1233) \ #github-link("omdurman-rules/src/lib.rs", 1233)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1233")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FoKVictoryLevel]]]], [#raw("1231 │ /// negative) so the loss penalty is a simple shift toward the British end.
1232 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
1233 │ pub enum FoKVictoryLevel {
1234 │     DervishDecisive = -3,
1235 │     DervishTactical = -2,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fok_victory_level_worked_example]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fok_victory_level_gordon_died_early]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fok_victory_level_late_gordon_death]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fok_victory_level_gordon_survived]]]
#v(0.3em)
#heading(level: 2, "§9.111 -- Dervish set up (Campaign)") <sect-9-111>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Dervish player sets up first, moves second.

- Isa Zachneih infantry unit: anywhere on the east bank, in or south of El Debeba.
- KHALIFA ABDULLAH: in the walled city of Omdurman, in either palace hex.
- 3 artillery units, and all Taiasha units: anywhere in the walled city of Omdurman.
- 17 forts: anywhere on the mapsheet south of the Khor Shambat on the west bank, and/or south of all Halfaya hut hexes on the east bank and Nile River islands.
- 2 gunboats: any south edge Nile River hexes.

\*\*#link(<sect-9-112>)[9.112]) Dervish reinforcements:\*\* all reinforcements enter on the west edge of the mapsheet, south of the Khor Shambat. Each unit pays the terrain cost of the hex through which it enters, no matter how many units enter through that hex.

- Turn 1) all Baggara, Jaalin, Danagla, Kehena, and Degheim units, and their leaders: YAKUB, SHERIF, and ALI WAD HELU.
- Turn 2) all Hadendowa units and their leader, OSMAN DIGNA.
- Turn 3) all Mulazmin and Jehadia units and their leader, SHEIK EL DIN.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-9-112>)[§9.112]]
#v(0.3em)
#heading(level: 2, "§9.112 -- Dervish reinforcements (Campaign)") <sect-9-112>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 122) \ #github-link("omdurman-rules/src/effects.rs", 122)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L122")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[PlaceReinforcements]]]], [#raw("120 │     // -- Reinforcement / placement -----------------------------------------
121 │     /// Place reinforcements onto the map (rulebook §9.112, §9.113).
122 │     PlaceReinforcements(Vec<UnitPlacement>),
123 │ 
124 │     // -- Scenario-specific -------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 3022) \ #github-link("omdurman-rules/src/effects.rs", 3022)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3022")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_place_reinforcements]]]], [#raw("3020 │ 
3021 │ /// Place reinforcements onto the map (rulebook §9.112, §9.113).
3022 │ pub fn apply_place_reinforcements(
3023 │     state: &mut GameState,
3024 │     placements: &[UnitPlacement],", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 509) \ #github-link("omdurman-types/src/lib.rs", 509)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L509")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Location]]]], [#raw("507 │ /// Named map landmarks (rulebook mapsheet, §9.111, §9.113, §9.212 scenarios).
508 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
509 │ pub enum Location {
510 │     FortMakran,
511 │     NorthFort,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 561) \ #github-link("omdurman-types/src/lib.rs", 561)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L561")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[SetupLetter]]]], [#raw("559 │ /// Each letter marks a specific hex where a Dervish leader is placed.
560 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
561 │ pub enum SetupLetter {
562 │     Y,
563 │     K,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 640) \ #github-link("omdurman-types/src/lib.rs", 640)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L640")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Faction]]]], [#raw("638 │ /// Dervish units have a tribe; Anglo-Egyptian infantry have a brigade.
639 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
640 │ pub enum Faction {
641 │     Dervish { tribe: DervishTribe },
642 │     BritishEgyptian { brigade: Brigade },", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.113 -- Anglo-Egyptian set up (Campaign)") <sect-9-113>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Anglo-Egyptian player moves first. There are no Anglo-Egyptian units on the mapsheet at start. The GORDON unit is not used in this scenario.

- The leader units KITCHENER, GATACRE, and HUNTER may enter anytime during the first four game turns and do not count against the 12 unit per turn limit. All three leaders must be in play by the end of turn four!
- All gunboats enter through any north edge Nile River hex, paying one movement point for the first hex entered. The "Friendlies" brigade enters through the Abu Alim hut hex on the east bank, paying eight movement points per unit. All other Anglo-Egyptian units enter through the west bank "ANGLO-EGYPTIAN ENTRANCE AREA", each unit paying one movement point to enter the mapsheet.

- Turn 1) Any three gunboats; "Friendlies" brigade; Egyptian Cavalry; Horse Artillery; and two infantry brigades from the Egyptian Division.
- Turn 2) Any three gunboats plus any twelve land units.
- Turn 3) Any three gunboats plus any twelve land units.
- Turn 4) All remaining Anglo-Egyptian units.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#heading(level: 2, "§9.211 -- Anglo-Egyptian set up first, moves second (Historical)") <sect-9-211>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Anglo-Egyptian player sets up first, and moves second:

- Not in play: GORDON leader unit, "Friendlies" brigade.
- Gunboats start in any Nile River hexes adjacent to the Zariba.
- Camel Corps, Egyptian Cavalry, and Horse Artillery start in the village of Kerreri hut hexes.
- All remaining Anglo-Egyptian units set up in the 13 hexes of the Zariba.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1782) \ #github-link("omdurman-rules/src/effects.rs", 1782)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1782")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[first_player]]]], [#raw("1780 │ 
1781 │ /// The player who moves first in a scenario (§4, §9.113, §9.212, §9.322).
1782 │ pub fn first_player(scenario: Scenario) -> Player {
1783 │     match scenario {
1784 │         Scenario::Campaign => Player::AngloEgyptian,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.212 -- Dervish set up (Historical) -- deployment zones") <sect-9-212>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Dervish player sets up second, and moves first.

- Not in play: Isa Zachneih, gunboats, and forts.
- All Dervish units must be set up out of the line of sight of all Anglo-Egyptian units.
- Dervish leaders start on the lettered hexes:
  - A: Ali Wad Helu
  - D: Sheik El Din
  - Y: Yakub
  - K: Khalifa Abdullah
  - S: Sherif
  - O: Osman Digna
- All remaining Dervish units set up within three hexes of their leader as identified by color (e.g. all green units set up within three hexes of Sheik El Din).]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 724) \ #github-link("omdurman-rules/src/effects.rs", 724)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L724")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[in_deployment_zone]]]], [#raw("722 │     ///   plan / UI rather than this hex predicate. Documented, not silently
723 │     ///   dropped.
724 │     pub fn in_deployment_zone(&self, player: Player, hex: HexCoord) -> bool {
725 │         // No board attached -> permissive (unit tests, sandbox).
726 │         if self.board.terrain.is_empty() {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[deploy_rejected_outside_zone]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[embedded_leaders_resolve_from_their_host_section]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[historical_places_all_six_leaders_when_anchors_present]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[missing_anchor_is_reported_not_dropped_silently]]]
#v(0.3em)
#heading(level: 2, "§9.231 -- Thorn hedge hexsides") <sect-9-231>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 817) \ #github-link("omdurman-rules/src/lib.rs", 817)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L817")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("815 │     Terrain(i16),
816 │     /// -2 thorn-hedge defensive modifier (§9.231).
817 │     ZaribaThornHedge,
818 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
819 │     /// units (those Nile-side of the trench hexside).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 211) \ #github-link("omdurman-types/src/lib.rs", 211)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L211")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("209 │     Crest,
210 │     /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
211 │     ZaribaThornHedge,
212 │     /// Historical-scenario trench segment of the Zariba (§9.232).
213 │     ZaribaTrench,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.232 -- Trench hexsides") <sect-9-232>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 820) \ #github-link("omdurman-rules/src/lib.rs", 820)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L820")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrenchEntrenched]]]], [#raw("818 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
819 │     /// units (those Nile-side of the trench hexside).
820 │     ZaribaTrenchEntrenched,
821 │ }
822 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 213) \ #github-link("omdurman-types/src/lib.rs", 213)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L213")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrench]]]], [#raw("211 │     ZaribaThornHedge,
212 │     /// Historical-scenario trench segment of the Zariba (§9.232).
213 │     ZaribaTrench,
214 │     /// One of the two end hexsides of a Zariba trench segment that connect to
215 │     /// the Nile River (§9.233).  Units may only enter/leave the Zariba via", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.233 -- Zariba entry/exit costs") <sect-9-233>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Units may only enter and/or leave the Zariba via the two end hexsides that connect to the Nile River, paying +2 movement points to cross (Exception: advance after combat across an entrenched hexside).]]
#v(0.5em)
#heading(level: 2, "§9.321 -- British set up (Bonus)") <sect-9-321>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The British player sets up first, moves second:

- General GORDON leader unit in the palace.
- Two old style (unnamed) gunboats in any Nile River hexes.
- Set up in any building or hut hexes of Khartoum, Forts Makran and/or Buri, and/or adjacent to any wall hex:
  - one Egyptian Battalion artillery unit
  - two British infantry units (represents Caucasian troops)
  - three Egyptian infantry units (represents Cairo "Bazouks")
  - four Sudan infantry units (represents Sudanese blacks)
  - four "Friendlies" units (represents the Shaggyeh)]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[confirm_ready_rejected_below_scenario_target]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fall_of_khartoum_places_gordon_in_the_palace]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fall_of_khartoum_reports_missing_palace]]]
#v(0.3em)
#heading(level: 2, "§9.322 -- Dervish enters turn one (Bonus)") <sect-9-322>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Dervish player moves first: enters turn one through any hexes on the south or east edge of the map.

- 32 Mulazmin units (represents combined forces of Wad El Nejumi, Abu Girgeh, and Sheik El Obeid)
- 2 Hadendowa; 6 Kehena; 5 Degheim (represents Mahdi's combined west bank forces)
- 3 Dervish artillery units.]]
#v(0.5em)
#heading(level: 2, "§9.341 -- Turn 1 is always a night turn (Bonus)") <sect-9-341>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Turn 1 is always a night turn (see #link(<sect-8-1>)[8.1]).]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-8-1>)[§8.1]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 171) \ #github-link("omdurman-rules/src/turn_track.rs", 171)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L171")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_TURN_TRACK]]]], [#raw("169 │ /// (the rulebook fixes none); only `day_night` is rule-bearing (night halves
170 │ /// Anglo-Egyptian movement and ranges and bars howitzer fire, §8.1).
171 │ pub const FALL_OF_KHARTOUM_TURN_TRACK: [TurnEntry; 8] = [
172 │     TurnEntry {
173 │         turn: 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fall_of_khartoum_turn_one_is_night]]]
#v(0.3em)
#heading(level: 2, "§9.343 -- Both players use the Dervish Range Effects Table (Bonus)") <sect-9-343>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Both players must use the Dervish Range Effects Table.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 2127) \ #github-link("omdurman-rules/src/effects.rs", 2127)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2127")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[range_band_for]]]], [#raw("2125 │ /// in FALL OF KHARTOUM *both* players use the Dervish Range Effects Table
2126 │ /// (§9.343).
2127 │ pub fn range_band_for(
2128 │     scenario: Scenario,
2129 │     player: Player,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.344 -- Dervish controls the North Fort (Bonus)") <sect-9-344>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Dervish player controls the "North Fort" and may fire its guns.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1309) \ #github-link("omdurman-rules/src/effects.rs", 1309)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1309")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_has_enemy_fort]]]], [#raw("1307 │     /// may neither occupy an enemy fort nor advance after combat into one
1308 │     /// (forts are never captured -- only destroyed, §6.62/§6.53/§7.6).
1309 │     pub fn hex_has_enemy_fort(&self, hex: HexCoord, mover: Player) -> bool {
1310 │         self.units.iter().any(|u| {
1311 │             u.position == hex", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.345 -- Gunboat White Nile <-> Blue Nile crossing (Bonus)") <sect-9-345>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The British gunboats may move from the White Nile to the Blue Nile and vice-versa at an off-board movement cost of six "upstream" movement points.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1293) \ #github-link("omdurman-rules/src/effects.rs", 1293)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1293")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_nile_mouth_crossing]]]], [#raw("1291 │     /// must be named on the board, else this is `false` and the move falls
1292 │     /// through to the ordinary contiguous-Nile rules.
1293 │     pub fn is_nile_mouth_crossing(&self, from: HexCoord, to: HexCoord) -> bool {
1294 │         let white = self
1295 │             .board", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 525) \ #github-link("omdurman-types/src/lib.rs", 525)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L525")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Location::WhiteNileMouth]]]], [#raw("523 │     /// The off-board mouth of the White Nile branch (FALL OF KHARTOUM §9.345) --
524 │     /// a British gunboat may cross to the Blue Nile mouth for 6 upstream MP.
525 │     WhiteNileMouth,
526 │     /// The off-board mouth of the Blue Nile branch (FALL OF KHARTOUM §9.345).
527 │     BlueNileMouth,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.346 -- GORDON immobile, eliminated only at the Palace (Bonus)") <sect-9-346>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The GORDON leader unit starts in the palace and may not move during the scenario. He may only be eliminated by a Dervish unit passing through or occupying the palace hex (as normal movement or as advance after combat).]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1935) \ #github-link("omdurman-rules/src/effects.rs", 1935)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1935")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[check_gordon_palace]]]], [#raw("1933 │ /// after combat). Records the turn (which fixes the §9.35 victory level) and
1934 │ /// ends the game. A no-op outside FoK, or once GORDON is already gone.
1935 │ pub fn check_gordon_palace(state: &mut GameState) {
1936 │     if state.scenario != Scenario::FallOfKhartoum || state.gordon_eliminated_turn.is_some() {
1937 │         return;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 595) \ #github-link("omdurman-rules/src/lib.rs", 595)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L595")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitIdentity::is_gordon]]]], [#raw("593 │     /// Whether this is the GORDON leader unit (§9.32, §9.346) -- the immobile
594 │     /// palace defender whose elimination ends FALL OF KHARTOUM (§9.35).
595 │     pub fn is_gordon(&self) -> bool {
596 │         matches!(
597 │             self,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[gordon_is_an_immobile_british_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[gordon_survives_means_no_elimination]]]
#v(0.3em)
#progress-bar(7, 10)
#heading(level: 1, "§10 -- Optional Rules") <sect-10>
#heading(level: 2, "§10 -- Optional Rules")
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Optional Rules (Campaign game only)

It is suggested that the most intriguing employment of the following two options is to permit the Dervish player to have either one or the other, but the Anglo-Egyptian player doesn't know which one until he stumbles onto it. Players are advised that the employment of both optionals in the same game is not recommended.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 313) \ #github-link("omdurman-rules/src/lib.rs", 313)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L313")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OptionalRule]]]], [#raw("311 │ /// two should be in play (rulebook §10).
312 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
313 │ pub enum OptionalRule {
314 │     RiverMines,
315 │     RiverChain,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§10.1 -- River Mines") <sect-10-1>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[River Mines

The Khalifa twice tried (unsuccessfully) to submerge a powerful mine in the Nile to sink or damage British gunboats. This option assumes that both attempts were successful.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 140) \ #github-link("omdurman-rules/src/effects.rs", 140)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L140")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RiverMine]]]], [#raw("138 │     // -- Optional rules ----------------------------------------------------
139 │     /// River mine resolution (rulebook §10.12).
140 │     RiverMine {
141 │         gunboat_id: UnitId,
142 │         hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 3286) \ #github-link("omdurman-rules/src/effects.rs", 3286)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3286")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("3284 │ 
3285 │ /// Apply a river-mine resolution (rulebook §10.12).
3286 │ pub fn apply_river_mine(
3287 │     state: &mut GameState,
3288 │     gunboat_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 986) \ #github-link("omdurman-rules/src/lib.rs", 986)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L986")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MineResult]]]], [#raw("984 │ /// British gunboat enters a mined hex.
985 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
986 │ pub enum MineResult {
987 │     /// Roll 1-4: no effect.
988 │     NoEffect,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§10.2 -- River Chain") <sect-10-2>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[River Chain

The Khalifa also tried (also unsuccessfully) to string a heavy chain across the Nile to stop or slow down the British gunboats. This option assumes the chain was emplaced.]]
#v(0.5em)
#heading(level: 2, "§10.11 -- Secretly record mine hexes") <sect-10-11>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Prior to the commencement of play the Dervish player secretly records two Nile River hexes to be mined (the mines may not both be placed in the same hex). These hexes must be south of the E–W hexrow in which the Khor Shambat empties into the Nile.]]
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[mine_and_chain_limits_enforced_in_setup]]]
#v(0.3em)
#heading(level: 2, "§10.12 -- Mine resolution") <sect-10-12>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[When a British gunboat enters a mined hex, the Dervish player must order it to stop as it has struck a mine. The Dervish player then resolves the effect of the mine's blast by rolling the ten-sided die:

- 1–4: No effect
- 5–7: Gunboat damaged, lost use of its engines and must drift two hexes per turn (with the current) for the rest of the game. No effect on guns or Maxims unless they drift out of range.
- 8–10: Gunboat sunk!]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 140) \ #github-link("omdurman-rules/src/effects.rs", 140)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L140")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RiverMine]]]], [#raw("138 │     // -- Optional rules ----------------------------------------------------
139 │     /// River mine resolution (rulebook §10.12).
140 │     RiverMine {
141 │         gunboat_id: UnitId,
142 │         hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 3286) \ #github-link("omdurman-rules/src/effects.rs", 3286)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3286")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("3284 │ 
3285 │ /// Apply a river-mine resolution (rulebook §10.12).
3286 │ pub fn apply_river_mine(
3287 │     state: &mut GameState,
3288 │     gunboat_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§10.13 -- Mines consumed after both rolled for") <sect-10-13>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[After both mines have been rolled for, no more are available.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 3286) \ #github-link("omdurman-rules/src/effects.rs", 3286)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3286")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("3284 │ 
3285 │ /// Apply a river-mine resolution (rulebook §10.12).
3286 │ pub fn apply_river_mine(
3287 │     state: &mut GameState,
3288 │     gunboat_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§10.14 -- Dervish gunboats pass safely") <sect-10-14>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Dervish player's gunboats may pass through the mined hexes with no ill effect (he knows where the mines are).]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 3286) \ #github-link("omdurman-rules/src/effects.rs", 3286)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3286")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("3284 │ 
3285 │ /// Apply a river-mine resolution (rulebook §10.12).
3286 │ pub fn apply_river_mine(
3287 │     state: &mut GameState,
3288 │     gunboat_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§10.21 -- Secretly record chain hexes") <sect-10-21>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Prior to the commencement of play the Dervish player secretly records a line of river hexes (not exceeding four hexes long) across which the chain is strung. The hexes must be south of the E–W hexrow in which the Khor Shambat empties into the Nile.]]
#v(0.5em)
#heading(level: 2, "§10.22 -- Gunboat stops on chained hex") <sect-10-22>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[When a British gunboat enters a "chained" river hex it must stop and may move no further that turn.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 981) \ #github-link("omdurman-rules/src/effects.rs", 981)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L981")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_gunboat]]]], [#raw("979 │     /// upstream movement allowance is their maximum for that turn.\" Chained Nile
980 │     /// hexes stop the gunboat (§10.22).
981 │     pub fn can_move_gunboat(
982 │         &self,
983 │         unit_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§10.23 -- Sinking the chain") <sect-10-23>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[No gunboats (British or Dervish) may cross the chain until it has been sunk by the British player. He may sink the chain by a) having an infantry or cavalry unit spend one complete turn on either riverbank adjacent to a "chained" river hex, or b) firing at the chain with artillery and achieving a 3 or more on the Combat Results Table.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 3345) \ #github-link("omdurman-rules/src/effects.rs", 3345)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3345")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_sink_chain]]]], [#raw("3343 │ /// Sink the river chain (rulebook §10.23). Marks the placed chain cleared so it
3344 │ /// no longer stops gunboats (§10.22).
3345 │ pub fn apply_sink_chain(state: &mut GameState) -> Result<(), RuleError> {
3346 │     match state.chain.as_mut() {
3347 │         Some(chain) if !chain.sunk => {", block: true, lang: "rs")],
)
#v(0.5em)
#progress-bar(0, 1)
#heading(level: 1, "§11 -- Historical Notes") <sect-11>
#heading(level: 2, "§11 -- Historical Notes")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Historical Notes

In 1881 Mohammed Ahmed Ibn Al-Sayid Abdullah, the son of an obscure carpenter in the hinterlands of the Sudan, proclaimed himself the "Mahdi" — the Messiah of the Islamic faith. His timing was propitious indeed. Since the early 1820's a corrupt Egypt, with the Sultan of Turkey's blessing, had incessantly raped the Sudan, taking ivory and flooding the slave markets with some half million captured Sudanese blacks. By 1880, nearly 40,000 Egyptian troops occupied outposts scattered throughout the Sudan, enforcing Egypt's hold on this lucrative ivory and slave trade and squeezing the native population dry through vicious and corrupt tax officials. All was controlled from Khartoum via the office of Governor General of the Sudan. The title had been held by a succession of individuals, including General Charles Gordon, whose appointment was an attempt to reinstate some rudimentary justice in the Sudan after France and Britain assumed joint political control of a bankrupt Egypt.

By 1881, however, Gordon's term had expired and a new Governor General, again corrupt and incompetent, attempted to deal with the Mahdi. Declining to come to terms with the representatives of Egypt's "benevolent civilization", the Mahdi butchered an armed force dispatched to arrest him in October, 1881. Three months later, the Dervishes (members of a fundamentalist sect following the Mahdi) again ambushed and slaughtered a punitive force of 1400 Egyptian troops sent against him. The effect of these two actions on the native Sudanese was electrifying and they flocked by the thousands to join his holy war and cast out their oppressors.

Egypt, in the meantime, was attempting to throw off Turkish rule and Britain, fearing a revolution and loss of Christian lives, ordered the Mediterranean Squadron to Alexandria in May, 1882. When Turkey refused to intervene, British Marines and Bluejackets went ashore and restored order in Alexandria. Britain next sent General Sir Garnet Wolseley to deal with the rebellious Egyptian army who still controlled Cairo and most of the Egyptian countryside. By mid-September Wolseley had subdued Egypt, winning the battles of Mahsama and Tel-el-Kebir. Thus, by the end of 1882, Britain unwillingly assumed responsibility for Egypt, protecting her communication lines to India in the bargain.

The Sudan, however, was another matter. In England, prime minister William Gladstone was opposed to any activity which would take British troops outside Egypt's borders. But London was very far away and the simple fact of the matter was that Egyptian security was dependent on a subjugated Sudan. Accordingly, the Egyptian army was reorganized along European lines under British officers and undertook its first major effort under General William Hicks, better known as Hicks Pasha, in February of 1883.

The Mahdi, in the meantime, was taking advantage of the situation in Egypt to expand his influence in the Sudan. Each success brought more recruits and the rebellion grew. He crushed an Egyptian force sent against him from Khartoum in March, 1882, and butchered another expedition in January, 1883.

Hicks Pasha marched his Egyptian army to Khartoum and, after a brief rest, moved out again on June 26th, 1883. After some four months of marching and several minor engagements, Hicks and his army met their end on November 4th at Kashgeil, about 225 miles southwest of Khartoum. The Mahdi's horde attacked on the 3rd and 4th and finally broke the square, the slaughter itself taking until the 5th to complete. Next into the fray was Valentine Baker Pasha, who led another Egyptian expedition in to the eastern Sudan via the Red Sea in early 1884. It was hacked to pieces early in February when one of the Mahdi's Emirs, Osman Digna, again broke the square with his Hadendowa troops, the notorious "Fuzzy-Wuzzies".

With Khartoum itself now menaced, London finally reacted and ordered General Sir Gerald Graham into the Sudan with a detachment from the British Army of Occupation in Egypt. On February 29th he engaged a portion of Osman Digna's forces at El-Teb, near Suakim in the eastern Sudan, and won by a narrow margin when his square formation held. Seeking to expand on this victory, General Graham ordered Osman Digna and his chiefs to disperse their forces and surrender themselves. When they refused, the British expedition again marched against the Dervishes on March 12th. This time, however, the "Fuzzy-Wuzzies" broke the square, a British square. Although the broken square rallied and the Dervishes were finally beaten off, it was another narrow victory. The Mahdi still ruled the vastness of the Sudan with the few remaining Anglo-Egyptian garrisons like tiny islands in a hostile ocean. Eyes on both sides now turned toward Khartoum.

However distasteful to his politics, prime minister Gladstone was now forced to take some action on behalf of the troops and civilians in the Sudan. Abhorring the cost of a major imperial expedition, the decision was made to evacuate and one man was sent to accomplish it, General Sir Charles Gordon. Upon arrival at Khartoum he again assumed the role of Governor General of the Sudan and announced to the startled population (who had expected an army) that he came without troops, but with God on his side. Supremely self-confident, he showed no intention of evacuating the city and instead set about reinforcing the defenses and recruiting native volunteers. Unimpressed with Gordon's offers of reconciliation, the Mahdi responded by investing Khartoum on March 12th, 1884. The siege was, however, only effective on land, as Gordon's little gunboats continued to steam up and down the Nile transferring women, children and wounded to Berber, north of the sixth cataract. In Khartoum itself, Gordon took personal charge of everything, imposing a rationing system, printing his own paper money and awarding his own medals.

When Berber fell to the Mahdi's troops in May of 1884, Khartoum's isolation was virtually complete, and yet it continued to hold out. By August the public outcry in England and the British press compelled Gladstone to take further action for the relief of General Gordon and the Sudan. The action took the form of an expeditionary force under Sir Garnet Wolseley, who arrived in Egypt September 9th and had the relief force under way by October 5th.

Progress was unfortunately slow. So slow that by December Wolseley had only progressed some 150 miles to the third cataract. Beyond lay the Mahdi's Dervish-infested territory and three more cataracts before the column would be anywhere near Khartoum, whose time was running out. A desert strike force of 1800 men was thus detached to move overland and set out early in January. It was attacked on the 17th near Abu Klea and disaster was narrowly averted when the Dervishes again broke a British square but were unable to exploit because the baggage animals were packed tightly in the center. On the 19th, the Dervishes struck again at Abu Kru but were repulsed, and the strike force proceeded without further incident to the Nile.

Due to casualties, command of the strike force had passed to a Colonel Wilson, a staff officer with little combat experience. Accordingly, when four of Gordon's steamers reached him on January 21st, he declined to embark his troops, instead feeling they needed a three day rest to recover and build a defensive position.

In Khartoum, meanwhile, the garrison became daily more weakened by hunger and fatigue. If Gordon's disinclination to evacuate seems strange, then even stranger was the Mahdi's apparent reluctance to apply the coup de grace to the city. Even after the inevitable end became painfully obvious, he continued to offer Gordon honorable surrender terms, safe passage, and other concessions. Gordon, however, remained adamant. He had apparently prepared himself a martyr's place in history and would not be dissuaded from it except by the total capitulation of the Mahdi and his followers. Then the Mahdi was informed that the relief expedition was within a few days of Khartoum and decided the garrison must be taken at once. Thus it was that in the pre-dawn hours of January 25th, 1885, some 20,000 Dervishes poured through a gap in Khartoum's outer defenses where the receding White Nile had eroded away a section of wall. The garrison was slaughtered, Gordon among them (FALL OF KHARTOUM scenario — #link(<sect-9-3>)[9.3]). Three days later (Col. Wilson's three days of rest?) the steamers carrying the advance guard of the strike force came within sight of Khartoum. Seeing only smoking ruins, they turned around and steamed back downstream to bring the news to the main body. Queen Victoria voiced the feelings of the nation when she recorded in her diary: "The government alone is to blame".

The relief column withdrew back into Egypt, and the fall of Khartoum thus effectively eliminated Britain's presence in the Sudan for the next eleven years, leaving that vast hinterland to the Mahdist empire. The Mahdi died in June of 1885 and was succeeded by the Khalifa, Abdullah the Taiasha, a chief of the Baggaras. The Khalifa made Omdurman his capital and expanded it from a few mud huts in 1885 to a vast, sprawling fifteen square mile urban slum by 1898. It housed the Dervishes' holiest shrine, the Mahdi's Tomb, as well as the palace and other structures in a walled city within a city.

By 1896 the spread of Mahdism led to British concern for the security of Egypt. In a move ostensibly made to take pressure off an Italian outpost on the Abyssinian border, London ordered an expedition into Dervish territory in the northern Sudan. It was led by General Herbert Kitchener, Sirdar (commander) of the Egyptian army. Kitchener had been a major in the Khartoum relief expedition and had never forgotten the rage and shame he felt when that force withdrew without attacking the Mahdi's army. An obsession to avenge Gordon's death stayed with him over the intervening years, so that he welcomed the instructions to move on the Sudan. To free himself from total dependence on the Nile for transportation, the Sudan Military Railroad was planned and overland construction begun. By July of 1896, Kitchener was underway. Progress was slow but steady, with the army halting periodically for the railway to catch up. Following infrequent skirmishing with the Dervishes, Kitchener's Egyptian Division under General Hunter re-occupied Berber in July of 1897. The balance of that year was spent reorganizing and re-supplying the army while again waiting for the railway to catch up.

If 1897 was the year of consolidation and organization, 1898 was the year in which those efforts bore fruit. Reinforced with a British brigade, the Sirdar's army was again on the move in March, 1898. After fighting three minor engagements during March and early April, the army (now the Anglo-Egyptian army) found itself confronted by a large Dervish force under Mahmud, one of the Khalifa's few remaining competent generals. Mahmud had entrenched his force inside a circular defensive zariba of camel thorn, with his back on the dry bed of the river Atbara, a strong defensive position. Mahmud, however, had not taken the new British heavy artillery into account and, after an hour and a half of heavy bombardment, the Sirdar's army went in, led by the Cameron Highlanders. Forty-five minutes later 3,000 Dervishes were dead at a loss to Kitchener of 80 men killed, and Mahmud was a prisoner. The way to Omdurman was open!

By mid-April the railroad had reached the Nile below Berber, bringing with it the new shallow draft gunboats designed specifically for river campaigns. The sections of these new iron monsters were assembled and floated in the Nile. One hundred and forty feet long by twenty-four feet wide and drawing only thirty-nine inches of water, they were formidable concentrations of firepower with their 12 pounders, 6 pounders, and Maxim guns on the upper deck, and 4 inch howitzers on the gun deck. By August 17th all was in readiness and, reinforced with a second British brigade, Kitchener marched steadily south, arriving at the little mud village of Kerreri on September 1st (CAMPAIGN GAME scenario — #link(<sect-9-1>)[9.1]).

The Khalifa, Abdullah the Taiasha, in the meantime, had not been idle. Throughout the Spring and Summer of 1898, the Sudan experienced a hectic and frantic mobilization as the leading Emirs of the empire gathered the faithful to the Jihad, or holy war. Estimates of the response vary widely, but it seems likely that some 60–70,000 warriors answered the call and assembled on the plains of Kerreri, north of Omdurman. To guard the approaches to the city, seventeen forts were constructed and armed with old artillery pieces. The few guns available, old Remingtons and brass muzzle loaders using home made cartridges, were issued to the Jehadia (commanded by the Khalifa's son, Sheik El Din) and the Danagla. The rest of the troops carried swords and spears.

Dawn of September 2nd saw the Sirdar and his Anglo-Egyptian army positioned inside a rough semi-circular formation protected by a zariba of thorn hedge and trenches. His back and flanks were on the Nile and guarded by the gunboats. At dawn the cavalry had gone out, but by 6:30 they were back in. Then they came — the Dervishes in their thousands and tens of thousands pouring over the ridges of the Jebel Surgham and the Kerreri Hills (HISTORICAL scenario — #link(<sect-9-2>)[9.2]).

By 11:30 the battle was virtually over. 10,000 Dervishes dead — 20,000 wounded, over ½ of whom would die unattended in the blazing sun during the next two days — 5,000 prisoners — all at a cost of just over 400 killed and wounded in the Sirdar's army. The rest of the story is known to the most casual student of the battle: the 21st Lancers win their first battle streamer and three Victoria Crosses in one of history's last great knee to knee cavalry charges — Maxwell and the XIII Sudanese first to enter the city — 30,000 captured cooks and concubines for whom Kitchener declared he had no use in either capacity — the unused Gatling guns and Nordenfeldts found in the Khalifa's arsenal — the repulsive battlefield with its several hundred acres of suffering wounded and bloating corpses piled around the flags of their dead Emirs — 30,000 Dervish survivors of the battle melted away in the desert, never to rise again. Rarely in modern history has an army and a civilization been so thoroughly crushed, consuming the efforts of half a generation. Fifty-eight years later, Britain would withdraw permanently from Egypt and the Anglo-Egyptian Sudan.

Two days after the battle, September 4th, 1898, Kitchener held a memorial service for General Sir Charles Gordon in front of the ruins of the Governor General's palace in Khartoum. He described it in moving phrases in a letter to Queen Victoria, who recorded in her diary: "Surely now he is avenged".


\#\# Credits

- \*\*Game Design:\*\* Peter Bertram
- \*\*Development:\*\* Peter Bertram and Fred Chatham
- \*\*Graphic Arts:\*\* Mike Williford, Graphics Unlimited
- \*\*Rules Editing:\*\* Randall Mac Innis
- \*\*Components Design:\*\* Peter Bertram
- \*\*Box Art:\*\* George I. Parrish Jr.
- \*\*Production Coordinator:\*\* Fred Chatham
- \*\*Printed By:\*\* Seiz Printing Inc.
- \*\*Playtesters:\*\* Martin Davisson, Dave Ferguson, Ron Glass, Randall Mac Innis, Henry Robinette, Michael Sincavage

Questions concerning the rules will be answered if they are a) phrased to be answered "yes" or "no", and b) accompanied by a stamped, self-addressed envelope. General comments about the game are always welcome.

Address all correspondence to:
Phoenix Enterprises, Ltd.
P.O. Box 81192
Chamblee, Ga. 30366

Copyright 1982 © Phoenix Enterprises, Ltd.


\#\# Reference: Charts and Tables

\*The Combat Results Table, Range Effects Tables, Line of Sight Table, Howitzer Fire Scattergram, Terrain Effects Chart, Turn Record Track, and Campaign Game Order of Appearance card are printed on the mapsheet and/or the back of the rulebook in the physical game. They are present in the source PDF as scanned tabular images and are not transcribed here — refer to the original mapsheet and rulebook back cover for table values.\*

\*\*Key constants referenced in the rules above (for quick lookup):\*\*

- Combat Results notation: `D` = ½ (round up) of units in the target hex disrupted; `1`/`2`/`3`/`4`/`5` = that many units in the target hex eliminated; `—` = no effect.
- Disrupted units: no ZOC; may not move; may not fire offensively or defensively; may not melee; are turned face up at the end of the owning player's turn.
- Melee modifiers: Dervish +2; Anglo-Egyptian +1.
- Direct fire modifiers (Anglo-Egyptian): +1 all direct fire attacks; +1 brigade integrity (cumulative).
- Modified die rolls of less than 1 are treated as 1, more than 10 treated as 10.
- Artillery requirements: 3+ to sink a gunboat; 2+ to destroy a fort; 2+ to breach a wall hexside.
- Howitzer fire: range 4–10 hexes; target hex hit on impact roll 7–10; otherwise scatters per Howitzer Fire Scattergram.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-9-1>)[§9.1], #link(<sect-9-2>)[§9.2], #link(<sect-9-3>)[§9.3]]
#v(0.3em)
#progress-bar(0, 1)
#heading(level: 1, "Credits") <sect-Credits>
#heading(level: 2, "§Credits -- Credits")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#progress-bar(1, 1)
#heading(level: 1, "Combat Results Table (shared reference)") <sect-CRT>
#heading(level: 2, "§CRT -- Combat Results Table (shared by §6.22 fire and §7.7 melee)")
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 8) \ #github-link("omdurman-rules/src/combat_results_table.rs", 8)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat_results_table.rs#L8")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireFactorRow]]]], [#raw("  6 │ /// to index into the result matrix.
  7 │ #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
  8 │ pub enum FireFactorRow {
  9 │     /// 1-5 factors
 10 │     Row01to05,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 31) \ #github-link("omdurman-rules/src/combat_results_table.rs", 31)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat_results_table.rs#L31")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[from_total]]]], [#raw(" 29 │ impl FireFactorRow {
 30 │     /// Determine which row a given total fire factor falls into (rulebook §6.22).
 31 │     pub fn from_total(total: u16) -> Self {
 32 │         match total {
 33 │             0..=5 => FireFactorRow::Row01to05,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 69) \ #github-link("omdurman-rules/src/combat_results_table.rs", 69)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat_results_table.rs#L69")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[combat_results_table]]]], [#raw(" 67 │ /// D = `Disrupt` (1/2 of target units, round up)
 68 │ /// 1...5 = `Eliminate(n)` (that many units removed)
 69 │ pub fn combat_results_table(row: FireFactorRow, roll: DieRoll) -> CombatResult {
 70 │     use CombatResult::*;
 71 │     use DieRoll::*;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 881) \ #github-link("omdurman-rules/src/lib.rs", 881)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L881")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CombatResult]]]], [#raw("879 │ /// * `--` -- no effect
880 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
881 │ pub enum CombatResult {
882 │     NoEffect,
883 │     Disrupt,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_combat_results_table_lowest_is_no_effect]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_combat_results_table_highest_is_eliminate_5]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_combat_results_table_progresses_with_roll]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_combat_results_table_progresses_with_factor]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_factor_row_boundaries]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_factor_row_remaining_boundaries]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_factor_row_index_sequential]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[crt_all_rows_monotone_non_decreasing]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[crt_eliminate_never_exceeds_5]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[crt_every_cell_matches_the_table]]]
#v(0.3em)
#progress-bar(0, 1)
#heading(level: 1, "Reference -- Charts and Tables") <sect-Reference>
#heading(level: 2, "§Reference -- Charts and Tables")
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#heading(level: 1, "Appendix: Symbol Index") <sect-symbol-index>
#v(0.5em)
#table(
  columns: (2fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*Symbol*], [*Sections*],
  [#text(weight: "bold", size: 9pt)[AdvanceAfterCombat]], [#link(<sect-6-82>)[§6.82], #link(<sect-7-6>)[§7.6]],
  [#text(weight: "bold", size: 9pt)[AdvancePhase]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[AngloEgyptianDirectFire]], [#link(<sect-6-24>)[§6.24]],
  [#text(weight: "bold", size: 9pt)[AngloEgyptianStandard]], [#link(<sect-7-7>)[§7.7]],
  [#text(weight: "bold", size: 9pt)[BattalionOrdinal]], [#link(<sect-5-54>)[§5.54]],
  [#text(weight: "bold", size: 9pt)[Breach]], [#link(<sect-6-63>)[§6.63]],
  [#text(weight: "bold", size: 9pt)[Brigade]], [#link(<sect-5-54>)[§5.54]],
  [#text(weight: "bold", size: 9pt)[BrigadeId]], [#link(<sect-2-3>)[§2.3]],
  [#text(weight: "bold", size: 9pt)[BrigadeIntegrity]], [#link(<sect-5-54>)[§5.54], #link(<sect-6-24>)[§6.24]],
  [#text(weight: "bold", size: 9pt)[BritishLeader]], [#link(<sect-6-51>)[§6.51]],
  [#text(weight: "bold", size: 9pt)[BritishLeaderUnit]], [#link(<sect-6-51>)[§6.51]],
  [#text(weight: "bold", size: 9pt)[CAMPAIGN_TURN_TRACK]], [#link(<sect-9-12>)[§9.12]],
  [#text(weight: "bold", size: 9pt)[CampaignVictoryLevel]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[CombatResult]], [#link(<sect-CRT>)[§CRT]],
  [#text(weight: "bold", size: 9pt)[ConstructZariba]], [#link(<sect-5-3>)[§5.3]],
  [#text(weight: "bold", size: 9pt)[DayNight]], [#link(<sect-8-1>)[§8.1]],
  [#text(weight: "bold", size: 9pt)[Demolition]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[DemolitionTarget]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[DervishDesertion]], [#link(<sect-8-2>)[§8.2]],
  [#text(weight: "bold", size: 9pt)[DervishGunboat]], [#link(<sect-2-32>)[§2.32]],
  [#text(weight: "bold", size: 9pt)[DervishLeaderCommandMismatch]], [#link(<sect-5-53>)[§5.53]],
  [#text(weight: "bold", size: 9pt)[DervishStandard]], [#link(<sect-7-7>)[§7.7]],
  [#text(weight: "bold", size: 9pt)[DervishTribe]], [#link(<sect-7-4>)[§7.4]],
  [#text(weight: "bold", size: 9pt)[DervishTribeMix]], [#link(<sect-5-52>)[§5.52]],
  [#text(weight: "bold", size: 9pt)[DervishVsTrenchedDefender]], [#link(<sect-7-7>)[§7.7]],
  [#text(weight: "bold", size: 9pt)[DieModifier]], [#link(<sect-7-7>)[§7.7]],
  [#text(weight: "bold", size: 9pt)[DirectFire]], [#link(<sect-6-41>)[§6.41]],
  [#text(weight: "bold", size: 9pt)[FALL_OF_KHARTOUM_TURN_TRACK]], [#link(<sect-9-33>)[§9.33], #link(<sect-9-341>)[§9.341]],
  [#text(weight: "bold", size: 9pt)[Faction]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[FireAttack]], [#link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[FireFactor]], [#link(<sect-6-11>)[§6.11]],
  [#text(weight: "bold", size: 9pt)[FireFactorRow]], [#link(<sect-CRT>)[§CRT]],
  [#text(weight: "bold", size: 9pt)[FoKVictoryLevel]], [#link(<sect-9-35>)[§9.35]],
  [#text(weight: "bold", size: 9pt)[Fort]], [#link(<sect-5-25>)[§5.25], #link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[Friendlies]], [#link(<sect-6-52>)[§6.52]],
  [#text(weight: "bold", size: 9pt)[FriendliesTransport]], [#link(<sect-5-21>)[§5.21]],
  [#text(weight: "bold", size: 9pt)[GameState]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[GameTime]], [#link(<sect-9-12>)[§9.12]],
  [#text(weight: "bold", size: 9pt)[GameTurnIndex]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[GunboatId]], [#link(<sect-2-32>)[§2.32], #link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[GunboatMovement]], [#link(<sect-5-24>)[§5.24]],
  [#text(weight: "bold", size: 9pt)[GunboatStack]], [#link(<sect-5-51>)[§5.51]],
  [#text(weight: "bold", size: 9pt)[HISTORICAL_TURN_TRACK]], [#link(<sect-9-22>)[§9.22]],
  [#text(weight: "bold", size: 9pt)[HexDirection]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[HexDistance]], [#link(<sect-6-22>)[§6.22], #link(<sect-7-5>)[§7.5]],
  [#text(weight: "bold", size: 9pt)[HexsideKind]], [#link(<sect-5-23>)[§5.23]],
  [#text(weight: "bold", size: 9pt)[HexsideRef]], [#link(<sect-5-23>)[§5.23]],
  [#text(weight: "bold", size: 9pt)[HistoricalVictoryLevel]], [#link(<sect-9-24>)[§9.24]],
  [#text(weight: "bold", size: 9pt)[Howitzer]], [#link(<sect-2-31>)[§2.31]],
  [#text(weight: "bold", size: 9pt)[HowitzerFire]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[HowitzerResolution]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[Immobile]], [#link(<sect-5-25>)[§5.25]],
  [#text(weight: "bold", size: 9pt)[Khor]], [#link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[Location]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[LosFirerTerrain]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[LosResult]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[LosSpecialNote]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[LosTargetTerrain]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[MaximSecondAndHowitzer]], [#link(<sect-6-42>)[§6.42]],
  [#text(weight: "bold", size: 9pt)[MeleeAttack]], [#link(<sect-7-7>)[§7.7]],
  [#text(weight: "bold", size: 9pt)[MeleeCombat]], [#link(<sect-7-3>)[§7.3]],
  [#text(weight: "bold", size: 9pt)[MeleeFactor]], [#link(<sect-7-1>)[§7.1]],
  [#text(weight: "bold", size: 9pt)[MeleeModifier]], [#link(<sect-7-7>)[§7.7]],
  [#text(weight: "bold", size: 9pt)[MineResult]], [#link(<sect-10-1>)[§10.1]],
  [#text(weight: "bold", size: 9pt)[MovementAllowance]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[MovementPoints]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[NamedGunboat]], [#link(<sect-2-32>)[§2.32]],
  [#text(weight: "bold", size: 9pt)[Old]], [#link(<sect-2-32>)[§2.32]],
  [#text(weight: "bold", size: 9pt)[OldGunboat]], [#link(<sect-2-32>)[§2.32]],
  [#text(weight: "bold", size: 9pt)[OptionalRule]], [#link(<sect-10>)[§10]],
  [#text(weight: "bold", size: 9pt)[OverLimit]], [#link(<sect-5-51>)[§5.51]],
  [#text(weight: "bold", size: 9pt)[PendingMelee]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[Phase]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[PlaceReinforcements]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[Range]], [#link(<sect-6-22>)[§6.22]],
  [#text(weight: "bold", size: 9pt)[RangeBand]], [#link(<sect-6-16>)[§6.16], #link(<sect-6-22>)[§6.22]],
  [#text(weight: "bold", size: 9pt)[RetreatBeforeMelee]], [#link(<sect-7-5>)[§7.5]],
  [#text(weight: "bold", size: 9pt)[RiverMine]], [#link(<sect-10-1>)[§10.1], #link(<sect-10-12>)[§10.12]],
  [#text(weight: "bold", size: 9pt)[RoyalEngineers]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[ScatterDirection]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[SetupLetter]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[SpriteAnnotation]], [#link(<sect-2-3>)[§2.3]],
  [#text(weight: "bold", size: 9pt)[Terrain]], [#link(<sect-6-23>)[§6.23]],
  [#text(weight: "bold", size: 9pt)[TurnEntry]], [#link(<sect-9-12>)[§9.12]],
  [#text(weight: "bold", size: 9pt)[TurnEvent]], [#link(<sect-8-2>)[§8.2]],
  [#text(weight: "bold", size: 9pt)[TurnLabel]], [#link(<sect-9-12>)[§9.12]],
  [#text(weight: "bold", size: 9pt)[UnitKind]], [#link(<sect-2-3>)[§2.3], #link(<sect-7-4>)[§7.4]],
  [#text(weight: "bold", size: 9pt)[UnitMovement]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[UnitProfile]], [#link(<sect-2-3>)[§2.3]],
  [#text(weight: "bold", size: 9pt)[UnitState]], [#link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[VictoryLedger]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[VictoryPoints]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[VpEvent]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[VpSource]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[Wall]], [#link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[WeaponClass]], [#link(<sect-2-31>)[§2.31], #link(<sect-6-6>)[§6.6], #link(<sect-6-61>)[§6.61], #link(<sect-6-62>)[§6.62]],
  [#text(weight: "bold", size: 9pt)[WhiteNileMouth]], [#link(<sect-9-345>)[§9.345]],
  [#text(weight: "bold", size: 9pt)[Zariba]], [#link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[ZaribaThornHedge]], [#link(<sect-9-23>)[§9.23], #link(<sect-9-231>)[§9.231]],
  [#text(weight: "bold", size: 9pt)[ZaribaTrench]], [#link(<sect-9-23>)[§9.23], #link(<sect-9-232>)[§9.232]],
  [#text(weight: "bold", size: 9pt)[ZaribaTrenchEntrenched]], [#link(<sect-9-23>)[§9.23], #link(<sect-9-232>)[§9.232]],
  [#text(weight: "bold", size: 9pt)[ZocReason]], [#link(<sect-5-41>)[§5.41], #link(<sect-5-44>)[§5.44], #link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[advance_phase]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[ae_range_effects]], [#link(<sect-6-22>)[§6.22]],
  [#text(weight: "bold", size: 9pt)[apply_advance_after_combat]], [#link(<sect-6-82>)[§6.82], #link(<sect-7-6>)[§7.6]],
  [#text(weight: "bold", size: 9pt)[apply_construct_zariba]], [#link(<sect-5-3>)[§5.3]],
  [#text(weight: "bold", size: 9pt)[apply_demolition]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[apply_friendlies_transport]], [#link(<sect-5-21>)[§5.21]],
  [#text(weight: "bold", size: 9pt)[apply_howitzer_fire]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[apply_melee_combat]], [#link(<sect-7-3>)[§7.3]],
  [#text(weight: "bold", size: 9pt)[apply_place_reinforcements]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[apply_retreat_before_melee]], [#link(<sect-7-5>)[§7.5]],
  [#text(weight: "bold", size: 9pt)[apply_river_mine]], [#link(<sect-10-1>)[§10.1], #link(<sect-10-12>)[§10.12], #link(<sect-10-13>)[§10.13], #link(<sect-10-14>)[§10.14]],
  [#text(weight: "bold", size: 9pt)[apply_sink_chain]], [#link(<sect-10-23>)[§10.23]],
  [#text(weight: "bold", size: 9pt)[blocks_advance_after_combat]], [#link(<sect-6-82>)[§6.82]],
  [#text(weight: "bold", size: 9pt)[blocks_los]], [#link(<sect-6-23>)[§6.23], #link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[blocks_melee]], [#link(<sect-7-2>)[§7.2]],
  [#text(weight: "bold", size: 9pt)[blocks_movement]], [#link(<sect-5-23>)[§5.23]],
  [#text(weight: "bold", size: 9pt)[brigade_integrity]], [#link(<sect-5-54>)[§5.54]],
  [#text(weight: "bold", size: 9pt)[can_advance_after_combat]], [#link(<sect-6-7>)[§6.7], #link(<sect-6-82>)[§6.82], #link(<sect-7-6>)[§7.6]],
  [#text(weight: "bold", size: 9pt)[can_fire_at]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[can_melee]], [#link(<sect-7-2>)[§7.2], #link(<sect-7-4>)[§7.4]],
  [#text(weight: "bold", size: 9pt)[can_move_gunboat]], [#link(<sect-10-22>)[§10.22]],
  [#text(weight: "bold", size: 9pt)[can_move_unit]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[can_move_unit_to]], [#link(<sect-5-22>)[§5.22], #link(<sect-5-26>)[§5.26], #link(<sect-5-43>)[§5.43]],
  [#text(weight: "bold", size: 9pt)[can_retreat_before_melee]], [#link(<sect-5-23>)[§5.23], #link(<sect-7-5>)[§7.5]],
  [#text(weight: "bold", size: 9pt)[check_gordon_palace]], [#link(<sect-9-346>)[§9.346]],
  [#text(weight: "bold", size: 9pt)[combat_results_table]], [#link(<sect-CRT>)[§CRT]],
  [#text(weight: "bold", size: 9pt)[constructing_zariba]], [#link(<sect-5-3>)[§5.3]],
  [#text(weight: "bold", size: 9pt)[defense_modifier]], [#link(<sect-6-23>)[§6.23]],
  [#text(weight: "bold", size: 9pt)[demolishing]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[dervish_range_effects]], [#link(<sect-6-22>)[§6.22]],
  [#text(weight: "bold", size: 9pt)[die_modifier]], [#link(<sect-6-24>)[§6.24]],
  [#text(weight: "bold", size: 9pt)[effective_movement_at_night]], [#link(<sect-8-1>)[§8.1]],
  [#text(weight: "bold", size: 9pt)[effective_range_at_night]], [#link(<sect-8-1>)[§8.1]],
  [#text(weight: "bold", size: 9pt)[end_player_turn]], [#link(<sect-4>)[§4], #link(<sect-5-13>)[§5.13]],
  [#text(weight: "bold", size: 9pt)[fires_twice]], [#link(<sect-6-42>)[§6.42]],
  [#text(weight: "bold", size: 9pt)[first_player]], [#link(<sect-9-211>)[§9.211]],
  [#text(weight: "bold", size: 9pt)[from_superiority]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[from_total]], [#link(<sect-CRT>)[§CRT]],
  [#text(weight: "bold", size: 9pt)[halve]], [#link(<sect-8-1>)[§8.1]],
  [#text(weight: "bold", size: 9pt)[has_combat_factors]], [#link(<sect-6-51>)[§6.51]],
  [#text(weight: "bold", size: 9pt)[has_los]], [#link(<sect-6-21>)[§6.21], #link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[hex_has_enemy_fort]], [#link(<sect-9-344>)[§9.344]],
  [#text(weight: "bold", size: 9pt)[hex_in_enemy_zoc]], [#link(<sect-4>)[§4], #link(<sect-5-43>)[§5.43]],
  [#text(weight: "bold", size: 9pt)[hit_target_hex]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[howitzer_scatter]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[in_deployment_zone]], [#link(<sect-9-212>)[§9.212]],
  [#text(weight: "bold", size: 9pt)[is_boat]], [#link(<sect-5-24>)[§5.24]],
  [#text(weight: "bold", size: 9pt)[is_crossroad]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[is_friendlies]], [#link(<sect-5-21>)[§5.21], #link(<sect-6-52>)[§6.52]],
  [#text(weight: "bold", size: 9pt)[is_gordon]], [#link(<sect-9-346>)[§9.346]],
  [#text(weight: "bold", size: 9pt)[is_los_trees]], [#link(<sect-6-23>)[§6.23]],
  [#text(weight: "bold", size: 9pt)[is_nile_mouth_crossing]], [#link(<sect-9-345>)[§9.345]],
  [#text(weight: "bold", size: 9pt)[loaded_on]], [#link(<sect-5-21>)[§5.21]],
  [#text(weight: "bold", size: 9pt)[los_table]], [#link(<sect-6-21>)[§6.21], #link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[may_act]], [#link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[may_attack_this_turn]], [#link(<sect-5-3>)[§5.3]],
  [#text(weight: "bold", size: 9pt)[may_be_melee_attacked]], [#link(<sect-7-1>)[§7.1]],
  [#text(weight: "bold", size: 9pt)[may_melee_attack]], [#link(<sect-7-4>)[§7.4]],
  [#text(weight: "bold", size: 9pt)[may_retreat_before_melee]], [#link(<sect-7-5>)[§7.5]],
  [#text(weight: "bold", size: 9pt)[movement_cost]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[movement_cost_with_road]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[mp_spent]], [#link(<sect-5-12>)[§5.12]],
  [#text(weight: "bold", size: 9pt)[net_modifier]], [#link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[new]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[passable_by_land]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[points]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[range_band_for]], [#link(<sect-9-343>)[§9.343]],
  [#text(weight: "bold", size: 9pt)[roads]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[score_elimination]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[sum]], [#link(<sect-7-1>)[§7.1]],
  [#text(weight: "bold", size: 9pt)[sum_to_row]], [#link(<sect-6-14>)[§6.14]],
  [#text(weight: "bold", size: 9pt)[superiority]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[terrain_effects_chart]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[total_for]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[turn_marker_pixel]], [#link(<sect-9-12>)[§9.12]],
  [#text(weight: "bold", size: 9pt)[unit_projects_zoc]], [#link(<sect-5-41>)[§5.41], #link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[value]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[who_scores]], [#link(<sect-9-14>)[§9.14]],
)

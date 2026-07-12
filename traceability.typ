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
#text(size: 9pt)[Total mappings: 108 · Total impl sites: 224]
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
  [#vscode-link("omdurman-types/src/lib.rs", 654) \ #github-link("omdurman-types/src/lib.rs", 654)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L654")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitFormKind]]]], [#raw("652 │     strum::EnumIter,
653 │ )]
654 │ pub enum UnitFormKind {
655 │     #[default]
656 │     Infantry,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 726) \ #github-link("omdurman-rules/src/lib.rs", 726)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L726")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitProfile]]]], [#raw("724 │ /// print no melee value).
725 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
726 │ pub struct UnitProfile {
727 │     pub kind: UnitKind,
728 │     pub identity: UnitIdentity,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 340) \ #github-link("omdurman-rules/src/lib.rs", 340)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L340")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeId]]]], [#raw("338 │ /// (§2.3, §5.54). The number is the brigade ordinal as printed, e.g. `2B`.
339 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
340 │ pub struct BrigadeId {
341 │     pub number: u8,
342 │     pub nationality: BrigadeNationality,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 806) \ #github-link("omdurman-types/src/lib.rs", 806)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L806")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[SpriteAnnotation]]]], [#raw("804 │ /// movement allowance (§5.24); leaders print movement only (§6.51).
805 │ #[derive(Serialize, Deserialize, Clone, Debug)]
806 │ pub struct SpriteAnnotation {
807 │     /// Command/tribe colour. A real game indicator: Dervish leaders may only
808 │     /// stack with units of their own colour, and different tribes may not", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 502) \ #github-link("omdurman-rules/src/lib.rs", 502)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L502")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("500 │ /// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
501 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
502 │ pub enum WeaponClass {
503 │     /// Dervish spears and swords -- no ranged fire at all.
504 │     Melee,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 946) \ #github-link("omdurman-rules/src/lib.rs", 946)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L946")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Howitzer]]]], [#raw("944 │ /// roll on the Howitzer Fire Scattergram (§6.64).
945 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
946 │ pub struct HowitzerResolution {
947 │     pub combat_results_table_roll: DieRoll,
948 │     pub impact_roll: DieRoll,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 419) \ #github-link("omdurman-rules/src/lib.rs", 419)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L419")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId]]]], [#raw("417 │ /// fire; \"old\" gunboats do not (rulebook §2.32).
418 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
419 │ pub enum GunboatId {
420 │     /// One of the five new-type named gunboats with howitzer capability.
421 │     Named(NamedGunboat),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 430) \ #github-link("omdurman-rules/src/lib.rs", 430)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L430")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[NamedGunboat]]]], [#raw("428 │ /// The five named gunboats with howitzer capability (rulebook §6.64, §2.32).
429 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
430 │ pub enum NamedGunboat {
431 │     Sultan,
432 │     Melik,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 443) \ #github-link("omdurman-rules/src/lib.rs", 443)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L443")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OldGunboat]]]], [#raw("441 │ /// in the Maxim Second Fire and Howitzer subphase (§6.42).
442 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
443 │ pub enum OldGunboat {
444 │     LordKitchener,
445 │     Tamai,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 443) \ #github-link("omdurman-rules/src/lib.rs", 443)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L443")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId::Old]]]], [#raw("441 │ /// in the Maxim Second Fire and Howitzer subphase (§6.42).
442 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
443 │ pub enum OldGunboat {
444 │     LordKitchener,
445 │     Tamai,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 425) \ #github-link("omdurman-rules/src/lib.rs", 425)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L425")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId::DervishGunboat]]]], [#raw("423 │     Old(OldGunboat),
424 │     /// A Dervish gunboat (§9.111, §10.14).
425 │     DervishGunboat(u8),
426 │ }
427 │ ", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 231) \ #github-link("omdurman-rules/src/lib.rs", 231)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L231")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTurnIndex]]]], [#raw("229 │ /// One-based Game Turn index (1, 2, ... up to the scenario length) (rulebook §4).
230 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
231 │ pub struct GameTurnIndex(pub u8);
232 │ 
233 │ impl GameTurnIndex {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 286) \ #github-link("omdurman-rules/src/lib.rs", 286)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L286")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Phase]]]], [#raw("284 │ /// etc.
285 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
286 │ pub enum Phase {
287 │     /// Pre-game deployment (§9.2/§9.3/§10): fixed units are placed, each side
288 │     /// deploys its order of battle within its legal zone, and river", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 385) \ #github-link("omdurman-rules/src/effects.rs", 385)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L385")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState]]]], [#raw("383 │ /// All mutable state of a game in progress (rulebook §4).
384 │ #[derive(Serialize, Deserialize, Clone, Debug)]
385 │ pub struct GameState {
386 │     pub scenario: Scenario,
387 │     pub current_turn: GameTurnIndex,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 474) \ #github-link("omdurman-rules/src/effects.rs", 474)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L474")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState::new]]]], [#raw("472 │ impl GameState {
473 │     /// Create a fresh game state for a given scenario (rulebook §4).
474 │     pub fn new(scenario: Scenario) -> Self {
475 │         let first = scenario_turn(scenario, GameTurnIndex(1));
476 │         // First player to *move* per scenario: Campaign -- Anglo-Egyptian moves", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 38) \ #github-link("omdurman-rules/src/effects.rs", 38)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L38")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvancePhase]]]], [#raw(" 36 │     // -- Turn / phase flow ------------------------------------------------
 37 │     /// Advance to the next phase (or next player-turn if melee is done) (rulebook §4).
 38 │     AdvancePhase,
 39 │ 
 40 │     // -- Movement ----------------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1503) \ #github-link("omdurman-rules/src/effects.rs", 1503)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1503")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[advance_phase]]]], [#raw("1501 │ 
1502 │ /// Advance the game state to the next phase (rulebook §4).
1503 │ pub fn advance_phase(state: &mut GameState) -> Result<(), RuleError> {
1504 │     match state.phase {
1505 │         // Leaving deployment is gated: both sides' required order of battle must", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1572) \ #github-link("omdurman-rules/src/effects.rs", 1572)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1572")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[end_player_turn]]]], [#raw("1570 │ 
1571 │ /// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
1572 │ pub fn end_player_turn(state: &mut GameState) -> Result<(), RuleError> {
1573 │     // §6.53: resolve all pending Royal Engineers demolitions before recovering
1574 │     // disrupted units. Each demolition checks adjacency + undisrupted status.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 234) \ #github-link("omdurman-rules/src/lib.rs", 234)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L234")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTurnIndex::value]]]], [#raw("232 │ 
233 │ impl GameTurnIndex {
234 │     pub fn value(self) -> u8 {
235 │         self.0
236 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 466) \ #github-link("omdurman-rules/src/effects.rs", 466)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L466")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[PendingMelee]]]], [#raw("464 │ /// resolution after the reaction window is deterministic and host-ordered (rulebook §7.5).
465 │ #[derive(Serialize, Deserialize, Clone, Debug)]
466 │ pub struct PendingMelee {
467 │     pub attack: MeleeAttack,
468 │     pub attacker_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1322) \ #github-link("omdurman-rules/src/effects.rs", 1322)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1322")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("1320 │     /// does not extend into or out of a Nile hex. With no board loaded these
1321 │     /// reduce to the plain adjacency rule.
1322 │     pub fn hex_in_enemy_zoc(
1323 │         &self,
1324 │         hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 754) \ #github-link("omdurman-rules/src/effects.rs", 754)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L754")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit]]]], [#raw("752 │     /// the same `RuleError` the `MoveUnit` effect would on rejection. Lets the
753 │     /// UI gate input without mutating or duplicating the rules.
754 │     pub fn can_move_unit(&self, unit_id: UnitId, cost: MovementPoints) -> Result<(), RuleError> {
755 │         self.can_move_unit_to(unit_id, None, cost)
756 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[new_game_starts_in_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[scenario_turn_dispatches_correctly]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_advances_through_phases]]]
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
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[disrupted_unit_may_not_act]]]
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
  [#vscode-link("omdurman-rules/src/lib.rs", 1582) \ #github-link("omdurman-rules/src/lib.rs", 1582)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1582")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[constructing_zariba]]]], [#raw("1580 │         // §5.3: a unit constructing Zariba \"may neither fire offensively nor
1581 │         // melee attack during the turn of construction.\"
1582 │         let s = UnitState {
1583 │             constructing_zariba: true,
1584 │             ..UnitState::default()", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 107) \ #github-link("omdurman-rules/src/effects.rs", 107)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L107")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ConstructZariba]]]], [#raw("105 │ 
106 │     /// Begin constructing a Zariba hexside (rulebook §5.3).
107 │     ConstructZariba {
108 │         unit_ids: Vec<UnitId>,
109 │         hexside: HexsideRef,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2510) \ #github-link("omdurman-rules/src/effects.rs", 2510)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2510")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_construct_zariba]]]], [#raw("2508 │ 
2509 │ /// Mark a set of units as constructing a Zariba hexside (rulebook §5.3).
2510 │ pub fn apply_construct_zariba(
2511 │     state: &mut GameState,
2512 │     unit_ids: &[UnitId],", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 779) \ #github-link("omdurman-rules/src/lib.rs", 779)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L779")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitState::may_attack_this_turn]]]], [#raw("777 │     /// A unit that began construction this turn may not fire offensively or
778 │     /// melee (§5.3, §6.53).
779 │     pub fn may_attack_this_turn(self) -> bool {
780 │         !self.disrupted && !self.constructing_zariba && !self.demolishing
781 │     }", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 112) \ #github-link("omdurman-rules/src/lib.rs", 112)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L112")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementAllowance]]]], [#raw("110 │     /// is a named variant.
111 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
112 │     pub enum MovementAllowance {
113 │         /// Immobile (forts, wrecked gunboats).
114 │         Immobile = 0,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 737) \ #github-link("omdurman-rules/src/lib.rs", 737)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L737")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitMovement]]]], [#raw("735 │ /// Movement allowance -- uniform for land units, split for gunboats (rulebook §5.11, §5.24, §5.25).
736 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
737 │ pub enum UnitMovement {
738 │     Land(MovementAllowance),
739 │     Gunboat(GunboatMovement),", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 344) \ #github-link("omdurman-types/src/lib.rs", 344)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L344")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[NileFlow]]]], [#raw("342 │ /// opposite way is **upstream**.
343 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
344 │ pub struct NileFlow {
345 │     /// Direction the current flows toward (downstream).
346 │     pub dir: HexDirection,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 298) \ #github-link("omdurman-types/src/lib.rs", 298)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L298")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDirection]]]], [#raw("296 │ /// (`+q`, `+q+r`, `+r`, `-q`, `-q-r`, `-r` for pointy-top hexes) (rulebook §5.11, §5.24).
297 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
298 │ pub enum HexDirection {
299 │     #[default]
300 │     East = 0,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 152) \ #github-link("omdurman-rules/src/lib.rs", 152)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L152")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementPoints]]]], [#raw("150 │ /// Movement points spent or remaining within a single phase (rulebook §5).
151 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
152 │ pub struct MovementPoints(pub i16);
153 │ 
154 │ /// A distance measured in hexes (range to target, length of a retreat, ...)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 21) \ #github-link("omdurman-rules/src/terrain_chart.rs", 21)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/terrain_chart.rs#L21")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[terrain_effects_chart]]]], [#raw(" 19 │ ///
 20 │ /// Source: printed Terrain Effects Chart on the mapsheet.
 21 │ pub fn terrain_effects_chart(terrain: Terrain) -> TerrainEntry {
 22 │     match terrain {
 23 │         Terrain::Clear => TerrainEntry {", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-types/src/lib.rs", 430) \ #github-link("omdurman-types/src/lib.rs", 430)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L430")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::passable_by_land]]]], [#raw("428 │     }
429 │     /// Whether this terrain may be entered by land units (rulebook §5.11).
430 │     pub fn passable_by_land(self) -> bool {
431 │         !self.is_nile()
432 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 569) \ #github-link("omdurman-types/src/lib.rs", 569)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L569")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexData::is_crossroad]]]], [#raw("567 │             name,
568 │             nile_flow,
569 │             is_crossroad: false,
570 │             setup_letter: None,
571 │         }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 569) \ #github-link("omdurman-types/src/lib.rs", 569)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L569")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TileInfo::is_crossroad]]]], [#raw("567 │             name,
568 │             nile_flow,
569 │             is_crossroad: false,
570 │             setup_letter: None,
571 │         }", block: true, lang: "rs")],
  [#vscode-link("omdurman-hexmap/src/map.rs", 151) \ #github-link("omdurman-hexmap/src/map.rs", 151)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-hexmap/src/map.rs#L151")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameMap::roads]]]], [#raw("149 │         tiles,
150 │         hexsides,
151 │         roads,
152 │         excluded: game_map.excluded.iter().map(|c| (c.q, c.r)).collect(),
153 │         overlay: game_map.overlay.clone(),", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[clear_terrain_no_bonus]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[nile_is_impassable]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[rough_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[swamp_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[hilltop_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[huts_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[movement_cost_convenience_matches_chart]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[movement_cost_with_road_always_one]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[land_unit_may_not_enter_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[movement_cost_without_road_matches_terrain]]]
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1147) \ #github-link("omdurman-rules/src/effects.rs", 1147)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1147")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[mp_spent]]]], [#raw("1145 │ 
1146 │     /// Movement points `unit_id` has already spent this turn (§5.11/§5.12).
1147 │     pub fn mp_spent(&self, unit_id: UnitId) -> i16 {
1148 │         self.mp_spent_this_turn
1149 │             .iter()", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1572) \ #github-link("omdurman-rules/src/effects.rs", 1572)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1572")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[end_player_turn]]]], [#raw("1570 │ 
1571 │ /// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
1572 │ pub fn end_player_turn(state: &mut GameState) -> Result<(), RuleError> {
1573 │     // §6.53: resolve all pending Royal Engineers demolitions before recovering
1574 │     // disrupted units. Each demolition checks adjacency + undisrupted status.", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 636) \ #github-link("omdurman-rules/src/lib.rs", 636)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L636")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_friendlies]]]], [#raw("634 │     /// \"Friendlies\" units obey several special rules (§5.21, §5.23, §6.52,
635 │     /// §9.14 victory conditions).
636 │     pub fn is_friendlies(&self) -> bool {
637 │         matches!(
638 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 757) \ #github-link("omdurman-rules/src/lib.rs", 757)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L757")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[loaded_on]]]], [#raw("755 │     pub disrupted: bool,
756 │     /// `Some(gunboat)` after a \"Friendlies\" unit loads onto a gunboat (§5.21).
757 │     pub loaded_on: Option<UnitId>,
758 │     /// Set while the unit is building Zariba hexsides -- neither offensive
759 │     /// fire nor melee allowed that turn (§5.3).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 134) \ #github-link("omdurman-rules/src/effects.rs", 134)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L134")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FriendliesTransport]]]], [#raw("132 │ 
133 │     /// Load/disembark the \"Friendlies\" brigade via gunboat (rulebook §5.21).
134 │     FriendliesTransport(crate::FriendliesTransport),
135 │ 
136 │     // -- Optional rules ----------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2769) \ #github-link("omdurman-rules/src/effects.rs", 2769)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2769")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_friendlies_transport]]]], [#raw("2767 │ ///     transport mission ends and the unit is freed (a disembarking `MoveUnit`
2768 │ ///     should follow, costed by terrain).
2769 │ pub fn apply_friendlies_transport(
2770 │     state: &mut GameState,
2771 │     action: FriendliesTransport,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1023) \ #github-link("omdurman-rules/src/lib.rs", 1023)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1023")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FriendliesTransport]]]], [#raw("1021 │ /// tracks each unit–gunboat pair independently.
1022 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
1023 │ pub enum FriendliesTransport {
1024 │     /// Turn N (the load turn): unit and gunboat started adjacent; unit
1025 │     /// loads onto (stacks with) the gunboat.", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 771) \ #github-link("omdurman-rules/src/effects.rs", 771)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L771")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw("769 │     ///
770 │     /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
771 │     pub fn can_move_unit_to(
772 │         &self,
773 │         unit_id: UnitId,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-types/src/lib.rs", 158) \ #github-link("omdurman-types/src/lib.rs", 158)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L158")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideRef]]]], [#raw("156 │ /// data by [`HexsideRef`].
157 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
158 │ pub struct HexsideRef {
159 │     pub a: HexCoord,
160 │     pub b: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 193) \ #github-link("omdurman-types/src/lib.rs", 193)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L193")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideKind]]]], [#raw("191 │     strum::EnumIter,
192 │ )]
193 │ pub enum HexsideKind {
194 │     /// City wall (Khartoum, walled city of Omdurman). Blocks LOS, blocks
195 │     /// movement except across gates/breaches (§5.23), blocks ZOC into the city", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 257) \ #github-link("omdurman-types/src/lib.rs", 257)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L257")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_movement]]]], [#raw("255 │     /// Whether land movement may *not* cross this side (§5.23). Walls block
256 │     /// movement except at gates/breaches.
257 │     pub fn blocks_movement(self) -> bool {
258 │         matches!(self, HexsideKind::Wall)
259 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2329) \ #github-link("omdurman-rules/src/effects.rs", 2329)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2329")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_retreat_before_melee]]]], [#raw("2327 │     /// two hexes away and empty. (Does not verify the attacker is infantry --
2328 │     /// the caller offers the retreat only in response to one.)
2329 │     pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2330 │         let unit = self
2331 │             .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-types/src/lib.rs", 344) \ #github-link("omdurman-types/src/lib.rs", 344)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L344")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[NileFlow]]]], [#raw("342 │ /// opposite way is **upstream**.
343 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
344 │ pub struct NileFlow {
345 │     /// Direction the current flows toward (downstream).
346 │     pub dir: HexDirection,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 570) \ #github-link("omdurman-rules/src/lib.rs", 570)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L570")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatMovement]]]], [#raw("568 │ /// the turn.
569 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
570 │ pub struct GunboatMovement {
571 │     pub upstream: MovementAllowance,
572 │     pub downstream: MovementAllowance,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 672) \ #github-link("omdurman-types/src/lib.rs", 672)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L672")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_boat]]]], [#raw("670 │ impl UnitFormKind {
671 │     /// Gunboats use the split upstream/downstream movement allowance (§5.24).
672 │     pub fn is_boat(self) -> bool {
673 │         matches!(self, UnitFormKind::Gunboat)
674 │     }", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/lib.rs", 114) \ #github-link("omdurman-rules/src/lib.rs", 114)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L114")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Immobile]]]], [#raw("112 │     pub enum MovementAllowance {
113 │         /// Immobile (forts, wrecked gunboats).
114 │         Immobile = 0,
115 │         One = 1,
116 │         Two = 2,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 469) \ #github-link("omdurman-rules/src/lib.rs", 469)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L469")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Fort]]]], [#raw("467 │     Gunboat,
468 │     /// Permanent emplacement -- may not move once placed (§5.25).
469 │     Fort,
470 │     /// Dervish leader: has fire/melee/movement factors and may melee attack.
471 │     DervishLeaderUnit,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 741) \ #github-link("omdurman-rules/src/lib.rs", 741)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L741")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitMovement::Immobile]]]], [#raw("739 │     Gunboat(GunboatMovement),
740 │     /// Forts may not move once placed (§5.25).
741 │     Immobile,
742 │ }
743 │ ", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 771) \ #github-link("omdurman-rules/src/effects.rs", 771)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L771")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw("769 │     ///
770 │     /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
771 │     pub fn can_move_unit_to(
772 │         &self,
773 │         unit_id: UnitId,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 812) \ #github-link("omdurman-rules/src/lib.rs", 812)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L812")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("810 │ /// Used by the engine when answering \"is this hex in an enemy ZOC?\".
811 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
812 │ pub enum ZocReason {
813 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
814 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1287) \ #github-link("omdurman-rules/src/effects.rs", 1287)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1287")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[unit_projects_zoc]]]], [#raw("1285 │     /// §5.44) need the game map, which the engine does not hold; the app layers
1286 │     /// those on top. This is the position/kind/disruption core of the rule.
1287 │     pub fn unit_projects_zoc(
1288 │         &self,
1289 │         unit: &UnitPlacement,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 771) \ #github-link("omdurman-rules/src/effects.rs", 771)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L771")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw("769 │     ///
770 │     /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
771 │     pub fn can_move_unit_to(
772 │         &self,
773 │         unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1322) \ #github-link("omdurman-rules/src/effects.rs", 1322)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1322")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("1320 │     /// does not extend into or out of a Nile hex. With no board loaded these
1321 │     /// reduce to the plain adjacency rule.
1322 │     pub fn hex_in_enemy_zoc(
1323 │         &self,
1324 │         hex: HexCoord,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 812) \ #github-link("omdurman-rules/src/lib.rs", 812)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L812")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("810 │ /// Used by the engine when answering \"is this hex in an enemy ZOC?\".
811 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
812 │ pub enum ZocReason {
813 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
814 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 198) \ #github-link("omdurman-types/src/lib.rs", 198)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L198")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Wall]]]], [#raw("196 │     /// (§5.44), blocks melee (§7.2), blocks advance-after-combat (§6.82).
197 │     #[default]
198 │     Wall,
199 │     /// Gate hexside in a wall. ZOC extends *out of* the walled city through
200 │     /// gates but not into it (§5.44). Melee may be made through a gate (§7.2).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 207) \ #github-link("omdurman-types/src/lib.rs", 207)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L207")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Khor]]]], [#raw("205 │     /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
206 │     /// combat may not cross (§6.82).
207 │     Khor,
208 │     /// Crest line. Blocks LOS unless the firer is on the higher side
209 │     /// (§6.3 note 7).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 825) \ #github-link("omdurman-rules/src/lib.rs", 825)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L825")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason::Zariba]]]], [#raw("823 │     /// Zariba hexside ZOC behaviour in the historical scenario / when the
824 │     /// Zariba is constructed (§5.44).
825 │     Zariba,
826 │ }
827 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1287) \ #github-link("omdurman-rules/src/effects.rs", 1287)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1287")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[unit_projects_zoc]]]], [#raw("1285 │     /// §5.44) need the game map, which the engine does not hold; the app layers
1286 │     /// those on top. This is the position/kind/disruption core of the rule.
1287 │     pub fn unit_projects_zoc(
1288 │         &self,
1289 │         unit: &UnitPlacement,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 834) \ #github-link("omdurman-rules/src/lib.rs", 834)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L834")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OverLimit]]]], [#raw("832 │     /// and the gunboat exception.
833 │     #[error(\"hex stack exceeds the four-unit limit\")]
834 │     OverLimit,
835 │     /// \"Gunboats may not stack with any other unit\" (§5.51, exception §5.21).
836 │     #[error(\"gunboats may not stack with non-gunboat units\")]", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 837) \ #github-link("omdurman-rules/src/lib.rs", 837)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L837")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatStack]]]], [#raw("835 │     /// \"Gunboats may not stack with any other unit\" (§5.51, exception §5.21).
836 │     #[error(\"gunboats may not stack with non-gunboat units\")]
837 │     GunboatStack,
838 │     /// \"Units of different Dervish tribes may not stack together\" (§5.52).
839 │     #[error(\"Dervish units of different tribes may not stack\")]", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 840) \ #github-link("omdurman-rules/src/lib.rs", 840)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L840")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishTribeMix]]]], [#raw("838 │     /// \"Units of different Dervish tribes may not stack together\" (§5.52).
839 │     #[error(\"Dervish units of different tribes may not stack\")]
840 │     DervishTribeMix,
841 │     /// \"If Dervish leaders elect to stack, they may only stack with units of
842 │     /// their command (i.e. colour)\" (§5.53).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 466) \ #github-link("omdurman-types/src/lib.rs", 466)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L466")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::overlay_color]]]], [#raw("464 │ 
465 │     /// Return an RGBA colour suitable for a terrain-type overlay.
466 │     pub fn overlay_color(self) -> [f32; 4] {
467 │         self.color().rgba()
468 │     }", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 844) \ #github-link("omdurman-rules/src/lib.rs", 844)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L844")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishLeaderCommandMismatch]]]], [#raw("842 │     /// their command (i.e. colour)\" (§5.53).
843 │     #[error(\"Dervish leader may only stack with units of their own command\")]
844 │     DervishLeaderCommandMismatch,
845 │ }
846 │ ", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 850) \ #github-link("omdurman-rules/src/lib.rs", 850)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L850")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeIntegrity]]]], [#raw("848 │ /// stack contains all four battalions of a single Anglo-Egyptian brigade.
849 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
850 │ pub enum BrigadeIntegrity {
851 │     None,
852 │     Integrated(BrigadeId),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 699) \ #github-link("omdurman-rules/src/lib.rs", 699)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L699")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[brigade_integrity]]]], [#raw("697 │ /// Only a full stack of four battalions qualifies.  Three or fewer may still
698 │ /// stack and fire, but they receive no brigade-integrity bonus.
699 │ pub fn brigade_integrity(identities: &[UnitIdentity]) -> BrigadeIntegrity {
700 │     let Some(brigade) = identities.first().and_then(|i| i.brigade()) else {
701 │         return BrigadeIntegrity::None;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 349) \ #github-link("omdurman-rules/src/lib.rs", 349)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L349")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BattalionOrdinal]]]], [#raw("347 │     /// brigade integrity requires all four stacked in one hex (§5.54).
348 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
349 │     pub enum BattalionOrdinal {
350 │         First = 1,
351 │         Second = 2,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 712) \ #github-link("omdurman-types/src/lib.rs", 712)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L712")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Brigade]]]], [#raw("710 │     Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display, strum::EnumIter,
711 │ )]
712 │ pub enum BrigadeNationality {
713 │     /// `xB` -- British.
714 │     British,", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/los_table.rs", 89) \ #github-link("omdurman-rules/src/los_table.rs", 89)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L89")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosSpecialNote]]]], [#raw(" 87 │ /// 7. Crest hexsides block LOS unless the firer is on the higher side
 88 │ ///    of the crest.
 89 │ pub enum LosSpecialNote {
 90 │     MaxTwoTreeHutHexes,
 91 │     HilltopToHilltop,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 234) \ #github-link("omdurman-types/src/lib.rs", 234)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L234")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_los]]]], [#raw("232 │     /// Whether this hexside blocks line of sight across it (§6.3). Crest is
233 │     /// directional and handled by the caller; here it is treated as blocking.
234 │     pub fn blocks_los(self) -> bool {
235 │         matches!(self, HexsideKind::Wall | HexsideKind::Crest)
236 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 436) \ #github-link("omdurman-types/src/lib.rs", 436)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L436")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::blocks_los]]]], [#raw("434 │     /// Whether an intervening hex of this terrain unconditionally blocks line
435 │     /// of sight (§6.3).
436 │     pub fn blocks_los(self) -> bool {
437 │         matches!(self, Terrain::Huts | Terrain::Building)
438 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[los_wall_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[los_ground_to_ground_clear]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[los_hilltop_to_huts_blocked]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ground_firer_covers_all_targets]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[rough_firer_covers_all_targets]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[hilltop_firer_covers_all_targets]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[all_24_cells_exercised]]]
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
  [#vscode-link("omdurman-rules/src/lib.rs", 502) \ #github-link("omdurman-rules/src/lib.rs", 502)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L502")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("500 │ /// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
501 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
502 │ pub enum WeaponClass {
503 │     /// Dervish spears and swords -- no ranged fire at all.
504 │     Melee,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 2374) \ #github-link("omdurman-rules/src/effects.rs", 2374)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2374")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("2372 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
2373 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
2374 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2375 │         let unit = self
2376 │             .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/lib.rs", 67) \ #github-link("omdurman-rules/src/lib.rs", 67)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L67")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireFactor]]]], [#raw(" 65 │     /// Every possible value from the annotated counter set is a named variant.
 66 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
 67 │     pub enum FireFactor {
 68 │         One = 1,
 69 │         Three = 3,", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/lib.rs", 81) \ #github-link("omdurman-rules/src/lib.rs", 81)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L81")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[sum_to_row]]]], [#raw(" 79 │ impl FireFactor {
 80 │     /// Sum multiple fire factors and return the corresponding Combat Results Table row (rulebook §6.11).
 81 │     pub fn sum_to_row<'a>(factors: impl IntoIterator<Item = &'a FireFactor>) -> FireFactorRow {
 82 │         let total: u16 = factors.into_iter().map(|f| f.value()).sum();
 83 │         crate::combat_results_table::FireFactorRow::from_total(total)", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 537) \ #github-link("omdurman-rules/src/lib.rs", 537)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L537")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RangeBand]]]], [#raw("535 │ /// multiplied at a given distance (§6.22).
536 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
537 │ pub enum RangeBand {
538 │     Tripled,
539 │     Doubled,", block: true, lang: "rs")],
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
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/lib.rs", 521) \ #github-link("omdurman-rules/src/lib.rs", 521)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L521")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Range]]]], [#raw("519 │ /// (rulebook §6.22). Distances beyond 10 hexes are out of range for all weapons.
520 │ #[derive(Clone, Copy, PartialEq, Eq, Debug)]
521 │ pub enum Range {
522 │     One,
523 │     Two,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 537) \ #github-link("omdurman-rules/src/lib.rs", 537)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L537")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RangeBand]]]], [#raw("535 │ /// multiplied at a given distance (§6.22).
536 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
537 │ pub enum RangeBand {
538 │     Tripled,
539 │     Doubled,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 157) \ #github-link("omdurman-rules/src/lib.rs", 157)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L157")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDistance]]]], [#raw("155 │ /// (rulebook §6.22, §7.5).
156 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
157 │ pub struct HexDistance(pub u16);
158 │ 
159 │ impl HexDistance {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_rifles_doubled_at_range_1]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_rifles_halved_at_range_4]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_howitzer_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_rifles_shorter_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[melee_only_range_1]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_range_effects_artillery_full]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_range_effects_maxims_match_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_range_effects_distance_over_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_range_effects_howitzer_halved_4_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_maxims_and_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_melee]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[dervish_range_effects_distance_over_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_combat_eliminates_target]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[max_day_range_all_combos]]]
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
  [#vscode-link("omdurman-rules/src/lib.rs", 871) \ #github-link("omdurman-rules/src/lib.rs", 871)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L871")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain]]]], [#raw("869 │     /// Negative modifier from the Terrain Effects Chart applied to the
870 │     /// defender's hex (§6.23).
871 │     Terrain(i16),
872 │     /// -2 thorn-hedge defensive modifier (§9.231).
873 │     ZaribaThornHedge,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 234) \ #github-link("omdurman-types/src/lib.rs", 234)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L234")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::blocks_los]]]], [#raw("232 │     /// Whether this hexside blocks line of sight across it (§6.3). Crest is
233 │     /// directional and handled by the caller; here it is treated as blocking.
234 │     pub fn blocks_los(self) -> bool {
235 │         matches!(self, HexsideKind::Wall | HexsideKind::Crest)
236 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 443) \ #github-link("omdurman-types/src/lib.rs", 443)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L443")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::is_los_trees]]]], [#raw("441 │     /// line of sight is blocked by more than two intervening tree hexes
442 │     /// (§6.3 note 1).
443 │     pub fn is_los_trees(self) -> bool {
444 │         matches!(self, Terrain::Trees)
445 │     }", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 865) \ #github-link("omdurman-rules/src/lib.rs", 865)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L865")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AngloEgyptianDirectFire]]]], [#raw("863 │ pub enum FireModifier {
864 │     /// +1 to all Anglo-Egyptian *direct* fire (§6.24).
865 │     AngloEgyptianDirectFire,
866 │     /// +1 brigade integrity, applied only if all four battalions fire at
867 │     /// the same enemy-occupied hex (§5.54, §6.24).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 850) \ #github-link("omdurman-rules/src/lib.rs", 850)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L850")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeIntegrity]]]], [#raw("848 │ /// stack contains all four battalions of a single Anglo-Egyptian brigade.
849 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
850 │ pub enum BrigadeIntegrity {
851 │     None,
852 │     Integrated(BrigadeId),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 881) \ #github-link("omdurman-rules/src/lib.rs", 881)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L881")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[die_modifier]]]], [#raw("879 │ impl FireModifier {
880 │     /// Return the numeric die-roll modifier for this bonus/penalty (rulebook §6.24, §5.54, §6.23, §9.231, §9.232).
881 │     pub fn die_modifier(self) -> i16 {
882 │         match self {
883 │             FireModifier::AngloEgyptianDirectFire | FireModifier::BrigadeIntegrity => 1,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 303) \ #github-link("omdurman-rules/src/lib.rs", 303)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L303")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DirectFire]]]], [#raw("301 │ pub enum FireSubPhase {
302 │     /// Direct fire (§6.41). Both sides participate in this sub-phase.
303 │     DirectFire,
304 │     /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
305 │     MaximSecondAndHowitzer,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 305) \ #github-link("omdurman-rules/src/lib.rs", 305)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L305")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MaximSecondAndHowitzer]]]], [#raw("303 │     DirectFire,
304 │     /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
305 │     MaximSecondAndHowitzer,
306 │ }
307 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 690) \ #github-link("omdurman-types/src/lib.rs", 690)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L690")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[fires_twice]]]], [#raw("688 │     /// again in the Maxim Second Fire Subphase (rulebook §6.42). The counter
689 │     /// is marked \"x2\" in the editor to surface this.
690 │     pub fn fires_twice(self) -> bool {
691 │         matches!(self, UnitFormKind::Maxim)
692 │     }", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 408) \ #github-link("omdurman-rules/src/lib.rs", 408)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L408")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BritishLeader]]]], [#raw("406 │ /// to claim the Mahdi's Tomb (§9.14).
407 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
408 │ pub enum BritishLeader {
409 │     Kitchener,
410 │     Gatacre,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 473) \ #github-link("omdurman-rules/src/lib.rs", 473)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L473")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BritishLeaderUnit]]]], [#raw("471 │     DervishLeaderUnit,
472 │     /// Anglo-Egyptian leader: movement only (§6.51).
473 │     BritishLeaderUnit,
474 │ }
475 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 683) \ #github-link("omdurman-types/src/lib.rs", 683)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L683")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_combat_factors]]]], [#raw("681 │     /// British and Dervish leaders print a movement factor only (§6.51); other
682 │     /// playable kinds carry fire and/or melee factors.
683 │     pub fn has_combat_factors(self) -> bool {
684 │         !matches!(self, UnitFormKind::BritishLeader | UnitFormKind::Marker)
685 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.52 -- Anglo-Egyptian Friendlies Brigade") <sect-6-52>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 636) \ #github-link("omdurman-rules/src/lib.rs", 636)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L636")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_friendlies]]]], [#raw("634 │     /// \"Friendlies\" units obey several special rules (§5.21, §5.23, §6.52,
635 │     /// §9.14 victory conditions).
636 │     pub fn is_friendlies(&self) -> bool {
637 │         matches!(
638 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 721) \ #github-link("omdurman-types/src/lib.rs", 721)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L721")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Friendlies]]]], [#raw("719 │     /// Native volunteer brigade -- the Shaggyeh (§6.52). Do not receive
720 │     /// brigade integrity (§5.54 enumerates only British/Egyptian/Sudanese).
721 │     Friendlies,
722 │ }
723 │ ", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 605) \ #github-link("omdurman-rules/src/lib.rs", 605)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L605")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RoyalEngineers]]]], [#raw("603 │     /// The Royal Engineers (§6.53) -- a *specific* unit, not a class, so we
604 │     /// model it explicitly.
605 │     RoyalEngineers,
606 │ }
607 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 763) \ #github-link("omdurman-rules/src/lib.rs", 763)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L763")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[demolishing]]]], [#raw("761 │     /// Set when the Royal Engineers are committed to a demolition this turn
762 │     /// (§6.53) -- neither offensive fire nor melee allowed that turn.
763 │     pub demolishing: bool,
764 │     /// Set when a gunboat has lost its engines to a river mine (§10.12, roll
765 │     /// 5-7): it may no longer move under power and instead drifts two hexes per", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 113) \ #github-link("omdurman-rules/src/effects.rs", 113)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L113")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Demolition]]]], [#raw("111 │ 
112 │     /// Royal Engineers demolition (rulebook §6.53).
113 │     Demolition {
114 │         unit_id: UnitId,
115 │         target: DemolitionTarget,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2530) \ #github-link("omdurman-rules/src/effects.rs", 2530)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2530")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_demolition]]]], [#raw("2528 │ /// resolution happens at end of turn via [`apply_resolve_demolition`], which
2529 │ /// checks the engineer is still adjacent and undisrupted.
2530 │ pub fn apply_demolition(
2531 │     state: &mut GameState,
2532 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1006) \ #github-link("omdurman-rules/src/lib.rs", 1006)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1006")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DemolitionTarget]]]], [#raw("1004 │ /// disrupted or driven off.
1005 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
1006 │ pub enum DemolitionTarget {
1007 │     Fort(UnitId),
1008 │     WallHexside(HexsideRef),", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 812) \ #github-link("omdurman-rules/src/lib.rs", 812)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L812")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("810 │ /// Used by the engine when answering \"is this hex in an enemy ZOC?\".
811 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
812 │ pub enum ZocReason {
813 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
814 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 469) \ #github-link("omdurman-rules/src/lib.rs", 469)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L469")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Fort]]]], [#raw("467 │     Gunboat,
468 │     /// Permanent emplacement -- may not move once placed (§5.25).
469 │     Fort,
470 │     /// Dervish leader: has fire/melee/movement factors and may melee attack.
471 │     DervishLeaderUnit,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 773) \ #github-link("omdurman-rules/src/lib.rs", 773)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L773")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitState::may_act]]]], [#raw("771 │ impl UnitState {
772 │     /// A disrupted unit may not move, fire, or melee (rulebook §5, reference notes).
773 │     pub fn may_act(self) -> bool {
774 │         !self.disrupted
775 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 751) \ #github-link("omdurman-rules/src/lib.rs", 751)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L751")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitState]]]], [#raw("749 │ /// rather than one big enum.
750 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
751 │ pub struct UnitState {
752 │     /// Reference table: \"Disrupted units: no ZOC; may not move; may not fire
753 │     /// offensively or defensively; may not melee; are turned face up at the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 910) \ #github-link("omdurman-rules/src/lib.rs", 910)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L910")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireAttack]]]], [#raw("908 │ /// modifiers (rulebook §6).
909 │ #[derive(Serialize, Deserialize, Clone, Debug)]
910 │ pub struct FireAttack {
911 │     pub firing_player: Player,
912 │     pub phase: Phase,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 925) \ #github-link("omdurman-rules/src/lib.rs", 925)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L925")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireAttack::net_modifier]]]], [#raw("923 │ impl FireAttack {
924 │     /// Sum of all fire modifiers applied to this attack (rulebook §6.24).
925 │     pub fn net_modifier(&self) -> i16 {
926 │         self.modifiers.iter().map(|m| m.die_modifier()).sum()
927 │     }", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 502) \ #github-link("omdurman-rules/src/lib.rs", 502)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L502")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("500 │ /// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
501 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
502 │ pub enum WeaponClass {
503 │     /// Dervish spears and swords -- no ranged fire at all.
504 │     Melee,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 502) \ #github-link("omdurman-rules/src/lib.rs", 502)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L502")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("500 │ /// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
501 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
502 │ pub enum WeaponClass {
503 │     /// Dervish spears and swords -- no ranged fire at all.
504 │     Melee,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-types/src/lib.rs", 204) \ #github-link("omdurman-types/src/lib.rs", 204)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L204")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Breach]]]], [#raw("202 │     /// Breach in a wall (artillery/§6.63 or Royal Engineers/§6.53). ZOC both
203 │     /// ways; LOS no longer blocked across the hexside.
204 │     Breach,
205 │     /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
206 │     /// combat may not cross (§6.82).", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/lib.rs", 419) \ #github-link("omdurman-rules/src/lib.rs", 419)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L419")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId]]]], [#raw("417 │ /// fire; \"old\" gunboats do not (rulebook §2.32).
418 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
419 │ pub enum GunboatId {
420 │     /// One of the five new-type named gunboats with howitzer capability.
421 │     Named(NamedGunboat),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 62) \ #github-link("omdurman-rules/src/effects.rs", 62)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L62")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HowitzerFire]]]], [#raw(" 60 │ 
 61 │     /// Resolve a howitzer bombardment (two rolls: Combat Results Table + impact scatter) (rulebook §6.64).
 62 │     HowitzerFire {
 63 │         attack: FireAttack,
 64 │         combat_results_table_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1912) \ #github-link("omdurman-rules/src/effects.rs", 1912)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1912")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_howitzer_fire]]]], [#raw("1910 │ 
1911 │ /// Validate and apply a howitzer fire attack (scatter path) (rulebook §6.64).
1912 │ pub fn apply_howitzer_fire(
1913 │     state: &mut GameState,
1914 │     attack: &FireAttack,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 946) \ #github-link("omdurman-rules/src/lib.rs", 946)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L946")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HowitzerResolution]]]], [#raw("944 │ /// roll on the Howitzer Fire Scattergram (§6.64).
945 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
946 │ pub struct HowitzerResolution {
947 │     pub combat_results_table_roll: DieRoll,
948 │     pub impact_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 953) \ #github-link("omdurman-rules/src/lib.rs", 953)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L953")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HowitzerResolution::hit_target_hex]]]], [#raw("951 │ impl HowitzerResolution {
952 │     /// The designated target hex is hit on impact roll 7-10 (§6.64).
953 │     pub fn hit_target_hex(self) -> bool {
954 │         use DieRoll::*;
955 │         matches!(self.impact_roll, Seven | Eight | Nine | Ten)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 975) \ #github-link("omdurman-rules/src/effects.rs", 975)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L975")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_fire_at]]]], [#raw("973 │     /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
974 │     /// (Howitzer fire ignores LOS entirely -- §6.64.)
975 │     pub fn can_fire_at(
976 │         &self,
977 │         firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_on_target_7_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_scatters_below_7]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_short_on_5_6]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_long_on_3_4]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[howitzer_left_right_on_1_2]]]
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
  [#vscode-link("omdurman-rules/src/effects.rs", 100) \ #github-link("omdurman-rules/src/effects.rs", 100)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L100")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw(" 98 │     /// after fire, §7.6 after melee). Eligible units are adjacent attackers
 99 │     /// that are not artillery; the target hex must be empty of enemies.
100 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
101 │ 
102 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2478) \ #github-link("omdurman-rules/src/effects.rs", 2478)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2478")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_advance_after_combat]]]], [#raw("2476 │ 
2477 │ /// Apply an advance-after-combat for a unit (rulebook §6.82, §7.6).
2478 │ pub fn apply_advance_after_combat(
2479 │     state: &mut GameState,
2480 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 100) \ #github-link("omdurman-rules/src/effects.rs", 100)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L100")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw(" 98 │     /// after fire, §7.6 after melee). Eligible units are adjacent attackers
 99 │     /// that are not artillery; the target hex must be empty of enemies.
100 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
101 │ 
102 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 245) \ #github-link("omdurman-types/src/lib.rs", 245)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L245")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_advance_after_combat]]]], [#raw("243 │ 
244 │     /// Whether advance-after-combat may *not* cross this side (§6.82, §7.6).
245 │     pub fn blocks_advance_after_combat(self) -> bool {
246 │         matches!(
247 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2374) \ #github-link("omdurman-rules/src/effects.rs", 2374)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2374")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("2372 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
2373 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
2374 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2375 │         let unit = self
2376 │             .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/lib.rs", 91) \ #github-link("omdurman-rules/src/lib.rs", 91)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L91")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeFactor]]]], [#raw(" 89 │     /// Every possible value from the annotated counter set is a named variant.
 90 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
 91 │     pub enum MeleeFactor {
 92 │         One = 1,
 93 │         Three = 3,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 102) \ #github-link("omdurman-rules/src/lib.rs", 102)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L102")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeFactor::sum]]]], [#raw("100 │ impl MeleeFactor {
101 │     /// Sum multiple melee factors (rulebook §7.1).
102 │     pub fn sum<'a>(factors: impl IntoIterator<Item = &'a MeleeFactor>) -> u16 {
103 │         factors.into_iter().map(|f| f.value()).sum()
104 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 487) \ #github-link("omdurman-rules/src/lib.rs", 487)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L487")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_be_melee_attacked]]]], [#raw("485 │ 
486 │     /// Gunboats neither attack nor are attacked in melee (§7.1).
487 │     pub fn may_be_melee_attacked(self) -> bool {
488 │         !matches!(self, UnitKind::Gunboat)
489 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 487) \ #github-link("omdurman-rules/src/lib.rs", 487)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L487")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind::may_be_melee_attacked]]]], [#raw("485 │ 
486 │     /// Gunboats neither attack nor are attacked in melee (§7.1).
487 │     pub fn may_be_melee_attacked(self) -> bool {
488 │         !matches!(self, UnitKind::Gunboat)
489 │     }", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-types/src/lib.rs", 240) \ #github-link("omdurman-types/src/lib.rs", 240)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L240")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_melee]]]], [#raw("238 │     /// Whether melee may *not* be made across this side (§7.2). Gates and
239 │     /// breaches are passable to melee.
240 │     pub fn blocks_melee(self) -> bool {
241 │         matches!(self, HexsideKind::Wall | HexsideKind::ZaribaThornHedge)
242 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1109) \ #github-link("omdurman-rules/src/effects.rs", 1109)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1109")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_melee]]]], [#raw("1107 │     /// Does **not** check wall/khor hexsides (§7.2) -- those need the game map,
1108 │     /// which the rules engine does not hold; the app gates on them.
1109 │     pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
1110 │         let unit = self
1111 │             .find_unit(attacker)", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/effects.rs", 72) \ #github-link("omdurman-rules/src/effects.rs", 72)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L72")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeCombat]]]], [#raw(" 70 │     /// Used for an immediate resolution with no reaction window (and as the
 71 │     /// resolution primitive in tests).
 72 │     MeleeCombat {
 73 │         attack: MeleeAttack,
 74 │         attacker_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2110) \ #github-link("omdurman-rules/src/effects.rs", 2110)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2110")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_melee_combat]]]], [#raw("2108 │ 
2109 │ /// Apply a simultaneous melee combat between two adjacent hexes (rulebook §7).
2110 │ pub fn apply_melee_combat(
2111 │     state: &mut GameState,
2112 │     attack: &MeleeAttack,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 479) \ #github-link("omdurman-rules/src/lib.rs", 479)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L479")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_melee_attack]]]], [#raw("477 │     /// Rulebook §7.4 -- only infantry, cavalry, camel and Dervish leaders may
478 │     /// melee *attack*. All others (except gunboats) may melee *defend* (§7.1).
479 │     pub fn may_melee_attack(self) -> bool {
480 │         matches!(
481 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 459) \ #github-link("omdurman-rules/src/lib.rs", 459)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L459")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind]]]], [#raw("457 │ /// engine prove the constraint.
458 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
459 │ pub enum UnitKind {
460 │     /// Foot infantry. Includes Anglo-Egyptian infantry, \"Friendlies\",
461 │     /// Royal Engineers, and Dervish foot tribes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 335) \ #github-link("omdurman-rules/src/lib.rs", 335)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L335")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishTribe]]]], [#raw("333 │ // ---------------------------------------------------------------------------
334 │ 
335 │ pub use omdurman_types::DervishTribe;
336 │ 
337 │ /// Anglo-Egyptian infantry brigades -- designation printed on the counter", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1109) \ #github-link("omdurman-rules/src/effects.rs", 1109)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1109")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_melee]]]], [#raw("1107 │     /// Does **not** check wall/khor hexsides (§7.2) -- those need the game map,
1108 │     /// which the rules engine does not hold; the app gates on them.
1109 │     pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
1110 │         let unit = self
1111 │             .find_unit(attacker)", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 95) \ #github-link("omdurman-rules/src/effects.rs", 95)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L95")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RetreatBeforeMelee]]]], [#raw(" 93 │     /// melee attack, *before* it is resolved (§7.5). One retreat per unit per
 94 │     /// turn. (rulebook §7.5).
 95 │     RetreatBeforeMelee { unit_id: UnitId, to: HexCoord },
 96 │ 
 97 │     /// An attacking unit advances into a hex vacated by combat (rulebook §6.82", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2463) \ #github-link("omdurman-rules/src/effects.rs", 2463)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2463")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_retreat_before_melee]]]], [#raw("2461 │ 
2462 │ /// Apply a retreat-before-melee for a cavalry/camel unit (rulebook §7.5).
2463 │ pub fn apply_retreat_before_melee(
2464 │     state: &mut GameState,
2465 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 493) \ #github-link("omdurman-rules/src/lib.rs", 493)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L493")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_retreat_before_melee]]]], [#raw("491 │     /// Cavalry and camel units may retreat two hexes from an infantry melee
492 │     /// attack (§7.5).
493 │     pub fn may_retreat_before_melee(self) -> bool {
494 │         matches!(self, UnitKind::Cavalry | UnitKind::Camel)
495 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 157) \ #github-link("omdurman-rules/src/lib.rs", 157)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L157")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDistance]]]], [#raw("155 │ /// (rulebook §6.22, §7.5).
156 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
157 │ pub struct HexDistance(pub u16);
158 │ 
159 │ impl HexDistance {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2329) \ #github-link("omdurman-rules/src/effects.rs", 2329)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2329")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_retreat_before_melee]]]], [#raw("2327 │     /// two hexes away and empty. (Does not verify the attacker is infantry --
2328 │     /// the caller offers the retreat only in response to one.)
2329 │     pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2330 │         let unit = self
2331 │             .find_unit(unit_id)", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 100) \ #github-link("omdurman-rules/src/effects.rs", 100)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L100")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw(" 98 │     /// after fire, §7.6 after melee). Eligible units are adjacent attackers
 99 │     /// that are not artillery; the target hex must be empty of enemies.
100 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
101 │ 
102 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2478) \ #github-link("omdurman-rules/src/effects.rs", 2478)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2478")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_advance_after_combat]]]], [#raw("2476 │ 
2477 │ /// Apply an advance-after-combat for a unit (rulebook §6.82, §7.6).
2478 │ pub fn apply_advance_after_combat(
2479 │     state: &mut GameState,
2480 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2374) \ #github-link("omdurman-rules/src/effects.rs", 2374)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2374")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("2372 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
2373 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
2374 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
2375 │         let unit = self
2376 │             .find_unit(unit_id)", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 964) \ #github-link("omdurman-rules/src/lib.rs", 964)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L964")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier]]]], [#raw("962 │ 
963 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
964 │ pub enum MeleeModifier {
965 │     /// +2 to all Dervish melee rolls (§7.7).
966 │     DervishStandard,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 198) \ #github-link("omdurman-rules/src/lib.rs", 198)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L198")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DieModifier]]]], [#raw("196 │ /// A die-roll modifier from a single named source (rulebook §6.24, §7.7).
197 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
198 │ pub enum DieModifier {
199 │     #[default]
200 │     Zero,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 987) \ #github-link("omdurman-rules/src/lib.rs", 987)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L987")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeAttack]]]], [#raw("985 │ /// A melee attack: simultaneous, both sides roll on the Combat Results Table (§7.3, §7.7).
986 │ #[derive(Serialize, Deserialize, Clone, Debug)]
987 │ pub struct MeleeAttack {
988 │     pub attacker_player: Player,
989 │     pub attacker_hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 968) \ #github-link("omdurman-rules/src/lib.rs", 968)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L968")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::AngloEgyptianStandard]]]], [#raw("966 │     DervishStandard,
967 │     /// +1 to all Anglo-Egyptian melee rolls (§7.7).
968 │     AngloEgyptianStandard,
969 │     /// Inverted to -2 when Dervish units melee-attack across a trench into
970 │     /// an entrenched defender (§9.232).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 971) \ #github-link("omdurman-rules/src/lib.rs", 971)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L971")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::DervishVsTrenchedDefender]]]], [#raw("969 │     /// Inverted to -2 when Dervish units melee-attack across a trench into
970 │     /// an entrenched defender (§9.232).
971 │     DervishVsTrenchedDefender,
972 │ }
973 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 966) \ #github-link("omdurman-rules/src/lib.rs", 966)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L966")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::DervishStandard]]]], [#raw("964 │ pub enum MeleeModifier {
965 │     /// +2 to all Dervish melee rolls (§7.7).
966 │     DervishStandard,
967 │     /// +1 to all Anglo-Egyptian melee rolls (§7.7).
968 │     AngloEgyptianStandard,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 137) \ #github-link("omdurman-rules/src/lib.rs", 137)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L137")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementAllowance::halve]]]], [#raw("135 │ impl MovementAllowance {
136 │     /// Night movement allowance = halved (round down) (rulebook §8.1, §5.11).
137 │     pub fn halve(self) -> Self {
138 │         let v = self.value() / 2;
139 │         MovementAllowance::try_from(v).expect(\"halved value always a named variant\")", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 266) \ #github-link("omdurman-rules/src/lib.rs", 266)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L266")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DayNight]]]], [#raw("264 │ /// (rulebook §8.1).
265 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
266 │ pub enum DayNight {
267 │     Day,
268 │     Night,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1351) \ #github-link("omdurman-rules/src/lib.rs", 1351)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1351")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[effective_range_at_night]]]], [#raw("1349 │ /// Apply night-turn range halving (§8.1): \"all fire ranges are halved for
1350 │ /// both sides (round down, but range 1 stays range 1).\"
1351 │ pub fn effective_range_at_night(range: HexDistance) -> HexDistance {
1352 │     if range.0 <= 1 {
1353 │         range", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1361) \ #github-link("omdurman-rules/src/lib.rs", 1361)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1361")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[effective_movement_at_night]]]], [#raw("1359 │ /// Apply night-turn movement halving for Anglo-Egyptian units (§8.1): all
1360 │ /// Anglo-Egyptian movement allowances are halved (round down).
1361 │ pub fn effective_movement_at_night(
1362 │     allowance: MovementAllowance,
1363 │     player: Player,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[night_max_ranges]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[night_max_ranges_remaining]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_rifle_at_night_matches_rulebook_example]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[max_day_range_all_combos]]]
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
  [#vscode-link("omdurman-rules/src/effects.rs", 128) \ #github-link("omdurman-rules/src/effects.rs", 128)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L128")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishDesertion]]]], [#raw("126 │     /// the effect. The Khalifa, gunboats, artillery, and forts may not be
127 │     /// chosen.
128 │     DervishDesertion {
129 │         roll: DieRoll,
130 │         deserters: Vec<UnitId>,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 60) \ #github-link("omdurman-rules/src/turn_track.rs", 60)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L60")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishDesertion]]]], [#raw(" 58 │     None,
 59 │     /// Dervish desertion roll (§8.2) -- occurs on the first night turn.
 60 │     DervishDesertion,
 61 │     /// Dervish reinforcements are available.
 62 │     DervishReinforcements,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 57) \ #github-link("omdurman-rules/src/turn_track.rs", 57)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L57")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnEvent]]]], [#raw(" 55 │ /// Special events that occur on specific turns (rulebook §8.2, §9.112, §9.113).
 56 │ #[derive(Clone, Copy, PartialEq, Eq, Debug)]
 57 │ pub enum TurnEvent {
 58 │     None,
 59 │     /// Dervish desertion roll (§8.2) -- occurs on the first night turn.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[desertion_on_first_night]]]
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
#heading(level: 2, "§9.2 -- The Historical Scenario") <sect-9-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Historical Scenario

Players should note that the historical scenario is an exercise in futility for the Dervish player. It is, however, an interesting demonstration of the absolute imbecility of the Khalifa's generalship and vividly shows the superiority of entrenched firepower over traditional tribal arms in the colonial period.]]
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 8) \ #github-link("omdurman-rules/src/turn_track.rs", 8)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L8")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTime]]]], [#raw("  6 │ /// starts at one of these twelve times.
  7 │ #[derive(Clone, Copy, PartialEq, Eq, Debug)]
  8 │ pub enum GameTime {
  9 │     SixAM,
 10 │     EightAM,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 44) \ #github-link("omdurman-rules/src/turn_track.rs", 44)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L44")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnEntry]]]], [#raw(" 42 │ /// A single entry on the Turn Record Track (rulebook §9.12, §9.22).
 43 │ #[derive(Clone, Copy, PartialEq, Eq, Debug)]
 44 │ pub struct TurnEntry {
 45 │     /// 1-based turn number.
 46 │     pub turn: u8,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 92) \ #github-link("omdurman-rules/src/turn_track.rs", 92)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L92")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CAMPAIGN_TURN_TRACK]]]], [#raw(" 90 │ /// is turn 9, which carries the once-per-game Dervish Desertion Roll (§8.2) --
 91 │ /// the printed track prints \"Dervish Desertion Roll / NIGHT\" on that cell.
 92 │ pub const CAMPAIGN_TURN_TRACK: [TurnEntry; 22] = [
 93 │     // Row 1, left->right: Sept 1, 6 am -> 8 pm, then the first NIGHT.
 94 │     entry(1, SixAM, Day, TurnEvent::None),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 238) \ #github-link("omdurman-rules/src/turn_track.rs", 238)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L238")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnLabel]]]], [#raw("236 │ /// The track is a 9 × 3 grid with a snake layout.
237 │ #[derive(Debug, Clone, Copy, PartialEq, Eq)]
238 │ pub enum TurnLabel {
239 │     /// A cell that has no printed label (unused position in the 9×3 grid).
240 │     Blank,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 299) \ #github-link("omdurman-rules/src/turn_track.rs", 299)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L299")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[turn_marker_pixel]]]], [#raw("297 │ ///
298 │ /// Rows 0 and 1 use all 9 columns; row 2 uses only columns 0–3.
299 │ pub fn turn_marker_pixel(track: &omdurman_types::CampaignTurnTrack, turn: u8) -> (f32, f32) {
300 │     let cell_w = track.w / 9.0;
301 │     let cell_h = track.h / 3.0;", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[campaign_track_22_turns]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[desertion_on_first_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[campaign_track_label_and_day_night_agree]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[game_time_display_all_variants]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_label_display]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_label_out_of_range_is_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_marker_pixel_row_0_left_to_right]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_marker_pixel_row_1_right_to_left]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[turn_marker_pixel_rows_are_stacked]]]
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
  [#vscode-link("omdurman-rules/src/lib.rs", 1090) \ #github-link("omdurman-rules/src/lib.rs", 1090)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1090")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource]]]], [#raw("1088 │ /// the manual and the engine.
1089 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
1090 │ pub enum VpSource {
1091 │     // ----- Anglo-Egyptian player receives:
1092 │     /// Mahdi's Tomb control at conclusion of play (§9.14).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1116) \ #github-link("omdurman-rules/src/lib.rs", 1116)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1116")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource::points]]]], [#raw("1114 │ impl VpSource {
1115 │     /// VP awarded to `who_scores()` (rulebook §9.14).
1116 │     pub fn points(self) -> VictoryPoints {
1117 │         match self {
1118 │             VpSource::MahdisTomb => VictoryPoints(25),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1131) \ #github-link("omdurman-rules/src/lib.rs", 1131)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1131")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource::who_scores]]]], [#raw("1129 │ 
1130 │     /// Which player receives these victory points (rulebook §9.14).
1131 │     pub fn who_scores(self) -> Player {
1132 │         match self {
1133 │             VpSource::MahdisTomb", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1148) \ #github-link("omdurman-rules/src/lib.rs", 1148)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1148")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger]]]], [#raw("1146 │ /// Cumulative victory ledger for one scenario (rulebook §9.14).
1147 │ #[derive(Serialize, Deserialize, Clone, Debug, Default)]
1148 │ pub struct VictoryLedger {
1149 │     pub events: Vec<VpEvent>,
1150 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1154) \ #github-link("omdurman-rules/src/lib.rs", 1154)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1154")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpEvent]]]], [#raw("1152 │ /// A single victory-point scoring event (rulebook §9.14).
1153 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
1154 │ pub struct VpEvent {
1155 │     pub turn: GameTurnIndex,
1156 │     pub source: VpSource,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1161) \ #github-link("omdurman-rules/src/lib.rs", 1161)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1161")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger::total_for]]]], [#raw("1159 │ impl VictoryLedger {
1160 │     /// Total victory points earned by a given player (rulebook §9.14).
1161 │     pub fn total_for(&self, player: Player) -> VictoryPoints {
1162 │         VictoryPoints(
1163 │             self.events", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1173) \ #github-link("omdurman-rules/src/lib.rs", 1173)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1173")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger::superiority]]]], [#raw("1171 │     /// Net superiority: positive = Anglo-Egyptian ahead, negative = Dervish ahead
1172 │     /// (rulebook §9.14).
1173 │     pub fn superiority(&self) -> VictoryPoints {
1174 │         VictoryPoints(self.total_for(Player::AngloEgyptian).0 - self.total_for(Player::Dervish).0)
1175 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1191) \ #github-link("omdurman-rules/src/lib.rs", 1191)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1191")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CampaignVictoryLevel]]]], [#raw("1189 │ /// Campaign-game victory levels (§9.14).
1190 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
1191 │ pub enum CampaignVictoryLevel {
1192 │     Draw,
1193 │     Marginal(Player),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1200) \ #github-link("omdurman-rules/src/lib.rs", 1200)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1200")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CampaignVictoryLevel::from_superiority]]]], [#raw("1198 │ impl CampaignVictoryLevel {
1199 │     /// Assign a level from the net superiority (§9.14).
1200 │     pub fn from_superiority(s: VictoryPoints) -> Self {
1201 │         let net = s.0;
1202 │         // Positive -> Anglo-Egyptian thresholds: 15/30/50", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 3195) \ #github-link("omdurman-rules/src/effects.rs", 3195)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3195")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[score_elimination]]]], [#raw("3193 │ 
3194 │ /// Score victory points for eliminating a unit (rulebook §9.14).
3195 │ pub fn score_elimination(state: &mut GameState, unit_id: UnitId, _owner: Player) {
3196 │     if let Some(unit) = state.find_unit(unit_id) {
3197 │         let identity = unit.profile.identity;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 227) \ #github-link("omdurman-rules/src/lib.rs", 227)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L227")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryPoints]]]], [#raw("225 │ /// (rulebook §9.14).
226 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
227 │ pub struct VictoryPoints(pub i32);
228 │ 
229 │ /// One-based Game Turn index (1, 2, ... up to the scenario length) (rulebook §4).", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 129) \ #github-link("omdurman-rules/src/turn_track.rs", 129)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L129")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HISTORICAL_TURN_TRACK]]]], [#raw("127 │ 
128 │ /// Historical scenario track (§9.22 -- 4 turns, Sept 2 6:00 am -> 12:00 pm).
129 │ pub const HISTORICAL_TURN_TRACK: [TurnEntry; 4] = [
130 │     TurnEntry {
131 │         turn: 1,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 873) \ #github-link("omdurman-rules/src/lib.rs", 873)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L873")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("871 │     Terrain(i16),
872 │     /// -2 thorn-hedge defensive modifier (§9.231).
873 │     ZaribaThornHedge,
874 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
875 │     /// units (those Nile-side of the trench hexside).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 876) \ #github-link("omdurman-rules/src/lib.rs", 876)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L876")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrenchEntrenched]]]], [#raw("874 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
875 │     /// units (those Nile-side of the trench hexside).
876 │     ZaribaTrenchEntrenched,
877 │ }
878 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 212) \ #github-link("omdurman-types/src/lib.rs", 212)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L212")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("210 │     Crest,
211 │     /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
212 │     ZaribaThornHedge,
213 │     /// Historical-scenario trench segment of the Zariba (§9.232).
214 │     ZaribaTrench,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 214) \ #github-link("omdurman-types/src/lib.rs", 214)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L214")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrench]]]], [#raw("212 │     ZaribaThornHedge,
213 │     /// Historical-scenario trench segment of the Zariba (§9.232).
214 │     ZaribaTrench,
215 │     /// One of the two end hexsides of a Zariba trench segment that connect to
216 │     /// the Nile River (§9.233).  Units may only enter/leave the Zariba via", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 1230) \ #github-link("omdurman-rules/src/lib.rs", 1230)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1230")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HistoricalVictoryLevel]]]], [#raw("1228 │ /// draw\").
1229 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
1230 │ pub enum HistoricalVictoryLevel {
1231 │     Draw = 1,
1232 │     Marginal = 2,", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 169) \ #github-link("omdurman-rules/src/turn_track.rs", 169)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L169")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_TURN_TRACK]]]], [#raw("167 │ /// (the rulebook fixes none); only `day_night` is rule-bearing (night halves
168 │ /// Anglo-Egyptian movement and ranges and bars howitzer fire, §8.1).
169 │ pub const FALL_OF_KHARTOUM_TURN_TRACK: [TurnEntry; 8] = [
170 │     TurnEntry {
171 │         turn: 1,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 1271) \ #github-link("omdurman-rules/src/lib.rs", 1271)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1271")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FoKVictoryLevel]]]], [#raw("1269 │ /// negative) so the loss penalty is a simple shift toward the British end.
1270 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
1271 │ pub enum FoKVictoryLevel {
1272 │     DervishDecisive = -3,
1273 │     DervishTactical = -2,", block: true, lang: "rs")],
)
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/effects.rs", 120) \ #github-link("omdurman-rules/src/effects.rs", 120)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L120")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[PlaceReinforcements]]]], [#raw("118 │     // -- Reinforcement / placement -----------------------------------------
119 │     /// Place reinforcements onto the map (rulebook §9.112, §9.113).
120 │     PlaceReinforcements(Vec<UnitPlacement>),
121 │ 
122 │     // -- Scenario-specific -------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2678) \ #github-link("omdurman-rules/src/effects.rs", 2678)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2678")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_place_reinforcements]]]], [#raw("2676 │ 
2677 │ /// Place reinforcements onto the map (rulebook §9.112, §9.113).
2678 │ pub fn apply_place_reinforcements(
2679 │     state: &mut GameState,
2680 │     placements: &[UnitPlacement],", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 473) \ #github-link("omdurman-types/src/lib.rs", 473)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L473")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Location]]]], [#raw("471 │ /// Named map landmarks (rulebook mapsheet, §9.111, §9.113, §9.212 scenarios).
472 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
473 │ pub enum Location {
474 │     FortMakran,
475 │     NorthFort,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 525) \ #github-link("omdurman-types/src/lib.rs", 525)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L525")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[SetupLetter]]]], [#raw("523 │ /// Each letter marks a specific hex where a Dervish leader is placed.
524 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
525 │ pub enum SetupLetter {
526 │     Y,
527 │     K,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 619) \ #github-link("omdurman-types/src/lib.rs", 619)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L619")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Faction]]]], [#raw("617 │ /// Dervish units have a tribe; Anglo-Egyptian infantry have a brigade.
618 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
619 │ pub enum Faction {
620 │     Dervish { tribe: DervishTribe },
621 │     BritishEgyptian { brigade: Brigade },", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1629) \ #github-link("omdurman-rules/src/effects.rs", 1629)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1629")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[first_player]]]], [#raw("1627 │ 
1628 │ /// The player who moves first in a scenario (§4, §9.113, §9.212, §9.322).
1629 │ pub fn first_player(scenario: Scenario) -> Player {
1630 │     match scenario {
1631 │         Scenario::Campaign => Player::AngloEgyptian,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 629) \ #github-link("omdurman-rules/src/effects.rs", 629)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L629")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[in_deployment_zone]]]], [#raw("627 │     ///   plan / UI rather than this hex predicate. Documented, not silently
628 │     ///   dropped.
629 │     pub fn in_deployment_zone(&self, player: Player, hex: HexCoord) -> bool {
630 │         // No board attached -> permissive (unit tests, sandbox).
631 │         if self.board.terrain.is_empty() {", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.231 -- Thorn hedge hexsides") <sect-9-231>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 873) \ #github-link("omdurman-rules/src/lib.rs", 873)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L873")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("871 │     Terrain(i16),
872 │     /// -2 thorn-hedge defensive modifier (§9.231).
873 │     ZaribaThornHedge,
874 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
875 │     /// units (those Nile-side of the trench hexside).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 212) \ #github-link("omdurman-types/src/lib.rs", 212)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L212")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("210 │     Crest,
211 │     /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
212 │     ZaribaThornHedge,
213 │     /// Historical-scenario trench segment of the Zariba (§9.232).
214 │     ZaribaTrench,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 876) \ #github-link("omdurman-rules/src/lib.rs", 876)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L876")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrenchEntrenched]]]], [#raw("874 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
875 │     /// units (those Nile-side of the trench hexside).
876 │     ZaribaTrenchEntrenched,
877 │ }
878 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 214) \ #github-link("omdurman-types/src/lib.rs", 214)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L214")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrench]]]], [#raw("212 │     ZaribaThornHedge,
213 │     /// Historical-scenario trench segment of the Zariba (§9.232).
214 │     ZaribaTrench,
215 │     /// One of the two end hexsides of a Zariba trench segment that connect to
216 │     /// the Nile River (§9.233).  Units may only enter/leave the Zariba via", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 169) \ #github-link("omdurman-rules/src/turn_track.rs", 169)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L169")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_TURN_TRACK]]]], [#raw("167 │ /// (the rulebook fixes none); only `day_night` is rule-bearing (night halves
168 │ /// Anglo-Egyptian movement and ranges and bars howitzer fire, §8.1).
169 │ pub const FALL_OF_KHARTOUM_TURN_TRACK: [TurnEntry; 8] = [
170 │     TurnEntry {
171 │         turn: 1,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1957) \ #github-link("omdurman-rules/src/effects.rs", 1957)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1957")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[range_band_for]]]], [#raw("1955 │ /// in FALL OF KHARTOUM *both* players use the Dervish Range Effects Table
1956 │ /// (§9.343).
1957 │ pub fn range_band_for(
1958 │     scenario: Scenario,
1959 │     player: Player,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1182) \ #github-link("omdurman-rules/src/effects.rs", 1182)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1182")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_has_enemy_fort]]]], [#raw("1180 │     /// may neither occupy an enemy fort nor advance after combat into one
1181 │     /// (forts are never captured -- only destroyed, §6.62/§6.53/§7.6).
1182 │     pub fn hex_has_enemy_fort(&self, hex: HexCoord, mover: Player) -> bool {
1183 │         self.units.iter().any(|u| {
1184 │             u.position == hex", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1166) \ #github-link("omdurman-rules/src/effects.rs", 1166)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1166")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_nile_mouth_crossing]]]], [#raw("1164 │     /// must be named on the board, else this is `false` and the move falls
1165 │     /// through to the ordinary contiguous-Nile rules.
1166 │     pub fn is_nile_mouth_crossing(&self, from: HexCoord, to: HexCoord) -> bool {
1167 │         let white = self
1168 │             .board", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 489) \ #github-link("omdurman-types/src/lib.rs", 489)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L489")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Location::WhiteNileMouth]]]], [#raw("487 │     /// The off-board mouth of the White Nile branch (FALL OF KHARTOUM §9.345) --
488 │     /// a British gunboat may cross to the Blue Nile mouth for 6 upstream MP.
489 │     WhiteNileMouth,
490 │     /// The off-board mouth of the Blue Nile branch (FALL OF KHARTOUM §9.345).
491 │     BlueNileMouth,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1769) \ #github-link("omdurman-rules/src/effects.rs", 1769)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1769")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[check_gordon_palace]]]], [#raw("1767 │ /// after combat). Records the turn (which fixes the §9.35 victory level) and
1768 │ /// ends the game. A no-op outside FoK, or once GORDON is already gone.
1769 │ pub fn check_gordon_palace(state: &mut GameState) {
1770 │     if state.scenario != Scenario::FallOfKhartoum || state.gordon_eliminated_turn.is_some() {
1771 │         return;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 651) \ #github-link("omdurman-rules/src/lib.rs", 651)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L651")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitIdentity::is_gordon]]]], [#raw("649 │     /// Whether this is the GORDON leader unit (§9.32, §9.346) -- the immobile
650 │     /// palace defender whose elimination ends FALL OF KHARTOUM (§9.35).
651 │     pub fn is_gordon(&self) -> bool {
652 │         matches!(
653 │             self,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[gordon_survives_means_no_elimination]]]
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
  [#vscode-link("omdurman-rules/src/lib.rs", 326) \ #github-link("omdurman-rules/src/lib.rs", 326)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L326")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OptionalRule]]]], [#raw("324 │ /// two should be in play (rulebook §10).
325 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
326 │ pub enum OptionalRule {
327 │     RiverMines,
328 │     RiverChain,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 138) \ #github-link("omdurman-rules/src/effects.rs", 138)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L138")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RiverMine]]]], [#raw("136 │     // -- Optional rules ----------------------------------------------------
137 │     /// River mine resolution (rulebook §10.12).
138 │     RiverMine {
139 │         gunboat_id: UnitId,
140 │         hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2928) \ #github-link("omdurman-rules/src/effects.rs", 2928)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2928")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("2926 │ 
2927 │ /// Apply a river-mine resolution (rulebook §10.12).
2928 │ pub fn apply_river_mine(
2929 │     state: &mut GameState,
2930 │     gunboat_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1042) \ #github-link("omdurman-rules/src/lib.rs", 1042)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1042")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MineResult]]]], [#raw("1040 │ /// British gunboat enters a mined hex.
1041 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
1042 │ pub enum MineResult {
1043 │     /// Roll 1-4: no effect.
1044 │     NoEffect,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 138) \ #github-link("omdurman-rules/src/effects.rs", 138)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L138")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RiverMine]]]], [#raw("136 │     // -- Optional rules ----------------------------------------------------
137 │     /// River mine resolution (rulebook §10.12).
138 │     RiverMine {
139 │         gunboat_id: UnitId,
140 │         hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2928) \ #github-link("omdurman-rules/src/effects.rs", 2928)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2928")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("2926 │ 
2927 │ /// Apply a river-mine resolution (rulebook §10.12).
2928 │ pub fn apply_river_mine(
2929 │     state: &mut GameState,
2930 │     gunboat_id: UnitId,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 2928) \ #github-link("omdurman-rules/src/effects.rs", 2928)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2928")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("2926 │ 
2927 │ /// Apply a river-mine resolution (rulebook §10.12).
2928 │ pub fn apply_river_mine(
2929 │     state: &mut GameState,
2930 │     gunboat_id: UnitId,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 2928) \ #github-link("omdurman-rules/src/effects.rs", 2928)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2928")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("2926 │ 
2927 │ /// Apply a river-mine resolution (rulebook §10.12).
2928 │ pub fn apply_river_mine(
2929 │     state: &mut GameState,
2930 │     gunboat_id: UnitId,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 872) \ #github-link("omdurman-rules/src/effects.rs", 872)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L872")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_gunboat]]]], [#raw("870 │     /// upstream movement allowance is their maximum for that turn.\" Chained Nile
871 │     /// hexes stop the gunboat (§10.22).
872 │     pub fn can_move_gunboat(
873 │         &self,
874 │         unit_id: UnitId,", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/effects.rs", 2987) \ #github-link("omdurman-rules/src/effects.rs", 2987)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2987")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_sink_chain]]]], [#raw("2985 │ /// Sink the river chain (rulebook §10.23). Marks the placed chain cleared so it
2986 │ /// no longer stops gunboats (§10.22).
2987 │ pub fn apply_sink_chain(state: &mut GameState) -> Result<(), RuleError> {
2988 │     match state.chain.as_mut() {
2989 │         Some(chain) if !chain.sunk => {", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/lib.rs", 937) \ #github-link("omdurman-rules/src/lib.rs", 937)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L937")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CombatResult]]]], [#raw("935 │ /// * `--` -- no effect
936 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
937 │ pub enum CombatResult {
938 │     NoEffect,
939 │     Disrupt,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_combat_results_table_lowest_is_no_effect]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_combat_results_table_highest_is_eliminate_5]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_combat_results_table_progresses_with_roll]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[ae_combat_results_table_progresses_with_factor]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_factor_row_boundaries]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_factor_row_remaining_boundaries]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[fire_factor_row_index_sequential]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[crt_all_rows_monotone_non_decreasing]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[crt_eliminate_never_exceeds_5]]]
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
  [#text(weight: "bold", size: 9pt)[NileFlow]], [#link(<sect-5-11>)[§5.11], #link(<sect-5-24>)[§5.24]],
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
  [#text(weight: "bold", size: 9pt)[UnitFormKind]], [#link(<sect-2-3>)[§2.3]],
  [#text(weight: "bold", size: 9pt)[UnitKind]], [#link(<sect-7-4>)[§7.4]],
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
  [#text(weight: "bold", size: 9pt)[overlay_color]], [#link(<sect-5-52>)[§5.52]],
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

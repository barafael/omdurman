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

#let root = "C:/workspace/sources/omdurman"

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
#align(center, text(size: 10pt, "REMEMBER GORDON! – Rulebook ⇌ Implementation Mapping"))
#align(center, text(size: 9pt, fill: luma(120), "Generated from `docs/traceability.toml`"))
#v(2em)
#heading(level: 1, "Overview") <sect-overview>
#v(0.3em)
#table(
  columns: (1fr, 1fr, 1fr, 1fr),
  stroke: 0.4pt + luma(190),
  [*Implemented*], [*Descriptive*], [*Implicit*], [*Out-of-scope*],
  [#text(fill: green.darken(20%))[83]], [#text(fill: blue.darken(20%))[28]], [#text(fill: yellow.darken(30%))[4]], [9],
)
#v(0.3em)
#text(size: 9pt)[Total mappings: 124 · Total impl sites: 246]
#v(1em)
#outline(title: [Table of Contents])
#pagebreak()
#progress-bar(0, 3)
#heading(level: 1, "§1 – Introduction") <sect-1>
#heading(level: 2, "§1 – Introduction")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Introduction]]
#v(0.5em)
#heading(level: 2, "§1.1 – General Comments") <sect-1-1>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120))[manual page 1]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[General Comments

"REMEMBER GORDON!" — THE BATTLE OF OMDURMAN is a simulation of the final battle in Great Britain's two-year campaign to reassert her presence in the Sudan (1896–1898). Fought September 2nd, 1898, Omdurman finally broke the back of the fanatical Dervish rebellion and gained Britain a million square miles of desolate territory and two million impoverished subjects. With two players, one assumes the role of Herbert Kitchener, Sirdar (CIC) of the Anglo-Egyptian army; the other player becomes the Khalifa, Abdullah the Taiasha, absolute ruler of the Dervishes. The game is also suited for multi-player participation, with each player assuming command of one or more Dervish tribes or Anglo-Egyptian brigades.

While "REMEMBER GORDON!" — THE BATTLE OF OMDURMAN is not, strictly speaking, a beginner's game, the mechanics of play should be familiar to players of modest experience. It is suggested that the bonus game, FALL OF KHARTOUM, and the historical scenario be played first to familiarize players with the game system prior to embarking on the full campaign game.

The designer would also like to point out that English spelling of Arabic names, places, and words is a process of transliteration rather than translation. Spellings thus tend to vary widely according to the source, author, and date of publication.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#heading(level: 2, "§1.2 – Game Scale") <sect-1-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Game Scale

Each hexagon of the mapsheet represents approximately 400–440 yards of real terrain and each day turn is the equivalent of two hours of real time. Each counter of infantry and cavalry represents between 400 and 700 men, and each of the gunboats present at the battle has its own counter. The upper echelon of command is represented by individual leader counters for the Anglo-Egyptian force; and leaders plus their retinues for the Dervish army.]]
#v(0.5em)
#progress-bar(3, 7)
#heading(level: 1, "§2 – Game Components") <sect-2>
#heading(level: 2, "§2 – Game Components")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Game Components]]
#v(0.5em)
#heading(level: 2, "§2.1 – The Game Maps") <sect-2-1>
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
#heading(level: 2, "§2.2 – Play Aids") <sect-2-2>
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
#heading(level: 2, "§2.3 – The Units") <sect-2-3>
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
  [#vscode-link("omdurman-types/src/lib.rs", 862) \ #github-link("omdurman-types/src/lib.rs", 862)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L862")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind]]]], [#raw("860 │ /// `Some(UnitKind::Marker)` or `None`.
861 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
862 │ pub enum UnitKind {
863 │     /// Foot infantry (§2.3): fire / melee / movement.
864 │     Infantry {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 736) \ #github-link("omdurman-rules/src/lib.rs", 736)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L736")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitProfile]]]], [#raw("734 │ }
735 │ 
736 │ /// The printed combat profile of a single counter (rulebook §2.3, §6.11, §7.1,
737 │ /// §5.11, §5.24). Optional factors are `None` only where the rulebook leaves the
738 │ /// value off the counter (e.g. British leaders print only movement; gunboats", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 16) \ #github-link("omdurman-rules/src/lib.rs", 16)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L16")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeId]]]], [#raw(" 14 │ 
 15 │ use omdurman_types::{
 16 │     BrigadeId, BrigadeNationality, DayNight, DervishTribe, Faction, HexCoord, HexsideRef, Player,
 17 │     SetupLetter, UnitKind,
 18 │ };", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 677) \ #github-link("omdurman-types/src/lib.rs", 677)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L677")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[SpriteAnnotation]]]], [#raw("675 │ /// as an optional overlay over the compiled `sprite_data` fallback.
676 │ #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
677 │ pub struct SpriteAnnotation {
678 │     pub color: SpriteColor,
679 │     pub faction: Option<Faction>,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::british_army_row_zero_specials_classify_by_counter]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::egyptian_army_row_zero_specials_classify_by_counter]]]
#v(0.3em)
#heading(level: 2, "§2.4 – Game Parts Inventory") <sect-2-4>
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
#heading(level: 2, "§2.31 – Dervish weapon types") <sect-2-31>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 468) \ #github-link("omdurman-rules/src/lib.rs", 468)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L468")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("466 │ // 5) Unit kinds and weapons
467 │ // ---------------------------------------------------------------------------
468 │ 
469 │ /// Weapon class -- chooses which line of the Range Effects Table applies and
470 │ /// which special artillery rules (§6.6) are available. Spelled out as an", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit_profiles.rs", 510) \ #github-link("omdurman-rules/src/unit_profiles.rs", 510)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit_profiles.rs#L510")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_tribe]]]], [#raw("508 │ /// Resolve a Dervish tribal foot counter (§2.31): Jehadia, Danagla and
509 │ /// Isa Zachneih fire on the rifles line; every other tribe is spear-armed.
510 │ pub fn dervish_tribe(tribe: DervishTribe) -> Option<Classification> {
511 │     // §2.31: \"Jehadia and Danagla units fire on the 'rifles' line as does the
512 │     // Isa Zachneih unit. All other Dervish units (including leaders) are armed", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit_profiles.rs", 292) \ #github-link("omdurman-rules/src/unit_profiles.rs", 292)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit_profiles.rs#L292")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[khalifa_abdullah]]]], [#raw("290 │ ///     battle (§9.322). All three are interchangeable, so they share the
291 │ ///     `DervishArtillery` identity.
292 │ pub fn khalifa_abdullah(col: u32, row: u32) -> Option<Classification> {
293 │     let artillery = || {
294 │         Some(Classification {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::dervish_weapon_class_follows_the_rifles_line]]]
#v(0.3em)
#heading(level: 2, "§2.32 – Anglo-Egyptian weapon types") <sect-2-32>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 422) \ #github-link("omdurman-rules/src/lib.rs", 422)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L422")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId]]]], [#raw("420 │     /// Used only in FALL OF KHARTOUM (§9.32, §9.346).
421 │     Gordon,
422 │ }
423 │ 
424 │ /// Named British gunboat (rulebook §6.64). Five \"named\" gunboats have howitzer", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 441) \ #github-link("omdurman-rules/src/lib.rs", 441)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L441")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[NamedGunboat]]]], [#raw("439 │     pub fn has_howitzer(self) -> bool {
440 │         matches!(self, GunboatId::Named(_))
441 │     }
442 │ }
443 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 454) \ #github-link("omdurman-rules/src/lib.rs", 454)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L454")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OldGunboat]]]], [#raw("452 │ }
453 │ 
454 │ /// Old-style gunboat -- no howitzer fire (rulebook §2.32).
455 │ /// May fire only once per turn (Direct Fire subphase only); it lacks the
456 │ /// howitzer equipped by the five named gunboats and thus cannot participate", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 454) \ #github-link("omdurman-rules/src/lib.rs", 454)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L454")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId::Old]]]], [#raw("452 │ }
453 │ 
454 │ /// Old-style gunboat -- no howitzer fire (rulebook §2.32).
455 │ /// May fire only once per turn (Direct Fire subphase only); it lacks the
456 │ /// howitzer equipped by the five named gunboats and thus cannot participate", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 428) \ #github-link("omdurman-rules/src/lib.rs", 428)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L428")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId::DervishGunboat]]]], [#raw("426 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
427 │ pub enum GunboatId {
428 │     /// One of the five new-type named gunboats with howitzer capability.
429 │     Named(NamedGunboat),
430 │     /// An old-style gunboat -- no howitzer fire (§2.32).", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::old_gunboat_lacks_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::old_gunboat_rejected_from_howitzer_subphase]]]
#v(0.3em)
#progress-bar(0, 1)
#heading(level: 1, "§3 – Getting Started") <sect-3>
#heading(level: 2, "§3 – Getting Started")
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
#heading(level: 1, "§4 – Turn Sequence") <sect-4>
#heading(level: 2, "§4 – Turn Sequence")
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
  [#vscode-link("omdurman-rules/src/lib.rs", 246) \ #github-link("omdurman-rules/src/lib.rs", 246)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L246")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTurnIndex]]]], [#raw("244 │     pub fn value(self) -> i32 {
245 │         self.0
246 │     }
247 │ }
248 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 268) \ #github-link("omdurman-rules/src/lib.rs", 268)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L268")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Phase]]]], [#raw("266 │ /// The fine-grained phase within a player-turn (rulebook §4).
267 │ ///
268 │ /// Fire-combat phase is broken down so that the legality of every fire is
269 │ /// statically checkable: e.g. a howitzer fire can only resolve inside the
270 │ /// `MaximSecondAndHowitzer` sub-phase, defensive fire only in `DefensiveFire`,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 823) \ #github-link("omdurman-rules/src/effects.rs", 823)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L823")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState]]]], [#raw("821 │ /// All mutable state of a game in progress (rulebook §4).
822 │ #[derive(Serialize, Deserialize, Clone, Debug)]
823 │ pub struct GameState {
824 │     pub scenario: Scenario,
825 │     pub current_turn: GameTurnIndex,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 959) \ #github-link("omdurman-rules/src/effects.rs", 959)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L959")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState::new]]]], [#raw("957 │ impl GameState {
958 │     /// Create a fresh game state for a given scenario (rulebook §4).
959 │     pub fn new(scenario: Scenario) -> Self {
960 │         let first = scenario_turn(scenario, GameTurnIndex::new(1));
961 │         // First player to *move* per scenario: Campaign -- Anglo-Egyptian moves", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 50) \ #github-link("omdurman-rules/src/effects.rs", 50)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L50")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvancePhase]]]], [#raw(" 48 │     /// At end-of-turn, disrupted units recover and per-turn tracking is
 49 │     /// cleared.
 50 │     AdvancePhase,
 51 │ 
 52 │     // -- Movement ----------------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2745) \ #github-link("omdurman-rules/src/effects.rs", 2745)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2745")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[advance_phase]]]], [#raw("2743 │ 
2744 │ /// Advance the game state to the next phase (rulebook §4).
2745 │ pub fn advance_phase(state: &mut GameState) -> Result<(), RuleError> {
2746 │     let old_phase = state.phase;
2747 │     debug!(old_phase = ?old_phase, active_player = ?state.active_player, \"advance_phase\");", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2839) \ #github-link("omdurman-rules/src/effects.rs", 2839)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2839")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[end_player_turn]]]], [#raw("2837 │ 
2838 │ /// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
2839 │ pub fn end_player_turn(state: &mut GameState) -> Result<(), RuleError> {
2840 │     debug!(
2841 │         old_player = ?state.active_player,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 57) \ #github-link("omdurman-rules/src/lib.rs", 57)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L57")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTurnIndex::value]]]], [#raw(" 55 │ 
 56 │         impl $name {
 57 │             /// Every variant, in declaration order. Generated so exhaustive
 58 │             /// callers (tests, Kani proofs) pick up new variants automatically
 59 │             /// instead of silently skipping them.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 951) \ #github-link("omdurman-rules/src/effects.rs", 951)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L951")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[PendingMelee]]]], [#raw("949 │ /// resolution after the reaction window is deterministic and host-ordered (rulebook §7.5).
950 │ #[derive(Serialize, Deserialize, Clone, Debug)]
951 │ pub struct PendingMelee {
952 │     pub attack: MeleeAttack,
953 │     pub attacker_roll: DieRoll,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::both_ready_auto_advances_out_of_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fire_combat_wrong_phase_rejected]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::new_game_starts_in_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::scenario_turn_dispatches_correctly]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::turn_advances_through_phases]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::game_turn_marker_cell_returns_none]]]
#v(0.3em)
#progress-bar(18, 23)
#heading(level: 1, "§5 – Movement Phase") <sect-5>
#heading(level: 2, "§5 – Movement Phase (general)")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Movement Phase]]
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::disrupted_unit_cannot_fire]]]
#v(0.3em)
#heading(level: 2, "§5.1 – General Rules") <sect-5-1>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[General Rules]]
#v(0.5em)
#heading(level: 2, "§5.2 – Movement Restrictions") <sect-5-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Movement Restrictions]]
#v(0.5em)
#heading(level: 2, "§5.3 – Constructing the Zariba") <sect-5-3>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 770) \ #github-link("omdurman-rules/src/lib.rs", 770)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L770")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[constructing_zariba]]]], [#raw("768 │     /// offensively or defensively; may not melee; are turned face up at the
769 │     /// end of the owning player's turn.\"
770 │     pub disrupted: bool,
771 │     /// `Some(gunboat)` after a \"Friendlies\" unit loads onto a gunboat (§5.21).
772 │     pub loaded_on: Option<UnitId>,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 202) \ #github-link("omdurman-rules/src/effects.rs", 202)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L202")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ConstructZariba]]]], [#raw("200 │ 
201 │     /// Begin constructing a Zariba hexside (rulebook §5.3).
202 │     ConstructZariba {
203 │         unit_ids: Vec<UnitId>,
204 │         hexside: HexsideRef,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4571) \ #github-link("omdurman-rules/src/effects.rs", 4571)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4571")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_construct_zariba]]]], [#raw("4569 │ 
4570 │ /// Mark a set of units as constructing a Zariba hexside (rulebook §5.3).
4571 │ pub fn apply_construct_zariba(
4572 │     state: &mut GameState,
4573 │     unit_ids: &[UnitId],", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::construct_zariba_marks_builders_and_records_hexside]]]
#v(0.3em)
#heading(level: 2, "§5.4 – Zones of Control") <sect-5-4>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Zones of Control]]
#v(0.5em)
#heading(level: 2, "§5.5 – Stacking") <sect-5-5>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Stacking]]
#v(0.5em)
#heading(level: 2, "§5.11 – Movement allowances printed on units") <sect-5-11>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 125) \ #github-link("omdurman-rules/src/lib.rs", 125)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L125")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementAllowance]]]], [#raw("123 │ }
124 │ 
125 │ value_enum! {
126 │     /// A unit's land movement allowance or a terrain-entry's movement cost
127 │     /// (rulebook §5.11). Every possible value from the annotated counter set", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 747) \ #github-link("omdurman-rules/src/lib.rs", 747)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L747")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitMovement]]]], [#raw("745 │     pub fire: Option<FireFactor>,
746 │     pub melee: Option<MeleeFactor>,
747 │     pub movement: UnitMovement,
748 │ }
749 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 300) \ #github-link("omdurman-types/src/lib.rs", 300)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L300")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDirection]]]], [#raw("298 │ /// (`+q`, `+q+r`, `+r`, `-q`, `-q-r`, `-r` for pointy-top hexes) (rulebook §5.11, §5.24).
299 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
300 │ pub enum HexDirection {
301 │     #[default]
302 │     East = 0,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 167) \ #github-link("omdurman-rules/src/lib.rs", 167)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L167")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementPoints]]]], [#raw("165 │     }
166 │ }
167 │ 
168 │ /// Movement points spent or remaining within a single phase (rulebook §5).
169 │ #[derive(", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 21) \ #github-link("omdurman-rules/src/terrain_chart.rs", 21)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/terrain_chart.rs#L21")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[terrain_effects_chart]]]], [#raw(" 19 │ ///
 20 │ /// Source: printed Terrain Effects Chart on the mapsheet.
 21 │ pub fn terrain_effects_chart(terrain: Terrain) -> TerrainEntry {
 22 │     match terrain {
 23 │         Terrain::Clear { .. } => TerrainEntry {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 77) \ #github-link("omdurman-rules/src/terrain_chart.rs", 77)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/terrain_chart.rs#L77")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[movement_cost]]]], [#raw(" 75 │ /// Convenience: get the movement cost for a terrain type (rulebook §5.11, Terrain Effects Chart).
 76 │ /// Returns `None` for impassable terrain (Nile).
 77 │ pub fn movement_cost(terrain: Terrain) -> Option<MovementAllowance> {
 78 │     terrain_effects_chart(terrain).movement_cost
 79 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 85) \ #github-link("omdurman-rules/src/terrain_chart.rs", 85)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/terrain_chart.rs#L85")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[movement_cost_with_road]]]], [#raw(" 83 │ /// underlying terrain; without a road it's the terrain's own cost. The road is
 84 │ /// a movement overlay only -- combat/LOS still use the underlying terrain.
 85 │ pub fn movement_cost_with_road(terrain: Terrain, road: bool) -> Option<MovementAllowance> {
 86 │     if road {
 87 │         Some(MovementAllowance::One)", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 496) \ #github-link("omdurman-types/src/lib.rs", 496)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L496")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::has_road]]]], [#raw("494 │ 
495 │     /// Whether this hex has any road touching it.
496 │     pub fn has_road(self) -> bool {
497 │         !matches!(self.road(), Road::None)
498 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 433) \ #github-link("omdurman-types/src/lib.rs", 433)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L433")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::passable_by_land]]]], [#raw("431 │ 
432 │     /// Whether this terrain may be entered by land units (rulebook §5.11).
433 │     pub fn passable_by_land(self) -> bool {
434 │         !self.is_nile()
435 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 501) \ #github-link("omdurman-types/src/lib.rs", 501)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L501")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::is_crossroad]]]], [#raw("499 │ 
500 │     /// Whether roads converge at this hex's centre.
501 │     pub fn is_crossroad(self) -> bool {
502 │         matches!(self.road(), Road::Crossroad)
503 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-hexmap/src/map.rs", 17) \ #github-link("omdurman-hexmap/src/map.rs", 17)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-hexmap/src/map.rs#L17")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameMap::roads]]]], [#raw(" 15 │     pub hexes: HashMap<HexCoord, HexData>,
 16 │     pub hexsides: HashMap<HexsideRef, HexsideKind>,
 17 │     pub roads: HashSet<HexsideRef>,
 18 │     pub excluded: HashSet<HexCoord>,
 19 │     pub overlay: OverlayParams,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::clear_terrain_no_bonus]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::nile_is_impassable]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::rough_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::swamp_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::hilltop_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::huts_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::movement_cost_convenience_matches_chart]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::movement_cost_with_road_always_one]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::land_unit_may_not_enter_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::movement_cost_without_road_matches_terrain]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::movement_cost_for_uses_terrain]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::movement_cost_for_road_costs_one]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::road_gives_crossroad]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::terrain_movement_costs_in_bounds]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::terrain_chart_road_always_costs_one]]]
#v(0.3em)
#heading(level: 2, "§5.12 – Move up to allowance, hex by hex (cumulative MP cap)") <sect-5-12>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[A player may move as many or as few of his units as desired during each movement phase, limited only by the units' movement allowance, the terrain costs paid in moving from hex to hex, and enemy zones of control (see #link(<sect-5-4>)[5.4]).]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-4>)[§5.4]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1483) \ #github-link("omdurman-rules/src/effects.rs", 1483)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1483")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit]]]], [#raw("1481 │     /// the same `RuleError` the `MoveUnit` effect would on rejection. Lets the
1482 │     /// UI gate input without mutating or duplicating the rules.
1483 │     pub fn can_move_unit(&self, unit_id: UnitId, cost: MovementPoints) -> Result<(), RuleError> {
1484 │         self.can_move_unit_to(unit_id, None, cost)
1485 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2174) \ #github-link("omdurman-rules/src/effects.rs", 2174)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2174")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[mp_spent]]]], [#raw("2172 │ 
2173 │     /// Movement points `unit_id` has already spent this turn (§5.11/§5.12).
2174 │     pub fn mp_spent(&self, unit_id: UnitId) -> i16 {
2175 │         self.mp_spent_this_turn.get(&unit_id).copied().unwrap_or(0)
2176 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::cumulative_move_cost_may_not_exceed_allowance]]]
#v(0.3em)
#heading(level: 2, "§5.13 – No MP accumulation between turns") <sect-5-13>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 2839) \ #github-link("omdurman-rules/src/effects.rs", 2839)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2839")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[end_player_turn]]]], [#raw("2837 │ 
2838 │ /// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
2839 │ pub fn end_player_turn(state: &mut GameState) -> Result<(), RuleError> {
2840 │     debug!(
2841 │         old_player = ?state.active_player,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::unused_movement_points_do_not_carry_over]]]
#v(0.3em)
#heading(level: 2, "§5.21 – Friendlies transport via gunboat") <sect-5-21>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 584) \ #github-link("omdurman-rules/src/lib.rs", 584)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L584")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_friendlies]]]], [#raw("582 │                 Player::AngloEgyptian => Faction::BritishEgyptian { brigade: None },
583 │             },
584 │         }
585 │     }
586 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4215) \ #github-link("omdurman-rules/src/effects.rs", 4215)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4215")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[friendlies_transport_offer]]]], [#raw("4213 │     /// regardless of selection. Pairs with [`GameEffect::FriendliesTransport`]
4214 │     /// so the UI can offer exactly the action the engine would accept.
4215 │     pub fn friendlies_transport_offer(&self, selected: Option<UnitId>) -> Option<FriendliesAction> {
4216 │         match self.friendlies_transport {
4217 │             None => {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 767) \ #github-link("omdurman-rules/src/lib.rs", 767)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L767")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[loaded_on]]]], [#raw("765 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
766 │ pub struct UnitState {
767 │     /// Reference table: \"Disrupted units: no ZOC; may not move; may not fire
768 │     /// offensively or defensively; may not melee; are turned face up at the
769 │     /// end of the owning player's turn.\"", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 229) \ #github-link("omdurman-rules/src/effects.rs", 229)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L229")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FriendliesTransport]]]], [#raw("227 │ 
228 │     /// Load/disembark the \"Friendlies\" brigade via gunboat (rulebook §5.21).
229 │     FriendliesTransport(crate::FriendliesAction),
230 │ 
231 │     // -- Optional rules ----------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4998) \ #github-link("omdurman-rules/src/effects.rs", 4998)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4998")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_friendlies_transport]]]], [#raw("4996 │ ///     unit is freed (a disembarking `MoveUnit` should follow, costed by
4997 │ ///     terrain).
4998 │ pub fn apply_friendlies_transport(
4999 │     state: &mut GameState,
5000 │     action: FriendliesAction,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1005) \ #github-link("omdurman-rules/src/lib.rs", 1005)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1005")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FriendliesAction]]]], [#raw("1003 │ /// The action payload for `GameEffect::FriendliesTransport` -- what the
1004 │ /// player wants to do with the Friendlies unit this turn (§5.21).
1005 │ ///
1006 │ /// The manual does not cap how many Friendlies may load onto a single gunboat
1007 │ /// (a hex has six neighbours, so multiple units can be adjacent).  The code", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1025) \ #github-link("omdurman-rules/src/lib.rs", 1025)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1025")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TransportState]]]], [#raw("1023 │     Disembark { unit: UnitId, gunboat: UnitId },
1024 │ }
1025 │ 
1026 │ /// The transport state stored on `GameState` (§5.21). Modelled as a state
1027 │ /// machine so the engine can enforce that disembarking can only happen on the", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::friendlies_transport_offer_load_requires_prerequisites]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::friendlies_transport_offer_follows_state_machine]]]
#v(0.3em)
#heading(level: 2, "§5.22 – Land units may never enter a Nile River hex") <sect-5-22>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1500) \ #github-link("omdurman-rules/src/effects.rs", 1500)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1500")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw("1498 │     ///
1499 │     /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
1500 │     pub fn can_move_unit_to(
1501 │         &self,
1502 │         unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1171) \ #github-link("omdurman-rules/src/effects.rs", 1171)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1171")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[in_deployment_zone]]]], [#raw("1169 │     ///   plan / UI rather than this hex predicate. Documented, not silently
1170 │     ///   dropped.
1171 │     pub fn in_deployment_zone(&self, player: Player, hex: HexCoord, is_boat: bool) -> bool {
1172 │         // No board attached -> permissive (unit tests, unbound session).
1173 │         if self.board.terrain.is_empty() {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::campaign_deployment_is_boat_land_exclusive]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_ae_gunboat_deploys_only_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_ae_land_unit_rejected_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::deploy_via_real_sprite_resolution_matches_engine]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_dervish_land_unit_rejected_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::retreat_before_melee_may_not_land_on_nile]]]
#v(0.3em)
#heading(level: 2, "§5.23 – Walled city entry restrictions") <sect-5-23>
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
  [#vscode-link("omdurman-types/src/lib.rs", 150) \ #github-link("omdurman-types/src/lib.rs", 150)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L150")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideRef]]]], [#raw("148 │ /// data by [`HexsideRef`].
149 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
150 │ pub struct HexsideRef {
151 │     pub a: HexCoord,
152 │     pub b: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 185) \ #github-link("omdurman-types/src/lib.rs", 185)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L185")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideKind]]]], [#raw("183 │     strum::EnumIter,
184 │ )]
185 │ pub enum HexsideKind {
186 │     /// City wall (Khartoum, walled city of Omdurman). Blocks LOS, blocks
187 │     /// movement except across gates/breaches (§5.23), blocks ZOC into the city", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 256) \ #github-link("omdurman-types/src/lib.rs", 256)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L256")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_movement]]]], [#raw("254 │     /// `omdurman-rules`). The trench *end* variants are therefore intentionally
255 │     /// not blocking.
256 │     pub fn blocks_movement(self) -> bool {
257 │         matches!(
258 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 610) \ #github-link("omdurman-rules/src/lib.rs", 610)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L610")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_enter_walled_city]]]], [#raw("608 │         )
609 │     }
610 │ 
611 │     /// Whether this unit may enter the walled portion of Omdurman (§5.23).
612 │     /// Dervish: only the Khalifa unit, the three artillery units, and the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/board.rs", 294) \ #github-link("omdurman-rules/src/board.rs", 294)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/board.rs#L294")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_walled_city]]]], [#raw("292 │     /// are its seeds). The set is derived once from the board data, replacing
293 │     /// the older \"at least two of six hexsides are Wall/Gate/Breach\" heuristic.
294 │     pub fn is_walled_city(&self, hex: HexCoord) -> bool {
295 │         // Membership in the precomputed enclosed area (see `walled_city`).
296 │         // Palace/Tomb hexes are always part of it (they are the seeds).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 409) \ #github-link("omdurman-rules/src/effects.rs", 409)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L409")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WalledCityEntry]]]], [#raw("407 │ 
408 │     #[error(\"unit {0:?} is not eligible to enter the walled city of Omdurman at {1:?} (§5.23)\")]
409 │     WalledCityEntry(UnitId, HexCoord),
410 │ 
411 │     #[error(\"movement cost {cost:?} exceeds allowance {allowance:?}\")]", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::hexside_ref_is_order_independent]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_move_rejects_wall_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_move_allows_gate_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::walled_city_entry_allows_khalifa]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::walled_city_entry_rejects_unauthorized_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::walled_city_entry_rejects_ae_gunboat]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::walled_city_entry_not_enforced_for_fok]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::board_data::campaign_walled_city_is_enclosed_by_walls]]]
#v(0.3em)
#heading(level: 2, "§5.24 – Gunboat upstream/downstream movement") <sect-5-24>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 520) \ #github-link("omdurman-rules/src/lib.rs", 520)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L520")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatMovement]]]], [#raw("518 │ }
519 │ 
520 │ /// Gunboats have two movement allowances -- the smaller upstream and the
521 │ /// larger downstream (§5.24).  Combined movement is permitted but as soon as
522 │ /// the gunboat moves one hex upstream its upstream allowance caps the rest of", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 947) \ #github-link("omdurman-types/src/lib.rs", 947)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L947")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_boat]]]], [#raw("945 │ 
946 │     /// Gunboats use the split upstream/downstream movement allowance (§5.24).
947 │     pub fn is_boat(self) -> bool {
948 │         matches!(self, UnitKind::Gunboat { .. })
949 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::boat_annotation_yields_split_gunboat_movement]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::gunboat_upstream_cap_is_sticky_across_moves]]]
#v(0.3em)
#heading(level: 2, "§5.25 – Dervish forts may not move") <sect-5-25>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 751) \ #github-link("omdurman-rules/src/lib.rs", 751)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L751")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Immobile]]]], [#raw("749 │ 
750 │ /// Movement allowance -- uniform for land units, split for gunboats (rulebook §5.11, §5.24, §5.25).
751 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
752 │ pub enum UnitMovement {
753 │     Land(MovementAllowance),", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 904) \ #github-link("omdurman-types/src/lib.rs", 904)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L904")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind::Fort]]]], [#raw("902 │     /// Permanent emplacement (§6.54): fire (artillery) / melee (defensive).
903 │     /// May not move once placed (§5.25).
904 │     Fort { fire: i32, melee: i32 },
905 │     /// Dervish leader (§6.51): fire / melee / movement. May melee attack (§7.4).
906 │     DervishLeader {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 751) \ #github-link("omdurman-rules/src/lib.rs", 751)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L751")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitMovement::Immobile]]]], [#raw("749 │ 
750 │ /// Movement allowance -- uniform for land units, split for gunboats (rulebook §5.11, §5.24, §5.25).
751 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
752 │ pub enum UnitMovement {
753 │     Land(MovementAllowance),", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::forts_are_never_advance_eligible]]]
#v(0.3em)
#heading(level: 2, "§5.26 – Units stop on entering enemy ZOC") <sect-5-26>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Units must stop their movement immediately upon entering an enemy zone of control (see #link(<sect-5-4>)[5.4]).]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-4>)[§5.4]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1500) \ #github-link("omdurman-rules/src/effects.rs", 1500)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1500")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw("1498 │     ///
1499 │     /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
1500 │     pub fn can_move_unit_to(
1501 │         &self,
1502 │         unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2435) \ #github-link("omdurman-rules/src/effects.rs", 2435)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2435")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("2433 │     /// does not extend into or out of a Nile hex. With no board loaded these
2434 │     /// reduce to the plain adjacency rule.
2435 │     pub fn hex_in_enemy_zoc(
2436 │         &self,
2437 │         hex: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::unit_entering_enemy_zoc_may_move_no_further_that_turn]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_transit_check_uses_the_actual_path]]]
#v(0.3em)
#heading(level: 2, "§5.41 – All units except AE leaders exert ZOC") <sect-5-41>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 806) \ #github-link("omdurman-rules/src/lib.rs", 806)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L806")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("804 │ // ---------------------------------------------------------------------------
805 │ // 8) Zones of control, stacking, brigade integrity
806 │ // ---------------------------------------------------------------------------
807 │ 
808 │ /// Why a unit can or cannot exert/receive ZOC into a given adjacent hex.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2401) \ #github-link("omdurman-rules/src/effects.rs", 2401)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2401")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[unit_projects_zoc]]]], [#raw("2399 │     /// §5.44) need the game map, which the engine does not hold; the app layers
2400 │     /// those on top. This is the position/kind/disruption core of the rule.
2401 │     pub fn unit_projects_zoc(
2402 │         &self,
2403 │         unit: &UnitPlacement,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2476) \ #github-link("omdurman-rules/src/effects.rs", 2476)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2476")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[zoc_hexes]]]], [#raw("2474 │     /// ZOC covers a given hex; this function returns *which* hexes a
2475 │     /// specific unit covers.
2476 │     pub fn zoc_hexes(
2477 │         &self,
2478 │         unit: &UnitPlacement,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_hexes_empty_for_anglo_egyptian_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_hexes_normal_unit_projects_six_adjacent_minus_exclusions]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_hexes_empty_for_disrupted_unit]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_hexes_matches_hex_in_enemy_zoc]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_hexes_excludes_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_hexes_excludes_khor]]]
#v(0.3em)
#heading(level: 2, "§5.42 – No MP cost to enter/leave enemy ZOC") <sect-5-42>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[There is no movement point cost to enter or leave an enemy ZOC.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1666) \ #github-link("omdurman-rules/src/effects.rs", 1666)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1666")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[movement_cost_for]]]], [#raw("1664 │     ///
1665 │     /// §5.42: entering or leaving an enemy ZOC adds no MP cost.
1666 │     pub fn movement_cost_for(
1667 │         &self,
1668 │         unit: &UnitPlacement,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::entering_enemy_zoc_costs_no_extra_mp]]]
#v(0.3em)
#heading(level: 2, "§5.43 – Units stop when entering enemy ZOC") <sect-5-43>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1500) \ #github-link("omdurman-rules/src/effects.rs", 1500)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1500")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw("1498 │     ///
1499 │     /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
1500 │     pub fn can_move_unit_to(
1501 │         &self,
1502 │         unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2435) \ #github-link("omdurman-rules/src/effects.rs", 2435)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2435")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("2433 │     /// does not extend into or out of a Nile hex. With no board loaded these
2434 │     /// reduce to the plain adjacency rule.
2435 │     pub fn hex_in_enemy_zoc(
2436 │         &self,
2437 │         hex: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::unit_entering_enemy_zoc_may_move_no_further_that_turn]]]
#v(0.3em)
#heading(level: 2, "§5.44 – ZOC limitations (walls, khor, fort, Nile, Zariba)") <sect-5-44>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 806) \ #github-link("omdurman-rules/src/lib.rs", 806)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L806")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("804 │ // ---------------------------------------------------------------------------
805 │ // 8) Zones of control, stacking, brigade integrity
806 │ // ---------------------------------------------------------------------------
807 │ 
808 │ /// Why a unit can or cannot exert/receive ZOC into a given adjacent hex.", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 190) \ #github-link("omdurman-types/src/lib.rs", 190)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L190")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Wall]]]], [#raw("188 │     /// (§5.44), blocks melee (§7.2), blocks advance-after-combat (§6.82).
189 │     #[default]
190 │     Wall,
191 │     /// Gate hexside in a wall. ZOC extends *out of* the walled city through
192 │     /// gates but not into it (§5.44). Melee may be made through a gate (§7.2).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 199) \ #github-link("omdurman-types/src/lib.rs", 199)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L199")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Khor]]]], [#raw("197 │     /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
198 │     /// combat may not cross (§6.82).
199 │     Khor,
200 │     /// Crest line. Blocks LOS unless the firer is on the higher side
201 │     /// (§6.3 note 7).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 819) \ #github-link("omdurman-rules/src/lib.rs", 819)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L819")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason::Zariba]]]], [#raw("817 │     /// Forts project ZOC out of, but not into, an empty fort (§5.44, §6.54).
818 │     Fort,
819 │     /// Walled-city ZOC: extends out through walls and gates but not in,
820 │     /// across a breach in both directions (§5.44).
821 │     WalledCity,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2401) \ #github-link("omdurman-rules/src/effects.rs", 2401)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2401")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[unit_projects_zoc]]]], [#raw("2399 │     /// §5.44) need the game map, which the engine does not hold; the app layers
2400 │     /// those on top. This is the position/kind/disruption core of the rule.
2401 │     pub fn unit_projects_zoc(
2402 │         &self,
2403 │         unit: &UnitPlacement,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 273) \ #github-link("omdurman-types/src/lib.rs", 273)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L273")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideKind::blocks_zoc]]]], [#raw("271 │     /// cannot express; those are left to the caller. This predicate captures the
272 │     /// symmetric \"does not extend across\" cases.
273 │     pub fn blocks_zoc(self) -> bool {
274 │         matches!(
275 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2435) \ #github-link("omdurman-rules/src/effects.rs", 2435)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2435")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("2433 │     /// does not extend into or out of a Nile hex. With no board loaded these
2434 │     /// reduce to the plain adjacency rule.
2435 │     pub fn hex_in_enemy_zoc(
2436 │         &self,
2437 │         hex: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_hexes_excludes_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_hexes_excludes_khor]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zoc_hexes_normal_unit_projects_six_adjacent_minus_exclusions]]]
#v(0.3em)
#heading(level: 2, "§5.51 – Stacking limit (4 units + leaders, gunboats isolated)") <sect-5-51>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 828) \ #github-link("omdurman-rules/src/lib.rs", 828)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L828")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OverLimit]]]], [#raw("826 │ 
827 │ /// Errors returned when a candidate stack would violate stacking rules.
828 │ #[derive(thiserror::Error, Clone, Copy, PartialEq, Eq, Debug)]
829 │ pub enum StackingError {
830 │     /// \"No more than four units may occupy a hex\" (§5.51), excluding leaders", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 831) \ #github-link("omdurman-rules/src/lib.rs", 831)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L831")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatStack]]]], [#raw("829 │ pub enum StackingError {
830 │     /// \"No more than four units may occupy a hex\" (§5.51), excluding leaders
831 │     /// and the gunboat exception.
832 │     #[error(\"hex stack exceeds the four-unit limit\")]
833 │     OverLimit,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2230) \ #github-link("omdurman-rules/src/effects.rs", 2230)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2230")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[check_stacking]]]], [#raw("2228 │     /// * §5.52 -- units of different Dervish tribes may not stack together.
2229 │     /// * §5.53 -- a Dervish leader may stack only with units of its command.
2230 │     pub fn check_stacking(
2231 │         &self,
2232 │         mover: &UnitPlacement,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::stacking_over_limit_rejected]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mid_move_stacking_allows_pass_through]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mid_move_stacking_rejects_over_limit_destination]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::validate_stacking_invariants_clean_state]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::validate_stacking_invariants_catches_stacking_violation]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::validate_stacking_invariants_allows_leaders_stacking]]]
#v(0.3em)
#heading(level: 2, "§5.52 – Different Dervish tribes may not stack together") <sect-5-52>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 834) \ #github-link("omdurman-rules/src/lib.rs", 834)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L834")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishTribeMix]]]], [#raw("832 │     #[error(\"hex stack exceeds the four-unit limit\")]
833 │     OverLimit,
834 │     /// \"Gunboats may not stack with any other unit\" (§5.51, exception §5.21).
835 │     #[error(\"gunboats may not stack with non-gunboat units\")]
836 │     GunboatStack,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::green_sections_are_mulazmin_tribal_units]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::deploy_rejects_dervish_tribe_mix]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::validate_stacking_invariants_clean_state]]]
#v(0.3em)
#heading(level: 2, "§5.53 – Leader stacking with command colour only") <sect-5-53>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 838) \ #github-link("omdurman-rules/src/lib.rs", 838)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L838")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishLeaderCommandMismatch]]]], [#raw("836 │     GunboatStack,
837 │     /// \"Units of different Dervish tribes may not stack together\" (§5.52).
838 │     #[error(\"Dervish units of different tribes may not stack\")]
839 │     DervishTribeMix,
840 │     /// \"If Dervish leaders elect to stack, they may only stack with units of", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::dervish_leader_stacks_only_with_command_colour]]]
#v(0.3em)
#heading(level: 2, "§5.54 – Anglo-Egyptian Brigade Integrity") <sect-5-54>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 844) \ #github-link("omdurman-rules/src/lib.rs", 844)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L844")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeIntegrity]]]], [#raw("842 │     #[error(\"Dervish leader may only stack with units of their own command\")]
843 │     DervishLeaderCommandMismatch,
844 │ }
845 │ 
846 │ /// Brigade-integrity status of a stack (§5.54). Carries the brigade if the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 709) \ #github-link("omdurman-rules/src/lib.rs", 709)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L709")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[brigade_integrity]]]], [#raw("707 │ /// Whether a set of firing units forms a brigade with integrity (§5.54): all
708 │ /// four distinct battalions (1-4) of one Anglo-Egyptian brigade present. Used
709 │ /// to grant the +1 brigade-integrity direct-fire modifier when they all fire
710 │ /// at the same hex.
711 │ ///", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 844) \ #github-link("omdurman-rules/src/lib.rs", 844)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L844")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireModifier::BrigadeIntegrity]]]], [#raw("842 │     #[error(\"Dervish leader may only stack with units of their own command\")]
843 │     DervishLeaderCommandMismatch,
844 │ }
845 │ 
846 │ /// Brigade-integrity status of a stack (§5.54). Carries the brigade if the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 323) \ #github-link("omdurman-rules/src/lib.rs", 323)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L323")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BattalionOrdinal]]]], [#raw("321 │ // 4) Unit identity -- tribes, brigades, named leaders, classes
322 │ // ---------------------------------------------------------------------------
323 │ 
324 │ value_enum! {
325 │     /// Battalion ordinal within a brigade. Four battalions form one brigade and", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 1012) \ #github-link("omdurman-types/src/lib.rs", 1012)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L1012")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeId]]]], [#raw("1010 │ /// same field for uniform handling.
1011 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
1012 │ pub struct BrigadeId {
1013 │     pub number: u8,
1014 │     pub nationality: BrigadeNationality,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::brigade_designation_ignored_for_non_infantry]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::printed_brigade_designation_overrides_column]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::tribe_stats_come_from_annotation]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::section_owner_anglo_egyptian_sections]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::section_owner_green_sections_are_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::brigade_integrity_four_battalions_returns_integrated]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::ae_infantry_fourth_battalion_from_col_3]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::brigade_integrity_empty_slice]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::brigade_integrity_friendlies_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::section_owner_dervish_sections]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::brigade_integrity_three_battalions_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::unit_identity_brigade_and_battalion_accessors]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::ae_infantry_brigade_number_three_from_col_7]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::brigade_integrity_non_infantry_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::brigade_integrity_mixed_brigades_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::ae_infantry_third_battalion_from_col_2]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::brigade_integrity_modifier_is_engine_derived]]]
#v(0.3em)
#progress-bar(21, 31)
#heading(level: 1, "§6 – Fire Combat Phase") <sect-6>
#heading(level: 2, "§6 – Fire Combat Phase")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Fire Combat Phase]]
#v(0.5em)
#heading(level: 2, "§6.1 – General Rules") <sect-6-1>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[General Rules]]
#v(0.5em)
#heading(level: 2, "§6.2 – How To Have Fire Combat") <sect-6-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[How To Have Fire Combat]]
#v(0.5em)
#heading(level: 2, "§6.3 – Line of Sight Table") <sect-6-3>
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
  [#vscode-link("omdurman-rules/src/los_table.rs", 57) \ #github-link("omdurman-rules/src/los_table.rs", 57)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L57")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosLevel]]]], [#raw(" 55 │     serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug,
 56 │ )]
 57 │ pub enum LosLevel {
 58 │     Ground,
 59 │     Rough,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 69) \ #github-link("omdurman-rules/src/los_table.rs", 69)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L69")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosFeature]]]], [#raw(" 67 │ /// authored RON spellings.
 68 │ #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
 69 │ pub enum LosFeature {
 70 │     /// A hex containing units (gunboats/forts excluded per note a).
 71 │     Units,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 91) \ #github-link("omdurman-rules/src/los_table.rs", 91)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L91")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosCondition]]]], [#raw(" 89 │ /// A positional condition from the LOS table Detail footnotes.
 90 │ #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
 91 │ pub enum LosCondition {
 92 │     /// (1) Blocks only if the ray passes through more than two such features.
 93 │     MoreThanTwo,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 170) \ #github-link("omdurman-rules/src/los_table.rs", 170)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L170")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[los_level]]]], [#raw("168 │ ///
169 │ /// For all other units, the level is derived from the terrain at `hex`.
170 │ pub fn los_level_for_unit(
171 │     kind: UnitKind,
172 │     hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 170) \ #github-link("omdurman-rules/src/los_table.rs", 170)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L170")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[los_level_for_unit]]]], [#raw("168 │ ///
169 │ /// For all other units, the level is derived from the terrain at `hex`.
170 │ pub fn los_level_for_unit(
171 │     kind: UnitKind,
172 │     hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 209) \ #github-link("omdurman-rules/src/los_table.rs", 209)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L209")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocking_rules]]]], [#raw("207 │ /// of its conditions are satisfied (AND semantics); an empty conditions list
208 │ /// means the feature always blocks.
209 │ pub fn blocking_rules(firer: LosLevel, target: LosLevel) -> &'static [BlockingRule] {
210 │     let table = crate::tables_data::los_table_data();
211 │     match table.cells.get(&(firer, target)) {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 295) \ #github-link("omdurman-rules/src/los_table.rs", 295)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L295")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_los]]]], [#raw("293 │ /// `unit_level_at` closure returns the LOS level of blocking units
294 │ /// (non-gunboat, non-fort per note a) in an intervening hex, or `None`.
295 │ pub fn has_los(
296 │     board: &crate::board::BoardInfo,
297 │     from: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 228) \ #github-link("omdurman-types/src/lib.rs", 228)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L228")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideKind::blocks_los]]]], [#raw("226 │     /// (LOS table conditions 2–4, 7) and note (e) are handled by the engine
227 │     /// in `omdurman_rules::los_table`, not by this predicate.
228 │     pub fn blocks_los(self) -> bool {
229 │         matches!(self, HexsideKind::Wall | HexsideKind::Crest)
230 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 228) \ #github-link("omdurman-types/src/lib.rs", 228)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L228")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::blocks_los]]]], [#raw("226 │     /// (LOS table conditions 2–4, 7) and note (e) are handled by the engine
227 │     /// in `omdurman_rules::los_table`, not by this predicate.
228 │     pub fn blocks_los(self) -> bool {
229 │         matches!(self, HexsideKind::Wall | HexsideKind::Crest)
230 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 450) \ #github-link("omdurman-types/src/lib.rs", 450)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L450")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::is_los_trees]]]], [#raw("448 │     /// (§6.3 note 1). Retained for compatibility; the full LOS engine
449 │     /// checks `Terrain::Trees` directly.
450 │     pub fn is_los_trees(self) -> bool {
451 │         matches!(self, Terrain::Trees { .. })
452 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::line_between_forms_a_connected_ray]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_mapping]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::blocking_rules_all_cells_covered]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_empty_board_is_clear]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_adjacent_clear]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_howitzer_bypasses]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_wall_hexside_blocks_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_gate_hexside_passes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_breach_hexside_passes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_rough_intervening_blocks_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_two_tree_hexes_pass_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_three_tree_hexes_block_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_two_hut_hexes_pass_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_three_hut_hexes_block_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_hilltop_to_hilltop_clear_no_units]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_hilltop_to_hilltop_blocked_by_hilltop_unit]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_hilltop_to_hilltop_not_blocked_by_ground_unit]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_rough_to_rough_unit_at_lower_level_passes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_rough_to_rough_unit_at_same_level_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_rough_to_rough_hilltop_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_ground_to_hilltop_intervening_hilltop_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_building_blocks_like_huts_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_two_building_hexes_pass_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_for_unit_gunboat_is_rough]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_for_unit_fort_is_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_for_unit_walled_city_adj_wall_is_rough]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::gunboat_firer_uses_rough_row_not_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_reflexive_all_terrains]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_reflexive_hilltop]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_symmetric_ground_to_ground_no_units]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_howitzer_always_has_los]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_howitzer_same_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_blocking_rules_match_reference_table]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_ground_to_ground_features_block_as_expected]]]
#v(0.3em)
#heading(level: 2, "§6.4 – Fire Combat Sequence") <sect-6-4>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Fire Combat Sequence

The sequence of fire combat resolution is the same for both defensive and offensive fire combat. During the Dervish player turn the Anglo-Egyptian player executes #link(<sect-6-41>)[6.41] AND #link(<sect-6-42>)[6.42] as defensive fire, after which the Dervish player executes #link(<sect-6-41>)[6.41] as offensive fire. During the Anglo-Egyptian player turn the Dervish player executes #link(<sect-6-41>)[6.41] as defensive fire, after which the Anglo-Egyptian player executes #link(<sect-6-41>)[6.41] AND #link(<sect-6-42>)[6.42] as offensive fire.

\*\*#link(<sect-6-41>)[6.41]) Direct Fire Subphase (Dervish and Anglo-Egyptian players):\*\*
The firing player must first allocate all of his fire attacks, combining his units' direct fire combat factors in any manner he wishes. After all fire has been allocated, the firing player then resolves his attacks in any order he wishes.

\*\*#link(<sect-6-42>)[6.42]) Maxim Second Fire and Howitzer Fire Subphase (Anglo-Egyptian player only):\*\*
Anglo-Egyptian named gunboats may now fire their artillery factor as howitzer fire (see #link(<sect-6-64>)[6.64]) and all Maxim guns may fire a second time. Once again, first allocate all fires, then resolve combat in any order desired. Howitzer fire may be combined with Maxim fire, but only if the howitzer fire impacts in the intended hex (see #link(<sect-6-64>)[6.64]). If any Maxim guns did not fire during the Direct Fire Subphase (#link(<sect-6-41>)[6.41]), they may still only fire once in the Maxim and Howitzer Subphase (#link(<sect-6-42>)[6.42]). Units firing in this subphase may fire at enemy units fired at in Direct Fire Subphase.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-6-41>)[§6.41], #link(<sect-6-42>)[§6.42], #link(<sect-6-64>)[§6.64]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 296) \ #github-link("omdurman-rules/src/lib.rs", 296)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L296")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireSubPhase]]]], [#raw("294 │             Phase::OffensiveFire(_) => \"Offensive Fire\",
295 │             Phase::Melee => \"Melee\",
296 │         }
297 │     }
298 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 105) \ #github-link("omdurman-rules/src/effects.rs", 105)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L105")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireCombat]]]], [#raw("103 │     /// - Firers marked as fired; target hex marked as fired-at.
104 │     /// - Victory points awarded for eliminations.
105 │     FireCombat { attack: FireAttack, roll: DieRoll },
106 │ 
107 │     /// Resolve a howitzer bombardment (two rolls: CRT + impact scatter)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::turn_advances_through_phases]]]
#v(0.3em)
#heading(level: 2, "§6.5 – Special Unit Capabilities") <sect-6-5>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Unit Capabilities

\*\*#link(<sect-6-51>)[6.51]) Leader Units:\*\*
Dervish leader units have a fire factor, a melee factor, and a movement factor. They may thus attack, melee, and be eliminated like any other combat unit. Their special benefit is that they stack free.

Anglo-Egyptian leaders have a movement factor only. They are eliminated if a) they are alone in a hex when a Dervish unit occupies or passes through that hex, or b) if all of the combat units a leader is stacked with are eliminated in fire combat or melee. The special function of Anglo-Egyptian leaders is that at least one must survive to occupy the Mahdi's tomb hex if it is to be taken from the Dervish player (see #link(<sect-9-14>)[9.14]).

\*\*#link(<sect-6-52>)[6.52]) Anglo-Egyptian "Friendlies" Brigade:\*\*
These units represent native volunteers in the Anglo-Egyptian army. They fire rifles on the Dervish Range Effects Table and melee with the Dervish melee modifier. They may not enter the walled city of Omdurman (see #link(<sect-5-23>)[5.23]). They may be transferred to the west bank (see #link(<sect-5-21>)[5.21]).

\*\*#link(<sect-6-53>)[6.53]) Anglo-Egyptian Royal Engineers (Royal Eng. 5-3-8):\*\*
In addition to normal combat and melee capabilities, this unit may breach a wall hexside or destroy a fort. The procedure is as follows: The Royal Engineers must move adjacent to a fort or a wall hexside and end their movement adjacent. They may neither fire offensively nor melee attack in the ensuing combat phase. If the Royal Engineers remain adjacent to their target and undisrupted at the end of the Anglo-Egyptian player turn, the target is destroyed. Remove a destroyed fort or place a breach marker adjacent to a breached wall hexside. See #link(<sect-6-62>)[6.62] and #link(<sect-6-63>)[6.63] for the effects on adjacent enemy units. The Royal Engineers may perform demolitions while stacked with other Anglo-Egyptian units.

\*\*#link(<sect-6-54>)[6.54]) Forts:\*\*
The artillery factor of a fort may be fired normally by the owning player, even if it is not stacked with a friendly unit. The melee value of a fort is defensive only, i.e. forts may not melee attack. The −3 defensive value is deducted from the die roll of enemy fire attacks on friendly units stacked inside the fort. Players may not occupy an enemy fort nor advance after combat into an unoccupied enemy fort. There is no additional movement point cost to enter or leave a friendly fort. Forts may be destroyed by: a) artillery fire (see #link(<sect-6-62>)[6.62]), b) infantry melee attack (see #link(<sect-7-6>)[7.6]), or c) the Royal Engineers (see #link(<sect-6-53>)[6.53]). Forts have a ZOC even if unoccupied (see #link(<sect-5-44>)[5.44]).]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-21>)[§5.21], #link(<sect-5-23>)[§5.23], #link(<sect-5-44>)[§5.44], #link(<sect-6-51>)[§6.51], #link(<sect-6-52>)[§6.52], #link(<sect-6-53>)[§6.53], #link(<sect-6-54>)[§6.54], #link(<sect-6-62>)[§6.62], #link(<sect-6-63>)[§6.63], #link(<sect-7-6>)[§7.6], #link(<sect-9-14>)[§9.14]]
#v(0.3em)
#heading(level: 2, "§6.6 – Special Artillery Capabilities") <sect-6-6>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Artillery Capabilities]]
#v(0.5em)
#heading(level: 2, "§6.7 – Defensive Fire") <sect-6-7>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Defensive Fire

In Defensive Fire phase, all of the non-moving player's units may fire at any of the moving player's units in range, within the limitations imposed by the rules of combat (see #link(<sect-6-1>)[6.1] to #link(<sect-6-6>)[6.6]). There is no advance after combat as a result of defensive fires.]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-6-1>)[§6.1], #link(<sect-6-6>)[§6.6]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 4084) \ #github-link("omdurman-rules/src/effects.rs", 4084)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4084")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("4082 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
4083 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
4084 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
4085 │         let unit = self.unit_or_err(unit_id)?;
4086 │         // §6.7: there is no advance after combat as a result of defensive fire.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::no_advance_after_defensive_fire]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::defensive_fire_opens_no_advance_window]]]
#v(0.3em)
#heading(level: 2, "§6.8 – Offensive Fire") <sect-6-8>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Offensive Fire]]
#v(0.5em)
#heading(level: 2, "§6.11 – Fire combat factor printed on units") <sect-6-11>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 80) \ #github-link("omdurman-rules/src/lib.rs", 80)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L80")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireFactor]]]], [#raw(" 78 │     };
 79 │ }
 80 │ 
 81 │ value_enum! {
 82 │     /// A unit's fire-combat factor as printed on the counter (rulebook §6.11).", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::fire_factor_sum_to_row]]]
#v(0.3em)
#heading(level: 2, "§6.12 – Fire combat is always voluntary") <sect-6-12>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Fire combat is always voluntary. A unit is never required to fire at enemy units merely because they are in range or adjacent.]]
#v(0.5em)
#heading(level: 2, "§6.13 – Fire factor is unitary (may not be divided)") <sect-6-13>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[If a unit elects to fire, its fire combat factor at an enemy unit, that fire combat factor is unitary. A unit's fire combat factor may not be divided up to fire at enemy units on different hexes.]]
#v(0.5em)
#heading(level: 2, "§6.14 – Players may combine fire factors into one attack") <sect-6-14>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Players may combine fire during fire combat phase, i.e. they may fire at an enemy-occupied hex with as many friendly units as may legally do so, combining all of their fire combat factors into one attack. Note that in any given fire combat phase, however, a combat unit may only fire once and may only be fired at once (exceptions: Maxim guns and gunboats — see #link(<sect-6-4>)[6.4]).]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-6-4>)[§6.4]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 94) \ #github-link("omdurman-rules/src/lib.rs", 94)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L94")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[sum_to_row]]]], [#raw(" 92 │         Nine = 9,
 93 │         Ten = 10,
 94 │     }
 95 │ }
 96 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::unit_may_only_be_fired_at_once_per_phase]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::gunboat_and_maxim_may_be_fired_at_repeatedly]]]
#v(0.3em)
#heading(level: 2, "§6.15 – May divide a stack to fire at different hexes") <sect-6-15>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Players may also divide a stack of units in order to fire at different enemy-occupied hexes. Anglo-Egyptian infantry units having brigade integrity, however, do not receive their +1 direct fire modifier unless they all fire at the same enemy-occupied hex (see #link(<sect-5-54>)[5.54]).]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-54>)[§5.54]]
#v(0.3em)
#heading(level: 2, "§6.16 – Halving fire strength rounds down, minimum 1") <sect-6-16>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 487) \ #github-link("omdurman-rules/src/lib.rs", 487)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L487")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RangeBand]]]], [#raw("485 │     /// No howitzer fire allowed at night (§8.1, §6.64).
486 │     Howitzer,
487 │ }
488 │ 
489 │ /// A range band on the Range Effects Table -- how the printed fire factor is", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::halving_rounds_down_and_never_below_one]]]
#v(0.3em)
#heading(level: 2, "§6.21 – First check LOS before firing") <sect-6-21>
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
  [#vscode-link("omdurman-rules/src/los_table.rs", 209) \ #github-link("omdurman-rules/src/los_table.rs", 209)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L209")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocking_rules]]]], [#raw("207 │ /// of its conditions are satisfied (AND semantics); an empty conditions list
208 │ /// means the feature always blocks.
209 │ pub fn blocking_rules(firer: LosLevel, target: LosLevel) -> &'static [BlockingRule] {
210 │     let table = crate::tables_data::los_table_data();
211 │     match table.cells.get(&(firer, target)) {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 295) \ #github-link("omdurman-rules/src/los_table.rs", 295)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L295")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_los]]]], [#raw("293 │ /// `unit_level_at` closure returns the LOS level of blocking units
294 │ /// (non-gunboat, non-fort per note a) in an intervening hex, or `None`.
295 │ pub fn has_los(
296 │     board: &crate::board::BoardInfo,
297 │     from: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_fire_at_rejects_blocked_los]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_fire_at_allows_clear_los]]]
#v(0.3em)
#heading(level: 2, "§6.22 – Consult Range Effects Table") <sect-6-22>
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
  [#vscode-link("omdurman-rules/src/range_effects.rs", 35) \ #github-link("omdurman-rules/src/range_effects.rs", 35)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/range_effects.rs#L35")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ae_range_effects]]]], [#raw(" 33 │ /// Look up the range band for an Anglo-Egyptian weapon (§6.22, §6.24).
 34 │ /// Distances > 10 are out of range for all weapons.
 35 │ pub fn ae_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
 36 │     band_at(faction_rows(true), weapon, distance)
 37 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/range_effects.rs", 41) \ #github-link("omdurman-rules/src/range_effects.rs", 41)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/range_effects.rs#L41")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_range_effects]]]], [#raw(" 39 │ /// Look up the range band for a Dervish weapon (§6.22).
 40 │ /// Distances > 10 are out of range for all weapons.
 41 │ pub fn dervish_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
 42 │     band_at(faction_rows(false), weapon, distance)
 43 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 487) \ #github-link("omdurman-rules/src/lib.rs", 487)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L487")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RangeBand]]]], [#raw("485 │     /// No howitzer fire allowed at night (§8.1, §6.64).
486 │     Howitzer,
487 │ }
488 │ 
489 │ /// A range band on the Range Effects Table -- how the printed fire factor is", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 181) \ #github-link("omdurman-rules/src/lib.rs", 181)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L181")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDistance]]]], [#raw("179 │         self.0
180 │     }
181 │ }
182 │ 
183 │ /// A distance measured in hexes (range to target, length of a retreat, ...)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::adjacency_iff_distance_one]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_rifles_doubled_at_range_1]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_rifles_halved_at_range_4]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_howitzer_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_rifles_shorter_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::melee_only_range_1]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_artillery_full]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_maxims_match_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_distance_over_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_howitzer_halved_4_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_maxims_and_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_melee]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_distance_over_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fire_combat_eliminates_target]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::max_day_range_all_combos]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_ae_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_ae_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_ae_maxims]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_ae_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_dervish_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_dervish_maxims_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_dervish_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_dervish_spears]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_fire_at_gates_phase_range_and_player]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mixed_attack_bands_per_firer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_monotone_non_increasing]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_howitzer_has_minimum_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_monotone_non_increasing]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_first_range_max_effect_last_range_oor]]]
#v(0.3em)
#heading(level: 2, "§6.23 – Terrain defensive modifier") <sect-6-23>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 865) \ #github-link("omdurman-rules/src/lib.rs", 865)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L865")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireModifier::Terrain]]]], [#raw("863 │     /// +1 to all Anglo-Egyptian *direct* fire (§6.24).
864 │     AngloEgyptianDirectFire,
865 │     /// +1 brigade integrity, applied only if all four battalions fire at
866 │     /// the same enemy-occupied hex (§5.54, §6.24).
867 │     BrigadeIntegrity,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::lib::fire_modifier_keeps_roll_legal]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::clear_terrain_no_bonus]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::building_gives_minus_3]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::palm_grove_gives_minus_2]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::rough_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::swamp_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::hilltop_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::huts_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::defense_modifier_convenience_matches_chart]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::terrain_movement_costs_in_bounds]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::terrain_defense_modifier_non_positive]]]
#v(0.3em)
#heading(level: 2, "§6.24 – Anglo-Egyptian direct fire accuracy bonus and brigade integrity") <sect-6-24>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 859) \ #github-link("omdurman-rules/src/lib.rs", 859)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L859")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AngloEgyptianDirectFire]]]], [#raw("857 │ 
858 │ /// Every distinct die-roll modifier the rulebook recognises during a fire
859 │ /// attack. Encoding each as a variant means the engine cannot silently
860 │ /// double-apply a bonus and can audit any combat after the fact.
861 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 844) \ #github-link("omdurman-rules/src/lib.rs", 844)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L844")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeIntegrity]]]], [#raw("842 │     #[error(\"Dervish leader may only stack with units of their own command\")]
843 │     DervishLeaderCommandMismatch,
844 │ }
845 │ 
846 │ /// Brigade-integrity status of a stack (§5.54). Carries the brigade if the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 959) \ #github-link("omdurman-rules/src/lib.rs", 959)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L959")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireModifier::die_modifier]]]], [#raw("957 │     /// Inverted to -2 when Dervish units melee-attack across a trench into
958 │     /// an entrenched defender (§9.232).
959 │     DervishVsTrenchedDefender,
960 │ }
961 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::lib::die_roll_apply_modifier_is_total]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::brigade_integrity_modifier_is_engine_derived]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fire_modifiers_are_engine_derived_and_mismatches_rejected]]]
#v(0.3em)
#heading(level: 2, "§6.41 – Direct Fire Subphase") <sect-6-41>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 298) \ #github-link("omdurman-rules/src/lib.rs", 298)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L298")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DirectFire]]]], [#raw("296 │         }
297 │     }
298 │ }
299 │ 
300 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fire_combat_eliminates_target]]]
#v(0.3em)
#heading(level: 2, "§6.42 – Maxim Second Fire and Howitzer Fire Subphase") <sect-6-42>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 300) \ #github-link("omdurman-rules/src/lib.rs", 300)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L300")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MaximSecondAndHowitzer]]]], [#raw("298 │ }
299 │ 
300 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
301 │ pub enum FireSubPhase {
302 │     /// Direct fire (§6.41). Both sides participate in this sub-phase.", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 965) \ #github-link("omdurman-types/src/lib.rs", 965)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L965")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[fires_twice]]]], [#raw("963 │     /// Maxim guns fire twice per turn -- once in the Direct Fire Subphase and
964 │     /// again in the Maxim Second Fire Subphase (rulebook §6.42).
965 │     pub fn fires_twice(self) -> bool {
966 │         matches!(self, UnitKind::Maxim { .. })
967 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_on_target_7_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_scatters_below_7]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::advance_window_bridges_fire_subphase_and_closes_at_melee]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fired_at_tracker_resets_at_maxim_subphase]]]
#v(0.3em)
#heading(level: 2, "§6.51 – Leader Units") <sect-6-51>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 411) \ #github-link("omdurman-rules/src/lib.rs", 411)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L411")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BritishLeader]]]], [#raw("409 │         SetupLetter::O => DervishLeader::OsmanDigna,
410 │     }
411 │ }
412 │ 
413 │ /// Named Anglo-Egyptian leader (§6.51, §9.113). Movement factor only; needed", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 912) \ #github-link("omdurman-types/src/lib.rs", 912)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L912")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BritishLeader]]]], [#raw("910 │     },
911 │     /// Anglo-Egyptian leader (§6.51): movement only.
912 │     BritishLeader { movement: i32 },
913 │     /// Wall-breach marker placed by artillery fire (§6.63). Not a combat unit.
914 │     Breech,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 953) \ #github-link("omdurman-types/src/lib.rs", 953)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L953")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_combat_factors]]]], [#raw("951 │     /// British leaders print a movement factor only (§6.51); other kinds carry
952 │     /// fire and/or melee factors. Markers carry no stats.
953 │     pub fn has_combat_factors(self) -> bool {
954 │         !matches!(
955 │             self,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::zero_factor_is_none_not_zero]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::kitchener_block_resolves_leaders_friendlies_camel_and_sudanese]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::dervish_leader_sections_resolve_leader_and_retinue_per_cell]]]
#v(0.3em)
#heading(level: 2, "§6.52 – Anglo-Egyptian Friendlies Brigade") <sect-6-52>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 584) \ #github-link("omdurman-rules/src/lib.rs", 584)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L584")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_friendlies]]]], [#raw("582 │                 Player::AngloEgyptian => Faction::BritishEgyptian { brigade: None },
583 │             },
584 │         }
585 │     }
586 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 984) \ #github-link("omdurman-types/src/lib.rs", 984)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L984")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Friendlies]]]], [#raw("982 │     /// Native volunteer brigade -- the Shaggyeh (§6.52). Do not receive
983 │     /// brigade integrity (§5.54 enumerates only British/Egyptian/Sudanese).
984 │     Friendlies,
985 │ }
986 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::friendlies_counters_score_by_bank_not_as_leaders]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::friendlies_validate_and_resolve_on_dervish_table]]]
#v(0.3em)
#heading(level: 2, "§6.53 – Royal Engineers demolition") <sect-6-53>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 555) \ #github-link("omdurman-rules/src/lib.rs", 555)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L555")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RoyalEngineers]]]], [#raw("553 │     AngloEgyptianCamelCorps,
554 │     AngloEgyptianArtillery,
555 │     AngloEgyptianMaxim,
556 │     AngloEgyptianGunboat(GunboatId),
557 │     AngloEgyptianLeader(BritishLeader),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 773) \ #github-link("omdurman-rules/src/lib.rs", 773)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L773")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[demolishing]]]], [#raw("771 │     /// `Some(gunboat)` after a \"Friendlies\" unit loads onto a gunboat (§5.21).
772 │     pub loaded_on: Option<UnitId>,
773 │     /// Set while the unit is building Zariba hexsides -- neither offensive
774 │     /// fire nor melee allowed that turn (§5.3).
775 │     pub constructing_zariba: bool,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 208) \ #github-link("omdurman-rules/src/effects.rs", 208)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L208")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Demolition]]]], [#raw("206 │ 
207 │     /// Royal Engineers demolition (rulebook §6.53).
208 │     Demolition {
209 │         unit_id: UnitId,
210 │         target: DemolitionTarget,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4590) \ #github-link("omdurman-rules/src/effects.rs", 4590)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4590")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_demolition]]]], [#raw("4588 │ /// resolution happens at end of turn via [`apply_resolve_demolition`], which
4589 │ /// checks the engineer is still adjacent and undisrupted.
4590 │ pub fn apply_demolition(
4591 │     state: &mut GameState,
4592 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 989) \ #github-link("omdurman-rules/src/lib.rs", 989)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L989")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DemolitionTarget]]]], [#raw("987 │ // ---------------------------------------------------------------------------
988 │ 
989 │ /// The Royal Engineers' two demolition targets (§6.53). The Engineers spend
990 │ /// the entire turn adjacent to the target (no offensive fire or melee that
991 │ /// turn) and the target is removed at end-of-turn unless the Engineers were", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4180) \ #github-link("omdurman-rules/src/effects.rs", 4180)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4180")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[demolition_targets]]]], [#raw("4178 │     /// the rules would accept. Empty when the unit doesn't exist or has no
4179 │     /// adjacent target.
4180 │     pub fn demolition_targets(&self, unit_id: UnitId) -> Vec<DemolitionTarget> {
4181 │         let Ok(unit) = self.unit_or_err(unit_id) else {
4182 │             return Vec::new();", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::demolition_cancelled_when_engineer_disrupted]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::demolition_cancelled_when_engineer_moved_away]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::demolition_targets_finds_adjacent_fort_and_wall]]]
#v(0.3em)
#heading(level: 2, "§6.54 – Forts") <sect-6-54>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 806) \ #github-link("omdurman-rules/src/lib.rs", 806)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L806")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("804 │ // ---------------------------------------------------------------------------
805 │ // 8) Zones of control, stacking, brigade integrity
806 │ // ---------------------------------------------------------------------------
807 │ 
808 │ /// Why a unit can or cannot exert/receive ZOC into a given adjacent hex.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 813) \ #github-link("omdurman-rules/src/lib.rs", 813)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L813")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Fort]]]], [#raw("811 │ pub enum ZocReason {
812 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
813 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.
814 │     Normal,
815 │     /// Gunboats project ZOC only against enemy gunboats (§5.41).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 761) \ #github-link("omdurman-rules/src/lib.rs", 761)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L761")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitState]]]], [#raw("759 │ /// Volatile per-turn state of a unit -- disrupted, loaded onto a gunboat,
760 │ /// constructing the Zariba, demolishing a target, etc. (rulebook §5, §6).
761 │ ///
762 │ /// Multiple state flags can be in effect at once (e.g. a unit may be both
763 │ /// loaded and disrupted), so `UnitState` is a struct of orthogonal fields", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 904) \ #github-link("omdurman-rules/src/lib.rs", 904)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L904")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireAttack]]]], [#raw("902 │     MaximSecondFire,
903 │ }
904 │ 
905 │ /// A fire attack as the rules engine sees it: who fires, at what hex, in
906 │ /// what sub-phase, with which kind of fire, with what total factor and what", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 919) \ #github-link("omdurman-rules/src/lib.rs", 919)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L919")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireAttack::net_modifier]]]], [#raw("917 │     /// per-unit at resolution time).
918 │     pub factor_row: FireFactorRow,
919 │     pub modifiers: Vec<FireModifier>,
920 │ }
921 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::retreat_before_melee_may_not_land_on_enemy_fort]]]
#v(0.3em)
#heading(level: 2, "§6.61 – Only artillery may fire at gunboats; 3+ to sink") <sect-6-61>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 468) \ #github-link("omdurman-rules/src/lib.rs", 468)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L468")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("466 │ // 5) Unit kinds and weapons
467 │ // ---------------------------------------------------------------------------
468 │ 
469 │ /// Weapon class -- chooses which line of the Range Effects Table applies and
470 │ /// which special artillery rules (§6.6) are available. Spelled out as an", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::rifles_may_not_sink_a_gunboat]]]
#v(0.3em)
#heading(level: 2, "§6.62 – Only artillery may fire at forts; 2+ to destroy") <sect-6-62>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 468) \ #github-link("omdurman-rules/src/lib.rs", 468)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L468")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("466 │ // 5) Unit kinds and weapons
467 │ // ---------------------------------------------------------------------------
468 │ 
469 │ /// Weapon class -- chooses which line of the Range Effects Table applies and
470 │ /// which special artillery rules (§6.6) are available. Spelled out as an", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::artillery_destroys_fort_on_two_or_better_only]]]
#v(0.3em)
#heading(level: 2, "§6.63 – Only artillery may breach wall hexsides; 2+ required") <sect-6-63>
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
  [#vscode-link("omdurman-types/src/lib.rs", 196) \ #github-link("omdurman-types/src/lib.rs", 196)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L196")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Breach]]]], [#raw("194 │     /// Breach in a wall (artillery/§6.63 or Royal Engineers/§6.53). ZOC both
195 │     /// ways; LOS no longer blocked across the hexside.
196 │     Breach,
197 │     /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
198 │     /// combat may not cross (§6.82).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 311) \ #github-link("omdurman-rules/src/effects.rs", 311)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L311")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ArtilleryBreachWall]]]], [#raw("309 │     /// pre-rolled d10 used for the CRT lookup; range/LOS are re-derived by the
310 │     /// engine from the firers and `target`.
311 │     ArtilleryBreachWall {
312 │         firers: Vec<UnitId>,
313 │         target: HexsideRef,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4741) \ #github-link("omdurman-rules/src/effects.rs", 4741)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4741")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_artillery_breach_wall]]]], [#raw("4739 │ /// artillery's CRT roll -- the rulebook specifies the same \"2+ required\"
4740 │ /// threshold for both trigger styles.
4741 │ pub fn apply_artillery_breach_wall(
4742 │     state: &mut GameState,
4743 │     firers: &[UnitId],", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2033) \ #github-link("omdurman-rules/src/effects.rs", 2033)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2033")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_fire_at_wall]]]], [#raw("2031 │     /// range band and resolving the CRT — this method only validates one
2032 │     /// firer at a time.
2033 │     pub fn can_fire_at_wall(
2034 │         &self,
2035 │         firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::breech_marker_cell_returns_none]]]
#v(0.3em)
#heading(level: 2, "§6.64 – Howitzer fire") <sect-6-64>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/howitzer_scatter.rs", 8) \ #github-link("omdurman-rules/src/howitzer_scatter.rs", 8)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/howitzer_scatter.rs#L8")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ScatterHexDirection]]]], [#raw("  6 │ /// the diagram away from the firing player.
  7 │ #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
  8 │ pub enum ScatterHexDirection {
  9 │     UpperLeft,
 10 │     UpperRight,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/howitzer_scatter.rs", 36) \ #github-link("omdurman-rules/src/howitzer_scatter.rs", 36)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/howitzer_scatter.rs#L36")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[howitzer_scatter]]]], [#raw(" 34 │ /// onto a hex-grid offset oriented away from the firer (see
 35 │ /// `GameState::howitzer_impact_hex`).
 36 │ pub fn howitzer_scatter(impact_roll: DieRoll) -> ScatterHexDirection {
 37 │     let table = crate::tables_data::scattergram_table();
 38 │     table", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 422) \ #github-link("omdurman-rules/src/lib.rs", 422)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L422")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId]]]], [#raw("420 │     /// Used only in FALL OF KHARTOUM (§9.32, §9.346).
421 │     Gordon,
422 │ }
423 │ 
424 │ /// Named British gunboat (rulebook §6.64). Five \"named\" gunboats have howitzer", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 121) \ #github-link("omdurman-rules/src/effects.rs", 121)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L121")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HowitzerFire]]]], [#raw("119 │     /// - CRT result applied to units at impact hex (not the original target).
120 │     /// - Firers marked as fired.
121 │     HowitzerFire {
122 │         attack: FireAttack,
123 │         combat_results_table_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 3384) \ #github-link("omdurman-rules/src/effects.rs", 3384)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3384")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_howitzer_fire]]]], [#raw("3382 │ 
3383 │ /// Validate and apply a howitzer fire attack (scatter path) (rulebook §6.64).
3384 │ pub fn apply_howitzer_fire(
3385 │     state: &mut GameState,
3386 │     attack: &FireAttack,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1848) \ #github-link("omdurman-rules/src/effects.rs", 1848)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1848")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_fire_at]]]], [#raw("1846 │     /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
1847 │     /// (Howitzer fire ignores LOS entirely -- §6.64.)
1848 │     pub fn can_fire_at(
1849 │         &self,
1850 │         firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_on_target_7_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_scatters_below_7]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_each_miss_gets_its_printed_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::named_and_old_gunboats_resolve]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::named_gunboat_has_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::named_gunboat_may_fire_howitzer_in_second_subphase]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::named_gunboat_direct_fire_uses_artillery_weapon]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::named_gunboat_no_howitzer_at_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::dervish_gunboat_lacks_howitzer]]]
#v(0.3em)
#heading(level: 2, "§6.81 – Moving player may fire with all capable units") <sect-6-81>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[During Offensive Fire phase, the moving player may fire with all of his units capable of firing, up to their maximum range, within the limitations imposed by the rules of combat.]]
#v(0.5em)
#heading(level: 2, "§6.82 – Advance after combat (offensive fire)") <sect-6-82>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 195) \ #github-link("omdurman-rules/src/effects.rs", 195)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L195")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw("193 │     /// **Postconditions:** Unit position moved to `to`; `vacated_by_combat`
194 │     /// entry consumed.
195 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
196 │ 
197 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4489) \ #github-link("omdurman-rules/src/effects.rs", 4489)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4489")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_advance_after_combat]]]], [#raw("4487 │ 
4488 │ /// Apply an advance-after-combat for a unit (rulebook §6.82, §7.6).
4489 │ pub fn apply_advance_after_combat(
4490 │     state: &mut GameState,
4491 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 239) \ #github-link("omdurman-types/src/lib.rs", 239)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L239")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_advance_after_combat]]]], [#raw("237 │ 
238 │     /// Whether advance-after-combat may *not* cross this side (§6.82, §7.6).
239 │     pub fn blocks_advance_after_combat(self) -> bool {
240 │         matches!(
241 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4084) \ #github-link("omdurman-rules/src/effects.rs", 4084)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4084")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("4082 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
4083 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
4084 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
4085 │         let unit = self.unit_or_err(unit_id)?;
4086 │         // §6.7: there is no advance after combat as a result of defensive fire.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::advance_phase_is_atomic]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_advance_after_combat_rejects_wall_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_advance_after_combat_rejects_khor_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::advance_requires_combat_vacated_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::advance_requires_participation]]]
#v(0.3em)
#progress-bar(8, 8)
#heading(level: 1, "§7 – Melee Phase") <sect-7>
#heading(level: 2, "§7 – Melee Phase (chapter)")
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Melee Phase]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 951) \ #github-link("omdurman-rules/src/effects.rs", 951)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L951")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[PendingMelee]]]], [#raw("949 │ /// resolution after the reaction window is deterministic and host-ordered (rulebook §7.5).
950 │ #[derive(Serialize, Deserialize, Clone, Debug)]
951 │ pub struct PendingMelee {
952 │     pub attack: MeleeAttack,
953 │     pub attacker_roll: DieRoll,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::declared_melee_blocks_phase_advance]]]
#v(0.3em)
#heading(level: 2, "§7.1 – Melee strength printed on counter") <sect-7-1>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 104) \ #github-link("omdurman-rules/src/lib.rs", 104)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L104")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeFactor]]]], [#raw("102 │     }
103 │ }
104 │ 
105 │ value_enum! {
106 │     /// A unit's melee factor as printed on the counter (rulebook §7.1).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 115) \ #github-link("omdurman-rules/src/lib.rs", 115)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L115")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeFactor::sum]]]], [#raw("113 │         Six = 6,
114 │         Seven = 7,
115 │     }
116 │ }
117 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 936) \ #github-link("omdurman-types/src/lib.rs", 936)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L936")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_be_melee_attacked]]]], [#raw("934 │ 
935 │     /// Gunboats neither attack nor are attacked in melee (§7.1).
936 │     pub fn may_be_melee_attacked(self) -> bool {
937 │         !matches!(self, UnitKind::Gunboat { .. })
938 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::reinforcement_rejected_onto_enemy_occupied_hex]]]
#v(0.3em)
#heading(level: 2, "§7.2 – Melee adjacent only, not across wall hexsides") <sect-7-2>
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
  [#vscode-link("omdurman-types/src/lib.rs", 234) \ #github-link("omdurman-types/src/lib.rs", 234)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L234")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_melee]]]], [#raw("232 │     /// Whether melee may *not* be made across this side (§7.2). Gates and
233 │     /// breaches are passable to melee.
234 │     pub fn blocks_melee(self) -> bool {
235 │         matches!(self, HexsideKind::Wall | HexsideKind::ZaribaThornHedge)
236 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2123) \ #github-link("omdurman-rules/src/effects.rs", 2123)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2123")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_melee]]]], [#raw("2121 │     /// that may be melee-attacked (gunboats may not -- §7.1), and no wall or
2122 │     /// thorn-hedge hexside blocks the attack (§7.2).
2123 │     pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
2124 │         let unit = self.unit_or_err(attacker)?;
2125 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_melee_gates_phase_adjacency_and_kind]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_melee_rejects_wall_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_melee_rejects_thorn_hedge_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_melee_allows_gate_hexside]]]
#v(0.3em)
#heading(level: 2, "§7.3 – Simultaneous melee combat") <sect-7-3>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 144) \ #github-link("omdurman-rules/src/effects.rs", 144)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L144")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeCombat]]]], [#raw("142 │     /// - Winner may advance into vacated hex (§7.6).
143 │     /// - Victory points awarded for eliminations.
144 │     MeleeCombat {
145 │         attack: MeleeAttack,
146 │         attacker_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 3741) \ #github-link("omdurman-rules/src/effects.rs", 3741)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3741")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_melee_combat]]]], [#raw("3739 │ 
3740 │ /// Apply a simultaneous melee combat between two adjacent hexes (rulebook §7).
3741 │ pub fn apply_melee_combat(
3742 │     state: &mut GameState,
3743 │     attack: &MeleeAttack,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::melee_resolves_simultaneously]]]
#v(0.3em)
#heading(level: 2, "§7.4 – Who may melee attack / defend") <sect-7-4>
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
  [#vscode-link("omdurman-types/src/lib.rs", 925) \ #github-link("omdurman-types/src/lib.rs", 925)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L925")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_melee_attack]]]], [#raw("923 │     /// Rulebook §7.4 -- only infantry, cavalry, camel and Dervish leaders may
924 │     /// melee *attack*. All others (except gunboats) may melee *defend* (§7.1).
925 │     pub fn may_melee_attack(self) -> bool {
926 │         matches!(
927 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 862) \ #github-link("omdurman-types/src/lib.rs", 862)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L862")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind]]]], [#raw("860 │ /// `Some(UnitKind::Marker)` or `None`.
861 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
862 │ pub enum UnitKind {
863 │     /// Foot infantry (§2.3): fire / melee / movement.
864 │     Infantry {", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 711) \ #github-link("omdurman-types/src/lib.rs", 711)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L711")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishTribe]]]], [#raw("709 │     Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display, strum::EnumIter,
710 │ )]
711 │ pub enum DervishTribe {
712 │     Baggara,
713 │     Jaalin,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2123) \ #github-link("omdurman-rules/src/effects.rs", 2123)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2123")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_melee]]]], [#raw("2121 │     /// that may be melee-attacked (gunboats may not -- §7.1), and no wall or
2122 │     /// thorn-hedge hexside blocks the attack (§7.2).
2123 │     pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
2124 │         let unit = self.unit_or_err(attacker)?;
2125 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::can_melee_gates_phase_adjacency_and_kind]]]
#v(0.3em)
#heading(level: 2, "§7.5 – Cavalry/camel retreat before melee") <sect-7-5>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 181) \ #github-link("omdurman-rules/src/effects.rs", 181)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L181")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RetreatBeforeMelee]]]], [#raw("179 │     ///
180 │     /// **Postconditions:** Unit position moved to `to`.
181 │     RetreatBeforeMelee { unit_id: UnitId, to: HexCoord },
182 │ 
183 │     /// An attacking unit advances into a hex vacated by combat (rulebook §6.82", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4454) \ #github-link("omdurman-rules/src/effects.rs", 4454)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4454")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_retreat_before_melee]]]], [#raw("4452 │ 
4453 │ /// Apply a retreat-before-melee for a cavalry/camel unit (rulebook §7.5).
4454 │ pub fn apply_retreat_before_melee(
4455 │     state: &mut GameState,
4456 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 942) \ #github-link("omdurman-types/src/lib.rs", 942)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L942")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_retreat_before_melee]]]], [#raw("940 │     /// Cavalry and camel units may retreat two hexes from an infantry melee
941 │     /// attack (§7.5).
942 │     pub fn may_retreat_before_melee(self) -> bool {
943 │         matches!(self, UnitKind::Cavalry { .. } | UnitKind::Camel { .. })
944 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 181) \ #github-link("omdurman-rules/src/lib.rs", 181)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L181")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDistance]]]], [#raw("179 │         self.0
180 │     }
181 │ }
182 │ 
183 │ /// A distance measured in hexes (range to target, length of a retreat, ...)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4015) \ #github-link("omdurman-rules/src/effects.rs", 4015)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4015")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_retreat_before_melee]]]], [#raw("4013 │     /// two hexes away and empty. (Does not verify the attacker is infantry --
4014 │     /// the caller offers the retreat only in response to one.)
4015 │     pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
4016 │         let unit = self.unit_or_err(unit_id)?;
4017 │         if !matches!(self.phase, Phase::Melee) {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::resolve_melee_is_atomic]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::retreat_before_melee_only_cavalry_two_hexes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::retreat_opens_window_only_when_hex_empties]]]
#v(0.3em)
#heading(level: 2, "§7.6 – Advance after melee") <sect-7-6>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 195) \ #github-link("omdurman-rules/src/effects.rs", 195)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L195")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw("193 │     /// **Postconditions:** Unit position moved to `to`; `vacated_by_combat`
194 │     /// entry consumed.
195 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
196 │ 
197 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4489) \ #github-link("omdurman-rules/src/effects.rs", 4489)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4489")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_advance_after_combat]]]], [#raw("4487 │ 
4488 │ /// Apply an advance-after-combat for a unit (rulebook §6.82, §7.6).
4489 │ pub fn apply_advance_after_combat(
4490 │     state: &mut GameState,
4491 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4084) \ #github-link("omdurman-rules/src/effects.rs", 4084)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4084")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("4082 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
4083 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
4084 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
4085 │         let unit = self.unit_or_err(unit_id)?;
4086 │         // §6.7: there is no advance after combat as a result of defensive fire.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::advance_phase_is_atomic]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::dervish_advance_after_melee_is_mandatory]]]
#v(0.3em)
#heading(level: 2, "§7.7 – Melee modifiers") <sect-7-7>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 947) \ #github-link("omdurman-rules/src/lib.rs", 947)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L947")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier]]]], [#raw("945 │ }
946 │ 
947 │ // ---------------------------------------------------------------------------
948 │ // 10) Melee combat
949 │ // ---------------------------------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 970) \ #github-link("omdurman-rules/src/lib.rs", 970)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L970")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeAttack]]]], [#raw("968 │             MeleeModifier::DervishVsTrenchedDefender => -2,
969 │         }
970 │     }
971 │ }
972 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 951) \ #github-link("omdurman-rules/src/lib.rs", 951)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L951")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::AngloEgyptianStandard]]]], [#raw("949 │ // ---------------------------------------------------------------------------
950 │ 
951 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
952 │ pub enum MeleeModifier {
953 │     /// +2 to all Dervish melee rolls (§7.7).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 954) \ #github-link("omdurman-rules/src/lib.rs", 954)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L954")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::DervishVsTrenchedDefender]]]], [#raw("952 │ pub enum MeleeModifier {
953 │     /// +2 to all Dervish melee rolls (§7.7).
954 │     DervishStandard,
955 │     /// +1 to all Anglo-Egyptian melee rolls (§7.7).
956 │     AngloEgyptianStandard,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 949) \ #github-link("omdurman-rules/src/lib.rs", 949)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L949")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::DervishStandard]]]], [#raw("947 │ // ---------------------------------------------------------------------------
948 │ // 10) Melee combat
949 │ // ---------------------------------------------------------------------------
950 │ 
951 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::lib::melee_modifier_keeps_roll_legal]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::melee_resolves_simultaneously]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::melee_modifiers_are_engine_derived_and_mismatches_rejected]]]
#v(0.3em)
#progress-bar(2, 3)
#heading(level: 1, "§8 – Night Game Turns") <sect-8>
#heading(level: 2, "§8 – Night Game Turns")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Night Game Turns]]
#v(0.5em)
#heading(level: 2, "§8.1 – Night effects") <sect-8-1>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 150) \ #github-link("omdurman-rules/src/lib.rs", 150)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L150")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementAllowance::halve]]]], [#raw("148 │         Sixteen = 16,
149 │         Eighteen = 18,
150 │     }
151 │ }
152 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 776) \ #github-link("omdurman-types/src/lib.rs", 776)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L776")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DayNight]]]], [#raw("774 │ /// (rulebook §8.1).
775 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
776 │ pub enum DayNight {
777 │     Day,
778 │     Night,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/range_effects.rs", 68) \ #github-link("omdurman-rules/src/range_effects.rs", 68)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/range_effects.rs#L68")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[night_max_range]]]], [#raw(" 66 │ 
 67 │ /// The halved maximum range at night (§8.1): round down, minimum 1.
 68 │ pub fn night_max_range(weapon: WeaponClass, ae: bool) -> u8 {
 69 │     let day = max_day_range(weapon, ae);
 70 │     if day <= 1 { 1 } else { day / 2 }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1442) \ #github-link("omdurman-rules/src/lib.rs", 1442)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1442")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[effective_movement_at_night]]]], [#raw("1440 │ 
1441 │ // ---------------------------------------------------------------------------
1442 │ // 16) Convenience: movement computation under night-turn halving
1443 │ // ---------------------------------------------------------------------------
1444 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::lib::movement_allowance_halve_never_panics]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::night_max_ranges]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::night_max_ranges_remaining]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_rifle_at_night_matches_rulebook_example]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::max_day_range_all_combos]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::night_movement_overlay_allowance_halved]]]
#v(0.3em)
#heading(level: 2, "§8.2 – Dervish Desertion Roll") <sect-8-2>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 223) \ #github-link("omdurman-rules/src/effects.rs", 223)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L223")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishDesertion]]]], [#raw("221 │     /// the effect. The Khalifa, gunboats, artillery, and forts may not be
222 │     /// chosen.
223 │     DervishDesertion {
224 │         roll: DieRoll,
225 │         deserters: Vec<UnitId>,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 63) \ #github-link("omdurman-rules/src/turn_track.rs", 63)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L63")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishDesertion]]]], [#raw(" 61 │     None,
 62 │     /// Dervish desertion roll (§8.2) -- occurs on the first night turn.
 63 │     DervishDesertion,
 64 │     /// Dervish reinforcements are available.
 65 │     DervishReinforcements,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 60) \ #github-link("omdurman-rules/src/turn_track.rs", 60)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L60")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnEvent]]]], [#raw(" 58 │ /// Special events that occur on specific turns (rulebook §8.2, §9.112, §9.113).
 59 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 60 │ pub enum TurnEvent {
 61 │     None,
 62 │     /// Dervish desertion roll (§8.2) -- occurs on the first night turn.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::desertion_count_is_floor_one_and_a_half]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::desertion_on_first_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::desertion_roll_required_before_first_night_movement_ends]]]
#v(0.3em)
#progress-bar(22, 33)
#heading(level: 1, "§9 – The Scenarios") <sect-9>
#heading(level: 2, "§9 – The Scenarios")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Scenarios]]
#v(0.5em)
#heading(level: 2, "§9.1 – The Campaign Game") <sect-9-1>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Campaign Game]]
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::campaign_has_no_fixed_placements]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::tests::scenario_maps_to_board]]]
#v(0.3em)
#heading(level: 2, "§9.2 – The Historical Scenario") <sect-9-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Historical Scenario

Players should note that the historical scenario is an exercise in futility for the Dervish player. It is, however, an interesting demonstration of the absolute imbecility of the Khalifa's generalship and vividly shows the superiority of entrenched firepower over traditional tribal arms in the colonial period.]]
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::tests::start_game_scenario_selects_board]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::remove_deployed_unit_happy_path]]]
#v(0.3em)
#heading(level: 2, "§9.3 – Bonus Game: Fall of Khartoum") <sect-9-3>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Bonus Game: FALL OF KHARTOUM]]
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::remove_deployed_unit_happy_path]]]
#v(0.3em)
#heading(level: 2, "§9.11 – Set Up (Campaign)") <sect-9-11>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Set Up]]
#v(0.5em)
#heading(level: 2, "§9.12 – Scenario Length (Campaign)") <sect-9-12>
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 11) \ #github-link("omdurman-rules/src/turn_track.rs", 11)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L11")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTime]]]], [#raw("  9 │ /// starts at one of these twelve times.
 10 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 11 │ pub enum GameTime {
 12 │     SixAM,
 13 │     EightAM,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 47) \ #github-link("omdurman-rules/src/turn_track.rs", 47)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L47")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnEntry]]]], [#raw(" 45 │ /// A single entry on the Turn Record Track (rulebook §9.12, §9.22).
 46 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 47 │ pub struct TurnEntry {
 48 │     /// 1-based turn number.
 49 │     pub turn: u8,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 95) \ #github-link("omdurman-rules/src/turn_track.rs", 95)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L95")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CAMPAIGN_TURN_TRACK]]]], [#raw(" 93 │ /// is turn 9, which carries the once-per-game Dervish Desertion Roll (§8.2) --
 94 │ /// the printed track prints \"Dervish Desertion Roll / NIGHT\" on that cell.
 95 │ pub const CAMPAIGN_TURN_TRACK: [TurnEntry; 22] = [
 96 │     // Row 1, left->right: Sept 1, 6 am -> 8 pm, then the first NIGHT.
 97 │     entry(1, SixAM, Day, TurnEvent::None),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 242) \ #github-link("omdurman-rules/src/turn_track.rs", 242)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L242")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnLabel]]]], [#raw("240 │ /// printed cell; `Blank` is for unused positions in the 9×3 grid.
241 │ #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
242 │ pub enum TurnLabel {
243 │     Blank,
244 │     /// \"SEPT. 1\\n6:00 am\" -- day header plus time (turn 1).", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::campaign_track_22_turns]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::desertion_on_first_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::campaign_track_label_and_day_night_agree]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::game_time_display_all_variants]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::turn_label_display]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::turn_label_out_of_range_is_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::game_over_after_campaign_turns]]]
#v(0.3em)
#heading(level: 2, "§9.13 – Special Rules (Campaign)") <sect-9-13>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Rules

None.]]
#v(0.5em)
#heading(level: 2, "§9.14 – Victory Conditions (Campaign)") <sect-9-14>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 1096) \ #github-link("omdurman-rules/src/lib.rs", 1096)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1096")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource]]]], [#raw("1094 │ // 15) Victory ledger
1095 │ // ---------------------------------------------------------------------------
1096 │ 
1097 │ /// Every distinct VP source the rulebook enumerates (§9.14). Each variant
1098 │ /// carries its point value as a method so the table cannot drift between", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1122) \ #github-link("omdurman-rules/src/lib.rs", 1122)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1122")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource::points]]]], [#raw("1120 │     FriendliesWestBankEliminated,
1121 │     /// 3 pts -- each Anglo-Egyptian land unit eliminated (§9.14).
1122 │     AngloEgyptianLandUnitEliminated,
1123 │ }
1124 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1137) \ #github-link("omdurman-rules/src/lib.rs", 1137)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1137")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource::who_scores]]]], [#raw("1135 │             VpSource::FriendliesEastBankEliminated => VictoryPoints::new(1),
1136 │             VpSource::FriendliesWestBankEliminated => VictoryPoints::new(3),
1137 │             VpSource::AngloEgyptianLandUnitEliminated => VictoryPoints::new(3),
1138 │         }
1139 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1172) \ #github-link("omdurman-rules/src/lib.rs", 1172)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1172")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger]]]], [#raw("1170 │             }
1171 │         }
1172 │     }
1173 │ }
1174 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1178) \ #github-link("omdurman-rules/src/lib.rs", 1178)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1178")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpEvent]]]], [#raw("1176 │ #[derive(Serialize, Deserialize, Clone, Debug, Default)]
1177 │ pub struct VictoryLedger {
1178 │     pub events: Vec<VpEvent>,
1179 │ }
1180 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1185) \ #github-link("omdurman-rules/src/lib.rs", 1185)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1185")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger::total_for]]]], [#raw("1183 │ pub struct VpEvent {
1184 │     pub turn: GameTurnIndex,
1185 │     pub source: VpSource,
1186 │ }
1187 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1197) \ #github-link("omdurman-rules/src/lib.rs", 1197)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1197")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger::superiority]]]], [#raw("1195 │                 .map(|e| e.source.points().0)
1196 │                 .sum(),
1197 │         )
1198 │     }
1199 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1217) \ #github-link("omdurman-rules/src/lib.rs", 1217)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1217")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CampaignVictoryLevel]]]], [#raw("1215 │             .filter(|e| e.source.who_scores() == player && e.source != VpSource::MahdisTomb)
1216 │             .count() as i16
1217 │     }
1218 │ }
1219 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1226) \ #github-link("omdurman-rules/src/lib.rs", 1226)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1226")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CampaignVictoryLevel::from_superiority]]]], [#raw("1224 │     Marginal(Player),
1225 │     Tactical(Player),
1226 │     Decisive(Player),
1227 │ }
1228 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 5503) \ #github-link("omdurman-rules/src/effects.rs", 5503)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L5503")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[score_elimination]]]], [#raw("5501 │ /// elimination under `cause`. The owner is derived from the unit's identity,
5502 │ /// so unlike the historical signature there is no caller-supplied player.
5503 │ pub fn score_elimination(state: &mut GameState, unit_id: UnitId, cause: ElimCause) {
5504 │     if let Some(unit) = state.find_unit(unit_id) {
5505 │         let identity = unit.profile.identity;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 233) \ #github-link("omdurman-rules/src/lib.rs", 233)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L233")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryPoints]]]], [#raw("231 │         DieRoll::try_from(v).unwrap_or(DieRoll::Ten)
232 │     }
233 │ }
234 │ 
235 │ /// Victory points (signed because they accumulate on either side of a ledger)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::friendlies_bank_scores_by_side]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mahdis_tomb_not_scored_without_a_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mahdis_tomb_scores_for_anglo_egyptian_when_held]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::vp_source_attributes]]]
#v(0.3em)
#heading(level: 2, "§9.21 – Set Up (Historical)") <sect-9-21>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Set Up]]
#v(0.5em)
#heading(level: 2, "§9.22 – Scenario Length (Historical)") <sect-9-22>
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 132) \ #github-link("omdurman-rules/src/turn_track.rs", 132)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L132")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HISTORICAL_TURN_TRACK]]]], [#raw("130 │ 
131 │ /// Historical scenario track (§9.22 -- 4 turns, Sept 2 6:00 am -> 12:00 pm).
132 │ pub const HISTORICAL_TURN_TRACK: [TurnEntry; 4] = [
133 │     TurnEntry {
134 │         turn: 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::historical_turn_all_four_turns]]]
#v(0.3em)
#heading(level: 2, "§9.23 – Special Rule: The Zariba") <sect-9-23>
#status-tag("descriptive")
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
#heading(level: 2, "§9.24 – Victory Conditions (Historical)") <sect-9-24>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 1256) \ #github-link("omdurman-rules/src/lib.rs", 1256)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1256")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HistoricalVictoryLevel]]]], [#raw("1254 │     }
1255 │ }
1256 │ 
1257 │ /// Historical-scenario victory levels (§9.24). Numeric so subtraction works
1258 │ /// per the rulebook example (\"decisive worth 5 minus strategic worth 4 = 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::historical_victory_level_for_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::historical_victory_level_for_anglo_egyptian]]]
#v(0.3em)
#heading(level: 2, "§9.31 – Bonus game map") <sect-9-31>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only the small FALL OF KHARTOUM scenario map is used for this game.]]
#v(0.5em)
#heading(level: 2, "§9.32 – Set Up (Bonus)") <sect-9-32>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Set Up]]
#v(0.5em)
#heading(level: 2, "§9.33 – Scenario Length (Bonus)") <sect-9-33>
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 172) \ #github-link("omdurman-rules/src/turn_track.rs", 172)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L172")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_TURN_TRACK]]]], [#raw("170 │ /// (the rulebook fixes none); only `day_night` is rule-bearing (night halves
171 │ /// Anglo-Egyptian movement and ranges and bars howitzer fire, §8.1).
172 │ pub const FALL_OF_KHARTOUM_TURN_TRACK: [TurnEntry; 8] = [
173 │     TurnEntry {
174 │         turn: 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::fall_of_khartoum_turn_one_is_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::fall_of_khartoum_turns_3_to_8_are_day]]]
#v(0.3em)
#heading(level: 2, "§9.34 – Special Rules (Bonus)") <sect-9-34>
#status-tag("out-of-scope")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Rules]]
#v(0.5em)
#heading(level: 2, "§9.35 – Victory Conditions (Bonus)") <sect-9-35>
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
  [#vscode-link("omdurman-rules/src/lib.rs", 1374) \ #github-link("omdurman-rules/src/lib.rs", 1374)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1374")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FoKVictoryLevel::resolve]]]], [#raw("1372 │         }
1373 │     }
1374 │ 
1375 │     /// Final level: the turn-based base shifted toward the British end of the
1376 │     /// ladder by the Dervish loss penalty (§9.35). Worked example from the", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::fok_victory_level_worked_example]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::fok_victory_level_gordon_died_early]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::fok_victory_level_late_gordon_death]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::lib::fok_victory_level_gordon_survived]]]
#v(0.3em)
#heading(level: 2, "§9.111 – Dervish set up (Campaign)") <sect-9-111>
#status-tag("implemented")
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
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1061) \ #github-link("omdurman-rules/src/effects.rs", 1061)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1061")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[setup_complete]]]], [#raw("1059 │     /// currently shares the same \"both sides deployed\" gate; when a scenario
1060 │     /// needs a different minimum, branch on `self.scenario` here.
1061 │     pub fn setup_complete(&self) -> Result<(), RuleError> {
1062 │         let has = |player| {
1063 │             self.units", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::hadendowa_first_cell_is_isa_zachneih]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::campaign_deployment_is_boat_land_exclusive]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::campaign_setup_rejects_non_initial_force]]]
#v(0.3em)
#heading(level: 2, "§9.112 – Dervish reinforcements (Campaign)") <sect-9-112>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 215) \ #github-link("omdurman-rules/src/effects.rs", 215)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L215")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[PlaceReinforcements]]]], [#raw("213 │     // -- Reinforcement / placement -----------------------------------------
214 │     /// Place reinforcements onto the map (rulebook §9.112, §9.113).
215 │     PlaceReinforcements(Vec<UnitPlacement>),
216 │ 
217 │     // -- Scenario-specific -------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4859) \ #github-link("omdurman-rules/src/effects.rs", 4859)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4859")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_place_reinforcements]]]], [#raw("4857 │ 
4858 │ /// Place reinforcements onto the map (rulebook §9.112, §9.113).
4859 │ pub fn apply_place_reinforcements(
4860 │     state: &mut GameState,
4861 │     placements: &[UnitPlacement],", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/reinforcements.rs", 69) \ #github-link("omdurman-rules/src/reinforcements.rs", 69)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/reinforcements.rs#L69")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_campaign_schedule]]]], [#raw(" 67 │ /// All reinforcements enter on the west edge, south of the Khor Shambat.
 68 │ /// Each unit pays terrain cost of the hex it enters through.
 69 │ pub fn dervish_campaign_schedule() -> ReinforcementSchedule {
 70 │     ReinforcementSchedule {
 71 │         player: Player::Dervish,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 522) \ #github-link("omdurman-types/src/lib.rs", 522)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L522")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Location]]]], [#raw("520 │ /// Named map landmarks (rulebook mapsheet, §9.111, §9.113, §9.212 scenarios).
521 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
522 │ pub enum Location {
523 │     FortMakran,
524 │     NorthFort,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 602) \ #github-link("omdurman-types/src/lib.rs", 602)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L602")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[SetupLetter]]]], [#raw("600 │ /// Each letter marks a specific hex where a Dervish leader is placed.
601 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
602 │ pub enum SetupLetter {
603 │     Y,
604 │     K,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 734) \ #github-link("omdurman-types/src/lib.rs", 734)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L734")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Faction]]]], [#raw("732 │ /// `Some(BrigadeId::friendlies())`.
733 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
734 │ pub enum Faction {
735 │     Dervish {
736 │         tribe: DervishTribe,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::dervish_schedule_has_three_waves]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::dervish_wave_one_has_baggaara_and_three_leaders]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::dervish_wave_two_has_hadendowa]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::dervish_wave_three_is_all_remaining]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::wave_for_turn_returns_correct_wave]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::campaign_reinforcements_gate_by_wave]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::campaign_setup_rejects_non_initial_force]]]
#v(0.3em)
#heading(level: 2, "§9.113 – Anglo-Egyptian reinforcements (Campaign)") <sect-9-113>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Anglo-Egyptian player moves first. There are no Anglo-Egyptian units on the mapsheet at start. The GORDON unit is not used in this scenario.

- The leader units KITCHENER, GATACRE, and HUNTER may enter anytime during the first four game turns and do not count against the 12 units per turn limit. All three leaders must be in play by the end of turn four!
- All gunboats enter through any north edge Nile River hex, paying one movement point for the first hex entered. The "Friendlies" brigade enters through the Abu Alim hut hex on the east bank, paying eight movement points per unit. All other Anglo-Egyptian units enter through the west bank "ANGLO-EGYPTIAN ENTRANCE AREA", each unit paying one movement point to enter the mapsheet.

- Turn 1) Any three gunboats; "Friendlies" brigade; Egyptian Cavalry; Horse Artillery; and two infantry brigades from the Egyptian Division.
- Turn 2) Any three gunboats plus any twelve land units.
- Turn 3) Any three gunboats plus any twelve land units.
- Turn 4) All remaining Anglo-Egyptian units.]],
  align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
)
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/reinforcements.rs", 126) \ #github-link("omdurman-rules/src/reinforcements.rs", 126)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/reinforcements.rs#L126")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[anglo_egyptian_campaign_schedule]]]], [#raw("124 │ /// - \"Friendlies\" enter via Abu Alim hut on the east bank (8 MP per unit).
125 │ /// - All other AE units enter via the Anglo-Egyptian Entrance Area (1 MP).
126 │ pub fn anglo_egyptian_campaign_schedule() -> ReinforcementSchedule {
127 │     let free_leaders = vec![
128 │         CampaignLeader::British(BritishLeader::Kitchener),", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::anglo_egyptian_schedule_has_four_waves]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::anglo_egyptian_leaders_available_each_wave]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::anglo_egyptian_turn_four_is_all_remaining]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::campaign_reinforcement_cap_and_double_entry]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::campaign_gunboats_quota_three_per_turn]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::reinforcement_rejected_onto_enemy_occupied_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::campaign_setup_rejects_non_initial_force]]]
#v(0.3em)
#heading(level: 2, "§9.211 – Anglo-Egyptian set up first, moves second (Historical)") <sect-9-211>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 2979) \ #github-link("omdurman-rules/src/effects.rs", 2979)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2979")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[first_player]]]], [#raw("2977 │ 
2978 │ /// The player who moves first in a scenario (§4, §9.113, §9.212, §9.322).
2979 │ pub fn first_player(scenario: Scenario) -> Player {
2980 │     match scenario {
2981 │         Scenario::Campaign => Player::AngloEgyptian,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::historical_setup_rejects_not_in_play_units]]]
#v(0.3em)
#heading(level: 2, "§9.212 – Dervish set up (Historical) – deployment zones and leader-to-hex mapping") <sect-9-212>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1171) \ #github-link("omdurman-rules/src/effects.rs", 1171)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1171")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[in_deployment_zone]]]], [#raw("1169 │     ///   plan / UI rather than this hex predicate. Documented, not silently
1170 │     ///   dropped.
1171 │     pub fn in_deployment_zone(&self, player: Player, hex: HexCoord, is_boat: bool) -> bool {
1172 │         // No board attached -> permissive (unit tests, unbound session).
1173 │         if self.board.terrain.is_empty() {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 381) \ #github-link("omdurman-rules/src/lib.rs", 381)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L381")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishLeader::setup_letter]]]], [#raw("379 │             DervishLeader::Sherif | DervishLeader::AliWadHelu => true,
380 │         }
381 │     }
382 │ 
383 │     /// The lettered Historical-scenario set-up hex this leader is pinned to", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 397) \ #github-link("omdurman-rules/src/lib.rs", 397)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L397")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_leader_for_setup_letter]]]], [#raw("395 │     }
396 │ }
397 │ 
398 │ /// The Dervish leader pinned to a lettered Historical-scenario set-up hex
399 │ /// (§9.212). `SetupLetter` lives in `omdurman-types` and cannot carry an", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::deploy_rejected_outside_zone]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::embedded_leaders_resolve_from_their_host_section]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::historical_places_all_six_leaders_when_anchors_present]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::missing_anchor_is_reported_not_dropped_silently]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::setup_letter_dervish_leader_roundtrip]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::setup_letter_to_dervish_leader_known_values]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::historical_setup_rejects_not_in_play_units]]]
#v(0.3em)
#heading(level: 2, "§9.231 – Thorn hedge hexsides") <sect-9-231>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 867) \ #github-link("omdurman-rules/src/lib.rs", 867)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L867")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("865 │     /// +1 brigade integrity, applied only if all four battalions fire at
866 │     /// the same enemy-occupied hex (§5.54, §6.24).
867 │     BrigadeIntegrity,
868 │     /// Negative modifier from the Terrain Effects Chart applied to the
869 │     /// defender's hex (§6.23).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 204) \ #github-link("omdurman-types/src/lib.rs", 204)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L204")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("202 │     Crest,
203 │     /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
204 │     ZaribaThornHedge,
205 │     /// Historical-scenario trench segment of the Zariba (§9.232).
206 │     ZaribaTrench,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/board.rs", 267) \ #github-link("omdurman-rules/src/board.rs", 267)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/board.rs#L267")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_zariba_thorn_hedge]]]], [#raw("265 │     /// hexside on its perimeter — i.e. whether the ZaribaThornHedge modifier
266 │     /// applies (§9.231).
267 │     pub fn has_zariba_thorn_hedge(&self, hex: HexCoord) -> bool {
268 │         for n in hex.neighbors() {
269 │             if let Some(kind) = self.hexside_between(hex, n)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zariba_fire_penalties_apply_to_dervish_fire_only]]]
#v(0.3em)
#heading(level: 2, "§9.232 – Trench hexsides") <sect-9-232>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 870) \ #github-link("omdurman-rules/src/lib.rs", 870)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L870")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrenchEntrenched]]]], [#raw("868 │     /// Negative modifier from the Terrain Effects Chart applied to the
869 │     /// defender's hex (§6.23).
870 │     Terrain(i16),
871 │     /// -2 thorn-hedge defensive modifier (§9.231).
872 │     ZaribaThornHedge,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 206) \ #github-link("omdurman-types/src/lib.rs", 206)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L206")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrench]]]], [#raw("204 │     ZaribaThornHedge,
205 │     /// Historical-scenario trench segment of the Zariba (§9.232).
206 │     ZaribaTrench,
207 │     /// One of the two end hexsides of a Zariba trench segment that connect to
208 │     /// the Nile River (§9.233).  Units may only enter/leave the Zariba via", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::lib::melee_modifier_keeps_roll_legal]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::trench_entrenched_units_take_trench_modifiers]]]
#v(0.3em)
#heading(level: 2, "§9.233 – Zariba entry/exit costs") <sect-9-233>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Units may only enter and/or leave the Zariba via the two end hexsides that connect to the Nile River, paying +2 movement points to cross (Exception: advance after combat across an entrenched hexside).]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 256) \ #github-link("omdurman-types/src/lib.rs", 256)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L256")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_movement]]]], [#raw("254 │     /// `omdurman-rules`). The trench *end* variants are therefore intentionally
255 │     /// not blocking.
256 │     pub fn blocks_movement(self) -> bool {
257 │         matches!(
258 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/board.rs", 282) \ #github-link("omdurman-rules/src/board.rs", 282)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/board.rs#L282")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[zariba_entry_surcharge]]]], [#raw("280 │     /// movement points to cross\"). Returns 2 when the edge between `from` and
281 │     /// `to` is one of the two trench ends, else 0.
282 │     pub fn zariba_entry_surcharge(&self, from: HexCoord, to: HexCoord) -> i16 {
283 │         match self.hexside_between(from, to) {
284 │             Some(k) if k.is_zariba_trench_end() => 2,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1666) \ #github-link("omdurman-rules/src/effects.rs", 1666)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1666")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[movement_cost_for]]]], [#raw("1664 │     ///
1665 │     /// §5.42: entering or leaving an enemy ZOC adds no MP cost.
1666 │     pub fn movement_cost_for(
1667 │         &self,
1668 │         unit: &UnitPlacement,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 3191) \ #github-link("omdurman-rules/src/effects.rs", 3191)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3191")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_move_unit]]]], [#raw("3189 │ /// the true terrain cost (§5.11) and enforces gunboat upstream/downstream
3190 │ /// allowances (§5.24); otherwise it falls back to the caller-supplied `cost`.
3191 │ pub fn apply_move_unit(
3192 │     state: &mut GameState,
3193 │     unit_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zariba_end_hexside_costs_extra_mp]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::zariba_thorn_hedge_blocks_movement]]]
#v(0.3em)
#heading(level: 2, "§9.321 – British set up (Bonus)") <sect-9-321>
#status-tag("implemented")
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
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/scenario_setup.rs", 42) \ #github-link("omdurman-rules/src/scenario_setup.rs", 42)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scenario_setup.rs#L42")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_SETUP]]]], [#raw(" 40 │ /// North Fort uses a campaign HadendowaForts counter (one of the spare fort
 41 │ /// sprites).
 42 │ pub const FALL_OF_KHARTOUM_SETUP: &[FixedPlacement] = &[
 43 │     FixedPlacement {
 44 │         section: SectionName::BritishBoats,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::confirm_ready_rejected_below_scenario_target]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_places_gordon_in_the_palace]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_reports_missing_palace]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_fort_landmarks_sit_at_the_correct_hexes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_ae_gunboat_deploys_only_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_ae_land_unit_rejected_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_setup_complete_requires_full_oob]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::deploy_via_real_sprite_resolution_matches_engine]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::british_boats_named_vs_old_gunboat_detection]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_order_of_battle_british]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_order_of_battle_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_caps_bind_across_counter_variants]]]
#v(0.3em)
#heading(level: 2, "§9.322 – Dervish enters turn one (Bonus)") <sect-9-322>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Dervish player moves first: enters turn one through any hexes on the south or east edge of the map.

- 32 Mulazmin units (represents combined forces of Wad El Nejumi, Abu Girgeh, and Sheik El Obeid)
- 2 Hadendowa; 6 Kehena; 5 Degheim (represents Mahdi's combined west bank forces)
- 3 Dervish artillery units.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/unit_profiles.rs", 332) \ #github-link("omdurman-rules/src/unit_profiles.rs", 332)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit_profiles.rs#L332")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ali_wad_helu]]]], [#raw("330 │ ///     (3-6-9) -- the Degheim force of §9.322, printed on Baggara-backed
331 │ ///     sprites.
332 │ pub fn ali_wad_helu(col: u32, row: u32) -> Option<Classification> {
333 │     match (col, row) {
334 │         (0, 0) => dervish_leader(DervishLeader::AliWadHelu),", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 826) \ #github-link("omdurman-types/src/lib.rs", 826)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L826")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[sections_for_picker]]]], [#raw("824 │     ///   provides the 3 artillery, and HadendowaForts supplies the
825 │     ///   Dervish-controlled North Fort sprite (§9.344).
826 │     pub fn sections_for_picker(self) -> Option<&'static [SectionName]> {
827 │         match self {
828 │             Scenario::Campaign | Scenario::Historical => None,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::ali_wad_helu_block_resolves_leader_and_degelim_tribes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_setup_complete_requires_full_oob]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_dervish_land_unit_rejected_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-types::src::lib::fok_picker_allowlist_has_dervish_entry_force_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_order_of_battle_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_dervish_east_edge_on_diamond_board]]]
#v(0.3em)
#heading(level: 2, "§9.341 – Turn 1 is always a night turn (Bonus)") <sect-9-341>
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 172) \ #github-link("omdurman-rules/src/turn_track.rs", 172)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L172")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_TURN_TRACK]]]], [#raw("170 │ /// (the rulebook fixes none); only `day_night` is rule-bearing (night halves
171 │ /// Anglo-Egyptian movement and ranges and bars howitzer fire, §8.1).
172 │ pub const FALL_OF_KHARTOUM_TURN_TRACK: [TurnEntry; 8] = [
173 │     TurnEntry {
174 │         turn: 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::fall_of_khartoum_turn_one_is_night]]]
#v(0.3em)
#heading(level: 2, "§9.342 – All hexes are playable, including half hexes (Bonus)") <sect-9-342>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[All hexes are playable, including hexes showing up half or less.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/board_data.rs", 37) \ #github-link("omdurman-rules/src/board_data.rs", 37)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/board_data.rs#L37")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[fall_of_khartoum_map_data]]]], [#raw(" 35 │ 
 36 │ /// The Fall-of-Khartoum board (§9.3).
 37 │ pub fn fall_of_khartoum_map_data() -> MapData {
 38 │     FOK.get_or_init(|| parse(FOK_RON)).clone()
 39 │ }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fall_of_khartoum_board_excludes_no_hexes]]]
#v(0.3em)
#heading(level: 2, "§9.343 – Both players use the Dervish Range Effects Table (Bonus)") <sect-9-343>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 3418) \ #github-link("omdurman-rules/src/effects.rs", 3418)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3418")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[range_band_for]]]], [#raw("3416 │ /// in FALL OF KHARTOUM *both* players use the Dervish Range Effects Table
3417 │ /// (§9.343).
3418 │ pub fn range_band_for(
3419 │     scenario: Scenario,
3420 │     player: Player,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_both_players_use_dervish_range_table]]]
#v(0.3em)
#heading(level: 2, "§9.344 – Dervish controls the North Fort (Bonus)") <sect-9-344>
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
  [#vscode-link("omdurman-rules/src/scenario_setup.rs", 42) \ #github-link("omdurman-rules/src/scenario_setup.rs", 42)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scenario_setup.rs#L42")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_SETUP]]]], [#raw(" 40 │ /// North Fort uses a campaign HadendowaForts counter (one of the spare fort
 41 │ /// sprites).
 42 │ pub const FALL_OF_KHARTOUM_SETUP: &[FixedPlacement] = &[
 43 │     FixedPlacement {
 44 │         section: SectionName::BritishBoats,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 2205) \ #github-link("omdurman-rules/src/effects.rs", 2205)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2205")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_has_enemy_fort]]]], [#raw("2203 │     /// may neither occupy an enemy fort nor advance after combat into one
2204 │     /// (forts are never captured -- only destroyed, §6.62/§6.53/§7.6).
2205 │     pub fn hex_has_enemy_fort(&self, hex: HexCoord, mover: Player) -> bool {
2206 │         self.units.iter().any(|u| {
2207 │             u.position == hex", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_places_gordon_in_the_palace]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_fort_landmarks_sit_at_the_correct_hexes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::placement_done_gate_matches_by_identity_not_allocated_id]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_order_of_battle_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-types::src::lib::fok_picker_allowlist_has_dervish_entry_force_blocks]]]
#v(0.3em)
#heading(level: 2, "§9.345 – Gunboat White Nile <-> Blue Nile crossing (Bonus)") <sect-9-345>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 2189) \ #github-link("omdurman-rules/src/effects.rs", 2189)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L2189")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_nile_mouth_crossing]]]], [#raw("2187 │     /// must be named on the board, else this is `false` and the move falls
2188 │     /// through to the ordinary contiguous-Nile rules.
2189 │     pub fn is_nile_mouth_crossing(&self, from: HexCoord, to: HexCoord) -> bool {
2190 │         let white = self
2191 │             .board", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 538) \ #github-link("omdurman-types/src/lib.rs", 538)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L538")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Location::WhiteNileMouth]]]], [#raw("536 │     /// The off-board mouth of the White Nile branch (FALL OF KHARTOUM §9.345) --
537 │     /// a British gunboat may cross to the Blue Nile mouth for 6 upstream MP.
538 │     WhiteNileMouth,
539 │     /// The off-board mouth of the Blue Nile branch (FALL OF KHARTOUM §9.345).
540 │     BlueNileMouth,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::fok_gunboat_crosses_between_nile_mouths]]]
#v(0.3em)
#heading(level: 2, "§9.346 – GORDON immobile, eliminated only at the Palace (Bonus)") <sect-9-346>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 3114) \ #github-link("omdurman-rules/src/effects.rs", 3114)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L3114")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[check_gordon_palace]]]], [#raw("3112 │ /// after combat). Records the turn (which fixes the §9.35 victory level) and
3113 │ /// ends the game. A no-op outside FoK, or once GORDON is already gone.
3114 │ pub fn check_gordon_palace(state: &mut GameState) {
3115 │     if state.scenario != Scenario::FallOfKhartoum || state.gordon_eliminated_turn.is_some() {
3116 │         return;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 599) \ #github-link("omdurman-rules/src/lib.rs", 599)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L599")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitIdentity::is_gordon]]]], [#raw("597 │                 ..
598 │             }
599 │         )
600 │     }
601 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::gordon_is_an_immobile_british_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::gordon_survives_means_no_elimination]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_places_gordon_in_the_palace]]]
#v(0.3em)
#progress-bar(7, 10)
#heading(level: 1, "§10 – Optional Rules") <sect-10>
#heading(level: 2, "§10 – Optional Rules")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Optional Rules (Campaign game only)

It is suggested that the most intriguing employment of the following two options is to permit the Dervish player to have either one or the other, but the Anglo-Egyptian player doesn't know which one until he stumbles onto it. Players are advised that the employment of both optionals in the same game is not recommended.]]
#v(0.5em)
#heading(level: 2, "§10.1 – River Mines") <sect-10-1>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[River Mines

The Khalifa twice tried (unsuccessfully) to submerge a powerful mine in the Nile to sink or damage British gunboats. This option assumes that both attempts were successful.]]
#v(0.5em)
#heading(level: 2, "§10.2 – River Chain") <sect-10-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[River Chain

The Khalifa also tried (also unsuccessfully) to string a heavy chain across the Nile to stop or slow down the British gunboats. This option assumes the chain was emplaced.]]
#v(0.5em)
#heading(level: 2, "§10.11 – Secretly record mine hexes") <sect-10-11>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Prior to the commencement of play the Dervish player secretly records two Nile River hexes to be mined (the mines may not both be placed in the same hex). These hexes must be south of the E–W hexrow in which the Khor Shambat empties into the Nile.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 5217) \ #github-link("omdurman-rules/src/effects.rs", 5217)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L5217")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_place_mine]]]], [#raw("5215 │ /// Lay a river mine during setup (§10.11). Validated by
5216 │ /// [`GameState::can_place_mine`].
5217 │ pub fn apply_place_mine(state: &mut GameState, hex: HexCoord) -> Result<(), RuleError> {
5218 │     state.can_place_mine(hex)?;
5219 │     state.mines.push(MinePlacement {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 7768) \ #github-link("omdurman-rules/src/effects.rs", 7768)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L7768")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState::mines]]]], [#raw("7766 │     #[rulebook(\"§10.11\", \"§10.21\")]
7767 │     #[test]
7768 │     fn mines_and_chain_require_their_optional_rule() {
7769 │         // Without the optional rules selected, placement is rejected even in
7770 │         // Setup with room to spare.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 310) \ #github-link("omdurman-rules/src/lib.rs", 310)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L310")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OptionalRule]]]], [#raw("308 │ // ---------------------------------------------------------------------------
309 │ // 3) Scenarios
310 │ // ---------------------------------------------------------------------------
311 │ 
312 │ /// Optional rules -- only legal in the campaign game, and at most one of the", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mine_and_chain_limits_enforced_in_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mines_and_chain_require_their_optional_rule]]]
#v(0.3em)
#heading(level: 2, "§10.12 – Mine resolution") <sect-10-12>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 233) \ #github-link("omdurman-rules/src/effects.rs", 233)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L233")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RiverMine]]]], [#raw("231 │     // -- Optional rules ----------------------------------------------------
232 │     /// River mine resolution (rulebook §10.12).
233 │     RiverMine {
234 │         gunboat_id: UnitId,
235 │         hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 5132) \ #github-link("omdurman-rules/src/effects.rs", 5132)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L5132")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("5130 │ 
5131 │ /// Apply a river-mine resolution (rulebook §10.12).
5132 │ pub fn apply_river_mine(
5133 │     state: &mut GameState,
5134 │     gunboat_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1048) \ #github-link("omdurman-rules/src/lib.rs", 1048)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L1048")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MineResult]]]], [#raw("1046 │ // ---------------------------------------------------------------------------
1047 │ // 14) Optional rules (mines and chain)
1048 │ // ---------------------------------------------------------------------------
1049 │ 
1050 │ /// A mine resolution result (§10.12). The Dervish player rolls 1d10 when a", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mine_fires_once_and_spares_dervish]]]
#v(0.3em)
#heading(level: 2, "§10.13 – Mines consumed after both rolled for") <sect-10-13>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 5132) \ #github-link("omdurman-rules/src/effects.rs", 5132)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L5132")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("5130 │ 
5131 │ /// Apply a river-mine resolution (rulebook §10.12).
5132 │ pub fn apply_river_mine(
5133 │     state: &mut GameState,
5134 │     gunboat_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mine_fires_once_and_spares_dervish]]]
#v(0.3em)
#heading(level: 2, "§10.14 – Dervish gunboats pass safely") <sect-10-14>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 5132) \ #github-link("omdurman-rules/src/effects.rs", 5132)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L5132")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("5130 │ 
5131 │ /// Apply a river-mine resolution (rulebook §10.12).
5132 │ pub fn apply_river_mine(
5133 │     state: &mut GameState,
5134 │     gunboat_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mine_fires_once_and_spares_dervish]]]
#v(0.3em)
#heading(level: 2, "§10.21 – Secretly record chain hexes") <sect-10-21>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Prior to the commencement of play the Dervish player secretly records a line of river hexes (not exceeding four hexes long) across which the chain is strung. The hexes must be south of the E–W hexrow in which the Khor Shambat empties into the Nile.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 1436) \ #github-link("omdurman-rules/src/effects.rs", 1436)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1436")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_place_chain]]]], [#raw("1434 │     /// Read-only check of a river-chain placement in setup (§10.21): Setup phase
1435 │     /// and at most [`MAX_CHAIN_HEXES`] hexes.
1436 │     pub fn can_place_chain(&self, hexes: &[HexCoord]) -> Result<(), RuleError> {
1437 │         self.require_setup_phase()?;
1438 │         // Optional-rule gate: the chain exists only when the River Chain option", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 5228) \ #github-link("omdurman-rules/src/effects.rs", 5228)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L5228")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_place_chain]]]], [#raw("5226 │ /// Lay (or replace) the river chain during setup (§10.21). Validated by
5227 │ /// [`GameState::can_place_chain`].
5228 │ pub fn apply_place_chain(state: &mut GameState, hexes: &[HexCoord]) -> Result<(), RuleError> {
5229 │     state.can_place_chain(hexes)?;
5230 │     state.chain = Some(ChainPlacement {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 4003) \ #github-link("omdurman-rules/src/effects.rs", 4003)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L4003")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MAX_CHAIN_HEXES]]]], [#raw("4001 │ 
4002 │ /// Maximum contiguous Nile hexes the river chain may span (§10.21).
4003 │ pub const MAX_CHAIN_HEXES: usize = 4;
4004 │ 
4005 │ // ---------------------------------------------------------------------------", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mine_and_chain_limits_enforced_in_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::mines_and_chain_require_their_optional_rule]]]
#v(0.3em)
#heading(level: 2, "§10.22 – Gunboat stops on chained hex") <sect-10-22>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 1705) \ #github-link("omdurman-rules/src/effects.rs", 1705)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L1705")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_gunboat]]]], [#raw("1703 │     /// upstream movement allowance is their maximum for that turn.\" Chained Nile
1704 │     /// hexes stop the gunboat (§10.22).
1705 │     pub fn can_move_gunboat(
1706 │         &self,
1707 │         unit_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::chain_stops_gunboat_until_sunk]]]
#v(0.3em)
#heading(level: 2, "§10.23 – Sinking the chain") <sect-10-23>
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
  [#vscode-link("omdurman-rules/src/effects.rs", 5176) \ #github-link("omdurman-rules/src/effects.rs", 5176)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects.rs#L5176")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_sink_chain]]]], [#raw("5174 │ /// Sink the river chain (rulebook §10.23). Marks the placed chain cleared so it
5175 │ /// no longer stops gunboats (§10.22).
5176 │ pub fn apply_sink_chain(state: &mut GameState) -> Result<(), RuleError> {
5177 │     match state.chain.as_mut() {
5178 │         Some(chain) if !chain.sunk => {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::chain_stops_gunboat_until_sunk]]]
#v(0.3em)
#progress-bar(0, 1)
#heading(level: 1, "§11 – Historical Notes") <sect-11>
#heading(level: 2, "§11 – Historical Notes")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#stack(
  block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Historical Notes

In 1881 Mohammed Ahmed Ibn Al-Sayid Abdullah, the son of an obscure carpenter in the hinterlands of the Sudan, proclaimed himself the "Mahdi" — the Messiah of the Islamic faith. His timing was propitious indeed. Since the early 1820's a corrupt Egypt, with the Sultan of Turkey's blessing, had incessantly raped the Sudan, taking ivory and flooding the slave markets with some half million captured Sudanese blacks. By 1880, nearly 40,000 Egyptian troops occupied outposts scattered throughout the Sudan, enforcing Egypt's hold on this lucrative ivory and slave trade and squeezing the native population dry through vicious and corrupt tax officials. All was controlled from Khartoum via the office of Governor General of the Sudan. The title had been held by a succession of individuals, including General Charles Gordon, whose appointment was an attempt to reinstate some rudimentary justice in the Sudan after France and Britain assumed joint political control of a bankrupt Egypt.

By 1881, however, Gordon's term had expired and a new Governor General, again corrupt and incompetent, attempted to deal with the Mahdi. Declining to come to terms with the representatives of Egypt's "benevolent civilization", the Mahdi butchered an armed force dispatched to arrest him in October, 1881. Three months later, the Dervishes (members of a fundamentalist sect following the Mahdi) again ambushed and slaughtered a punitive force of 1400 Egyptian troops sent against him. The effect of these two actions on the native Sudanese was electrifying and they flocked by the thousands to join his holy war and cast out their oppressors.

Egypt, in the meantime, was attempting to throw off Turkish rule and Britain, fearing a revolution and loss of Christian lives, ordered the Mediterranean Squadron to Alexandria in May, 1882. When Turkey refused to intervene, British Marines and Bluejackets went ashore and restored order in Alexandria. Britain next sent General Sir Garnet Wolseley to deal with the rebellious Egyptian army who still controlled Cairo and most of the Egyptian countryside. By mid-September Wolseley had subdued Egypt, winning the battles of Mahsama and Tel-el-Kabir. Thus, by the end of 1882, Britain unwillingly assumed responsibility for Egypt, protecting her communication lines to India in the bargain.

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

By 11:30 the battle was virtually over. 10,000 Dervishes dead — 20,000 wounded, over ¼ of whom would die unattended in the blazing sun during the next two days — 5,000 prisoners — all at a cost of just over 400 killed and wounded in the Sirdar's army. The rest of the story is known to the most casual student of the battle: the 21st Lancers win their first battle streamer and three Victoria Crosses in one of history's last great knee to knee cavalry charges — Maxwell and the XIII Sudanese first to enter the city — 30,000 captured cooks and concubines for whom Kitchener declared he had no use in either capacity — the unused Gatling guns and Nordenfeldts found in the Khalifa's arsenal — the repulsive battlefield with its several hundred acres of suffering wounded and bloating corpses piled around the flags of their dead Emirs — 30,000 Dervish survivors of the battle melted away in the desert, never to rise again. Rarely in modern history has an army and a civilization been so thoroughly crushed, consuming the efforts of half a generation. Fifty-eight years later, Britain would withdraw permanently from Egypt and the Anglo-Egyptian Sudan.

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
#heading(level: 2, "§Credits – Credits")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#progress-bar(1, 1)
#heading(level: 1, "Combat Results Table (shared reference)") <sect-CRT>
#heading(level: 2, "§CRT – Combat Results Table (shared by §6.22 fire and §7.7 melee)")
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 8) \ #github-link("omdurman-rules/src/combat_results_table.rs", 8)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat_results_table.rs#L8")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireFactorRow]]]], [#raw("  6 │ /// to index into the result matrix.
  7 │ #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
  8 │ pub enum FireFactorRow {
  9 │     /// 1-5 factors
 10 │     Row01to05,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 44) \ #github-link("omdurman-rules/src/combat_results_table.rs", 44)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat_results_table.rs#L44")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[from_total]]]], [#raw(" 42 │ 
 43 │     /// Determine which row a given total fire factor falls into (rulebook §6.22).
 44 │     pub fn from_total(total: u16) -> Self {
 45 │         match total {
 46 │             0..=5 => FireFactorRow::Row01to05,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 85) \ #github-link("omdurman-rules/src/combat_results_table.rs", 85)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat_results_table.rs#L85")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[combat_results_table]]]], [#raw(" 83 │ /// D = `Disrupt` (1/2 of target units, round up)
 84 │ /// 1...5 = `Eliminate(n)` (that many units removed)
 85 │ pub fn combat_results_table(row: FireFactorRow, roll: DieRoll) -> CombatResult {
 86 │     let cells = crate::tables_data::crt_table()
 87 │         .get(&row)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 936) \ #github-link("omdurman-rules/src/lib.rs", 936)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L936")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CombatResult]]]], [#raw("934 │ /// A single row of the Combat Results Table, expressed as an enum (rulebook §6.22, §7.7).
935 │ /// Notation from the reference table at the foot of the manual:
936 │ ///
937 │ /// * `D` -- half (round up) of units in the target hex disrupted
938 │ /// * `1`/`2`/`3`/`4`/`5` -- that many units in the target hex eliminated", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::ae_combat_results_table_lowest_is_no_effect]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::ae_combat_results_table_highest_is_eliminate_5]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::ae_combat_results_table_progresses_with_roll]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::ae_combat_results_table_progresses_with_factor]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::fire_factor_row_boundaries]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::fire_factor_row_remaining_boundaries]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::fire_factor_row_index_sequential]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::crt_all_rows_monotone_non_decreasing]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::crt_cross_row_monotone_for_each_roll]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::crt_lowest_row_is_worst_highest_row_is_best]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::crt_eliminate_never_exceeds_5]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::crt_every_cell_matches_the_table]]]
#v(0.3em)
#progress-bar(0, 1)
#heading(level: 1, "Reference – Charts and Tables") <sect-Reference>
#heading(level: 2, "§Reference – Charts and Tables")
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
  [#text(weight: "bold", size: 9pt)[ArtilleryBreachWall]], [#link(<sect-6-63>)[§6.63]],
  [#text(weight: "bold", size: 9pt)[BattalionOrdinal]], [#link(<sect-5-54>)[§5.54]],
  [#text(weight: "bold", size: 9pt)[Breach]], [#link(<sect-6-63>)[§6.63]],
  [#text(weight: "bold", size: 9pt)[BrigadeId]], [#link(<sect-2-3>)[§2.3], #link(<sect-5-54>)[§5.54]],
  [#text(weight: "bold", size: 9pt)[BrigadeIntegrity]], [#link(<sect-5-54>)[§5.54], #link(<sect-6-24>)[§6.24]],
  [#text(weight: "bold", size: 9pt)[BritishLeader]], [#link(<sect-6-51>)[§6.51]],
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
  [#text(weight: "bold", size: 9pt)[DirectFire]], [#link(<sect-6-41>)[§6.41]],
  [#text(weight: "bold", size: 9pt)[FALL_OF_KHARTOUM_SETUP]], [#link(<sect-9-321>)[§9.321], #link(<sect-9-344>)[§9.344]],
  [#text(weight: "bold", size: 9pt)[FALL_OF_KHARTOUM_TURN_TRACK]], [#link(<sect-9-33>)[§9.33], #link(<sect-9-341>)[§9.341]],
  [#text(weight: "bold", size: 9pt)[Faction]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[FireAttack]], [#link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[FireCombat]], [#link(<sect-6-4>)[§6.4]],
  [#text(weight: "bold", size: 9pt)[FireFactor]], [#link(<sect-6-11>)[§6.11]],
  [#text(weight: "bold", size: 9pt)[FireFactorRow]], [#link(<sect-CRT>)[§CRT]],
  [#text(weight: "bold", size: 9pt)[FireSubPhase]], [#link(<sect-6-4>)[§6.4]],
  [#text(weight: "bold", size: 9pt)[Fort]], [#link(<sect-5-25>)[§5.25], #link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[Friendlies]], [#link(<sect-6-52>)[§6.52]],
  [#text(weight: "bold", size: 9pt)[FriendliesAction]], [#link(<sect-5-21>)[§5.21]],
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
  [#text(weight: "bold", size: 9pt)[HowitzerFire]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[Immobile]], [#link(<sect-5-25>)[§5.25]],
  [#text(weight: "bold", size: 9pt)[Khor]], [#link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[Location]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[LosCondition]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[LosFeature]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[LosLevel]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[MAX_CHAIN_HEXES]], [#link(<sect-10-21>)[§10.21]],
  [#text(weight: "bold", size: 9pt)[MaximSecondAndHowitzer]], [#link(<sect-6-42>)[§6.42]],
  [#text(weight: "bold", size: 9pt)[MeleeAttack]], [#link(<sect-7-7>)[§7.7]],
  [#text(weight: "bold", size: 9pt)[MeleeCombat]], [#link(<sect-7-3>)[§7.3]],
  [#text(weight: "bold", size: 9pt)[MeleeFactor]], [#link(<sect-7-1>)[§7.1]],
  [#text(weight: "bold", size: 9pt)[MeleeModifier]], [#link(<sect-7-7>)[§7.7]],
  [#text(weight: "bold", size: 9pt)[MineResult]], [#link(<sect-10-12>)[§10.12]],
  [#text(weight: "bold", size: 9pt)[MovementAllowance]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[MovementPoints]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[NamedGunboat]], [#link(<sect-2-32>)[§2.32]],
  [#text(weight: "bold", size: 9pt)[Old]], [#link(<sect-2-32>)[§2.32]],
  [#text(weight: "bold", size: 9pt)[OldGunboat]], [#link(<sect-2-32>)[§2.32]],
  [#text(weight: "bold", size: 9pt)[OptionalRule]], [#link(<sect-10-11>)[§10.11]],
  [#text(weight: "bold", size: 9pt)[OverLimit]], [#link(<sect-5-51>)[§5.51]],
  [#text(weight: "bold", size: 9pt)[PendingMelee]], [#link(<sect-4>)[§4], #link(<sect-7>)[§7]],
  [#text(weight: "bold", size: 9pt)[Phase]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[PlaceReinforcements]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[RangeBand]], [#link(<sect-6-16>)[§6.16], #link(<sect-6-22>)[§6.22]],
  [#text(weight: "bold", size: 9pt)[RetreatBeforeMelee]], [#link(<sect-7-5>)[§7.5]],
  [#text(weight: "bold", size: 9pt)[RiverMine]], [#link(<sect-10-12>)[§10.12]],
  [#text(weight: "bold", size: 9pt)[RoyalEngineers]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[ScatterHexDirection]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[SetupLetter]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[SpriteAnnotation]], [#link(<sect-2-3>)[§2.3]],
  [#text(weight: "bold", size: 9pt)[Terrain]], [#link(<sect-6-23>)[§6.23]],
  [#text(weight: "bold", size: 9pt)[TransportState]], [#link(<sect-5-21>)[§5.21]],
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
  [#text(weight: "bold", size: 9pt)[WalledCityEntry]], [#link(<sect-5-23>)[§5.23]],
  [#text(weight: "bold", size: 9pt)[WeaponClass]], [#link(<sect-2-31>)[§2.31], #link(<sect-6-61>)[§6.61], #link(<sect-6-62>)[§6.62]],
  [#text(weight: "bold", size: 9pt)[WhiteNileMouth]], [#link(<sect-9-345>)[§9.345]],
  [#text(weight: "bold", size: 9pt)[Zariba]], [#link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[ZaribaThornHedge]], [#link(<sect-9-231>)[§9.231]],
  [#text(weight: "bold", size: 9pt)[ZaribaTrench]], [#link(<sect-9-232>)[§9.232]],
  [#text(weight: "bold", size: 9pt)[ZaribaTrenchEntrenched]], [#link(<sect-9-232>)[§9.232]],
  [#text(weight: "bold", size: 9pt)[ZocReason]], [#link(<sect-5-41>)[§5.41], #link(<sect-5-44>)[§5.44], #link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[advance_phase]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[ae_range_effects]], [#link(<sect-6-22>)[§6.22]],
  [#text(weight: "bold", size: 9pt)[ali_wad_helu]], [#link(<sect-9-322>)[§9.322]],
  [#text(weight: "bold", size: 9pt)[anglo_egyptian_campaign_schedule]], [#link(<sect-9-113>)[§9.113]],
  [#text(weight: "bold", size: 9pt)[apply_advance_after_combat]], [#link(<sect-6-82>)[§6.82], #link(<sect-7-6>)[§7.6]],
  [#text(weight: "bold", size: 9pt)[apply_artillery_breach_wall]], [#link(<sect-6-63>)[§6.63]],
  [#text(weight: "bold", size: 9pt)[apply_construct_zariba]], [#link(<sect-5-3>)[§5.3]],
  [#text(weight: "bold", size: 9pt)[apply_demolition]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[apply_friendlies_transport]], [#link(<sect-5-21>)[§5.21]],
  [#text(weight: "bold", size: 9pt)[apply_howitzer_fire]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[apply_melee_combat]], [#link(<sect-7-3>)[§7.3]],
  [#text(weight: "bold", size: 9pt)[apply_move_unit]], [#link(<sect-9-233>)[§9.233]],
  [#text(weight: "bold", size: 9pt)[apply_place_chain]], [#link(<sect-10-21>)[§10.21]],
  [#text(weight: "bold", size: 9pt)[apply_place_mine]], [#link(<sect-10-11>)[§10.11]],
  [#text(weight: "bold", size: 9pt)[apply_place_reinforcements]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[apply_retreat_before_melee]], [#link(<sect-7-5>)[§7.5]],
  [#text(weight: "bold", size: 9pt)[apply_river_mine]], [#link(<sect-10-12>)[§10.12], #link(<sect-10-13>)[§10.13], #link(<sect-10-14>)[§10.14]],
  [#text(weight: "bold", size: 9pt)[apply_sink_chain]], [#link(<sect-10-23>)[§10.23]],
  [#text(weight: "bold", size: 9pt)[blocking_rules]], [#link(<sect-6-21>)[§6.21], #link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[blocks_advance_after_combat]], [#link(<sect-6-82>)[§6.82]],
  [#text(weight: "bold", size: 9pt)[blocks_los]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[blocks_melee]], [#link(<sect-7-2>)[§7.2]],
  [#text(weight: "bold", size: 9pt)[blocks_movement]], [#link(<sect-5-23>)[§5.23], #link(<sect-9-233>)[§9.233]],
  [#text(weight: "bold", size: 9pt)[blocks_zoc]], [#link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[brigade_integrity]], [#link(<sect-5-54>)[§5.54]],
  [#text(weight: "bold", size: 9pt)[can_advance_after_combat]], [#link(<sect-6-7>)[§6.7], #link(<sect-6-82>)[§6.82], #link(<sect-7-6>)[§7.6]],
  [#text(weight: "bold", size: 9pt)[can_fire_at]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[can_fire_at_wall]], [#link(<sect-6-63>)[§6.63]],
  [#text(weight: "bold", size: 9pt)[can_melee]], [#link(<sect-7-2>)[§7.2], #link(<sect-7-4>)[§7.4]],
  [#text(weight: "bold", size: 9pt)[can_move_gunboat]], [#link(<sect-10-22>)[§10.22]],
  [#text(weight: "bold", size: 9pt)[can_move_unit]], [#link(<sect-5-12>)[§5.12]],
  [#text(weight: "bold", size: 9pt)[can_move_unit_to]], [#link(<sect-5-22>)[§5.22], #link(<sect-5-26>)[§5.26], #link(<sect-5-43>)[§5.43]],
  [#text(weight: "bold", size: 9pt)[can_place_chain]], [#link(<sect-10-21>)[§10.21]],
  [#text(weight: "bold", size: 9pt)[can_retreat_before_melee]], [#link(<sect-7-5>)[§7.5]],
  [#text(weight: "bold", size: 9pt)[check_gordon_palace]], [#link(<sect-9-346>)[§9.346]],
  [#text(weight: "bold", size: 9pt)[check_stacking]], [#link(<sect-5-51>)[§5.51]],
  [#text(weight: "bold", size: 9pt)[combat_results_table]], [#link(<sect-CRT>)[§CRT]],
  [#text(weight: "bold", size: 9pt)[constructing_zariba]], [#link(<sect-5-3>)[§5.3]],
  [#text(weight: "bold", size: 9pt)[defense_modifier]], [#link(<sect-6-23>)[§6.23]],
  [#text(weight: "bold", size: 9pt)[demolishing]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[demolition_targets]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[dervish_campaign_schedule]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[dervish_leader_for_setup_letter]], [#link(<sect-9-212>)[§9.212]],
  [#text(weight: "bold", size: 9pt)[dervish_range_effects]], [#link(<sect-6-22>)[§6.22]],
  [#text(weight: "bold", size: 9pt)[dervish_tribe]], [#link(<sect-2-31>)[§2.31]],
  [#text(weight: "bold", size: 9pt)[die_modifier]], [#link(<sect-6-24>)[§6.24]],
  [#text(weight: "bold", size: 9pt)[effective_movement_at_night]], [#link(<sect-8-1>)[§8.1]],
  [#text(weight: "bold", size: 9pt)[end_player_turn]], [#link(<sect-4>)[§4], #link(<sect-5-13>)[§5.13]],
  [#text(weight: "bold", size: 9pt)[fall_of_khartoum_map_data]], [#link(<sect-9-342>)[§9.342]],
  [#text(weight: "bold", size: 9pt)[fires_twice]], [#link(<sect-6-42>)[§6.42]],
  [#text(weight: "bold", size: 9pt)[first_player]], [#link(<sect-9-211>)[§9.211]],
  [#text(weight: "bold", size: 9pt)[friendlies_transport_offer]], [#link(<sect-5-21>)[§5.21]],
  [#text(weight: "bold", size: 9pt)[from_superiority]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[from_total]], [#link(<sect-CRT>)[§CRT]],
  [#text(weight: "bold", size: 9pt)[halve]], [#link(<sect-8-1>)[§8.1]],
  [#text(weight: "bold", size: 9pt)[has_combat_factors]], [#link(<sect-6-51>)[§6.51]],
  [#text(weight: "bold", size: 9pt)[has_los]], [#link(<sect-6-21>)[§6.21], #link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[has_road]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[has_zariba_thorn_hedge]], [#link(<sect-9-231>)[§9.231]],
  [#text(weight: "bold", size: 9pt)[hex_has_enemy_fort]], [#link(<sect-9-344>)[§9.344]],
  [#text(weight: "bold", size: 9pt)[hex_in_enemy_zoc]], [#link(<sect-5-26>)[§5.26], #link(<sect-5-43>)[§5.43], #link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[howitzer_scatter]], [#link(<sect-6-64>)[§6.64]],
  [#text(weight: "bold", size: 9pt)[in_deployment_zone]], [#link(<sect-5-22>)[§5.22], #link(<sect-9-212>)[§9.212]],
  [#text(weight: "bold", size: 9pt)[is_boat]], [#link(<sect-5-24>)[§5.24]],
  [#text(weight: "bold", size: 9pt)[is_crossroad]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[is_friendlies]], [#link(<sect-5-21>)[§5.21], #link(<sect-6-52>)[§6.52]],
  [#text(weight: "bold", size: 9pt)[is_gordon]], [#link(<sect-9-346>)[§9.346]],
  [#text(weight: "bold", size: 9pt)[is_los_trees]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[is_nile_mouth_crossing]], [#link(<sect-9-345>)[§9.345]],
  [#text(weight: "bold", size: 9pt)[is_walled_city]], [#link(<sect-5-23>)[§5.23]],
  [#text(weight: "bold", size: 9pt)[khalifa_abdullah]], [#link(<sect-2-31>)[§2.31]],
  [#text(weight: "bold", size: 9pt)[loaded_on]], [#link(<sect-5-21>)[§5.21]],
  [#text(weight: "bold", size: 9pt)[los_level]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[los_level_for_unit]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[may_be_melee_attacked]], [#link(<sect-7-1>)[§7.1]],
  [#text(weight: "bold", size: 9pt)[may_enter_walled_city]], [#link(<sect-5-23>)[§5.23]],
  [#text(weight: "bold", size: 9pt)[may_melee_attack]], [#link(<sect-7-4>)[§7.4]],
  [#text(weight: "bold", size: 9pt)[may_retreat_before_melee]], [#link(<sect-7-5>)[§7.5]],
  [#text(weight: "bold", size: 9pt)[mines]], [#link(<sect-10-11>)[§10.11]],
  [#text(weight: "bold", size: 9pt)[movement_cost]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[movement_cost_for]], [#link(<sect-5-42>)[§5.42], #link(<sect-9-233>)[§9.233]],
  [#text(weight: "bold", size: 9pt)[movement_cost_with_road]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[mp_spent]], [#link(<sect-5-12>)[§5.12]],
  [#text(weight: "bold", size: 9pt)[net_modifier]], [#link(<sect-6-54>)[§6.54]],
  [#text(weight: "bold", size: 9pt)[new]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[night_max_range]], [#link(<sect-8-1>)[§8.1]],
  [#text(weight: "bold", size: 9pt)[passable_by_land]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[points]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[range_band_for]], [#link(<sect-9-343>)[§9.343]],
  [#text(weight: "bold", size: 9pt)[resolve]], [#link(<sect-9-35>)[§9.35]],
  [#text(weight: "bold", size: 9pt)[roads]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[score_elimination]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[sections_for_picker]], [#link(<sect-9-322>)[§9.322]],
  [#text(weight: "bold", size: 9pt)[setup_complete]], [#link(<sect-9-111>)[§9.111]],
  [#text(weight: "bold", size: 9pt)[setup_letter]], [#link(<sect-9-212>)[§9.212]],
  [#text(weight: "bold", size: 9pt)[sum]], [#link(<sect-7-1>)[§7.1]],
  [#text(weight: "bold", size: 9pt)[sum_to_row]], [#link(<sect-6-14>)[§6.14]],
  [#text(weight: "bold", size: 9pt)[superiority]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[terrain_effects_chart]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[total_for]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[unit_projects_zoc]], [#link(<sect-5-41>)[§5.41], #link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[value]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[who_scores]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[zariba_entry_surcharge]], [#link(<sect-9-233>)[§9.233]],
  [#text(weight: "bold", size: 9pt)[zoc_hexes]], [#link(<sect-5-41>)[§5.41]],
)

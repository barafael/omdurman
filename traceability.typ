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
#align(center, text(size: 10pt, "REMEMBER GORDON! – Rulebook ⇌ Implementation Mapping"))
#align(center, text(size: 9pt, fill: luma(120), "Generated from `docs/traceability.toml`"))
#v(2em)
#heading(level: 1, "Overview") <sect-overview>
#v(0.3em)
#table(
  columns: (1fr, 1fr, 1fr, 1fr),
  stroke: 0.4pt + luma(190),
  [*Implemented*], [*Descriptive*], [*Implicit*], [*Out-of-scope*],
  [#text(fill: green.darken(20%))[86]], [#text(fill: blue.darken(20%))[33]], [#text(fill: yellow.darken(30%))[2]], [3],
)
#v(0.3em)
#text(size: 9pt)[Total mappings: 124 · Total impl sites: 261]
#v(1em)
#outline(title: [Table of Contents])
#pagebreak()
#progress-bar(1, 3)
#heading(level: 1, "§1 – Introduction") <sect-1>
#heading(level: 2, "§1 – Introduction")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Introduction]]
#v(0.5em)
#heading(level: 2, "§1.1 – General Comments") <sect-1-1>
#status-tag("implemented")
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
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/unit_profiles.rs", 194) \ #github-link("omdurman-rules/src/unit_profiles.rs", 194)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit_profiles.rs#L194")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[command_owns_unit]]]], [#raw("192 │ /// treat *nobody-claims-it* units as the faction's communal pool, so `false`
193 │ /// here means \"any member of the faction may act on this unit\".
194 │ pub fn command_owns_unit(scope: &omdurman_types::CommandScope, identity: &UnitIdentity) -> bool {
195 │     match scope {
196 │         omdurman_types::CommandScope::Army => false,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::command_owns_unit_follows_tribes_and_brigades]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-net::src::lib::start_game_commands_round_trip]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-net::src::lib::legacy_start_game_without_commands_still_loads]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-net::src::lib::brigade_scope_display_and_membership]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::tests::replayed_start_game_stages_commands_and_resets_ready]]]
#v(0.3em)
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
  [#vscode-link("omdurman-types/src/lib.rs", 1059) \ #github-link("omdurman-types/src/lib.rs", 1059)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L1059")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeId]]]], [#raw("1057 │ /// same field for uniform handling.
1058 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
1059 │ pub struct BrigadeId {
1060 │     pub number: u8,
1061 │     pub nationality: BrigadeNationality,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 491) \ #github-link("omdurman-rules/src/unit.rs", 491)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L491")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitProfile]]]], [#raw("489 │ /// print no melee value).
490 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
491 │ pub struct UnitProfile {
492 │     pub kind: UnitKind,
493 │     pub identity: UnitIdentity,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 14) \ #github-link("omdurman-rules/src/lib.rs", 14)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/lib.rs#L14")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeId]]]], [#raw(" 12 │ 
 13 │ use omdurman_types::{
 14 │     BrigadeId, BrigadeNationality, DayNight, DervishTribe, HexCoord, Player, UnitKind,
 15 │ };
 16 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 693) \ #github-link("omdurman-types/src/lib.rs", 693)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L693")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[SpriteAnnotation]]]], [#raw("691 │ /// as an optional overlay over the compiled `sprite_data` fallback.
692 │ #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
693 │ pub struct SpriteAnnotation {
694 │     pub color: SpriteColor,
695 │     pub faction: Option<Faction>,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::british_army_row_zero_specials_classify_by_counter]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::egyptian_army_row_zero_specials_classify_by_counter]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::tribe_stats_come_from_annotation]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::section_owner_dervish_sections]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::section_owner_green_sections_are_dervish]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 167) \ #github-link("omdurman-rules/src/unit.rs", 167)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L167")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("165 │     Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display,
166 │ )]
167 │ pub enum WeaponClass {
168 │     /// Dervish spears and swords -- no ranged fire at all.
169 │     Melee,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit_profiles.rs", 578) \ #github-link("omdurman-rules/src/unit_profiles.rs", 578)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit_profiles.rs#L578")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_tribe]]]], [#raw("576 │ /// Resolve a Dervish tribal foot counter (§2.31): Jehadia, Danagla and
577 │ /// Isa Zachneih fire on the rifles line; every other tribe is spear-armed.
578 │ pub fn dervish_tribe(tribe: DervishTribe) -> Option<Classification> {
579 │     // §2.31: \"Jehadia and Danagla units fire on the 'rifles' line as does the
580 │     // Isa Zachneih unit. All other Dervish units (including leaders) are armed", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit_profiles.rs", 344) \ #github-link("omdurman-rules/src/unit_profiles.rs", 344)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit_profiles.rs#L344")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[khalifa_abdullah]]]], [#raw("342 │ ///     battle (§9.322). All three are interchangeable, so they share the
343 │ ///     `DervishArtillery` identity.
344 │ pub fn khalifa_abdullah(col: u32, row: u32) -> Option<Classification> {
345 │     let artillery = || {
346 │         Some(Classification {", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/unit.rs", 119) \ #github-link("omdurman-rules/src/unit.rs", 119)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L119")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId]]]], [#raw("117 │ /// fire; \"old\" gunboats do not (rulebook §2.32).
118 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
119 │ pub enum GunboatId {
120 │     /// One of the five new-type named gunboats with howitzer capability.
121 │     Named(NamedGunboat),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 138) \ #github-link("omdurman-rules/src/unit.rs", 138)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L138")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[NamedGunboat]]]], [#raw("136 │ /// The five named gunboats with howitzer capability (rulebook §6.64, §2.32).
137 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
138 │ pub enum NamedGunboat {
139 │     Sultan,
140 │     Melik,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 151) \ #github-link("omdurman-rules/src/unit.rs", 151)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L151")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OldGunboat]]]], [#raw("149 │ /// in the Maxim Second Fire and Howitzer subphase (§6.42).
150 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
151 │ pub enum OldGunboat {
152 │     LordKitchener,
153 │     Tamai,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 151) \ #github-link("omdurman-rules/src/unit.rs", 151)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L151")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId::Old]]]], [#raw("149 │ /// in the Maxim Second Fire and Howitzer subphase (§6.42).
150 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
151 │ pub enum OldGunboat {
152 │     LordKitchener,
153 │     Tamai,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 125) \ #github-link("omdurman-rules/src/unit.rs", 125)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L125")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId::DervishGunboat]]]], [#raw("123 │     Old(OldGunboat),
124 │     /// A Dervish gunboat (§9.111, §10.14).
125 │     DervishGunboat(u8),
126 │ }
127 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::old_gunboat_lacks_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::old_gunboat_rejected_from_howitzer_subphase]]]
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
  [#vscode-link("omdurman-rules/src/scalars.rs", 191) \ #github-link("omdurman-rules/src/scalars.rs", 191)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L191")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTurnIndex]]]], [#raw("189 │ /// One-based Game Turn index (1, 2, ... up to the scenario length) (rulebook §4).
190 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
191 │ pub struct GameTurnIndex(pub(crate) u8);
192 │ 
193 │ impl GameTurnIndex {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn.rs", 16) \ #github-link("omdurman-rules/src/turn.rs", 16)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn.rs#L16")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Phase]]]], [#raw(" 14 │ /// etc.
 15 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
 16 │ pub enum Phase {
 17 │     /// Pre-game deployment (§9.2/§9.3/§10): fixed units are placed, each side
 18 │     /// deploys its order of battle within its legal zone, and river", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state.rs", 21) \ #github-link("omdurman-rules/src/effects/state.rs", 21)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state.rs#L21")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState]]]], [#raw(" 19 │ /// All mutable state of a game in progress (rulebook §4).
 20 │ #[derive(Serialize, Deserialize, Clone, Debug)]
 21 │ pub struct GameState {
 22 │     pub scenario: Scenario,
 23 │     pub current_turn: GameTurnIndex,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state.rs", 178) \ #github-link("omdurman-rules/src/effects/state.rs", 178)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state.rs#L178")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState::new]]]], [#raw("176 │ impl GameState {
177 │     /// Create a fresh game state for a given scenario (rulebook §4).
178 │     pub fn new(scenario: Scenario) -> Self {
179 │         let first = scenario_turn(scenario, GameTurnIndex::new(1));
180 │         // First player to *move* per scenario: Campaign -- Anglo-Egyptian moves", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 17) \ #github-link("omdurman-rules/src/effects/effect.rs", 17)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L17")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvancePhase]]]], [#raw(" 15 │     /// At end-of-turn, disrupted units recover and per-turn tracking is
 16 │     /// cleared.
 17 │     AdvancePhase,
 18 │ 
 19 │     // -- Movement ----------------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 192) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 192)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L192")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[advance_phase]]]], [#raw("190 │ 
191 │ /// Advance the game state to the next phase (rulebook §4).
192 │ pub fn advance_phase(state: &mut GameState) -> Result<(), RuleError> {
193 │     let old_phase = state.phase;
194 │     #[cfg(not(feature = \"kani\"))]", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 306) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 306)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L306")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[end_player_turn]]]], [#raw("304 │ 
305 │ /// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
306 │ pub fn end_player_turn(state: &mut GameState) {
307 │     #[cfg(not(feature = \"kani\"))]
308 │     debug!(", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/scalars.rs", 184) \ #github-link("omdurman-rules/src/scalars.rs", 184)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L184")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameTurnIndex::value]]]], [#raw("182 │         Self(value)
183 │     }
184 │     pub fn value(self) -> i32 {
185 │         self.0
186 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state.rs", 170) \ #github-link("omdurman-rules/src/effects/state.rs", 170)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state.rs#L170")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[PendingMelee]]]], [#raw("168 │ /// resolution after the reaction window is deterministic and host-ordered (rulebook §7.5).
169 │ #[derive(Serialize, Deserialize, Clone, Debug)]
170 │ pub struct PendingMelee {
171 │     pub attack: MeleeAttack,
172 │     pub attacker_roll: DieRoll,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::both_ready_auto_advances_out_of_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fire_combat_wrong_phase_rejected]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::new_game_starts_in_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::scenario_turn_dispatches_correctly]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::turn_advances_through_phases]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::game_turn_marker_cell_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::declared_melee_blocks_phase_advance]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 525) \ #github-link("omdurman-rules/src/unit.rs", 525)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L525")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[constructing_zariba]]]], [#raw("523 │     /// Set while the unit is building Zariba hexsides -- neither offensive
524 │     /// fire nor melee allowed that turn (§5.3).
525 │     pub constructing_zariba: bool,
526 │     /// Set when the Royal Engineers are committed to a demolition this turn
527 │     /// (§6.53) -- neither offensive fire nor melee allowed that turn.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 170) \ #github-link("omdurman-rules/src/effects/effect.rs", 170)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L170")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ConstructZariba]]]], [#raw("168 │ 
169 │     /// Begin constructing a Zariba hexside (rulebook §5.3).
170 │     ConstructZariba {
171 │         unit_ids: Vec<UnitId>,
172 │         hexside: HexsideRef,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 49) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 49)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L49")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_construct_zariba]]]], [#raw(" 47 │         GameEffect::RecoverUnit { unit_id } => apply_recover_unit(state, *unit_id),
 48 │         GameEffect::ConstructZariba { unit_ids, hexside } => {
 49 │             apply_construct_zariba(state, unit_ids, *hexside)
 50 │         }
 51 │         GameEffect::Demolition { unit_id, target } => apply_demolition(state, *unit_id, *target),", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::construct_zariba_marks_builders_and_records_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zariba_builders_may_not_fire_offensively_or_melee_until_turn_end]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zariba_construction_is_a_campaign_movement_phase_action]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::world_lens_composes_breaches_and_constructed_zariba]]]
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
  [#vscode-link("omdurman-rules/src/scalars.rs", 63) \ #github-link("omdurman-rules/src/scalars.rs", 63)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L63")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementAllowance]]]], [#raw(" 61 │     /// is a named variant.
 62 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
 63 │     pub enum MovementAllowance {
 64 │         /// Immobile (forts, wrecked gunboats).
 65 │         Immobile = 0,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 502) \ #github-link("omdurman-rules/src/unit.rs", 502)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L502")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitMovement]]]], [#raw("500 │ /// Movement allowance -- uniform for land units, split for gunboats (rulebook §5.11, §5.24, §5.25).
501 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
502 │ pub enum UnitMovement {
503 │     Land(MovementAllowance),
504 │     Gunboat(GunboatMovement),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/scalars.rs", 105) \ #github-link("omdurman-rules/src/scalars.rs", 105)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L105")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementPoints]]]], [#raw("103 │     Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default,
104 │ )]
105 │ pub struct MovementPoints(pub(crate) i16);
106 │ 
107 │ impl std::fmt::Display for MovementPoints {", block: true, lang: "rs")],
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
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 87) \ #github-link("omdurman-rules/src/terrain_chart.rs", 87)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/terrain_chart.rs#L87")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[movement_cost_with_road]]]], [#raw(" 85 │ /// movement overlay only -- combat/LOS use the underlying terrain (\"Per
 86 │ /// other terrain in hex\"). The Nile stays impassable to land units.
 87 │ pub fn movement_cost_with_road(terrain: Terrain, along_road: bool) -> Option<MovementAllowance> {
 88 │     if along_road && !terrain.is_nile() {
 89 │         Some(MovementAllowance::One)", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 512) \ #github-link("omdurman-types/src/lib.rs", 512)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L512")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::has_road]]]], [#raw("510 │ 
511 │     /// Whether this hex has any road touching it.
512 │     pub fn has_road(self) -> bool {
513 │         !matches!(self.road(), Road::None)
514 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 449) \ #github-link("omdurman-types/src/lib.rs", 449)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L449")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::passable_by_land]]]], [#raw("447 │ 
448 │     /// Whether this terrain may be entered by land units (rulebook §5.11).
449 │     pub fn passable_by_land(self) -> bool {
450 │         !self.is_nile()
451 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 517) \ #github-link("omdurman-types/src/lib.rs", 517)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L517")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::is_crossroad]]]], [#raw("515 │ 
516 │     /// Whether roads converge at this hex's centre.
517 │     pub fn is_crossroad(self) -> bool {
518 │         matches!(self.road(), Road::Crossroad)
519 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/board.rs", 341) \ #github-link("omdurman-rules/src/board.rs", 341)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/board.rs#L341")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BoardInfo::roads]]]], [#raw("339 │             tiles: tiles.into_iter().collect(),
340 │             hexsides: Vec::new(),
341 │             roads: Vec::new(),
342 │             excluded: BTreeSet::new(),
343 │             overlay: default_overlay(),", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::movement_column_matches_the_printed_chart]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::terrain_road_state_round_trips]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::road_rate_applies_only_along_the_road]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::crossing_a_hexside_costs_the_chart_surcharge]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::step_cost_adds_the_crossed_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::engine_matches_the_transcribed_terrain_effects_chart]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::extreme_coordinates_are_rejected_without_panicking]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::move_cost_is_engine_computed_and_caller_cost_ignored]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::move_path_must_be_contiguous_and_end_at_destination]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::clear_terrain_no_bonus]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::nile_is_impassable]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::rough_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::swamp_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::hilltop_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::huts_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::movement_cost_convenience_matches_chart]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::movement_cost_with_road_always_one]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::land_unit_may_not_enter_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::movement_cost_without_road_matches_terrain]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::movement_cost_for_uses_terrain]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::movement_cost_for_road_costs_one]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::road_gives_crossroad]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::terrain_movement_costs_in_bounds]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::terrain_chart_road_always_costs_one]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::trees_cost_one_and_give_no_defense]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/movement.rs", 25) \ #github-link("omdurman-rules/src/effects/state/movement.rs", 25)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/movement.rs#L25")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit]]]], [#raw(" 23 │     /// Returns the same `RuleError` the `MoveUnit` effect would on rejection.
 24 │     /// Lets the UI gate input without mutating or duplicating the rules.
 25 │     pub fn can_move_unit(&self, unit_id: UnitId, cost: MovementPoints) -> Result<(), RuleError> {
 26 │         let unit = self.unit_or_err(unit_id)?;
 27 │         self.movement_preconditions(unit)?;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state.rs", 389) \ #github-link("omdurman-rules/src/effects/state.rs", 389)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state.rs#L389")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[mp_spent]]]], [#raw("387 │ 
388 │     /// Movement points `unit_id` has already spent this turn (§5.11/§5.12).
389 │     pub fn mp_spent(&self, unit_id: UnitId) -> i16 {
390 │         self.mp_spent_this_turn.get(&unit_id).copied().unwrap_or(0)
391 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::cumulative_move_cost_may_not_exceed_allowance]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::wrong_faction_move_is_rejected]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::wrong_faction_gunboat_move_is_rejected]]]
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
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 306) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 306)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L306")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[end_player_turn]]]], [#raw("304 │ 
305 │ /// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
306 │ pub fn end_player_turn(state: &mut GameState) {
307 │     #[cfg(not(feature = \"kani\"))]
308 │     debug!(", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::recover_unit_is_the_owners_end_of_turn_only]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::unused_movement_points_do_not_carry_over]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 317) \ #github-link("omdurman-rules/src/unit.rs", 317)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L317")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_friendlies]]]], [#raw("315 │     /// \"Friendlies\" units obey several special rules (§5.21, §5.23, §6.52,
316 │     /// §9.14 victory conditions).
317 │     pub fn is_friendlies(&self) -> bool {
318 │         matches!(
319 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/movement.rs", 492) \ #github-link("omdurman-rules/src/effects/state/movement.rs", 492)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/movement.rs#L492")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[friendlies_transport_offer]]]], [#raw("490 │     /// regardless of selection. Pairs with [`GameEffect::FriendliesTransport`]
491 │     /// so the UI can offer exactly the action the engine would accept.
492 │     pub fn friendlies_transport_offer(&self, selected: Option<UnitId>) -> Option<FriendliesAction> {
493 │         match self.friendlies_transport {
494 │             None => {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 522) \ #github-link("omdurman-rules/src/unit.rs", 522)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L522")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[loaded_on]]]], [#raw("520 │     pub disrupted: bool,
521 │     /// `Some(gunboat)` after a \"Friendlies\" unit loads onto a gunboat (§5.21).
522 │     pub loaded_on: Option<UnitId>,
523 │     /// Set while the unit is building Zariba hexsides -- neither offensive
524 │     /// fire nor melee allowed that turn (§5.3).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 197) \ #github-link("omdurman-rules/src/effects/effect.rs", 197)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L197")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FriendliesTransport]]]], [#raw("195 │ 
196 │     /// Load/disembark the \"Friendlies\" brigade via gunboat (rulebook §5.21).
197 │     FriendliesTransport(crate::FriendliesAction),
198 │ 
199 │     // -- Optional rules ----------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/river.rs", 12) \ #github-link("omdurman-rules/src/effects/river.rs", 12)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/river.rs#L12")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_friendlies_transport]]]], [#raw(" 10 │ ///     unit is freed (a disembarking `MoveUnit` should follow, costed by
 11 │ ///     terrain).
 12 │ pub fn apply_friendlies_transport(
 13 │     state: &mut GameState,
 14 │     action: FriendliesAction,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/transport.rs", 20) \ #github-link("omdurman-rules/src/transport.rs", 20)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/transport.rs#L20")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FriendliesAction]]]], [#raw(" 18 │ /// tracks each unit–gunboat pair independently.
 19 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 20 │ pub enum FriendliesAction {
 21 │     /// Turn N (the load turn): unit and gunboat started adjacent; unit
 22 │     /// loads onto (stacks with) the gunboat.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/transport.rs", 40) \ #github-link("omdurman-rules/src/transport.rs", 40)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/transport.rs#L40")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TransportState]]]], [#raw(" 38 │ /// third turn.
 39 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 40 │ pub enum TransportState {
 41 │     /// Turn N (the load turn): unit and gunboat started adjacent; unit
 42 │     /// loads onto (stacks with) the gunboat.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_mined_gunboat_scores_and_takes_its_friendlies_down]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::friendlies_transport_offer_load_requires_prerequisites]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::friendlies_transport_offer_follows_state_machine]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/movement.rs", 43) \ #github-link("omdurman-rules/src/effects/state/movement.rs", 43)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/movement.rs#L43")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw(" 41 │     ///
 42 │     /// Stacking (§5.51) is checked when the move is applied, not here.
 43 │     pub fn can_move_unit_to(
 44 │         &self,
 45 │         unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/setup.rs", 215) \ #github-link("omdurman-rules/src/effects/state/setup.rs", 215)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/setup.rs#L215")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[in_deployment_zone]]]], [#raw("213 │     ///   plan / UI rather than this hex predicate. Documented, not silently
214 │     ///   dropped.
215 │     pub fn in_deployment_zone(&self, player: Player, hex: HexCoord, is_boat: bool) -> bool {
216 │         // No board attached -> permissive (unit tests, unbound session).
217 │         if self.board.terrain.is_empty() {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::campaign_deployment_is_boat_land_exclusive]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_ae_gunboat_deploys_only_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_ae_land_unit_rejected_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::deploy_via_real_sprite_resolution_matches_engine]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_dervish_land_unit_rejected_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::retreat_before_melee_may_not_land_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_gunboat_is_not_offered_a_land_advance]]]
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
  [#vscode-link("omdurman-types/src/lib.rs", 159) \ #github-link("omdurman-types/src/lib.rs", 159)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L159")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideRef]]]], [#raw("157 │ /// data by [`HexsideRef`].
158 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
159 │ pub struct HexsideRef {
160 │     pub a: HexCoord,
161 │     pub b: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 194) \ #github-link("omdurman-types/src/lib.rs", 194)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L194")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideKind]]]], [#raw("192 │     strum::EnumIter,
193 │ )]
194 │ pub enum HexsideKind {
195 │     /// City wall (Khartoum, walled city of Omdurman). Blocks LOS, blocks
196 │     /// movement except across gates/breaches (§5.23), blocks ZOC into the city", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 272) \ #github-link("omdurman-types/src/lib.rs", 272)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L272")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_movement]]]], [#raw("270 │     /// `omdurman-rules`). The trench *end* variants are therefore intentionally
271 │     /// not blocking.
272 │     pub fn blocks_movement(self) -> bool {
273 │         matches!(
274 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 343) \ #github-link("omdurman-rules/src/unit.rs", 343)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L343")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_enter_walled_city]]]], [#raw("341 │     /// Taiasha bodyguard may enter. Anglo-Egyptian: any unit that can reach the
342 │     /// walled city *except* gunboats and \"Friendlies\".
343 │     pub fn may_enter_walled_city(&self) -> bool {
344 │         match self {
345 │             // §5.23 Dervish: Khalifa, artillery, Taiasha.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/board.rs", 249) \ #github-link("omdurman-rules/src/board.rs", 249)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/board.rs#L249")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_walled_city]]]], [#raw("247 │     /// are its seeds). The set is derived once from the board data, replacing
248 │     /// the older \"at least two of six hexsides are Wall/Gate/Breach\" heuristic.
249 │     pub fn is_walled_city(&self, hex: HexCoord) -> bool {
250 │         // Membership in the precomputed enclosed area (see `walled_city`).
251 │         // Palace/Tomb hexes are always part of it (they are the seeds).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/error.rs", 101) \ #github-link("omdurman-rules/src/effects/error.rs", 101)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/error.rs#L101")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WalledCityEntry]]]], [#raw(" 99 │ 
100 │     #[error(\"unit {0} is not eligible to enter the walled city of Omdurman at {1} (§5.23)\")]
101 │     WalledCityEntry(UnitId, HexCoord),
102 │ 
103 │     #[error(\"movement cost {cost} exceeds allowance {allowance}\")]", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::hexside_ref_is_order_independent]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::hexside_blocking_classifiers_are_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_wall_blocks_every_step_not_just_the_endpoints]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_move_rejects_wall_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_move_allows_gate_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::walled_city_entry_allows_khalifa]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::walled_city_entry_rejects_unauthorized_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::walled_city_entry_rejects_ae_gunboat]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::walled_city_entry_not_enforced_for_fok]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::board_data::campaign_walled_city_is_enclosed_by_walls]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::breach_is_an_opening_for_los_and_effective_kinds]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 243) \ #github-link("omdurman-rules/src/unit.rs", 243)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L243")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatMovement]]]], [#raw("241 │ /// the turn.
242 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
243 │ pub struct GunboatMovement {
244 │     pub upstream: MovementAllowance,
245 │     pub downstream: MovementAllowance,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 983) \ #github-link("omdurman-types/src/lib.rs", 983)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L983")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_boat]]]], [#raw("981 │ 
982 │     /// Gunboats use the split upstream/downstream movement allowance (§5.24).
983 │     pub fn is_boat(self) -> bool {
984 │         matches!(self, UnitKind::Gunboat { .. })
985 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::unit_kind_fire_and_movement_capability_law_is_exact]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::hex_direction_from_index_is_total_mod_six]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::nile_flow_rotation_composes_and_never_leaves_the_nile]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::boat_annotation_yields_split_gunboat_movement]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::gunboat_upstream_cap_is_sticky_across_moves]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::picker::movement::gunboat_cap_gate_mirrors_the_engine]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::picker::movement::remaining_mp_respects_the_sticky_upstream_cap]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 506) \ #github-link("omdurman-rules/src/unit.rs", 506)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L506")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Immobile]]]], [#raw("504 │     Gunboat(GunboatMovement),
505 │     /// Forts may not move once placed (§5.25).
506 │     Immobile,
507 │ }
508 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 940) \ #github-link("omdurman-types/src/lib.rs", 940)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L940")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind::Fort]]]], [#raw("938 │     /// Permanent emplacement (§6.54): fire (artillery) / melee (defensive).
939 │     /// May not move once placed (§5.25).
940 │     Fort { fire: i32, melee: i32 },
941 │     /// Dervish leader (§6.51): fire / melee / movement. May melee attack (§7.4).
942 │     DervishLeader {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 506) \ #github-link("omdurman-rules/src/unit.rs", 506)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L506")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitMovement::Immobile]]]], [#raw("504 │     Gunboat(GunboatMovement),
505 │     /// Forts may not move once placed (§5.25).
506 │     Immobile,
507 │ }
508 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::forts_are_never_advance_eligible]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::immobile_fort_rejects_move_unit]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/movement.rs", 43) \ #github-link("omdurman-rules/src/effects/state/movement.rs", 43)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/movement.rs#L43")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw(" 41 │     ///
 42 │     /// Stacking (§5.51) is checked when the move is applied, not here.
 43 │     pub fn can_move_unit_to(
 44 │         &self,
 45 │         unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 218) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 218)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L218")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("216 │     /// does not extend into or out of a Nile hex. With no board loaded these
217 │     /// reduce to the plain adjacency rule.
218 │     pub fn hex_in_enemy_zoc(
219 │         &self,
220 │         hex: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::unit_entering_enemy_zoc_may_move_no_further_that_turn]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_transit_check_uses_the_actual_path]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 110) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 110)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L110")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[unit_projects_zoc_rule]]]], [#raw("108 │ /// * A fort projects ZOC out of its hex even when unoccupied (§5.44),
109 │ ///   modelled by the fort unit projecting normally.
110 │ pub fn unit_projects_zoc_rule(
111 │     unit: &UnitPlacement,
112 │     mover_player: Player,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 561) \ #github-link("omdurman-rules/src/unit.rs", 561)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L561")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("559 │ /// Used by the engine when answering \"is this hex in an enemy ZOC?\".
560 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
561 │ pub enum ZocReason {
562 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
563 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 200) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 200)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L200")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[unit_projects_zoc]]]], [#raw("198 │     /// §5.44) need the game map, which the engine does not hold; the app layers
199 │     /// those on top. This is the position/kind/disruption core of the rule.
200 │     pub fn unit_projects_zoc(
201 │         &self,
202 │         unit: &UnitPlacement,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 289) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 289)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L289")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[zoc_hexes]]]], [#raw("287 │     /// ZOC covers a given hex; this function returns *which* hexes a
288 │     /// specific unit covers.
289 │     pub fn zoc_hexes(
290 │         &self,
291 │         unit: &UnitPlacement,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::unit_projects_zoc_matches_manual_clauses]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::player_opponent_is_a_fixed_point_free_involution]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_hexes_empty_for_anglo_egyptian_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_hexes_normal_unit_projects_six_adjacent_minus_exclusions]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_hexes_empty_for_disrupted_unit]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_hexes_matches_hex_in_enemy_zoc]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_hexes_excludes_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_hexes_excludes_khor]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/movement.rs", 437) \ #github-link("omdurman-rules/src/effects/state/movement.rs", 437)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/movement.rs#L437")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[movement_cost_for]]]], [#raw("435 │     ///
436 │     /// §5.42: entering or leaving an enemy ZOC adds no MP cost.
437 │     pub fn movement_cost_for(
438 │         &self,
439 │         unit: &UnitPlacement,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::entering_enemy_zoc_costs_no_extra_mp]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/movement.rs", 43) \ #github-link("omdurman-rules/src/effects/state/movement.rs", 43)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/movement.rs#L43")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_unit_to]]]], [#raw(" 41 │     ///
 42 │     /// Stacking (§5.51) is checked when the move is applied, not here.
 43 │     pub fn can_move_unit_to(
 44 │         &self,
 45 │         unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 218) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 218)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L218")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("216 │     /// does not extend into or out of a Nile hex. With no board loaded these
217 │     /// reduce to the plain adjacency rule.
218 │     pub fn hex_in_enemy_zoc(
219 │         &self,
220 │         hex: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::unit_entering_enemy_zoc_may_move_no_further_that_turn]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 561) \ #github-link("omdurman-rules/src/unit.rs", 561)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L561")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("559 │ /// Used by the engine when answering \"is this hex in an enemy ZOC?\".
560 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
561 │ pub enum ZocReason {
562 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
563 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 199) \ #github-link("omdurman-types/src/lib.rs", 199)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L199")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Wall]]]], [#raw("197 │     /// (§5.44), blocks melee (§7.2), blocks advance-after-combat (§6.82).
198 │     #[default]
199 │     Wall,
200 │     /// Gate hexside in a wall. ZOC extends *out of* the walled city through
201 │     /// gates but not into it (§5.44). Melee may be made through a gate (§7.2).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 208) \ #github-link("omdurman-types/src/lib.rs", 208)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L208")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Khor]]]], [#raw("206 │     /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
207 │     /// combat may not cross (§6.82).
208 │     Khor,
209 │     /// Crest line. Blocks LOS unless the firer is on the higher side
210 │     /// (§6.3 note 7).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 574) \ #github-link("omdurman-rules/src/unit.rs", 574)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L574")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason::Zariba]]]], [#raw("572 │     /// Zariba hexside ZOC behaviour in the historical scenario / when the
573 │     /// Zariba is constructed (§5.44).
574 │     Zariba,
575 │ }
576 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 200) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 200)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L200")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[unit_projects_zoc]]]], [#raw("198 │     /// §5.44) need the game map, which the engine does not hold; the app layers
199 │     /// those on top. This is the position/kind/disruption core of the rule.
200 │     pub fn unit_projects_zoc(
201 │         &self,
202 │         unit: &UnitPlacement,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 289) \ #github-link("omdurman-types/src/lib.rs", 289)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L289")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideKind::blocks_zoc]]]], [#raw("287 │     /// cannot express; those are left to the caller. This predicate captures the
288 │     /// symmetric \"does not extend across\" cases.
289 │     pub fn blocks_zoc(self) -> bool {
290 │         matches!(
291 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 218) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 218)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L218")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_in_enemy_zoc]]]], [#raw("216 │     /// does not extend into or out of a Nile hex. With no board loaded these
217 │     /// reduce to the plain adjacency rule.
218 │     pub fn hex_in_enemy_zoc(
219 │         &self,
220 │         hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 242) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 242)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L242")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[zoc_extends]]]], [#raw("240 │     ///   only outward (\"out of, but not into, a walled city hex\"), across a
241 │     ///   breach both ways.
242 │     pub fn zoc_extends(&self, unit: &UnitPlacement, into: HexCoord) -> bool {
243 │         use omdurman_types::HexsideKind;
244 │         let from = unit.position;", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::unit_projects_zoc_matches_manual_clauses]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::hexside_blocking_classifiers_are_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_extends_out_of_but_not_into_buildings_and_forts]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_crosses_walls_and_gates_only_outward]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_hexes_excludes_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_hexes_excludes_khor]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zoc_hexes_normal_unit_projects_six_adjacent_minus_exclusions]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 586) \ #github-link("omdurman-rules/src/unit.rs", 586)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L586")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OverLimit]]]], [#raw("584 │     /// (§6.51: a Dervish unit occupying his hex eliminates him).
585 │     #[error(\"hex stack exceeds the four-unit limit [§5.51]\")]
586 │     OverLimit,
587 │     /// \"Gunboats may not stack with any other unit\" (§5.51, exception §5.21).
588 │     #[error(\"gunboats may not stack with non-gunboat units [§5.51]\")]", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 589) \ #github-link("omdurman-rules/src/unit.rs", 589)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L589")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatStack]]]], [#raw("587 │     /// \"Gunboats may not stack with any other unit\" (§5.51, exception §5.21).
588 │     #[error(\"gunboats may not stack with non-gunboat units [§5.51]\")]
589 │     GunboatStack,
590 │     /// \"Units of different Dervish tribes may not stack together\" (§5.52).
591 │     /// The Dervish artillery (§9.322's three guns) is not a tribe but is not", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 599) \ #github-link("omdurman-rules/src/unit.rs", 599)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L599")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[EnemyCohabitation]]]], [#raw("597 │     /// lone Anglo-Egyptian leader is exempt (§6.51).
598 │     #[error(\"enemy units may not share a hex; melee, not movement, engages them [§5.51, §7.1]\")]
599 │     EnemyCohabitation,
600 │     /// \"If Dervish leaders elect to stack, they may only stack with units of
601 │     /// their command (i.e. colour)\" (§5.53).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 151) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 151)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L151")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[check_stacking]]]], [#raw("149 │     ///   Dervish artillery is its own group) may not stack together.
150 │     /// * §5.53 -- a Dervish leader may stack only with units of its command.
151 │     pub fn check_stacking(
152 │         &self,
153 │         mover: &UnitPlacement,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/stacking.rs", 21) \ #github-link("omdurman-rules/src/effects/state/stacking.rs", 21)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/stacking.rs#L21")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[stacking_rule]]]], [#raw(" 19 │ ///   artillery as its own group) may not share a hex.
 20 │ /// * §5.53: a Dervish leader stacks only with units of its command.
 21 │ pub fn stacking_rule(occupants: &[&UnitPlacement]) -> Result<(), crate::StackingError> {
 22 │     use crate::StackingError;
 23 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::stacking_rule_cohabitation_is_exact]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::stacking_rule_is_symmetric]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::stacking_rule_leaders_are_free_stacking]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::stacking_rule_limit_is_four_counted_units]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::stacking_rule_gunboat_never_shares]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::stacking_over_limit_rejected]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mid_move_stacking_allows_pass_through]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mid_move_stacking_rejects_over_limit_destination]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::validate_stacking_invariants_clean_state]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::validate_stacking_invariants_catches_stacking_violation]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::validate_stacking_invariants_allows_leaders_stacking]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::deploy_rejects_enemy_cohabitation_during_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::validate_stacking_invariants_catches_enemy_cohabitation]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::ae_garrison_stacks_freely_under_gordon]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::melee_mandatory_advance_leaders_are_free_stacking]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 595) \ #github-link("omdurman-rules/src/unit.rs", 595)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L595")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishTribeMix]]]], [#raw("593 │     /// [`UnitIdentity::dervish_stacking_group`]).
594 │     #[error(\"Dervish units of different tribes may not stack [§5.52]\")]
595 │     DervishTribeMix,
596 │     /// A unit may never share a hex with enemy units (§5.51, §7.1); only the
597 │     /// lone Anglo-Egyptian leader is exempt (§6.51).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 395) \ #github-link("omdurman-rules/src/unit.rs", 395)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L395")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_stacking_group]]]], [#raw("393 │     /// `None` for every unit the §5.52 law does not constrain: Dervish
394 │     /// leaders, forts and gunboats, and all Anglo-Egyptian units.
395 │     pub fn dervish_stacking_group(&self) -> Option<DervishStackingGroup> {
396 │         match self {
397 │             UnitIdentity::DervishTribal { tribe } => Some(DervishStackingGroup::Tribe(*tribe)),", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::stacking_rule_group_purity_is_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::green_sections_are_mulazmin_tribal_units]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::deploy_rejects_dervish_tribe_mix]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::deploy_rejects_hadendowa_on_dervish_gun]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::dervish_guns_stack_with_guns_and_their_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::validate_stacking_invariants_clean_state]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 603) \ #github-link("omdurman-rules/src/unit.rs", 603)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L603")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishLeaderCommandMismatch]]]], [#raw("601 │     /// their command (i.e. colour)\" (§5.53).
602 │     #[error(\"Dervish leader may only stack with units of their own command [§5.53]\")]
603 │     DervishLeaderCommandMismatch,
604 │ }
605 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::dervish_leader_stacks_only_with_command_colour]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::command_owns_unit_follows_tribes_and_brigades]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 609) \ #github-link("omdurman-rules/src/unit.rs", 609)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L609")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeIntegrity]]]], [#raw("607 │ /// stack contains all four battalions of a single Anglo-Egyptian brigade.
608 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
609 │ pub enum BrigadeIntegrity {
610 │     None,
611 │     Integrated(BrigadeId),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 459) \ #github-link("omdurman-rules/src/unit.rs", 459)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L459")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[brigade_integrity]]]], [#raw("457 │ /// Only a full stack of four battalions qualifies.  Three or fewer may still
458 │ /// stack and fire, but they receive no brigade-integrity bonus.
459 │ pub fn brigade_integrity(identities: &[UnitIdentity]) -> BrigadeIntegrity {
460 │     let Some(brigade) = identities.first().and_then(|i| i.brigade()) else {
461 │         return BrigadeIntegrity::None;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 609) \ #github-link("omdurman-rules/src/unit.rs", 609)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L609")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireModifier::BrigadeIntegrity]]]], [#raw("607 │ /// stack contains all four battalions of a single Anglo-Egyptian brigade.
608 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
609 │ pub enum BrigadeIntegrity {
610 │     None,
611 │     Integrated(BrigadeId),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 20) \ #github-link("omdurman-rules/src/unit.rs", 20)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L20")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BattalionOrdinal]]]], [#raw(" 18 │     /// brigade integrity requires all four stacked in one hex (§5.54).
 19 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
 20 │     pub enum BattalionOrdinal {
 21 │         First = 1,
 22 │         Second = 2,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 1059) \ #github-link("omdurman-types/src/lib.rs", 1059)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L1059")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeId]]]], [#raw("1057 │ /// same field for uniform handling.
1058 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
1059 │ pub struct BrigadeId {
1060 │     pub number: u8,
1061 │     pub nationality: BrigadeNationality,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::brigade_integrity_requires_all_four_distinct_battalions_of_one_brigade]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::brigade_designation_ignored_for_non_infantry]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::printed_brigade_designation_overrides_column]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::section_owner_anglo_egyptian_sections]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::brigade_integrity_four_battalions_returns_integrated]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::ae_infantry_fourth_battalion_from_col_3]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::brigade_integrity_empty_slice]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::brigade_integrity_friendlies_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::brigade_integrity_three_battalions_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::unit_identity_brigade_and_battalion_accessors]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::ae_infantry_brigade_number_three_from_col_7]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::brigade_integrity_non_infantry_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::brigade_integrity_mixed_brigades_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::ae_infantry_third_battalion_from_col_2]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::brigade_integrity_modifier_is_engine_derived]]]
#v(0.3em)
#progress-bar(23, 31)
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
  [#vscode-link("omdurman-rules/src/los_table.rs", 81) \ #github-link("omdurman-rules/src/los_table.rs", 81)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L81")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosFeature]]]], [#raw(" 79 │ /// authored RON spellings.
 80 │ #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
 81 │ pub enum LosFeature {
 82 │     /// A hex containing units (gunboats/forts excluded per note a).
 83 │     Units,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 103) \ #github-link("omdurman-rules/src/los_table.rs", 103)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L103")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[LosCondition]]]], [#raw("101 │ /// A positional condition from the LOS table Detail footnotes.
102 │ #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
103 │ pub enum LosCondition {
104 │     /// (1) Blocks only if the ray passes through more than two such features.
105 │     MoreThanTwo,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 183) \ #github-link("omdurman-rules/src/los_table.rs", 183)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L183")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[los_level]]]], [#raw("181 │ ///
182 │ /// For all other units, the level is derived from the terrain at `hex`.
183 │ pub fn los_level_for_unit(
184 │     kind: UnitKind,
185 │     hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 183) \ #github-link("omdurman-rules/src/los_table.rs", 183)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L183")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[los_level_for_unit]]]], [#raw("181 │ ///
182 │ /// For all other units, the level is derived from the terrain at `hex`.
183 │ pub fn los_level_for_unit(
184 │     kind: UnitKind,
185 │     hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 223) \ #github-link("omdurman-rules/src/los_table.rs", 223)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L223")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocking_rules]]]], [#raw("221 │ /// always blocks. Indexing is in-bounds by construction (both enums have
222 │ /// exactly three variants).
223 │ pub fn blocking_rules(firer: LosLevel, target: LosLevel) -> &'static [BlockingRule] {
224 │     crate::tables_data::LOS_CELLS[firer.index()][target.index()]
225 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 558) \ #github-link("omdurman-rules/src/los_table.rs", 558)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L558")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_los]]]], [#raw("556 │ /// board-only tools).
557 │ #[allow(clippy::too_many_arguments)]
558 │ pub fn has_los(
559 │     board: &crate::board::BoardInfo,
560 │     from: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 237) \ #github-link("omdurman-types/src/lib.rs", 237)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L237")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexsideKind::blocks_los]]]], [#raw("235 │     /// (LOS table conditions 2–4, 7) and note (e) are handled by the engine
236 │     /// in `omdurman_rules::los_table`, not by this predicate.
237 │     pub fn blocks_los(self) -> bool {
238 │         matches!(self, HexsideKind::Wall | HexsideKind::Crest)
239 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 237) \ #github-link("omdurman-types/src/lib.rs", 237)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L237")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::blocks_los]]]], [#raw("235 │     /// (LOS table conditions 2–4, 7) and note (e) are handled by the engine
236 │     /// in `omdurman_rules::los_table`, not by this predicate.
237 │     pub fn blocks_los(self) -> bool {
238 │         matches!(self, HexsideKind::Wall | HexsideKind::Crest)
239 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 466) \ #github-link("omdurman-types/src/lib.rs", 466)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L466")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Terrain::is_los_trees]]]], [#raw("464 │     /// (§6.3 note 1). Retained for compatibility; the full LOS engine
465 │     /// checks `Terrain::Trees` directly.
466 │     pub fn is_los_trees(self) -> bool {
467 │         matches!(self, Terrain::Trees { .. })
468 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::line_between_forms_a_connected_ray]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::hexside_blocking_classifiers_are_exact]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_depends_only_on_the_ground]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_overrides_hold_for_gunboats_and_forts]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::los_table::blocking_grid_is_total_and_walls_block_ground_firers]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_mapping]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::blocking_rules_all_cells_covered]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_empty_board_is_clear]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_adjacent_clear]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_howitzer_bypasses]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_wall_hexside_blocks_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_gate_hexside_passes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_breach_hexside_passes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::breach_is_an_opening_for_los_and_effective_kinds]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_rough_intervening_blocks_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_two_tree_hexes_pass_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_three_tree_hexes_block_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_two_hut_hexes_pass_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_three_hut_hexes_block_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_hilltop_to_hilltop_clear_no_units]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_hilltop_to_hilltop_blocked_by_hilltop_unit]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_hilltop_to_hilltop_not_blocked_by_ground_unit]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_rough_to_rough_unit_at_lower_level_passes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_rough_to_rough_unit_at_same_level_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_rough_to_rough_hilltop_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_ground_to_hilltop_intervening_hilltop_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_building_blocks_like_huts_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::has_los_two_building_hexes_pass_ground_to_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_for_unit_gunboat_is_rough]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_for_unit_fort_is_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_level_for_unit_walled_city_adj_wall_is_rough]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::gunboat_firer_uses_rough_row_not_ground]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_reflexive_all_terrains]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_reflexive_hilltop]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_symmetric_ground_to_ground_no_units]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_howitzer_always_has_los]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_howitzer_same_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_blocking_rules_match_reference_table]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::los_table::los_ground_to_ground_features_block_as_expected]]]
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
  [#vscode-link("omdurman-rules/src/turn.rs", 44) \ #github-link("omdurman-rules/src/turn.rs", 44)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn.rs#L44")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireSubPhase]]]], [#raw(" 42 │ 
 43 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
 44 │ pub enum FireSubPhase {
 45 │     /// Direct fire (§6.41). Both sides participate in this sub-phase.
 46 │     DirectFire,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 73) \ #github-link("omdurman-rules/src/effects/effect.rs", 73)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L73")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireCombat]]]], [#raw(" 71 │     /// - Firers marked as fired; target hex marked as fired-at.
 72 │     /// - Victory points awarded for eliminations.
 73 │     FireCombat { attack: FireAttack, roll: DieRoll },
 74 │ 
 75 │     /// Resolve a howitzer bombardment (two rolls: CRT + impact scatter)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::turn_advances_through_phases]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/melee.rs", 141) \ #github-link("omdurman-rules/src/effects/state/melee.rs", 141)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/melee.rs#L141")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("139 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
140 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
141 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
142 │         let unit = self.unit_or_err(unit_id)?;
143 │         // §6.7: there is no advance after combat as a result of defensive fire.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::no_advance_after_defensive_fire]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::defensive_fire_opens_no_advance_window]]]
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
  [#vscode-link("omdurman-rules/src/scalars.rs", 18) \ #github-link("omdurman-rules/src/scalars.rs", 18)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L18")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireFactor]]]], [#raw(" 16 │     /// Every possible value from the annotated counter set is a named variant.
 17 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
 18 │     pub enum FireFactor {
 19 │         One = 1,
 20 │         Three = 3,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::fire_factor_sum_to_row]]]
#v(0.3em)
#heading(level: 2, "§6.12 – Fire combat is always voluntary") <sect-6-12>
#status-tag("implicit")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Fire combat is always voluntary. A unit is never required to fire at enemy units merely because they are in range or adjacent.]]
#v(0.5em)
#heading(level: 2, "§6.13 – Fire factor is unitary (may not be divided)") <sect-6-13>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[If a unit elects to fire, its fire combat factor at an enemy unit, that fire combat factor is unitary. A unit's fire combat factor may not be divided up to fire at enemy units on different hexes.]]
#v(0.5em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects/fire.rs", 561) \ #github-link("omdurman-rules/src/effects/fire.rs", 561)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/fire.rs#L561")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[build_fire_attack_from]]]], [#raw("559 │ /// derived exactly as in [`build_fire_attack`]. Returns `None` for an empty,
560 │ /// duplicated, or not-fully-legal list.
561 │ pub fn build_fire_attack_from(
562 │     gs: &GameState,
563 │     firer_hex: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fire_rejects_duplicate_firers_and_a_foreign_firing_player]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::single_unit_fires_alone_from_a_stack]]]
#v(0.3em)
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
  [#vscode-link("omdurman-rules/src/scalars.rs", 32) \ #github-link("omdurman-rules/src/scalars.rs", 32)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L32")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[sum_to_row]]]], [#raw(" 30 │ impl FireFactor {
 31 │     /// Sum multiple fire factors and return the corresponding Combat Results Table row (rulebook §6.11).
 32 │     pub fn sum_to_row<'a>(factors: impl IntoIterator<Item = &'a FireFactor>) -> FireFactorRow {
 33 │         let total: u16 = factors.into_iter().map(|f| f.value()).sum();
 34 │         crate::combat_results_table::FireFactorRow::from_total(total)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/fire.rs", 49) \ #github-link("omdurman-rules/src/effects/state/fire.rs", 49)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/fire.rs#L49")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState::can_fire_at]]]], [#raw(" 47 │     /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
 48 │     /// (Howitzer fire ignores LOS entirely -- §6.64.)
 49 │     pub fn can_fire_at(
 50 │         &self,
 51 │         firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::dead_units_are_pruned_from_every_tracker]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::unit_may_only_be_fired_at_once_per_phase]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::gunboat_and_maxim_may_be_fired_at_repeatedly]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::rejected_fire_attack_does_not_mark_firers_as_fired]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::single_unit_fires_alone_from_a_stack]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::infantry_may_fire_only_once_per_subphase]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::maxim_may_fire_twice_per_turn]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fire_from_two_hexes_combines_into_one_attack]]]
#v(0.3em)
#heading(level: 2, "§6.15 – May divide a stack to fire at different hexes") <sect-6-15>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Players may also divide a stack of units in order to fire at different enemy-occupied hexes. Anglo-Egyptian infantry units having brigade integrity, however, do not receive their +1 direct fire modifier unless they all fire at the same enemy-occupied hex (see #link(<sect-5-54>)[5.54]).]]
#v(0.5em)
#text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #link(<sect-5-54>)[§5.54]]
#v(0.3em)
#table(
  columns: (1.2fr, 1.8fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects/fire.rs", 561) \ #github-link("omdurman-rules/src/effects/fire.rs", 561)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/fire.rs#L561")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[build_fire_attack_from]]]], [#raw("559 │ /// derived exactly as in [`build_fire_attack`]. Returns `None` for an empty,
560 │ /// duplicated, or not-fully-legal list.
561 │ pub fn build_fire_attack_from(
562 │     gs: &GameState,
563 │     firer_hex: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::stack_may_split_its_fire_across_hexes]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 210) \ #github-link("omdurman-rules/src/unit.rs", 210)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L210")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RangeBand]]]], [#raw("208 │ /// multiplied at a given distance (§6.22).
209 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
210 │ pub enum RangeBand {
211 │     Tripled,
212 │     Doubled,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::range_band_halved_is_max_of_one_and_floor_half]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::range_band_apply_is_monotone_in_raw]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::disrupt_half_is_rounded_up]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::halving_rounds_down_and_never_below_one]]]
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
  [#vscode-link("omdurman-rules/src/los_table.rs", 223) \ #github-link("omdurman-rules/src/los_table.rs", 223)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L223")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocking_rules]]]], [#raw("221 │ /// always blocks. Indexing is in-bounds by construction (both enums have
222 │ /// exactly three variants).
223 │ pub fn blocking_rules(firer: LosLevel, target: LosLevel) -> &'static [BlockingRule] {
224 │     crate::tables_data::LOS_CELLS[firer.index()][target.index()]
225 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 558) \ #github-link("omdurman-rules/src/los_table.rs", 558)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/los_table.rs#L558")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_los]]]], [#raw("556 │ /// board-only tools).
557 │ #[allow(clippy::too_many_arguments)]
558 │ pub fn has_los(
559 │     board: &crate::board::BoardInfo,
560 │     from: HexCoord,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_fire_at_rejects_blocked_los]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_fire_at_allows_clear_los]]]
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
  [#vscode-link("omdurman-rules/src/range_effects.rs", 34) \ #github-link("omdurman-rules/src/range_effects.rs", 34)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/range_effects.rs#L34")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ae_range_effects]]]], [#raw(" 32 │ /// Look up the range band for an Anglo-Egyptian weapon (§6.22, §6.24).
 33 │ /// Distances > 10 are out of range for all weapons.
 34 │ pub fn ae_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
 35 │     band_at(faction_rows(true), weapon, distance)
 36 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/range_effects.rs", 40) \ #github-link("omdurman-rules/src/range_effects.rs", 40)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/range_effects.rs#L40")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_range_effects]]]], [#raw(" 38 │ /// Look up the range band for a Dervish weapon (§6.22).
 39 │ /// Distances > 10 are out of range for all weapons.
 40 │ pub fn dervish_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
 41 │     band_at(faction_rows(false), weapon, distance)
 42 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 210) \ #github-link("omdurman-rules/src/unit.rs", 210)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L210")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RangeBand]]]], [#raw("208 │ /// multiplied at a given distance (§6.22).
209 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
210 │ pub enum RangeBand {
211 │     Tripled,
212 │     Doubled,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/scalars.rs", 126) \ #github-link("omdurman-rules/src/scalars.rs", 126)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L126")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDistance]]]], [#raw("124 │ /// (rulebook §6.22, §7.5).
125 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
126 │ pub struct HexDistance(pub(crate) u16);
127 │ 
128 │ impl HexDistance {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::adjacency_iff_distance_one]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::fire_factor_row_from_total_matches_printed_bands]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::range_band_multiplier_arithmetic_is_exact]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_are_out_of_range_outside_the_printed_ten_hex_distance_window]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::max_day_range_is_the_last_in_range_hex]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::fire::range_band_for_routes_to_the_right_faction_table]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::numbered_result_eliminates_without_disrupting_survivors]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_rifles_doubled_at_range_1]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_rifles_halved_at_range_4]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_howitzer_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_rifles_shorter_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::melee_only_range_1]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_artillery_full]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_maxims_match_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_distance_over_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_howitzer_halved_4_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_maxims_and_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_melee]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_distance_over_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fire_combat_eliminates_target]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::max_day_range_all_combos]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_ae_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_ae_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_ae_maxims]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_ae_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_dervish_artillery]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_dervish_maxims_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_dervish_rifles]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_every_cell_dervish_spears]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_fire_at_gates_phase_range_and_player]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mixed_attack_bands_per_firer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_range_effects_monotone_non_increasing]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_howitzer_has_minimum_range]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::dervish_range_effects_monotone_non_increasing]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::range_effects_first_range_max_effect_last_range_oor]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_disrupt_result_falls_on_undisrupted_units_first]]]
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
  [#vscode-link("omdurman-rules/src/combat.rs", 26) \ #github-link("omdurman-rules/src/combat.rs", 26)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L26")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireModifier::Terrain]]]], [#raw(" 24 │     /// Negative modifier from the Terrain Effects Chart applied to the
 25 │     /// defender's hex (§6.23).
 26 │     Terrain(i16),
 27 │     /// -2 thorn-hedge defensive modifier (§9.231).
 28 │     ZaribaThornHedge,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::fire_modifier_keeps_roll_legal]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::defence_column_never_helps_the_attacker]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::engine_matches_the_transcribed_terrain_effects_chart]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fire_across_a_crest_or_wall_is_penalised]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::clear_terrain_no_bonus]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::building_gives_minus_3]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::trees_cost_one_and_give_no_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::rough_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::swamp_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::hilltop_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::huts_movement_and_defense]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::defense_modifier_convenience_matches_chart]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::terrain_movement_costs_in_bounds]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::terrain_chart::terrain_defense_modifier_non_positive]]]
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
  [#vscode-link("omdurman-rules/src/combat.rs", 20) \ #github-link("omdurman-rules/src/combat.rs", 20)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L20")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AngloEgyptianDirectFire]]]], [#raw(" 18 │ pub enum FireModifier {
 19 │     /// +1 to all Anglo-Egyptian *direct* fire (§6.24).
 20 │     AngloEgyptianDirectFire,
 21 │     /// +1 brigade integrity, applied only if all four battalions fire at
 22 │     /// the same enemy-occupied hex (§5.54, §6.24).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 609) \ #github-link("omdurman-rules/src/unit.rs", 609)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L609")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BrigadeIntegrity]]]], [#raw("607 │ /// stack contains all four battalions of a single Anglo-Egyptian brigade.
608 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
609 │ pub enum BrigadeIntegrity {
610 │     None,
611 │     Integrated(BrigadeId),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat.rs", 36) \ #github-link("omdurman-rules/src/combat.rs", 36)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L36")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireModifier::die_modifier]]]], [#raw(" 34 │ impl FireModifier {
 35 │     /// Return the numeric die-roll modifier for this bonus/penalty (rulebook §6.24, §5.54, §6.23, §9.231, §9.232).
 36 │     pub fn die_modifier(self) -> i16 {
 37 │         match self {
 38 │             FireModifier::AngloEgyptianDirectFire | FireModifier::BrigadeIntegrity => 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::die_roll_apply_modifier_is_total]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::brigade_integrity_modifier_is_engine_derived]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fire_modifiers_are_engine_derived_and_mismatches_rejected]]]
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
  [#vscode-link("omdurman-rules/src/turn.rs", 46) \ #github-link("omdurman-rules/src/turn.rs", 46)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn.rs#L46")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DirectFire]]]], [#raw(" 44 │ pub enum FireSubPhase {
 45 │     /// Direct fire (§6.41). Both sides participate in this sub-phase.
 46 │     DirectFire,
 47 │     /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
 48 │     MaximSecondAndHowitzer,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fire_rejects_duplicate_firers_and_a_foreign_firing_player]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::disrupted_unit_cannot_fire]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fire_combat_eliminates_target]]]
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
  [#vscode-link("omdurman-rules/src/turn.rs", 48) \ #github-link("omdurman-rules/src/turn.rs", 48)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn.rs#L48")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MaximSecondAndHowitzer]]]], [#raw(" 46 │     DirectFire,
 47 │     /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
 48 │     MaximSecondAndHowitzer,
 49 │ }
 50 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 1001) \ #github-link("omdurman-types/src/lib.rs", 1001)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L1001")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[fires_twice]]]], [#raw("999 │     /// Maxim guns fire twice per turn -- once in the Direct Fire Subphase and
1000 │     /// again in the Maxim Second Fire Subphase (rulebook §6.42).
1001 │     pub fn fires_twice(self) -> bool {
1002 │         matches!(self, UnitKind::Maxim { .. })
1003 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::unit_kind_fire_and_movement_capability_law_is_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fall_of_khartoum_has_no_maxim_howitzer_subphase]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_on_target_7_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_scatters_below_7]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::advance_window_bridges_fire_subphase_and_closes_at_melee]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fired_at_tracker_resets_at_maxim_subphase]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::maxim_may_fire_twice_per_turn]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::maxim_that_skipped_direct_may_fire_once_in_second_subphase]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 108) \ #github-link("omdurman-rules/src/unit.rs", 108)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L108")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BritishLeader]]]], [#raw("106 │ /// to claim the Mahdi's Tomb (§9.14).
107 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
108 │ pub enum BritishLeader {
109 │     Kitchener,
110 │     Gatacre,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 948) \ #github-link("omdurman-types/src/lib.rs", 948)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L948")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[BritishLeader]]]], [#raw("946 │     },
947 │     /// Anglo-Egyptian leader (§6.51): movement only.
948 │     BritishLeader { movement: i32 },
949 │     /// Wall-breach marker placed by artillery fire (§6.63). Not a combat unit.
950 │     Breech,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 989) \ #github-link("omdurman-types/src/lib.rs", 989)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L989")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_combat_factors]]]], [#raw("987 │     /// British leaders print a movement factor only (§6.51); other kinds carry
988 │     /// fire and/or melee factors. Markers carry no stats.
989 │     pub fn has_combat_factors(self) -> bool {
990 │         !matches!(
991 │             self,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::unit_kind_fire_and_movement_capability_law_is_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::zero_factor_is_none_not_zero]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::kitchener_block_resolves_leaders_friendlies_camel_and_sudanese]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::dervish_leader_sections_resolve_leader_and_retinue_per_cell]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::deploy_rejects_enemy_cohabitation_during_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::dervish_move_through_lone_ae_leader_hex_eliminates_the_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::dervish_move_onto_lone_ae_leader_hex_eliminates_the_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::ae_leader_with_combat_unit_is_not_overrun_by_dervish_move]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::ae_leader_eliminated_with_last_combat_unit_in_fire_combat]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_lone_gordon_makes_no_melee_roll]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::melee_advance_overruns_a_lone_ae_leader]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 317) \ #github-link("omdurman-rules/src/unit.rs", 317)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L317")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_friendlies]]]], [#raw("315 │     /// \"Friendlies\" units obey several special rules (§5.21, §5.23, §6.52,
316 │     /// §9.14 victory conditions).
317 │     pub fn is_friendlies(&self) -> bool {
318 │         matches!(
319 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 1031) \ #github-link("omdurman-types/src/lib.rs", 1031)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L1031")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Friendlies]]]], [#raw("1029 │     /// Native volunteer brigade -- the Shaggyeh (§6.52). Do not receive
1030 │     /// brigade integrity (§5.54 enumerates only British/Egyptian/Sudanese).
1031 │     Friendlies,
1032 │ }
1033 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::fire::range_table_player_for_routes_friendlies_and_fok_to_the_dervish_table]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::friendlies_counters_score_by_bank_not_as_leaders]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::friendlies_validate_and_resolve_on_dervish_table]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::friendlies_melee_with_the_dervish_modifier]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 288) \ #github-link("omdurman-rules/src/unit.rs", 288)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L288")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RoyalEngineers]]]], [#raw("286 │     /// The Royal Engineers (§6.53) -- a *specific* unit, not a class, so we
287 │     /// model it explicitly.
288 │     RoyalEngineers,
289 │ }
290 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 528) \ #github-link("omdurman-rules/src/unit.rs", 528)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L528")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[demolishing]]]], [#raw("526 │     /// Set when the Royal Engineers are committed to a demolition this turn
527 │     /// (§6.53) -- neither offensive fire nor melee allowed that turn.
528 │     pub demolishing: bool,
529 │     /// Set when a gunboat has lost its engines to a river mine (§10.12, roll
530 │     /// 5-7): it may no longer move under power and instead drifts two hexes per", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 176) \ #github-link("omdurman-rules/src/effects/effect.rs", 176)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L176")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Demolition]]]], [#raw("174 │ 
175 │     /// Royal Engineers demolition (rulebook §6.53).
176 │     Demolition {
177 │         unit_id: UnitId,
178 │         target: DemolitionTarget,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/setup.rs", 37) \ #github-link("omdurman-rules/src/effects/setup.rs", 37)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/setup.rs#L37")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_demolition]]]], [#raw(" 35 │ /// resolution happens at end of turn via [`apply_resolve_demolition`], which
 36 │ /// checks the engineer is still adjacent and undisrupted.
 37 │ pub fn apply_demolition(
 38 │     state: &mut GameState,
 39 │     unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat.rs", 155) \ #github-link("omdurman-rules/src/combat.rs", 155)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L155")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DemolitionTarget]]]], [#raw("153 │ /// disrupted or driven off.
154 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
155 │ pub enum DemolitionTarget {
156 │     Fort(UnitId),
157 │     WallHexside(HexsideRef),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/engineering.rs", 39) \ #github-link("omdurman-rules/src/effects/state/engineering.rs", 39)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/engineering.rs#L39")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[demolition_targets]]]], [#raw(" 37 │     /// the rules would accept. Empty when the unit doesn't exist or has no
 38 │     /// adjacent target.
 39 │     pub fn demolition_targets(&self, unit_id: UnitId) -> Vec<DemolitionTarget> {
 40 │         let Ok(unit) = self.unit_or_err(unit_id) else {
 41 │             return Vec::new();", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::demolition_requires_engineers_an_adjacent_target_and_a_pending_commit]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::demolition_cancelled_when_engineer_disrupted]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::demolition_cancelled_when_engineer_moved_away]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::demolition_targets_finds_adjacent_fort_and_wall]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 561) \ #github-link("omdurman-rules/src/unit.rs", 561)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L561")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZocReason]]]], [#raw("559 │ /// Used by the engine when answering \"is this hex in an enemy ZOC?\".
560 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
561 │ pub enum ZocReason {
562 │     /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
563 │     /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 568) \ #github-link("omdurman-rules/src/unit.rs", 568)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L568")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Fort]]]], [#raw("566 │     GunboatVsGunboat,
567 │     /// Forts project ZOC out of, but not into, an empty fort (§5.44, §6.54).
568 │     Fort,
569 │     /// Walled-city ZOC: extends out through walls and gates but not in,
570 │     /// across a breach in both directions (§5.44).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 516) \ #github-link("omdurman-rules/src/unit.rs", 516)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L516")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitState]]]], [#raw("514 │ /// rather than one big enum.
515 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
516 │ pub struct UnitState {
517 │     /// Reference table: \"Disrupted units: no ZOC; may not move; may not fire
518 │     /// offensively or defensively; may not melee; are turned face up at the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat.rs", 65) \ #github-link("omdurman-rules/src/combat.rs", 65)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L65")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireAttack]]]], [#raw(" 63 │ /// modifiers (rulebook §6).
 64 │ #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
 65 │ pub struct FireAttack {
 66 │     pub firing_player: Player,
 67 │     pub phase: Phase,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat.rs", 80) \ #github-link("omdurman-rules/src/combat.rs", 80)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L80")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireAttack::net_modifier]]]], [#raw(" 78 │ impl FireAttack {
 79 │     /// Sum of all fire modifiers applied to this attack (rulebook §6.24).
 80 │     pub fn net_modifier(&self) -> i16 {
 81 │         // Saturating fold rather than `sum()`: the modifier list is unbounded
 82 │         // and arrives over the network, so a plain sum can overflow `i16`.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::printed_fok_forts_are_british_forts]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::board_data::fok_forts_are_not_walled_in]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::retreat_before_melee_may_not_land_on_enemy_fort]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mandatory_advance_stops_at_an_enemy_fort]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 167) \ #github-link("omdurman-rules/src/unit.rs", 167)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L167")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("165 │     Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display,
166 │ )]
167 │ pub enum WeaponClass {
168 │     /// Dervish spears and swords -- no ranged fire at all.
169 │     Melee,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/fire.rs", 49) \ #github-link("omdurman-rules/src/effects/state/fire.rs", 49)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/fire.rs#L49")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState::can_fire_at]]]], [#raw(" 47 │     /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
 48 │     /// (Howitzer fire ignores LOS entirely -- §6.64.)
 49 │     pub fn can_fire_at(
 50 │         &self,
 51 │         firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_gunboat_sunk_by_artillery_scores_victory_points]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::rifles_may_not_sink_a_gunboat]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::artillery_sinks_gunboat_only_on_three_plus]]]
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
  [#vscode-link("omdurman-rules/src/unit.rs", 167) \ #github-link("omdurman-rules/src/unit.rs", 167)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L167")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[WeaponClass]]]], [#raw("165 │     Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display,
166 │ )]
167 │ pub enum WeaponClass {
168 │     /// Dervish spears and swords -- no ranged fire at all.
169 │     Melee,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/fire.rs", 49) \ #github-link("omdurman-rules/src/effects/state/fire.rs", 49)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/fire.rs#L49")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState::can_fire_at]]]], [#raw(" 47 │     /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
 48 │     /// (Howitzer fire ignores LOS entirely -- §6.64.)
 49 │     pub fn can_fire_at(
 50 │         &self,
 51 │         firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_destroyed_fort_takes_one_occupant_with_it_and_scores_it]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::artillery_destroys_fort_on_two_or_better_only]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::destroyed_fort_takes_one_occupant_with_it]]]
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
  [#vscode-link("omdurman-types/src/lib.rs", 205) \ #github-link("omdurman-types/src/lib.rs", 205)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L205")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Breach]]]], [#raw("203 │     /// Breach in a wall (artillery/§6.63 or Royal Engineers/§6.53). ZOC both
204 │     /// ways; LOS no longer blocked across the hexside.
205 │     Breach,
206 │     /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
207 │     /// combat may not cross (§6.82).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 282) \ #github-link("omdurman-rules/src/effects/effect.rs", 282)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L282")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ArtilleryBreachWall]]]], [#raw("280 │     /// pre-rolled d10 used for the CRT lookup; range/LOS are re-derived by the
281 │     /// engine from the firers and `target`.
282 │     ArtilleryBreachWall {
283 │         firers: Vec<UnitId>,
284 │         target: HexsideRef,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/fire.rs", 847) \ #github-link("omdurman-rules/src/effects/fire.rs", 847)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/fire.rs#L847")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_artillery_breach_wall]]]], [#raw("845 │ /// artillery's CRT roll -- the rulebook specifies the same \"2+ required\"
846 │ /// threshold for both trigger styles.
847 │ pub fn apply_artillery_breach_wall(
848 │     state: &mut GameState,
849 │     firers: &[UnitId],", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/fire.rs", 250) \ #github-link("omdurman-rules/src/effects/state/fire.rs", 250)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/fire.rs#L250")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_fire_at_wall]]]], [#raw("248 │     /// range band and resolving the CRT — this method only validates one
249 │     /// firer at a time.
250 │     pub fn can_fire_at_wall(
251 │         &self,
252 │         firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::breach_casualty_stands_at_the_wall]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::breech_marker_cell_returns_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::artillery_breaches_wall_only_on_crt_two_or_better]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::wall_breach_eliminates_one_adjacent_enemy_unit]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::breach_is_an_opening_for_los_and_effective_kinds]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::artillery_may_fire_at_the_wall_it_faces]]]
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
  [#vscode-link("omdurman-rules/src/howitzer_scatter.rs", 38) \ #github-link("omdurman-rules/src/howitzer_scatter.rs", 38)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/howitzer_scatter.rs#L38")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[howitzer_scatter]]]], [#raw(" 36 │ /// onto a hex-grid offset oriented away from the firer (see
 37 │ /// `GameState::howitzer_impact_hex`).
 38 │ pub fn howitzer_scatter(impact_roll: DieRoll) -> ScatterHexDirection {
 39 │     crate::tables_data::SCATTERGRAM[(impact_roll.value() - 1) as usize]
 40 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 119) \ #github-link("omdurman-rules/src/unit.rs", 119)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L119")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GunboatId]]]], [#raw("117 │ /// fire; \"old\" gunboats do not (rulebook §2.32).
118 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
119 │ pub enum GunboatId {
120 │     /// One of the five new-type named gunboats with howitzer capability.
121 │     Named(NamedGunboat),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 89) \ #github-link("omdurman-rules/src/effects/effect.rs", 89)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L89")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HowitzerFire]]]], [#raw(" 87 │     /// - CRT result applied to units at impact hex (not the original target).
 88 │     /// - Firers marked as fired.
 89 │     HowitzerFire {
 90 │         attack: FireAttack,
 91 │         combat_results_table_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/fire.rs", 18) \ #github-link("omdurman-rules/src/effects/fire.rs", 18)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/fire.rs#L18")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_howitzer_fire]]]], [#raw(" 16 │ 
 17 │ /// Validate and apply a howitzer fire attack (scatter path) (rulebook §6.64).
 18 │ pub fn apply_howitzer_fire(
 19 │     state: &mut GameState,
 20 │     attack: &FireAttack,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/fire.rs", 49) \ #github-link("omdurman-rules/src/effects/state/fire.rs", 49)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/fire.rs#L49")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_fire_at]]]], [#raw(" 47 │     /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
 48 │     /// (Howitzer fire ignores LOS entirely -- §6.64.)
 49 │     pub fn can_fire_at(
 50 │         &self,
 51 │         firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::opposite_is_an_involution]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::step_toward_lands_on_an_adjacent_hex]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::scatter_is_center_exactly_for_rolls_7_to_10]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_on_target_7_to_10]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_scatters_below_7]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::howitzer_scatter::howitzer_each_miss_gets_its_printed_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::named_and_old_gunboats_resolve]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::named_gunboat_has_howitzer]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_scattered_howitzer_shell_hits_a_friendly_occupied_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_rejected_howitzer_shot_leaves_no_impact_record]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::named_gunboat_may_fire_howitzer_in_second_subphase]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::named_gunboat_direct_fire_uses_artillery_weapon]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::named_gunboat_no_howitzer_at_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::dervish_gunboat_lacks_howitzer]]]
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
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 163) \ #github-link("omdurman-rules/src/effects/effect.rs", 163)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L163")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw("161 │     /// **Postconditions:** Unit position moved to `to`; `vacated_by_combat`
162 │     /// entry consumed.
163 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
164 │ 
165 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 45) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 45)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L45")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_advance_after_combat]]]], [#raw(" 43 │         }
 44 │         GameEffect::AdvanceAfterCombat { unit_id, to } => {
 45 │             apply_advance_after_combat(state, *unit_id, *to)
 46 │         }
 47 │         GameEffect::RecoverUnit { unit_id } => apply_recover_unit(state, *unit_id),", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 255) \ #github-link("omdurman-types/src/lib.rs", 255)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L255")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_advance_after_combat]]]], [#raw("253 │ 
254 │     /// Whether advance-after-combat may *not* cross this side (§6.82, §7.6).
255 │     pub fn blocks_advance_after_combat(self) -> bool {
256 │         matches!(
257 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/melee.rs", 141) \ #github-link("omdurman-rules/src/effects/state/melee.rs", 141)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/melee.rs#L141")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("139 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
140 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
141 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
142 │         let unit = self.unit_or_err(unit_id)?;
143 │         // §6.7: there is no advance after combat as a result of defensive fire.", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::advance_phase_is_atomic]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::hexside_blocking_classifiers_are_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_advance_after_combat_rejects_wall_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_advance_after_combat_rejects_khor_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::advance_requires_combat_vacated_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::advance_requires_participation]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::rejected_advance_phase_keeps_vacated_windows]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_gunboat_is_not_offered_a_land_advance]]]
#v(0.3em)
#progress-bar(7, 8)
#heading(level: 1, "§7 – Melee Phase") <sect-7>
#heading(level: 2, "§7 – Melee Phase (chapter)")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Melee Phase]]
#v(0.5em)
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
  [#vscode-link("omdurman-rules/src/scalars.rs", 42) \ #github-link("omdurman-rules/src/scalars.rs", 42)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L42")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeFactor]]]], [#raw(" 40 │     /// Every possible value from the annotated counter set is a named variant.
 41 │     #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
 42 │     pub enum MeleeFactor {
 43 │         One = 1,
 44 │         Three = 3,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/scalars.rs", 53) \ #github-link("omdurman-rules/src/scalars.rs", 53)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L53")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeFactor::sum]]]], [#raw(" 51 │ impl MeleeFactor {
 52 │     /// Sum multiple melee factors (rulebook §7.1).
 53 │     pub fn sum<'a>(factors: impl IntoIterator<Item = &'a MeleeFactor>) -> u16 {
 54 │         factors.into_iter().map(|f| f.value()).sum()
 55 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 972) \ #github-link("omdurman-types/src/lib.rs", 972)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L972")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_be_melee_attacked]]]], [#raw("970 │ 
971 │     /// Gunboats neither attack nor are attacked in melee (§7.1).
972 │     pub fn may_be_melee_attacked(self) -> bool {
973 │         !matches!(self, UnitKind::Gunboat { .. })
974 │     }", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::unit_kind_melee_capability_law_is_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::melee_rejects_duplicate_attackers_and_partial_defenders]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::single_shot_melee_is_validated_like_a_declaration]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_path_may_not_pass_through_an_enemy_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::melee_factor_values_and_sum]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::unit_kind_melee_capability]]]
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
  [#vscode-link("omdurman-types/src/lib.rs", 244) \ #github-link("omdurman-types/src/lib.rs", 244)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L244")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_melee]]]], [#raw("242 │     /// Effects Chart, Khor: \"May not melee across\"). Gates and breaches are
243 │     /// passable to melee.
244 │     pub fn blocks_melee(self) -> bool {
245 │         matches!(
246 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/melee.rs", 13) \ #github-link("omdurman-rules/src/effects/state/melee.rs", 13)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/melee.rs#L13")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_melee]]]], [#raw(" 11 │     /// that may be melee-attacked (gunboats may not -- §7.1), and no wall or
 12 │     /// thorn-hedge hexside blocks the attack (§7.2).
 13 │     pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
 14 │         let unit = self.unit_or_err(attacker)?;
 15 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::hexside_blocking_classifiers_are_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_melee_gates_phase_adjacency_and_kind]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_melee_rejects_wall_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_melee_rejects_thorn_hedge_hexside]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_melee_allows_gate_hexside]]]
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
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 112) \ #github-link("omdurman-rules/src/effects/effect.rs", 112)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L112")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeCombat]]]], [#raw("110 │     /// - Winner may advance into vacated hex (§7.6).
111 │     /// - Victory points awarded for eliminations.
112 │     MeleeCombat {
113 │         attack: MeleeAttack,
114 │         attacker_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/melee.rs", 6) \ #github-link("omdurman-rules/src/effects/melee.rs", 6)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/melee.rs#L6")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_melee_combat]]]], [#raw("  4 │ /// Validated exactly like a declaration ([`validate_melee_attack`]), then
  5 │ /// resolved at once (no §7.5 reaction window).
  6 │ pub fn apply_melee_combat(
  7 │     state: &mut GameState,
  8 │     attack: &MeleeAttack,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::melee_resolves_simultaneously]]]
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
  [#vscode-link("omdurman-types/src/lib.rs", 961) \ #github-link("omdurman-types/src/lib.rs", 961)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L961")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_melee_attack]]]], [#raw("959 │     /// Rulebook §7.4 -- only infantry, cavalry, camel and Dervish leaders may
960 │     /// melee *attack*. All others (except gunboats) may melee *defend* (§7.1).
961 │     pub fn may_melee_attack(self) -> bool {
962 │         matches!(
963 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 898) \ #github-link("omdurman-types/src/lib.rs", 898)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L898")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitKind]]]], [#raw("896 │ /// `Some(UnitKind::Marker)` or `None`.
897 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
898 │ pub enum UnitKind {
899 │     /// Foot infantry (§2.3): fire / melee / movement.
900 │     Infantry {", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 738) \ #github-link("omdurman-types/src/lib.rs", 738)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L738")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishTribe]]]], [#raw("736 │     strum::EnumIter,
737 │ )]
738 │ pub enum DervishTribe {
739 │     Baggara,
740 │     Jaalin,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/melee.rs", 13) \ #github-link("omdurman-rules/src/effects/state/melee.rs", 13)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/melee.rs#L13")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_melee]]]], [#raw(" 11 │     /// that may be melee-attacked (gunboats may not -- §7.1), and no wall or
 12 │     /// thorn-hedge hexside blocks the attack (§7.2).
 13 │     pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
 14 │         let unit = self.unit_or_err(attacker)?;
 15 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::unit_kind_melee_capability_law_is_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::can_melee_gates_phase_adjacency_and_kind]]]
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
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 149) \ #github-link("omdurman-rules/src/effects/effect.rs", 149)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L149")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RetreatBeforeMelee]]]], [#raw("147 │     ///
148 │     /// **Postconditions:** Unit position moved to `to`.
149 │     RetreatBeforeMelee { unit_id: UnitId, to: HexCoord },
150 │ 
151 │     /// An attacking unit advances into a hex vacated by combat (rulebook §6.82", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 42) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 42)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L42")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_retreat_before_melee]]]], [#raw(" 40 │         GameEffect::ResolveMelee => apply_resolve_melee(state),
 41 │         GameEffect::RetreatBeforeMelee { unit_id, to } => {
 42 │             apply_retreat_before_melee(state, *unit_id, *to)
 43 │         }
 44 │         GameEffect::AdvanceAfterCombat { unit_id, to } => {", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 978) \ #github-link("omdurman-types/src/lib.rs", 978)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L978")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[may_retreat_before_melee]]]], [#raw("976 │     /// Cavalry and camel units may retreat two hexes from an infantry melee
977 │     /// attack (§7.5).
978 │     pub fn may_retreat_before_melee(self) -> bool {
979 │         matches!(self, UnitKind::Cavalry { .. } | UnitKind::Camel { .. })
980 │     }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/scalars.rs", 126) \ #github-link("omdurman-rules/src/scalars.rs", 126)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L126")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HexDistance]]]], [#raw("124 │ /// (rulebook §6.22, §7.5).
125 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
126 │ pub struct HexDistance(pub(crate) u16);
127 │ 
128 │ impl HexDistance {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/melee.rs", 72) \ #github-link("omdurman-rules/src/effects/state/melee.rs", 72)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/melee.rs#L72")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_retreat_before_melee]]]], [#raw(" 70 │     /// two hexes away and empty. (Does not verify the attacker is infantry --
 71 │     /// the caller offers the retreat only in response to one.)
 72 │     pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
 73 │         let unit = self.unit_or_err(unit_id)?;
 74 │         if !matches!(self.phase, Phase::Melee) {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::resolve_melee_is_atomic]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::advance_phase_is_atomic]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::unit_kind_melee_capability_law_is_exact]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::retreat_before_melee_only_cavalry_two_hexes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::retreat_opens_window_only_when_hex_empties]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::rejected_resolve_melee_keeps_the_declaration]]]
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
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 163) \ #github-link("omdurman-rules/src/effects/effect.rs", 163)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L163")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[AdvanceAfterCombat]]]], [#raw("161 │     /// **Postconditions:** Unit position moved to `to`; `vacated_by_combat`
162 │     /// entry consumed.
163 │     AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },
164 │ 
165 │     // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 45) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 45)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L45")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_advance_after_combat]]]], [#raw(" 43 │         }
 44 │         GameEffect::AdvanceAfterCombat { unit_id, to } => {
 45 │             apply_advance_after_combat(state, *unit_id, *to)
 46 │         }
 47 │         GameEffect::RecoverUnit { unit_id } => apply_recover_unit(state, *unit_id),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/melee.rs", 141) \ #github-link("omdurman-rules/src/effects/state/melee.rs", 141)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/melee.rs#L141")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_advance_after_combat]]]], [#raw("139 │     /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
140 │     /// Wall/khor hexside restrictions are not enforced (no hexside map data).
141 │     pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
142 │         let unit = self.unit_or_err(unit_id)?;
143 │         // §6.7: there is no advance after combat as a result of defensive fire.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/melee.rs", 6) \ #github-link("omdurman-rules/src/effects/melee.rs", 6)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/melee.rs#L6")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_melee_combat]]]], [#raw("  4 │ /// Validated exactly like a declaration ([`validate_melee_attack`]), then
  5 │ /// resolved at once (no §7.5 reaction window).
  6 │ pub fn apply_melee_combat(
  7 │     state: &mut GameState,
  8 │     attack: &MeleeAttack,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::advance_phase_is_atomic]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::advancing_consumes_the_units_eligibility]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::dervish_advance_after_melee_is_mandatory]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::rejected_advance_phase_keeps_vacated_windows]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::melee_mandatory_advance_leaders_are_free_stacking]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_lone_gordon_makes_no_melee_roll]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::melee_advance_overruns_a_lone_ae_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mandatory_advance_stops_at_an_enemy_fort]]]
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
  [#vscode-link("omdurman-rules/src/combat.rs", 108) \ #github-link("omdurman-rules/src/combat.rs", 108)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L108")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier]]]], [#raw("106 │ 
107 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
108 │ pub enum MeleeModifier {
109 │     /// +2 to all Dervish melee rolls (§7.7).
110 │     DervishStandard,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/melee.rs", 421) \ #github-link("omdurman-rules/src/effects/melee.rs", 421)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/melee.rs#L421")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[mandatory_melee_modifiers]]]], [#raw("419 │ /// `(attacker_modifiers, defender_modifiers)`; the engine applies exactly
420 │ /// these at resolution and rejects a declared attack whose lists differ.
421 │ pub fn mandatory_melee_modifiers(
422 │     state: &GameState,
423 │     attack: &MeleeAttack,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat.rs", 136) \ #github-link("omdurman-rules/src/combat.rs", 136)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L136")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeAttack]]]], [#raw("134 │ /// A melee attack: simultaneous, both sides roll on the Combat Results Table (§7.3, §7.7).
135 │ #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
136 │ pub struct MeleeAttack {
137 │     pub attacker_player: Player,
138 │     pub attacker_hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat.rs", 112) \ #github-link("omdurman-rules/src/combat.rs", 112)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L112")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::AngloEgyptianStandard]]]], [#raw("110 │     DervishStandard,
111 │     /// +1 to all Anglo-Egyptian melee rolls (§7.7).
112 │     AngloEgyptianStandard,
113 │     /// Inverted to -2 when Dervish units melee-attack across a trench into
114 │     /// an entrenched defender (§9.232).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat.rs", 115) \ #github-link("omdurman-rules/src/combat.rs", 115)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L115")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::DervishVsTrenchedDefender]]]], [#raw("113 │     /// Inverted to -2 when Dervish units melee-attack across a trench into
114 │     /// an entrenched defender (§9.232).
115 │     DervishVsTrenchedDefender,
116 │     /// +2: the Anglo-Egyptian \"Friendlies\" melee with the Dervish melee
117 │     /// modifier (§6.52) -- in place of the +1 standard when every", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat.rs", 110) \ #github-link("omdurman-rules/src/combat.rs", 110)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L110")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MeleeModifier::DervishStandard]]]], [#raw("108 │ pub enum MeleeModifier {
109 │     /// +2 to all Dervish melee rolls (§7.7).
110 │     DervishStandard,
111 │     /// +1 to all Anglo-Egyptian melee rolls (§7.7).
112 │     AngloEgyptianStandard,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::melee_modifier_keeps_roll_legal]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::melee_resolves_simultaneously]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::melee_modifiers_are_engine_derived_and_mismatches_rejected]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::friendlies_melee_with_the_dervish_modifier]]]
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
  [#vscode-link("omdurman-rules/src/scalars.rs", 88) \ #github-link("omdurman-rules/src/scalars.rs", 88)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L88")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MovementAllowance::halve]]]], [#raw(" 86 │ impl MovementAllowance {
 87 │     /// Night movement allowance = halved (round down) (rulebook §8.1, §5.11).
 88 │     pub fn halve(self) -> Self {
 89 │         let v = self.value() / 2;
 90 │         MovementAllowance::try_from(v).expect(\"halved value always a named variant\")", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 805) \ #github-link("omdurman-types/src/lib.rs", 805)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L805")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DayNight]]]], [#raw("803 │ /// (rulebook §8.1).
804 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
805 │ pub enum DayNight {
806 │     Day,
807 │     Night,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/range_effects.rs", 71) \ #github-link("omdurman-rules/src/range_effects.rs", 71)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/range_effects.rs#L71")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[night_max_range]]]], [#raw(" 69 │ 
 70 │ /// The halved maximum range at night (§8.1): round down, minimum 1.
 71 │ pub fn night_max_range(weapon: WeaponClass, ae: bool) -> u8 {
 72 │     let day = max_day_range(weapon, ae);
 73 │     if day <= 1 { 1 } else { day / 2 }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/scalars.rs", 208) \ #github-link("omdurman-rules/src/scalars.rs", 208)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L208")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[effective_movement_at_night]]]], [#raw("206 │ /// Apply night-turn movement halving for Anglo-Egyptian units (§8.1): all
207 │ /// Anglo-Egyptian movement allowances are halved (round down).
208 │ pub fn effective_movement_at_night(
209 │     allowance: MovementAllowance,
210 │     player: Player,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/fire.rs", 112) \ #github-link("omdurman-rules/src/effects/state/fire.rs", 112)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/fire.rs#L112")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[NoHowitzerAtNight]]]], [#raw("110 │         // §6.64: no howitzer fire at night.
111 │         if kind == FireKind::Howitzer && self.day_night == DayNight::Night {
112 │             return Err(RuleError::NoHowitzerAtNight);
113 │         }
114 │ ", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::movement_allowance_halve_never_panics]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::night_halving_is_ae_only_and_day_neutral]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::night_cap_is_halved_day_max_floored_at_one]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::night_range_effects_gates_the_day_band_by_the_cap]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::fire::night_capped_distance_is_some_exactly_within_the_night_max]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::night_max_ranges]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::night_max_ranges_remaining]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::ae_rifle_at_night_matches_rulebook_example]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::range_effects::max_day_range_all_combos]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::night_movement_overlay_allowance_halved]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::named_gunboat_no_howitzer_at_night]]]
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
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 191) \ #github-link("omdurman-rules/src/effects/effect.rs", 191)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L191")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishDesertion]]]], [#raw("189 │     /// the effect. The Khalifa, gunboats, artillery, and forts may not be
190 │     /// chosen.
191 │     DervishDesertion {
192 │         roll: DieRoll,
193 │         deserters: Vec<UnitId>,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 366) \ #github-link("omdurman-rules/src/unit.rs", 366)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L366")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_desertion_exempt]]]], [#raw("364 │     /// Non-Dervish identities are trivially not eligible to desert and so are
365 │     /// reported as exempt too.
366 │     pub fn is_desertion_exempt(&self) -> bool {
367 │         match self {
368 │             UnitIdentity::DervishLeader(DervishLeader::KhalifaAbdullah)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 63) \ #github-link("omdurman-rules/src/turn_track.rs", 63)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L63")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishDesertion]]]], [#raw(" 61 │     None,
 62 │     /// Dervish desertion roll (§8.2) -- occurs on the first night turn.
 63 │     DervishDesertion,
 64 │     /// Dervish reinforcements are available.
 65 │     DervishReinforcements,", block: true, lang: "rs")],
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
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::desertion_count_is_one_and_a_half_times_the_roll]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::desertion_event_is_unique_to_the_first_night_turn]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::desertion_rejects_a_unit_listed_twice]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::desertion_count_is_floor_one_and_a_half]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::desertion_on_first_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::desertion_roll_required_before_first_night_movement_ends]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::desertion_removes_chosen_count_and_respects_exemptions]]]
#v(0.3em)
#progress-bar(23, 33)
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
#heading(level: 2, "§9.2 – The Historical Scenario") <sect-9-2>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Historical Scenario

Players should note that the historical scenario is an exercise in futility for the Dervish player. It is, however, an interesting demonstration of the absolute imbecility of the Khalifa's generalship and vividly shows the superiority of entrenched firepower over traditional tribal arms in the colonial period.]]
#v(0.5em)
#heading(level: 2, "§9.3 – Bonus Game: Fall of Khartoum") <sect-9-3>
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Bonus Game: FALL OF KHARTOUM]]
#v(0.5em)
#heading(level: 2, "§9.11 – Set Up (Campaign)") <sect-9-11>
#status-tag("descriptive")
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 243) \ #github-link("omdurman-rules/src/turn_track.rs", 243)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L243")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[TurnLabel]]]], [#raw("241 │ /// printed cell; `Blank` is for unused positions in the 9×3 grid.
242 │ #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
243 │ pub enum TurnLabel {
244 │     Blank,
245 │     /// \"SEPT. 1\\n6:00 am\" -- day header plus time (turn 1).", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::scenario_turn_indexing_is_exact_for_every_scenario]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::turn_labels_agree_with_the_rule_bearing_track]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::campaign_track_22_turns]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::desertion_on_first_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::campaign_track_label_and_day_night_agree]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::game_time_display_all_variants]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::turn_label_display]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::turn_label_out_of_range_is_none]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::game_over_after_campaign_turns]]]
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
  [#vscode-link("omdurman-rules/src/victory.rs", 16) \ #github-link("omdurman-rules/src/victory.rs", 16)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L16")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource]]]], [#raw(" 14 │ /// the manual and the engine.
 15 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 16 │ pub enum VpSource {
 17 │     // ----- Anglo-Egyptian player receives:
 18 │     /// 25 pts -- the Mahdi's Tomb taken from the Dervish: held at the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/victory.rs", 48) \ #github-link("omdurman-rules/src/victory.rs", 48)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L48")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource::points]]]], [#raw(" 46 │ impl VpSource {
 47 │     /// VP awarded to `who_scores()` (rulebook §9.14).
 48 │     pub fn points(self) -> VictoryPoints {
 49 │         match self {
 50 │             VpSource::MahdisTombTaken | VpSource::MahdisTombHeld => VictoryPoints::new(25),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/victory.rs", 63) \ #github-link("omdurman-rules/src/victory.rs", 63)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L63")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpSource::who_scores]]]], [#raw(" 61 │ 
 62 │     /// Which player receives these victory points (rulebook §9.14).
 63 │     pub fn who_scores(self) -> Player {
 64 │         match self {
 65 │             VpSource::MahdisTombTaken", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/victory.rs", 100) \ #github-link("omdurman-rules/src/victory.rs", 100)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L100")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger]]]], [#raw(" 98 │ /// Cumulative victory ledger for one scenario (rulebook §9.14).
 99 │ #[derive(Serialize, Deserialize, Clone, Debug, Default)]
100 │ pub struct VictoryLedger {
101 │     pub events: Vec<VpEvent>,
102 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/victory.rs", 106) \ #github-link("omdurman-rules/src/victory.rs", 106)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L106")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VpEvent]]]], [#raw("104 │ /// A single victory-point scoring event (rulebook §9.14).
105 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
106 │ pub struct VpEvent {
107 │     pub turn: GameTurnIndex,
108 │     pub source: VpSource,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/victory.rs", 113) \ #github-link("omdurman-rules/src/victory.rs", 113)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L113")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger::total_for]]]], [#raw("111 │ impl VictoryLedger {
112 │     /// Total victory points earned by a given player (rulebook §9.14).
113 │     pub fn total_for(&self, player: Player) -> VictoryPoints {
114 │         VictoryPoints(
115 │             self.events", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/victory.rs", 125) \ #github-link("omdurman-rules/src/victory.rs", 125)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L125")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryLedger::superiority]]]], [#raw("123 │     /// Net superiority: positive = Anglo-Egyptian ahead, negative = Dervish ahead
124 │     /// (rulebook §9.14).
125 │     pub fn superiority(&self) -> VictoryPoints {
126 │         VictoryPoints(
127 │             self.total_for(Player::AngloEgyptian).value() - self.total_for(Player::Dervish).value(),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/victory.rs", 151) \ #github-link("omdurman-rules/src/victory.rs", 151)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L151")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CampaignVictoryLevel]]]], [#raw("149 │ /// Campaign-game victory levels (§9.14).
150 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
151 │ pub enum CampaignVictoryLevel {
152 │     Draw,
153 │     Marginal(Player),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/victory.rs", 160) \ #github-link("omdurman-rules/src/victory.rs", 160)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L160")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CampaignVictoryLevel::from_superiority]]]], [#raw("158 │ impl CampaignVictoryLevel {
159 │     /// Assign a level from the net superiority (§9.14).
160 │     pub fn from_superiority(s: VictoryPoints) -> Self {
161 │         let net = s.0;
162 │         // Positive -> Anglo-Egyptian thresholds: 15/30/50", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/victory.rs", 75) \ #github-link("omdurman-rules/src/effects/victory.rs", 75)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/victory.rs#L75")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[score_elimination]]]], [#raw(" 73 │ /// elimination under `cause`. The owner is derived from the unit's identity,
 74 │ /// so unlike the historical signature there is no caller-supplied player.
 75 │ pub fn score_elimination(state: &mut GameState, unit_id: UnitId, cause: ElimCause) {
 76 │     if let Some(unit) = state.find_unit(unit_id) {
 77 │         let identity = unit.profile.identity;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/scalars.rs", 178) \ #github-link("omdurman-rules/src/scalars.rs", 178)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/scalars.rs#L178")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[VictoryPoints]]]], [#raw("176 │ /// (rulebook §9.14).
177 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
178 │ pub struct VictoryPoints(pub(crate) i32);
179 │ 
180 │ impl VictoryPoints {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::campaign_victory_levels_match_manual_superiority_table]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::vp_source_points_and_scorer_match_the_printed_schedule]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::victory::vp_source_for_routes_every_elimination_to_the_printed_source]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::victory::score_elimination_records_exactly_what_it_scores]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_destroyed_fort_takes_one_occupant_with_it_and_scores_it]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_mined_gunboat_scores_and_takes_its_friendlies_down]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_gunboat_sunk_by_artillery_scores_victory_points]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::friendlies_bank_scores_by_side]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mahdis_tomb_stays_dervish_without_a_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mahdis_tomb_scores_for_anglo_egyptian_when_taken]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::campaign_end_scores_an_untaken_tomb_for_the_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::vp_source_attributes]]]
#v(0.3em)
#heading(level: 2, "§9.21 – Set Up (Historical)") <sect-9-21>
#status-tag("descriptive")
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
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::scenario_turn_indexing_is_exact_for_every_scenario]]]
#v(0.3em)
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
  [#vscode-link("omdurman-rules/src/victory.rs", 190) \ #github-link("omdurman-rules/src/victory.rs", 190)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L190")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[HistoricalVictoryLevel]]]], [#raw("188 │ /// draw\").
189 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
190 │ pub enum HistoricalVictoryLevel {
191 │     Draw = 1,
192 │     Marginal = 2,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::historical_victory_ladders_match_manual_bands]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::historical_victory_level_for_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::historical_victory_level_for_anglo_egyptian]]]
#v(0.3em)
#heading(level: 2, "§9.31 – Bonus game map") <sect-9-31>
#status-tag("implemented")
#linebreak()
#text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only the small FALL OF KHARTOUM scenario map is used for this game.]]
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
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::tests::scenario_maps_to_board]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::tests::start_game_scenario_selects_board]]]
#v(0.3em)
#heading(level: 2, "§9.32 – Set Up (Bonus)") <sect-9-32>
#status-tag("descriptive")
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 173) \ #github-link("omdurman-rules/src/turn_track.rs", 173)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L173")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_TURN_TRACK]]]], [#raw("171 │ /// rule-bearing (night halves Anglo-Egyptian movement and ranges and bars
172 │ /// howitzer fire, §8.1).
173 │ pub const FALL_OF_KHARTOUM_TURN_TRACK: [TurnEntry; 8] = [
174 │     TurnEntry {
175 │         turn: 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::scenario_turn_indexing_is_exact_for_every_scenario]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::fall_of_khartoum_night_window_is_turns_one_and_two]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::fall_of_khartoum_turn_one_is_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::fall_of_khartoum_turns_2_to_8_are_day]]]
#v(0.3em)
#heading(level: 2, "§9.34 – Special Rules (Bonus)") <sect-9-34>
#status-tag("descriptive")
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
  [#vscode-link("omdurman-rules/src/victory.rs", 308) \ #github-link("omdurman-rules/src/victory.rs", 308)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/victory.rs#L308")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FoKVictoryLevel::resolve]]]], [#raw("306 │     /// rulebook: GORDON dies turn 5 (tactical) with 24 Dervish losses (−2
307 │     /// levels) nets a British marginal.
308 │     pub fn resolve(gordon_died_turn: Option<u8>, scenario_end_turn: u8, dervish_lost: i16) -> Self {
309 │         let base = Self::base(gordon_died_turn, scenario_end_turn);
310 │         let base_idx = Self::LADDER", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::fok_victory_ladder_penalties_shift_monotonically]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::gordon_dying_in_melee_ends_fall_of_khartoum]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::fok_victory_level_worked_example]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::fok_victory_level_gordon_died_early]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::fok_victory_level_late_gordon_death]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::tests::fok_victory_level_gordon_survived]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::newspaper::fok_headlines_never_contradict_gordons_fate]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/setup.rs", 105) \ #github-link("omdurman-rules/src/effects/state/setup.rs", 105)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/setup.rs#L105")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[setup_complete]]]], [#raw("103 │     /// currently shares the same \"both sides deployed\" gate; when a scenario
104 │     /// needs a different minimum, branch on `self.scenario` here.
105 │     pub fn setup_complete(&self) -> Result<(), RuleError> {
106 │         let has = |player| {
107 │             self.units", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::setup_is_sequential_first_side_then_second]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::campaign_has_no_fixed_placements]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::hadendowa_first_cell_is_isa_zachneih]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::campaign_deployment_is_boat_land_exclusive]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::campaign_setup_rejects_non_initial_force]]]
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
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 183) \ #github-link("omdurman-rules/src/effects/effect.rs", 183)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L183")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[PlaceReinforcements]]]], [#raw("181 │     // -- Reinforcement / placement -----------------------------------------
182 │     /// Place reinforcements onto the map (rulebook §9.112, §9.113).
183 │     PlaceReinforcements(Vec<UnitPlacement>),
184 │ 
185 │     // -- Scenario-specific -------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 53) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 53)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L53")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_place_reinforcements]]]], [#raw(" 51 │         GameEffect::Demolition { unit_id, target } => apply_demolition(state, *unit_id, *target),
 52 │         GameEffect::PlaceReinforcements(placements) => {
 53 │             apply_place_reinforcements(state, placements)
 54 │         }
 55 │         GameEffect::DervishDesertion { roll, deserters } => {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/reinforcements.rs", 69) \ #github-link("omdurman-rules/src/reinforcements.rs", 69)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/reinforcements.rs#L69")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_campaign_schedule]]]], [#raw(" 67 │ /// All reinforcements enter on the west edge, south of the Khor Shambat.
 68 │ /// Each unit pays terrain cost of the hex it enters through.
 69 │ pub fn dervish_campaign_schedule() -> ReinforcementSchedule {
 70 │     ReinforcementSchedule {
 71 │         player: Player::Dervish,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 538) \ #github-link("omdurman-types/src/lib.rs", 538)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L538")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Location]]]], [#raw("536 │ /// Named map landmarks (rulebook mapsheet, §9.111, §9.113, §9.212 scenarios).
537 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
538 │ pub enum Location {
539 │     FortMakran,
540 │     NorthFort,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 618) \ #github-link("omdurman-types/src/lib.rs", 618)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L618")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[SetupLetter]]]], [#raw("616 │ /// Each letter marks a specific hex where a Dervish leader is placed.
617 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
618 │ pub enum SetupLetter {
619 │     Y,
620 │     K,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 761) \ #github-link("omdurman-types/src/lib.rs", 761)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L761")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Faction]]]], [#raw("759 │ /// `Some(BrigadeId::friendlies())`.
760 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
761 │ pub enum Faction {
762 │     Dervish {
763 │         tribe: DervishTribe,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::wave_for_turn_answers_exactly_on_the_printed_schedule]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::tests::place_unit_translates_by_phase]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::dervish_schedule_has_three_waves]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::dervish_wave_one_has_baggaara_and_three_leaders]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::dervish_wave_two_has_hadendowa]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::dervish_wave_three_is_all_remaining]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::wave_for_turn_returns_correct_wave]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::campaign_reinforcements_gate_by_wave]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::campaign_setup_rejects_non_initial_force]]]
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
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::wave_for_turn_answers_exactly_on_the_printed_schedule]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::anglo_egyptian_schedule_has_four_waves]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::anglo_egyptian_leaders_available_each_wave]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::reinforcements::anglo_egyptian_turn_four_is_all_remaining]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::campaign_reinforcement_cap_and_double_entry]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::campaign_gunboats_quota_three_per_turn]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::reinforcement_rejected_onto_enemy_occupied_hex]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::campaign_setup_rejects_non_initial_force]]]
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
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 462) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 462)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L462")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[first_player]]]], [#raw("460 │ 
461 │ /// The player who moves first in a scenario (§4, §9.113, §9.212, §9.322).
462 │ pub fn first_player(scenario: Scenario) -> Player {
463 │     match scenario {
464 │         Scenario::Campaign => Player::AngloEgyptian,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::setup_is_sequential_first_side_then_second]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::historical_setup_rejects_not_in_play_units]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/setup.rs", 215) \ #github-link("omdurman-rules/src/effects/state/setup.rs", 215)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/setup.rs#L215")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[in_deployment_zone]]]], [#raw("213 │     ///   plan / UI rather than this hex predicate. Documented, not silently
214 │     ///   dropped.
215 │     pub fn in_deployment_zone(&self, player: Player, hex: HexCoord, is_boat: bool) -> bool {
216 │         // No board attached -> permissive (unit tests, unbound session).
217 │         if self.board.terrain.is_empty() {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 78) \ #github-link("omdurman-rules/src/unit.rs", 78)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L78")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[DervishLeader::setup_letter]]]], [#raw(" 76 │     /// (§9.212): A→Ali Wad Helu, D→Sheik El Din, Y→Yakub, K→Khalifa Abdullah,
 77 │     /// S→Sherif, O→Osman Digna. Inverse of [`dervish_leader_for_setup_letter`].
 78 │     pub fn setup_letter(self) -> SetupLetter {
 79 │         match self {
 80 │             DervishLeader::AliWadHelu => SetupLetter::A,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 94) \ #github-link("omdurman-rules/src/unit.rs", 94)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L94")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[dervish_leader_for_setup_letter]]]], [#raw(" 92 │ /// inherent impl here, so the mapping is a free function -- the bijective
 93 │ /// inverse of [`DervishLeader::setup_letter`].
 94 │ pub fn dervish_leader_for_setup_letter(letter: SetupLetter) -> DervishLeader {
 95 │     match letter {
 96 │         SetupLetter::A => DervishLeader::AliWadHelu,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::deploy_rejected_outside_zone]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::embedded_leaders_resolve_from_their_host_section]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::historical_places_all_six_leaders_when_anchors_present]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::missing_anchor_is_reported_not_dropped_silently]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::setup_letter_dervish_leader_roundtrip]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::setup_letter_to_dervish_leader_known_values]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::historical_setup_rejects_not_in_play_units]]]
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
  [#vscode-link("omdurman-rules/src/combat.rs", 28) \ #github-link("omdurman-rules/src/combat.rs", 28)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L28")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw(" 26 │     Terrain(i16),
 27 │     /// -2 thorn-hedge defensive modifier (§9.231).
 28 │     ZaribaThornHedge,
 29 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
 30 │     /// units (those Nile-side of the trench hexside).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 213) \ #github-link("omdurman-types/src/lib.rs", 213)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L213")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaThornHedge]]]], [#raw("211 │     Crest,
212 │     /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
213 │     ZaribaThornHedge,
214 │     /// Historical-scenario trench segment of the Zariba (§9.232).
215 │     ZaribaTrench,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state.rs", 318) \ #github-link("omdurman-rules/src/effects/state.rs", 318)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state.rs#L318")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[has_zariba_thorn_hedge]]]], [#raw("316 │     /// *effective* hexsides, so constructed (§5.3) and authored hedges both
317 │     /// count.
318 │     pub fn has_zariba_thorn_hedge(&self, hex: HexCoord) -> bool {
319 │         for n in hex.neighbors() {
320 │             if self.hexside_effective_is(hex, n, |k| k == HexsideKind::ZaribaThornHedge) {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zariba_fire_penalties_apply_to_dervish_fire_only]]]
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
  [#vscode-link("omdurman-rules/src/combat.rs", 31) \ #github-link("omdurman-rules/src/combat.rs", 31)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L31")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrenchEntrenched]]]], [#raw(" 29 │     /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
 30 │     /// units (those Nile-side of the trench hexside).
 31 │     ZaribaTrenchEntrenched,
 32 │ }
 33 │ ", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 215) \ #github-link("omdurman-types/src/lib.rs", 215)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L215")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ZaribaTrench]]]], [#raw("213 │     ZaribaThornHedge,
214 │     /// Historical-scenario trench segment of the Zariba (§9.232).
215 │     ZaribaTrench,
216 │     /// One of the two end hexsides of a Zariba trench segment that connect to
217 │     /// the Nile River (§9.233).  Units may only enter/leave the Zariba via", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::melee_modifier_keeps_roll_legal]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::trench_entrenched_units_take_trench_modifiers]]]
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
  [#vscode-link("omdurman-types/src/lib.rs", 272) \ #github-link("omdurman-types/src/lib.rs", 272)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L272")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[blocks_movement]]]], [#raw("270 │     /// `omdurman-rules`). The trench *end* variants are therefore intentionally
271 │     /// not blocking.
272 │     pub fn blocks_movement(self) -> bool {
273 │         matches!(
274 │             self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state.rs", 331) \ #github-link("omdurman-rules/src/effects/state.rs", 331)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state.rs#L331")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[zariba_entry_surcharge]]]], [#raw("329 │     /// +2 movement points to cross\"). Trench ends are authored FoK geography
330 │     /// (players cannot construct them), so this reads the authored board.
331 │     pub fn zariba_entry_surcharge(&self, from: HexCoord, to: HexCoord) -> i16 {
332 │         match self.board.hexside_between(from, to) {
333 │             Some(k) if k.is_zariba_trench_end() => 2,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/movement.rs", 437) \ #github-link("omdurman-rules/src/effects/state/movement.rs", 437)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/movement.rs#L437")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[movement_cost_for]]]], [#raw("435 │     ///
436 │     /// §5.42: entering or leaving an enemy ZOC adds no MP cost.
437 │     pub fn movement_cost_for(
438 │         &self,
439 │         unit: &UnitPlacement,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/movement.rs", 33) \ #github-link("omdurman-rules/src/effects/movement.rs", 33)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/movement.rs#L33")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_move_unit]]]], [#raw(" 31 │ /// engine from the board (§5.11/§5.24): the caller-supplied `cost` is ignored
 32 │ /// (it survives in the effect only for wire compatibility).
 33 │ pub fn apply_move_unit(
 34 │     state: &mut GameState,
 35 │     unit_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-types::src::lib::zariba_trench_ends_differ_only_in_the_entry_rule]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zariba_end_hexside_costs_extra_mp]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::zariba_thorn_hedge_blocks_movement]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::picker::movement::trench_end_crossing_costs_two_extra_mp]]]
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
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::setup_is_sequential_first_side_then_second]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::printed_fok_forts_are_british_forts]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fall_of_khartoum_has_no_maxim_howitzer_subphase]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::confirm_ready_rejected_below_scenario_target]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::remove_deployed_unit_happy_path]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_places_gordon_in_the_palace]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_reports_missing_palace]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_fort_landmarks_sit_at_the_correct_hexes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_ae_gunboat_deploys_only_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_ae_land_unit_rejected_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_setup_complete_requires_full_oob]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::deploy_via_real_sprite_resolution_matches_engine]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::british_boats_named_vs_old_gunboat_detection]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_order_of_battle_british]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_order_of_battle_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_caps_bind_across_counter_variants]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_ae_may_set_up_behind_a_gate]]]
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
  [#vscode-link("omdurman-rules/src/unit_profiles.rs", 392) \ #github-link("omdurman-rules/src/unit_profiles.rs", 392)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit_profiles.rs#L392")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[ali_wad_helu]]]], [#raw("390 │ /// §9.2/§9.3) unplaceable -- the FoK order-of-battle table
391 │ /// (`fok_cap_group`) is keyed by identity.
392 │ pub fn ali_wad_helu(col: u32, row: u32) -> Option<Classification> {
393 │     match (col, row) {
394 │         (0, 0) => dervish_leader(DervishLeader::AliWadHelu),", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 862) \ #github-link("omdurman-types/src/lib.rs", 862)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L862")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[sections_for_picker]]]], [#raw("860 │     /// counter of those two forces and leave the §9.322 entry force -- and
861 │     /// with it the setup Ready gate (§9.2/§9.3) -- unplaceable.
862 │     pub fn sections_for_picker(self) -> Option<&'static [SectionName]> {
863 │         match self {
864 │             Scenario::Campaign | Scenario::Historical => None,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::deployment_requires_the_canonical_counter]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_reinforcements_need_the_owners_movement_phase_and_unique_ids]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::ali_wad_helu_block_resolves_leader_and_degelim_tribes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::tests::fok_setup_flow::degheim_counters_resolve_to_the_degheim_tribe]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_setup_complete_requires_full_oob]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_dervish_land_unit_rejected_on_nile]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-types::src::lib::fok_picker_allowlist_has_dervish_entry_force_blocks]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_order_of_battle_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_dervish_east_edge_on_diamond_board]]]
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
  [#vscode-link("omdurman-rules/src/turn_track.rs", 173) \ #github-link("omdurman-rules/src/turn_track.rs", 173)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn_track.rs#L173")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FALL_OF_KHARTOUM_TURN_TRACK]]]], [#raw("171 │ /// rule-bearing (night halves Anglo-Egyptian movement and ranges and bars
172 │ /// howitzer fire, §8.1).
173 │ pub const FALL_OF_KHARTOUM_TURN_TRACK: [TurnEntry; 8] = [
174 │     TurnEntry {
175 │         turn: 1,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::scenario_turn_indexing_is_exact_for_every_scenario]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::fall_of_khartoum_night_window_is_turns_one_and_two]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::fall_of_khartoum_turn_one_is_night]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::turn_track::fall_of_khartoum_turns_2_to_8_are_day]]]
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
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fall_of_khartoum_board_excludes_no_hexes]]]
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
  [#vscode-link("omdurman-rules/src/effects/fire.rs", 78) \ #github-link("omdurman-rules/src/effects/fire.rs", 78)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/fire.rs#L78")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[range_band_for]]]], [#raw(" 76 │ /// in FALL OF KHARTOUM *both* players use the Dervish Range Effects Table
 77 │ /// (§9.343).
 78 │ pub fn range_band_for(
 79 │     scenario: Scenario,
 80 │     player: Player,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::fire::range_band_for_routes_to_the_right_faction_table]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::fire::range_table_player_for_routes_friendlies_and_fok_to_the_dervish_table]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_both_players_use_dervish_range_table]]]
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
  [#vscode-link("omdurman-rules/src/effects/state.rs", 425) \ #github-link("omdurman-rules/src/effects/state.rs", 425)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state.rs#L425")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[hex_has_enemy_fort]]]], [#raw("423 │     /// positions (§9.321), while the North Fort is the Dervish one (§9.344,
424 │     /// represented by its fort counter).
425 │     pub fn hex_has_enemy_fort(&self, hex: HexCoord, mover: Player) -> bool {
426 │         self.printed_fort_owner(hex)
427 │             .is_some_and(|owner| owner != mover)", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_places_gordon_in_the_palace]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_fort_landmarks_sit_at_the_correct_hexes]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::placement_done_gate_matches_by_identity_not_allocated_id]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_order_of_battle_dervish]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-types::src::lib::fok_picker_allowlist_has_dervish_entry_force_blocks]]]
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
  [#vscode-link("omdurman-rules/src/effects/state.rs", 404) \ #github-link("omdurman-rules/src/effects/state.rs", 404)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state.rs#L404")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[is_nile_mouth_crossing]]]], [#raw("402 │     /// must be named on the board, else this is `false` and the move falls
403 │     /// through to the ordinary contiguous-Nile rules.
404 │     pub fn is_nile_mouth_crossing(&self, from: HexCoord, to: HexCoord) -> bool {
405 │         let white = self
406 │             .board", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 554) \ #github-link("omdurman-types/src/lib.rs", 554)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-types/src/lib.rs#L554")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[Location::WhiteNileMouth]]]], [#raw("552 │     /// The off-board mouth of the White Nile branch (FALL OF KHARTOUM §9.345) --
553 │     /// a British gunboat may cross to the Blue Nile mouth for 6 upstream MP.
554 │     WhiteNileMouth,
555 │     /// The off-board mouth of the Blue Nile branch (FALL OF KHARTOUM §9.345).
556 │     BlueNileMouth,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fok_gunboat_crosses_between_nile_mouths]]]
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
  [#vscode-link("omdurman-rules/src/effects/dispatch.rs", 603) \ #github-link("omdurman-rules/src/effects/dispatch.rs", 603)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/dispatch.rs#L603")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[check_gordon_palace]]]], [#raw("601 │ /// (overrun in passing, §6.51). A no-op outside FoK or while he lives
602 │ /// unthreatened.
603 │ pub fn check_gordon_palace(state: &mut GameState) {
604 │     if state.scenario != Scenario::FallOfKhartoum {
605 │         return;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/unit.rs", 332) \ #github-link("omdurman-rules/src/unit.rs", 332)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/unit.rs#L332")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[UnitIdentity::is_gordon]]]], [#raw("330 │     /// Whether this is the GORDON leader unit (§9.32, §9.346) -- the immobile
331 │     /// palace defender whose elimination ends FALL OF KHARTOUM (§9.35).
332 │     pub fn is_gordon(&self) -> bool {
333 │         matches!(
334 │             self,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::breach_casualty_stands_at_the_wall]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::unit_profiles::gordon_is_an_immobile_british_leader]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::gordon_survives_means_no_elimination]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::gordon_dying_in_melee_ends_fall_of_khartoum]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::fire_never_kills_gordon]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_lone_gordon_makes_no_melee_roll]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-app::src::scenario_setup::fall_of_khartoum_places_gordon_in_the_palace]]]
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
  [#vscode-link("omdurman-rules/src/effects/setup.rs", 350) \ #github-link("omdurman-rules/src/effects/setup.rs", 350)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/setup.rs#L350")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_place_mine]]]], [#raw("348 │ /// Lay a river mine during setup (§10.11). Validated by
349 │ /// [`GameState::can_place_mine`].
350 │ pub fn apply_place_mine(state: &mut GameState, hex: HexCoord) -> Result<(), RuleError> {
351 │     state.can_place_mine(hex)?;
352 │     state.mines.push(MinePlacement {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state.rs", 212) \ #github-link("omdurman-rules/src/effects/state.rs", 212)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state.rs#L212")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[GameState::mines]]]], [#raw("210 │             friendlies_transport: None,
211 │             optional_rules: Vec::new(),
212 │             mines: Vec::new(),
213 │             chain: None,
214 │             board: Arc::new(BoardInfo::default()),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn.rs", 58) \ #github-link("omdurman-rules/src/turn.rs", 58)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/turn.rs#L58")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[OptionalRule]]]], [#raw(" 56 │ /// two should be in play (rulebook §10).
 57 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 58 │ pub enum OptionalRule {
 59 │     RiverMines,
 60 │     RiverChain,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mine_and_chain_limits_enforced_in_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mines_and_chain_require_their_optional_rule]]]
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
  [#vscode-link("omdurman-rules/src/effects/effect.rs", 201) \ #github-link("omdurman-rules/src/effects/effect.rs", 201)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/effect.rs#L201")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[RiverMine]]]], [#raw("199 │     // -- Optional rules ----------------------------------------------------
200 │     /// River mine resolution (rulebook §10.12).
201 │     RiverMine {
202 │         gunboat_id: UnitId,
203 │         hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/river.rs", 128) \ #github-link("omdurman-rules/src/effects/river.rs", 128)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/river.rs#L128")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("126 │ 
127 │ /// Apply a river-mine resolution (rulebook §10.12).
128 │ pub fn apply_river_mine(
129 │     state: &mut GameState,
130 │     gunboat_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/transport.rs", 63) \ #github-link("omdurman-rules/src/transport.rs", 63)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/transport.rs#L63")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MineResult]]]], [#raw(" 61 │ /// British gunboat enters a mined hex.
 62 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 63 │ pub enum MineResult {
 64 │     /// Roll 1-4: no effect.
 65 │     NoEffect,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::mine_result_bands_match_the_printed_rule]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::river::river_mine_triggers_even_on_a_harmless_roll]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::river::river_mine_engine_loss_arms_the_drift_flag]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::river::river_mine_sinking_removes_the_gunboat]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::a_mined_gunboat_scores_and_takes_its_friendlies_down]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mine_fires_once_and_spares_dervish]]]
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
  [#vscode-link("omdurman-rules/src/effects/river.rs", 128) \ #github-link("omdurman-rules/src/effects/river.rs", 128)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/river.rs#L128")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("126 │ 
127 │ /// Apply a river-mine resolution (rulebook §10.12).
128 │ pub fn apply_river_mine(
129 │     state: &mut GameState,
130 │     gunboat_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::river::river_mine_triggers_even_on_a_harmless_roll]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::river::a_triggered_mine_never_fires_again]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mine_fires_once_and_spares_dervish]]]
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
  [#vscode-link("omdurman-rules/src/effects/river.rs", 128) \ #github-link("omdurman-rules/src/effects/river.rs", 128)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/river.rs#L128")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_river_mine]]]], [#raw("126 │ 
127 │ /// Apply a river-mine resolution (rulebook §10.12).
128 │ pub fn apply_river_mine(
129 │     state: &mut GameState,
130 │     gunboat_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::effects::river::dervish_gunboats_pass_mined_hexes_unharmed]]]
#v(0.3em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mine_fires_once_and_spares_dervish]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/setup.rs", 531) \ #github-link("omdurman-rules/src/effects/state/setup.rs", 531)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/setup.rs#L531")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_place_chain]]]], [#raw("529 │     /// Read-only check of a river-chain placement in setup (§10.21): Setup phase
530 │     /// and at most [`MAX_CHAIN_HEXES`] hexes.
531 │     pub fn can_place_chain(&self, hexes: &[HexCoord]) -> Result<(), RuleError> {
532 │         self.require_setup_phase()?;
533 │         self.require_setup_turn(Player::Dervish)?;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/setup.rs", 361) \ #github-link("omdurman-rules/src/effects/setup.rs", 361)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/setup.rs#L361")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_place_chain]]]], [#raw("359 │ /// Lay (or replace) the river chain during setup (§10.21). Validated by
360 │ /// [`GameState::can_place_chain`].
361 │ pub fn apply_place_chain(state: &mut GameState, hexes: &[HexCoord]) -> Result<(), RuleError> {
362 │     state.can_place_chain(hexes)?;
363 │     state.chain = Some(ChainPlacement {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects/state/setup.rs", 91) \ #github-link("omdurman-rules/src/effects/state/setup.rs", 91)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/setup.rs#L91")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[MAX_CHAIN_HEXES]]]], [#raw(" 89 │ 
 90 │ /// Maximum contiguous Nile hexes the river chain may span (§10.21).
 91 │ pub const MAX_CHAIN_HEXES: usize = 4;
 92 │ 
 93 │ impl GameState {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mine_and_chain_limits_enforced_in_setup]] #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::mines_and_chain_require_their_optional_rule]]]
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
  [#vscode-link("omdurman-rules/src/effects/state/movement.rs", 468) \ #github-link("omdurman-rules/src/effects/state/movement.rs", 468)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/state/movement.rs#L468")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[can_move_gunboat]]]], [#raw("466 │     /// The engine computes the cost (one MP per hex, a flat 6 for the §9.345
467 │     /// mouth crossing); a larger caller `cost` must also fit.
468 │     pub fn can_move_gunboat(
469 │         &self,
470 │         unit_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::chain_stops_gunboat_until_sunk]]]
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
  [#vscode-link("omdurman-rules/src/effects/river.rs", 173) \ #github-link("omdurman-rules/src/effects/river.rs", 173)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/effects/river.rs#L173")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[apply_sink_chain]]]], [#raw("171 │ /// Sink the river chain (rulebook §10.23). Marks the placed chain cleared so it
172 │ /// no longer stops gunboats (§10.22).
173 │ pub fn apply_sink_chain(state: &mut GameState) -> Result<(), RuleError> {
174 │     match state.chain.as_mut() {
175 │         Some(chain) if !chain.sunk => {", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Covered by tests: #box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[omdurman-rules::src::effects::tests::chain_stops_gunboat_until_sunk]]]
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
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 10) \ #github-link("omdurman-rules/src/combat_results_table.rs", 10)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat_results_table.rs#L10")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[FireFactorRow]]]], [#raw("  8 │     serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug,
  9 │ )]
 10 │ pub enum FireFactorRow {
 11 │     /// 1-5 factors
 12 │     Row01to05,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 46) \ #github-link("omdurman-rules/src/combat_results_table.rs", 46)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat_results_table.rs#L46")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[from_total]]]], [#raw(" 44 │ 
 45 │     /// Determine which row a given total fire factor falls into (rulebook §6.22).
 46 │     pub fn from_total(total: u16) -> Self {
 47 │         match total {
 48 │             0..=5 => FireFactorRow::Row01to05,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 88) \ #github-link("omdurman-rules/src/combat_results_table.rs", 88)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat_results_table.rs#L88")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[combat_results_table]]]], [#raw(" 86 │ /// D = `Disrupt` (1/2 of target units, round up)
 87 │ /// 1...5 = `Eliminate(n)` (that many units removed)
 88 │ pub fn combat_results_table(row: FireFactorRow, roll: DieRoll) -> CombatResult {
 89 │     crate::tables_data::CRT[row.index()][(roll.value() - 1) as usize]
 90 │ }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat.rs", 97) \ #github-link("omdurman-rules/src/combat.rs", 97)],  [#link("https://github.com/barafael/omdurman/blob/HEAD/omdurman-rules/src/combat.rs#L97")[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[CombatResult]]]], [#raw(" 95 │ /// * `--` -- no effect
 96 │ #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
 97 │ pub enum CombatResult {
 98 │     NoEffect,
 99 │     Disrupt,", block: true, lang: "rs")],
)
#v(0.5em)
#text(size: 9pt, fill: luma(80))[Proven by: #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::verification::disrupt_half_is_rounded_up]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::crt_eliminate_count_stays_within_printed_bounds]] #box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[omdurman-rules::src::combat_results_table::crt_is_monotone_in_the_die_roll_for_every_row]]]
#v(0.3em)
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
  [#text(weight: "bold", size: 9pt)[EnemyCohabitation]], [#link(<sect-5-51>)[§5.51]],
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
  [#text(weight: "bold", size: 9pt)[NoHowitzerAtNight]], [#link(<sect-8-1>)[§8.1]],
  [#text(weight: "bold", size: 9pt)[Old]], [#link(<sect-2-32>)[§2.32]],
  [#text(weight: "bold", size: 9pt)[OldGunboat]], [#link(<sect-2-32>)[§2.32]],
  [#text(weight: "bold", size: 9pt)[OptionalRule]], [#link(<sect-10-11>)[§10.11]],
  [#text(weight: "bold", size: 9pt)[OverLimit]], [#link(<sect-5-51>)[§5.51]],
  [#text(weight: "bold", size: 9pt)[PendingMelee]], [#link(<sect-4>)[§4]],
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
  [#text(weight: "bold", size: 9pt)[UnitKind]], [#link(<sect-7-4>)[§7.4]],
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
  [#text(weight: "bold", size: 9pt)[apply_melee_combat]], [#link(<sect-7-3>)[§7.3], #link(<sect-7-6>)[§7.6]],
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
  [#text(weight: "bold", size: 9pt)[build_fire_attack_from]], [#link(<sect-6-13>)[§6.13], #link(<sect-6-15>)[§6.15]],
  [#text(weight: "bold", size: 9pt)[can_advance_after_combat]], [#link(<sect-6-7>)[§6.7], #link(<sect-6-82>)[§6.82], #link(<sect-7-6>)[§7.6]],
  [#text(weight: "bold", size: 9pt)[can_fire_at]], [#link(<sect-6-14>)[§6.14], #link(<sect-6-61>)[§6.61], #link(<sect-6-62>)[§6.62], #link(<sect-6-64>)[§6.64]],
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
  [#text(weight: "bold", size: 9pt)[command_owns_unit]], [#link(<sect-1-1>)[§1.1]],
  [#text(weight: "bold", size: 9pt)[constructing_zariba]], [#link(<sect-5-3>)[§5.3]],
  [#text(weight: "bold", size: 9pt)[defense_modifier]], [#link(<sect-6-23>)[§6.23]],
  [#text(weight: "bold", size: 9pt)[demolishing]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[demolition_targets]], [#link(<sect-6-53>)[§6.53]],
  [#text(weight: "bold", size: 9pt)[dervish_campaign_schedule]], [#link(<sect-9-112>)[§9.112]],
  [#text(weight: "bold", size: 9pt)[dervish_leader_for_setup_letter]], [#link(<sect-9-212>)[§9.212]],
  [#text(weight: "bold", size: 9pt)[dervish_range_effects]], [#link(<sect-6-22>)[§6.22]],
  [#text(weight: "bold", size: 9pt)[dervish_stacking_group]], [#link(<sect-5-52>)[§5.52]],
  [#text(weight: "bold", size: 9pt)[dervish_tribe]], [#link(<sect-2-31>)[§2.31]],
  [#text(weight: "bold", size: 9pt)[die_modifier]], [#link(<sect-6-24>)[§6.24]],
  [#text(weight: "bold", size: 9pt)[effective_movement_at_night]], [#link(<sect-8-1>)[§8.1]],
  [#text(weight: "bold", size: 9pt)[end_player_turn]], [#link(<sect-4>)[§4], #link(<sect-5-13>)[§5.13]],
  [#text(weight: "bold", size: 9pt)[fall_of_khartoum_map_data]], [#link(<sect-9-31>)[§9.31], #link(<sect-9-342>)[§9.342]],
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
  [#text(weight: "bold", size: 9pt)[is_desertion_exempt]], [#link(<sect-8-2>)[§8.2]],
  [#text(weight: "bold", size: 9pt)[is_friendlies]], [#link(<sect-5-21>)[§5.21], #link(<sect-6-52>)[§6.52]],
  [#text(weight: "bold", size: 9pt)[is_gordon]], [#link(<sect-9-346>)[§9.346]],
  [#text(weight: "bold", size: 9pt)[is_los_trees]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[is_nile_mouth_crossing]], [#link(<sect-9-345>)[§9.345]],
  [#text(weight: "bold", size: 9pt)[is_walled_city]], [#link(<sect-5-23>)[§5.23]],
  [#text(weight: "bold", size: 9pt)[khalifa_abdullah]], [#link(<sect-2-31>)[§2.31]],
  [#text(weight: "bold", size: 9pt)[loaded_on]], [#link(<sect-5-21>)[§5.21]],
  [#text(weight: "bold", size: 9pt)[los_level]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[los_level_for_unit]], [#link(<sect-6-3>)[§6.3]],
  [#text(weight: "bold", size: 9pt)[mandatory_melee_modifiers]], [#link(<sect-7-7>)[§7.7]],
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
  [#text(weight: "bold", size: 9pt)[stacking_rule]], [#link(<sect-5-51>)[§5.51]],
  [#text(weight: "bold", size: 9pt)[sum]], [#link(<sect-7-1>)[§7.1]],
  [#text(weight: "bold", size: 9pt)[sum_to_row]], [#link(<sect-6-14>)[§6.14]],
  [#text(weight: "bold", size: 9pt)[superiority]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[terrain_effects_chart]], [#link(<sect-5-11>)[§5.11]],
  [#text(weight: "bold", size: 9pt)[total_for]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[unit_projects_zoc]], [#link(<sect-5-41>)[§5.41], #link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[unit_projects_zoc_rule]], [#link(<sect-5-41>)[§5.41]],
  [#text(weight: "bold", size: 9pt)[value]], [#link(<sect-4>)[§4]],
  [#text(weight: "bold", size: 9pt)[who_scores]], [#link(<sect-9-14>)[§9.14]],
  [#text(weight: "bold", size: 9pt)[zariba_entry_surcharge]], [#link(<sect-9-233>)[§9.233]],
  [#text(weight: "bold", size: 9pt)[zoc_extends]], [#link(<sect-5-44>)[§5.44]],
  [#text(weight: "bold", size: 9pt)[zoc_hexes]], [#link(<sect-5-41>)[§5.41]],
)

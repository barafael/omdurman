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
#align(center, text(size: 18pt, weight: "bold", "Traceability Matrix"))
#align(center, text(size: 10pt, "REMEMBER GORDON! -- Rulebook ⇌ Implementation Mapping"))
#align(center, text(size: 9pt, fill: luma(120), "Generated from `docs/traceability.toml`"))
#v(2em)
#heading(level: 1, "§1 -- Introduction")
#heading(level: 2, "§1.1 -- General Comments")
#status-tag("descriptive")
#linebreak()
#text(size: 8.5pt, fill: luma(120))[manual page 1]
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[General Comments

"REMEMBER GORDON!" — THE BATTLE OF OMDURMAN is a simulation of the final battle in Great Britain's two-year campaign to reassert her presence in the Sudan (1896–1898). Fought September 2nd, 1898, Omdurman finally broke the back of the fanatical Dervish rebellion and gained Britain a million square miles of desolate territory and two million impoverished subjects. With two players, one assumes the role of Herbert Kitchener, Sirdar (CIC) of the Anglo-Egyptian army; the other player becomes the Khalifa, Abdullah the Taiasha, absolute ruler of the Dervishes. The game is also suited for multi-player participation, with each player assuming command of one or more Dervish tribes or Anglo-Egyptian brigades.

While "REMEMBER GORDON!" — THE BATTLE OF OMDURMAN is not, strictly speaking, a beginner's game, the mechanics of play should be familiar to players of modest experience. It is suggested that the bonus game, FALL OF KHARTOUM, and the historical scenario be played first to familiarize players with the game system prior to embarking on the full campaign game.

The designer would also like to point out that English spelling of Arabic names, places, and words is a process of transliteration rather than translation. Spellings thus tend to vary widely accordingly to the source, author, and date of publication.]]
#v(0.5em)
#heading(level: 2, "§1.2 -- Game Scale")
#status-tag("descriptive")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Game Scale

Each hexagon of the mapsheet represents approximately 400–440 yards of real terrain and each day turn is the equivalent of two hours of real time. Each counter of infantry and cavalry represents between 400 and 700 men, and each of the gunboats present at the battle has its own counter. The upper echelon of command is represented by individual leader counters for the Anglo-Egyptian force; and leaders plus their retinues for the Dervish army.]]
#v(0.5em)
#heading(level: 1, "§2 -- Game Components")
#heading(level: 2, "§2.1 -- The Game Maps")
#status-tag("descriptive")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Game Maps

The Omdurman battle map represents approximately 100 square miles of real territory and includes the area north of Omdurman in which the historical battle took place as well as the dominant terrain features that influenced the course of the battle. Note that the mapsheet also contains the Turn Record Track, Turn Sequence, and Terrain Effects Chart at the top; and the Combat Tables and Howitzer Fire Scattergram in the lower right corner. The large letters "A", "D", "Y", etc. are set-up hexes for the historical scenario only (9.2) and should be ignored in the campaign game. Similarly, the hexsides of the Zariba exist only in the historical scenario and should be considered clear terrain in the campaign game. Note, however, that the Anglo-Egyptian player may "construct" the Zariba in the campaign game if desired (see 5.3). All full hexes of the Omdurman game map are playable, including the seven hexes of the Howitzer Fire Scattergram.

The mini-map for the bonus game, FALL OF KHARTOUM, represents that city as it appeared in 1885. The portion of wall conspicuous by its absence represents the area washed away by the receding White Nile after the flood. Players will note that the north edge of the Khartoum mini-map abuts the middle portion of the Omdurman map south edge. After Khartoum fell, it was destroyed by the Mahdi's troops and lay in ruins in 1898.]]
#v(0.5em)
#heading(level: 2, "§2.2 -- Play Aids")
#status-tag("descriptive")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Play Aids

Certain charts and tables are needed to play the game. The Terrain Effects Chart lists all terrain found on the mapsheet and the effect of each type on movement and combat. The Combat Tables describe the range effects on various weapon types and includes the Combat Results Table. Also note the Line of Sight Table on the back of this rulebook. It tells players when certain terrain types block line of sight, thus preventing direct fire attacks on enemy units. Players should become familiar with these various charts and tables prior to the beginning of play.]]
#v(0.5em)
#heading(level: 2, "§2.3 -- The Units")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Units]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 494)], [#text[UnitFormKind]], [#raw("    strum::EnumIter,
)]
pub enum UnitFormKind {
    #[default]
    Infantry,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 673)], [#text[UnitProfile]], [#raw("/// print no melee value).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct UnitProfile {
    pub kind: UnitKind,
    pub identity: UnitIdentity,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 348)], [#text[BrigadeId]], [#raw("/// (§2.3, §5.54). The number is the brigade ordinal as printed, e.g. `2B`.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BrigadeId {
    pub number: u8,
    pub nationality: BrigadeNationality,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 646)], [#text[SpriteAnnotation]], [#raw("/// movement allowance (§5.24); leaders print movement only (§6.51).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpriteAnnotation {
    /// Command/tribe colour. A real game indicator: Dervish leaders may only
    /// stack with units of their own colour, and different tribes may not", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§2.4 -- Game Parts Inventory")
#status-tag("out-of-scope")
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
#heading(level: 2, "§2.31 -- Dervish weapon types")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Dervish artillery, gunboats, and forts fire on the "artillery" line of the Dervish Range Effects Table; Jehadia and Danagla units fire on the "rifles" line as does the Isa Zachneih unit. All other Dervish units (including leaders) are armed with spears and swords.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 483)], [#text[WeaponClass]], [#raw("/// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum WeaponClass {
    /// Dervish spears and swords -- no ranged fire at all.
    Melee,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 494)], [#text[Howitzer]], [#raw("    /// (old + new), and Anglo-Egyptian artillery.
    Artillery,
    /// \"Howitzer\" line -- only the five named British gunboats (§6.64).
    /// No howitzer fire allowed at night (§8.1, §6.64).
    Howitzer,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§2.32 -- Anglo-Egyptian weapon types")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[All Anglo-Egyptian units (except gunboats, Maxims, artillery, and leaders) are armed with rifles. Maxims fire on the "Maxims" line of the Anglo-Egyptian Range Effects Table, and artillery and old gunboats fire on the "Artillery" line. New type (named) gunboats may fire on the "Howitzer", "Artillery", and "Maxims" lines of the Range Effects Table. (See 6.52 for the fire capabilities of the "Friendlies".)

\*\*Sample Dervish Units\*\* (printed on counters): combat unit (Combat / Melee / Movement values, plus Tribe identifier); Leader (e.g. OSMAN DIGNA); Camel unit (e.g. Danagla, 4-6-12); Fort.

\*\*Sample Anglo-Egyptian Units\*\* (printed on counters): Cavalry (e.g. 21 Lancers); Artillery (e.g. 32 Battery); Old Gunboat (e.g. LORD KITCHENER, 0-0-15); New Gunboat — named (e.g. Sultan, with artillery and howitzer factor, plus movement downstream / movement upstream values); Maxim Guns (fire twice per turn); Infantry (Fire Combat Factor / Melee / Movement, plus Battalion ID and Brigade ID — e.g. "2B" = 2nd British Brigade, "3E" = 3rd Egyptian Brigade).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 403)], [#text[GunboatId]], [#raw("/// fire; \"old\" gunboats do not (rulebook §2.32).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum GunboatId {
    /// One of the five new-type named gunboats with howitzer capability.
    Named(NamedGunboat),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 414)], [#text[NamedGunboat]], [#raw("/// The five named gunboats with howitzer capability (rulebook §6.64, §2.32).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum NamedGunboat {
    Sultan,
    Melik,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 424)], [#text[OldGunboat]], [#raw("/// Old-style gunboat -- no howitzer fire (rulebook §2.32).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum OldGunboat {
    LordKitchener,
    Tamai,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 407)], [#text[GunboatId::Old]], [#raw("    Named(NamedGunboat),
    /// An old-style gunboat -- no howitzer fire (§2.32).
    Old(OldGunboat),
    /// A Dervish gunboat (§9.111, §10.14).
    DervishGunboat(u8),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 409)], [#text[GunboatId::DervishGunboat]], [#raw("    Old(OldGunboat),
    /// A Dervish gunboat (§9.111, §10.14).
    DervishGunboat(u8),
}
", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 1, "§3 -- Getting Started")
#heading(level: 2, "§3 -- Getting Started")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Getting Started

Spread out the mapsheet on a table. It should lie flat if you backfold it against the scored lines. The Dervish player should sit next to the west edge of the map and the Anglo-Egyptian player opposite him on the east edge. Read through the rules once, looking over the various charts as they are referred to in the various sections. Next, select a scenario and punch out only those unit counters needed to play. Later on, the rest of the unit counters should be punched out, sorted and stored by unit type.]]
#v(0.5em)
#heading(level: 1, "§4 -- Turn Sequence")
#heading(level: 2, "§4 -- Turn Sequence")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Turn Sequence

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

\*\*C)\*\* After both players have completed their "Player Turns", advance the "Game Turn" marker to the next hour. Continue in this manner, alternating turns, until the end of the scenario being played.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 230)], [#text[GameTurnIndex]], [#raw("/// One-based Game Turn index (1, 2, ... up to the scenario length) (rulebook §4).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct GameTurnIndex(pub u8);

impl GameTurnIndex {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 285)], [#text[Phase]], [#raw("/// etc.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Movement,
    DefensiveFire(FireSubPhase),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 180)], [#text[GameState]], [#raw("
// ---------------------------------------------------------------------------
// 3) GameState -- authoritative mutable snapshot
// ---------------------------------------------------------------------------
", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 223)], [#text[GameState::new]], [#raw("impl GameState {
    /// Create a fresh game state for a given scenario (rulebook §4).
    pub fn new(scenario: Scenario) -> Self {
        let first = match scenario {
            Scenario::Campaign => campaign_turn(GameTurnIndex(1)),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 36)], [#text[AdvancePhase]], [#raw("    // -- Turn / phase flow ------------------------------------------------
    /// Advance to the next phase (or next player-turn if melee is done) (rulebook §4).
    AdvancePhase,

    // -- Movement ----------------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 599)], [#text[advance_phase]], [#raw("
/// Advance the game state to the next phase (rulebook §4).
fn advance_phase(state: &mut GameState) -> Result<(), RuleError> {
    match state.phase {
        Phase::Movement => {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 650)], [#text[end_player_turn]], [#raw("
/// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
fn end_player_turn(state: &mut GameState) -> Result<(), RuleError> {
    // Collect disrupted units first, then apply recovery.
    let to_recover: Vec<UnitId> = state", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 233)], [#text[GameTurnIndex::value]], [#raw("
impl GameTurnIndex {
    pub fn value(self) -> u8 {
        self.0
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 215)], [#text[PendingMelee]], [#raw("/// resolution after the reaction window is deterministic and host-ordered (rulebook §7.5).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PendingMelee {
    pub attack: MeleeAttack,
    pub attacker_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 515)], [#text[hex_in_enemy_zoc]], [#raw("    /// `mover_player` (§5.41). A unit moving into such a hex must stop there
    /// and may move no further that turn (§5.26, §5.43).
    pub fn hex_in_enemy_zoc(&self, hex: HexCoord, mover_player: Player) -> bool {
        self.units.iter().any(|u| {
            self.unit_projects_zoc(u, mover_player).is_some()", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 269)], [#text[can_move_unit]], [#raw("    /// the same `RuleError` the `MoveUnit` effect would on rejection. Lets the
    /// UI gate input without mutating or duplicating the rules.
    pub fn can_move_unit(&self, unit_id: UnitId, cost: MovementPoints) -> Result<(), RuleError> {
        self.can_move_unit_to(unit_id, None, cost)
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 1, "§5 -- Movement Phase")
#heading(level: 2, "§5.3 -- Constructing the Zariba")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Constructing the Zariba

The Zariba trench and thorn hedge hexsides are built and in place in the historical scenario only. These hexsides are considered clear terrain in the campaign game. The Anglo-Egyptian player may, however, find it useful to construct this defensive position during the campaign game. The Zariba hexsides may only be built in their position as displayed on the mapsheet. Construction procedure is as follows: any Anglo-Egyptian infantry unit that begins and ends the Anglo-Egyptian player turn adjacent to (and on the Nile side of) Zariba hexsides has constructed all Zariba hexsides to which he is adjacent. The constructing unit may neither fire offensively nor melee attack during the turn of construction. Use a blank counter to denote units constructing Zariba hexsides. See 9.23 for defensive benefits and movement restrictions of Zariba hexsides.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 1377)], [#text[constructing_zariba]], [#raw("        // melee attack during the turn of construction.\"
        let s = UnitState {
            constructing_zariba: true,
            ..UnitState::default()
        };", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 98)], [#text[ConstructZariba]], [#raw("
    /// Begin constructing a Zariba hexside (rulebook §5.3).
    ConstructZariba {
        unit_ids: Vec<UnitId>,
        hexside: HexsideRef,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 578)], [#text[apply_construct_zariba]], [#raw("        GameEffect::RecoverUnit { unit_id } => apply_recover_unit(state, *unit_id),
        GameEffect::ConstructZariba { unit_ids, hexside } => {
            apply_construct_zariba(state, unit_ids, *hexside)
        }
        GameEffect::Demolition { unit_id, target } => apply_demolition(state, *unit_id, *target),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 721)], [#text[UnitState::may_attack_this_turn]], [#raw("    /// A unit that began construction this turn may not fire offensively or
    /// melee (§5.3, §6.53).
    pub fn may_attack_this_turn(self) -> bool {
        !self.disrupted && !self.constructing_zariba && !self.demolishing
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.11 -- Movement allowances printed on units")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The movement allowances of the various unit types are printed directly on the units (see 2.3). A unit may move up to this printed movement allowance, paying varying costs for different terrain types (see the Terrain Effects Chart).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 111)], [#text[MovementAllowance]], [#raw("    /// is a named variant.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum MovementAllowance {
        /// Immobile (forts, wrecked gunboats).
        Immobile = 0,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 679)], [#text[UnitMovement]], [#raw("    pub fire: Option<FireFactor>,
    pub melee: Option<MeleeFactor>,
    pub movement: UnitMovement,
}
", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 247)], [#text[NileFlow]], [#raw("/// opposite way is **upstream**.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct NileFlow {
    /// Direction the current flows toward (downstream).
    pub dir: HexDirection,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 201)], [#text[HexDirection]], [#raw("/// (`+q`, `+q+r`, `+r`, `-q`, `-q-r`, `-r` for pointy-top hexes) (rulebook §5.11, §5.24).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum HexDirection {
    #[default]
    East = 0,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 151)], [#text[MovementPoints]], [#raw("/// Movement points spent or remaining within a single phase (rulebook §5).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MovementPoints(pub i16);

/// A distance measured in hexes (range to target, length of a retreat, ...)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 65)], [#text[movement_cost]], [#raw("/// Convenience: get the movement cost for a terrain type (rulebook §5.11, Terrain Effects Chart).
/// Returns `None` for impassable terrain (Nile).
pub fn movement_cost(terrain: Terrain) -> Option<MovementAllowance> {
    terrain_effects_chart(terrain).movement_cost
}", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 73)], [#text[movement_cost_with_road]], [#raw("/// underlying terrain; without a road it's the terrain's own cost. The road is
/// a movement overlay only -- combat/LOS still use the underlying terrain.
pub fn movement_cost_with_road(terrain: Terrain, road: bool) -> Option<MovementAllowance> {
    if road {
        Some(MovementAllowance::One)", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 333)], [#text[Terrain::passable_by_land]], [#raw("    }
    /// Whether this terrain may be entered by land units (rulebook §5.11).
    pub fn passable_by_land(self) -> bool {
        !self.is_nile()
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.12 -- May move as many or as few units as desired")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[A player may move as many or as few of his units as desired during each movement phase, limited only by the units' movement allowance, the terrain costs paid in moving from hex to hex, and enemy zones of control (see 5.4).]]
#v(0.5em)
#heading(level: 2, "§5.13 -- No MP accumulation between turns")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[A unit may never accumulate movement points from turn to turn, nor may a unit transfer unused movement points to other units. A unit's unused movement points in any given turn are considered lost.]]
#v(0.5em)
#heading(level: 2, "§5.21 -- Friendlies transport via gunboat")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[In general, naval transport missions are not allowed, i.e. gunboats may not carry any land units. The sole exception is that the Anglo-Egyptian player may transport the surviving units of the "Friendlies" brigade from the east bank of the Nile to the west bank after, and only after, the Dervish east bank unit (Isa Zachneih) has been eliminated. The transport is accomplished in the following sequence:
a) on any turn that a "Friendlies" unit and any Anglo-Egyptian gunboat start their turn adjacent, that unit may load onto (i.e. stack with) the gunboat;
b) during the Anglo-Egyptian player's next turn the gunboat may move to any Nile hex adjacent to a west bank hex (up to the gunboat's movement allowance);
c) on the Anglo-Egyptian player's third turn the "Friendlies" unit may disembark and move normally, paying the normal terrain cost for the first hex entered. The gunboat may also move normally that turn.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 610)], [#text[is_friendlies]], [#raw("    /// \"Friendlies\" units obey several special rules (§5.21, §5.23, §6.52,
    /// §9.14 victory conditions).
    pub fn is_friendlies(&self) -> bool {
        matches!(
            self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 704)], [#text[loaded_on]], [#raw("    pub disrupted: bool,
    /// `Some(gunboat)` after a \"Friendlies\" unit loads onto a gunboat (§5.21).
    pub loaded_on: Option<UnitId>,
    /// Set while the unit is building Zariba hexsides -- neither offensive
    /// fire nor melee allowed that turn (§5.3).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 118)], [#text[FriendliesTransport]], [#raw("
    /// Load/disembark the \"Friendlies\" brigade via gunboat (rulebook §5.21).
    FriendliesTransport(crate::FriendliesTransport),

    // -- Optional rules ----------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1284)], [#text[apply_friendlies_transport]], [#raw("
/// Apply a Friendlies-transport state transition (rulebook §5.21).
fn apply_friendlies_transport(
    state: &mut GameState,
    action: FriendliesTransport,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 961)], [#text[FriendliesTransport]], [#raw("/// can only happen on the third turn.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FriendliesTransport {
    /// Turn N (the load turn): unit and gunboat started adjacent; unit
    /// loads onto (stacks with) the gunboat.", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.22 -- Land units may never enter a Nile River hex")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[With the exception of 5.21, land units may never enter a Nile River hex. Only gunboats may enter and move along Nile River hexes.]]
#v(0.5em)
#heading(level: 2, "§5.23 -- Walled city entry restrictions")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only certain units may enter the walled portion of Omdurman. For the Dervish player these are the Khalifa unit, the three Dervish artillery units, and the Taiasha units (the Khalifa's bodyguard). Any Anglo-Egyptian units that can get to the walled city may enter it (except gunboats and "Friendlies"). Units entering and/or exiting the walled city may only do so through a gate or breach hexside.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 101)], [#text[HexsideRef]], [#raw("/// (low->high) order so the same physical edge always compares and hashes equal
/// regardless of which side names it -- this lets a map key per-edge hexside
/// data by [`HexsideRef`].
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HexsideRef {", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 138)], [#text[HexsideKind]], [#raw("    strum::EnumIter,
)]
pub enum HexsideKind {
    /// City wall (Khartoum, walled city of Omdurman). Blocks LOS, blocks
    /// movement except across gates/breaches (§5.23), blocks ZOC into the city", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 193)], [#text[blocks_movement]], [#raw("    /// Whether land movement may *not* cross this side (§5.23). Walls block
    /// movement except at gates/breaches.
    pub fn blocks_movement(self) -> bool {
        matches!(self, HexsideKind::Wall)
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1079)], [#text[can_retreat_before_melee]], [#raw("    /// two hexes away and empty. (Does not verify the attacker is infantry --
    /// the caller offers the retreat only in response to one.)
    pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
        let unit = self
            .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.24 -- Gunboat upstream/downstream movement")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Note that gunboats have two movement allowances separated by a slash, e.g. 10/16. The smaller number is the movement allowance when moving upstream, i.e. against the current (the direction of the current is indicated by arrows in the Nile). The larger number is the movement allowance when moving downstream, i.e. with the current. Gunboats may combine movement in both directions, but if they move even one hex upstream, their upstream movement allowance is their maximum movement allowance for that turn, and may not be exceeded.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 247)], [#text[NileFlow]], [#raw("/// opposite way is **upstream**.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct NileFlow {
    /// Direction the current flows toward (downstream).
    pub dir: HexDirection,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 551)], [#text[GunboatMovement]], [#raw("/// the turn.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct GunboatMovement {
    pub upstream: MovementAllowance,
    pub downstream: MovementAllowance,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 512)], [#text[is_boat]], [#raw("impl UnitFormKind {
    /// Gunboats use the split upstream/downstream movement allowance (§5.24).
    pub fn is_boat(self) -> bool {
        matches!(self, UnitFormKind::Gunboat)
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.25 -- Dervish forts may not move")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Dervish forts may not move in any way once placed.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 112)], [#text[Immobile]], [#raw("    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum MovementAllowance {
        /// Immobile (forts, wrecked gunboats).
        Immobile = 0,
        One = 1,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 450)], [#text[Fort]], [#raw("    Gunboat,
    /// Permanent emplacement -- may not move once placed (§5.25).
    Fort,
    /// Dervish leader: has fire/melee/movement factors and may melee attack.
    DervishLeaderUnit,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 688)], [#text[UnitMovement::Immobile]], [#raw("    Gunboat(GunboatMovement),
    /// Forts may not move once placed (§5.25).
    Immobile,
}
", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.26 -- Units stop on entering enemy ZOC")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Units must stop their movement immediately upon entering an enemy zone of control (see 5.4).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 286)], [#text[can_move_unit_to]], [#raw("    ///
    /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
    pub fn can_move_unit_to(
        &self,
        unit_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.41 -- All units except AE leaders exert ZOC")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[All units except Anglo-Egyptian leaders exert a zone of control (hereafter called a ZOC) into their six adjacent hexes (exception: Gunboats exert a ZOC only against enemy gunboats). Disrupted units have no ZOC.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 754)], [#text[ZocReason]], [#raw("/// Used by the engine when answering \"is this hex in an enemy ZOC?\".
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZocReason {
    /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
    /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 493)], [#text[unit_projects_zoc]], [#raw("    /// §5.44) need the game map, which the engine does not hold; the app layers
    /// those on top. This is the position/kind/disruption core of the rule.
    fn unit_projects_zoc(&self, unit: &UnitPlacement, mover_player: Player) -> Option<ZocReason> {
        if unit.state.disrupted {
            return None;", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.42 -- No MP cost to enter/leave enemy ZOC")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[There is no movement point cost to enter or leave an enemy ZOC.]]
#v(0.5em)
#heading(level: 2, "§5.43 -- Units stop when entering enemy ZOC")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[All units must stop when they enter an enemy ZOC and may move no further that turn. In their next movement phase they may withdraw or, if desired, move directly into another enemy ZOC.]]
#v(0.5em)
#heading(level: 2, "§5.44 -- ZOC limitations (walls, khor, fort, Nile, Zariba)")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[ZOCs do not extend into or out of a Nile River hex (exception: Gunboats, see 5.41). ZOCs do not extend across a khor, into a fort, or into a hex inside the walled city across a wall hexside. ZOCs do extend out of a fort (even if unoccupied), and from a walled city hex into an adjacent non-walled-city hex across a wall hexside. ZOCs also extend out of (but not into) a walled city hex across a gate hexside. ZOCs extend both ways across a breach hexside. ZOCs also extend out of, but not into, a hut or building hex. In the historical scenario ZOCs extend out of, but not into, the Zariba across a Zariba hexside (also in the campaign game if the Zariba is constructed).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 754)], [#text[ZocReason]], [#raw("/// Used by the engine when answering \"is this hex in an enemy ZOC?\".
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZocReason {
    /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
    /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 143)], [#text[Wall]], [#raw("    /// (§5.44), blocks melee (§7.2), blocks advance-after-combat (§6.82).
    #[default]
    Wall,
    /// Gate hexside in a wall. ZOC extends *out of* the walled city through
    /// gates but not into it (§5.44). Melee may be made through a gate (§7.2).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 150)], [#text[Khor]], [#raw("    /// ways; LOS no longer blocked across the hexside.
    Breach,
    /// Khor -- gully/wadi. ZOCs do not extend across (§5.44); advance after
    /// combat may not cross (§6.82).
    Khor,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 765)], [#text[ZocReason::Zariba]], [#raw("    /// across a breach in both directions (§5.44).
    WalledCity,
    /// Zariba hexside ZOC behaviour in the historical scenario / when the
    /// Zariba is constructed (§5.44).
    Zariba,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 493)], [#text[unit_projects_zoc]], [#raw("    /// §5.44) need the game map, which the engine does not hold; the app layers
    /// those on top. This is the position/kind/disruption core of the rule.
    fn unit_projects_zoc(&self, unit: &UnitPlacement, mover_player: Player) -> Option<ZocReason> {
        if unit.state.disrupted {
            return None;", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.51 -- Stacking limit (4 units + leaders, gunboats isolated)")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[No more than four units may occupy a hex, with the exception of leaders and gunboats. All leader units are free stacking, i.e. they may stack in addition to the four-unit-per-hex stacking limitation. Gunboats may not stack with any other unit (Exception: 5.21). Players may move through friendly units at no additional cost in movement points. The stacking limitation applies only at the end of the movement phase and during combat.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 776)], [#text[OverLimit]], [#raw("    /// and the gunboat exception.
    #[error(\"hex stack exceeds the four-unit limit\")]
    OverLimit,
    /// \"Gunboats may not stack with any other unit\" (§5.51, exception §5.21).
    #[error(\"gunboats may not stack with non-gunboat units\")]", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 779)], [#text[GunboatStack]], [#raw("    /// \"Gunboats may not stack with any other unit\" (§5.51, exception §5.21).
    #[error(\"gunboats may not stack with non-gunboat units\")]
    GunboatStack,
    /// \"Units of different Dervish tribes may not stack together\" (§5.52).
    #[error(\"Dervish units of different tribes may not stack\")]", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.52 -- Different Dervish tribes may not stack together")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The units of different Dervish tribes may not stack together, even if they are the same color (e.g. although both are green, Mulazmin and Jehadia units may not stack with each other).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 782)], [#text[DervishTribeMix]], [#raw("    /// \"Units of different Dervish tribes may not stack together\" (§5.52).
    #[error(\"Dervish units of different tribes may not stack\")]
    DervishTribeMix,
    /// \"If Dervish leaders elect to stack, they may only stack with units of
    /// their command (i.e. colour)\" (§5.53).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 355)], [#text[color]], [#raw("    }

    fn color(self) -> TerrainColor {
        match self {
            Terrain::Clear => TerrainColor::Sandy,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.53 -- Leader stacking with command colour only")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Leader units are not required to stack. If Dervish leaders elect to stack, however, they may only stack with units of their command (i.e. color). For example, Sheik El Din may only stack with Mulazmins or Jehadias.

\*\*5.54) Anglo-Egyptian Brigade Integrity:\*\* All British, Sudanese, and Egyptian infantry units have their brigade designation printed in the upper right corner (e.g. "2B" = 2nd British Brigade; "3E" = 3rd Egyptian Brigade, etc.). In any combat phase in which all four infantry battalions belonging to any Anglo-Egyptian infantry brigade are stacked in the same hex they are said to have brigade integrity. Stacks having brigade integrity receive a +1 modifier to their fire combat die roll provided they all fire at the same enemy occupied hex. This modifier is in addition to the normal +1 bonus given to all Anglo-Egyptian direct fire attacks (see 6.24).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 786)], [#text[DervishLeaderCommandMismatch]], [#raw("    /// their command (i.e. colour)\" (§5.53).
    #[error(\"Dervish leader may only stack with units of their own command\")]
    DervishLeaderCommandMismatch,
}
", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§5.54 -- Anglo-Egyptian Brigade Integrity")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 792)], [#text[BrigadeIntegrity]], [#raw("/// stack contains all four battalions of a single Anglo-Egyptian brigade.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BrigadeIntegrity {
    None,
    Integrated(BrigadeId),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 646)], [#text[brigade_integrity]], [#raw("/// to grant the +1 brigade-integrity direct-fire modifier when they all fire
/// at the same hex.
pub fn brigade_integrity(identities: &[UnitIdentity]) -> BrigadeIntegrity {
    let Some(brigade) = identities.first().and_then(|i| i.brigade()) else {
        return BrigadeIntegrity::None;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 357)], [#text[BattalionOrdinal]], [#raw("    /// brigade integrity requires all four stacked in one hex (§5.54).
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
    pub enum BattalionOrdinal {
        First = 1,
        Second = 2,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 586)], [#text[Brigade]], [#raw("    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default, strum::EnumIter,
)]
pub enum Brigade {
    #[default]
    None,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 1, "§6 -- Fire Combat Phase")
#heading(level: 2, "§6.3 -- Line of Sight Table")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Line of Sight Table

This table is located on the back of this rulebook and should be self-explanatory. Locate the terrain type the firing unit is in and cross-index it with the terrain type the target unit is in. Terrain types in the intersecting box block line of sight, with exceptions as footnoted. Also study the "Special LOS Notes" given and remember that (with the exception of howitzer fire — see 6.64) you can't fire at anything you can't see!]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/los_table.rs", 3)], [#text[LosFirerTerrain]], [#raw("/// Terrain type of the *firing* unit's hex for LOS purposes (rulebook §6.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LosFirerTerrain {
    Ground,
    Rough,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 11)], [#text[LosTargetTerrain]], [#raw("/// Terrain type of the *target* unit's hex for LOS purposes (rulebook §6.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LosTargetTerrain {
    Ground,
    /// Units in the hex (including friendly -- LOS is blocked if the", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 27)], [#text[LosResult]], [#raw("/// Whether LOS is blocked (rulebook §6.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LosResult {
    Clear,
    Blocked,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 37)], [#text[los_table]], [#raw("/// If the cell says \"Blocks\", LOS is blocked; otherwise it is clear
/// (subject to the special notes below).
pub fn los_table(firer: LosFirerTerrain, target: LosTargetTerrain) -> LosResult {
    use LosFirerTerrain as F;
    use LosResult::*;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/los_table.rs", 89)], [#text[LosSpecialNote]], [#raw("/// 7. Crest hexsides block LOS unless the firer is on the higher side
///    of the crest.
pub enum LosSpecialNote {
    MaxTwoTreeHutHexes,
    HilltopToHilltop,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 170)], [#text[blocks_los]], [#raw("    /// Whether this hexside blocks line of sight across it (§6.3). Crest is
    /// directional and handled by the caller; here it is treated as blocking.
    pub fn blocks_los(self) -> bool {
        matches!(self, HexsideKind::Wall | HexsideKind::Crest)
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 339)], [#text[Terrain::blocks_los]], [#raw("    /// Whether an intervening hex of this terrain unconditionally blocks line
    /// of sight (§6.3).
    pub fn blocks_los(self) -> bool {
        matches!(self, Terrain::Huts | Terrain::Building)
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.6 -- Special Artillery Capabilities")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Artillery Capabilities]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 483)], [#text[WeaponClass]], [#raw("/// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum WeaponClass {
    /// Dervish spears and swords -- no ranged fire at all.
    Melee,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.7 -- Defensive Fire")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Defensive Fire

In Defensive Fire phase, all of the non-moving player's units may fire at any of the moving player's units in range, within the limitations imposed by the rules of combat (see 6.1 to 6.6). There is no advance after combat as a result of defensive fires.]]
#v(0.5em)
#heading(level: 2, "§6.11 -- Fire combat factor printed on units")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The fire combat factor of the various unit types is printed directly on the units and is a numerical expression of the unit's fire strength.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 66)], [#text[FireFactor]], [#raw("    /// Every possible value from the annotated counter set is a named variant.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
    pub enum FireFactor {
        One = 1,
        Three = 3,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.12 -- Fire combat is always voluntary")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Fire combat is always voluntary. A unit is never required to fire at enemy units merely because they are in range or adjacent.]]
#v(0.5em)
#heading(level: 2, "§6.13 -- Fire factor is unitary (may not be divided)")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[If a unit elects to fire, its fire combat factor at an enemy unit, that fire combat factor is unitary. A unit's fire combat factor may not be divided up to fire at enemy units on different hexes.]]
#v(0.5em)
#heading(level: 2, "§6.14 -- Players may combine fire factors into one attack")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Players may combine fire during fire combat phase, i.e. they may fire at an enemy-occupied hex with as many friendly units as may legally do so, combining all of their fire combat factors into one attack. Note that in any given fire combat phase, however, a combat unit may only fire once and may only be fired at once (exceptions: Maxim guns and gunboats — see 6.4).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 80)], [#text[sum_to_row]], [#raw("impl FireFactor {
    /// Sum multiple fire factors and return the corresponding Combat Results Table row (rulebook §6.11).
    pub fn sum_to_row<'a>(factors: impl IntoIterator<Item = &'a FireFactor>) -> FireFactorRow {
        let total: u16 = factors.into_iter().map(|f| f.value()).sum();
        crate::combat_results_table::FireFactorRow::from_total(total)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.15 -- May divide a stack to fire at different hexes")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Players may also divide a stack of units in order to fire at different enemy-occupied hexes. Anglo-Egyptian infantry units having brigade integrity, however, do not receive their +1 direct fire modifier unless they all fire at the same enemy-occupied hex (see 5.54).]]
#v(0.5em)
#heading(level: 2, "§6.16 -- Halving fire strength rounds down, minimum 1")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[When halving fire combat strength, always round down each individual unit. For example, an Egyptian brigade of four battalions, each having a printed strength of 9 fire factors, will fire a total of 16 factors when halved. However, a unit's firing strength is never reduced below one by halving.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 518)], [#text[RangeBand]], [#raw("/// multiplied at a given distance (§6.22).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RangeBand {
    Tripled,
    Doubled,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.21 -- First check LOS before firing")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[When combat units wish to fire at enemy units, first check the Line of Sight Table to be sure the firing unit can see the target hex (exception: howitzer fire, see 6.64).]]
#v(0.5em)
#heading(level: 2, "§6.22 -- Consult Range Effects Table")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Next consult the Range Effects Table to see if the firing unit's fire combat factor is tripled, doubled, normal, halved, or if the target hex is out of range. Add up the total number of fire combat factors firing at the enemy-occupied hex.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/range_effects.rs", 23)], [#text[ae_range_effects]], [#raw("/// Look up the range band for an Anglo-Egyptian weapon (§6.22, §6.24).
/// Distances > 10 are out of range for all weapons.
pub fn ae_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
    if distance.value() > 10 {
        return RangeBand::OutOfRange;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/range_effects.rs", 61)], [#text[dervish_range_effects]], [#raw("/// Look up the range band for a Dervish weapon (§6.22).
/// Distances > 10 are out of range for all weapons.
pub fn dervish_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
    if distance.value() > 10 {
        return RangeBand::OutOfRange;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 502)], [#text[Range]], [#raw("/// (rulebook §6.22). Distances beyond 10 hexes are out of range for all weapons.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Range {
    One,
    Two,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 518)], [#text[RangeBand]], [#raw("/// multiplied at a given distance (§6.22).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RangeBand {
    Tripled,
    Doubled,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 156)], [#text[HexDistance]], [#raw("/// (rulebook §6.22, §7.5).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct HexDistance(pub u16);

impl HexDistance {", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.23 -- Terrain defensive modifier")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Next check the Terrain Effects Chart to see if the enemy-occupied hex fired upon contains any terrain which gives the enemy units in that hex a defensive benefit. If so, apply this negative modifier to the roll of the ten-sided die and cross-index your net die roll on the Combat Results Table with the number of combat factors firing.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/terrain_chart.rs", 59)], [#text[defense_modifier]], [#raw("
/// Convenience: get the defense modifier for a terrain type (rulebook §6.23, Terrain Effects Chart).
pub fn defense_modifier(terrain: Terrain) -> i16 {
    terrain_effects_chart(terrain).defense_modifier
}", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 811)], [#text[Terrain]], [#raw("    /// the same enemy-occupied hex (§5.54, §6.24).
    BrigadeIntegrity,
    /// Negative modifier from the Terrain Effects Chart applied to the
    /// defender's hex (§6.23).
    Terrain(i16),", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 170)], [#text[Terrain::blocks_los]], [#raw("    /// Whether this hexside blocks line of sight across it (§6.3). Crest is
    /// directional and handled by the caller; here it is treated as blocking.
    pub fn blocks_los(self) -> bool {
        matches!(self, HexsideKind::Wall | HexsideKind::Crest)
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 346)], [#text[Terrain::is_los_trees]], [#raw("    /// line of sight is blocked by more than two intervening tree hexes
    /// (§6.3 note 1).
    pub fn is_los_trees(self) -> bool {
        matches!(self, Terrain::Trees)
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.24 -- Anglo-Egyptian direct fire accuracy bonus and brigade integrity")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[All Anglo-Egyptian direct fire attacks receive a +1 modifier to their die roll as an accuracy bonus. In addition, any stack of Anglo-Egyptian infantry having brigade integrity (see 5.54) receives a +1 modifier to their die roll if all four fire at the same enemy-occupied hex. These modifiers are cumulative.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 807)], [#text[AngloEgyptianDirectFire]], [#raw("pub enum FireModifier {
    /// +1 to all Anglo-Egyptian *direct* fire (§6.24).
    AngloEgyptianDirectFire,
    /// +1 brigade integrity, applied only if all four battalions fire at
    /// the same enemy-occupied hex (§5.54, §6.24).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 792)], [#text[BrigadeIntegrity]], [#raw("/// stack contains all four battalions of a single Anglo-Egyptian brigade.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BrigadeIntegrity {
    None,
    Integrated(BrigadeId),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 823)], [#text[die_modifier]], [#raw("impl FireModifier {
    /// Return the numeric die-roll modifier for this bonus/penalty (rulebook §6.24, §5.54, §6.23, §9.231, §9.232).
    pub fn die_modifier(self) -> i16 {
        match self {
            FireModifier::AngloEgyptianDirectFire | FireModifier::BrigadeIntegrity => 1,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.41 -- Direct Fire Subphase")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 295)], [#text[DirectFire]], [#raw("pub enum FireSubPhase {
    /// Direct fire (§6.41). Both sides participate in this sub-phase.
    DirectFire,
    /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
    MaximSecondAndHowitzer,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.42 -- Maxim Second Fire and Howitzer Fire Subphase")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 297)], [#text[MaximSecondAndHowitzer]], [#raw("    DirectFire,
    /// Anglo-Egyptian only: Maxim second fire + named-gunboat howitzer fire (§6.42).
    MaximSecondAndHowitzer,
}
", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 530)], [#text[fires_twice]], [#raw("    /// again in the Maxim Second Fire Subphase (rulebook §6.42). The counter
    /// is marked \"x2\" in the editor to surface this.
    pub fn fires_twice(self) -> bool {
        matches!(self, UnitFormKind::Maxim)
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.51 -- Leader Units")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 392)], [#text[BritishLeader]], [#raw("/// to claim the Mahdi's Tomb (§9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum BritishLeader {
    Kitchener,
    Gatacre,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 454)], [#text[BritishLeaderUnit]], [#raw("    DervishLeaderUnit,
    /// Anglo-Egyptian leader: movement only (§6.51).
    BritishLeaderUnit,
}
", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 523)], [#text[has_combat_factors]], [#raw("    /// British and Dervish leaders print a movement factor only (§6.51); other
    /// playable kinds carry fire and/or melee factors.
    pub fn has_combat_factors(self) -> bool {
        !matches!(self, UnitFormKind::BritishLeader | UnitFormKind::Marker)
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.52 -- Anglo-Egyptian Friendlies Brigade")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 610)], [#text[is_friendlies]], [#raw("    /// \"Friendlies\" units obey several special rules (§5.21, §5.23, §6.52,
    /// §9.14 victory conditions).
    pub fn is_friendlies(&self) -> bool {
        matches!(
            self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 561)], [#text[Friendlies]], [#raw("    /// Native volunteer brigade -- the Shaggyeh (§6.52). Do not receive
    /// brigade integrity (§5.54 enumerates only British/Egyptian/Sudanese).
    Friendlies,
}
", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.53 -- Royal Engineers demolition")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 586)], [#text[RoyalEngineers]], [#raw("    /// The Royal Engineers (§6.53) -- a *specific* unit, not a class, so we
    /// model it explicitly.
    RoyalEngineers,
}
", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 692)], [#text[demolishing]], [#raw("
/// Volatile per-turn state of a unit -- disrupted, loaded onto a gunboat,
/// constructing the Zariba, demolishing a target, etc. (rulebook §5, §6).
///
/// Multiple state flags can be in effect at once (e.g. a unit may be both", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 104)], [#text[Demolition]], [#raw("
    /// Royal Engineers demolition (rulebook §6.53).
    Demolition {
        unit_id: UnitId,
        target: DemolitionTarget,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1212)], [#text[apply_demolition]], [#raw("
/// Apply a Royal Engineers demolition action (rulebook §6.53).
fn apply_demolition(
    state: &mut GameState,
    unit_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 948)], [#text[DemolitionTarget]], [#raw("/// disrupted or driven off.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DemolitionTarget {
    Fort(UnitId),
    WallHexside(HexsideRef),", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.54 -- Forts")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 754)], [#text[ZocReason]], [#raw("/// Used by the engine when answering \"is this hex in an enemy ZOC?\".
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZocReason {
    /// Normal ZOC: any non-disrupted unit other than an Anglo-Egyptian
    /// leader (§5.41) projects ZOC into each of its six adjacent hexes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 450)], [#text[Fort]], [#raw("    Gunboat,
    /// Permanent emplacement -- may not move once placed (§5.25).
    Fort,
    /// Dervish leader: has fire/melee/movement factors and may melee attack.
    DervishLeaderUnit,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 715)], [#text[UnitState::may_act]], [#raw("impl UnitState {
    /// A disrupted unit may not move, fire, or melee (rulebook §5, reference notes).
    pub fn may_act(self) -> bool {
        !self.disrupted
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 695)], [#text[UnitState]], [#raw("///
/// Multiple state flags can be in effect at once (e.g. a unit may be both
/// loaded and disrupted), so `UnitState` is a struct of orthogonal fields
/// rather than one big enum.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 852)], [#text[FireAttack]], [#raw("/// modifiers (rulebook §6).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FireAttack {
    pub firing_player: Player,
    pub phase: Phase,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 867)], [#text[FireAttack::net_modifier]], [#raw("impl FireAttack {
    /// Sum of all fire modifiers applied to this attack (rulebook §6.24).
    pub fn net_modifier(&self) -> i16 {
        self.modifiers.iter().map(|m| m.die_modifier()).sum()
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.61 -- Only artillery may fire at gunboats; 3+ to sink")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only artillery may fire at gunboats. A result of 3 or more on the combat results table is required to sink a gunboat. Any other result is a miss.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 483)], [#text[WeaponClass]], [#raw("/// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum WeaponClass {
    /// Dervish spears and swords -- no ranged fire at all.
    Melee,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.62 -- Only artillery may fire at forts; 2+ to destroy")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only artillery may fire at forts. A result of 2 or more on the combat results table is required to eliminate a fort. Any other result is a miss. If the fort contains any enemy units at the instant it is destroyed, one unit is eliminated with the fort.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 483)], [#text[WeaponClass]], [#raw("/// enum so a \"spear\" unit cannot accidentally fire on the \"Howitzer\" line.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum WeaponClass {
    /// Dervish spears and swords -- no ranged fire at all.
    Melee,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.63 -- Only artillery may breach wall hexsides; 2+ required")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only artillery may fire to breach a wall hexside of Khartoum or the walled city of Omdurman. A result of 2 or more on the combat results table is required to breach a wall. Any other result is a miss. The effect of the breach is to negate the wall hexside for line of sight purposes. Place a "BREACH" marker in an adjacent hex so that the arrow points to the breached hexside. If any enemy units are adjacent to the wall hexside at the instant it is breached, one enemy unit is eliminated.

\*\*6.64) Howitzer fire:\*\*
Five units in the game have howitzer fire capability. These are the five named British gunboats. They may fire their artillery factor as direct fire during the Direct Fire Subphase (see 4 and 6.41) and may then fire the same artillery factor as howitzer fire during the Maxim Second Fire and Howitzer Subphase (see 4 and 6.42). Exception: no howitzer fire is allowed during night game turns. To fire howitzer fire, select any target hex between 4 and 10 hexes from the firing gunboat (ignoring the Line of Sight Table) and roll the ten-sided die twice. The first die roll is the Combat Results Table die roll and the second roll is the impact hex die roll. Refer to the Howitzer Fire Scattergram on the mapsheet for the impact hex. The designated target hex is hit on a roll of 7–10. Once a howitzer fire die roll has been made the results must take effect, even if the fire scatters into a friendly-occupied hex.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 147)], [#text[Breach]], [#raw("    /// gates but not into it (§5.44). Melee may be made through a gate (§7.2).
    Gate,
    /// Breach in a wall (artillery/§6.63 or Royal Engineers/§6.53). ZOC both
    /// ways; LOS no longer blocked across the hexside.
    Breach,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.64 -- Howitzer fire")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/howitzer_scatter.rs", 6)], [#text[ScatterDirection]], [#raw("/// (§6.64). The caller maps these to hex-grid offsets.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScatterDirection {
    /// Roll 7-10: hit the target hex.
    OnTarget,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/howitzer_scatter.rs", 28)], [#text[howitzer_scatter]], [#raw("/// | 3-4  | [`ScatterDirection::Long`] (upstream) |
/// | 1-2  | [`ScatterDirection::LeftRight`] |
pub fn howitzer_scatter(impact_roll: DieRoll) -> ScatterDirection {
    use DieRoll::*;
    match impact_roll {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 403)], [#text[GunboatId]], [#raw("/// fire; \"old\" gunboats do not (rulebook §2.32).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum GunboatId {
    /// One of the five new-type named gunboats with howitzer capability.
    Named(NamedGunboat),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 53)], [#text[HowitzerFire]], [#raw("
    /// Resolve a howitzer bombardment (two rolls: Combat Results Table + impact scatter) (rulebook §6.64).
    HowitzerFire {
        attack: FireAttack,
        combat_results_table_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 764)], [#text[apply_howitzer_fire]], [#raw("
/// Validate and apply a howitzer fire attack (scatter path) (rulebook §6.64).
fn apply_howitzer_fire(
    state: &mut GameState,
    attack: &FireAttack,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 888)], [#text[HowitzerResolution]], [#raw("/// roll on the Howitzer Fire Scattergram (§6.64).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct HowitzerResolution {
    pub combat_results_table_roll: DieRoll,
    pub impact_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 895)], [#text[HowitzerResolution::hit_target_hex]], [#raw("impl HowitzerResolution {
    /// The designated target hex is hit on impact roll 7-10 (§6.64).
    pub fn hit_target_hex(self) -> bool {
        use DieRoll::*;
        matches!(self.impact_roll, Seven | Eight | Nine | Ten)", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 352)], [#text[can_fire_at]], [#raw("    /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
    /// (Howitzer fire ignores LOS entirely -- §6.64.)
    pub fn can_fire_at(
        &self,
        firer: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§6.81 -- Moving player may fire with all capable units")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[During Offensive Fire phase, the moving player may fire with all of his units capable of firing, up to their maximum range, within the limitations imposed by the rules of combat.]]
#v(0.5em)
#heading(level: 2, "§6.82 -- Advance after combat (offensive fire)")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[If an enemy-occupied hex is vacated as a result of offensive fire, friendly units may advance after combat into the vacated hex. To be eligible to advance, the friendly units must have participated in the attack and must have been adjacent to the vacated hex. Note that artillery may not advance, nor may units advance across a wall hexside (except at a gate or breach). Units may never advance after combat across a khor.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 91)], [#text[AdvanceAfterCombat]], [#raw("    /// after fire, §7.6 after melee). Eligible units are adjacent attackers
    /// that are not artillery; the target hex must be empty of enemies.
    AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },

    // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 574)], [#text[apply_advance_after_combat]], [#raw("        }
        GameEffect::AdvanceAfterCombat { unit_id, to } => {
            apply_advance_after_combat(state, *unit_id, *to)
        }
        GameEffect::RecoverUnit { unit_id } => apply_recover_unit(state, *unit_id),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 91)], [#text[AdvanceAfterCombat]], [#raw("    /// after fire, §7.6 after melee). Eligible units are adjacent attackers
    /// that are not artillery; the target hex must be empty of enemies.
    AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },

    // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 181)], [#text[blocks_advance_after_combat]], [#raw("
    /// Whether advance-after-combat may *not* cross this side (§6.82, §7.6).
    pub fn blocks_advance_after_combat(self) -> bool {
        matches!(
            self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1124)], [#text[can_advance_after_combat]], [#raw("    /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
    /// Wall/khor hexside restrictions are not enforced (no hexside map data).
    pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
        let unit = self
            .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 1, "§7 -- Melee Phase")
#heading(level: 2, "§7.1 -- Melee strength printed on counter")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The melee strength of all units is printed on the counter. Note that gunboats have no melee strength. Gunboats may neither melee attack nor be melee attacked.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 90)], [#text[MeleeFactor]], [#raw("    /// Every possible value from the annotated counter set is a named variant.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, strum::Display)]
    pub enum MeleeFactor {
        One = 1,
        Three = 3,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 101)], [#text[MeleeFactor::sum]], [#raw("impl MeleeFactor {
    /// Sum multiple melee factors (rulebook §7.1).
    pub fn sum<'a>(factors: impl IntoIterator<Item = &'a MeleeFactor>) -> u16 {
        factors.into_iter().map(|f| f.value()).sum()
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 468)], [#text[may_be_melee_attacked]], [#raw("
    /// Gunboats neither attack nor are attacked in melee (§7.1).
    pub fn may_be_melee_attacked(self) -> bool {
        !matches!(self, UnitKind::Gunboat)
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 468)], [#text[UnitKind::may_be_melee_attacked]], [#raw("
    /// Gunboats neither attack nor are attacked in melee (§7.1).
    pub fn may_be_melee_attacked(self) -> bool {
        !matches!(self, UnitKind::Gunboat)
    }", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.2 -- Melee adjacent only, not across wall hexsides")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Melee simulates the hand-to-hand fighting of the period. Units may melee attack adjacent enemy units only. Units may not melee attack across a wall hexside, but may melee attack through a gate or breach hexside.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-types/src/lib.rs", 176)], [#text[blocks_melee]], [#raw("    /// Whether melee may *not* be made across this side (§7.2). Gates and
    /// breaches are passable to melee.
    pub fn blocks_melee(self) -> bool {
        matches!(self, HexsideKind::Wall | HexsideKind::ZaribaThornHedge)
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 436)], [#text[can_melee]], [#raw("    /// Does **not** check wall/khor hexsides (§7.2) -- those need the game map,
    /// which the rules engine does not hold; the app gates on them.
    pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
        let unit = self
            .find_unit(attacker)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.3 -- Simultaneous melee combat")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Melee combat is considered simultaneous, so that units eliminated by melee attacks still get a melee combat die roll.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 63)], [#text[MeleeCombat]], [#raw("    /// Used for an immediate resolution with no reaction window (and as the
    /// resolution primitive in tests).
    MeleeCombat {
        attack: MeleeAttack,
        attacker_roll: DieRoll,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1063)], [#text[apply_melee_combat]], [#raw("        ));
    }
    apply_melee_combat(state, &attack, pending.attacker_roll, pending.defender_roll)
}
", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.4 -- Who may melee attack / defend")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only infantry, cavalry, camel units, and Dervish leaders may melee attack. All units (except gunboats — see 7.1) may melee defend.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 460)], [#text[may_melee_attack]], [#raw("    /// Rulebook §7.4 -- only infantry, cavalry, camel and Dervish leaders may
    /// melee *attack*. All others (except gunboats) may melee *defend* (§7.1).
    pub fn may_melee_attack(self) -> bool {
        matches!(
            self,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 440)], [#text[UnitKind]], [#raw("/// engine prove the constraint.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnitKind {
    /// Foot infantry. Includes Anglo-Egyptian infantry, \"Friendlies\",
    /// Royal Engineers, and Dervish foot tribes.", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 330)], [#text[DervishTribe]], [#raw("/// restriction (§5.52) and the leader->troops command match (§5.53).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
pub enum DervishTribe {
    Baggara,
    Jaalin,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 436)], [#text[can_melee]], [#raw("    /// Does **not** check wall/khor hexsides (§7.2) -- those need the game map,
    /// which the rules engine does not hold; the app gates on them.
    pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
        let unit = self
            .find_unit(attacker)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.5 -- Cavalry/camel retreat before melee")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Cavalry and camel units may retreat two hexes from an infantry melee attack. Note, however, that only one retreat per unit per turn is permitted. Thus, if their retreat places them adjacent to enemy units whose melee attacks have not yet been resolved, those enemy units may elect to attack the retreating unit(s).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 86)], [#text[RetreatBeforeMelee]], [#raw("    /// melee attack, *before* it is resolved (§7.5). One retreat per unit per
    /// turn. (rulebook §7.5).
    RetreatBeforeMelee { unit_id: UnitId, to: HexCoord },

    /// An attacking unit advances into a hex vacated by combat (rulebook §6.82", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 571)], [#text[apply_retreat_before_melee]], [#raw("        GameEffect::ResolveMelee => apply_resolve_melee(state),
        GameEffect::RetreatBeforeMelee { unit_id, to } => {
            apply_retreat_before_melee(state, *unit_id, *to)
        }
        GameEffect::AdvanceAfterCombat { unit_id, to } => {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 474)], [#text[may_retreat_before_melee]], [#raw("    /// Cavalry and camel units may retreat two hexes from an infantry melee
    /// attack (§7.5).
    pub fn may_retreat_before_melee(self) -> bool {
        matches!(self, UnitKind::Cavalry | UnitKind::Camel)
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 156)], [#text[HexDistance]], [#raw("/// (rulebook §6.22, §7.5).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct HexDistance(pub u16);

impl HexDistance {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1079)], [#text[can_retreat_before_melee]], [#raw("    /// two hexes away and empty. (Does not verify the attacker is infantry --
    /// the caller offers the retreat only in response to one.)
    pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
        let unit = self
            .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.6 -- Advance after melee")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[If a melee attack eliminates all of the defenders in an adjacent hex, the Dervish player MUST advance into the vacated hex. To be eligible to advance, the Dervish units must have been adjacent to the vacated hex and participated in the melee attack that eliminated the defenders. All surviving eligible Dervish units MUST advance, up to the stacking limit. The Anglo-Egyptian player may advance if desired. Note that only attacking units may advance.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 91)], [#text[AdvanceAfterCombat]], [#raw("    /// after fire, §7.6 after melee). Eligible units are adjacent attackers
    /// that are not artillery; the target hex must be empty of enemies.
    AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },

    // -- Unit state changes ------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 574)], [#text[apply_advance_after_combat]], [#raw("        }
        GameEffect::AdvanceAfterCombat { unit_id, to } => {
            apply_advance_after_combat(state, *unit_id, *to)
        }
        GameEffect::RecoverUnit { unit_id } => apply_recover_unit(state, *unit_id),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1124)], [#text[can_advance_after_combat]], [#raw("    /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
    /// Wall/khor hexside restrictions are not enforced (no hexside map data).
    pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
        let unit = self
            .find_unit(unit_id)", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§7.7 -- Melee modifiers")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[To resolve melee, both the attacker and the defender roll on the Combat Results Table and apply the applicable melee modifier to their die roll. The Dervish player receives a +2 melee modifier, the Anglo-Egyptian player receives a +1 melee modifier. No terrain modifiers are applied to melee combat (Exception: Zariba hexsides in the historical scenario and the campaign game, if constructed — see 9.23). Melee losses must be taken from meleeing units first!]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 906)], [#text[MeleeModifier]], [#raw("
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MeleeModifier {
    /// +2 to all Dervish melee rolls (§7.7).
    DervishStandard,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 197)], [#text[DieModifier]], [#raw("/// A die-roll modifier from a single named source (rulebook §6.24, §7.7).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DieModifier {
    #[default]
    Zero,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 929)], [#text[MeleeAttack]], [#raw("/// A melee attack: simultaneous, both sides roll on the Combat Results Table (§7.3, §7.7).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MeleeAttack {
    pub attacker_player: Player,
    pub attacker_hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 921)], [#text[MeleeModifier::AngloEgyptianStandard]], [#raw("        match self {
            MeleeModifier::DervishStandard => 2,
            MeleeModifier::AngloEgyptianStandard => 1,
            MeleeModifier::DervishVsTrenchedDefender => -2,
        }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 922)], [#text[MeleeModifier::DervishVsTrenchedDefender]], [#raw("            MeleeModifier::DervishStandard => 2,
            MeleeModifier::AngloEgyptianStandard => 1,
            MeleeModifier::DervishVsTrenchedDefender => -2,
        }
    }", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 908)], [#text[MeleeModifier::DervishStandard]], [#raw("pub enum MeleeModifier {
    /// +2 to all Dervish melee rolls (§7.7).
    DervishStandard,
    /// +1 to all Anglo-Egyptian melee rolls (§7.7).
    AngloEgyptianStandard,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 1, "§8 -- Night Game Turns")
#heading(level: 2, "§8.1 -- Night effects")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The effects of night game turns are: a) all Anglo-Egyptian movement allowances are halved (round down), b) there is no Anglo-Egyptian howitzer fire, and c) all fire ranges are halved for both sides (round down, but range 1 stays range 1). Range effects on fire combat are the same as during day game turns. For example, an Anglo-Egyptian infantry unit firing at night will be doubled at range 1, normal at range 2, and may not fire at range 3 or greater.

\*\*8.2) Dervish Desertion Roll:\*\* Once each campaign game, during the first night turn of the game, the Dervish player rolls one die to see how many of his units desert. The roll is made during the movement phase and the number of deserting Dervish units is equal to 1½ times the roll of one die. The Dervish player may choose which units desert by merely removing them from the mapsheet. The KHALIFA unit, gunboats, artillery units, and forts are the only Dervish units that may not be chosen. No victory points are awarded to the Anglo-Egyptian player for deserting Dervishes.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 135)], [#text[MovementAllowance::halve]], [#raw("
impl MovementAllowance {
    /// Night movement allowance = halved (round down) (rulebook §8.1, §5.11).
    pub fn halve(self) -> Self {
        let v = self.value() / 2;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 265)], [#text[DayNight]], [#raw("/// (rulebook §8.1).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DayNight {
    Day,
    Night,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1171)], [#text[effective_range_at_night]], [#raw("/// Apply night-turn range halving (§8.1): \"all fire ranges are halved for
/// both sides (round down, but range 1 stays range 1).\"
pub fn effective_range_at_night(range: HexDistance) -> HexDistance {
    if range.0 <= 1 {
        range", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1181)], [#text[effective_movement_at_night]], [#raw("/// Apply night-turn movement halving for Anglo-Egyptian units (§8.1): all
/// Anglo-Egyptian movement allowances are halved (round down).
pub fn effective_movement_at_night(
    allowance: MovementAllowance,
    player: Player,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§8.2 -- Dervish Desertion Roll")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 115)], [#text[DervishDesertion]], [#raw("    // -- Scenario-specific -------------------------------------------------
    /// Dervish desertion roll (turn 8 -- first night of campaign) (rulebook §8.2).
    DervishDesertion { roll: DieRoll },

    /// Load/disembark the \"Friendlies\" brigade via gunboat (rulebook §5.21).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 60)], [#text[DervishDesertion]], [#raw("    None,
    /// Dervish desertion roll (§8.2) -- occurs on the first night turn.
    DervishDesertion,
    /// Dervish reinforcements are available.
    DervishReinforcements,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 52)], [#text[TurnEvent]], [#raw("    pub day_night: DayNight,
    /// Any special event on this turn.
    pub event: TurnEvent,
}
", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 1, "§9 -- The Scenarios")
#heading(level: 2, "§9.1 -- The Campaign Game")
#status-tag("descriptive")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Campaign Game]]
#v(0.5em)
#heading(level: 2, "§9.2 -- The Historical Scenario")
#status-tag("descriptive")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Historical Scenario

Players should note that the historical scenario is an exercise in futility for the Dervish player. It is, however, an interesting demonstration of the absolute imbecility of the Khalifa's generalship and vividly shows the superiority of entrenched firepower over traditional tribal arms in the colonial period.]]
#v(0.5em)
#heading(level: 2, "§9.3 -- Bonus Game: Fall of Khartoum")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Bonus Game: FALL OF KHARTOUM scenario]]
#v(0.5em)
#heading(level: 2, "§9.11 -- Set Up (Campaign)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Set Up]]
#v(0.5em)
#heading(level: 2, "§9.12 -- Scenario Length (Campaign)")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Scenario Length

6:00 am, Sept. 1 through 8:00 am, Sept. 3, 22 Game Turns.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 8)], [#text[GameTime]], [#raw("/// starts at one of these twelve times.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameTime {
    SixAM,
    EightAM,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 44)], [#text[TurnEntry]], [#raw("/// A single entry on the Turn Record Track (rulebook §9.12, §9.22).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TurnEntry {
    /// 1-based turn number.
    pub turn: u8,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 72)], [#text[CAMPAIGN_TURN_TRACK]], [#raw("/// Turns 1-4 are day turns on Sept 1, then night turns alternate with
/// day turns on Sept 2-3 per the printed track.
const CAMPAIGN_TURN_TRACK: [TurnEntry; 22] = [
    //  Sept 1
    TurnEntry {", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.13 -- Special Rules (Campaign)")
#status-tag("descriptive")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Rules

None.]]
#v(0.5em)
#heading(level: 2, "§9.14 -- Victory Conditions (Campaign)")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Victory Conditions

The Mahdi's Tomb in Omdurman was not only the tallest structure in the entire Sudan in 1898, it was also a Dervish holy shrine. Its loss or destruction would be a severe blow to the Mahdist cause. It is accordingly assigned 25 victory points which are awarded to the player who controls it at the conclusion of play. The Dervish player controls it at the start of play. As a tactical note, the Anglo-Egyptian player will find a decisive victory almost impossible unless he takes the Mahdi's Tomb from the Dervish player. To take the Tomb hex, it must be occupied by one British leader plus any one non-"Friendlies" Anglo-Egyptian combat unit (both undisrupted) at the conclusion of play.

Additional victory points are awarded as follows:

\*\*Dervish Player receives:\*\*
- 10 pts: each British leader eliminated
- 10 pts: each British gunboat sunk
- 1 pt: each "Friendlies" unit eliminated on the east bank side
- 3 pts: each "Friendlies" unit eliminated on the west bank (see 5.21)
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

Alternatively, a decisive victory is awarded to the Anglo-Egyptian player if he eliminates every Dervish unit in play (including gunboats and forts). A decisive victory may be awarded the Dervish player if he eliminates all Anglo-Egyptian units on the west bank (excluding gunboats).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 1028)], [#text[VpSource]], [#raw("/// the manual and the engine.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum VpSource {
    // ----- Anglo-Egyptian player receives:
    /// Mahdi's Tomb control at conclusion of play (§9.14).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1054)], [#text[VpSource::points]], [#raw("impl VpSource {
    /// VP awarded to `who_scores()` (rulebook §9.14).
    pub fn points(self) -> VictoryPoints {
        match self {
            VpSource::MahdisTomb => VictoryPoints(25),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1069)], [#text[VpSource::who_scores]], [#raw("
    /// Which player receives these victory points (rulebook §9.14).
    pub fn who_scores(self) -> Player {
        match self {
            VpSource::MahdisTomb", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1086)], [#text[VictoryLedger]], [#raw("/// Cumulative victory ledger for one scenario (rulebook §9.14).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VictoryLedger {
    pub events: Vec<VpEvent>,
}", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1087)], [#text[VpEvent]], [#raw("#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VictoryLedger {
    pub events: Vec<VpEvent>,
}
", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1099)], [#text[VictoryLedger::total_for]], [#raw("impl VictoryLedger {
    /// Total victory points earned by a given player (rulebook §9.14).
    pub fn total_for(&self, player: Player) -> VictoryPoints {
        VictoryPoints(
            self.events", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1109)], [#text[VictoryLedger::superiority]], [#raw("    }

    /// Net superiority: positive = Anglo-Egyptian ahead, negative = Dervish ahead
    /// (rulebook §9.14).
    pub fn superiority(&self) -> VictoryPoints {", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1118)], [#text[CampaignVictoryLevel]], [#raw("/// Campaign-game victory levels (§9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CampaignVictoryLevel {
    Draw,
    Marginal(Player),", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 1127)], [#text[CampaignVictoryLevel::from_superiority]], [#raw("impl CampaignVictoryLevel {
    /// Assign a level from the net superiority (§9.14).
    pub fn from_superiority(s: VictoryPoints) -> Self {
        let net = s.0;
        // Positive -> Anglo-Egyptian thresholds: 15/30/50", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1455)], [#text[score_elimination]], [#raw("            for &id in target_ids.iter().take(n) {
                state.log(format!(\"Unit {:?} eliminated\", id));
                score_elimination(state, id, target_player);
            }
            state.units.retain(|u| !target_ids[..n].contains(&u.id));", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 226)], [#text[VictoryPoints]], [#raw("/// (rulebook §9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct VictoryPoints(pub i32);

/// One-based Game Turn index (1, 2, ... up to the scenario length) (rulebook §4).", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.21 -- Set Up (Historical)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Set Up]]
#v(0.5em)
#heading(level: 2, "§9.22 -- Scenario Length (Historical)")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Scenario Length

6:00 am, September 2 through 12:00 noon, September 2. Four game turns.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/turn_track.rs", 216)], [#text[HISTORICAL_TURN_TRACK]], [#raw("
/// Historical scenario track (§9.22 -- 4 turns, Sept 2 6:00 am -> 12:00 pm).
const HISTORICAL_TURN_TRACK: [TurnEntry; 4] = [
    TurnEntry {
        turn: 1,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.23 -- Special Rule: The Zariba")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Rule: "The Zariba"

\*\*9.231) Thorn hedge hexsides:\*\* −2 to die roll on all Dervish fire attacks; may not melee across in either direction; may not advance after combat across in either direction.

\*\*9.232) Trench hexsides:\*\* −4 to die roll on all Dervish fire attacks vs. entrenched units only; −2 (instead of +2) melee modifier to Dervish units melee attacking an entrenched unit; entrenched units may be fired "over" in both directions (i.e. they do not block line of sight); units are considered "entrenched" if they are directly adjacent to (and on the Nile River side of) a trench hexside.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 815)], [#text[ZaribaThornHedge]], [#raw("    Terrain(i16),
    /// -2 thorn-hedge defensive modifier (§9.231).
    ZaribaThornHedge,
    /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
    /// units (those Nile-side of the trench hexside).", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 818)], [#text[ZaribaTrenchEntrenched]], [#raw("    /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
    /// units (those Nile-side of the trench hexside).
    ZaribaTrenchEntrenched,
}
", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 157)], [#text[ZaribaThornHedge]], [#raw("    Crest,
    /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
    ZaribaThornHedge,
    /// Historical-scenario trench segment of the Zariba (§9.232).
    ZaribaTrench,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 159)], [#text[ZaribaTrench]], [#raw("    ZaribaThornHedge,
    /// Historical-scenario trench segment of the Zariba (§9.232).
    ZaribaTrench,
    /// Khor Shambat -- the specific named khor that empties into the Nile (a
    /// scenario landmark; used as a setup/reinforcement boundary). Same blocking", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.24 -- Victory Conditions (Historical)")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Victory Conditions

Victory Levels are based solely on eliminating enemy units while conserving your own force as much as possible.

| Victory Level | Anglo-Egyptian Player (Dervish units eliminated) | Dervish Player (Anglo-Egyptian units eliminated) |
|---|---|---|
| 5 — DECISIVE | 100+ | 30+ |
| 4 — STRATEGIC | 60–99 | 15–29 |
| 3 — TACTICAL | 45–59 | 10–14 |
| 2 — MARGINAL | 30–44 | 5–9 |
| 1 — DRAW | 0–29 | 0–4 |

The lower value victory level is then subtracted from the higher level to determine a player's net victory. For example, if the Anglo-Egyptian player eliminates 104 Dervish units (decisive victory) but loses 18 units doing it (Dervish Strategic), the Anglo-Egyptian player only nets out with a draw (decisive worth 5 minus strategic worth 4 = 1, draw).]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 1157)], [#text[HistoricalVictoryLevel]], [#raw("/// draw\").
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum HistoricalVictoryLevel {
    Draw = 1,
    Marginal = 2,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.31 -- Bonus game map")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Only the small FALL OF KHARTOUM scenario map is used for this game.]]
#v(0.5em)
#heading(level: 2, "§9.32 -- Set Up (Bonus)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Set Up]]
#v(0.5em)
#heading(level: 2, "§9.33 -- Scenario Length (Bonus)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Scenario Length

Variable, see victory conditions (9.35). Rarely lasts five turns.]]
#v(0.5em)
#heading(level: 2, "§9.34 -- Special Rules (Bonus)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Special Rules]]
#v(0.5em)
#heading(level: 2, "§9.35 -- Victory Conditions (Bonus)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Victory Conditions

Victory is determined by how many turns it takes the Dervish player to eliminate the GORDON leader unit and how many Dervish units are eliminated:

- Dervish decisive: eliminate GORDON turn four or sooner.
- Dervish tactical: eliminate GORDON turn five.
- Dervish marginal: eliminate GORDON turn six.
- British marginal: GORDON survives end of turn six.
- British tactical: GORDON survives end of turn seven.
- British decisive: GORDON survives end of turn eight.

The Dervish player then loses one victory level if he has lost 16–23 units, two victory levels if he has lost 24–31 units, and three victory levels if he has lost 32 units or more. Thus, for example, a Dervish tactical victory becomes a British marginal victory if the Dervish player eliminates GORDON on turn five, but loses 24 Dervish units doing it!]]
#v(0.5em)
#heading(level: 2, "§9.111 -- Dervish set up (Campaign)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Dervish player sets up first, moves second.

- Isa Zachneih infantry unit: anywhere on the east bank, in or south of El Debeba.
- KHALIFA ABDULLAH: in the walled city of Omdurman, in either palace hex.
- 3 artillery units, and all Taiasha units: anywhere in the walled city of Omdurman.
- 17 forts: anywhere on the mapsheet south of the Khor Shambat on the west bank, and/or south of all Halfaya hut hexes on the east bank and Nile River islands.
- 2 gunboats: any south edge Nile River hexes.

\*\*9.112) Dervish reinforcements:\*\* all reinforcements enter on the west edge of the mapsheet, south of the Khor Shambat. Each unit pays the terrain cost of the hex through which it enters, no matter how many units enter through that hex.

- Turn 1) all Baggara, Jaalin, Danagla, Kehena, and Degheim units, and their leaders: YAKUB, SHERIF, and ALI WAD HELU.
- Turn 2) all Hadendowa units and their leader, OSMAN DIGNA.
- Turn 3) all Mulazmin and Jehadia units and their leader, SHEIK EL DIN.]]
#v(0.5em)
#heading(level: 2, "§9.112 -- Dervish reinforcements (Campaign)")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 111)], [#text[PlaceReinforcements]], [#raw("    // -- Reinforcement / placement -----------------------------------------
    /// Place reinforcements onto the map (rulebook §9.112, §9.113).
    PlaceReinforcements(Vec<UnitPlacement>),

    // -- Scenario-specific -------------------------------------------------", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 582)], [#text[apply_place_reinforcements]], [#raw("        GameEffect::Demolition { unit_id, target } => apply_demolition(state, *unit_id, *target),
        GameEffect::PlaceReinforcements(placements) => {
            apply_place_reinforcements(state, placements)
        }
        GameEffect::DervishDesertion { roll } => apply_dervish_desertion(state, *roll),", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 376)], [#text[Location]], [#raw("/// Named map landmarks (rulebook mapsheet, §9.111, §9.113, §9.212 scenarios).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
pub enum Location {
    FortMakran,
    NorthFort,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 395)], [#text[SetupLetter]], [#raw("/// Each letter marks a specific hex where a Dervish leader is placed.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
pub enum SetupLetter {
    Y,
    K,", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 468)], [#text[Faction]], [#raw("    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, strum::Display, strum::EnumIter,
)]
pub enum Faction {
    Dervish,
    BritishEgyptian,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.113 -- Anglo-Egyptian set up (Campaign)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Anglo-Egyptian player moves first. There are no Anglo-Egyptian units on the mapsheet at start. The GORDON unit is not used in this scenario.

- The leader units KITCHENER, GATACRE, and HUNTER may enter anytime during the first four game turns and do not count against the 12 unit per turn limit. All three leaders must be in play by the end of turn four!
- All gunboats enter through any north edge Nile River hex, paying one movement point for the first hex entered. The "Friendlies" brigade enters through the Abu Alim hut hex on the east bank, paying eight movement points per unit. All other Anglo-Egyptian units enter through the west bank "ANGLO-EGYPTIAN ENTRANCE AREA", each unit paying one movement point to enter the mapsheet.

- Turn 1) Any three gunboats; "Friendlies" brigade; Egyptian Cavalry; Horse Artillery; and two infantry brigades from the Egyptian Division.
- Turn 2) Any three gunboats plus any twelve land units.
- Turn 3) Any three gunboats plus any twelve land units.
- Turn 4) All remaining Anglo-Egyptian units.]]
#v(0.5em)
#heading(level: 2, "§9.212 -- Dervish set up (Historical)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Dervish player sets up second, and moves first.

- Not in play: Isa Zachneih, gunboats, and forts.
- All Dervish units must be set up out of the line of sight of all Anglo-Egyptian units.
- Dervish leaders start on the lettered hexes:
  - A: Ali Wad Helu
  - D: Sheik El Din
  - Y: Yakub
  - K: Khalifa Abdullah
  - S: Sherif
  - O: Osman Digna
- All remaining Dervish units set up within three hexes of their leader as identified by color (e.g. all green units set up within three hexes of Sheik El Din).]]
#v(0.5em)
#heading(level: 2, "§9.231 -- Thorn hedge hexsides")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 815)], [#text[ZaribaThornHedge]], [#raw("    Terrain(i16),
    /// -2 thorn-hedge defensive modifier (§9.231).
    ZaribaThornHedge,
    /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
    /// units (those Nile-side of the trench hexside).", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 157)], [#text[ZaribaThornHedge]], [#raw("    Crest,
    /// Historical-scenario thorn-hedge segment of the Zariba (§9.231).
    ZaribaThornHedge,
    /// Historical-scenario trench segment of the Zariba (§9.232).
    ZaribaTrench,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.232 -- Trench hexsides")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 818)], [#text[ZaribaTrenchEntrenched]], [#raw("    /// -4 trench defensive modifier (§9.232). Only applies vs. \"entrenched\"
    /// units (those Nile-side of the trench hexside).
    ZaribaTrenchEntrenched,
}
", block: true, lang: "rs")],
  [#vscode-link("omdurman-types/src/lib.rs", 159)], [#text[ZaribaTrench]], [#raw("    ZaribaThornHedge,
    /// Historical-scenario trench segment of the Zariba (§9.232).
    ZaribaTrench,
    /// Khor Shambat -- the specific named khor that empties into the Nile (a
    /// scenario landmark; used as a setup/reinforcement boundary). Same blocking", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§9.233 -- Zariba entry/exit costs")
#status-tag("implicit")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Units may only enter and/or leave the Zariba via the two end hexsides that connect to the Nile River, paying +2 movement points to cross (Exception: advance after combat across an entrenched hexside).]]
#v(0.5em)
#heading(level: 2, "§9.346 -- Gunboat White Nile <-> Blue Nile (Bonus)")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The GORDON leader unit starts in the palace and may not move during the scenario. He may only be eliminated by a Dervish unit passing through or occupying the palace hex (as normal movement or as advance after combat).]]
#v(0.5em)
#heading(level: 1, "§10 -- Optional Rules")
#heading(level: 2, "§10 -- Optional Rules")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Optional Rules (Campaign game only)

It is suggested that the most intriguing employment of the following two options is to permit the Dervish player to have either one or the other, but the Anglo-Egyptian player doesn't know which one until he stumbles onto it. Players are advised that the employment of both optionals in the same game is not recommended.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/lib.rs", 318)], [#text[OptionalRule]], [#raw("/// two should be in play (rulebook §10).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OptionalRule {
    RiverMines,
    RiverChain,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§10.1 -- River Mines")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[River Mines

The Khalifa twice tried (unsuccessfully) to submerge a powerful mine in the Nile to sink or damage British gunboats. This option assumes that both attempts were successful.]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 122)], [#text[RiverMine]], [#raw("    // -- Optional rules ----------------------------------------------------
    /// River mine resolution (rulebook §10.12).
    RiverMine {
        gunboat_id: UnitId,
        hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1308)], [#text[apply_river_mine]], [#raw("
/// Apply a river-mine resolution (rulebook §10.12).
fn apply_river_mine(
    state: &mut GameState,
    gunboat_id: UnitId,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 980)], [#text[MineResult]], [#raw("/// British gunboat enters a mined hex.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MineResult {
    /// Roll 1-4: no effect.
    NoEffect,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§10.2 -- River Chain")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[River Chain

The Khalifa also tried (also unsuccessfully) to string a heavy chain across the Nile to stop or slow down the British gunboats. This option assumes the chain was emplaced.]]
#v(0.5em)
#heading(level: 2, "§10.11 -- Secretly record mine hexes")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Prior to the commencement of play the Dervish player secretly records two Nile River hexes to be mined (the mines may not both be placed in the same hex). These hexes must be south of the E–W hexrow in which the Khor Shambat empties into the Nile.]]
#v(0.5em)
#heading(level: 2, "§10.12 -- Mine resolution")
#status-tag("implemented")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[When a British gunboat enters a mined hex, the Dervish player must order it to stop as it has struck a mine. The Dervish player then resolves the effect of the mine's blast by rolling the ten-sided die:

- 1–4: No effect
- 5–7: Gunboat damaged, lost use of its engines and must drift two hexes per turn (with the current) for the rest of the game. No effect on guns or Maxims unless they drift out of range.
- 8–10: Gunboat sunk!]]
#v(0.5em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/effects.rs", 122)], [#text[RiverMine]], [#raw("    // -- Optional rules ----------------------------------------------------
    /// River mine resolution (rulebook §10.12).
    RiverMine {
        gunboat_id: UnitId,
        hex: HexCoord,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/effects.rs", 1308)], [#text[apply_river_mine]], [#raw("
/// Apply a river-mine resolution (rulebook §10.12).
fn apply_river_mine(
    state: &mut GameState,
    gunboat_id: UnitId,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 2, "§10.13 -- Mines consumed after both rolled for")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[After both mines have been rolled for, no more are available.]]
#v(0.5em)
#heading(level: 2, "§10.14 -- Dervish gunboats pass safely")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[The Dervish player's gunboats may pass through the mined hexes with no ill effect (he knows where the mines are).]]
#v(0.5em)
#heading(level: 2, "§10.21 -- Secretly record chain hexes")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Prior to the commencement of play the Dervish player secretly records a line of river hexes (not exceeding four hexes long) across which the chain is strung. The hexes must be south of the E–W hexrow in which the Khor Shambat empties into the Nile.]]
#v(0.5em)
#heading(level: 2, "§10.22 -- Gunboat stops on chained hex")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[When a British gunboat enters a "chained" river hex it must stop and may move no further that turn.]]
#v(0.5em)
#heading(level: 2, "§10.23 -- Sinking the chain")
#status-tag("out-of-scope")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[No gunboats (British or Dervish) may cross the chain until it has been sunk by the British player. He may sink the chain by a) having an infantry or cavalry unit spend one complete turn on either riverbank adjacent to a "chained" river hex, or b) firing at the chain with artillery and achieving a 3 or more on the Combat Results Table.]]
#v(0.5em)
#heading(level: 1, "§11 -- Historical Notes")
#heading(level: 2, "§11 -- Historical Notes")
#status-tag("descriptive")
#v(0.3em)
#block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[Historical Notes

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

In Khartoum, meanwhile, the garrison became daily more weakened by hunger and fatigue. If Gordon's disinclination to evacuate seems strange, then even stranger was the Mahdi's apparent reluctance to apply the coup de grace to the city. Even after the inevitable end became painfully obvious, he continued to offer Gordon honorable surrender terms, safe passage, and other concessions. Gordon, however, remained adamant. He had apparently prepared himself a martyr's place in history and would not be dissuaded from it except by the total capitulation of the Mahdi and his followers. Then the Mahdi was informed that the relief expedition was within a few days of Khartoum and decided the garrison must be taken at once. Thus it was that in the pre-dawn hours of January 25th, 1885, some 20,000 Dervishes poured through a gap in Khartoum's outer defenses where the receding White Nile had eroded away a section of wall. The garrison was slaughtered, Gordon among them (FALL OF KHARTOUM scenario — 9.3). Three days later (Col. Wilson's three days of rest?) the steamers carrying the advance guard of the strike force came within sight of Khartoum. Seeing only smoking ruins, they turned around and steamed back downstream to bring the news to the main body. Queen Victoria voiced the feelings of the nation when she recorded in her diary: "The government alone is to blame".

The relief column withdrew back into Egypt, and the fall of Khartoum thus effectively eliminated Britain's presence in the Sudan for the next eleven years, leaving that vast hinterland to the Mahdist empire. The Mahdi died in June of 1885 and was succeeded by the Khalifa, Abdullah the Taiasha, a chief of the Baggaras. The Khalifa made Omdurman his capital and expanded it from a few mud huts in 1885 to a vast, sprawling fifteen square mile urban slum by 1898. It housed the Dervishes' holiest shrine, the Mahdi's Tomb, as well as the palace and other structures in a walled city within a city.

By 1896 the spread of Mahdism led to British concern for the security of Egypt. In a move ostensibly made to take pressure off an Italian outpost on the Abyssinian border, London ordered an expedition into Dervish territory in the northern Sudan. It was led by General Herbert Kitchener, Sirdar (commander) of the Egyptian army. Kitchener had been a major in the Khartoum relief expedition and had never forgotten the rage and shame he felt when that force withdrew without attacking the Mahdi's army. An obsession to avenge Gordon's death stayed with him over the intervening years, so that he welcomed the instructions to move on the Sudan. To free himself from total dependence on the Nile for transportation, the Sudan Military Railroad was planned and overland construction begun. By July of 1896, Kitchener was underway. Progress was slow but steady, with the army halting periodically for the railway to catch up. Following infrequent skirmishing with the Dervishes, Kitchener's Egyptian Division under General Hunter re-occupied Berber in July of 1897. The balance of that year was spent reorganizing and re-supplying the army while again waiting for the railway to catch up.

If 1897 was the year of consolidation and organization, 1898 was the year in which those efforts bore fruit. Reinforced with a British brigade, the Sirdar's army was again on the move in March, 1898. After fighting three minor engagements during March and early April, the army (now the Anglo-Egyptian army) found itself confronted by a large Dervish force under Mahmud, one of the Khalifa's few remaining competent generals. Mahmud had entrenched his force inside a circular defensive zariba of camel thorn, with his back on the dry bed of the river Atbara, a strong defensive position. Mahmud, however, had not taken the new British heavy artillery into account and, after an hour and a half of heavy bombardment, the Sirdar's army went in, led by the Cameron Highlanders. Forty-five minutes later 3,000 Dervishes were dead at a loss to Kitchener of 80 men killed, and Mahmud was a prisoner. The way to Omdurman was open!

By mid-April the railroad had reached the Nile below Berber, bringing with it the new shallow draft gunboats designed specifically for river campaigns. The sections of these new iron monsters were assembled and floated in the Nile. One hundred and forty feet long by twenty-four feet wide and drawing only thirty-nine inches of water, they were formidable concentrations of firepower with their 12 pounders, 6 pounders, and Maxim guns on the upper deck, and 4 inch howitzers on the gun deck. By August 17th all was in readiness and, reinforced with a second British brigade, Kitchener marched steadily south, arriving at the little mud village of Kerreri on September 1st (CAMPAIGN GAME scenario — 9.1).

The Khalifa, Abdullah the Taiasha, in the meantime, had not been idle. Throughout the Spring and Summer of 1898, the Sudan experienced a hectic and frantic mobilization as the leading Emirs of the empire gathered the faithful to the Jihad, or holy war. Estimates of the response vary widely, but it seems likely that some 60–70,000 warriors answered the call and assembled on the plains of Kerreri, north of Omdurman. To guard the approaches to the city, seventeen forts were constructed and armed with old artillery pieces. The few guns available, old Remingtons and brass muzzle loaders using home made cartridges, were issued to the Jehadia (commanded by the Khalifa's son, Sheik El Din) and the Danagla. The rest of the troops carried swords and spears.

Dawn of September 2nd saw the Sirdar and his Anglo-Egyptian army positioned inside a rough semi-circular formation protected by a zariba of thorn hedge and trenches. His back and flanks were on the Nile and guarded by the gunboats. At dawn the cavalry had gone out, but by 6:30 they were back in. Then they came — the Dervishes in their thousands and tens of thousands pouring over the ridges of the Jebel Surgham and the Kerreri Hills (HISTORICAL scenario — 9.2).

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
- Howitzer fire: range 4–10 hexes; target hex hit on impact roll 7–10; otherwise scatters per Howitzer Fire Scattergram.]]
#v(0.5em)
#heading(level: 1, "Credits")
#heading(level: 2, "§Credits -- Credits")
#status-tag("descriptive")
#v(0.3em)
#heading(level: 1, "Combat Results Table (shared reference)")
#heading(level: 2, "§CRT -- Combat Results Table (shared by §6.22 fire and §7.7 melee)")
#status-tag("implemented")
#v(0.3em)
#table(
  columns: (1fr, 3fr, 4.5fr),
  stroke: 0.4pt + luma(190),
  [*File*], [*Symbol*], [*Code Snippet*],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 8)], [#text[FireFactorRow]], [#raw("/// to index into the result matrix.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FireFactorRow {
    /// 1-5 factors
    Row01to05,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 31)], [#text[from_total]], [#raw("impl FireFactorRow {
    /// Determine which row a given total fire factor falls into (rulebook §6.22).
    pub fn from_total(total: u16) -> Self {
        match total {
            0..=5 => FireFactorRow::Row01to05,", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/combat_results_table.rs", 53)], [#text[combat_results_table]], [#raw("/// D = `Disrupt` (1/2 of target units, round up)
/// 1...5 = `Eliminate(n)` (that many units removed)
pub fn combat_results_table(row: FireFactorRow, roll: DieRoll) -> CombatResult {
    use CombatResult::*;
    use DieRoll::*;", block: true, lang: "rs")],
  [#vscode-link("omdurman-rules/src/lib.rs", 879)], [#text[CombatResult]], [#raw("/// * `--` -- no effect
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CombatResult {
    NoEffect,
    Disrupt,", block: true, lang: "rs")],
)
#v(0.5em)
#heading(level: 1, "Reference -- Charts and Tables")
#heading(level: 2, "§Reference -- Charts and Tables")
#status-tag("out-of-scope")
#v(0.3em)

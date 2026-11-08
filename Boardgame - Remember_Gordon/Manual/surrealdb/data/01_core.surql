-- Factions, maps, components, markers, constants, combat results, states, weapons, range tables.

-- ------------------------------------------------------------------ factions (1.1, 3, 9.32)
INSERT INTO faction [
  { id: faction:anglo_egyptian, name: "Anglo-Egyptian", aka: ["British"], player_role: "Herbert Kitchener, Sirdar (CIC) of the Anglo-Egyptian army",
    seat: "east edge", section: section:`1.1`, note: "Called 'the British player' in the FALL OF KHARTOUM bonus game (9.321)." },
  { id: faction:dervish, name: "Dervish", aka: ["Mahdist"], player_role: "the Khalifa, Abdullah the Taiasha, absolute ruler of the Dervishes",
    seat: "west edge", section: section:`1.1` }
];
UPDATE faction:anglo_egyptian SET opponent = faction:dervish, seat_section = section:`3`;
UPDATE faction:dervish SET opponent = faction:anglo_egyptian, seat_section = section:`3`;

-- ------------------------------------------------------------------ maps (2.1, 2.4)
INSERT INTO map [
  { id: map:omdurman, name: "Battle of Omdurman mapsheet", size_inches: [22, 28], area_sq_miles: 100, area_basis: "approximately",
    depicts: "the area north of Omdurman in which the historical battle took place", year: 1898,
    printed_aids: [component:turn_record_track, component:turn_sequence_chart, component:terrain_effects_chart, component:combat_tables, component:howitzer_scattergram],
    all_full_hexes_playable: true, section: section:`2.1` },
  { id: map:khartoum, name: "FALL OF KHARTOUM mini-map", size_inches: [8.5, 11], depicts: "Khartoum as it appeared in 1885", year: 1885,
    abuts: { own_edge: "north", other_map: map:omdurman, other_edge: "south (middle portion)" },
    wall_gap: "The portion of wall conspicuous by its absence represents the area washed away by the receding White Nile after the flood.",
    all_hexes_playable_including_partial: true, section: section:`2.1`, partial_hex_section: section:`9.342` }
];

-- ------------------------------------------------------------------ components (2.2, 2.4)
INSERT INTO component [
  { id: component:omdurman_mapsheet, name: "Battle of Omdurman mapsheet", kind: "map", count: 1, size: "22 × 28", map: map:omdurman, section: section:`2.4` },
  { id: component:khartoum_mapsheet, name: "FALL OF KHARTOUM mapsheet", kind: "map", count: 1, size: "8½ × 11", map: map:khartoum, section: section:`2.4` },
  { id: component:rules_booklet, name: "Rules Booklet", kind: "rules", count: 1, section: section:`2.4` },
  { id: component:counter_sheet, name: "Unit Counter Sheet", kind: "counters", count: 1, note: "die-cut", section: section:`2.4` },
  { id: component:order_of_appearance_card, name: "Campaign Game Order of Appearance Card", kind: "card", count: 1, section: section:`2.4` },
  { id: component:d10, name: "ten-sided die", kind: "die", count: 1, faces: 10, section: section:`2.4` },
  { id: component:game_box, name: "game box", kind: "box", count: 1, section: section:`2.4` },
  { id: component:terrain_effects_chart, name: "Terrain Effects Chart", kind: "play_aid", printed_on: "top of the Omdurman mapsheet",
    purpose: "lists all terrain found on the mapsheet and the effect of each type on movement and combat", section: section:`2.2` },
  { id: component:combat_tables, name: "Combat Tables", kind: "play_aid", printed_on: "lower right corner of the Omdurman mapsheet",
    purpose: "describe the range effects on various weapon types and include the Combat Results Table",
    contains: [component:range_effects_table_dervish, component:range_effects_table_ae, component:combat_results_table], section: section:`2.2` },
  { id: component:range_effects_table_dervish, name: "Dervish Range Effects Table", kind: "play_aid", part_of: component:combat_tables,
    range_table: range_table:dervish, section: section:`2.31` },
  { id: component:range_effects_table_ae, name: "Anglo-Egyptian Range Effects Table", kind: "play_aid", part_of: component:combat_tables,
    range_table: range_table:anglo_egyptian, section: section:`2.32` },
  { id: component:combat_results_table, name: "Combat Results Table", kind: "play_aid", part_of: component:combat_tables,
    also_printed_on: "back cover of the rulebook (with the Explanation of Combat Results)",
    purpose: "cross-index the net die roll with the number of combat factors firing", section: section:`6.23` },
  { id: component:los_table, name: "Line of Sight Table", kind: "play_aid", printed_on: "back of the rulebook",
    purpose: "tells players when certain terrain types block line of sight, thus preventing direct fire attacks on enemy units",
    usage: "Locate the terrain type the firing unit is in and cross-index it with the terrain type the target unit is in; terrain types in the intersecting box block line of sight, with exceptions as footnoted.",
    section: section:`6.3` },
  { id: component:special_los_notes, name: "Special LOS Notes", kind: "play_aid", part_of: component:los_table, section: section:`6.3` },
  { id: component:howitzer_scattergram, name: "Howitzer Fire Scattergram", kind: "play_aid", printed_on: "lower right corner of the Omdurman mapsheet",
    hexes: 7, hexes_playable: true, section: section:`2.1`, use_section: section:`6.64` },
  { id: component:turn_record_track, name: "Turn Record Track", kind: "play_aid", printed_on: "top of the Omdurman mapsheet", boxes: 22, night_boxes: 4,
    layout: "three rows of boxes joined by arrows (left to right, back right to left, then left to right again); boxes carry no turn numbers",
    section: section:`TurnTrack` },
  { id: component:turn_sequence_chart, name: "Turn Sequence", kind: "play_aid", printed_on: "top of the Omdurman mapsheet", section: section:`2.1` }
];

-- ------------------------------------------------------------------ markers
INSERT INTO marker [
  { id: marker:game_turn, name: "Game Turn marker", used_on: component:turn_record_track, purpose: "advanced to the next box after both Player Turns", section: section:`4.C` },
  { id: marker:breach, name: "BREACH marker", placement: "in a hex adjacent to the breached wall hexside, arrow pointing to the breached hexside", section: section:`6.63`, also: section:`6.53` },
  { id: marker:blank, name: "blank counter", purpose: "denote units constructing Zariba hexsides", section: section:`5.3` }
];

-- ------------------------------------------------------------------ constants
INSERT INTO constant [
  { id: constant:hex_yards, name: "hex size in yards", min: 400, max: 440, basis_word: "approximately", section: section:`1.2` },
  { id: constant:day_turn_hours, name: "real time per day game turn (hours)", value: 2, section: section:`1.2` },
  { id: constant:men_per_counter, name: "men per infantry or cavalry counter", min: 400, max: 700, section: section:`1.2` },
  { id: constant:gunboat_counter_scale, name: "gunboats per gunboat counter", value: 1, section: section:`1.2` },
  { id: constant:map_area_sq_miles, name: "Omdurman map area (square miles)", value: 100, basis_word: "approximately", section: section:`2.1` },
  { id: constant:stacking_limit, name: "maximum units per hex (leaders and gunboats excepted)", value: 4, section: section:`5.51` },
  { id: constant:brigade_battalions, name: "infantry battalions in an Anglo-Egyptian infantry brigade", value: 4, section: section:`5.54` },
  { id: constant:howitzer_min_range, name: "howitzer minimum target distance (hexes)", value: 4, section: section:`6.64` },
  { id: constant:howitzer_max_range, name: "howitzer maximum target distance (hexes)", value: 10, section: section:`6.64` },
  { id: constant:howitzer_units, name: "units with howitzer fire capability", value: 5, section: section:`6.64` },
  { id: constant:howitzer_rolls, name: "die rolls per howitzer fire", value: 2, section: section:`6.64` },
  { id: constant:maxim_fires_per_turn, name: "fires per turn of Maxim guns", value: 2, section: section:`2.3` },
  { id: constant:cav_retreat_hexes, name: "hexes a cavalry/camel unit may retreat from infantry melee", value: 2, section: section:`7.5` },
  { id: constant:retreats_per_unit_turn, name: "retreats per unit per turn", value: 1, section: section:`7.5` },
  { id: constant:desertion_multiplier, name: "deserting units per pip of the desertion die", value: 1.5, section: section:`8.2` },
  { id: constant:desertion_rolls, name: "desertion rolls per campaign game", value: 1, section: section:`8.2` },
  { id: constant:campaign_forts, name: "Dervish forts in the campaign game", value: 17, section: section:`9.111` },
  { id: constant:dervish_artillery_units, name: "Dervish artillery units", value: 3, section: section:`5.23` },
  { id: constant:dervish_gunboats, name: "Dervish gunboats (campaign game)", value: 2, section: section:`9.111` },
  { id: constant:ae_land_units_per_turn, name: "Anglo-Egyptian land units entering per turn, campaign turns 2-3", value: 12, section: section:`9.113` },
  { id: constant:ae_gunboats_per_turn, name: "Anglo-Egyptian gunboats entering per turn, campaign turns 1-3", value: 3, section: section:`9.113` },
  { id: constant:ae_leader_deadline_turn, name: "turn by whose end all three Anglo-Egyptian leaders must be in play", value: 4, section: section:`9.113` },
  { id: constant:zariba_hexes, name: "hexes of the Zariba", value: 13, section: section:`9.211` },
  { id: constant:historical_leader_radius, name: "hexes from their leader within which Dervish units set up (historical)", value: 3, section: section:`9.212` },
  { id: constant:zariba_crossing_mp, name: "extra MP to cross a Zariba end hexside", value: 2, section: section:`9.233` },
  { id: constant:tomb_vp, name: "victory points for controlling the Mahdi's Tomb", value: 25, section: section:`9.14` },
  { id: constant:mines, name: "river mines (optional rule)", value: 2, section: section:`10.11` },
  { id: constant:chain_max_hexes, name: "maximum length of the river chain (hexes)", value: 4, section: section:`10.21` },
  { id: constant:mine_drift_hexes, name: "hexes per turn a mine-damaged gunboat drifts", value: 2, section: section:`10.12` },
  { id: constant:nile_switch_cost, name: "off-board upstream MP to move a gunboat between White and Blue Nile (Khartoum)", value: 6, section: section:`9.345` },
  { id: constant:friendlies_entry_mp, name: "MP each Friendlies unit pays to enter at Abu Alim (campaign)", value: 8, section: section:`9.113` },
  { id: constant:die_min, name: "lowest usable modified die roll", value: 1, section: section:`CombatResults` },
  { id: constant:die_max, name: "highest usable modified die roll", value: 10, section: section:`CombatResults` }
];

-- ------------------------------------------------------------------ combat results (back cover)
INSERT INTO combat_result [
  { id: combat_result:miss, symbol: "—", meaning: "miss, no effect", section: section:`CombatResults` },
  { id: combat_result:disrupted, symbol: "D", meaning: "½ (round up) of the units in the target hex are disrupted (inverted).",
    fraction: 0.5, rounding: "up", applies_state: state:disrupted, section: section:`CombatResults` },
  { id: combat_result:eliminated, symbol: "#", meaning: "That many units in the target hex are eliminated, i.e. removed from play.", section: section:`CombatResults` }
];

-- ------------------------------------------------------------------ states
INSERT INTO state [
  { id: state:disrupted, name: "disrupted", marked_by: "counter inverted", cause: combat_result:disrupted,
    effects: ["have no ZOC", "may not move", "may not fire offensively or defensively", "may not melee"],
    recovery: "turned face up at the end of the owning player's turn", section: section:`Disrupted`, also: [section:`5.41`] },
  { id: state:eliminated, name: "eliminated", meaning: "removed from play", section: section:`CombatResults` },
  { id: state:entrenched, name: "entrenched",
    definition: "directly adjacent to (and on the Nile River side of) a trench hexside",
    scenarios: [scenario:historical, scenario:campaign], campaign_condition: "Zariba constructed (5.3)", section: section:`9.232` },
  { id: state:brigade_integrity, name: "brigade integrity",
    definition: "all four infantry battalions belonging to any Anglo-Egyptian infantry brigade are stacked in the same hex (in a combat phase)",
    section: section:`5.54` },
  { id: state:constructing_zariba, name: "constructing Zariba", marked_by: marker:blank, section: section:`5.3` },
  { id: state:loaded, name: "loaded on a gunboat", applies_to: "Friendlies unit stacked with an Anglo-Egyptian gunboat during transport", section: section:`5.21` },
  { id: state:demolishing, name: "performing demolitions", applies_to: "Royal Engineers adjacent to a fort or wall hexside", section: section:`6.53` },
  { id: state:mine_damaged, name: "mine-damaged gunboat", definition: "lost use of its engines; drifts two hexes per turn with the current for the rest of the game; no effect on guns or Maxims unless they drift out of range",
    section: section:`10.12` },
  { id: state:in_enemy_zoc, name: "in an enemy ZOC", definition: "must stop and may move no further that turn", section: section:`5.43` },
  { id: state:night, name: "night game turn", kind: "turn condition", section: section:`8.1` }
];

-- ------------------------------------------------------------------ weapons, range tables, range lines (2.31, 2.32)
INSERT INTO weapon [
  { id: weapon:artillery, name: "artillery", section: section:`2.31` },
  { id: weapon:rifles, name: "rifles", section: section:`2.31` },
  { id: weapon:spears_and_swords, name: "spears and swords", section: section:`2.31` },
  { id: weapon:maxims, name: "Maxim guns", section: section:`2.32` },
  { id: weapon:howitzer, name: "howitzer", section: section:`2.32` }
];
INSERT INTO range_table [
  { id: range_table:dervish, name: "Dervish Range Effects Table", faction: faction:dervish, section: section:`2.31` },
  { id: range_table:anglo_egyptian, name: "Anglo-Egyptian Range Effects Table", faction: faction:anglo_egyptian, section: section:`2.32` }
];
INSERT INTO range_line [
  { id: range_line:d_artillery, name: "artillery", table: range_table:dervish, weapon: weapon:artillery, basis: "explicit", section: section:`2.31` },
  { id: range_line:d_rifles, name: "rifles", table: range_table:dervish, weapon: weapon:rifles, basis: "explicit", section: section:`2.31` },
  { id: range_line:d_spears_swords, name: "spears and swords", table: range_table:dervish, weapon: weapon:spears_and_swords, basis: "implied",
    note: "2.31 says the other Dervish units 'are armed with spears and swords' without naming a table line for them.", section: section:`2.31` },
  { id: range_line:ae_rifles, name: "Rifles", table: range_table:anglo_egyptian, weapon: weapon:rifles, basis: "explicit", section: section:`2.32` },
  { id: range_line:ae_maxims, name: "Maxims", table: range_table:anglo_egyptian, weapon: weapon:maxims, basis: "explicit", section: section:`2.32` },
  { id: range_line:ae_artillery, name: "Artillery", table: range_table:anglo_egyptian, weapon: weapon:artillery, basis: "explicit", section: section:`2.32` },
  { id: range_line:ae_howitzer, name: "Howitzer", table: range_table:anglo_egyptian, weapon: weapon:howitzer, basis: "explicit", section: section:`2.32` }
];
-- Range multipliers listed by the Range Effects Table (6.22); per-range cells are not in the manual.
UPSERT constant:range_multipliers CONTENT { name: "possible Range Effects Table outcomes", values: ["tripled", "doubled", "normal", "halved", "out of range"], factors: [3, 2, 1, 0.5], section: section:`6.22` };

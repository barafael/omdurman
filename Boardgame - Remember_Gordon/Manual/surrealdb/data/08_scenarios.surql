-- Scenario set-ups, reinforcements, victory points, victory levels, victory conditions.

-- ------------------------------------------------------------------ set-up
INSERT INTO setup [
  -- campaign, Dervish (9.111)
  { id: setup:c_isa, scenario: scenario:campaign, faction: faction:dervish, subjects: [unit:isa_zachneih], quantity: 1,
    locations: [location:east_bank, location:el_debeba], constraint: "anywhere on the east bank, in or south of El Debeba", section: section:`9.111` },
  { id: setup:c_khalifa, scenario: scenario:campaign, faction: faction:dervish, subjects: [unit:khalifa_abdullah], quantity: 1,
    locations: [location:omdurman_palace], constraint: "in the walled city of Omdurman, in either palace hex", section: section:`9.111` },
  { id: setup:c_city_garrison, scenario: scenario:campaign, faction: faction:dervish, subjects: [unit:dervish_artillery, tribe:taiasha], quantity_text: "3 artillery units and all Taiasha units",
    locations: [location:walled_city], constraint: "anywhere in the walled city of Omdurman", section: section:`9.111` },
  { id: setup:c_forts, scenario: scenario:campaign, faction: faction:dervish, subjects: [unit:fort], quantity: 17,
    locations: [location:west_bank, location:khor_shambat, location:east_bank, location:halfaya, location:nile_islands],
    constraint: "anywhere on the mapsheet south of the Khor Shambat on the west bank, and/or south of all Halfaya hut hexes on the east bank and Nile River islands", section: section:`9.111` },
  { id: setup:c_dervish_gunboats, scenario: scenario:campaign, faction: faction:dervish, subjects: [unit:dervish_gunboat], quantity: 2,
    locations: [location:south_edge_nile], constraint: "any south edge Nile River hexes", section: section:`9.111` },
  { id: setup:c_ae_none, scenario: scenario:campaign, faction: faction:anglo_egyptian, subjects: [], quantity: 0, constraint: "There are no Anglo-Egyptian units on the mapsheet at start.", section: section:`9.113` },

  -- historical, Anglo-Egyptian (9.211)
  { id: setup:h_gunboats, scenario: scenario:historical, faction: faction:anglo_egyptian, subjects: [unit:old_gunboat, unit:sultan], subject_text: "gunboats",
    locations: [location:nile, location:zariba], constraint: "any Nile River hexes adjacent to the Zariba", section: section:`9.211` },
  { id: setup:h_kerreri, scenario: scenario:historical, faction: faction:anglo_egyptian, subjects: [unit:camel_corps, unit:egyptian_cavalry, unit:horse_artillery],
    locations: [location:kerreri], constraint: "the village of Kerreri hut hexes", section: section:`9.211` },
  { id: setup:h_zariba, scenario: scenario:historical, faction: faction:anglo_egyptian, subject_text: "all remaining Anglo-Egyptian units",
    subjects: [unit:kitchener, unit:gatacre, unit:hunter, unit:lancers_21, unit:battery_32, unit:maxim_battery, unit:royal_engineers, formation:brigade_2b, formation:brigade_3e, formation:brigade_4e],
    subjects_note: "subjects lists the remaining units the manual names; the rest of the counter mix is not in the manual",
    locations: [location:zariba], constraint: "set up in the 13 hexes of the Zariba", section: section:`9.211` },
  -- historical, Dervish (9.212)
  { id: setup:h_leader_ali, scenario: scenario:historical, faction: faction:dervish, subjects: [unit:ali_wad_helu], locations: [location:hex_a], constraint: "lettered hex A", section: section:`9.212` },
  { id: setup:h_leader_sheik, scenario: scenario:historical, faction: faction:dervish, subjects: [unit:sheik_el_din], locations: [location:hex_d], constraint: "lettered hex D", section: section:`9.212` },
  { id: setup:h_leader_yakub, scenario: scenario:historical, faction: faction:dervish, subjects: [unit:yakub], locations: [location:hex_y], constraint: "lettered hex Y", section: section:`9.212` },
  { id: setup:h_leader_khalifa, scenario: scenario:historical, faction: faction:dervish, subjects: [unit:khalifa_abdullah], locations: [location:hex_k], constraint: "lettered hex K", section: section:`9.212` },
  { id: setup:h_leader_sherif, scenario: scenario:historical, faction: faction:dervish, subjects: [unit:sherif], locations: [location:hex_s], constraint: "lettered hex S", section: section:`9.212` },
  { id: setup:h_leader_osman, scenario: scenario:historical, faction: faction:dervish, subjects: [unit:osman_digna], locations: [location:hex_o], constraint: "lettered hex O", section: section:`9.212` },
  { id: setup:h_dervish_rest, scenario: scenario:historical, faction: faction:dervish, subject_text: "all remaining Dervish units",
    subjects: [tribe:baggara, tribe:jaalin, tribe:danagla, tribe:kehena, tribe:degheim, tribe:hadendowa, tribe:mulazmin, tribe:jehadia, tribe:taiasha, unit:dervish_artillery],
    constraint: "within three hexes of their leader as identified by color (e.g. all green units within three hexes of Sheik El Din); all out of the line of sight of all Anglo-Egyptian units", radius: 3, section: section:`9.212` },

  -- Fall of Khartoum, British (9.321)
  { id: setup:k_gordon, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, subjects: [unit:gordon], quantity: 1, locations: [location:khartoum_palace], constraint: "in the palace", section: section:`9.321` },
  { id: setup:k_gunboats, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, subjects: [unit:old_gunboat], quantity: 2, locations: [location:white_nile, location:blue_nile],
    constraint: "two old style (unnamed) gunboats in any Nile River hexes", section: section:`9.321` },
  { id: setup:k_artillery, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, subjects: [unit:egyptian_battalion_artillery], quantity: 1,
    locations: [location:khartoum, location:fort_makran, location:fort_buri, location:khartoum_wall], constraint: "any building or hut hexes of Khartoum, Forts Makran and/or Buri, and/or adjacent to any wall hex", section: section:`9.321` },
  { id: setup:k_british, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, subjects: [unit_class:british_infantry], quantity: 2, represents: "Caucasian troops",
    locations: [location:khartoum, location:fort_makran, location:fort_buri, location:khartoum_wall], constraint: "any building or hut hexes of Khartoum, Forts Makran and/or Buri, and/or adjacent to any wall hex", section: section:`9.321` },
  { id: setup:k_egyptian, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, subjects: [unit_class:egyptian_infantry], quantity: 3, represents: "Cairo \"Bazouks\"",
    locations: [location:khartoum, location:fort_makran, location:fort_buri, location:khartoum_wall], constraint: "any building or hut hexes of Khartoum, Forts Makran and/or Buri, and/or adjacent to any wall hex", section: section:`9.321` },
  { id: setup:k_sudan, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, subjects: [unit_class:sudanese_infantry], quantity: 4, represents: "Sudanese blacks",
    locations: [location:khartoum, location:fort_makran, location:fort_buri, location:khartoum_wall], constraint: "any building or hut hexes of Khartoum, Forts Makran and/or Buri, and/or adjacent to any wall hex", section: section:`9.321` },
  { id: setup:k_friendlies, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, subjects: [unit_class:friendlies], quantity: 4, represents: "the Shaggyeh",
    locations: [location:khartoum, location:fort_makran, location:fort_buri, location:khartoum_wall], constraint: "any building or hut hexes of Khartoum, Forts Makran and/or Buri, and/or adjacent to any wall hex", section: section:`9.321` }
];

-- not in play
INSERT INTO setup [
  { id: setup:c_out_gordon, scenario: scenario:campaign, faction: faction:anglo_egyptian, not_in_play: true, subjects: [unit:gordon], section: section:`9.113` },
  { id: setup:h_out_ae, scenario: scenario:historical, faction: faction:anglo_egyptian, not_in_play: true, subjects: [unit:gordon, formation:friendlies_brigade], section: section:`9.211` },
  { id: setup:h_out_dervish, scenario: scenario:historical, faction: faction:dervish, not_in_play: true, subjects: [unit:isa_zachneih, unit:dervish_gunboat, unit:fort], section: section:`9.212` }
];

-- ------------------------------------------------------------------ reinforcements / entry
INSERT INTO reinforcement [
  -- campaign Dervish (9.112)
  { id: reinforcement:c_d1, scenario: scenario:campaign, faction: faction:dervish, turn: 1,
    subjects: [tribe:baggara, tribe:jaalin, tribe:danagla, tribe:kehena, tribe:degheim, unit:yakub, unit:sherif, unit:ali_wad_helu],
    text: "all Baggara, Jaalin, Danagla, Kehena, and Degheim units, and their leaders: YAKUB, SHERIF, and ALI WAD HELU",
    entry: [location:west_edge], entry_text: "west edge of the mapsheet, south of the Khor Shambat", entry_cost: "terrain cost of the entry hex, per unit", section: section:`9.112` },
  { id: reinforcement:c_d2, scenario: scenario:campaign, faction: faction:dervish, turn: 2, subjects: [tribe:hadendowa, unit:osman_digna],
    text: "all Hadendowa units and their leader, OSMAN DIGNA", entry: [location:west_edge], entry_text: "west edge of the mapsheet, south of the Khor Shambat", entry_cost: "terrain cost of the entry hex, per unit", section: section:`9.112` },
  { id: reinforcement:c_d3, scenario: scenario:campaign, faction: faction:dervish, turn: 3, subjects: [tribe:mulazmin, tribe:jehadia, unit:sheik_el_din],
    text: "all Mulazmin and Jehadia units and their leader, SHEIK EL DIN", entry: [location:west_edge], entry_text: "west edge of the mapsheet, south of the Khor Shambat", entry_cost: "terrain cost of the entry hex, per unit", section: section:`9.112` },
  -- campaign Anglo-Egyptian (9.113)
  { id: reinforcement:c_ae1, scenario: scenario:campaign, faction: faction:anglo_egyptian, turn: 1,
    subjects: [unit:old_gunboat, unit:sultan, formation:friendlies_brigade, unit:egyptian_cavalry, unit:horse_artillery, formation:egyptian_division],
    text: "Any three gunboats; \"Friendlies\" brigade; Egyptian Cavalry; Horse Artillery; and two infantry brigades from the Egyptian Division",
    gunboats: 3, infantry_brigades: 2, section: section:`9.113` },
  { id: reinforcement:c_ae2, scenario: scenario:campaign, faction: faction:anglo_egyptian, turn: 2, text: "Any three gunboats plus any twelve land units", gunboats: 3, land_units: 12, section: section:`9.113` },
  { id: reinforcement:c_ae3, scenario: scenario:campaign, faction: faction:anglo_egyptian, turn: 3, text: "Any three gunboats plus any twelve land units", gunboats: 3, land_units: 12, section: section:`9.113` },
  { id: reinforcement:c_ae4, scenario: scenario:campaign, faction: faction:anglo_egyptian, turn: 4, text: "All remaining Anglo-Egyptian units", all_remaining: true, section: section:`9.113` },
  { id: reinforcement:c_ae_leaders, scenario: scenario:campaign, faction: faction:anglo_egyptian, turn_window: [1, 2, 3, 4], deadline_turn: 4,
    subjects: [unit:kitchener, unit:gatacre, unit:hunter], counts_against_limit: false,
    text: "may enter anytime during the first four game turns and do not count against the 12 units per turn limit; all three must be in play by the end of turn four", section: section:`9.113` },
  -- Fall of Khartoum Dervish (9.322)
  { id: reinforcement:k_d1, scenario: scenario:fall_of_khartoum, faction: faction:dervish, turn: 1,
    subjects: [tribe:mulazmin, tribe:hadendowa, tribe:kehena, tribe:degheim, unit:dervish_artillery],
    counts: [{ subject: tribe:mulazmin, n: 32, represents: "combined forces of Wad El Nejumi, Abu Girgeh, and Sheik El Obeid" },
             { subject: tribe:hadendowa, n: 2, represents: "Mahdi's combined west bank forces" },
             { subject: tribe:kehena, n: 6, represents: "Mahdi's combined west bank forces" },
             { subject: tribe:degheim, n: 5, represents: "Mahdi's combined west bank forces" },
             { subject: unit:dervish_artillery, n: 3 }],
    total_units: 48, entry: [location:khartoum_south_edge, location:khartoum_east_edge], entry_text: "any hexes on the south or east edge of the map", section: section:`9.322` }
];
-- Anglo-Egyptian entry points and costs (9.113)
INSERT INTO reinforcement [
  { id: reinforcement:c_ae_entry_gunboats, scenario: scenario:campaign, faction: faction:anglo_egyptian, entry_rule: true, subjects: [unit_class:gunboat],
    entry: [location:north_edge_nile], entry_cost_mp: 1, entry_text: "through any north edge Nile River hex, paying one movement point for the first hex entered", section: section:`9.113` },
  { id: reinforcement:c_ae_entry_friendlies, scenario: scenario:campaign, faction: faction:anglo_egyptian, entry_rule: true, subjects: [formation:friendlies_brigade, unit_class:friendlies],
    entry: [location:abu_alim], entry_cost_mp: 8, entry_text: "through the Abu Alim hut hex on the east bank, paying eight movement points per unit", section: section:`9.113` },
  { id: reinforcement:c_ae_entry_others, scenario: scenario:campaign, faction: faction:anglo_egyptian, entry_rule: true, subjects_text: "all other Anglo-Egyptian units",
    entry: [location:ae_entrance_area], entry_cost_mp: 1, entry_text: "through the west bank ANGLO-EGYPTIAN ENTRANCE AREA, each unit paying one movement point to enter the mapsheet", section: section:`9.113` }
];
UPDATE reinforcement SET total_units_note = "32 + 2 + 6 + 5 + 3 = 48 (sum of 9.322's counts)" WHERE id = reinforcement:k_d1;

-- ------------------------------------------------------------------ victory points (campaign, 9.14)
INSERT INTO vp_award [
  { id: vp_award:tomb, scenario: scenario:campaign, beneficiary: "controller", points: 25, event: "controls the Mahdi's Tomb at the conclusion of play", subject: location:mahdis_tomb, initial_controller: faction:dervish, section: section:`9.14` },
  { id: vp_award:d_british_leader, scenario: scenario:campaign, beneficiary: faction:dervish, points: 10, event: "each British leader eliminated", subject: unit_class:ae_leader, section: section:`9.14` },
  { id: vp_award:d_gunboat, scenario: scenario:campaign, beneficiary: faction:dervish, points: 10, event: "each British gunboat sunk", subject: unit_class:gunboat, section: section:`9.14` },
  { id: vp_award:d_friendlies_east, scenario: scenario:campaign, beneficiary: faction:dervish, points: 1, event: "each Friendlies unit eliminated on the east bank side", subject: unit_class:friendlies, where: location:east_bank, section: section:`9.14` },
  { id: vp_award:d_friendlies_west, scenario: scenario:campaign, beneficiary: faction:dervish, points: 3, event: "each Friendlies unit eliminated on the west bank (see 5.21)", subject: unit_class:friendlies, where: location:west_bank, section: section:`9.14` },
  { id: vp_award:d_land_unit, scenario: scenario:campaign, beneficiary: faction:dervish, points: 3, event: "each Anglo-Egyptian land unit eliminated", subject: unit_class:land_unit, section: section:`9.14` },
  { id: vp_award:ae_fort, scenario: scenario:campaign, beneficiary: faction:anglo_egyptian, points: 0, event: "eliminating forts", subject: unit_class:fort, section: section:`9.14` },
  { id: vp_award:ae_isa, scenario: scenario:campaign, beneficiary: faction:anglo_egyptian, points: 1, event: "eliminating Isa Zachneih unit", subject: unit:isa_zachneih, section: section:`9.14` },
  { id: vp_award:ae_khalifa, scenario: scenario:campaign, beneficiary: faction:anglo_egyptian, points: 10, event: "eliminating KHALIFA ABDULLAH", subject: unit:khalifa_abdullah, section: section:`9.14` },
  { id: vp_award:ae_dervish_unit, scenario: scenario:campaign, beneficiary: faction:anglo_egyptian, points: 1, event: "each Dervish unit eliminated, including gunboats, artillery and all other leaders", subject: unit_class:unit, section: section:`9.14` },
  { id: vp_award:ae_deserters, scenario: scenario:campaign, beneficiary: faction:anglo_egyptian, points: 0, event: "deserting Dervishes", subject: action:desert, section: section:`8.2` }
];

-- ------------------------------------------------------------------ victory levels
INSERT INTO victory_level [
  -- campaign: point superiority (9.14)
  { id: victory_level:c_d_decisive, scenario: scenario:campaign, faction: faction:dervish, level: "Decisive", rank: 4, min: 30, max: NONE, basis: "victory point superiority", section: section:`9.14` },
  { id: victory_level:c_d_tactical, scenario: scenario:campaign, faction: faction:dervish, level: "Tactical", rank: 3, min: 20, max: 29, basis: "victory point superiority", section: section:`9.14` },
  { id: victory_level:c_d_marginal, scenario: scenario:campaign, faction: faction:dervish, level: "Marginal", rank: 2, min: 10, max: 19, basis: "victory point superiority", section: section:`9.14` },
  { id: victory_level:c_d_draw, scenario: scenario:campaign, faction: faction:dervish, level: "Draw", rank: 1, min: 1, max: 9, basis: "victory point superiority", section: section:`9.14` },
  { id: victory_level:c_ae_decisive, scenario: scenario:campaign, faction: faction:anglo_egyptian, level: "Decisive", rank: 4, min: 50, max: NONE, basis: "victory point superiority", section: section:`9.14` },
  { id: victory_level:c_ae_tactical, scenario: scenario:campaign, faction: faction:anglo_egyptian, level: "Tactical", rank: 3, min: 30, max: 49, basis: "victory point superiority", section: section:`9.14` },
  { id: victory_level:c_ae_marginal, scenario: scenario:campaign, faction: faction:anglo_egyptian, level: "Marginal", rank: 2, min: 15, max: 29, basis: "victory point superiority", section: section:`9.14` },
  { id: victory_level:c_ae_draw, scenario: scenario:campaign, faction: faction:anglo_egyptian, level: "Draw", rank: 1, min: 1, max: 14, basis: "victory point superiority", section: section:`9.14` },
  -- historical: units eliminated (9.24)
  { id: victory_level:h_ae_5, scenario: scenario:historical, faction: faction:anglo_egyptian, level: "DECISIVE", rank: 5, min: 100, max: NONE, basis: "Dervish units eliminated", section: section:`9.24` },
  { id: victory_level:h_ae_4, scenario: scenario:historical, faction: faction:anglo_egyptian, level: "STRATEGIC", rank: 4, min: 60, max: 99, basis: "Dervish units eliminated", section: section:`9.24` },
  { id: victory_level:h_ae_3, scenario: scenario:historical, faction: faction:anglo_egyptian, level: "TACTICAL", rank: 3, min: 45, max: 59, basis: "Dervish units eliminated", section: section:`9.24` },
  { id: victory_level:h_ae_2, scenario: scenario:historical, faction: faction:anglo_egyptian, level: "MARGINAL", rank: 2, min: 30, max: 44, basis: "Dervish units eliminated", section: section:`9.24` },
  { id: victory_level:h_ae_1, scenario: scenario:historical, faction: faction:anglo_egyptian, level: "DRAW", rank: 1, min: 0, max: 29, basis: "Dervish units eliminated", section: section:`9.24` },
  { id: victory_level:h_d_5, scenario: scenario:historical, faction: faction:dervish, level: "DECISIVE", rank: 5, min: 30, max: NONE, basis: "Anglo-Egyptian units eliminated", section: section:`9.24` },
  { id: victory_level:h_d_4, scenario: scenario:historical, faction: faction:dervish, level: "STRATEGIC", rank: 4, min: 15, max: 29, basis: "Anglo-Egyptian units eliminated", section: section:`9.24` },
  { id: victory_level:h_d_3, scenario: scenario:historical, faction: faction:dervish, level: "TACTICAL", rank: 3, min: 10, max: 14, basis: "Anglo-Egyptian units eliminated", section: section:`9.24` },
  { id: victory_level:h_d_2, scenario: scenario:historical, faction: faction:dervish, level: "MARGINAL", rank: 2, min: 5, max: 9, basis: "Anglo-Egyptian units eliminated", section: section:`9.24` },
  { id: victory_level:h_d_1, scenario: scenario:historical, faction: faction:dervish, level: "DRAW", rank: 1, min: 0, max: 4, basis: "Anglo-Egyptian units eliminated", section: section:`9.24` },
  -- Fall of Khartoum: turn GORDON is eliminated (9.35); scale ordinal 1 (Dervish decisive) .. 6 (British decisive)
  { id: victory_level:k_d_decisive, scenario: scenario:fall_of_khartoum, faction: faction:dervish, level: "decisive", scale: 1, condition: "eliminate GORDON turn four or sooner", gordon_eliminated_by_turn: 4, section: section:`9.35` },
  { id: victory_level:k_d_tactical, scenario: scenario:fall_of_khartoum, faction: faction:dervish, level: "tactical", scale: 2, condition: "eliminate GORDON turn five", gordon_eliminated_on_turn: 5, section: section:`9.35` },
  { id: victory_level:k_d_marginal, scenario: scenario:fall_of_khartoum, faction: faction:dervish, level: "marginal", scale: 3, condition: "eliminate GORDON turn six", gordon_eliminated_on_turn: 6, section: section:`9.35` },
  { id: victory_level:k_b_marginal, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, level: "marginal", scale: 4, condition: "GORDON survives end of turn six", gordon_survives_turn: 6, section: section:`9.35` },
  { id: victory_level:k_b_tactical, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, level: "tactical", scale: 5, condition: "GORDON survives end of turn seven", gordon_survives_turn: 7, section: section:`9.35` },
  { id: victory_level:k_b_decisive, scenario: scenario:fall_of_khartoum, faction: faction:anglo_egyptian, level: "decisive", scale: 6, condition: "GORDON survives end of turn eight", gordon_survives_turn: 8, section: section:`9.35` }
];
INSERT INTO victory_condition [
  { id: victory_condition:c_ae_annihilation, scenario: scenario:campaign, faction: faction:anglo_egyptian, result: "Decisive", kind: "alternative",
    condition: "eliminates every Dervish unit in play (including gunboats and forts)", section: section:`9.14` },
  { id: victory_condition:c_d_west_bank, scenario: scenario:campaign, faction: faction:dervish, result: "Decisive", kind: "alternative", award_word: "may be awarded",
    condition: "eliminates all Anglo-Egyptian units on the west bank (excluding gunboats)", section: section:`9.14` },
  { id: victory_condition:h_net, scenario: scenario:historical, kind: "net level", condition: "the lower value victory level is subtracted from the higher level to determine a player's net victory", rule: rule:r9_24_net_victory, section: section:`9.24` },
  { id: victory_condition:k_demotion, scenario: scenario:fall_of_khartoum, faction: faction:dervish, kind: "demotion", rule: rule:r9_35_demotion,
    bands: [{ min: 16, max: 23, levels_lost: 1 }, { min: 24, max: 31, levels_lost: 2 }, { min: 32, max: NONE, levels_lost: 3 }], section: section:`9.35` },
  { id: victory_condition:k_length, scenario: scenario:fall_of_khartoum, kind: "length", condition: "Variable: the game ends when GORDON is eliminated or survives; rarely lasts five turns", section: section:`9.33` }
];

-- 9.113 turns 2-4: "any twelve land units" / "all remaining Anglo-Egyptian units" draw on the same pool,
-- so these units may arrive on turn 2, 3 or 4 at the player's choice (turn 4 at the latest).
UPDATE reinforcement:c_ae2, reinforcement:c_ae3, reinforcement:c_ae4 SET
  pool = [unit:camel_corps, unit:lancers_21, unit:battery_32, unit:maxim_battery, unit:royal_engineers, formation:brigade_2b, formation:brigade_3e, formation:brigade_4e, unit:old_gunboat, unit:sultan],
  pool_note = "units the manual names that are not fixed to turn 1; the rest of the counter mix is not in the manual";

-- scenario membership of units / tribes
FOR $s IN (SELECT * FROM setup WHERE not_in_play != true) { FOR $x IN $s.subjects { IF count(SELECT id FROM appears_in WHERE in = $x AND out = $s.scenario) = 0 { RELATE $x->appears_in->($s.scenario) SET via = $s.id; }; }; };
FOR $r IN (SELECT * FROM reinforcement WHERE subjects != NONE OR pool != NONE) { FOR $x IN array::distinct(array::concat($r.subjects ?? [], $r.pool ?? [])) { IF count(SELECT id FROM appears_in WHERE in = $x AND out = $r.scenario) = 0 { RELATE $x->appears_in->($r.scenario) SET via = $r.id; }; }; };

-- whose units an award counts (the subject classes are shared by both sides)
UPDATE vp_award:d_british_leader, vp_award:d_gunboat, vp_award:d_land_unit, vp_award:d_friendlies_east, vp_award:d_friendlies_west SET subject_faction = faction:anglo_egyptian;
UPDATE vp_award:ae_dervish_unit, vp_award:ae_fort, vp_award:ae_isa, vp_award:ae_khalifa SET subject_faction = faction:dervish;

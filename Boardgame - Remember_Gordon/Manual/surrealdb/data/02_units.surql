-- Unit taxonomy, tribes, formations, Dervish commands, individual units, armament.

-- ------------------------------------------------------------------ unit classes
INSERT INTO unit_class [
  -- abstract
  { id: unit_class:unit, name: "unit", abstract: true, description: "any counter that is a unit", section: section:`2.3` },
  { id: unit_class:combat_unit, name: "combat unit", abstract: true, description: "unit with combat values; every unit except Anglo-Egyptian leaders", section: section:`6.51` },
  { id: unit_class:land_unit, name: "land unit", abstract: true, description: "every unit that is not a gunboat", section: section:`5.22` },
  { id: unit_class:infantry, name: "infantry", abstract: true, section: section:`7.4` },
  { id: unit_class:cavalry, name: "cavalry", abstract: true, section: section:`7.4` },
  { id: unit_class:camel, name: "camel unit", abstract: true, section: section:`7.4` },
  { id: unit_class:artillery, name: "artillery", abstract: true, section: section:`6.6` },
  { id: unit_class:gunboat, name: "gunboat", abstract: true, description: "each gunboat present at the battle has its own counter", section: section:`1.2` },
  { id: unit_class:leader, name: "leader", abstract: true, description: "free stacking (5.51)", section: section:`6.51` },
  { id: unit_class:ae_infantry, name: "Anglo-Egyptian infantry battalion", abstract: true, faction: faction:anglo_egyptian,
    description: "British, Sudanese and Egyptian infantry; brigade designation printed in the upper right corner", section: section:`5.54` },
  -- Dervish leaves
  { id: unit_class:d_infantry, name: "Dervish tribal combat unit", faction: faction:dervish,
    description: "Dervish combat unit with Combat / Melee / Movement values plus a tribe identifier", section: section:`2.3`,
    basis: "implied", note: "The sample counter is called a 'combat unit'; 9.111 calls Isa Zachneih an 'infantry unit' and 7.4 lets only infantry, cavalry, camel units and Dervish leaders melee attack, so tribal combat units are read as infantry." },
  { id: unit_class:d_camel, name: "Dervish camel unit", faction: faction:dervish, sample: "Danagla, 4-6-12", section: section:`2.3` },
  { id: unit_class:d_leader, name: "Dervish leader", faction: faction:dervish,
    description: "leader plus retinue; has fire, melee and movement factors; may attack, melee and be eliminated like any other combat unit; stacks free", section: section:`6.51` },
  { id: unit_class:d_artillery, name: "Dervish artillery", faction: faction:dervish, section: section:`2.31` },
  { id: unit_class:d_gunboat, name: "Dervish gunboat", faction: faction:dervish, section: section:`2.31` },
  { id: unit_class:fort, name: "fort", faction: faction:dervish, description: "Dervish fort: artillery factor, defensive-only melee value, −3 defensive value; never moves", section: section:`6.54` },
  -- Anglo-Egyptian leaves
  { id: unit_class:british_infantry, name: "British infantry", faction: faction:anglo_egyptian, section: section:`5.54` },
  { id: unit_class:egyptian_infantry, name: "Egyptian infantry", faction: faction:anglo_egyptian, section: section:`5.54` },
  { id: unit_class:sudanese_infantry, name: "Sudanese infantry", faction: faction:anglo_egyptian, section: section:`5.54` },
  { id: unit_class:friendlies, name: "\"Friendlies\" unit", faction: faction:anglo_egyptian, description: "native volunteers in the Anglo-Egyptian army", section: section:`6.52`,
    basis: "implied", note: "Classed as infantry: 6.52 gives them a melee modifier and 7.4 lets only infantry, cavalry, camel units and Dervish leaders melee attack." },
  { id: unit_class:ae_cavalry, name: "Anglo-Egyptian cavalry", faction: faction:anglo_egyptian, sample: "21 Lancers 8·5·15", section: section:`2.3` },
  { id: unit_class:camel_corps, name: "Camel Corps", faction: faction:anglo_egyptian, section: section:`9.211`,
    basis: "implied", note: "Classed as a camel unit by its name." },
  { id: unit_class:ae_artillery, name: "Anglo-Egyptian artillery", faction: faction:anglo_egyptian, sample: "32 Battery 10·1·7", section: section:`2.3` },
  { id: unit_class:maxim_battery, name: "Maxim guns", faction: faction:anglo_egyptian, sample: "Maxim Batt. 6·1·12", fires_per_turn: 2, section: section:`2.3` },
  { id: unit_class:old_gunboat, name: "old (unnamed) gunboat", faction: faction:anglo_egyptian, sample: "GUNBOAT 4·10/16", section: section:`2.3` },
  { id: unit_class:named_gunboat, name: "new type (named) gunboat", faction: faction:anglo_egyptian, sample: "Sultan 5·6×2·12/18", count: 5, section: section:`2.32`, count_section: section:`6.64` },
  { id: unit_class:ae_leader, name: "Anglo-Egyptian leader", faction: faction:anglo_egyptian, description: "movement factor only", section: section:`6.51` },
  { id: unit_class:royal_engineers, name: "Royal Engineers", faction: faction:anglo_egyptian, sample: "Royal Eng. 5-3-8", section: section:`6.53` }
];

-- taxonomy edges (child -> parent)
LET $tax = [
  [unit_class:combat_unit, unit_class:unit], [unit_class:land_unit, unit_class:unit],
  [unit_class:infantry, unit_class:land_unit], [unit_class:infantry, unit_class:combat_unit],
  [unit_class:cavalry, unit_class:land_unit], [unit_class:cavalry, unit_class:combat_unit],
  [unit_class:camel, unit_class:land_unit], [unit_class:camel, unit_class:combat_unit],
  [unit_class:artillery, unit_class:land_unit], [unit_class:artillery, unit_class:combat_unit],
  [unit_class:gunboat, unit_class:combat_unit],
  [unit_class:leader, unit_class:land_unit],
  [unit_class:ae_infantry, unit_class:infantry],
  [unit_class:d_infantry, unit_class:infantry],
  [unit_class:d_camel, unit_class:camel],
  [unit_class:d_leader, unit_class:leader], [unit_class:d_leader, unit_class:combat_unit],
  [unit_class:d_artillery, unit_class:artillery],
  [unit_class:d_gunboat, unit_class:gunboat],
  [unit_class:fort, unit_class:land_unit], [unit_class:fort, unit_class:combat_unit],
  [unit_class:british_infantry, unit_class:ae_infantry], [unit_class:egyptian_infantry, unit_class:ae_infantry], [unit_class:sudanese_infantry, unit_class:ae_infantry],
  [unit_class:friendlies, unit_class:infantry],
  [unit_class:ae_cavalry, unit_class:cavalry],
  [unit_class:camel_corps, unit_class:camel],
  [unit_class:ae_artillery, unit_class:artillery],
  [unit_class:maxim_battery, unit_class:land_unit], [unit_class:maxim_battery, unit_class:combat_unit],
  [unit_class:old_gunboat, unit_class:gunboat], [unit_class:named_gunboat, unit_class:gunboat],
  [unit_class:ae_leader, unit_class:leader],
  [unit_class:royal_engineers, unit_class:land_unit], [unit_class:royal_engineers, unit_class:combat_unit]
];
FOR $e IN $tax { RELATE ($e[0])->is_a->($e[1]); };

-- ------------------------------------------------------------------ tribes (Dervish)
INSERT INTO tribe [
  { id: tribe:baggara, name: "Baggara", faction: faction:dervish, unit_class: unit_class:d_infantry, section: section:`9.112` },
  { id: tribe:jaalin, name: "Jaalin", faction: faction:dervish, unit_class: unit_class:d_infantry, section: section:`9.112` },
  { id: tribe:danagla, name: "Danagla", faction: faction:dervish, unit_class: unit_class:d_camel, section: section:`2.31`,
    basis_class: "implied", note: "The sample camel counter is a Danagla unit (2.3)." },
  { id: tribe:kehena, name: "Kehena", faction: faction:dervish, unit_class: unit_class:d_infantry, section: section:`9.112` },
  { id: tribe:degheim, name: "Degheim", faction: faction:dervish, unit_class: unit_class:d_infantry, section: section:`9.112` },
  { id: tribe:hadendowa, name: "Hadendowa", aka: ["Fuzzy-Wuzzies"], faction: faction:dervish, unit_class: unit_class:d_infantry, section: section:`9.112`, aka_section: section:`11` },
  { id: tribe:mulazmin, name: "Mulazmin", faction: faction:dervish, unit_class: unit_class:d_infantry, color: "green", section: section:`5.52` },
  { id: tribe:jehadia, name: "Jehadia", faction: faction:dervish, unit_class: unit_class:d_infantry, color: "green", section: section:`5.52`,
    history: "issued the few guns available (old Remingtons and brass muzzle loaders); commanded by the Khalifa's son, Sheik El Din (11)" },
  { id: tribe:taiasha, name: "Taiasha", faction: faction:dervish, unit_class: unit_class:d_infantry, role: "the Khalifa's bodyguard", section: section:`5.23` }
];
FOR $t IN (SELECT VALUE id FROM tribe) { RELATE $t->belongs_to->faction:dervish; };

-- ------------------------------------------------------------------ Dervish command groups (5.53, 9.112, 9.212)
INSERT INTO command_group [
  { id: command_group:sheik_el_din, name: "Sheik El Din's command", color: "green", basis: "explicit",
    note: "5.53: 'Sheik El Din may only stack with Mulazmins or Jehadias'; 9.112: Mulazmin and Jehadia units 'and their leader, SHEIK EL DIN'.", section: section:`5.53` },
  { id: command_group:osman_digna, name: "Osman Digna's command", basis: "explicit", note: "9.112: 'all Hadendowa units and their leader, OSMAN DIGNA'.", section: section:`9.112` },
  { id: command_group:khalifa, name: "the Khalifa's bodyguard", basis: "implied", note: "5.23 calls the Taiasha units 'the Khalifa's bodyguard'.", section: section:`5.23` },
  { id: command_group:turn1_leaders, name: "turn-1 tribes and their leaders", basis: "explicit",
    note: "9.112: 'all Baggara, Jaalin, Danagla, Kehena, and Degheim units, and their leaders: YAKUB, SHERIF, and ALI WAD HELU'. Which of the three leaders commands which tribe is not stated.",
    individual_assignment: "unstated", section: section:`9.112` }
];
RELATE tribe:mulazmin->member_of->command_group:sheik_el_din;
RELATE tribe:jehadia->member_of->command_group:sheik_el_din;
RELATE tribe:hadendowa->member_of->command_group:osman_digna;
RELATE tribe:taiasha->member_of->command_group:khalifa;
FOR $t IN [tribe:baggara, tribe:jaalin, tribe:danagla, tribe:kehena, tribe:degheim] { RELATE $t->member_of->command_group:turn1_leaders; };

-- ------------------------------------------------------------------ formations (Anglo-Egyptian)
INSERT INTO formation [
  { id: formation:egyptian_division, name: "Egyptian Division", kind: "division", faction: faction:anglo_egyptian, section: section:`9.113`,
    history: "Kitchener's Egyptian Division under General Hunter re-occupied Berber in July 1897 (11)" },
  { id: formation:brigade_2b, name: "2nd British Brigade", designation: "2B", kind: "infantry brigade", nationality: "British", battalions: 4, faction: faction:anglo_egyptian, section: section:`5.54` },
  { id: formation:brigade_3e, name: "3rd Egyptian Brigade", designation: "3E", kind: "infantry brigade", nationality: "Egyptian", battalions: 4, faction: faction:anglo_egyptian, section: section:`5.54` },
  { id: formation:brigade_4e, name: "4th Egyptian Brigade", designation: "4E", kind: "infantry brigade", nationality: "Egyptian", battalions: 4, faction: faction:anglo_egyptian, section: section:`2.3`,
    note: "Named on the sample infantry counter ('Brigade I.D. (4th Egyptian)')." },
  { id: formation:friendlies_brigade, name: "\"Friendlies\" brigade", kind: "brigade", nationality: "native volunteers", faction: faction:anglo_egyptian, section: section:`6.52`,
    fok_role: "represents the Shaggyeh (9.321)" }
];
FOR $f IN (SELECT VALUE id FROM formation) { RELATE $f->belongs_to->faction:anglo_egyptian; };

-- ------------------------------------------------------------------ units
INSERT INTO unit [
  -- Anglo-Egyptian leaders
  { id: unit:kitchener, name: "Lord Kitchener", title: "Sirdar", faction: faction:anglo_egyptian, class: unit_class:ae_leader,
    printed: "0·0·15", fire: 0, melee: 0, move: 15, british_leader: true, section: section:`2.3` },
  { id: unit:gatacre, name: "Gatacre", faction: faction:anglo_egyptian, class: unit_class:ae_leader, british_leader: true, section: section:`9.113` },
  { id: unit:hunter, name: "Hunter", faction: faction:anglo_egyptian, class: unit_class:ae_leader, british_leader: true, section: section:`9.113` },
  { id: unit:gordon, name: "General Gordon", faction: faction:anglo_egyptian, class: unit_class:ae_leader, british_leader: true,
    immobile_in: scenario:fall_of_khartoum, section: section:`9.346` },
  -- Dervish leaders
  { id: unit:khalifa_abdullah, name: "Khalifa Abdullah", aka: ["the Khalifa", "Abdullah the Taiasha", "KHALIFA"], faction: faction:dervish, class: unit_class:d_leader,
    may_enter_walled_city: true, section: section:`9.111` },
  { id: unit:yakub, name: "Yakub", faction: faction:dervish, class: unit_class:d_leader, section: section:`9.112` },
  { id: unit:sherif, name: "Sherif", faction: faction:dervish, class: unit_class:d_leader, section: section:`9.112` },
  { id: unit:ali_wad_helu, name: "Ali Wad Helu", faction: faction:dervish, class: unit_class:d_leader, section: section:`9.112` },
  { id: unit:osman_digna, name: "Osman Digna", faction: faction:dervish, class: unit_class:d_leader, section: section:`2.3` },
  { id: unit:sheik_el_din, name: "Sheik El Din", faction: faction:dervish, class: unit_class:d_leader, section: section:`5.53`,
    history: "the Khalifa's son, commanded the Jehadia (11)" },
  -- Dervish combat units
  { id: unit:isa_zachneih, name: "Isa Zachneih", faction: faction:dervish, class: unit_class:d_infantry, role: "the Dervish east bank unit", section: section:`5.21` },
  { id: unit:danagla_sample, name: "Danagla camel unit (sample counter)", generic: true, tribe: tribe:danagla, faction: faction:dervish, class: unit_class:d_camel,
    printed: "4-6-12", fire: 4, melee: 6, move: 12, section: section:`2.3` },
  { id: unit:dervish_artillery, name: "Dervish artillery unit", generic: true, count: 3, faction: faction:dervish, class: unit_class:d_artillery, section: section:`5.23` },
  { id: unit:dervish_gunboat, name: "Dervish gunboat", generic: true, count: 2, faction: faction:dervish, class: unit_class:d_gunboat, section: section:`9.111` },
  { id: unit:fort, name: "Dervish fort", generic: true, count: 17, count_scenario: scenario:campaign, faction: faction:dervish, class: unit_class:fort,
    defensive_value: -3, section: section:`6.54` },
  -- Anglo-Egyptian combat units
  { id: unit:lancers_21, name: "21 Lancers", faction: faction:anglo_egyptian, class: unit_class:ae_cavalry, printed: "8·5·15", fire: 8, melee: 5, move: 15, section: section:`2.3`,
    history: "won their first battle streamer and three Victoria Crosses in one of history's last great knee to knee cavalry charges (11)" },
  { id: unit:egyptian_cavalry, name: "Egyptian Cavalry", faction: faction:anglo_egyptian, class: unit_class:ae_cavalry, section: section:`9.113` },
  { id: unit:camel_corps, name: "Camel Corps", faction: faction:anglo_egyptian, class: unit_class:camel_corps, section: section:`9.211` },
  { id: unit:battery_32, name: "32 Battery", faction: faction:anglo_egyptian, class: unit_class:ae_artillery, printed: "10·1·7", fire: 10, melee: 1, move: 7, section: section:`2.3` },
  { id: unit:horse_artillery, name: "Horse Artillery", faction: faction:anglo_egyptian, class: unit_class:ae_artillery, section: section:`9.113` },
  { id: unit:egyptian_battalion_artillery, name: "Egyptian Battalion artillery unit", faction: faction:anglo_egyptian, class: unit_class:ae_artillery, section: section:`9.321` },
  { id: unit:maxim_battery, name: "Maxim Batt.", faction: faction:anglo_egyptian, class: unit_class:maxim_battery, printed: "6·1·12", fire: 6, melee: 1, move: 12, fires_per_turn: 2, section: section:`2.3` },
  { id: unit:royal_engineers, name: "Royal Engineers", faction: faction:anglo_egyptian, class: unit_class:royal_engineers, printed: "5-3-8", fire: 5, melee: 3, move: 8, section: section:`6.53` },
  { id: unit:egyptian_battalion_sample, name: "Egyptian infantry battalion (sample counter)", generic: true, faction: faction:anglo_egyptian, class: unit_class:egyptian_infantry,
    printed: "9·5·8", fire: 9, melee: 5, move: 8, formation: formation:brigade_4e, section: section:`2.3`,
    note: "6.16's example uses Egyptian battalions of printed strength 9." },
  { id: unit:old_gunboat, name: "GUNBOAT (old type)", generic: true, faction: faction:anglo_egyptian, class: unit_class:old_gunboat,
    printed: "4·10/16", artillery_factor: 4, move_upstream: 10, move_downstream: 16, section: section:`2.3` },
  { id: unit:sultan, name: "Sultan", faction: faction:anglo_egyptian, class: unit_class:named_gunboat,
    printed: "5·6×2·12/18", artillery_factor: 5, howitzer_factor: 5, maxim_factor: 6, maxim_fires_per_turn: 2, move_upstream: 12, move_downstream: 18, section: section:`2.3` }
];
-- leader titles / roles that the manual states
UPDATE unit:kitchener SET role = "Sirdar (CIC) of the Anglo-Egyptian army", role_section = section:`1.1`;
UPDATE unit:khalifa_abdullah SET role = "absolute ruler of the Dervishes", role_section = section:`1.1`;

-- unit memberships
RELATE unit:danagla_sample->belongs_to->tribe:danagla;
RELATE unit:egyptian_battalion_sample->belongs_to->formation:brigade_4e;
RELATE unit:khalifa_abdullah->leads->command_group:khalifa;
RELATE unit:sheik_el_din->leads->command_group:sheik_el_din;
RELATE unit:osman_digna->leads->command_group:osman_digna;
FOR $l IN [unit:yakub, unit:sherif, unit:ali_wad_helu] { RELATE $l->leads->command_group:turn1_leaders; };

-- ------------------------------------------------------------------ armament (2.31, 2.32, 6.52, 9.343)
INSERT RELATION INTO fires_on [
  { in: unit_class:d_artillery, out: range_line:d_artillery, basis: "explicit", section: section:`2.31` },
  { in: unit_class:d_gunboat, out: range_line:d_artillery, basis: "explicit", section: section:`2.31` },
  { in: unit_class:fort, out: range_line:d_artillery, basis: "explicit", section: section:`2.31` },
  { in: tribe:jehadia, out: range_line:d_rifles, basis: "explicit", section: section:`2.31` },
  { in: tribe:danagla, out: range_line:d_rifles, basis: "explicit", section: section:`2.31` },
  { in: unit:isa_zachneih, out: range_line:d_rifles, basis: "explicit", section: section:`2.31` },
  { in: tribe:baggara, out: range_line:d_spears_swords, basis: "explicit", section: section:`2.31` },
  { in: tribe:jaalin, out: range_line:d_spears_swords, basis: "explicit", section: section:`2.31` },
  { in: tribe:kehena, out: range_line:d_spears_swords, basis: "explicit", section: section:`2.31` },
  { in: tribe:degheim, out: range_line:d_spears_swords, basis: "explicit", section: section:`2.31` },
  { in: tribe:hadendowa, out: range_line:d_spears_swords, basis: "explicit", section: section:`2.31` },
  { in: tribe:mulazmin, out: range_line:d_spears_swords, basis: "explicit", section: section:`2.31` },
  { in: tribe:taiasha, out: range_line:d_spears_swords, basis: "explicit", section: section:`2.31` },
  { in: unit_class:d_leader, out: range_line:d_spears_swords, basis: "explicit", section: section:`2.31` },
  { in: unit_class:ae_infantry, out: range_line:ae_rifles, basis: "explicit", section: section:`2.32` },
  { in: unit_class:ae_cavalry, out: range_line:ae_rifles, basis: "explicit", section: section:`2.32` },
  { in: unit_class:camel_corps, out: range_line:ae_rifles, basis: "explicit", section: section:`2.32` },
  { in: unit_class:royal_engineers, out: range_line:ae_rifles, basis: "explicit", section: section:`2.32` },
  { in: unit_class:friendlies, out: range_line:d_rifles, basis: "explicit", section: section:`6.52`, note: "exception to 2.32, which points to 6.52" },
  { in: unit_class:maxim_battery, out: range_line:ae_maxims, basis: "explicit", section: section:`2.32` },
  { in: unit_class:ae_artillery, out: range_line:ae_artillery, basis: "explicit", section: section:`2.32` },
  { in: unit_class:old_gunboat, out: range_line:ae_artillery, basis: "explicit", section: section:`2.32` },
  { in: unit_class:named_gunboat, out: range_line:ae_howitzer, basis: "explicit", section: section:`2.32` },
  { in: unit_class:named_gunboat, out: range_line:ae_artillery, basis: "explicit", section: section:`2.32` },
  { in: unit_class:named_gunboat, out: range_line:ae_maxims, basis: "explicit", section: section:`2.32` }
];

-- Terrain types, named locations, ZOC boundaries, entry permissions.

UPDATE unit:kitchener, unit:gatacre, unit:hunter, unit:gordon SET british_leader_basis = "implied",
  british_leader_note = "9.14 speaks of 'British leader(s)' for the Tomb and for victory points; the Anglo-Egyptian leader counters are the only candidates.";

-- ------------------------------------------------------------------ terrain
INSERT INTO terrain [
  { id: terrain:clear, name: "clear", kind: "hex", section: section:`2.1` },
  { id: terrain:nile_river, name: "Nile River hex", kind: "hex", current_shown_by: "arrows in the Nile",
    only_gunboats_enter: true, zoc: "no ZOC into or out of (except gunboat vs gunboat)", section: section:`5.22` },
  { id: terrain:khor, name: "khor", kind: "feature", description: "dry watercourse",
    effects: ["ZOCs do not extend across a khor (5.44)", "units may never advance after combat across a khor (6.82)"], section: section:`5.44`,
    note: "The manual does not say whether a khor is a hex or a hexside feature; it speaks of extending/advancing 'across' it." },
  { id: terrain:hut, name: "hut hex", kind: "hex", zoc: "ZOCs extend out of, but not into", section: section:`5.44` },
  { id: terrain:building, name: "building hex", kind: "hex", zoc: "ZOCs extend out of, but not into", section: section:`5.44` },
  { id: terrain:walled_city_hex, name: "walled city hex", kind: "hex", section: section:`5.23` },
  { id: terrain:fort_hex, name: "fort (as terrain)", kind: "hex", zoc: "ZOCs extend out of (even if unoccupied), not into", section: section:`5.44` },
  { id: terrain:wall, name: "wall hexside", kind: "hexside",
    effects: ["blocks entry/exit of the walled city (5.23)", "ZOC extends out of the walled city across it, not into (5.44)",
              "no melee across (7.2)", "no advance after combat across (6.82)"], section: section:`5.23` },
  { id: terrain:gate, name: "gate hexside", kind: "hexside",
    effects: ["entry/exit of the walled city (5.23)", "ZOC extends out of, not into (5.44)", "melee through allowed (7.2)", "advance after combat through allowed (6.82)"], section: section:`5.23` },
  { id: terrain:breach, name: "breach hexside", kind: "hexside", created_by: ["artillery fire (6.63)", "Royal Engineers (6.53)"],
    effects: ["entry/exit of the walled city (5.23)", "ZOCs extend both ways (5.44)", "melee through allowed (7.2)",
              "advance after combat through allowed (6.82)", "negates the wall hexside for line of sight (6.63)"], marked_by: marker:breach, section: section:`6.63` },
  { id: terrain:thorn_hedge, name: "Zariba thorn hedge hexside", kind: "hexside", part_of_zariba: true,
    effects: ["−2 to die roll on all Dervish fire attacks", "may not melee across in either direction", "may not advance after combat across in either direction"], section: section:`9.231` },
  { id: terrain:trench, name: "Zariba trench hexside", kind: "hexside", part_of_zariba: true,
    effects: ["−4 to die roll on all Dervish fire attacks vs. entrenched units only", "−2 (instead of +2) melee modifier to Dervish units melee attacking an entrenched unit",
              "entrenched units may be fired over in both directions (they do not block line of sight)"], section: section:`9.232` },
  { id: terrain:zariba_hexside, name: "Zariba hexside (thorn hedge or trench)", kind: "hexside",
    scenario_status: { historical: "built and in place", campaign: "clear terrain unless constructed (5.3)" },
    zoc: "ZOCs extend out of, but not into, the Zariba across a Zariba hexside", section: section:`5.3` },
  { id: terrain:palace, name: "palace hex", kind: "hex", section: section:`9.111` },
  { id: terrain:map_edge_partial_hex, name: "partial hex (half or less showing)", kind: "hex", playable_in: [scenario:fall_of_khartoum], section: section:`9.342` }
];
RELATE terrain:thorn_hedge->is_a->terrain:zariba_hexside;
RELATE terrain:trench->is_a->terrain:zariba_hexside;

-- ------------------------------------------------------------------ locations: Omdurman map
INSERT INTO location [
  { id: location:omdurman, name: "Omdurman", kind: "city", map: map:omdurman, section: section:`2.1`,
    history: "the Khalifa's capital, expanded from a few mud huts in 1885 to a fifteen square mile urban slum by 1898 (11)" },
  { id: location:walled_city, name: "walled city of Omdurman", kind: "area", map: map:omdurman, terrain: terrain:walled_city_hex,
    entry_only_through: [terrain:gate, terrain:breach], section: section:`5.23` },
  { id: location:omdurman_palace, name: "palace hexes (Omdurman)", kind: "hex group", hexes: 2, map: map:omdurman, terrain: terrain:palace, section: section:`9.111` },
  { id: location:mahdis_tomb, name: "Mahdi's Tomb", kind: "hex", map: map:omdurman, vp: 25,
    description: "not only the tallest structure in the entire Sudan in 1898, it was also a Dervish holy shrine", section: section:`9.14`,
    in_walled_city_basis: "explicit", in_walled_city_section: section:`11` },
  { id: location:nile, name: "Nile River", kind: "river", map: map:omdurman, terrain: terrain:nile_river, section: section:`5.22` },
  { id: location:nile_islands, name: "Nile River islands", kind: "area", map: map:omdurman, section: section:`9.111` },
  { id: location:east_bank, name: "east bank", kind: "area", map: map:omdurman, section: section:`5.21` },
  { id: location:west_bank, name: "west bank", kind: "area", map: map:omdurman, section: section:`5.21` },
  { id: location:khor_shambat, name: "Khor Shambat", kind: "khor", map: map:omdurman, terrain: terrain:khor, empties_into: location:nile,
    reference_line_for: ["Dervish forts (9.111)", "Dervish reinforcements (9.112)", "river mines (10.11)", "river chain (10.21)"], section: section:`9.111` },
  { id: location:el_debeba, name: "El Debeba", kind: "village", map: map:omdurman, bank: location:east_bank, section: section:`9.111` },
  { id: location:halfaya, name: "Halfaya hut hexes", kind: "hex group", map: map:omdurman, terrain: terrain:hut, bank: location:east_bank, section: section:`9.111` },
  { id: location:abu_alim, name: "Abu Alim hut hex", kind: "hex", map: map:omdurman, terrain: terrain:hut, bank: location:east_bank, section: section:`9.113` },
  { id: location:kerreri, name: "village of Kerreri (hut hexes)", kind: "hex group", map: map:omdurman, terrain: terrain:hut, section: section:`9.211`,
    history: "the little mud village Kitchener reached on September 1st 1898 (11)" },
  { id: location:ae_entrance_area, name: "ANGLO-EGYPTIAN ENTRANCE AREA", kind: "map edge area", map: map:omdurman, bank: location:west_bank, section: section:`9.113` },
  { id: location:zariba, name: "the Zariba", kind: "hex group", hexes: 13, map: map:omdurman, hexside_terrain: [terrain:thorn_hedge, terrain:trench],
    access: "only via the two end hexsides that connect to the Nile River, +2 MP", section: section:`9.23` },
  { id: location:scattergram_hexes, name: "the seven hexes of the Howitzer Fire Scattergram", kind: "hex group", hexes: 7, playable: true, map: map:omdurman, section: section:`2.1` },
  { id: location:west_edge, name: "west edge", kind: "map edge", map: map:omdurman, section: section:`9.112` },
  { id: location:east_edge, name: "east edge", kind: "map edge", map: map:omdurman, section: section:`3` },
  { id: location:north_edge_nile, name: "north edge Nile River hexes", kind: "map edge", map: map:omdurman, section: section:`9.113` },
  { id: location:south_edge_nile, name: "south edge Nile River hexes", kind: "map edge", map: map:omdurman, section: section:`9.111` },
  { id: location:hex_a, name: "lettered hex A", kind: "set-up hex", letter: "A", map: map:omdurman, scenarios: [scenario:historical], section: section:`9.212` },
  { id: location:hex_d, name: "lettered hex D", kind: "set-up hex", letter: "D", map: map:omdurman, scenarios: [scenario:historical], section: section:`9.212` },
  { id: location:hex_y, name: "lettered hex Y", kind: "set-up hex", letter: "Y", map: map:omdurman, scenarios: [scenario:historical], section: section:`9.212` },
  { id: location:hex_k, name: "lettered hex K", kind: "set-up hex", letter: "K", map: map:omdurman, scenarios: [scenario:historical], section: section:`9.212` },
  { id: location:hex_s, name: "lettered hex S", kind: "set-up hex", letter: "S", map: map:omdurman, scenarios: [scenario:historical], section: section:`9.212` },
  { id: location:hex_o, name: "lettered hex O", kind: "set-up hex", letter: "O", map: map:omdurman, scenarios: [scenario:historical], section: section:`9.212` },
  { id: location:kerreri_hills, name: "Kerreri Hills", kind: "terrain feature", history_only: true, section: section:`11` },
  { id: location:jebel_surgham, name: "Jebel Surgham", kind: "terrain feature", history_only: true, section: section:`11` },
  -- Khartoum map
  { id: location:khartoum, name: "Khartoum", kind: "city", map: map:khartoum, section: section:`2.1`,
    note: "After Khartoum fell, it was destroyed by the Mahdi's troops and lay in ruins in 1898." },
  { id: location:khartoum_palace, name: "the palace (Khartoum)", kind: "hex", map: map:khartoum, terrain: terrain:palace, section: section:`9.321` },
  { id: location:khartoum_wall, name: "Khartoum wall", kind: "hexside line", map: map:khartoum, terrain: terrain:wall,
    gap: "washed away by the receding White Nile after the flood", section: section:`2.1` },
  { id: location:fort_makran, name: "Fort Makran", kind: "fort", map: map:khartoum, controller: faction:anglo_egyptian, section: section:`9.321` },
  { id: location:fort_buri, name: "Fort Buri", kind: "fort", map: map:khartoum, controller: faction:anglo_egyptian, section: section:`9.321` },
  { id: location:north_fort, name: "North Fort", kind: "fort", map: map:khartoum, controller: faction:dervish, fires: true, section: section:`9.344` },
  { id: location:white_nile, name: "White Nile", kind: "river", map: map:khartoum, terrain: terrain:nile_river, section: section:`9.345` },
  { id: location:blue_nile, name: "Blue Nile", kind: "river", map: map:khartoum, terrain: terrain:nile_river, section: section:`9.345` },
  { id: location:khartoum_south_edge, name: "south edge (Khartoum map)", kind: "map edge", map: map:khartoum, section: section:`9.322` },
  { id: location:khartoum_east_edge, name: "east edge (Khartoum map)", kind: "map edge", map: map:khartoum, section: section:`9.322` }
];
UPDATE location:fort_makran, location:fort_buri SET controller_basis = "implied", controller_note = "9.321 lets the British player set up in Forts Makran and/or Buri.";

-- part-of hierarchy
LET $partof = [
  [location:walled_city, location:omdurman], [location:omdurman_palace, location:walled_city], [location:mahdis_tomb, location:walled_city],
  [location:el_debeba, location:east_bank], [location:halfaya, location:east_bank], [location:abu_alim, location:east_bank],
  [location:ae_entrance_area, location:west_bank], [location:khor_shambat, location:west_bank],
  [location:khartoum_palace, location:khartoum], [location:fort_makran, location:khartoum], [location:fort_buri, location:khartoum], [location:north_fort, location:khartoum]
];
FOR $p IN $partof { RELATE ($p[0])->member_of->($p[1]) SET relation = "part_of"; };
UPDATE location:mahdis_tomb SET walled_city_note = "Section 11: Omdurman 'housed the Dervishes' holiest shrine, the Mahdi's Tomb, as well as the palace and other structures in a walled city within a city.'";

-- ------------------------------------------------------------------ ZOC boundaries (5.44)
INSERT INTO zoc_boundary [
  { id: zoc_boundary:nile, boundary: terrain:nile_river, out_of: false, into: false, exception: "Gunboats (5.41): exert a ZOC against enemy gunboats", section: section:`5.44` },
  { id: zoc_boundary:khor, boundary: terrain:khor, out_of: false, into: false, across: false, section: section:`5.44` },
  { id: zoc_boundary:fort, boundary: terrain:fort_hex, out_of: true, into: false, note: "out of a fort even if unoccupied", section: section:`5.44` },
  { id: zoc_boundary:wall, boundary: terrain:wall, out_of: true, into: false,
    note: "out of: from a walled city hex into an adjacent non-walled-city hex across a wall hexside; not into a hex inside the walled city across a wall hexside", section: section:`5.44` },
  { id: zoc_boundary:gate, boundary: terrain:gate, out_of: true, into: false, note: "out of (but not into) a walled city hex across a gate hexside", section: section:`5.44` },
  { id: zoc_boundary:breach, boundary: terrain:breach, out_of: true, into: true, section: section:`5.44` },
  { id: zoc_boundary:hut, boundary: terrain:hut, out_of: true, into: false, section: section:`5.44` },
  { id: zoc_boundary:building, boundary: terrain:building, out_of: true, into: false, section: section:`5.44` },
  { id: zoc_boundary:zariba, boundary: terrain:zariba_hexside, out_of: true, into: false,
    scenarios: [scenario:historical, scenario:campaign], campaign_condition: "only if the Zariba is constructed", section: section:`5.44` }
];

-- ------------------------------------------------------------------ entry permissions (5.22, 5.23, 6.52, 6.54, 9.233)
INSERT RELATION INTO entry [
  { in: unit_class:land_unit, out: terrain:nile_river, allowed: false, exception: "Friendlies transported by gunboat (5.21)", section: section:`5.22` },
  { in: unit_class:gunboat, out: terrain:nile_river, allowed: true, section: section:`5.22` },
  { in: unit:khalifa_abdullah, out: location:walled_city, allowed: true, section: section:`5.23` },
  { in: unit_class:d_artillery, out: location:walled_city, allowed: true, section: section:`5.23` },
  { in: tribe:taiasha, out: location:walled_city, allowed: true, section: section:`5.23` },
  { in: unit_class:d_infantry, out: location:walled_city, allowed: false, except_tribes: [tribe:taiasha], section: section:`5.23`,
    note: "For the Dervish player only the Khalifa, the three artillery units and the Taiasha may enter." },
  { in: unit_class:d_camel, out: location:walled_city, allowed: false, section: section:`5.23` },
  { in: unit_class:d_leader, out: location:walled_city, allowed: false, except_units: [unit:khalifa_abdullah], section: section:`5.23` },
  { in: unit:isa_zachneih, out: location:walled_city, allowed: false, section: section:`5.23` },
  { in: unit_class:d_gunboat, out: location:walled_city, allowed: false, section: section:`5.23` },
  { in: unit_class:ae_infantry, out: location:walled_city, allowed: true, condition: "can get to the walled city; through a gate or breach hexside", section: section:`5.23` },
  { in: unit_class:ae_cavalry, out: location:walled_city, allowed: true, condition: "can get to the walled city; through a gate or breach hexside", section: section:`5.23` },
  { in: unit_class:camel_corps, out: location:walled_city, allowed: true, condition: "can get to the walled city; through a gate or breach hexside", section: section:`5.23` },
  { in: unit_class:ae_artillery, out: location:walled_city, allowed: true, condition: "can get to the walled city; through a gate or breach hexside", section: section:`5.23` },
  { in: unit_class:maxim_battery, out: location:walled_city, allowed: true, condition: "can get to the walled city; through a gate or breach hexside", section: section:`5.23` },
  { in: unit_class:royal_engineers, out: location:walled_city, allowed: true, condition: "can get to the walled city; through a gate or breach hexside", section: section:`5.23` },
  { in: unit_class:ae_leader, out: location:walled_city, allowed: true, condition: "can get to the walled city; through a gate or breach hexside", section: section:`5.23` },
  { in: unit_class:old_gunboat, out: location:walled_city, allowed: false, section: section:`5.23` },
  { in: unit_class:named_gunboat, out: location:walled_city, allowed: false, section: section:`5.23` },
  { in: unit_class:friendlies, out: location:walled_city, allowed: false, section: section:`5.23`, also: section:`6.52` },
  { in: unit_class:land_unit, out: location:zariba, allowed: true, condition: "only via the two end hexsides that connect to the Nile River, paying +2 MP (exception: advance after combat across an entrenched hexside)", section: section:`9.233` }
];

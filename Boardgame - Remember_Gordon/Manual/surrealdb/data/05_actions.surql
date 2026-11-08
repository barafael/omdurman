-- Actions (verbs) and the phases they happen in.

INSERT INTO action [
  { id: action:set_up, name: "set up", category: "scenario", section: section:`9.111` },
  { id: action:enter_map, name: "enter the map (reinforcement)", category: "movement", section: section:`9.112` },
  { id: action:move, name: "move", category: "movement", description: "spend movement points moving hex to hex, up to the printed movement allowance", section: section:`5.11` },
  { id: action:move_upstream, name: "move upstream", category: "movement", description: "gunboat movement against the current", section: section:`5.24` },
  { id: action:move_downstream, name: "move downstream", category: "movement", description: "gunboat movement with the current", section: section:`5.24` },
  { id: action:drift, name: "drift", category: "movement", description: "mine-damaged gunboat drifts two hexes per turn with the current", section: section:`10.12` },
  { id: action:accumulate_mp, name: "accumulate movement points from turn to turn", category: "movement", section: section:`5.13` },
  { id: action:transfer_mp, name: "transfer unused movement points to another unit", category: "movement", section: section:`5.13` },
  { id: action:naval_transport, name: "carry land units on a gunboat (naval transport)", category: "movement", section: section:`5.21` },
  { id: action:load, name: "load onto a gunboat", category: "movement", section: section:`5.21` },
  { id: action:disembark, name: "disembark from a gunboat", category: "movement", section: section:`5.21` },
  { id: action:enter_nile, name: "enter a Nile River hex", category: "movement", section: section:`5.22` },
  { id: action:enter_walled_city, name: "enter the walled city of Omdurman", category: "movement", section: section:`5.23` },
  { id: action:move_through_friendly, name: "move through friendly units", category: "movement", section: section:`5.51` },
  { id: action:withdraw_from_zoc, name: "withdraw from an enemy ZOC", category: "movement", section: section:`5.43` },
  { id: action:move_zoc_to_zoc, name: "move directly from one enemy ZOC into another", category: "movement", section: section:`5.43` },
  { id: action:exert_zoc, name: "exert a zone of control", category: "zoc", section: section:`5.41` },
  { id: action:stack, name: "stack", category: "stacking", section: section:`5.51` },
  { id: action:construct_zariba, name: "construct Zariba hexsides", category: "engineering", section: section:`5.3` },
  { id: action:demolish, name: "perform demolitions (breach a wall hexside or destroy a fort)", category: "engineering", section: section:`6.53` },
  { id: action:fire, name: "fire (fire combat)", category: "fire", section: section:`6.1` },
  { id: action:direct_fire, name: "direct fire", category: "fire", section: section:`6.41` },
  { id: action:defensive_fire, name: "defensive fire", category: "fire", section: section:`6.7` },
  { id: action:offensive_fire, name: "offensive fire", category: "fire", section: section:`6.8` },
  { id: action:combine_fire, name: "combine fire into one attack", category: "fire", section: section:`6.14` },
  { id: action:divide_fire_factor, name: "divide one unit's fire combat factor between hexes", category: "fire", section: section:`6.13` },
  { id: action:split_stack_fire, name: "divide a stack to fire at different hexes", category: "fire", section: section:`6.15` },
  { id: action:allocate_fire, name: "allocate fire attacks", category: "fire", section: section:`6.41` },
  { id: action:resolve_fire, name: "resolve fire attacks", category: "fire", section: section:`6.41` },
  { id: action:maxim_second_fire, name: "Maxim second fire", category: "fire", section: section:`6.42` },
  { id: action:howitzer_fire, name: "howitzer fire", category: "fire", section: section:`6.64` },
  { id: action:fire_at_gunboat, name: "fire at a gunboat", category: "fire", section: section:`6.61` },
  { id: action:fire_at_fort, name: "fire at a fort", category: "fire", section: section:`6.62` },
  { id: action:breach_wall, name: "breach a wall hexside", category: "fire", section: section:`6.63` },
  { id: action:fire_at_chain, name: "fire at the river chain", category: "fire", section: section:`10.23` },
  { id: action:advance_after_combat, name: "advance after combat", category: "combat", section: section:`6.82` },
  { id: action:melee_attack, name: "melee attack", category: "melee", section: section:`7.4` },
  { id: action:melee_defend, name: "melee defend", category: "melee", section: section:`7.4` },
  { id: action:retreat, name: "retreat (two hexes from an infantry melee attack)", category: "melee", section: section:`7.5` },
  { id: action:disrupt, name: "disrupt (invert) units", category: "result", section: section:`CombatResults` },
  { id: action:eliminate, name: "eliminate (remove from play)", category: "result", section: section:`CombatResults` },
  { id: action:recover, name: "turn a disrupted unit face up", category: "result", section: section:`Disrupted` },
  { id: action:sink_gunboat, name: "sink a gunboat", category: "result", section: section:`6.61` },
  { id: action:overrun_leader, name: "eliminate an Anglo-Egyptian leader by occupying or passing through its hex", category: "result", section: section:`6.51` },
  { id: action:desert, name: "desert (Dervish units removed)", category: "event", section: section:`8.2` },
  { id: action:occupy_tomb, name: "occupy / take the Mahdi's Tomb", category: "victory", section: section:`9.14` },
  { id: action:occupy_enemy_fort, name: "occupy an enemy fort", category: "movement", section: section:`6.54` },
  { id: action:record_mines, name: "secretly record mined hexes", category: "optional", section: section:`10.11` },
  { id: action:strike_mine, name: "strike a mine", category: "optional", section: section:`10.12` },
  { id: action:record_chain, name: "secretly record the chain line", category: "optional", section: section:`10.21` },
  { id: action:cross_chain, name: "cross the river chain", category: "optional", section: section:`10.23` },
  { id: action:sink_chain, name: "sink the river chain", category: "optional", section: section:`10.23` },
  { id: action:switch_nile, name: "move between White Nile and Blue Nile", category: "movement", section: section:`9.345` },
  { id: action:advance_turn_marker, name: "advance the Game Turn marker", category: "turn", section: section:`4.C` }
];

-- action -> phase
LET $in = [
  [action:move, phase:ae_movement], [action:move, phase:d_movement],
  [action:enter_map, phase:ae_movement], [action:enter_map, phase:d_movement],
  [action:desert, phase:d_movement],
  [action:direct_fire, phase:ae_turn_dervish_defensive_direct], [action:direct_fire, phase:ae_turn_ae_offensive_direct],
  [action:direct_fire, phase:d_turn_ae_defensive_direct], [action:direct_fire, phase:d_turn_dervish_offensive_direct],
  [action:defensive_fire, phase:ae_turn_dervish_defensive_fire], [action:defensive_fire, phase:d_turn_ae_defensive_fire],
  [action:offensive_fire, phase:ae_turn_ae_offensive_fire], [action:offensive_fire, phase:d_turn_dervish_offensive_fire],
  [action:maxim_second_fire, phase:ae_turn_ae_offensive_maxim_howitzer], [action:maxim_second_fire, phase:d_turn_ae_defensive_maxim_howitzer],
  [action:howitzer_fire, phase:ae_turn_ae_offensive_maxim_howitzer], [action:howitzer_fire, phase:d_turn_ae_defensive_maxim_howitzer],
  [action:advance_after_combat, phase:ae_turn_ae_offensive_fire], [action:advance_after_combat, phase:d_turn_dervish_offensive_fire],
  [action:advance_after_combat, phase:ae_melee], [action:advance_after_combat, phase:d_melee],
  [action:melee_attack, phase:ae_melee], [action:melee_attack, phase:d_melee],
  [action:melee_defend, phase:ae_melee], [action:melee_defend, phase:d_melee],
  [action:retreat, phase:ae_melee], [action:retreat, phase:d_melee],
  [action:construct_zariba, phase:ae_player_turn], [action:demolish, phase:ae_player_turn],
  [action:recover, phase:ae_player_turn], [action:recover, phase:d_player_turn],
  [action:advance_turn_marker, phase:end_of_game_turn]
];
FOR $p IN $in { RELATE ($p[0])->allowed_in->($p[1]); };
UPDATE allowed_in SET note = "spans the whole Anglo-Egyptian player turn: the unit begins and ends the player turn adjacent" WHERE in = action:construct_zariba;
UPDATE allowed_in SET note = "Royal Engineers end movement adjacent; the target is destroyed at the end of the Anglo-Egyptian player turn" WHERE in = action:demolish;
UPDATE allowed_in SET note = "at the end of the owning player's turn" WHERE in = action:recover;
UPDATE allowed_in SET note = "only during the first night turn of the campaign game" WHERE in = action:desert;
UPDATE allowed_in SET note = "cavalry and camel units retreating from an infantry melee attack" WHERE in = action:retreat;
UPDATE allowed_in SET note = "no howitzer fire during night game turns (6.64, 8.1)" WHERE in = action:howitzer_fire;
UPDATE allowed_in SET note = "not after defensive fire (6.7)" WHERE in = action:advance_after_combat AND out IN [phase:ae_turn_ae_offensive_fire, phase:d_turn_dervish_offensive_fire];

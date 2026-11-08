-- Turn sequence (section 4, 6.4), Turn Record Track, scenarios.

-- ------------------------------------------------------------------ phases
-- kind: game_turn | player_turn | phase | fire_step | subphase | end_step
-- player_turn: whose Player Turn the phase belongs to; actor: who acts in it.
INSERT INTO phase [
  { id: phase:game_turn, name: "Game Turn", kind: "game_turn", ord: 0, description: "two Player Turns", section: section:`4` },

  { id: phase:ae_player_turn, name: "Anglo-Egyptian Player Turn", kind: "player_turn", ord: 1, parent: phase:game_turn, player_turn: faction:anglo_egyptian, actor: faction:anglo_egyptian,
    ord_note: "ord 1 = first in the campaign game, where the Anglo-Egyptian player moves first; the first player varies by scenario", section: section:`4.A` },
  { id: phase:ae_movement, name: "Anglo-Egyptian Movement Phase", kind: "phase", ord: 1, parent: phase:ae_player_turn, player_turn: faction:anglo_egyptian, actor: faction:anglo_egyptian, section: section:`4.A` },
  { id: phase:ae_fire_combat, name: "Fire Combat Phase", kind: "phase", ord: 2, parent: phase:ae_player_turn, player_turn: faction:anglo_egyptian, section: section:`4.A` },
  { id: phase:ae_turn_dervish_defensive_fire, name: "Dervish Defensive Fire", kind: "fire_step", ord: 1, parent: phase:ae_fire_combat, player_turn: faction:anglo_egyptian, actor: faction:dervish, fire_mode: "defensive", section: section:`4.A` },
  { id: phase:ae_turn_dervish_defensive_direct, name: "Direct Fire Subphase (Dervish defensive)", kind: "subphase", ord: 1, parent: phase:ae_turn_dervish_defensive_fire, player_turn: faction:anglo_egyptian, actor: faction:dervish, fire_mode: "defensive", executes: section:`6.41`, section: section:`6.4` },
  { id: phase:ae_turn_ae_offensive_fire, name: "Anglo-Egyptian Offensive Fire", kind: "fire_step", ord: 2, parent: phase:ae_fire_combat, player_turn: faction:anglo_egyptian, actor: faction:anglo_egyptian, fire_mode: "offensive", section: section:`4.A` },
  { id: phase:ae_turn_ae_offensive_direct, name: "Direct Fire Subphase (Anglo-Egyptian offensive)", kind: "subphase", ord: 1, parent: phase:ae_turn_ae_offensive_fire, player_turn: faction:anglo_egyptian, actor: faction:anglo_egyptian, fire_mode: "offensive", executes: section:`6.41`, section: section:`4.A` },
  { id: phase:ae_turn_ae_offensive_maxim_howitzer, name: "Maxim Second Fire and Howitzer Fire Subphase (Anglo-Egyptian offensive)", kind: "subphase", ord: 2, parent: phase:ae_turn_ae_offensive_fire, player_turn: faction:anglo_egyptian, actor: faction:anglo_egyptian, fire_mode: "offensive", executes: section:`6.42`, section: section:`4.A` },
  { id: phase:ae_melee, name: "Anglo-Egyptian Melee Attacks", kind: "phase", ord: 3, parent: phase:ae_player_turn, player_turn: faction:anglo_egyptian, actor: faction:anglo_egyptian, section: section:`4.A` },

  { id: phase:d_player_turn, name: "Dervish Player Turn", kind: "player_turn", ord: 2, parent: phase:game_turn, player_turn: faction:dervish, actor: faction:dervish, section: section:`4.B` },
  { id: phase:d_movement, name: "Dervish Movement Phase", kind: "phase", ord: 1, parent: phase:d_player_turn, player_turn: faction:dervish, actor: faction:dervish, section: section:`4.B` },
  { id: phase:d_fire_combat, name: "Fire Combat Phase", kind: "phase", ord: 2, parent: phase:d_player_turn, player_turn: faction:dervish, section: section:`4.B` },
  { id: phase:d_turn_ae_defensive_fire, name: "Anglo-Egyptian Defensive Fire", kind: "fire_step", ord: 1, parent: phase:d_fire_combat, player_turn: faction:dervish, actor: faction:anglo_egyptian, fire_mode: "defensive", section: section:`4.B` },
  { id: phase:d_turn_ae_defensive_direct, name: "Direct Fire Subphase (Anglo-Egyptian defensive)", kind: "subphase", ord: 1, parent: phase:d_turn_ae_defensive_fire, player_turn: faction:dervish, actor: faction:anglo_egyptian, fire_mode: "defensive", executes: section:`6.41`, section: section:`4.B` },
  { id: phase:d_turn_ae_defensive_maxim_howitzer, name: "Maxim Second Fire and Howitzer Fire Subphase (Anglo-Egyptian defensive)", kind: "subphase", ord: 2, parent: phase:d_turn_ae_defensive_fire, player_turn: faction:dervish, actor: faction:anglo_egyptian, fire_mode: "defensive", executes: section:`6.42`, section: section:`4.B` },
  { id: phase:d_turn_dervish_offensive_fire, name: "Dervish Offensive Fire", kind: "fire_step", ord: 2, parent: phase:d_fire_combat, player_turn: faction:dervish, actor: faction:dervish, fire_mode: "offensive", section: section:`4.B` },
  { id: phase:d_turn_dervish_offensive_direct, name: "Direct Fire Subphase (Dervish offensive)", kind: "subphase", ord: 1, parent: phase:d_turn_dervish_offensive_fire, player_turn: faction:dervish, actor: faction:dervish, fire_mode: "offensive", executes: section:`6.41`, section: section:`6.4` },
  { id: phase:d_melee, name: "Dervish Melee Attacks", kind: "phase", ord: 3, parent: phase:d_player_turn, player_turn: faction:dervish, actor: faction:dervish, section: section:`4.B` },

  { id: phase:end_of_game_turn, name: "advance the Game Turn marker", kind: "end_step", ord: 3, parent: phase:game_turn,
    description: "After both players have completed their Player Turns, advance the Game Turn marker to the next hour.", section: section:`4.C` }
];
-- sequence edges inside a player turn
LET $seq = [
  [phase:ae_movement, phase:ae_turn_dervish_defensive_direct], [phase:ae_turn_dervish_defensive_direct, phase:ae_turn_ae_offensive_direct],
  [phase:ae_turn_ae_offensive_direct, phase:ae_turn_ae_offensive_maxim_howitzer], [phase:ae_turn_ae_offensive_maxim_howitzer, phase:ae_melee],
  [phase:d_movement, phase:d_turn_ae_defensive_direct], [phase:d_turn_ae_defensive_direct, phase:d_turn_ae_defensive_maxim_howitzer],
  [phase:d_turn_ae_defensive_maxim_howitzer, phase:d_turn_dervish_offensive_direct], [phase:d_turn_dervish_offensive_direct, phase:d_melee]
];
FOR $s IN $seq { RELATE ($s[0])->next->($s[1]); };

-- ------------------------------------------------------------------ Turn Record Track (campaign game)
INSERT INTO turn [
  { id: turn:1,  n: 1,  date: "1898-09-01", label: "1898 SEPT. 1 6:00 am", time: "06:00", night: false },
  { id: turn:2,  n: 2,  date: "1898-09-01", label: "8:00",     time: "08:00", night: false },
  { id: turn:3,  n: 3,  date: "1898-09-01", label: "10:00",    time: "10:00", night: false },
  { id: turn:4,  n: 4,  date: "1898-09-01", label: "12:00",    time: "12:00", night: false },
  { id: turn:5,  n: 5,  date: "1898-09-01", label: "2:00 pm",  time: "14:00", night: false },
  { id: turn:6,  n: 6,  date: "1898-09-01", label: "4:00",     time: "16:00", night: false },
  { id: turn:7,  n: 7,  date: "1898-09-01", label: "6:00",     time: "18:00", night: false },
  { id: turn:8,  n: 8,  date: "1898-09-01", label: "8:00",     time: "20:00", night: false },
  { id: turn:9,  n: 9,  date: "1898-09-01", label: "Dervish Desertion Roll NIGHT", time: NONE, night: true, special: "Dervish Desertion Roll" },
  { id: turn:10, n: 10, date: "1898-09-02", label: "SEPT. 2 NIGHT", time: NONE, night: true },
  { id: turn:11, n: 11, date: "1898-09-02", label: "6:00 am",  time: "06:00", night: false },
  { id: turn:12, n: 12, date: "1898-09-02", label: "8:00",     time: "08:00", night: false },
  { id: turn:13, n: 13, date: "1898-09-02", label: "10:00",    time: "10:00", night: false },
  { id: turn:14, n: 14, date: "1898-09-02", label: "12:00",    time: "12:00", night: false },
  { id: turn:15, n: 15, date: "1898-09-02", label: "2:00 pm",  time: "14:00", night: false },
  { id: turn:16, n: 16, date: "1898-09-02", label: "4:00",     time: "16:00", night: false },
  { id: turn:17, n: 17, date: "1898-09-02", label: "6:00",     time: "18:00", night: false },
  { id: turn:18, n: 18, date: "1898-09-02", label: "8:00",     time: "20:00", night: false },
  { id: turn:19, n: 19, date: "1898-09-02", label: "NIGHT", time: NONE, night: true },
  { id: turn:20, n: 20, date: "1898-09-03", label: "SEPT. 3 NIGHT", time: NONE, night: true },
  { id: turn:21, n: 21, date: "1898-09-03", label: "6:00 am",  time: "06:00", night: false },
  { id: turn:22, n: 22, date: "1898-09-03", label: "8:00",     time: "08:00", night: false }
];
UPDATE turn SET track = component:turn_record_track, scenario = scenario:campaign, section = section:`TurnTrack`;
UPDATE turn SET date_note = "The track prints SEPT. 2 / SEPT. 3 in the second night box of each night; dates for the first night box are the preceding day by reading order." WHERE n IN [9, 19];
UPDATE turn:9 SET desertion_section = section:`8.2`;
FOR $i IN 1..=21 { RELATE (type::record("turn", $i))->next->(type::record("turn", $i + 1)); };

-- ------------------------------------------------------------------ scenarios
INSERT INTO scenario [
  { id: scenario:campaign, name: "The Campaign Game", section: section:`9.1`, maps: [map:omdurman],
    sets_up_first: faction:dervish, moves_first: faction:anglo_egyptian,
    start: "6:00 am, Sept. 1", end: "8:00 am, Sept. 3", length_turns: 22, first_turn: 1, last_turn: 22, night_turns: [9, 10, 19, 20],
    special_rules: "None.", optional_rules_allowed: true,
    history: "Kitchener arrived at the little mud village of Kerreri on September 1st (11)" },
  { id: scenario:historical, name: "The Historical Scenario", section: section:`9.2`, maps: [map:omdurman],
    sets_up_first: faction:anglo_egyptian, moves_first: faction:dervish,
    start: "6:00 am, September 2", end: "12:00 noon, September 2", length_turns: 4, first_turn: 1, last_turn: 4, night_turns: [],
    special_rules: "The Zariba (9.23)", zariba_built: true, lettered_hexes_used: true,
    character: "an exercise in futility for the Dervish player; demonstrates the superiority of entrenched firepower over traditional tribal arms",
    recommended_before_campaign: true },
  { id: scenario:fall_of_khartoum, name: "Bonus Game: FALL OF KHARTOUM", section: section:`9.3`, maps: [map:khartoum],
    sets_up_first: faction:anglo_egyptian, moves_first: faction:dervish,
    length_turns: NONE, length_text: "Variable, see victory conditions (9.35). Rarely lasts five turns.", first_turn: 1, night_turns: [1],
    night_turns_note: "Only turn 1 is stated to be a night turn (9.341); the other turns' day/night status is not stated.",
    recommended_before_campaign: true, history: "the pre-dawn hours of January 25th, 1885 (11)" }
];
UPDATE scenario:campaign SET zariba_built = false, zariba_constructible = true, lettered_hexes_used = false,
  zariba_note = "Zariba hexsides are clear terrain in the campaign game unless constructed (2.1, 5.3).";
UPDATE scenario:historical SET recommendation_section = section:`1.1`;
UPDATE scenario:fall_of_khartoum SET recommendation_section = section:`1.1`, uses_range_table = range_table:dervish, range_table_section = section:`9.343`;

-- step: position of a leaf phase inside its Player Turn (movement 1, defensive direct 2,
-- defensive Maxim/howitzer 3, offensive direct 4, offensive Maxim/howitzer 5, melee 6).
UPDATE phase:ae_movement, phase:d_movement SET step = 1;
UPDATE phase:ae_turn_dervish_defensive_direct, phase:d_turn_ae_defensive_direct SET step = 2;
UPDATE phase:d_turn_ae_defensive_maxim_howitzer SET step = 3;
UPDATE phase:ae_turn_ae_offensive_direct, phase:d_turn_dervish_offensive_direct SET step = 4;
UPDATE phase:ae_turn_ae_offensive_maxim_howitzer SET step = 5;
UPDATE phase:ae_melee, phase:d_melee SET step = 6;
UPDATE phase SET player_turn_phase = IF player_turn = faction:anglo_egyptian { phase:ae_player_turn } ELSE { phase:d_player_turn } WHERE step != NONE;

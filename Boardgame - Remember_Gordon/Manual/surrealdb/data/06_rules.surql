-- Atomic rules. `actions`, `concerns` and `exception_to` are turned into governs / concerns /
-- exception_to edges by 99_derive.surql. `params` holds the rule's structured values.

INSERT INTO rule [
  -- 1 Introduction
  { id: rule:r1_1_two_players, section: section:`1.1`, kind: "definition", statement: "With two players, one is Herbert Kitchener (Anglo-Egyptian), the other the Khalifa (Dervish).", concerns: [faction:anglo_egyptian, faction:dervish, unit:kitchener, unit:khalifa_abdullah] },
  { id: rule:r1_1_multiplayer, section: section:`1.1`, kind: "permission", statement: "Multi-player: each player commands one or more Dervish tribes or Anglo-Egyptian brigades.", concerns: [tribe:baggara, formation:brigade_2b] , params: { command_units: ["Dervish tribe", "Anglo-Egyptian brigade"] } },
  { id: rule:r1_1_learning_order, section: section:`1.1`, kind: "recommendation", statement: "Play the bonus game FALL OF KHARTOUM and the historical scenario first, before the full campaign game.", concerns: [scenario:fall_of_khartoum, scenario:historical, scenario:campaign],
    params: { order: [scenario:fall_of_khartoum, scenario:historical, scenario:campaign] } },
  { id: rule:r1_1_not_beginner, section: section:`1.1`, kind: "definition", statement: "Not strictly a beginner's game; mechanics familiar to players of modest experience." },
  { id: rule:r1_1_spelling, section: section:`1.1`, kind: "definition", statement: "English spellings of Arabic names are transliterations and vary by source." },
  { id: rule:r1_2_scale, section: section:`1.2`, kind: "definition", statement: "A hex is about 400–440 yards; a day turn is two hours; an infantry or cavalry counter is 400–700 men; each gunboat has its own counter; Anglo-Egyptian leaders are individual counters, Dervish leader counters include their retinues.",
    concerns: [constant:hex_yards, constant:day_turn_hours, constant:men_per_counter, unit_class:gunboat, unit_class:ae_leader, unit_class:d_leader] },

  -- 2 Components
  { id: rule:r2_1_lettered_hexes, section: section:`2.1`, kind: "scope", statement: "The large letters (A, D, Y, ...) are set-up hexes for the historical scenario only and are ignored in the campaign game.", scenarios: [scenario:historical],
    concerns: [location:hex_a, location:hex_d, location:hex_y, location:hex_k, location:hex_s, location:hex_o] },
  { id: rule:r2_1_zariba_clear_in_campaign, section: section:`2.1`, kind: "scope", statement: "The Zariba hexsides exist only in the historical scenario and are clear terrain in the campaign game; the Anglo-Egyptian player may construct them in the campaign game (5.3).",
    concerns: [terrain:zariba_hexside, location:zariba, scenario:campaign, scenario:historical] },
  { id: rule:r2_1_all_hexes_playable, section: section:`2.1`, kind: "definition", statement: "All full hexes of the Omdurman map are playable, including the seven hexes of the Howitzer Fire Scattergram.", concerns: [map:omdurman, location:scattergram_hexes] },
  { id: rule:r2_1_khartoum_map, section: section:`2.1`, kind: "definition", statement: "The Khartoum mini-map shows the city in 1885; its north edge abuts the middle of the Omdurman map's south edge; the missing wall was washed away by the receding White Nile.", concerns: [map:khartoum, location:khartoum_wall] },
  { id: rule:r2_2_los_blocks_direct_fire, section: section:`2.2`, kind: "definition", statement: "The Line of Sight Table tells when terrain blocks line of sight, preventing direct fire attacks.", actions: [action:direct_fire], concerns: [component:los_table] },
  { id: rule:r2_31_dervish_weapons, section: section:`2.31`, kind: "definition", statement: "Dervish artillery, gunboats and forts fire on the artillery line; Jehadia, Danagla and Isa Zachneih on the rifles line; all other Dervish units, including leaders, are armed with spears and swords.",
    actions: [action:fire], concerns: [range_table:dervish, tribe:jehadia, tribe:danagla, unit:isa_zachneih, unit_class:d_leader, unit_class:fort, unit_class:d_gunboat, unit_class:d_artillery] },
  { id: rule:r2_32_ae_weapons, section: section:`2.32`, kind: "definition", statement: "All Anglo-Egyptian units except gunboats, Maxims, artillery and leaders are armed with rifles; Maxims fire on the Maxims line; artillery and old gunboats on the Artillery line; named gunboats may fire on the Howitzer, Artillery and Maxims lines.",
    actions: [action:fire], concerns: [range_table:anglo_egyptian, unit_class:named_gunboat, unit_class:old_gunboat, unit_class:maxim_battery, unit_class:ae_artillery] },

  -- 3 Getting started
  { id: rule:r3_seating, section: section:`3`, kind: "procedure", statement: "The Dervish player sits next to the west edge of the map, the Anglo-Egyptian player opposite on the east edge.", concerns: [faction:dervish, faction:anglo_egyptian, location:west_edge, location:east_edge] },
  { id: rule:r3_preparation, section: section:`3`, kind: "procedure", statement: "Read the rules once, select a scenario and punch out only the counters needed; later punch, sort and store the rest by unit type.", concerns: [component:counter_sheet] },

  -- 4 Turn sequence
  { id: rule:r4_game_turn, section: section:`4`, kind: "sequence", statement: "Each Game Turn has two Player Turns; the player moving first varies by scenario (Anglo-Egyptian first in the campaign game).", concerns: [phase:game_turn, scenario:campaign] },
  { id: rule:r4_player_turn_order, section: section:`4`, kind: "sequence", statement: "A Player Turn is Movement, then Fire Combat (non-moving player's defensive fire, then moving player's offensive fire), then the moving player's Melee.", concerns: [phase:ae_player_turn, phase:d_player_turn] },
  { id: rule:r4_c_advance_marker, section: section:`4.C`, kind: "procedure", statement: "After both Player Turns, advance the Game Turn marker; continue alternating until the end of the scenario.", actions: [action:advance_turn_marker], concerns: [marker:game_turn] },

  -- 5.1 Movement
  { id: rule:r5_11_allowance, section: section:`5.11`, kind: "limit", statement: "A unit may move up to its printed movement allowance, paying terrain costs from the Terrain Effects Chart.", actions: [action:move], concerns: [component:terrain_effects_chart] },
  { id: rule:r5_12_optional_movement, section: section:`5.12`, kind: "permission", statement: "A player may move as many or as few units as desired, limited by movement allowance, terrain costs and enemy ZOCs.", actions: [action:move] },
  { id: rule:r5_13_no_accumulation, section: section:`5.13`, kind: "prohibition", statement: "Movement points may not be accumulated from turn to turn.", actions: [action:accumulate_mp] },
  { id: rule:r5_13_no_transfer, section: section:`5.13`, kind: "prohibition", statement: "Unused movement points may not be transferred to other units.", actions: [action:transfer_mp] },
  { id: rule:r5_13_unused_lost, section: section:`5.13`, kind: "effect", statement: "A unit's unused movement points in a turn are lost.", actions: [action:move] },

  -- 5.2 Movement restrictions
  { id: rule:r5_21_no_naval_transport, section: section:`5.21`, kind: "prohibition", statement: "Naval transport missions are not allowed: gunboats may not carry land units.", actions: [action:naval_transport, action:load], concerns: [unit_class:gunboat] },
  { id: rule:r5_21_friendlies_ferry, section: section:`5.21`, kind: "permission", statement: "The Anglo-Egyptian player may transport surviving Friendlies from the east bank to the west bank, only after Isa Zachneih has been eliminated.",
    actions: [action:naval_transport, action:load, action:disembark], concerns: [unit_class:friendlies, unit:isa_zachneih, location:east_bank, location:west_bank],
    exception_to: [rule:r5_21_no_naval_transport, rule:r5_22_land_units_no_nile, rule:r5_51_gunboats_alone],
    params: { precondition: "Isa Zachneih eliminated", direction: "east bank to west bank", turns: 3,
      steps: [
        { step: "a", turn: 1, text: "on any turn that a Friendlies unit and any Anglo-Egyptian gunboat start their turn adjacent, that unit may load onto (stack with) the gunboat" },
        { step: "b", turn: 2, text: "during the Anglo-Egyptian player's next turn the gunboat may move to any Nile hex adjacent to a west bank hex (up to its movement allowance)" },
        { step: "c", turn: 3, text: "on the Anglo-Egyptian player's third turn the Friendlies unit may disembark and move normally, paying the normal terrain cost for the first hex entered; the gunboat may also move normally" } ] } },
  { id: rule:r5_22_land_units_no_nile, section: section:`5.22`, kind: "prohibition", statement: "Except under 5.21, land units may never enter a Nile River hex; only gunboats may enter and move along Nile River hexes.",
    actions: [action:enter_nile], concerns: [terrain:nile_river, unit_class:land_unit, unit_class:gunboat] },
  { id: rule:r5_23_walled_city_dervish, section: section:`5.23`, kind: "prohibition", statement: "Of the Dervish units only the Khalifa, the three Dervish artillery units and the Taiasha units (the Khalifa's bodyguard) may enter the walled portion of Omdurman.",
    actions: [action:enter_walled_city], concerns: [location:walled_city, unit:khalifa_abdullah, unit_class:d_artillery, tribe:taiasha] },
  { id: rule:r5_23_walled_city_ae, section: section:`5.23`, kind: "permission", statement: "Any Anglo-Egyptian units that can get to the walled city may enter it, except gunboats and Friendlies.",
    actions: [action:enter_walled_city], concerns: [location:walled_city, unit_class:gunboat, unit_class:friendlies] },
  { id: rule:r5_23_gate_or_breach, section: section:`5.23`, kind: "prohibition", statement: "Units enter and/or exit the walled city only through a gate or breach hexside.", actions: [action:enter_walled_city, action:move], concerns: [terrain:gate, terrain:breach, terrain:wall, location:walled_city] },
  { id: rule:r5_24_two_allowances, section: section:`5.24`, kind: "definition", statement: "Gunboats have two movement allowances (e.g. 10/16): the smaller upstream (against the current shown by arrows in the Nile), the larger downstream.",
    actions: [action:move_upstream, action:move_downstream], concerns: [unit_class:gunboat, unit:old_gunboat, unit:sultan] },
  { id: rule:r5_24_upstream_cap, section: section:`5.24`, kind: "limit", statement: "Gunboats may combine movement in both directions, but if they move even one hex upstream, the upstream allowance is their maximum for that turn.",
    actions: [action:move_upstream, action:move_downstream], concerns: [unit_class:gunboat] },
  { id: rule:r5_25_forts_immobile, section: section:`5.25`, kind: "prohibition", statement: "Dervish forts may not move in any way once placed.", actions: [action:move], concerns: [unit_class:fort] },
  { id: rule:r5_26_stop_in_zoc, section: section:`5.26`, kind: "obligation", statement: "Units must stop their movement immediately upon entering an enemy ZOC.", actions: [action:move], concerns: [state:in_enemy_zoc] },

  -- 5.3 Zariba construction
  { id: rule:r5_3_zariba_construction, section: section:`5.3`, kind: "procedure", scenarios: [scenario:campaign],
    statement: "In the campaign game, any Anglo-Egyptian infantry unit that begins and ends the Anglo-Egyptian player turn adjacent to (and on the Nile side of) Zariba hexsides has constructed all Zariba hexsides to which it is adjacent; hexsides may only be built in their printed positions.",
    actions: [action:construct_zariba], concerns: [unit_class:ae_infantry, terrain:zariba_hexside, location:zariba, marker:blank, state:constructing_zariba],
    params: { who: "any Anglo-Egyptian infantry unit", duration: "one Anglo-Egyptian player turn (begin and end adjacent)", side: "Nile side", builds: "all Zariba hexsides to which it is adjacent", positions: "only as displayed on the mapsheet", marker: "blank counter" } },
  { id: rule:r5_3_constructor_restrictions, section: section:`5.3`, kind: "prohibition", scenarios: [scenario:campaign],
    statement: "The constructing unit may neither fire offensively nor melee attack during the turn of construction.", actions: [action:offensive_fire, action:melee_attack], concerns: [state:constructing_zariba] },

  -- 5.4 ZOC
  { id: rule:r5_41_who_exerts, section: section:`5.41`, kind: "definition", statement: "All units except Anglo-Egyptian leaders exert a ZOC into their six adjacent hexes.", actions: [action:exert_zoc], concerns: [unit_class:ae_leader], params: { hexes: 6 } },
  { id: rule:r5_41_gunboat_zoc, section: section:`5.41`, kind: "limit", statement: "Gunboats exert a ZOC only against enemy gunboats.", actions: [action:exert_zoc], concerns: [unit_class:gunboat], exception_to: [rule:r5_41_who_exerts] },
  { id: rule:r5_41_disrupted_no_zoc, section: section:`5.41`, kind: "effect", statement: "Disrupted units have no ZOC.", actions: [action:exert_zoc], concerns: [state:disrupted], exception_to: [rule:r5_41_who_exerts] },
  { id: rule:r5_42_no_mp_cost, section: section:`5.42`, kind: "definition", statement: "There is no movement point cost to enter or leave an enemy ZOC.", actions: [action:move, action:withdraw_from_zoc], params: { mp_cost: 0 } },
  { id: rule:r5_43_stop, section: section:`5.43`, kind: "obligation", statement: "All units must stop when they enter an enemy ZOC and may move no further that turn.", actions: [action:move], concerns: [state:in_enemy_zoc] },
  { id: rule:r5_43_next_turn, section: section:`5.43`, kind: "permission", statement: "In their next movement phase they may withdraw or move directly into another enemy ZOC.", actions: [action:withdraw_from_zoc, action:move_zoc_to_zoc] },
  { id: rule:r5_44_boundaries, section: section:`5.44`, kind: "limit", statement: "ZOC propagation across terrain boundaries follows the zoc_boundary records (Nile, khor, fort, wall, gate, breach, hut/building, Zariba).",
    actions: [action:exert_zoc], concerns: [terrain:nile_river, terrain:khor, terrain:fort_hex, terrain:wall, terrain:gate, terrain:breach, terrain:hut, terrain:building, terrain:zariba_hexside] },

  -- 5.5 Stacking
  { id: rule:r5_51_limit, section: section:`5.51`, kind: "limit", statement: "No more than four units may occupy a hex, with the exception of leaders and gunboats.", actions: [action:stack], concerns: [constant:stacking_limit], params: { max_units: 4 } },
  { id: rule:r5_51_leaders_free, section: section:`5.51`, kind: "permission", statement: "All leader units are free stacking: they may stack in addition to the four-unit limit.", actions: [action:stack], concerns: [unit_class:leader], exception_to: [rule:r5_51_limit] },
  { id: rule:r5_51_gunboats_alone, section: section:`5.51`, kind: "prohibition", statement: "Gunboats may not stack with any other unit (exception: 5.21).", actions: [action:stack], concerns: [unit_class:gunboat], exception_to: [rule:r5_51_limit] },
  { id: rule:r5_51_move_through, section: section:`5.51`, kind: "permission", statement: "Players may move through friendly units at no additional cost in movement points.", actions: [action:move_through_friendly], params: { mp_cost: 0 } },
  { id: rule:r5_51_when, section: section:`5.51`, kind: "scope", statement: "The stacking limitation applies only at the end of the movement phase and during combat.", actions: [action:stack], params: { applies: ["end of the movement phase", "during combat"] } },
  { id: rule:r5_52_tribes_apart, section: section:`5.52`, kind: "prohibition", statement: "Units of different Dervish tribes may not stack together, even if they are the same color (e.g. Mulazmin and Jehadia).", actions: [action:stack], concerns: [tribe:mulazmin, tribe:jehadia] },
  { id: rule:r5_53_leaders_optional, section: section:`5.53`, kind: "permission", statement: "Leader units are not required to stack.", actions: [action:stack], concerns: [unit_class:leader] },
  { id: rule:r5_53_dervish_leader_command, section: section:`5.53`, kind: "limit", statement: "Dervish leaders that stack may only stack with units of their command (color); e.g. Sheik El Din only with Mulazmins or Jehadias.",
    actions: [action:stack], concerns: [unit_class:d_leader, unit:sheik_el_din, command_group:sheik_el_din] },
  { id: rule:r5_54_brigade_integrity, section: section:`5.54`, kind: "modifier", statement: "In a combat phase in which all four infantry battalions of an Anglo-Egyptian infantry brigade are stacked in the same hex, the stack has brigade integrity and gets +1 to its fire combat die roll if all fire at the same enemy-occupied hex; this is in addition to the normal +1 of 6.24.",
    actions: [action:fire, action:direct_fire], concerns: [state:brigade_integrity, unit_class:ae_infantry, modifier:brigade_integrity] },

  -- 6.1 Fire general
  { id: rule:r6_11_fire_factor, section: section:`6.11`, kind: "definition", statement: "The fire combat factor is printed on the unit and expresses its fire strength.", actions: [action:fire] },
  { id: rule:r6_12_voluntary, section: section:`6.12`, kind: "permission", statement: "Fire combat is always voluntary; a unit is never required to fire because enemy units are in range or adjacent.", actions: [action:fire] },
  { id: rule:r6_13_unitary, section: section:`6.13`, kind: "prohibition", statement: "A unit's fire combat factor is unitary and may not be divided to fire at enemy units on different hexes.", actions: [action:divide_fire_factor] },
  { id: rule:r6_14_combine, section: section:`6.14`, kind: "permission", statement: "Players may fire at an enemy-occupied hex with as many friendly units as may legally do so, combining their fire combat factors into one attack.", actions: [action:combine_fire] },
  { id: rule:r6_14_once, section: section:`6.14`, kind: "limit", statement: "In any fire combat phase a combat unit may only fire once and may only be fired at once (exceptions: Maxim guns and gunboats, 6.4).", actions: [action:fire],
    params: { fires_per_phase: 1, fired_at_per_phase: 1, exceptions: [unit_class:maxim_battery, unit_class:gunboat] } },
  { id: rule:r6_15_split_stack, section: section:`6.15`, kind: "permission", statement: "A stack may be divided to fire at different enemy-occupied hexes; brigade-integrity infantry then lose their +1 unless all fire at the same hex.", actions: [action:split_stack_fire], concerns: [state:brigade_integrity] },
  { id: rule:r6_16_halving, section: section:`6.16`, kind: "procedure", statement: "When halving fire strength, round down each individual unit; a unit's firing strength is never reduced below one by halving. Example: four Egyptian battalions of 9 fire 16 when halved.",
    actions: [action:fire], params: { rounding: "down, per unit", minimum: 1, example: { units: 4, printed: 9, halved_total: 16 } } },

  -- 6.2 How to fire
  { id: rule:r6_21_los_first, section: section:`6.21`, kind: "procedure", statement: "First check the Line of Sight Table to be sure the firing unit can see the target hex (exception: howitzer fire).", actions: [action:direct_fire], concerns: [component:los_table], params: { step: 1 } },
  { id: rule:r6_22_range, section: section:`6.22`, kind: "procedure", statement: "Next consult the Range Effects Table: the fire factor is tripled, doubled, normal, halved, or the target is out of range; add up all factors firing at the hex.", actions: [action:fire], concerns: [component:range_effects_table_ae, component:range_effects_table_dervish, constant:range_multipliers], params: { step: 2 } },
  { id: rule:r6_23_terrain_and_crt, section: section:`6.23`, kind: "procedure", statement: "Next check the Terrain Effects Chart for a defensive benefit in the target hex, apply it as a negative modifier to the d10 roll, and cross-index the net roll with the number of factors firing on the Combat Results Table.",
    actions: [action:fire], concerns: [component:terrain_effects_chart, component:combat_results_table, component:d10], params: { step: 3 } },
  { id: rule:r6_24_ae_accuracy, section: section:`6.24`, kind: "modifier", statement: "All Anglo-Egyptian direct fire attacks get +1 to the die roll (accuracy bonus); a brigade-integrity stack firing all four at one hex gets another +1; the modifiers are cumulative.",
    actions: [action:direct_fire], concerns: [modifier:ae_direct_fire, modifier:brigade_integrity, faction:anglo_egyptian], params: { step: 4 } },

  -- 6.3, 6.4
  { id: rule:r6_3_los, section: section:`6.3`, kind: "prohibition", statement: "Cross-index the firer's terrain with the target's on the Line of Sight Table; terrain in the box blocks LOS (with footnoted exceptions and the Special LOS Notes). Except for howitzer fire, you can't fire at anything you can't see.",
    actions: [action:direct_fire], concerns: [component:los_table, component:special_los_notes] },
  { id: rule:r6_4_sequence, section: section:`6.4`, kind: "sequence", statement: "The resolution sequence is the same for defensive and offensive fire. Dervish player turn: Anglo-Egyptian 6.41 and 6.42 defensively, then Dervish 6.41 offensively. Anglo-Egyptian player turn: Dervish 6.41 defensively, then Anglo-Egyptian 6.41 and 6.42 offensively.",
    actions: [action:defensive_fire, action:offensive_fire] },
  { id: rule:r6_41_allocate_then_resolve, section: section:`6.41`, kind: "procedure", statement: "Direct Fire Subphase: the firing player first allocates all fire attacks, combining factors as wished, then resolves them in any order.", actions: [action:allocate_fire, action:resolve_fire, action:direct_fire] },
  { id: rule:r6_42_second_subphase, section: section:`6.42`, kind: "permission", statement: "Maxim Second Fire and Howitzer Fire Subphase (Anglo-Egyptian only): named gunboats may fire their artillery factor as howitzer fire and all Maxim guns may fire a second time; allocate all, then resolve in any order.",
    actions: [action:maxim_second_fire, action:howitzer_fire], concerns: [unit_class:named_gunboat, unit_class:maxim_battery, faction:anglo_egyptian] },
  { id: rule:r6_42_combine_howitzer, section: section:`6.42`, kind: "condition", statement: "Howitzer fire may be combined with Maxim fire, but only if the howitzer fire impacts in the intended hex.", actions: [action:howitzer_fire, action:combine_fire] },
  { id: rule:r6_42_maxim_once_if_skipped, section: section:`6.42`, kind: "limit", statement: "Maxim guns that did not fire in the Direct Fire Subphase may still only fire once in the Maxim and Howitzer Subphase.", actions: [action:maxim_second_fire] },
  { id: rule:r6_42_refire_targets, section: section:`6.42`, kind: "permission", statement: "Units firing in this subphase may fire at enemy units fired at in the Direct Fire Subphase.", actions: [action:maxim_second_fire, action:howitzer_fire], exception_to: [rule:r6_14_once] },

  -- 6.5 special units
  { id: rule:r6_51_dervish_leaders, section: section:`6.51`, kind: "definition", statement: "Dervish leaders have fire, melee and movement factors; they may attack, melee and be eliminated like any other combat unit; their special benefit is free stacking.",
    actions: [action:fire, action:melee_attack, action:stack], concerns: [unit_class:d_leader] },
  { id: rule:r6_51_ae_leaders, section: section:`6.51`, kind: "definition", statement: "Anglo-Egyptian leaders have a movement factor only.", concerns: [unit_class:ae_leader] },
  { id: rule:r6_51_ae_leader_elimination, section: section:`6.51`, kind: "effect", statement: "An Anglo-Egyptian leader is eliminated (a) if alone in a hex when a Dervish unit occupies or passes through that hex, or (b) if all combat units it is stacked with are eliminated in fire combat or melee.",
    actions: [action:overrun_leader, action:eliminate], concerns: [unit_class:ae_leader], params: { conditions: ["alone in a hex a Dervish unit occupies or passes through", "all combat units it is stacked with eliminated in fire combat or melee"] } },
  { id: rule:r6_51_ae_leader_tomb, section: section:`6.51`, kind: "condition", statement: "At least one Anglo-Egyptian leader must survive to occupy the Mahdi's Tomb hex if it is to be taken from the Dervish player.", actions: [action:occupy_tomb], concerns: [unit_class:ae_leader, location:mahdis_tomb] },
  { id: rule:r6_52_friendlies, section: section:`6.52`, kind: "definition", statement: "The Friendlies (native volunteers) fire rifles on the Dervish Range Effects Table and melee with the Dervish melee modifier; they may not enter the walled city; they may be transferred to the west bank.",
    actions: [action:fire, action:melee_attack, action:enter_walled_city, action:naval_transport], concerns: [unit_class:friendlies, range_line:d_rifles, modifier:dervish_melee], exception_to: [rule:r2_32_ae_weapons] },
  { id: rule:r6_53_engineers, section: section:`6.53`, kind: "procedure", statement: "The Royal Engineers may breach a wall hexside or destroy a fort: they end their movement adjacent, may neither fire offensively nor melee attack in the ensuing combat phase, and if still adjacent and undisrupted at the end of the Anglo-Egyptian player turn the target is destroyed (remove the fort / place a breach marker). They may demolish while stacked with other units.",
    actions: [action:demolish, action:breach_wall, action:offensive_fire, action:melee_attack], concerns: [unit:royal_engineers, unit_class:royal_engineers, unit_class:fort, terrain:wall, marker:breach, state:demolishing],
    params: { conditions_at_end_of_ae_turn: ["adjacent to the target", "undisrupted"], forbidden_in_combat_phase: [action:offensive_fire, action:melee_attack], stacked_allowed: true } },
  { id: rule:r6_54_fort_fires_alone, section: section:`6.54`, kind: "permission", statement: "A fort's artillery factor may be fired normally by the owning player even if no friendly unit is stacked with it.", actions: [action:fire], concerns: [unit_class:fort] },
  { id: rule:r6_54_fort_melee_defensive, section: section:`6.54`, kind: "prohibition", statement: "A fort's melee value is defensive only: forts may not melee attack.", actions: [action:melee_attack, action:melee_defend], concerns: [unit_class:fort] },
  { id: rule:r6_54_fort_defense, section: section:`6.54`, kind: "modifier", statement: "The fort's −3 defensive value is deducted from the die roll of enemy fire attacks on friendly units stacked inside the fort.", actions: [action:fire], concerns: [unit_class:fort, modifier:fort_defense] },
  { id: rule:r6_54_no_enemy_fort, section: section:`6.54`, kind: "prohibition", statement: "Players may not occupy an enemy fort nor advance after combat into an unoccupied enemy fort.", actions: [action:occupy_enemy_fort, action:advance_after_combat], concerns: [unit_class:fort] },
  { id: rule:r6_54_friendly_fort_free, section: section:`6.54`, kind: "definition", statement: "There is no additional movement point cost to enter or leave a friendly fort.", actions: [action:move], concerns: [unit_class:fort], params: { mp_cost: 0 } },
  { id: rule:r6_54_fort_destruction, section: section:`6.54`, kind: "definition", statement: "Forts may be destroyed by artillery fire (6.62), infantry melee attack (7.6) or the Royal Engineers (6.53).",
    actions: [action:fire_at_fort, action:melee_attack, action:demolish], concerns: [unit_class:fort, unit_class:artillery, unit_class:infantry, unit:royal_engineers],
    params: { means: ["artillery fire (6.62)", "infantry melee attack (7.6)", "the Royal Engineers (6.53)"] } },
  { id: rule:r6_54_fort_zoc, section: section:`6.54`, kind: "definition", statement: "Forts have a ZOC even if unoccupied.", actions: [action:exert_zoc], concerns: [unit_class:fort] },

  -- 6.6 special artillery
  { id: rule:r6_61_gunboat_target, section: section:`6.61`, kind: "threshold", statement: "Only artillery may fire at gunboats; a result of 3 or more sinks a gunboat; any other result is a miss.", actions: [action:fire_at_gunboat, action:sink_gunboat], concerns: [unit_class:gunboat, unit_class:artillery, threshold:sink_gunboat] },
  { id: rule:r6_62_fort_target, section: section:`6.62`, kind: "threshold", statement: "Only artillery may fire at forts; a result of 2 or more eliminates the fort; any other result is a miss; if the fort contains enemy units when destroyed, one unit is eliminated with it.",
    actions: [action:fire_at_fort, action:eliminate], concerns: [unit_class:fort, unit_class:artillery, threshold:destroy_fort] },
  { id: rule:r6_63_breach, section: section:`6.63`, kind: "threshold", statement: "Only artillery may fire to breach a wall hexside of Khartoum or the walled city of Omdurman; 2 or more breaches; any other result is a miss. The breach negates the wall hexside for LOS; place a BREACH marker in an adjacent hex pointing to the hexside; if enemy units are adjacent to the hexside when breached, one enemy unit is eliminated.",
    actions: [action:breach_wall, action:eliminate], concerns: [terrain:wall, terrain:breach, marker:breach, location:khartoum_wall, location:walled_city, threshold:breach_wall] },
  { id: rule:r6_64_howitzer, section: section:`6.64`, kind: "procedure", statement: "The five named British gunboats may fire their artillery factor as direct fire in the Direct Fire Subphase and the same factor again as howitzer fire in the Maxim Second Fire and Howitzer Subphase. To fire, select a target hex 4–10 hexes away (ignoring LOS) and roll the d10 twice: first the CRT roll, then the impact roll on the Howitzer Fire Scattergram; the target hex is hit on 7–10. Once rolled, the results take effect even if the fire scatters into a friendly-occupied hex.",
    actions: [action:howitzer_fire, action:direct_fire], concerns: [unit_class:named_gunboat, component:howitzer_scattergram, constant:howitzer_min_range, constant:howitzer_max_range, die_outcome:howitzer_on_target],
    exception_to: [rule:r6_21_los_first, rule:r6_3_los, rule:r6_14_once],
    params: { min_range: 4, max_range: 10, rolls: ["Combat Results Table", "impact hex"], on_target: [7, 8, 9, 10], friendly_fire: true } },
  { id: rule:r6_64_no_night_howitzer, section: section:`6.64`, kind: "prohibition", statement: "No howitzer fire is allowed during night game turns.", actions: [action:howitzer_fire], concerns: [state:night] },

  -- 6.7, 6.8
  { id: rule:r6_7_defensive_fire, section: section:`6.7`, kind: "permission", statement: "In Defensive Fire all of the non-moving player's units may fire at any of the moving player's units in range, within the rules of combat (6.1–6.6).", actions: [action:defensive_fire] },
  { id: rule:r6_7_no_advance, section: section:`6.7`, kind: "prohibition", statement: "There is no advance after combat as a result of defensive fire.", actions: [action:advance_after_combat, action:defensive_fire] },
  { id: rule:r6_81_offensive_fire, section: section:`6.81`, kind: "permission", statement: "During Offensive Fire the moving player may fire with all units capable of firing, up to their maximum range.", actions: [action:offensive_fire] },
  { id: rule:r6_82_advance, section: section:`6.82`, kind: "permission", statement: "If an enemy-occupied hex is vacated by offensive fire, friendly units that participated in the attack and were adjacent to the hex may advance into it.",
    actions: [action:advance_after_combat, action:offensive_fire], params: { eligibility: ["participated in the attack", "adjacent to the vacated hex"] } },
  { id: rule:r6_82_advance_limits, section: section:`6.82`, kind: "prohibition", statement: "Artillery may not advance; units may not advance across a wall hexside except at a gate or breach; units may never advance after combat across a khor.",
    actions: [action:advance_after_combat], concerns: [unit_class:artillery, terrain:wall, terrain:gate, terrain:breach, terrain:khor] },

  -- 7 Melee
  { id: rule:r7_1_printed, section: section:`7.1`, kind: "definition", statement: "Melee strength is printed on the counter; gunboats have no melee strength.", actions: [action:melee_attack, action:melee_defend] },
  { id: rule:r7_1_gunboats_no_melee, section: section:`7.1`, kind: "prohibition", statement: "Gunboats may neither melee attack nor be melee attacked.", actions: [action:melee_attack, action:melee_defend], concerns: [unit_class:gunboat] },
  { id: rule:r7_2_adjacent, section: section:`7.2`, kind: "limit", statement: "Melee simulates hand-to-hand fighting; units may melee attack adjacent enemy units only.", actions: [action:melee_attack] },
  { id: rule:r7_2_walls, section: section:`7.2`, kind: "prohibition", statement: "Units may not melee attack across a wall hexside, but may melee attack through a gate or breach hexside.", actions: [action:melee_attack], concerns: [terrain:wall, terrain:gate, terrain:breach] },
  { id: rule:r7_3_simultaneous, section: section:`7.3`, kind: "definition", statement: "Melee combat is simultaneous: units eliminated by melee attacks still get a melee combat die roll.", actions: [action:melee_attack, action:melee_defend] },
  { id: rule:r7_4_attackers, section: section:`7.4`, kind: "limit", statement: "Only infantry, cavalry, camel units and Dervish leaders may melee attack.", actions: [action:melee_attack], concerns: [unit_class:infantry, unit_class:cavalry, unit_class:camel, unit_class:d_leader] },
  { id: rule:r7_4_defenders, section: section:`7.4`, kind: "permission", statement: "All units except gunboats may melee defend.", actions: [action:melee_defend], concerns: [unit_class:gunboat] },
  { id: rule:r7_5_retreat, section: section:`7.5`, kind: "permission", statement: "Cavalry and camel units may retreat two hexes from an infantry melee attack; only one retreat per unit per turn; enemy units whose melee attacks are not yet resolved may attack a unit that retreats adjacent to them.",
    actions: [action:retreat], concerns: [unit_class:cavalry, unit_class:camel, unit_class:infantry], params: { hexes: 2, per_unit_per_turn: 1, trigger: "infantry melee attack" } },
  { id: rule:r7_6_dervish_must_advance, section: section:`7.6`, kind: "obligation", statement: "If a melee attack eliminates all defenders in an adjacent hex, the Dervish player MUST advance into the vacated hex with all surviving eligible units (adjacent and participating), up to the stacking limit.",
    actions: [action:advance_after_combat], concerns: [faction:dervish], params: { eligibility: ["adjacent to the vacated hex", "participated in the melee attack that eliminated the defenders"], all_must_advance: true, cap: "stacking limit" } },
  { id: rule:r7_6_ae_may_advance, section: section:`7.6`, kind: "permission", statement: "The Anglo-Egyptian player may advance if desired after a melee attack eliminates all defenders.", actions: [action:advance_after_combat], concerns: [faction:anglo_egyptian] },
  { id: rule:r7_6_attackers_only, section: section:`7.6`, kind: "limit", statement: "Only attacking units may advance after melee.", actions: [action:advance_after_combat] },
  { id: rule:r7_7_resolution, section: section:`7.7`, kind: "procedure", statement: "To resolve melee, attacker and defender both roll on the Combat Results Table and apply their melee modifier: Dervish +2, Anglo-Egyptian +1.",
    actions: [action:melee_attack, action:melee_defend], concerns: [modifier:dervish_melee, modifier:ae_melee, component:combat_results_table] },
  { id: rule:r7_7_no_terrain, section: section:`7.7`, kind: "limit", statement: "No terrain modifiers apply to melee combat (exception: Zariba hexsides in the historical scenario, and in the campaign game if constructed).", actions: [action:melee_attack], concerns: [terrain:zariba_hexside] },
  { id: rule:r7_7_losses_first, section: section:`7.7`, kind: "obligation", statement: "Melee losses must be taken from meleeing units first.", actions: [action:melee_attack, action:melee_defend, action:eliminate] },

  -- 8 Night
  { id: rule:r8_1_ae_movement_halved, section: section:`8.1`, kind: "modifier", statement: "Night: all Anglo-Egyptian movement allowances are halved (round down).", actions: [action:move], concerns: [state:night, faction:anglo_egyptian], params: { factor: 0.5, rounding: "down" } },
  { id: rule:r8_1_no_howitzer, section: section:`8.1`, kind: "prohibition", statement: "Night: there is no Anglo-Egyptian howitzer fire.", actions: [action:howitzer_fire], concerns: [state:night] },
  { id: rule:r8_1_ranges_halved, section: section:`8.1`, kind: "modifier", statement: "Night: all fire ranges are halved for both sides (round down, but range 1 stays range 1); range effects are the same as by day. E.g. Anglo-Egyptian infantry at night: doubled at range 1, normal at range 2, may not fire at range 3 or greater.",
    actions: [action:fire], concerns: [state:night], params: { factor: 0.5, rounding: "down", min_range: 1,
      example: { firer: "Anglo-Egyptian infantry", night: [{ range: 1, effect: "doubled" }, { range: 2, effect: "normal" }, { range: 3, effect: "may not fire" }] } } },
  { id: rule:r8_2_desertion, section: section:`8.2`, kind: "procedure", scenarios: [scenario:campaign], statement: "Once each campaign game, during the movement phase of the first night turn, the Dervish player rolls one die; the number of deserting Dervish units equals 1½ times the roll. The Dervish player chooses which units desert (remove them), except the KHALIFA unit, gunboats, artillery units and forts. No victory points are awarded for deserting Dervishes.",
    actions: [action:desert], concerns: [faction:dervish, unit:khalifa_abdullah, unit_class:d_gunboat, unit_class:d_artillery, unit_class:fort, turn:9],
    params: { when: "first night turn, movement phase", multiplier: 1.5, chooser: faction:dervish, exempt: [unit:khalifa_abdullah, unit_class:d_gunboat, unit_class:d_artillery, unit_class:fort], vp: 0 } },

  -- 9.1 campaign
  { id: rule:r9_112_entry_cost, section: section:`9.112`, kind: "definition", scenarios: [scenario:campaign], statement: "Each entering Dervish reinforcement pays the terrain cost of the hex through which it enters, no matter how many units enter through that hex.", actions: [action:enter_map] },
  { id: rule:r9_113_leaders_timing, section: section:`9.113`, kind: "limit", scenarios: [scenario:campaign], statement: "KITCHENER, GATACRE and HUNTER may enter any time during the first four game turns and do not count against the 12-units-per-turn limit; all three must be in play by the end of turn four.",
    actions: [action:enter_map], concerns: [unit:kitchener, unit:gatacre, unit:hunter], params: { window_turns: [1, 2, 3, 4], deadline_turn: 4, counts_against_limit: false } },
  { id: rule:r9_113_no_gordon, section: section:`9.113`, kind: "scope", scenarios: [scenario:campaign], statement: "There are no Anglo-Egyptian units on the map at start; the GORDON unit is not used in the campaign game.", concerns: [unit:gordon] },
  { id: rule:r9_14_tomb_control, section: section:`9.14`, kind: "definition", scenarios: [scenario:campaign], statement: "The Mahdi's Tomb is worth 25 VP to the player who controls it at the conclusion of play; the Dervish player controls it at the start.", actions: [action:occupy_tomb], concerns: [location:mahdis_tomb, vp_award:tomb] },
  { id: rule:r9_14_take_tomb, section: section:`9.14`, kind: "condition", scenarios: [scenario:campaign], statement: "To take the Tomb hex it must be occupied at the conclusion of play by one British leader plus any one non-Friendlies Anglo-Egyptian combat unit, both undisrupted.",
    actions: [action:occupy_tomb], concerns: [location:mahdis_tomb, unit_class:ae_leader, unit_class:friendlies, state:disrupted],
    params: { needs: ["one British leader", "one non-Friendlies Anglo-Egyptian combat unit"], both_undisrupted: true, when: "conclusion of play" } },
  { id: rule:r9_14_tactical_note, section: section:`9.14`, kind: "recommendation", scenarios: [scenario:campaign], statement: "The Anglo-Egyptian player will find a decisive victory almost impossible without taking the Mahdi's Tomb.", concerns: [location:mahdis_tomb] },
  { id: rule:r9_14_levels, section: section:`9.14`, kind: "procedure", scenarios: [scenario:campaign], statement: "At the conclusion of play victory points are totaled and victory levels assigned by point superiority (victory_level records).", concerns: [scenario:campaign] },

  -- 9.2 historical
  { id: rule:r9_212_out_of_los, section: section:`9.212`, kind: "limit", scenarios: [scenario:historical], statement: "All Dervish units must be set up out of the line of sight of all Anglo-Egyptian units.", actions: [action:set_up], concerns: [faction:dervish, component:los_table] },
  { id: rule:r9_212_near_leader, section: section:`9.212`, kind: "limit", scenarios: [scenario:historical], statement: "All remaining Dervish units set up within three hexes of their leader as identified by color.", actions: [action:set_up], concerns: [unit_class:d_leader], params: { radius: 3 } },
  { id: rule:r9_231_thorn_hedge, section: section:`9.231`, kind: "modifier", scenarios: [scenario:historical, scenario:campaign], statement: "Thorn hedge hexsides: −2 to the die roll on all Dervish fire attacks; no melee across in either direction; no advance after combat across in either direction.",
    actions: [action:fire, action:melee_attack, action:advance_after_combat], concerns: [terrain:thorn_hedge, modifier:thorn_hedge_fire], exception_to: [rule:r7_7_no_terrain] },
  { id: rule:r9_232_trench, section: section:`9.232`, kind: "modifier", scenarios: [scenario:historical, scenario:campaign], statement: "Trench hexsides: −4 to the die roll on all Dervish fire attacks vs. entrenched units only; −2 (instead of +2) melee modifier to Dervish units melee attacking an entrenched unit; entrenched units may be fired over in both directions.",
    actions: [action:fire, action:melee_attack], concerns: [terrain:trench, state:entrenched, modifier:trench_fire, modifier:trench_melee], exception_to: [rule:r7_7_no_terrain] },
  { id: rule:r9_233_access, section: section:`9.233`, kind: "limit", scenarios: [scenario:historical, scenario:campaign], statement: "Units may only enter and/or leave the Zariba via the two end hexsides that connect to the Nile River, paying +2 MP to cross (exception: advance after combat across an entrenched hexside).",
    actions: [action:move, action:advance_after_combat], concerns: [location:zariba, modifier:zariba_crossing] },
  { id: rule:r9_24_net_victory, section: section:`9.24`, kind: "procedure", scenarios: [scenario:historical], statement: "Each side's victory level comes from enemy units eliminated; the lower level value is subtracted from the higher to give the net victory. Example: 104 Dervish eliminated (5) vs 18 Anglo-Egyptian lost (4) nets 1, a draw.",
    concerns: [scenario:historical], params: { example: { ae_kills: 104, ae_level: 5, dervish_kills: 18, dervish_level: 4, net: 1, result: "draw" } } },

  -- 9.3 Fall of Khartoum
  { id: rule:r9_31_map, section: section:`9.31`, kind: "scope", scenarios: [scenario:fall_of_khartoum], statement: "Only the small FALL OF KHARTOUM map is used.", concerns: [map:khartoum] },
  { id: rule:r9_341_night_turn1, section: section:`9.341`, kind: "scope", scenarios: [scenario:fall_of_khartoum], statement: "Turn 1 is always a night turn (8.1).", concerns: [state:night] },
  { id: rule:r9_342_partial_hexes, section: section:`9.342`, kind: "definition", scenarios: [scenario:fall_of_khartoum], statement: "All hexes are playable, including hexes showing up half or less.", concerns: [terrain:map_edge_partial_hex, map:khartoum] },
  { id: rule:r9_343_dervish_table, section: section:`9.343`, kind: "scope", scenarios: [scenario:fall_of_khartoum], statement: "Both players must use the Dervish Range Effects Table.", actions: [action:fire], concerns: [range_table:dervish], exception_to: [rule:r2_32_ae_weapons] },
  { id: rule:r9_344_north_fort, section: section:`9.344`, kind: "permission", scenarios: [scenario:fall_of_khartoum], statement: "The Dervish player controls the North Fort and may fire its guns.", actions: [action:fire], concerns: [location:north_fort] },
  { id: rule:r9_345_nile_switch, section: section:`9.345`, kind: "permission", scenarios: [scenario:fall_of_khartoum], statement: "British gunboats may move between the White Nile and the Blue Nile at an off-board cost of six upstream movement points.", actions: [action:switch_nile, action:move_upstream], concerns: [location:white_nile, location:blue_nile, constant:nile_switch_cost], params: { mp: 6, kind: "upstream" } },
  { id: rule:r9_346_gordon, section: section:`9.346`, kind: "limit", scenarios: [scenario:fall_of_khartoum], statement: "GORDON starts in the palace and may not move; he may only be eliminated by a Dervish unit passing through or occupying the palace hex (by normal movement or advance after combat).",
    actions: [action:move, action:overrun_leader, action:advance_after_combat], concerns: [unit:gordon, location:khartoum_palace], exception_to: [rule:r6_51_ae_leader_elimination] },
  { id: rule:r9_35_demotion, section: section:`9.35`, kind: "procedure", scenarios: [scenario:fall_of_khartoum], statement: "The Dervish player loses one victory level for 16–23 units lost, two for 24–31, three for 32 or more. Example: Gordon eliminated on turn five (Dervish tactical) with 24 Dervish units lost becomes a British marginal victory.",
    concerns: [unit:gordon], params: { bands: [{ min: 16, max: 23, levels: 1 }, { min: 24, max: 31, levels: 2 }, { min: 32, max: NONE, levels: 3 }],
      scale: ["Dervish decisive", "Dervish tactical", "Dervish marginal", "British marginal", "British tactical", "British decisive"] } },

  -- 10 optional rules
  { id: rule:r10_campaign_only, section: section:`10`, kind: "scope", scenarios: [scenario:campaign], statement: "The optional rules are for the campaign game only.", concerns: [scenario:campaign] },
  { id: rule:r10_one_or_other, section: section:`10`, kind: "recommendation", scenarios: [scenario:campaign], statement: "Most intriguing: let the Dervish player have one option or the other without the Anglo-Egyptian player knowing which; using both in the same game is not recommended." },
  { id: rule:r10_11_record_mines, section: section:`10.11`, kind: "procedure", scenarios: [scenario:campaign], statement: "Before play the Dervish player secretly records two Nile River hexes to be mined (not both in the same hex), south of the E–W hexrow in which the Khor Shambat empties into the Nile.",
    actions: [action:record_mines], concerns: [location:nile, location:khor_shambat, constant:mines], params: { count: 2, distinct: true, area: "south of the E–W hexrow in which the Khor Shambat empties into the Nile" } },
  { id: rule:r10_12_strike_mine, section: section:`10.12`, kind: "procedure", scenarios: [scenario:campaign], statement: "When a British gunboat enters a mined hex it must stop; the Dervish player rolls the d10: 1–4 no effect, 5–7 damaged (drifts two hexes per turn with the current for the rest of the game; guns and Maxims unaffected unless they drift out of range), 8–10 sunk.",
    actions: [action:strike_mine, action:drift, action:sink_gunboat], concerns: [unit_class:old_gunboat, unit_class:named_gunboat, state:mine_damaged] },
  { id: rule:r10_13_two_only, section: section:`10.13`, kind: "limit", scenarios: [scenario:campaign], statement: "After both mines have been rolled for, no more are available.", actions: [action:strike_mine] },
  { id: rule:r10_14_dervish_gunboats_safe, section: section:`10.14`, kind: "permission", scenarios: [scenario:campaign], statement: "Dervish gunboats may pass through mined hexes with no ill effect.", actions: [action:strike_mine], concerns: [unit_class:d_gunboat] },
  { id: rule:r10_21_record_chain, section: section:`10.21`, kind: "procedure", scenarios: [scenario:campaign], statement: "Before play the Dervish player secretly records a line of river hexes (not exceeding four hexes) across which the chain is strung, south of the E–W hexrow in which the Khor Shambat empties into the Nile.",
    actions: [action:record_chain], concerns: [location:nile, location:khor_shambat, constant:chain_max_hexes], params: { max_hexes: 4 } },
  { id: rule:r10_22_chain_stop, section: section:`10.22`, kind: "obligation", scenarios: [scenario:campaign], statement: "A British gunboat entering a chained river hex must stop and may move no further that turn.", actions: [action:move], concerns: [unit_class:old_gunboat, unit_class:named_gunboat] },
  { id: rule:r10_23_chain_block, section: section:`10.23`, kind: "prohibition", scenarios: [scenario:campaign], statement: "No gunboats (British or Dervish) may cross the chain until the British player sinks it.", actions: [action:cross_chain], concerns: [unit_class:gunboat] },
  { id: rule:r10_23_sink_chain, section: section:`10.23`, kind: "procedure", scenarios: [scenario:campaign], statement: "The British player sinks the chain by (a) having an infantry or cavalry unit spend one complete turn on either riverbank adjacent to a chained river hex, or (b) firing at it with artillery and achieving 3 or more on the CRT.",
    actions: [action:sink_chain, action:fire_at_chain], concerns: [unit_class:infantry, unit_class:cavalry, unit_class:artillery, threshold:sink_chain] },

  -- back cover
  { id: rule:rcrt_results, section: section:`CombatResults`, kind: "definition", statement: "CRT results: — miss; D: ½ (round up) of the units in the target hex are disrupted; #: that many units are eliminated.", actions: [action:disrupt, action:eliminate], concerns: [combat_result:miss, combat_result:disrupted, combat_result:eliminated] },
  { id: rule:rcrt_clamp, section: section:`CombatResults`, kind: "limit", statement: "Modified die rolls of less than 1 are treated as 1, more than 10 as 10.", concerns: [component:d10, constant:die_min, constant:die_max], params: { min: 1, max: 10 } },
  { id: rule:rdisrupted, section: section:`Disrupted`, kind: "effect", statement: "Disrupted units have no ZOC, may not move, may not fire offensively or defensively, may not melee, and are turned face up at the end of the owning player's turn.",
    actions: [action:exert_zoc, action:move, action:fire, action:melee_attack, action:recover], concerns: [state:disrupted] },
  { id: rule:rcredits_questions, section: section:`credits`, kind: "procedure", statement: "Rules questions are answered if phrased to be answered yes or no and accompanied by a stamped, self-addressed envelope (Phoenix Enterprises, Ltd., P.O. Box 81192, Chamblee, Ga. 30366)." }
];

-- ------------------------------------------------------------------ modifiers
INSERT INTO modifier [
  { id: modifier:ae_direct_fire, name: "Anglo-Egyptian accuracy bonus", value: 1, applies_to: "fire die roll", who: faction:anglo_egyptian, condition: "all Anglo-Egyptian direct fire attacks", cumulative: true, rule: rule:r6_24_ae_accuracy, section: section:`6.24` },
  { id: modifier:brigade_integrity, name: "brigade integrity", value: 1, applies_to: "fire die roll", who: faction:anglo_egyptian, condition: "stack with brigade integrity, all four firing at the same enemy-occupied hex", cumulative: true, rule: rule:r5_54_brigade_integrity, section: section:`5.54` },
  { id: modifier:dervish_melee, name: "Dervish melee modifier", value: 2, applies_to: "melee die roll", who: faction:dervish, also_used_by: [unit_class:friendlies], rule: rule:r7_7_resolution, section: section:`7.7` },
  { id: modifier:ae_melee, name: "Anglo-Egyptian melee modifier", value: 1, applies_to: "melee die roll", who: faction:anglo_egyptian, not_used_by: [unit_class:friendlies], rule: rule:r7_7_resolution, section: section:`7.7` },
  { id: modifier:fort_defense, name: "fort defensive value", value: -3, applies_to: "fire die roll", against: "enemy fire attacks on friendly units stacked inside the fort", rule: rule:r6_54_fort_defense, section: section:`6.54` },
  { id: modifier:terrain_defense, name: "terrain defensive benefit", value: NONE, applies_to: "fire die roll", sign: "negative", source: component:terrain_effects_chart, rule: rule:r6_23_terrain_and_crt, section: section:`6.23`,
    note: "Values are on the Terrain Effects Chart, not in the manual text." },
  { id: modifier:thorn_hedge_fire, name: "thorn hedge vs Dervish fire", value: -2, applies_to: "fire die roll", who: faction:dervish, condition: "all Dervish fire attacks (across thorn hedge hexsides)", rule: rule:r9_231_thorn_hedge, section: section:`9.231` },
  { id: modifier:trench_fire, name: "trench vs Dervish fire", value: -4, applies_to: "fire die roll", who: faction:dervish, condition: "Dervish fire attacks vs. entrenched units only", rule: rule:r9_232_trench, section: section:`9.232` },
  { id: modifier:trench_melee, name: "Dervish melee vs entrenched", value: -2, replaces: modifier:dervish_melee, applies_to: "melee die roll", who: faction:dervish, condition: "Dervish units melee attacking an entrenched unit", rule: rule:r9_232_trench, section: section:`9.232` },
  { id: modifier:zariba_crossing, name: "Zariba end hexside crossing", value: 2, applies_to: "movement points", condition: "entering or leaving the Zariba via an end hexside", rule: rule:r9_233_access, section: section:`9.233` },
  { id: modifier:night_ae_movement, name: "night Anglo-Egyptian movement", factor: 0.5, rounding: "down", applies_to: "movement allowance", who: faction:anglo_egyptian, rule: rule:r8_1_ae_movement_halved, section: section:`8.1` },
  { id: modifier:night_range, name: "night fire range", factor: 0.5, rounding: "down (range 1 stays range 1)", applies_to: "fire range", who: "both sides", rule: rule:r8_1_ranges_halved, section: section:`8.1` }
];

-- ------------------------------------------------------------------ thresholds & die outcomes
INSERT INTO threshold [
  { id: threshold:sink_gunboat, target: "gunboat", min_result: 3, firer: unit_class:artillery, otherwise: "miss", rule: rule:r6_61_gunboat_target, section: section:`6.61` },
  { id: threshold:destroy_fort, target: "fort", min_result: 2, firer: unit_class:artillery, otherwise: "miss", side_effect: "one enemy unit inside is eliminated with the fort", rule: rule:r6_62_fort_target, section: section:`6.62` },
  { id: threshold:breach_wall, target: "wall hexside", min_result: 2, firer: unit_class:artillery, otherwise: "miss", side_effect: "one enemy unit adjacent to the hexside is eliminated", rule: rule:r6_63_breach, section: section:`6.63` },
  { id: threshold:sink_chain, target: "river chain", min_result: 3, firer: unit_class:artillery, rule: rule:r10_23_sink_chain, section: section:`10.23` }
];
INSERT INTO die_outcome [
  { id: die_outcome:howitzer_on_target, table: "howitzer impact roll", min: 7, max: 10, outcome: "designated target hex is hit", rule: rule:r6_64_howitzer, section: section:`6.64` },
  { id: die_outcome:howitzer_scatter, table: "howitzer impact roll", min: 1, max: 6, outcome: "impact hex per the Howitzer Fire Scattergram", rule: rule:r6_64_howitzer, section: section:`6.64`,
    basis: "implied", note: "6.64 names 7–10 as the hit on the target; the scattergram on the map gives the other impact hexes." },
  { id: die_outcome:mine_no_effect, table: "mine blast", min: 1, max: 4, outcome: "no effect", rule: rule:r10_12_strike_mine, section: section:`10.12` },
  { id: die_outcome:mine_damaged, table: "mine blast", min: 5, max: 7, outcome: "gunboat damaged: lost use of its engines, drifts two hexes per turn with the current for the rest of the game", state: state:mine_damaged, rule: rule:r10_12_strike_mine, section: section:`10.12` },
  { id: die_outcome:mine_sunk, table: "mine blast", min: 8, max: 10, outcome: "gunboat sunk", rule: rule:r10_12_strike_mine, section: section:`10.12` }
];

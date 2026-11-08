-- capability edges: (unit_class | unit | tribe) -> action, allowed true/false.
-- `exception: true` marks an edge that overrides a more general one (more specific subject,
-- or a stated exception); 99_checks.surql verifies no other conflicts exist.

LET $dervish_leaves = [unit_class:d_infantry, unit_class:d_camel, unit_class:d_leader, unit_class:d_artillery, unit_class:d_gunboat, unit_class:fort];
LET $ae_leaves = [unit_class:british_infantry, unit_class:egyptian_infantry, unit_class:sudanese_infantry, unit_class:friendlies, unit_class:ae_cavalry,
  unit_class:camel_corps, unit_class:ae_artillery, unit_class:maxim_battery, unit_class:old_gunboat, unit_class:named_gunboat, unit_class:ae_leader, unit_class:royal_engineers];
LET $all_leaves = array::concat($dervish_leaves, $ae_leaves);
LET $gunboats = [unit_class:d_gunboat, unit_class:old_gunboat, unit_class:named_gunboat];

-- melee attack (7.4, 6.51, 6.53, 6.54, 7.1)
INSERT RELATION INTO capability [
  { in: unit_class:infantry, out: action:melee_attack, allowed: true, rule: rule:r7_4_attackers, basis: "explicit" },
  { in: unit_class:cavalry, out: action:melee_attack, allowed: true, rule: rule:r7_4_attackers, basis: "explicit" },
  { in: unit_class:camel, out: action:melee_attack, allowed: true, rule: rule:r7_4_attackers, basis: "explicit" },
  { in: unit_class:d_leader, out: action:melee_attack, allowed: true, rule: rule:r7_4_attackers, basis: "explicit" },
  { in: unit_class:royal_engineers, out: action:melee_attack, allowed: true, rule: rule:r6_53_engineers, basis: "implied", note: "6.53: 'In addition to normal combat and melee capabilities'; 6.53 also forbids melee attack in the demolition turn, which presupposes the capability." },
  { in: unit_class:artillery, out: action:melee_attack, allowed: false, rule: rule:r7_4_attackers, basis: "explicit" },
  { in: unit_class:maxim_battery, out: action:melee_attack, allowed: false, rule: rule:r7_4_attackers, basis: "explicit" },
  { in: unit_class:gunboat, out: action:melee_attack, allowed: false, rule: rule:r7_1_gunboats_no_melee, basis: "explicit" },
  { in: unit_class:fort, out: action:melee_attack, allowed: false, rule: rule:r6_54_fort_melee_defensive, basis: "explicit" },
  { in: unit_class:ae_leader, out: action:melee_attack, allowed: false, rule: rule:r7_4_attackers, basis: "explicit", note: "7.4 names only Dervish leaders; 6.51 gives Anglo-Egyptian leaders a movement factor only." }
];
-- melee defend (7.4, 7.1, 6.54)
FOR $c IN $all_leaves {
  RELATE $c->capability->action:melee_defend CONTENT { allowed: $c NOT IN $gunboats, rule: IF $c IN $gunboats { rule:r7_1_gunboats_no_melee } ELSE { rule:r7_4_defenders }, basis: "explicit" };
};
-- fire (6.51, 6.54, 2.31, 2.32)
FOR $c IN $all_leaves {
  RELATE $c->capability->action:fire CONTENT { allowed: $c != unit_class:ae_leader,
    rule: IF $c = unit_class:ae_leader { rule:r6_51_ae_leaders } ELSE IF $c = unit_class:fort { rule:r6_54_fort_fires_alone } ELSE { rule:r6_11_fire_factor }, basis: "explicit" };
};
-- ZOC (5.41, 6.54)
FOR $c IN $all_leaves {
  RELATE $c->capability->action:exert_zoc CONTENT { allowed: $c != unit_class:ae_leader,
    condition: IF $c IN $gunboats { "only against enemy gunboats" } ELSE IF $c = unit_class:fort { "even if unoccupied" } ELSE { NONE },
    rule: IF $c = unit_class:ae_leader { rule:r5_41_who_exerts } ELSE IF $c IN $gunboats { rule:r5_41_gunboat_zoc } ELSE IF $c = unit_class:fort { rule:r6_54_fort_zoc } ELSE { rule:r5_41_who_exerts }, basis: "explicit" };
};
-- movement (5.11, 5.25, 9.346)
FOR $c IN $all_leaves {
  RELATE $c->capability->action:move CONTENT { allowed: $c != unit_class:fort, rule: IF $c = unit_class:fort { rule:r5_25_forts_immobile } ELSE { rule:r5_11_allowance }, basis: "explicit" };
};
RELATE unit:gordon->capability->action:move CONTENT { allowed: false, exception: true, scenario: scenario:fall_of_khartoum, rule: rule:r9_346_gordon, basis: "explicit" };
-- gunboat river movement (5.22, 5.24)
FOR $c IN $gunboats {
  RELATE $c->capability->action:move_upstream CONTENT { allowed: true, rule: rule:r5_24_two_allowances, basis: "explicit" };
  RELATE $c->capability->action:move_downstream CONTENT { allowed: true, rule: rule:r5_24_two_allowances, basis: "explicit" };
  RELATE $c->capability->action:enter_nile CONTENT { allowed: true, rule: rule:r5_22_land_units_no_nile, basis: "explicit" };
};
RELATE unit_class:land_unit->capability->action:enter_nile CONTENT { allowed: false, rule: rule:r5_22_land_units_no_nile, basis: "explicit" };
RELATE unit_class:friendlies->capability->action:enter_nile CONTENT { allowed: true, exception: true, condition: "only as transported by an Anglo-Egyptian gunboat after Isa Zachneih is eliminated (5.21)", rule: rule:r5_21_friendlies_ferry, basis: "explicit" };
-- stacking (5.51, 5.53)
RELATE unit_class:leader->capability->action:stack CONTENT { allowed: true, condition: "free stacking, in addition to the four-unit limit; not required to stack; Dervish leaders only with units of their command", rule: rule:r5_51_leaders_free, basis: "explicit" };
RELATE unit_class:gunboat->capability->action:stack CONTENT { allowed: false, condition: "may not stack with any other unit (exception 5.21)", rule: rule:r5_51_gunboats_alone, basis: "explicit" };
-- naval transport (5.21)
RELATE unit_class:gunboat->capability->action:naval_transport CONTENT { allowed: false, rule: rule:r5_21_no_naval_transport, basis: "explicit" };
FOR $c IN [unit_class:old_gunboat, unit_class:named_gunboat] {
  RELATE $c->capability->action:naval_transport CONTENT { allowed: true, exception: true, condition: "only Friendlies units, east bank to west bank, after Isa Zachneih is eliminated", rule: rule:r5_21_friendlies_ferry, basis: "explicit" };
};
RELATE unit_class:land_unit->capability->action:load CONTENT { allowed: false, rule: rule:r5_21_no_naval_transport, basis: "explicit" };
RELATE unit_class:friendlies->capability->action:load CONTENT { allowed: true, exception: true, condition: "start the turn adjacent to an Anglo-Egyptian gunboat; Isa Zachneih eliminated", rule: rule:r5_21_friendlies_ferry, basis: "explicit" };
RELATE unit_class:friendlies->capability->action:disembark CONTENT { allowed: true, condition: "on the Anglo-Egyptian player's third turn of the transport", rule: rule:r5_21_friendlies_ferry, basis: "explicit" };
-- second subphase (6.42, 6.64)
RELATE unit_class:maxim_battery->capability->action:maxim_second_fire CONTENT { allowed: true, rule: rule:r6_42_second_subphase, basis: "explicit" };
RELATE unit_class:named_gunboat->capability->action:maxim_second_fire CONTENT { allowed: true, rule: rule:r6_42_second_subphase, basis: "implied", note: "Named gunboats carry 'Maxim Guns (fire twice per turn)' (2.3); 6.42 lets all Maxim guns fire a second time." };
RELATE unit_class:named_gunboat->capability->action:howitzer_fire CONTENT { allowed: true, condition: "not during night game turns", rule: rule:r6_64_howitzer, basis: "explicit" };
FOR $c IN [unit_class:old_gunboat, unit_class:ae_artillery, unit_class:d_artillery, unit_class:d_gunboat, unit_class:fort] {
  RELATE $c->capability->action:howitzer_fire CONTENT { allowed: false, rule: rule:r6_64_howitzer, basis: "explicit", note: "6.64: the five named British gunboats are the only units with howitzer fire capability." };
};
-- special targets (6.61-6.63, 10.23)
FOR $a IN [action:fire_at_gunboat, action:fire_at_fort, action:breach_wall, action:fire_at_chain] {
  RELATE unit_class:artillery->capability->$a CONTENT { allowed: true, rule: (SELECT VALUE rule FROM ONLY threshold WHERE (target = "gunboat" AND $a = action:fire_at_gunboat) OR (target = "fort" AND $a = action:fire_at_fort) OR (target = "wall hexside" AND $a = action:breach_wall) OR (target = "river chain" AND $a = action:fire_at_chain) LIMIT 1), basis: "explicit" };
};
FOR $c IN [unit_class:infantry, unit_class:cavalry, unit_class:camel, unit_class:maxim_battery, unit_class:d_leader, unit_class:royal_engineers] {
  RELATE $c->capability->action:fire_at_gunboat CONTENT { allowed: false, rule: rule:r6_61_gunboat_target, basis: "explicit" };
  RELATE $c->capability->action:fire_at_fort CONTENT { allowed: false, rule: rule:r6_62_fort_target, basis: "explicit" };
  RELATE $c->capability->action:breach_wall CONTENT { allowed: false, rule: rule:r6_63_breach, basis: "explicit" };
};
FOR $c IN [unit_class:gunboat, unit_class:fort] {
  RELATE $c->capability->action:fire_at_gunboat CONTENT { allowed: true, rule: rule:r6_61_gunboat_target, basis: "implied",
    note: "Gunboats and forts fire an 'artillery factor' on the artillery line (2.31, 2.32, 6.54); the manual does not say whether 'only artillery' in 6.61 includes them. See open_question:artillery_meaning." };
};
RELATE unit_class:gunboat->capability->action:fire_at_fort CONTENT { allowed: true, rule: rule:r6_62_fort_target, basis: "implied", note: "As for fire_at_gunboat; see open_question:artillery_meaning." };
RELATE unit_class:gunboat->capability->action:breach_wall CONTENT { allowed: true, rule: rule:r6_63_breach, basis: "implied", note: "As for fire_at_gunboat; see open_question:artillery_meaning." };
-- engineering (5.3, 6.53)
FOR $c IN [unit_class:british_infantry, unit_class:egyptian_infantry, unit_class:sudanese_infantry] {
  RELATE $c->capability->action:construct_zariba CONTENT { allowed: true, scenario: scenario:campaign, rule: rule:r5_3_zariba_construction, basis: "explicit" };
};
FOR $c IN [unit_class:ae_cavalry, unit_class:camel_corps, unit_class:ae_artillery, unit_class:maxim_battery, unit_class:ae_leader, unit_class:old_gunboat, unit_class:named_gunboat] {
  RELATE $c->capability->action:construct_zariba CONTENT { allowed: false, scenario: scenario:campaign, rule: rule:r5_3_zariba_construction, basis: "explicit", note: "5.3 names 'any Anglo-Egyptian infantry unit'." };
};
RELATE unit_class:royal_engineers->capability->action:demolish CONTENT { allowed: true, rule: rule:r6_53_engineers, basis: "explicit" };
-- advance after combat (6.82, 7.6)
RELATE unit_class:artillery->capability->action:advance_after_combat CONTENT { allowed: false, rule: rule:r6_82_advance_limits, basis: "explicit" };
FOR $c IN [unit_class:infantry, unit_class:cavalry, unit_class:camel, unit_class:d_leader, unit_class:royal_engineers, unit_class:maxim_battery] {
  RELATE $c->capability->action:advance_after_combat CONTENT { allowed: true, condition: "participated in the attack and adjacent to the vacated hex", rule: rule:r6_82_advance, basis: "explicit" };
};
-- retreat (7.5)
RELATE unit_class:cavalry->capability->action:retreat CONTENT { allowed: true, condition: "two hexes, from an infantry melee attack, once per turn", rule: rule:r7_5_retreat, basis: "explicit" };
RELATE unit_class:camel->capability->action:retreat CONTENT { allowed: true, condition: "two hexes, from an infantry melee attack, once per turn", rule: rule:r7_5_retreat, basis: "explicit" };
-- desertion (8.2)
FOR $c IN [unit_class:d_infantry, unit_class:d_camel, unit_class:d_leader] {
  RELATE $c->capability->action:desert CONTENT { allowed: true, rule: rule:r8_2_desertion, basis: "explicit", note: "Any Dervish unit except the KHALIFA, gunboats, artillery units and forts may be chosen." };
};
RELATE unit:khalifa_abdullah->capability->action:desert CONTENT { allowed: false, exception: true, rule: rule:r8_2_desertion, basis: "explicit" };
FOR $c IN [unit_class:d_gunboat, unit_class:d_artillery, unit_class:fort] {
  RELATE $c->capability->action:desert CONTENT { allowed: false, rule: rule:r8_2_desertion, basis: "explicit" };
};
-- the Tomb (9.14, 6.51)
RELATE unit_class:ae_leader->capability->action:occupy_tomb CONTENT { allowed: true, condition: "one British leader plus one non-Friendlies Anglo-Egyptian combat unit, both undisrupted, at the conclusion of play", rule: rule:r9_14_take_tomb, basis: "explicit" };
RELATE unit_class:friendlies->capability->action:occupy_tomb CONTENT { allowed: false, condition: "cannot be the combat unit that takes the Tomb", rule: rule:r9_14_take_tomb, basis: "explicit" };
FOR $c IN [unit_class:british_infantry, unit_class:egyptian_infantry, unit_class:sudanese_infantry, unit_class:ae_cavalry, unit_class:camel_corps, unit_class:ae_artillery, unit_class:maxim_battery, unit_class:royal_engineers] {
  RELATE $c->capability->action:occupy_tomb CONTENT { allowed: true, condition: "together with a British leader, both undisrupted, at the conclusion of play; must reach the walled city through a gate or breach", rule: rule:r9_14_take_tomb, basis: "explicit" };
};
-- leader overrun (6.51) and Gordon (9.346)
FOR $c IN [unit_class:d_infantry, unit_class:d_camel, unit_class:d_leader, unit_class:d_artillery] {
  RELATE $c->capability->action:overrun_leader CONTENT { allowed: true, condition: "the Anglo-Egyptian leader is alone in the hex the Dervish unit occupies or passes through", rule: rule:r6_51_ae_leader_elimination, basis: "explicit", note: "6.51 says 'a Dervish unit'." };
};
-- river chain (10.23)
RELATE unit_class:infantry->capability->action:sink_chain CONTENT { allowed: true, condition: "spend one complete turn on either riverbank adjacent to a chained river hex", scenario: scenario:campaign, rule: rule:r10_23_sink_chain, basis: "explicit" };
RELATE unit_class:cavalry->capability->action:sink_chain CONTENT { allowed: true, condition: "spend one complete turn on either riverbank adjacent to a chained river hex", scenario: scenario:campaign, rule: rule:r10_23_sink_chain, basis: "explicit" };
RELATE unit_class:gunboat->capability->action:cross_chain CONTENT { allowed: false, condition: "until the British player has sunk the chain", scenario: scenario:campaign, rule: rule:r10_23_chain_block, basis: "explicit" };
-- Nile switch (9.345)
FOR $c IN [unit_class:old_gunboat, unit_class:named_gunboat] {
  RELATE $c->capability->action:switch_nile CONTENT { allowed: true, scenario: scenario:fall_of_khartoum, condition: "off-board cost of six upstream movement points", rule: rule:r9_345_nile_switch, basis: "explicit" };
};

-- disrupted state prohibitions (back cover) and other state effects
INSERT RELATION INTO prohibits [
  { in: state:disrupted, out: action:exert_zoc, rule: rule:rdisrupted, basis: "explicit" },
  { in: state:disrupted, out: action:move, rule: rule:rdisrupted, basis: "explicit" },
  { in: state:disrupted, out: action:fire, rule: rule:rdisrupted, basis: "explicit" },
  { in: state:disrupted, out: action:offensive_fire, rule: rule:rdisrupted, basis: "explicit" },
  { in: state:disrupted, out: action:defensive_fire, rule: rule:rdisrupted, basis: "explicit" },
  { in: state:disrupted, out: action:melee_attack, rule: rule:rdisrupted, basis: "explicit" },
  { in: state:disrupted, out: action:demolish, rule: rule:r6_53_engineers, basis: "explicit", note: "the target is destroyed only if the Engineers are undisrupted at the end of the Anglo-Egyptian player turn" },
  { in: state:disrupted, out: action:occupy_tomb, rule: rule:r9_14_take_tomb, basis: "explicit", note: "both Tomb occupiers must be undisrupted" },
  { in: state:constructing_zariba, out: action:offensive_fire, rule: rule:r5_3_constructor_restrictions, basis: "explicit" },
  { in: state:constructing_zariba, out: action:melee_attack, rule: rule:r5_3_constructor_restrictions, basis: "explicit" },
  { in: state:demolishing, out: action:offensive_fire, rule: rule:r6_53_engineers, basis: "explicit" },
  { in: state:demolishing, out: action:melee_attack, rule: rule:r6_53_engineers, basis: "explicit" },
  { in: state:in_enemy_zoc, out: action:move, rule: rule:r5_43_stop, basis: "explicit", note: "may move no further that turn; next movement phase may withdraw or move ZOC to ZOC" },
  { in: state:night, out: action:howitzer_fire, rule: rule:r6_64_no_night_howitzer, basis: "explicit" },
  { in: state:mine_damaged, out: action:move, rule: rule:r10_12_strike_mine, basis: "explicit", note: "lost use of its engines; drifts instead" }
];

-- old (unnamed) gunboats carry no Maxim guns (counter 4·10/16, 2.3; Artillery line only, 2.32)
RELATE unit_class:old_gunboat->capability->action:maxim_second_fire CONTENT { allowed: false, rule: rule:r6_42_second_subphase, basis: "implied",
  note: "The old gunboat counter (GUNBOAT 4·10/16) shows no Maxim guns and fires on the Artillery line only (2.32); 6.42 gives the second fire to Maxim guns." };

-- 7.5 gives the two-hex retreat to cavalry and camel units only. Dervish tribal units are left
-- unstated: their infantry classing is only implied and open_question:dervish_cavalry asks which are mounted.
FOR $c IN [unit_class:ae_infantry, unit_class:friendlies, unit:isa_zachneih, unit_class:artillery, unit_class:leader, unit_class:fort, unit_class:maxim_battery, unit_class:royal_engineers, unit_class:gunboat] {
  RELATE $c->capability->action:retreat CONTENT { allowed: false, rule: rule:r7_5_retreat, basis: "implied", note: "7.5 names only cavalry and camel units." };
};
-- the chain is another 'only artillery' target (10.23); gunboats as for 6.61-6.63
RELATE unit_class:gunboat->capability->action:fire_at_chain CONTENT { allowed: true, rule: rule:r10_23_sink_chain, basis: "implied", note: "As for fire_at_gunboat; see open_question:artillery_meaning." };

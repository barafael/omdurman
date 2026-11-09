-- Consistency checks. Each block starts with "-- CHECK <name>"; its last statement must return
-- [] (or NONE/0). They guard against contradictions the corpus could introduce on its own:
-- dangling links, conflicting permissions, overlapping victory bands, and numbers stated in
-- several places that disagree.

-- CHECK every `section` link resolves
RETURN array::flatten([
  (SELECT VALUE id FROM rule WHERE section.id = NONE),
  (SELECT VALUE id FROM modifier WHERE section.id = NONE), (SELECT VALUE id FROM threshold WHERE section.id = NONE),
  (SELECT VALUE id FROM die_outcome WHERE section.id = NONE), (SELECT VALUE id FROM zoc_boundary WHERE section.id = NONE),
  (SELECT VALUE id FROM setup WHERE section.id = NONE), (SELECT VALUE id FROM reinforcement WHERE section.id = NONE),
  (SELECT VALUE id FROM vp_award WHERE section.id = NONE), (SELECT VALUE id FROM victory_level WHERE section.id = NONE),
  (SELECT VALUE id FROM victory_condition WHERE section.id = NONE), (SELECT VALUE id FROM constant WHERE section.id = NONE),
  (SELECT VALUE id FROM unit WHERE section.id = NONE), (SELECT VALUE id FROM unit_class WHERE section.id = NONE),
  (SELECT VALUE id FROM tribe WHERE section.id = NONE), (SELECT VALUE id FROM location WHERE section.id = NONE),
  (SELECT VALUE id FROM terrain WHERE section.id = NONE), (SELECT VALUE id FROM action WHERE section.id = NONE),
  (SELECT VALUE id FROM phase WHERE section.id = NONE), (SELECT VALUE id FROM scenario WHERE section.id = NONE),
  (SELECT VALUE id FROM state WHERE section.id = NONE), (SELECT VALUE id FROM component WHERE section.id = NONE)
]);

-- CHECK every record referenced by a rule exists
SELECT id FROM rule WHERE array::len(array::concat(actions ?? [], array::concat(concerns ?? [], exception_to ?? [])).filter(|$x| $x.id = NONE)) > 0;

-- CHECK every edge endpoint exists
RETURN array::flatten([
  (SELECT VALUE id FROM is_a WHERE in.id = NONE OR out.id = NONE), (SELECT VALUE id FROM belongs_to WHERE in.id = NONE OR out.id = NONE),
  (SELECT VALUE id FROM leads WHERE in.id = NONE OR out.id = NONE), (SELECT VALUE id FROM member_of WHERE in.id = NONE OR out.id = NONE),
  (SELECT VALUE id FROM fires_on WHERE in.id = NONE OR out.id = NONE), (SELECT VALUE id FROM capability WHERE in.id = NONE OR out.id = NONE OR rule.id = NONE),
  (SELECT VALUE id FROM entry WHERE in.id = NONE OR out.id = NONE), (SELECT VALUE id FROM governs WHERE in.id = NONE OR out.id = NONE),
  (SELECT VALUE id FROM concerns WHERE in.id = NONE OR out.id = NONE), (SELECT VALUE id FROM exception_to WHERE in.id = NONE OR out.id = NONE),
  (SELECT VALUE id FROM allowed_in WHERE in.id = NONE OR out.id = NONE), (SELECT VALUE id FROM prohibits WHERE in.id = NONE OR out.id = NONE OR rule.id = NONE),
  (SELECT VALUE id FROM next WHERE in.id = NONE OR out.id = NONE), (SELECT VALUE id FROM raises WHERE in.id = NONE OR out.id = NONE),
  (SELECT VALUE id FROM appears_in WHERE in.id = NONE OR out.id = NONE), (SELECT VALUE id FROM about WHERE in.id = NONE OR out.id = NONE),
  (SELECT VALUE id FROM cites WHERE in.id = NONE OR out.id = NONE)
]);

-- CHECK other record links resolve (units, setups, reinforcements, awards, modifiers)
RETURN array::flatten([
  (SELECT VALUE id FROM unit WHERE class.id = NONE OR faction.id = NONE OR (tribe != NONE AND tribe.id = NONE) OR (formation != NONE AND formation.id = NONE)),
  (SELECT VALUE id FROM setup WHERE scenario.id = NONE OR faction.id = NONE OR array::len((subjects ?? []).filter(|$x| $x.id = NONE)) > 0 OR array::len((locations ?? []).filter(|$x| $x.id = NONE)) > 0),
  (SELECT VALUE id FROM reinforcement WHERE scenario.id = NONE OR array::len((subjects ?? []).filter(|$x| $x.id = NONE)) > 0 OR array::len((entry ?? []).filter(|$x| $x.id = NONE)) > 0),
  (SELECT VALUE id FROM vp_award WHERE scenario.id = NONE OR subject.id = NONE OR (where != NONE AND where.id = NONE)),
  (SELECT VALUE id FROM modifier WHERE rule.id = NONE), (SELECT VALUE id FROM threshold WHERE rule.id = NONE OR firer.id = NONE),
  (SELECT VALUE id FROM die_outcome WHERE rule.id = NONE), (SELECT VALUE id FROM zoc_boundary WHERE boundary.id = NONE),
  (SELECT VALUE id FROM open_question WHERE array::len(refs.filter(|$x| $x.id = NONE)) > 0),
  (SELECT VALUE id FROM tribe WHERE unit_class.id = NONE), (SELECT VALUE id FROM range_line WHERE table.id = NONE OR weapon.id = NONE)
]);

-- CHECK no two capability edges on the same subject disagree
SELECT in, out, values FROM (SELECT in, out, array::distinct(allowed) AS values FROM capability GROUP BY in, out) WHERE array::len(values) > 1;

-- CHECK an inherited capability is overridden only by an edge marked exception
LET $subjects = array::flatten([(SELECT VALUE id FROM unit), (SELECT VALUE id FROM tribe), (SELECT VALUE id FROM unit_class WHERE abstract = false)]);
LET $actions = (SELECT VALUE id FROM action);
LET $bad = $subjects.map(|$s| {
  LET $p = fn::precedence($s);
  RETURN (SELECT out AS action, array::group(allowed) AS vals, array::first(array::group({ via: in, allowed: allowed, exception: exception ?? false, pr: array::find_index($p, in) })) AS first
          FROM (SELECT * FROM capability WHERE in IN $p ORDER BY array::find_index($p, in)) GROUP BY out)
    .filter(|$g| array::len(array::distinct($g.vals)) > 1 AND $g.first.exception != true)
    .map(|$g| { subject: $s, action: $g.action, values: $g.vals });
});
RETURN array::flatten($bad);

-- CHECK an inherited entry permission is overridden only by a more specific subject
LET $bad = array::flatten([(SELECT VALUE id FROM unit), (SELECT VALUE id FROM tribe)]).map(|$s| {
  LET $p = fn::precedence($s);
  LET $e = (SELECT out, allowed, array::find_index($p, in) AS pr FROM entry WHERE in IN $p ORDER BY pr);
  RETURN $e.filter(|$x| array::len($e.filter(|$y| $y.out = $x.out AND $y.pr = $x.pr AND $y.allowed != $x.allowed)) > 0).map(|$x| { subject: $s, place: $x.out });
});
RETURN array::flatten($bad);

-- CHECK victory bands neither overlap nor leave gaps (campaign, historical)
LET $groups = (SELECT scenario, faction, array::group({ min: min, max: max }) AS bands FROM victory_level WHERE scenario IN [scenario:campaign, scenario:historical] GROUP BY scenario, faction);
RETURN $groups.map(|$g| {
  LET $b = (SELECT * FROM $g.bands ORDER BY min);
  RETURN { g: [$g.scenario, $g.faction], bad: array::len($b.filter(|$x| $x.max = NONE)) != 1
           OR array::len($b.map(|$x, $i| $i > 0 AND $x.min != $b[$i - 1].max + 1).filter(|$v| $v)) > 0 };
}).filter(|$x| $x.bad);

-- CHECK Fall of Khartoum scale is the six levels 1..6 in order
RETURN IF (SELECT VALUE scale FROM victory_level WHERE scenario = scenario:fall_of_khartoum ORDER BY scale) = [1, 2, 3, 4, 5, 6] { [] } ELSE { ["scale broken"] };

-- CHECK the turn track agrees with the campaign scenario and the track component
LET $n = count(SELECT id FROM turn);
LET $night = (SELECT VALUE n FROM turn WHERE night = true ORDER BY n);
LET $chain = count(SELECT id FROM next WHERE record::tb(in) = "turn");
RETURN array::flatten([
  IF $n != scenario:campaign.length_turns { ["turn count vs campaign length"] } ELSE { [] },
  IF $n != component:turn_record_track.boxes { ["turn count vs track boxes"] } ELSE { [] },
  IF $night != scenario:campaign.night_turns { ["night turns vs campaign"] } ELSE { [] },
  IF array::len($night) != component:turn_record_track.night_boxes { ["night box count"] } ELSE { [] },
  IF $chain != $n - 1 { ["turn chain"] } ELSE { [] },
  IF turn:9.special != "Dervish Desertion Roll" OR $night[0] != 9 { ["desertion roll must be on the first night turn (8.2)"] } ELSE { [] }
]);

-- CHECK reinforcement turns lie inside the scenario
SELECT id FROM reinforcement WHERE (turn != NONE AND scenario.length_turns != NONE AND turn > scenario.length_turns)
  OR (turn_window != NONE AND array::max(turn_window) > scenario.length_turns);

-- CHECK the player who sets up first moves second (9.111, 9.211, 9.321/9.322)
SELECT id FROM scenario WHERE sets_up_first = moves_first OR sets_up_first = NONE OR moves_first = NONE;

-- CHECK numbers stated in several places agree
RETURN array::flatten([
  IF constant:stacking_limit.value != rule:r5_51_limit.params.max_units { ["stacking limit"] } ELSE { [] },
  IF constant:campaign_forts.value != setup:c_forts.quantity OR constant:campaign_forts.value != unit:fort.count { ["forts"] } ELSE { [] },
  IF constant:dervish_artillery_units.value != unit:dervish_artillery.count { ["dervish artillery"] } ELSE { [] },
  IF constant:dervish_gunboats.value != unit:dervish_gunboat.count OR constant:dervish_gunboats.value != setup:c_dervish_gunboats.quantity { ["dervish gunboats"] } ELSE { [] },
  IF constant:howitzer_units.value != unit_class:named_gunboat.count { ["named gunboats"] } ELSE { [] },
  IF constant:zariba_hexes.value != location:zariba.hexes { ["zariba hexes"] } ELSE { [] },
  IF component:howitzer_scattergram.hexes != location:scattergram_hexes.hexes { ["scattergram hexes"] } ELSE { [] },
  IF constant:tomb_vp.value != vp_award:tomb.points OR constant:tomb_vp.value != location:mahdis_tomb.vp { ["tomb vp"] } ELSE { [] },
  IF constant:howitzer_min_range.value != rule:r6_64_howitzer.params.min_range OR constant:howitzer_max_range.value != rule:r6_64_howitzer.params.max_range { ["howitzer range"] } ELSE { [] },
  IF constant:chain_max_hexes.value != rule:r10_21_record_chain.params.max_hexes { ["chain"] } ELSE { [] },
  IF constant:mines.value != rule:r10_11_record_mines.params.count { ["mines"] } ELSE { [] },
  IF constant:historical_leader_radius.value != rule:r9_212_near_leader.params.radius OR constant:historical_leader_radius.value != setup:h_dervish_rest.radius { ["leader radius"] } ELSE { [] },
  IF constant:zariba_crossing_mp.value != modifier:zariba_crossing.value { ["zariba crossing"] } ELSE { [] },
  IF constant:nile_switch_cost.value != rule:r9_345_nile_switch.params.mp { ["nile switch"] } ELSE { [] },
  IF constant:friendlies_entry_mp.value != reinforcement:c_ae_entry_friendlies.entry_cost_mp { ["friendlies entry"] } ELSE { [] },
  IF constant:ae_land_units_per_turn.value != reinforcement:c_ae2.land_units OR constant:ae_land_units_per_turn.value != reinforcement:c_ae3.land_units { ["ae land units"] } ELSE { [] },
  IF constant:ae_gunboats_per_turn.value != reinforcement:c_ae1.gunboats OR constant:ae_gunboats_per_turn.value != reinforcement:c_ae2.gunboats { ["ae gunboats"] } ELSE { [] },
  IF constant:ae_leader_deadline_turn.value != reinforcement:c_ae_leaders.deadline_turn OR constant:ae_leader_deadline_turn.value != rule:r9_113_leaders_timing.params.deadline_turn { ["leader deadline"] } ELSE { [] },
  IF reinforcement:k_d1.total_units != math::sum(reinforcement:k_d1.counts.n) { ["fok dervish total"] } ELSE { [] },
  IF constant:desertion_multiplier.value != rule:r8_2_desertion.params.multiplier { ["desertion"] } ELSE { [] },
  IF constant:cav_retreat_hexes.value != rule:r7_5_retreat.params.hexes { ["retreat"] } ELSE { [] },
  IF constant:brigade_battalions.value != formation:brigade_2b.battalions { ["brigade size"] } ELSE { [] },
  IF fn::halve(array::repeat(9, 4)) != rule:r6_16_halving.params.example.halved_total { ["6.16 example"] } ELSE { [] },
  IF fn::historical_result(rule:r9_24_net_victory.params.example.ae_kills, rule:r9_24_net_victory.params.example.dervish_kills).net != rule:r9_24_net_victory.params.example.net { ["9.24 example"] } ELSE { [] },
  IF fn::fok_result(5, 24).result != "British marginal" { ["9.35 example"] } ELSE { [] },
  IF unit:egyptian_battalion_sample.fire != rule:r6_16_halving.params.example.printed { ["6.16 printed strength vs sample counter"] } ELSE { [] }
]);

-- CHECK every range line and printed counter value is consistent
RETURN array::flatten([
  (SELECT VALUE id FROM unit WHERE printed != NONE AND fire != NONE AND melee != NONE AND move != NONE
     AND string::replace(string::replace(printed, "·", "-"), " ", "") != string::concat(<string> fire, "-", <string> melee, "-", <string> move)),
  (SELECT VALUE id FROM unit WHERE move_upstream != NONE AND move_upstream >= move_downstream)
]);

-- CHECK every unit that may fire has a range line, and Anglo-Egyptian leaders have none
LET $subjects = array::flatten([(SELECT VALUE id FROM unit WHERE generic != true OR class != unit_class:d_infantry), (SELECT VALUE id FROM tribe),
  (SELECT VALUE id FROM unit_class WHERE abstract = false AND id NOT IN [unit_class:d_infantry, unit_class:d_camel])]);
RETURN $subjects.filter(|$s| (fn::allowed($s, action:fire) = true AND array::len(fn::lines($s)) = 0) OR (fn::allowed($s, action:fire) = false AND array::len(fn::lines($s)) > 0));

-- CHECK every faction-bound record agrees with its class's faction
SELECT id FROM unit WHERE class.faction != NONE AND class.faction != faction;

-- CHECK rule exceptions point between different rules and never at themselves
SELECT id FROM exception_to WHERE in = out;

-- CHECK every phase leaf reaches the game turn root
SELECT id FROM phase WHERE step != NONE AND parent.parent.id != phase:game_turn AND parent.parent.parent.id != phase:game_turn AND parent.parent.parent.parent.id != phase:game_turn;

-- CHECK every section has a mapping target for its rules (no rule cites a container heading)
SELECT id, section.num FROM rule WHERE section.num IN ["2", "5", "6", "6.5", "6.6", "7", "9", "9.1", "9.11", "9.2", "9.21", "9.3", "9.32", "9.34"];

-- CHECK every question links to an existing section and has an answer
SELECT id FROM question WHERE section.id = NONE OR answer = NONE OR text = NONE;

-- CHECK every unit and tribe has its counter-sheet sprite (data/10_sprites.surql)
RETURN array::flatten([(SELECT VALUE id FROM unit WHERE sprite = NONE), (SELECT VALUE id FROM tribe WHERE sprite = NONE)]);

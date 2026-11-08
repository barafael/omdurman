-- Rule functions: each one computes a ruling from the corpus records, so questions can be asked
-- as queries. Every function documents the sections it encodes.

-- Precedence list for capability/entry/armament lookup: the subject itself, its tribe, then its
-- classes from most to least specific. The first level carrying an edge decides.
DEFINE FUNCTION OVERWRITE fn::precedence($s: record) {
  LET $tb = record::tb($s);
  RETURN IF $tb = "unit" {
    array::flatten([[$s], IF $s.tribe != NONE { [$s.tribe, $s.tribe.unit_class] } ELSE { [] }, $s.classes])
  } ELSE IF $tb = "tribe" {
    array::flatten([[$s], [$s.unit_class], $s.unit_class.ancestors])
  } ELSE IF $tb = "unit_class" {
    array::concat([$s], $s.ancestors)
  } ELSE { [$s] };
} COMMENT 'subject, tribe, classes (specific to general)';

-- May subject $s perform action $a?  Returns NONE when the manual gives no ruling.
DEFINE FUNCTION OVERWRITE fn::can($s: record, $a: record<action>) {
  LET $p = fn::precedence($s);
  LET $e = (SELECT allowed, condition, basis, note, scenario, rule, rule.section.num AS section, in AS via, array::find_index($p, in) AS pr
            FROM capability WHERE out = $a AND in IN $p ORDER BY pr);
  RETURN $e[0];
} COMMENT 'capability resolution (most specific edge wins)';

DEFINE FUNCTION OVERWRITE fn::allowed($s: record, $a: record<action>) {
  RETURN fn::can($s, $a).allowed;
} COMMENT 'true / false / NONE (unstated)';

-- May subject $s enter place $x (location or terrain)?  (5.22, 5.23, 6.52, 9.233)
DEFINE FUNCTION OVERWRITE fn::may_enter($s: record, $x: record) {
  LET $p = fn::precedence($s);
  LET $e = (SELECT allowed, condition, exception, section.num AS section, in AS via, array::find_index($p, in) AS pr
            FROM entry WHERE out = $x AND in IN $p ORDER BY pr);
  RETURN $e[0];
};

-- Range lines a subject fires on (2.31, 2.32, 6.52); FALL OF KHARTOUM forces the Dervish table (9.343).
DEFINE FUNCTION OVERWRITE fn::lines($s: record) {
  LET $p = fn::precedence($s);
  LET $e = (SELECT out, array::find_index($p, in) AS pr FROM fires_on WHERE in IN $p ORDER BY pr);
  IF array::len($e) = 0 { RETURN []; };
  LET $best = $e[0].pr;
  RETURN (SELECT VALUE out FROM $e WHERE pr = $best);
};
DEFINE FUNCTION OVERWRITE fn::range_table($s: record, $scenario: option<record<scenario>>) {
  IF $scenario = scenario:fall_of_khartoum { RETURN range_table:dervish; };
  LET $l = fn::lines($s);
  RETURN IF array::len($l) = 0 { NONE } ELSE { array::distinct($l.table) };
} COMMENT '9.343 override for FALL OF KHARTOUM';

-- Halving fire strength: round down per unit, never below 1 (6.16).
DEFINE FUNCTION OVERWRITE fn::halve($factors: array<int>) {
  RETURN math::sum($factors.map(|$f| math::max([1, math::floor($f / 2)])));
};

-- Night: Anglo-Egyptian movement allowance halved, round down (8.1).
DEFINE FUNCTION OVERWRITE fn::night_ma($faction: record<faction>, $ma: int) {
  RETURN IF $faction = faction:anglo_egyptian { math::floor($ma / 2) } ELSE { $ma };
};
-- Night: fire range halved, round down, range 1 stays 1 (8.1).
DEFINE FUNCTION OVERWRITE fn::night_range($range: int) {
  RETURN math::max([1, math::floor($range / 2)]);
};

-- Gunboat movement allowance for the turn (5.24).
DEFINE FUNCTION OVERWRITE fn::gunboat_ma($g: record<unit>, $moves_upstream: bool) {
  RETURN IF $moves_upstream { $g.move_upstream } ELSE { $g.move_downstream };
};

-- D result: half the units in the target hex, rounded up, are disrupted.
DEFINE FUNCTION OVERWRITE fn::disrupted_count($units_in_hex: int) {
  RETURN <int> math::ceil(<float> $units_in_hex / 2);
};

-- Modified die rolls are clamped to 1..10 (back cover).
DEFINE FUNCTION OVERWRITE fn::clamp_roll($r: int) {
  RETURN math::clamp($r, constant:die_min.value, constant:die_max.value);
};

-- Fire die-roll modifier total. $o fields (all optional):
--   firer: faction, direct: bool, integrity: bool (all four of a brigade at one hex), target_in_fort: bool,
--   terrain: int (TEC defensive value, negative), across_thorn_hedge: bool, target_entrenched: bool.
-- Encodes 6.23, 6.24, 5.54, 6.54, 9.231, 9.232.
DEFINE FUNCTION OVERWRITE fn::fire_drm($o: object) {
  LET $ae = $o.firer = faction:anglo_egyptian;
  LET $d = $o.firer = faction:dervish;
  RETURN (IF $ae AND $o.direct = true { modifier:ae_direct_fire.value } ELSE { 0 })
       + (IF $ae AND $o.integrity = true { modifier:brigade_integrity.value } ELSE { 0 })
       + (IF $o.target_in_fort = true { modifier:fort_defense.value } ELSE { 0 })
       + ($o.terrain ?? 0)
       + (IF $d AND $o.across_thorn_hedge = true { modifier:thorn_hedge_fire.value } ELSE { 0 })
       + (IF $d AND $o.target_entrenched = true { modifier:trench_fire.value } ELSE { 0 });
};
DEFINE FUNCTION OVERWRITE fn::net_roll($die: int, $drm: int) {
  RETURN fn::clamp_roll($die + $drm);
};

-- Melee die-roll modifier (7.7, 6.52, 9.232). $s: faction, unit, unit_class or tribe.
DEFINE FUNCTION OVERWRITE fn::melee_mod($s: record, $attacking_entrenched: bool) {
  LET $tb = record::tb($s);
  LET $classes = IF $tb = "unit" { $s.classes } ELSE IF $tb = "unit_class" { array::concat([$s], $s.ancestors) } ELSE { [] };
  LET $faction = IF $tb = "faction" { $s } ELSE IF $tb = "unit_class" { $s.faction } ELSE { $s.faction };
  IF $classes CONTAINS unit_class:friendlies { RETURN modifier:dervish_melee.value; };
  IF $faction = faction:dervish { RETURN IF $attacking_entrenched { modifier:trench_melee.value } ELSE { modifier:dervish_melee.value }; };
  RETURN modifier:ae_melee.value;
};

-- Special targets (6.61-6.63, 10.23): does CRT result $r succeed against $target?
DEFINE FUNCTION OVERWRITE fn::special_hit($target: string, $r: int) {
  LET $t = (SELECT VALUE min_result FROM ONLY threshold WHERE target = $target LIMIT 1);
  RETURN $r >= $t;
};

-- Howitzer (6.64, 8.1).
DEFINE FUNCTION OVERWRITE fn::howitzer_range_ok($hexes: int) {
  RETURN $hexes >= constant:howitzer_min_range.value AND $hexes <= constant:howitzer_max_range.value;
};
DEFINE FUNCTION OVERWRITE fn::howitzer_impact($impact_roll: int) {
  RETURN (SELECT VALUE outcome FROM ONLY die_outcome WHERE table = "howitzer impact roll" AND min <= $impact_roll AND max >= $impact_roll LIMIT 1);
};

-- River mine blast (10.12).
DEFINE FUNCTION OVERWRITE fn::mine($roll: int) {
  RETURN (SELECT VALUE outcome FROM ONLY die_outcome WHERE table = "mine blast" AND min <= $roll AND max >= $roll LIMIT 1);
};

-- Desertion (8.2): units = 1½ × roll. Rounding of half units is unstated (open_question:desertion_rounding).
DEFINE FUNCTION OVERWRITE fn::desertion($roll: int) {
  LET $n = <float> $roll * constant:desertion_multiplier.value;
  RETURN { units: $n, whole: math::floor($n) = $n, rounding: IF math::floor($n) = $n { "none needed" } ELSE { "unstated" } };
};

-- Is a ZOC projected across terrain boundary $t in direction $dir ('into' | 'out_of')? (5.44)
DEFINE FUNCTION OVERWRITE fn::zoc_extends($t: record<terrain>, $dir: string) {
  LET $b = (SELECT * FROM ONLY zoc_boundary WHERE boundary = $t LIMIT 1);
  RETURN IF $b = NONE { NONE } ELSE IF $dir = "into" { $b.into } ELSE { $b.out_of };
};

-- Turn Record Track lookups.
DEFINE FUNCTION OVERWRITE fn::turn($n: int) {
  RETURN type::record("turn", $n).*;
};
DEFINE FUNCTION OVERWRITE fn::is_night($scenario: record<scenario>, $n: int) {
  IF $scenario.night_turns CONTAINS $n { RETURN true; };
  IF $scenario = scenario:fall_of_khartoum { RETURN NONE; };
  RETURN false;
} COMMENT 'NONE where the manual does not say (FALL OF KHARTOUM after turn 1)';

-- The order of one Game Turn's steps for a scenario (4, 6.4).
DEFINE FUNCTION OVERWRITE fn::sequence($scenario: record<scenario>) {
  LET $first = IF $scenario.moves_first = faction:anglo_egyptian { phase:ae_player_turn } ELSE { phase:d_player_turn };
  LET $second = IF $first = phase:ae_player_turn { phase:d_player_turn } ELSE { phase:ae_player_turn };
  LET $leaves = |$pt: record| (SELECT VALUE name FROM phase WHERE player_turn_phase = $pt ORDER BY step);
  RETURN array::flatten([$leaves($first), $leaves($second), [phase:end_of_game_turn.name]]);
};

-- Victory level for a side's point superiority, campaign (9.14).
DEFINE FUNCTION OVERWRITE fn::campaign_level($faction: record<faction>, $superiority: int) {
  RETURN (SELECT VALUE level FROM ONLY victory_level WHERE scenario = scenario:campaign AND faction = $faction AND min <= $superiority AND (max = NONE OR max >= $superiority) LIMIT 1);
};
-- Campaign result from both sides' VP totals.
DEFINE FUNCTION OVERWRITE fn::campaign_result($ae_vp: int, $d_vp: int) {
  LET $diff = $ae_vp - $d_vp;
  IF $diff = 0 { RETURN { superiority: 0, winner: NONE, level: NONE, note: "equal totals: no band of 9.14 covers zero superiority" }; };
  LET $w = IF $diff > 0 { faction:anglo_egyptian } ELSE { faction:dervish };
  RETURN { superiority: math::abs($diff), winner: $w.name, level: fn::campaign_level($w, math::abs($diff)) };
};

-- Historical scenario net victory (9.24).
DEFINE FUNCTION OVERWRITE fn::historical_rank($faction: record<faction>, $kills: int) {
  RETURN (SELECT VALUE rank FROM ONLY victory_level WHERE scenario = scenario:historical AND faction = $faction AND min <= $kills AND (max = NONE OR max >= $kills) LIMIT 1);
};
DEFINE FUNCTION OVERWRITE fn::historical_result($dervish_units_eliminated: int, $ae_units_eliminated: int) {
  LET $ae = fn::historical_rank(faction:anglo_egyptian, $dervish_units_eliminated);
  LET $d = fn::historical_rank(faction:dervish, $ae_units_eliminated);
  LET $net = math::abs($ae - $d);
  LET $name = (SELECT VALUE level FROM ONLY victory_level WHERE scenario = scenario:historical AND faction = faction:anglo_egyptian AND rank = $net LIMIT 1);
  RETURN { ae_level: $ae, dervish_level: $d, net: $net, net_level: $name,
           winner: IF $ae > $d { "Anglo-Egyptian" } ELSE IF $d > $ae { "Dervish" } ELSE { NONE } };
};

-- FALL OF KHARTOUM result (9.35). $gordon_eliminated_turn = NONE if GORDON survives turn 8.
DEFINE FUNCTION OVERWRITE fn::fok_result($gordon_eliminated_turn: option<int>, $dervish_units_lost: int) {
  LET $t = $gordon_eliminated_turn;
  LET $base = IF $t = NONE { 6 } ELSE IF $t <= 4 { 1 } ELSE IF $t = 5 { 2 } ELSE IF $t = 6 { 3 } ELSE IF $t = 7 { 4 } ELSE { 5 };
  LET $lost = IF $dervish_units_lost >= 32 { 3 } ELSE IF $dervish_units_lost >= 24 { 2 } ELSE IF $dervish_units_lost >= 16 { 1 } ELSE { 0 };
  LET $final = math::min([6, $base + $lost]);
  LET $lv = (SELECT * FROM ONLY victory_level WHERE scenario = scenario:fall_of_khartoum AND scale = $final LIMIT 1);
  RETURN { base_scale: $base, levels_lost: $lost, scale: $final,
           result: string::concat(IF $lv.faction = faction:dervish { "Dervish " } ELSE { "British " }, $lv.level) };
} COMMENT 'Gordon eliminated on turn 7 or 8 means he survived the end of turn 6 or 7 (implied by 9.35)';

-- Stacking legality of a hex (5.51, 5.52, 5.53). Items: unit records, tribe records (one unit of
-- that tribe) or unit_class records (one unit of that class).
-- Returns legal: true | false | NONE (undetermined: the manual does not settle a point; listed in
-- `undetermined`). Dervish leaders stack only with units of their command (5.53); another Dervish
-- leader is a unit (6.51), so leaders of different commands may not stack either.
DEFINE FUNCTION OVERWRITE fn::stack_check($items: array) {
  LET $info = $items.map(|$x| {
    LET $tb = record::tb($x);
    LET $classes = IF $tb = "unit" { $x.classes } ELSE IF $tb = "tribe" { array::concat([$x.unit_class], $x.unit_class.ancestors) } ELSE { array::concat([$x], $x.ancestors) };
    LET $tribe = IF $tb = "tribe" { $x } ELSE IF $tb = "unit" { $x.tribe } ELSE { NONE };
    LET $groups = IF $tb = "unit" { $x->leads->command_group } ELSE { [] };
    RETURN { item: $x, name: IF $tb = "unit_class" { $x.name } ELSE { $x.name }, leader: $classes CONTAINS unit_class:leader, gunboat: $classes CONTAINS unit_class:gunboat,
             fort: $classes CONTAINS unit_class:fort, dervish_leader: $classes CONTAINS unit_class:d_leader, tribe: $tribe, groups: $groups,
             group_unstated: array::len($groups.filter(|$g| $g.individual_assignment = "unstated")) > 0,
             commands: array::flatten($groups.map(|$g| $g<-member_of<-tribe)) };
  });
  LET $counted = $info.filter(|$i| !$i.leader AND !$i.gunboat);
  LET $counted_no_fort = $counted.filter(|$i| !$i.fort);
  LET $tribes = array::distinct($info.filter(|$i| $i.tribe != NONE AND !$i.leader).map(|$i| $i.tribe));
  LET $limit = constant:stacking_limit.value;
  LET $v1 = IF array::len($counted_no_fort) > $limit { ["more than four units in the hex (5.51)"] } ELSE { [] };
  LET $u1 = IF array::len($counted_no_fort) <= $limit AND array::len($counted) > $limit { ["whether a fort counts toward the four-unit limit is not stated (5.51)"] } ELSE { [] };
  LET $v2 = IF array::len($info.filter(|$i| $i.gunboat)) > 0 AND array::len($info) > 1 { ["gunboats may not stack with any other unit (5.51; exception 5.21)"] } ELSE { [] };
  LET $v3 = IF array::len($tribes) > 1 { ["units of different Dervish tribes may not stack together (5.52)"] } ELSE { [] };
  -- per Dervish leader: verdict on each stackmate
  LET $leader_checks = $info.filter(|$i| $i.dervish_leader).map(|$l| {
    LET $mates = $info.filter(|$o| $o.item != $l.item);
    LET $bad = $mates.filter(|$o|
      ($o.dervish_leader AND array::len($o.groups.filter(|$g| $l.groups CONTAINS $g)) = 0)
      OR (!$o.leader AND $o.tribe != NONE AND $l.commands CONTAINSNOT $o.tribe));
    LET $unsure = $mates.filter(|$o|
      (!$o.leader AND $o.tribe = NONE AND !$o.gunboat)
      OR ($l.group_unstated AND ((!$o.leader AND $o.tribe != NONE AND $l.commands CONTAINS $o.tribe)
                                  OR ($o.dervish_leader AND array::len($o.groups.filter(|$g| $l.groups CONTAINS $g)) > 0))));
    RETURN {
      v: IF array::len($bad) > 0 { string::concat($l.name, " may only stack with units of his command (5.53)") } ELSE { NONE },
      u: IF array::len($bad) = 0 AND array::len($unsure) > 0 { string::concat("the manual does not say whether ", array::join($unsure.map(|$o| $o.name), ", "), " belongs to ", $l.name, "'s command (5.53, 9.112)") } ELSE { NONE }
    };
  });
  LET $v4 = $leader_checks.map(|$c| $c.v).filter(|$s| $s != NONE);
  LET $u2 = $leader_checks.map(|$c| $c.u).filter(|$s| $s != NONE);
  LET $violations = array::flatten([$v1, $v2, $v3, $v4]);
  LET $undetermined = array::flatten([$u1, $u2]);
  RETURN { legal: IF array::len($violations) > 0 { false } ELSE IF array::len($undetermined) > 0 { NONE } ELSE { true },
           counted_units: array::len($counted), violations: $violations, undetermined: $undetermined };
};

-- Units a scenario brings in on a turn (9.112, 9.113, 9.322).
DEFINE FUNCTION OVERWRITE fn::arrivals($scenario: record<scenario>, $faction: record<faction>, $turn: int) {
  RETURN (SELECT VALUE text FROM reinforcement WHERE scenario = $scenario AND faction = $faction AND (turn = $turn OR turn_window CONTAINS $turn));
};

-- Derived structure: class closures, rule edges, section links. Runs after every data file.

-- transitive is_a closure of every unit class
FOR $c IN (SELECT VALUE id FROM unit_class) {
  UPDATE $c SET ancestors = array::distinct(array::flatten($c.{..+collect}(->is_a->unit_class)));
};
-- unit.classes = class + its ancestors; is_a edge unit -> class
FOR $u IN (SELECT id, class FROM unit) {
  UPDATE ($u.id) SET classes = array::concat([$u.class], $u.class.ancestors);
  RELATE ($u.id)->is_a->($u.class);
};
-- unit -> faction, class -> faction
FOR $u IN (SELECT id, faction FROM unit) { RELATE ($u.id)->belongs_to->($u.faction); };
FOR $c IN (SELECT id, faction FROM unit_class WHERE faction != NONE) { RELATE ($c.id)->belongs_to->($c.faction); };

-- rule edges from the rule arrays
FOR $r IN (SELECT id, actions, concerns, exception_to FROM rule) {
  FOR $a IN ($r.actions ?? []) { RELATE ($r.id)->governs->$a; };
  FOR $x IN ($r.concerns ?? []) { RELATE ($r.id)->concerns->$x; };
  FOR $e IN ($r.exception_to ?? []) { RELATE ($r.id)->exception_to->$e; };
};
-- Zariba rules apply in the campaign game only once constructed (5.3, 7.7)
UPDATE rule SET campaign_condition = "only if the Zariba is constructed (5.3)" WHERE id IN [rule:r9_231_thorn_hedge, rule:r9_232_trench, rule:r9_233_access];

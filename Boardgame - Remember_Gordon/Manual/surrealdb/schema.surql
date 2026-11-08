-- Remember Gordon! rulebook corpus -- schema.
--
-- Node tables hold the manual's nouns (units, places, terrain, phases, turns, scenarios, ...),
-- verbs (`action`) and mechanisms (`rule`, plus structured rule data: `modifier`, `threshold`,
-- `die_outcome`, `zoc_boundary`, `setup`, `reinforcement`, `vp_award`, `victory_level`, ...).
-- Edge tables relate them. Every rule-bearing record carries `section` (the manual paragraph
-- it comes from); `section.text` is the verbatim manual text.
--
-- Provenance field `basis` on facts and edges:
--   'explicit'  -- stated in the cited section.
--   'implied'   -- follows from the cited section(s) by direct reading; `note` says how.
--   'unstated'  -- deliberately recorded as unknown; see `open_question`.
-- Tables stay SCHEMALESS so records may carry rule-specific parameters; the fields declared
-- below are the ones every query may rely on. Every table grants `select` to everyone, so the
-- browser archive (web/) can seal the database as a read-only guest session.

DEFINE NAMESPACE IF NOT EXISTS rg;
USE NS rg;
DEFINE DATABASE IF NOT EXISTS rg;
USE DB rg;

-- ---------------------------------------------------------------- provenance
DEFINE TABLE section SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'One manual section/paragraph, verbatim text.';
DEFINE FIELD num ON section TYPE string;
DEFINE FIELD title ON section TYPE option<string>;
DEFINE FIELD text ON section TYPE string;
DEFINE FIELD kind ON section TYPE string;
DEFINE FIELD ord ON section TYPE int;
DEFINE FIELD parent ON section TYPE option<record<section>>;
DEFINE INDEX section_num ON section FIELDS num UNIQUE;

DEFINE TABLE cites TYPE RELATION FROM section TO section PERMISSIONS FOR select FULL COMMENT 'Section text cross-references another section ("see 6.64").';

-- ---------------------------------------------------------------- nouns
DEFINE TABLE faction SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'The two sides.';
DEFINE TABLE unit_class SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Unit type taxonomy (leaf and abstract classes); is_a edges form the hierarchy.';
DEFINE FIELD name ON unit_class TYPE string;
DEFINE FIELD abstract ON unit_class TYPE bool DEFAULT false;
DEFINE FIELD faction ON unit_class TYPE option<record<faction>>;
DEFINE FIELD ancestors ON unit_class TYPE array<record<unit_class>> DEFAULT [] COMMENT 'Denormalized transitive is_a closure (filled at load).';

DEFINE TABLE unit SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Individually identified counters or counter kinds named in the manual.';
DEFINE FIELD name ON unit TYPE string;
DEFINE FIELD faction ON unit TYPE record<faction>;
DEFINE FIELD class ON unit TYPE record<unit_class>;
DEFINE FIELD classes ON unit TYPE array<record<unit_class>> DEFAULT [] COMMENT 'class + its ancestors (filled at load).';

DEFINE TABLE tribe SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Dervish tribe: stacking (5.52), command (5.53), setup and reinforcement unit.';
DEFINE TABLE formation SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Anglo-Egyptian brigade / division.';
DEFINE TABLE command_group SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Dervish leader(s) and the tribe(s) of their command (5.53, 9.112, 9.212).';
DEFINE TABLE weapon SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Armament named by the manual.';
DEFINE TABLE range_table SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'The two Range Effects Tables.';
DEFINE TABLE range_line SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'A weapon line of a Range Effects Table.';
DEFINE TABLE terrain SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Terrain types, hex or hexside.';
DEFINE TABLE location SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Named places, areas, edges and hex groups on the two maps.';
DEFINE TABLE map SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'The two mapsheets.';
DEFINE TABLE component SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Game parts and play aids (charts, tables, die).';
DEFINE TABLE marker SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Counters that are not units: turn marker, breach marker, blank counter.';
DEFINE TABLE state SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Conditions a unit/stack/side can be in (disrupted, entrenched, ...).';
DEFINE TABLE combat_result SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Combat Results Table result kinds.';
DEFINE TABLE constant SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Named numeric values of the rules (scale, limits, ranges).';
DEFINE TABLE person SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Historical persons named in the manual (section 11 and counters).';
DEFINE TABLE historical_event SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Dated events of the historical notes.';

-- ---------------------------------------------------------------- time
DEFINE TABLE phase SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Turn sequence tree: player turn > phase > fire step > subphase.';
DEFINE FIELD name ON phase TYPE string;
DEFINE FIELD ord ON phase TYPE int COMMENT 'Order among siblings.';
DEFINE FIELD parent ON phase TYPE option<record<phase>>;
DEFINE TABLE turn SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Turn Record Track box (campaign game) / scenario turn.';
DEFINE TABLE scenario SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Campaign game, historical scenario, Fall of Khartoum.';

-- ---------------------------------------------------------------- verbs and mechanisms
DEFINE TABLE action SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Game actions (verbs).';
DEFINE TABLE rule SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Atomic rule statement of one section.';
DEFINE FIELD section ON rule TYPE record<section>;
DEFINE FIELD statement ON rule TYPE string;
DEFINE FIELD kind ON rule TYPE string ASSERT $value IN ['definition','permission','prohibition','obligation','procedure','modifier','threshold','effect','condition','recommendation','scope','sequence','limit'];
DEFINE FIELD scenarios ON rule TYPE option<array<record<scenario>>> COMMENT 'NONE = every scenario.';
DEFINE TABLE modifier SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Die-roll / movement modifiers.';
DEFINE TABLE threshold SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Minimum Combat Results Table result for a special target.';
DEFINE TABLE die_outcome SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Die-roll bands with an outcome (mines, howitzer impact, desertion).';
DEFINE TABLE zoc_boundary SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Whether a ZOC extends out of / into across a terrain boundary (5.44).';
DEFINE TABLE setup SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Scenario set-up instruction.';
DEFINE TABLE reinforcement SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Scenario reinforcement / entry instruction.';
DEFINE TABLE vp_award SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Victory point award.';
DEFINE TABLE victory_level SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Victory level band.';
DEFINE TABLE victory_condition SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Non-point victory conditions.';
DEFINE TABLE open_question SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'What the manual leaves open, or where it is inconsistent with itself.';

-- ---------------------------------------------------------------- benchmark
DEFINE TABLE question SCHEMALESS PERMISSIONS FOR select FULL COMMENT 'Player question, its SurrealQL formulation and expected answer.';

-- ---------------------------------------------------------------- edges
DEFINE TABLE is_a TYPE RELATION PERMISSIONS FOR select FULL COMMENT 'unit_class -> unit_class (taxonomy); unit -> unit_class.';
DEFINE TABLE belongs_to TYPE RELATION PERMISSIONS FOR select FULL COMMENT 'unit/tribe/formation/class -> faction, unit -> formation/tribe, formation -> formation.';
DEFINE TABLE leads TYPE RELATION FROM unit TO command_group PERMISSIONS FOR select FULL;
DEFINE TABLE member_of TYPE RELATION PERMISSIONS FOR select FULL COMMENT 'tribe -> command_group; location -> location (part of).';
DEFINE TABLE fires_on TYPE RELATION TO range_line PERMISSIONS FOR select FULL COMMENT 'unit/unit_class/tribe -> range_line.';
DEFINE TABLE capability TYPE RELATION TO action PERMISSIONS FOR select FULL COMMENT 'unit/unit_class/tribe -> action, allowed true/false, with rule provenance.';
DEFINE TABLE entry TYPE RELATION PERMISSIONS FOR select FULL COMMENT 'unit/unit_class/tribe -> location|terrain: may (not) enter.';
DEFINE TABLE governs TYPE RELATION FROM rule TO action PERMISSIONS FOR select FULL COMMENT 'Rule constrains/defines this action (from rule.actions).';
DEFINE TABLE concerns TYPE RELATION FROM rule PERMISSIONS FOR select FULL COMMENT 'Rule is about this entity (from rule.concerns).';
DEFINE TABLE exception_to TYPE RELATION FROM rule TO rule PERMISSIONS FOR select FULL COMMENT 'Rule is an exception to another rule (from rule.exception_to).';
DEFINE TABLE allowed_in TYPE RELATION FROM action TO phase PERMISSIONS FOR select FULL COMMENT 'Action is performed in this phase.';
DEFINE TABLE prohibits TYPE RELATION FROM state TO action PERMISSIONS FOR select FULL COMMENT 'Units in this state may not do the action.';
DEFINE TABLE next TYPE RELATION PERMISSIONS FOR select FULL COMMENT 'turn -> turn, phase -> phase sequence.';
DEFINE TABLE about TYPE RELATION FROM question PERMISSIONS FOR select FULL COMMENT 'question -> section / entity it is about.';
DEFINE TABLE raises TYPE RELATION FROM open_question PERMISSIONS FOR select FULL COMMENT 'open_question -> section/rule/entity.';
DEFINE TABLE appears_in TYPE RELATION PERMISSIONS FOR select FULL COMMENT 'unit/tribe/person -> scenario|historical_event.';

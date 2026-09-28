//! [`GameState`] and its core accessors. The read-only validators are split
//! by domain into the `state/` submodules (setup, movement, fire, melee,
//! engineering, stacking/ZOC), each adding its own `impl GameState` block.

use super::*;

mod engineering;
mod fire;
mod melee;
mod movement;
mod setup;
mod stacking;

pub use movement::{MAX_MOVE_PATH_LEN, MovePlan};
pub use setup::{
    FokCapGroup, HISTORICAL_KERRERI_UNITS, MAX_CHAIN_HEXES, MAX_MINES, fok_cap_group,
    historical_counter_in_play, historical_in_play,
};
pub(crate) use stacking::STACKING_LIMIT;
pub use stacking::{stacking_rule, unit_projects_zoc_rule};

/// All mutable state of a game in progress (rulebook §4).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    pub scenario: Scenario,
    pub current_turn: GameTurnIndex,
    pub day_night: DayNight,
    pub active_player: Player,
    pub phase: Phase,
    pub units: Vec<UnitPlacement>,
    pub victory: VictoryLedger,
    /// Index into [`UnitId::ALL`] for the next auto-assigned ID.
    /// Used only by test helpers -- production code uses
    /// [`unit_id_for_section_pos`][crate::unit_id_for_section_pos] instead.
    /// Skipped from serialisation so it never leaks into a saved game or a
    /// replay record.
    #[serde(skip)]
    pub next_alloc_index: usize,
    pub units_fired_this_phase: Vec<UnitId>,
    /// Units that have been fired at this fire phase (§6.14: "a combat unit
    /// may only fire once and may only be fired at once"). Exceptions per
    /// §6.14 parenthetical: Maxim guns and gunboats. Cleared with `units_fired_this_phase`
    /// at each phase change and turn end.
    #[serde(default)]
    pub units_fired_at_this_phase: Vec<UnitId>,
    /// Movement points each unit has spent this turn (§5.11/§5.12). A unit may
    /// move hex by hex up to its (night-adjusted) allowance, so the cumulative
    /// spend -- not a binary "moved" flag -- is what caps further movement.
    /// "Has this unit moved at all?" is derived as `mp_spent(id) > 0`
    /// (used by retreat-before-melee, §7.5). Cleared each turn (§5.13: MP
    /// never carry over).
    #[serde(default)]
    pub mp_spent_this_turn: BTreeMap<UnitId, i16>,
    /// Gunboats that have moved at least one hex upstream this turn (§5.24:
    /// "if they move even one hex upstream, their upstream movement allowance
    /// is their maximum movement allowance for that turn"). The cap is
    /// *sticky* for the rest of the turn -- a later all-downstream move must
    /// still be capped at the upstream allowance. Set when a gunboat move is
    /// applied; cleared in `clear_per_turn_tracking`.
    #[serde(default)]
    pub gunboats_upstream_this_turn: Vec<UnitId>,
    /// Units that entered an enemy zone of control this movement phase
    /// (§5.26/§5.43: "All units must stop when they enter an enemy ZOC and may
    /// move no further that turn"). A listed unit may not move again until
    /// its next movement phase ("In their next movement phase they may
    /// withdraw"). Cleared in `clear_per_turn_tracking`.
    #[serde(default)]
    pub zoc_stopped_this_turn: Vec<UnitId>,
    /// Hexes vacated by combat this phase, mapping each to the surviving
    /// participants (attackers/firers) that may advance into it (§6.82, §7.5,
    /// §7.6). An advance-after-combat is legal only into a keyed hex and only
    /// for a listed unit -- the manual's participation requirement. Windows
    /// open when offensive fire, melee, or a retreat-before-melee vacates a
    /// hex, and close on the next phase change (except the Direct→Maxim/
    /// Howitzer subphase bridge, §6.42) and at end of turn.
    #[serde(default)]
    pub vacated_by_combat: BTreeMap<HexCoord, Vec<UnitId>>,
    /// Reinforcements placed onto the board this player-turn (§9.112/§9.113),
    /// used to enforce the per-turn unit and gunboat quotas against
    /// cumulative batches. Cleared at end of turn.
    #[serde(default)]
    pub reinforcements_placed_this_turn: Vec<(Player, UnitId)>,
    /// Every unit eliminated so far, in order. A destroyed counter never
    /// returns to play: reinforcement entry and deployment refuse it (it is
    /// off the board like a not-yet-arrived unit, so board presence alone
    /// cannot tell the two apart).
    #[serde(default)]
    pub eliminated: Vec<UnitId>,
    pub game_over: bool,
    pub zariba_hexsides: Vec<HexsideRef>,
    /// The active "Friendlies" transport mission (§5.21), if any. Single-mission
    /// at a time: the manual is ambiguous on whether multiple concurrent
    /// transports are allowed; we model one mission for simplicity.
    /// `None` when no transport is in progress (or after disembarkation).
    #[serde(default)]
    pub friendlies_transport: Option<TransportState>,
    pub optional_rules: Vec<OptionalRule>,
    pub mines: Vec<MinePlacement>,
    pub chain: Option<ChainPlacement>,
    /// Static per-board map facts (hexsides, terrain, Nile current, landmarks)
    /// the engine consults to enforce map-dependent rules (§5.11, §5.24, §5.44,
    /// §6.6x, §9.14, §10). Empty until the app attaches the active board at game
    /// start; an empty board makes every map lookup rule-neutral.
    ///
    /// `Arc` (read-only sharing) because the board is static for the whole
    /// game: `GameState::clone` -- the workhorse of clone-and-try validation --
    /// would otherwise deep-copy every hex, hexside and entrance on each
    /// candidate. The one game-time board change, a wall breach (§6.63), is
    /// *not* a board mutation: it is recorded in [`Self::breaches`] and read
    /// through [`Self::hexside_effective`]. Serialises transparently as
    /// `BoardInfo`.
    #[serde(default)]
    pub board: Arc<BoardInfo>,
    /// Wall hexsides breached during play (§6.53 demolition, §6.63 artillery
    /// breach). The authored board is static; a breach is game state, so it
    /// replays, serialises and clones with the rest of `GameState`. Every
    /// game-time wall check reads the board through
    /// [`Self::hexside_effective`], which applies this set.
    #[serde(default)]
    pub breaches: BTreeSet<HexsideRef>,
    /// Whether the once-per-game Dervish desertion roll has already happened
    /// (§8.2). Prevents re-applying the desertion effect.
    #[serde(default)]
    pub dervish_deserted: bool,
    /// A melee that has been *declared* but not yet resolved (§7.5): while it
    /// is pending, the defender's cavalry/camel may retreat before resolution.
    /// `None` outside a declaration window.
    pub pending_melee: Option<PendingMelee>,
    /// The turn on which GORDON was eliminated in FALL OF KHARTOUM (§9.346),
    /// which fixes the Dervish victory level (§9.35). `None` while he survives.
    #[serde(default)]
    pub gordon_eliminated_turn: Option<GameTurnIndex>,
    /// Setup-phase readiness per faction (§9.2/§9.3). Setup is sequential
    /// (`GameState::require_setup_turn`): the first side confirms, then the
    /// second deploys and confirms; the game leaves [`Phase::Setup`] only once
    /// *both* are ready (and `setup_complete` holds). One-way: once set, a faction stays ready. `#[serde(default)]`
    /// (false) so pre-setup records/snapshots load unchanged.
    #[serde(default)]
    pub setup_ready_ae: bool,
    #[serde(default)]
    pub setup_ready_dervish: bool,
    /// Whether the Isa Zachneih unit has been eliminated. Unlocks the §5.21
    /// "Friendlies" transport (the unit may only load after Isa Zachneih dies).
    #[serde(default)]
    pub isa_zachneih_eliminated: bool,
    /// Pending Royal Engineers demolitions (§6.53): each entry is an engineer
    /// that began a demolition this turn and must be resolved at end of turn
    /// (still adjacent + undisrupted → target destroyed; otherwise cancelled).
    #[serde(default)]
    pub pending_demolitions: Vec<(UnitId, DemolitionTarget)>,
    /// Side-channel signals emitted by `apply_effect` (demolition results,
    /// leader deaths, VP awards, etc.).  Drained by the app after each effect
    /// application and translated into Bevy events.  Serialized so replay /
    /// late-join produces the same stream.
    #[serde(default)]
    pub observations: Vec<Observation>,
    /// Structured events accumulated during the current game turn.
    /// Cleared when the turn advances (snapshotted into `turn_summaries`).
    #[serde(default)]
    pub turn_events: Vec<crate::turn_summary::TurnEventRecord>,
    /// Append-only history of completed turn summaries.
    #[serde(default)]
    pub turn_summaries: Vec<crate::turn_summary::TurnSummary>,
    /// Typed game result, set by [`finish_game`] once the scenario ends.
    /// Used by the app layer to look up newspaper templates.
    #[serde(default)]
    pub game_result: Option<crate::GameResult>,
}

/// A declared-but-unresolved melee attack, with its pre-rolled dice held so
/// resolution after the reaction window is deterministic and host-ordered (rulebook §7.5).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PendingMelee {
    pub attack: MeleeAttack,
    pub attacker_roll: DieRoll,
    pub defender_roll: DieRoll,
}

impl GameState {
    /// Create a fresh game state for a given scenario (rulebook §4).
    pub fn new(scenario: Scenario) -> Self {
        let first = scenario_turn(scenario, GameTurnIndex::new(1));
        // First player to *move* per scenario: Campaign -- Anglo-Egyptian moves
        // first (§9.113); Historical -- Dervish moves first (§9.212); Fall of
        // Khartoum -- Dervish moves first (§9.322).
        let active = match scenario {
            Scenario::Campaign => Player::AngloEgyptian,
            Scenario::Historical | Scenario::FallOfKhartoum => Player::Dervish,
        };
        let day_night = first.map_or(DayNight::Day, |t| t.day_night);
        GameState {
            scenario,
            current_turn: GameTurnIndex::new(1),
            day_night,
            active_player: active,
            // Every scenario opens in deployment; `advance_phase` leaves Setup
            // for the first player's Movement turn once `setup_complete` holds
            // (§9.2/§9.3/§10).
            phase: Phase::Setup,
            units: Vec::new(),
            victory: VictoryLedger::default(),
            next_alloc_index: 0,
            units_fired_this_phase: Vec::new(),
            units_fired_at_this_phase: Vec::new(),
            mp_spent_this_turn: BTreeMap::new(),
            gunboats_upstream_this_turn: Vec::new(),
            zoc_stopped_this_turn: Vec::new(),
            vacated_by_combat: BTreeMap::new(),
            reinforcements_placed_this_turn: Vec::new(),
            eliminated: Vec::new(),
            game_over: false,
            zariba_hexsides: Vec::new(),
            friendlies_transport: None,
            optional_rules: Vec::new(),
            mines: Vec::new(),
            chain: None,
            board: Arc::new(BoardInfo::default()),
            breaches: BTreeSet::new(),
            dervish_deserted: false,
            pending_melee: None,
            gordon_eliminated_turn: None,
            setup_ready_ae: false,
            setup_ready_dervish: false,
            isa_zachneih_eliminated: false,
            pending_demolitions: Vec::new(),
            observations: Vec::new(),
            turn_events: Vec::new(),
            turn_summaries: Vec::new(),
            game_result: None,
        }
    }

    /// Create a fresh game state with the active board's map facts attached
    /// (rulebook §5.11, §5.24, §5.44). The app builds [`BoardInfo`] from the
    /// loaded annotations at game start so map-dependent rules can be enforced.
    pub fn with_board(scenario: Scenario, board: BoardInfo) -> Self {
        let mut state = Self::new(scenario);
        state.board = Arc::new(board);
        state
    }

    /// The player who is to act in the current phase.
    ///
    /// During Movement, Offensive Fire and Melee this is the moving player
    /// (`active_player`); during Defensive Fire the *non-moving* side fires
    /// back first (rules 6.4/6.7, the fire-combat sequence), so control passes
    /// to `active_player.opponent()` while the mover's turn continues.
    pub fn phase_player(&self) -> Player {
        match self.phase {
            Phase::DefensiveFire(_) => self.active_player.opponent(),
            _ => self.active_player,
        }
    }

    /// The effective hexside between `a` and `b`: the authored kind, with
    /// §6.53/§6.63 breaches overriding an authored Wall and §5.3/§9.231
    /// constructed zariba filling an otherwise-empty hexside. *Every*
    /// game-time hexside check must read through this (movement §5.23, melee
    /// §7.2, ZOC §5.41, advance/retreat §6.82/§7.6, fire-at-wall §6.63) so a
    /// breach is an opening and a constructed zariba is a thorn hedge
    /// everywhere at once; static map derivation (walled-city footprint
    /// §5.23, LOS levels §6.3 note b) stays on the authored board.
    pub fn hexside_effective(&self, a: HexCoord, b: HexCoord) -> Option<HexsideKind> {
        match self.board.hexside_between(a, b) {
            Some(HexsideKind::Wall) if self.wall_is_breached(a, b) => Some(HexsideKind::Breach),
            Some(authored) => Some(authored),
            None if self.zariba_hexsides.contains(&HexsideRef::new(a, b)) => {
                Some(HexsideKind::ZaribaThornHedge)
            }
            None => None,
        }
    }

    /// [`Self::hexside_effective`] under a predicate.
    pub fn hexside_effective_is(
        &self,
        a: HexCoord,
        b: HexCoord,
        pred: impl Fn(HexsideKind) -> bool,
    ) -> bool {
        self.hexside_effective(a, b).is_some_and(pred)
    }

    /// Whether the wall hexside between `a` and `b` has been breached
    /// (§6.53/§6.63). Passed into the LOS machinery so a breach is an
    /// opening for line of sight too.
    pub fn wall_is_breached(&self, a: HexCoord, b: HexCoord) -> bool {
        self.breaches.contains(&HexsideRef::new(a, b))
    }

    /// Record a breach of the wall hexside between `a` and `b` (§6.53/§6.63).
    /// Only authored Walls can be breached; the board itself is never
    /// mutated.
    pub fn breach_wall(&mut self, a: HexCoord, b: HexCoord) {
        if self.board.hexside_between(a, b) == Some(HexsideKind::Wall) {
            self.breaches.insert(HexsideRef::new(a, b));
        }
    }

    /// Whether `hex` is "entrenched" (§9.232: "directly adjacent to (and on
    /// the Nile River side of) a trench hexside"): a Zariba hex with a trench
    /// hexside on its perimeter. The Nile side is the Zariba's own side --
    /// the hex across the trench is the open desert, never the river. Reads
    /// *effective* hexsides.
    pub fn is_zariba_entrenched(&self, hex: HexCoord) -> bool {
        use omdurman_types::HexsideKind::{ZaribaTrench, ZaribaTrenchEndA, ZaribaTrenchEndB};
        self.board.is_zariba(hex)
            && hex.neighbors().into_iter().any(|n| {
                self.hexside_effective_is(hex, n, |k| {
                    matches!(k, ZaribaTrench | ZaribaTrenchEndA | ZaribaTrenchEndB)
                })
            })
    }

    /// Whether `hex` has a zariba thorn hedge on its perimeter (§9.231: −2
    /// fire modifier against units in a zariba-defended hex). Reads
    /// *effective* hexsides, so constructed (§5.3) and authored hedges both
    /// count.
    pub fn has_zariba_thorn_hedge(&self, hex: HexCoord) -> bool {
        for n in hex.neighbors() {
            if self.hexside_effective_is(hex, n, |k| k == HexsideKind::ZaribaThornHedge) {
                return true;
            }
        }
        false
    }

    /// The +2 MP cost of crossing a Zariba end hexside (§9.233: "Units may
    /// only enter and/or leave the Zariba via the two end hexsides ... paying
    /// +2 movement points to cross"). Trench ends are authored FoK geography
    /// (players cannot construct them), so this reads the authored board.
    pub fn zariba_entry_surcharge(&self, from: HexCoord, to: HexCoord) -> i16 {
        match self.board.hexside_between(from, to) {
            Some(k) if k.is_zariba_trench_end() => 2,
            _ => 0,
        }
    }

    /// The mine in `hex`, if any (§10.11). The world lens for river rules:
    /// mines are game state with a lifecycle (`triggered`), not map data.
    pub fn mine_at(&self, hex: HexCoord) -> Option<&MinePlacement> {
        self.mines.iter().find(|m| m.hex == hex)
    }

    /// Whether the (unsunk) river chain spans `hex` (§10.21/§10.22).
    pub fn chain_covers(&self, hex: HexCoord) -> bool {
        self.chain
            .as_ref()
            .is_some_and(|c| !c.sunk && c.hexes.contains(&hex))
    }

    /// Find a unit by ID (rulebook §4).
    pub fn find_unit(&self, id: UnitId) -> Option<&UnitPlacement> {
        self.units.iter().find(|u| u.id == id)
    }

    /// Look up a unit by ID, returning [`RuleError::UnitNotFound`] on miss.
    /// Convenience used by the `can_*` predicates so they open with a one-liner.
    pub(crate) fn unit_or_err(&self, id: UnitId) -> Result<&UnitPlacement, RuleError> {
        self.find_unit(id).ok_or(RuleError::UnitNotFound(id))
    }

    /// Verify the active Friendlies transport mission matches the expected
    /// state for the unit+gunboat pair (§5.21). `matching` selects the variant
    /// (Loaded / Crossing / ...) and unit/gunboat identity; `err` is returned
    /// when no mission is in progress or the predicate fails. Used by the
    /// Crossing and Disembark arms of `apply_friendlies_transport`.
    pub(crate) fn require_transport_state(
        &self,
        matching: impl FnOnce(&TransportState) -> bool,
        err: RuleError,
    ) -> Result<(), RuleError> {
        match &self.friendlies_transport {
            Some(state) if matching(state) => Ok(()),
            _ => Err(err),
        }
    }

    /// Mutable lookup by ID (rulebook §4).
    pub fn find_unit_mut(&mut self, id: UnitId) -> Option<&mut UnitPlacement> {
        self.units.iter_mut().find(|u| u.id == id)
    }

    /// All units in a given hex (rulebook §5).
    pub fn units_in_hex(&self, hex: HexCoord) -> Vec<&UnitPlacement> {
        self.units.iter().filter(|u| u.position == hex).collect()
    }

    /// Movement points `unit_id` has already spent this turn (§5.11/§5.12).
    pub fn mp_spent(&self, unit_id: UnitId) -> i16 {
        self.mp_spent_this_turn.get(&unit_id).copied().unwrap_or(0)
    }

    /// Drain and return all pending [`Observation`]s pushed by `apply_effect`
    /// since the last call.  The app calls this after each effect application
    /// and translates the result into Bevy events.
    pub fn drain_observations(&mut self) -> Vec<Observation> {
        std::mem::take(&mut self.observations)
    }

    /// Whether moving from `from` to `to` is the §9.345 off-board crossing
    /// between the two Nile-branch mouths (in either direction). Both mouths
    /// must be named on the board, else this is `false` and the move falls
    /// through to the ordinary contiguous-Nile rules.
    pub fn is_nile_mouth_crossing(&self, from: HexCoord, to: HexCoord) -> bool {
        let white = self
            .board
            .hex_of_location(omdurman_types::Location::WhiteNileMouth);
        let blue = self
            .board
            .hex_of_location(omdurman_types::Location::BlueNileMouth);
        match (white, blue) {
            (Some(w), Some(b)) => (from == w && to == b) || (from == b && to == w),
            _ => false,
        }
    }

    /// Whether `hex` holds a fort owned by `mover`'s enemy. Per §6.54 a player
    /// may neither occupy an enemy fort nor advance after combat into one
    /// (forts are never captured -- only destroyed, §6.62/§6.53/§7.6).
    ///
    /// Every fort is a counter: FALL OF KHARTOUM's North Fort (§9.344) and
    /// the British Forts Makran and Buri (§9.321) included. A destroyed fort
    /// leaves only the printed building behind.
    pub fn hex_has_enemy_fort(&self, hex: HexCoord, mover: Player) -> bool {
        self.units.iter().any(|u| {
            u.position == hex
                && matches!(u.profile.kind, UnitKind::Fort { .. })
                && u.profile.identity.owner() != mover
        })
    }

    /// All units of a given player in a hex (rulebook §5).
    pub fn player_units_in_hex(&self, hex: HexCoord, player: Player) -> Vec<&UnitPlacement> {
        self.units
            .iter()
            .filter(|u| u.position == hex && u.profile.identity.owner() == player)
            .collect()
    }

    /// Produce the next UnitId from [`UnitId::ALL`] (rulebook §4).
    /// Used internally by test helpers; production code should call
    /// [`unit_id_for_section_pos`][crate::unit_id_for_section_pos] instead.
    pub fn alloc_unit_id(&mut self) -> UnitId {
        let id = UnitId::ALL[self.next_alloc_index];
        self.next_alloc_index += 1;
        id
    }
}

/// A minimal, roster-free [`GameState`] for Kani harnesses whose property does
/// not depend on the scenario order of battle. `GameState::new` symexes the
/// full campaign roster, which on its own dominates CBMC's memory (the
/// `score_elimination_records_exactly_what_it_scores` harness OOMs on it);
/// harnesses that only need "a state with empty ledgers" use this instead.
///
/// Deliberately `cfg(kani)`-gated and field-complete: adding a `GameState`
/// field breaks the proof build here loudly instead of drifting silently.
#[cfg(kani)]
impl GameState {
    pub(crate) fn kani_minimal() -> Self {
        GameState {
            scenario: Scenario::Campaign,
            current_turn: GameTurnIndex::new(1),
            day_night: DayNight::Day,
            active_player: Player::AngloEgyptian,
            phase: Phase::Setup,
            units: Vec::new(),
            victory: VictoryLedger::default(),
            next_alloc_index: 0,
            units_fired_this_phase: Vec::new(),
            units_fired_at_this_phase: Vec::new(),
            mp_spent_this_turn: BTreeMap::new(),
            gunboats_upstream_this_turn: Vec::new(),
            zoc_stopped_this_turn: Vec::new(),
            vacated_by_combat: BTreeMap::new(),
            reinforcements_placed_this_turn: Vec::new(),
            eliminated: Vec::new(),
            game_over: false,
            zariba_hexsides: Vec::new(),
            friendlies_transport: None,
            optional_rules: Vec::new(),
            mines: Vec::new(),
            chain: None,
            board: Arc::new(BoardInfo::default()),
            breaches: BTreeSet::new(),
            dervish_deserted: false,
            pending_melee: None,
            gordon_eliminated_turn: None,
            setup_ready_ae: false,
            setup_ready_dervish: false,
            isa_zachneih_eliminated: false,
            pending_demolitions: Vec::new(),
            observations: Vec::new(),
            turn_events: Vec::new(),
            turn_summaries: Vec::new(),
            game_result: None,
        }
    }
}

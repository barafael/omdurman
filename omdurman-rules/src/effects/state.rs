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

pub use movement::{MAX_MOVE_PATH_LEN, MovePlan, NILE_MOUTH_CROSSING_MP};
pub use setup::{
    FokCapGroup, HISTORICAL_KERRERI_UNITS, MAX_CHAIN_HEXES, MAX_MINES, campaign_counter_in_play,
    entrance_area_for, fok_cap_group, historical_counter_in_play, historical_in_play,
    in_campaign_initial_force,
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
    /// may only fire once and may only be fired at once" -- its Maxim and
    /// gunboat exceptions are to firing once, §6.42). Cleared with
    /// `units_fired_this_phase` at each phase change and turn end.
    #[serde(default)]
    pub units_fired_at_this_phase: Vec<UnitId>,
    /// Units a howitzer shell has struck in the hex it was aimed at this
    /// fire subphase (§6.42: "Howitzer fire may be combined with Maxim
    /// fire, but only if the howitzer fire impacts in the intended hex").
    /// Kept apart from `units_fired_at_this_phase` so the shell and the
    /// Maxims' second fire may both land on one hex; a second shell may
    /// not. Cleared with it.
    #[serde(default)]
    pub units_shelled_this_phase: Vec<UnitId>,
    /// Named gunboats whose Maxim guns have fired this fire subphase (§6.42:
    /// they fire once in each subphase, independently of the gunboat's
    /// artillery, which `units_fired_this_phase` tracks). Cleared with it.
    #[serde(default)]
    pub gunboat_maxims_fired_this_phase: Vec<UnitId>,
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
    /// Units that have made their melee attack this turn (§7.5: "enemy units
    /// whose melee attacks have not yet been resolved" -- each unit attacks
    /// once per melee phase). Set when the melee is declared; cleared in
    /// `clear_per_turn_tracking`.
    #[serde(default)]
    pub units_meleed_this_turn: Vec<UnitId>,
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
    /// The British gunboat stopped on a mine this turn, until the Dervish
    /// player rolls for it (§10.12). No phase ends while it is pending.
    #[serde(default)]
    pub pending_mine: Option<crate::StruckMine>,
    /// Gunboats ordered to stop for the rest of the turn -- on a mine
    /// (§10.12). Cleared in `clear_per_turn_tracking`.
    #[serde(default)]
    pub gunboats_stopped_this_turn: Vec<UnitId>,
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
    /// Whether an Anglo-Egyptian unit other than a gunboat has been
    /// eliminated on the west bank. The §9.14 Dervish alternative decisive
    /// victory ("eliminates all Anglo-Egyptian units on the west bank") is
    /// only judged once there were such units to eliminate: with none ever
    /// lost there, an empty west bank means none entered, not that all fell.
    #[serde(default)]
    pub ae_lost_on_west_bank: bool,
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
    /// Which units each side's `D` result disrupts (§CombatResults).
    pub disruption: DisruptionDraw,
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
            gunboat_maxims_fired_this_phase: Vec::new(),
            units_fired_at_this_phase: Vec::new(),
            units_shelled_this_phase: Vec::new(),
            mp_spent_this_turn: BTreeMap::new(),
            gunboats_upstream_this_turn: Vec::new(),
            zoc_stopped_this_turn: Vec::new(),
            units_meleed_this_turn: Vec::new(),
            vacated_by_combat: BTreeMap::new(),
            reinforcements_placed_this_turn: Vec::new(),
            eliminated: Vec::new(),
            game_over: false,
            zariba_hexsides: Vec::new(),
            friendlies_transport: None,
            optional_rules: Vec::new(),
            mines: Vec::new(),
            chain: None,
            pending_mine: None,
            gunboats_stopped_this_turn: Vec::new(),
            board: Arc::new(BoardInfo::default()),
            breaches: BTreeSet::new(),
            dervish_deserted: false,
            pending_melee: None,
            gordon_eliminated_turn: None,
            setup_ready_ae: false,
            setup_ready_dervish: false,
            isa_zachneih_eliminated: false,
            ae_lost_on_west_bank: false,
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
    /// §6.53/§6.63 breaches overriding an authored Wall, and the printed
    /// Zariba present in the Campaign only where constructed (§2.1: "the
    /// hexsides of the Zariba exist only in the historical scenario and
    /// should be considered clear terrain in the campaign game"; §5.3: they
    /// "may only be built in their position as displayed on the mapsheet").
    /// *Every* game-time hexside check must read through this (movement
    /// §5.23, melee §7.2, ZOC §5.41, advance/retreat §6.82/§7.6, fire §6.63,
    /// §9.23) so a breach is an opening and an unbuilt Zariba is clear
    /// ground everywhere at once; static map derivation (walled-city
    /// footprint §5.23, LOS levels §6.3 note b) stays on the authored board.
    pub fn hexside_effective(&self, a: HexCoord, b: HexCoord) -> Option<HexsideKind> {
        match self.board.hexside_between(a, b) {
            Some(HexsideKind::Wall) if self.wall_is_breached(a, b) => Some(HexsideKind::Breach),
            Some(side)
                if side.is_zariba()
                    && self.scenario == Scenario::Campaign
                    && !self.zariba_hexsides.contains(&HexsideRef::new(a, b)) =>
            {
                None
            }
            authored => authored,
        }
    }

    /// Whether the hexside between `a` and `b` is one of the Zariba's as
    /// *printed* on the mapsheet (§5.3: the Zariba "may only be built in its
    /// position as displayed on the mapsheet"), built or not.
    pub fn is_printed_zariba_side(&self, a: HexCoord, b: HexCoord) -> bool {
        self.board
            .hexside_between(a, b)
            .is_some_and(HexsideKind::is_zariba)
    }

    /// Whether every printed Zariba hexside has been built (§5.3): each
    /// printed side borders a Zariba hex, so walking those hexes finds all.
    pub fn zariba_complete(&self) -> bool {
        self.board.zariba.iter().all(|&hex| {
            hex.neighbors()
                .into_iter()
                .filter(|&n| self.is_printed_zariba_side(hex, n))
                .all(|n| self.zariba_hexsides.contains(&HexsideRef::new(hex, n)))
        })
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
        self.board.is_zariba(hex)
            && hex
                .neighbors()
                .into_iter()
                .any(|n| self.hexside_effective_is(hex, n, HexsideKind::is_zariba_trench))
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
    /// "from the White Nile to the Blue Nile and vice-versa": from a hex
    /// where one river leaves the map to a hex where the other does (the
    /// board names them -- a river may leave by more than one hex). `false`
    /// on a board that names none, and the move falls through to the
    /// ordinary contiguous-Nile rules.
    pub fn is_nile_mouth_crossing(&self, from: HexCoord, to: HexCoord) -> bool {
        use omdurman_types::Location::{BlueNileMouth, WhiteNileMouth};
        matches!(
            (self.board.location_at(from), self.board.location_at(to)),
            (Some(WhiteNileMouth), Some(BlueNileMouth))
                | (Some(BlueNileMouth), Some(WhiteNileMouth))
        )
    }

    /// The hexes a gunboat on `from` may cross to off the board (§9.345):
    /// the other river's mouth hexes. Empty unless `from` is a mouth hex.
    pub fn nile_mouth_crossings(&self, from: HexCoord) -> Vec<HexCoord> {
        self.board
            .locations
            .keys()
            .copied()
            .filter(|&to| self.is_nile_mouth_crossing(from, to))
            .collect()
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
/// full campaign roster, which on its own dominates CBMC's memory; harnesses
/// that only need "a state with empty ledgers" use this instead.
///
/// Deliberately gated to proof builds (and the sampled run of the expensive
/// harnesses under `cargo test`) and field-complete: adding a `GameState`
/// field breaks the proof build here loudly instead of drifting silently.
#[cfg(any(test, kani))]
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
            gunboat_maxims_fired_this_phase: Vec::new(),
            units_fired_at_this_phase: Vec::new(),
            units_shelled_this_phase: Vec::new(),
            mp_spent_this_turn: BTreeMap::new(),
            gunboats_upstream_this_turn: Vec::new(),
            zoc_stopped_this_turn: Vec::new(),
            units_meleed_this_turn: Vec::new(),
            vacated_by_combat: BTreeMap::new(),
            reinforcements_placed_this_turn: Vec::new(),
            eliminated: Vec::new(),
            game_over: false,
            zariba_hexsides: Vec::new(),
            friendlies_transport: None,
            optional_rules: Vec::new(),
            mines: Vec::new(),
            chain: None,
            pending_mine: None,
            gunboats_stopped_this_turn: Vec::new(),
            board: Arc::new(BoardInfo::default()),
            breaches: BTreeSet::new(),
            dervish_deserted: false,
            pending_melee: None,
            gordon_eliminated_turn: None,
            setup_ready_ae: false,
            setup_ready_dervish: false,
            isa_zachneih_eliminated: false,
            ae_lost_on_west_bank: false,
            pending_demolitions: Vec::new(),
            observations: Vec::new(),
            turn_events: Vec::new(),
            turn_summaries: Vec::new(),
            game_result: None,
        }
    }

    /// End a harness without dropping the state. Dropping walks the drop glue
    /// of every ledger -- `Observation` and `TurnEventRecord` carry
    /// `Vec<String>` payloads, each unrolled to the unwind bound -- which
    /// dominated the symex of the heaviest harnesses (`sink_chain_is_atomic`:
    /// 1.31M steps / 187 s with the drop, 233k / 24 s without). Skipping it
    /// proves no less: every assertion has already run, the engine holds no
    /// `unsafe` (`#![forbid(unsafe_code)]`) and no `Drop` impl of its own,
    /// so the drop glue is std's, and Kani does not check for leaks.
    pub(crate) fn kani_discard(self) {
        core::mem::forget(self);
    }
}

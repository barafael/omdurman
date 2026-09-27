use super::*;

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
    /// Setup-phase readiness per faction (§9.2/§9.3). Setup is concurrent -- both
    /// players deploy at once -- so each faction confirms independently; the game
    /// leaves [`Phase::Setup`] only once *both* are ready (and `setup_complete`
    /// holds). One-way: once set, a faction stays ready. `#[serde(default)]`
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

/// Longest `MoveUnit` path the engine accepts: well beyond any allowance
/// (the largest is 18, §5.11/§5.24), so it only bounds hostile input.
pub const MAX_MOVE_PATH_LEN: usize = 64;

/// An engine-validated move (see [`GameState::validate_move`]): the
/// movement points it costs, computed from the board (§5.11/§5.24), and
/// whether a gunboat took an upstream step (§5.24's sticky cap).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovePlan {
    pub cost: MovementPoints,
    pub went_upstream: bool,
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

    /// Whether `hex` is "entrenched": Nile-side of a ZaribaTrench hexside
    /// (§9.232: −2 melee for a Dervish attack on an entrenched unit). Reads
    /// *effective* hexsides, so authored FoK trenches and any constructed
    /// zariba both count.
    pub fn is_zariba_entrenched(&self, hex: HexCoord) -> bool {
        use omdurman_types::HexsideKind::{ZaribaTrench, ZaribaTrenchEndA, ZaribaTrenchEndB};
        for n in hex.neighbors() {
            if self.hexside_effective_is(hex, n, |k| {
                matches!(k, ZaribaTrench | ZaribaTrenchEndA | ZaribaTrenchEndB)
            }) && self.board.is_nile(n)
            {
                return true;
            }
        }
        false
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

    /// Whether deployment is finished and the game may leave [`Phase::Setup`]
    /// for the first Movement turn (§9.2/§9.3/§10). Both factions must have at
    /// least one unit on the board. The concrete per-scenario order of battle
    /// (which units, where) is enforced by the app's set-up plan, not here (the
    /// engine's `BoardInfo` carries no OOB); river mines/chain within limits are
    /// enforced at placement time, so they need no re-check here.
    ///
    /// Returns [`RuleError::SetupIncomplete`] naming the first unmet requirement,
    /// so the UI can surface *why* "Begin battle" is disabled. Every scenario
    /// currently shares the same "both sides deployed" gate; when a scenario
    /// needs a different minimum, branch on `self.scenario` here.
    pub fn setup_complete(&self) -> Result<(), RuleError> {
        let has = |player| {
            self.units
                .iter()
                .any(|u| u.profile.identity.owner() == player)
        };
        // §9.113: in the Campaign game the Anglo-Egyptian side starts with
        // *no* units on the map (they arrive as reinforcements from turn 1),
        // so only the Dervish §9.111 initial presence gates leaving Setup.
        if self.scenario != Scenario::Campaign && !has(Player::AngloEgyptian) {
            return Err(RuleError::SetupIncomplete(
                "Anglo-Egyptian forces not yet deployed",
            ));
        }
        if !has(Player::Dervish) {
            return Err(RuleError::SetupIncomplete(
                "Dervish forces not yet deployed",
            ));
        }
        // Fall of Khartoum pins both orders of battle (§9.321-9.322), so don't
        // let the game leave Setup until each side has deployed its full
        // contingent. The per-faction Ready button already gates on
        // `setup_target_met`; this is defense-in-depth for the unbound
        // "Begin battle" path and any future caller. Other scenarios have no
        // fixed target (`setup_target_met` reduces to "at least one"), so they
        // are unaffected.
        if !self.setup_target_met(Player::AngloEgyptian) {
            return Err(RuleError::SetupIncomplete(
                "Anglo-Egyptian order of battle not fully deployed",
            ));
        }
        if !self.setup_target_met(Player::Dervish) {
            return Err(RuleError::SetupIncomplete(
                "Dervish order of battle not fully deployed",
            ));
        }
        Ok(())
    }

    /// Whether `player` has confirmed it is ready to leave setup (§9.2/§9.3).
    pub fn setup_ready(&self, player: Player) -> bool {
        match player {
            Player::AngloEgyptian => self.setup_ready_ae,
            Player::Dervish => self.setup_ready_dervish,
        }
    }

    /// How many of `player`'s units are currently on the board -- the deployed
    /// count shown during setup and compared against [`Self::setup_target`].
    pub fn setup_deployed_count(&self, player: Player) -> usize {
        self.units
            .iter()
            .filter(|u| u.profile.identity.owner() == player)
            .count()
    }

    /// The number of units `player` must deploy before turn 1, when the scenario
    /// pins it down. Only **Fall of Khartoum** has a bounded deploy-everything
    /// setup -- British 17, Dervish 48 (§9.321-9.322), plus the §9.344 North Fort
    /// (a scenario-fixed fort auto-placed by `FALL_OF_KHARTOUM_SETUP`). The
    /// Historical scenario deploys by rule ("all remaining in the Zariba",
    /// "within three hexes of a leader") and the Campaign is reinforcement-driven
    /// (the A-E player starts with *no* units on the map, §9.113), so neither has
    /// a fixed target: `None` there means "no hard count -- just show what's
    /// deployed".
    pub fn setup_target(&self, player: Player) -> Option<usize> {
        match (self.scenario, player) {
            (Scenario::FallOfKhartoum, Player::AngloEgyptian) => Some(17),
            // 48 player-deployed entry force + 1 scenario-fixed North Fort fort.
            (Scenario::FallOfKhartoum, Player::Dervish) => Some(49),
            _ => None,
        }
    }

    /// Whether `player` has deployed enough to be allowed to confirm ready: it
    /// meets its `setup_target` when the scenario sets one, else just needs the
    /// board-wide `setup_complete` minimum (at least one unit).
    pub fn setup_target_met(&self, player: Player) -> bool {
        match self.setup_target(player) {
            Some(target) => self.setup_deployed_count(player) >= target,
            // §9.113: the Campaign A-E side deploys nothing at setup.
            None if self.scenario == Scenario::Campaign && player == Player::AngloEgyptian => true,
            None => self.setup_deployed_count(player) >= 1,
        }
    }

    /// Whether `hex` is inside `player`'s deployment zone for this scenario
    /// (§9.211-9.212 Historical, §9.321-9.322 Fall of Khartoum). A hex must first
    /// be on the board (present in `board.terrain`); an empty board (no map facts
    /// attached) is treated as fully permissive so headless tests can deploy
    /// anywhere.
    ///
    /// Zones, from the manual:
    /// - **Fall of Khartoum British** (§9.321): the garrison sets up in building
    ///   or hut hexes, at Fort Makran / Fort Buri / the Palace, or adjacent to a
    ///   wall hexside. (Gordon is pre-placed.) Per §5.22 the split is exclusive
    ///   -- gunboats deploy *only* on Nile hexes, and land units may never
    ///   deploy on the Nile.
    /// - **Fall of Khartoum Dervish** (§9.322): enters from the south or east
    ///   map edge. The FoK board is diamond-shaped: the south edge is the
    ///   bottom row (no hex at `r+1`); the east edge is the diagonal of
    ///   rightmost hexes per row (no hex at `q+1`). Gunboats may also enter
    ///   from the west (Nile) edge (no hex at `q-1`).
    /// - **Historical / Campaign** (§9.211-9.212, §9.11): permissive. The
    ///   manual's constraints there are the 13 Zariba hexes, the Kerreri huts,
    ///   and per-leader "within three hexes" color groups -- data the engine's
    ///   `BoardInfo` does not carry (no Zariba-hex set, no Kerreri landmark, no
    ///   per-unit leader color), so those are enforced by the scenario set-up
    ///   plan / UI rather than this hex predicate. Documented, not silently
    ///   dropped.
    pub fn in_deployment_zone(&self, player: Player, hex: HexCoord, is_boat: bool) -> bool {
        // No board attached -> permissive (unit tests, unbound session).
        if self.board.terrain.is_empty() {
            return true;
        }
        if self.board.terrain_at(hex).is_none() {
            return false; // off the playable map
        }
        // §5.22 is universal during deployment (all scenarios, both factions):
        // gunboats deploy *only* on the Nile, and land units *never* deploy on
        // the Nile. Previously this was only checked for Fall of Khartoum, so
        // Campaign/Historical set-ups could anchor a gunboat on land or drop
        // an infantry counter in the river (audit §5.22/§9.111).
        let is_nile = matches!(
            self.board.terrain_at(hex),
            Some(omdurman_types::Terrain::Nile { .. })
        );
        if is_boat {
            if !is_nile {
                return false;
            }
        } else if is_nile {
            return false;
        }
        match self.scenario {
            Scenario::Historical | Scenario::Campaign => true,
            Scenario::FallOfKhartoum => {
                // (§5.22 was already applied above.)
                match player {
                    Player::Dervish => {
                        // The North Fort is Dervish-controlled from the start
                        // (§9.344) and is a fixed fortification, not part of the
                        // entry force -- so it's a legal deploy hex for the
                        // Dervish forts regardless of the south/east-edge rule
                        // below.
                        if matches!(
                            self.board.location_at(hex),
                            Some(omdurman_types::Location::NorthFort)
                        ) {
                            return true;
                        }
                        // South or east map edge (§9.322), plus the western Nile
                        // edge for gunboats -- the Nile runs along the west side
                        // of the FoK map and gunboats need water to deploy.
                        //
                        // The FoK board is diamond-shaped: the "east edge" is
                        // the diagonal of rightmost hexes per row (where no
                        // hex exists at q+1), not just q == global max_q.
                        // Similarly the south edge is the bottom row (no hex
                        // at r+1) and the west edge is the leftmost diagonal
                        // (no hex at q-1).
                        let on_south_edge = !self
                            .board
                            .terrain
                            .contains_key(&HexCoord::new(hex.q, hex.r + 1));
                        let on_east_edge = !self
                            .board
                            .terrain
                            .contains_key(&HexCoord::new(hex.q + 1, hex.r));
                        let on_west_edge = !self
                            .board
                            .terrain
                            .contains_key(&HexCoord::new(hex.q - 1, hex.r));
                        on_south_edge || on_east_edge || (is_boat && on_west_edge)
                    }
                    Player::AngloEgyptian => {
                        // The North Fort is Dervish-controlled (§9.344) and must
                        // not appear in the AE deployment zone.
                        if matches!(
                            self.board.location_at(hex),
                            Some(omdurman_types::Location::NorthFort)
                        ) {
                            return false;
                        }
                        // A gunboat was already constrained to a Nile hex by the
                        // §5.22 check above; any Nile hex is a legal anchor for
                        // the two old FoK gunboats (§9.321), with no further
                        // restriction.
                        if is_boat {
                            return true;
                        }
                        // Land units (§9.321): a building or hut hex, a garrison
                        // landmark (Palace / Fort Makran / Fort Buri), or a hex
                        // adjacent to a wall hexside. (Already guaranteed
                        // not-Nile above.)
                        let terrain = self.board.terrain_at(hex);
                        let is_garrison_terrain = matches!(
                            terrain,
                            Some(
                                omdurman_types::Terrain::Building { .. }
                                    | omdurman_types::Terrain::Huts { .. }
                            )
                        );
                        let at_landmark = matches!(
                            self.board.location_at(hex),
                            Some(
                                omdurman_types::Location::Palace
                                    | omdurman_types::Location::FortMakran
                                    | omdurman_types::Location::FortBuri
                            )
                        );
                        let adjacent_to_wall = hex.neighbors().iter().any(|&n| {
                            self.hexside_effective_is(hex, n, |k| k == HexsideKind::Wall)
                        });
                        is_garrison_terrain || at_landmark || adjacent_to_wall
                    }
                }
            }
        }
    }

    /// Guard shared by every setup placement: the action is legal only during
    /// [`Phase::Setup`] (§9.2/§9.3/§10).
    fn require_setup_phase(&self) -> Result<(), RuleError> {
        if self.phase != Phase::Setup {
            return Err(RuleError::WrongPhase);
        }
        Ok(())
    }

    /// Read-only check of whether `placement` may be deployed in [`Phase::Setup`]
    /// (§9.2/§9.3): right phase, the counter isn't already on the board (each
    /// physical unit deploys once), inside the owner's deployment zone, and legal
    /// stacking. Mirrors the `DeployUnit` effect so the UI can gate input.
    pub fn can_deploy_unit(&self, placement: &UnitPlacement) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        if self.units.iter().any(|u| u.id == placement.id) {
            return Err(RuleError::AlreadyDeployed(placement.id));
        }
        // Scenario-specific "in play at setup" filter: the Campaign's initial
        // force is the §9.111 Dervish set (everything else arrives as a
        // reinforcement, §9.112/§9.113), and the Historical scenario excludes
        // its not-in-play units outright (§9.211/§9.212).
        self.unit_in_play_at_setup(placement)?;
        let owner = placement.profile.identity.owner();
        if !self.in_deployment_zone(owner, placement.position, placement.profile.kind.is_boat()) {
            return Err(RuleError::OutsideDeploymentZone(placement.position));
        }
        self.check_stacking(placement, placement.position)?;
        // A peer may not invent unit values: the counter enters with its
        // canonical profile and a fresh state.
        require_canonical_placement(placement)
    }

    /// The FALL OF KHARTOUM orders of battle (§9.321 British, §9.322 Dervish):
    /// the exact number of counters of each type that may deploy at setup, and
    /// `None` for every unit type not in the scenario at all. The single North
    /// Fort (§9.344) and GORDON in the palace (§9.321) are scenario-fixed and
    /// bypass this table.
    ///
    /// §9.321/§9.322: how many more counters of `identity`'s order-of-battle
    /// group may still deploy at setup (`None`: the type is not in the FoK
    /// orders of battle). The bot's setup generator uses this to stop
    /// offering candidates the engine would reject.
    pub fn fok_setup_slots_remaining(&self, identity: &crate::UnitIdentity) -> Option<usize> {
        let (group, cap) = fok_cap_group(identity)?;
        let already = self
            .units
            .iter()
            .filter(|u| fok_cap_group(&u.profile.identity).is_some_and(|(g, _)| g == group))
            .count();
        Some(cap.saturating_sub(already))
    }

    /// Whether `profile` belongs to a unit that may be on the board at setup
    /// in the current scenario (§9.111 Campaign initial force; §9.211/§9.212
    /// Historical not-in-play lists; §9.321/§9.322 Fall of Khartoum orders of
    /// battle, including their exact per-type counts).
    fn unit_in_play_at_setup(&self, placement: &UnitPlacement) -> Result<(), RuleError> {
        use crate::UnitIdentity;
        match self.scenario {
            Scenario::Campaign => match placement.profile.identity {
                // §9.111: the Anglo-Egyptian side starts empty (§9.113).
                UnitIdentity::AngloEgyptianInfantry { .. }
                | UnitIdentity::AngloEgyptianCavalry
                | UnitIdentity::AngloEgyptianCamelCorps
                | UnitIdentity::AngloEgyptianArtillery
                | UnitIdentity::AngloEgyptianMaxim
                | UnitIdentity::AngloEgyptianGunboat(_)
                | UnitIdentity::AngloEgyptianLeader(_)
                | UnitIdentity::RoyalEngineers => Err(RuleError::NotInPlay(placement.id)),
                // §9.111 Dervish initial force: the Khalifa, Isa Zachneih,
                // the three artillery, the Taiasha bodyguard, the forts and
                // the two gunboats. Every other tribe/leader is a §9.112
                // reinforcement wave.
                UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah) => Ok(()),
                UnitIdentity::DervishTribal {
                    tribe: crate::DervishTribe::Taiasha,
                }
                | UnitIdentity::DervishTribal {
                    tribe: crate::DervishTribe::IsaZachneih,
                } => Ok(()),
                UnitIdentity::DervishArtillery
                | UnitIdentity::DervishFort
                | UnitIdentity::DervishGunboat(_) => Ok(()),
                _ => Err(RuleError::NotInPlay(placement.id)),
            },
            Scenario::Historical => match placement.profile.identity {
                // §9.211: GORDON and the "Friendlies" brigade are not in play.
                UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Gordon) => {
                    Err(RuleError::NotInPlay(placement.id))
                }
                identity if identity.is_friendlies() => Err(RuleError::NotInPlay(placement.id)),
                // §9.212: Isa Zachneih, gunboats, and forts are not in play.
                UnitIdentity::DervishTribal {
                    tribe: crate::DervishTribe::IsaZachneih,
                } => Err(RuleError::NotInPlay(placement.id)),
                UnitIdentity::DervishGunboat(_) | UnitIdentity::DervishFort => {
                    Err(RuleError::NotInPlay(placement.id))
                }
                _ => Ok(()),
            },
            Scenario::FallOfKhartoum => {
                // §9.321/§9.322 orders of battle with their exact per-type
                // counts (grouped: the manual counts "two British infantry
                // units", not per battalion ordinal). The scenario-fixed
                // counters (GORDON in the palace, §9.344's single North
                // Fort) deploy through this same table.
                match self.fok_setup_slots_remaining(&placement.profile.identity) {
                    None => Err(RuleError::NotInPlay(placement.id)),
                    Some(0) => Err(RuleError::FoKOrderOfBattleFull),
                    Some(_) => Ok(()),
                }
            }
        }
    }

    /// Read-only check of whether `player` may pick a deployed unit back up off
    /// the board during [`Phase::Setup`] (§9.2/§9.3): right phase, the unit is on
    /// the board, and it belongs to `player` (you may only re-pick your own
    /// counters). Mirrors the `RemoveDeployedUnit` effect.
    pub fn can_remove_deployed_unit(
        &self,
        unit_id: UnitId,
        player: Player,
    ) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        let unit = self.unit_or_err(unit_id)?;
        if unit.profile.identity.owner() != player {
            return Err(RuleError::NotOwner(unit_id));
        }
        Ok(())
    }

    /// Read-only check of a river-mine placement in setup (§10.11): Setup phase,
    /// at most [`MAX_MINES`], and no two mines on the same hex.
    pub fn can_place_mine(&self, hex: HexCoord) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        // Optional-rule gate: mines exist only when the River Mines option was
        // selected at game start (§10.11).
        if !self.optional_rules.contains(&OptionalRule::RiverMines) {
            return Err(RuleError::SetupLimit(
                "the River Mines optional rule is not in play (§10.11)",
            ));
        }
        if self.mines.iter().any(|m| m.hex == hex) {
            return Err(RuleError::SetupLimit("a mine is already laid on that hex"));
        }
        if self.mines.len() >= MAX_MINES {
            return Err(RuleError::SetupLimit("at most two river mines (§10.11)"));
        }
        Ok(())
    }

    /// Read-only check of a river-chain placement in setup (§10.21): Setup phase
    /// and at most [`MAX_CHAIN_HEXES`] hexes.
    pub fn can_place_chain(&self, hexes: &[HexCoord]) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        // Optional-rule gate: the chain exists only when the River Chain option
        // was selected at game start (§10.21).
        if !self.optional_rules.contains(&OptionalRule::RiverChain) {
            return Err(RuleError::SetupLimit(
                "the River Chain optional rule is not in play (§10.21)",
            ));
        }
        if hexes.is_empty() {
            return Err(RuleError::SetupLimit(
                "the chain must span at least one hex",
            ));
        }
        if hexes.len() > MAX_CHAIN_HEXES {
            return Err(RuleError::SetupLimit(
                "the river chain spans at most four hexes (§10.21)",
            ));
        }
        Ok(())
    }

    /// Read-only check of a pre-placed Zariba hexside in setup (§9.231-9.232):
    /// only during Setup.
    pub fn can_place_zariba(&self) -> Result<(), RuleError> {
        self.require_setup_phase()
    }

    /// Read-only check of whether `player` may confirm ready to leave setup
    /// (§9.2/§9.3): must be in Setup and have deployed enough
    /// ([`Self::setup_target_met`]), so a player can't lock in before placing its
    /// order of battle. Re-confirming an already-ready faction is allowed (no-op).
    pub fn can_confirm_setup_ready(&self, player: Player) -> Result<(), RuleError> {
        self.require_setup_phase()?;
        if !self.setup_target_met(player) {
            return Err(RuleError::SetupIncomplete(
                "deploy your forces before confirming ready",
            ));
        }
        Ok(())
    }

    /// Read-only check of whether `unit_id` may move `cost` movement points in
    /// the current state (§5): right phase, right player, not disrupted, not
    /// stopped in an enemy ZOC, land-mobile, within (night-adjusted) allowance.
    /// Returns the same `RuleError` the `MoveUnit` effect would on rejection.
    /// Lets the UI gate input without mutating or duplicating the rules.
    pub fn can_move_unit(&self, unit_id: UnitId, cost: MovementPoints) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        self.movement_preconditions(unit)?;
        let allowance = self.land_allowance(unit)?;
        self.check_move_allowance(unit_id, i32::from(cost.value()), allowance, false)
    }

    /// As [`can_move_unit`](Self::can_move_unit), but when `to` is supplied the
    /// move is validated hex by hex along the straight line from the unit's
    /// current hex to `to` (see [`validate_move`](Self::validate_move)): every
    /// step adjacent, passable and on the board, no pass-through of an enemy
    /// ZOC (§5.26, §5.43 -- the destination itself may be a ZOC hex; a unit
    /// that *begins* in an enemy ZOC may still move out), and the
    /// engine-computed terrain cost (or the caller's `cost`, whichever is
    /// larger) within the allowance. Land units only (gunboats:
    /// [`can_move_gunboat`](Self::can_move_gunboat)).
    ///
    /// Stacking (§5.51) is checked when the move is applied, not here.
    pub fn can_move_unit_to(
        &self,
        unit_id: UnitId,
        to: Option<HexCoord>,
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        let Some(to) = to else {
            return self.can_move_unit(unit_id, cost);
        };
        check_coord(to)?;
        let unit = self.unit_or_err(unit_id)?;
        self.movement_preconditions(unit)?;
        let allowance = self.land_allowance(unit)?;
        let mut path = unit.position.line_between(to);
        path.push(to);
        let plan = self.validate_move(unit_id, to, &path)?;
        let cost = i32::from(plan.cost.value()).max(i32::from(cost.value()));
        self.check_move_allowance(unit_id, cost, allowance, false)
    }

    /// As [`can_move_unit_to`](Self::can_move_unit_to), but along the *actual*
    /// stepped `path` (the entered hexes, excluding the start, ending at `to`)
    /// -- a bent path that avoids ZOC hexes is legal even when the straight
    /// line would cross one. Exactly the `MoveUnit` validation
    /// ([`validate_move`](Self::validate_move)); the engine computes the cost,
    /// and a larger caller `cost` must also fit the allowance.
    pub fn can_move_unit_along(
        &self,
        unit_id: UnitId,
        to: HexCoord,
        path: &[HexCoord],
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        let plan = self.validate_move(unit_id, to, path)?;
        self.check_caller_cost(unit_id, &plan, cost)
    }

    /// For the UI/bot predicates: a caller may ask whether it can afford a
    /// move at a (higher) cost it computed itself -- the engine's own cost
    /// was already checked by [`validate_move`](Self::validate_move), so the
    /// larger of the two must fit the allowance (the `MoveUnit` effect
    /// itself ignores the caller's cost).
    fn check_caller_cost(
        &self,
        unit_id: UnitId,
        plan: &MovePlan,
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        if cost.value() <= plan.cost.value() {
            return Ok(());
        }
        let unit = self.unit_or_err(unit_id)?;
        match unit.profile.movement {
            crate::UnitMovement::Gunboat(ga) => {
                let capped =
                    plan.went_upstream || self.gunboats_upstream_this_turn.contains(&unit_id);
                let allowance = if capped { ga.upstream } else { ga.downstream };
                self.check_move_allowance(unit_id, i32::from(cost.value()), allowance, capped)
            }
            _ => {
                let allowance = self.land_allowance(unit)?;
                self.check_move_allowance(unit_id, i32::from(cost.value()), allowance, false)
            }
        }
    }

    /// The shared movement preconditions (§5): Movement phase, the active
    /// player's unit (§5.1), not disrupted, not already stopped in an enemy
    /// ZOC this turn (§5.26/§5.43), and not GORDON in FALL OF KHARTOUM
    /// (§9.346).
    fn movement_preconditions(&self, unit: &UnitPlacement) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        // §5.1: only the active player's units move during their player turn.
        // Fire (§6.41), melee (§7.1) and reinforcements (§9.112/§9.113) all
        // compare the actor to `active_player` the same way.
        if unit.profile.identity.owner() != self.active_player {
            return Err(RuleError::NotYourTurn);
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit.id));
        }
        // §5.26/§5.43: a unit that entered an enemy ZOC this movement phase
        // "may move no further that turn" (it may withdraw next phase).
        if self.zoc_stopped_this_turn.contains(&unit.id) {
            return Err(RuleError::StoppedInEnemyZoc(unit.id));
        }
        // §9.346: the GORDON leader unit may not move during FALL OF KHARTOUM.
        if self.scenario == Scenario::FallOfKhartoum && unit.profile.identity.is_gordon() {
            return Err(RuleError::GordonMayNotMove);
        }
        Ok(())
    }

    /// The (night-adjusted, §8.1) land movement allowance of `unit`;
    /// `NotMobile` for gunboats and immobile units.
    fn land_allowance(&self, unit: &UnitPlacement) -> Result<MovementAllowance, RuleError> {
        match unit.profile.movement {
            crate::UnitMovement::Land(a) => Ok(crate::effective_movement_at_night(
                a,
                unit.profile.identity.owner(),
                self.day_night,
            )),
            crate::UnitMovement::Gunboat(_) | crate::UnitMovement::Immobile => {
                Err(RuleError::NotMobile(unit.id))
            }
        }
    }

    /// §5.11/§5.12: a unit moves hex by hex up to its allowance; the *running
    /// total* spent this turn plus `cost` must fit `allowance`. Widened
    /// arithmetic, so no caller-supplied value can overflow it.
    fn check_move_allowance(
        &self,
        unit_id: UnitId,
        cost: i32,
        allowance: MovementAllowance,
        upstream_cap: bool,
    ) -> Result<(), RuleError> {
        let total = i32::from(self.mp_spent(unit_id)).saturating_add(cost);
        if total > i32::from(allowance.value()) {
            let cost = MovementPoints(i16::try_from(total).unwrap_or(i16::MAX));
            return Err(if upstream_cap {
                RuleError::GunboatUpstreamCap { cost, allowance }
            } else {
                RuleError::MovementExceedsAllowance { cost, allowance }
            });
        }
        Ok(())
    }

    /// Validate a `MoveUnit` (§5) step by step and return its engine-computed
    /// plan. `path` is the ordered hexes *entered* (excluding the start,
    /// ending at `to`); an empty path means the single step to `to`. The
    /// caller never supplies the cost: the engine is authoritative for it.
    ///
    /// Shared by land and gunboat moves. Every step must be a single hex
    /// (§5.11) that is not enemy-occupied (§7.1; lone Anglo-Egyptian leaders
    /// are overrun instead, §6.51) and not an enemy fort (§6.54); no hex
    /// before the destination may lie in an enemy ZOC (§5.26/§5.43). Land
    /// steps must stay on the board, off the Nile (§5.22), not cross a wall
    /// (§5.23; gates and breaches pass) and respect the walled-city entry
    /// restriction (§5.23); each costs its Terrain Effects Chart value
    /// (§5.11, road overlay) plus the §9.233 zariba-end surcharge. Gunboat
    /// steps stay on the Nile (§5.22), stop at the chain (§10.22) and cost
    /// one MP each, capped by the upstream/downstream allowance (§5.24); the
    /// FALL OF KHARTOUM mouth crossing (§9.345) is a single flat-6 "step".
    /// The running total spent this turn plus the path's cost must fit the
    /// allowance. Stacking (§5.51) is checked at the destination on apply.
    pub fn validate_move(
        &self,
        unit_id: UnitId,
        to: HexCoord,
        path: &[HexCoord],
    ) -> Result<MovePlan, RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        check_coord(to)?;
        for hex in path {
            check_coord(*hex)?;
        }
        if path.len() > MAX_MOVE_PATH_LEN {
            return Err(RuleError::PathTooLong(path.len()));
        }
        let steps: &[HexCoord] = if path.is_empty() {
            std::slice::from_ref(&to)
        } else {
            path
        };
        if steps.last() != Some(&to) {
            return Err(RuleError::PathEndMismatch(to));
        }
        match unit.profile.movement {
            // §5.25: forts may never move once placed.
            crate::UnitMovement::Immobile => Err(RuleError::AlreadyPlaced(unit_id)),
            crate::UnitMovement::Gunboat(ga) => {
                self.movement_preconditions(unit)?;
                self.plan_gunboat_move(unit, ga, to, steps)
            }
            crate::UnitMovement::Land(_) => {
                self.movement_preconditions(unit)?;
                let allowance = self.land_allowance(unit)?;
                self.plan_land_move(unit, allowance, steps)
            }
        }
    }

    /// Per-step checks shared by land and gunboat moves: the step is a
    /// single hex (§5.11), the entered hex is not an enemy fort (§6.54) and
    /// not enemy-occupied (§7.1 with §5.26 -- movement may only bring a unit
    /// adjacent; engaging is melee's job). Lone Anglo-Egyptian leaders do not
    /// block: §6.51 eliminates them when a Dervish unit occupies or passes
    /// through their hex (the overrun in `apply_move_unit`).
    fn check_move_step(
        &self,
        mover: &UnitPlacement,
        from: HexCoord,
        to: HexCoord,
    ) -> Result<(), RuleError> {
        if !from.is_adjacent_to(to) {
            return Err(RuleError::PathNotContiguous { from, to });
        }
        let owner = mover.profile.identity.owner();
        if self.hex_has_enemy_fort(to, owner) {
            return Err(RuleError::EnemyFort(to));
        }
        let enemy = owner.opponent();
        if self.units.iter().any(|u| {
            u.position == to
                && u.profile.identity.owner() == enemy
                && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
        }) {
            return Err(RuleError::EnemyOccupied(to));
        }
        Ok(())
    }

    /// The land half of [`validate_move`](Self::validate_move).
    fn plan_land_move(
        &self,
        unit: &UnitPlacement,
        allowance: MovementAllowance,
        steps: &[HexCoord],
    ) -> Result<MovePlan, RuleError> {
        let owner = unit.profile.identity.owner();
        let kind = unit.profile.kind;
        let last = steps.len() - 1;
        let mut cost: i32 = 0;
        let mut prev = unit.position;
        for (i, &next) in steps.iter().enumerate() {
            if !prev.is_adjacent_to(next) {
                return Err(RuleError::PathNotContiguous {
                    from: prev,
                    to: next,
                });
            }
            // §5.22: land units may never enter a Nile hex.
            if self.board.is_nile(next) {
                return Err(RuleError::LandIntoNile(next));
            }
            // A unit may never step off the board (with no board loaded, map
            // constraints don't apply).
            if !self.board.terrain.is_empty() && self.board.terrain_at(next).is_none() {
                return Err(RuleError::OffBoard(next));
            }
            self.check_move_step(unit, prev, next)?;
            // §5.26/§5.43: a unit must stop the instant it enters an enemy
            // ZOC, so no hex entered before the destination may lie in one.
            if i < last && self.hex_in_enemy_zoc(next, owner, kind) {
                return Err(RuleError::BlockedByEnemyZoc(next));
            }
            // §5.23: a wall hexside blocks movement (gates and breaches pass).
            // Read through `hexside_effective` so a §6.63 breach is an opening.
            if self.hexside_effective_is(prev, next, HexsideKind::blocks_movement) {
                return Err(RuleError::MoveBlockedByHexside(prev, next));
            }
            // §5.23: only certain units may enter the walled portion of Omdurman
            // -- Dervish: the Khalifa, the artillery, and the Taiasha bodyguard;
            // Anglo-Egyptian: any unit except gunboats and "Friendlies". Scoped
            // to the Omdurman map: FALL OF KHARTOUM is a different walled city
            // (Khartoum) whose set-up places units inside it freely (§9.32).
            if self.scenario != Scenario::FallOfKhartoum
                && self.board.is_walled_city(next)
                && !self.board.is_walled_city(prev)
                && !unit.profile.identity.may_enter_walled_city()
            {
                return Err(RuleError::WalledCityEntry(unit.id, next));
            }
            cost = cost.saturating_add(self.land_step_cost(prev, next));
            prev = next;
        }
        self.check_move_allowance(unit.id, cost, allowance, false)?;
        Ok(MovePlan {
            cost: MovementPoints(i16::try_from(cost).unwrap_or(i16::MAX)),
            went_upstream: false,
        })
    }

    /// Terrain Effects Chart cost of entering `to` from `from` (§5.11, road
    /// overlay) plus the §9.233 zariba-end surcharge. Impassable (Nile) or
    /// unknown terrain counts as clear -- passability is checked separately.
    fn land_step_cost(&self, from: HexCoord, to: HexCoord) -> i32 {
        let terrain = self
            .board
            .terrain_at(to)
            .unwrap_or(omdurman_types::Terrain::Clear {
                road: Default::default(),
            });
        let base = crate::terrain_chart::movement_cost_with_road(terrain, self.board.has_road(to))
            .map_or(1, |a| i32::from(a.value()));
        base + i32::from(self.zariba_entry_surcharge(from, to))
    }

    /// The gunboat half of [`validate_move`](Self::validate_move) (§5.22,
    /// §5.24, §9.345, §10.22).
    fn plan_gunboat_move(
        &self,
        unit: &UnitPlacement,
        ga: crate::GunboatMovement,
        to: HexCoord,
        steps: &[HexCoord],
    ) -> Result<MovePlan, RuleError> {
        // §9.345 (FALL OF KHARTOUM): a British gunboat may cross between the
        // White and Blue Nile mouths off-board for a flat 6 "upstream" MP,
        // bypassing the normal contiguous-Nile path. Only the two named mouth
        // hexes participate; the move is otherwise a normal move (and counts
        // against the upstream allowance, §5.24).
        if self.scenario == Scenario::FallOfKhartoum
            && steps.len() == 1
            && self.is_nile_mouth_crossing(unit.position, to)
        {
            const CROSS_NILE_MP: i32 = 6;
            let owner = unit.profile.identity.owner();
            if self.hex_has_enemy_fort(to, owner)
                || self
                    .units
                    .iter()
                    .any(|u| u.position == to && u.profile.identity.owner() == owner.opponent())
            {
                return Err(RuleError::EnemyOccupied(to));
            }
            self.check_move_allowance(unit.id, CROSS_NILE_MP, ga.upstream, true)?;
            return Ok(MovePlan {
                cost: MovementPoints(CROSS_NILE_MP as i16),
                went_upstream: true,
            });
        }

        let owner = unit.profile.identity.owner();
        let kind = unit.profile.kind;
        let last = steps.len() - 1;
        let mut moved_upstream = false;
        let mut prev = unit.position;
        for (i, &next) in steps.iter().enumerate() {
            if !prev.is_adjacent_to(next) {
                return Err(RuleError::PathNotContiguous {
                    from: prev,
                    to: next,
                });
            }
            // §5.22: gunboats stay on the Nile. With a board loaded, every
            // entered hex must be a Nile hex.
            if !self.board.terrain.is_empty() && !self.board.is_nile(next) {
                return Err(RuleError::GunboatOffNile(next));
            }
            // §10.22: a chained Nile hex stops the gunboat.
            if self.chain_covers(next) {
                return Err(RuleError::BlockedByChain(next));
            }
            self.check_move_step(unit, prev, next)?;
            // §5.41/§5.43: a gunboat stops on entering an enemy *gunboat's*
            // ZOC (which `hex_in_enemy_zoc` encodes for a gunboat mover).
            if i < last && self.hex_in_enemy_zoc(next, owner, kind) {
                return Err(RuleError::BlockedByEnemyZoc(next));
            }
            if self.board.step_direction(prev, next) == Some(crate::board::StepDirection::Upstream)
            {
                moved_upstream = true;
            }
            prev = next;
        }

        // §5.24: any upstream step caps the whole turn at the upstream
        // allowance; otherwise the downstream allowance applies. The cap is
        // *sticky*: an upstream hex taken in an earlier move of the same turn
        // still caps this (all-downstream) move -- "if they move even one hex
        // upstream, their upstream movement allowance is their maximum
        // movement allowance for that turn". Gunboats pay one MP per hex.
        let capped = moved_upstream || self.gunboats_upstream_this_turn.contains(&unit.id);
        let allowance = if capped { ga.upstream } else { ga.downstream };
        // `steps` is bounded by `MAX_MOVE_PATH_LEN`, so the length fits.
        let cost = steps.len() as i32;
        self.check_move_allowance(unit.id, cost, allowance, capped)?;
        Ok(MovePlan {
            cost: MovementPoints(cost as i16),
            went_upstream: moved_upstream,
        })
    }

    /// The true movement-point cost of a move along `path` (the entered hexes,
    /// excluding the start), computed from the board's Terrain Effects Chart
    /// (§5.11). Returns `None` when no board/path is available. Land units pay
    /// each hex's terrain cost (plus the §9.233 zariba-end surcharge);
    /// gunboats pay one MP per Nile hex entered (§5.24 counts hexes, not
    /// terrain). Per-hex passability is enforced separately by
    /// [`validate_move`](Self::validate_move), so an off-map hex here
    /// contributes the clear-terrain base of 1.
    ///
    /// §5.42: entering or leaving an enemy ZOC adds no MP cost.
    pub fn movement_cost_for(
        &self,
        unit: &UnitPlacement,
        path: &[HexCoord],
    ) -> Option<MovementPoints> {
        if path.is_empty() || self.board.terrain.is_empty() {
            return None;
        }
        let total: i32 = match unit.profile.movement {
            crate::UnitMovement::Gunboat(_) => i32::try_from(path.len()).unwrap_or(i32::MAX),
            _ => {
                let mut sum = 0i32;
                let mut prev = unit.position;
                for hex in path {
                    sum = sum.saturating_add(self.land_step_cost(prev, *hex));
                    prev = *hex;
                }
                sum
            }
        };
        Some(MovementPoints(i16::try_from(total).unwrap_or(i16::MAX)))
    }

    /// Validate a gunboat move along `path` (§5.22, §5.24, §10.22): exactly
    /// the `MoveUnit` validation ([`validate_move`](Self::validate_move)) for
    /// a gunboat. Gunboats may move only along Nile hexes; their two
    /// allowances are upstream (smaller) and downstream (larger); and "if they
    /// move even one hex upstream, their upstream movement allowance is their
    /// maximum for that turn." Chained Nile hexes stop the gunboat (§10.22).
    /// The engine computes the cost (one MP per hex, a flat 6 for the §9.345
    /// mouth crossing); a larger caller `cost` must also fit.
    pub fn can_move_gunboat(
        &self,
        unit_id: UnitId,
        to: HexCoord,
        path: &[HexCoord],
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if !matches!(unit.profile.movement, crate::UnitMovement::Gunboat(_)) {
            self.movement_preconditions(unit)?;
            return Err(RuleError::NotAGunboat(unit_id));
        }
        let plan = self.validate_move(unit_id, to, path)?;
        self.check_caller_cost(unit_id, &plan, cost)
    }

    /// The player whose fire attacks are legal right now (§4): the active
    /// player during Offensive Fire, their opponent during Defensive Fire.
    /// `Err(WrongPhase)` outside both fire phases. Shared by
    /// [`Self::can_fire_at`] and [`Self::can_fire_at_wall`].
    fn fire_phase_player(&self) -> Result<Player, RuleError> {
        match self.phase {
            Phase::OffensiveFire(_) => Ok(self.active_player),
            Phase::DefensiveFire(_) => Ok(self.active_player.opponent()),
            _ => Err(RuleError::WrongPhase),
        }
    }

    /// The "Units" line-of-sight blocker (§6.3 note a): a hex occupied by any
    /// non-gunboat, non-fort unit blocks LOS at that hex's terrain level.
    /// Shared by [`Self::can_fire_at`] and [`Self::can_fire_at_wall`].
    fn los_unit_blocker(&self) -> impl Fn(HexCoord) -> Option<crate::los_table::LosLevel> + '_ {
        move |hex| {
            let has_blocking_unit = self.units.iter().any(|u| {
                u.position == hex
                    && !matches!(
                        u.profile.kind,
                        crate::UnitKind::Gunboat { .. } | crate::UnitKind::Fort { .. }
                    )
            });
            if has_blocking_unit {
                self.board.terrain_at(hex).map(crate::los_table::los_level)
            } else {
                None
            }
        }
    }

    /// Read-only check of whether `firer` may fire `kind` at `target_hex` in
    /// the current state (§6): right fire sub-phase for the kind, right player,
    /// firer has a fire factor, weapon class permits the kind, not disrupted,
    /// hasn't already fired this phase, and the target is within (night-
    /// adjusted) range for the firer's weapon.
    ///
    /// Does **not** check line of sight or terrain -- those need the game map,
    /// which the rules engine does not hold; the app supplies the terrain
    /// modifier in the [`FireAttack`] and is responsible for the LOS gate.
    /// (Howitzer fire ignores LOS entirely -- §6.64.)
    pub fn can_fire_at(
        &self,
        firer: UnitId,
        target_hex: HexCoord,
        kind: FireKind,
    ) -> Result<(), RuleError> {
        let unit = self.unit_or_err(firer)?;

        if unit.profile.identity.owner() != self.fire_phase_player()? {
            return Err(RuleError::NotYourTurn);
        }

        // The fire kind must match the current sub-phase (§6.42): direct fire
        // in the Direct sub-phase; Maxim-second / howitzer in the second.
        let sub = match self.phase {
            Phase::OffensiveFire(s) | Phase::DefensiveFire(s) => s,
            _ => return Err(RuleError::WrongPhase),
        };
        let kind_ok = matches!(
            (sub, kind),
            (FireSubPhase::DirectFire, FireKind::Direct)
                | (
                    FireSubPhase::MaximSecondAndHowitzer,
                    FireKind::MaximSecondFire | FireKind::Howitzer
                )
        );
        if !kind_ok {
            return Err(RuleError::WrongPhase);
        }

        // §6.42: the Maxim Second Fire and Howitzer Subphase is restricted to
        // Maxim guns and Howitzer-class units -- no other weapon may fire here
        // even if the FireKind were miscategorised.  Named gunboats (§6.64)
        // carry howitzers even though their profile weapon is Artillery.
        let is_named_gunboat = matches!(
            unit.profile.identity,
            crate::UnitIdentity::AngloEgyptianGunboat(gb) if gb.has_howitzer()
        );
        if sub == FireSubPhase::MaximSecondAndHowitzer
            && !matches!(
                unit.profile.weapon,
                WeaponClass::Maxims | WeaponClass::Howitzer
            )
            && !is_named_gunboat
        {
            return Err(RuleError::WrongWeaponForSubphase(firer));
        }

        // Weapon class must permit the chosen kind.  Named gunboats may fire
        // howitzer despite carrying Artillery on their profile.
        match kind {
            FireKind::Howitzer
                if unit.profile.weapon != WeaponClass::Howitzer && !is_named_gunboat =>
            {
                return Err(RuleError::OnlyHowitzerMayFireHowitzer(firer));
            }
            FireKind::MaximSecondFire if unit.profile.weapon != WeaponClass::Maxims => {
                return Err(RuleError::OnlyMaximSecondFire(firer));
            }
            _ => {}
        }
        // §6.64: no howitzer fire at night.
        if kind == FireKind::Howitzer && self.day_night == DayNight::Night {
            return Err(RuleError::NoHowitzerAtNight);
        }

        if unit.state.disrupted {
            return Err(RuleError::Disrupted(firer));
        }
        // §5.3/§6.53: units constructing a zariba or committed to a
        // demolition "may neither fire offensively nor melee attack" that
        // turn (defensive fire stays legal).
        if matches!(self.phase, Phase::OffensiveFire(_))
            && (unit.state.constructing_zariba || unit.state.demolishing)
        {
            return Err(RuleError::BusyWithEngineering(firer));
        }
        if unit.profile.fire.is_none() {
            return Err(RuleError::NoFireFactor(firer));
        }
        if self.units_fired_this_phase.contains(&firer) {
            return Err(RuleError::AlreadyFired(firer));
        }

        // §6.61/§6.62: only artillery (or howitzer) may fire at a gunboat or
        // fort. Check it here so the app pre-blocks the shot rather than the
        // engine rejecting it after the fact.
        let target_units: Vec<UnitId> = self
            .player_units_in_hex(target_hex, unit.profile.identity.owner().opponent())
            .iter()
            .map(|u| u.id)
            .collect();
        if self.special_fire_target(&target_units).is_some()
            && !matches!(
                unit.profile.weapon,
                WeaponClass::Artillery | WeaponClass::Howitzer
            )
        {
            return Err(RuleError::ArtilleryOnlyVsGunboatOrFort(firer));
        }

        // §6.15: fire may only target *enemy-occupied* hexes. Without this
        // gate a click on a friendly or empty hex passes every other check
        // and resolves the CRT against whoever (if anyone) is there.
        if target_units.is_empty() {
            return Err(RuleError::FireTargetNotEnemyOccupied);
        }

        let range = HexDistance(unit.position.distance(target_hex) as u16);
        // Named gunboats (§6.64) carry Artillery on their profile but fire
        // howitzers in the second subphase; the howitzer CRT line applies.
        let effective_weapon = effective_fire_weapon(unit, kind);
        // §6.52/§9.343: the table this unit fires on (per firer, shared with
        // `resolve_fire_attack` so validation and resolution agree on range).
        let table_player = range_table_player_for(self.scenario, unit);
        // §8.1: at night, "all fire ranges are halved (round down, but range 1
        // stays range 1)." The correct interpretation (verified against the
        // rulebook's worked AE-rifle example: doubled@1, normal@2, out@3+) is
        // to halve the weapon's *maximum* range, then consult the day table at
        // the *physical* distance. Halving the distance and consulting the day
        // table at that reduced distance collapses too many bands.
        let effective_range = if self.day_night == DayNight::Night {
            match night_capped_distance(effective_weapon, table_player, range) {
                Some(capped) => capped, // consult day table at the physical distance
                None => {
                    return Err(RuleError::OutOfRangeAtNight {
                        firer: unit.position,
                        target: target_hex,
                    });
                }
            }
        } else {
            range
        };
        let band = range_band_for(
            self.scenario,
            table_player,
            effective_weapon,
            effective_range,
        );
        if !band.in_range() {
            return Err(RuleError::TargetOutOfRange {
                firer: unit.position,
                target: target_hex,
            });
        }

        // §6.21 / §6.3: line of sight. The engine derives LOS from
        // `self.board` (populated at game start from the board annotations)
        // so it can validate fire legality without app-side help. Howitzer
        // fire bypasses LOS (§6.64).
        //
        // The firer and target LOS levels are computed with notes (b) and
        // (c): gunboats → Rough, forts → Ground, walled-city-wall-adjacent
        // units → Rough. The "Units" blocker excludes gunboats and forts
        // (note a).
        let firer_los_level =
            crate::los_table::los_level_for_unit(unit.profile.kind, unit.position, &self.board);
        let target_los_level = self
            .units
            .iter()
            .find(|u| u.position == target_hex)
            .map(|u| crate::los_table::los_level_for_unit(u.profile.kind, u.position, &self.board))
            .unwrap_or_else(|| {
                self.board
                    .terrain_at(target_hex)
                    .map(crate::los_table::los_level)
                    .unwrap_or(crate::los_table::LosLevel::Ground)
            });
        if !crate::los_table::has_los(
            &self.board,
            unit.position,
            target_hex,
            kind,
            firer_los_level,
            target_los_level,
            self.los_unit_blocker(),
            |a, b| self.wall_is_breached(a, b),
        ) {
            return Err(RuleError::LineOfSightBlocked(unit.position, target_hex));
        }
        Ok(())
    }

    /// Read-only validation for §6.63 artillery-fire wall breaching. The firer
    /// must:
    ///   - exist,
    ///   - belong to the side whose turn it is to fire (active player on
    ///     offensive, opponent on defensive),
    ///   - be artillery- or howitzer-class (§6.63 "only artillery"),
    ///   - not be disrupted,
    ///   - have a printed fire factor,
    ///   - not have already fired this phase,
    ///   - be within range of the *nearer* endpoint of the wall hexside,
    ///     respecting the §8.1 night cap,
    ///   - have line of sight to the nearer endpoint.
    ///
    /// On success returns `(fire_factor, effective_range, nearer_endpoint)`.
    /// The caller is responsible for summing per-firer factors with the
    /// range band and resolving the CRT — this method only validates one
    /// firer at a time.
    pub fn can_fire_at_wall(
        &self,
        firer: UnitId,
        target: HexsideRef,
    ) -> Result<(FireFactor, HexDistance, HexCoord), RuleError> {
        let unit = self.unit_or_err(firer)?;

        let firing_player = self.fire_phase_player()?;
        if unit.profile.identity.owner() != firing_player {
            return Err(RuleError::NotYourTurn);
        }
        if !matches!(
            unit.profile.weapon,
            WeaponClass::Artillery | WeaponClass::Howitzer
        ) {
            return Err(RuleError::OnlyArtilleryMayBreachWall(firer));
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(firer));
        }
        let Some(fire_factor) = unit.profile.fire else {
            return Err(RuleError::NoFireFactor(firer));
        };
        if self.units_fired_this_phase.contains(&firer) {
            return Err(RuleError::AlreadyFired(firer));
        }

        // Range to the wall = distance to the nearer endpoint.
        let da = unit.position.distance(target.a);
        let db = unit.position.distance(target.b);
        let nearer_hex = if da <= db { target.a } else { target.b };
        let range = HexDistance(da.min(db) as u16);

        let effective_range = if self.day_night == DayNight::Night {
            let night_max = crate::range_effects::night_max_range(
                unit.profile.weapon,
                firing_player == Player::AngloEgyptian,
            );
            if range.value() > night_max as u16 {
                return Err(RuleError::OutOfRangeAtNight {
                    firer: unit.position,
                    target: nearer_hex,
                });
            }
            range
        } else {
            range
        };

        let band = range_band_for(
            self.scenario,
            firing_player,
            unit.profile.weapon,
            effective_range,
        );
        if !band.in_range() {
            return Err(RuleError::TargetOutOfRange {
                firer: unit.position,
                target: nearer_hex,
            });
        }

        // §6.3 LOS to the wall's nearer endpoint.
        let firer_los =
            crate::los_table::los_level_for_unit(unit.profile.kind, unit.position, &self.board);
        let target_los = self
            .board
            .terrain_at(nearer_hex)
            .map(crate::los_table::los_level)
            .unwrap_or(crate::los_table::LosLevel::Ground);
        if !crate::los_table::has_los(
            &self.board,
            unit.position,
            nearer_hex,
            FireKind::Direct,
            firer_los,
            target_los,
            self.los_unit_blocker(),
            |a, b| self.wall_is_breached(a, b),
        ) {
            return Err(RuleError::LineOfSightBlocked(unit.position, nearer_hex));
        }
        Ok((fire_factor, effective_range, nearer_hex))
    }

    /// Read-only check of whether `attacker` may melee-attack the adjacent
    /// `defender_hex` in the current state (§7): Melee phase, attacker is the
    /// active player, attacker is a melee-capable kind (§7.4), not disrupted,
    /// adjacent to the target, the target hex holds at least one enemy unit
    /// that may be melee-attacked (gunboats may not -- §7.1), and no wall or
    /// thorn-hedge hexside blocks the attack (§7.2).
    pub fn can_melee(&self, attacker: UnitId, defender_hex: HexCoord) -> Result<(), RuleError> {
        let unit = self.unit_or_err(attacker)?;

        if !matches!(self.phase, Phase::Melee) {
            return Err(RuleError::WrongPhase);
        }
        if self.active_player != unit.profile.identity.owner() {
            return Err(RuleError::NotYourTurn);
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(attacker));
        }
        // A unit that recovered from disruption early (`RecoverUnit`) is
        // still spent for this turn: recovery happens at the *end* of the
        // owning player's turn (reference notes), after all melee.
        if self.turn_events.iter().any(|e| {
            matches!(e, crate::turn_summary::TurnEventRecord::UnitRecovered { unit } if *unit == attacker)
        }) {
            return Err(RuleError::Disrupted(attacker));
        }
        // §5.3/§6.53: constructing or demolishing units may not melee attack.
        if unit.state.constructing_zariba || unit.state.demolishing {
            return Err(RuleError::BusyWithEngineering(attacker));
        }
        if !unit.profile.kind.may_melee_attack() {
            return Err(RuleError::KindMayNotMelee(attacker));
        }
        if !unit.position.neighbors().contains(&defender_hex) {
            return Err(RuleError::TargetNotAdjacent {
                from: unit.position,
                to: defender_hex,
            });
        }
        let enemy = unit.profile.identity.owner().opponent();
        let has_target = self.units.iter().any(|u| {
            u.position == defender_hex
                && u.profile.identity.owner() == enemy
                && u.profile.kind.may_be_melee_attacked()
        });
        if !has_target {
            return Err(RuleError::NoMeleeableEnemy(defender_hex));
        }
        // §7.2: walls and thorn-hedges block melee across them (gates and
        // breaches pass). Read through `hexside_effective` so a §6.63 breach
        // is an opening.
        if self.hexside_effective_is(unit.position, defender_hex, HexsideKind::blocks_melee) {
            return Err(RuleError::MeleeBlockedByHexside(
                unit.position,
                defender_hex,
            ));
        }
        Ok(())
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

    /// Whether the unit `mover` may legally end its move stacked in `dest` given
    /// the units already there (§5.51-5.53). Stacking is checked only at the end
    /// of a move (§5.51), so this evaluates the resulting stack: every non-mover
    /// already in `dest` plus `mover`.
    ///
    /// * §5.51 -- at most four units per hex, *excluding* free-stacking leaders
    ///   and gunboats; gunboats may not share a hex with any other unit; and no
    ///   unit may share a hex with *enemy* units (§7.1) except a lone
    ///   Anglo-Egyptian leader (§6.51).
    /// * §5.52 -- units of different Dervish tribes (or stacking groups -- the
    ///   Dervish artillery is its own group) may not stack together.
    /// * §5.53 -- a Dervish leader may stack only with units of its command.
    pub fn check_stacking(
        &self,
        mover: &UnitPlacement,
        dest: HexCoord,
    ) -> Result<(), crate::StackingError> {
        // The prospective occupants: everyone already in `dest` except the
        // mover itself, plus the mover.
        let occupants: Vec<&UnitPlacement> = self
            .units
            .iter()
            .filter(|u| u.position == dest && u.id != mover.id)
            .chain(std::iter::once(mover))
            .collect();
        stacking_rule(&occupants)
    }

    /// Whole-state stacking invariant check (§5.51-5.53): every occupied hex
    /// must satisfy the stacking law ([`stacking_rule`]) on its *actual*
    /// occupants. Unlike [`Self::check_stacking`] this is not a prospective-move
    /// check — it validates the state as it stands, so it can be used as a
    /// post-condition after any mutation (see `apply_effect`) and to audit
    /// replayed records. Delegates to the same [`stacking_rule`] as
    /// [`Self::check_stacking`], so the prospective and whole-state views of
    /// the law cannot drift.
    pub fn validate_stacking_invariants(&self) -> Result<(), String> {
        // Ordered map: this is a grouping helper, and keeping `GameState`'s
        // reachable code free of `hashbrown` keeps it verifiable (see the
        // `verification` module).
        let mut by_hex: BTreeMap<HexCoord, Vec<&UnitPlacement>> = BTreeMap::new();
        for u in &self.units {
            by_hex.entry(u.position).or_default().push(u);
        }
        for (hex, occupants) in by_hex {
            stacking_rule(&occupants).map_err(|e| format!("{hex:?}: {e}"))?;
        }
        Ok(())
    }

    /// Whether `unit` projects a zone of control over a mover of
    /// `mover_kind` belonging to `mover_player` (§5.41, §5.44).
    ///
    /// * A disrupted unit projects no ZOC.
    /// * Anglo-Egyptian leaders project no ZOC.
    /// * Gunboats project ZOC only against enemy gunboats.
    ///
    /// Returns the [`ZocReason`] when ZOC applies, else `None`. The hexside
    /// subtleties (walls/gates/khor/forts/Zariba block or redirect ZOC --
    /// §5.44) need the game map, which the engine does not hold; the app layers
    /// those on top. This is the position/kind/disruption core of the rule.
    pub fn unit_projects_zoc(
        &self,
        unit: &UnitPlacement,
        mover_player: Player,
        mover_kind: UnitKind,
    ) -> Option<ZocReason> {
        unit_projects_zoc_rule(unit, mover_player, mover_kind)
    }

    /// Whether `hex` lies in a zone of control exerted by a unit hostile to a
    /// mover of `mover_kind` belonging to `mover_player` (§5.41, §5.44). A unit
    /// moving into such a hex must stop there and may move no further that turn
    /// (§5.26, §5.43).
    ///
    /// Applies the §5.44 hexside exceptions using the attached board: a ZOC does
    /// not extend across a khor/wall/Zariba hexside, and (except for gunboats)
    /// does not extend into or out of a Nile hex. With no board loaded these
    /// reduce to the plain adjacency rule.
    pub fn hex_in_enemy_zoc(
        &self,
        hex: HexCoord,
        mover_player: Player,
        mover_kind: UnitKind,
    ) -> bool {
        self.units.iter().any(|u| {
            if self
                .unit_projects_zoc(u, mover_player, mover_kind)
                .is_none()
            {
                return false;
            }
            if !u.position.neighbors().contains(&hex) {
                return false;
            }
            // §5.44: ZOC does not cross a khor/wall/Zariba hexside (read
            // through `hexside_effective` so a §6.63 breach no longer blocks).
            if self.hexside_effective_is(u.position, hex, omdurman_types::HexsideKind::blocks_zoc) {
                return false;
            }
            // §5.44: ZOC does not extend into or out of a Nile hex (exception:
            // gunboats, §5.41 -- already gated by `unit_projects_zoc`).
            if !matches!(u.profile.kind, UnitKind::Gunboat { .. })
                && (self.board.is_nile(u.position) || self.board.is_nile(hex))
            {
                return false;
            }
            true
        })
    }

    /// Compute the set of hexes that a given unit projects a zone of control
    /// into (§5.41, §5.44). Returns the 6 adjacent hexes minus exclusions.
    ///
    /// This is a pure function — it computes the ZOC footprint without
    /// side effects. `hex_in_enemy_zoc` checks whether *any* hostile unit's
    /// ZOC covers a given hex; this function returns *which* hexes a
    /// specific unit covers.
    pub fn zoc_hexes(
        &self,
        unit: &UnitPlacement,
        mover_player: Player,
        mover_kind: UnitKind,
    ) -> Vec<HexCoord> {
        let Some(reason) = self.unit_projects_zoc(unit, mover_player, mover_kind) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for &adj in &unit.position.neighbors() {
            // §5.44: ZOC does not cross a khor/wall/Zariba hexside (read
            // through `hexside_effective` so a §6.63 breach no longer blocks).
            if self.hexside_effective_is(
                unit.position,
                adj,
                omdurman_types::HexsideKind::blocks_zoc,
            ) {
                continue;
            }
            // §5.44: ZOC does not extend into or out of a Nile hex
            // (exception: gunboats, §5.41 — gated by `unit_projects_zoc`).
            if !matches!(reason, ZocReason::GunboatVsGunboat)
                && (self.board.is_nile(unit.position) || self.board.is_nile(adj))
            {
                continue;
            }
            result.push(adj);
        }
        result
    }

    /// The hex a howitzer shell actually lands in given its scatter entry
    /// (§6.64). The printed Scattergram is a flower of six hexes around the
    /// designated target; this orients it relative to the firer: "upper"
    /// entries flank the away-from-firer direction (over-shoot), "lower"
    /// entries flank the toward-firer direction (fall-short), and left/right
    /// are the perpendicular sides. Each miss roll (1-6) thus lands on a
    /// distinct, deterministic neighbour; rolls 7-10 (`Center`) hit the
    /// designated hex.
    pub(crate) fn howitzer_impact_hex(
        &self,
        target: HexCoord,
        firer: Option<HexCoord>,
        scatter: ScatterHexDirection,
    ) -> HexCoord {
        use ScatterHexDirection as S;
        let neighbors = target.neighbors();
        // Bearing from target toward the firer (0 when unknown).
        let base = firer.map_or(0, |f| toward_index(target, f));
        let ring = |offset: usize| neighbors[(base + offset) % 6];
        // Upper half = the away-from-firer side of the flower (over-shoots),
        // lower half = the near side (fall-short), laterals in between. Each
        // of the six miss rolls lands on a distinct neighbour.
        match scatter {
            S::Center => target,
            S::UpperLeft => ring(2),
            S::UpperRight => ring(3),
            S::Right => ring(1),
            S::LowerRight => ring(0),
            S::LowerLeft => ring(5),
            S::Left => ring(4),
        }
    }

    /// If `target_ids` contains a gunboat or fort, return it, its kind, and
    /// the elimination threshold its destruction needs — `3` for a gunboat
    /// (§6.61) or `2` for a fort (§6.62) — rather than the generic Combat
    /// Results Table effect. Carrying the threshold out centrally means the
    /// caller never has to map an open `UnitKind` to a number (no
    /// `unreachable!` to drift if a third kind is ever added). A gunboat is
    /// reported in preference to a fort (a gunboat never stacks, so this is
    /// unambiguous in practice).
    pub(crate) fn special_fire_target(
        &self,
        target_ids: &[UnitId],
    ) -> Option<(UnitId, UnitKind, u8)> {
        let mut fort = None;
        for &id in target_ids {
            match self.find_unit(id).map(|u| u.profile.kind) {
                Some(UnitKind::Gunboat { .. }) => {
                    return Some((
                        id,
                        UnitKind::Gunboat {
                            fire: 0,
                            upstream: 0,
                            downstream: 0,
                        },
                        3,
                    ));
                }
                Some(UnitKind::Fort { .. }) if fort.is_none() => {
                    fort = Some((id, UnitKind::Fort { fire: 0, melee: 0 }, 2))
                }
                _ => {}
            }
        }
        fort
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

/// A placement entering play (setup deployment §9.2/§9.3, reinforcement
/// §9.112/§9.113) must be the physical counter it names: the canonical
/// profile from `unit_profiles::profile_for_unit` and a fresh
/// [`UnitState`](crate::UnitState). A peer may not invent unit values (a
/// 99-factor infantry, a pre-loaded or engine-less gunboat).
pub(crate) fn require_canonical_placement(p: &UnitPlacement) -> Result<(), RuleError> {
    let canonical = crate::unit_profiles::profile_for_unit(p.id);
    if canonical != Some(p.profile) || p.state != crate::UnitState::default() {
        return Err(RuleError::NonCanonicalPlacement(p.id));
    }
    Ok(())
}

/// The stacking law (§5.51-5.53) evaluated over an explicit list of
/// `occupants` of one hex. Pure and stateless, so [`GameState::check_stacking`]
/// (the prospective move/deploy check) and
/// [`GameState::validate_stacking_invariants`] (the whole-state post-condition)
/// are the same function over different occupant sources -- the two views
/// cannot drift again.
///
/// * §5.51: at most `STACKING_LIMIT` counted units (leaders and gunboats
///   free); gunboats share a hex with nothing (§5.21 transport is modelled
///   separately); and no enemy cohabitation -- engaging the enemy is what
///   melee is for (§7.1). The lone exception is an Anglo-Egyptian leader,
///   who never blocks an enemy stack (§6.51: a Dervish unit occupying his
///   hex eliminates him; §9.346 makes this how GORDON dies).
/// * §5.52: Dervish units of different stacking groups (tribes, plus the
///   artillery as its own group) may not share a hex.
/// * §5.53: a Dervish leader stacks only with units of its command.
pub fn stacking_rule(occupants: &[&UnitPlacement]) -> Result<(), crate::StackingError> {
    use crate::StackingError;

    // §5.51: no enemy cohabitation. Two units of opposite factions may share
    // a hex only while the §6.51 exception applies: an Anglo-Egyptian leader
    // is never a *blocker* (a Dervish unit arriving on his hex eliminates
    // him -- §9.346 is how GORDON dies), so any opposite-owner pair with at
    // least one non-leader on *both* sides is illegal (§7.1).
    for (i, a) in occupants.iter().enumerate() {
        for b in &occupants[i + 1..] {
            let mixed_factions = a.profile.identity.owner() != b.profile.identity.owner();
            let neither_is_ae_leader = !matches!(a.profile.kind, UnitKind::BritishLeader { .. })
                && !matches!(b.profile.kind, UnitKind::BritishLeader { .. });
            if mixed_factions && neither_is_ae_leader {
                return Err(StackingError::EnemyCohabitation);
            }
        }
    }

    // §5.51: gunboats may not stack with anything (Friendlies transport,
    // §5.21, is modelled separately and not via a normal move).
    let gunboats = occupants
        .iter()
        .filter(|u| matches!(u.profile.kind, UnitKind::Gunboat { .. }))
        .count();
    if gunboats > 0 && occupants.len() > 1 {
        return Err(StackingError::GunboatStack);
    }

    // §5.51: the four-unit limit counts neither leaders nor gunboats.
    let counted = occupants
        .iter()
        .filter(|u| {
            !matches!(
                u.profile.kind,
                UnitKind::DervishLeader { .. }
                    | UnitKind::BritishLeader { .. }
                    | UnitKind::Gunboat { .. }
            )
        })
        .count();
    if counted > STACKING_LIMIT {
        return Err(StackingError::OverLimit);
    }

    // §5.52: no two different Dervish stacking groups (tribes, plus the
    // artillery as its own group) in the same hex.
    let mut seen_group: Option<crate::DervishStackingGroup> = None;
    for u in occupants {
        if let Some(group) = u.profile.identity.dervish_stacking_group() {
            match seen_group {
                Some(seen) if seen != group => return Err(StackingError::DervishTribeMix),
                _ => seen_group = Some(group),
            }
        }
    }

    // §5.53: a Dervish leader may only stack with units of its command.
    for u in occupants {
        if let crate::UnitIdentity::DervishLeader(leader) = u.profile.identity {
            let bad = occupants.iter().any(|other| {
                matches!(
                    other.profile.identity,
                    crate::UnitIdentity::DervishTribal { tribe } if !leader.commands(tribe)
                )
            });
            if bad {
                return Err(StackingError::DervishLeaderCommandMismatch);
            }
        }
    }

    Ok(())
}

/// Whether `unit` projects a zone of control over a mover of `mover_kind`
/// belonging to `mover_player` (§5.41, §6.51). Pure and stateless: the
/// hexside subtleties (§5.44) need the board and live in
/// [`GameState::hex_in_enemy_zoc`] / [`GameState::zoc_hexes`], which call this
/// as their per-unit core. Extracted as a free function (like
/// [`stacking_rule`]) so the Kani harnesses can verify it without
/// constructing a `GameState`.
///
/// * A disrupted unit projects no ZOC (§5.41).
/// * Friendly units never project ZOC on each other.
/// * Anglo-Egyptian leaders exert no ZOC (§5.41, §6.51).
/// * Gunboats project ZOC *only* against enemy gunboats (§5.41).
/// * A fort projects ZOC out of its hex even when unoccupied (§5.44),
///   modelled by the fort unit projecting normally.
pub fn unit_projects_zoc_rule(
    unit: &UnitPlacement,
    mover_player: Player,
    mover_kind: UnitKind,
) -> Option<ZocReason> {
    if unit.state.disrupted {
        return None;
    }
    if unit.profile.identity.owner() == mover_player {
        return None;
    }
    match unit.profile.kind {
        // §6.51: Anglo-Egyptian leaders exert no ZOC.
        UnitKind::BritishLeader { .. } => None,
        // §5.41: gunboats project ZOC *only* against enemy gunboats.
        UnitKind::Gunboat { .. } => {
            matches!(mover_kind, UnitKind::Gunboat { .. }).then_some(ZocReason::GunboatVsGunboat)
        }
        // §5.44: a fort projects ZOC out of its hex even when unoccupied;
        // that is modelled by the fort *unit* itself projecting normally.
        UnitKind::Fort { .. } => Some(ZocReason::Fort),
        _ => Some(ZocReason::Normal),
    }
}

/// A Fall-of-Khartoum order-of-battle slot group (§9.321/§9.322): the
/// manual counts by type and nationality, not by exact counter -- "two
/// British infantry units" binds across all British battalions whatever
/// their ordinal, "two old style gunboats" across the four old boat
/// counters. Counting exact identities would let one of each variant in.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FokCapGroup {
    Tribe(DervishTribe),
    DervishArtillery,
    DervishFort,
    OldGunboat,
    AeArtillery,
    Infantry(crate::BrigadeNationality),
    Gordon,
}

/// Which FoK slot group `identity` belongs to (`None`: not in the order of
/// battle at all), and how many counters of that group may deploy.
pub fn fok_cap_group(identity: &crate::UnitIdentity) -> Option<(FokCapGroup, usize)> {
    use crate::UnitIdentity;
    use FokCapGroup::*;
    Some(match identity {
        // §9.322: "32 Mulazmin units ... 2 Hadendowa; 6 Kehena; 5 Degheim
        // ... 3 Dervish artillery units" (the Mulazmin are the two green
        // print runs, 16 + 16). The Kehena and Degheim counters are the
        // "Deghelim" cells of the Ali_Wad_Helu block (see
        // `unit_profiles::ali_wad_helu`): row 1 resolves to Kehena, row 0
        // cols 1-5 to Degheim -- keying this table by *identity* therefore
        // reaches both forces through their cut sprites.
        UnitIdentity::DervishTribal {
            tribe: crate::DervishTribe::Mulazmin,
        } => (Tribe(crate::DervishTribe::Mulazmin), 32),
        UnitIdentity::DervishTribal {
            tribe: crate::DervishTribe::Hadendowa,
        } => (Tribe(crate::DervishTribe::Hadendowa), 2),
        UnitIdentity::DervishTribal {
            tribe: crate::DervishTribe::Kehena,
        } => (Tribe(crate::DervishTribe::Kehena), 6),
        UnitIdentity::DervishTribal {
            tribe: crate::DervishTribe::Degheim,
        } => (Tribe(crate::DervishTribe::Degheim), 5),
        UnitIdentity::DervishArtillery => (DervishArtillery, 3),
        // §9.321: "Two old style (unnamed) gunboats", "one Egyptian
        // Battalion artillery unit", "two British infantry units", "three
        // Egyptian infantry units", "four Sudan infantry units", "four
        // 'Friendlies' units" -- any counter of the group may stand in.
        UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(_)) => (OldGunboat, 2),
        UnitIdentity::AngloEgyptianArtillery => (AeArtillery, 1),
        UnitIdentity::AngloEgyptianInfantry {
            brigade: crate::BrigadeId { nationality, .. },
            ..
        } => (
            Infantry(*nationality),
            match *nationality {
                crate::BrigadeNationality::British => 2,
                crate::BrigadeNationality::Egyptian => 3,
                crate::BrigadeNationality::Sudanese => 4,
                crate::BrigadeNationality::Friendlies => 4,
            },
        ),
        // §9.321: GORDON starts in the palace (the scenario's one leader).
        UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Gordon) => (Gordon, 1),
        // §9.344: the single North Fort is the only Dervish fort in play.
        UnitIdentity::DervishFort => (DervishFort, 1),
        _ => return None,
    })
}

/// Maximum units per hex (§5.51), excluding free-stacking leaders/gunboats.
pub(crate) const STACKING_LIMIT: usize = 4;

/// Maximum river mines a player may lay (§10.11).
pub const MAX_MINES: usize = 2;

/// Maximum contiguous Nile hexes the river chain may span (§10.21).
pub const MAX_CHAIN_HEXES: usize = 4;

// ---------------------------------------------------------------------------
// 8b) Retreat before melee / advance after combat
// ---------------------------------------------------------------------------

impl GameState {
    /// Read-only check of whether `unit_id` may retreat two hexes to `to`
    /// before an impending infantry melee (§7.5): Melee phase, cavalry/camel
    /// kind, not disrupted, not already moved/retreated this turn, `to` exactly
    /// two hexes away and empty. (Does not verify the attacker is infantry --
    /// the caller offers the retreat only in response to one.)
    pub fn can_retreat_before_melee(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if !matches!(self.phase, Phase::Melee) {
            return Err(RuleError::WrongPhase);
        }
        // Retreat is a *reaction* to a declared *infantry* melee attack on the
        // unit's hex (§7.5): there must be a pending melee targeting where it
        // stands, made by at least one infantry attacker.
        match &self.pending_melee {
            Some(p)
                if p.attack.defender_hex == unit.position
                    && p.attack.attackers.iter().any(|id| {
                        self.find_unit(*id)
                            .is_some_and(|u| matches!(u.profile.kind, UnitKind::Infantry { .. }))
                    }) => {}
            _ => {
                return Err(RuleError::NoInfantryMeleeThreatens(unit_id));
            }
        }
        if !unit.profile.kind.may_retreat_before_melee() {
            return Err(RuleError::MayNotRetreatBeforeMelee(unit_id));
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit_id));
        }
        if self.mp_spent(unit_id) > 0 {
            return Err(RuleError::AlreadyMoved(unit_id));
        }
        if unit.position.distance(to) != 2 {
            return Err(RuleError::RetreatMustBeTwoHexes);
        }
        if self.units.iter().any(|u| u.position == to) {
            return Err(RuleError::RetreatHexOccupied(to));
        }
        // §5.22: a retreating unit must stay on the board (with no board
        // loaded, map constraints don't apply).
        if !self.board.terrain.is_empty() && self.board.terrain_at(to).is_none() {
            return Err(RuleError::OffBoard(to));
        }
        // §5.22: land units may *never* enter a Nile hex -- a retreat is no
        // exception (a cavalry retiring two hexes onto the river is not a
        // legal move).
        if !unit.profile.kind.is_boat() && self.board.is_nile(to) {
            return Err(RuleError::LandIntoNile(to));
        }
        // §6.54: a retreat may not end on an enemy fort -- players may not
        // occupy an enemy fort under any circumstances.
        if self.hex_has_enemy_fort(to, unit.profile.identity.owner()) {
            return Err(RuleError::EnemyFort(to));
        }
        // §5.23: movement may not cross a wall hexside except through a gate
        // or breach -- a retreat is no exception. A two-hex retreat passes
        // through one of the (at most two) common neighbours of `from` and
        // `to`; at least one intermediate must have both legs non-wall.
        let wall_free_path = unit.position.neighbors().iter().any(|mid| {
            mid.neighbors().contains(&to)
                && self.hexside_effective(unit.position, *mid) != Some(HexsideKind::Wall)
                && self.hexside_effective(*mid, to) != Some(HexsideKind::Wall)
        });
        if !wall_free_path {
            return Err(RuleError::RetreatBlockedByWall(unit.position, to));
        }
        Ok(())
    }

    /// Read-only check of whether `unit_id` may advance after combat into the
    /// vacated `to` hex (§6.82, §7.6): a fire or melee phase, the active
    /// player's unit, not artillery, adjacent to `to`, and `to` now empty.
    /// Wall/khor hexside restrictions are not enforced (no hexside map data).
    pub fn can_advance_after_combat(&self, unit_id: UnitId, to: HexCoord) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        // §6.7: there is no advance after combat as a result of defensive fire.
        // Advance is permitted only after melee (§7.6) and offensive fire
        // (§6.82) -- never in a defensive-fire subphase.
        if !matches!(self.phase, Phase::Melee | Phase::OffensiveFire(_)) {
            return Err(RuleError::WrongPhase);
        }
        if matches!(unit.profile.kind, UnitKind::Artillery { .. }) {
            return Err(RuleError::ArtilleryMayNotAdvance(unit_id));
        }
        // §5.25: "Dervish forts may not move in any way once placed" -- an
        // advance-after-combat is movement.
        if matches!(unit.profile.kind, UnitKind::Fort { .. }) {
            return Err(RuleError::FortMayNotAdvance(unit_id));
        }
        if !unit.position.neighbors().contains(&to) {
            return Err(RuleError::AdvanceNotAdjacent);
        }
        // §6.82/§7.6: the hex must have been vacated by combat this phase --
        // an advance answers the attack that emptied it, so merely-empty
        // hexes are not advance targets (this is what stops advance-after-
        // combat being used as free out-of-phase movement).
        let eligible = self
            .vacated_by_combat
            .get(&to)
            .ok_or(RuleError::HexNotVacatedByCombat(to))?;
        // §6.82/§7.6: "the friendly units must have participated in the
        // attack" -- only listed participants may advance.
        if !eligible.contains(&unit_id) {
            return Err(RuleError::UnitDidNotParticipate(unit_id, to));
        }
        // §5.22: a unit may only advance into a hex it could occupy -- boats
        // stay on the Nile, land units stay off it, and nobody advances off
        // the board (with no board loaded, map constraints don't apply).
        if matches!(unit.profile.kind, UnitKind::Gunboat { .. }) {
            if !self.board.terrain.is_empty() && !self.board.is_nile(to) {
                return Err(RuleError::GunboatOffNile(to));
            }
        } else {
            if !self.board.terrain.is_empty() && self.board.terrain_at(to).is_none() {
                return Err(RuleError::OffBoard(to));
            }
            if self.board.is_nile(to) {
                return Err(RuleError::LandIntoNile(to));
            }
        }
        // §6.54: may not advance after combat into an enemy fort, even if the
        // fort is unoccupied (a fort is never captured -- only destroyed).
        if self.hex_has_enemy_fort(to, unit.profile.identity.owner()) {
            return Err(RuleError::EnemyFort(to));
        }
        if self.units.iter().any(|u| u.position == to) {
            return Err(RuleError::AdvanceNotVacant(to));
        }
        // §6.82 / §7.6: may not advance across a wall (except gate/breach),
        // khor, or thorn-hedge hexside. Read through `hexside_effective` so a
        // §6.63 breach is an opening.
        if self.hexside_effective_is(unit.position, to, HexsideKind::blocks_advance_after_combat) {
            return Err(RuleError::AdvanceBlockedByHexside(unit.position, to));
        }
        Ok(())
    }

    /// Read-only check of whether `unit_id` may recover from disruption
    /// (paired with [`apply_recover_unit`]). Per the reference notes,
    /// disrupted units "are turned face up at the end of the owning player's
    /// turn" -- `end_player_turn` does that automatically. An explicit
    /// `RecoverUnit` is therefore only the owner turning the counter a little
    /// early, at the end of their turn: the unit must be disrupted and the
    /// active player's own, in the Melee phase (the last phase of the player
    /// turn) with no declared melee pending. A unit so recovered still may
    /// not melee this turn (see [`Self::can_melee`]).
    pub fn can_recover_unit(&self, unit_id: UnitId) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if !unit.state.disrupted {
            return Err(RuleError::NotDisrupted(unit_id));
        }
        if unit.profile.identity.owner() != self.active_player {
            return Err(RuleError::NotYourTurn);
        }
        if !matches!(self.phase, Phase::Melee) {
            return Err(RuleError::WrongPhase);
        }
        if self.pending_melee.is_some() {
            return Err(RuleError::MeleeAlreadyPending);
        }
        Ok(())
    }

    /// Read-only check of whether a Royal Engineers demolition may begin
    /// (§6.53): the unit is the Royal Engineers, it is its owner's Movement
    /// phase ("the Royal Engineers must move adjacent ... and end their
    /// movement adjacent"), and it is undisrupted and not already committed
    /// to a demolition or a zariba construction. The target itself must be
    /// one of [`Self::demolition_targets`] (checked by the effect).
    pub fn can_demolition(&self, unit_id: UnitId) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if unit.profile.identity != crate::UnitIdentity::RoyalEngineers {
            return Err(RuleError::NotRoyalEngineers(unit_id));
        }
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        if unit.profile.identity.owner() != self.active_player {
            return Err(RuleError::NotYourTurn);
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit_id));
        }
        if unit.state.demolishing || unit.state.constructing_zariba {
            return Err(RuleError::BusyWithEngineering(unit_id));
        }
        Ok(())
    }

    /// Read-only discovery of the demolition targets adjacent to `unit_id`
    /// (§6.53): *enemy* fort units in the six neighbouring hexes plus standing
    /// Wall hexsides on the six neighbouring sides. Pairs with [`GameState::can_demolition`]
    /// and [`GameEffect::Demolition`] so the UI can offer exactly the targets
    /// the rules would accept. Empty when the unit doesn't exist or has no
    /// adjacent target.
    pub fn demolition_targets(&self, unit_id: UnitId) -> Vec<DemolitionTarget> {
        let Ok(unit) = self.unit_or_err(unit_id) else {
            return Vec::new();
        };
        let mut targets = Vec::new();
        for n in unit.position.neighbors() {
            if let Some(fort) = self.units.iter().find(|u| {
                u.position == n
                    && matches!(u.profile.kind, UnitKind::Fort { .. })
                    && u.profile.identity.owner() != unit.profile.identity.owner()
            }) {
                targets.push(DemolitionTarget::Fort(fort.id));
            }
            if self.hexside_effective_is(unit.position, n, |k| k == HexsideKind::Wall) {
                targets.push(DemolitionTarget::WallHexside(HexsideRef::new(
                    unit.position,
                    n,
                )));
            }
        }
        targets
    }

    /// The next Friendlies-transport action the rules would accept (§5.21),
    /// given the locally selected unit -- `None` when no transport action is
    /// available. During the load turn the offer requires a friendly-suitable
    /// selected unit (undisrupted, not building a zariba or demolishing,
    /// `is_friendlies`) adjacent to a same-side gunboat, with the Isa Zachneih
    /// already eliminated; later turns follow the transport state machine
    /// regardless of selection. Pairs with [`GameEffect::FriendliesTransport`]
    /// so the UI can offer exactly the action the engine would accept.
    pub fn friendlies_transport_offer(&self, selected: Option<UnitId>) -> Option<FriendliesAction> {
        match self.friendlies_transport {
            None => {
                let unit = self.find_unit(selected?)?;
                if unit.state.disrupted || unit.state.constructing_zariba || unit.state.demolishing
                {
                    return None;
                }
                if !unit.profile.identity.is_friendlies() {
                    return None;
                }
                let gunboat = unit.position.neighbors().iter().find_map(|&n| {
                    self.units.iter().find(|u| {
                        u.position == n
                            && matches!(
                                u.profile.identity,
                                crate::UnitIdentity::AngloEgyptianGunboat(_)
                            )
                            && u.profile.identity.owner() == unit.profile.identity.owner()
                    })
                })?;
                if !self.isa_zachneih_eliminated {
                    return None;
                }
                Some(FriendliesAction::Load {
                    unit: unit.id,
                    gunboat: gunboat.id,
                })
            }
            Some(TransportState::Loaded { unit, gunboat }) => {
                let to = self
                    .find_unit(gunboat)
                    .or_else(|| self.find_unit(unit))
                    .map(|u| u.position)
                    .unwrap_or(HexCoord::new(0, 0));
                Some(FriendliesAction::Cross { unit, gunboat, to })
            }
            Some(TransportState::Crossing { unit, gunboat, .. })
            | Some(TransportState::ReadyToDisembark { unit, gunboat }) => {
                Some(FriendliesAction::Disembark { unit, gunboat })
            }
        }
    }

    /// Read-only check of whether the given units may construct the Zariba
    /// `hexside` (§5.3): a Campaign-game option ("these hexsides are
    /// considered clear terrain in the campaign game ... the Anglo-Egyptian
    /// player may, however, ... construct this defensive position") begun in
    /// the Anglo-Egyptian Movement phase by undisrupted Anglo-Egyptian
    /// infantry that have not moved yet this turn ("begins and ends the
    /// Anglo-Egyptian player turn adjacent") and are not already building or
    /// demolishing, each on one of the hexside's two hexes. The hexside must
    /// join two adjacent hexes and carry no authored feature.
    ///
    /// Not modelled: the Nile-side restriction and the fixed mapsheet
    /// position (the campaign board authors the historical Zariba hexsides as
    /// already built), and the end-of-turn adjacency re-check.
    pub fn can_construct_zariba(
        &self,
        unit_ids: &[UnitId],
        hexside: HexsideRef,
    ) -> Result<(), RuleError> {
        if self.scenario != Scenario::Campaign {
            return Err(RuleError::IllegalZariba(
                "the Zariba is only constructed in the campaign game",
            ));
        }
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        if self.active_player != Player::AngloEgyptian {
            return Err(RuleError::NotYourTurn);
        }
        if unit_ids.is_empty() {
            return Err(RuleError::IllegalZariba("no constructing units"));
        }
        crate::effects::reject_duplicate_units(unit_ids)?;
        if !hexside.a.is_adjacent_to(hexside.b) {
            return Err(RuleError::IllegalZariba(
                "a hexside joins two adjacent hexes",
            ));
        }
        if self.board.hexside_between(hexside.a, hexside.b).is_some() {
            return Err(RuleError::IllegalZariba(
                "the hexside already carries a map feature",
            ));
        }
        for &id in unit_ids {
            let unit = self.unit_or_err(id)?;
            if unit.profile.identity.owner() != Player::AngloEgyptian {
                return Err(RuleError::NotOwner(id));
            }
            if !matches!(unit.profile.kind, UnitKind::Infantry { .. }) {
                return Err(RuleError::IllegalZariba(
                    "only Anglo-Egyptian infantry construct the Zariba",
                ));
            }
            if unit.state.disrupted {
                return Err(RuleError::Disrupted(id));
            }
            if unit.state.constructing_zariba || unit.state.demolishing {
                return Err(RuleError::BusyWithEngineering(id));
            }
            if self.mp_spent(id) > 0 {
                return Err(RuleError::AlreadyMoved(id));
            }
            if unit.position != hexside.a && unit.position != hexside.b {
                return Err(RuleError::IllegalZariba(
                    "the constructing unit must be adjacent to the hexside",
                ));
            }
        }
        Ok(())
    }

    /// Read-only check of whether a batch of reinforcement placements is legal:
    /// each placement is a canonical counter entering fresh, during its
    /// owner's Movement phase, not already on the board and listed once
    /// (§9.112/§9.113/§9.322); in the Campaign game it follows the order of
    /// appearance. Each destination must satisfy the full stacking rules
    /// (§5.51-5.53), not just the four-unit count, checked *cumulatively* so a
    /// batch that would over-stack a single hex is rejected as a whole.
    pub fn can_place_reinforcements(&self, placements: &[UnitPlacement]) -> Result<(), RuleError> {
        let ids: Vec<UnitId> = placements.iter().map(|p| p.id).collect();
        crate::effects::reject_duplicate_units(&ids)?;
        for p in placements {
            self.reinforcement_preconditions(p)?;
        }
        // §9.112/§9.113: in the Campaign game, off-board arrivals are bound
        // to the order of appearance -- the owning player's wave for the
        // current turn, its quotas, and its leader list.
        if self.scenario == Scenario::Campaign {
            self.validate_campaign_reinforcements(placements)?;
        }
        // Validate each placement against the board *plus* the units placed
        // earlier in this same batch onto the same hex, so two reinforcements
        // landing together can't jointly break stacking.
        for (i, p) in placements.iter().enumerate() {
            // §7.1: a reinforcing unit materialises on its entry hex -- it
            // may not appear on top of enemy units (engaging the enemy is
            // what melee is for). Lone AE leaders do not block a Dervish
            // arrival (§6.51 overrun applies to occupation).
            let enemy = p.profile.identity.owner().opponent();
            if self.units.iter().any(|u| {
                u.position == p.position
                    && u.profile.identity.owner() == enemy
                    && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
            }) {
                return Err(RuleError::EnemyOccupied(p.position));
            }
            let occupants: Vec<&UnitPlacement> = self
                .units
                .iter()
                .chain(placements[..=i].iter())
                .filter(|u| u.position == p.position)
                .collect();
            stacking_rule(&occupants)?;
        }
        // A peer may not invent unit values (canonical counters only).
        placements.iter().try_for_each(require_canonical_placement)
    }

    /// Per-placement reinforcement preconditions shared by the batch and
    /// single-placement checks: the owner's Movement phase (§9.112/§9.113/§9.322), and a
    /// counter that is not already on the board.
    fn reinforcement_preconditions(&self, p: &UnitPlacement) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        if p.profile.identity.owner() != self.active_player {
            return Err(RuleError::NotYourTurn);
        }
        if self.units.iter().any(|u| u.id == p.id) {
            return Err(RuleError::AlreadyDeployed(p.id));
        }
        Ok(())
    }

    /// Read-only preview of [`Self::can_place_reinforcements`] for a single
    /// placement, for the placing UI's click/preview gate: the campaign
    /// order of appearance (§9.112/§9.113), enemy occupation (§7.1), and
    /// full stacking (§5.51-5.53) — everything of the batch check that one
    /// placement can influence on its own (a lone placement cannot interact
    /// with other batch members). Non-campaign entry (the FoK turn-1 edge,
    /// §9.322) checks board presence instead of the wave schedule.
    pub fn can_place_single_reinforcement(&self, p: &UnitPlacement) -> Result<(), RuleError> {
        self.reinforcement_preconditions(p)?;
        if self.scenario == Scenario::Campaign {
            self.validate_campaign_reinforcements(std::slice::from_ref(p))?;
        }
        let enemy = p.profile.identity.owner().opponent();
        if self.units.iter().any(|u| {
            u.position == p.position
                && u.profile.identity.owner() == enemy
                && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
        }) {
            return Err(RuleError::EnemyOccupied(p.position));
        }
        self.check_stacking(p, p.position)?;
        require_canonical_placement(p)
    }

    /// Campaign order-of-appearance validation (§9.112 Dervish, §9.113
    /// Anglo-Egyptian). Reinforcements enter during the owning player's
    /// Movement phase; each placement must belong to that side's wave for the
    /// current turn -- by tribe or leader for the Dervish, by the land-unit
    /// cap / three-gunboat quota / free leaders for the Anglo-Egyptian. A
    /// unit may never enter twice, and units that skipped an earlier wave may
    /// still enter in a later one (the schedule gates, it does not expire).
    fn validate_campaign_reinforcements(
        &self,
        placements: &[UnitPlacement],
    ) -> Result<(), RuleError> {
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        for p in placements {
            let owner = p.profile.identity.owner();
            if owner != self.active_player {
                return Err(RuleError::NotYourTurn);
            }
            if self.units.iter().any(|u| u.id == p.id)
                || self
                    .reinforcements_placed_this_turn
                    .iter()
                    .any(|&(_, id)| id == p.id)
            {
                return Err(RuleError::AlreadyDeployed(p.id));
            }
            let schedule = match owner {
                Player::Dervish => crate::reinforcements::dervish_campaign_schedule(),
                Player::AngloEgyptian => crate::reinforcements::anglo_egyptian_campaign_schedule(),
            };
            let turn = self.current_turn.value();
            let Some(wave) = schedule.wave_for_turn(turn) else {
                return Err(RuleError::NoReinforcementWave { turn });
            };
            // §9.112/§9.113: when the board carries authored entrance-area
            // annotations, arrivals must enter through the annotated hexes
            // (Dervish: west edge south of the Khor Shambat; AE: entrance
            // area / north Nile edge / Abu Alim hut). Boards without the
            // annotation stay permissive (the bot falls back to geometry).
            let entrance_area = match &p.profile.identity {
                crate::UnitIdentity::DervishLeader(_)
                | crate::UnitIdentity::DervishTribal { .. } => {
                    Some(omdurman_types::NamedArea::DervishWestEdge)
                }
                crate::UnitIdentity::AngloEgyptianLeader(_) => {
                    Some(omdurman_types::NamedArea::AngloEgyptianEntrance)
                }
                _ if matches!(p.profile.kind, UnitKind::Gunboat { .. }) => {
                    Some(omdurman_types::NamedArea::GunboatNorthEdge)
                }
                _ if p.profile.identity.is_friendlies() => {
                    Some(omdurman_types::NamedArea::AbuAlimHut)
                }
                _ => Some(omdurman_types::NamedArea::AngloEgyptianEntrance),
            };
            if let Some(area) = entrance_area {
                let annotated = self.board.entrance_hexes(area);
                if !annotated.is_empty() && !annotated.contains(&p.position) {
                    return Err(RuleError::OutsideEntranceArea(p.position));
                }
            }
            match &p.profile.identity {
                crate::UnitIdentity::DervishTribal { tribe } => {
                    if !wave.tribes.contains(tribe) {
                        return Err(RuleError::TribeNotInWave { turn });
                    }
                }
                crate::UnitIdentity::DervishLeader(leader) => {
                    let listed = wave
                        .leaders
                        .iter()
                        .any(|l| matches!(l, crate::reinforcements::CampaignLeader::Dervish(d) if d == leader));
                    if !listed {
                        return Err(RuleError::TribeNotInWave { turn });
                    }
                }
                _ if owner == Player::Dervish => {
                    // Forts, artillery, gunboats: part of the §9.111 initial
                    // force, never reinforcements.
                    return Err(RuleError::TribeNotInWave { turn });
                }
                crate::UnitIdentity::AngloEgyptianLeader(leader) => {
                    let listed = wave.leaders.iter().any(|l| {
                        matches!(l, crate::reinforcements::CampaignLeader::British(d) if d == leader)
                    });
                    if !listed {
                        return Err(RuleError::LeaderNotInWave { turn });
                    }
                }
                _ => {
                    // Non-leader Anglo-Egyptian arrival (§9.113): gunboats
                    // are quota'd three per turn and do not count against
                    // the land-unit cap; land units share the wave's cap
                    // (leaders exempt).
                    let batch_gunboats = placements
                        .iter()
                        .filter(|q| matches!(q.profile.kind, UnitKind::Gunboat { .. }))
                        .count();
                    let batch_land = placements.len() - batch_gunboats;
                    // Count what this side already placed this player-turn,
                    // resolving each recorded id's kind from the board (or
                    // from the current batch for ids placed moments ago).
                    let mut placed_gunboats = 0usize;
                    let mut placed_land = 0usize;
                    for &(player, id) in &self.reinforcements_placed_this_turn {
                        if player != owner {
                            continue;
                        }
                        let is_boat = placements
                            .iter()
                            .find(|q| q.id == id)
                            .or_else(|| self.units.iter().find(|u| u.id == id))
                            .is_some_and(|u| matches!(u.profile.kind, UnitKind::Gunboat { .. }));
                        if is_boat {
                            placed_gunboats += 1;
                        } else {
                            placed_land += 1;
                        }
                    }
                    if matches!(p.profile.kind, UnitKind::Gunboat { .. }) {
                        if placed_gunboats + batch_gunboats > 3 {
                            return Err(RuleError::GunboatQuotaExceeded { turn });
                        }
                    } else if let Some(cap) = wave.unit_cap
                        && placed_land + batch_land > cap
                    {
                        return Err(RuleError::ReinforcementCapExceeded { turn, cap });
                    }
                }
            }
        }
        Ok(())
    }
}

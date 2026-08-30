use super::*;

/// A semantic game action, fully determined (all dice pre-rolled)
/// (rulebook §4, §5, §6, §7, §8, §10).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, strum::IntoStaticStr)]
pub enum GameEffect {
    // -- Turn / phase flow ------------------------------------------------
    /// Advance to the next phase (or next player-turn if melee is done)
    /// (rulebook §4).
    ///
    /// **Preconditions:** Game is not over; `phase` is a valid current phase.
    ///
    /// **Postconditions:** `phase` advances to the next valid phase per the
    /// turn sequence in §4. Stacking is re-checked after melee resolution.
    /// At end-of-turn, disrupted units recover and per-turn tracking is
    /// cleared.
    AdvancePhase,

    // -- Movement ----------------------------------------------------------
    /// Move a unit to `to` (rulebook §5).
    ///
    /// **Preconditions:**
    /// - Unit exists in `state.units` and is not disrupted (§5).
    /// - Unit has not already moved this turn (§5.13).
    /// - Source hex matches current position.
    /// - `to` is reachable within remaining movement points.
    /// - `to` is not enemy-occupied (§5.26).
    /// - Unit stops in enemy ZOC (§5.43).
    /// - Stacking ≤ 4 at `to` (§5.51); leaders and gunboats are free stacking.
    /// - Dervish tribes do not mix stacks (§5.52).
    ///
    /// **Postconditions:**
    /// - Unit position is set to `to`.
    /// - Movement points spent are recorded in `mp_spent_this_turn`.
    /// - `zoc_stopped_this_turn` set if entered enemy ZOC.
    /// - GORDON elimination checked for Fall of Khartoum (§9.346).
    ///
    /// When `path` (the ordered hexes entered, excluding the start and
    /// including `to`) is supplied, the engine computes the true movement
    /// cost from the board's terrain (§5.11) and, for gunboats, enforces
    /// the Nile upstream/downstream allowance (§5.24) -- the caller-supplied
    /// `cost` is then only a fallback. When `path` is empty the engine
    /// trusts `cost` and treats the move as raw distance (legacy/tests).
    /// On success the unit's position is set to `to`, making the rules
    /// engine authoritative for position.
    MoveUnit {
        unit_id: UnitId,
        to: HexCoord,
        cost: MovementPoints,
        #[serde(default)]
        path: Vec<HexCoord>,
    },

    // -- Fire combat -------------------------------------------------------
    /// Resolve a direct or Maxim-second-fire attack (rulebook §6).
    ///
    /// **Preconditions:**
    /// - Active phase is `OffensiveFire(DirectFire)` or `OffensiveFire(MaximSecondAndHowitzer)`.
    /// - Firing player owns the attacking units.
    /// - All firers are legal for the sub-phase (§6.42: only Maxims in Maxim sub-phase).
    /// - Target hex is occupied by enemy units (§6.14).
    /// - Each firer has not already fired this phase (§6.14).
    /// - Target hex has not already been fired at this phase (§6.14).
    /// - Target is within range and has LOS (§6.21/§6.22), except howitzers (§6.64).
    ///
    /// **Postconditions:**
    /// - Fire factors are summed, range-band-adjusted, terrain-modified, and
    ///   cross-referenced on the CRT with the die roll.
    /// - Target units are disrupted/eliminated per CRT result.
    /// - Firers marked as fired; target hex marked as fired-at.
    /// - Victory points awarded for eliminations.
    FireCombat { attack: FireAttack, roll: DieRoll },

    /// Resolve a howitzer bombardment (two rolls: CRT + impact scatter)
    /// (rulebook §6.64).
    ///
    /// **Preconditions:**
    /// - Active phase is `OffensiveFire(MaximSecondAndHowitzer)`.
    /// - All firers have Howitzer weapon class.
    /// - Target hex is within range 4-10 (§6.64).
    /// - It is not night (§8.1: howitzers may not fire at night).
    /// - Target hex is occupied by enemy units.
    ///
    /// **Postconditions:**
    /// - Impact hex determined by scatter roll.
    /// - CRT result applied to units at impact hex (not the original target).
    /// - Firers marked as fired.
    HowitzerFire {
        attack: FireAttack,
        combat_results_table_roll: DieRoll,
        impact_roll: DieRoll,
    },

    // -- Melee combat ------------------------------------------------------
    /// Resolve melee between adjacent hexes (simultaneous, two rolls)
    /// (rulebook §7).
    ///
    /// **Preconditions:**
    /// - Active phase is `Melee`.
    /// - Attacker and defender hexes are adjacent (§7.1).
    /// - Attacker is owned by the active player; defender is enemy.
    /// - Attacker has not already melee'd this turn.
    ///
    /// **Postconditions:**
    /// - Both rolls are applied simultaneously.
    /// - Terrain defense modifiers excluded from melee (§7.7); only
    ///   standard +1/+2 modifiers and zariba/trench apply.
    /// - Losers are eliminated or disrupted per CRT.
    /// - Winner may advance into vacated hex (§7.6).
    /// - Victory points awarded for eliminations.
    MeleeCombat {
        attack: MeleeAttack,
        attacker_roll: DieRoll,
        defender_roll: DieRoll,
    },

    /// Declare a melee, opening the defender's reaction window (§7.5): the
    /// attack and its pre-rolled dice are stored as `pending_melee`; eligible
    /// defenders may retreat before [`GameEffect::ResolveMelee`] is applied.
    ///
    /// **Preconditions:** Same as `MeleeCombat`.
    /// **Postconditions:** `pending_melee` is set; no combat resolution yet.
    DeclareMelee {
        attack: MeleeAttack,
        attacker_roll: DieRoll,
        defender_roll: DieRoll,
    },

    /// Resolve the currently-pending declared melee against whoever still
    /// occupies the target hex (so a retreated defender is spared). Clears the
    /// reaction window.
    ///
    /// **Preconditions:** `pending_melee` is `Some`.
    /// **Postconditions:** Same as `MeleeCombat`; `pending_melee` cleared.
    ResolveMelee,

    /// A cavalry/camel unit retreats two hexes from an impending infantry
    /// melee attack, *before* it is resolved (§7.5). One retreat per unit per
    /// turn.
    ///
    /// **Preconditions:**
    /// - Unit is cavalry or camel corps.
    /// - Unit has not already retreated before melee this turn.
    /// - `to` is exactly two hexes away from the defender position.
    /// - `to` is not enemy-occupied.
    ///
    /// **Postconditions:** Unit position moved to `to`.
    RetreatBeforeMelee { unit_id: UnitId, to: HexCoord },

    /// An attacking unit advances into a hex vacated by combat (rulebook §6.82
    /// after fire, §7.6 after melee). Eligible units are adjacent attackers
    /// that are not artillery; the target hex must be empty of enemies.
    ///
    /// **Preconditions:**
    /// - Unit is adjacent to the vacated hex.
    /// - Unit is not artillery (§6.82).
    /// - Unit has not already moved this turn (except via melee advance).
    /// - `to` is listed in `vacated_by_combat` for this unit.
    ///
    /// **Postconditions:** Unit position moved to `to`; `vacated_by_combat`
    /// entry consumed.
    AdvanceAfterCombat { unit_id: UnitId, to: HexCoord },

    // -- Unit state changes ------------------------------------------------
    /// Remove disrupted status from a unit (end of owning player's turn) (rulebook §5, reference notes).
    RecoverUnit { unit_id: UnitId },

    /// Begin constructing a Zariba hexside (rulebook §5.3).
    ConstructZariba {
        unit_ids: Vec<UnitId>,
        hexside: HexsideRef,
    },

    /// Royal Engineers demolition (rulebook §6.53).
    Demolition {
        unit_id: UnitId,
        target: DemolitionTarget,
    },

    // -- Reinforcement / placement -----------------------------------------
    /// Place reinforcements onto the map (rulebook §9.112, §9.113).
    PlaceReinforcements(Vec<UnitPlacement>),

    // -- Scenario-specific -------------------------------------------------
    /// Dervish desertion roll, once per campaign on the first night turn
    /// (rulebook §8.2). The number of deserters is `floor(1.5 * roll)`; the
    /// Dervish player chooses which units desert, so the chosen IDs travel with
    /// the effect. The Khalifa, gunboats, artillery, and forts may not be
    /// chosen.
    DervishDesertion {
        roll: DieRoll,
        deserters: Vec<UnitId>,
    },

    /// Load/disembark the "Friendlies" brigade via gunboat (rulebook §5.21).
    FriendliesTransport(crate::FriendliesAction),

    // -- Optional rules ----------------------------------------------------
    /// River mine resolution (rulebook §10.12).
    RiverMine {
        gunboat_id: UnitId,
        hex: HexCoord,
        roll: DieRoll,
    },

    /// Sink the river chain (rulebook §10.23): the chain is cleared once an
    /// infantry/cavalry unit spends a full turn adjacent on either bank, or
    /// artillery scores 3+ on the Combat Results Table. The caller establishes
    /// which condition was met (it has the positional/turn context); the engine
    /// records the state transition so no gunboat is stopped by it thereafter.
    SinkChain,

    // -- Setup / deployment (§9.2/§9.3/§10) --------------------------------
    /// Place one of a player's order-of-battle units onto the board during
    /// [`Phase::Setup`], within that side's legal deployment zone (§9.2/§9.3).
    /// Rejected outside Setup, off-zone, or if it would break stacking.
    DeployUnit(UnitPlacement),

    /// Remove an already-deployed unit from the board during [`Phase::Setup`]
    /// (§9.2/§9.3) so its counter can be re-placed. Only legal in Setup, only by
    /// the unit's owner, and only for a unit that is actually on the board. The
    /// app's net-layer `RemoveUnit` event resolves to this effect so removal is
    /// validated by the engine, not by the input layer.
    RemoveDeployedUnit { unit_id: UnitId, player: Player },

    /// Lay a river mine during setup (§10.11): at most two, never sharing a hex.
    PlaceMine { hex: HexCoord },

    /// Lay the river chain during setup (§10.21): up to four contiguous Nile
    /// hexes. Replaces any previously-laid chain.
    PlaceChain { hexes: Vec<HexCoord> },

    /// Fortify a hexside with a Zariba before play (§9.231-9.232). Unlike
    /// [`ConstructZariba`](GameEffect::ConstructZariba) (which units *build*
    /// during a turn), this is the historical scenario's pre-placed
    /// fortification.
    PlaceZariba { hexside: HexsideRef },

    /// A faction confirms it is ready to leave setup (§9.2/§9.3). Setup is
    /// concurrent, so each side confirms independently; when *both* have
    /// confirmed (and `setup_complete` holds) the engine auto-advances to the
    /// first Movement turn. One-way -- re-confirming is a no-op.
    ConfirmSetupReady { player: Player },

    /// Resolve a pending Royal Engineers demolition at end of turn (§6.53).
    /// Auto-emitted by `end_player_turn` for each entry in
    /// `state.pending_demolitions`. The engine checks the engineer is still
    /// adjacent and undisrupted; if so the target is destroyed (fort removed
    /// or wall breached per §6.63) and the engineer is freed.
    ResolveDemolition {
        unit_id: UnitId,
        target: DemolitionTarget,
    },

    // -- Drift (§10.12) ----------------------------------------------------
    /// A gunboat with lost engines drifts one hex downstream with the Nile
    /// current (rulebook §10.12).  Applied automatically at the start of each
    /// movement phase.  If no flow data exists at the current hex (dead end),
    /// the gunboat is stuck and nothing happens.
    ///
    /// `mine_roll` is the pre-rolled d10 (§10.12) used iff the drift
    /// destination holds an untriggered river mine: the mine is resolved
    /// against this roll exactly as a [`GameEffect::RiverMine`] would
    /// (Dervish gunboats pass through unharmed, §10.14). Carrying the roll in
    /// the effect keeps the pre-rolled-dice determinism invariant -- every
    /// peer replaying the effect resolves the same outcome.
    DriftGunboat { unit_id: UnitId, mine_roll: DieRoll },

    // -- Artillery wall-breaching (§6.63 3rd bullet) -----------------------
    /// Resolve artillery fire aimed at breaching a wall hexside (rulebook
    /// §6.63). Only artillery-class firers may participate; a CRT result of
    /// `Eliminate(2)` or higher flips the targeted `Wall` hexside to `Breach`
    /// (negating it for LOS / movement / melee / ZOC) and eliminates one enemy
    /// unit adjacent to the breached hexside, mirroring the Royal-Engineers
    /// demolition path. Any other CRT result is a miss. The `roll` is the
    /// pre-rolled d10 used for the CRT lookup; range/LOS are re-derived by the
    /// engine from the firers and `target`.
    ArtilleryBreachWall {
        firers: Vec<UnitId>,
        target: HexsideRef,
        roll: DieRoll,
    },
}

// ---------------------------------------------------------------------------
// 2) RuleError -- why an effect was rejected
// ---------------------------------------------------------------------------

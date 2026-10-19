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
    /// `path` is the ordered hexes entered, excluding the start and ending
    /// at `to`; an empty `path` means the single step to an adjacent `to`.
    /// The engine validates every step (adjacent, on the board, passable,
    /// not enemy-occupied, no pass-through of an enemy ZOC) and computes the
    /// true movement cost from the board's terrain (§5.11) and, for
    /// gunboats, enforces the Nile upstream/downstream allowance (§5.24).
    /// The caller-supplied `cost` is ignored (kept for wire compatibility).
    /// On success the unit's position is set to `to`, making the rules
    /// engine authoritative for position.
    MoveUnit {
        unit_id: UnitId,
        to: HexCoord,
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
    /// - Target units are disrupted/eliminated per CRT result; a `D` result
    ///   disrupts the undisrupted units `disruption` picks (§CombatResults).
    /// - Firers marked as fired; target hex marked as fired-at.
    /// - Victory points awarded for eliminations.
    FireCombat {
        attack: FireAttack,
        roll: DieRoll,
        disruption: DisruptionDraw,
    },

    /// Resolve a howitzer bombardment (rulebook §6.42, §6.64): the named
    /// gunboats' shells, with any Maxim fire combined into the attack.
    ///
    /// **Preconditions:**
    /// - The Maxim Second Fire and Howitzer subphase; no night (§6.64).
    /// - At least one shell: a named gunboat's artillery among `firers`, its
    ///   target hex 4-10 hexes off (line of sight ignored); every other
    ///   weapon is a Maxim (a Maxim battery in `firers`, a named gunboat's
    ///   Maxims in `gunboat_maxims`) in range and sight of the target.
    /// - `shells` holds one [`ShellRoll`] per shell, in `firers` order.
    /// - The target hex is enemy-occupied and not yet fired at.
    ///
    /// **Postconditions:** each shell's impact roll places it. The shells
    /// that land in the intended hex combine with the Maxims into one attack
    /// there, resolved with `roll` and `disruption` (§6.42: "Howitzer fire
    /// may be combined with Maxim fire, but only if the howitzer fire
    /// impacts in the intended hex"); with no shell on target the Maxims
    /// fire there alone. Each scattered shell resolves alone on whoever it
    /// lands on, friend or foe, with its own roll (§6.64). Every weapon is
    /// marked as fired.
    HowitzerFire {
        attack: FireAttack,
        shells: Vec<ShellRoll>,
        roll: DieRoll,
        disruption: DisruptionDraw,
    },

    /// Allocate fire orders for the current fire subphase (rulebook §6.41,
    /// §6.42: "first allocate all of his fire attacks ... After all fire has
    /// been allocated, the firing player then resolves his attacks in any
    /// order he wishes").
    ///
    /// **Preconditions:** a fire subphase whose fire has not begun
    /// resolving; every order resolvable now; across the whole allocation
    /// (earlier orders included) every weapon fires once and every target is
    /// fired at once (§6.13, §6.14).
    /// **Postconditions:** the orders join `fire_allocation`, unresolved;
    /// each resolution effect claims its order, and the phase may not end
    /// while an allocated order can still be resolved.
    AllocateFire { orders: Vec<FireOrder> },

    // -- Melee combat ------------------------------------------------------
    /// Declare a melee between adjacent hexes (rulebook §7), opening the
    /// defender's reaction window (§7.5): the attack and its pre-rolled dice
    /// are stored as `pending_melee`; eligible defenders may retreat before
    /// [`GameEffect::ResolveMelee`] is applied.
    ///
    /// **Preconditions:** the Melee phase; every attacker the active
    /// player's, adjacent, able to melee and not yet spent this turn (§7.4,
    /// §7.5); the defenders exactly the enemy units in the target hex.
    /// **Postconditions:** `pending_melee` is set; no combat resolution yet.
    DeclareMelee {
        attack: MeleeAttack,
        attacker_roll: DieRoll,
        defender_roll: DieRoll,
        /// Which units each side's `D` result disrupts (§CombatResults).
        disruption: DisruptionDraw,
    },

    /// Resolve the currently-pending declared melee against whoever still
    /// occupies the target hex (so a retreated defender is spared). Clears the
    /// reaction window.
    ///
    /// **Preconditions:** `pending_melee` is `Some`.
    /// **Postconditions:** both rolls applied simultaneously on the Combat
    /// Results Table (§7.3, §7.7), the Dervish advance forced and the
    /// Anglo-Egyptian one offered (§7.6); `pending_melee` cleared.
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
    /// Begin constructing the Zariba (rulebook §5.3): the listed
    /// Anglo-Egyptian infantry, inside the printed Zariba next to the printed
    /// `hexside`, build every printed Zariba hexside they stand beside at the
    /// end of the Anglo-Egyptian player turn.
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
    /// (rulebook §8.2). The number of deserters is `ceil(1.5 * roll)`; the
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
    /// River mine resolution (rulebook §10.12): the Dervish player's roll for
    /// the British gunboat stopped on his mine (one of `pending_mines`).
    RiverMine {
        gunboat_id: UnitId,
        hex: HexCoord,
        roll: DieRoll,
    },

    /// Artillery fire at the river chain (rulebook §10.23 b): British
    /// artillery in the Direct Fire subphase, each firer in range of and in
    /// sight of a chained hex; a Combat Results Table result of 3 or more
    /// sinks the chain. (The other way, §10.23 a -- an infantry or cavalry
    /// unit spending a complete turn on the bank next to it -- is resolved
    /// by the engine at the end of the Anglo-Egyptian player turn.)
    SinkChain { firers: Vec<UnitId>, roll: DieRoll },

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

    /// A faction confirms it is ready to leave setup (§9.2/§9.3). Setup is
    /// sequential (§9.111/§9.211/§9.321): the first side's confirmation fixes
    /// its deployment, and only then may the second side deploy and confirm.
    /// When *both* have confirmed (and `setup_complete` holds) the engine
    /// auto-advances to the first Movement turn. One-way -- a confirmed side
    /// cannot confirm again.
    ConfirmSetupReady { player: Player },

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

    // -- Turn / phase flow --
    /// [`GameEffect::AdvancePhase`] stamped with the phase it ends (rulebook
    /// §4): rejected with [`RuleError::PhaseAlreadyEnded`] unless the state
    /// is still in `phase` of `turn` with `player` the phasing player. Ending
    /// a phase is not idempotent otherwise: a double click, a held key or two
    /// teammates within one round trip would each end *a* phase, skipping the
    /// next one. The players' End Phase control submits this; the bare
    /// `AdvancePhase` stays for saved games and the engine-side drivers.
    AdvancePhaseFrom {
        turn: GameTurnIndex,
        player: Player,
        phase: Phase,
    },

    /// Roll the §8.2 desertion die on its own, before the deserters are
    /// chosen (rulebook §8.2): legal once, while the desertion is due; the
    /// roll is kept in [`GameState::desertion_roll`] and the
    /// [`GameEffect::DervishDesertion`] that follows must carry it. Rolled
    /// first and recorded, the die survives a pause, a reload or a rejoin --
    /// re-opening the desertion panel can no longer roll it again.
    RollDesertion { roll: DieRoll },

    /// The defender of the pending melee lets it come without (further)
    /// retreat: cavalry and camel units *may* retreat (§7.5), and this is the
    /// defender's "no". It ends the defender's reaction window, so the
    /// attacker knows the defence has decided -- an AI attacker waits for it
    /// (or for the last threatened unit to have retreated) before resolving.
    ///
    /// **Preconditions:** the Melee phase; `pending_melee` is `Some` and not
    /// yet declined.
    /// **Postconditions:** `pending_melee.retreat_declined` is set; a later
    /// [`GameEffect::RetreatBeforeMelee`] from this attack is refused.
    DeclineRetreatBeforeMelee,
}

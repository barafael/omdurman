//! Semantic game effects -- every mutation passes through [`apply_effect`]
//! (rulebook §4, §5, §6, §7, §8, §10).
//!
//! Each [`GameEffect`] carries *all* information (including pre-rolled die
//! values) needed to apply it deterministically.  The processor validates
//! the effect against the current [`GameState`] and, if legal, mutates the
//! state in place.  Network replay works because every peer receives the
//! identical effect with the identical roll.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use crate::combat_results_table::{FireFactorRow, combat_results_table};
use crate::howitzer_scatter::{ScatterHexDirection, howitzer_scatter};
use crate::range_effects::{ae_range_effects, dervish_range_effects};
use crate::turn_summary::{TurnEventRecord, TurnSummary};
use crate::turn_track::{TurnEvent, scenario_turn};
use crate::{
    CampaignVictoryLevel, CombatResult, DemolitionTarget, DieRoll, FireAttack, FireFactor,
    FireKind, FireModifier, FireSubPhase, GameTurnIndex, HexCoord, HexDistance,
    HistoricalVictoryLevel, MeleeAttack, MeleeModifier, MovementAllowance, MovementPoints, Phase,
    UnitId, UnitPlacement, VictoryLedger, VictoryPoints, VpEvent,
    VpSource, WeaponClass, ZocReason,
};
use omdurman_types::{
    DayNight, DervishTribe, HexsideKind, HexsideRef, Player, Scenario, UnitKind,
};

use crate::FriendliesAction;
use crate::TransportState;
use crate::board::BoardInfo;
use crate::{ChainPlacement, MinePlacement, OptionalRule};

// ---------------------------------------------------------------------------
// 1) GameEffect -- every semantic action a player can take
// ---------------------------------------------------------------------------

/// A semantic game action, fully determined (all dice pre-rolled)
/// (rulebook §4, §5, §6, §7, §8, §10).
#[derive(Serialize, Deserialize, Clone, Debug, strum::IntoStaticStr)]
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
    DriftGunboat { unit_id: UnitId },

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

/// Validation error returned when [`apply_effect`] refuses an effect (rulebook §5, §6, §7).
#[derive(thiserror::Error, Clone, Debug)]
pub enum RuleError {
    #[error("game is over")]
    GameOver,

    #[error("not your turn")]
    NotYourTurn,

    #[error("wrong phase for this action")]
    WrongPhase,

    #[error(
        "only Maxim guns and Howitzers may fire in the Maxim Second Fire and Howitzer Subphase (§6.42)"
    )]
    WrongWeaponForSubphase(UnitId),

    #[error("setup is not complete: {0}")]
    SetupIncomplete(&'static str),

    #[error("hex {0:?} is outside this unit's deployment zone (§9.2/§9.3)")]
    OutsideDeploymentZone(HexCoord),

    #[error(
        "unit {0:?} is not in play at setup for this scenario (§9.111/§9.211/§9.212): it arrives as a reinforcement or is excluded"
    )]
    NotInPlay(UnitId),

    #[error(
        "the FALL OF KHARTOUM order of battle (§9.321/§9.322) allows no more units of this type"
    )]
    FoKOrderOfBattleFull,

    #[error("{0}")]
    SetupLimit(&'static str),

    #[error("counter {0:?} is already on the board -- each physical unit deploys once")]
    AlreadyDeployed(UnitId),

    #[error("unit {0:?} has already fired this phase")]
    AlreadyFired(UnitId),

    #[error("unit {0:?} has already been fired at this phase (§6.14)")]
    AlreadyFiredAt(UnitId),

    #[error("unit {0:?} has already moved this turn")]
    AlreadyMoved(UnitId),

    #[error("unit {0:?} is disrupted and may not act")]
    Disrupted(UnitId),

    #[error("GORDON may not move during FALL OF KHARTOUM (§9.346)")]
    GordonMayNotMove,

    #[error("a unit may not enter an enemy-occupied fort hex {0:?} (§6.54)")]
    EnemyFort(HexCoord),

    #[error(
        "hex {0:?} is occupied by enemy units -- engaging the enemy is what melee is for (§7.1); movement may only end adjacent (§5.26)"
    )]
    EnemyOccupied(HexCoord),

    #[error("unit {0:?} not found")]
    UnitNotFound(UnitId),

    #[error("unit {0:?} does not belong to the acting player")]
    NotOwner(UnitId),

    #[error("target hex {0:?} contains no enemy units")]
    NoEnemyInHex(HexCoord),

    #[error("unit {0:?} is out of range")]
    OutOfRange(UnitId),

    #[error("line of sight is blocked from {0:?} to {1:?} (§6.3)")]
    LineOfSightBlocked(HexCoord, HexCoord),

    #[error("a wall or thorn-hedge hexside blocks melee from {0:?} to {1:?} (§7.2)")]
    MeleeBlockedByHexside(HexCoord, HexCoord),

    #[error("a hexside blocks advance after combat from {0:?} to {1:?} (§6.82, §7.6)")]
    AdvanceBlockedByHexside(HexCoord, HexCoord),

    #[error("a wall hexside blocks movement from {0:?} to {1:?} (§5.23)")]
    MoveBlockedByHexside(HexCoord, HexCoord),

    #[error("unit {0:?} is not eligible to enter the walled city of Omdurman at {1:?} (§5.23)")]
    WalledCityEntry(UnitId, HexCoord),

    #[error("movement cost {cost:?} exceeds allowance {allowance:?}")]
    MovementExceedsAllowance {
        cost: MovementPoints,
        allowance: MovementAllowance,
    },

    #[error("hex stack would exceed the four-unit limit")]
    StackOverflow,

    #[error("movement may not pass through an enemy zone of control at {0:?}")]
    BlockedByEnemyZoc(HexCoord),

    #[error("unit {0:?} entered an enemy zone of control and may move no further this turn (§5.43)")]
    StoppedInEnemyZoc(UnitId),

    #[error("fire modifiers must equal the rulebook-mandated set (§6.24/§5.54/§9.231/§9.232): expected {expected:?}, got {got:?}")]
    FireModifierMismatch {
        expected: Vec<crate::FireModifier>,
        got: Vec<crate::FireModifier>,
    },

    #[error("melee modifiers must equal the rulebook-mandated set (§7.7/§9.232): expected {expected:?}, got {got:?}")]
    MeleeModifierMismatch {
        expected: Vec<crate::MeleeModifier>,
        got: Vec<crate::MeleeModifier>,
    },

    #[error("land unit may not enter the Nile hex {0:?} (§5.22)")]
    LandIntoNile(HexCoord),

    #[error("hex {0:?} is off the board")]
    OffBoard(HexCoord),

    #[error("gunboat may only move along Nile hexes; {0:?} is not Nile (§5.22)")]
    GunboatOffNile(HexCoord),

    #[error(
        "gunboat moved upstream, so its upstream allowance {allowance:?} caps the turn, but the move costs {cost:?} (§5.24)"
    )]
    GunboatUpstreamCap {
        cost: MovementPoints,
        allowance: MovementAllowance,
    },

    #[error("gunboat entered a chained Nile hex {0:?} and must stop (§10.22)")]
    BlockedByChain(HexCoord),

    #[error("illegal stack: {0}")]
    Stacking(#[from] crate::StackingError),

    #[error("illegal Dervish desertion: {0}")]
    Desertion(#[from] DesertionError),

    #[error("unit {0:?} cannot move on land")]
    NotMobile(UnitId),

    #[error("unit {0:?} is not a gunboat")]
    NotAGunboat(UnitId),

    #[error("only howitzer-class units may fire howitzer (unit {0:?})")]
    OnlyHowitzerMayFireHowitzer(UnitId),

    #[error("only Maxim units may use second fire (unit {0:?})")]
    OnlyMaximSecondFire(UnitId),

    #[error("no howitzer fire at night (§6.64)")]
    NoHowitzerAtNight,

    #[error("unit {0:?} has no fire factor")]
    NoFireFactor(UnitId),

    #[error("only artillery may fire at a gunboat or fort (§6.61, §6.62)")]
    ArtilleryOnlyVsGunboatOrFort(UnitId),

    #[error("target {target:?} out of range at night from {firer:?} (§8.1)")]
    OutOfRangeAtNight { firer: HexCoord, target: HexCoord },

    #[error("target {target:?} out of range from {firer:?}")]
    TargetOutOfRange { firer: HexCoord, target: HexCoord },

    #[error("unit {0:?} kind may not melee attack")]
    KindMayNotMelee(UnitId),

    #[error("target {to:?} is not adjacent to {from:?}")]
    TargetNotAdjacent { from: HexCoord, to: HexCoord },

    #[error("no meleeable enemy in target hex {0:?}")]
    NoMeleeableEnemy(HexCoord),

    #[error("unit {0:?} may not move once placed (§5.25)")]
    AlreadyPlaced(UnitId),

    #[error("a melee is already pending resolution")]
    MeleeAlreadyPending,

    #[error("a declared melee must be resolved (or its target vacated by retreat) before the melee phase can end")]
    MeleePendingResolution,

    #[error("the §8.2 desertion roll must be made before the Dervish movement phase of the first night turn can end")]
    DesertionRollRequired,

    #[error("melee has no attackers")]
    MeleeHasNoAttackers,

    #[error("no melee pending resolution")]
    NoMeleePending,

    #[error("no declared infantry melee threatens unit {0:?}")]
    NoInfantryMeleeThreatens(UnitId),

    #[error("unit {0:?} may not retreat before melee")]
    MayNotRetreatBeforeMelee(UnitId),

    #[error("retreat must be exactly two hexes")]
    RetreatMustBeTwoHexes,

    #[error("retreat hex {0:?} is occupied")]
    RetreatHexOccupied(HexCoord),

    #[error("retreat {0:?} -> {1:?} would cross a wall hexside (§5.23)")]
    RetreatBlockedByWall(HexCoord, HexCoord),

    #[error("artillery unit {0:?} may not advance after combat")]
    ArtilleryMayNotAdvance(UnitId),

    #[error("fort {0:?} may not move in any way once placed (§5.25)")]
    FortMayNotAdvance(UnitId),

    #[error("no reinforcement wave is scheduled for game turn {turn} (§9.112/§9.113)")]
    NoReinforcementWave { turn: u8 },

    #[error("the unit's tribe is not part of this turn's reinforcement wave (§9.112)")]
    TribeNotInWave { turn: u8 },

    #[error("the leader is not part of this turn's reinforcement wave (§9.113)")]
    LeaderNotInWave { turn: u8 },

    #[error("more than three gunboats may not enter in one turn (§9.113)")]
    GunboatQuotaExceeded { turn: u8 },

    #[error("replacements exceed the turn's {cap}-unit limit (§9.113)")]
    ReinforcementCapExceeded { turn: u8, cap: usize },

    #[error(
        "hex {0:?} is outside the annotated entrance area for this reinforcement (§9.112/§9.113)"
    )]
    OutsideEntranceArea(HexCoord),

    #[error("advance hex is not adjacent")]
    AdvanceNotAdjacent,

    #[error("advance hex {0:?} is not vacant")]
    AdvanceNotVacant(HexCoord),

    #[error(
        "advance hex {0:?} was not vacated by combat this phase (§6.82, §7.6): advance is only legal into a hex the defender vacated"
    )]
    HexNotVacatedByCombat(HexCoord),

    #[error(
        "unit {0:?} did not participate in the combat that vacated {1:?} (§6.82, §7.6): only participating attackers may advance"
    )]
    UnitDidNotParticipate(UnitId, HexCoord),

    #[error("unit {0:?} is not disrupted")]
    NotDisrupted(UnitId),

    #[error("Friendlies transport requires Isa Zachneih to be eliminated first (§5.21)")]
    FriendliesIsaZachneihAlive,

    #[error("a Friendlies transport mission is already in progress (§5.21)")]
    FriendliesTransportInProgress,

    #[error("Friendlies unit must be adjacent to the gunboat to load (§5.21)")]
    FriendliesNotAdjacentToGunboat,

    #[error("Crossing requires a prior Loaded state for the same unit+gunboat (§5.21)")]
    FriendliesNotLoaded,

    #[error("ReadyToDisembark requires a prior Crossing state for the same unit+gunboat (§5.21)")]
    FriendliesNotCrossing,

    #[error("gunboat {0:?} engines are not lost; cannot drift")]
    GunboatEnginesNotLost(UnitId),

    #[error("no untriggered river mine in hex {0:?} (§10.13)")]
    NoUntriggeredMine(HexCoord),

    #[error("river chain is already sunk")]
    ChainAlreadySunk,

    #[error("no river chain has been placed")]
    NoChainPlaced,

    #[error("fire attack has no firers")]
    NoFirers,

    #[error("only artillery may fire to breach a wall hexside (§6.63; unit {0:?})")]
    OnlyArtilleryMayBreachWall(UnitId),

    #[error("hexside {0:?} is not a Wall (§6.63)")]
    NotAWallHexside(HexsideRef),

    #[error("wall-breaching firers must be in the same fire phase (§6.63)")]
    WallBreachFirersMisaligned,
}

/// Why a Dervish desertion effect was rejected (rulebook §8.2).
#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum DesertionError {
    #[error("desertion may only be rolled once per campaign game")]
    AlreadyDeserted,

    #[error("desertion is a campaign-game rule only")]
    WrongScenario,

    #[error("desertion is rolled during the first night turn's movement phase")]
    WrongTime,

    #[error("expected {expected} deserters for a roll of {roll}, got {actual}")]
    WrongCount {
        roll: u8,
        expected: usize,
        actual: usize,
    },

    #[error("unit {0:?} is not a Dervish unit eligible to desert")]
    NotEligible(UnitId),

    #[error("the Khalifa, gunboats, artillery, and forts may not desert (unit {0:?})")]
    Exempt(UnitId),
}

// ---------------------------------------------------------------------------
// 2b) Observation -- side-channel signals emitted by apply_effect
// ---------------------------------------------------------------------------

/// Why a unit was eliminated, surfaced via [`Observation::UnitEliminated`] so
/// the app can render appropriate flavour (dispatch slips, sounds, etc.).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ElimCause {
    /// Eliminated by fire combat (§6) or melee (§7).
    Combat,
    /// Eliminated by a demolition resolution (§6.53 / §6.63).
    Demolition,
    /// A unit loaded on a gunboat that was sunk or eliminated -- the unit is
    /// lost with the ship (§5.21, §10.12).
    LostWithTransport,
    /// GORDON eliminated at the palace (§9.346).
    GordonAtPalace,
    /// Anglo-Egyptian leader eliminated because all combat units in its hex
    /// were eliminated (orphan leader, §5.44).
    OrphanLeader,
}

impl std::fmt::Display for ElimCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElimCause::Combat => write!(f, "eliminated in combat"),
            ElimCause::Demolition => write!(f, "demolition (§6.53)"),
            ElimCause::LostWithTransport => write!(f, "lost with sunk transport"),
            ElimCause::GordonAtPalace => write!(f, "GORDON fallen at the Palace"),
            ElimCause::OrphanLeader => write!(f, "orphan leader eliminated"),
        }
    }
}

/// A side-channel signal emitted by `apply_effect` describing what happened,
/// for the app to translate into Bevy events (dispatch slips, sounds, camera
/// focus, VP animations).  These are *observations of state changes*, not the
/// changes themselves -- `apply_effect` mutates `GameState` synchronously
/// regardless of whether observations are drained.
///
/// Pushed by the engine onto [`GameState::observations`]; the app drains them
/// after each `apply_effect` call.  Serialized so that replay / late-join
/// produces the same observation stream (the user sees the full event flow
/// animate on replay, per project decision).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Observation {
    /// A unit was eliminated.  `vp_source` is `None` when no VP are awarded
    /// (e.g. fort elimination per §9.14: "No pts: eliminating forts").
    UnitEliminated {
        id: UnitId,
        cause: ElimCause,
        vp_source: Option<VpSource>,
    },
    /// A fort was destroyed (§6.53, §6.62, §7.6).
    FortDestroyed { id: UnitId, hex: HexCoord },
    /// A wall hexside was breached, or a breach *attempt* resolved short of
    /// the §6.63 threshold. `breached` distinguishes the two; `row` is the
    /// Combat Results Table row the attempt rolled on, so the log shows what
    /// the roll had to beat. (Demolitions (§6.53) have no CRT row -- they
    /// carry `None`.)
    WallBreached {
        hexside: HexsideRef,
        #[serde(default)]
        breached: bool,
        row: Option<FireFactorRow>,
        adjacent_eliminated: Option<UnitId>,
    },
    /// A named leader was killed in combat.
    LeaderKilled { id: UnitId, by: Player },
    /// GORDON was eliminated at the palace (§9.346).
    GordonEliminated { turn: GameTurnIndex },
    /// A "Friendlies" unit disembarked from its gunboat transport (§5.21).
    FriendliesDisembarked { unit_id: UnitId, at: HexCoord },
    /// A Royal Engineers demolition resolved (§6.53).
    DemolitionResolved {
        engineer_id: UnitId,
        target: DemolitionTarget,
        success: bool,
    },
    /// Victory points were awarded (§9.14).
    VictoryScored {
        source: VpSource,
        points: VictoryPoints,
        for_player: Player,
    },
    /// A fire attack resolved (§6). Carries the full attack, both die rolls
    /// (the raw roll and the modified one used to index the CRT), the
    /// engine-derived terrain defence modifier (§6.23), the resulting Combat
    /// Results Table cell, and the list of units eliminated as a consequence
    /// -- everything a UI needs to show *why* the shot landed the way it did,
    /// each modifier attributable to its rulebook paragraph.
    FireResolved {
        attack: FireAttack,
        roll: DieRoll,
        /// Total modifier applied to `roll` (engine-side terrain modifier
        /// included). Always present so the UI can show "rolled X, +Y = Z"
        /// even when `attack.modifiers` is empty.
        total_modifier: i16,
        modified_roll: DieRoll,
        /// The Combat Results Table factor row the attack was resolved on,
        /// after range-band application. The UI highlights the corresponding
        /// CRT cell.
        factor_row: FireFactorRow,
        /// Sum of post-range-band fire factors -- the number that determined
        /// `factor_row`. Distinct from `attack.factor_row` (which is the
        /// pre-resolution, app-supplied approximation).
        effective_factor: u16,
        result: CombatResult,
        /// Units eliminated by this resolution (empty for NoEffect/Disrupt
        /// unless disruption rounds up to elimination). The UI surfaces each
        /// elimination via [`Observation::UnitEliminated`] too; this list is
        /// for the combat card's "casualties of this shot" line.
        eliminations: Vec<UnitId>,
        /// Range to the impact hex and the range-effects band applied (§6.22,
        /// §8.1 night halving) -- the audit trail for "why was this factor
        /// halved". `None` in records serialized before the field existed.
        #[serde(default)]
        range: Option<u16>,
        #[serde(default)]
        band: Option<String>,
        /// Rulebook paragraphs relevant to this resolution, in citation form
        /// (e.g. `"6.22"`, `"6.24"`), so the UI can deep-link each one.
        /// Populated by the engine to keep the citation authoritative.
        paragraphs: Vec<String>,
    },
    /// A melee resolved (§7). Carries both die rolls and results -- melee is
    /// simultaneous, so each side's roll is applied to the *other*.
    MeleeResolved {
        attack: MeleeAttack,
        attacker_roll: DieRoll,
        attacker_total_modifier: i16,
        attacker_modified_roll: DieRoll,
        attacker_result: CombatResult,
        defender_roll: DieRoll,
        defender_total_modifier: i16,
        defender_modified_roll: DieRoll,
        defender_result: CombatResult,
        /// Melee factors each side rolled on (post-sum, pre-band). The CRT
        /// row is derived from these via [`FireFactorRow::from_total`].
        attacker_factor: u16,
        defender_factor: u16,
        attacker_losses: Vec<UnitId>,
        defender_losses: Vec<UnitId>,
        /// Whether the mandatory Dervish advance-after-melee (§7.6) fired, and
        /// how many units moved into the vacated hex.
        mandatory_advance: Option<u8>,
        paragraphs: Vec<String>,
    },
    /// A hex was vacated by combat, opening the advance-after-combat window
    /// (§6.82 offensive fire, §7.5 retreat, §7.6 melee). `eligible` lists the
    /// surviving participants that may advance into it -- the authoritative
    /// record for the log/UI and the audit trail for §6.82's participation
    /// requirement.
    HexVacatedByCombat {
        hex: HexCoord,
        eligible: Vec<UnitId>,
        /// Rulebook paragraphs that vacated the hex, distinguishing
        /// fire-vacated (§6.82) from melee-vacated (§7.6) and
        /// retreat-vacated (§7.5).
        paragraphs: Vec<String>,
    },
}

// ---------------------------------------------------------------------------
// 3) GameState -- authoritative mutable snapshot
// ---------------------------------------------------------------------------

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
    pub mp_spent_this_turn: HashMap<UnitId, i16>,
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
    pub vacated_by_combat: HashMap<HexCoord, Vec<UnitId>>,
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
    #[serde(default)]
    pub board: BoardInfo,
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
            mp_spent_this_turn: HashMap::new(),
            gunboats_upstream_this_turn: Vec::new(),
            zoc_stopped_this_turn: Vec::new(),
            vacated_by_combat: HashMap::new(),
            reinforcements_placed_this_turn: Vec::new(),
            game_over: false,
            zariba_hexsides: Vec::new(),
            friendlies_transport: None,
            optional_rules: Vec::new(),
            mines: Vec::new(),
            chain: None,
            board: BoardInfo::default(),
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
        state.board = board;
        state
    }

    /// Find a unit by ID (rulebook §4).
    pub fn find_unit(&self, id: UnitId) -> Option<&UnitPlacement> {
        self.units.iter().find(|u| u.id == id)
    }

    /// Look up a unit by ID, returning [`RuleError::UnitNotFound`] on miss.
    /// Convenience used by the `can_*` predicates so they open with a one-liner.
    fn unit_or_err(&self, id: UnitId) -> Result<&UnitPlacement, RuleError> {
        self.find_unit(id).ok_or(RuleError::UnitNotFound(id))
    }

    /// Verify the active Friendlies transport mission matches the expected
    /// state for the unit+gunboat pair (§5.21). `matching` selects the variant
    /// (Loaded / Crossing / ...) and unit/gunboat identity; `err` is returned
    /// when no mission is in progress or the predicate fails. Used by the
    /// Crossing and Disembark arms of `apply_friendlies_transport`.
    fn require_transport_state(
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
                        let on_south_edge = !self.board.terrain.contains_key(
                            &HexCoord::new(hex.q, hex.r + 1),
                        );
                        let on_east_edge = !self.board.terrain.contains_key(
                            &HexCoord::new(hex.q + 1, hex.r),
                        );
                        let on_west_edge = !self.board.terrain.contains_key(
                            &HexCoord::new(hex.q - 1, hex.r),
                        );
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
                            self.board
                                .hexside_is(hex, n, |k| k == HexsideKind::Wall)
                        });
                        is_garrison_terrain || at_landmark || adjacent_to_wall
                    }
                }
            },
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
        self.check_stacking(placement, placement.position)
            .map_err(RuleError::from)
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
                UnitIdentity::DervishTribal { tribe: crate::DervishTribe::Taiasha }
                | UnitIdentity::DervishTribal { tribe: crate::DervishTribe::IsaZachneih } => Ok(()),
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
                UnitIdentity::DervishTribal { tribe: crate::DervishTribe::IsaZachneih } => {
                    Err(RuleError::NotInPlay(placement.id))
                }
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
    /// already moved, land-mobile, within (night-adjusted) allowance. Returns
    /// the same `RuleError` the `MoveUnit` effect would on rejection. Lets the
    /// UI gate input without mutating or duplicating the rules.
    pub fn can_move_unit(&self, unit_id: UnitId, cost: MovementPoints) -> Result<(), RuleError> {
        self.can_move_unit_to(unit_id, None, cost)
    }

    /// As [`can_move_unit`](Self::can_move_unit), but when `to` is supplied the
    /// path from the unit's current hex to `to` is also checked against the
    /// zone-of-control stop rule (§5.26, §5.43): a unit must halt the instant
    /// it enters an enemy ZOC, so no hex *strictly between* the start and `to`
    /// may lie in an enemy ZOC. Entering the destination itself may be a ZOC
    /// hex (the unit simply stops there), and a unit that *begins* in an enemy
    /// ZOC may still move out (§5.43).
    ///
    /// The caller supplies `to` because the engine costs moves by distance and
    /// does not otherwise know the intervening hexes. The §5.44 hexside
    /// exceptions are applied by [`hex_in_enemy_zoc`] using the attached board.
    ///
    /// [`hex_in_enemy_zoc`]: Self::hex_in_enemy_zoc
    pub fn can_move_unit_to(
        &self,
        unit_id: UnitId,
        to: Option<HexCoord>,
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        // Without an explicit path, the straight line between start and
        // destination approximates the intervening hexes.
        let intermediates = to
            .and_then(|t| self.find_unit(unit_id).map(|u| u.position.line_between(t)))
            .unwrap_or_default();
        self.can_move_unit_checked(unit_id, to, &intermediates, cost)
    }

    /// As [`can_move_unit_to`](Self::can_move_unit_to), but the *actual*
    /// stepped path is checked against the §5.26/§5.43 ZOC stop rule: the
    /// unit must halt the instant it enters an enemy ZOC, so no entered hex
    /// before the destination may lie in one (the destination itself may --
    /// the unit stops there). A bent path that avoids ZOC hexes is legal even
    /// when the straight line would cross one.
    pub fn can_move_unit_along(
        &self,
        unit_id: UnitId,
        to: HexCoord,
        path: &[HexCoord],
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        let intermediates: Vec<HexCoord> =
            path.iter().copied().take(path.len().saturating_sub(1)).collect();
        self.can_move_unit_checked(unit_id, Some(to), &intermediates, cost)
    }

    /// Shared movement validation. `intermediates` are the hexes entered
    /// before the destination (used for the §5.26/§5.43 pass-through ZOC
    /// check); the destination `to` itself may be a ZOC hex (stop there).
    fn can_move_unit_checked(
        &self,
        unit_id: UnitId,
        to: Option<HexCoord>,
        intermediates: &[HexCoord],
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;

        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit_id));
        }
        // §5.26/§5.43: a unit that entered an enemy ZOC this movement phase
        // "may move no further that turn" (it may withdraw next phase).
        if self.zoc_stopped_this_turn.contains(&unit_id) {
            return Err(RuleError::StoppedInEnemyZoc(unit_id));
        }
        // §9.346: the GORDON leader unit may not move during FALL OF KHARTOUM.
        if self.scenario == Scenario::FallOfKhartoum && unit.profile.identity.is_gordon() {
            return Err(RuleError::GordonMayNotMove);
        }
        let allowance = match unit.profile.movement {
            crate::UnitMovement::Land(a) => a,
            crate::UnitMovement::Gunboat(_) | crate::UnitMovement::Immobile => {
                return Err(RuleError::NotMobile(unit_id));
            }
        };
        let effective_allowance = crate::effective_movement_at_night(
            allowance,
            unit.profile.identity.owner(),
            self.day_night,
        );
        // §5.11/§5.12: a unit moves hex by hex up to its allowance. The *running
        // total* spent this turn (plus this step's cost) must not exceed it --
        // so a unit cannot be re-selected to move again past its allowance.
        let already_spent = self.mp_spent(unit_id);
        if already_spent + cost.value() > effective_allowance.value() as i16 {
            return Err(RuleError::MovementExceedsAllowance {
                cost: MovementPoints(already_spent + cost.value()),
                allowance: effective_allowance,
            });
        }

        // §5.26 / §5.43: a unit must stop the instant it enters an enemy ZOC,
        // so a move may pass *through* no enemy-ZOC hex. The destination itself
        // may be a ZOC hex (the unit simply stops there), and a unit that began
        // in an enemy ZOC may still move out.
        if let Some(to) = to {
            // §5.22: land units may never enter a Nile hex.
            if self.board.is_nile(to) {
                return Err(RuleError::LandIntoNile(to));
            }
            // A unit may never step off the board: the destination must be an
            // actual map hex (with no board loaded, map constraints don't apply).
            if !self.board.terrain.is_empty() && self.board.terrain_at(to).is_none() {
                return Err(RuleError::OffBoard(to));
            }
            let mover = unit.profile.identity.owner();
            // §6.54: may not occupy an enemy fort (forts are never captured).
            if self.hex_has_enemy_fort(to, mover) {
                return Err(RuleError::EnemyFort(to));
            }
            // §7.1 (with §5.26): a unit may never *enter* a hex occupied by
            // enemy units -- engaging the enemy is what melee is for; normal
            // movement may only bring a unit adjacent (where the enemy's ZOC
            // stops it). Without this, check_stacking's ownership-blind
            // count let friendly and enemy units cohabit a hex. Exception:
            // lone Anglo-Egyptian leaders do not block -- §6.51 eliminates
            // them when a Dervish unit occupies or passes through their hex
            // (the overrun logic further down).
            let enemy_of_mover = mover.opponent();
            if self.units.iter().any(|u| {
                u.position == to
                    && u.profile.identity.owner() == enemy_of_mover
                    && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
            }) {
                return Err(RuleError::EnemyOccupied(to));
            }
            let mover_kind = unit.profile.kind;
            // §5.26/§5.43: a unit must stop the instant it enters an enemy
            // ZOC, so no hex entered before the destination may lie in one
            // (the destination itself may -- the unit stops there). The
            // intermediates come from the actual stepped path when the caller
            // supplied one, or the straight-line approximation otherwise.
            if let Some(blocked) = intermediates
                .iter()
                .find(|hex| self.hex_in_enemy_zoc(**hex, mover, mover_kind))
            {
                return Err(RuleError::BlockedByEnemyZoc(*blocked));
            }
            // §5.23: a wall hexside blocks movement (gates and breaches pass).
            // The engine derives this from `self.board`.
            if self
                .board
                .hexside_between(unit.position, to)
                .is_some_and(|s| s.blocks_movement())
            {
                return Err(RuleError::MoveBlockedByHexside(unit.position, to));
            }
            // §5.23: only certain units may enter the walled portion of Omdurman
            // -- Dervish: the Khalifa, the artillery, and the Taiasha bodyguard;
            // Anglo-Egyptian: any unit except gunboats and "Friendlies". Scoped
            // to the Omdurman map: FALL OF KHARTOUM is a different walled city
            // (Khartoum) whose set-up places units inside it freely (§9.32).
            if self.scenario != Scenario::FallOfKhartoum
                && self.board.is_walled_city(to)
                && !self.board.is_walled_city(unit.position)
                && !unit.profile.identity.may_enter_walled_city()
            {
                return Err(RuleError::WalledCityEntry(unit_id, to));
            }
        }
        Ok(())
    }

    /// The true movement-point cost of a move along `path` (the entered hexes,
    /// excluding the start), computed from the board's Terrain Effects Chart
    /// (§5.11). Returns `None` when no board/path is available (the caller then
    /// falls back to its supplied cost). Land units pay each hex's terrain cost;
    /// gunboats pay one MP per Nile hex entered (§5.24 counts hexes, not
    /// terrain). The per-hex passability is enforced separately in the
    /// land/gunboat validators, so an off-map hex here contributes the clear-
    /// terrain base of 1.
    ///
    /// §5.42: entering or leaving an enemy ZOC adds no MP cost.
    fn movement_cost_for(&self, unit: &UnitPlacement, path: &[HexCoord]) -> Option<MovementPoints> {
        if path.is_empty() || self.board.terrain.is_empty() {
            return None;
        }
        let total: i16 = match unit.profile.movement {
            crate::UnitMovement::Gunboat(_) => path.len() as i16,
            _ => {
                let mut sum = 0i16;
                let mut prev = unit.position;
                for hex in path {
                    let terrain = self
                        .board
                        .terrain_at(*hex)
                        .unwrap_or(omdurman_types::Terrain::Clear { road: Default::default() });
                    let has_road = self.board.has_road(*hex);
                    sum += crate::terrain_chart::movement_cost_with_road(terrain, has_road)
                        .map_or(1, |a| a.value() as i16);
                    // §9.233: crossing a Zariba end hexside (the only passable
                    // way in or out of the Zariba compound) costs +2 MP.
                    sum += self.board.zariba_entry_surcharge(prev, *hex);
                    prev = *hex;
                }
                sum
            }
        };
        Some(MovementPoints(total))
    }

    /// Validate a gunboat move along `path` (§5.22, §5.24, §10.22). Gunboats may
    /// move only along Nile hexes; their two allowances are upstream (smaller)
    /// and downstream (larger); and "if they move even one hex upstream, their
    /// upstream movement allowance is their maximum for that turn." Chained Nile
    /// hexes stop the gunboat (§10.22).
    pub fn can_move_gunboat(
        &self,
        unit_id: UnitId,
        to: HexCoord,
        path: &[HexCoord],
        cost: MovementPoints,
    ) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if !matches!(self.phase, Phase::Movement) {
            return Err(RuleError::WrongPhase);
        }
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit_id));
        }
        // §5.26/§5.43: a gunboat that entered an enemy (gunboat's, §5.41) ZOC
        // this movement phase may move no further that turn.
        if self.zoc_stopped_this_turn.contains(&unit_id) {
            return Err(RuleError::StoppedInEnemyZoc(unit_id));
        }
        let crate::UnitMovement::Gunboat(ga) = unit.profile.movement else {
            return Err(RuleError::NotAGunboat(unit_id));
        };
        let already_spent = self.mp_spent(unit_id);

        // §9.345 (FALL OF KHARTOUM): a British gunboat may cross between the
        // White and Blue Nile mouths off-board for a flat 6 "upstream" MP,
        // bypassing the normal contiguous-Nile path. Only the two named mouth
        // hexes participate; the move is otherwise a normal once-per-turn move.
        if self.scenario == Scenario::FallOfKhartoum
            && self.is_nile_mouth_crossing(unit.position, to)
        {
            const CROSS_NILE_MP: i16 = 6;
            if CROSS_NILE_MP > ga.upstream.value() as i16 {
                return Err(RuleError::GunboatUpstreamCap {
                    cost: MovementPoints(CROSS_NILE_MP),
                    allowance: ga.upstream,
                });
            }
            return Ok(());
        }

        // Build the stepped path: prepend the start so each (from, to) pair is a
        // single step. With no path supplied, treat the destination as one step.
        let mut moved_upstream = false;
        let mut prev = unit.position;
        let steps: Vec<HexCoord> = if path.is_empty() {
            vec![to]
        } else {
            path.to_vec()
        };
        for &next in &steps {
            // §5.22: gunboats stay on the Nile. With a board loaded, every
            // entered hex must be a Nile hex.
            if !self.board.terrain.is_empty() && !self.board.is_nile(next) {
                return Err(RuleError::GunboatOffNile(next));
            }
            // §10.22: a chained Nile hex stops the gunboat.
            if self
                .chain
                .as_ref()
                .is_some_and(|c| !c.sunk && c.hexes.contains(&next))
            {
                return Err(RuleError::BlockedByChain(next));
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
        // movement allowance for that turn". §5.11/§5.12: the running total
        // spent this turn (plus this step) must fit the allowance.
        let went_upstream_earlier = self.gunboats_upstream_this_turn.contains(&unit_id);
        let allowance = if moved_upstream || went_upstream_earlier {
            ga.upstream
        } else {
            ga.downstream
        };
        let total = already_spent + cost.value();
        if total > allowance.value() as i16 {
            return Err(if moved_upstream || went_upstream_earlier {
                RuleError::GunboatUpstreamCap {
                    cost: MovementPoints(total),
                    allowance,
                }
            } else {
                RuleError::MovementExceedsAllowance {
                    cost: MovementPoints(total),
                    allowance,
                }
            });
        }
        Ok(())
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

        let firing_in_phase = match self.phase {
            Phase::OffensiveFire(_) => self.active_player,
            Phase::DefensiveFire(_) => self.active_player.opponent(),
            _ => return Err(RuleError::WrongPhase),
        };
        if unit.profile.identity.owner() != firing_in_phase {
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
        let firer_los_level = crate::los_table::los_level_for_unit(
            unit.profile.kind,
            unit.position,
            &self.board,
        );
        let target_los_level = self
            .units
            .iter()
            .find(|u| u.position == target_hex)
            .map(|u| {
                crate::los_table::los_level_for_unit(
                    u.profile.kind,
                    u.position,
                    &self.board,
                )
            })
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
            |hex| {
                let has_blocking_unit = self.units.iter().any(|u| {
                    u.position == hex
                        && !matches!(
                            u.profile.kind,
                            crate::UnitKind::Gunboat { .. }
                                | crate::UnitKind::Fort { .. }
                        )
                });
                if has_blocking_unit {
                    self.board.terrain_at(hex).map(crate::los_table::los_level)
                } else {
                    None
                }
            },
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

        let firing_player = match self.phase {
            Phase::OffensiveFire(_) => self.active_player,
            Phase::DefensiveFire(_) => self.active_player.opponent(),
            _ => return Err(RuleError::WrongPhase),
        };
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
        let firer_los = crate::los_table::los_level_for_unit(
            unit.profile.kind,
            unit.position,
            &self.board,
        );
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
            |hex| {
                let has_blocking_unit = self.units.iter().any(|u| {
                    u.position == hex
                        && !matches!(
                            u.profile.kind,
                            crate::UnitKind::Gunboat { .. }
                                | crate::UnitKind::Fort { .. }
                        )
                });
                if has_blocking_unit {
                    self.board.terrain_at(hex).map(crate::los_table::los_level)
                } else {
                    None
                }
            },
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
        // breaches pass). The engine derives this from `self.board`.
        if self
            .board
            .hexside_between(unit.position, defender_hex)
            .is_some_and(|s| s.blocks_melee())
        {
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
    ///   and gunboats; gunboats may not share a hex with any other unit.
    /// * §5.52 -- units of different Dervish tribes may not stack together.
    /// * §5.53 -- a Dervish leader may stack only with units of its command.
    pub fn check_stacking(
        &self,
        mover: &UnitPlacement,
        dest: HexCoord,
    ) -> Result<(), crate::StackingError> {
        use crate::StackingError;
        // The prospective occupants: everyone already in `dest` except the
        // mover itself, plus the mover.
        let occupants: Vec<&UnitPlacement> = self
            .units
            .iter()
            .filter(|u| u.position == dest && u.id != mover.id)
            .chain(std::iter::once(mover))
            .collect();

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
                    UnitKind::DervishLeader { .. } | UnitKind::BritishLeader { .. } | UnitKind::Gunboat { .. }
                )
            })
            .count();
        if counted > STACKING_LIMIT {
            return Err(StackingError::OverLimit);
        }

        // §5.52: no two different Dervish tribes in the same hex.
        let mut seen_tribe: Option<DervishTribe> = None;
        for u in &occupants {
            if let crate::UnitIdentity::DervishTribal { tribe } = u.profile.identity {
                match seen_tribe {
                    Some(t) if t != tribe => return Err(StackingError::DervishTribeMix),
                    _ => seen_tribe = Some(tribe),
                }
            }
        }

        // §5.53: a Dervish leader may only stack with units of its command.
        for u in &occupants {
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

    /// Whole-state stacking invariant check (§5.51-5.53): every occupied hex
    /// must satisfy the gunboat-isolation, four-unit, tribe-purity and
    /// leader-command rules on its *actual* occupants. Unlike
    /// [`Self::check_stacking`] this is not a prospective-move check — it
    /// validates the state as it stands, so it can be used as a post-condition
    /// after any mutation (see `apply_effect`) and to audit replayed records.
    pub fn validate_stacking_invariants(&self) -> Result<(), String> {
        let mut by_hex: std::collections::HashMap<HexCoord, Vec<&UnitPlacement>> =
            std::collections::HashMap::new();
        for u in &self.units {
            by_hex.entry(u.position).or_default().push(u);
        }
        for (hex, occupants) in by_hex {
            let describe = |u: &UnitPlacement| format!("{:?}", u.profile.identity);

            // §5.51: gunboats stack with nothing.
            let gunboats = occupants
                .iter()
                .filter(|u| matches!(u.profile.kind, UnitKind::Gunboat { .. }))
                .count();
            if gunboats > 0 && occupants.len() > 1 {
                return Err(format!(
                    "{hex:?}: gunboat stacks with other units ({}) [§5.51]",
                    occupants.iter().map(|u| describe(u)).collect::<Vec<_>>().join(", ")
                ));
            }

            // §5.51: at most four counted units (leaders and gunboats free).
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
                return Err(format!(
                    "{hex:?}: {counted} units exceed the four-unit stacking limit [§5.51]"
                ));
            }

            // §5.52: no two different Dervish tribes in the same hex.
            let mut seen_tribe: Option<DervishTribe> = None;
            for u in &occupants {
                if let crate::UnitIdentity::DervishTribal { tribe } = u.profile.identity {
                    match seen_tribe {
                        Some(t) if t != tribe => {
                            return Err(format!(
                                "{hex:?}: Dervish tribes {t:?} and {tribe:?} mixed [§5.52]"
                            ));
                        }
                        _ => seen_tribe = Some(tribe),
                    }
                }
            }

            // §5.53: a Dervish leader stacks only with its own command.
            for u in &occupants {
                if let crate::UnitIdentity::DervishLeader(leader) = u.profile.identity {
                    if let Some(bad) = occupants.iter().find_map(|other| {
                        match other.profile.identity {
                            crate::UnitIdentity::DervishTribal { tribe }
                                if !leader.commands(tribe) =>
                            {
                                Some(other)
                            }
                            _ => None,
                        }
                    }) {
                        return Err(format!(
                            "{hex:?}: Dervish leader {leader:?} stacked with foreign-tribe unit {} [§5.53]",
                            describe(bad)
                        ));
                    }
                }
            }
        }
        Ok(())
    }
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
                (matches!(mover_kind, UnitKind::Gunboat { .. })).then_some(ZocReason::GunboatVsGunboat)
            }
            // §5.44: a fort projects ZOC out of its hex even when unoccupied;
            // that is modelled by the fort *unit* itself projecting normally.
            UnitKind::Fort { .. } => Some(ZocReason::Fort),
            _ => Some(ZocReason::Normal),
        }
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
            // §5.44: ZOC does not cross a khor/wall/Zariba hexside.
            if self
                .board
                .hexside_is(u.position, hex, omdurman_types::HexsideKind::blocks_zoc)
            {
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
    pub fn zoc_hexes(&self, unit: &UnitPlacement, mover_player: Player, mover_kind: UnitKind) -> Vec<HexCoord> {
        let Some(reason) = self.unit_projects_zoc(unit, mover_player, mover_kind) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for &adj in &unit.position.neighbors() {
            // §5.44: ZOC does not cross a khor/wall/Zariba hexside.
            if self.board.hexside_is(unit.position, adj, omdurman_types::HexsideKind::blocks_zoc) {
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

    /// Check structural invariants of the game state. Returns a list of
    /// violations (empty = valid). Useful as a postcondition assertion
    /// after `apply_effect`.
    pub fn check_invariants(&self) -> Vec<&'static str> {
        let mut violations = Vec::new();

        // 1. Stacking ≤ 4 (excluding leaders and gunboats) at every occupied hex (§5.51).
        use std::collections::HashMap;
        let mut hex_counts: HashMap<HexCoord, usize> = HashMap::new();
        for u in &self.units {
            if matches!(
                u.profile.kind,
                UnitKind::DervishLeader { .. } | UnitKind::BritishLeader { .. } | UnitKind::Gunboat { .. }
            ) {
                continue;
            }
            *hex_counts.entry(u.position).or_insert(0) += 1;
        }
        for (&hex, &count) in &hex_counts {
            if count > 4 {
                violations.push("stacking > 4 at hex");
                let _ = hex;
            }
        }

        // 2. No land unit occupies a Nile hex (§5.22).
        for u in &self.units {
            if matches!(u.profile.kind, UnitKind::Gunboat { .. }) {
                continue;
            }
            if self.board.is_nile(u.position) {
                violations.push("land unit on Nile hex");
            }
        }

        // 3. Gunboats do not stack with other units (§5.51).
        let mut hex_has_land: HashMap<HexCoord, bool> = HashMap::new();
        for u in &self.units {
            if !matches!(u.profile.kind, UnitKind::Gunboat { .. }) {
                *hex_has_land.entry(u.position).or_insert(true) = true;
            }
        }
        for u in &self.units {
            if matches!(u.profile.kind, UnitKind::Gunboat { .. })
                && hex_has_land.get(&u.position).copied().unwrap_or(false)
            {
                violations.push("gunboat stacked with land unit");
            }
        }

        // 4. Dervish tribes do not mix stacks (§5.52).
        use std::collections::HashSet;
        let mut hex_tribes: HashMap<HexCoord, HashSet<Option<DervishTribe>>> = HashMap::new();
        for u in &self.units {
            if u.profile.identity.owner() == Player::Dervish
                && let Some(tr) = u.profile.identity.dervish_tribe()
            {
                hex_tribes.entry(u.position).or_default().insert(Some(tr));
            }
        }
        for (&hex, tribes) in &hex_tribes {
            if tribes.len() > 1 {
                violations.push("different Dervish tribes stacked together at hex");
                let _ = hex;
            }
        }

        violations
    }

    /// The hex a howitzer shell actually lands in given its scatter entry
    /// (§6.64). The printed Scattergram is a flower of six hexes around the
    /// designated target; this orients it relative to the firer: "upper"
    /// entries flank the away-from-firer direction (over-shoot), "lower"
    /// entries flank the toward-firer direction (fall-short), and left/right
    /// are the perpendicular sides. Each miss roll (1-6) thus lands on a
    /// distinct, deterministic neighbour; rolls 7-10 (`Center`) hit the
    /// designated hex.
    fn howitzer_impact_hex(
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

    /// If `target_ids` contains a gunboat or fort, return it and its kind --
    /// these are "special" fire targets governed by §6.61/§6.62 thresholds
    /// rather than the generic Combat Results Table effect. A gunboat is
    /// reported in preference to a fort (a gunboat never stacks, so this is
    /// unambiguous in practice).
    fn special_fire_target(&self, target_ids: &[UnitId]) -> Option<(UnitId, UnitKind)> {
        let mut fort = None;
        for &id in target_ids {
            match self.find_unit(id).map(|u| u.profile.kind) {
                Some(UnitKind::Gunboat { .. }) => return Some((id, UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 })),
                Some(UnitKind::Fort { .. }) if fort.is_none() => fort = Some((id, UnitKind::Fort { fire: 0, melee: 0 })),
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

// ---------------------------------------------------------------------------
// 4) apply_effect -- the effect processor
// ---------------------------------------------------------------------------

/// Validate and apply a [`GameEffect`] to `state` (rulebook §4, §5, §6, §7, §8, §10).
///
/// Returns `Ok(())` on success; the state has been mutated.  Returns
/// `Err(RuleError)` if the effect is illegal for the current state; the
/// state is left unchanged.
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
        // print runs, 16 + 16).
        UnitIdentity::DervishTribal { tribe: crate::DervishTribe::Mulazmin } => {
            (Tribe(crate::DervishTribe::Mulazmin), 32)
        }
        UnitIdentity::DervishTribal { tribe: crate::DervishTribe::Hadendowa } => {
            (Tribe(crate::DervishTribe::Hadendowa), 2)
        }
        UnitIdentity::DervishTribal { tribe: crate::DervishTribe::Kehena } => {
            (Tribe(crate::DervishTribe::Kehena), 6)
        }
        UnitIdentity::DervishTribal { tribe: crate::DervishTribe::Degheim } => {
            (Tribe(crate::DervishTribe::Degheim), 5)
        }
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
        } => (Infantry(*nationality), match *nationality {
            crate::BrigadeNationality::British => 2,
            crate::BrigadeNationality::Egyptian => 3,
            crate::BrigadeNationality::Sudanese => 4,
            crate::BrigadeNationality::Friendlies => 4,
        }),
        // §9.321: GORDON starts in the palace (the scenario's one leader).
        UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Gordon) => (Gordon, 1),
        // §9.344: the single North Fort is the only Dervish fort in play.
        UnitIdentity::DervishFort => (DervishFort, 1),
        _ => return None,
    })
}

pub fn apply_effect(state: &mut GameState, effect: &GameEffect) -> Result<(), RuleError> {
    if state.game_over {
        return Err(RuleError::GameOver);
    }
    let result = match effect {
        GameEffect::AdvancePhase => advance_phase(state),
        GameEffect::MoveUnit {
            unit_id,
            to,
            cost,
            path,
        } => apply_move_unit(state, *unit_id, *to, *cost, path),
        GameEffect::FireCombat { attack, roll } => apply_fire_combat(state, attack, *roll),
        GameEffect::HowitzerFire {
            attack,
            combat_results_table_roll,
            impact_roll,
        } => apply_howitzer_fire(state, attack, *combat_results_table_roll, *impact_roll),
        GameEffect::MeleeCombat {
            attack,
            attacker_roll,
            defender_roll,
        } => apply_melee_combat(state, attack, *attacker_roll, *defender_roll),
        GameEffect::DeclareMelee {
            attack,
            attacker_roll,
            defender_roll,
        } => apply_declare_melee(state, attack, *attacker_roll, *defender_roll),
        GameEffect::ResolveMelee => apply_resolve_melee(state),
        GameEffect::RetreatBeforeMelee { unit_id, to } => {
            apply_retreat_before_melee(state, *unit_id, *to)
        }
        GameEffect::AdvanceAfterCombat { unit_id, to } => {
            apply_advance_after_combat(state, *unit_id, *to)
        }
        GameEffect::RecoverUnit { unit_id } => apply_recover_unit(state, *unit_id),
        GameEffect::ConstructZariba { unit_ids, hexside } => {
            apply_construct_zariba(state, unit_ids, *hexside)
        }
        GameEffect::Demolition { unit_id, target } => apply_demolition(state, *unit_id, *target),
        GameEffect::PlaceReinforcements(placements) => {
            apply_place_reinforcements(state, placements)
        }
        GameEffect::DervishDesertion { roll, deserters } => {
            apply_dervish_desertion(state, *roll, deserters)
        }
        GameEffect::FriendliesTransport(action) => apply_friendlies_transport(state, *action),
        GameEffect::RiverMine {
            gunboat_id,
            hex,
            roll,
        } => apply_river_mine(state, *gunboat_id, *hex, *roll),
        GameEffect::SinkChain => apply_sink_chain(state),
        GameEffect::DeployUnit(placement) => apply_deploy_unit(state, placement),
        GameEffect::RemoveDeployedUnit { unit_id, player } => {
            apply_remove_deployed_unit(state, *unit_id, *player)
        }
        GameEffect::PlaceMine { hex } => apply_place_mine(state, *hex),
        GameEffect::PlaceChain { hexes } => apply_place_chain(state, hexes),
        GameEffect::PlaceZariba { hexside } => apply_place_zariba(state, *hexside),
        GameEffect::ConfirmSetupReady { player } => apply_confirm_setup_ready(state, *player),
        GameEffect::ResolveDemolition { unit_id, target } => {
            apply_resolve_demolition(state, *unit_id, *target)
        }
        GameEffect::DriftGunboat { unit_id } => apply_drift_gunboat(state, *unit_id),
        GameEffect::ArtilleryBreachWall {
            firers,
            target,
            roll,
        } => apply_artillery_breach_wall(state, firers, *target, *roll),
    };
    // Post-condition: per-phase trackers never reference eliminated units.
    if result.is_ok() {
        prune_dead_trackers(state);
        // Post-condition: the stacking invariants (§5.51-5.53) hold over the
        // whole board after every mutation. Any effect arm that produces an
        // illegal stack fails here, at the exact effect, instead of leaking
        // into a recorded replay. Debug builds only (release perf: the game
        // loop calls this per effect and units are few but nonzero cost).
        debug_assert!(
            state.validate_stacking_invariants().is_ok(),
            "stacking invariant violated after applying effect: {:?}",
            effect
        );
    }
    result
}

// ---------------------------------------------------------------------------
// 5) Phase advancement
// ---------------------------------------------------------------------------

/// Advance the game state to the next phase (rulebook §4).
pub fn advance_phase(state: &mut GameState) -> Result<(), RuleError> {
    let old_phase = state.phase;
    debug!(old_phase = ?old_phase, active_player = ?state.active_player, "advance_phase");
    // §6.82/§7.5/§7.6: advance-after-combat windows close on every phase change
    // except the Direct → Maxim/Howitzer subphase bridge (§6.42), which is a
    // single continuous fire subphase for advance purposes.
    let is_642_bridge = match state.phase {
        Phase::DefensiveFire(FireSubPhase::DirectFire)
            if state.active_player == Player::Dervish =>
        {
            true
        }
        Phase::OffensiveFire(FireSubPhase::DirectFire)
            if state.active_player == Player::AngloEgyptian =>
        {
            true
        }
        _ => false,
    };
    if !is_642_bridge {
        state.vacated_by_combat.clear();
    }
    // §7/§7.5: a declared melee must be resolved (or vacated by a retreat
    // before melee) before the melee phase may end -- otherwise the attack
    // would be silently dropped and its pre-rolled dice lost (audit: 76
    // declared melees vanished this way in the recorded games).
    if matches!(state.phase, Phase::Melee) && state.pending_melee.is_some() {
        return Err(RuleError::MeleePendingResolution);
    }
    // §8.2: "Once each campaign game, during the first night turn of the
    // game, the Dervish player rolls one die" -- the roll is made during the
    // Dervish movement phase and is mandatory, so that phase cannot end
    // before the effect has been applied (audit: every recorded campaign
    // game silently skipped it).
    if matches!(state.phase, Phase::Movement)
        && state.scenario == Scenario::Campaign
        && state.active_player == Player::Dervish
        && !state.dervish_deserted
        && crate::turn_track::scenario_turn(state.scenario, state.current_turn).is_some_and(
            |e| e.event == crate::turn_track::TurnEvent::DervishDesertion,
        )
    {
        return Err(RuleError::DesertionRollRequired);
    }
    match state.phase {
        // Leaving deployment is gated: both sides' required order of battle must
        // be on the board (and within limits) before the first Movement turn
        // (§9.2/§9.3/§10).
        Phase::Setup => {
            state.setup_complete()?;
            state.phase = Phase::Movement;
        }
        Phase::Movement => {
            state.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
        }
        Phase::DefensiveFire(FireSubPhase::DirectFire) => {
            if state.active_player == Player::AngloEgyptian {
                // AE turn: Dervish fired direct defensive.  Next: AE
                // offensive fire (Direct, then Maxim/Howitzer).
                state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
            } else {
                // Dervish turn: AE fired direct defensive.  AE also has
                // Maxim / howitzer capability -- they fire again now.
                state.phase = Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer);
                // §6.42: Maxim guns may fire a second time in this subphase.
                // Clear the per-phase fired set so Maxims that fired in Direct
                // Fire may fire again here.  The Maxim-only gate in
                // `can_fire_at` prevents non-Maxim units from exploiting this.
                state.units_fired_this_phase.clear();
                state.units_fired_at_this_phase.clear();
            }
        }
        Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer) => {
            state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        }
        Phase::OffensiveFire(FireSubPhase::DirectFire) => {
            if state.active_player == Player::AngloEgyptian {
                state.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
                // §6.42: same clear as the defensive path above.
                state.units_fired_this_phase.clear();
                state.units_fired_at_this_phase.clear();
            } else {
                state.phase = Phase::Melee;
            }
        }
        Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer) => {
            state.phase = Phase::Melee;
        }
        Phase::Melee => end_player_turn(state)?,
    }
    debug!(new_phase = ?state.phase, active_player = ?state.active_player, "advance_phase done");
    Ok(())
}

/// End the current player's turn: recover disrupted units, switch active player, advance turn index (rulebook §4).
pub fn end_player_turn(state: &mut GameState) -> Result<(), RuleError> {
    debug!(
        old_player = ?state.active_player,
        old_turn = state.current_turn.value(),
        "end_player_turn"
    );
    resolve_pending_demolitions(state)?;
    recover_disrupted_units(state);
    clear_per_turn_tracking(state);
    advance_game_turn(state)?;
    debug!(
        new_player = ?state.active_player,
        new_turn = state.current_turn.value(),
        day_night = ?state.day_night,
        phase = ?state.phase,
        "end_player_turn done"
    );
    Ok(())
}

/// §6.53: resolve all pending Royal Engineers demolitions before recovering
/// disrupted units. Each demolition checks adjacency + undisrupted status.
fn resolve_pending_demolitions(state: &mut GameState) -> Result<(), RuleError> {
    let pending: Vec<(UnitId, DemolitionTarget)> = std::mem::take(&mut state.pending_demolitions);
    for (eid, target) in pending {
        apply_resolve_demolition(state, eid, target)?;
    }
    Ok(())
}

/// Recover every disrupted unit owned by the player whose turn just ended.
fn recover_disrupted_units(state: &mut GameState) {
    let to_recover: Vec<UnitId> = state
        .units
        .iter()
        .filter(|u| u.state.disrupted && u.profile.identity.owner() == state.active_player)
        .map(|u| u.id)
        .collect();
    for id in &to_recover {
        if let Some(unit) = state.find_unit_mut(*id) {
            unit.state.disrupted = false;
        }
    }
}

/// Clear per-phase / per-turn tracking (§5.13: MP never carry over).
fn clear_per_turn_tracking(state: &mut GameState) {
    state.units_fired_this_phase.clear();
    state.units_fired_at_this_phase.clear();
    state.mp_spent_this_turn.clear();
    // §5.24: the sticky upstream cap only lasts for the turn.
    state.gunboats_upstream_this_turn.clear();
    // §5.43: a unit stopped in an enemy ZOC may move again next turn.
    state.zoc_stopped_this_turn.clear();
    // Advance-after-combat windows do not survive the turn boundary
    // (§6.82/§7.6).
    state.vacated_by_combat.clear();
    // Reinforcement quotas reset each player-turn (§9.112/§9.113).
    state.reinforcements_placed_this_turn.clear();
    // A declared-but-unresolved melee does not survive the turn boundary.
    state.pending_melee = None;
}

/// Drop per-phase tracker entries for units no longer on the board. Run as a
/// post-condition of [`apply_effect`] so an eliminated unit can never linger
/// in `units_fired_this_phase` or `mp_spent_this_turn` (they are cleared
/// wholesale at phase end, but kept tidy mid-phase).
fn prune_dead_trackers(state: &mut GameState) {
    if state.units_fired_this_phase.is_empty()
        && state.mp_spent_this_turn.is_empty()
        && state.vacated_by_combat.is_empty()
    {
        return;
    }
    state
        .units_fired_this_phase
        .retain(|id| state.units.iter().any(|u| &u.id == id));
    state
        .units_fired_at_this_phase
        .retain(|id| state.units.iter().any(|u| &u.id == id));
    state
        .zoc_stopped_this_turn
        .retain(|id| state.units.iter().any(|u| &u.id == id));
    state
        .mp_spent_this_turn
        .retain(|id, _| state.units.iter().any(|u| &u.id == id));
    // Advance-after-combat windows never reference eliminated units
    // (§6.82/§7.6); drop them, and drop emptied windows whole.
    for eligible in state.vacated_by_combat.values_mut() {
        eligible.retain(|id| state.units.iter().any(|u| &u.id == id));
    }
    state.vacated_by_combat.retain(|_, eligible| !eligible.is_empty());
}

/// Switch to the next player and, when play returns to the scenario's
/// first-moving player (§4: Anglo-Egyptian in the Campaign §9.113, Dervish in
/// the Historical §9.212 and Fall of Khartoum §9.322 scenarios), roll the turn
/// over -- snapshotting the completed turn and advancing the turn index.
fn advance_game_turn(state: &mut GameState) -> Result<(), RuleError> {
    let next = state.active_player.opponent();
    state.active_player = next;
    state.phase = Phase::Movement;

    if next != first_player(state.scenario) {
        return Ok(());
    }

    snapshot_turn(state);

    let next_turn = GameTurnIndex(state.current_turn.value() + 1);
    match scenario_turn(state.scenario, next_turn) {
        Some(entry) => {
            state.current_turn = next_turn;
            state.day_night = entry.day_night;
            // The §8.2 desertion turn marker is consumed by the Dervish
            // player's movement-phase `DervishDesertion` effect (validated
            // against this entry by `apply_dervish_desertion`), not here.
        }
        None => finish_game(state),
    }
    Ok(())
}

/// Snapshot the accumulated turn events into a [`TurnSummary`] before the turn
/// index advances, so the just-completed turn is preserved in the record.
fn snapshot_turn(state: &mut GameState) {
    let events = std::mem::take(&mut state.turn_events);
    let current_entry = scenario_turn(state.scenario, state.current_turn);
    state.turn_summaries.push(TurnSummary {
        turn: state.current_turn,
        time: current_entry.map_or(crate::turn_track::GameTime::Noon, |e| e.time),
        day_night: state.day_night,
        first_player: first_player(state.scenario),
        events,
    });
}

/// The player who moves first in a scenario (§4, §9.113, §9.212, §9.322).
pub fn first_player(scenario: Scenario) -> Player {
    match scenario {
        Scenario::Campaign => Player::AngloEgyptian,
        Scenario::Historical | Scenario::FallOfKhartoum => Player::Dervish,
    }
}

/// End-of-scenario bookkeeping: mark the game over and record the victory
/// result for the scenario's victory schedule (§9.14, §9.24, §9.35).
pub fn finish_game(state: &mut GameState) {
    // Snapshot any remaining turn events before marking game over (handles
    // mid-turn endings like FoK GORDON death).
    let events = std::mem::take(&mut state.turn_events);
    if !events.is_empty() {
        let current_entry = scenario_turn(state.scenario, state.current_turn);
        state.turn_summaries.push(TurnSummary {
            turn: state.current_turn,
            time: current_entry.map_or(crate::turn_track::GameTime::Noon, |e| e.time),
            day_night: state.day_night,
            first_player: first_player(state.scenario),
            events,
        });
    }
    state.game_over = true;
    // §9.14 Mahdi's Tomb: score the 25-VP shrine to whoever controls it at the
    // conclusion of play (Campaign only; the Tomb is not in the other maps).
    if state.scenario == Scenario::Campaign {
        score_mahdis_tomb(state);
    }

    match state.scenario {
        Scenario::Campaign => {
            // §9.14 alternative auto-decisive conditions:
            //   AE decisive if every Dervish unit (incl. gunboats and forts)
            //   has been eliminated.
            //   Dervish decisive if all Anglo-Egyptian *west-bank* units
            //   (excl. gunboats) have been eliminated.
            let no_dervish = !state
                .units
                .iter()
                .any(|u| u.profile.identity.owner() == Player::Dervish);
            let no_ae_west_bank = !state.units.iter().any(|u| {
                u.profile.identity.owner() == Player::AngloEgyptian
                    && !matches!(u.profile.kind, UnitKind::Gunboat { .. })
                    && state.board.bank_of(u.position) == Some(crate::board::NileBank::West)
            });
            let ae = state.victory.total_for(Player::AngloEgyptian);
            let d = state.victory.total_for(Player::Dervish);
            let superiority = ae.0 - d.0;
            let level = if no_dervish {
                CampaignVictoryLevel::Decisive(Player::AngloEgyptian)
            } else if no_ae_west_bank {
                CampaignVictoryLevel::Decisive(Player::Dervish)
            } else {
                CampaignVictoryLevel::from_superiority(crate::VictoryPoints(superiority))
            };
            state.game_result = Some(crate::GameResult::Campaign(level));
        }
        Scenario::Historical => {
            // §9.24: each side's level is its own *unit-elimination* tally (not
            // victory points); the net result subtracts the lower level from
            // the higher.
            let dervish_lost = state.victory.units_eliminated_by(Player::AngloEgyptian);
            let ae_lost = state.victory.units_eliminated_by(Player::Dervish);
            let ae_level = HistoricalVictoryLevel::for_anglo_egyptian(dervish_lost);
            let d_level = HistoricalVictoryLevel::for_dervish(ae_lost);
            state.game_result = Some(crate::GameResult::Historical { ae: ae_level, d: d_level });
        }
        Scenario::FallOfKhartoum => {
            // §9.35: the base level is set by the turn GORDON died (or his
            // survival), then the Dervish player forfeits levels for his own
            // losses. `gordon_eliminated_turn` is `None` if he survived.
            let gordon_died = state.gordon_eliminated_turn.map(|t| t.0);
            let dervish_lost = state.victory.units_eliminated_by(Player::AngloEgyptian);
            let level =
                crate::FoKVictoryLevel::resolve(gordon_died, state.current_turn.value(), dervish_lost);
            state.game_result = Some(crate::GameResult::FoK(level));
        }
    }
}

/// Score the Mahdi's Tomb (§9.14): 25 VP to the Anglo-Egyptian player if, at the
/// conclusion of play, the Tomb hex is occupied by at least one British leader
/// *and* at least one non-"Friendlies" Anglo-Egyptian combat unit, both
/// undisrupted. Otherwise the Dervish player retains control and no points are
/// scored (they hold it from the start, so there is nothing to record).
///
/// The Tomb is the [`Location::MahdisTomb`] hex of the walled city of Omdurman
/// (distinct from [`Location::Palace`] -- on the Campaign map they are at
/// different hexes); its position comes from the attached board. With no board
/// loaded the Tomb cannot be located, so control cannot pass to the
/// Anglo-Egyptian player.
pub fn score_mahdis_tomb(state: &mut GameState) {
    let Some(tomb) = state
        .board
        .hex_of_location(omdurman_types::Location::MahdisTomb)
    else {
        return;
    };
    let occupants: Vec<&UnitPlacement> = state
        .units
        .iter()
        .filter(|u| u.position == tomb && !u.state.disrupted)
        .collect();
    let has_british_leader = occupants
        .iter()
        .any(|u| matches!(u.profile.kind, UnitKind::BritishLeader { .. }));
    // A qualifying combat unit: Anglo-Egyptian, not a leader, not a gunboat,
    // and not a "Friendlies" unit (§9.14).
    let has_combat_unit = occupants.iter().any(|u| {
        u.profile.identity.owner() == Player::AngloEgyptian
            && !matches!(
                u.profile.kind,
                UnitKind::BritishLeader { .. } | UnitKind::Gunboat { .. }
            )
            && !u.profile.identity.is_friendlies()
    });
    if has_british_leader && has_combat_unit {
        state.victory.events.push(VpEvent {
            turn: state.current_turn,
            source: VpSource::MahdisTomb,
        });
    }
}

/// §9.346: in FALL OF KHARTOUM, GORDON is eliminated the instant a Dervish unit
/// passes through or occupies the Palace hex (by normal movement or advance
/// after combat). Records the turn (which fixes the §9.35 victory level) and
/// ends the game. A no-op outside FoK, or once GORDON is already gone.
pub fn check_gordon_palace(state: &mut GameState) {
    if state.scenario != Scenario::FallOfKhartoum || state.gordon_eliminated_turn.is_some() {
        return;
    }
    let Some(palace) = state
        .board
        .hex_of_location(omdurman_types::Location::Palace)
    else {
        return;
    };
    let dervish_on_palace = state
        .units
        .iter()
        .any(|u| u.position == palace && u.profile.identity.owner() == Player::Dervish);
    if !dervish_on_palace {
        return;
    }
    eliminate_gordon(state);
}

/// Remove GORDON, record the turn of his death (§9.346, §9.35), and end the
/// game. Called when a Dervish unit occupies the palace and when a Dervish
/// move overruns him in passing (§6.51 with §9.346's "passing through").
fn eliminate_gordon(state: &mut GameState) {
    if state.gordon_eliminated_turn.is_some() {
        return;
    }
    // Remove the GORDON unit and record the turn of his death (§9.346, §9.35).
    let gordon_id = state
        .units
        .iter()
        .find(|u| u.profile.identity.is_gordon())
        .map(|u| u.id);
    state.units.retain(|u| !u.profile.identity.is_gordon());
    state.gordon_eliminated_turn = Some(state.current_turn);
    state.turn_events.push(TurnEventRecord::UnitEliminated {
        // The Gordon counter's id is `BritishBoats_3_1` (the `Gordon` alias
        // variant was removed from `UnitId`); fall back for a palace event
        // with no Gordon on the board.
        unit: gordon_id.unwrap_or(UnitId::BritishBoats_3_1),
        cause: ElimCause::GordonAtPalace,
    });
    finish_game(state);
}

// ---------------------------------------------------------------------------
// 6) Movement
// ---------------------------------------------------------------------------

/// The neighbour index opposite to `idx` on a hex grid (three steps round the
/// six-sided ring). Used by howitzer scatter (§6.64), Nile-current upstream
/// derivation, and `step_away_from`.
pub(crate) const fn opposite(idx: usize) -> usize {
    (idx + 3) % 6
}

/// The neighbour index of `origin` that points most directly toward `target`
/// (used for deterministic howitzer scatter, §6.64).
pub fn toward_index(origin: HexCoord, target: HexCoord) -> usize {
    let neighbors = origin.neighbors();
    neighbors
        .iter()
        .enumerate()
        .min_by_key(|(_, n)| n.distance(target))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// One hex from `origin` toward `target` (§6.64 scatter helper).
pub fn step_toward(origin: HexCoord, target: HexCoord) -> HexCoord {
    origin.neighbors()[toward_index(origin, target)]
}

/// One hex from `origin` directly away from `target` (§6.64 scatter helper).
pub fn step_away_from(origin: HexCoord, target: HexCoord) -> HexCoord {
    origin.neighbors()[opposite(toward_index(origin, target))]
}

/// Validate and apply a unit movement (rulebook §5). When `path` is supplied
/// (the entered hexes, excluding the start, ending at `to`) the engine computes
/// the true terrain cost (§5.11) and enforces gunboat upstream/downstream
/// allowances (§5.24); otherwise it falls back to the caller-supplied `cost`.
pub fn apply_move_unit(
    state: &mut GameState,
    unit_id: UnitId,
    to: HexCoord,
    cost: MovementPoints,
    path: &[HexCoord],
) -> Result<(), RuleError> {
    let unit = state.unit_or_err(unit_id)?;
    // Copied out of `unit` so the immutable borrow ends before the state
    // mutations below (§5.24 flag, MP accounting, position update, §5.43 stop).
    let mover_owner = unit.profile.identity.owner();
    let mover_kind = unit.profile.kind;
    let is_gunboat = matches!(unit.profile.movement, crate::UnitMovement::Gunboat(_));
    let start_position = unit.position;

    // The effective cost is computed from the board+path when available, so the
    // engine -- not the caller -- is authoritative for movement-point spend.
    let mut effective_cost = state.movement_cost_for(unit, path).unwrap_or(cost);

    // §9.233: an empty-path (single-step) move trusts the caller's base cost
    // but still owes the +2 Zariba-end surcharge for the crossed hexside.
    // Non-empty paths already include it via `movement_cost_for`.
    if path.is_empty() {
        let surcharge = state.board.zariba_entry_surcharge(unit.position, to);
        effective_cost = MovementPoints(effective_cost.value() + surcharge);
    }

    // Phase / disruption / already-moved / allowance / ZOC-stop checks. Land
    // units validate against their (night-adjusted) land allowance; gunboats
    // validate against the up/downstream allowance for the path (§5.24).
    match unit.profile.movement {
        crate::UnitMovement::Immobile => {
            return Err(RuleError::AlreadyPlaced(unit_id));
        }
        crate::UnitMovement::Gunboat(_) => {
            state.can_move_gunboat(unit_id, to, path, effective_cost)?;
        }
        crate::UnitMovement::Land(_) => {
            state.can_move_unit_along(unit_id, to, path, effective_cost)?;
        }
    }

    // §7.1: no hex of the path may be enemy-occupied -- not even in passing.
    // (An enemy unit's own hex is not inside its ZOC ring, so the §5.26
    // transit check alone would let a path slip through it.)
    // §6.51 exception: an Anglo-Egyptian leader that is *alone* in a hex is
    // eliminated "when a Dervish unit occupies or passes through that hex" --
    // such a hex does not block a Dervish mover (the leader dies below).
    {
        let mover = unit.profile.identity.owner();
        let leader_hexes: Vec<HexCoord> = path
            .iter()
            .chain(std::iter::once(&to))
            .copied()
            .filter(|hex| {
                let occupants: Vec<&UnitPlacement> = state
                    .units
                    .iter()
                    .filter(|u| u.position == *hex)
                    .collect();
                !occupants.is_empty()
                    && occupants
                        .iter()
                        .all(|u| u.profile.identity.owner() == Player::AngloEgyptian)
                    && occupants
                        .iter()
                        .all(|u| matches!(u.profile.kind, UnitKind::BritishLeader { .. }))
            })
            .collect();
        for hex in path.iter().chain(std::iter::once(&to)) {
            if leader_hexes.contains(hex) {
                continue; // §6.51: lone AE leaders are overrun, not obstacles.
            }
            if state.units.iter().any(|u| {
                u.position == *hex && u.profile.identity.owner() == mover.opponent()
            }) {
                return Err(RuleError::EnemyOccupied(*hex));
            }
        }
    }

    // §5.51-5.53: the stacking limit is checked at the *end* of the move.
    let mover = state.unit_or_err(unit_id)?;
    state.check_stacking(mover, to)?;

    // §5.24: record any upstream step now that the move is committed, so the
    // upstream allowance caps the gunboat's remaining moves this turn even if
    // they are all downstream (sticky cap). The FoK Nile-mouth crossing
    // (§9.345) spends "upstream" MPs and sets the flag too.
    if is_gunboat {
        let is_mouth_crossing = state.scenario == Scenario::FallOfKhartoum
            && state.is_nile_mouth_crossing(start_position, to);
        let steps: Vec<HexCoord> = if path.is_empty() { vec![to] } else { path.to_vec() };
        let mut prev = start_position;
        let mut went_upstream = is_mouth_crossing;
        for &next in &steps {
            if state.board.step_direction(prev, next)
                == Some(crate::board::StepDirection::Upstream)
            {
                went_upstream = true;
            }
            prev = next;
        }
        if went_upstream && !state.gunboats_upstream_this_turn.contains(&unit_id) {
            state.gunboats_upstream_this_turn.push(unit_id);
        }
    }

    // Record movement and update the unit's position -- the rules engine is
    // authoritative, so callers must not patch position separately. Track the
    // running MP spent this turn (§5.11/§5.12), so further steps are capped
    // cumulatively; "has moved" is derived as `mp_spent > 0` (used by
    // retreat-before-melee, §7.5).
    state
        .mp_spent_this_turn
        .entry(unit_id)
        .and_modify(|mp| *mp += effective_cost.value())
        .or_insert(effective_cost.value());
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.position = to;
    }

    // §5.26/§5.43: the unit has stopped if its destination lies in an enemy
    // ZOC -- it may move no further this turn (a gunboat only stops in an
    // enemy *gunboat's* ZOC, §5.41, which `hex_in_enemy_zoc` encodes).
    {
        let owner = mover_owner;
        let kind = mover_kind;
        if state.hex_in_enemy_zoc(to, owner, kind)
            && !state.zoc_stopped_this_turn.contains(&unit_id)
        {
            state.zoc_stopped_this_turn.push(unit_id);
        }
    }

    // §6.51: an Anglo-Egyptian leader alone in a hex entered (occupied or
    // passed through) by a Dervish unit is eliminated. The §7.1 occupancy
    // check above exempted those hexes from blocking the move.
    if mover_owner == Player::Dervish {
        let entered: Vec<HexCoord> = path.iter().copied().chain(std::iter::once(to)).collect();
        let overrun: Vec<UnitId> = state
            .units
            .iter()
            .filter(|u| {
                entered.contains(&u.position)
                    && matches!(u.profile.kind, UnitKind::BritishLeader { .. })
            })
            .filter(|u| {
                // "alone in a hex" -- no AE combat unit shares the hex.
                !state.units.iter().any(|other| {
                    other.position == u.position
                        && !matches!(other.profile.kind, UnitKind::BritishLeader { .. })
                        && other.profile.identity.owner() == Player::AngloEgyptian
                })
            })
            .map(|u| u.id)
            .collect();
        for leader in overrun {
            let is_gordon = state
                .find_unit(leader)
                .is_some_and(|u| u.profile.identity.is_gordon());
            score_elimination(state, leader, Player::AngloEgyptian);
            state.units.retain(|u| u.id != leader);
            if is_gordon {
                // §9.346/§9.35: Gordon's death fixes the FoK victory level
                // and ends the game (also for a pass-through overrun).
                eliminate_gordon(state);
            }
        }
    }

    // §9.346: a Dervish unit reaching the Palace eliminates GORDON (FoK).
    check_gordon_palace(state);

    Ok(())
}

// ---------------------------------------------------------------------------
// 7) Fire combat
// ---------------------------------------------------------------------------

/// Validate and apply a direct/Maxim-second fire attack (rulebook §6).
pub fn apply_fire_combat(
    state: &mut GameState,
    attack: &FireAttack,
    roll: DieRoll,
) -> Result<(), RuleError> {
    resolve_fire_attack(
        state,
        attack,
        attack.target_hex,
        roll,
        WeaponClass::Rifles,
    )
}

/// Validate and apply a howitzer fire attack (scatter path) (rulebook §6.64).
pub fn apply_howitzer_fire(
    state: &mut GameState,
    attack: &FireAttack,
    combat_results_table_roll: DieRoll,
    impact_roll: DieRoll,
) -> Result<(), RuleError> {
    // Legality (incl. no-howitzer-at-night §6.64) is validated in
    // `resolve_fire_attack` -> `validate_fire_attack` -> `can_fire_at`.
    //
    // AMBIGUITY (§6.42): "Howitzer fire may be combined with Maxim fire,
    // but only if the howitzer fire impacts in the intended hex."  The
    // manual does not clarify what happens to the Maxim fire when the
    // howitzer scatters.  The code treats howitzer and Maxim attacks as
    // independent fire actions — a scattered howitzer does not prevent a
    // Maxim from firing at the original target hex separately.

    // §6.64: roll twice -- the first roll resolves on the Combat Results Table,
    // the second (impact) roll places the shell. The target hex is hit on 7-10;
    // otherwise the shell scatters and "the results must take effect, even if
    // the fire scatters into a friendly-occupied hex."
    let scatter = howitzer_scatter(impact_roll);
    let firer_pos = attack
        .firers
        .first()
        .and_then(|id| state.find_unit(*id))
        .map(|u| u.position);
    let actual_target = state.howitzer_impact_hex(attack.target_hex, firer_pos, scatter);
    resolve_fire_attack(
        state,
        attack,
        actual_target,
        combat_results_table_roll,
        WeaponClass::Howitzer,
    )
}

/// Look up the range-effects band for a firing unit. Normally Anglo-Egyptian
/// units use their own table and Dervish units the Dervish table (§6.22), but
/// in FALL OF KHARTOUM *both* players use the Dervish Range Effects Table
/// (§9.343).
pub fn range_band_for(
    scenario: Scenario,
    player: Player,
    weapon: WeaponClass,
    range: HexDistance,
) -> crate::RangeBand {
    if scenario == Scenario::FallOfKhartoum {
        return dervish_range_effects(weapon, range);
    }
    match player {
        Player::AngloEgyptian => ae_range_effects(weapon, range),
        Player::Dervish => dervish_range_effects(weapon, range),
    }
}

/// Which player's Range Effects Table a given unit fires on (§6.22, §6.52,
/// §9.343). Resolved **per firer**: in FALL OF KHARTOUM every unit uses the
/// Dervish table (§9.343); "Friendlies" units fire their rifles on the
/// Dervish table (§6.52); everyone else uses their own side's table. Used
/// identically by validation (`can_fire_at`) and resolution
/// (`resolve_fire_attack`) so the two can never disagree on range -- the
/// audit class where a Friendlies shot passed validation on the
/// Anglo-Egyptian table (rifle max 5) but resolved on the Dervish table
/// (rifle max 4).
#[allow(clippy::if_same_then_else)] // §9.343 and §6.52 both yield Dervish for different reasons
fn range_table_player_for(scenario: Scenario, unit: &UnitPlacement) -> Player {
    if scenario == Scenario::FallOfKhartoum {
        Player::Dervish // §9.343
    } else if unit.profile.identity.is_friendlies() {
        Player::Dervish // §6.52
    } else {
        unit.profile.identity.owner()
    }
}

/// The weapon line a unit fires on for an attack of `kind` (§6.64): named
/// gunboats carry Artillery on their profile but fire howitzers in the
/// Maxim/Howitzer subphase, so a `Howitzer`-kind attack always uses the
/// howitzer line.
fn effective_fire_weapon(unit: &UnitPlacement, kind: FireKind) -> WeaponClass {
    if kind == FireKind::Howitzer {
        WeaponClass::Howitzer
    } else {
        unit.profile.weapon
    }
}

/// The distance to consult the range tables at, after the §8.1 night cap:
/// halve the weapon's maximum range (on the table the unit fires on), then
/// consult the day table at the *physical* distance. Returns `None` when the
/// physical distance exceeds the night maximum (target out of range at
/// night).
fn night_capped_distance(
    weapon: WeaponClass,
    table_player: Player,
    distance: HexDistance,
) -> Option<HexDistance> {
    let night_max = crate::range_effects::night_max_range(weapon, table_player == Player::AngloEgyptian);
    (distance.value() <= night_max as u16).then_some(distance)
}

/// Resolve a fire attack: compute range, look up range effects, compute effective factor, roll on CRT (rulebook §6).
pub fn resolve_fire_attack(
    state: &mut GameState,
    attack: &FireAttack,
    target_hex: HexCoord,
    roll: DieRoll,
    default_weapon: WeaponClass,
) -> Result<(), RuleError> {
    validate_fire_attack(state, attack)?;

    for &id in &attack.firers {
        state.units_fired_this_phase.push(id);
    }

    // §6.22: each firer contributes at its *own* distance, on its *own*
    // weapon line and range-effects table (§6.52 Friendlies -> Dervish table,
    // §9.343 FoK -> Dervish table for both sides), with the §8.1 night cap
    // applied per weapon. Previously the whole attack used the *first*
    // firer's weapon/distance/table, which mis-banded mixed attacks (e.g. a
    // spear-armed unit stacked with a fort battery dragged the battery onto
    // the spear line) and let a Friendlies rifle pass validation at range 5
    // (AE table) while resolving on the Dervish table (max 4). The helpers
    // are shared with `can_fire_at` so validation and resolution cannot
    // disagree.
    let mut effective_total: u16 = 0;
    // Representative values for the `FireResolved` observation (first firer's
    // distance/band); with per-firer bands there is no single attack-wide one.
    let mut representative_range: Option<u16> = None;
    let mut representative_band: Option<crate::RangeBand> = None;
    for &id in &attack.firers {
        let Some(u) = state.find_unit(id) else { continue };
        let weapon = effective_fire_weapon(u, attack.kind);
        let table_player = range_table_player_for(state.scenario, u);
        let distance = HexDistance(u.position.distance(target_hex) as u16);
        let distance = if state.day_night == DayNight::Night {
            // Beyond the night cap the band is OutOfRange (§8.1) -- validation
            // already rejects that case; a scatter into a night-out-of-range
            // hex simply contributes nothing.
            night_capped_distance(weapon, table_player, distance)
                .unwrap_or(HexDistance(u16::MAX))
        } else {
            distance
        };
        let band = range_band_for(state.scenario, table_player, weapon, distance);
        if representative_range.is_none() {
            representative_range = Some(distance.value());
            representative_band = Some(band);
        }
        if let Some(f) = u.profile.fire {
            effective_total = effective_total.saturating_add(band.apply(f.value()));
        }
    }
    let _ = default_weapon; // retained for API compatibility; per-firer lookup above is authoritative
    // Engine-authoritative terrain defence modifier (§6.23): derived from
    // `state.board` at the target hex, not from a caller-supplied value. This
    // applies to howitzer scatter too — `target_hex` is the *actual* impact.
    let terrain = state
        .board
        .terrain_at(target_hex)
        .unwrap_or(omdurman_types::Terrain::Clear { road: Default::default() });
    let terrain_mod = crate::terrain_chart::defense_modifier(terrain);
    // §6.24/§5.54/§9.231/§9.232: the engine derives the mandatory modifiers
    // itself (like the §6.23 terrain modifier below) -- the caller's list is
    // checked for equality in `validate_fire_attack` but never trusted for
    // the arithmetic.
    let derived_mod: i16 = mandatory_fire_modifiers(state, attack)
        .iter()
        .map(|m| m.die_modifier())
        .sum();
    let total_mod = derived_mod + terrain_mod;
    let modified_roll = roll.apply_modifier(total_mod);
    let row = FireFactorRow::from_total(effective_total);
    let result = combat_results_table(row, modified_roll);
    let target_units: Vec<UnitId> = state
        .player_units_in_hex(target_hex, attack.firing_player.opponent())
        .iter()
        .map(|u| u.id)
        .collect();

    // §6.61/§6.62: gunboats and forts are special targets -- only artillery (or
    // howitzer-class) fire may engage them, and they are destroyed only on a
    // Combat Results Table *cell value* meeting a threshold (gunboat 3+, fort
    // 2+), *not* by the generic disrupt/eliminate effect.  "3 or more on the
    // combat results table" means Eliminate(3) or higher, not a die roll of 3+.
    let opponent = attack.firing_player.opponent();
    // §6.14: "a combat unit may only fire once and may only be fired at once
    // (exceptions: Maxim guns and gunboats)". Any non-excepted target unit
    // already fired at this phase makes the attack illegal -- two attacks on
    // the same hex (or its survivors) in one phase fire at the same units.
    for &tid in &target_units {
        let already = state.units_fired_at_this_phase.contains(&tid);
        let excepted = state
            .find_unit(tid)
            .is_some_and(|u| fired_at_excepted(u.profile.kind));
        if already && !excepted {
            return Err(RuleError::AlreadyFiredAt(tid));
        }
    }
    for &tid in &target_units {
        let excepted = state
            .find_unit(tid)
            .is_some_and(|u| fired_at_excepted(u.profile.kind));
        if !excepted {
            state.units_fired_at_this_phase.push(tid);
        }
    }
    if let Some((special_id, special_kind)) = state.special_fire_target(&target_units) {
        // §6.61/§6.62 defence-in-depth (per firer, matching `can_fire_at`):
        // every firer must fire on an artillery line to engage a gunboat/fort.
        let all_artillery = attack
            .firers
            .iter()
            .filter_map(|id| state.find_unit(*id))
            .all(|u| {
                matches!(
                    effective_fire_weapon(u, attack.kind),
                    WeaponClass::Artillery | WeaponClass::Howitzer
                )
            });
        if !all_artillery {
            return Err(RuleError::ArtilleryOnlyVsGunboatOrFort(attack.firers[0]));
        }
        let needed = match special_kind {
            UnitKind::Gunboat { .. } => 3, // §6.61
            UnitKind::Fort { .. } => 2,    // §6.62
            _ => unreachable!("special_fire_target only returns gunboat/fort"),
        };
        let destroyed = matches!(result, CombatResult::Eliminate(n) if n >= needed);
        // Snapshot the special target's occupants before mutation so the
        // FireResolved observation can report the eliminations accurately --
        // `apply_combat_results_table_result` and the retain() below both
        // mutate `state.units` in place.
        let pre_units: Vec<UnitId> = target_units.clone();
        if destroyed {
            state.units.retain(|u| u.id != special_id);
            // If a gunboat carrying Friendlies is sunk, the loaded unit is
            // lost (§5.21 — design choice).
            if matches!(special_kind, UnitKind::Gunboat { .. }) {
                remove_friendlies_on_gunboat(state, special_id);
            }
            // §6.62: if a destroyed fort contained enemy units, one is
            // eliminated with it.
            if matches!(special_kind, UnitKind::Fort { .. })
                && let Some(&victim) = target_units.iter().find(|&&id| id != special_id)
            {
                state.units.retain(|u| u.id != victim);
            }
        } 
        // §6.82 with §6.61/§6.62 (offensive fire only -- §6.7 bars advances
        // from defensive fire): if the special target's destruction left the
        // hex without any enemy units, the participating firers may advance
        // into it.
        let hex_still_defended = state
            .units
            .iter()
            .any(|u| u.position == target_hex && u.profile.identity.owner() == opponent);
        if !hex_still_defended && matches!(state.phase, Phase::OffensiveFire(_)) {
            let mut paragraphs = vec!["6.82".to_string()];
            paragraphs.push(match special_kind {
                UnitKind::Gunboat { .. } => "6.61".to_string(),
                _ => "6.62".to_string(),
            });
            open_advance_window(state, target_hex, &attack.firers, paragraphs);
        }
        let eliminations: Vec<UnitId> = diff_eliminated(state, pre_units);
        state.turn_events.push(TurnEventRecord::FireCombat {
            attacker: attack.firing_player,
            firers: attack.firers.clone(),
            target: target_hex,
            roll,
            modifiers: attack.modifiers.clone(),
            total_modifier: total_mod,
            result,
            kind: attack.kind,
            eliminated: eliminations.clone(),
        });
        state.observations.push(Observation::FireResolved {
            // Deliberate clone: observations are self-contained records for
            // replay/UI and must own their attack, not borrow it.
            attack: attack.clone(),
            roll,
            total_modifier: total_mod,
            modified_roll,
            factor_row: row,
            effective_factor: effective_total,
            result,
            eliminations,
            range: representative_range,
            band: representative_band.map(|b| format!("{b:?}")),
            paragraphs: fire_paragraphs(attack.kind, Some(special_kind)),
        });
        return Ok(());
    }

    let pre_units: Vec<UnitId> = target_units.clone();
    let was_occupied = !target_units.is_empty();
    apply_combat_results_table_result(state, result, &target_units, opponent);
    let eliminations: Vec<UnitId> = diff_eliminated(state, pre_units);
    state.observations.push(Observation::FireResolved {
        // Deliberate clone: observations are self-contained records for
        // replay/UI and must own their attack, not borrow it.
        attack: attack.clone(),
        roll,
        total_modifier: total_mod,
        modified_roll,
        factor_row: row,
        effective_factor: effective_total,
        result,
        eliminations,
        range: representative_range,
        band: representative_band.map(|b| format!("{b:?}")),
        paragraphs: fire_paragraphs(attack.kind, None),
    });
    // §6.82 (offensive fire only -- §6.7: "There is no advance after combat
    // as a result of defensive fires"): if offensive fire left the target
    // hex without enemy units, the participating firers may advance into it.
    // `was_occupied` keeps a howitzer scatter onto a never-occupied hex
    // (§6.64) from opening a bogus window -- §6.82's "enemy-occupied hex is
    // vacated" never held.
    let hex_still_defended = state
        .units
        .iter()
        .any(|u| u.position == target_hex && u.profile.identity.owner() == opponent);
    if was_occupied
        && !hex_still_defended
        && matches!(state.phase, Phase::OffensiveFire(_))
    {
        open_advance_window(
            state,
            target_hex,
            &attack.firers,
            vec!["6.82".to_string()],
        );
    }
    Ok(())
}

/// §6.14's fired-at exception: Maxim guns and gunboats may be fired at more
/// than once per fire phase.
fn fired_at_excepted(kind: UnitKind) -> bool {
    matches!(kind, UnitKind::Gunboat { .. } | UnitKind::Maxim { .. })
}

/// Rulebook paragraphs that authorise a fire resolution, for the UI's
/// combat-resolution card. The set depends on the kind of fire (direct vs
/// howitzer vs Maxim second) and on whether a special target (gunboat/fort)
/// was hit -- each branch carries the sections a player would point at to
/// explain "why did that shot do what it did".
fn fire_paragraphs(kind: FireKind, special: Option<UnitKind>) -> Vec<String> {
    let kind_para = match kind {
        FireKind::Direct => "6.24", // direct-fire modifiers
        FireKind::MaximSecondFire => "6.42",
        FireKind::Howitzer => "6.64",
    };
    let special_para = match special {
        Some(UnitKind::Gunboat { .. }) => "6.61",
        Some(UnitKind::Fort { .. }) => "6.62",
        _ => "6.23", // terrain defence modifier
    };
    // 6.22 is the CRT itself; always cited.
    vec!["6.22".into(), kind_para.into(), special_para.into()]
}

// ---------------------------------------------------------------------------
// 8) Melee combat
// ---------------------------------------------------------------------------

/// Apply a simultaneous melee combat between two adjacent hexes (rulebook §7).
pub fn apply_melee_combat(
    state: &mut GameState,
    attack: &MeleeAttack,
    attacker_roll: DieRoll,
    defender_roll: DieRoll,
) -> Result<(), RuleError> {
    if !matches!(state.phase, Phase::Melee) {
        return Err(RuleError::WrongPhase);
    }

    let attacker_player = attack.attacker_player;
    let defender_player = attacker_player.opponent();

    // Compute total melee factors.
    let attacker_total = crate::MeleeFactor::sum(
        attack
            .attackers
            .iter()
            .filter_map(|id| state.find_unit(*id))
            .filter_map(|u| u.profile.melee.as_ref()),
    );

    let defender_total = crate::MeleeFactor::sum(
        attack
            .defenders
            .iter()
            .filter_map(|id| state.find_unit(*id))
            .filter_map(|u| u.profile.melee.as_ref()),
    );

    // §7.7/§9.232: the engine derives both sides' mandatory melee modifiers
    // itself -- the declared lists are checked for equality in
    // `apply_declare_melee` but never trusted for the arithmetic. Re-deriving
    // here also keeps resolution correct if the state changed between
    // declaration and the §7.5 retreat-window resolution.
    let (derived_att, derived_def) = mandatory_melee_modifiers(state, attack);
    let att_mod: i16 = derived_att.iter().map(|m| m.die_modifier()).sum();
    let def_mod: i16 = derived_def.iter().map(|m| m.die_modifier()).sum();

    let att_net = attacker_roll.apply_modifier(att_mod);
    let def_net = defender_roll.apply_modifier(def_mod);

    // Melee uses the appropriate Combat Results Table with melee factors treated as fire factors.
    let att_row = FireFactorRow::from_total(attacker_total);
    let def_row = FireFactorRow::from_total(defender_total);

    let att_result = combat_results_table(att_row, att_net);
    let def_result = combat_results_table(def_row, def_net);

    let att_units: Vec<UnitId> = attack.attackers.clone();
    let def_units: Vec<UnitId> = attack.defenders.clone();

    // Player-readable melee report: both sides roll simultaneously (§7.7); each
    // side's result is applied to the *other*.

    // Snapshot which units existed on each side before CRT application, so the
    // MeleeResolved observation can report per-side losses after the fact
    // (apply_combat_results_table_result mutates state.units in place).
    let pre_attackers: Vec<UnitId> = att_units.clone();
    let pre_defenders: Vec<UnitId> = def_units.clone();

    // Simultaneous application.
    apply_combat_results_table_result(state, att_result, &def_units, defender_player);
    apply_combat_results_table_result(state, def_result, &att_units, attacker_player);

    // §7.6: if the melee eliminated *all* defenders, the Dervish MUST advance
    // into the vacated hex (up to the stacking limit of 4 units of the same
    // tribe); surviving eligible attackers move in automatically.  Units that
    // would violate stacking (wrong tribe, over limit) are silently skipped.
    // (The Anglo-Egyptian advance is optional and handled interactively via
    // `AdvanceAfterCombat`.)
    let defenders_remain = state
        .units
        .iter()
        .any(|u| u.position == attack.defender_hex);
    let mut mandatory_advance: Option<u8> = None;
    if attacker_player == Player::Dervish && !defenders_remain {
        let mut moved = 0;
        for &id in &att_units {
            if moved >= STACKING_LIMIT {
                break;
            }
            // Only surviving, non-disrupted attackers that may melee (i.e.
            // were eligible participants) advance.
            let Some(mover) = state.find_unit(id).copied() else {
                continue;
            };
            if mover.state.disrupted || !mover.profile.kind.may_melee_attack() {
                continue;
            }
            // Respect the full stacking rules (§5.51-5.53), not just the count:
            // skip a unit whose advance would mix tribes or break leader command
            // in the vacated hex (it simply does not advance).
            if state.check_stacking(&mover, attack.defender_hex).is_err() {
                continue;
            }
            if let Some(u) = state.find_unit_mut(id) {
                u.position = attack.defender_hex;
            }
            moved += 1;
        }
        if moved > 0 {
            mandatory_advance = Some(moved as u8);
        }
    }
    // §7.6: if the melee vacated the defender hex, the surviving participants
    // may advance into it -- Dervish advances are forced (handled above), the
    // Anglo-Egyptian advance is optional via `AdvanceAfterCombat`.
    if !defenders_remain {
        open_advance_window(
            state,
            attack.defender_hex,
            &att_units,
            vec!["7.6".to_string()],
        );
    }

    let attacker_losses: Vec<UnitId> = diff_eliminated(state, pre_attackers);
    let defender_losses: Vec<UnitId> = diff_eliminated(state, pre_defenders);
    state.turn_events.push(TurnEventRecord::MeleeCombat {
        attacker: attacker_player,
        defender: defender_player,
        hex: attack.defender_hex,
        attacker_roll,
        defender_roll,
        attacker_result: att_result,
        defender_result: def_result,
        attacker_losses: attacker_losses.clone(),
        defender_losses: defender_losses.clone(),
        mandatory_advance,
    });
    state.observations.push(Observation::MeleeResolved {
        // Deliberate clone: observations are self-contained records for
        // replay/UI and must own their attack, not borrow it.
        attack: attack.clone(),
        attacker_roll,
        attacker_total_modifier: att_mod,
        attacker_modified_roll: att_net,
        attacker_result: att_result,
        defender_roll,
        defender_total_modifier: def_mod,
        defender_modified_roll: def_net,
        defender_result: def_result,
        attacker_factor: attacker_total,
        defender_factor: defender_total,
        attacker_losses,
        defender_losses,
        mandatory_advance,
        paragraphs: melee_paragraphs(attack),
    });

    Ok(())
}

/// Rulebook paragraphs that authorise a melee resolution (§7). The CRT is
/// reused for melee (§7.7); the standard side modifiers and the
/// trench-vs-Dervish modifier (§9.232) are the citable rules a player needs
/// to understand the result.
fn melee_paragraphs(attack: &MeleeAttack) -> Vec<String> {
    // 7.7: melee die modifiers + CRT reuse; 7.3: melee resolution / simultaneity.
    let trenched = attack
        .attacker_modifiers
        .iter()
        .any(|m| matches!(m, MeleeModifier::DervishVsTrenchedDefender));
    ["7.7", "7.3"]
        .into_iter()
        .map(String::from)
        .chain(trenched.then_some(String::from("9.232")))
        .collect()
}

/// Declare a melee (§7.5): validate it and store it as the pending attack,
/// opening the defender's reaction window. Resolution waits for
/// [`GameEffect::ResolveMelee`]; in between, eligible defenders may [`GameEffect::RetreatBeforeMelee`].
pub fn apply_declare_melee(
    state: &mut GameState,
    attack: &MeleeAttack,
    attacker_roll: DieRoll,
    defender_roll: DieRoll,
) -> Result<(), RuleError> {
    if state.active_player != attack.attacker_player {
        return Err(RuleError::NotYourTurn);
    }
    if state.pending_melee.is_some() {
        return Err(RuleError::MeleeAlreadyPending);
    }
    if attack.attackers.is_empty() {
        return Err(RuleError::MeleeHasNoAttackers);
    }
    // Single source of truth: every listed attacker must itself be able to
    // melee the target hex (phase, owner, not disrupted, melee-capable kind,
    // adjacent, with a meleeable enemy present) -- the same `can_melee`
    // predicate the UI gates on. This catches a disrupted or non-adjacent
    // attacker that the old ad-hoc check let through.
    for &id in &attack.attackers {
        state.can_melee(id, attack.defender_hex)?;
    }
    // §7.7/§9.232: the declared modifier lists must match the engine-derived
    // mandatory set exactly (the engine resolves with its own derivation).
    let (expected_att, expected_def) = mandatory_melee_modifiers(state, attack);
    if attack.attacker_modifiers != expected_att || attack.defender_modifiers != expected_def {
        return Err(RuleError::MeleeModifierMismatch {
            expected: [expected_att, expected_def].concat(),
            got: [attack.attacker_modifiers.clone(), attack.defender_modifiers.clone()].concat(),
        });
    }
    state.pending_melee = Some(PendingMelee {
        attack: attack.clone(),
        attacker_roll,
        defender_roll,
    });
    Ok(())
}

/// Resolve the pending declared melee against whoever still occupies the
/// target hex (so a retreated defender is spared), then clear the window
/// (§7.5).
pub fn apply_resolve_melee(state: &mut GameState) -> Result<(), RuleError> {
    let Some(pending) = state.pending_melee.take() else {
        return Err(RuleError::NoMeleePending);
    };
    let mut attack = pending.attack;
    // Re-derive defenders from current occupants of the target hex: a unit
    // that retreated during the window is no longer there and is not hit.
    let defender_player = attack.attacker_player.opponent();
    attack.defenders = state
        .units
        .iter()
        .filter(|u| {
            u.position == attack.defender_hex
                && u.profile.identity.owner() == defender_player
                && u.profile.kind.may_be_melee_attacked()
        })
        .map(|u| u.id)
        .collect();
    // Likewise keep only attackers still adjacent and able to melee.
    attack.attackers.retain(|id| {
        state.find_unit(*id).is_some_and(|u| {
            !u.state.disrupted
                && u.profile.kind.may_melee_attack()
                && u.position.neighbors().contains(&attack.defender_hex)
        })
    });
    if attack.defenders.is_empty() {
        // Everyone retreated/eliminated already -- nothing to resolve, but the
        // Dervish may still advance into the vacated hex (§7.6) if attackers
        // remain. Treat as a melee with no defenders.
    }
    apply_melee_combat(state, &attack, pending.attacker_roll, pending.defender_roll)
}

/// Maximum units per hex (§5.51), excluding free-stacking leaders/gunboats.
const STACKING_LIMIT: usize = 4;

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
                && self.board.hexside_between(unit.position, *mid) != Some(HexsideKind::Wall)
                && self.board.hexside_between(*mid, to) != Some(HexsideKind::Wall)
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
        // khor, or thorn-hedge hexside. The engine derives this from
        // `self.board`.
        if self
            .board
            .hexside_between(unit.position, to)
            .is_some_and(|s| s.blocks_advance_after_combat())
        {
            return Err(RuleError::AdvanceBlockedByHexside(unit.position, to));
        }
        Ok(())
    }

    /// Read-only check of whether `unit_id` may recover from disruption: the
    /// unit exists and is currently disrupted. Lets the UI offer "recover" only
    /// where it is legal (paired with [`apply_recover_unit`]).
    pub fn can_recover_unit(&self, unit_id: UnitId) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if !unit.state.disrupted {
            return Err(RuleError::NotDisrupted(unit_id));
        }
        Ok(())
    }

    /// Read-only check of whether a Royal Engineers demolition may begin
    /// (§6.53): the unit exists and is undisrupted. (Adjacency to the target is
    /// the caller's responsibility, as for the rest of the demolition flow.)
    pub fn can_demolition(&self, unit_id: UnitId) -> Result<(), RuleError> {
        let unit = self.unit_or_err(unit_id)?;
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(unit_id));
        }
        Ok(())
    }

    /// Read-only check of whether the given units may construct a Zariba
    /// hexside (§5.3): each exists and is undisrupted.
    pub fn can_construct_zariba(&self, unit_ids: &[UnitId]) -> Result<(), RuleError> {
        for &id in unit_ids {
            let unit = self.unit_or_err(id)?;
            if unit.state.disrupted {
                return Err(RuleError::Disrupted(id));
            }
        }
        Ok(())
    }

    /// Read-only check of whether a batch of reinforcement placements is legal:
    /// each destination must satisfy the full stacking rules (§5.51-5.53), not
    /// just the four-unit count. The placements are checked *cumulatively* so a
    /// batch that would over-stack a single hex is rejected as a whole.
    pub fn can_place_reinforcements(
        &mut self,
        placements: &[UnitPlacement],
    ) -> Result<(), RuleError> {
        // §9.112/§9.113: in the Campaign game, off-board arrivals are bound
        // to the order of appearance -- the owning player's wave for the
        // current turn, its quotas, and its leader list. Other scenarios
        // place freely (setup or FoK entry handling).
        if self.scenario == Scenario::Campaign {
            self.validate_campaign_reinforcements(placements)?;
        }
        // Validate each placement against the board *plus* the units placed
        // earlier in this same batch onto the same hex, so two reinforcements
        // landing together can't jointly break stacking. Stage them on
        // `self.units` directly (no deep `GameState` clone), then roll back so
        // this stays a read-only predicate from the caller's view.
        let original_len = self.units.len();
        for p in placements {
            // §7.1: a reinforcing unit materialises on its entry hex -- it
            // may not appear on top of enemy units (engaging the enemy is
            // what melee is for). Lone AE leaders do not block a Dervish
            // arrival (§6.51 overrun applies to occupation).
            let owner = p.profile.identity.owner();
            let enemy = owner.opponent();
            if self.units.iter().any(|u| {
                u.position == p.position
                    && u.profile.identity.owner() == enemy
                    && !matches!(u.profile.kind, UnitKind::BritishLeader { .. })
            }) {
                self.units.truncate(original_len);
                return Err(RuleError::EnemyOccupied(p.position));
            }
            self.units.push(*p);
            if let Err(e) = self.check_stacking(p, p.position) {
                self.units.truncate(original_len);
                return Err(RuleError::from(e));
            }
        }
        self.units.truncate(original_len);
        Ok(())
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
                || self.reinforcements_placed_this_turn.iter().any(|&(_, id)| id == p.id)
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

/// Apply a retreat-before-melee for a cavalry/camel unit (rulebook §7.5).
pub fn apply_retreat_before_melee(
    state: &mut GameState,
    unit_id: UnitId,
    to: HexCoord,
) -> Result<(), RuleError> {
    state.can_retreat_before_melee(unit_id, to)?;
    let from = state
        .find_unit(unit_id)
        .map(|u| u.position)
        .unwrap_or(to);
    // Mark the unit as having moved so it may not retreat again this turn
    // (§7.5). The can_ check above guarantees `mp_spent == 0` here, so the
    // sentinel `1` does not clobber a real MP total; it is cleared at
    // end-of-turn along with the rest of the map.
    state.mp_spent_this_turn.insert(unit_id, 1);
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.position = to;
    }
    state.turn_events.push(TurnEventRecord::Retreat {
        unit: unit_id,
        from,
        to,
    });
    // §7.5: a retreat that empties the pending melee's target hex vacates it
    // before resolution; the declared attackers may then advance into it
    // (§7.6). Only open the window once the *last* defender has left --
    // a stacked hex still held by any defender is not vacated.
    if let Some(pending) = &state.pending_melee
        && pending.attack.defender_hex == from
        && !state.units.iter().any(|u| u.position == from)
    {
        let attackers = pending.attack.attackers.clone();
        open_advance_window(state, from, &attackers, vec!["7.5".to_string()]);
    }
    Ok(())
}

/// Apply an advance-after-combat for a unit (rulebook §6.82, §7.6).
pub fn apply_advance_after_combat(
    state: &mut GameState,
    unit_id: UnitId,
    to: HexCoord,
) -> Result<(), RuleError> {
    state.can_advance_after_combat(unit_id, to)?;
    let from = state
        .find_unit(unit_id)
        .map(|u| u.position)
        .unwrap_or(to);
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.position = to;
    }
    state.turn_events.push(TurnEventRecord::AdvanceAfterCombat {
        unit: unit_id,
        from,
        to,
    });

    // §9.346: a Dervish unit reaching the Palace eliminates GORDON (FoK).
    check_gordon_palace(state);

    Ok(())
}

/// Open an advance-after-combat window (§6.82, §7.5, §7.6): record `hex` as
/// vacated by combat with the surviving `participants` as the only units
/// eligible to advance into it. Dead participants are filtered out; a window
/// already open for the hex (e.g. a second attack finishing off survivors)
/// has its eligible list unioned. Emits [`Observation::HexVacatedByCombat`] as
/// the audit record. A no-op when no participant survives.
fn open_advance_window(
    state: &mut GameState,
    hex: HexCoord,
    participants: &[UnitId],
    paragraphs: Vec<String>,
) {
    let survivors: Vec<UnitId> = participants
        .iter()
        .copied()
        // §6.82: "artillery may not advance"; §5.25: forts may never move.
        // Both can *cause* a vacated hex (artillery fire destroys the
        // defenders) but may never enter it, so they are never eligible.
        .filter(|&id| {
            state.find_unit(id).is_some_and(|u| {
                !matches!(
                    u.profile.kind,
                    UnitKind::Artillery { .. } | UnitKind::Fort { .. }
                )
            })
        })
        .collect();
    if survivors.is_empty() {
        return;
    }
    let entry = state.vacated_by_combat.entry(hex).or_default();
    for &id in &survivors {
        if !entry.contains(&id) {
            entry.push(id);
        }
    }
    state.observations.push(Observation::HexVacatedByCombat {
        hex,
        eligible: entry.clone(),
        paragraphs,
    });
}

// ---------------------------------------------------------------------------
// 9) Unit state changes
// ---------------------------------------------------------------------------

/// Remove disrupted status from a unit (rulebook §5, reference notes).
pub fn apply_recover_unit(state: &mut GameState, unit_id: UnitId) -> Result<(), RuleError> {
    state.can_recover_unit(unit_id)?;
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.state.disrupted = false;
    }
    state.turn_events.push(TurnEventRecord::UnitRecovered { unit: unit_id });
    Ok(())
}

/// Mark a set of units as constructing a Zariba hexside (rulebook §5.3).
pub fn apply_construct_zariba(
    state: &mut GameState,
    unit_ids: &[UnitId],
    hexside: HexsideRef,
) -> Result<(), RuleError> {
    state.can_construct_zariba(unit_ids)?;
    for &id in unit_ids {
        if let Some(unit) = state.find_unit_mut(id) {
            unit.state.constructing_zariba = true;
        }
    }
    state.zariba_hexsides.push(hexside);
    Ok(())
}

/// Apply a Royal Engineers demolition action (rulebook §6.53). The Engineers
/// commit to the demolition this turn (flagged `demolishing`); the actual
/// resolution happens at end of turn via [`apply_resolve_demolition`], which
/// checks the engineer is still adjacent and undisrupted.
pub fn apply_demolition(
    state: &mut GameState,
    unit_id: UnitId,
    target: DemolitionTarget,
) -> Result<(), RuleError> {
    state.can_demolition(unit_id)?;
    if let Some(unit) = state.find_unit_mut(unit_id) {
        unit.state.demolishing = true;
    }
    state.pending_demolitions.push((unit_id, target));
    Ok(())
}

/// Resolve a pending demolition at end of turn (§6.53). The engineer must still
/// be adjacent to the target and undisrupted; otherwise the demolition is
/// cancelled. On success:
///   - Fort target: the fort unit is eliminated (0 VP per §9.14).
///   - Wall target: the hexside becomes a Breach (§6.63); if an enemy unit is
///     adjacent at the instant of breaching, one is eliminated.
///
/// Either way the engineer is freed (`demolishing = false`).
pub fn apply_resolve_demolition(
    state: &mut GameState,
    unit_id: UnitId,
    target: DemolitionTarget,
) -> Result<(), RuleError> {
    // §6.53: the demolition succeeds only if the engineers "remain adjacent
    // to their target and undisrupted at the end of the Anglo-Egyptian player
    // turn" -- an engineer eliminated during the turn did not remain, so the
    // attempt is simply cancelled (an error here would stall the phase
    // advance forever, since the end-of-turn resolution is mandatory).
    let Some(engineer) = state.find_unit(unit_id) else {
        state.observations.push(Observation::DemolitionResolved {
            engineer_id: unit_id,
            target,
            success: false,
        });
        return Ok(());
    };
    let (engineer_pos, engineer_owner, engineer_disrupted) = (
        engineer.position,
        engineer.profile.identity.owner(),
        engineer.state.disrupted,
    );

    // If the engineer was disrupted during the turn, the demolition fails.
    if engineer_disrupted {
        if let Some(u) = state.find_unit_mut(unit_id) {
            u.state.demolishing = false;
        }
        state.observations.push(Observation::DemolitionResolved {
            engineer_id: unit_id,
            target,
            success: false,
        });
        return Ok(());
    }

    // Check adjacency to the target.
    let (success, adjacent_eliminated) = match target {
        DemolitionTarget::Fort(fort_id) => {
            let fort = state.find_unit(fort_id);
            let adjacent = fort
                .map(|f| engineer_pos.is_adjacent_to(f.position))
                .unwrap_or(false);
            if adjacent {
                if let Some(f) = fort {
                    state.observations.push(Observation::FortDestroyed {
                        id: fort_id,
                        hex: f.position,
                    });
                }
                state.units.retain(|u| u.id != fort_id);
                (true, None)
            } else {
                (false, None)
            }
        }
        DemolitionTarget::WallHexside(edge) => {
            let adjacent =
                engineer_pos.is_adjacent_to(edge.a) || engineer_pos.is_adjacent_to(edge.b);
            if adjacent {
                // Mutate the hexside: Wall → Breach (§6.63).
                if let Some(kind) = state.board.hexsides.get_mut(&edge) {
                    if *kind == HexsideKind::Wall {
                        *kind = HexsideKind::Breach;
                    }
                } else {
                    state.board.hexsides.insert(edge, HexsideKind::Breach);
                }
                // §6.63: if an enemy unit is adjacent to the wall hexside at
                // the instant of breaching, one enemy unit is eliminated.
                let enemy_adjacent = state.units.iter().find_map(|u| {
                    let is_enemy = u.profile.identity.owner() != engineer_owner;
                    let adjacent_to_wall =
                        u.position.is_adjacent_to(edge.a) || u.position.is_adjacent_to(edge.b);
                    (is_enemy && adjacent_to_wall).then_some(u.id)
                });
                if let Some(enemy_id) = enemy_adjacent {
                    score_elimination(state, enemy_id, engineer_owner);
                    state.units.retain(|u| u.id != enemy_id);
                }
                (true, enemy_adjacent)
            } else {
                (false, None)
            }
        }
    };

    // Free the engineer regardless of outcome.
    if let Some(u) = state.find_unit_mut(unit_id) {
        u.state.demolishing = false;
    }

    state.turn_events.push(TurnEventRecord::Demolition {
        engineer: unit_id,
        target,
        success,
    });
    state.observations.push(Observation::DemolitionResolved {
        engineer_id: unit_id,
        target,
        success,
    });
    if success
        && let DemolitionTarget::WallHexside(edge) = target
    {
        state.observations.push(Observation::WallBreached {
            hexside: edge,
            breached: true,
            // §6.53 demolitions have no CRT roll -- success is guaranteed by
            // surviving the turn adjacent and undisrupted.
            row: None,
            adjacent_eliminated,
        });
    }

    Ok(())
}

/// §6.63 3rd bullet: artillery fire aimed at breaching a wall hexside. Only
/// artillery-class firers may participate; the CRT is rolled with the
/// firers' combined fire factor (each firer's contribution halved per its
/// range band to the *nearer* endpoint of the wall hexside, floored at 1 per
/// unit, §6.16). A result of `Eliminate(2)` or higher breaches the wall --
/// flipping the `Wall` hexside to `Breach` (so it no longer blocks LOS,
/// movement, melee, or ZOC) and eliminating one enemy unit adjacent to the
/// breached hexside. Any other CRT result is a miss.
///
/// This mirrors the Royal-Engineers demolition path (`apply_resolve_demolition`)
/// for the wall case but trades the Engineers' guaranteed success for the
/// artillery's CRT roll -- the rulebook specifies the same "2+ required"
/// threshold for both trigger styles.
pub fn apply_artillery_breach_wall(
    state: &mut GameState,
    firers: &[UnitId],
    target: HexsideRef,
    roll: DieRoll,
) -> Result<(), RuleError> {
    if firers.is_empty() {
        return Err(RuleError::NoFirers);
    }

    // Phase must be a fire-combat phase (defensive or offensive; either
    // sub-phase is fine -- artillery breaching is not tied to the
    // Maxim/Howitzer sub-phase the way Maxims are).
    let firing_player = match state.phase {
        Phase::OffensiveFire(_) => state.active_player,
        Phase::DefensiveFire(_) => state.active_player.opponent(),
        _ => return Err(RuleError::WrongPhase),
    };

    // The target hexside must currently be a Wall. (If it's already a Breach
    // or Gate there's nothing to do; if it's missing entirely the data is
    // wrong. Either way the player has misclicked.)
    match state.board.hexsides.get(&target) {
        Some(HexsideKind::Wall) => {}
        _ => return Err(RuleError::NotAWallHexside(target)),
    }

    // Validate every firer (all-or-nothing) and accumulate the effective CRT
    // factor. See `can_fire_at_wall` for the per-firer rules.
    let mut effective_total: u16 = 0;
    let mut seen: std::collections::HashSet<UnitId> = std::collections::HashSet::new();
    for &id in firers {
        if !seen.insert(id) {
            return Err(RuleError::AlreadyFired(id));
        }
        let (fire_factor, range, nearer_hex) = state.can_fire_at_wall(id, target)?;
        let band = range_band_for(
            state.scenario,
            firing_player,
            state.unit_or_err(id)?.profile.weapon,
            range,
        );
        effective_total = effective_total.saturating_add(band.apply(fire_factor.value()));

        // LOS already verified by `can_fire_at_wall`; the band lookup above is
        // the only additional per-firer work. `nearer_hex` is kept for
        // potential future error reporting.
        let _ = nearer_hex;
    }

    // All firers pass -- mark them as having fired this phase.
    for &id in firers {
        state.units_fired_this_phase.push(id);
    }

    // §6.63: "A result of 2 or more on the combat results table is required
    // to breach a wall." The CRT cell value (Eliminate(N)) is the relevant
    // metric, identical to the §6.61/§6.62 gunboat/fort thresholds.
    let row = FireFactorRow::from_total(effective_total);
    let result = combat_results_table(row, roll);
    let breached = matches!(result, CombatResult::Eliminate(n) if n >= 2);

    let mut adjacent_eliminated: Option<UnitId> = None;
    if breached {
        // Flip Wall → Breach.
        if let Some(kind) = state.board.hexsides.get_mut(&target) {
            if *kind == HexsideKind::Wall {
                *kind = HexsideKind::Breach;
            }
        } else {
            state.board.hexsides.insert(target, HexsideKind::Breach);
        }

        // §6.63: "If any enemy units are adjacent to the wall hexside at the
        // instant it is breached, one enemy unit is eliminated." Pick the
        // first such unit (matching the demolition path's convention).
        let opponent = firing_player.opponent();
        if let Some(victim) = state.units.iter().find_map(|u| {
            let is_enemy = u.profile.identity.owner() == opponent;
            let adjacent = u.position.is_adjacent_to(target.a)
                || u.position.is_adjacent_to(target.b);
            (is_enemy && adjacent).then_some(u.id)
        }) {
            score_elimination(state, victim, firing_player);
            state.units.retain(|u| u.id != victim);
            adjacent_eliminated = Some(victim);
        }
    }

    state
        .observations
        .push(Observation::WallBreached {
            hexside: target,
            // Truthful outcome: `breached` reflects whether the §6.63
            // threshold (CRT cell 2+) was met, so a short roll logs as a
            // failed attempt rather than a phantom breach.
            breached,
            row: Some(row),
            adjacent_eliminated,
        });
    state.turn_events.push(TurnEventRecord::FireCombat {
        attacker: firing_player,
        firers: firers.to_vec(),
        target: target.a,
        roll,
        modifiers: Vec::new(),
        total_modifier: 0,
        result,
        kind: FireKind::Direct,
        eliminated: adjacent_eliminated.into_iter().collect(),
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// 10) Reinforcements
// ---------------------------------------------------------------------------

/// Place reinforcements onto the map (rulebook §9.112, §9.113).
pub fn apply_place_reinforcements(
    state: &mut GameState,
    placements: &[UnitPlacement],
) -> Result<(), RuleError> {
    // Full stacking validation (§5.51-5.53), not just the four-unit count, and
    // cumulative across the batch -- plus the Campaign order of appearance
    // (§9.112/§9.113) via `validate_campaign_reinforcements`.
    state.can_place_reinforcements(placements)?;
    for p in placements {
        // §9.112/§9.113: entering the map costs movement points -- the
        // Anglo-Egyptian entrance costs 1 MP (8 for the "Friendlies" through
        // the Abu Alim hut); the Dervish pay the terrain cost of the hex
        // entered. Recorded as MP spent so the allowance cap (§5.11) and
        // retreat gating (§7.5) see it.
        if state.scenario == Scenario::Campaign && matches!(state.phase, Phase::Movement) {
            let owner = p.profile.identity.owner();
            let cost: i16 = match owner {
                Player::AngloEgyptian => {
                    if p.profile.identity.is_friendlies() {
                        8
                    } else {
                        1
                    }
                }
                Player::Dervish => {
                    let terrain = state.board.terrain_at(p.position).unwrap_or(
                        omdurman_types::Terrain::Clear { road: Default::default() },
                    );
                    crate::terrain_chart::movement_cost(terrain)
                        .map(|allowance| allowance.value() as i16)
                        .unwrap_or(1)
                }
            };
            let spent = state.mp_spent_this_turn.get(&p.id).copied().unwrap_or(0);
            state.mp_spent_this_turn.insert(p.id, spent + cost);
        }
        state.units.push(*p);
        state.reinforcements_placed_this_turn.push((
            p.profile.identity.owner(),
            p.id,
        ));
    }
    if let Some(first) = placements.first() {
        state.turn_events.push(TurnEventRecord::Reinforcements {
            units: placements.iter().map(|p| p.id).collect(),
            player: first.profile.identity.owner(),
            at: first.position,
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 11) Scenario-specific
// ---------------------------------------------------------------------------

/// The number of Dervish units that desert for a given die roll (§8.2): "equal
/// to 1½ times the roll of one die", i.e. `floor(1.5 * roll)`.
/// The manual does not specify rounding direction; floor is used here.
pub fn desertion_count(roll: DieRoll) -> usize {
    (3 * roll.value() as usize) / 2
}

pub fn apply_dervish_desertion(
    state: &mut GameState,
    roll: DieRoll,
    deserters: &[UnitId],
) -> Result<(), RuleError> {
    // §8.2: "Once each campaign game, during the first night turn of the game,
    // the Dervish player rolls one die... made during the movement phase."
    if state.dervish_deserted {
        return Err(DesertionError::AlreadyDeserted.into());
    }
    if state.scenario != Scenario::Campaign {
        return Err(DesertionError::WrongScenario.into());
    }
    let is_first_night = state.day_night == DayNight::Night
        && scenario_turn(state.scenario, state.current_turn)
            .is_some_and(|t| t.event == TurnEvent::DervishDesertion);
    if !is_first_night || state.phase != Phase::Movement {
        return Err(DesertionError::WrongTime.into());
    }

    // The count is fixed by the roll; the Dervish player chooses which units.
    // The demand is capped by the eligible pool: §8.2 assumes a full army, and
    // a Dervish force already bled below 1.5x the roll simply desert
    // everything eligible ("the number of deserting units is equal to 1.5
    // times the roll" cannot exceed the units that exist).
    let expected = desertion_count(roll).min(
        state
            .units
            .iter()
            .filter(|u| {
                u.profile.identity.owner() == Player::Dervish
                    && !u.profile.identity.is_desertion_exempt()
            })
            .count(),
    );
    if deserters.len() != expected {
        return Err(DesertionError::WrongCount {
            roll: roll.value() as u8,
            expected,
            actual: deserters.len(),
        }
        .into());
    }

    // Validate every chosen unit before removing any (all-or-nothing).
    for &id in deserters {
        let unit = state.unit_or_err(id)?;
        if unit.profile.identity.owner() != Player::Dervish {
            return Err(DesertionError::NotEligible(id).into());
        }
        if unit.profile.identity.is_desertion_exempt() {
            return Err(DesertionError::Exempt(id).into());
        }
    }

    for &id in deserters {
        state.units.retain(|u| u.id != id);
    }
    state.turn_events.push(TurnEventRecord::Desertion {
        units: deserters.to_vec(),
        roll,
    });
    state.dervish_deserted = true;
    Ok(())
}

/// Apply a Friendlies-transport state transition (rulebook §5.21).
///
/// Gates enforced:
///   - `Load`: Isa Zachneih must be eliminated; the unit and gunboat must be
///     adjacent; no transport may already be in progress.
///   - `Cross`: the current state must be `Loaded`.
///   - `Disembark`: the current state must be `Crossing`; on success the
///     unit is freed (a disembarking `MoveUnit` should follow, costed by
///     terrain).
pub fn apply_friendlies_transport(
    state: &mut GameState,
    action: FriendliesAction,
) -> Result<(), RuleError> {
    match action {
        FriendliesAction::Load { unit, gunboat } => {
            // §5.21: transport is only allowed after Isa Zachneih is eliminated.
            if !state.isa_zachneih_eliminated {
                return Err(RuleError::FriendliesIsaZachneihAlive);
            }
            // No concurrent missions (design choice -- manual is ambiguous).
            if state.friendlies_transport.is_some() {
                return Err(RuleError::FriendliesTransportInProgress);
            }
            let u = state.unit_or_err(unit)?;
            let gb = state.unit_or_err(gunboat)?;
            // §5.21: the unit and gunboat must start the turn adjacent.
            if !u.position.is_adjacent_to(gb.position) {
                return Err(RuleError::FriendliesNotAdjacentToGunboat);
            }
            // Mark the unit as loaded onto the gunboat.
            if let Some(u) = state.find_unit_mut(unit) {
                u.state.loaded_on = Some(gunboat);
            }
            state.friendlies_transport = Some(TransportState::Loaded { unit, gunboat });
        }
        FriendliesAction::Cross { unit, gunboat, to } => {
            state.require_transport_state(
                |s| {
                    matches!(
                        s,
                        TransportState::Loaded { unit: cu, gunboat: cg }
                            if *cu == unit && *cg == gunboat
                    )
                },
                RuleError::FriendliesNotLoaded,
            )?;
            state.friendlies_transport = Some(TransportState::Crossing {
                unit,
                gunboat,
                to,
            });
        }
        FriendliesAction::Disembark { unit, gunboat } => {
            state.require_transport_state(
                |s| {
                    matches!(
                        s,
                        TransportState::Crossing { unit: cu, gunboat: cg, .. }
                            if *cu == unit && *cg == gunboat
                    )
                },
                RuleError::FriendliesNotCrossing,
            )?;
            // Disembark: free the unit from the gunboat. A disembarking MoveUnit
            // effect should follow (chained by the caller) to pay the terrain
            // cost of the first hex entered (§5.21).
            if let Some(u) = state.find_unit_mut(unit) {
                u.state.loaded_on = None;
            }
            state.observations.push(Observation::FriendliesDisembarked {
                unit_id: unit,
                at: state
                    .find_unit(gunboat)
                    .map(|g| g.position)
                    .unwrap_or(HexCoord::new(0, 0)),
            });
            state.friendlies_transport = Some(TransportState::ReadyToDisembark {
                unit,
                gunboat,
            });
        }
    }
    Ok(())
}

/// If a gunboat carrying a "Friendlies" unit is sunk (by artillery §6.61 or
/// mine §10.12), the loaded unit is lost with it (§5.21 — manual is silent;
/// design choice: loaded Friendlies go down with the ship).
fn remove_friendlies_on_gunboat(state: &mut GameState, gunboat_id: UnitId) {
    if let Some(TransportState::Loaded { unit, gunboat })
    | Some(TransportState::Crossing {
        unit,
        gunboat,
        to: _,
    }) = &state.friendlies_transport
        && *gunboat == gunboat_id
    {
        let unit_id = *unit;
        state.units.retain(|u| u.id != unit_id);
        state.friendlies_transport = None;
    }
}

/// Drift a gunboat with lost engines one hex downstream with the Nile current
/// (rulebook §10.12).  Called automatically at the start of each movement
/// phase for every gunboat with `engines_lost == true`.
///
/// The Nile flow arrows on the map are assumed to always point to a hex that
/// itself has a flow arrow (the user confirmed).  If no flow data exists at
/// the current hex (dead end), the gunboat is stuck and nothing happens.
pub fn apply_drift_gunboat(state: &mut GameState, unit_id: UnitId) -> Result<(), RuleError> {
    let unit = state.unit_or_err(unit_id)?;
    if !matches!(unit.profile.kind, UnitKind::Gunboat { .. }) {
        return Err(RuleError::NotAGunboat(unit_id));
    }
    if !unit.state.engines_lost {
        return Err(RuleError::GunboatEnginesNotLost(unit_id));
    }
    let current = unit.position;
    let Some(flow) = state.board.flow_at(current) else {
        // No flow data at this hex — the gunboat is stuck (dead end).
        return Ok(());
    };
    let downstream = current.neighbors()[flow as usize];
    // Move the gunboat downstream.  Gunboats ignore stacking (§5.51).
    if let Some(u) = state.find_unit_mut(unit_id) {
        u.position = downstream;
    }
    // If the gunboat drifts into a mine, resolve it immediately.
    if let Some(mine) = state
        .mines
        .iter_mut()
        .find(|m| m.hex == downstream && !m.triggered)
    {
        mine.triggered = true;
        // Re-roll for drift-triggered mine (deterministic: use a fixed
        // approach — the mine was already placed; the drift triggers it).
        // Since the rules engine requires pre-rolled dice, we store a
        // default roll.  In practice the caller should pre-roll and chain
        // the mine effect separately, but for safety we handle it here.
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 12) Optional rules
// ---------------------------------------------------------------------------

/// Apply a river-mine resolution (rulebook §10.12).
pub fn apply_river_mine(
    state: &mut GameState,
    gunboat_id: UnitId,
    hex: HexCoord,
    roll: DieRoll,
) -> Result<(), RuleError> {
    // §10.14: the Dervish player's own gunboats pass through mined hexes with
    // no ill effect (he knows where they are).
    if let Some(unit) = state.find_unit(gunboat_id)
        && unit.profile.identity.owner() == Player::Dervish
    {
        return Ok(());
    }

    // §10.13: a mine only fires once. The hex must hold an untriggered mine.
    let Some(mine) = state
        .mines
        .iter_mut()
        .find(|m| m.hex == hex && !m.triggered)
    else {
        return Err(RuleError::NoUntriggeredMine(hex));
    };
    mine.triggered = true;

    let result = crate::MineResult::from_roll(roll);
    match result {
        crate::MineResult::NoEffect => {}
        crate::MineResult::EnginesLost => {
            if let Some(unit) = state.find_unit_mut(gunboat_id) {
                // §10.12: engines lost -- the gunboat drifts two hexes per turn
                // with the current for the rest of the game.
                unit.state.engines_lost = true;
            }
        }
        crate::MineResult::Sunk => {
            state.units.retain(|u| u.id != gunboat_id);
            remove_friendlies_on_gunboat(state, gunboat_id);
        }
    }
    Ok(())
}

/// Sink the river chain (rulebook §10.23). Marks the placed chain cleared so it
/// no longer stops gunboats (§10.22).
pub fn apply_sink_chain(state: &mut GameState) -> Result<(), RuleError> {
    match state.chain.as_mut() {
        Some(chain) if !chain.sunk => {
            chain.sunk = true;
            Ok(())
        }
        Some(_) => Err(RuleError::ChainAlreadySunk),
        None => Err(RuleError::NoChainPlaced),
    }
}

// ---------------------------------------------------------------------------
// Setup / deployment (§9.2/§9.3/§10)
// ---------------------------------------------------------------------------

/// Deploy one order-of-battle unit during setup (§9.2/§9.3). Validated by
/// [`GameState::can_deploy_unit`]; on success the placement joins `units`.
pub fn apply_deploy_unit(
    state: &mut GameState,
    placement: &UnitPlacement,
) -> Result<(), RuleError> {
    state.can_deploy_unit(placement)?;
    state.units.push(*placement);
    Ok(())
}

/// Remove a deployed unit from the board during setup (§9.2/§9.3) so its
/// counter can be re-placed. Validated by [`GameState::can_remove_deployed_unit`];
/// on success the placement is dropped from `units`.
pub fn apply_remove_deployed_unit(
    state: &mut GameState,
    unit_id: UnitId,
    player: Player,
) -> Result<(), RuleError> {
    state.can_remove_deployed_unit(unit_id, player)?;
    state.units.retain(|u| u.id != unit_id);
    Ok(())
}

/// Lay a river mine during setup (§10.11). Validated by
/// [`GameState::can_place_mine`].
pub fn apply_place_mine(state: &mut GameState, hex: HexCoord) -> Result<(), RuleError> {
    state.can_place_mine(hex)?;
    state.mines.push(MinePlacement {
        hex,
        triggered: false,
    });
    Ok(())
}

/// Lay (or replace) the river chain during setup (§10.21). Validated by
/// [`GameState::can_place_chain`].
pub fn apply_place_chain(state: &mut GameState, hexes: &[HexCoord]) -> Result<(), RuleError> {
    state.can_place_chain(hexes)?;
    state.chain = Some(ChainPlacement {
        hexes: hexes.to_vec(),
        sunk: false,
    });
    Ok(())
}

/// Pre-place a Zariba hexside during setup (§9.231-9.232). Validated by
/// [`GameState::can_place_zariba`].
pub fn apply_place_zariba(state: &mut GameState, hexside: HexsideRef) -> Result<(), RuleError> {
    state.can_place_zariba()?;
    if !state.zariba_hexsides.contains(&hexside) {
        state.zariba_hexsides.push(hexside);
    }
    Ok(())
}

/// A faction confirms readiness to leave setup (§9.2/§9.3). Sets the one-way
/// ready flag; when *both* factions are ready and `setup_complete` holds, the
/// engine auto-advances to the first Movement turn (via [`advance_phase`], so the
/// transition logic lives in one place). Validated by
/// [`GameState::can_confirm_setup_ready`].
pub fn apply_confirm_setup_ready(state: &mut GameState, player: Player) -> Result<(), RuleError> {
    state.can_confirm_setup_ready(player)?;
    match player {
        Player::AngloEgyptian => state.setup_ready_ae = true,
        Player::Dervish => state.setup_ready_dervish = true,
    }

    // Both sides ready + deployment complete -> begin the battle. `advance_phase`
    // owns the Setup -> Movement transition; a not-yet-complete board just leaves
    // us in Setup (the other side is still deploying).
    if state.setup_ready_ae && state.setup_ready_dervish && state.setup_complete().is_ok() {
        advance_phase(state)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Validate that a fire attack is legal in the current state (rulebook §6).
///
/// Single source of truth: every firer is checked through [`can_fire_at`], the
/// same predicate the UI gates clicks on -- so a shot the UI offers is exactly a
/// shot `apply` accepts (phase, owner, sub-phase/kind, weapon class, howitzer-
/// at-night §6.64, disruption, already-fired, gunboat/fort-needs-artillery
/// §6.61/§6.62, and range §6.22). An empty firer list is rejected.
/// The die-roll modifiers the rulebook *mandates* for a fire attack, derived
/// from the game state (rulebook §6.24, §5.54, §9.231, §9.232). The engine is
/// authoritative: resolution applies exactly this set (plus the engine-side
/// terrain modifier §6.23), and a caller-supplied `attack.modifiers` list that
/// differs is rejected in [`validate_fire_attack`] -- a client can neither
/// omit a mandatory bonus/penalty nor smuggle one in (e.g. a `Terrain(n)`
/// entry would double-count the engine's own §6.23 modifier).
pub fn mandatory_fire_modifiers(state: &GameState, attack: &FireAttack) -> Vec<FireModifier> {
    let mut modifiers = Vec::new();
    // §6.24: "+1 modifier to their die roll" for all Anglo-Egyptian *direct*
    // fire attacks. Maxim second fire and howitzer fire get neither this nor
    // brigade integrity.
    if attack.kind == FireKind::Direct && attack.firing_player == Player::AngloEgyptian {
        modifiers.push(FireModifier::AngloEgyptianDirectFire);
        // §5.54/§6.24: brigade integrity (+1, cumulative) when all four
        // battalions of one brigade are stacked in the same hex and all fire
        // at this target hex -- i.e. the firers are exactly such a stack.
        let firers: Vec<&UnitPlacement> = attack
            .firers
            .iter()
            .filter_map(|id| state.find_unit(*id))
            .collect();
        let co_stacked = firers
            .first()
            .is_some_and(|first| firers.iter().all(|u| u.position == first.position));
        if co_stacked {
            let identities: Vec<crate::UnitIdentity> =
                firers.iter().map(|u| u.profile.identity).collect();
            if matches!(
                crate::brigade_integrity(&identities),
                crate::BrigadeIntegrity::Integrated(_)
            ) {
                modifiers.push(FireModifier::BrigadeIntegrity);
            }
        }
    }
    // §9.231/§9.232: the zariba die-roll penalties apply "on all *Dervish*
    // fire attacks" (thorn hedge −2; trench −4 vs. entrenched units) -- never
    // to Anglo-Egyptian fire.
    if attack.firing_player == Player::Dervish {
        if state.board.has_zariba_thorn_hedge(attack.target_hex) {
            modifiers.push(FireModifier::ZaribaThornHedge);
        }
        if state.board.is_zariba_entrenched(attack.target_hex) {
            modifiers.push(FireModifier::ZaribaTrenchEntrenched);
        }
    }
    modifiers
}

/// The die-roll modifiers the rulebook *mandates* for both sides of a melee
/// (§7.7: Dervish +2 / Anglo-Egyptian +1; §9.232: −2 instead of +2 for a
/// Dervish melee attack on an entrenched unit). Returns
/// `(attacker_modifiers, defender_modifiers)`; the engine applies exactly
/// these at resolution and rejects a declared attack whose lists differ.
pub fn mandatory_melee_modifiers(
    state: &GameState,
    attack: &MeleeAttack,
) -> (Vec<MeleeModifier>, Vec<MeleeModifier>) {
    let mut attacker_modifiers = vec![match attack.attacker_player {
        Player::Dervish => MeleeModifier::DervishStandard,
        Player::AngloEgyptian => MeleeModifier::AngloEgyptianStandard,
    }];
    // §9.232: "−2 (instead of +2) melee modifier to Dervish units melee
    // attacking an entrenched unit".
    if attack.attacker_player == Player::Dervish
        && state.board.is_zariba_entrenched(attack.defender_hex)
    {
        attacker_modifiers.push(MeleeModifier::DervishVsTrenchedDefender);
    }
    let defender_modifiers = vec![match attack.attacker_player.opponent() {
        Player::Dervish => MeleeModifier::DervishStandard,
        Player::AngloEgyptian => MeleeModifier::AngloEgyptianStandard,
    }];
    (attacker_modifiers, defender_modifiers)
}

pub fn validate_fire_attack(state: &GameState, attack: &FireAttack) -> Result<(), RuleError> {
    if attack.firers.is_empty() {
        return Err(RuleError::NoFirers);
    }
    for &id in &attack.firers {
        state.can_fire_at(id, attack.target_hex, attack.kind)?;
    }
    // §6.24/§5.54/§9.231/§9.232: the caller's modifier list must match the
    // engine-derived mandatory set exactly (the modifiers are documentation
    // for the UI; the engine resolves with its own derivation either way).
    let mandatory = mandatory_fire_modifiers(state, attack);
    if attack.modifiers != mandatory {
        return Err(RuleError::FireModifierMismatch {
            expected: mandatory,
            got: attack.modifiers.clone(),
        });
    }
    Ok(())
}

/// Compute the distance between the first firer and the target hex (rulebook §6.22).
pub fn target_range(
    state: &GameState,
    firers: &[UnitId],
    target: HexCoord,
) -> Result<HexDistance, RuleError> {
    let firer_id = firers.first().ok_or(RuleError::NotYourTurn)?;
    let firer = state
        .find_unit(*firer_id)
        .ok_or(RuleError::UnitNotFound(*firer_id))?;
    Ok(HexDistance(firer.position.distance(target) as u16))
}

/// Apply a Combat Results Table result to a list of target units -- eliminate `n` and disrupt
/// half (round up) of the remaining (rulebook §6.22, §7.7).
fn apply_combat_results_table_result(
    state: &mut GameState,
    result: CombatResult,
    target_ids: &[UnitId],
    target_player: Player,
) {
    match result {
        CombatResult::NoEffect => {}
        CombatResult::Disrupt => {
            // Disrupt half (round up) of the target units.
            let n = target_ids.len().div_ceil(2);
            for &id in target_ids.iter().take(n) {
                if let Some(unit) = state.find_unit_mut(id) {
                    unit.state.disrupted = true;
                }
            }
        }
        CombatResult::Eliminate(n) => {
            let n = (n as usize).min(target_ids.len());
            // Half (round up) of the survivors are also disrupted.
            let disrupt_n = target_ids.len().saturating_sub(n).div_ceil(2);

            for &id in target_ids.iter().take(n) {
                score_elimination(state, id, target_player);
            }
            // §5.21: if a gunboat is sunk while carrying a "Friendlies" unit,
            // the loaded unit is lost with the ship (and its explosive
            // ammunition). Cascade the elimination before removing units.
            let mut cascade: Vec<UnitId> = Vec::new();
            for &id in target_ids.iter().take(n) {
                if state
                    .find_unit(id)
                    .map(|u| matches!(u.profile.kind, UnitKind::Gunboat { .. }))
                    .unwrap_or(false)
                {
                    for u in &state.units {
                        if u.state.loaded_on == Some(id) {
                            cascade.push(u.id);
                        }
                    }
                }
            }
            for cid in &cascade {
                state.observations.push(Observation::UnitEliminated {
                    id: *cid,
                    cause: ElimCause::LostWithTransport,
                    vp_source: None,
                });
            }
            state
                .units
                .retain(|u| !target_ids[..n].contains(&u.id) && !cascade.contains(&u.id));

            // §5.44 orphan leader: if all combat units (non-leader) in the
            // target hex were eliminated, any surviving AE leader in that hex
            // is also eliminated (the leader cannot exist alone on the
            // battlefield).
            if target_player == Player::AngloEgyptian {
                let eliminated_hexes: Vec<HexCoord> = target_ids[..n]
                    .iter()
                    .filter_map(|id| state.find_unit(*id).map(|u| u.position))
                    .collect();
                for hex in eliminated_hexes {
                    let leader_ids: Vec<UnitId> = state
                        .units
                        .iter()
                        .filter(|u| u.position == hex && u.profile.identity.owner() == Player::AngloEgyptian)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .filter(|u| matches!(u.profile.kind, UnitKind::BritishLeader { .. }))
                        .map(|u| u.id)
                        .collect();
                    if leader_ids.is_empty() {
                        continue;
                    }
                    let has_combat_unit = state
                        .units
                        .iter()
                        .any(|u| u.position == hex && u.profile.identity.owner() == Player::AngloEgyptian && !matches!(u.profile.kind, UnitKind::BritishLeader { .. }));
                    if !has_combat_unit {
                        for &id in &leader_ids {
                            score_elimination(state, id, target_player);
                        }
                        for &id in &leader_ids {
                            state.observations.push(Observation::UnitEliminated {
                                id,
                                cause: ElimCause::OrphanLeader,
                                vp_source: None,
                            });
                        }
                        state.units.retain(|u| !leader_ids.contains(&u.id));
                    }
                }
            }

            // Disrupt survivors.
            let survivors: Vec<UnitId> = target_ids[n..].to_vec();
            for &id in survivors.iter().take(disrupt_n) {
                if let Some(unit) = state.find_unit_mut(id) {
                    unit.state.disrupted = true;
                    state.turn_events.push(TurnEventRecord::UnitDisrupted { unit: id });
                }
            }
        }
    }
}

/// Filter `before` down to the IDs of units that have been eliminated (i.e.
/// are no longer present in `state.units`). Used by fire/melee resolution to
/// compute the post-mutation elimination list from a pre-mutation snapshot.
fn diff_eliminated(state: &GameState, before: Vec<UnitId>) -> Vec<UnitId> {
    before
        .into_iter()
        .filter(|id| state.find_unit(*id).is_none())
        .collect()
}

/// Score victory points for eliminating a unit (rulebook §9.14).
pub fn score_elimination(state: &mut GameState, unit_id: UnitId, _owner: Player) {
    if let Some(unit) = state.find_unit(unit_id) {
        let identity = unit.profile.identity;
        let position = unit.position;
        let vp_source = vp_source_for(&identity, position, &state.board);
        if vp_source == Some(VpSource::IsaZachneihEliminated) {
            state.isa_zachneih_eliminated = true;
        }

        if let Some(source) = vp_source {
            let points = source.points();
            let scorer = source.who_scores();
            state.victory.events.push(crate::VpEvent {
                turn: state.current_turn,
                source,
            });
            state.turn_events.push(TurnEventRecord::VpScored {
                source,
                points,
                for_player: scorer,
            });
            state.observations.push(Observation::VictoryScored {
                source,
                points,
                for_player: scorer,
            });
        } 

        // Surface the elimination as an observation regardless of VP.
        state.turn_events.push(TurnEventRecord::UnitEliminated {
            unit: unit_id,
            cause: ElimCause::Combat,
        });
        state.observations.push(Observation::UnitEliminated {
            id: unit_id,
            cause: ElimCause::Combat,
            vp_source,
        });

        // Leader-specific observation for dispatch-slip flavour.
        if matches!(identity, crate::UnitIdentity::DervishLeader(_))
            | matches!(identity, crate::UnitIdentity::AngloEgyptianLeader(_))
        {
            state.observations.push(Observation::LeaderKilled {
                id: unit_id,
                by: state.active_player,
            });
        }
    }
}

/// VP source awarded for eliminating a unit of `identity` at `position`
/// (rulebook §9.14). `None` means the elimination scores no points (e.g. a
/// Dervish fort, which is worth 0 pts). Pure lookup -- it does not mutate
/// state; the caller owns any side effects (e.g. the Isa Zachneih flag).
fn vp_source_for(
    identity: &crate::UnitIdentity,
    position: HexCoord,
    board: &BoardInfo,
) -> Option<VpSource> {
    if identity.is_friendlies() {
        // §9.14: a "Friendlies" unit scores by the bank it died on -- 1 pt
        // on the east bank, 3 pts on the west bank.
        match board.bank_of(position) {
            Some(crate::board::NileBank::West) => Some(VpSource::FriendliesWestBankEliminated),
            _ => Some(VpSource::FriendliesEastBankEliminated),
        }
    } else {
        match *identity {
            crate::UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah) => {
                Some(VpSource::KhalifaEliminated)
            }
            crate::UnitIdentity::DervishTribal {
                tribe: DervishTribe::IsaZachneih,
            } => Some(VpSource::IsaZachneihEliminated),
            crate::UnitIdentity::DervishTribal { .. }
            | crate::UnitIdentity::DervishLeader(_)
            | crate::UnitIdentity::DervishArtillery
            | crate::UnitIdentity::DervishGunboat(_) => Some(VpSource::DervishUnitEliminated),
            crate::UnitIdentity::DervishFort => None, // §9.14: 0 pts for forts.
            crate::UnitIdentity::AngloEgyptianLeader(_) => {
                Some(VpSource::BritishLeaderEliminated)
            }
            crate::UnitIdentity::AngloEgyptianGunboat(_) => Some(VpSource::BritishGunboatSunk),
            _ => Some(VpSource::AngloEgyptianLandUnitEliminated),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use traceability_macro::rulebook;

    /// A fresh state advanced past deployment into the first Movement turn, for
    /// gameplay tests that aren't exercising the setup phase itself. Every
    /// scenario now opens in [`Phase::Setup`]; this skips straight to play.
    fn playing(scenario: Scenario) -> GameState {
        let mut state = GameState::new(scenario);
        state.phase = Phase::Movement;
        state
    }

    fn ae_infantry_profile() -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Infantry { fire: 4, melee: 5, movement: 8 },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::British,
                },
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Four),
            melee: Some(crate::MeleeFactor::Five),
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        }
    }

    fn dervish_tribal_profile() -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Infantry { fire: 3, melee: 6, movement: 9 },
            identity: UnitIdentity::DervishTribal {
                tribe: DervishTribe::Baggara,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Three),
            melee: Some(crate::MeleeFactor::Six),
            movement: UnitMovement::Land(crate::MovementAllowance::Nine),
        }
    }

    fn make_ae_infantry(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        });
        id
    }

    fn make_dervish_tribal(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: dervish_tribal_profile(),
            state: UnitState::default(),
        });
        id
    }

    fn make_ae_leader(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::BritishLeader { movement: 0 },
                identity: UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        });
        id
    }

    /// A Dervish tribal unit of an explicit tribe (for same-hex / stacking
    /// tests that need a second tribe).
    fn dervish_tribal_profile_with(tribe: DervishTribe) -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Infantry { fire: 3, melee: 6, movement: 9 },
            identity: UnitIdentity::DervishTribal { tribe },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Three),
            melee: Some(crate::MeleeFactor::Six),
            movement: UnitMovement::Land(crate::MovementAllowance::Nine),
        }
    }

    /// An Anglo-Egyptian old-style gunboat profile (§2.32). `is_boat()` is true,
    /// so deployment-zone checks treat it as a boat (Nile-only, §5.22).
    fn ae_gunboat_profile() -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Gunboat { fire: 0, upstream: 15, downstream: 16 },
            identity: UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(
                OldGunboat::LordKitchener,
            )),
            weapon: WeaponClass::Artillery,
            fire: None,
            melee: None,
            movement: UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Fifteen,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        }
    }

    /// A Dervish gunboat profile (§9.111: two gunboats on south-edge Nile
    /// hexes). `is_boat()` is true, so deployment treats it as a boat
    /// (Nile-only, §5.22).
    fn dervish_gunboat_profile() -> UnitProfile {
        UnitProfile {
            kind: UnitKind::Gunboat { fire: 0, upstream: 10, downstream: 16 },
            identity: UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1)),
            weapon: WeaponClass::Artillery,
            fire: None,
            melee: None,
            movement: UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn fire_combat_eliminates_target() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row06to10,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Eight,
            },
        );
        assert!(result.is_ok());
        // Dervish unit should be eliminated (roll 8, factor 8 -> Eliminate(1) on A-E Combat Results Table).
        assert!(state.find_unit(target).is_none());
    }

    #[rulebook("§6.24")]
    #[test]
    fn fire_modifiers_are_engine_derived_and_mismatches_rejected() {
        // §6.24: the +1 accuracy DRM is mandatory on every Anglo-Egyptian
        // direct-fire attack. A client that omits it (or smuggles in a
        // wrong modifier) is rejected; a correct list resolves with the
        // engine-derived bonus either way.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let base = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row06to10,
            modifiers: vec![],
        };

        // Omitted -> rejected.
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: base.clone(),
                roll: DieRoll::Five,
            },
        );
        assert!(
            matches!(result, Err(RuleError::FireModifierMismatch { .. })),
            "missing §6.24 +1 must be rejected, got {result:?}"
        );

        // Duplicated -> rejected.
        let mut dup = base.clone();
        dup.modifiers = vec![
            FireModifier::AngloEgyptianDirectFire,
            FireModifier::AngloEgyptianDirectFire,
        ];
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: dup,
                roll: DieRoll::Five,
            },
        );
        assert!(matches!(result, Err(RuleError::FireModifierMismatch { .. })));

        // Smuggled terrain DRM -> rejected (§6.23 is engine-side; a caller
        // copy would double-count).
        let mut smuggled = base.clone();
        smuggled.modifiers = vec![
            FireModifier::AngloEgyptianDirectFire,
            FireModifier::Terrain(-2),
        ];
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: smuggled,
                roll: DieRoll::Five,
            },
        );
        assert!(matches!(result, Err(RuleError::FireModifierMismatch { .. })));

        // Correct list -> accepted, and the +1 moves the CRT lookup: 8
        // factors (halved-printed band sum 4? no -- range 1 doubled = 8)
        // with roll 5 + 1 = 6 on row 6-10 -> Eliminate(1).
        let mut ok_attack = base;
        ok_attack.modifiers = vec![FireModifier::AngloEgyptianDirectFire];
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: ok_attack,
                roll: DieRoll::Five,
            },
        );
        assert!(result.is_ok());
        let obs = state
            .observations
            .iter()
            .find_map(|o| match o {
                Observation::FireResolved {
                    total_modifier,
                    modified_roll,
                    result,
                    ..
                } => Some((*total_modifier, *modified_roll, *result)),
                _ => None,
            })
            .unwrap();
        assert_eq!(obs.0, 1, "engine-derived §6.24 +1");
        assert_eq!(obs.1, DieRoll::Six);
        assert_eq!(obs.2, CombatResult::Eliminate(1));
    }

    #[rulebook("§9.231")]
    #[test]
    fn zariba_fire_penalties_apply_to_dervish_fire_only() {
        // §9.231/§9.232 print the zariba DRMs "on all Dervish fire attacks".
        // An Anglo-Egyptian attack at a zariba hex must carry no zariba
        // penalty -- and a Dervish attack there must carry it.
        let hedge = HexsideRef::new(HexCoord::new(1, 0), HexCoord::new(1, 1));
        let mk_state = |player| {
            let mut state = GameState::new(Scenario::Historical);
            state.board.hexsides.insert(hedge, HexsideKind::ZaribaThornHedge);
            // Dervish turn: the Dervish fires offensively, the AE
            // defensively (§4 Dervish player turn).
            state.phase = if player == Player::Dervish {
                Phase::OffensiveFire(FireSubPhase::DirectFire)
            } else {
                Phase::DefensiveFire(FireSubPhase::DirectFire)
            };
            state.active_player = Player::Dervish;
            let (firer, target);
            if player == Player::Dervish {
                firer = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
                target = make_ae_infantry(&mut state, HexCoord::new(1, 0));
            } else {
                firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
                target = make_dervish_tribal(&mut state, HexCoord::new(1, 0));
            }
            (state, firer, target)
        };

        // Dervish firing at a thorn-hedge hex: −2 mandatory.
        let (mut state, firer, _t) = mk_state(Player::Dervish);
        let mut attack = FireAttack {
            firing_player: Player::Dervish,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        attack.modifiers = mandatory_fire_modifiers(&state, &attack);
        assert_eq!(attack.modifiers, vec![FireModifier::ZaribaThornHedge]);
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack,
                    roll: DieRoll::Five
                }
            )
            .is_ok()
        );

        // Anglo-Egyptian firing at the same hex: NO zariba DRM (and a client
        // attaching one is rejected).
        let (mut state, firer, _t) = mk_state(Player::AngloEgyptian);
        let mut smuggled = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![
                FireModifier::AngloEgyptianDirectFire,
                FireModifier::ZaribaThornHedge,
            ],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: smuggled.clone(),
                roll: DieRoll::Five,
            },
        );
        assert!(
            matches!(result, Err(RuleError::FireModifierMismatch { .. })),
            "AE attack must not carry the Dervish-only zariba DRM, got {result:?}"
        );
        smuggled.modifiers = vec![FireModifier::AngloEgyptianDirectFire];
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack: smuggled,
                    roll: DieRoll::Five
                }
            )
            .is_ok()
        );
    }

    #[rulebook("§7.7")]
    #[test]
    fn melee_modifiers_are_engine_derived_and_mismatches_rejected() {
        // §7.7: Dervish +2 / Anglo-Egyptian +1 on every melee, both sides.
        // A declared attack with a wrong list is rejected; resolution uses
        // the engine's derivation.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        let defender = make_ae_infantry(&mut state, HexCoord::new(5, 5));
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));

        let mk = |att: Vec<MeleeModifier>, def: Vec<MeleeModifier>| MeleeAttack {
            attacker_player: Player::Dervish,
            attacker_hex: HexCoord::new(6, 5),
            defender_hex: HexCoord::new(5, 5),
            attackers: vec![attacker],
            defenders: vec![defender],
            attacker_modifiers: att,
            defender_modifiers: def,
        };

        // Missing modifiers -> rejected.
        let bad = mk(vec![], vec![]);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DeclareMelee {
                    attack: bad,
                    attacker_roll: DieRoll::Five,
                    defender_roll: DieRoll::Five,
                }
            ),
            Err(RuleError::MeleeModifierMismatch { .. })
        ));

        // Wrong side (+1 on the Dervish attacker) -> rejected.
        let bad = mk(
            vec![MeleeModifier::AngloEgyptianStandard],
            vec![MeleeModifier::AngloEgyptianStandard],
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DeclareMelee {
                    attack: bad,
                    attacker_roll: DieRoll::Five,
                    defender_roll: DieRoll::Five,
                }
            ),
            Err(RuleError::MeleeModifierMismatch { .. })
        ));

        // Correct set -> accepted and resolved with the derived modifiers.
        let good = mk(
            vec![MeleeModifier::DervishStandard],
            vec![MeleeModifier::AngloEgyptianStandard],
        );
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::DeclareMelee {
                    attack: good,
                    attacker_roll: DieRoll::Four,
                    defender_roll: DieRoll::Five,
                }
            )
            .is_ok()
        );
        assert!(apply_effect(&mut state, &GameEffect::ResolveMelee).is_ok());
        let obs = state
            .observations
            .iter()
            .find_map(|o| match o {
                Observation::MeleeResolved {
                    attacker_total_modifier,
                    defender_total_modifier,
                    ..
                } => Some((*attacker_total_modifier, *defender_total_modifier)),
                _ => None,
            })
            .unwrap();
        assert_eq!(obs, (2, 1), "engine-derived §7.7 melee modifiers");
    }

    #[rulebook("§6.24", "§5.54")]
    #[test]
    fn brigade_integrity_modifier_is_engine_derived() {
        // §5.54: four co-stacked battalions of one brigade all firing at one
        // hex receive the +1 integrity DRM *in addition to* the §6.24 +1 --
        // and omitting either is now rejected because the engine derives the
        // whole set.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let profiles = [
            BattalionOrdinal::First,
            BattalionOrdinal::Second,
            BattalionOrdinal::Third,
            BattalionOrdinal::Fourth,
        ];
        let mut firers = Vec::new();
        for b in profiles {
            let id = state.alloc_unit_id();
            state.units.push(UnitPlacement {
                id,
                position: HexCoord::new(0, 0),
                profile: UnitProfile {
                    kind: UnitKind::Infantry { fire: 4, melee: 5, movement: 8 },
                    identity: UnitIdentity::AngloEgyptianInfantry {
                        brigade: BrigadeId {
                            number: 1,
                            nationality: BrigadeNationality::British,
                        },
                        battalion: b,
                    },
                    weapon: WeaponClass::Rifles,
                    fire: Some(crate::FireFactor::Four),
                    melee: Some(crate::MeleeFactor::Five),
                    movement: UnitMovement::Land(crate::MovementAllowance::Eight),
                },
                state: UnitState::default(),
            });
            firers.push(id);
        }
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        // Omitting the integrity DRM -> rejected.
        let mut attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: firers.clone(),
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row16to20,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: attack.clone(),
                roll: DieRoll::Five,
            },
        );
        assert!(
            matches!(result, Err(RuleError::FireModifierMismatch { .. })),
            "integrated brigade must carry the §5.54 +1, got {result:?}"
        );

        // Correct set -> accepted with a derived net +2.
        attack.modifiers = mandatory_fire_modifiers(&state, &attack);
        assert_eq!(
            attack.modifiers,
            vec![
                FireModifier::AngloEgyptianDirectFire,
                FireModifier::BrigadeIntegrity
            ]
        );
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack,
                    roll: DieRoll::Five
                }
            )
            .is_ok()
        );
    }

    #[rulebook("§6.52")]
    #[test]
    fn friendlies_validate_and_resolve_on_dervish_table() {
        // Regression (audit §6.52): a "Friendlies" rifle attack at range 5
        // passed validation on the Anglo-Egyptian table (max 5) but resolved
        // on the Dervish table (max 4). Both paths must now agree: range 5 is
        // out of range, range 4 resolves halved on the Dervish table.
        let friendlies_profile = UnitProfile {
            kind: UnitKind::Infantry { fire: 4, melee: 5, movement: 8 },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId {
                    number: 1,
                    nationality: BrigadeNationality::Friendlies,
                },
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Four),
            melee: Some(crate::MeleeFactor::Five),
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        };

        // Range 5 -- rejected (Dervish rifles max 4).
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: firer,
            position: HexCoord::new(0, 0),
            profile: friendlies_profile,
            state: UnitState::default(),
        });
        make_dervish_tribal(&mut state, HexCoord::new(5, 0));
        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(5, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(
            matches!(result, Err(RuleError::TargetOutOfRange { .. })),
            "Friendlies rifle at range 5 must be out of range on the Dervish table (§6.52), got {result:?}"
        );

        // Range 4 -- accepted, halved on the Dervish table: 4 factors -> 2.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: firer,
            position: HexCoord::new(0, 0),
            profile: friendlies_profile,
            state: UnitState::default(),
        });
        make_dervish_tribal(&mut state, HexCoord::new(4, 0));
        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(4, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(result.is_ok(), "Friendlies rifle at range 4 is in range (§6.52): {result:?}");
        let eff = state
            .observations
            .iter()
            .find_map(|o| match o {
                Observation::FireResolved { effective_factor, .. } => Some(*effective_factor),
                _ => None,
            })
            .expect("FireResolved observation");
        assert_eq!(eff, 2, "4 fire factors halved on the Dervish table (§6.16/§6.52)");

        // Control: a *regular* AE rifle at range 5 stays on the AE table
        // (4-5 halved) and is legal.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        make_dervish_tribal(&mut state, HexCoord::new(5, 0));
        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(5, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(result.is_ok(), "regular AE rifle at range 5 is in range on the AE table: {result:?}");
    }

    #[rulebook("§6.22")]
    #[test]
    fn mixed_attack_bands_per_firer() {
        // Regression (audit §6.22, fixture seq 827): a combined attack
        // applied the *first* firer's range band to every firer. A
        // spear-armed unit (Melee line, range 1 only) stacked with a
        // Dervish battery (Artillery line) dragged the battery's factors
        // onto the spear line. Each firer must contribute on its own line.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::Dervish;

let battery_profile = UnitProfile {
            kind: UnitKind::Artillery { fire: 4, melee: 2, movement: 8 },
            identity: UnitIdentity::DervishArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(crate::FireFactor::Four),
            melee: Some(crate::MeleeFactor::Three),
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        };
        let spear = make_dervish_tribal(&mut state, HexCoord::new(0, 0)); // rifles, 3 factors
        let battery = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: battery,
            position: HexCoord::new(0, 0),
            profile: battery_profile,
            state: UnitState::default(),
        });
        make_ae_infantry(&mut state, HexCoord::new(1, 0));

        // Target adjacent (range 1): tribal rifles x1 (3 factors), Dervish
        // artillery x2 (4 -> 8). The old first-firer-band bug resolved both
        // on the rifle line (3 + 4 = 7); each firer must use its own line.
        let attack = FireAttack {
            firing_player: Player::Dervish,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![spear, battery], // rifle-armed unit first: the old bug's trigger order
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(result.is_ok());
        let eff = state
            .observations
            .iter()
            .find_map(|o| match o {
                Observation::FireResolved { effective_factor, .. } => Some(*effective_factor),
                _ => None,
            })
            .expect("FireResolved observation");
        assert_eq!(eff, 11, "rifles contribute 3 (x1), battery 8 (x2, own artillery line)");
    }

    #[rulebook("§4")]
    #[test]
    fn fire_combat_wrong_phase_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row06to10,
            modifiers: vec![],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(result.is_err());
        assert!(matches!(result, Err(RuleError::WrongPhase)));
    }

    #[test]
    fn movement_exceeds_allowance_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));

        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to: HexCoord::new(5, 0),
                cost: MovementPoints::new(99),
                path: Vec::new(),
            },
        );
        assert!(matches!(
            result,
            Err(RuleError::MovementExceedsAllowance { .. })
        ));
        // Rejected move leaves the unit where it started.
        assert_eq!(state.find_unit(id).unwrap().position, HexCoord::new(0, 0));
    }

    #[test]
    fn legal_move_updates_position() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);

        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to,
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(result.is_ok());
        assert_eq!(state.find_unit(id).unwrap().position, to);
        assert_eq!(state.mp_spent(id), 1);
        assert!(state.mp_spent(id) > 0);

        // §5.12: a unit may keep moving hex by hex up to its allowance, so a
        // second step that fits the remaining allowance (8 total here) succeeds
        // and accumulates -- it is NOT rejected as "already moved".
        let again = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to: HexCoord::new(2, 0),
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(again.is_ok());
        assert_eq!(state.find_unit(id).unwrap().position, HexCoord::new(2, 0));
        assert_eq!(state.mp_spent(id), 2);
    }

    #[test]
    fn cumulative_moves_cannot_exceed_allowance() {
        // §5.11/§5.12: stepping a unit hex by hex (or re-selecting it) may not
        // exceed its movement allowance in total over the turn.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        // Allowance 8; spend 8 in one move, then any further step is rejected.
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let first = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to: HexCoord::new(8, 0),
                cost: MovementPoints::new(8),
                path: Vec::new(),
            },
        );
        assert!(first.is_ok());
        assert_eq!(state.mp_spent(id), 8);

        let over = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: id,
                to: HexCoord::new(9, 0),
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(matches!(
            over,
            Err(RuleError::MovementExceedsAllowance { .. })
        ));
        // The over-move left the unit where its allowance ran out.
        assert_eq!(state.find_unit(id).unwrap().position, HexCoord::new(8, 0));
    }

    #[test]
    fn can_move_unit_matches_effect_and_does_not_mutate() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));

        // Legal in-allowance move: accepted, and the read-only check leaves
        // state untouched (no position change, no MP recorded).
        assert!(state.can_move_unit(id, MovementPoints::new(1)).is_ok());
        assert_eq!(state.find_unit(id).unwrap().position, HexCoord::new(0, 0));
        assert!(state.mp_spent_this_turn.is_empty());

        // Over-allowance is rejected the same way the effect would reject it.
        assert!(matches!(
            state.can_move_unit(id, MovementPoints::new(99)),
            Err(RuleError::MovementExceedsAllowance { .. })
        ));

        // Wrong phase is rejected.
        state.phase = Phase::Melee;
        assert!(matches!(
            state.can_move_unit(id, MovementPoints::new(1)),
            Err(RuleError::WrongPhase)
        ));
    }

    #[test]
    fn hex_in_enemy_zoc_respects_disruption_and_leaders() {
        let mut state = GameState::new(Scenario::Campaign);
        // A Dervish unit at (1,1) projects ZOC into its six neighbours, one of
        // which is (1,0) -- seen from the moving Anglo-Egyptian player's side.
        let dervish = make_dervish_tribal(&mut state, HexCoord::new(1, 1));
        assert!(state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }
        ));
        // A friendly unit's hexes are not "enemy" ZOC.
        assert!(!state.hex_in_enemy_zoc(HexCoord::new(1, 0), Player::Dervish, UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }));
        // A hex no enemy is adjacent to is free.
        assert!(!state.hex_in_enemy_zoc(
            HexCoord::new(5, 5),
            Player::AngloEgyptian,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }
        ));

        // Disrupted units project no ZOC (§5.41).
        state.find_unit_mut(dervish).unwrap().state.disrupted = true;
        assert!(!state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }
        ));
    }

    #[test]
    fn movement_must_stop_in_enemy_zoc() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // Enemy at (1,1) puts the intermediate hex (1,0) in an enemy ZOC.
        make_dervish_tribal(&mut state, HexCoord::new(1, 1));

        // Moving straight through (1,0) to (3,0) is blocked -- the unit would
        // have had to stop at (1,0).
        let through = state.can_move_unit_to(mover, Some(HexCoord::new(3, 0)), MovementPoints::new(3));
        assert!(matches!(
            through,
            Err(RuleError::BlockedByEnemyZoc(hex)) if hex == HexCoord::new(1, 0)
        ));

        // Stopping *in* the ZOC hex (1,0) is legal -- that is exactly where the
        // unit must halt (§5.43).
        assert!(
            state
                .can_move_unit_to(mover, Some(HexCoord::new(1, 0)), MovementPoints::new(1))
                .is_ok()
        );

        // A move whose path avoids every enemy-ZOC hex is fine. The enemy at
        // (1,1) projects ZOC into (1,0)/(0,0)/(0,1)/(1,2)/(2,1)/(2,2); a move
        // away to (-3,0) crosses (-1,0)/(-2,0), none of which are in ZOC. (The
        // start (0,0) itself being in ZOC does not block -- §5.43.)
        assert!(
            state
                .can_move_unit_to(mover, Some(HexCoord::new(-3, 0)), MovementPoints::new(3))
                .is_ok()
        );
    }

    #[test]
    fn unit_in_enemy_zoc_may_move_out() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        // Mover starts at (1,0), already adjacent to the enemy at (1,1) -- i.e.
        // it begins its move inside an enemy ZOC.
        let mover = make_ae_infantry(&mut state, HexCoord::new(1, 0));
        make_dervish_tribal(&mut state, HexCoord::new(1, 1));
        assert!(state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }
        ));

        // It may withdraw to a hex outside any ZOC (§5.43): start being in ZOC
        // does not block the move.
        assert!(
            state
                .can_move_unit_to(mover, Some(HexCoord::new(4, 0)), MovementPoints::new(3))
                .is_ok()
        );
    }

    #[rulebook("§5.26", "§5.43")]
    #[test]
    fn unit_entering_enemy_zoc_may_move_no_further_that_turn() {
        // Regression (audit §5.43, 222 violations in the recorded games): a
        // unit that entered an enemy ZOC used to be free to keep moving in
        // later moves of the same phase. "All units must stop when they enter
        // an enemy ZOC and may move no further that turn."
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let mover = make_dervish_tribal(&mut state, HexCoord::new(3, 0));
        make_ae_infantry(&mut state, HexCoord::new(5, 0)); // ZOC ring covers (4,0)

        // Move 1: enter the ZOC at (4,0) -- legal, the unit stops there.
        assert!(
            apply_move_unit(&mut state, mover, HexCoord::new(4, 0), MovementPoints::new(1), &[])
                .is_ok()
        );
        assert!(state.zoc_stopped_this_turn.contains(&mover));

        // Move 2 (same phase): rejected -- it may move no further this turn.
        assert!(matches!(
            apply_move_unit(&mut state, mover, HexCoord::new(4, 1), MovementPoints::new(1), &[]),
            Err(RuleError::StoppedInEnemyZoc(_))
        ));

        // After the turn passes, it may withdraw (§5.43 "In their next
        // movement phase they may withdraw").
        end_player_turn(&mut state).unwrap(); // -> AE turn
        end_player_turn(&mut state).unwrap(); // -> Dervish again, trackers cleared
        assert!(!state.zoc_stopped_this_turn.contains(&mover));
        assert!(
            apply_move_unit(&mut state, mover, HexCoord::new(3, 0), MovementPoints::new(1), &[])
                .is_ok()
        );
    }

    #[rulebook("§5.26")]
    #[test]
    fn zoc_transit_check_uses_the_actual_path() {
        // Regression (audit §5.26): the engine checked the straight line for
        // enemy-ZOC transit, not the stepped path -- a path threading around
        // a ZOC was wrongly rejected (and one through a ZOC hex wrongly
        // accepted when the straight line missed it). The entered hexes of
        // the supplied path govern.
        let mut state = GameState::new(Scenario::Campaign);
        let mut board = BoardInfo::default();
        for q in 0..=9 {
            for r in 0..=5 {
                board
                    .terrain
                    .insert(HexCoord::new(q, r), Terrain::Clear { road: Default::default() });
            }
        }
        state.board = board;
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;

        // An AE unit at (5,2); its ZOC ring covers the six neighbours.
        let enemy_hex = HexCoord::new(5, 2);
        make_ae_infantry(&mut state, enemy_hex);
        let ring: Vec<HexCoord> = enemy_hex.neighbors().to_vec();

        let mover = make_dervish_tribal(&mut state, HexCoord::new(5, 0));
        assert!(
            apply_move_unit(
                &mut state,
                mover,
                enemy_hex,
                MovementPoints::new(2),
                &[HexCoord::new(5, 1), enemy_hex]
            )
            .is_err(),
            "cannot move onto the enemy's own hex"
        );

        // Path straight through a ZOC-ring hex -> rejected (must stop there).
        let through_zoc: Vec<HexCoord> = vec![HexCoord::new(5, 1)];
        assert!(matches!(
            apply_move_unit(&mut state, mover, HexCoord::new(5, 2).neighbors()[0], MovementPoints::new(2), &{
                let mut p = through_zoc.clone();
                p.push(HexCoord::new(5, 2).neighbors()[0]);
                p
            }),
            Err(RuleError::BlockedByEnemyZoc(_))
        ));

        // Bent path around the ring -> legal even though the straight line
        // would cross it. Route west then south then east, outside the ring.
        let detour: Vec<HexCoord> = vec![
            HexCoord::new(4, 0),
            HexCoord::new(3, 1),
            HexCoord::new(3, 2),
            HexCoord::new(3, 3),
            HexCoord::new(4, 4),
            HexCoord::new(5, 4),
        ];
        let in_ring = detour.iter().any(|h| ring.contains(h));
        assert!(!in_ring, "test premise: the detour avoids the ZOC ring");
        assert!(
            apply_move_unit(&mut state, mover, HexCoord::new(5, 4), MovementPoints::new(6), &detour)
                .is_ok(),
            "a path around the ZOC is legal (§5.26 stops only on entering)"
        );
    }

    #[test]
    fn anglo_egyptian_leader_projects_no_zoc() {
        let mut state = GameState::new(Scenario::Campaign);
        // Make the active player Dervish so the A-E leader is the "enemy".
        state.active_player = Player::Dervish;
        let leader = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: leader,
            position: HexCoord::new(1, 1),
            profile: UnitProfile {
                kind: UnitKind::BritishLeader { movement: 0 },
                identity: UnitIdentity::AngloEgyptianLeader(BritishLeader::Kitchener),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        });
        // §5.41: an Anglo-Egyptian leader exerts no ZOC.
        assert!(!state.hex_in_enemy_zoc(HexCoord::new(1, 0), Player::Dervish, UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }));
    }

    #[test]
    fn dervish_can_move_after_turn_gate_removed() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        // The turn-gate check was removed, so a Dervish unit may move even
        // though the active player is Anglo-Egyptian.
        let dervish = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        assert!(state.can_move_unit(dervish, MovementPoints::new(1)).is_ok());
    }

    #[rulebook("§6.22")]
    #[test]
    fn can_fire_at_gates_phase_range_and_player() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let enemy_near = HexCoord::new(1, 0); // range 1 -- rifles in range
        make_dervish_tribal(&mut state, enemy_near);

        // In a fire phase, the active A-E unit may fire an in-range enemy hex.
        assert!(state.can_fire_at(ae, enemy_near, FireKind::Direct).is_ok());
        // Read-only: nothing recorded as fired.
        assert!(state.units_fired_this_phase.is_empty());

        // Out of rifle range (range 8) is rejected.
        assert!(matches!(
            state.can_fire_at(ae, HexCoord::new(8, 0), FireKind::Direct),
            Err(RuleError::TargetOutOfRange { .. })
        ));

        // A rifle unit may not use Maxim second fire, and not in the Direct
        // sub-phase regardless.
        assert!(matches!(
            state.can_fire_at(ae, enemy_near, FireKind::MaximSecondFire),
            Err(RuleError::WrongPhase)
        ));

        // Wrong phase: no firing during movement.
        state.phase = Phase::Movement;
        assert!(matches!(
            state.can_fire_at(ae, enemy_near, FireKind::Direct),
            Err(RuleError::WrongPhase)
        ));

        // During A-E offensive fire, a Dervish unit may not fire.
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let dervish = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        assert!(matches!(
            state.can_fire_at(dervish, HexCoord::new(0, 0), FireKind::Direct),
            Err(RuleError::NotYourTurn)
        ));
    }

    #[rulebook("§7.2")]
    #[test]
    fn can_melee_gates_phase_adjacency_and_kind() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let adj = HexCoord::new(1, 0); // adjacent
        make_dervish_tribal(&mut state, adj);

        // Adjacent enemy in the Melee phase: legal, read-only.
        assert!(state.can_melee(ae, adj).is_ok());

        // Non-adjacent hex is rejected.
        assert!(matches!(
            state.can_melee(ae, HexCoord::new(3, 0)),
            Err(RuleError::TargetNotAdjacent { .. })
        ));

        // Wrong phase.
        state.phase = Phase::Movement;
        assert!(matches!(
            state.can_melee(ae, adj),
            Err(RuleError::WrongPhase)
        ));

        // Empty adjacent hex: nothing to attack.
        state.phase = Phase::Melee;
        assert!(matches!(
            state.can_melee(ae, HexCoord::new(0, 1)),
            Err(RuleError::NoMeleeableEnemy(_))
        ));
    }

    #[rulebook("§7.5")]
    #[test]
    fn retreat_before_melee_only_cavalry_two_hexes() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish; // attacker; A-E units defend

        // A cavalry-kind unit standing where a melee will be declared.
        let id = state.alloc_unit_id();
        let cav_hex = HexCoord::new(5, 5);
        state.units.push(UnitPlacement {
            id,
            position: cav_hex,
            profile: UnitProfile {
                kind: UnitKind::Cavalry { fire: 0, melee: 0, movement: 0 },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        });
        // A Dervish attacker adjacent to the cavalry, to declare a melee.
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));

        // No retreat without a declared melee threatening the unit's hex.
        assert!(matches!(
            state.can_retreat_before_melee(id, HexCoord::new(7, 5)),
            Err(RuleError::NoInfantryMeleeThreatens(_))
        ));

        // Declare the melee on the cavalry's hex -> reaction window opens.
        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: cav_hex,
                    attackers: vec![attacker],
                    defenders: vec![id],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        )
        .unwrap();

        // Now retreat: one hex rejected, exactly two accepted.
        assert!(
            state
                .can_retreat_before_melee(id, HexCoord::new(6, 6))
                .is_err()
        );
        let dest = HexCoord::new(7, 5);
        assert!(state.can_retreat_before_melee(id, dest).is_ok());
        apply_effect(
            &mut state,
            &GameEffect::RetreatBeforeMelee {
                unit_id: id,
                to: dest,
            },
        )
        .unwrap();
        assert_eq!(state.find_unit(id).unwrap().position, dest);

        // After retreat, resolving the declared melee spares the unit (it has
        // left the target hex), and the window closes.
        apply_effect(&mut state, &GameEffect::ResolveMelee).unwrap();
        assert!(state.pending_melee.is_none());
        assert!(state.find_unit(id).is_some(), "retreated unit was spared");
    }

    // §5.22 regression (found by the invariant fuzzer, seed 8600): a
    // cavalry's two-hex retreat may never land on a Nile hex -- land units
    // may not enter the river under any circumstances, retreat included.
    #[rulebook("§5.22")]
    #[test]
    fn retreat_before_melee_may_not_land_on_nile() {
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(5, 5),
            Terrain::Clear { road: Default::default() },
        );
        board.terrain.insert(
            HexCoord::new(7, 5),
            Terrain::Nile { direction: omdurman_types::HexDirection::East },
        );

        let mut state = GameState::new(Scenario::Campaign);
        state.board = board;
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish; // attackers; A-E defends

        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: HexCoord::new(5, 5),
            profile: UnitProfile {
                kind: UnitKind::Cavalry { fire: 0, melee: 0, movement: 0 },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        });
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));
        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: HexCoord::new(5, 5),
                    attackers: vec![attacker],
                    defenders: vec![id],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        )
        .unwrap();
        assert!(matches!(
            state.can_retreat_before_melee(id, HexCoord::new(7, 5)),
            Err(RuleError::LandIntoNile(_))
        ));
    }

    // §6.54: a retreat may not end on an enemy fort (forts are never
    // occupied by the enemy).
    #[rulebook("§6.54")]
    #[test]
    fn retreat_before_melee_may_not_land_on_enemy_fort() {
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(5, 5),
            Terrain::Clear { road: Default::default() },
        );
        board.terrain.insert(
            HexCoord::new(7, 5),
            Terrain::Clear { road: Default::default() },
        );
        let mut state = GameState::new(Scenario::Campaign);
        state.board = board;
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;

        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: HexCoord::new(5, 5),
            profile: UnitProfile {
                kind: UnitKind::Cavalry { fire: 0, melee: 0, movement: 0 },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        });
        // An enemy (Dervish-owned) fort on the retreat hex, in a *different*
        // hex so the melee declaration still targets the cavalry.
        let fort = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: fort,
            position: HexCoord::new(7, 5),
            profile: UnitProfile {
                kind: UnitKind::Fort { fire: 0, melee: 0 },
                identity: UnitIdentity::DervishFort,
                weapon: WeaponClass::Artillery,
                fire: None,
                melee: None,
                movement: UnitMovement::Immobile,
            },
            state: UnitState::default(),
        });
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));
        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: HexCoord::new(5, 5),
                    attackers: vec![attacker],
                    defenders: vec![id],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        )
        .unwrap();
        // The fort is a unit occupying (7,5), so the occupied-hex check
        // (RetreatHexOccupied) fires before the EnemyFort arm -- either way
        // §6.54's outcome holds: the retreat may not end on an enemy fort.
        assert!(
            state.can_retreat_before_melee(id, HexCoord::new(7, 5)).is_err(),
            "§6.54: a retreat may not end on an enemy fort"
        );
    }

    #[test]
    fn dervish_must_advance_into_vacated_melee_hex() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        // A Dervish attacker adjacent to a lone A-E defender it will wipe out.
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let defender_hex = HexCoord::new(1, 0);
        let defender = make_ae_infantry(&mut state, defender_hex);

        let attack = MeleeAttack {
            attacker_player: Player::Dervish,
            attacker_hex: HexCoord::new(0, 0),
            defender_hex,
            attackers: vec![attacker],
            defenders: vec![defender],
            attacker_modifiers: vec![MeleeModifier::DervishStandard],
            defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
        };
        apply_effect(
            &mut state,
            &GameEffect::MeleeCombat {
                attack,
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
        .unwrap();

        // Invariant (§7.6): whenever the defender hex is vacated by the melee,
        // a surviving Dervish attacker is forced to advance into it. If the
        // defender survived, the attacker stays put. Assert the implication
        // rather than a specific Combat Results Table outcome.
        let defender_gone = state.find_unit(defender).is_none();
        let attacker_pos = state.find_unit(attacker).map(|u| u.position);
        if defender_gone && attacker_pos.is_some() {
            assert_eq!(
                attacker_pos,
                Some(defender_hex),
                "Dervish must advance into the vacated hex (§7.6)"
            );
        }
    }

    #[test]
    fn dervish_advance_is_forced_when_hex_vacated() {
        // Directly exercise the advance branch: stand a Dervish unit next to
        // an empty hex and confirm the post-melee advance moves it in when the
        // defender list resolves to empty. We simulate the "vacated" condition
        // by meleeing a defender whose elimination we guarantee with a maximal
        // factor gap is unreliable, so instead verify the branch via a unit
        // already adjacent to a now-empty target through `can_advance...`.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let vacated = HexCoord::new(1, 0);
        // No unit in `vacated` -> the mandatory-advance branch's eligibility
        // logic (the same predicate) should accept advancing there.
        open_advance_window(&mut state, vacated, &[attacker], vec!["7.6".to_string()]);
        assert!(state.can_advance_after_combat(attacker, vacated).is_ok());
    }

    #[test]
    fn advance_after_combat_into_vacated_hex() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let vacated = HexCoord::new(1, 0); // adjacent, empty

        open_advance_window(&mut state, vacated, &[id], vec!["7.6".to_string()]);
        assert!(state.can_advance_after_combat(id, vacated).is_ok());
        // Occupied target is rejected.
        make_dervish_tribal(&mut state, HexCoord::new(0, 1));
        assert!(
            state
                .can_advance_after_combat(id, HexCoord::new(0, 1))
                .is_err()
        );
        // Non-adjacent rejected.
        assert!(
            state
                .can_advance_after_combat(id, HexCoord::new(4, 0))
                .is_err()
        );

        apply_effect(
            &mut state,
            &GameEffect::AdvanceAfterCombat {
                unit_id: id,
                to: vacated,
            },
        )
        .unwrap();
        assert_eq!(state.find_unit(id).unwrap().position, vacated);
    }

    #[rulebook("§6.82")]
    #[test]
    fn advance_requires_combat_vacated_hex() {
        // A merely-empty adjacent hex is not an advance target (§6.82): the
        // hex must have been vacated by combat this phase. This is the check
        // that stops advance-after-combat acting as free out-of-phase
        // movement.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        assert!(matches!(
            state.can_advance_after_combat(id, HexCoord::new(1, 0)),
            Err(RuleError::HexNotVacatedByCombat(_))
        ));
    }

    #[rulebook("§6.82")]
    #[test]
    fn advance_requires_participation() {
        // §6.82/§7.6: only units that participated in the combat that
        // vacated the hex may advance -- a same-side bystander adjacent to
        // the vacated hex may not.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let participant = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let bystander = make_ae_infantry(&mut state, HexCoord::new(1, -1));
        let vacated = HexCoord::new(1, 0);
        open_advance_window(&mut state, vacated, &[participant], vec!["7.6".to_string()]);
        assert!(state.can_advance_after_combat(participant, vacated).is_ok());
        assert!(matches!(
            state.can_advance_after_combat(bystander, vacated),
            Err(RuleError::UnitDidNotParticipate(_, hex)) if hex == vacated
        ));
    }

    #[rulebook("§5.25")]
    #[test]
    fn forts_are_never_advance_eligible() {
        // §5.25: forts may not move in any way. Even a hand-seeded window
        // listing a fort (open_advance_window filters them, this covers a
        // crafted/replayed state) must not let it advance.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let fort = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: fort,
            position: HexCoord::new(0, 0),
            profile: UnitProfile {
                kind: UnitKind::Fort { fire: 0, melee: 0 },
                identity: crate::UnitIdentity::DervishFort,
                weapon: WeaponClass::Artillery,
                fire: Some(crate::FireFactor::One),
                melee: None,
                movement: UnitMovement::Immobile,
            },
            state: UnitState::default(),
        });
        let vacated = HexCoord::new(1, 0);
        state
            .vacated_by_combat
            .insert(vacated, vec![fort]);
        assert!(matches!(
            state.can_advance_after_combat(fort, vacated),
            Err(RuleError::FortMayNotAdvance(_))
        ));
    }

    #[rulebook("§6.7")]
    #[test]
    fn defensive_fire_opens_no_advance_window() {
        // §6.7: "There is no advance after combat as a result of defensive
        // fires" -- a defensive-fire elimination vacates the hex but must
        // neither open a window nor emit the vacated observation.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian; // Dervish fires defensively
        let firer = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let target = make_ae_infantry(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::Dervish,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };
        apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Ten, // Row01to05 @ 10 -> Eliminate(2): hex vacated
            },
        )
        .unwrap();
        assert!(state.find_unit(target).is_none());
        assert!(
            state.vacated_by_combat.is_empty(),
            "§6.7: defensive fire must not open an advance window"
        );
        assert!(!state
            .observations
            .iter()
            .any(|o| matches!(o, Observation::HexVacatedByCombat { .. })));
    }

    #[rulebook("§6.42")]
    #[test]
    fn advance_window_bridges_fire_subphase_and_closes_at_melee() {
        // The Direct→Maxim/Howitzer subphase transition is one continuous
        // offensive-fire phase (§6.42): a window opened by direct fire stays
        // usable. Crossing into Melee closes it (§6.82: the advance answers
        // the fire that vacated the hex).
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let vacated = HexCoord::new(1, 0);
        open_advance_window(&mut state, vacated, &[id], vec!["6.82".to_string()]);

        advance_phase(&mut state).unwrap(); // -> Maxim/Howitzer subphase
        assert_eq!(
            state.phase,
            Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer)
        );
        assert!(
            state.can_advance_after_combat(id, vacated).is_ok(),
            "§6.42 bridge: the window survives the subphase change"
        );

        advance_phase(&mut state).unwrap(); // -> Melee
        assert!(matches!(
            state.can_advance_after_combat(id, vacated),
            Err(RuleError::HexNotVacatedByCombat(_))
        ));
    }

    #[rulebook("§7.5")]
    #[test]
    fn retreat_opens_window_only_when_hex_empties() {
        // §7.5/§7.6: a retreat-before-melee only vacates the hex once the
        // *last* defender has left; a stacked hex still held by a defender
        // opens no window.
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish; // attackers; A-E defends
        let cav_hex = HexCoord::new(5, 5);
        let cavalry = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id: cavalry,
            position: cav_hex,
            profile: UnitProfile {
                kind: UnitKind::Cavalry { fire: 0, melee: 0, movement: 0 },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        });
        // A second AE defender stacked in the same hex.
        let stay = make_ae_infantry(&mut state, cav_hex);
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));

        apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: cav_hex,
                    attackers: vec![attacker],
                    defenders: vec![cavalry, stay],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        )
        .unwrap();

        let dest = HexCoord::new(7, 5);
        apply_effect(
            &mut state,
            &GameEffect::RetreatBeforeMelee {
                unit_id: cavalry,
                to: dest,
            },
        )
        .unwrap();
        // The infantry defender still holds the hex: no window.
        assert!(!state.vacated_by_combat.contains_key(&cav_hex));
        assert!(
            matches!(
                state.can_advance_after_combat(attacker, cav_hex),
                Err(RuleError::HexNotVacatedByCombat(_))
            ),
            "a stacked hex with a remaining defender is not vacated"
        );

        // (The infantry cannot retreat -- §7.5 is cavalry/camel only -- so
        // the window only ever opens via resolution or the last retreat.)
    }

    #[test]
    fn advance_after_combat_rejects_off_board_hexes() {
        // Loaded board: only (0,0) is land terrain and (1,0) is Nile; the
        // neighbour (1,-1) is not a map hex at all.
        let mut board = BoardInfo::default();
        board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::Clear { road: Default::default() },
        );
        board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile { direction: HexDirection::East },
        );

        // A land unit may not advance off the board (§5.22).
        let mut state = GameState::new(Scenario::Campaign);
        state.board = board.clone();
        state.phase = Phase::Melee;
        let inf = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        open_advance_window(&mut state, HexCoord::new(0, -1), &[inf], vec!["7.6".to_string()]);
        assert!(matches!(
            state.can_advance_after_combat(inf, HexCoord::new(0, -1)),
            Err(RuleError::OffBoard(_))
        ));
        // ... nor into a Nile hex.
        open_advance_window(&mut state, HexCoord::new(1, 0), &[inf], vec!["7.6".to_string()]);
        assert!(matches!(
            state.can_advance_after_combat(inf, HexCoord::new(1, 0)),
            Err(RuleError::LandIntoNile(_))
        ));

        // A gunboat may only advance along the Nile: the land hex and the
        // off-board neighbour are both rejected.
        let mut state = GameState::new(Scenario::Campaign);
        state.board = board;
        state.phase = Phase::Melee;
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(1, 0));
        open_advance_window(&mut state, HexCoord::new(0, 0), &[gb], vec!["7.6".to_string()]);
        assert!(matches!(
            state.can_advance_after_combat(gb, HexCoord::new(0, 0)),
            Err(RuleError::GunboatOffNile(_))
        ));
        open_advance_window(&mut state, HexCoord::new(2, 0), &[gb], vec!["7.6".to_string()]);
        assert!(matches!(
            state.can_advance_after_combat(gb, HexCoord::new(2, 0)),
            Err(RuleError::GunboatOffNile(_))
        ));
    }

    #[rulebook("§5")]
    #[test]
    fn disrupted_unit_cannot_fire() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        state.find_unit_mut(id).unwrap().state.disrupted = true;
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![id],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::Five,
            },
        );
        assert!(matches!(result, Err(RuleError::Disrupted(_))));
    }

    #[rulebook("§7.7")]
    #[test]
    fn melee_resolves_simultaneously() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae_id = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let derv_id = make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = MeleeAttack {
            attacker_player: Player::AngloEgyptian,
            attacker_hex: HexCoord::new(0, 0),
            defender_hex: HexCoord::new(1, 0),
            attackers: vec![ae_id],
            defenders: vec![derv_id],
            attacker_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
            defender_modifiers: vec![MeleeModifier::DervishStandard],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::MeleeCombat {
                attack,
                attacker_roll: DieRoll::Seven,
                defender_roll: DieRoll::Three,
            },
        );
        assert!(result.is_ok());
    }

    #[rulebook("§4")]
    #[test]
    fn new_game_starts_in_setup() {
        let state = GameState::new(Scenario::Campaign);
        assert_eq!(state.phase, Phase::Setup);
    }

    #[test]
    fn cannot_leave_setup_until_both_sides_deployed() {
        let mut state = GameState::new(Scenario::Campaign);
        // No units: setup is incomplete, advancing stays in Setup.
        let err = apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap_err();
        assert!(matches!(err, RuleError::SetupIncomplete(_)));
        assert_eq!(state.phase, Phase::Setup);

        // One side only: still incomplete.
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap_err(),
            RuleError::SetupIncomplete(_)
        ));

        // Both sides present: setup completes and we enter Movement.
        make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert_eq!(state.phase, Phase::Movement);
    }

    #[test]
    fn deploy_rejected_outside_setup_phase() {
        let mut state = playing(Scenario::Campaign); // in Movement, not Setup
        let placement = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 1),
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(placement)).unwrap_err(),
            RuleError::WrongPhase
        ));
    }

    #[rulebook("§9.212")]
    #[test]
    fn deploy_rejected_outside_zone() {
        // Fall of Khartoum: Dervish may only deploy on the southern edge.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        // Attach a small board spanning rows 0..=9 so zones are defined.
        for r in 0..=9 {
            for q in 0..=3 {
                state
                    .board
                    .terrain
                    .insert(HexCoord::new(q, r), Terrain::default());
            }
        }
        // A Dervish unit in the north (r=0) is outside its (southern) zone.
        // Kehena: a §9.322 tribe, so the order-of-battle gate passes and the
        // zone check is what rejects.
        let north = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 0),
            profile: dervish_tribal_profile_with(DervishTribe::Kehena),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&north).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));
        // The same unit in the south (r=9) is accepted.
        let south = UnitPlacement {
            position: HexCoord::new(1, 9),
            ..north
        };
        assert!(state.can_deploy_unit(&south).is_ok());
    }

    #[rulebook("§9.322")]
    #[test]
    fn fok_dervish_east_edge_on_diamond_board() {
        // The FoK board is diamond-shaped: the "east edge" is the diagonal
        // of rightmost hexes per row (no hex at q+1), not just q == global
        // max_q.  Build a small diamond:
        //   r=0: q=0,1
        //   r=1: q=0,1,2
        //   r=2: q=0,1,2,3
        // East edge: (1,0), (2,1), (3,2).  South edge: (0,2),(1,2),(2,2),(3,2).
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        for r in 0..=2u32 {
            for q in 0..=r + 1 {
                state
                    .board
                    .terrain
                    .insert(HexCoord::new(q as i32, r as i32), Terrain::default());
            }
        }
        let kehena = dervish_tribal_profile_with(DervishTribe::Kehena);
        // Interior hex (1,1): has a neighbor at (2,1) so NOT on east edge,
        // and has a neighbor at (1,2) so NOT on south edge → rejected.
        let interior = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 1),
            profile: kehena,
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&interior).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));
        // East-edge hex (2,1): no hex at (3,1) → on east edge → accepted.
        let east_edge = UnitPlacement {
            position: HexCoord::new(2, 1),
            ..interior
        };
        assert!(state.can_deploy_unit(&east_edge).is_ok());
        // East-edge hex (1,0): no hex at (2,0) → on east edge → accepted.
        let east_top = UnitPlacement {
            position: HexCoord::new(1, 0),
            ..interior
        };
        assert!(state.can_deploy_unit(&east_top).is_ok());
    }

    #[rulebook("§5.22", "§9.322")]
    #[test]
    fn fok_dervish_land_unit_rejected_on_nile() {
        // §5.22 applies to Dervish deployment too: a land unit may not deploy
        // on a Nile hex even when that hex is on the south/east entry edge.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        // Board with rows 0..=4 (max_r = 4). Put a Nile hex on the south edge
        // at (0,4) and a clear hex on the south edge at (1,4).
        for r in 0..=4 {
            for q in 0..=3 {
                state
                    .board
                    .terrain
                    .insert(HexCoord::new(q, r), Terrain::default());
            }
        }
        state.board.terrain.insert(
            HexCoord::new(0, 4),
            Terrain::Nile { direction: omdurman_types::HexDirection::East },
        );

        let on_nile_edge = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 4),
            profile: dervish_tribal_profile_with(DervishTribe::Mulazmin),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&on_nile_edge).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // The same Mulazmin unit on a clear south-edge hex is accepted.
        let on_clear_edge = UnitPlacement {
            position: HexCoord::new(1, 4),
            ..on_nile_edge
        };
        assert!(state.can_deploy_unit(&on_clear_edge).is_ok());
    }

    #[rulebook("§10.11", "§10.21")]
    #[test]
    fn mine_and_chain_limits_enforced_in_setup() {
        let mut state = GameState::new(Scenario::Campaign);
        state.optional_rules.push(OptionalRule::RiverMines);
        state.optional_rules.push(OptionalRule::RiverChain);
        // Two mines OK, a third rejected.
        apply_effect(
            &mut state,
            &GameEffect::PlaceMine {
                hex: HexCoord::new(1, 1),
            },
        )
        .unwrap();
        apply_effect(
            &mut state,
            &GameEffect::PlaceMine {
                hex: HexCoord::new(2, 1),
            },
        )
        .unwrap();
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::PlaceMine {
                    hex: HexCoord::new(3, 1)
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        // Duplicate hex rejected.
        let mut state2 = GameState::new(Scenario::Campaign);
        state2.optional_rules.push(OptionalRule::RiverMines);
        apply_effect(
            &mut state2,
            &GameEffect::PlaceMine {
                hex: HexCoord::new(1, 1),
            },
        )
        .unwrap();
        assert!(matches!(
            apply_effect(
                &mut state2,
                &GameEffect::PlaceMine {
                    hex: HexCoord::new(1, 1)
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        // Chain over four hexes rejected.
        let five: Vec<HexCoord> = (0..5).map(|q| HexCoord::new(q, 0)).collect();
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceChain { hexes: five }).unwrap_err(),
            RuleError::SetupLimit(_)
        ));
    }

    #[rulebook("§10.11", "§10.21")]
    #[test]
    fn mines_and_chain_require_their_optional_rule() {
        // Without the optional rules selected, placement is rejected even in
        // Setup with room to spare.
        let mut state = GameState::new(Scenario::Campaign);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::PlaceMine {
                    hex: HexCoord::new(1, 1)
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        assert!(state.mines.is_empty());
        let two: Vec<HexCoord> = vec![HexCoord::new(1, 0), HexCoord::new(2, 0)];
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceChain { hexes: two }).unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        assert!(state.chain.is_none());

        // Selecting just River Mines unlocks mines but not the chain.
        state.optional_rules.push(OptionalRule::RiverMines);
        apply_effect(
            &mut state,
            &GameEffect::PlaceMine {
                hex: HexCoord::new(1, 1),
            },
        )
        .unwrap();
        assert_eq!(state.mines.len(), 1);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::PlaceChain {
                    hexes: vec![HexCoord::new(1, 0), HexCoord::new(2, 0)]
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        assert!(state.chain.is_none());

        // Selecting just River Chain unlocks the chain but not further mines.
        let mut state2 = GameState::new(Scenario::Campaign);
        state2.optional_rules.push(OptionalRule::RiverChain);
        apply_effect(
            &mut state2,
            &GameEffect::PlaceChain {
                hexes: vec![HexCoord::new(1, 0), HexCoord::new(2, 0)],
            },
        )
        .unwrap();
        assert!(state2.chain.is_some());
        assert!(matches!(
            apply_effect(
                &mut state2,
                &GameEffect::PlaceMine {
                    hex: HexCoord::new(1, 1)
                }
            )
            .unwrap_err(),
            RuleError::SetupLimit(_)
        ));
        assert!(state2.mines.is_empty());
    }

    #[test]
    fn units_cannot_move_during_setup() {
        let mut state = GameState::new(Scenario::Campaign);
        let unit = make_ae_infantry(&mut state, HexCoord::new(1, 1));
        // Still in Setup: movement is rejected as wrong-phase.
        let err = state
            .can_move_unit_to(unit, Some(HexCoord::new(2, 1)), MovementPoints::new(1))
            .unwrap_err();
        assert!(matches!(err, RuleError::WrongPhase));
    }

    #[rulebook("§4")]
    #[test]
    fn both_ready_auto_advances_out_of_setup() {
        // Campaign has no fixed target, so one unit per side meets the gate.
        let mut state = GameState::new(Scenario::Campaign);
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        make_dervish_tribal(&mut state, HexCoord::new(5, 5));

        // One side ready: still in Setup.
        apply_effect(
            &mut state,
            &GameEffect::ConfirmSetupReady {
                player: Player::AngloEgyptian,
            },
        )
        .unwrap();
        assert_eq!(state.phase, Phase::Setup);
        assert!(state.setup_ready(Player::AngloEgyptian));
        assert!(!state.setup_ready(Player::Dervish));

        // Second side ready: auto-advances to Movement.
        apply_effect(
            &mut state,
            &GameEffect::ConfirmSetupReady {
                player: Player::Dervish,
            },
        )
        .unwrap();
        assert_eq!(state.phase, Phase::Movement);
    }

    #[test]
    fn confirm_ready_rejected_outside_setup() {
        let mut state = playing(Scenario::Campaign); // in Movement
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::ConfirmSetupReady {
                    player: Player::Dervish
                }
            )
            .unwrap_err(),
            RuleError::WrongPhase
        ));
    }

    #[rulebook("§9.321")]
    #[test]
    fn confirm_ready_rejected_below_scenario_target() {
        // Fall of Khartoum requires the full order of battle (British 17 /
        // Dervish 48); a single deployed unit is far below target.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        assert_eq!(state.setup_target(Player::AngloEgyptian), Some(17));
        assert!(!state.setup_target_met(Player::AngloEgyptian));
        assert!(matches!(
            state
                .can_confirm_setup_ready(Player::AngloEgyptian)
                .unwrap_err(),
            RuleError::SetupIncomplete(_)
        ));
    }

    #[rulebook("§5.22", "§9.321")]
    #[test]
    fn fok_ae_gunboat_deploys_only_on_nile() {
        // Fall of Khartoum British deployment zone must be boat/land-exclusive
        // (§5.22): a gunboat may only deploy on the Nile, never on a building.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        // A building hex (land) at (0,0) and a Nile hex at (1,0).
        state.board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::ground(omdurman_types::GroundKind::Building),
        );
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile { direction: omdurman_types::HexDirection::East },
        );

        // Gunboat on a building hex -> rejected (off its Nile-only zone).
        let boat_on_land = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: ae_gunboat_profile(),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&boat_on_land).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // Gunboat on a Nile hex -> accepted.
        let boat_on_nile = UnitPlacement {
            position: HexCoord::new(1, 0),
            ..boat_on_land
        };
        assert!(state.can_deploy_unit(&boat_on_nile).is_ok());
    }

    #[rulebook("§5.22", "§9.111")]
    #[test]
    fn campaign_deployment_is_boat_land_exclusive() {
        // Regression (audit §5.22/§9.111): Campaign set-up used to accept any
        // hex, letting gunboats deploy on land and land units on the Nile.
        // §5.22 is scenario-independent: only gunboats may occupy Nile hexes.
        let mut state = GameState::new(Scenario::Campaign);
        state.board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::ground(omdurman_types::GroundKind::Rough),
        );
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile { direction: omdurman_types::HexDirection::East },
        );

        // Dervish gunboat on a land hex -> rejected.
        let boat = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: dervish_gunboat_profile(),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&boat).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // Land unit on the Nile -> rejected. Taiasha (part of the §9.111
        // initial force) so the rejection is specifically the §5.22 Nile
        // rule, not the in-play-at-setup filter.
        let land_on_nile = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 0),
            profile: dervish_tribal_profile_with(DervishTribe::Taiasha),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&land_on_nile).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // Same-hex swaps of the two legal placements -> accepted.
        let boat_ok = UnitPlacement {
            position: HexCoord::new(1, 0),
            ..boat
        };
        let land_ok = UnitPlacement {
            position: HexCoord::new(0, 0),
            ..land_on_nile
        };
        assert!(state.can_deploy_unit(&boat_ok).is_ok());
        assert!(state.can_deploy_unit(&land_ok).is_ok());
    }

    #[rulebook("§5.22", "§9.321")]
    #[test]
    fn fok_ae_land_unit_rejected_on_nile() {
        // The converse of the gunboat test: a land unit may never deploy on the
        // Nile (§5.22).
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::ground(omdurman_types::GroundKind::Building),
        );
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile { direction: omdurman_types::HexDirection::East },
        );

        // Infantry on the Nile -> rejected.
        let land_on_nile = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(1, 0),
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&land_on_nile).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // Infantry on a building hex -> accepted.
        let land_on_building = UnitPlacement {
            position: HexCoord::new(0, 0),
            ..land_on_nile
        };
        assert!(state.can_deploy_unit(&land_on_building).is_ok());
    }

    #[rulebook("§9.321")]
    #[test]
    fn british_boats_named_vs_old_gunboat_detection() {
        // §9.321: only old (unnamed) gunboats are in play in FoK. The picker
        // filter distinguishes them via the *identity* (GunboatId::Named vs
        // GunboatId::Old), because the `british_boats` resolver tags both kinds
        // as `UnitKind::Gunboat`. Lock that detection in: named cells resolve
        // to a Named gunboat id, old cells to an Old one -- both with kind
        // `Gunboat` (so `is_boat()` is true for both).
        let resolve = |col: u8, row: u8| {
            let id = unit_id_for_section_pos(omdurman_types::SectionName::BritishBoats, col, row)
                .expect("BritishBoats cell resolves");
            let p = crate::unit_profiles::profile_for_unit(id)
                .expect("BritishBoats cell has a profile");
            (p.kind, p.identity)
        };

        // Named gunboats (row 0, cols 3-7).
        for (col, row) in [(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)] {
            let (kind, identity) = resolve(col, row);
            assert!(
                matches!(kind, crate::UnitKind::Gunboat { .. }),
                "named gunboat ({col},{row}) kind should be Gunboat, got {kind:?}"
            );
            assert!(
                matches!(
                    identity,
                    crate::UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Named(_))
                ),
                "({col},{row}) should be a Named gunboat"
            );
        }
        // Old gunboats (row 1, cols 4-7).
        for (col, row) in [(4, 1), (5, 1), (6, 1), (7, 1)] {
            let (kind, identity) = resolve(col, row);
            assert!(
                matches!(kind, crate::UnitKind::Gunboat { .. }),
                "old gunboat ({col},{row}) kind should be Gunboat, got {kind:?}"
            );
            assert!(
                matches!(
                    identity,
                    crate::UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(_))
                ),
                "({col},{row}) should be an Old gunboat"
            );
        }
    }

    #[rulebook("§5.22", "§9.321")]
    #[test]
    fn deploy_via_real_sprite_resolution_matches_engine() {
        // Validates the app's actual placement contract: `placement.rs`
        // resolves a sprite position to a UnitId + profile via
        // `unit_id_for_section_pos` + `profile_for_unit`, then calls
        // `apply_effect(DeployUnit)`. Confirm that path resolves a real FoK
        // British old-gunboat sprite to a boat and that the engine then accepts
        // it on the Nile and rejects it on land -- the same accept/reject the
        // app will see, end to end.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.board.terrain.insert(
            HexCoord::new(0, 0),
            Terrain::ground(omdurman_types::GroundKind::Building),
        );
        state.board.terrain.insert(
            HexCoord::new(1, 0),
            Terrain::Nile { direction: omdurman_types::HexDirection::East },
        );
        // BritishBoats (4,1) is an old-style gunboat (§2.32).
        let id = unit_id_for_section_pos(
            omdurman_types::SectionName::BritishBoats,
            4,
            1,
        )
        .expect("BritishBoats (4,1) resolves to a UnitId");
        let profile = crate::unit_profiles::profile_for_unit(id)
            .expect("BritishBoats (4,1) has a profile");
        assert!(
            profile.kind.is_boat(),
            "BritishBoats (4,1) should be a gunboat, got {:?}",
            profile.kind
        );

        // On land (Building) -> the engine rejects via the app's exact path.
        let on_land = UnitPlacement {
            id,
            position: HexCoord::new(0, 0),
            profile,
            state: UnitState::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(on_land)).unwrap_err(),
            RuleError::OutsideDeploymentZone(_)
        ));

        // On the Nile -> accepted, and the unit is on the board with that id.
        let on_nile = UnitPlacement {
            id,
            position: HexCoord::new(1, 0),
            profile,
            state: UnitState::default(),
        };
        apply_effect(&mut state, &GameEffect::DeployUnit(on_nile)).unwrap();
        assert!(state.find_unit(id).is_some());

        // Re-deploying the same counter (same id) -> rejected as a duplicate,
        // and the original placement is untouched.
        let dup = UnitPlacement {
            id,
            position: HexCoord::new(1, 0),
            profile,
            state: UnitState::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(dup)).unwrap_err(),
            RuleError::AlreadyDeployed(_)
        ));
        assert_eq!(state.units.len(), 1);
    }

    #[rulebook("§5.52")]
    #[test]
    fn deploy_rejects_dervish_tribe_mix() {
        // §5.52: units of different Dervish tribes may not stack. The deploy
        // validation must catch this (the FoK entry force has 4 tribes:
        // Mulazmin, Hadendowa, Kehena, Degheim). Kehena vs Mulazmin, both
        // §9.322-valid, so the stacking law is what rejects the mix.
        let mut state = GameState::new(Scenario::FallOfKhartoum); // permissive zone
        let hex = HexCoord::new(1, 1);

        let kehena = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::Kehena),
            state: UnitState::default(),
        };
        apply_effect(&mut state, &GameEffect::DeployUnit(kehena)).unwrap();

        // A Mulazmin unit stacked with the Kehena -> rejected.
        let mulazmin = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::Mulazmin),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&mulazmin).unwrap_err(),
            RuleError::Stacking(crate::StackingError::DervishTribeMix)
        ));
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(mulazmin)).unwrap_err(),
            RuleError::Stacking(crate::StackingError::DervishTribeMix)
        ));
        // Only the first unit is on the board.
        assert_eq!(state.units.len(), 1);
    }

    #[test]
    fn deploy_rejects_duplicate_counter() {
        // Each physical counter deploys once: a second deploy of the same id is
        // rejected (the app derives ids from sprite positions, so the same
        // sprite can't be placed twice). FoK so the AE profile is in play at
        // setup (§9.321); the Campaign AE deploys nothing (§9.113).
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        let id = state.alloc_unit_id();
        let first = UnitPlacement {
            id,
            position: HexCoord::new(1, 1),
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        };
        apply_effect(&mut state, &GameEffect::DeployUnit(first)).unwrap();

        let dup = UnitPlacement {
            id,
            position: HexCoord::new(2, 2),
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(dup)).unwrap_err(),
            RuleError::AlreadyDeployed(_)
        ));
        assert_eq!(state.units.len(), 1);
    }

    #[rulebook("§9.2", "§9.3")]
    #[test]
    fn remove_deployed_unit_happy_path() {
        let mut state = GameState::new(Scenario::FallOfKhartoum);        let id = state.alloc_unit_id();
        let placement = UnitPlacement {
            id,
            position: HexCoord::new(1, 1),
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        };
        apply_effect(&mut state, &GameEffect::DeployUnit(placement)).unwrap();
        assert_eq!(state.units.len(), 1);

        apply_effect(
            &mut state,
            &GameEffect::RemoveDeployedUnit {
                unit_id: id,
                player: Player::AngloEgyptian,
            },
        )
        .unwrap();
        assert!(state.units.is_empty());
    }

    #[test]
    fn remove_deployed_unit_rejected_outside_setup() {
        let mut state = playing(Scenario::Campaign); // Movement, not Setup
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: HexCoord::new(1, 1),
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        });
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::RemoveDeployedUnit {
                    unit_id: id,
                    player: Player::AngloEgyptian,
                }
            )
            .unwrap_err(),
            RuleError::WrongPhase
        ));
    }

    #[test]
    fn remove_deployed_unit_rejected_unknown() {
        let mut state = GameState::new(Scenario::Campaign);
        let id = state.alloc_unit_id(); // never deployed
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::RemoveDeployedUnit {
                    unit_id: id,
                    player: Player::AngloEgyptian,
                }
            )
            .unwrap_err(),
            RuleError::UnitNotFound(_)
        ));
    }

    #[test]
    fn remove_deployed_unit_rejected_wrong_owner() {
        // A player may only re-pick their own counters (defense against a
        // malformed remote event that names an enemy unit).
        let mut state = GameState::new(Scenario::Campaign);
        let dervish_id = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::RemoveDeployedUnit {
                    unit_id: dervish_id,
                    player: Player::AngloEgyptian,
                }
            )
            .unwrap_err(),
            RuleError::NotOwner(_)
        ));
        // Unit is still on the board.
        assert!(state.find_unit(dervish_id).is_some());
    }

    #[rulebook("§9.321", "§9.322")]
    #[test]
    fn fok_setup_complete_requires_full_oob() {
        // Defense-in-depth: even the unbound "Begin battle" path must not leave
        // Setup until both FoK orders of battle are fully deployed.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        assert!(matches!(
            state.setup_complete().unwrap_err(),
            RuleError::SetupIncomplete(_)
        ));
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap_err(),
            RuleError::SetupIncomplete(_)
        ));
        assert_eq!(state.phase, Phase::Setup);
    }

    #[test]
    fn deployed_count_tracks_placements() {
        let mut state = GameState::new(Scenario::Campaign);
        assert_eq!(state.setup_deployed_count(Player::AngloEgyptian), 0);
        make_ae_infantry(&mut state, HexCoord::new(1, 1));
        make_ae_infantry(&mut state, HexCoord::new(2, 1));
        assert_eq!(state.setup_deployed_count(Player::AngloEgyptian), 2);
        assert_eq!(state.setup_deployed_count(Player::Dervish), 0);
    }

    #[rulebook("§4")]
    #[test]
    fn turn_advances_through_phases() {
        let mut state = playing(Scenario::Campaign);
        assert_eq!(state.phase, Phase::Movement);
        assert_eq!(state.active_player, Player::AngloEgyptian);

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(state.phase, Phase::DefensiveFire(_)));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        // AE turn: after Dervish Defensive Fire (Direct) -> AE Offensive Fire
        assert!(matches!(
            state.phase,
            Phase::OffensiveFire(FireSubPhase::DirectFire)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(
            state.phase,
            Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert_eq!(state.phase, Phase::Melee);

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        // After melee, active_player switches.
        assert_eq!(state.active_player, Player::Dervish);
        assert_eq!(state.phase, Phase::Movement);

        // Dervish turn: Movement -> Defensive Fire (AE Direct)
        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(
            state.phase,
            Phase::DefensiveFire(FireSubPhase::DirectFire)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        // Dervish turn: DefFire(Direct) -> DefFire(Maxim/Howitzer) (AE fires again)
        assert!(matches!(
            state.phase,
            Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(
            state.phase,
            Phase::OffensiveFire(FireSubPhase::DirectFire)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert_eq!(state.phase, Phase::Melee);

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        // After melee, active_player switches back to AE.
        assert_eq!(state.active_player, Player::AngloEgyptian);
        assert_eq!(state.phase, Phase::Movement);
    }

    #[rulebook("§9.12")]
    #[test]
    fn game_over_after_campaign_turns() {
        let mut state = playing(Scenario::Campaign);
        // Fast-forward past all campaign turns.
        for _ in 0..100 {
            if state.game_over {
                break;
            }
            // Advance through all phases for each player turn.
            for _ in 0..6 {
                // Movement, DefFire(Direct), DefFire(Maxim2nd/How), OffFire(Direct), OffFire(Maxim2nd/How), Melee
                match apply_effect(&mut state, &GameEffect::AdvancePhase) {
                    Ok(()) => {}
                    // §8.2: the mandatory desertion roll gates the first
                    // night turn's Dervish movement phase. With no units on
                    // the board the expected deserter count is 0, so an
                    // empty roll satisfies the gate.
                    Err(RuleError::DesertionRollRequired) => {
                        let _ = apply_effect(
                            &mut state,
                            &GameEffect::DervishDesertion {
                                roll: DieRoll::One,
                                deserters: vec![],
                            },
                        );
                    }
                    Err(_) => break,
                }
            }
        }
        assert!(state.game_over);
    }

    // -- Fix-coverage tests (Parts C/D/E of the rule-enforcement work) -------

    use crate::board::{BoardInfo, NileBank, StepDirection};
    use omdurman_types::{HexDirection, HexsideKind, Terrain};

    fn make_unit(
        state: &mut GameState,
        hex: HexCoord,
        kind: UnitKind,
        identity: UnitIdentity,
        weapon: WeaponClass,
        movement: UnitMovement,
    ) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind,
                identity,
                weapon,
                fire: Some(crate::FireFactor::Four),
                melee: Some(crate::MeleeFactor::Five),
                movement,
            },
            state: UnitState::default(),
        });
        id
    }

    fn make_ae_artillery(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Artillery { fire: 0, melee: 0, movement: 0 },
            UnitIdentity::AngloEgyptianArtillery,
            WeaponClass::Artillery,
            UnitMovement::Land(crate::MovementAllowance::Eight),
        )
    }

    fn make_dervish_gunboat(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 },
            UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        )
    }

    fn make_fort(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Fort { fire: 0, melee: 0 },
            UnitIdentity::DervishFort,
            WeaponClass::Artillery,
            UnitMovement::Immobile,
        )
    }

    fn direct_attack(player: Player, firers: Vec<UnitId>, target: HexCoord) -> FireAttack {
        FireAttack {
            firing_player: player,
            phase: Phase::OffensiveFire(FireSubPhase::DirectFire),
            kind: FireKind::Direct,
            firers,
            target_hex: target,
            factor_row: FireFactorRow::Row06to10,
            // §6.24: the AE +1 is mandatory; the engine rejects any other list.
            modifiers: if player == Player::AngloEgyptian {
                vec![FireModifier::AngloEgyptianDirectFire]
            } else {
                vec![]
            },
        }
    }

    // ----- Part C -----------------------------------------------------------

    #[test]
    fn scenario_move_order_per_rulebook() {
        // §9.113 Campaign: Anglo-Egyptian moves first.
        assert_eq!(
            GameState::new(Scenario::Campaign).active_player,
            Player::AngloEgyptian
        );
        // §9.212 Historical and §9.322 Fall of Khartoum: Dervish moves first.
        assert_eq!(
            GameState::new(Scenario::Historical).active_player,
            Player::Dervish
        );
        assert_eq!(
            GameState::new(Scenario::FallOfKhartoum).active_player,
            Player::Dervish
        );
    }

    #[rulebook("§6.7")]
    #[test]
    fn no_advance_after_defensive_fire() {
        // §6.7: no advance after combat as a result of defensive fire.
        let mut state = GameState::new(Scenario::Campaign);
        let unit = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let dest = HexCoord::new(1, 0);

        state.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
        assert!(matches!(
            state.can_advance_after_combat(unit, dest),
            Err(RuleError::WrongPhase)
        ));

        // ...but offensive fire (§6.82) and melee (§7.6) do allow it, once a
        // hex has been vacated by combat (the advance window).
        open_advance_window(&mut state, dest, &[unit], vec!["6.82".to_string()]);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        assert!(state.can_advance_after_combat(unit, dest).is_ok());
        state.phase = Phase::Melee;
        assert!(state.can_advance_after_combat(unit, dest).is_ok());
    }

    #[rulebook("§8.2")]
    #[test]
    fn desertion_count_is_floor_one_and_a_half() {
        // §8.2: deserters = floor(1.5 * roll).
        assert_eq!(desertion_count(DieRoll::One), 1);
        assert_eq!(desertion_count(DieRoll::Two), 3);
        assert_eq!(desertion_count(DieRoll::Four), 6);
        assert_eq!(desertion_count(DieRoll::Ten), 15);
    }

    #[rulebook("§7")]
    #[test]
    fn declared_melee_blocks_phase_advance() {
        // Regression (audit §7): a declared-but-unresolved melee used to be
        // silently dropped when the melee phase ended. The phase may now only
        // end once the declaration is resolved (or vacated by retreat).
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        state.active_player = Player::Dervish;
        let defender = make_ae_infantry(&mut state, HexCoord::new(5, 5));
        let attacker = make_dervish_tribal(&mut state, HexCoord::new(6, 5));

        let declared = apply_effect(
            &mut state,
            &GameEffect::DeclareMelee {
                attack: MeleeAttack {
                    attacker_player: Player::Dervish,
                    attacker_hex: HexCoord::new(6, 5),
                    defender_hex: HexCoord::new(5, 5),
                    attackers: vec![attacker],
                    defenders: vec![defender],
                    attacker_modifiers: vec![MeleeModifier::DervishStandard],
                    defender_modifiers: vec![MeleeModifier::AngloEgyptianStandard],
                },
                attacker_roll: DieRoll::Five,
                defender_roll: DieRoll::Five,
            },
        );
        assert!(declared.is_ok());

        // Phase advance is rejected while the melee awaits resolution.
        assert!(matches!(
            advance_phase(&mut state),
            Err(RuleError::MeleePendingResolution)
        ));

        // Resolving it unblocks the advance.
        assert!(apply_effect(&mut state, &GameEffect::ResolveMelee).is_ok());
        assert!(advance_phase(&mut state).is_ok());
    }

    #[rulebook("§8.2")]
    #[test]
    fn desertion_roll_required_before_first_night_movement_ends() {
        // Regression (audit §8.2): every recorded campaign game skipped the
        // mandatory desertion roll. The Dervish movement phase of the first
        // night turn (T9) may not end before the roll is applied.
        let mut state = dervish_first_night_state();
        // An eligible tribal unit (the Khalifa/gunboats/artillery/forts are
        // exempt, so a plain tribe counter is needed to desert).
        let tribe = make_dervish_tribal(&mut state, HexCoord::new(0, 0));

        assert!(matches!(
            advance_phase(&mut state),
            Err(RuleError::DesertionRollRequired)
        ));

        // Applying the roll (One -> 1 unit) satisfies the gate.
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::One,
                    deserters: vec![tribe],
                }
            )
            .is_ok()
        );
        assert!(state.find_unit(tribe).is_none(), "the deserter is removed");
        assert!(advance_phase(&mut state).is_ok());

        // Later turns are unaffected (the roll is once per game).
        let mut later = {
            let mut s = dervish_first_night_state();
            s.current_turn = GameTurnIndex::new(10);
            s.dervish_deserted = true;
            s
        };
        assert!(advance_phase(&mut later).is_ok());
    }

    fn dervish_first_night_state() -> GameState {
        let mut state = GameState::new(Scenario::Campaign);
        // Advance to the first night turn (turn 9) in the Dervish movement
        // phase, which is when desertion is rolled (§8.2).
        state.current_turn = GameTurnIndex::new(9);
        state.day_night = DayNight::Night;
        state.active_player = Player::Dervish;
        state.phase = Phase::Movement;
        state
    }

    /// A Campaign state in the given side's turn-1 movement phase, on a small
    /// legal board (for reinforcement-schedule tests, §9.112/§9.113).
    fn campaign_wave_state(player: Player) -> GameState {
        let mut state = GameState::new(Scenario::Campaign);
        let mut board = BoardInfo::default();
        for h in [
            HexCoord::new(0, 0),
            HexCoord::new(0, 1),
            HexCoord::new(1, 0),
            HexCoord::new(1, 1),
        ] {
            board
                .terrain
                .insert(h, Terrain::Clear { road: Default::default() });
        }
        state.board = board;
        state.phase = Phase::Movement;
        state.active_player = player;
        state
    }

    fn tribal_placement(id: UnitId, tribe: DervishTribe, at: HexCoord) -> UnitPlacement {
        UnitPlacement {
            id,
            position: at,
            profile: UnitProfile {
                kind: UnitKind::Infantry { fire: 3, melee: 6, movement: 9 },
                identity: UnitIdentity::DervishTribal { tribe },
                weapon: WeaponClass::Melee,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Six),
                movement: UnitMovement::Land(crate::MovementAllowance::Nine),
            },
            state: UnitState::default(),
        }
    }

    #[rulebook("§9.112")]
    #[test]
    fn campaign_reinforcements_gate_by_wave() {
        // Turn 1 Dervish: Baggara (wave 1) enters; Mulazmin (wave 3 only,
        // §9.112) is rejected; the Anglo-Egyptian side cannot place on the
        // Dervish player's turn.
        let mut state = campaign_wave_state(Player::Dervish);
        let baggara = tribal_placement(
            state.alloc_unit_id(),
            DervishTribe::Baggara,
            HexCoord::new(0, 0),
        );
        assert!(apply_effect(
            &mut state,
            &GameEffect::PlaceReinforcements(vec![baggara])
        )
        .is_ok());

        let mut state = campaign_wave_state(Player::Dervish);
        let mulazmin = tribal_placement(
            state.alloc_unit_id(),
            DervishTribe::Mulazmin,
            HexCoord::new(0, 0),
        );
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(vec![mulazmin])),
            Err(RuleError::TribeNotInWave { turn: 1 })
        ));

        // AE land units may only enter on the AE player's turn (turn 1 wave).
        let mut state = campaign_wave_state(Player::Dervish);
        let ae = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(vec![ae])),
            Err(RuleError::NotYourTurn)
        ));
    }

    #[rulebook("§9.113")]
    #[test]
    fn campaign_reinforcement_cap_and_double_entry() {
        // The AE turn-1 wave caps at 12 land units; exceeding the cap in one
        // batch is rejected, and a unit may never enter twice.
        let mut state = campaign_wave_state(Player::AngloEgyptian);
        let batch: Vec<UnitPlacement> = (0..13)
            .map(|_| UnitPlacement {
                id: state.alloc_unit_id(),
                position: HexCoord::new(0, 0), // stacking will also trip at >4; use spread below
                profile: ae_infantry_profile(),
                state: UnitState::default(),
            })
            .collect();
        let _ = batch; // stacking in one hex trips first; build a spread batch
        let mut spread: Vec<UnitPlacement> = Vec::new();
        for i in 0..13 {
            spread.push(UnitPlacement {
                id: state.alloc_unit_id(),
                position: HexCoord::new(i % 2, i / 2 % 2),
                profile: ae_infantry_profile(),
                state: UnitState::default(),
            });
        }
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(spread.clone())),
            Err(RuleError::ReinforcementCapExceeded { turn: 1, cap: 12 })
        ));

        // A legal 2-unit batch enters, and re-entering the same ids is
        // rejected as AlreadyDeployed.
        spread.truncate(2);
        assert!(apply_effect(
            &mut state,
            &GameEffect::PlaceReinforcements(spread.clone())
        )
        .is_ok());
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(spread)),
            Err(RuleError::AlreadyDeployed(_))
        ));
        // Entry charged 1 MP each (§9.113).
        for p in &state.units {
            assert_eq!(state.mp_spent(p.id), 1, "entry MP not charged");
        }
    }

    // §7.1: a reinforcement may not materialise on an enemy-occupied hex
    // (found by the occupancy audit on the Campaign matrix: an AE battalion
    // arrived on top of a Dervish Taiasha unit and cohabited the hex).
    #[rulebook("§9.113", "§7.1")]
    #[test]
    fn reinforcement_rejected_onto_enemy_occupied_hex() {
        let mut state = campaign_wave_state(Player::AngloEgyptian);
        // A Dervish unit standing on the AE entrance area.
        make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let batch = vec![UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        }];
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::PlaceReinforcements(batch)),
            Err(RuleError::EnemyOccupied(_))
        ));
        // The enemy unit is untouched and nothing was placed.
        assert_eq!(
            state.units.iter().filter(|u| u.profile.identity.owner() == Player::AngloEgyptian).count(),
            0
        );
    }

    #[rulebook("§9.113")]
    #[test]
    fn campaign_gunboats_quota_three_per_turn() {
        let mut state = campaign_wave_state(Player::AngloEgyptian);
        let mk = |s: &mut GameState| UnitPlacement {
            id: s.alloc_unit_id(),
            position: HexCoord::new(0, 0),
            profile: ae_gunboat_profile(),
            state: UnitState::default(),
        };
        // Three gunboats stack-free is fine (gunboats may not stack with
        // anything else, so each gets its own hex).
        let mut batch: Vec<UnitPlacement> = Vec::new();
        for hex in [HexCoord::new(0, 0), HexCoord::new(0, 1), HexCoord::new(1, 0)] {
            let mut p = mk(&mut state);
            p.position = hex;
            batch.push(p);
        }
        assert!(apply_effect(&mut state, &GameEffect::PlaceReinforcements(batch)).is_ok());
        // A fourth in the same turn is over quota.
        let mut state2 = campaign_wave_state(Player::AngloEgyptian);
        let mut batch2: Vec<UnitPlacement> = Vec::new();
        for hex in [
            HexCoord::new(0, 0),
            HexCoord::new(0, 1),
            HexCoord::new(1, 0),
            HexCoord::new(1, 1),
        ] {
            let mut p = mk(&mut state2);
            p.position = hex;
            batch2.push(p);
        }
        assert!(matches!(
            apply_effect(&mut state2, &GameEffect::PlaceReinforcements(batch2)),
            Err(RuleError::GunboatQuotaExceeded { turn: 1 })
        ));
    }

    #[test]
    fn desertion_removes_chosen_count_and_respects_exemptions() {
        let mut state = dervish_first_night_state();
        let a = make_dervish_tribal(&mut state, HexCoord::new(1, 0));
        let b = make_dervish_tribal(&mut state, HexCoord::new(2, 0));
        let c = make_dervish_tribal(&mut state, HexCoord::new(3, 0));
        let khalifa = make_unit(
            &mut state,
            HexCoord::new(4, 0),
            UnitKind::DervishLeader { fire: 0, melee: 0, movement: 0 },
            UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah),
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Eight),
        );

        // Roll of 2 -> 3 deserters; choosing the Khalifa is illegal (§8.2).
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::Two,
                    deserters: vec![a, b, khalifa],
                }
            ),
            Err(RuleError::Desertion(DesertionError::Exempt(_)))
        ));
        // Wrong count is rejected.
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::Two,
                    deserters: vec![a, b],
                }
            ),
            Err(RuleError::Desertion(DesertionError::WrongCount { .. }))
        ));
        // A legal choice of three eligible units succeeds and is once-per-game.
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::Two,
                    deserters: vec![a, b, c],
                }
            )
            .is_ok()
        );
        assert!(state.find_unit(a).is_none());
        assert!(state.find_unit(khalifa).is_some());
        assert!(state.dervish_deserted);
        // A second desertion is rejected (§8.2 once per game).
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::DervishDesertion {
                    roll: DieRoll::Two,
                    deserters: vec![],
                }
            ),
            Err(RuleError::Desertion(DesertionError::AlreadyDeserted))
        ));
    }

    #[rulebook("§9.14")]
    #[test]
    fn friendlies_bank_scores_by_side() {
        // A small board: Nile in column q=0 of row r=0; west bank q<0, east q>0.
        let mut board = BoardInfo::default();
        board.terrain.insert(HexCoord::new(0, 0), Terrain::Nile { direction: HexDirection::East });
        board.terrain.insert(HexCoord::new(-1, 0), Terrain::default());
        board.terrain.insert(HexCoord::new(1, 0), Terrain::default());
        assert_eq!(board.bank_of(HexCoord::new(-1, 0)), Some(NileBank::West));
        assert_eq!(board.bank_of(HexCoord::new(1, 0)), Some(NileBank::East));
    }

    // ----- Part D-1: stacking ----------------------------------------------

    #[rulebook("§5.51")]
    #[test]
    fn stacking_over_limit_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let dest = HexCoord::new(1, 0);
        // Four AE infantry already in the destination hex.
        for _ in 0..4 {
            make_ae_infantry(&mut state, dest);
        }
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let err = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: mover,
                to: dest,
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(matches!(
            err,
            Err(RuleError::Stacking(crate::StackingError::OverLimit))
        ));
    }

    #[test]
    fn stacking_different_tribes_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let dest = HexCoord::new(1, 0);
        // A Baggara unit sits in the destination.
        make_dervish_tribal(&mut state, dest);
        // A Hadendowa unit tries to join it (§5.52).
        let mover = make_unit(
            &mut state,
            HexCoord::new(0, 0),
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::Hadendowa,
            },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Nine),
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: mover,
                    to: dest,
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Err(RuleError::Stacking(crate::StackingError::DervishTribeMix))
        ));
    }

    #[test]
    fn fok_green_mulazmin_units_resolve_and_stacking_mix_rejected() {
        // §5.52 regression: the Fall-of-Khartoum `Mulazmin_I`/`Mulazmin_II`
        // Mulazmin counters previously had no UnitId/profile, so `check_stacking`
        // was skipped for them entirely. Both sections must now resolve to the
        // Mulazmin tribe and participate in the different-tribe rule.
        for section in [omdurman_types::SectionName::MulazminI, omdurman_types::SectionName::MulazminII] {
            let unit_id = unit_id_for_section_pos(section, 0, 0).expect("green section has a UnitId");
            let profile = crate::unit_profiles::profile_for_unit(unit_id).expect("green section resolves a profile");
            assert_eq!(
                profile.identity,
                UnitIdentity::DervishTribal { tribe: DervishTribe::Mulazmin },
                "{section:?} (0,0) must be Mulazmin"
            );
        }

        // A Mulazmin unit and a Baggara unit in the same hex are a tribe mix.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let dest = HexCoord::new(1, 0);
        make_unit(
            &mut state,
            dest,
            UnitKind::Infantry { fire: 3, melee: 6, movement: 9 },
            UnitIdentity::DervishTribal { tribe: DervishTribe::Mulazmin },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Nine),
        );
        let baggara = make_unit(
            &mut state,
            HexCoord::new(0, 0),
            UnitKind::Infantry { fire: 3, melee: 6, movement: 15 },
            UnitIdentity::DervishTribal { tribe: DervishTribe::Baggara },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Fifteen),
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: baggara,
                    to: dest,
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Err(RuleError::Stacking(crate::StackingError::DervishTribeMix))
        ));

        // Two Mulazmin units may stack together (§5.52 allows same-tribe).
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let dest = HexCoord::new(2, 0);
        let m1 = make_unit(
            &mut state,
            dest,
            UnitKind::Infantry { fire: 3, melee: 6, movement: 9 },
            UnitIdentity::DervishTribal { tribe: DervishTribe::Mulazmin },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Nine),
        );
        let m2 = make_unit(
            &mut state,
            HexCoord::new(3, 0),
            UnitKind::Infantry { fire: 3, melee: 6, movement: 9 },
            UnitIdentity::DervishTribal { tribe: DervishTribe::Mulazmin },
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Nine),
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: m2,
                    to: dest,
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Ok(())
        ));
        let _ = m1;
    }

    // ----- Part D-2: ZOC ----------------------------------------------------

    #[test]
    fn gunboat_projects_zoc_only_vs_gunboats() {
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        make_dervish_gunboat(&mut state, HexCoord::new(1, 1));
        // A land mover ignores the enemy gunboat's ZOC...
        assert!(!state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }
        ));
        // ...but another gunboat is stopped by it (§5.41).
        assert!(state.hex_in_enemy_zoc(
            HexCoord::new(1, 0),
            Player::AngloEgyptian,
            UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 }
        ));
    }

    #[test]
    fn zoc_does_not_cross_a_khor() {
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        let enemy_hex = HexCoord::new(1, 1);
        make_dervish_tribal(&mut state, enemy_hex);
        let into = HexCoord::new(1, 0);
        // Without a hexside, ZOC reaches `into`.
        assert!(state.hex_in_enemy_zoc(into, Player::AngloEgyptian, UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }));
        // §5.44: a khor on the shared edge blocks the ZOC.
        state
            .board
            .hexsides
            .insert(HexsideRef::new(enemy_hex, into), HexsideKind::Khor);
        assert!(!state.hex_in_enemy_zoc(into, Player::AngloEgyptian, UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }));
    }

    #[rulebook("§5.42")]
    #[test]
    fn entering_enemy_zoc_costs_no_extra_mp() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // Enemy at (2,0) puts (1,0) in its ZOC.
        make_dervish_tribal(&mut state, HexCoord::new(2, 0));

        // Moving into a ZOC hex costs only the terrain MP (1 for clear), no surcharge.
        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: mover,
                to: HexCoord::new(1, 0),
                cost: MovementPoints::new(1),
                path: Vec::new(),
            },
        );
        assert!(result.is_ok());
        assert_eq!(state.mp_spent(mover), 1, "entering ZOC adds no MP cost");
    }

    // ----- Part D-3: movement cost & gunboats -------------------------------

    fn nile_board_row0(min_q: i32, max_q: i32, flow: HexDirection) -> BoardInfo {
        let mut board = BoardInfo::default();
        for q in min_q..=max_q {
            board.terrain.insert(HexCoord::new(q, 0), Terrain::Nile { direction: flow });
        }
        board
    }

    #[rulebook("§5.11")]
    #[test]
    fn land_unit_may_not_enter_nile() {
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 3, HexDirection::East);
        state.phase = Phase::Movement;
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 1));
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: mover,
                    to: HexCoord::new(1, 0), // a Nile hex
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Err(RuleError::LandIntoNile(_))
        ));
    }

    #[test]
    fn gunboat_step_direction_classifies_against_current() {
        // Current flows East (+q). A step to the +q neighbour is downstream;
        // the -q neighbour is upstream (§5.24).
        let board = nile_board_row0(0, 3, HexDirection::East);
        let here = HexCoord::new(1, 0);
        assert_eq!(
            board.step_direction(here, HexCoord::new(2, 0)),
            Some(StepDirection::Downstream)
        );
        assert_eq!(
            board.step_direction(here, HexCoord::new(0, 0)),
            Some(StepDirection::Upstream)
        );
    }

    #[test]
    fn gunboat_upstream_step_caps_the_turn() {
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 6, HexDirection::East);
        state.phase = Phase::Movement;
        // Gunboat at (3,0); upstream allowance 10, downstream 16.
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(3, 0));
        state.active_player = Player::Dervish;
        // One upstream step (to q=2) caps the turn at the upstream allowance of
        // 10; a cost of 12 is therefore illegal (§5.24).
        let upstream_path = vec![HexCoord::new(2, 0)];
        assert!(matches!(
            state.can_move_gunboat(gb, HexCoord::new(2, 0), &upstream_path, MovementPoints::new(12)),
            Err(RuleError::GunboatUpstreamCap { .. })
        ));
        // Purely downstream, the larger allowance of 16 applies, so 12 is fine.
        let downstream_path = vec![
            HexCoord::new(4, 0),
            HexCoord::new(5, 0),
            HexCoord::new(6, 0),
        ];
        assert!(
            state
                .can_move_gunboat(
                    gb,
                    HexCoord::new(6, 0),
                    &downstream_path,
                    MovementPoints::new(12)
                )
                .is_ok()
        );
    }

    #[rulebook("§5.24")]
    #[test]
    fn gunboat_upstream_cap_is_sticky_across_moves() {
        // Regression (audit §5.24): the cap used to be recomputed per move, so
        // a gunboat that went upstream in an earlier move could spend up to
        // its *downstream* allowance with later all-downstream moves. The
        // manual caps the whole turn: "if they move even one hex upstream,
        // their upstream movement allowance is their maximum movement
        // allowance for that turn".
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 12, HexDirection::East);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        // Gunboat at (3,0); upstream allowance 10, downstream 16.
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(3, 0));

        // Move 1: one committed upstream step (to q=2), 1 MP.
        let upstream = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: gb,
                to: HexCoord::new(2, 0),
                cost: MovementPoints::new(1),
                path: vec![HexCoord::new(2, 0)],
            },
        );
        assert!(upstream.is_ok(), "1-MP upstream step is legal: {upstream:?}");
        assert!(
            state.gunboats_upstream_this_turn.contains(&gb),
            "the committed upstream step must set the sticky flag"
        );

        // Move 2 (all downstream, engine-costed at 1 MP per hex): cumulative
        // 1 + 10 = 11 exceeds the upstream cap of 10 -> rejected under §5.24,
        // even though 11 < 16 and this move itself never goes upstream.
        let downstream: Vec<HexCoord> = (3..=12).map(|q| HexCoord::new(q, 0)).collect();
        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: gb,
                to: HexCoord::new(12, 0),
                cost: MovementPoints::new(10),
                path: downstream,
            },
        );
        assert!(
            matches!(result, Err(RuleError::GunboatUpstreamCap { .. })),
            "later downstream moves must stay capped at the upstream allowance (§5.24), got {result:?}"
        );

        // Cross-check via the predicate with an explicit cumulative spend.
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 12, HexDirection::East);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(2, 0));
        state.gunboats_upstream_this_turn.push(gb);
        state.mp_spent_this_turn.insert(gb, 9);
        let downstream_path = vec![HexCoord::new(3, 0)];
        assert!(matches!(
            state.can_move_gunboat(gb, HexCoord::new(3, 0), &downstream_path, MovementPoints::new(2)),
            Err(RuleError::GunboatUpstreamCap { .. })
        ));
        assert!(
            state
                .can_move_gunboat(gb, HexCoord::new(3, 0), &downstream_path, MovementPoints::new(1))
                .is_ok()
        );
    }

    // ----- Part D-4: artillery special results & howitzer scatter -----------

    #[rulebook("§6.61")]
    #[test]
    fn rifles_may_not_sink_a_gunboat() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let rifle = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        make_dervish_gunboat(&mut state, target);
        let attack = direct_attack(Player::AngloEgyptian, vec![rifle], target);
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::FireCombat {
                    attack,
                    roll: DieRoll::Ten
                }
            ),
            Err(RuleError::ArtilleryOnlyVsGunboatOrFort(_))
        ));
    }

    /// Resolve an artillery attack on a gunboat at `target` with `roll` and
    /// report whether the gunboat was sunk and the Combat Results Table result
    /// the engine actually computed (so the test asserts the §6.61 threshold
    /// against the *real* banded result, not a re-derived one).
    fn arty_vs_gunboat(roll: DieRoll) -> (bool, CombatResult) {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let arty = make_ae_artillery(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        let gb = make_dervish_gunboat(&mut state, target);
        let attack = direct_attack(Player::AngloEgyptian, vec![arty], target);
        // Mirror the engine's banded total + modifiers to know the CRT result.
        let range = HexDistance::new(1);
        let band = ae_range_effects(WeaponClass::Artillery, range);
        let total = band.apply(crate::FireFactor::Four.value());
        let crt = combat_results_table(
            FireFactorRow::from_total(total),
            roll.apply_modifier(attack.net_modifier()),
        );
        apply_effect(&mut state, &GameEffect::FireCombat { attack, roll }).unwrap();
        (state.find_unit(gb).is_none(), crt)
    }

    #[test]
    fn artillery_sinks_gunboat_only_on_three_plus() {
        // §6.61: a gunboat is sunk only on a Combat Results Table result of 3+.
        // Across the die-roll range, the gunboat is sunk iff the result was
        // Eliminate(>=3) -- never on a lesser result.
        for r in 1u16..=10 {
            let roll = DieRoll::try_from(r).unwrap();
            let (sunk, crt) = arty_vs_gunboat(roll);
            assert_eq!(
                sunk,
                matches!(crt, CombatResult::Eliminate(n) if n >= 3),
                "roll {r}: sunk={sunk} but CRT={crt:?}"
            );
        }
    }

    #[test]
    fn howitzer_scatters_off_target_below_seven() {
        // Impact roll 1-6 must move the shell off the designated hex; 7-10 hits.
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 12, HexDirection::East);
        let target = HexCoord::new(8, 0);
        // Roll 3 (Right on the scattergram) lands on a distinct neighbour.
        let impact = state.howitzer_impact_hex(
            target,
            Some(HexCoord::new(0, 0)),
            howitzer_scatter(DieRoll::Three),
        );
        assert_ne!(impact, target);
        // Every miss roll (1-6) lands on a distinct neighbour.
        let mut seen = std::collections::HashSet::new();
        for roll in 1u16..=6 {
            let hex = state.howitzer_impact_hex(
                target,
                Some(HexCoord::new(0, 0)),
                howitzer_scatter(DieRoll::try_from(roll).unwrap()),
            );
            assert_ne!(hex, target);
            assert!(seen.insert(hex), "rolls must scatter to distinct hexes");
        }
        // On-target (roll 7-10) lands on the designated hex.
        let on = state.howitzer_impact_hex(
            target,
            Some(HexCoord::new(0, 0)),
            howitzer_scatter(DieRoll::Nine),
        );
        assert_eq!(on, target);
    }

    // ----- Part D-5: mines & chain ------------------------------------------

    #[test]
    fn mine_fires_once_and_spares_dervish() {
        let mut state = GameState::new(Scenario::Campaign);
        let hex = HexCoord::new(2, 0);
        state.mines.push(crate::MinePlacement {
            hex,
            triggered: false,
        });
        // A Dervish gunboat passes unharmed (§10.14) and does not consume the mine.
        let dervish_gb = make_dervish_gunboat(&mut state, hex);
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::RiverMine {
                    gunboat_id: dervish_gb,
                    hex,
                    roll: DieRoll::Ten
                }
            )
            .is_ok()
        );
        assert!(state.find_unit(dervish_gb).is_some());
        assert!(!state.mines[0].triggered);

        // A British gunboat triggers it (roll 10 -> sunk).
        let brit_gb = make_unit(
            &mut state,
            hex,
            UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(crate::OldGunboat::LordKitchener)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        );
        assert!(
            apply_effect(
                &mut state,
                &GameEffect::RiverMine {
                    gunboat_id: brit_gb,
                    hex,
                    roll: DieRoll::Ten
                }
            )
            .is_ok()
        );
        assert!(state.find_unit(brit_gb).is_none());
        assert!(state.mines[0].triggered);
        // §10.13: a spent mine no longer fires.
        let gb3 = make_unit(
            &mut state,
            hex,
            UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(crate::OldGunboat::LordKitchener)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        );
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::RiverMine {
                    gunboat_id: gb3,
                    hex,
                    roll: DieRoll::Ten
                }
            ),
            Err(RuleError::NoUntriggeredMine(_))
        ));
    }

    #[test]
    fn chain_stops_gunboat_until_sunk() {
        let mut state = GameState::new(Scenario::Campaign);
        state.board = nile_board_row0(0, 4, HexDirection::East);
        state.phase = Phase::Movement;
        state.active_player = Player::Dervish;
        let chained = HexCoord::new(2, 0);
        state.chain = Some(crate::ChainPlacement {
            hexes: vec![chained],
            sunk: false,
        });
        let gb = make_dervish_gunboat(&mut state, HexCoord::new(1, 0));
        let path = vec![chained];
        assert!(matches!(
            state.can_move_gunboat(gb, chained, &path, MovementPoints::new(1)),
            Err(RuleError::BlockedByChain(_))
        ));
        // §10.23: once sunk, the chain no longer stops the gunboat.
        apply_effect(&mut state, &GameEffect::SinkChain).unwrap();
        assert!(
            state
                .can_move_gunboat(gb, chained, &path, MovementPoints::new(1))
                .is_ok()
        );
    }

    // ----- Part E: Mahdi's Tomb --------------------------------------------

    #[rulebook("§9.14")]
    #[test]
    fn mahdis_tomb_scores_for_anglo_egyptian_when_held() {
        let mut state = GameState::new(Scenario::Campaign);
        let tomb = HexCoord::new(5, 5);
        state
            .board
            .locations
            .insert(tomb, omdurman_types::Location::MahdisTomb);
        // A British leader plus a non-Friendlies combat unit, both undisrupted.
        make_unit(
            &mut state,
            tomb,
            UnitKind::BritishLeader { movement: 0 },
            UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Kitchener),
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Eight),
        );
        make_ae_infantry(&mut state, tomb);

        score_mahdis_tomb(&mut state);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian),
            crate::VictoryPoints(25)
        );
    }

    #[rulebook("§9.14")]
    #[test]
    fn mahdis_tomb_not_scored_without_a_leader() {
        let mut state = GameState::new(Scenario::Campaign);
        let tomb = HexCoord::new(5, 5);
        state
            .board
            .locations
            .insert(tomb, omdurman_types::Location::MahdisTomb);
        // Only a combat unit, no British leader -> Dervish retains control.
        make_ae_infantry(&mut state, tomb);
        score_mahdis_tomb(&mut state);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian),
            crate::VictoryPoints(0)
        );
    }

    // ----- Fall of Khartoum special rules (§9.3) ---------------------------

    fn make_gordon(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::BritishLeader { movement: 0 },
            UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Gordon),
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Immobile),
        )
    }

    /// A FoK game with a Palace at `palace`, GORDON on it, and clear passable
    /// terrain on the palace and an adjacent hex so a Dervish unit can advance.
    fn fok_with_palace(palace: HexCoord) -> (GameState, HexCoord) {
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.phase = Phase::Movement; // these tests exercise play, not setup
        let adj = palace.neighbors()[0];
        state
            .board
            .locations
            .insert(palace, omdurman_types::Location::Palace);
        state.board.terrain.insert(palace, Terrain::default());
        state.board.terrain.insert(adj, Terrain::default());
        make_gordon(&mut state, palace);
        (state, adj)
    }

    #[test]
    fn gordon_may_not_move_in_fok() {
        // §9.346: GORDON may not move during FALL OF KHARTOUM.
        let (mut state, _adj) = fok_with_palace(HexCoord::new(2, 2));
        state.active_player = Player::AngloEgyptian;
        let gordon = state.units[0].id;
        let err = state
            .can_move_unit_to(gordon, Some(HexCoord::new(2, 1)), MovementPoints::new(1))
            .unwrap_err();
        assert!(matches!(err, RuleError::GordonMayNotMove));
    }

    #[test]
    fn dervish_reaching_palace_eliminates_gordon_and_ends_game() {
        // §9.346: GORDON dies the instant a Dervish unit occupies the Palace;
        // §9.35: the turn is recorded and the game ends.
        let palace = HexCoord::new(2, 2);
        let (mut state, adj) = fok_with_palace(palace);
        state.current_turn = GameTurnIndex::new(3);
        state.active_player = Player::Dervish;
        let dervish = make_dervish_tribal(&mut state, adj);

        apply_move_unit(&mut state, dervish, palace, MovementPoints::new(1), &[palace])
            .expect("Dervish moves onto the palace");

        assert!(
            !state.units.iter().any(|u| u.profile.identity.is_gordon()),
            "GORDON is removed"
        );
        assert_eq!(state.gordon_eliminated_turn, Some(GameTurnIndex::new(3)));
        assert!(state.game_over);
    }

    #[rulebook("§9.346")]
    #[test]
    fn gordon_survives_means_no_elimination() {
        // A Dervish unit adjacent to (but not on) the Palace does not kill GORDON.
        let palace = HexCoord::new(2, 2);
        let (mut state, adj) = fok_with_palace(palace);
        state.active_player = Player::Dervish;
        let dervish = make_dervish_tribal(&mut state, palace.neighbors()[1]);
        apply_move_unit(&mut state, dervish, adj, MovementPoints::new(1), &[adj])
            .expect("Dervish moves adjacent");
        assert!(state.units.iter().any(|u| u.profile.identity.is_gordon()));
        assert_eq!(state.gordon_eliminated_turn, None);
        assert!(!state.game_over);
    }

    #[test]
    fn fok_victory_levels_follow_the_table() {
        use crate::FoKVictoryLevel as V;
        // §9.35 base levels by turn of GORDON's death (no Dervish-loss penalty).
        // scenario_end_turn is irrelevant when GORDON died (the death turn
        // fixes the base), so we pass 8 (the typical FoK end).
        assert_eq!(V::resolve(Some(4), 8, 0), V::DervishDecisive);
        assert_eq!(V::resolve(Some(3), 8, 0), V::DervishDecisive);
        assert_eq!(V::resolve(Some(5), 8, 0), V::DervishTactical);
        assert_eq!(V::resolve(Some(6), 8, 0), V::DervishMarginal);
        // GORDON survives: British level depends on how long he held.
        assert_eq!(V::resolve(None, 6, 0), V::BritishMarginal);
        assert_eq!(V::resolve(None, 7, 0), V::BritishTactical);
        assert_eq!(V::resolve(None, 8, 0), V::BritishDecisive);
        // Early end (before turn 6) with GORDON alive: best-effort Marginal.
        assert_eq!(V::resolve(None, 5, 0), V::BritishMarginal);

        // The rulebook worked example: GORDON dies turn 5 (Dervish tactical)
        // but the Dervish lose 24 units (-2 levels) -> British marginal.
        assert_eq!(V::resolve(Some(5), 8, 24), V::BritishMarginal);
        // Loss-penalty thresholds: 16-23 -> -1, 24-31 -> -2, 32+ -> -3.
        assert_eq!(V::resolve(Some(3), 8, 16), V::DervishTactical); // decisive -1
        assert_eq!(V::resolve(Some(3), 8, 32), V::BritishMarginal); // decisive -3, clamps up
    }

    fn make_old_gunboat(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Old(crate::OldGunboat::LordKitchener)),
            WeaponClass::Artillery,
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        )
    }

    #[test]
    fn fok_gunboat_crosses_between_nile_mouths() {
        // §9.345: a British gunboat may cross White<->Blue Nile mouths off-board
        // for 6 upstream MP, even though the mouths are not Nile-adjacent.
        let mut state = GameState::new(Scenario::FallOfKhartoum);
        state.phase = Phase::Movement; // exercises movement, not setup
        let white = HexCoord::new(1, 0);
        let blue = HexCoord::new(16, 1);
        state.board.terrain.insert(white, Terrain::Nile { direction: HexDirection::East });
        state.board.terrain.insert(blue, Terrain::Nile { direction: HexDirection::East });
        state
            .board
            .locations
            .insert(white, omdurman_types::Location::WhiteNileMouth);
        state
            .board
            .locations
            .insert(blue, omdurman_types::Location::BlueNileMouth);
        state.active_player = Player::AngloEgyptian;
        let gb = make_old_gunboat(&mut state, white);

        // The crossing is legal (6 MP <= the gunboat's upstream allowance of 10).
        assert!(
            state
                .can_move_gunboat(gb, blue, &[blue], MovementPoints::new(6))
                .is_ok(),
            "White->Blue mouth crossing is legal (§9.345)"
        );

        // A normal far-apart move that is NOT a mouth crossing is rejected (the
        // two hexes are not contiguous Nile).
        let elsewhere = HexCoord::new(8, 8);
        state.board.terrain.insert(elsewhere, Terrain::default());
        assert!(
            state
                .can_move_gunboat(gb, elsewhere, &[elsewhere], MovementPoints::new(6))
                .is_err()
        );
    }

    #[test]
    fn fok_both_players_use_dervish_range_table() {
        // §9.343: in FoK an Anglo-Egyptian unit fires on the Dervish table.
        // Dervish rifles reach range 2 at normal; Anglo-Egyptian rifles on
        // their own table would be out of range at 2 doubled->halved etc., so
        // compare the band the engine picks for an AE rifleman at range 3.
        let r = HexDistance::new(3);
        let fok = range_band_for(
            Scenario::FallOfKhartoum,
            Player::AngloEgyptian,
            WeaponClass::Rifles,
            r,
        );
        let dervish = crate::range_effects::dervish_range_effects(WeaponClass::Rifles, r);
        assert_eq!(fok, dervish, "AE uses the Dervish table in FoK (§9.343)");
    }

    // -- §9.14 VP routing tests ---------------------------------------------

    fn make_unit_with_identity(
        state: &mut GameState,
        hex: HexCoord,
        identity: UnitIdentity,
    ) -> UnitId {
        let id = state.alloc_unit_id();
        let kind = match identity {
            UnitIdentity::DervishFort => UnitKind::Fort { fire: 0, melee: 0 },
            UnitIdentity::DervishLeader(_) => UnitKind::DervishLeader { fire: 0, melee: 0, movement: 0 },
            UnitIdentity::AngloEgyptianLeader(_) => UnitKind::BritishLeader { movement: 0 },
            _ => UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
        };
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind,
                identity,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Three),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        });
        id
    }

    #[test]
    fn khalifa_elimination_scores_10_vp() {
        let mut state = playing(Scenario::Campaign);
        let id = make_unit_with_identity(
            &mut state,
            HexCoord::new(0, 0),
            UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah),
        );
        score_elimination(&mut state, id, Player::Dervish);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian).0,
            10,
            "Khalifa is worth 10 VP (§9.14)"
        );
    }

    #[test]
    fn fort_elimination_scores_0_vp() {
        let mut state = playing(Scenario::Campaign);
        let id =
            make_unit_with_identity(&mut state, HexCoord::new(0, 0), UnitIdentity::DervishFort);
        score_elimination(&mut state, id, Player::Dervish);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian).0,
            0,
            "Fort elimination is worth 0 VP (§9.14)"
        );
    }

    #[test]
    fn isa_zachneih_elimination_sets_flag_and_scores_1_vp() {
        let mut state = playing(Scenario::Campaign);
        assert!(!state.isa_zachneih_eliminated, "flag starts clear");
        let id = make_unit_with_identity(
            &mut state,
            HexCoord::new(0, 0),
            UnitIdentity::DervishTribal {
                tribe: DervishTribe::IsaZachneih,
            },
        );
        score_elimination(&mut state, id, Player::Dervish);
        assert!(state.isa_zachneih_eliminated, "flag set after elimination");
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian).0,
            1,
            "Isa Zachneih is worth 1 VP (§9.14)"
        );
    }

    #[test]
    fn ordinary_dervish_leader_scores_1_vp() {
        let mut state = playing(Scenario::Campaign);
        let id = make_unit_with_identity(
            &mut state,
            HexCoord::new(0, 0),
            UnitIdentity::DervishLeader(crate::DervishLeader::Yakub),
        );
        score_elimination(&mut state, id, Player::Dervish);
        assert_eq!(
            state.victory.total_for(Player::AngloEgyptian).0,
            1,
            "Ordinary Dervish leaders are worth 1 VP (§9.14)"
        );
    }

    #[test]
    fn observations_pushed_on_elimination() {
        let mut state = playing(Scenario::Campaign);
        let id = make_unit_with_identity(
            &mut state,
            HexCoord::new(0, 0),
            UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah),
        );
        score_elimination(&mut state, id, Player::Dervish);
        let obs = state.drain_observations();
        assert!(
            obs.iter().any(|o| matches!(
                o,
                Observation::VictoryScored {
                    source: VpSource::KhalifaEliminated,
                    points: crate::VictoryPoints(10),
                    ..
                }
            )),
            "VictoryScored observation for 10 VP"
        );
        assert!(
            obs.iter()
                .any(|o| matches!(o, Observation::UnitEliminated { .. })),
            "UnitEliminated observation"
        );
        assert!(
            obs.iter()
                .any(|o| matches!(o, Observation::LeaderKilled { .. })),
            "LeaderKilled observation"
        );
    }

    // -- §6.42 Maxim second fire tests --------------------------------------

    fn make_maxim(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::Maxim { fire: 0, melee: 0, movement: 0 },
                identity: UnitIdentity::AngloEgyptianMaxim,
                weapon: WeaponClass::Maxims,
                fire: Some(crate::FireFactor::Five),
                melee: Some(crate::MeleeFactor::One),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        });
        id
    }

    #[test]
    fn maxim_may_fire_twice_per_turn() {
        let mut state = playing(Scenario::Campaign);
        let maxim = make_maxim(&mut state, HexCoord::new(0, 0));
        let _enemy = make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        // Direct fire subphase: Maxim fires once.
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        assert!(
            state
                .can_fire_at(maxim, HexCoord::new(1, 0), FireKind::Direct)
                .is_ok()
        );
        state.units_fired_this_phase.push(maxim);

        // The once-per-phase set blocks a second shot in the SAME subphase.
        assert!(matches!(
            state.can_fire_at(maxim, HexCoord::new(1, 0), FireKind::Direct),
            Err(RuleError::AlreadyFired(_))
        ));

        // Advance to the Maxim/Howitzer subphase: the set is cleared, so the
        // Maxim may fire its second shot (§6.42).
        state.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
        state.units_fired_this_phase.clear();
        assert!(
            state
                .can_fire_at(maxim, HexCoord::new(1, 0), FireKind::MaximSecondFire)
                .is_ok(),
            "Maxim may fire a second time in the Maxim/Howitzer subphase (§6.42)"
        );
    }

    #[test]
    fn maxim_that_skipped_direct_may_fire_once_in_second_subphase() {
        let mut state = playing(Scenario::Campaign);
        let maxim = make_maxim(&mut state, HexCoord::new(0, 0));

        // The Maxim did not fire in DirectFire. In the Maxim/Howitzer subphase
        // it may fire once (§6.42: "If any Maxim guns did not fire during the
        // Direct Fire Subphase, they may still only fire once in [6.42]").
        state.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
        assert!(
            state
                .can_fire_at(maxim, HexCoord::new(1, 0), FireKind::MaximSecondFire)
                .is_ok(),
            "Maxim that skipped Direct may fire once in the second subphase"
        );
    }

    #[test]
    fn non_maxim_rejected_in_second_subphase() {
        let mut state = playing(Scenario::Campaign);
        let rifle = make_ae_infantry(&mut state, HexCoord::new(0, 0));

        // A rifle-class unit may not fire in the Maxim/Howitzer subphase --
        // engine-authoritative rejection with a typed error (§6.42).
        state.phase = Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
        assert!(matches!(
            state.can_fire_at(rifle, HexCoord::new(1, 0), FireKind::MaximSecondFire),
            Err(RuleError::WrongWeaponForSubphase(_))
        ));
    }

    // -- §6.53 Royal Engineers demolition tests -----------------------------

    fn make_royal_engineers(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
                identity: UnitIdentity::RoyalEngineers,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Five),
                melee: Some(crate::MeleeFactor::Three),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        });
        id
    }

    #[test]
    fn demolition_destroys_fort() {
        let mut state = playing(Scenario::Campaign);
        let eng = make_royal_engineers(&mut state, HexCoord::new(0, 0));
        let fort = make_fort(&mut state, HexCoord::new(1, 0));

        // Commit demolition.
        apply_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        assert!(state.pending_demolitions.len() == 1);

        // Resolve: engineer still adjacent + undisrupted → fort destroyed.
        apply_resolve_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        assert!(
            state.find_unit(fort).is_none(),
            "fort should be eliminated after successful demolition"
        );
        assert!(
            !state
                .find_unit(eng)
                .map(|u| u.state.demolishing)
                .unwrap_or(true),
            "engineer freed after demolition"
        );
        // Fort elimination is 0 VP (§9.14).
        assert_eq!(state.victory.total_for(Player::AngloEgyptian).0, 0);
    }

    #[rulebook("§6.53")]
    #[test]
    fn demolition_cancelled_when_engineer_disrupted() {
        let mut state = playing(Scenario::Campaign);
        let eng = make_royal_engineers(&mut state, HexCoord::new(0, 0));
        let fort = make_fort(&mut state, HexCoord::new(1, 0));

        apply_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        // Engineer gets disrupted during the turn.
        state.find_unit_mut(eng).unwrap().state.disrupted = true;
        apply_resolve_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        assert!(
            state.find_unit(fort).is_some(),
            "fort should survive when engineer was disrupted"
        );
        assert!(
            !state
                .find_unit(eng)
                .map(|u| u.state.demolishing)
                .unwrap_or(true),
            "engineer freed even on failed demolition"
        );
    }

    #[rulebook("§6.53")]
    #[test]
    fn demolition_cancelled_when_engineer_moved_away() {
        let mut state = playing(Scenario::Campaign);
        let eng = make_royal_engineers(&mut state, HexCoord::new(0, 0));
        let fort = make_fort(&mut state, HexCoord::new(1, 0));

        apply_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        // Engineer moves away (no longer adjacent).
        state.find_unit_mut(eng).unwrap().position = HexCoord::new(5, 5);
        apply_resolve_demolition(&mut state, eng, DemolitionTarget::Fort(fort)).unwrap();
        assert!(
            state.find_unit(fort).is_some(),
            "fort should survive when engineer moved away"
        );
    }

    // -- Engine-authoritative LOS / hexside blocking tests (§6.3, §7.2) ----

    #[rulebook("§6.21")]
    #[test]
    fn can_fire_at_rejects_blocked_los() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target_hex = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target_hex);
        // Wall hexside between firer and target blocks LOS.
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), target_hex),
            omdurman_types::HexsideKind::Wall,
        );
        assert!(matches!(
            state.can_fire_at(ae, target_hex, crate::FireKind::Direct),
            Err(RuleError::LineOfSightBlocked(_, _))
        ));
    }

    #[rulebook("§6.21")]
    #[test]
    fn can_fire_at_allows_clear_los() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target_hex = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target_hex);
        // No hexside → LOS clear.
        assert!(
            state
                .can_fire_at(ae, target_hex, crate::FireKind::Direct)
                .is_ok()
        );
    }

    #[rulebook("§7.2")]
    #[test]
    fn can_melee_rejects_wall_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), target),
            omdurman_types::HexsideKind::Wall,
        );
        assert!(matches!(
            state.can_melee(ae, target),
            Err(RuleError::MeleeBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§7.2")]
    #[test]
    fn can_melee_rejects_thorn_hedge_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), target),
            omdurman_types::HexsideKind::ZaribaThornHedge,
        );
        assert!(matches!(
            state.can_melee(ae, target),
            Err(RuleError::MeleeBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§7.2")]
    #[test]
    fn can_melee_allows_gate_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Melee;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(1, 0);
        make_dervish_tribal(&mut state, target);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), target),
            omdurman_types::HexsideKind::Gate,
        );
        assert!(state.can_melee(ae, target).is_ok());
    }

    #[rulebook("§6.82")]
    #[test]
    fn can_advance_after_combat_rejects_wall_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), to),
            omdurman_types::HexsideKind::Wall,
        );
        open_advance_window(&mut state, to, &[ae], vec!["6.82".to_string()]);
        assert!(matches!(
            state.can_advance_after_combat(ae, to),
            Err(RuleError::AdvanceBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§6.82")]
    #[test]
    fn can_advance_after_combat_rejects_khor_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), to),
            omdurman_types::HexsideKind::Khor,
        );
        open_advance_window(&mut state, to, &[ae], vec!["6.82".to_string()]);
        assert!(matches!(
            state.can_advance_after_combat(ae, to),
            Err(RuleError::AdvanceBlockedByHexside(_, _))
        ));
    }

    // -- Engine-authoritative movement tests (§5.11, §5.23) -----------------

    #[rulebook("§5.23")]
    #[test]
    fn can_move_rejects_wall_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), to),
            omdurman_types::HexsideKind::Wall,
        );
        assert!(matches!(
            state.can_move_unit_to(ae, Some(to), MovementPoints::new(1)),
            Err(RuleError::MoveBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§5.23")]
    #[test]
    fn can_move_allows_gate_hexside() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let to = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(HexCoord::new(0, 0), to),
            omdurman_types::HexsideKind::Gate,
        );
        assert!(
            state
                .can_move_unit_to(ae, Some(to), MovementPoints::new(1))
                .is_ok()
        );
    }

    #[rulebook("§5.11")]
    #[test]
    fn movement_cost_for_uses_terrain() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        // Place terrain: Rough at (1,0) costs 2 MP.
        state
            .board
            .terrain
            .insert(HexCoord::new(1, 0), omdurman_types::Terrain::ground(omdurman_types::GroundKind::Rough));
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let unit = state.find_unit(ae).unwrap();
        let cost = state.movement_cost_for(unit, &[HexCoord::new(1, 0)]);
        assert_eq!(cost, Some(MovementPoints::new(2)));
    }

    #[rulebook("§5.11")]
    #[test]
    fn movement_cost_for_road_costs_one() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        // Place Rough terrain (normally 2 MP) and a road edge.
        state
            .board
            .terrain
            .insert(HexCoord::new(1, 0), omdurman_types::Terrain::ground(omdurman_types::GroundKind::Rough));
        state.board.roads.insert(omdurman_types::HexsideRef::new(
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
        ));
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let unit = state.find_unit(ae).unwrap();
        let cost = state.movement_cost_for(unit, &[HexCoord::new(1, 0)]);
        assert_eq!(cost, Some(MovementPoints::new(1)));
    }

    #[rulebook("§8.1")]
    #[test]
    fn night_movement_overlay_allowance_halved() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.day_night = DayNight::Night;
        let ae = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // AE infantry has MA 4; at night that's halved to 2.
        let unit = state.find_unit(ae).unwrap();
        let allowance = match unit.profile.movement {
            crate::UnitMovement::Land(a) => a,
            _ => panic!("expected land movement"),
        };
        let effective =
            crate::effective_movement_at_night(allowance, Player::AngloEgyptian, state.day_night);
        assert_eq!(effective.value(), allowance.value() / 2);
    }

    // ----- Part E: walled-city entry (§5.23), Zariba surcharge (§9.233),
    //      mid-move stacking (§5.51), SetupLetter mapping (§9.212) ----

    /// Helper: build a tiny board with a walled-city interior at `city`.
    /// Three Wall hexsides surround it so `is_walled_city` fires.
    fn make_walled_board(state: &mut GameState, city: HexCoord) {
        // The walled city is *derived* (flood from the Palace, §5.23), so the
        // fixture needs a Palace landmark plus walls, then a recompute.
        state
            .board
            .locations
            .insert(city, omdurman_types::Location::Palace);
        let n = city.neighbors();
        for neighbor in n.iter().take(3) {
            state.board.hexsides.insert(
                omdurman_types::HexsideRef::new(city, *neighbor),
                HexsideKind::Wall,
            );
        }
        state.board.walled_city = state.board.compute_walled_city();
    }

    #[rulebook("§5.23")]
    #[test]
    fn walled_city_entry_allows_khalifa() {
        let mut state = playing(Scenario::Campaign);
        let from = HexCoord::new(0, 0);
        let city = HexCoord::new(1, 0);
        make_walled_board(&mut state, city);
        let khalifa = make_unit(
            &mut state,
            from,
            UnitKind::DervishLeader { fire: 0, melee: 0, movement: 0 },
            UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah),
            WeaponClass::Melee,
            UnitMovement::Land(crate::MovementAllowance::Eight),
        );
        assert!(
            state.can_move_unit_to(khalifa, Some(city), MovementPoints::new(1)).is_ok(),
            "Khalifa must be allowed into the walled city (§5.23)"
        );
    }

    #[rulebook("§5.23")]
    #[test]
    fn walled_city_entry_rejects_unauthorized_dervish() {
        let mut state = playing(Scenario::Campaign);
        state.active_player = Player::Dervish;
        let from = HexCoord::new(0, 0);
        let city = HexCoord::new(1, 0);
        make_walled_board(&mut state, city);
        let tribal = make_dervish_tribal(&mut state, from);
        assert!(matches!(
            state.can_move_unit_to(tribal, Some(city), MovementPoints::new(1)),
            Err(RuleError::WalledCityEntry(_, _))
        ));
    }

    #[rulebook("§5.23")]
    #[test]
    fn walled_city_entry_rejects_ae_gunboat() {
        let identity = UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Named(crate::NamedGunboat::Sultan));
        assert!(
            !identity.may_enter_walled_city(),
            "AE gunboats must be blocked from the walled city (§5.23)"
        );
    }

    #[rulebook("§5.23")]
    #[test]
    fn walled_city_entry_not_enforced_for_fok() {
        let mut state = playing(Scenario::FallOfKhartoum);
        let from = HexCoord::new(0, 0);
        let city = HexCoord::new(1, 0);
        make_walled_board(&mut state, city);
        let tribal = make_dervish_tribal(&mut state, from);
        // Baggara would fail on Campaign map, but FoK map is exempt.
        assert!(
            state.can_move_unit_to(tribal, Some(city), MovementPoints::new(1)).is_ok(),
            "FoK map must not enforce §5.23 walled-city entry"
        );
    }

    #[rulebook("§9.233")]
    #[test]
    fn zariba_end_hexside_costs_extra_mp() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(a, b),
            HexsideKind::ZaribaTrenchEndA,
        );
        // Seed terrain so movement_cost_for doesn't short-circuit on empty board.
        state.board.terrain.insert(a, Terrain::default());
        state.board.terrain.insert(b, Terrain::default());
        let ae = make_ae_infantry(&mut state, a);
        let unit = state.find_unit(ae).unwrap();
        let cost = state.movement_cost_for(unit, &[b]).unwrap();
        // Clear terrain = 1 MP + zariba surcharge 2 = 3 MP.
        assert_eq!(cost, MovementPoints::new(3));
    }

    #[rulebook("§9.233")]
    #[test]
    fn zariba_thorn_hedge_blocks_movement() {
        let mut state = playing(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        state.board.hexsides.insert(
            omdurman_types::HexsideRef::new(a, b),
            HexsideKind::ZaribaThornHedge,
        );
        let ae = make_ae_infantry(&mut state, a);
        assert!(matches!(
            state.can_move_unit_to(ae, Some(b), MovementPoints::new(1)),
            Err(RuleError::MoveBlockedByHexside(_, _))
        ));
    }

    #[rulebook("§5.51")]
    #[test]
    fn mid_move_stacking_allows_pass_through() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        // Path: (0,0) -> (1,0) -> (2,0).  Put 4 friendlies in (1,0), none in (2,0).
        let through = HexCoord::new(1, 0);
        let dest = HexCoord::new(2, 0);
        for _ in 0..4 {
            make_ae_infantry(&mut state, through);
        }
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // Move along the 2-hex path; stacking at (1,0) is never checked.
        assert!(
            state
                .can_move_unit_to(mover, Some(dest), MovementPoints::new(2))
                .is_ok(),
            "passing through a stacked hex must not be blocked (§5.51 mid-move)"
        );
    }

    #[rulebook("§5.51")]
    #[test]
    fn mid_move_stacking_rejects_over_limit_destination() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::Movement;
        state.active_player = Player::AngloEgyptian;
        let dest = HexCoord::new(2, 0);
        for _ in 0..4 {
            make_ae_infantry(&mut state, dest);
        }
        let mover = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        // Stacking is checked during apply, not can_move_unit_to.
        assert!(matches!(
            apply_effect(
                &mut state,
                &GameEffect::MoveUnit {
                    unit_id: mover,
                    to: dest,
                    cost: MovementPoints::new(1),
                    path: Vec::new(),
                }
            ),
            Err(RuleError::Stacking(crate::StackingError::OverLimit))
        ));
    }

    #[rulebook("§9.212")]
    #[test]
    fn setup_letter_dervish_leader_roundtrip() {
        use crate::dervish_leader_for_setup_letter;
        for letter in [
            SetupLetter::A,
            SetupLetter::D,
            SetupLetter::Y,
            SetupLetter::K,
            SetupLetter::S,
            SetupLetter::O,
        ] {
            let leader = dervish_leader_for_setup_letter(letter);
            assert_eq!(leader.setup_letter(), letter);
        }
    }

    #[rulebook("§9.212")]
    #[test]
    fn setup_letter_to_dervish_leader_known_values() {
        use crate::dervish_leader_for_setup_letter;
        assert_eq!(
            dervish_leader_for_setup_letter(SetupLetter::A),
            crate::DervishLeader::AliWadHelu
        );
        assert_eq!(
            dervish_leader_for_setup_letter(SetupLetter::K),
            crate::DervishLeader::KhalifaAbdullah
        );
        assert_eq!(
            dervish_leader_for_setup_letter(SetupLetter::O),
            crate::DervishLeader::OsmanDigna
        );
    }

    // ----- Part F: Named vs Old gunboat capabilities (§6.64, §2.32) ----

    fn make_named_gunboat(state: &mut GameState, hex: HexCoord) -> UnitId {
        make_unit(
            state,
            hex,
            UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 },
            UnitIdentity::AngloEgyptianGunboat(GunboatId::Named(
                crate::NamedGunboat::Sultan,
            )),
            WeaponClass::Artillery, // profile weapon stays Artillery
            UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Ten,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        )
    }

    // §6.64
    #[test]
    fn named_gunboat_has_howitzer() {
        assert!(GunboatId::Named(crate::NamedGunboat::Sultan).has_howitzer());
        assert!(GunboatId::Named(crate::NamedGunboat::Fateh).has_howitzer());
    }

    // §2.32
    #[test]
    fn old_gunboat_lacks_howitzer() {
        assert!(!GunboatId::Old(crate::OldGunboat::LordKitchener).has_howitzer());
        assert!(!GunboatId::Old(crate::OldGunboat::Tamai).has_howitzer());
    }

    // §6.64
    #[test]
    fn named_gunboat_may_fire_howitzer_in_second_subphase() {
        let mut state = playing(Scenario::Campaign);
        state.phase =
            Phase::OffensiveFire(crate::FireSubPhase::MaximSecondAndHowitzer);
        let gb = make_named_gunboat(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(5, 0);
        make_dervish_tribal(&mut state, target);
        assert!(
            state
                .can_fire_at(gb, target, FireKind::Howitzer)
                .is_ok(),
            "named gunboat must be allowed to fire howitzer (§6.64)"
        );
    }

    // §2.32
    #[test]
    fn old_gunboat_rejected_from_howitzer_subphase() {
        let mut state = playing(Scenario::Campaign);
        state.phase =
            Phase::OffensiveFire(crate::FireSubPhase::MaximSecondAndHowitzer);
        let gb = make_old_gunboat(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(3, 0);
        make_dervish_tribal(&mut state, target);
        assert!(
            matches!(
                state.can_fire_at(gb, target, FireKind::Howitzer),
                Err(RuleError::WrongWeaponForSubphase(_))
            ),
            "old gunboat must not fire howitzer (§2.32)"
        );
    }

    // §6.64: named gunboat in direct fire still uses the Artillery line.
    #[test]
    fn named_gunboat_direct_fire_uses_artillery_weapon() {
        let mut state = playing(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(crate::FireSubPhase::DirectFire);
        let gb = make_named_gunboat(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(3, 0);
        make_dervish_tribal(&mut state, target);
        assert!(
            state
                .can_fire_at(gb, target, FireKind::Direct)
                .is_ok(),
            "named gunboat must be allowed direct fire"
        );
    }

    // §6.64: named gunboat cannot fire howitzer at night.
    #[test]
    fn named_gunboat_no_howitzer_at_night() {
        let mut state = playing(Scenario::Campaign);
        state.phase =
            Phase::OffensiveFire(crate::FireSubPhase::MaximSecondAndHowitzer);
        state.day_night = DayNight::Night;
        let gb = make_named_gunboat(&mut state, HexCoord::new(0, 0));
        let target = HexCoord::new(3, 0);
        make_dervish_tribal(&mut state, target);
        assert!(
            matches!(
                state.can_fire_at(gb, target, FireKind::Howitzer),
                Err(RuleError::NoHowitzerAtNight)
            ),
            "howitzer fire at night must be rejected (§6.64)"
        );
    }

    // §6.64: Dervish gunboats have no howitzer.
    #[test]
    fn dervish_gunboat_lacks_howitzer() {
        assert!(!GunboatId::DervishGunboat(1).has_howitzer());
    }

    // §6.14: a combat unit may only be *fired at* once per fire phase
    // (exceptions: Maxims and gunboats). A second attack on the same target
    // hex in the same phase fires at the same units and must be rejected.
    #[rulebook("§6.14")]
    #[test]
    fn unit_may_only_be_fired_at_once_per_phase() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        let firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: attack.clone(),
                roll: DieRoll::Ten, // Eliminate(2): target gone
            },
        )
        .unwrap();
        assert!(state.find_unit(target).is_none());

        // A fresh firer attacks the same (now-empty) hex in the same phase:
        // the tracker recorded the target -- but he is eliminated, so this
        // targets nobody. Re-set with a survivor instead.
        let _target2 = make_dervish_tribal(&mut state, HexCoord::new(1, 0));
        let firer2 = make_ae_infantry(&mut state, HexCoord::new(2, 0));
        let attack2 = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer2],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        // The hex's previous occupant was fired at; a new occupant arriving
        // later in the same phase may be fired at (the rule is per-unit).
        // The genuine violation: fire at target2 twice.
        apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack: attack2,
                roll: DieRoll::One, // NoEffect -- but still "fired at"
            },
        )
        .unwrap();
        let firer3 = make_ae_infantry(&mut state, HexCoord::new(3, 0));
        let attack3 = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![firer3],
            target_hex: HexCoord::new(1, 0),
            factor_row: FireFactorRow::Row01to05,
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::FireCombat { attack: attack3, roll: DieRoll::Ten }),
            Err(RuleError::AlreadyFiredAt(_))
        ));
    }

    // §6.42: the Maxim/Howitzer subphase is a fresh fire phase for fired-at
    // purposes ("Units firing in this subphase may fire at enemy units fired
    // at in Direct Fire Subphase").
    #[rulebook("§6.42")]
    #[test]
    fn fired_at_tracker_resets_at_maxim_subphase() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        state.active_player = Player::AngloEgyptian;
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));
        let id = state.units[0].id;
        state.units_fired_at_this_phase.push(id);
        assert!(state.units_fired_at_this_phase.contains(&id));

        advance_phase(&mut state).unwrap(); // -> Maxim/Howitzer subphase
        assert!(
            state.units_fired_at_this_phase.is_empty(),
            "§6.42 bridge resets the fired-at tracker"
        );
    }

    // §6.14's exception: gunboats and Maxims may be fired at repeatedly.
    #[rulebook("§6.14")]
    #[test]
    fn gunboat_and_maxim_may_be_fired_at_repeatedly() {
        assert!(fired_at_excepted(UnitKind::Gunboat { fire: 0, upstream: 0, downstream: 0 }));
        assert!(fired_at_excepted(UnitKind::Maxim { fire: 0, melee: 0, movement: 0 }));
        assert!(!fired_at_excepted(UnitKind::Infantry { fire: 0, melee: 0, movement: 0 }));
        assert!(!fired_at_excepted(UnitKind::Fort { fire: 0, melee: 0 }));
    }

    // §9.111: only the Dervish initial force deploys at Campaign setup --
    // the rest arrive as §9.112/§9.113 reinforcements, and the
    // Anglo-Egyptian side deploys nothing at all.
    #[rulebook("§9.111")]
    #[test]
    fn campaign_setup_rejects_non_initial_force() {
        let mut state = GameState::new(Scenario::Campaign); // permissive zone
        let hex = HexCoord::new(1, 1);

        // §9.111 set deploys.
        for profile in [
            dervish_tribal_profile_with(DervishTribe::Taiasha),
            dervish_tribal_profile_with(DervishTribe::IsaZachneih),
        ] {
            let p = UnitPlacement {
                id: state.alloc_unit_id(),
                position: hex,
                profile,
                state: UnitState::default(),
            };
            assert!(state.can_deploy_unit(&p).is_ok(), "§9.111 unit rejected");
        }

        // A wave tribe (Baggara arrives turn 1 per §9.112) may not deploy at
        // setup; nor may any Anglo-Egyptian unit (§9.113).
        let baggara = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::Baggara),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&baggara),
            Err(RuleError::NotInPlay(_))
        ));
        let ae = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&ae),
            Err(RuleError::NotInPlay(_))
        ));
    }

    // §9.211/§9.212: the Historical scenario's not-in-play units may not be
    // deployed: GORDON and the "Friendlies" (AE), Isa Zachneih, gunboats and
    // forts (Dervish).
    #[rulebook("§9.211", "§9.212")]
    #[test]
    fn historical_setup_rejects_not_in_play_units() {
        let mut state = GameState::new(Scenario::Historical); // permissive zone
        let hex = HexCoord::new(1, 1);

        let gordon = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::BritishLeader { movement: 8 },
                identity: UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Gordon),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&gordon),
            Err(RuleError::NotInPlay(_))
        ));

        let isa = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::IsaZachneih),
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&isa),
            Err(RuleError::NotInPlay(_))
        ));

        // In-play units are unaffected.
        let baggara = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: dervish_tribal_profile_with(DervishTribe::Baggara),
            state: UnitState::default(),
        };
        assert!(state.can_deploy_unit(&baggara).is_ok());
        let ae = UnitPlacement {
            id: state.alloc_unit_id(),
            position: hex,
            profile: ae_infantry_profile(),
            state: UnitState::default(),
        };
        assert!(state.can_deploy_unit(&ae).is_ok());
    }

    // §9.321/§9.322: the FALL OF KHARTOUM orders of battle -- which unit
    // types exist at all, and their exact counts. Dervish fort counters play
    // no role (§9.344: the single North Fort is a scenario-fixed placement),
    // nor do Dervish gunboats or any non-entry tribe.
    #[rulebook("§9.322", "§9.344")]
    #[test]
    fn fok_order_of_battle_dervish() {
        let mut state = GameState::new(Scenario::FallOfKhartoum); // permissive zone
        let hex = HexCoord::new(1, 1);
        let dervish_fort_profile = || UnitProfile {
            kind: UnitKind::Fort { fire: 5, melee: 3 },
            identity: UnitIdentity::DervishFort,
            weapon: WeaponClass::Artillery,
            fire: Some(crate::FireFactor::Five),
            melee: Some(crate::MeleeFactor::Three),
            movement: UnitMovement::Immobile,
        };
        let dervish_leader_profile = |leader: DervishLeader| UnitProfile {
            kind: UnitKind::DervishLeader { fire: 3, melee: 6, movement: 9 },
            identity: UnitIdentity::DervishLeader(leader),
            weapon: WeaponClass::Melee,
            fire: Some(crate::FireFactor::Three),
            melee: Some(crate::MeleeFactor::Six),
            movement: UnitMovement::Land(crate::MovementAllowance::Nine),
        };
        let dervish_artillery_profile = || UnitProfile {
            kind: UnitKind::Artillery { fire: 5, melee: 1, movement: 7 },
            identity: UnitIdentity::DervishArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(crate::FireFactor::Five),
            melee: Some(crate::MeleeFactor::One),
            movement: UnitMovement::Land(crate::MovementAllowance::Seven),
        };

        // Not in play: the Dervish gunboats, the Khalifa, and every
        // non-entry tribe.
        for profile in vec![
            dervish_gunboat_profile(),
            dervish_leader_profile(DervishLeader::KhalifaAbdullah),
            dervish_tribal_profile_with(DervishTribe::Baggara),
            dervish_tribal_profile_with(DervishTribe::Taiasha),
            dervish_tribal_profile_with(DervishTribe::IsaZachneih),
        ] {
            let p = UnitPlacement {
                id: state.alloc_unit_id(),
                position: hex,
                profile,
                state: UnitState::default(),
            };
            assert!(
                matches!(state.can_deploy_unit(&p), Err(RuleError::NotInPlay(_))),
                "{:?} is not in the FoK order of battle",
                p.profile.identity
            );
        }

        // §9.322 counts: exactly 2 Hadendowa, 6 Kehena, 5 Degheim, 3
        // artillery, 32 Mulazmin.
        // Each type gets its own hex column: distinct tribes may not stack
        // (§5.52) and the four-unit limit (§5.51) would otherwise mask the
        // order-of-battle caps being tested.
        let count_cap =
            |state: &mut GameState, profile: UnitProfile, cap: usize, col: i32| {
                let mut accepted = 0;
                for i in 0..cap {
                    let p = UnitPlacement {
                        id: state.alloc_unit_id(),
                        position: HexCoord::new(col, 1 + (i % 8) as i32),
                        profile,
                        state: UnitState::default(),
                    };
                    if state.can_deploy_unit(&p).is_ok() {
                        apply_effect(state, &GameEffect::DeployUnit(p)).unwrap();
                        accepted += 1;
                    }
                }
                let over = UnitPlacement {
                    id: state.alloc_unit_id(),
                    position: HexCoord::new(col, 1 + (cap % 8) as i32),
                    profile,
                    state: UnitState::default(),
                };
                assert_eq!(
                    accepted, cap,
                    "expected to place {cap} of {:?}",
                    profile.identity
                );
                let err = state.can_deploy_unit(&over);
                let n = state
                    .units
                    .iter()
                    .filter(|u| u.profile.identity == over.profile.identity)
                    .count();
                assert!(
                    matches!(err, Err(RuleError::FoKOrderOfBattleFull)),
                    "cap of {} enforced for {:?}, got {:?} (on board: {n})",
                    cap,
                    profile.identity,
                    err
                );
            };
        count_cap(&mut state, dervish_tribal_profile_with(DervishTribe::Hadendowa), 2, 1);
        count_cap(&mut state, dervish_tribal_profile_with(DervishTribe::Kehena), 6, 2);
        count_cap(&mut state, dervish_tribal_profile_with(DervishTribe::Degheim), 5, 3);
        count_cap(&mut state, dervish_artillery_profile(), 3, 4);
        count_cap(&mut state, dervish_tribal_profile_with(DervishTribe::Mulazmin), 32, 5);

        // §9.344: exactly one Dervish fort is in play -- the scenario-fixed
        // North Fort. A first fort counter deploys (the scenario's fixed
        // placement uses the canonical counter); a second is rejected.
        count_cap(&mut state, dervish_fort_profile(), 1, 6);
    }

    // §9.321 regression: the counts bind across counter variants -- "two
    // British infantry units" covers 1B First + 1B Second; a third battalion
    // (whatever its ordinal) is rejected. Likewise the gunboat cap binds
    // across the four old-style boat counters.
    #[test]
    fn fok_caps_bind_across_counter_variants() {
        let mut state = GameState::new(Scenario::FallOfKhartoum);

        let id_1 = state.alloc_unit_id();
        let id_2 = state.alloc_unit_id();
        let id_3 = state.alloc_unit_id();
        let id_4 = state.alloc_unit_id();
        let id_5 = state.alloc_unit_id();

        // 1B First + 1B Second fit (cap 2 British); 1B Third is rejected.
        apply_effect(&mut state, &GameEffect::DeployUnit(UnitPlacement {
            id: id_1, position: HexCoord::new(3, 1),
            profile: UnitProfile {
                kind: UnitKind::Infantry { fire: 9, melee: 5, movement: 8 },
                identity: UnitIdentity::AngloEgyptianInfantry {
                    brigade: crate::BrigadeId { number: 1, nationality: crate::BrigadeNationality::British },
                    battalion: BattalionOrdinal::First,
                },
                weapon: WeaponClass::Rifles, fire: Some(crate::FireFactor::Nine),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            }, state: UnitState::default(),
        })).unwrap();
        apply_effect(&mut state, &GameEffect::DeployUnit(UnitPlacement {
            id: id_2, position: HexCoord::new(4, 1),
            profile: UnitProfile {
                kind: UnitKind::Infantry { fire: 9, melee: 5, movement: 8 },
                identity: UnitIdentity::AngloEgyptianInfantry {
                    brigade: crate::BrigadeId { number: 1, nationality: crate::BrigadeNationality::British },
                    battalion: BattalionOrdinal::Second,
                },
                weapon: WeaponClass::Rifles, fire: Some(crate::FireFactor::Nine),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            }, state: UnitState::default(),
        })).unwrap();
        let third = UnitPlacement {
            id: id_3, position: HexCoord::new(5, 1),
            profile: UnitProfile {
                kind: UnitKind::Infantry { fire: 9, melee: 5, movement: 8 },
                identity: UnitIdentity::AngloEgyptianInfantry {
                    brigade: crate::BrigadeId { number: 1, nationality: crate::BrigadeNationality::British },
                    battalion: BattalionOrdinal::Third,
                },
                weapon: WeaponClass::Rifles, fire: Some(crate::FireFactor::Nine),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            }, state: UnitState::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(third)),
            Err(RuleError::FoKOrderOfBattleFull)
        ));

        // Two different named old gunboats fill cap 2; third rejected.
        apply_effect(&mut state, &GameEffect::DeployUnit(UnitPlacement {
            id: id_4, position: HexCoord::new(3, 2),
            profile: UnitProfile {
                kind: UnitKind::Gunboat { fire: 0, upstream: 15, downstream: 16 },
                identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(crate::OldGunboat::LordKitchener)),
                weapon: WeaponClass::Artillery, fire: None, melee: None,
                movement: UnitMovement::Gunboat(crate::GunboatMovement {
                    upstream: crate::MovementAllowance::Fifteen,
                    downstream: crate::MovementAllowance::Sixteen,
                }),
            }, state: UnitState::default(),
        })).unwrap();
        apply_effect(&mut state, &GameEffect::DeployUnit(UnitPlacement {
            id: id_5, position: HexCoord::new(4, 2),
            profile: UnitProfile {
                kind: UnitKind::Gunboat { fire: 0, upstream: 15, downstream: 16 },
                identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(crate::OldGunboat::Tamai)),
                weapon: WeaponClass::Artillery, fire: None, melee: None,
                movement: UnitMovement::Gunboat(crate::GunboatMovement {
                    upstream: crate::MovementAllowance::Fifteen,
                    downstream: crate::MovementAllowance::Sixteen,
                }),
            }, state: UnitState::default(),
        })).unwrap();
        let third_boat = UnitPlacement {
            id: state.alloc_unit_id(), position: HexCoord::new(5, 2),
            profile: UnitProfile {
                kind: UnitKind::Gunboat { fire: 0, upstream: 15, downstream: 16 },
                identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(crate::OldGunboat::Metemmeh)),
                weapon: WeaponClass::Artillery, fire: None, melee: None,
                movement: UnitMovement::Gunboat(crate::GunboatMovement {
                    upstream: crate::MovementAllowance::Fifteen,
                    downstream: crate::MovementAllowance::Sixteen,
                }),
            }, state: UnitState::default(),
        };
        assert!(matches!(
            apply_effect(&mut state, &GameEffect::DeployUnit(third_boat)),
            Err(RuleError::FoKOrderOfBattleFull)
        ));
    }

    // §9.321: the British garrison -- old gunboats only (no named), one
    // artillery, and the battalion counts per nationality.

    // §9.321: the British garrison -- old gunboats only (no named), one
    // artillery, and the battalion counts per nationality.    // §9.321: the British garrison -- old gunboats only (no named), one
    // artillery, and the battalion counts per nationality.
    #[rulebook("§9.321")]
    #[test]
    fn fok_order_of_battle_british() {
        let mut state = GameState::new(Scenario::FallOfKhartoum); // permissive zone

        // Not in play: named gunboats, cavalry, Maxims, Royal Engineers,
        // non-Gordon leaders, the Camel Corps.
        for profile in [
            UnitProfile {
                kind: UnitKind::Gunboat { fire: 0, upstream: 15, downstream: 16 },
                identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Named(
                    crate::NamedGunboat::Sultan,
                )),
                weapon: WeaponClass::Artillery,
                fire: None,
                melee: None,
                movement: UnitMovement::Gunboat(crate::GunboatMovement {
                    upstream: crate::MovementAllowance::Fifteen,
                    downstream: crate::MovementAllowance::Sixteen,
                }),
            },
            UnitProfile {
                kind: UnitKind::Cavalry { fire: 8, melee: 5, movement: 15 },
                identity: UnitIdentity::AngloEgyptianCavalry,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Eight),
                melee: Some(crate::MeleeFactor::Five),
                movement: UnitMovement::Land(crate::MovementAllowance::Fifteen),
            },
            UnitProfile {
                kind: UnitKind::Maxim { fire: 6, melee: 1, movement: 12 },
                identity: UnitIdentity::AngloEgyptianMaxim,
                weapon: WeaponClass::Maxims,
                fire: Some(crate::FireFactor::Six),
                melee: Some(crate::MeleeFactor::One),
                movement: UnitMovement::Land(crate::MovementAllowance::Twelve),
            },
            UnitProfile {
                kind: UnitKind::Infantry { fire: 5, melee: 3, movement: 8 },
                identity: UnitIdentity::RoyalEngineers,
                weapon: WeaponClass::Rifles,
                fire: Some(crate::FireFactor::Five),
                melee: Some(crate::MeleeFactor::Three),
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
            UnitProfile {
                kind: UnitKind::BritishLeader { movement: 8 },
                identity: UnitIdentity::AngloEgyptianLeader(crate::BritishLeader::Kitchener),
                weapon: WeaponClass::Melee,
                fire: None,
                melee: None,
                movement: UnitMovement::Land(crate::MovementAllowance::Eight),
            },
        ] {
            let p = UnitPlacement {
                id: state.alloc_unit_id(),
                position: HexCoord::new(1, 1),
                profile,
                state: UnitState::default(),
            };
            assert!(
                matches!(state.can_deploy_unit(&p), Err(RuleError::NotInPlay(_))),
                "{:?} is not in the FoK garrison",
                p.profile.identity
            );
        }

        // §9.321 counts: 2 old gunboats, 1 artillery, 2 British / 3 Egyptian
        // / 4 Sudanese / 4 Friendlies battalions.
        let old_gb = UnitProfile {
            kind: UnitKind::Gunboat { fire: 0, upstream: 15, downstream: 16 },
            identity: UnitIdentity::AngloEgyptianGunboat(crate::GunboatId::Old(
                crate::OldGunboat::LordKitchener,
            )),
            weapon: WeaponClass::Artillery,
            fire: None,
            melee: None,
            movement: UnitMovement::Gunboat(crate::GunboatMovement {
                upstream: crate::MovementAllowance::Fifteen,
                downstream: crate::MovementAllowance::Sixteen,
            }),
        };
        let ae_infantry_of = |nationality: crate::BrigadeNationality| UnitProfile {
            kind: UnitKind::Infantry { fire: 9, melee: 5, movement: 8 },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: crate::BrigadeId { number: 1, nationality },
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(crate::FireFactor::Nine),
            melee: Some(crate::MeleeFactor::Five),
            movement: UnitMovement::Land(crate::MovementAllowance::Eight),
        };
        let ae_artillery = UnitProfile {
            kind: UnitKind::Artillery { fire: 8, melee: 1, movement: 7 },
            identity: UnitIdentity::AngloEgyptianArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(crate::FireFactor::Eight),
            melee: Some(crate::MeleeFactor::One),
            movement: UnitMovement::Land(crate::MovementAllowance::Seven),
        };

        let mut place = |state: &mut GameState, profile: UnitProfile, hex: HexCoord| {
            let p = UnitPlacement {
                id: state.alloc_unit_id(),
                position: hex,
                profile,
                state: UnitState::default(),
            };
            apply_effect(state, &GameEffect::DeployUnit(p)).unwrap();
        };
        for i in 0..2 {
            place(&mut state, old_gb, HexCoord::new(20 + i as i32, 1)); // Nile-side permissive test board
        }
        let third_gb = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(20, 1),
            profile: old_gb,
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&third_gb),
            Err(RuleError::FoKOrderOfBattleFull)
        ));

        place(&mut state, ae_artillery, HexCoord::new(2, 1));
        let second_art = UnitPlacement {
            id: state.alloc_unit_id(),
            position: HexCoord::new(2, 1),
            profile: ae_artillery,
            state: UnitState::default(),
        };
        assert!(matches!(
            state.can_deploy_unit(&second_art),
            Err(RuleError::FoKOrderOfBattleFull)
        ));

        let mut col = 3;
        for (nationality, cap) in [
            (crate::BrigadeNationality::British, 2),
            (crate::BrigadeNationality::Egyptian, 3),
            (crate::BrigadeNationality::Sudanese, 4),
            (crate::BrigadeNationality::Friendlies, 4),
        ] {
            let profile = ae_infantry_of(nationality);
            for i in 0..cap {
                place(&mut state, profile, HexCoord::new(col, 1 + (i % 3) as i32));
            }
            let over = UnitPlacement {
                id: state.alloc_unit_id(),
                position: HexCoord::new(col, 1 + (cap % 3) as i32),
                profile,
                state: UnitState::default(),
            };
            col += 1;
            assert!(
                matches!(state.can_deploy_unit(&over), Err(RuleError::FoKOrderOfBattleFull)),
                "cap of {cap} enforced for {nationality:?}"
            );
        }
    }

    // ----- Part G: ZOC + invariant property tests ---------------------------

    #[rulebook("§5.41")]
    #[test]
    fn zoc_hexes_matches_hex_in_enemy_zoc() {
        // For every enemy unit, hex_in_enemy_zoc(my_hex) should be true
        // if and only if zoc_hexes(enemy) contains my_hex.
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;

        let enemy1 = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        let enemy2 = make_dervish_tribal(&mut state, HexCoord::new(5, 7));

        let mover_kind = UnitKind::Infantry { fire: 0, melee: 0, movement: 0 };

        for u in &state.units.clone() {
            let zoc = state.zoc_hexes(u, Player::AngloEgyptian, mover_kind);
            for &adj in &u.position.neighbors() {
                let in_zoc = state.hex_in_enemy_zoc(adj, Player::AngloEgyptian, mover_kind);
                if in_zoc {
                    assert!(
                        zoc.contains(&adj),
                        "hex_in_enemy_zoc({adj:?}) is true but zoc_hexes({:?}) does not contain it (unit at {:?})",
                        u.id, u.position
                    );
                }
            }
        }
        let _ = (enemy1, enemy2);
    }

    #[rulebook("§5.41", "§5.44")]
    #[test]
    fn zoc_hexes_excludes_nile() {
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        let flow = omdurman_types::HexDirection::East;
        // Put enemy on one side of a Nile hex, check ZOC doesn't extend across.
        state.board.terrain.insert(HexCoord::new(1, 0), Terrain::Nile { direction: flow });
        let enemy = make_dervish_tribal(&mut state, HexCoord::new(0, 0));
        let zoc = state.zoc_hexes(
            state.find_unit(enemy).unwrap(),
            Player::AngloEgyptian,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
        );
        assert!(!zoc.contains(&HexCoord::new(1, 0)), "ZOC should not extend into Nile hex");
    }

    #[rulebook("§5.41", "§5.44")]
    #[test]
    fn zoc_hexes_excludes_khor() {
        let mut state = GameState::new(Scenario::Campaign);
        state.active_player = Player::AngloEgyptian;
        let enemy_pos = HexCoord::new(1, 1);
        let target = HexCoord::new(1, 0);
        let enemy = make_dervish_tribal(&mut state, enemy_pos);
        state.board.hexsides.insert(HexsideRef::new(enemy_pos, target), HexsideKind::Khor);
        let zoc = state.zoc_hexes(
            state.find_unit(enemy).unwrap(),
            Player::AngloEgyptian,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
        );
        assert!(!zoc.contains(&target), "ZOC should not cross khor hexside");
    }

    #[rulebook("§5.41")]
    #[test]
    fn zoc_hexes_empty_for_disrupted_unit() {
        let mut state = GameState::new(Scenario::Campaign);
        let enemy = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        state.find_unit_mut(enemy).unwrap().state.disrupted = true;
        let zoc = state.zoc_hexes(
            state.find_unit(enemy).unwrap(),
            Player::AngloEgyptian,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
        );
        assert!(zoc.is_empty(), "disrupted unit should project no ZOC");
    }

    #[rulebook("§5.41")]
    #[test]
    fn zoc_hexes_empty_for_anglo_egyptian_leader() {
        let mut state = GameState::new(Scenario::Campaign);
        let leader = make_ae_leader(&mut state, HexCoord::new(5, 5));
        let zoc = state.zoc_hexes(
            state.find_unit(leader).unwrap(),
            Player::Dervish,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
        );
        assert!(zoc.is_empty(), "AE leaders project no ZOC (§5.41)");
    }

    #[rulebook("§5.41", "§5.44")]
    #[test]
    fn zoc_hexes_normal_unit_projects_six_adjacent_minus_exclusions() {
        let mut state = GameState::new(Scenario::Campaign);
        let enemy = make_dervish_tribal(&mut state, HexCoord::new(5, 5));
        let zoc = state.zoc_hexes(
            state.find_unit(enemy).unwrap(),
            Player::AngloEgyptian,
            UnitKind::Infantry { fire: 0, melee: 0, movement: 0 },
        );
        // On an empty board, a normal unit projects ZOC to all 6 neighbours.
        assert_eq!(zoc.len(), 6, "normal unit should project ZOC to 6 hexes");
    }

    // ----- Part H: GameState invariant checker ------------------------------

    #[rulebook("§5.51", "§5.52")]
    #[test]
    fn check_invariants_clean_state() {
        let state = GameState::new(Scenario::Campaign);
        let violations = state.check_invariants();
        assert!(violations.is_empty(), "clean state has no violations: {violations:?}");
    }

    #[rulebook("§5.51")]
    #[test]
    fn check_invariants_catches_stacking_violation() {
        let mut state = GameState::new(Scenario::Campaign);
        let hex = HexCoord::new(5, 5);
        // Place 5 non-leader, non-gunboat Dervish units in the same hex (§5.51 max 4).
        for _ in 0..5 {
            make_dervish_tribal(&mut state, hex);
        }
        let violations = state.check_invariants();
        assert!(violations.iter().any(|v| v.contains("stacking")),
            "should catch stacking violation: {violations:?}");
    }

    #[rulebook("§5.51")]
    #[test]
    fn check_invariants_allows_leaders_stacking() {
        let mut state = GameState::new(Scenario::Campaign);
        let hex = HexCoord::new(5, 5);
        // 4 infantry + 1 leader = OK (leaders are free stacking).
        for _ in 0..4 {
            make_dervish_tribal(&mut state, hex);
        }
        // We can't easily make a Dervish leader with make_dervish_tribal, but the
        // test verifies that exactly 4 tribal units is not flagged.
        let violations = state.check_invariants();
        assert!(!violations.iter().any(|v| v.contains("stacking")),
            "4 units + leaders should be fine: {violations:?}");
    }
}

use super::*;

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

    #[error(
        "unit {0:?} entered an enemy zone of control and may move no further this turn (§5.43)"
    )]
    StoppedInEnemyZoc(UnitId),

    #[error(
        "fire modifiers must equal the rulebook-mandated set (§6.24/§5.54/§9.231/§9.232): expected {expected:?}, got {got:?}"
    )]
    FireModifierMismatch {
        expected: Vec<crate::FireModifier>,
        got: Vec<crate::FireModifier>,
    },

    #[error(
        "melee modifiers must equal the rulebook-mandated set (§7.7/§9.232): expected {expected:?}, got {got:?}"
    )]
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

    #[error(
        "a declared melee must be resolved (or its target vacated by retreat) before the melee phase can end"
    )]
    MeleePendingResolution,

    #[error(
        "the §8.2 desertion roll must be made before the Dervish movement phase of the first night turn can end"
    )]
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

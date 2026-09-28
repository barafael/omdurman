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

    #[error("hex {0} is outside this unit's deployment zone (§9.2/§9.3)")]
    OutsideDeploymentZone(HexCoord),

    /// §9.211: the Anglo-Egyptian set-up areas of the Historical scenario.
    #[error("{hex} is outside this unit's set-up area: {area}")]
    HistoricalSetUpArea { hex: HexCoord, area: &'static str },

    /// §9.212: "All remaining Dervish units set up within three hexes of
    /// their leader as identified by color."
    #[error(
        "{hex} is more than three hexes from {leader}: Dervish units set up within three hexes of the leader of their colour (§9.212)"
    )]
    SetUpFarFromLeader {
        hex: HexCoord,
        leader: crate::DervishLeader,
    },

    /// §9.212: "All Dervish units must be set up out of the line of sight of
    /// all Anglo-Egyptian units."
    #[error(
        "{hex} is in sight of the Anglo-Egyptian unit at {seen_from}: Dervish units set up out of the line of sight of all Anglo-Egyptian units (§9.212)"
    )]
    SetUpInEnemySight { hex: HexCoord, seen_from: HexCoord },

    #[error(
        "unit {0} is not in play at setup for this scenario (§9.111/§9.211/§9.212): it arrives as a reinforcement or is excluded"
    )]
    NotInPlay(UnitId),

    #[error(
        "the FALL OF KHARTOUM order of battle (§9.321/§9.322) allows no more units of this type"
    )]
    FoKOrderOfBattleFull,

    #[error("{0}")]
    SetupLimit(&'static str),

    /// §9.111/§9.211/§9.321: the other side sets up first and has not yet
    /// confirmed its deployment.
    #[error("{0}")]
    SetupOrder(&'static str),

    #[error("counter {0} is already on the board -- each physical unit deploys once")]
    AlreadyDeployed(UnitId),

    #[error("{0} was eliminated -- a destroyed unit never returns to play")]
    UnitEliminated(UnitId),

    #[error("unit {0} has already fired this phase")]
    AlreadyFired(UnitId),

    #[error("fire must target an enemy-occupied hex (§6.15)")]
    FireTargetNotEnemyOccupied,

    #[error("unit {0} has already been fired at this phase (§6.14)")]
    AlreadyFiredAt(UnitId),

    #[error("unit {0} has already moved this turn")]
    AlreadyMoved(UnitId),

    #[error("unit {0} is disrupted and may not act")]
    Disrupted(UnitId),

    #[error("GORDON may not move during FALL OF KHARTOUM (§9.346)")]
    GordonMayNotMove,

    #[error("a unit may not enter an enemy-occupied fort hex {0} (§6.54)")]
    EnemyFort(HexCoord),

    #[error(
        "hex {0} is occupied by enemy units -- engaging the enemy is what melee is for (§7.1); movement may only end adjacent (§5.26)"
    )]
    EnemyOccupied(HexCoord),

    #[error("unit {0} not found")]
    UnitNotFound(UnitId),

    #[error("unit {0} does not belong to the acting player")]
    NotOwner(UnitId),

    #[error("line of sight is blocked from {0} to {1} (§6.3)")]
    LineOfSightBlocked(HexCoord, HexCoord),

    #[error("{0} to {1} crosses {side}", side = melee_block_reason(*.2))]
    MeleeBlockedByHexside(HexCoord, HexCoord, HexsideKind),

    #[error("a hexside blocks advance after combat from {0} to {1} (§6.82, §7.6)")]
    AdvanceBlockedByHexside(HexCoord, HexCoord),

    #[error("a wall hexside blocks movement from {0} to {1} (§5.23)")]
    MoveBlockedByHexside(HexCoord, HexCoord),

    #[error("unit {0} is not eligible to enter the walled city of Omdurman at {1} (§5.23)")]
    WalledCityEntry(UnitId, HexCoord),

    #[error("movement cost {cost} exceeds allowance {allowance}")]
    MovementExceedsAllowance {
        cost: MovementPoints,
        allowance: MovementAllowance,
    },

    #[error("movement may not pass through an enemy zone of control at {0}")]
    BlockedByEnemyZoc(HexCoord),

    #[error("unit {0} entered an enemy zone of control and may move no further this turn (§5.43)")]
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

    #[error("land unit may not enter the Nile hex {0} (§5.22)")]
    LandIntoNile(HexCoord),

    #[error("hex {0} is off the board")]
    OffBoard(HexCoord),

    #[error("gunboat may only move along Nile hexes; {0} is not Nile (§5.22)")]
    GunboatOffNile(HexCoord),

    #[error(
        "gunboat moved upstream, so its upstream allowance {allowance} caps the turn, but the move costs {cost} (§5.24)"
    )]
    GunboatUpstreamCap {
        cost: MovementPoints,
        allowance: MovementAllowance,
    },

    #[error("gunboat entered a chained Nile hex {0} and must stop (§10.22)")]
    BlockedByChain(HexCoord),

    #[error("illegal stack: {0}")]
    Stacking(#[from] crate::StackingError),

    #[error("illegal Dervish desertion: {0}")]
    Desertion(#[from] DesertionError),

    #[error("unit {0} cannot move on land")]
    NotMobile(UnitId),

    #[error("unit {0} is not a gunboat")]
    NotAGunboat(UnitId),

    #[error("only howitzer-class units may fire howitzer (unit {0})")]
    OnlyHowitzerMayFireHowitzer(UnitId),

    #[error("only Maxim units may use second fire (unit {0})")]
    OnlyMaximSecondFire(UnitId),

    #[error("no howitzer fire at night (§6.64)")]
    NoHowitzerAtNight,

    #[error("unit {0} has no fire factor")]
    NoFireFactor(UnitId),

    #[error("only artillery may fire at a gunboat or fort (§6.61, §6.62)")]
    ArtilleryOnlyVsGunboatOrFort(UnitId),

    #[error("target {target} out of range at night from {firer} (§8.1)")]
    OutOfRangeAtNight { firer: HexCoord, target: HexCoord },

    #[error("target {target} out of range from {firer}")]
    TargetOutOfRange { firer: HexCoord, target: HexCoord },

    #[error("unit {0} kind may not melee attack")]
    KindMayNotMelee(UnitId),

    #[error("target {to} is not adjacent to {from}")]
    TargetNotAdjacent { from: HexCoord, to: HexCoord },

    #[error("no meleeable enemy in target hex {0}")]
    NoMeleeableEnemy(HexCoord),

    #[error("unit {0} may not move once placed (§5.25)")]
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

    #[error("no declared infantry melee threatens unit {0}")]
    NoInfantryMeleeThreatens(UnitId),

    #[error("unit {0} may not retreat before melee")]
    MayNotRetreatBeforeMelee(UnitId),

    #[error("retreat must be exactly two hexes")]
    RetreatMustBeTwoHexes,

    #[error("retreat hex {0} is held by the enemy")]
    RetreatHexOccupied(HexCoord),

    #[error("there is no enemy fort at {0} to fire at (§6.62)")]
    NoFortToFireAt(HexCoord),

    #[error("the fort at {0} is empty: only artillery fire at the fort itself may hit it (§6.62)")]
    FortStandsEmpty(HexCoord),

    #[error("no open two-hex retreat path from {0} to {1} (§7.5)")]
    RetreatPathBlocked(HexCoord, HexCoord),

    #[error("unit {0} has already made its melee attack this turn (§7.5)")]
    AlreadyMeleed(UnitId),

    #[error("artillery unit {0} may not advance after combat")]
    ArtilleryMayNotAdvance(UnitId),

    #[error("fort {0} may not move in any way once placed (§5.25)")]
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
        "hex {0} is outside the annotated entrance area for this reinforcement (§9.112/§9.113)"
    )]
    OutsideEntranceArea(HexCoord),

    #[error("advance hex is not adjacent")]
    AdvanceNotAdjacent,

    #[error("advance hex {0} is not vacant")]
    AdvanceNotVacant(HexCoord),

    #[error(
        "advance hex {0} was not vacated by combat this phase (§6.82, §7.6): advance is only legal into a hex the defender vacated"
    )]
    HexNotVacatedByCombat(HexCoord),

    #[error(
        "unit {0} did not participate in the combat that vacated {1} (§6.82, §7.6): only participating attackers may advance"
    )]
    UnitDidNotParticipate(UnitId, HexCoord),

    #[error("unit {0} is not disrupted")]
    NotDisrupted(UnitId),

    #[error("Friendlies transport requires Isa Zachneih to be eliminated first (§5.21)")]
    FriendliesIsaZachneihAlive,

    #[error("a Friendlies transport mission is already in progress (§5.21)")]
    FriendliesTransportInProgress,

    #[error("Friendlies unit must be adjacent to the gunboat to load (§5.21)")]
    FriendliesNotAdjacentToGunboat,

    #[error("the unit and the gunboat must start their turn adjacent to load (§5.21)")]
    FriendliesMustStartTurnAdjacent,

    #[error("only a \"Friendlies\" unit loads, onto an Anglo-Egyptian gunboat of its own (§5.21)")]
    FriendliesWrongUnits,

    #[error("that unit is not aboard that gunboat (§5.21)")]
    FriendliesNotLoaded,

    #[error("the \"Friendlies\" may disembark on turn {0} at the earliest (§5.21)")]
    FriendliesDisembarkTooEarly(u8),

    #[error("{0} is no west-bank land hex next to the gunboat (§5.21)")]
    FriendliesDisembarkHex(HexCoord),

    #[error("gunboat {0} has lost its engines and only drifts with the current (§10.12)")]
    EnginesLost(UnitId),

    #[error("gunboat {0} struck a mine and stops for the turn (§10.12)")]
    StruckMine(UnitId),

    #[error(
        "{0} is not in the first wave: three gunboats, the \"Friendlies\", the Egyptian Cavalry, the Horse Artillery and two Egyptian Division brigades (§9.113)"
    )]
    NotInFirstWave(UnitId),

    #[error("{hex}: {area}")]
    CampaignSetUpArea { hex: HexCoord, area: &'static str },

    #[error("Kitchener, Gatacre and Hunter must all be in play by the end of turn four (§9.113)")]
    LeadersMustEnterByTurnFour,

    #[error("the mine a gunboat struck must be rolled for first (§10.12)")]
    MinePendingResolution,

    #[error("unit {0} is aboard a gunboat: it moves with it and leaves it by disembarking (§5.21)")]
    LoadedOnGunboat(UnitId),

    #[error(
        "gunboat {0} took the \"Friendlies\" aboard this turn and carries them next turn (§5.21)"
    )]
    TransportLoadingTurn(UnitId),

    #[error("gunboat {0} engines are not lost; cannot drift")]
    GunboatEnginesNotLost(UnitId),

    #[error("no untriggered river mine in hex {0} (§10.13)")]
    NoUntriggeredMine(HexCoord),

    #[error("river chain is already sunk")]
    ChainAlreadySunk,

    #[error("no river chain has been placed")]
    NoChainPlaced,

    #[error("fire attack has no firers")]
    NoFirers,

    #[error("only artillery may fire to breach a wall hexside (§6.63; unit {0})")]
    OnlyArtilleryMayBreachWall(UnitId),

    #[error("hexside {0:?} is not a Wall (§6.63)")]
    NotAWallHexside(HexsideRef),

    /// A coordinate in the effect lies outside the engine's sane coordinate
    /// range ([`MAX_COORD_ABS`](crate::effects::MAX_COORD_ABS)). Rejected up
    /// front so hex arithmetic on network-supplied values can never overflow.
    #[error("hex {0} is outside the playable coordinate range")]
    CoordinateOutOfBounds(HexCoord),

    /// A `MoveUnit` path step is not a single hex (§5.11: units move hex by
    /// hex to adjacent hexes).
    #[error("movement path is not contiguous: {from} -> {to} is not a single-hex step (§5.11)")]
    PathNotContiguous { from: HexCoord, to: HexCoord },

    /// A `MoveUnit` path does not end at the declared destination.
    #[error("movement path must end at the destination {0}")]
    PathEndMismatch(HexCoord),

    /// A `MoveUnit` path is longer than any allowance could pay for.
    #[error("movement path is too long ({0} steps)")]
    PathTooLong(usize),

    /// The same unit is listed more than once in an effect (fire, melee,
    /// desertion, reinforcement or zariba batch).
    #[error("unit {0} is listed more than once")]
    DuplicateUnit(UnitId),

    /// A fire attack's `firing_player` does not match the player whose fire
    /// phase it is (§4, §6.41).
    #[error("the attack's firing player is not the player whose fire phase it is")]
    FiringPlayerMismatch,

    /// A declared melee's defender list is not exactly the meleeable enemy
    /// units in the target hex (§7.1).
    #[error("melee defenders must be exactly the meleeable enemy units in {0} (§7.1)")]
    MeleeDefendersMismatch(HexCoord),

    /// A placement's profile/state differs from the canonical counter
    /// (`unit_profiles::profile_for_unit`): a peer may not invent unit values.
    #[error("unit {0} must be placed with its canonical counter profile and a fresh state")]
    NonCanonicalPlacement(UnitId),

    /// Only the Royal Engineers may demolish (§6.53).
    #[error("only the Royal Engineers may demolish (§6.53; unit {0})")]
    NotRoyalEngineers(UnitId),

    /// The demolition target is not adjacent to the engineers, or is not a
    /// standing enemy fort / wall hexside (§6.53).
    #[error(
        "the demolition target is not a standing fort or wall adjacent to the engineers (§6.53)"
    )]
    InvalidDemolitionTarget,

    /// The unit is busy constructing a zariba (§5.3) or demolishing (§6.53)
    /// this turn: it may neither fire offensively nor melee attack.
    #[error("unit {0} is constructing a zariba or demolishing this turn (§5.3, §6.53)")]
    BusyWithEngineering(UnitId),

    /// A zariba construction request that §5.3 does not allow.
    #[error("illegal zariba construction: {0} (§5.3)")]
    IllegalZariba(&'static str),

    /// `HowitzerFire` must carry a howitzer attack and `FireCombat` must not
    /// (§6.64: howitzer fire always rolls for scatter).
    #[error(
        "fire kind does not match the effect (§6.64: howitzer fire is resolved with a scatter roll)"
    )]
    FireKindMismatch,
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

    #[error("unit {0} is not a Dervish unit eligible to desert")]
    NotEligible(UnitId),

    #[error("the Khalifa, gunboats, artillery, and forts may not desert (unit {0})")]
    Exempt(UnitId),
}

/// The hexside of `kind` that stops a melee across it, and why (§7.2,
/// §9.231, Terrain Effects Chart).
fn melee_block_reason(kind: HexsideKind) -> &'static str {
    match kind {
        HexsideKind::ZaribaThornHedge => {
            "the Zariba's thorn hedge, closed to melee both ways (§9.231)"
        }
        HexsideKind::Khor | HexsideKind::KhorShambat => {
            "a khor, closed to melee (Terrain Effects Chart)"
        }
        HexsideKind::Wall => "a wall, open to melee only at a gate or breach (§7.2)",
        _ => "a hexside closed to melee (§7.2)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Refusal reasons reach players verbatim (the "Order Refused" and
    /// "Field Telegraph" slips): hexes read as the board shows them and
    /// counters by name, never as `HexCoord { .. }` or `MulazminII_5_1`.
    #[test]
    fn refusals_name_hexes_and_units_as_players_see_them() {
        let night = RuleError::OutOfRangeAtNight {
            firer: HexCoord::new(14, 8),
            target: HexCoord::new(13, 6),
        };
        assert_eq!(
            night.to_string(),
            "target (13, 6) out of range at night from (14, 8) (§8.1)"
        );
        let fired = RuleError::AlreadyFired(UnitId::MulazminII_5_1).to_string();
        assert_eq!(fired, "unit Mulazmin has already fired this phase");
    }
}

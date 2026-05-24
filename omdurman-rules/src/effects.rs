//! Semantic game effects — every mutation passes through [`apply_effect`].
//!
//! Each [`GameEffect`] carries *all* information (including pre-rolled die
//! values) needed to apply it deterministically.  The processor validates
//! the effect against the current [`GameState`] and, if legal, mutates the
//! state in place.  Network replay works because every peer receives the
//! identical effect with the identical roll.

use serde::{Deserialize, Serialize};

use crate::tables::{
    ae_range_effects, anglo_egyptian_crt, campaign_turn, dervish_crt, dervish_range_effects,
    historical_turn, howitzer_scatter, FireFactorRow, ScatterDirection,
};
use crate::{
    CampaignVictoryLevel, CombatResult, DayNight, DemolitionTarget, DieModifier, DieRoll,
    FireAttack, FireKind, FireSubPhase, GameTurnIndex, HexCoord, HexDistance, HexsideRef,
    MeleeAttack, MovementAllowance, MovementPoints, Phase, Player, Scenario, UnitId,
    UnitPlacement, VictoryLedger, VpSource, WeaponClass,
};
use crate::tables::TurnEvent;

use crate::FriendliesTransport;
use crate::{ChainPlacement, MinePlacement, OptionalRule};

// ---------------------------------------------------------------------------
// 1) GameEffect — every semantic action a player can take
// ---------------------------------------------------------------------------

/// A semantic game action, fully determined (all dice pre-rolled).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GameEffect {
    // -- Turn / phase flow ------------------------------------------------

    /// Advance to the next phase (or next player-turn if melee is done).
    AdvancePhase,

    // -- Movement ----------------------------------------------------------

    /// Move a unit along a path (vector of hex coords, starting with the
    /// origin and ending with the final hex).  The total movement-point
    /// cost is validated against the unit's allowance.
    MoveUnit {
        unit_id: UnitId,
        cost: MovementPoints,
    },

    // -- Fire combat -------------------------------------------------------

    /// Resolve a direct or Maxim-second-fire attack.
    FireCombat {
        attack: FireAttack,
        roll: DieRoll,
    },

    /// Resolve a howitzer bombardment (two rolls: CRT + impact scatter).
    HowitzerFire {
        attack: FireAttack,
        crt_roll: DieRoll,
        impact_roll: DieRoll,
    },

    // -- Melee combat ------------------------------------------------------

    /// Resolve melee between adjacent hexes (simultaneous, two rolls).
    MeleeCombat {
        attack: MeleeAttack,
        attacker_roll: DieRoll,
        defender_roll: DieRoll,
    },

    // -- Unit state changes ------------------------------------------------

    /// Remove disrupted status from a unit (end of owning player's turn).
    RecoverUnit {
        unit_id: UnitId,
    },

    /// Begin constructing a Zariba hexside.
    ConstructZariba {
        unit_ids: Vec<UnitId>,
        hexside: HexsideRef,
    },

    /// Royal Engineers demolition.
    Demolition {
        unit_id: UnitId,
        target: DemolitionTarget,
    },

    // -- Reinforcement / placement -----------------------------------------

    /// Place reinforcements onto the map.
    PlaceReinforcements(Vec<UnitPlacement>),

    // -- Scenario-specific -------------------------------------------------

    /// Dervish desertion roll (turn 8 — first night of campaign).
    DervishDesertion {
        roll: DieRoll,
    },

    /// Load/disembark the "Friendlies" brigade via gunboat.
    FriendliesTransport(crate::FriendliesTransport),

    // -- Optional rules ----------------------------------------------------

    RiverMine {
        gunboat_id: UnitId,
        hex: HexCoord,
        roll: DieRoll,
    },
}

// ---------------------------------------------------------------------------
// 2) RuleError — why an effect was rejected
// ---------------------------------------------------------------------------

/// Validation error returned when [`apply_effect`] refuses an effect.
#[derive(thiserror::Error, Clone, Debug)]
pub enum RuleError {
    #[error("game is over")]
    GameOver,

    #[error("not your turn")]
    NotYourTurn,

    #[error("wrong phase for this action")]
    WrongPhase,

    #[error("unit {0:?} has already fired this phase")]
    AlreadyFired(UnitId),

    #[error("unit {0:?} has already moved this turn")]
    AlreadyMoved(UnitId),

    #[error("unit {0:?} is disrupted and may not act")]
    Disrupted(UnitId),

    #[error("unit {0:?} not found")]
    UnitNotFound(UnitId),

    #[error("target hex {0:?} contains no enemy units")]
    NoEnemyInHex(HexCoord),

    #[error("unit {0:?} is out of range")]
    OutOfRange(UnitId),

    #[error("movement cost {cost:?} exceeds allowance {allowance:?}")]
    MovementExceedsAllowance {
        cost: MovementPoints,
        allowance: MovementAllowance,
    },

    #[error("hex stack would exceed the four-unit limit")]
    StackOverflow,

    #[error("{0}")]
    Other(&'static str),
}

// ---------------------------------------------------------------------------
// 3) GameState — authoritative mutable snapshot
// ---------------------------------------------------------------------------

/// All mutable state of a game in progress.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    pub scenario: Scenario,
    pub current_turn: GameTurnIndex,
    pub day_night: DayNight,
    pub active_player: Player,
    pub phase: Phase,
    pub units: Vec<UnitPlacement>,
    pub victory: VictoryLedger,
    pub next_unit_id: u32,
    pub units_fired_this_phase: Vec<UnitId>,
    pub units_moved_this_turn: Vec<UnitId>,
    pub game_over: bool,
    pub zariba_hexsides: Vec<HexsideRef>,
    pub friendlies_transport: Vec<FriendliesTransport>,
    pub optional_rules: Vec<OptionalRule>,
    pub mines: Vec<MinePlacement>,
    pub chain: Option<ChainPlacement>,
    pub log: Vec<String>,
}

impl GameState {
    /// Create a fresh game state for a given scenario.
    pub fn new(scenario: Scenario) -> Self {
        let first = match scenario {
            Scenario::Campaign => campaign_turn(1),
            Scenario::Historical | Scenario::FallOfKhartoum => historical_turn(1),
        };
        let (day_night, active) = match first {
            Some(t) => (t.day_night, Player::AngloEgyptian),
            None => (DayNight::Day, Player::AngloEgyptian),
        };
        GameState {
            scenario,
            current_turn: GameTurnIndex(1),
            day_night,
            active_player: active,
            phase: Phase::Movement,
            units: Vec::new(),
            victory: VictoryLedger::default(),
            next_unit_id: 1,
            units_fired_this_phase: Vec::new(),
            units_moved_this_turn: Vec::new(),
            game_over: false,
            zariba_hexsides: Vec::new(),
            friendlies_transport: Vec::new(),
            optional_rules: Vec::new(),
            mines: Vec::new(),
            chain: None,
            log: Vec::new(),
        }
    }

    /// Find a unit by ID.
    pub fn find_unit(&self, id: UnitId) -> Option<&UnitPlacement> {
        self.units.iter().find(|u| u.id == id)
    }

    /// Mutable lookup by ID.
    pub fn find_unit_mut(&mut self, id: UnitId) -> Option<&mut UnitPlacement> {
        self.units.iter_mut().find(|u| u.id == id)
    }

    /// All units in a given hex.
    pub fn units_in_hex(&self, hex: HexCoord) -> Vec<&UnitPlacement> {
        self.units.iter().filter(|u| u.position == hex).collect()
    }

    /// All units of a given player in a hex.
    pub fn player_units_in_hex(&self, hex: HexCoord, player: Player) -> Vec<&UnitPlacement> {
        self.units
            .iter()
            .filter(|u| u.position == hex && u.profile.identity.owner() == player)
            .collect()
    }

    /// Brushfire next-unit-id and produce a fresh ID.
    pub fn alloc_unit_id(&mut self) -> UnitId {
        let id = UnitId(self.next_unit_id);
        self.next_unit_id += 1;
        id
    }

    /// Log a human-readable message.
    pub fn log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
    }
}

// ---------------------------------------------------------------------------
// 4) apply_effect — the effect processor
// ---------------------------------------------------------------------------

/// Validate and apply a [`GameEffect`] to `state`.
///
/// Returns `Ok(())` on success; the state has been mutated.  Returns
/// `Err(RuleError)` if the effect is illegal for the current state; the
/// state is left unchanged.
pub fn apply_effect(state: &mut GameState, effect: &GameEffect) -> Result<(), RuleError> {
    if state.game_over {
        return Err(RuleError::GameOver);
    }
    match effect {
        GameEffect::AdvancePhase => advance_phase(state),
        GameEffect::MoveUnit { unit_id, cost } => apply_move_unit(state, *unit_id, *cost),
        GameEffect::FireCombat { attack, roll } => apply_fire_combat(state, attack, *roll),
        GameEffect::HowitzerFire {
            attack,
            crt_roll,
            impact_roll,
        } => apply_howitzer_fire(state, attack, *crt_roll, *impact_roll),
        GameEffect::MeleeCombat {
            attack,
            attacker_roll,
            defender_roll,
        } => apply_melee_combat(state, attack, *attacker_roll, *defender_roll),
        GameEffect::RecoverUnit { unit_id } => apply_recover_unit(state, *unit_id),
        GameEffect::ConstructZariba { unit_ids, hexside } => {
            apply_construct_zariba(state, unit_ids, *hexside)
        }
        GameEffect::Demolition { unit_id, target } => {
            apply_demolition(state, *unit_id, *target)
        }
        GameEffect::PlaceReinforcements(placements) => {
            apply_place_reinforcements(state, placements)
        }
        GameEffect::DervishDesertion { roll } => apply_dervish_desertion(state, *roll),
        GameEffect::FriendliesTransport(action) => {
            apply_friendlies_transport(state, action.clone())
        }
        GameEffect::RiverMine {
            gunboat_id,
            hex,
            roll,
        } => apply_river_mine(state, *gunboat_id, *hex, *roll),
    }
}

// ---------------------------------------------------------------------------
// 5) Phase advancement
// ---------------------------------------------------------------------------

fn advance_phase(state: &mut GameState) -> Result<(), RuleError> {
    match state.phase {
        Phase::Movement => {
            state.phase = Phase::DefensiveFire(FireSubPhase::DirectFire);
            state.log(format!(
                "--- {} Defensive Fire (Direct) ---",
                state.active_player.opponent()
            ));
        }
        Phase::DefensiveFire(FireSubPhase::DirectFire) => {
            if state.active_player == Player::AngloEgyptian {
                state.phase =
                    Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer);
                state.log("--- Defensive Fire (Maxim 2nd / Howitzer) ---");
            } else {
                state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
                state.log(format!(
                    "--- {} Offensive Fire (Direct) ---",
                    state.active_player
                ));
            }
        }
        Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer) => {
            state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
            state.log(format!(
                "--- {} Offensive Fire (Direct) ---",
                state.active_player
            ));
        }
        Phase::OffensiveFire(FireSubPhase::DirectFire) => {
            if state.active_player == Player::AngloEgyptian {
                state.phase =
                    Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer);
                state.log("--- Offensive Fire (Maxim 2nd / Howitzer) ---");
            } else {
                state.phase = Phase::Melee;
                state.log("--- Melee ---");
            }
        }
        Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer) => {
            state.phase = Phase::Melee;
            state.log("--- Melee ---");
        }
        Phase::Melee => end_player_turn(state)?,
    }
    Ok(())
}

fn end_player_turn(state: &mut GameState) -> Result<(), RuleError> {
    // Collect disrupted units first, then apply recovery.
    let to_recover: Vec<UnitId> = state
        .units
        .iter()
        .filter(|u| u.state.disrupted && u.profile.identity.owner() == state.active_player)
        .map(|u| u.id)
        .collect();
    for id in &to_recover {
        if let Some(unit) = state.find_unit_mut(*id) {
            unit.state.disrupted = false;
            state.log(format!("Unit {:?} recovers from disruption", id));
        }
    }

    // Clear per-phase tracking.
    state.units_fired_this_phase.clear();
    state.units_moved_this_turn.clear();

    // Switch to next player.
    let next = state.active_player.opponent();
    state.active_player = next;
    state.phase = Phase::Movement;

    // If we've wrapped around, advance the game turn.
    if next == Player::AngloEgyptian {
        let next_idx = state.current_turn.0 + 1;
        match state.scenario {
            Scenario::Campaign => {
                match campaign_turn(next_idx) {
                    Some(entry) => {
                        state.current_turn = GameTurnIndex(next_idx);
                        state.day_night = entry.day_night;
                        state.log(format!(
                            "=== Turn {} ({}) ===",
                            entry.turn, entry.time
                        ));
                        // Trigger desertion on first night turn (turn 8).
                        if entry.event == TurnEvent::DervishDesertion {
                            state.log("Dervish desertion phase begins — roll required");
                        }
                    }
                    None => {
                        state.game_over = true;
                        state.log("=== GAME OVER ===");
                        let ae = state.victory.total_for(Player::AngloEgyptian);
                        let d = state.victory.total_for(Player::Dervish);
                        let superiority = ae.0 - d.0;
                        let level =
                            CampaignVictoryLevel::from_superiority(crate::VictoryPoints(superiority));
                        state.log(format!(
                            "A-E VP: {:?}, Dervish VP: {:?}, Superiority: {}, Result: {:?}",
                            ae, d, superiority, level
                        ));
                    }
                }
            }
            Scenario::Historical | Scenario::FallOfKhartoum => {
                match historical_turn(next_idx) {
                    Some(entry) => {
                        state.current_turn = GameTurnIndex(next_idx);
                        state.day_night = entry.day_night;
                    }
                    None => {
                        state.game_over = true;
                        state.log("=== GAME OVER ===");
                    }
                }
            }
        }
    } else {
        state.log(format!("--- {} Movement ---", next));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 6) Movement
// ---------------------------------------------------------------------------

fn apply_move_unit(
    state: &mut GameState,
    unit_id: UnitId,
    cost: MovementPoints,
) -> Result<(), RuleError> {
    let unit = state
        .find_unit(unit_id)
        .ok_or(RuleError::UnitNotFound(unit_id))?;

    if !matches!(state.phase, Phase::Movement) {
        return Err(RuleError::WrongPhase);
    }
    if state.active_player != unit.profile.identity.owner() {
        return Err(RuleError::NotYourTurn);
    }
    if unit.state.disrupted {
        return Err(RuleError::Disrupted(unit_id));
    }
    if state.units_moved_this_turn.contains(&unit_id) {
        return Err(RuleError::AlreadyMoved(unit_id));
    }

    let allowance = match unit.profile.movement {
        crate::UnitMovement::Land(a) => a,
        crate::UnitMovement::Gunboat(_) | crate::UnitMovement::Immobile => {
            return Err(RuleError::Other("unit cannot move on land"));
        }
    };

    let effective_allowance = crate::effective_movement_at_night(
        allowance,
        unit.profile.identity.owner(),
        state.day_night,
    );

    if cost.0 > effective_allowance.0 as i16 {
        return Err(RuleError::MovementExceedsAllowance {
            cost,
            allowance: effective_allowance,
        });
    }

    // Record movement.
    state.units_moved_this_turn.push(unit_id);
    state.log(format!(
        "Unit {:?} moves (cost {})",
        unit_id,
        cost.0
    ));

    Ok(())
}

// ---------------------------------------------------------------------------
// 7) Fire combat
// ---------------------------------------------------------------------------

fn apply_fire_combat(
    state: &mut GameState,
    attack: &FireAttack,
    roll: DieRoll,
) -> Result<(), RuleError> {
    // -- Validation --
    validate_fire_attack(state, attack)?;

    // Mark firers as having fired this phase.
    for &id in &attack.firers {
        state.units_fired_this_phase.push(id);
    }

    // -- Resolution --
    let is_ae = attack.firing_player == Player::AngloEgyptian;

    // Compute range.
    let range = target_range(state, &attack.firers, attack.target_hex)?;

    // Night halving.
    let effective_range = if state.day_night == DayNight::Night {
        crate::effective_range_at_night(range)
    } else {
        range
    };

    // Look up range effects — firer's first unit determines weapon class.
    let weapon = state
        .find_unit(attack.firers[0])
        .map(|u| u.profile.weapon)
        .unwrap_or(WeaponClass::Rifles);

    let band = if is_ae {
        ae_range_effects(weapon, effective_range)
    } else {
        dervish_range_effects(weapon, effective_range)
    };

    let effective_factor = band.apply(attack.total_factor);

    // Terrain defense modifier.
    let terrain_mod = target_terrain_defense(state, attack.target_hex);

    // Net die modifier.
    let net_mod = attack.net_modifier();
    let total_mod = DieModifier(net_mod.0 + terrain_mod.0);
    let modified_roll = total_mod.apply(roll);

    // CRT lookup.
    let row = FireFactorRow::from_factor(effective_factor);
    let result = if is_ae {
        anglo_egyptian_crt(row, modified_roll)
    } else {
        dervish_crt(row, modified_roll)
    };

    // Apply result.
    let target_units: Vec<UnitId> = state
        .player_units_in_hex(attack.target_hex, attack.firing_player.opponent())
        .iter()
        .map(|u| u.id)
        .collect();

    let log_line = format!(
        "{} fire @ hex {:?}: roll={}, mod={}, net_roll={}, factor={}->{}, CRT={:?}, units={:?}",
        attack.firing_player,
        attack.target_hex,
        roll.get(),
        total_mod.0,
        modified_roll.get(),
        attack.total_factor.0,
        effective_factor.0,
        result,
        target_units,
    );
    state.log(log_line);

    apply_crt_result(state, result, &target_units, attack.firing_player.opponent());

    Ok(())
}

fn apply_howitzer_fire(
    state: &mut GameState,
    attack: &FireAttack,
    crt_roll: DieRoll,
    impact_roll: DieRoll,
) -> Result<(), RuleError> {
    // Howitzer has special validation.
    if state.day_night == DayNight::Night {
        return Err(RuleError::Other("no howitzer fire at night"));
    }
    validate_fire_attack(state, attack)?;

    for &id in &attack.firers {
        state.units_fired_this_phase.push(id);
    }

    // Resolve scatter.
    let scatter = howitzer_scatter(impact_roll);
    let actual_target = match scatter.direction {
        ScatterDirection::OnTarget => attack.target_hex,
        // For MVP: scatter directions are placeholder — actual hex offset
        // depends on the hex grid orientation on the map.
        _ => attack.target_hex,
    };

    let is_ae = attack.firing_player == Player::AngloEgyptian;
    let weapon = state
        .find_unit(attack.firers[0])
        .map(|u| u.profile.weapon)
        .unwrap_or(WeaponClass::Howitzer);

    let range = target_range(state, &attack.firers, actual_target)?;
    let effective_range = if state.day_night == DayNight::Night {
        crate::effective_range_at_night(range)
    } else {
        range
    };

    let band = if is_ae {
        ae_range_effects(weapon, effective_range)
    } else {
        dervish_range_effects(weapon, effective_range)
    };

    let effective_factor = band.apply(attack.total_factor);
    let terrain_mod = target_terrain_defense(state, actual_target);
    let net_mod = attack.net_modifier();
    let total_mod = DieModifier(net_mod.0 + terrain_mod.0);
    let modified_roll = total_mod.apply(crt_roll);

    let row = FireFactorRow::from_factor(effective_factor);
    let result = if is_ae {
        anglo_egyptian_crt(row, modified_roll)
    } else {
        dervish_crt(row, modified_roll)
    };

    let target_units: Vec<UnitId> = state
        .player_units_in_hex(actual_target, attack.firing_player.opponent())
        .iter()
        .map(|u| u.id)
        .collect();

    state.log(format!(
        "Howitzer fire {} @ {:?}: impact={}, scatter={:?}",
        attack.firing_player,
        attack.target_hex,
        impact_roll.get(),
        scatter.direction,
    ));

    apply_crt_result(state, result, &target_units, attack.firing_player.opponent());

    Ok(())
}

// ---------------------------------------------------------------------------
// 8) Melee combat
// ---------------------------------------------------------------------------

fn apply_melee_combat(
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
    let attacker_total = attack
        .attackers
        .iter()
        .filter_map(|id| state.find_unit(*id))
        .map(|u| u.profile.melee.unwrap_or(crate::MeleeFactor(0)).0)
        .sum::<u16>();

    let defender_total = attack
        .defenders
        .iter()
        .filter_map(|id| state.find_unit(*id))
        .map(|u| u.profile.melee.unwrap_or(crate::MeleeFactor(0)).0)
        .sum::<u16>();

    // Compute modifiers.
    let att_mod: i16 = attack.attacker_modifiers.iter().map(|m| m.die_modifier().0).sum();
    let def_mod: i16 = attack.defender_modifiers.iter().map(|m| m.die_modifier().0).sum();

    let att_net = DieModifier(att_mod).apply(attacker_roll);
    let def_net = DieModifier(def_mod).apply(defender_roll);

    // Melee uses the appropriate CRT with melee factors treated as fire factors.
    let att_row = FireFactorRow::from_factor(crate::FireFactor(attacker_total));
    let def_row = FireFactorRow::from_factor(crate::FireFactor(defender_total));

    let att_result = if attacker_player == Player::AngloEgyptian {
        anglo_egyptian_crt(att_row, att_net)
    } else {
        dervish_crt(att_row, att_net)
    };

    let def_result = if defender_player == Player::AngloEgyptian {
        anglo_egyptian_crt(def_row, def_net)
    } else {
        dervish_crt(def_row, def_net)
    };

    // Compare severity: attackers use their result against defenders and vice versa.
    let _attacker_elim = elimination_count(att_result);
    let _defender_elim = elimination_count(def_result);

    let att_units: Vec<UnitId> = attack.attackers.clone();
    let def_units: Vec<UnitId> = attack.defenders.clone();

    state.log(format!(
        "Melee {} vs {} @ {:?}: AT roll={} (net {}), DEF roll={} (net {}), AT factor={}, DEF factor={}, AT result={:?}, DEF result={:?}",
        attacker_player,
        defender_player,
        attack.defender_hex,
        attacker_roll.get(),
        att_net.get(),
        defender_roll.get(),
        def_net.get(),
        attacker_total,
        defender_total,
        att_result,
        def_result,
    ));

    // Simultaneous application.
    apply_crt_result(state, att_result, &def_units, defender_player);
    apply_crt_result(state, def_result, &att_units, attacker_player);

    Ok(())
}

fn elimination_count(result: CombatResult) -> u8 {
    match result {
        CombatResult::NoEffect => 0,
        CombatResult::Disrupt => 0,
        CombatResult::Eliminate(n) => n,
    }
}

// ---------------------------------------------------------------------------
// 9) Unit state changes
// ---------------------------------------------------------------------------

fn apply_recover_unit(
    state: &mut GameState,
    unit_id: UnitId,
) -> Result<(), RuleError> {
    let unit = state
        .find_unit_mut(unit_id)
        .ok_or(RuleError::UnitNotFound(unit_id))?;
    if !unit.state.disrupted {
        return Err(RuleError::Other("unit is not disrupted"));
    }
    unit.state.disrupted = false;
    state.log(format!("Unit {:?} recovers from disruption", unit_id));
    Ok(())
}

fn apply_construct_zariba(
    state: &mut GameState,
    unit_ids: &[UnitId],
    hexside: HexsideRef,
) -> Result<(), RuleError> {
    for &id in unit_ids {
        let unit = state
            .find_unit_mut(id)
            .ok_or(RuleError::UnitNotFound(id))?;
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(id));
        }
        unit.state.constructing_zariba = true;
    }
    state.zariba_hexsides.push(hexside);
    state.log(format!("Zariba constructed at {:?}", hexside));
    Ok(())
}

fn apply_demolition(
    state: &mut GameState,
    unit_id: UnitId,
    target: DemolitionTarget,
) -> Result<(), RuleError> {
    let unit = state
        .find_unit_mut(unit_id)
        .ok_or(RuleError::UnitNotFound(unit_id))?;
    if unit.state.disrupted {
        return Err(RuleError::Disrupted(unit_id));
    }
    unit.state.demolishing = true;
    state.log(format!(
        "Unit {:?} begins demolition of {:?}",
        unit_id, target
    ));
    Ok(())
}

// ---------------------------------------------------------------------------
// 10) Reinforcements
// ---------------------------------------------------------------------------

fn apply_place_reinforcements(
    state: &mut GameState,
    placements: &[UnitPlacement],
) -> Result<(), RuleError> {
    for p in placements {
        // Check stacking limit.
        let count = state.units_in_hex(p.position).len() as u16;
        if count >= 4 {
            return Err(RuleError::StackOverflow);
        }
    }
    for p in placements {
        state.log(format!(
            "Reinforcements: Unit {:?} placed at hex {:?}",
            p.id, p.position
        ));
        state.units.push(*p);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 11) Scenario-specific
// ---------------------------------------------------------------------------

fn apply_dervish_desertion(
    state: &mut GameState,
    roll: DieRoll,
) -> Result<(), RuleError> {
    // §8.2: on a roll of 1–4, one Dervish unit is removed (furthest from
    // the walled city). On a 5+, no effect.
    state.log(format!("Dervish desertion roll: {}", roll.get()));
    if roll.get() <= 4 {
        // Find the Dervish unit furthest from Omdurman (simplified: remove
        // the first Dervish unit in the list).
        if let Some(idx) = state
            .units
            .iter()
            .position(|u| u.profile.identity.owner() == Player::Dervish)
        {
            let removed = state.units.remove(idx);
            state.log(format!(
                "Desertion: Dervish unit {:?} removed",
                removed.id
            ));
        }
    }
    Ok(())
}

fn apply_friendlies_transport(
    state: &mut GameState,
    action: FriendliesTransport,
) -> Result<(), RuleError> {
    // Validate that the referenced units exist.
    match &action {
        FriendliesTransport::Loaded { unit, gunboat }
        | FriendliesTransport::Crossing { unit, gunboat }
        | FriendliesTransport::ReadyToDisembark { unit, gunboat } => {
            if state.find_unit(*unit).is_none() || state.find_unit(*gunboat).is_none() {
                return Err(RuleError::Other("friendlies transport unit not found"));
            }
        }
    }
    state.log(format!("Friendlies transport: {:?}", action));
    state.friendlies_transport.push(action);
    Ok(())
}

// ---------------------------------------------------------------------------
// 12) Optional rules
// ---------------------------------------------------------------------------

fn apply_river_mine(
    state: &mut GameState,
    gunboat_id: UnitId,
    hex: HexCoord,
    roll: DieRoll,
) -> Result<(), RuleError> {
    let result = crate::MineResult::from_roll(roll);
    state.log(format!(
        "River mine at {:?}, gunboat {:?}: roll {} -> {:?}",
        hex,
        gunboat_id,
        roll.get(),
        result
    ));
    match result {
        crate::MineResult::NoEffect => {}
        crate::MineResult::EnginesLost => {
            if state.find_unit_mut(gunboat_id).is_some() {
                state.log(format!(
                    "Gunboat {:?} engines lost — drifts with current",
                    gunboat_id
                ));
            }
        }
        crate::MineResult::Sunk => {
            state.units.retain(|u| u.id != gunboat_id);
            state.log(format!("Gunboat {:?} sunk by mine", gunboat_id));
        }
    }
    // Mark mine as triggered.
    for mine in &mut state.mines {
        if mine.hex == hex {
            mine.triggered = true;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Validate that a fire attack is legal in the current state.
fn validate_fire_attack(state: &GameState, attack: &FireAttack) -> Result<(), RuleError> {
    // Phase check.
    let allowed = match (&state.phase, attack.kind) {
        (Phase::DefensiveFire(FireSubPhase::DirectFire), FireKind::Direct) => true,
        (Phase::DefensiveFire(FireSubPhase::DirectFire), FireKind::MaximSecondFire) => false,
        (
            Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer),
            FireKind::MaximSecondFire | FireKind::Howitzer,
        ) => true,
        (Phase::OffensiveFire(FireSubPhase::DirectFire), FireKind::Direct) => true,
        (Phase::OffensiveFire(FireSubPhase::DirectFire), FireKind::MaximSecondFire) => false,
        (
            Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
            FireKind::MaximSecondFire | FireKind::Howitzer,
        ) => true,
        (Phase::Melee, _) => false,
        (Phase::Movement, _) => false,
        _ => false,
    };
    if !allowed {
        return Err(RuleError::WrongPhase);
    }

    // Player check.
    match &state.phase {
        Phase::DefensiveFire(_) => {
            if attack.firing_player != state.active_player.opponent() {
                return Err(RuleError::NotYourTurn);
            }
        }
        Phase::OffensiveFire(_) => {
            if attack.firing_player != state.active_player {
                return Err(RuleError::NotYourTurn);
            }
        }
        _ => {}
    }

    // Check that firers exist, are not disrupted, haven't already fired.
    for &id in &attack.firers {
        let unit = state.find_unit(id).ok_or(RuleError::UnitNotFound(id))?;
        if unit.state.disrupted {
            return Err(RuleError::Disrupted(id));
        }
        if state.units_fired_this_phase.contains(&id) {
            return Err(RuleError::AlreadyFired(id));
        }
        // Howitzer check: only named gunboats.
        if attack.kind == FireKind::Howitzer {
            match &unit.profile.weapon {
                WeaponClass::Howitzer => {}
                _ => return Err(RuleError::Other("only howitzer-class units may fire howitzer")),
            }
        }
        // Maxim second fire check.
        if attack.kind == FireKind::MaximSecondFire {
            if unit.profile.weapon != WeaponClass::Maxims {
                return Err(RuleError::Other("only Maxim units may use second fire"));
            }
        }
    }

    Ok(())
}

/// Compute the distance between the first firer and the target hex.
fn target_range(state: &GameState, firers: &[UnitId], target: HexCoord) -> Result<HexDistance, RuleError> {
    let firer = state
        .find_unit(firers[0])
        .ok_or(RuleError::UnitNotFound(firers[0]))?;
    // Simple hex-distance: Manhattan-like for axial coords.
    let dq = (firer.position.q - target.q).unsigned_abs();
    let dr = (firer.position.r - target.r).unsigned_abs();
    let ds = (firer.position.q + firer.position.r - target.q - target.r).unsigned_abs();
    // Cube-distance max.
    let dist = dq.max(dr.max(ds / 2));
    Ok(HexDistance(dist as u16))
}

/// Get the terrain defense modifier for a hex.
///
/// For MVP this returns 0; the caller must push a
/// [`FireModifier::Terrain`] into the attack's modifier list with the
/// correct value obtained from the game-map terrain data.
fn target_terrain_defense(_state: &GameState, _hex: HexCoord) -> DieModifier {
    DieModifier(0)
}

/// Apply a CRT result to a list of target units — eliminate `n` and disrupt
/// half (round up) of the remaining.
fn apply_crt_result(
    state: &mut GameState,
    result: CombatResult,
    target_ids: &[UnitId],
    target_player: Player,
) {
    match result {
        CombatResult::NoEffect => {}
        CombatResult::Disrupt => {
            // Disrupt half (round up) of the target units.
            let n = (target_ids.len() + 1) / 2;
            for &id in target_ids.iter().take(n) {
                if let Some(unit) = state.find_unit_mut(id) {
                    unit.state.disrupted = true;
                    state.log(format!("Unit {:?} disrupted", id));
                }
            }
        }
        CombatResult::Eliminate(n) => {
            let n = (n as usize).min(target_ids.len());
            // Half (round up) of the survivors are also disrupted.
            let disrupt_n = (target_ids.len().saturating_sub(n) + 1) / 2;

            for &id in target_ids.iter().take(n) {
                state.log(format!("Unit {:?} eliminated", id));
                score_elimination(state, id, target_player);
            }
            state.units.retain(|u| !target_ids[..n].contains(&u.id));

            // Disrupt survivors.
            let survivors: Vec<UnitId> = target_ids[n..].to_vec();
            for &id in survivors.iter().take(disrupt_n) {
                if let Some(unit) = state.find_unit_mut(id) {
                    unit.state.disrupted = true;
                    state.log(format!("Unit {:?} disrupted", id));
                }
            }
        }
    }
}

/// Score victory points for eliminating a unit.
fn score_elimination(state: &mut GameState, unit_id: UnitId, _owner: Player) {
    if let Some(unit) = state.find_unit(unit_id) {
        let source = match &unit.profile.identity {
            crate::UnitIdentity::DervishTribal { .. }
            | crate::UnitIdentity::DervishLeader(_)
            | crate::UnitIdentity::DervishArtillery
            | crate::UnitIdentity::DervishGunboat(_)
            | crate::UnitIdentity::DervishFort => VpSource::DervishUnitEliminated,
            crate::UnitIdentity::AngloEgyptianLeader(_) => VpSource::BritishLeaderEliminated,
            crate::UnitIdentity::AngloEgyptianGunboat(_) => VpSource::BritishGunboatSunk,
            _ => VpSource::AngloEgyptianLandUnitEliminated,
        };
        // Check if it's a "Friendlies" unit on east or west bank.
        let is_friendlies = unit.profile.identity.is_friendlies();
        let vp_source = if is_friendlies {
            // Simplified: assume west bank for MVP.
            VpSource::FriendliesWestBankEliminated
        } else {
            source
        };
        state.victory.events.push(crate::VpEvent {
            turn: state.current_turn,
            source: vp_source,
        });
        state.log(format!(
            "Scored {:?} ({} pts)",
            vp_source,
            vp_source.points().0
        ));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn make_ae_infantry(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::Infantry,
                identity: UnitIdentity::AngloEgyptianInfantry {
                    brigade: BrigadeId {
                        number: 1,
                        nationality: BrigadeNationality::British,
                    },
                    battalion: BattalionOrdinal(1),
                },
                weapon: WeaponClass::Rifles,
                fire: Some(FireFactor(4)),
                melee: Some(MeleeFactor(2)),
                movement: UnitMovement::Land(MovementAllowance(6)),
            },
            state: UnitState::default(),
        });
        id
    }

    fn make_dervish_tribal(state: &mut GameState, hex: HexCoord) -> UnitId {
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: hex,
            profile: UnitProfile {
                kind: UnitKind::Infantry,
                identity: UnitIdentity::DervishTribal {
                    tribe: DervishTribe::Baggara,
                },
                weapon: WeaponClass::Rifles,
                fire: Some(FireFactor(2)),
                melee: Some(MeleeFactor(3)),
                movement: UnitMovement::Land(MovementAllowance(6)),
            },
            state: UnitState::default(),
        });
        id
    }

    #[test]
    fn fire_combat_eliminates_target() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::OffensiveFire(FireSubPhase::DirectFire);
        let _firer = make_ae_infantry(&mut state, HexCoord::new(0, 0));
        let target = make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![UnitId(1)],
            target_hex: HexCoord::new(1, 0),
            total_factor: FireFactor(8),
            modifiers: vec![FireModifier::AngloEgyptianDirectFire],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::new(8),
            },
        );
        assert!(result.is_ok());
        // Dervish unit should be eliminated (roll 8, factor 8 -> Eliminate(1) on A-E CRT).
        assert!(state.find_unit(target).is_none());
    }

    #[test]
    fn fire_combat_wrong_phase_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        make_ae_infantry(&mut state, HexCoord::new(0, 0));
        make_dervish_tribal(&mut state, HexCoord::new(1, 0));

        let attack = FireAttack {
            firing_player: Player::AngloEgyptian,
            phase: state.phase,
            kind: FireKind::Direct,
            firers: vec![UnitId(1)],
            target_hex: HexCoord::new(1, 0),
            total_factor: FireFactor(8),
            modifiers: vec![],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::new(5),
            },
        );
        assert!(result.is_err());
        assert!(matches!(result, Err(RuleError::WrongPhase)));
    }

    #[test]
    fn movement_exceeds_allowance_rejected() {
        let mut state = GameState::new(Scenario::Campaign);
        state.phase = Phase::Movement;
        make_ae_infantry(&mut state, HexCoord::new(0, 0));

        let result = apply_effect(
            &mut state,
            &GameEffect::MoveUnit {
                unit_id: UnitId(1),
                cost: MovementPoints(99),
            },
        );
        assert!(result.is_err());
    }

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
            total_factor: FireFactor(4),
            modifiers: vec![],
        };

        let result = apply_effect(
            &mut state,
            &GameEffect::FireCombat {
                attack,
                roll: DieRoll::new(5),
            },
        );
        assert!(matches!(result, Err(RuleError::Disrupted(_))));
    }

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
                attacker_roll: DieRoll::new(7),
                defender_roll: DieRoll::new(3),
            },
        );
        assert!(result.is_ok());
        // At least one should have a log entry.
        assert!(!state.log.is_empty());
    }

    #[test]
    fn turn_advances_through_phases() {
        let mut state = GameState::new(Scenario::Campaign);
        assert_eq!(state.phase, Phase::Movement);
        assert_eq!(state.active_player, Player::AngloEgyptian);

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(state.phase, Phase::DefensiveFire(_)));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        // A-E gets MaximSecondAndHowitzer sub-phase.
        assert!(matches!(
            state.phase,
            Phase::DefensiveFire(FireSubPhase::MaximSecondAndHowitzer)
        ));

        apply_effect(&mut state, &GameEffect::AdvancePhase).unwrap();
        assert!(matches!(state.phase, Phase::OffensiveFire(_)));

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
    }

    #[test]
    fn game_over_after_campaign_turns() {
        let mut state = GameState::new(Scenario::Campaign);
        // Fast-forward past all campaign turns.
        for _ in 0..100 {
            if state.game_over {
                break;
            }
            // Advance through all phases for each player turn.
            for _ in 0..6 {
                // Movement, DefFire(Direct), DefFire(Maxim2nd/How), OffFire(Direct), OffFire(Maxim2nd/How), Melee
                if apply_effect(&mut state, &GameEffect::AdvancePhase).is_err() {
                    break;
                }
            }
        }
        assert!(state.game_over);
    }
}

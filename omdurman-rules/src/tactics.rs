//! Deterministic rules vignettes ("tactics") for _Remember Gordon!_.
//!
//! A **tactics script** is a small, hand-built [`GameState`] plus an ordered
//! list of scripted steps. Legal steps assert that [`apply_effect`] accepts an
//! effect; illegal steps probe that it is rejected with exactly the expected
//! [`RuleError`] shape; asserts run a predicate over the state.
//!
//! Every vignette uses pre-rolled dice and a fixed board, so each script is
//! fully deterministic. The same scripts drive both the unit-test runner
//! (`omdurman-rules/tests/tactics.rs`) and the bot CLI's `tactics` subcommand,
//! making them a living, human-readable regression suite for the rules engine.
//!
//! Rulebook citations use the `§N.M` form and mirror the citations already
//! present in the engine sources that implement them (see
//! `docs/traceability.toml`).

use std::sync::Arc;

use omdurman_types::{
    BrigadeId, DayNight, DervishTribe, HexCoord, Player, Scenario, UnitKind,
};

use crate::board::BoardInfo;
use crate::board_data::{campaign_map_data, fall_of_khartoum_map_data};
use crate::combat_results_table::FireFactorRow;
use crate::effects::{apply_effect, GameEffect, GameState, RuleError};
use crate::unit_profiles::profile_for_unit;
use crate::{
    BattalionOrdinal, DieRoll, FireAttack, FireFactor, FireKind, FireModifier, FireSubPhase,
    MeleeAttack, MeleeFactor, MeleeModifier, MovementAllowance, MovementPoints, Phase,
    UnitIdentity, UnitId, UnitMovement, UnitPlacement, UnitProfile, UnitState, WeaponClass,
};

/// A scripted step: either an effect that must be accepted, an effect that must
/// be rejected with a given [`Probe`] shape, or a state predicate.
#[derive(Clone)]
pub enum ScriptStep {
    /// `apply_effect` must return `Ok`.
    Legal {
        note: &'static str,
        effect: GameEffect,
    },
    /// `apply_effect` must return `Err` matching `probe`.
    Illegal {
        note: &'static str,
        probe: Probe,
        effect: GameEffect,
    },
    /// `predicate(&GameState)` must return true.
    Assert {
        note: &'static str,
        predicate: Arc<dyn Fn(&GameState) -> bool>,
    },
}

impl std::fmt::Debug for ScriptStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptStep::Legal { note, effect } => {
                write!(f, "Legal {{ note: {note}, effect: {effect:?} }}")
            }
            ScriptStep::Illegal {
                note,
                probe,
                effect,
            } => write!(f, "Illegal {{ note: {note}, probe: {probe:?}, effect: {effect:?} }}"),
            ScriptStep::Assert { note, .. } => write!(f, "Assert {{ note: {note} }}"),
        }
    }
}

/// Describes which rejection shape an illegal probe expects.
#[derive(Clone)]
pub enum Probe {
    /// Any rejection is acceptable (the effect is illegal, full stop).
    Any(&'static str),
    /// The predicate must hold on the returned error.
    Matches(&'static str, Arc<dyn Fn(&RuleError) -> bool>),
}

impl std::fmt::Debug for Probe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Probe::Any(label) => write!(f, "Any({label})"),
            Probe::Matches(label, _) => write!(f, "Matches({label})"),
        }
    }
}

impl Probe {
    /// Build a `Matches` probe from a capturing closure.
    pub fn matched<F>(label: &'static str, pred: F) -> Self
    where
        F: Fn(&RuleError) -> bool + 'static,
    {
        Probe::Matches(label, Arc::new(pred))
    }

    /// Whether the rejected `err` matches this probe's expectation.
    pub fn matches(&self, err: &RuleError) -> bool {
        match self {
            Probe::Any(_) => true,
            Probe::Matches(_, pred) => pred(err),
        }
    }

    /// Human-readable description for diagnostics.
    pub fn label(&self) -> &'static str {
        match self {
            Probe::Any(label) | Probe::Matches(label, _) => label,
        }
    }
}

/// A named, deterministic rules vignette.
#[derive(Clone, Debug)]
pub struct TacticsScript {
    pub name: &'static str,
    /// Rulebook sections exercised by this script, e.g. `"§5.11, §5.12"`.
    pub citation: &'static str,
    pub state: GameState,
    pub steps: Vec<ScriptStep>,
}

impl TacticsScript {
    pub fn new(name: &'static str, citation: &'static str, state: GameState) -> Self {
        Self {
            name,
            citation,
            state,
            steps: Vec::new(),
        }
    }

    pub fn legal(mut self, note: &'static str, effect: GameEffect) -> Self {
        self.steps.push(ScriptStep::Legal { note, effect });
        self
    }

    pub fn illegal(mut self, note: &'static str, probe: Probe, effect: GameEffect) -> Self {
        self.steps.push(ScriptStep::Illegal { note, probe, effect });
        self
    }

    pub fn assert<F>(mut self, note: &'static str, predicate: F) -> Self
    where
        F: Fn(&GameState) -> bool + 'static,
    {
        self.steps
            .push(ScriptStep::Assert { note, predicate: Arc::new(predicate) });
        self
    }
}

/// Run one scripted step against `state`, mutating it on legal steps.
/// Returns a human-readable failure message, or `None` on success.
pub fn run_step(state: &mut GameState, step: &ScriptStep) -> Option<String> {
    match step {
        ScriptStep::Legal { effect, .. } => match apply_effect(state, effect) {
            Ok(()) => None,
            Err(e) => Some(format!("expected Ok, got {e:?}")),
        },
        ScriptStep::Illegal { probe, effect, .. } => match apply_effect(state, effect) {
            Ok(()) => Some("expected Err, got Ok".to_string()),
            Err(e) if probe.matches(&e) => None,
            Err(e) => Some(format!("expected {}, got {e:?}", probe.label())),
        },
        ScriptStep::Assert { predicate, .. } => {
            if predicate(state) {
                None
            } else {
                Some("predicate returned false".to_string())
            }
        }
    }
}

/// The suite of all tactics scripts, in rulebook order.
pub fn all_scripts() -> Vec<TacticsScript> {
    vec![
        movement_allowance(),
        walled_city_entry_artillery(),
        walled_city_entry_denied(),
        gunboat_river_move(),
        artillery_sinks_gunboat(),
        artillery_destroys_fort(),
        maxim_second_fire(),
        howitzer_on_target(),
        howitzer_scatter_miss(),
        no_howitzer_at_night(),
        retreat_before_melee(),
        infantry_cannot_retreat(),
        melee_edges(),
        artillery_may_not_melee(),
        advance_after_combat(),
        phase_sequence(),
        zone_of_control(),
        stacking_limits(),
        gordon_immobile(),
        disrupted_unit_inert(),
        wrong_owner_cannot_fire(),
        out_of_range(),
    ]
}

// ---------------------------------------------------------------------------
// State construction helpers
// ---------------------------------------------------------------------------

/// The Campaign board as a `BoardInfo`.
fn campaign_board() -> BoardInfo {
    BoardInfo::from_map_data(&campaign_map_data())
}

/// The Fall-of-Khartoum board as a `BoardInfo`.
fn fall_of_khartoum_board() -> BoardInfo {
    BoardInfo::from_map_data(&fall_of_khartoum_map_data())
}

/// A Campaign `GameState` in the given phase, with `active_player` set.
fn campaign_state(phase: Phase, active: Player, day_night: DayNight) -> GameState {
    let mut state = GameState::with_board(Scenario::Campaign, campaign_board());
    state.phase = phase;
    state.active_player = active;
    state.day_night = day_night;
    state
}

/// A Fall-of-Khartoum `GameState` in the given phase, with `active_player` set.
fn fall_of_khartoum_state(phase: Phase, active: Player, day_night: DayNight) -> GameState {
    let mut state = GameState::with_board(Scenario::FallOfKhartoum, fall_of_khartoum_board());
    state.phase = phase;
    state.active_player = active;
    state.day_night = day_night;
    state
}

/// Place a real compiled unit (from the sprite/profile data) at `hex`.
fn place(state: &mut GameState, id: UnitId, hex: HexCoord) {
    let profile = profile_for_unit(id)
        .unwrap_or_else(|| panic!("vignette unit {id:?} has no compiled profile"));
    state.units.push(UnitPlacement {
        id,
        position: hex,
        profile,
        state: UnitState::default(),
    });
}

/// Allocate a synthetic (non-counter) unit id unique within this vignette.
/// Synthetic units stand in for counters the engine does not model as distinct
/// `UnitId`s; their profile carries the real combat values.
fn alloc_synthetic(state: &mut GameState) -> UnitId {
    loop {
        let id = state.alloc_unit_id();
        if !state.units.iter().any(|u| u.id == id) {
            return id;
        }
    }
}

/// A synthetic (non-counter) Anglo-Egyptian infantry unit (8-6-9).
fn ae_infantry(state: &mut GameState, hex: HexCoord) -> UnitId {
    let id = alloc_synthetic(state);
    state.units.push(UnitPlacement {
        id,
        position: hex,
        profile: UnitProfile {
            kind: UnitKind::Infantry {
                fire: 8,
                melee: 6,
                movement: 8,
            },
            identity: UnitIdentity::AngloEgyptianInfantry {
                brigade: BrigadeId::british(1),
                battalion: BattalionOrdinal::First,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(FireFactor::Eight),
            melee: Some(MeleeFactor::Six),
            movement: UnitMovement::Land(MovementAllowance::Eight),
        },
        state: UnitState::default(),
    });
    id
}

/// A synthetic (non-counter) Dervish camel unit, eligible to retreat before
/// melee (§7.5).
fn dervish_camel(state: &mut GameState, hex: HexCoord) -> UnitId {
    let id = alloc_synthetic(state);
    state.units.push(UnitPlacement {
        id,
        position: hex,
        profile: UnitProfile {
            kind: UnitKind::Camel {
                fire: 6,
                melee: 5,
                movement: 10,
            },
            identity: UnitIdentity::DervishTribal {
                tribe: DervishTribe::Baggara,
            },
            weapon: WeaponClass::Rifles,
            fire: Some(FireFactor::Six),
            melee: Some(MeleeFactor::Five),
            movement: UnitMovement::Land(MovementAllowance::Ten),
        },
        state: UnitState::default(),
    });
    id
}

/// A synthetic (non-counter) Anglo-Egyptian Maxim (§6.42): fires in both the
/// Direct and the Maxim Second Fire and Howitzer subphase.
fn ae_maxim(state: &mut GameState, hex: HexCoord) -> UnitId {
    let id = alloc_synthetic(state);
    state.units.push(UnitPlacement {
        id,
        position: hex,
        profile: UnitProfile {
            kind: UnitKind::Maxim {
                fire: 5,
                melee: 1,
                movement: 8,
            },
            identity: UnitIdentity::AngloEgyptianMaxim,
            weapon: WeaponClass::Maxims,
            fire: Some(FireFactor::Five),
            melee: Some(MeleeFactor::One),
            movement: UnitMovement::Land(MovementAllowance::Eight),
        },
        state: UnitState::default(),
    });
    id
}

/// A synthetic (non-counter) Anglo-Egyptian artillery piece.
fn ae_artillery(state: &mut GameState, hex: HexCoord, fire: FireFactor) -> UnitId {
    let id = alloc_synthetic(state);
    state.units.push(UnitPlacement {
        id,
        position: hex,
        profile: UnitProfile {
            kind: UnitKind::Artillery {
                fire: fire.value() as i32,
                melee: 2,
                movement: 4,
            },
            identity: UnitIdentity::AngloEgyptianArtillery,
            weapon: WeaponClass::Artillery,
            fire: Some(fire),
            melee: Some(MeleeFactor::Three),
            movement: UnitMovement::Land(MovementAllowance::Four),
        },
        state: UnitState::default(),
    });
    id
}

/// A synthetic (non-counter) Anglo-Egyptian howitzer (only ever modelled on
/// named gunboats, §6.64) so a vignette can exercise howitzer fire without
/// depending on Nile hex placement.
fn ae_howitzer(state: &mut GameState, hex: HexCoord) -> UnitId {
    let id = alloc_synthetic(state);
    state.units.push(UnitPlacement {
        id,
        position: hex,
        profile: UnitProfile {
            kind: UnitKind::Artillery {
                fire: 5,
                melee: 0,
                movement: 0,
            },
            identity: UnitIdentity::AngloEgyptianArtillery,
            weapon: WeaponClass::Howitzer,
            fire: Some(FireFactor::Five),
            melee: None,
            movement: UnitMovement::Immobile,
        },
        state: UnitState::default(),
    });
    id
}

/// Build a `FireAttack` for a single firer against `target_hex`.
fn fire_attack(
    firing_player: Player,
    phase: Phase,
    kind: FireKind,
    firer: UnitId,
    target_hex: HexCoord,
    modifiers: Vec<FireModifier>,
) -> FireAttack {
    FireAttack {
        firing_player,
        phase,
        kind,
        firers: vec![firer],
        target_hex,
        factor_row: FireFactorRow::Row01to05,
        modifiers,
    }
}

/// Build a `MeleeAttack` with a single attacker and a single defender.
fn melee_attack(
    attacker_player: Player,
    attacker: UnitId,
    attacker_hex: HexCoord,
    defender: UnitId,
    defender_hex: HexCoord,
    attacker_modifiers: Vec<MeleeModifier>,
) -> MeleeAttack {
    MeleeAttack {
        attacker_player,
        attacker_hex,
        defender_hex,
        attackers: vec![attacker],
        defenders: vec![defender],
        attacker_modifiers,
        defender_modifiers: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Vignette builders
// ---------------------------------------------------------------------------

/// §5.11/§5.12: a unit moves hex by hex up to its movement allowance; the
/// running total spent this turn is tracked and cannot be exceeded.
fn movement_allowance() -> TacticsScript {
    let mut state = campaign_state(Phase::Setup, Player::AngloEgyptian, DayNight::Day);
    let mover = ae_infantry(&mut state, HexCoord::new(30, 8));
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 11));
    TacticsScript::new("movement_allowance", "§4, §5.11, §5.12", state)
        .legal("leave Setup once both sides are deployed", GameEffect::AdvancePhase)
        .assert("we are now in the Movement phase", |s| {
            matches!(s.phase, Phase::Movement)
        })
        .legal(
            "infantry (allowance 8) steps one hex at cost 1",
            GameEffect::MoveUnit {
                unit_id: mover,
                to: HexCoord::new(30, 9),
                cost: MovementPoints(1),
                path: Vec::new(),
            },
        )
        .assert("the infantry has spent at least 1 MP", move |s| {
            s.mp_spent(mover) >= 1
        })
        .illegal(
            "a second step costing 8 more would total 9 MP > allowance 8",
            Probe::matched("MovementExceedsAllowance", |e| {
                matches!(e, RuleError::MovementExceedsAllowance { .. })
            }),
            GameEffect::MoveUnit {
                unit_id: mover,
                to: HexCoord::new(30, 10),
                cost: MovementPoints(8),
                path: Vec::new(),
            },
        )
}

/// §5.23: Dervish artillery may enter the walled portion of Omdurman.
fn walled_city_entry_artillery() -> TacticsScript {
    let mut state = campaign_state(Phase::Movement, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::KhalifaAbdullah_0_1, HexCoord::new(30, 40));
    TacticsScript::new("walled_city_entry_artillery", "§5.23", state)
        .assert("(30,40) is outside the walled city", |s| {
            !s.board.is_walled_city(HexCoord::new(30, 40))
        })
        .assert("(30,39) is inside the walled city", |s| {
            s.board.is_walled_city(HexCoord::new(30, 39))
        })
        .legal(
            "Dervish artillery enters the walled city",
            GameEffect::MoveUnit {
                unit_id: UnitId::KhalifaAbdullah_0_1,
                to: HexCoord::new(30, 39),
                cost: MovementPoints(1),
                path: Vec::new(),
            },
        )
        .assert("artillery now stands inside the city", |s| {
            s.find_unit(UnitId::KhalifaAbdullah_0_1)
                .map(|u| u.position)
                == Some(HexCoord::new(30, 39))
        })
}

/// §5.23: a Dervish tribal that is neither the Khalifa, nor artillery, nor the
/// Taiasha bodyguard may not enter the walled city; a wall hexside itself
/// blocks movement.
fn walled_city_entry_denied() -> TacticsScript {
    let mut state = campaign_state(Phase::Movement, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 40));
    place(&mut state, UnitId::Baggara_0_1, HexCoord::new(30, 38));
    TacticsScript::new("walled_city_entry_denied", "§5.23", state)
        .illegal(
            "Baggara tribals may not enter the walled city",
            Probe::matched("WalledCityEntry", |e| {
                matches!(e, RuleError::WalledCityEntry(UnitId::Baggara_0_0, _))
            }),
            GameEffect::MoveUnit {
                unit_id: UnitId::Baggara_0_0,
                to: HexCoord::new(30, 39),
                cost: MovementPoints(1),
                path: Vec::new(),
            },
        )
        .illegal(
            "a wall hexside itself blocks movement",
            Probe::matched("MoveBlockedByHexside", |e| {
                matches!(e, RuleError::MoveBlockedByHexside(_, _))
            }),
            GameEffect::MoveUnit {
                unit_id: UnitId::Baggara_0_1,
                to: HexCoord::new(31, 38),
                cost: MovementPoints(1),
                path: Vec::new(),
            },
        )
}

/// §5.22/§5.24: a gunboat moves along the Nile, paying one MP per hex; it may
/// never leave the river.
fn gunboat_river_move() -> TacticsScript {
    let mut state = campaign_state(Phase::Movement, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::BritishBoats_4_0, HexCoord::new(34, 12));
    TacticsScript::new("gunboat_river_move", "§5.22, §5.24", state)
        .assert("the gunboat starts on the Nile", |s| {
            s.board.is_nile(HexCoord::new(34, 12))
        })
        .legal(
            "the gunboat steams one Nile hex downstream at 1 MP",
            GameEffect::MoveUnit {
                unit_id: UnitId::BritishBoats_4_0,
                to: HexCoord::new(35, 11),
                cost: MovementPoints(1),
                path: vec![HexCoord::new(35, 11)],
            },
        )
        .assert("gunboat spent exactly 1 MP", |s| {
            s.mp_spent(UnitId::BritishBoats_4_0) == 1
        })
        .illegal(
            "a gunboat may not enter a land hex",
            Probe::matched("GunboatOffNile", |e| matches!(e, RuleError::GunboatOffNile(_))),
            GameEffect::MoveUnit {
                unit_id: UnitId::BritishBoats_4_0,
                to: HexCoord::new(30, 8),
                cost: MovementPoints(1),
                path: vec![HexCoord::new(30, 8)],
            },
        )
}

/// §6.61: only artillery (or howitzer) fire may sink a gunboat; a CRT cell of
/// Eliminate(3)+ is required.
fn artillery_sinks_gunboat() -> TacticsScript {
    let mut state = campaign_state(
        Phase::OffensiveFire(FireSubPhase::DirectFire),
        Player::AngloEgyptian,
        DayNight::Day,
    );
    place(&mut state, UnitId::KhalifaAbdullah_1_0, HexCoord::new(34, 12));
    let gun = UnitId::KhalifaAbdullah_1_0;
    let artillery = ae_artillery(&mut state, HexCoord::new(33, 11), FireFactor::Six);
    let rifle = ae_infantry(&mut state, HexCoord::new(32, 11));
    TacticsScript::new("artillery_sinks_gunboat", "§6.22, §6.61", state)
        .illegal(
            "an infantry rifle may not fire at a gunboat",
            Probe::matched("ArtilleryOnlyVsGunboatOrFort", |e| {
                matches!(e, RuleError::ArtilleryOnlyVsGunboatOrFort(_))
            }),
            GameEffect::FireCombat {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::DirectFire),
                    FireKind::Direct,
                    rifle,
                    HexCoord::new(34, 12),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                roll: DieRoll::Ten,
            },
        )
        .legal(
            "artillery at range 2 (doubled factor) rolls 10 and sinks the gunboat",
            GameEffect::FireCombat {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::DirectFire),
                    FireKind::Direct,
                    artillery,
                    HexCoord::new(34, 12),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                roll: DieRoll::Ten,
            },
        )
        .assert("the gunboat has been sunk", move |s| s.find_unit(gun).is_none())
}

/// §6.62: artillery may destroy a fort on a CRT cell of Eliminate(2)+.
fn artillery_destroys_fort() -> TacticsScript {
    let mut state = campaign_state(
        Phase::OffensiveFire(FireSubPhase::DirectFire),
        Player::AngloEgyptian,
        DayNight::Day,
    );
    place(&mut state, UnitId::HadendowaForts_0_0, HexCoord::new(30, 15));
    let fort = UnitId::HadendowaForts_0_0;
    let artillery = ae_artillery(&mut state, HexCoord::new(30, 12), FireFactor::Eight);
    TacticsScript::new("artillery_destroys_fort", "§6.22, §6.62", state)
        .legal(
            "artillery at range 3 (normal factor 8) rolls 10 and destroys the fort",
            GameEffect::FireCombat {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::DirectFire),
                    FireKind::Direct,
                    artillery,
                    HexCoord::new(30, 15),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                roll: DieRoll::Ten,
            },
        )
        .assert("the fort has been destroyed", move |s| s.find_unit(fort).is_none())
}

/// §6.42/§6.14: a Maxim fires in the Direct subphase and again in the Maxim
/// Second Fire and Howitzer subphase; the per-phase fired-set is cleared at the
/// subphase boundary.
fn maxim_second_fire() -> TacticsScript {
    let mut state = campaign_state(
        Phase::OffensiveFire(FireSubPhase::DirectFire),
        Player::AngloEgyptian,
        DayNight::Day,
    );
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 15));
    let maxim = ae_maxim(&mut state, HexCoord::new(30, 12));
    let rifle = ae_infantry(&mut state, HexCoord::new(30, 13));
    TacticsScript::new("maxim_second_fire", "§6.14, §6.42", state)
        .legal(
            "Maxim fires direct (roll 2, no effect)",
            GameEffect::FireCombat {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::DirectFire),
                    FireKind::Direct,
                    maxim,
                    HexCoord::new(30, 15),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                roll: DieRoll::Two,
            },
        )
        .assert("the Maxim is marked as fired", move |s| {
            s.units_fired_this_phase.contains(&maxim)
        })
        .illegal(
            "the Maxim may not fire direct twice in the same subphase",
            Probe::matched("AlreadyFired", move |e| {
                matches!(e, RuleError::AlreadyFired(id) if *id == maxim)
            }),
            GameEffect::FireCombat {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::DirectFire),
                    FireKind::Direct,
                    maxim,
                    HexCoord::new(30, 15),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                roll: DieRoll::Two,
            },
        )
        .legal(
            "advance to the Maxim Second Fire and Howitzer subphase",
            GameEffect::AdvancePhase,
        )
        .assert("the per-phase fired-set was cleared (§6.42)", |s| {
            matches!(
                s.phase,
                Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer)
            ) && s.units_fired_this_phase.is_empty()
        })
        .legal(
            "the Maxim may fire a second time in the second subphase",
            GameEffect::FireCombat {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
                    FireKind::MaximSecondFire,
                    maxim,
                    HexCoord::new(30, 15),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                roll: DieRoll::Two,
            },
        )
        .illegal(
            "a rifle may not use Maxim second fire in the second subphase",
            Probe::matched("WrongWeaponForSubphase", move |e| {
                matches!(e, RuleError::WrongWeaponForSubphase(id) if *id == rifle)
            }),
            GameEffect::FireCombat {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
                    FireKind::MaximSecondFire,
                    rifle,
                    HexCoord::new(30, 15),
                    vec![],
                ),
                roll: DieRoll::Two,
            },
        )
}

/// §6.64: howitzer fire hits its designated hex on an impact roll of 7-10.
fn howitzer_on_target() -> TacticsScript {
    let mut state = campaign_state(
        Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
        Player::AngloEgyptian,
        DayNight::Day,
    );
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 12));
    let target = UnitId::Baggara_0_0;
    let howitzer = ae_howitzer(&mut state, HexCoord::new(34, 12));
    TacticsScript::new("howitzer_on_target", "§6.22, §6.64", state)
        .assert("the howitzer is at range 4 (in band)", |_| {
            HexCoord::new(34, 12).distance(HexCoord::new(30, 12)) == 4
        })
        .legal(
            "howitzer at range 4 fires; CRT roll 10 eliminates the target",
            GameEffect::HowitzerFire {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
                    FireKind::Howitzer,
                    howitzer,
                    HexCoord::new(30, 12),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                combat_results_table_roll: DieRoll::Ten,
                impact_roll: DieRoll::Ten,
            },
        )
        .assert("the target hex was hit and the defender eliminated", move |s| {
            s.find_unit(target).is_none()
        })
}

/// §6.64: a scatter roll below 7 means the shell lands somewhere else and the
/// designated target survives.
fn howitzer_scatter_miss() -> TacticsScript {
    let mut state = campaign_state(
        Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
        Player::AngloEgyptian,
        DayNight::Day,
    );
    place(&mut state, UnitId::Baggara_0_1, HexCoord::new(30, 13));
    let target = UnitId::Baggara_0_1;
    let howitzer = ae_howitzer(&mut state, HexCoord::new(34, 12));
    TacticsScript::new("howitzer_scatter_miss", "§6.64", state)
        .legal(
            "impact roll 2 scatters the shell off-target, sparing the defender",
            GameEffect::HowitzerFire {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
                    FireKind::Howitzer,
                    howitzer,
                    HexCoord::new(30, 13),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                combat_results_table_roll: DieRoll::Ten,
                impact_roll: DieRoll::Two,
            },
        )
        .assert("the designated target hex was missed", move |s| {
            s.find_unit(target).map(|u| u.position) == Some(HexCoord::new(30, 13))
        })
}

/// §6.64/§8.1: no howitzer fire at night.
fn no_howitzer_at_night() -> TacticsScript {
    let mut state = campaign_state(
        Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
        Player::AngloEgyptian,
        DayNight::Night,
    );
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 12));
    let howitzer = ae_howitzer(&mut state, HexCoord::new(34, 12));
    TacticsScript::new("no_howitzer_at_night", "§6.64, §8.1", state)
        .illegal(
            "howitzer fire is forbidden after dark",
            Probe::matched("NoHowitzerAtNight", |e| matches!(e, RuleError::NoHowitzerAtNight)),
            GameEffect::HowitzerFire {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::MaximSecondAndHowitzer),
                    FireKind::Howitzer,
                    howitzer,
                    HexCoord::new(30, 12),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                combat_results_table_roll: DieRoll::Ten,
                impact_roll: DieRoll::Ten,
            },
        )
}

/// §7.5: a cavalry/camel unit may retreat two hexes before an infantry melee
/// attack resolves, once per turn.
fn retreat_before_melee() -> TacticsScript {
    let mut state = campaign_state(Phase::Melee, Player::AngloEgyptian, DayNight::Day);
    let camel = dervish_camel(&mut state, HexCoord::new(31, 13));
    let infantry = ae_infantry(&mut state, HexCoord::new(30, 13));
    let attack = melee_attack(
        Player::AngloEgyptian,
        infantry,
        HexCoord::new(30, 13),
        camel,
        HexCoord::new(31, 13),
        vec![MeleeModifier::AngloEgyptianStandard],
    );
    TacticsScript::new("retreat_before_melee", "§7.5, §7.7", state)
        .legal(
            "an infantry melee is declared against the camel",
            GameEffect::DeclareMelee {
                attack: attack.clone(),
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
        .legal(
            "the camel retreats two hexes before the melee resolves",
            GameEffect::RetreatBeforeMelee {
                unit_id: camel,
                to: HexCoord::new(33, 13),
            },
        )
        .assert("the camel now stands two hexes away", move |s| {
            s.find_unit(camel).map(|u| u.position) == Some(HexCoord::new(33, 13))
        })
        .assert("the retreat cost the camel its move", move |s| {
            s.mp_spent(camel) == 1
        })
        .illegal(
            "after retreating, the camel is no longer under the infantry threat and cannot retreat again",
            Probe::matched("NoInfantryMeleeThreatens", move |e| {
                matches!(e, RuleError::NoInfantryMeleeThreatens(id) if *id == camel)
            }),
            GameEffect::RetreatBeforeMelee {
                unit_id: camel,
                to: HexCoord::new(35, 13),
            },
        )
        .illegal(
            "only one melee may be pending at a time",
            Probe::matched("MeleeAlreadyPending", |e| {
                matches!(e, RuleError::MeleeAlreadyPending)
            }),
            GameEffect::DeclareMelee {
                attack,
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
}

/// §7.5: only cavalry and camel units may retreat before melee.
fn infantry_cannot_retreat() -> TacticsScript {
    let mut state = campaign_state(Phase::Melee, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::Baggara_0_1, HexCoord::new(31, 13));
    let defender = UnitId::Baggara_0_1;
    let infantry = ae_infantry(&mut state, HexCoord::new(30, 13));
    TacticsScript::new("infantry_cannot_retreat", "§7.5", state)
        .legal(
            "an infantry melee is declared against the Baggara",
            GameEffect::DeclareMelee {
                attack: melee_attack(
                    Player::AngloEgyptian,
                    infantry,
                    HexCoord::new(30, 13),
                    defender,
                    HexCoord::new(31, 13),
                    vec![MeleeModifier::AngloEgyptianStandard],
                ),
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
        .illegal(
            "foot infantry may not retreat before melee",
            Probe::matched("MayNotRetreatBeforeMelee", move |e| {
                matches!(e, RuleError::MayNotRetreatBeforeMelee(id) if *id == defender)
            }),
            GameEffect::RetreatBeforeMelee {
                unit_id: defender,
                to: HexCoord::new(33, 13),
            },
        )
}

/// §7.2/§7.4: melee is blocked by a wall hexside, requires adjacency, and only
/// melee-capable kinds may attack.
fn melee_edges() -> TacticsScript {
    let mut state = campaign_state(Phase::Melee, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(29, 40));
    place(&mut state, UnitId::Baggara_0_1, HexCoord::new(30, 9));
    place(&mut state, UnitId::Baggara_1_0, HexCoord::new(30, 12));
    let wall_def = UnitId::Baggara_0_0;
    let near_def = UnitId::Baggara_0_1;
    let far_def = UnitId::Baggara_1_0;
    let wall_att = ae_infantry(&mut state, HexCoord::new(28, 40));
    let infantry = ae_infantry(&mut state, HexCoord::new(30, 8));
    let maxim = ae_maxim(&mut state, HexCoord::new(31, 9));
    TacticsScript::new("melee_edges", "§7.1, §7.2, §7.4", state)
        .illegal(
            "a wall hexside blocks the melee attack",
            Probe::matched("MeleeBlockedByHexside", |e| {
                matches!(
                    e,
                    RuleError::MeleeBlockedByHexside(
                        HexCoord { q: 28, r: 40 },
                        HexCoord { q: 29, r: 40 }
                    )
                )
            }),
            GameEffect::DeclareMelee {
                attack: melee_attack(
                    Player::AngloEgyptian,
                    wall_att,
                    HexCoord::new(28, 40),
                    wall_def,
                    HexCoord::new(29, 40),
                    vec![],
                ),
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
        .illegal(
            "a Maxim is not a melee-attack-capable kind",
            Probe::matched("KindMayNotMelee", move |e| {
                matches!(e, RuleError::KindMayNotMelee(id) if *id == maxim)
            }),
            GameEffect::DeclareMelee {
                attack: melee_attack(
                    Player::AngloEgyptian,
                    maxim,
                    HexCoord::new(31, 9),
                    near_def,
                    HexCoord::new(30, 9),
                    vec![],
                ),
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
        .illegal(
            "the target hex must be adjacent",
            Probe::matched("TargetNotAdjacent", |e| {
                matches!(e, RuleError::TargetNotAdjacent { .. })
            }),
            GameEffect::DeclareMelee {
                attack: melee_attack(
                    Player::AngloEgyptian,
                    infantry,
                    HexCoord::new(30, 8),
                    far_def,
                    HexCoord::new(30, 12),
                    vec![],
                ),
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
        .illegal(
            "the target hex must contain a meleeable enemy",
            Probe::matched("NoMeleeableEnemy", |e| {
                matches!(
                    e,
                    RuleError::NoMeleeableEnemy(HexCoord { q: 31, r: 9 })
                )
            }),
            GameEffect::DeclareMelee {
                attack: melee_attack(
                    Player::AngloEgyptian,
                    infantry,
                    HexCoord::new(30, 8),
                    maxim,
                    HexCoord::new(31, 9),
                    vec![],
                ),
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
}

/// §7.4: artillery may defend in melee but never attack.
fn artillery_may_not_melee() -> TacticsScript {
    let mut state = campaign_state(Phase::Melee, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 9));
    let defender = UnitId::Baggara_0_0;
    let artillery = ae_artillery(&mut state, HexCoord::new(30, 8), FireFactor::Six);
    TacticsScript::new("artillery_may_not_melee", "§7.4", state)
        .illegal(
            "artillery may not launch a melee attack",
            Probe::matched("KindMayNotMelee", move |e| {
                matches!(e, RuleError::KindMayNotMelee(id) if *id == artillery)
            }),
            GameEffect::DeclareMelee {
                attack: melee_attack(
                    Player::AngloEgyptian,
                    artillery,
                    HexCoord::new(30, 8),
                    defender,
                    HexCoord::new(30, 9),
                    vec![],
                ),
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
}

/// §6.82/§7.6: after combat clears a hex, an adjacent non-artillery attacker
/// may advance into it.
fn advance_after_combat() -> TacticsScript {
    let mut state = campaign_state(Phase::Melee, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 9));
    let defender = UnitId::Baggara_0_0;
    let infantry = ae_infantry(&mut state, HexCoord::new(30, 8));
    let artillery = ae_artillery(&mut state, HexCoord::new(30, 7), FireFactor::Six);
    TacticsScript::new("advance_after_combat", "§6.82, §7.6, §7.7", state)
        .legal(
            "melee resolves (roll 10 vs 1) and eliminates the defender",
            GameEffect::MeleeCombat {
                attack: melee_attack(
                    Player::AngloEgyptian,
                    infantry,
                    HexCoord::new(30, 8),
                    defender,
                    HexCoord::new(30, 9),
                    vec![MeleeModifier::AngloEgyptianStandard],
                ),
                attacker_roll: DieRoll::Ten,
                defender_roll: DieRoll::One,
            },
        )
        .assert("the defender hex is now vacant", |s| {
            !s.units.iter().any(|u| u.position == HexCoord::new(30, 9))
        })
        .legal(
            "the attacker advances into the vacated hex",
            GameEffect::AdvanceAfterCombat {
                unit_id: infantry,
                to: HexCoord::new(30, 9),
            },
        )
        .assert("the attacker now holds the former defender hex", move |s| {
            s.find_unit(infantry).map(|u| u.position) == Some(HexCoord::new(30, 9))
        })
        .illegal(
            "artillery may not advance after combat",
            Probe::matched("ArtilleryMayNotAdvance", move |e| {
                matches!(e, RuleError::ArtilleryMayNotAdvance(id) if *id == artillery)
            }),
            GameEffect::AdvanceAfterCombat {
                unit_id: artillery,
                to: HexCoord::new(30, 9),
            },
        )
}

/// §4: the Campaign player-turn flows through every phase in order.
fn phase_sequence() -> TacticsScript {
    let mut state = campaign_state(Phase::Setup, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 8));
    place(&mut state, UnitId::Kitchener_3_0, HexCoord::new(30, 9));
    TacticsScript::new("phase_sequence", "§4", state)
        .assert("starts in Setup", |s| matches!(s.phase, Phase::Setup))
        .legal("Setup -> Movement", GameEffect::AdvancePhase)
        .legal("Movement -> Defensive Fire (Direct)", GameEffect::AdvancePhase)
        .legal(
            "Defensive Fire -> Offensive Fire (Direct)",
            GameEffect::AdvancePhase,
        )
        .legal(
            "Offensive Fire -> Maxim Second Fire and Howitzer subphase",
            GameEffect::AdvancePhase,
        )
        .legal("second subphase -> Melee", GameEffect::AdvancePhase)
        .assert("we are in Melee at the end of the AE turn", |s| {
            matches!(s.phase, Phase::Melee)
        })
        .legal(
            "Melee ends the turn: the Dervish player begins Movement",
            GameEffect::AdvancePhase,
        )
        .assert("the Dervish player now moves", |s| {
            matches!(s.phase, Phase::Movement) && s.active_player == Player::Dervish
        })
}

/// §5.26/§5.43: a unit may enter an enemy ZOC and stop, but may not pass
/// through one.
fn zone_of_control() -> TacticsScript {
    let mut state = campaign_state(Phase::Movement, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 8));
    let infantry = ae_infantry(&mut state, HexCoord::new(30, 10));
    TacticsScript::new("zone_of_control", "§5.26, §5.43", state)
        .assert("the Baggara at (30,8) projects ZOC into (30,9)", |s| {
            s.hex_in_enemy_zoc(
                HexCoord::new(30, 9),
                Player::AngloEgyptian,
                UnitKind::Infantry {
                    fire: 8,
                    melee: 6,
                    movement: 8,
                },
            )
        })
        .illegal(
            "a move may not pass through the ZOC hex (30,9)",
            Probe::matched("BlockedByEnemyZoc", |e| {
                matches!(e, RuleError::BlockedByEnemyZoc(HexCoord { q: 30, r: 9 }))
            }),
            GameEffect::MoveUnit {
                unit_id: infantry,
                to: HexCoord::new(30, 8),
                cost: MovementPoints(2),
                path: vec![HexCoord::new(30, 9), HexCoord::new(30, 8)],
            },
        )
        .legal(
            "a unit may enter a ZOC hex and stop",
            GameEffect::MoveUnit {
                unit_id: infantry,
                to: HexCoord::new(30, 9),
                cost: MovementPoints(1),
                path: vec![HexCoord::new(30, 9)],
            },
        )
        .assert("the unit stopped in the enemy ZOC", move |s| {
            s.find_unit(infantry).map(|u| u.position) == Some(HexCoord::new(30, 9))
        })
}

/// §5.51-§5.53: at most four non-leader units per hex; different Dervish
/// tribes may not share a hex.
fn stacking_limits() -> TacticsScript {
    let mut state = campaign_state(Phase::Movement, Player::AngloEgyptian, DayNight::Day);
    for _ in 0..4 {
        ae_infantry(&mut state, HexCoord::new(30, 8));
    }
    let mover = ae_infantry(&mut state, HexCoord::new(30, 9));
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 10));
    place(&mut state, UnitId::Jehadia_0_0, HexCoord::new(30, 11));
    let baggara = UnitId::Baggara_0_0;
    TacticsScript::new("stacking_limits", "§5.51, §5.52", state)
        .assert("four units already stack in (30,8)", |s| {
            s.units
                .iter()
                .filter(|u| u.position == HexCoord::new(30, 8))
                .count()
                == 4
        })
        .illegal(
            "a fifth unit may not enter the four-unit stack",
            Probe::matched("OverLimit", |e| {
                matches!(e, RuleError::Stacking(crate::StackingError::OverLimit))
            }),
            GameEffect::MoveUnit {
                unit_id: mover,
                to: HexCoord::new(30, 8),
                cost: MovementPoints(1),
                path: vec![HexCoord::new(30, 8)],
            },
        )
        .illegal(
            "a Baggara may not stack with Jehadia units",
            Probe::matched("DervishTribeMix", |e| {
                matches!(e, RuleError::Stacking(crate::StackingError::DervishTribeMix))
            }),
            GameEffect::MoveUnit {
                unit_id: baggara,
                to: HexCoord::new(30, 11),
                cost: MovementPoints(1),
                path: vec![HexCoord::new(30, 11)],
            },
        )
}

/// §9.346: the GORDON leader may not move during FALL OF KHARTOUM.
fn gordon_immobile() -> TacticsScript {
    let mut state = fall_of_khartoum_state(Phase::Movement, Player::Dervish, DayNight::Day);
    place(&mut state, UnitId::BritishBoats_3_1, HexCoord::new(13, 5));
    TacticsScript::new("gordon_immobile", "§9.346", state)
        .illegal(
            "GORDON may not move once FALL OF KHARTOUM has begun",
            Probe::matched("GordonMayNotMove", |e| matches!(e, RuleError::GordonMayNotMove)),
            GameEffect::MoveUnit {
                unit_id: UnitId::BritishBoats_3_1,
                to: HexCoord::new(14, 5),
                cost: MovementPoints(1),
                path: Vec::new(),
            },
        )
}

/// §5: a disrupted unit may not move, fire, or melee.
fn disrupted_unit_inert() -> TacticsScript {
    let mut state = campaign_state(Phase::Movement, Player::AngloEgyptian, DayNight::Day);
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 8));
    let unit = UnitId::Baggara_0_0;
    if let Some(u) = state.units.last_mut() {
        u.state.disrupted = true;
    }
    TacticsScript::new("disrupted_unit_inert", "§5", state)
        .illegal(
            "a disrupted unit may not move",
            Probe::matched("Disrupted", move |e| {
                matches!(e, RuleError::Disrupted(id) if *id == unit)
            }),
            GameEffect::MoveUnit {
                unit_id: unit,
                to: HexCoord::new(30, 9),
                cost: MovementPoints(1),
                path: Vec::new(),
            },
        )
}

/// §6.11: only the side whose turn it is may fire offensively.
fn wrong_owner_cannot_fire() -> TacticsScript {
    let mut state = campaign_state(
        Phase::OffensiveFire(FireSubPhase::DirectFire),
        Player::AngloEgyptian,
        DayNight::Day,
    );
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 9));
    let defender = UnitId::Baggara_0_0;
    ae_infantry(&mut state, HexCoord::new(30, 8));
    TacticsScript::new("wrong_owner_cannot_fire", "§6.11", state)
        .illegal(
            "a Dervish unit may not fire during the AE offensive fire phase",
            Probe::matched("NotYourTurn", |e| matches!(e, RuleError::NotYourTurn)),
            GameEffect::FireCombat {
                attack: fire_attack(
                    Player::Dervish,
                    Phase::OffensiveFire(FireSubPhase::DirectFire),
                    FireKind::Direct,
                    defender,
                    HexCoord::new(30, 8),
                    vec![],
                ),
                roll: DieRoll::Ten,
            },
        )
}

/// §6.22: targets beyond a weapon's range band are unreachable.
fn out_of_range() -> TacticsScript {
    let mut state = campaign_state(
        Phase::OffensiveFire(FireSubPhase::DirectFire),
        Player::AngloEgyptian,
        DayNight::Day,
    );
    place(&mut state, UnitId::Baggara_0_0, HexCoord::new(30, 15));
    let rifle = ae_infantry(&mut state, HexCoord::new(30, 8));
    TacticsScript::new("out_of_range", "§6.22", state)
        .illegal(
            "AE rifles (max range 5) cannot reach a target 7 hexes away",
            Probe::matched("TargetOutOfRange", |e| {
                matches!(e, RuleError::TargetOutOfRange { .. })
            }),
            GameEffect::FireCombat {
                attack: fire_attack(
                    Player::AngloEgyptian,
                    Phase::OffensiveFire(FireSubPhase::DirectFire),
                    FireKind::Direct,
                    rifle,
                    HexCoord::new(30, 15),
                    vec![FireModifier::AngloEgyptianDirectFire],
                ),
                roll: DieRoll::Ten,
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every script must at least build; the behavioural assertions live in
    /// `omdurman-rules/tests/tactics.rs`.
    #[test]
    fn all_scripts_construct() {
        for script in all_scripts() {
            assert!(!script.name.is_empty(), "unnamed script");
            assert!(
                !script.citation.is_empty(),
                "uncited script {:?}",
                script.name
            );
            assert!(
                !script.steps.is_empty(),
                "script {:?} has no steps",
                script.name
            );
        }
    }
}

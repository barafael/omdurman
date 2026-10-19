//! The owner's choice of losses (rulebook §7.7, §CombatResults: "# = That
//! many units in the target hex are eliminated"). The manual does not say
//! who picks them; the owner does, as the wargame convention has it. A
//! combat whose result takes fewer units than could fall -- and not from
//! interchangeable ones -- stops before it is applied: its owner answers
//! with [`GameEffect::TakeLosses`], and the rest of the resolution (the
//! eliminations, the records, the advance windows, the §7.6 forced advance)
//! runs with that choice. Nothing else may happen meanwhile.
//!
//! Combats wait in [`GameState::combat_queue`] behind an open choice (a
//! howitzer attack resolves several shells, an end of turn several
//! demolitions); each is examined only when its turn comes, on the board as
//! the earlier ones left it.

use super::*;

/// One side's open choice of losses (§7.7).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LossChoice {
    /// Whose units fall: the owner chooses.
    pub player: Player,
    /// How many of `candidates` fall.
    pub count: u8,
    /// The units the losses may be taken from -- after any that fall in any
    /// case (§7.7: "from meleeing units first").
    pub candidates: Vec<UnitId>,
    /// The owner's pick, once made.
    pub chosen: Option<Vec<UnitId>>,
}

/// A combat resolution waiting to be applied (§7.7).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum CombatTail {
    /// A fire attack's result on `target_units` in `target_hex` (§6.22).
    Fire {
        attack: FireAttack,
        target_hex: HexCoord,
        target_units: Vec<UnitId>,
        roll: DieRoll,
        disruption: DisruptionDraw,
        impact: Option<(DieRoll, HexCoord)>,
    },
    /// A melee's two simultaneous results (§7.3, §7.7).
    Melee {
        attack: MeleeAttack,
        attacker_roll: DieRoll,
        defender_roll: DieRoll,
        disruption: DisruptionDraw,
    },
    /// "One unit is eliminated" among `candidates` (§6.53, §6.63).
    Eliminate {
        player: Player,
        candidates: Vec<UnitId>,
        cause: ElimCause,
    },
}

/// A combat stopped for its owners' choice of losses.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PendingLosses {
    pub choices: Vec<LossChoice>,
    pub tail: CombatTail,
}

impl GameState {
    /// The first choice of losses still to be made (§7.7), if a combat
    /// waits on one.
    pub fn open_loss_choice(&self) -> Option<&LossChoice> {
        self.pending_losses
            .as_ref()?
            .choices
            .iter()
            .find(|c| c.chosen.is_none())
    }
}

/// Whether one unit of `units` falling is as good as any other: the same
/// counter in the same condition in the same hex.
fn interchangeable(state: &GameState, units: &[UnitId]) -> bool {
    let mut placements = units.iter().filter_map(|id| state.find_unit(*id));
    let Some(first) = placements.next() else {
        return true;
    };
    placements.all(|u| {
        u.profile == first.profile && u.state == first.state && u.position == first.position
    })
}

/// The choice `n` losses leave `player` among `tiers`, taken in order (§7.7:
/// "Melee losses must be taken from meleeing units first!"): whole tiers
/// fall, and in the tier the losses run out in the owner picks -- unless
/// its units are interchangeable.
pub(crate) fn loss_choice(
    state: &GameState,
    player: Player,
    tiers: &[Vec<UnitId>],
    n: usize,
) -> Option<LossChoice> {
    let mut left = n;
    for tier in tiers {
        if left == 0 {
            return None;
        }
        if left >= tier.len() {
            left -= tier.len();
            continue;
        }
        if interchangeable(state, tier) {
            return None;
        }
        return Some(LossChoice {
            player,
            count: left as u8,
            candidates: tier.clone(),
            chosen: None,
        });
    }
    None
}

/// The order losses fall in, tier by tier (§7.7): within the tier holding
/// the owner's pick, the pick first, then the rest of the tier.
pub(crate) fn casualty_order(tiers: &[Vec<UnitId>], chosen: Option<&[UnitId]>) -> Vec<UnitId> {
    let chosen = chosen.unwrap_or_default();
    let mut order = Vec::new();
    for tier in tiers {
        order.extend(chosen.iter().copied().filter(|id| tier.contains(id)));
        order.extend(tier.iter().copied().filter(|id| !chosen.contains(id)));
    }
    order
}

/// The owner's choices `tail` calls for now, on the board as it stands.
fn choices_for(state: &GameState, tail: &CombatTail) -> Vec<LossChoice> {
    match tail {
        CombatTail::Fire {
            attack,
            target_hex,
            target_units,
            roll,
            ..
        } => {
            let outcome = fire_outcome(state, attack, *target_hex, target_units, *roll);
            let owner = |ids: &[UnitId]| {
                ids.first()
                    .and_then(|id| state.find_unit(*id))
                    .map(|u| u.profile.identity.owner())
            };
            let choice = match (outcome.special, outcome.result) {
                // §6.62: a destroyed fort takes one of its occupants.
                (Some((fort, UnitKind::Fort { .. }, needed)), CombatResult::Eliminate(n))
                    if n >= needed =>
                {
                    let occupants = fort_occupants(state, target_units, fort);
                    owner(&occupants).and_then(|p| loss_choice(state, p, &[occupants], 1))
                }
                (Some(_), _) => None,
                (None, CombatResult::Eliminate(n)) => {
                    let tier = fire_casualty_candidates(state, target_units);
                    owner(&tier).and_then(|p| loss_choice(state, p, &[tier], n as usize))
                }
                (None, _) => None,
            };
            choice.into_iter().collect()
        }
        CombatTail::Melee {
            attack,
            attacker_roll,
            defender_roll,
            ..
        } => {
            let outcome = melee_outcome(state, attack, *attacker_roll, *defender_roll);
            let defender = attack.attacker_player.opponent();
            let mut choices = Vec::new();
            if let CombatResult::Eliminate(n) = outcome.att_result
                && let Some(c) = loss_choice(state, defender, &outcome.def_tiers, n as usize)
            {
                choices.push(c);
            }
            if let CombatResult::Eliminate(n) = outcome.def_result
                && let Some(c) = loss_choice(
                    state,
                    attack.attacker_player,
                    &outcome.att_tiers,
                    n as usize,
                )
            {
                choices.push(c);
            }
            choices
        }
        CombatTail::Eliminate {
            player, candidates, ..
        } => {
            let alive: Vec<UnitId> = candidates
                .iter()
                .copied()
                .filter(|id| state.find_unit(*id).is_some())
                .collect();
            loss_choice(state, *player, &[alive], 1)
                .into_iter()
                .collect()
        }
    }
}

/// Apply `tail` with the owners' `choices` (empty: none was needed).
fn finish(state: &mut GameState, tail: CombatTail, choices: &[LossChoice]) {
    let chosen_by = |player: Player| {
        choices
            .iter()
            .find(|c| c.player == player)
            .and_then(|c| c.chosen.clone())
    };
    match tail {
        CombatTail::Fire {
            attack,
            target_hex,
            target_units,
            roll,
            disruption,
            impact,
        } => {
            let chosen = choices.first().and_then(|c| c.chosen.clone());
            commit_fire_attack(
                state,
                &attack,
                target_hex,
                &target_units,
                roll,
                disruption,
                impact,
                chosen.as_deref(),
            );
        }
        CombatTail::Melee {
            attack,
            attacker_roll,
            defender_roll,
            disruption,
        } => {
            let defenders = chosen_by(attack.attacker_player.opponent());
            let attackers = chosen_by(attack.attacker_player);
            resolve_melee_combat(
                state,
                &attack,
                attacker_roll,
                defender_roll,
                disruption,
                [defenders.as_deref(), attackers.as_deref()],
            );
        }
        CombatTail::Eliminate {
            candidates, cause, ..
        } => {
            let alive: Vec<UnitId> = candidates
                .iter()
                .copied()
                .filter(|id| state.find_unit(*id).is_some())
                .collect();
            let victim = choices
                .first()
                .and_then(|c| c.chosen.as_ref())
                .and_then(|c| c.first().copied())
                .or_else(|| super::fire::first_casualty(state, &alive));
            if let Some(victim) = victim {
                eliminate_unit(state, victim, cause);
            }
        }
    }
}

/// "One unit is eliminated" among `candidates` (§6.53, §6.63): at once when
/// it makes no difference which, else by its owner's choice (§7.7). The
/// unit eliminated at once, if any.
pub(crate) fn eliminate_one(
    state: &mut GameState,
    player: Player,
    candidates: Vec<UnitId>,
    cause: ElimCause,
) -> Option<UnitId> {
    if candidates.is_empty() {
        return None;
    }
    resolve_combat(
        state,
        CombatTail::Eliminate {
            player,
            candidates: candidates.clone(),
            cause,
        },
    );
    candidates
        .into_iter()
        .find(|id| state.find_unit(*id).is_none())
}

/// Queue a combat resolution and apply what can be applied (§7.7).
pub(crate) fn resolve_combat(state: &mut GameState, tail: CombatTail) {
    state.combat_queue.push(tail);
    drain_combat_queue(state);
}

/// Apply the queued resolutions in order until one waits on a choice.
fn drain_combat_queue(state: &mut GameState) {
    while state.pending_losses.is_none() && !state.combat_queue.is_empty() {
        let tail = state.combat_queue.remove(0);
        let choices = choices_for(state, &tail);
        if choices.is_empty() {
            finish(state, tail, &[]);
        } else {
            for choice in &choices {
                state.observations.push(Observation::LossesToChoose {
                    player: choice.player,
                    count: choice.count,
                    candidates: choice.candidates.clone(),
                });
            }
            state.pending_losses = Some(PendingLosses { choices, tail });
        }
    }
}

/// `player` takes the losses of the open choice (§7.7): exactly its count,
/// all from its candidates, each once. When every side has chosen, the
/// combat is applied and the queue runs on.
pub fn apply_take_losses(
    state: &mut GameState,
    player: Player,
    units: &[UnitId],
) -> Result<(), RuleError> {
    let Some(pending) = state.pending_losses.as_ref() else {
        return Err(RuleError::NoLossesToTake);
    };
    let Some(index) = pending
        .choices
        .iter()
        .position(|c| c.player == player && c.chosen.is_none())
    else {
        return Err(RuleError::NoLossesToTake);
    };
    let choice = &pending.choices[index];
    reject_duplicate_units(units)?;
    if units.len() != usize::from(choice.count) {
        return Err(RuleError::WrongLossCount {
            expected: choice.count,
            got: units.len(),
        });
    }
    if let Some(&stranger) = units.iter().find(|id| !choice.candidates.contains(id)) {
        return Err(RuleError::NotALossCandidate(stranger));
    }
    // ---- validation complete; from here on the state is mutated ----
    let Some(mut pending) = state.pending_losses.take() else {
        return Err(RuleError::NoLossesToTake);
    };
    pending.choices[index].chosen = Some(units.to_vec());
    if pending.choices.iter().any(|c| c.chosen.is_none()) {
        state.pending_losses = Some(pending);
        return Ok(());
    }
    finish(state, pending.tail, &pending.choices);
    drain_combat_queue(state);
    Ok(())
}

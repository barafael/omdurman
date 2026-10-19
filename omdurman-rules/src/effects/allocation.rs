//! Fire allocation (rulebook §6.41, §6.42): "The firing player must first
//! allocate all of his fire attacks ... After all fire has been allocated,
//! the firing player then resolves his attacks in any order he wishes."
//!
//! [`GameEffect::AllocateFire`] records a subphase's fire orders in
//! [`GameState::fire_allocation`]; each resolution effect (`FireCombat`,
//! `HowitzerFire`, `ArtilleryBreachWall`, `SinkChain`) then claims its order.
//! Once the first order resolves the allocation is closed. A resolution that
//! was never allocated is allowed while the allocation is still open -- it
//! joins the allocation and closes it, so fire resolved one shot at a time
//! has allocated exactly that shot. The phase may not end while an
//! allocated order can still be resolved.

use super::*;

/// One fire order of the firing player's allocation (§6.41, §6.42).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FireOrder {
    /// Fire at an enemy-occupied hex: resolved by `FireCombat`, or by
    /// `HowitzerFire` for a howitzer attack.
    Attack(FireAttack),
    /// §6.63 artillery fire at a wall hexside: `ArtilleryBreachWall`.
    BreachWall {
        firers: Vec<UnitId>,
        target: HexsideRef,
    },
    /// §10.23 b artillery fire at the river chain: `SinkChain`.
    SinkChain { firers: Vec<UnitId> },
}

impl FireOrder {
    /// The weapons the order fires.
    pub fn shots(&self) -> Vec<Shot> {
        match self {
            FireOrder::Attack(attack) => attack.shots(),
            FireOrder::BreachWall { firers, .. } | FireOrder::SinkChain { firers } => firers
                .iter()
                .map(|&unit| Shot {
                    unit,
                    mount: FireMount::Main,
                })
                .collect(),
        }
    }
}

impl FireOrder {
    /// The effect resolving this order, its dice rolled from `rng` now --
    /// every howitzer shell its own (§6.64).
    pub fn resolution(&self, state: &GameState, rng: &mut crate::rng::GameRng) -> GameEffect {
        match self {
            FireOrder::Attack(attack) if attack.kind == FireKind::Howitzer => {
                let shells = shell_gunboats(state, attack)
                    .iter()
                    .map(|_| ShellRoll {
                        impact: rng.roll_d10(),
                        roll: rng.roll_d10(),
                        disruption: rng.disruption_draw(),
                    })
                    .collect();
                GameEffect::HowitzerFire {
                    attack: attack.clone(),
                    shells,
                    roll: rng.roll_d10(),
                    disruption: rng.disruption_draw(),
                }
            }
            FireOrder::Attack(attack) => GameEffect::FireCombat {
                attack: attack.clone(),
                roll: rng.roll_d10(),
                disruption: rng.disruption_draw(),
            },
            FireOrder::BreachWall { firers, target } => GameEffect::ArtilleryBreachWall {
                firers: firers.clone(),
                target: *target,
                roll: rng.roll_d10(),
            },
            FireOrder::SinkChain { firers } => GameEffect::SinkChain {
                firers: firers.clone(),
                roll: rng.roll_d10(),
            },
        }
    }
}

/// An order of the current subphase's allocation, and whether it has been
/// resolved.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AllocatedFire {
    pub order: FireOrder,
    pub resolved: bool,
}

/// How a resolution effect stands to the allocation ([`claim_fire_order`]).
pub(crate) enum FireClaim {
    /// The unresolved allocated order at this index.
    Allocated(usize),
    /// An order resolved without allocation, joining it.
    Unallocated(FireOrder),
}

impl GameState {
    /// Whether the current subphase's fire has begun resolving: nothing more
    /// may be allocated (§6.41).
    pub fn fire_resolution_begun(&self) -> bool {
        self.fire_allocation.iter().any(|a| a.resolved)
    }

    /// Read-only: whether `order` may be resolved now -- every check its
    /// resolution effect makes before rolling.
    pub fn validate_fire_order(&self, order: &FireOrder) -> Result<(), RuleError> {
        match order {
            FireOrder::Attack(attack) => validate_fire_resolution(self, attack),
            FireOrder::BreachWall { firers, target } => {
                validate_breach_wall(self, firers, *target).map(|_| ())
            }
            FireOrder::SinkChain { firers } => validate_sink_chain(self, firers).map(|_| ()),
        }
    }

    /// The first allocated order not yet resolved that may still be
    /// resolved: the phase may not end before it is (§6.41). An order the
    /// fire already resolved has made impossible -- its target gone, its
    /// firers disrupted by a scattered shell -- does not hold the phase.
    pub fn unresolved_fire(&self) -> Option<&FireOrder> {
        self.fire_allocation
            .iter()
            .filter(|a| !a.resolved)
            .map(|a| &a.order)
            .find(|order| self.validate_fire_order(order).is_ok())
    }
}

/// §6.13/§6.14: across one allocation every weapon fires once, and every
/// target -- the units of a hex, a wall hexside, the chain -- is fired at
/// once.
fn check_disjoint(state: &GameState, orders: &[&FireOrder]) -> Result<(), RuleError> {
    let mut shots: Vec<Shot> = Vec::new();
    let mut targets: Vec<UnitId> = Vec::new();
    let mut walls: Vec<HexsideRef> = Vec::new();
    let mut chain = false;
    for order in orders {
        for shot in order.shots() {
            if shots.contains(&shot) {
                return Err(RuleError::FireAllocatedTwice(shot.unit));
            }
            shots.push(shot);
        }
        match order {
            FireOrder::Attack(attack) => {
                for unit in fire_target_units(state, attack, attack.target_hex) {
                    if targets.contains(&unit) {
                        return Err(RuleError::TargetAllocatedTwice(attack.target_hex));
                    }
                    targets.push(unit);
                }
            }
            FireOrder::BreachWall { target, .. } => {
                let same = |w: &HexsideRef| {
                    (w.a, w.b) == (target.a, target.b) || (w.a, w.b) == (target.b, target.a)
                };
                if walls.iter().any(same) {
                    return Err(RuleError::TargetAllocatedTwice(target.a));
                }
                walls.push(*target);
            }
            FireOrder::SinkChain { .. } => {
                if chain {
                    return Err(RuleError::ChainAllocatedTwice);
                }
                chain = true;
            }
        }
    }
    Ok(())
}

/// Allocate fire orders for the current subphase (§6.41, §6.42): legal only
/// before any of the subphase's fire has been resolved; every order must be
/// resolvable now, and the whole allocation -- earlier orders included, a
/// teammate's among them -- must fire each weapon and strike each target
/// once.
pub fn apply_allocate_fire(state: &mut GameState, orders: &[FireOrder]) -> Result<(), RuleError> {
    state.fire_phase_player()?;
    if orders.is_empty() {
        return Err(RuleError::NoFireOrders);
    }
    if state.fire_resolution_begun() {
        return Err(RuleError::FireAllocationClosed);
    }
    for order in orders {
        state.validate_fire_order(order)?;
    }
    let all: Vec<&FireOrder> = state
        .fire_allocation
        .iter()
        .map(|a| &a.order)
        .chain(orders)
        .collect();
    check_disjoint(state, &all)?;
    // ---- validation complete; from here on the state is mutated ----
    state
        .fire_allocation
        .extend(orders.iter().cloned().map(|order| AllocatedFire {
            order,
            resolved: false,
        }));
    Ok(())
}

/// Read-only: the claim a resolution effect makes on the allocation -- its
/// allocated order, or, while the allocation is still open, an order of its
/// own joining it. `Err(FireAllocationClosed)` once resolution has begun
/// and the order was never allocated.
pub(crate) fn claim_fire_order(
    state: &GameState,
    order: &FireOrder,
) -> Result<FireClaim, RuleError> {
    if let Some(i) = state
        .fire_allocation
        .iter()
        .position(|a| !a.resolved && a.order == *order)
    {
        return Ok(FireClaim::Allocated(i));
    }
    if state.fire_resolution_begun() {
        return Err(RuleError::FireAllocationClosed);
    }
    let all: Vec<&FireOrder> = state
        .fire_allocation
        .iter()
        .map(|a| &a.order)
        .chain(std::iter::once(order))
        .collect();
    check_disjoint(state, &all)?;
    Ok(FireClaim::Unallocated(order.clone()))
}

/// Mark a claimed order resolved (the claim of [`claim_fire_order`]).
pub(crate) fn commit_fire_order(state: &mut GameState, claim: FireClaim) {
    match claim {
        FireClaim::Allocated(i) => state.fire_allocation[i].resolved = true,
        FireClaim::Unallocated(order) => state.fire_allocation.push(AllocatedFire {
            order,
            resolved: true,
        }),
    }
}

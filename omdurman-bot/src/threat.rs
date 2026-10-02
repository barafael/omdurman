//! Fire lanes: how much enemy fire can reach a hex (§6.22 range bands, §8.1
//! night ranges, §6.3 line of sight over the terrain).
//!
//! The Dervish commander uses it to stage out of the Maxims' and gunboats'
//! reach and to cross the killing ground only on the way into contact; it is
//! the "where will I be shot from" half of the defensive fire (§6.7) a unit
//! ends its move in.
//!
//! Units are left out of the line-of-sight test (the terrain alone decides),
//! so the map depends only on the enemy's positions, the board and the time
//! of day -- it is computed lazily per hex and cached until those change,
//! which keeps a long movement phase cheap.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use omdurman_rules::effects::{GameState, range_band_for};
use omdurman_rules::los_table::{has_los, los_level_for_unit};
use omdurman_rules::range_effects::night_range_effects;
use omdurman_rules::{FireKind, HexDistance, RangeBand};
use omdurman_types::{DayNight, HexCoord, Player, UnitKind};

struct Cache {
    key: u64,
    hexes: HashMap<HexCoord, f32>,
}

thread_local! {
    static CACHE: RefCell<Option<Cache>> = const { RefCell::new(None) };
}

fn key(state: &GameState, shooter: Player) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (state.scenario as u8).hash(&mut h);
    (state.day_night == DayNight::Night).hash(&mut h);
    shooter.hash(&mut h);
    state.breaches.len().hash(&mut h);
    for u in &state.units {
        if u.profile.identity.owner() == shooter && u.profile.fire.is_some() {
            u.id.hash(&mut h);
            u.position.hash(&mut h);
            u.state.disrupted.hash(&mut h);
        }
    }
    h.finish()
}

fn band_multiplier(band: RangeBand) -> f32 {
    match band {
        RangeBand::Tripled => 3.0,
        RangeBand::Doubled => 2.0,
        RangeBand::Normal => 1.0,
        RangeBand::Halved => 0.5,
        RangeBand::OutOfRange => 0.0,
    }
}

/// Effective fire factors of `shooter`'s undisrupted units that can reach
/// `hex` with direct fire (a foot unit standing there as the target).
pub fn fire_reaching(state: &GameState, hex: HexCoord, shooter: Player) -> f32 {
    let k = key(state, shooter);
    if let Some(v) = CACHE.with(|c| {
        c.borrow()
            .as_ref()
            .filter(|c| c.key == k)
            .and_then(|c| c.hexes.get(&hex).copied())
    }) {
        return v;
    }
    let v = compute(state, hex, shooter);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        match c.as_mut() {
            Some(cache) if cache.key == k => {
                cache.hexes.insert(hex, v);
            }
            _ => {
                let mut hexes = HashMap::new();
                hexes.insert(hex, v);
                *c = Some(Cache { key: k, hexes });
            }
        }
    });
    v
}

fn compute(state: &GameState, hex: HexCoord, shooter: Player) -> f32 {
    let night = state.day_night == DayNight::Night;
    let target_level = los_level_for_unit(
        UnitKind::Infantry {
            fire: 0,
            melee: 0,
            movement: 0,
        },
        hex,
        &state.board,
    );
    let mut total = 0.0;
    for u in &state.units {
        if u.profile.identity.owner() != shooter || u.state.disrupted {
            continue;
        }
        let Some(fire) = u.profile.fire else { continue };
        let d = u.position.distance(hex);
        if d == 0 || d > 12 {
            continue;
        }
        let distance = HexDistance::new(d as u16);
        let band = if night {
            night_range_effects(u.profile.weapon, distance, shooter == Player::AngloEgyptian)
        } else {
            range_band_for(state.scenario, shooter, u.profile.weapon, distance)
        };
        let mult = band_multiplier(band);
        if mult == 0.0 {
            continue;
        }
        let firer_level = los_level_for_unit(u.profile.kind, u.position, &state.board);
        if !has_los(
            &state.board,
            u.position,
            hex,
            FireKind::Direct,
            firer_level,
            target_level,
            |_| None,
            |a, b| state.wall_is_breached(a, b),
        ) {
            continue;
        }
        // A Maxim battery and a named gunboat's Maxims fire twice (§6.42).
        let twice = matches!(u.profile.kind, UnitKind::Maxim { .. })
            || matches!(
                u.profile.identity,
                omdurman_rules::UnitIdentity::AngloEgyptianGunboat(g) if g.maxim_factor().is_some()
            );
        total += fire.value() as f32 * mult * if twice { 1.6 } else { 1.0 };
    }
    total
}

/// Melee factors of `attacker`'s undisrupted mobile units that could reach
/// a hex next to `hex` this coming turn (movement allowance in hexes, §5.11
/// clear-terrain cost; §8.1 halves it at night for the Anglo-Egyptians
/// only) -- the spears that kill a leader with his stack (§6.51, §7).
pub fn melee_reaching(state: &GameState, hex: HexCoord, attacker: Player) -> f32 {
    let night = state.day_night == DayNight::Night;
    state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == attacker && !u.state.disrupted)
        .filter_map(|u| {
            let melee = u.profile.melee?.value() as f32;
            let omdurman_rules::UnitMovement::Land(allowance) = u.profile.movement else {
                return None;
            };
            let mut reach = allowance.value() as u32;
            if night && attacker == Player::AngloEgyptian {
                reach /= 2;
            }
            (reach > 0 && u.position.distance(hex) <= reach + 1).then_some(melee)
        })
        .sum()
}

struct PathCache {
    key: (HexCoord, usize, u8),
    cost: HashMap<HexCoord, i32>,
}

thread_local! {
    static PATHS: RefCell<Vec<PathCache>> = const { RefCell::new(Vec::new()) };
}

/// Movement-point distance from `from` to `goal` for a land unit (§5.11
/// terrain costs, §5.23 walls closed except at gates and breaches, the Nile
/// and the closed Zariba impassable), ignoring units and roads. `None` when
/// `goal` cannot be reached. A Dijkstra field per goal, cached.
pub fn path_cost(state: &GameState, from: HexCoord, goal: HexCoord) -> Option<i32> {
    let key = (goal, state.breaches.len(), state.scenario as u8);
    if let Some(v) = PATHS.with(|p| {
        p.borrow()
            .iter()
            .find(|c| c.key == key)
            .map(|c| c.cost.get(&from).copied())
    }) {
        return v;
    }
    let field = dijkstra(state, goal);
    let v = field.get(&from).copied();
    PATHS.with(|p| {
        let mut p = p.borrow_mut();
        if p.len() >= 8 {
            p.remove(0);
        }
        p.push(PathCache { key, cost: field });
    });
    v
}

fn dijkstra(state: &GameState, goal: HexCoord) -> HashMap<HexCoord, i32> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let mut cost: HashMap<HexCoord, i32> = HashMap::new();
    let mut heap = BinaryHeap::new();
    cost.insert(goal, 0);
    heap.push(Reverse((0i32, goal.q, goal.r)));
    while let Some(Reverse((c, q, r))) = heap.pop() {
        let hex = HexCoord::new(q, r);
        if cost.get(&hex).is_some_and(|&best| c > best) {
            continue;
        }
        for n in hex.neighbors() {
            // Reverse search: the step is n -> hex, costed by the entered hex.
            let (Some(terrain), Some(_)) = (state.board.terrain_at(hex), state.board.terrain_at(n))
            else {
                continue;
            };
            let Some(step) = omdurman_rules::terrain_chart::land_step_cost(
                terrain,
                false,
                state.hexside_effective(n, hex),
            ) else {
                continue;
            };
            let nc = c + step as i32;
            if cost.get(&n).is_none_or(|&best| nc < best) {
                cost.insert(n, nc);
                heap.push(Reverse((nc, n.q, n.r)));
            }
        }
    }
    cost
}

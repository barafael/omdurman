//! Fire planning for the commanders: expected value of an attack, and the
//! allocation of every available weapon to targets as **combined** attacks.
//!
//! The candidate enumerator offers one attack per firing stack and target
//! (plus each named gunboat's Maxims, §2.32). But a hex may be fired at only
//! once per phase (§6.14), so firing those one by one wastes everything that
//! could have joined the first shot. §6.14 lets any number of weapons
//! combine at one hex -- a higher Combat Results Table row (§6.22) and, for
//! a whole brigade, the +1 of §5.54. This module merges the per-stack
//! candidates into combined attacks with [`combine_fire_attacks`], choosing
//! which weapons go where by greedy marginal expected value.
//!
//! The expected value only reads the attack and the board -- never the dice
//! already embedded in a candidate -- so planning cannot peek at outcomes.

use omdurman_rules::combat_results_table::{FireFactorRow, combat_results_table};
use omdurman_rules::effects::{
    GameEffect, GameState, combine_fire_attacks, fire_target_units, firer_contributions,
    mandatory_fire_modifiers, target_defence_modifier,
};
use omdurman_rules::{
    BritishLeader, CombatResult, DieRoll, FireAttack, FireKind, UnitIdentity, UnitPlacement,
};
use omdurman_types::{HexCoord, Player, Scenario, UnitKind};

/// What eliminating `u` is worth to its enemy, in §9.14 victory points (with
/// a few doctrine adjustments: GORDON is the Fall of Khartoum, forts shoot
/// and block even though they score nothing).
pub fn unit_value(state: &GameState, u: &UnitPlacement) -> f64 {
    match u.profile.identity {
        UnitIdentity::AngloEgyptianLeader(BritishLeader::Gordon) => 60.0,
        UnitIdentity::AngloEgyptianLeader(_) => 10.0,
        UnitIdentity::DervishLeader(omdurman_rules::DervishLeader::KhalifaAbdullah) => 10.0,
        UnitIdentity::DervishLeader(_) => 1.0,
        UnitIdentity::AngloEgyptianGunboat(_) => 10.0,
        // A fort scores nothing (§9.14) but fires every turn, even alone,
        // out to the artillery line's 7 hexes (§6.54, §6.22).
        UnitIdentity::DervishFort | UnitIdentity::AngloEgyptianFort => 2.0,
        _ => match u.profile.identity.owner() {
            // §9.14: 3 VP per Anglo-Egyptian land unit; in Fall of Khartoum
            // every defender lost is a hole in the wall line.
            Player::AngloEgyptian => {
                if state.scenario == Scenario::FallOfKhartoum {
                    4.0
                } else {
                    3.0
                }
            }
            // §9.14 / §9.24 / §9.35: each Dervish unit is 1 VP, one step on
            // the Historical schedule, one step of the FoK loss penalty.
            Player::Dervish => 1.0,
        },
    }
}

/// Expected value (victory points destroyed, plus a little for disruption)
/// of resolving `attack` now. Howitzer fire counts only its on-target chance
/// (impact 7-10, §6.64).
pub fn fire_value(state: &GameState, attack: &FireAttack) -> f64 {
    let total: u16 = firer_contributions(state, attack)
        .iter()
        .fold(0u16, |s, c| s.saturating_add(c.factor));
    if total == 0 {
        return 0.0;
    }
    let row = FireFactorRow::from_total(total);
    let targets = fire_target_units(state, attack, attack.target_hex);
    if targets.is_empty() {
        return 0.0;
    }
    let units: Vec<&UnitPlacement> = targets
        .iter()
        .filter_map(|id| state.find_unit(*id))
        .collect();
    let modifier: i16 = mandatory_fire_modifiers(state, attack)
        .iter()
        .map(|m| m.die_modifier())
        .sum::<i16>()
        + target_defence_modifier(state, attack, attack.target_hex, &targets);
    // Special targets (§6.61/§6.62): a gunboat sinks on 3+, a fort aimed at
    // falls on 2+ (with one occupant); nothing less does anything.
    let gunboat = units
        .iter()
        .find(|u| matches!(u.profile.kind, UnitKind::Gunboat { .. }));
    let fort = units
        .iter()
        .find(|u| matches!(u.profile.kind, UnitKind::Fort { .. }));
    let special: Option<(f64, u8)> = if let Some(g) = gunboat {
        Some((unit_value(state, g), 3))
    } else if let Some(f) = fort.filter(|_| attack.at_fort || units.len() == 1) {
        let occupant = units
            .iter()
            .filter(|u| u.id != f.id)
            .map(|u| unit_value(state, u))
            .fold(0.0, f64::max);
        Some((unit_value(state, f) + occupant, 2))
    } else {
        None
    };
    let mut values: Vec<f64> = units.iter().map(|u| unit_value(state, u)).collect();
    values.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let undisrupted = units.iter().filter(|u| !u.state.disrupted).count();
    let mut ev = 0.0;
    for face in 1u16..=10 {
        let Ok(roll) = DieRoll::try_from(face) else {
            continue;
        };
        let result = combat_results_table(row, roll.apply_modifier(modifier));
        ev += match (special, result) {
            (Some((value, needed)), CombatResult::Eliminate(n)) if n >= needed => value,
            (Some(_), _) => 0.0,
            (None, CombatResult::Eliminate(n)) => mean * (n as usize).min(values.len()) as f64,
            (None, CombatResult::Disrupt) => {
                // §CombatResults: half (round up) disrupted -- they may not
                // move, fire or melee until their turn ends.
                0.3 * mean * undisrupted.div_ceil(2).min(undisrupted) as f64
            }
            (None, CombatResult::NoEffect) => 0.0,
        };
    }
    ev /= 10.0;
    if attack.kind == FireKind::Howitzer {
        ev *= 0.4;
    }
    ev
}

/// The attack inside a fire candidate.
pub fn attack_of(effect: &GameEffect) -> Option<&FireAttack> {
    match effect {
        GameEffect::FireCombat { attack, .. } | GameEffect::HowitzerFire { attack, .. } => {
            Some(attack)
        }
        _ => None,
    }
}

fn with_attack(effect: &GameEffect, attack: FireAttack) -> GameEffect {
    match effect {
        GameEffect::FireCombat {
            roll, disruption, ..
        } => GameEffect::FireCombat {
            attack,
            roll: *roll,
            disruption: *disruption,
        },
        GameEffect::HowitzerFire {
            combat_results_table_roll,
            impact_roll,
            disruption,
            ..
        } => GameEffect::HowitzerFire {
            attack,
            combat_results_table_roll: *combat_results_table_roll,
            impact_roll: *impact_roll,
            disruption: *disruption,
        },
        other => other.clone(),
    }
}

/// A weapon group that can be assigned to one target: a firing stack's
/// main weapons of one kind, or one gunboat's Maxims.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Stack(HexCoord, FireKind),
    Maxims(omdurman_rules::UnitId, FireKind),
}

fn source_of(state: &GameState, attack: &FireAttack) -> Option<Source> {
    if let Some(&g) = attack.gunboat_maxims.first()
        && attack.firers.is_empty()
    {
        return Some(Source::Maxims(g, attack.kind));
    }
    let hex = state.find_unit(*attack.firers.first()?)?.position;
    Some(Source::Stack(hex, attack.kind))
}

type GroupKey = (HexCoord, FireKind, bool);

/// Replace the per-stack fire candidates by planned combined attacks.
///
/// Greedy marginal allocation: sources are taken strongest first, each goes
/// to the target where it adds the most expected value. The resulting groups
/// are returned as fire candidates (the dice of each group's first
/// candidate, in enumeration order); every other candidate passes through
/// unchanged. A group worth nothing is dropped -- such a shot would only
/// spend the firers.
pub fn plan_fire(state: &GameState, candidates: &[GameEffect]) -> Vec<GameEffect> {
    let mut others: Vec<GameEffect> = Vec::new();
    // (source, key, candidate, standalone value)
    let mut options: Vec<(Source, GroupKey, &GameEffect, f64)> = Vec::new();
    for c in candidates {
        let Some(attack) = attack_of(c) else {
            others.push(c.clone());
            continue;
        };
        let Some(source) = source_of(state, attack) else {
            others.push(c.clone());
            continue;
        };
        let v = fire_value(state, attack);
        options.push((
            source,
            (attack.target_hex, attack.kind, attack.at_fort),
            c,
            v,
        ));
    }
    if options.is_empty() {
        return others;
    }
    // Sources, strongest (best standalone value) first.
    let mut sources: Vec<(Source, f64)> = Vec::new();
    for (s, _, _, v) in &options {
        match sources.iter_mut().find(|(x, _)| x == s) {
            Some((_, best)) => *best = best.max(*v),
            None => sources.push((*s, *v)),
        }
    }
    sources.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Groups: key -> (combined candidate, value).
    let mut groups: Vec<(GroupKey, GameEffect, f64)> = Vec::new();
    for (source, _) in &sources {
        let mut best: Option<(usize, Option<usize>, FireAttack, f64)> = None; // (option idx, group idx, attack, gain)
        for (i, (s, key, cand, alone)) in options.iter().enumerate() {
            if s != source {
                continue;
            }
            let attack = attack_of(cand).expect("fire candidate");
            let (merged, gain, gi) = match groups.iter().position(|(k, _, _)| k == key) {
                Some(gi) => {
                    let existing = attack_of(&groups[gi].1).expect("fire group");
                    let Some(m) = combine_fire_attacks(state, existing, attack) else {
                        continue;
                    };
                    let v = fire_value(state, &m);
                    (m, v - groups[gi].2, Some(gi))
                }
                None => (attack.clone(), *alone, None),
            };
            if best.as_ref().is_none_or(|b| gain > b.3) {
                best = Some((i, gi, merged, gain));
            }
        }
        let Some((i, gi, merged, gain)) = best else {
            continue;
        };
        if gain <= 0.0 {
            continue;
        }
        let key = options[i].1;
        match gi {
            Some(gi) => {
                let effect = with_attack(&groups[gi].1, merged);
                groups[gi].2 += gain;
                groups[gi].1 = effect;
            }
            None => groups.push((key, options[i].2.clone(), gain)),
        }
    }
    others.extend(
        groups
            .into_iter()
            .filter(|(_, _, v)| *v > 0.02)
            .map(|(_, e, _)| e),
    );
    others
}

/// Expected value of a declared melee for the attacker: expected victory
/// points it destroys minus those it loses (§7.3: both sides roll on the
/// Combat Results Table at once, melee factors as fire factors, §7.7
/// modifiers: Dervish +2, Anglo-Egyptian +1, §9.232 entrenched -2).
pub fn melee_value(state: &GameState, attack: &omdurman_rules::MeleeAttack) -> f64 {
    use omdurman_rules::effects::{mandatory_melee_modifiers, melee_strength};
    let (att_mods, def_mods) = mandatory_melee_modifiers(state, attack);
    let att_mod: i16 = att_mods.iter().map(|m| m.die_modifier()).sum();
    let def_mod: i16 = def_mods.iter().map(|m| m.die_modifier()).sum();
    let side = |ids: &[omdurman_rules::UnitId]| -> Vec<f64> {
        ids.iter()
            .filter_map(|id| state.find_unit(*id))
            .map(|u| unit_value(state, u))
            .collect()
    };
    let att_values = side(&attack.attackers);
    let def_values = side(&attack.defenders);
    let expected_losses = |strength: u16, modifier: i16, victims: &[f64]| -> f64 {
        if strength == 0 || victims.is_empty() {
            return 0.0;
        }
        let row = FireFactorRow::from_total(strength);
        let mean = victims.iter().sum::<f64>() / victims.len() as f64;
        let mut ev = 0.0;
        for face in 1u16..=10 {
            let Ok(roll) = DieRoll::try_from(face) else {
                continue;
            };
            ev += match combat_results_table(row, roll.apply_modifier(modifier)) {
                CombatResult::Eliminate(n) => mean * (n as usize).min(victims.len()) as f64,
                CombatResult::Disrupt => 0.3 * mean * victims.len().div_ceil(2) as f64,
                CombatResult::NoEffect => 0.0,
            };
        }
        ev / 10.0
    };
    let gain = expected_losses(
        melee_strength(state, &attack.attackers),
        att_mod,
        &def_values,
    );
    let loss = expected_losses(
        melee_strength(state, &attack.defenders),
        def_mod,
        &att_values,
    );
    gain - loss
}

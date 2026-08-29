//! Human-readable rendering of [`GameEffect`]s, [`Observation`]s, and
//! [`TurnEventRecord`]s for the game log.
//!
//! Unit names come from the compiled counter profiles (`short_label`), falling
//! back to the `Debug` form of the `UnitId` when a profile is missing. Hexes are
//! printed as `(q,r)`; hexsides as `(q,r)-(q,r)`. Dice are spelled out so an
//! observer can re-derive CRT lookups and movement-cost arithmetic from the log
//! alone.

use omdurman_rules::combat_results_table::FireFactorRow;
use omdurman_rules::effects::{GameEffect, GameState, Observation};
use omdurman_rules::turn_summary::TurnEventRecord;
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_rules::{
    DemolitionTarget, FireAttack, FireModifier, FriendliesAction, MeleeAttack, UnitId, VpSource,
};
use omdurman_types::{HexCoord, HexsideRef};

/// The printed name of a unit counter (rulebook §2.3), falling back to the
/// `Debug` form of the ID. Where several counters share one printed name
/// (tribal blocks, forts, the gunboat pairs), a 1-based `#n` ordinal (in
/// `UnitId::ALL` order) is appended so log lines stay unambiguous.
pub fn unit_name(id: UnitId) -> String {
    let label = profile_for_unit(id)
        .map(|p| p.identity.short_label())
        .unwrap_or_else(|| format!("{id:?}"));
    let matches: Vec<UnitId> = UnitId::ALL
        .iter()
        .copied()
        .filter(|&other| {
            profile_for_unit(other)
                .map(|p| p.identity.short_label() == label)
                .unwrap_or(false)
        })
        .collect();
    if matches.len() > 1 {
        let n = matches
            .iter()
            .position(|&other| other == id)
            .map(|i| i + 1)
            .unwrap_or(0);
        format!("{label} #{n}")
    } else {
        label
    }
}

/// A hex as `(q,r)`.
pub fn hex(h: HexCoord) -> String {
    format!("({},{})", h.q, h.r)
}

/// A canonical hexside as `(q,r)-(q,r)`.
pub fn hexside_str(h: HexsideRef) -> String {
    format!("{}-{}", hex(h.a), hex(h.b))
}

/// The CRT row label in short form, e.g. `1-5`.
fn row_str(row: FireFactorRow) -> &'static str {
    match row {
        FireFactorRow::Row01to05 => "1-5",
        FireFactorRow::Row06to10 => "6-10",
        FireFactorRow::Row11to15 => "11-15",
        FireFactorRow::Row16to20 => "16-20",
        FireFactorRow::Row21to25 => "21-25",
        FireFactorRow::Row26to30 => "26-30",
        FireFactorRow::Row31to35 => "31-35",
        FireFactorRow::Row36to40 => "36-40",
        FireFactorRow::Row41Plus => "41+",
    }
}

/// Comma-joined unit names; empty string when the list is empty.
fn names(ids: &[UnitId]) -> String {
    ids.iter()
        .map(|&id| unit_name(id))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A "losses: a, b" suffix for combat resolutions; empty when none.
fn losses_suffix(ids: &[UnitId]) -> String {
    if ids.is_empty() {
        String::new()
    } else {
        format!("; losses: {}", names(ids))
    }
}

/// A ` modifiers[list]` suffix with each modifier's die modifier; empty when
/// none.
fn modifiers_suffix(modifiers: &[FireModifier]) -> String {
    if modifiers.is_empty() {
        String::new()
    } else {
        let list = modifiers
            .iter()
            .map(|m| format!("{m:?}({})", m.die_modifier()))
            .collect::<Vec<_>>()
            .join(", ");
        format!(" modifiers[{list}]")
    }
}

/// The shared opening of a fire-attack description. The pre-resolution
/// factor row is deliberately omitted: it is the caller's guess, and the
/// authoritative row rides on the `FireResolved` observation.
fn describe_fire_attack(a: &FireAttack, verb: &str) -> String {
    format!(
        "{verb} {} at {}{}",
        names(&a.firers),
        hex(a.target_hex),
        modifiers_suffix(&a.modifiers),
    )
}

/// A melee resolution's shared body.
fn describe_melee(
    a: &MeleeAttack,
    ar: omdurman_rules::DieRoll,
    dr: omdurman_rules::DieRoll,
) -> String {
    format!(
        "melee at {}: {} [roll {}] vs {} [roll {}]",
        hex(a.defender_hex),
        names(&a.attackers),
        ar.value(),
        names(&a.defenders),
        dr.value(),
    )
}

/// A demolition target.
fn target_str(t: DemolitionTarget) -> String {
    match t {
        DemolitionTarget::Fort(id) => format!("fort {}", unit_name(id)),
        DemolitionTarget::WallHexside(hs) => format!("wall {}", hexside_str(hs)),
    }
}

/// A `FriendliesAction` (§5.21).
fn describe_friendlies(a: &FriendliesAction) -> String {
    match a {
        FriendliesAction::Load { unit, gunboat } => {
            format!("load {} onto {}", unit_name(*unit), unit_name(*gunboat))
        }
        FriendliesAction::Cross { unit, gunboat, to } => format!(
            "{} crosses on {} to {}",
            unit_name(*unit),
            unit_name(*gunboat),
            hex(*to)
        ),
        FriendliesAction::Disembark { unit, gunboat } => {
            format!(
                "{} disembarks from {}",
                unit_name(*unit),
                unit_name(*gunboat)
            )
        }
    }
}

/// Render a `GameEffect` as one log line. Call with the *pre-apply* state so
/// unit positions reflect where the action started.
pub fn describe_effect(effect: &GameEffect, state: &GameState) -> String {
    match effect {
        GameEffect::AdvancePhase => format!("AdvancePhase (end {})", state.phase.top_level_name()),

        GameEffect::MoveUnit {
            unit_id,
            to,
            cost,
            path,
        } => {
            let unit = state.find_unit(*unit_id);
            let from = unit
                .map(|u| hex(u.position))
                .unwrap_or_else(|| "?".to_string());
            let via = if path.is_empty() {
                String::new()
            } else {
                let route = path.iter().map(|h| hex(*h)).collect::<Vec<_>>().join(" → ");
                format!(" via [{route}]")
            };
            // §5.11/§5.24 audit trail: cumulative MP spent vs the allowance
            // (the upstream allowance for gunboats).
            let mp_note = unit
                .map(|u| {
                    let spent = state.mp_spent(*unit_id) + cost.value();
                    let allow: i16 = match &u.profile.movement {
                        omdurman_rules::UnitMovement::Gunboat(g) => {
                            (g.upstream.value().min(g.downstream.value())) as i16
                        }
                        omdurman_rules::UnitMovement::Land(a) => a.value() as i16,
                        omdurman_rules::UnitMovement::Immobile => 0,
                    };
                    format!(" mp {spent}/{allow}")
                })
                .unwrap_or_default();
            format!(
                "MoveUnit {}: {from} → {} ({} MP){mp_note}{via}",
                unit_name(*unit_id),
                hex(*to),
                cost.value()
            )
        }

        GameEffect::FireCombat { attack, roll } => {
            format!(
                "{} [roll {}]",
                describe_fire_attack(attack, "fire"),
                roll.value()
            )
        }
        GameEffect::HowitzerFire {
            attack,
            combat_results_table_roll,
            impact_roll,
        } => {
            format!(
                "{} [CRT roll {}, impact roll {}]",
                describe_fire_attack(attack, "howitzer bombardment"),
                combat_results_table_roll.value(),
                impact_roll.value()
            )
        }
        GameEffect::MeleeCombat {
            attack,
            attacker_roll,
            defender_roll,
        } => describe_melee(attack, *attacker_roll, *defender_roll),
        GameEffect::DeclareMelee {
            attack,
            attacker_roll,
            defender_roll,
        } => format!(
            "declare melee; {}",
            describe_melee(attack, *attacker_roll, *defender_roll)
        ),
        GameEffect::ResolveMelee => "resolve declared melee".to_string(),
        GameEffect::RetreatBeforeMelee { unit_id, to } => {
            let from = state
                .find_unit(*unit_id)
                .map(|u| hex(u.position))
                .unwrap_or_else(|| "?".to_string());
            format!(
                "RetreatBeforeMelee {}: {from} → {}",
                unit_name(*unit_id),
                hex(*to)
            )
        }
        GameEffect::AdvanceAfterCombat { unit_id, to } => {
            let from = state
                .find_unit(*unit_id)
                .map(|u| hex(u.position))
                .unwrap_or_else(|| "?".to_string());
            format!(
                "AdvanceAfterCombat {}: {from} → {}",
                unit_name(*unit_id),
                hex(*to)
            )
        }
        GameEffect::RecoverUnit { unit_id } => format!("RecoverUnit {}", unit_name(*unit_id)),
        GameEffect::ConstructZariba { unit_ids, hexside } => {
            format!(
                "ConstructZariba {} on {}",
                names(unit_ids),
                hexside_str(*hexside)
            )
        }
        GameEffect::Demolition { unit_id, target } => {
            format!(
                "Demolition {} → {}",
                unit_name(*unit_id),
                target_str(*target)
            )
        }
        GameEffect::PlaceReinforcements(placements) => {
            let list = placements
                .iter()
                .map(|p| format!("{} at {}", unit_name(p.id), hex(p.position)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("PlaceReinforcements: {list}")
        }
        GameEffect::DervishDesertion { roll, deserters } => format!(
            "DervishDesertion roll {}: {} desert",
            roll.value(),
            names(deserters)
        ),
        GameEffect::FriendliesTransport(a) => {
            format!("FriendliesTransport: {}", describe_friendlies(a))
        }
        GameEffect::RiverMine {
            gunboat_id,
            hex: at,
            roll,
        } => {
            format!(
                "RiverMine on {} at {} roll {}",
                unit_name(*gunboat_id),
                hex(*at),
                roll.value()
            )
        }
        GameEffect::SinkChain => "SinkChain".to_string(),
        GameEffect::DeployUnit(p) => {
            format!(
                "DeployUnit [{}] {} at {}",
                p.profile.identity.owner(),
                unit_name(p.id),
                hex(p.position)
            )
        }
        GameEffect::RemoveDeployedUnit { unit_id, player } => {
            format!(
                "RemoveDeployedUnit {} ({player} pulls back)",
                unit_name(*unit_id)
            )
        }
        GameEffect::PlaceMine { hex: at } => format!("PlaceMine at {}", hex(*at)),
        GameEffect::PlaceChain { hexes } => {
            let list = hexes.iter().map(|h| hex(*h)).collect::<Vec<_>>().join(", ");
            format!("PlaceChain [{list}]")
        }
        GameEffect::PlaceZariba { hexside } => {
            format!("PlaceZariba on {}", hexside_str(*hexside))
        }
        GameEffect::ConfirmSetupReady { player } => {
            format!("ConfirmSetupReady ({player} ready)")
        }
        GameEffect::ResolveDemolition { unit_id, target } => {
            format!(
                "ResolveDemolition {} → {}",
                unit_name(*unit_id),
                target_str(*target)
            )
        }
        GameEffect::DriftGunboat { unit_id, .. } => {
            format!("DriftGunboat {}", unit_name(*unit_id))
        }
        GameEffect::ArtilleryBreachWall {
            firers,
            target,
            roll,
        } => format!(
            "ArtilleryBreachWall {} → {} roll {}",
            names(firers),
            hexside_str(*target),
            roll.value()
        ),
    }
}

/// A `VpSource` with its point value and recipient (§9.14).
fn vp_str(src: VpSource) -> String {
    format!("{src} → {} VP for {src:?}", src.points().value())
}

/// Render an engine `Observation` as a log line. The engine is the
/// authoritative source of the § citations (`paragraphs`).
pub fn describe_observation(obs: &Observation) -> String {
    match obs {
        Observation::UnitEliminated {
            id,
            cause,
            vp_source,
        } => {
            let vp = match vp_source {
                Some(src) => format!(" [{}]", vp_str(*src)),
                None => String::new(),
            };
            format!("UnitEliminated: {} {cause}{vp}", unit_name(*id))
        }
        Observation::FortDestroyed { id, hex: at } => {
            format!("FortDestroyed: {} at {}", unit_name(*id), hex(*at))
        }
        Observation::WallBreached {
            hexside,
            breached,
            row,
            adjacent_eliminated,
        } => {
            let adj = match adjacent_eliminated {
                Some(id) => format!("; adjacent {} eliminated", unit_name(*id)),
                None => String::new(),
            };
            // Truthful outcome + the CRT row the attempt rolled on (§6.63);
            // demolitions (§6.53) carry no row.
            let outcome = if *breached {
                "BREACHED".to_string()
            } else {
                match row {
                    Some(r) => {
                        format!("breach attempt FAILED (row {}, needed CRT 2+)", row_str(*r))
                    }
                    None => "breach attempt FAILED".to_string(),
                }
            };
            format!("WallBreached: {} {outcome}{adj}", hexside_str(*hexside))
        }
        Observation::LeaderKilled { id, by } => {
            format!("LeaderKilled: {} (killed by {by})", unit_name(*id))
        }
        Observation::GordonEliminated { turn } => {
            format!(
                "GordonEliminated: GORDON fallen at the Palace (turn {}) [§9.346]",
                turn.value()
            )
        }
        Observation::FriendliesDisembarked { unit_id, at } => {
            format!(
                "FriendliesDisembarked: {} at {}",
                unit_name(*unit_id),
                hex(*at)
            )
        }
        Observation::DemolitionResolved {
            engineer_id,
            target,
            success,
        } => {
            let outcome = if *success { "succeeded" } else { "failed" };
            format!(
                "DemolitionResolved: {} → {} {outcome}",
                unit_name(*engineer_id),
                target_str(*target)
            )
        }
        Observation::VictoryScored {
            source,
            points,
            for_player,
        } => format!(
            "VictoryScored: {for_player} +{} ({}) [§9.14]",
            points.value(),
            vp_str(*source),
        ),
        Observation::FireResolved {
            attack,
            roll,
            total_modifier,
            modified_roll,
            factor_row,
            effective_factor,
            result,
            eliminations,
            range,
            band,
            paragraphs,
        } => {
            // Range + range-effects band (§6.22, §8.1) -- the audit trail for
            // factor halving/doubling. Absent in pre-range records.
            let range_note = match (range, band) {
                (Some(r), Some(b)) => format!(", range {r}, band {b}"),
                _ => String::new(),
            };
            format!(
                "FireResolved at {}: {} roll {} ({:+}) = {} → {:?} [{}, {} eff factors{}]{} [§{}]",
                hex(attack.target_hex),
                names(&attack.firers),
                roll.value(),
                total_modifier,
                modified_roll.value(),
                result,
                row_str(*factor_row),
                effective_factor,
                range_note,
                losses_suffix(eliminations),
                paragraphs.join(" §"),
            )
        }
        Observation::MeleeResolved {
            attack,
            attacker_roll,
            attacker_total_modifier,
            attacker_modified_roll,
            attacker_result,
            defender_roll,
            defender_total_modifier,
            defender_modified_roll,
            defender_result,
            attacker_factor,
            defender_factor,
            attacker_losses,
            defender_losses,
            mandatory_advance,
            paragraphs,
        } => {
            let adv = match mandatory_advance {
                Some(n) => format!("; {n} Dervish advance into vacated hex"),
                None => String::new(),
            };
            format!(
                "MeleeResolved at {}: {} roll {} ({:+}) = {} → {:?} vs {} roll {} ({:+}) = {} → {:?} [eff {} vs {}]{}{}{} [§{}]",
                hex(attack.defender_hex),
                names(&attack.attackers),
                attacker_roll.value(),
                attacker_total_modifier,
                attacker_modified_roll.value(),
                attacker_result,
                names(&attack.defenders),
                defender_roll.value(),
                defender_total_modifier,
                defender_modified_roll.value(),
                defender_result,
                attacker_factor,
                defender_factor,
                losses_suffix(attacker_losses),
                losses_suffix(defender_losses),
                adv,
                paragraphs.join(" §"),
            )
        }
        Observation::HexVacatedByCombat {
            hex: at,
            eligible,
            paragraphs,
        } => format!(
            "HexVacatedByCombat at {}: {} may advance [§{}]",
            hex(*at),
            names(eligible),
            paragraphs.join(" §"),
        ),
    }
}

/// Render a structured turn event as a one-line dispatch (§4 turn record).
pub fn describe_turn_event(ev: &TurnEventRecord) -> String {
    match ev {
        TurnEventRecord::Movement {
            unit,
            from,
            to,
            cost,
        } => format!(
            "{} moved {} → {} (cost {cost} MP)",
            unit_name(*unit),
            hex(*from),
            hex(*to)
        ),
        TurnEventRecord::FireCombat {
            attacker,
            firers,
            target,
            roll,
            modifiers,
            total_modifier,
            result,
            kind,
            eliminated,
        } => {
            let mods = modifiers_suffix(modifiers);
            format!(
                "{attacker} fire ({kind:?}) {} at {}: roll {} ({:+}) → {:?}{mods}{}",
                names(firers),
                hex(*target),
                roll.value(),
                total_modifier,
                result,
                losses_suffix(eliminated),
            )
        }
        TurnEventRecord::MeleeCombat {
            attacker,
            defender,
            hex: at,
            attacker_roll,
            defender_roll,
            attacker_result,
            defender_result,
            attacker_losses,
            defender_losses,
            mandatory_advance,
        } => {
            let adv = match mandatory_advance {
                Some(n) => format!("; {n} Dervish advance"),
                None => String::new(),
            };
            format!(
                "Melee at {}: {attacker} roll {} → {:?}{}; {defender} roll {} → {:?}{}{adv}",
                hex(*at),
                attacker_roll.value(),
                attacker_result,
                losses_suffix(attacker_losses),
                defender_roll.value(),
                defender_result,
                losses_suffix(defender_losses),
            )
        }
        TurnEventRecord::Retreat { unit, from, to } => format!(
            "{} retreated {} → {}",
            unit_name(*unit),
            hex(*from),
            hex(*to)
        ),
        TurnEventRecord::AdvanceAfterCombat { unit, from, to } => format!(
            "{} advanced {} → {}",
            unit_name(*unit),
            hex(*from),
            hex(*to)
        ),
        TurnEventRecord::Reinforcements { units, player, at } => {
            format!(
                "{player} reinforcements ({}) placed at {}",
                names(units),
                hex(*at)
            )
        }
        TurnEventRecord::Demolition {
            engineer,
            target,
            success,
        } => {
            let outcome = if *success { "succeeded" } else { "failed" };
            format!(
                "Demolition by {} on {} {outcome}",
                unit_name(*engineer),
                target_str(*target)
            )
        }
        TurnEventRecord::Desertion { units, roll } => format!(
            "Dervish desertion (roll {}): {} removed",
            roll.value(),
            names(units)
        ),
        TurnEventRecord::UnitEliminated { unit, cause } => {
            format!("{} {cause}", unit_name(*unit))
        }
        TurnEventRecord::UnitDisrupted { unit } => {
            format!("{} disrupted", unit_name(*unit))
        }
        TurnEventRecord::UnitRecovered { unit } => {
            format!("{} recovered", unit_name(*unit))
        }
        TurnEventRecord::VpScored {
            source,
            points,
            for_player,
        } => format!(
            "{for_player} scored +{} ({})",
            points.value(),
            vp_str(*source)
        ),
    }
}

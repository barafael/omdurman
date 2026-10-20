//! Shared combat-presentation helpers: modifier descriptions, result
//! labels, and the fire/melee direction arrow. One home for the mappings
//! and geometry the combat panels must not let drift apart.

use bevy::prelude::*;
use omdurman_rules::{CombatResult, FireModifier, MeleeModifier};

/// One rulebook modifier rendered as a label + the § paragraph that
/// justifies it.
#[derive(Clone)]
pub(crate) struct ModifierLine {
    pub label: String,
    pub paragraph: String,
}

/// Canonical fire-modifier description (§6.24/§5.54/§6.23/§9.231/§9.232).
pub(crate) fn describe_fire_modifier(m: FireModifier) -> ModifierLine {
    let (label, paragraph) = match m {
        FireModifier::AngloEgyptianDirectFire => {
            ("+1 Anglo-Egyptian direct fire".to_string(), "6.24")
        }
        FireModifier::BrigadeIntegrity => ("+1 brigade integrity".to_string(), "5.54"),
        FireModifier::Terrain(n) => (format!("{n:+} terrain defence"), "6.23"),
        FireModifier::ZaribaThornHedge => ("-2 zariba thorn hedge".to_string(), "9.231"),
        FireModifier::ZaribaTrenchEntrenched => {
            ("-4 zariba trench (entrenched)".to_string(), "9.232")
        }
    };
    ModifierLine {
        label,
        paragraph: paragraph.into(),
    }
}

/// Canonical melee-modifier description (§7.7/§9.232).
pub(crate) fn describe_melee_modifier(m: MeleeModifier) -> ModifierLine {
    let (label, paragraph) = match m {
        MeleeModifier::DervishStandard => ("+2 Dervish standard".to_string(), "7.7"),
        MeleeModifier::AngloEgyptianStandard => ("+1 Anglo-Egyptian standard".to_string(), "7.7"),
        MeleeModifier::DervishVsTrenchedDefender => (
            "-2 instead of +2 vs. entrenched defender".to_string(),
            "9.232",
        ),
        MeleeModifier::FriendliesStandard => {
            ("+2 Friendlies (Dervish modifier)".to_string(), "6.52")
        }
    };
    ModifierLine {
        label,
        paragraph: paragraph.into(),
    }
}

/// Canonical combat-result label for the combat card.
pub(crate) fn describe_result(result: CombatResult) -> String {
    match result {
        CombatResult::NoEffect => "No effect".to_string(),
        CombatResult::Disrupt => "Disrupt".to_string(),
        CombatResult::Eliminate(n) => format!("Eliminate {n}"),
    }
}

/// A player-readable name for a rules unit. Tries the live engine state
/// first, then the static counter roster: an eliminated unit is gone from
/// `GameState.units`, yet its elimination is exactly when its name is needed
/// (dispatch slips, combat-card casualty lists).
pub(crate) fn unit_name(
    id: omdurman_rules::UnitId,
    gs: Option<&omdurman_rules::effects::GameState>,
) -> String {
    let scenario = gs.map_or(omdurman_types::Scenario::Campaign, |s| s.scenario);
    gs.and_then(|s| s.find_unit(id))
        .map(|u| u.profile.identity.label_in(scenario))
        .or_else(|| {
            omdurman_rules::unit_profiles::profile_for_unit(id)
                .map(|p| p.identity.label_in(scenario))
        })
        .unwrap_or_else(|| format!("unit {id:?}"))
}

/// A player-readable name for one weapon firing: the counter's name, and
/// "<gunboat> Maxims" for a named gunboat's Maxim guns (§2.32).
/// Collapse repeated names (or whole lines) into counts, in first-seen
/// order: eight identical Mulazmin read "8\u{d7} Mulazmin (3)", not eight
/// copies chained with " + " across five lines of the rail.
pub(crate) fn tally_names(names: Vec<String>) -> Vec<String> {
    let mut counted: Vec<(String, usize)> = Vec::new();
    for name in names {
        match counted.iter_mut().find(|(n, _)| *n == name) {
            Some((_, count)) => *count += 1,
            None => counted.push((name, 1)),
        }
    }
    counted
        .into_iter()
        .map(|(name, count)| match count {
            1 => name,
            n => format!("{n}\u{d7} {name}"),
        })
        .collect()
}

pub(crate) fn shot_name(
    shot: omdurman_rules::Shot,
    gs: Option<&omdurman_rules::effects::GameState>,
) -> String {
    let name = unit_name(shot.unit, gs);
    match shot.mount {
        omdurman_rules::FireMount::Main => name,
        omdurman_rules::FireMount::GunboatMaxims => format!("{name} Maxims"),
    }
}

#[cfg(test)]
mod tests {
    use super::tally_names;

    #[test]
    fn repeated_names_collapse_into_counts_in_first_seen_order() {
        let names = ["Mulazmin", "Kehena", "Mulazmin", "Mulazmin"]
            .map(String::from)
            .to_vec();
        assert_eq!(tally_names(names), ["3\u{d7} Mulazmin", "Kehena"]);
    }
}

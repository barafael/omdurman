//! Combat outcome prediction (§decision: legibility).
//!
//! Given a Combat Results Table factor row and a net die-roll modifier, the
//! outcome of any raw roll 1..=10 is fully determined. This module groups
//! those rolls into *bands* of identical outcome so a UI can show:
//!
//! > "no effect on 1-3, disrupt on 4-5, eliminate 1 on 6-8, eliminate 2 on 9-10"
//!
//! before the player commits to the shot. The preview is informational: the
//! engine still pre-rolls the die for canonical resolution, and the actual
//! combat resolution card (sourced from [`Observation::FireResolved`] /
//! [`Observation::MeleeResolved`]) is the authoritative report after the fact.
//!
//! Keeping this pure-functional on the engine's CRT types -- no Bevy, no egui
//! -- lets the fire and melee previews share one helper.

use omdurman_rules::combat_results_table::{FireFactorRow, combat_results_table};
use omdurman_rules::{CombatResult, DieRoll};

/// One band of raw rolls that all produce the same Combat Results Table
/// outcome. `lo` and `hi` are inclusive raw-die-roll values (1..=10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutcomeBand {
    pub lo: u8,
    pub hi: u8,
    pub result: CombatResult,
}

impl OutcomeBand {
    /// A short label suitable for a one-line preview, e.g. "1-3 no effect" or
    /// "6 eliminate 1" (a single-value band shows just the number).
    pub fn label(&self) -> String {
        let range = if self.lo == self.hi {
            format!("{}", self.lo)
        } else {
            format!("{}-{}", self.lo, self.hi)
        };
        let outcome = describe_result(self.result);
        format!("{range} {outcome}")
    }
}

/// Group raw die rolls 1..=10 into bands of identical CRT outcome, given the
/// factor row and net modifier. The modifier is applied to each raw roll
/// (clamped to 1..=10) before looking up the CRT -- this matches how the
/// engine resolves the attack at `apply_effect` time.
///
/// Returns at most ten bands (one per roll); consecutive rolls producing the
/// same result are merged.
pub fn outcome_bands(factor_row: FireFactorRow, net_modifier: i16) -> Vec<OutcomeBand> {
    let mut bands: Vec<OutcomeBand> = Vec::new();
    for raw in 1u8..=10u8 {
        let modified = ((raw as i16) + net_modifier).clamp(1, 10) as u16;
        let roll = DieRoll::try_from(modified).unwrap();
        let result = combat_results_table(factor_row, roll);
        match bands.last_mut() {
            Some(b) if b.result == result && b.hi == raw - 1 => b.hi = raw,
            _ => bands.push(OutcomeBand {
                lo: raw,
                hi: raw,
                result,
            }),
        }
    }
    bands
}

fn describe_result(result: CombatResult) -> &'static str {
    match result {
        CombatResult::NoEffect => "no effect",
        CombatResult::Disrupt => "disrupt",
        // Eliminate(n) collapses to "eliminate n" -- the count matters for
        // higher factor rows.
        CombatResult::Eliminate(1) => "eliminate 1",
        CombatResult::Eliminate(2) => "eliminate 2",
        CombatResult::Eliminate(3) => "eliminate 3",
        CombatResult::Eliminate(4) => "eliminate 4",
        CombatResult::Eliminate(_) => "eliminate 5+",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_factor_low_roll_is_no_effect() {
        let bands = outcome_bands(FireFactorRow::Row01to05, 0);
        assert_eq!(bands.first().unwrap().result, CombatResult::NoEffect);
        assert_eq!(bands.first().unwrap().lo, 1);
        assert_eq!(bands.first().unwrap().hi, 3);
    }

    #[test]
    fn positive_modifier_shifts_bands_down() {
        // Row 1-5, +3 modifier: raw 1 -> modified 4 -> Disrupt. So the
        // "no effect" band is empty (rolls 1-3 all become 4-6, Eliminate(1)).
        let bands = outcome_bands(FireFactorRow::Row01to05, 3);
        // Every raw roll produces a result at least as severe as Disrupt.
        assert!(bands.iter().all(|b| b.result != CombatResult::NoEffect));
    }

    #[test]
    fn labels_render_single_and_multi_roll_bands() {
        let bands = outcome_bands(FireFactorRow::Row01to05, 0);
        let labels: Vec<String> = bands.iter().map(|b| b.label()).collect();
        // 1-3 no effect, 4-5 disrupt, 6-8 eliminate 1, 9-10 eliminate 2
        assert!(labels.iter().any(|l| l == "1-3 no effect"));
        assert!(labels.iter().any(|l| l == "4-5 disrupt"));
        assert!(labels.iter().any(|l| l == "9-10 eliminate 2"));
    }
}

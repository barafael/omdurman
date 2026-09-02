use crate::{CombatResult, DieRoll};

/// Fire-factor row thresholds on the Combat Results Table (rulebook §6.22).
///
/// The printed table groups fire factors into bands.  The band index is used
/// to index into the result matrix.
#[derive(
    serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug,
)]
pub enum FireFactorRow {
    /// 1-5 factors
    Row01to05,
    /// 6-10 factors
    Row06to10,
    /// 11-15 factors
    Row11to15,
    /// 16-20 factors
    Row16to20,
    /// 21-25 factors
    Row21to25,
    /// 26-30 factors
    Row26to30,
    /// 31-35 factors
    Row31to35,
    /// 36-40 factors
    Row36to40,
    /// 41+ factors
    Row41Plus,
}

impl FireFactorRow {
    /// All rows in printed-table order.
    pub const ALL: [FireFactorRow; 9] = [
        FireFactorRow::Row01to05,
        FireFactorRow::Row06to10,
        FireFactorRow::Row11to15,
        FireFactorRow::Row16to20,
        FireFactorRow::Row21to25,
        FireFactorRow::Row26to30,
        FireFactorRow::Row31to35,
        FireFactorRow::Row36to40,
        FireFactorRow::Row41Plus,
    ];

    /// Determine which row a given total fire factor falls into (rulebook §6.22).
    pub fn from_total(total: u16) -> Self {
        match total {
            0..=5 => FireFactorRow::Row01to05,
            6..=10 => FireFactorRow::Row06to10,
            11..=15 => FireFactorRow::Row11to15,
            16..=20 => FireFactorRow::Row16to20,
            21..=25 => FireFactorRow::Row21to25,
            26..=30 => FireFactorRow::Row26to30,
            31..=35 => FireFactorRow::Row31to35,
            36..=40 => FireFactorRow::Row36to40,
            _ => FireFactorRow::Row41Plus,
        }
    }

    /// Zero-based row index on the printed table (top row `1-5` = 0), matching
    /// the row order of the Combat Results Table scan.
    pub fn index(self) -> usize {
        match self {
            FireFactorRow::Row01to05 => 0,
            FireFactorRow::Row06to10 => 1,
            FireFactorRow::Row11to15 => 2,
            FireFactorRow::Row16to20 => 3,
            FireFactorRow::Row21to25 => 4,
            FireFactorRow::Row26to30 => 5,
            FireFactorRow::Row31to35 => 6,
            FireFactorRow::Row36to40 => 7,
            FireFactorRow::Row41Plus => 8,
        }
    }
}

/// Look up a result on the Combat Results Table (rulebook §6.22, §7.7).
///
/// The table is a `static` constant transcribed from
/// `Boardgame - Remember_Gordon/tables/combat_results_table.ron`
/// (parity-tested in [`crate::tables_data`]); columns = modified die roll
/// (1-10), rows = total fire factors. Indexing is in-bounds by construction:
/// `FireFactorRow::index()` is 0..=8 and a `DieRoll` is 1..=10.
///
/// -- = `NoEffect`
/// D = `Disrupt` (1/2 of target units, round up)
/// 1...5 = `Eliminate(n)` (that many units removed)
pub fn combat_results_table(row: FireFactorRow, roll: DieRoll) -> CombatResult {
    crate::tables_data::CRT[row.index()][(roll.value() - 1) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use traceability_macro::rulebook;

    #[rulebook("§CRT")]
    #[test]
    fn ae_combat_results_table_lowest_is_no_effect() {
        let result = combat_results_table(FireFactorRow::Row01to05, DieRoll::One);
        assert_eq!(result, CombatResult::NoEffect);
    }

    #[rulebook("§CRT")]
    #[test]
    fn ae_combat_results_table_highest_is_eliminate_5() {
        let result = combat_results_table(FireFactorRow::Row41Plus, DieRoll::Ten);
        assert_eq!(result, CombatResult::Eliminate(5));
    }

    #[rulebook("§CRT")]
    #[test]
    fn ae_combat_results_table_progresses_with_roll() {
        let r1 = combat_results_table(FireFactorRow::Row16to20, DieRoll::One);
        let r10 = combat_results_table(FireFactorRow::Row16to20, DieRoll::Ten);
        assert!(r1 != CombatResult::Eliminate(3));
        assert_eq!(r10, CombatResult::Eliminate(3));
    }

    #[rulebook("§CRT")]
    #[test]
    fn ae_combat_results_table_progresses_with_factor() {
        let low = combat_results_table(FireFactorRow::Row01to05, DieRoll::Eight);
        let high = combat_results_table(FireFactorRow::Row41Plus, DieRoll::Eight);
        assert!(low != high);
        assert_eq!(high, CombatResult::Eliminate(4));
    }

    #[rulebook("§CRT")]
    #[test]
    fn fire_factor_row_boundaries() {
        assert_eq!(FireFactorRow::from_total(0), FireFactorRow::Row01to05);
        assert_eq!(FireFactorRow::from_total(5), FireFactorRow::Row01to05);
        assert_eq!(FireFactorRow::from_total(6), FireFactorRow::Row06to10);
        assert_eq!(FireFactorRow::from_total(15), FireFactorRow::Row11to15);
        assert_eq!(FireFactorRow::from_total(41), FireFactorRow::Row41Plus);
        assert_eq!(FireFactorRow::from_total(999), FireFactorRow::Row41Plus);
    }

    #[rulebook("§CRT")]
    #[test]
    fn fire_factor_row_remaining_boundaries() {
        assert_eq!(FireFactorRow::from_total(10), FireFactorRow::Row06to10);
        assert_eq!(FireFactorRow::from_total(11), FireFactorRow::Row11to15);
        assert_eq!(FireFactorRow::from_total(20), FireFactorRow::Row16to20);
        assert_eq!(FireFactorRow::from_total(21), FireFactorRow::Row21to25);
        assert_eq!(FireFactorRow::from_total(25), FireFactorRow::Row21to25);
        assert_eq!(FireFactorRow::from_total(26), FireFactorRow::Row26to30);
        assert_eq!(FireFactorRow::from_total(30), FireFactorRow::Row26to30);
        assert_eq!(FireFactorRow::from_total(31), FireFactorRow::Row31to35);
        assert_eq!(FireFactorRow::from_total(35), FireFactorRow::Row31to35);
        assert_eq!(FireFactorRow::from_total(36), FireFactorRow::Row36to40);
        assert_eq!(FireFactorRow::from_total(40), FireFactorRow::Row36to40);
    }

    #[rulebook("§CRT")]
    #[test]
    fn fire_factor_row_index_sequential() {
        assert_eq!(FireFactorRow::Row01to05.index(), 0);
        assert_eq!(FireFactorRow::Row06to10.index(), 1);
        assert_eq!(FireFactorRow::Row11to15.index(), 2);
        assert_eq!(FireFactorRow::Row16to20.index(), 3);
        assert_eq!(FireFactorRow::Row21to25.index(), 4);
        assert_eq!(FireFactorRow::Row26to30.index(), 5);
        assert_eq!(FireFactorRow::Row31to35.index(), 6);
        assert_eq!(FireFactorRow::Row36to40.index(), 7);
        assert_eq!(FireFactorRow::Row41Plus.index(), 8);
    }

    #[rulebook("§CRT")]
    #[test]
    fn crt_all_rows_monotone_non_decreasing() {
        // For every row, the result must be non-decreasing (in severity)
        // as the die roll increases.
        let rows = [
            FireFactorRow::Row01to05,
            FireFactorRow::Row06to10,
            FireFactorRow::Row11to15,
            FireFactorRow::Row16to20,
            FireFactorRow::Row21to25,
            FireFactorRow::Row26to30,
            FireFactorRow::Row31to35,
            FireFactorRow::Row36to40,
            FireFactorRow::Row41Plus,
        ];
        fn severity(r: CombatResult) -> u8 {
            match r {
                CombatResult::NoEffect => 0,
                CombatResult::Disrupt => 1,
                CombatResult::Eliminate(n) => 2 + n,
            }
        }
        for row in rows {
            for roll_val in 1u16..=10 {
                let roll = DieRoll::try_from(roll_val).unwrap();
                let prev = if roll_val > 1 {
                    Some(combat_results_table(
                        row,
                        DieRoll::try_from(roll_val - 1).unwrap(),
                    ))
                } else {
                    None
                };
                let curr = combat_results_table(row, roll);
                if let Some(p) = prev {
                    assert!(
                        severity(curr) >= severity(p),
                        "non-monotone on {row:?} at roll {roll_val}: {p:?} -> {curr:?}"
                    );
                }
            }
        }
    }

    #[rulebook("§CRT")]
    #[test]
    fn crt_every_cell_matches_the_table() {
        use CombatResult::*;
        // Expected results from the rulebook Combat Results Table (9 rows x 10 columns).
        // Each row is die rolls 1..=10; values: 0=NoEffect, 10=Disrupt, 11..15=Eliminate(n-10).
        let expected: [[u8; 10]; 9] = [
            // FF 1-5:   -  -  -  D  D  1  1  1  2  2
            [0, 0, 0, 10, 10, 11, 11, 11, 12, 12],
            // FF 6-10:  -  -  D  D  1  1  1  2  2  2
            [0, 0, 10, 10, 11, 11, 11, 12, 12, 12],
            // FF 11-15: -  D  D  1  1  1  2  2  2  3
            [0, 10, 10, 11, 11, 11, 12, 12, 12, 13],
            // FF 16-20: D  D  1  1  1  2  2  2  3  3
            [10, 10, 11, 11, 11, 12, 12, 12, 13, 13],
            // FF 21-25: D  1  1  1  2  2  2  3  3  3
            [10, 11, 11, 11, 12, 12, 12, 13, 13, 13],
            // FF 26-30: 1  1  1  2  2  2  3  3  3  4
            [11, 11, 11, 12, 12, 12, 13, 13, 13, 14],
            // FF 31-35: 1  1  2  2  2  3  3  3  4  4
            [11, 11, 12, 12, 12, 13, 13, 13, 14, 14],
            // FF 36-40: 1  2  2  2  3  3  3  4  4  4
            [11, 12, 12, 12, 13, 13, 13, 14, 14, 14],
            // FF 41+:   2  2  2  3  3  3  4  4  4  5
            [12, 12, 12, 13, 13, 13, 14, 14, 14, 15],
        ];
        let rows = [
            FireFactorRow::Row01to05,
            FireFactorRow::Row06to10,
            FireFactorRow::Row11to15,
            FireFactorRow::Row16to20,
            FireFactorRow::Row21to25,
            FireFactorRow::Row26to30,
            FireFactorRow::Row31to35,
            FireFactorRow::Row36to40,
            FireFactorRow::Row41Plus,
        ];
        for (row_idx, &row) in rows.iter().enumerate() {
            for roll_val in 1u16..=10 {
                let roll = DieRoll::try_from(roll_val).unwrap();
                let got = combat_results_table(row, roll);
                let enc = expected[row_idx][(roll_val - 1) as usize];
                let want = match enc {
                    0 => NoEffect,
                    10 => Disrupt,
                    n @ 11..=15 => Eliminate(n - 10),
                    _ => unreachable!(),
                };
                assert_eq!(
                    got, want,
                    "CRT mismatch at row {row_idx} (FF {:?}), roll {roll_val}: got {got:?}, want {want:?}",
                    row,
                );
            }
        }
    }

    #[rulebook("§CRT")]
    #[test]
    fn crt_cross_row_monotone_for_each_roll() {
        let rows = [
            FireFactorRow::Row01to05,
            FireFactorRow::Row06to10,
            FireFactorRow::Row11to15,
            FireFactorRow::Row16to20,
            FireFactorRow::Row21to25,
            FireFactorRow::Row26to30,
            FireFactorRow::Row31to35,
            FireFactorRow::Row36to40,
            FireFactorRow::Row41Plus,
        ];
        fn severity(r: CombatResult) -> u8 {
            match r {
                CombatResult::NoEffect => 0,
                CombatResult::Disrupt => 1,
                CombatResult::Eliminate(n) => 2 + n,
            }
        }
        for roll_val in 1u16..=10 {
            let roll = DieRoll::try_from(roll_val).unwrap();
            let mut prev_severity = 0u8;
            for row in rows {
                let result = combat_results_table(row, roll);
                let sev = severity(result);
                assert!(
                    sev >= prev_severity,
                    "cross-row decrease at roll {roll_val}: {row:?} {result:?} < prev severity {prev_severity}"
                );
                prev_severity = sev;
            }
        }
    }

    #[rulebook("§CRT")]
    #[test]
    fn crt_lowest_row_is_worst_highest_row_is_best() {
        for roll_val in 1u16..=10 {
            let roll = DieRoll::try_from(roll_val).unwrap();
            let low = combat_results_table(FireFactorRow::Row01to05, roll);
            let high = combat_results_table(FireFactorRow::Row41Plus, roll);
            for row in [
                FireFactorRow::Row06to10,
                FireFactorRow::Row11to15,
                FireFactorRow::Row16to20,
                FireFactorRow::Row21to25,
                FireFactorRow::Row26to30,
                FireFactorRow::Row31to35,
                FireFactorRow::Row36to40,
            ] {
                let mid = combat_results_table(row, roll);
                let sev = |r: CombatResult| -> u8 {
                    match r {
                        CombatResult::NoEffect => 0,
                        CombatResult::Disrupt => 1,
                        CombatResult::Eliminate(n) => 2 + n,
                    }
                };
                assert!(
                    sev(low) <= sev(mid),
                    "Row01to05 not worst at roll {roll_val}: low={low:?} > mid={mid:?} ({row:?})"
                );
                assert!(
                    sev(mid) <= sev(high),
                    "Row41Plus not best at roll {roll_val}: mid={mid:?} > high={high:?} ({row:?})"
                );
            }
        }
    }

    #[rulebook("§CRT")]
    #[test]
    fn crt_eliminate_never_exceeds_5() {
        let rows = [
            FireFactorRow::Row01to05,
            FireFactorRow::Row06to10,
            FireFactorRow::Row11to15,
            FireFactorRow::Row16to20,
            FireFactorRow::Row21to25,
            FireFactorRow::Row26to30,
            FireFactorRow::Row31to35,
            FireFactorRow::Row36to40,
            FireFactorRow::Row41Plus,
        ];
        for row in rows {
            for roll_val in 1u16..=10 {
                let roll = DieRoll::try_from(roll_val).unwrap();
                let result = combat_results_table(row, roll);
                if let CombatResult::Eliminate(n) = result {
                    assert!(
                        n <= 5,
                        "Eliminate({n}) exceeds 5 on {row:?} roll {roll_val}"
                    );
                }
            }
        }
    }
}

/// Kani proof harnesses over the authored Combat Results Table (`cargo kani`,
/// see `scripts/kani.sh`). The CRT is a `static` constant in `tables_data`
/// (parity-tested against the authored RON), so these proofs reason over the
/// real printed data symbolically and cover the whole 9-row × 10-roll domain.
#[cfg(kani)]
mod verification {
    use super::{FireFactorRow, combat_results_table};
    use crate::{CombatResult, DieRoll};

    /// An arbitrary legal die roll.
    fn any_roll() -> DieRoll {
        let i: usize = kani::any();
        kani::assume(i < DieRoll::ALL.len());
        DieRoll::ALL[i]
    }

    /// An arbitrary fire-factor row.
    fn any_row() -> FireFactorRow {
        let i: usize = kani::any();
        kani::assume(i < FireFactorRow::ALL.len());
        FireFactorRow::ALL[i]
    }

    fn severity(r: CombatResult) -> u8 {
        match r {
            CombatResult::NoEffect => 0,
            CombatResult::Disrupt => 1,
            CombatResult::Eliminate(n) => 1 + n,
        }
    }

    /// No cell ever yields an out-of-range elimination count: indexing is
    /// in-bounds by construction and every `Eliminate` payload stays within
    /// the printed 1..=5, for the entire table.
    // §CRT
    #[kani::proof]
    fn crt_eliminate_count_stays_within_printed_bounds() {
        let row = any_row();
        let roll = any_roll();
        if let CombatResult::Eliminate(n) = combat_results_table(row, roll) {
            assert!(n >= 1 && n <= 5);
        }
    }

    /// For every fire-factor row the result is non-decreasing in the modified
    /// die roll -- a better roll is never worse, anywhere on the table.
    // §CRT
    #[kani::proof]
    fn crt_is_monotone_in_the_die_roll_for_every_row() {
        let row = any_row();
        let a = any_roll();
        let b = any_roll();
        kani::assume(a.value() <= b.value());
        assert!(severity(combat_results_table(row, a)) <= severity(combat_results_table(row, b)));
    }
}

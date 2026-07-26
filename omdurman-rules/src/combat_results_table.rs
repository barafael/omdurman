use crate::{CombatResult, DieRoll};

/// Fire-factor row thresholds on the Combat Results Table (rulebook §6.22).
///
/// The printed table groups fire factors into bands.  The band index is used
/// to index into the result matrix.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
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
/// Columns = modified die roll (1-10), rows = total fire factors.
///
/// -- = `NoEffect`
/// D = `Disrupt` (1/2 of target units, round up)
/// 1...5 = `Eliminate(n)` (that many units removed)
pub fn combat_results_table(row: FireFactorRow, roll: DieRoll) -> CombatResult {
    use CombatResult::*;
    use DieRoll::*;
    use FireFactorRow::*;
    match (row, roll) {
        // 1-5:   -  -  -  D  D  1  1  1  2  2
        (Row01to05, One | Two | Three) => NoEffect,
        (Row01to05, Four | Five) => Disrupt,
        (Row01to05, Six | Seven | Eight) => Eliminate(1),
        (Row01to05, Nine | Ten) => Eliminate(2),
        // 6-10:  -  -  D  D  1  1  1  2  2  2
        (Row06to10, One | Two) => NoEffect,
        (Row06to10, Three | Four) => Disrupt,
        (Row06to10, Five | Six | Seven) => Eliminate(1),
        (Row06to10, Eight | Nine | Ten) => Eliminate(2),
        // 11-15: -  D  D  1  1  1  2  2  2  3
        (Row11to15, One) => NoEffect,
        (Row11to15, Two | Three) => Disrupt,
        (Row11to15, Four | Five | Six) => Eliminate(1),
        (Row11to15, Seven | Eight | Nine) => Eliminate(2),
        (Row11to15, Ten) => Eliminate(3),
        // 16-20: D  D  1  1  1  2  2  2  3  3
        (Row16to20, One | Two) => Disrupt,
        (Row16to20, Three | Four | Five) => Eliminate(1),
        (Row16to20, Six | Seven | Eight) => Eliminate(2),
        (Row16to20, Nine | Ten) => Eliminate(3),
        // 21-25: D  1  1  1  2  2  2  3  3  3
        (Row21to25, One) => Disrupt,
        (Row21to25, Two | Three | Four) => Eliminate(1),
        (Row21to25, Five | Six | Seven) => Eliminate(2),
        (Row21to25, Eight | Nine | Ten) => Eliminate(3),
        // 26-30: 1  1  1  2  2  2  3  3  3  4
        (Row26to30, One | Two | Three) => Eliminate(1),
        (Row26to30, Four | Five | Six) => Eliminate(2),
        (Row26to30, Seven | Eight | Nine) => Eliminate(3),
        (Row26to30, Ten) => Eliminate(4),
        // 31-35: 1  1  2  2  2  3  3  3  4  4
        (Row31to35, One | Two) => Eliminate(1),
        (Row31to35, Three | Four | Five) => Eliminate(2),
        (Row31to35, Six | Seven | Eight) => Eliminate(3),
        (Row31to35, Nine | Ten) => Eliminate(4),
        // 36-40: 1  2  2  2  3  3  3  4  4  4
        (Row36to40, One) => Eliminate(1),
        (Row36to40, Two | Three | Four) => Eliminate(2),
        (Row36to40, Five | Six | Seven) => Eliminate(3),
        (Row36to40, Eight | Nine | Ten) => Eliminate(4),
        // 41+:   2  2  2  3  3  3  4  4  4  5
        (Row41Plus, One | Two | Three) => Eliminate(2),
        (Row41Plus, Four | Five | Six) => Eliminate(3),
        (Row41Plus, Seven | Eight | Nine) => Eliminate(4),
        (Row41Plus, Ten) => Eliminate(5),
    }
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

use crate::{CombatResult, DieRoll};

/// Fire-factor row thresholds on the CRT.
///
/// The printed table groups fire factors into bands.  The band index is used
/// to index into the result matrix.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FireFactorRow {
    /// 1–5 factors
    Row01to05,
    /// 6–10 factors
    Row06to10,
    /// 11–15 factors
    Row11to15,
    /// 16–20 factors
    Row16to20,
    /// 21–25 factors
    Row21to25,
    /// 26–30 factors
    Row26to30,
    /// 31–35 factors
    Row31to35,
    /// 36–40 factors
    Row36to40,
    /// 41+ factors
    Row41Plus,
}

impl FireFactorRow {
    /// Determine which row a given total fire factor falls into.
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
}

/// Look up a result on the Combat Results Table.
///
/// Columns = modified die roll (1–10), rows = total fire factors.
///
/// — = `NoEffect`
/// D = `Disrupt` (½ of target units, round up)
/// 1…5 = `Eliminate(n)` (that many units removed)
pub fn combat_results_table(row: FireFactorRow, roll: DieRoll) -> CombatResult {
    use CombatResult::*;
    use DieRoll::*;
    use FireFactorRow::*;
    match (row, roll) {
        // 1–5:   -  -  -  D  D  1  1  1  2  2
        (Row01to05, One | Two | Three) => NoEffect,
        (Row01to05, Four | Five) => Disrupt,
        (Row01to05, Six | Seven | Eight) => Eliminate(1),
        (Row01to05, Nine | Ten) => Eliminate(2),
        // 6–10:  -  -  D  D  1  1  1  2  2  2
        (Row06to10, One | Two) => NoEffect,
        (Row06to10, Three | Four) => Disrupt,
        (Row06to10, Five | Six | Seven) => Eliminate(1),
        (Row06to10, Eight | Nine | Ten) => Eliminate(2),
        // 11–15: -  D  D  1  1  1  2  2  2  3
        (Row11to15, One) => NoEffect,
        (Row11to15, Two | Three) => Disrupt,
        (Row11to15, Four | Five | Six) => Eliminate(1),
        (Row11to15, Seven | Eight | Nine) => Eliminate(2),
        (Row11to15, Ten) => Eliminate(3),
        // 16–20: D  D  1  1  1  2  2  2  3  3
        (Row16to20, One | Two) => Disrupt,
        (Row16to20, Three | Four | Five) => Eliminate(1),
        (Row16to20, Six | Seven | Eight) => Eliminate(2),
        (Row16to20, Nine | Ten) => Eliminate(3),
        // 21–25: D  1  1  1  2  2  2  3  3  3
        (Row21to25, One) => Disrupt,
        (Row21to25, Two | Three | Four) => Eliminate(1),
        (Row21to25, Five | Six | Seven) => Eliminate(2),
        (Row21to25, Eight | Nine | Ten) => Eliminate(3),
        // 26–30: 1  1  1  2  2  2  3  3  3  4
        (Row26to30, One | Two | Three) => Eliminate(1),
        (Row26to30, Four | Five | Six) => Eliminate(2),
        (Row26to30, Seven | Eight | Nine) => Eliminate(3),
        (Row26to30, Ten) => Eliminate(4),
        // 31–35: 1  1  2  2  2  3  3  3  4  4
        (Row31to35, One | Two) => Eliminate(1),
        (Row31to35, Three | Four | Five) => Eliminate(2),
        (Row31to35, Six | Seven | Eight) => Eliminate(3),
        (Row31to35, Nine | Ten) => Eliminate(4),
        // 36–40: 1  2  2  2  3  3  3  4  4  4
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

    #[test]
    fn ae_crt_lowest_is_no_effect() {
        let result = combat_results_table(FireFactorRow::Row01to05, DieRoll::One);
        assert_eq!(result, CombatResult::NoEffect);
    }

    #[test]
    fn ae_crt_highest_is_eliminate_5() {
        let result = combat_results_table(FireFactorRow::Row41Plus, DieRoll::Ten);
        assert_eq!(result, CombatResult::Eliminate(5));
    }

    #[test]
    fn ae_crt_progresses_with_roll() {
        let r1 = combat_results_table(FireFactorRow::Row16to20, DieRoll::One);
        let r10 = combat_results_table(FireFactorRow::Row16to20, DieRoll::Ten);
        assert!(r1 != CombatResult::Eliminate(3));
        assert_eq!(r10, CombatResult::Eliminate(3));
    }

    #[test]
    fn ae_crt_progresses_with_factor() {
        let low = combat_results_table(FireFactorRow::Row01to05, DieRoll::Eight);
        let high = combat_results_table(FireFactorRow::Row41Plus, DieRoll::Eight);
        assert!(low != high);
        assert_eq!(high, CombatResult::Eliminate(4));
    }

    #[test]
    fn fire_factor_row_boundaries() {
        assert_eq!(FireFactorRow::from_total(0), FireFactorRow::Row01to05);
        assert_eq!(FireFactorRow::from_total(5), FireFactorRow::Row01to05);
        assert_eq!(FireFactorRow::from_total(6), FireFactorRow::Row06to10);
        assert_eq!(FireFactorRow::from_total(15), FireFactorRow::Row11to15);
        assert_eq!(FireFactorRow::from_total(41), FireFactorRow::Row41Plus);
        assert_eq!(FireFactorRow::from_total(999), FireFactorRow::Row41Plus);
    }
}

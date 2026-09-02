use crate::DieRoll;

/// The seven impact hexes of the printed Howitzer Fire Scattergram diagram
/// (§6.64): a centre hex (the designated target) ringed by six neighbours,
/// addressed relative to the printed diagram. "Upper" edges are the side of
/// the diagram away from the firing player.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ScatterHexDirection {
    UpperLeft,
    UpperRight,
    Right,
    LowerRight,
    LowerLeft,
    Left,
    Center,
}

impl ScatterHexDirection {
    /// Whether this entry leaves the shell on the designated target hex
    /// (impact roll 7-10, §6.64).
    pub fn is_center(self) -> bool {
        self == ScatterHexDirection::Center
    }
}

/// Resolve the impact hex of a howitzer salvo from the second die roll
/// (§6.64): a lookup into the Howitzer Fire Scattergram, a `static`
/// constant transcribed from
/// `Boardgame - Remember_Gordon/tables/howitzer_scattergram.ron`
/// (parity-tested in [`crate::tables_data`]). The index is in-bounds by
/// construction: a `DieRoll` is 1..=10.
///
/// The first die roll is the Combat Results Table roll (handled by
/// [`crate::combat_results_table`]); this function determines the *impact
/// hex* from the second roll. The caller maps the [`ScatterHexDirection`]
/// onto a hex-grid offset oriented away from the firer (see
/// `GameState::howitzer_impact_hex`).
pub fn howitzer_scatter(impact_roll: DieRoll) -> ScatterHexDirection {
    crate::tables_data::SCATTERGRAM[(impact_roll.value() - 1) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use traceability_macro::rulebook;

    #[rulebook("§6.42", "§6.64")]
    #[test]
    fn howitzer_on_target_7_to_10() {
        for roll in 7u8..=10 {
            assert_eq!(
                howitzer_scatter(DieRoll::try_from(roll as u16).unwrap()),
                ScatterHexDirection::Center
            );
        }
    }

    #[rulebook("§6.42", "§6.64")]
    #[test]
    fn howitzer_scatters_below_7() {
        for roll in 1u8..=6 {
            assert!(!howitzer_scatter(DieRoll::try_from(roll as u16).unwrap()).is_center());
        }
    }

    /// The authored scattergram assigns a distinct ring hex to each of the
    /// rolls 1-6 (printed order: UL, UR, R, LR, LL, L).
    #[rulebook("§6.64")]
    #[test]
    fn howitzer_each_miss_gets_its_printed_hex() {
        let expected = [
            ScatterHexDirection::UpperLeft,
            ScatterHexDirection::UpperRight,
            ScatterHexDirection::Right,
            ScatterHexDirection::LowerRight,
            ScatterHexDirection::LowerLeft,
            ScatterHexDirection::Left,
        ];
        for (i, want) in expected.iter().enumerate() {
            assert_eq!(
                howitzer_scatter(DieRoll::try_from(i as u16 + 1).unwrap()),
                *want
            );
        }
    }
}

/// Kani proof harnesses over the authored Howitzer Scattergram (`cargo kani`,
/// see `scripts/kani.sh`). The scattergram is a `static` constant in
/// `tables_data` (parity-tested against the authored RON), so this proof
/// covers the whole d10 impact-roll domain.
#[cfg(kani)]
mod verification {
    use super::{ScatterHexDirection, howitzer_scatter};
    use crate::DieRoll;

    /// An arbitrary legal die roll.
    fn any_roll() -> DieRoll {
        let i: usize = kani::any();
        kani::assume(i < DieRoll::ALL.len());
        DieRoll::ALL[i]
    }

    /// Impact rolls land on the designated target hex exactly when the roll
    /// is 7 or better; every lower roll scatters to a ring hex. The full d10
    /// domain, proven over the authored table.
    // §6.64
    #[kani::proof]
    fn scatter_is_center_exactly_for_rolls_7_to_10() {
        let roll = any_roll();
        assert!((howitzer_scatter(roll) == ScatterHexDirection::Center) == (roll.value() >= 7));
    }
}

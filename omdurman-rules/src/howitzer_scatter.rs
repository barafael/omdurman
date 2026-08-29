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
/// (§6.64): a lookup into the Howitzer Fire Scattergram authored in
/// `Boardgame - Remember_Gordon/tables/howitzer_scattergram.ron` (embedded
/// at compile time by [`crate::tables_data`]).
///
/// The first die roll is the Combat Results Table roll (handled by
/// [`crate::combat_results_table`]); this function determines the *impact
/// hex* from the second roll. The caller maps the [`ScatterHexDirection`]
/// onto a hex-grid offset oriented away from the firer (see
/// `GameState::howitzer_impact_hex`).
pub fn howitzer_scatter(impact_roll: DieRoll) -> ScatterHexDirection {
    let table = crate::tables_data::scattergram_table();
    table
        .get((impact_roll.value() - 1) as usize)
        .copied()
        .unwrap_or(ScatterHexDirection::Center)
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

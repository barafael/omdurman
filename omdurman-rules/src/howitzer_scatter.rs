use crate::DieRoll;

/// Scatter direction for howitzer fire, matching the rulebook terminology
/// (§6.64). The caller maps these to hex-grid offsets.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScatterDirection {
    /// Roll 7–10: hit the target hex.
    OnTarget,
    /// Roll 5–6: short (downstream along the Nile).
    Short,
    /// Roll 3–4: long (upstream along the Nile).
    Long,
    /// Roll 1–2: left/right scatter.
    LeftRight,
}

/// Resolve howitzer fire scatter (§6.64).
///
/// The first die roll is the Combat Results Table roll (handled by [`crate::combat_results_table`]).
/// This function determines the *impact hex* from the second die roll:
///
/// | Roll | Result |
/// |------|--------|
/// | 7–10 | [`ScatterDirection::OnTarget`] |
/// | 5–6  | [`ScatterDirection::Short`] (downstream) |
/// | 3–4  | [`ScatterDirection::Long`] (upstream) |
/// | 1–2  | [`ScatterDirection::LeftRight`] |
pub fn howitzer_scatter(impact_roll: DieRoll) -> ScatterDirection {
    use DieRoll::*;
    match impact_roll {
        Seven | Eight | Nine | Ten => ScatterDirection::OnTarget,
        Five | Six => ScatterDirection::Short,
        Three | Four => ScatterDirection::Long,
        One | Two => ScatterDirection::LeftRight,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn howitzer_on_target_7_to_10() {
        for roll in 7u8..=10 {
            assert_eq!(
                howitzer_scatter(DieRoll::from(roll)),
                ScatterDirection::OnTarget
            );
        }
    }

    #[test]
    fn howitzer_scatters_below_7() {
        for roll in 1u8..=6 {
            assert_ne!(
                howitzer_scatter(DieRoll::from(roll)),
                ScatterDirection::OnTarget
            );
        }
    }
}

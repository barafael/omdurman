use crate::DieRoll;
use omdurman_types::HexCoord;

/// The seven impact hexes of the printed Howitzer Fire Scattergram diagram
/// (§6.64): a centre hex (the designated target) ringed by six neighbours,
/// addressed as printed on the mapsheet -- "upper" is north, like the rest
/// of the map.
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

/// The hex a howitzer shell lands in (§6.64: "Refer to the Howitzer Fire
/// Scattergram on the mapsheet for the impact hex"): the designated
/// `target` on 7-10, otherwise the neighbour the printed diagram names --
/// the diagram lies on the mapsheet, so its directions are the map's
/// (pointy-top neighbour order: east, south-east, south-west, west,
/// north-west, north-east).
pub fn scatter_impact_hex(target: HexCoord, scatter: ScatterHexDirection) -> HexCoord {
    let n = target.neighbors();
    match scatter {
        ScatterHexDirection::Center => target,
        ScatterHexDirection::Right => n[0],
        ScatterHexDirection::LowerRight => n[1],
        ScatterHexDirection::LowerLeft => n[2],
        ScatterHexDirection::Left => n[3],
        ScatterHexDirection::UpperLeft => n[4],
        ScatterHexDirection::UpperRight => n[5],
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
/// hex* from the second roll, placed by [`scatter_impact_hex`].
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

    /// The diagram's directions are the map's: roll 3 ("Right") lands east
    /// of the target, roll 1 ("Upper Left") north-west, whoever fires.
    #[rulebook("§6.64")]
    #[test]
    fn scatter_follows_the_printed_diagram() {
        let target = HexCoord::new(8, 8);
        let impact = |roll: u16| {
            scatter_impact_hex(target, howitzer_scatter(DieRoll::try_from(roll).unwrap()))
        };
        assert_eq!(impact(3), HexCoord::new(9, 8), "3: right (east)");
        assert_eq!(impact(6), HexCoord::new(7, 8), "6: left (west)");
        assert_eq!(impact(1), HexCoord::new(7, 7), "1: upper left");
        assert_eq!(impact(2), HexCoord::new(8, 7), "2: upper right");
        assert_eq!(impact(4), HexCoord::new(9, 9), "4: lower right");
        assert_eq!(impact(5), HexCoord::new(8, 9), "5: lower left");
        for roll in 7..=10 {
            assert_eq!(impact(roll), target);
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

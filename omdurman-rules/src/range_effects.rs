use crate::{HexDistance, Range, RangeBand, WeaponClass};

/// Convert a hex distance (1‑based) to a [`Range`] enum variant.
/// Distances > 10 are clamped to `Range::Ten`; callers should check
/// `distance.value() > 10` and return `OutOfRange` before calling this.
fn hex_distance_to_range(distance: HexDistance) -> Range {
    match distance.value() {
        1 => Range::One,
        2 => Range::Two,
        3 => Range::Three,
        4 => Range::Four,
        5 => Range::Five,
        6 => Range::Six,
        7 => Range::Seven,
        8 => Range::Eight,
        9 => Range::Nine,
        _ => Range::Ten,
    }
}

/// Look up the range band for an Anglo-Egyptian weapon (§6.22, §6.24).
/// Distances > 10 are out of range for all weapons.
pub fn ae_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
    if distance.value() > 10 {
        return RangeBand::OutOfRange;
    }
    if weapon == WeaponClass::Melee {
        return if distance.value() == 1 {
            RangeBand::Normal
        } else {
            RangeBand::OutOfRange
        };
    }
    let range = hex_distance_to_range(distance);
    use Range::*;
    use RangeBand::*;
    match weapon {
        WeaponClass::Rifles | WeaponClass::Maxims => match range {
            One => Doubled,
            Two | Three => Normal,
            Four | Five => Halved,
            _ => OutOfRange,
        },
        WeaponClass::Artillery => match range {
            One => Tripled,
            Two => Doubled,
            Three | Four | Five | Six => Normal,
            Seven | Eight => Halved,
            _ => OutOfRange,
        },
        WeaponClass::Howitzer => match range {
            One | Two | Three => OutOfRange,
            _ => Halved,
        },
        WeaponClass::Melee => unreachable!(),
    }
}

/// Look up the range band for a Dervish weapon (§6.22).
/// Distances > 10 are out of range for all weapons.
pub fn dervish_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
    if distance.value() > 10 {
        return RangeBand::OutOfRange;
    }
    if weapon == WeaponClass::Melee {
        return if distance.value() == 1 {
            RangeBand::Normal
        } else {
            RangeBand::OutOfRange
        };
    }
    let range = hex_distance_to_range(distance);
    use Range::*;
    use RangeBand::*;
    match weapon {
        WeaponClass::Rifles => match range {
            One | Two => Normal,
            Three | Four => Halved,
            _ => OutOfRange,
        },
        WeaponClass::Artillery => match range {
            One => Doubled,
            Two | Three | Four => Normal,
            Five | Six | Seven => Halved,
            _ => OutOfRange,
        },
        WeaponClass::Maxims | WeaponClass::Howitzer => match range {
            One | Two => Normal,
            Three | Four => Halved,
            _ => OutOfRange,
        },
        WeaponClass::Melee => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ae_rifles_doubled_at_range_1() {
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance(1)),
            RangeBand::Doubled
        );
    }

    #[test]
    fn ae_rifles_halved_at_range_4() {
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance(4)),
            RangeBand::Halved
        );
    }

    #[test]
    fn ae_howitzer_range() {
        assert_eq!(
            ae_range_effects(WeaponClass::Howitzer, HexDistance(1)),
            RangeBand::OutOfRange
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Howitzer, HexDistance(4)),
            RangeBand::Halved
        );
    }

    #[test]
    fn dervish_rifles_shorter_range() {
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance(5)),
            RangeBand::OutOfRange
        );
    }

    #[test]
    fn melee_only_range_1() {
        assert_eq!(
            ae_range_effects(WeaponClass::Melee, HexDistance(1)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Melee, HexDistance(2)),
            RangeBand::OutOfRange
        );
    }
}
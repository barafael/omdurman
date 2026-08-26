use crate::{HexDistance, RangeBand, WeaponClass};

/// The faction rows of the Range Effects Table.
type FactionRows = std::collections::HashMap<WeaponClass, Vec<RangeBand>>;

fn faction_rows(ae: bool) -> &'static FactionRows {
    let table = crate::tables_data::range_effects_data();
    if ae {
        &table.anglo_egyptian
    } else {
        &table.dervish
    }
}

fn band_at(rows: &FactionRows, weapon: WeaponClass, distance: HexDistance) -> RangeBand {
    if distance.value() == 0 || distance.value() > 10 {
        return RangeBand::OutOfRange;
    }
    // Spears appear on the printed table as the "Melee" line: Normal at
    // range 1, out of range beyond (§2.31).
    if weapon == WeaponClass::Melee {
        return if distance.value() == 1 {
            RangeBand::Normal
        } else {
            RangeBand::OutOfRange
        };
    }
    rows.get(&weapon)
        .and_then(|cells| cells.get((distance.value() - 1) as usize).copied())
        .unwrap_or(RangeBand::OutOfRange)
}

/// Look up the range band for an Anglo-Egyptian weapon (§6.22, §6.24).
/// Distances > 10 are out of range for all weapons.
pub fn ae_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
    band_at(faction_rows(true), weapon, distance)
}

/// Look up the range band for a Dervish weapon (§6.22).
/// Distances > 10 are out of range for all weapons.
pub fn dervish_range_effects(weapon: WeaponClass, distance: HexDistance) -> RangeBand {
    band_at(faction_rows(false), weapon, distance)
}

/// The maximum day-time range (in hexes) at which a weapon's fire factor is
/// anything but `OutOfRange`. Used to compute the night maximum: §8.1 says "all
/// fire ranges are halved (round down, but range 1 stays range 1)".
pub fn max_day_range(weapon: WeaponClass, ae: bool) -> u8 {
    if weapon == WeaponClass::Melee {
        return 1;
    }
    let rows = faction_rows(ae);
    let Some(cells) = rows.get(&weapon) else {
        return 1;
    };
    // Howitzer fires only in the MaximSecondAndHowitzer subphase, never at
    // night (§8.1); the day max is 10 but this value is irrelevant at night
    // because the howitzer-at-night ban short-circuits before range lookup.
    for (idx, band) in cells.iter().enumerate().rev() {
        if band.in_range() {
            return (idx + 1) as u8;
        }
    }
    1
}

/// The halved maximum range at night (§8.1): round down, minimum 1.
pub fn night_max_range(weapon: WeaponClass, ae: bool) -> u8 {
    let day = max_day_range(weapon, ae);
    if day <= 1 { 1 } else { day / 2 }
}

/// The range band for a weapon at night (§8.1).
///
/// Night fire is only allowed if the physical distance ≤ `night_max_range`.
/// Within that limit the *daytime* range-band table is used unchanged
/// (the night restriction is purely on maximum range, §8.1).
/// Returns `OutOfRange` if the distance exceeds the night cap.
pub fn night_range_effects(weapon: WeaponClass, distance: HexDistance, ae: bool) -> RangeBand {
    let cap = night_max_range(weapon, ae) as u16;
    if distance.value() > cap {
        RangeBand::OutOfRange
    } else {
        if ae {
            ae_range_effects(weapon, distance)
        } else {
            dervish_range_effects(weapon, distance)
        }
    }
}

/// Per-firer display data for the fire visualiser panel.
///
/// One entry per firing unit in a group attack.  The caller builds a
/// `Vec<PerFirerRow>` for a target and renders it in the combat-card
/// overlay — one row per firer with range band, factor and modifier.
#[derive(Clone, Copy, Debug)]
pub struct PerFirerRow {
    /// Opaque caller-side key (typically an ECS entity id).
    pub key: u64,
    /// Physical hex distance from firer to target.
    pub distance: HexDistance,
    /// Range band *after* night capping (§8.1) — the caller passes
    /// `is_night` and the faction info.
    pub range_band: RangeBand,
    /// The firer's printed fire factor (§6.11).
    pub factor: u16,
    /// Terrain defence modifier at the target hex (§6.23).
    pub defense_modifier: i16,
}

/// Build one `PerFirerRow` for a single firer.
///
/// Call this once per firing unit to build up the rows for the overlay.
pub fn single_firer_row(
    key: u64,
    distance: HexDistance,
    weapon: WeaponClass,
    fire_factor: u16,
    is_ae: bool,
    is_night: bool,
    defense_modifier: i16,
) -> PerFirerRow {
    let range_band = if is_night {
        night_range_effects(weapon, distance, is_ae)
    } else if is_ae {
        ae_range_effects(weapon, distance)
    } else {
        dervish_range_effects(weapon, distance)
    };
    PerFirerRow {
        key,
        distance,
        range_band,
        factor: fire_factor,
        defense_modifier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traceability_macro::rulebook;

    #[rulebook("§6.22")]
    #[test]
    fn ae_rifles_doubled_at_range_1() {
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(1)),
            RangeBand::Doubled
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn ae_rifles_halved_at_range_4() {
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(4)),
            RangeBand::Halved
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn ae_howitzer_range() {
        assert_eq!(
            ae_range_effects(WeaponClass::Howitzer, HexDistance::new(1)),
            RangeBand::OutOfRange
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Howitzer, HexDistance::new(4)),
            RangeBand::Halved
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn dervish_rifles_shorter_range() {
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(5)),
            RangeBand::OutOfRange
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn melee_only_range_1() {
        assert_eq!(
            ae_range_effects(WeaponClass::Melee, HexDistance::new(1)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Melee, HexDistance::new(2)),
            RangeBand::OutOfRange
        );
    }

    // -- §8.1 night range halving ------------------------------------------
    // The rulebook worked example: "an AE infantry unit firing at night will
    // be doubled at range 1, normal at range 2, and may not fire at range 3+."
    // AE rifle day bands: 1=Doubled, 2-3=Normal, 4-5=Halved. Day max=5.
    // Night max = 5/2 = 2.

    #[rulebook("§8.1")]
    #[test]
    fn night_max_ranges() {
        // AE weapons
        assert_eq!(night_max_range(WeaponClass::Rifles, true), 2);
        assert_eq!(night_max_range(WeaponClass::Maxims, true), 2);
        assert_eq!(night_max_range(WeaponClass::Artillery, true), 4);
        // Dervish weapons
        assert_eq!(night_max_range(WeaponClass::Rifles, false), 2);
        assert_eq!(night_max_range(WeaponClass::Artillery, false), 3);
        assert_eq!(night_max_range(WeaponClass::Melee, false), 1);
    }

    #[rulebook("§8.1")]
    #[test]
    fn night_max_ranges_remaining() {
        assert_eq!(night_max_range(WeaponClass::Howitzer, true), 5);
        // Dervish never fields howitzers (§6.64: only the five named British
        // gunboats); the authored Dervish fallback row is the rifles pattern,
        // so the derived night cap is 2. Unreachable in play either way.
        assert_eq!(night_max_range(WeaponClass::Howitzer, false), 2);
        assert_eq!(night_max_range(WeaponClass::Maxims, false), 2);
        assert_eq!(night_max_range(WeaponClass::Melee, true), 1);
    }

    #[rulebook("§8.1")]
    #[test]
    fn ae_rifle_at_night_matches_rulebook_example() {
        // Physical range 1: ≤ night max (2), day band = Doubled ✓
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(1)),
            RangeBand::Doubled
        );
        // Physical range 2: ≤ night max (2), day band = Normal ✓
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(2)),
            RangeBand::Normal
        );
        // Physical range 3: > night max (2) → Out (the caller rejects this)
        assert!(HexDistance::new(3).value() > night_max_range(WeaponClass::Rifles, true) as u16);
    }

    #[rulebook("§6.22", "§8.1")]
    #[test]
    fn max_day_range_all_combos() {
        assert_eq!(max_day_range(WeaponClass::Melee, true), 1);
        assert_eq!(max_day_range(WeaponClass::Melee, false), 1);
        assert_eq!(max_day_range(WeaponClass::Rifles, true), 5);
        assert_eq!(max_day_range(WeaponClass::Rifles, false), 4);
        assert_eq!(max_day_range(WeaponClass::Maxims, true), 5);
        // Authored Dervish fallback row (rifles pattern) — no Dervish unit
        // fields Maxims, so this value is unreachable in play.
        assert_eq!(max_day_range(WeaponClass::Maxims, false), 4);
        assert_eq!(max_day_range(WeaponClass::Artillery, true), 8);
        assert_eq!(max_day_range(WeaponClass::Artillery, false), 7);
        assert_eq!(max_day_range(WeaponClass::Howitzer, true), 10);
        // Authored Dervish fallback row (rifles pattern); unreachable — only
        // the named British gunboats fire howitzer (§6.64).
        assert_eq!(max_day_range(WeaponClass::Howitzer, false), 4);
    }

    #[rulebook("§6.22")]
    #[test]
    fn ae_range_effects_artillery_full() {
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(1)),
            RangeBand::Tripled
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(2)),
            RangeBand::Doubled
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(3)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(6)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(7)),
            RangeBand::Halved
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(8)),
            RangeBand::Halved
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(9)),
            RangeBand::OutOfRange
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn ae_range_effects_maxims_match_rifles() {
        for d in 1u16..=6 {
            let dist = HexDistance(d);
            assert_eq!(
                ae_range_effects(WeaponClass::Maxims, dist),
                ae_range_effects(WeaponClass::Rifles, dist),
                "Maxims/Rifles differ at distance {d}"
            );
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn ae_range_effects_distance_over_10() {
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(11)),
            RangeBand::OutOfRange
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(20)),
            RangeBand::OutOfRange
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn ae_range_effects_howitzer_halved_4_to_10() {
        for d in 4u16..=10 {
            assert_eq!(
                ae_range_effects(WeaponClass::Howitzer, HexDistance(d)),
                RangeBand::Halved,
                "Howitzer at distance {d}"
            );
        }
        assert_eq!(
            ae_range_effects(WeaponClass::Howitzer, HexDistance::new(11)),
            RangeBand::OutOfRange
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn dervish_range_effects_rifles() {
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(1)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(2)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(3)),
            RangeBand::Halved
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(4)),
            RangeBand::Halved
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(5)),
            RangeBand::OutOfRange
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn dervish_range_effects_artillery() {
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(1)),
            RangeBand::Doubled
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(2)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(4)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(5)),
            RangeBand::Halved
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(7)),
            RangeBand::Halved
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(8)),
            RangeBand::OutOfRange
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn dervish_range_effects_maxims_and_howitzer() {
        for d in 1u16..=5 {
            let dist = HexDistance(d);
            assert_eq!(
                dervish_range_effects(WeaponClass::Maxims, dist),
                dervish_range_effects(WeaponClass::Howitzer, dist),
                "Maxims/Howitzer differ at distance {d}"
            );
        }
        assert_eq!(
            dervish_range_effects(WeaponClass::Maxims, HexDistance::new(1)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Maxims, HexDistance::new(2)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Maxims, HexDistance::new(3)),
            RangeBand::Halved
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Maxims, HexDistance::new(5)),
            RangeBand::OutOfRange
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn dervish_range_effects_melee() {
        assert_eq!(
            dervish_range_effects(WeaponClass::Melee, HexDistance::new(1)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Melee, HexDistance::new(2)),
            RangeBand::OutOfRange
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn dervish_range_effects_distance_over_10() {
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(11)),
            RangeBand::OutOfRange
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(15)),
            RangeBand::OutOfRange
        );
    }

    // §6.22 -- exhaustive cell-by-cell verification against range_effects_table.ron

    #[rulebook("§6.22")]
    #[test]
    fn range_effects_every_cell_dervish_spears() {
        // Spears: 1:x1, 2..=10: -
        assert_eq!(
            dervish_range_effects(WeaponClass::Melee, HexDistance::new(1)),
            RangeBand::Normal
        );
        for d in 2u16..=10 {
            assert_eq!(
                dervish_range_effects(WeaponClass::Melee, HexDistance(d)),
                RangeBand::OutOfRange,
                "Dervish Spears at distance {d}"
            );
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn range_effects_every_cell_dervish_rifles() {
        // Rifles: 1..=2: x1, 3..=4: x1/2, 5..=10: -
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(1)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(2)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(3)),
            RangeBand::Halved
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Rifles, HexDistance::new(4)),
            RangeBand::Halved
        );
        for d in 5u16..=10 {
            assert_eq!(
                dervish_range_effects(WeaponClass::Rifles, HexDistance(d)),
                RangeBand::OutOfRange,
                "Dervish Rifles at distance {d}"
            );
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn range_effects_every_cell_dervish_artillery() {
        // Artillery: 1: x2, 2..=4: x1, 5..=7: x1/2, 8..=10: -
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(1)),
            RangeBand::Doubled
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(2)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(3)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(4)),
            RangeBand::Normal
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(5)),
            RangeBand::Halved
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(6)),
            RangeBand::Halved
        );
        assert_eq!(
            dervish_range_effects(WeaponClass::Artillery, HexDistance::new(7)),
            RangeBand::Halved
        );
        for d in 8u16..=10 {
            assert_eq!(
                dervish_range_effects(WeaponClass::Artillery, HexDistance(d)),
                RangeBand::OutOfRange,
                "Dervish Artillery at distance {d}"
            );
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn range_effects_every_cell_dervish_maxims_howitzer() {
        // Maxims/Howitzer share table: 1..=2: x1, 3..=4: x1/2, 5..=10: -
        for weapon in [WeaponClass::Maxims, WeaponClass::Howitzer] {
            assert_eq!(
                dervish_range_effects(weapon, HexDistance::new(1)),
                RangeBand::Normal
            );
            assert_eq!(
                dervish_range_effects(weapon, HexDistance::new(2)),
                RangeBand::Normal
            );
            assert_eq!(
                dervish_range_effects(weapon, HexDistance::new(3)),
                RangeBand::Halved
            );
            assert_eq!(
                dervish_range_effects(weapon, HexDistance::new(4)),
                RangeBand::Halved
            );
            for d in 5u16..=10 {
                assert_eq!(
                    dervish_range_effects(weapon, HexDistance(d)),
                    RangeBand::OutOfRange,
                    "Dervish {weapon:?} at distance {d}"
                );
            }
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn range_effects_every_cell_ae_rifles() {
        // AE Rifles: 1: x2, 2..=3: x1, 4..=5: x1/2, 6..=10: -
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(1)),
            RangeBand::Doubled
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(2)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(3)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(4)),
            RangeBand::Halved
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Rifles, HexDistance::new(5)),
            RangeBand::Halved
        );
        for d in 6u16..=10 {
            assert_eq!(
                ae_range_effects(WeaponClass::Rifles, HexDistance(d)),
                RangeBand::OutOfRange,
                "AE Rifles at distance {d}"
            );
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn range_effects_every_cell_ae_maxims() {
        // AE Maxims: identical to Rifles
        for d in 1u16..=10 {
            assert_eq!(
                ae_range_effects(WeaponClass::Maxims, HexDistance(d)),
                ae_range_effects(WeaponClass::Rifles, HexDistance(d)),
                "AE Maxims differ from Rifles at distance {d}"
            );
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn range_effects_every_cell_ae_artillery() {
        // AE Artillery: 1: x3, 2: x2, 3..=6: x1, 7..=8: x1/2, 9..=10: -
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(1)),
            RangeBand::Tripled
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(2)),
            RangeBand::Doubled
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(3)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(4)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(5)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(6)),
            RangeBand::Normal
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(7)),
            RangeBand::Halved
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(8)),
            RangeBand::Halved
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(9)),
            RangeBand::OutOfRange
        );
        assert_eq!(
            ae_range_effects(WeaponClass::Artillery, HexDistance::new(10)),
            RangeBand::OutOfRange
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn range_effects_every_cell_ae_howitzer() {
        // AE Howitzer: 1..=3: -, 4..=10: x1/2
        for d in 1u16..=3 {
            assert_eq!(
                ae_range_effects(WeaponClass::Howitzer, HexDistance(d)),
                RangeBand::OutOfRange,
                "AE Howitzer at distance {d}"
            );
        }
        for d in 4u16..=10 {
            assert_eq!(
                ae_range_effects(WeaponClass::Howitzer, HexDistance(d)),
                RangeBand::Halved,
                "AE Howitzer at distance {d}"
            );
        }
    }

    // -- Property tests: monotonicity ---------------------------------------

    fn band_order(b: RangeBand) -> u8 {
        match b {
            RangeBand::Tripled => 4,
            RangeBand::Doubled => 3,
            RangeBand::Normal => 2,
            RangeBand::Halved => 1,
            RangeBand::OutOfRange => 0,
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn ae_range_effects_monotone_non_increasing() {
        // Howitzers are excluded: they have a minimum range (OOR at 1-3,
        // then Halved at 4+) which is non-monotone by design.
        for weapon in [WeaponClass::Rifles, WeaponClass::Maxims, WeaponClass::Artillery] {
            let mut prev_order = 5u8;
            for d in 1u16..=11 {
                let dist = HexDistance(d);
                let band = ae_range_effects(weapon, dist);
                let order = band_order(band);
                assert!(
                    order <= prev_order,
                    "AE {weapon:?} non-monotone at distance {d}: {band:?} (order {order}) > prev order {prev_order}"
                );
                prev_order = order;
            }
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn ae_howitzer_has_minimum_range() {
        // Howitzers: OOR at ranges 1-3, Halved at ranges 4-10, OOR at 11+.
        for d in 1u16..=3 {
            assert_eq!(
                ae_range_effects(WeaponClass::Howitzer, HexDistance(d)),
                RangeBand::OutOfRange,
                "Howitzer should be OOR at range {d}"
            );
        }
        for d in 4u16..=10 {
            assert_eq!(
                ae_range_effects(WeaponClass::Howitzer, HexDistance(d)),
                RangeBand::Halved,
                "Howitzer should be Halved at range {d}"
            );
        }
        assert_eq!(
            ae_range_effects(WeaponClass::Howitzer, HexDistance(11)),
            RangeBand::OutOfRange,
            "Howitzer should be OOR at range 11"
        );
    }

    #[rulebook("§6.22")]
    #[test]
    fn dervish_range_effects_monotone_non_increasing() {
        for weapon in [WeaponClass::Rifles, WeaponClass::Artillery, WeaponClass::Melee] {
            let mut prev_order = 5u8;
            for d in 1u16..=11 {
                let dist = HexDistance(d);
                let band = dervish_range_effects(weapon, dist);
                let order = band_order(band);
                assert!(
                    order <= prev_order,
                    "Dervish {weapon:?} non-monotone at distance {d}: {band:?} (order {order}) > prev order {prev_order}"
                );
                prev_order = order;
            }
        }
    }

    #[rulebook("§6.22")]
    #[test]
    fn range_effects_first_range_max_effect_last_range_oor() {
        // For ranged weapons, range 1 should be the maximum effectiveness
        // (or OOR for howitzer), and range 11+ should always be OOR.
        // AE
        assert_eq!(ae_range_effects(WeaponClass::Rifles, HexDistance::new(1)), RangeBand::Doubled);
        assert_eq!(ae_range_effects(WeaponClass::Artillery, HexDistance::new(1)), RangeBand::Tripled);
        assert_eq!(ae_range_effects(WeaponClass::Rifles, HexDistance::new(11)), RangeBand::OutOfRange);
        assert_eq!(ae_range_effects(WeaponClass::Artillery, HexDistance::new(11)), RangeBand::OutOfRange);
        assert_eq!(ae_range_effects(WeaponClass::Howitzer, HexDistance::new(11)), RangeBand::OutOfRange);
        // Dervish
        assert_eq!(dervish_range_effects(WeaponClass::Rifles, HexDistance::new(1)), RangeBand::Normal);
        assert_eq!(dervish_range_effects(WeaponClass::Artillery, HexDistance::new(1)), RangeBand::Doubled);
        assert_eq!(dervish_range_effects(WeaponClass::Rifles, HexDistance::new(11)), RangeBand::OutOfRange);
        assert_eq!(dervish_range_effects(WeaponClass::Artillery, HexDistance::new(11)), RangeBand::OutOfRange);
    }
}

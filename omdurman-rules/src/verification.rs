//! Kani proof harnesses for the rules-engine value types (`cargo kani`, see
//! `scripts/kani.sh`).
//!
//! Scope note: the four printed tables (Combat Results, Range Effects,
//! Scattergram, Line of Sight) live as `static` constants in `tables_data`
//! (parity-checked against the authored RON by its `#[cfg(test)]` tests), so
//! proofs over the table-backed functions see plain data and can verify the
//! contents symbolically -- see the `verification` modules in
//! `range_effects`, `combat_results_table`, and `howitzer_scatter`. What is
//! proven here is the arithmetic and the conversions around the other
//! lookups -- the parts that are pure functions of small enum domains.

use super::{
    CampaignVictoryLevel, DieRoll, FireModifier, MeleeModifier, MovementAllowance, RangeBand,
};

// -- value_enum! conversions -------------------------------------------
//
// `value_enum!` generates `value() -> u16` and `TryFrom<u16>`. The
// round-trip `try_from(x.value()) == Ok(x)` requires `value` to be
// injective: it holds today by inspection, but a new variant reusing an
// existing printed value would break it silently. These iterate `ALL`, so
// they cover new variants automatically.

/// Round-trip and injectivity for every `value_enum!` enum.
macro_rules! prove_value_enum {
    ($name:ident, $ty:ty) => {
        #[kani::proof]
        fn $name() {
            let all = <$ty>::ALL;
            let i: usize = kani::any();
            let j: usize = kani::any();
            kani::assume(i < all.len());
            kani::assume(j < all.len());
            // `TryFrom` inverts `value`.
            assert!(<$ty>::try_from(all[i].value()) == Ok(all[i]));
            // Distinct variants never share a printed value.
            if i != j {
                assert!(all[i].value() != all[j].value());
            }
        }
    };
}

prove_value_enum!(fire_factor_value_roundtrips, super::FireFactor);
prove_value_enum!(melee_factor_value_roundtrips, super::MeleeFactor);
prove_value_enum!(die_roll_value_roundtrips, DieRoll);
prove_value_enum!(battalion_ordinal_value_roundtrips, super::BattalionOrdinal);
prove_value_enum!(movement_allowance_value_roundtrips, MovementAllowance);

/// §8.1 halves the movement allowance at night. The `expect` in
/// [`MovementAllowance::halve`] is safe only because every variant's value
/// halves onto *another* variant -- an arithmetic coincidence a new variant
/// could break (e.g. `TwentyTwo = 22` halves to 11, which is not a
/// variant, and would panic). This proves it holds for all variants,
/// including any added later.
// §8.1
#[kani::proof]
fn movement_allowance_halve_never_panics() {
    let i: usize = kani::any();
    kani::assume(i < MovementAllowance::ALL.len());
    let a = MovementAllowance::ALL[i];
    let halved = a.halve();
    // Halving is exactly integer division by two, and never increases.
    assert!(halved.value() == a.value() / 2);
    assert!(halved.value() <= a.value());
}

// -- die-roll arithmetic (§6.24, §7.7) ---------------------------------

/// An arbitrary legal die roll.
fn any_roll() -> DieRoll {
    let i: usize = kani::any();
    kani::assume(i < DieRoll::ALL.len());
    DieRoll::ALL[i]
}

/// `apply_modifier` is total over *every* `i16`. It is a `pub` method
/// taking an unconstrained modifier, and `FireAttack::net_modifier` folds
/// an unbounded list whose `FireModifier::Terrain(i16)` arrives over the
/// network -- so a plain `+` overflowed here. Saturating arithmetic plus
/// the 1..=10 clamp makes it total, which also makes the
/// `unwrap_or(DieRoll::Ten)` fallback unreachable.
// §6.24
#[kani::proof]
fn die_roll_apply_modifier_is_total() {
    let roll = any_roll();
    let modifier: i16 = kani::any();
    let out = roll.apply_modifier(modifier);
    assert!(out.value() >= 1 && out.value() <= 10);
}

/// A larger modifier never yields a lower roll. The outcome-prediction UI
/// renders modifier bands assuming this.
#[kani::proof]
fn die_roll_apply_modifier_is_monotone() {
    let roll = any_roll();
    let a: i16 = kani::any();
    let b: i16 = kani::any();
    kani::assume(a <= b);
    assert!(roll.apply_modifier(a).value() <= roll.apply_modifier(b).value());
}

/// A zero modifier is the identity.
#[kani::proof]
fn die_roll_zero_modifier_is_identity() {
    let roll = any_roll();
    assert!(roll.apply_modifier(0) == roll);
}

/// Applying any single fire modifier keeps the roll legal, including
/// `FireModifier::Terrain(n)` for an arbitrary `n` -- the variant that
/// carries an unbounded `i16` straight off the wire (§6.23).
#[kani::proof]
fn fire_modifier_keeps_roll_legal() {
    let n: i16 = kani::any();
    let mods = [
        FireModifier::AngloEgyptianDirectFire,
        FireModifier::BrigadeIntegrity,
        FireModifier::Terrain(n),
        FireModifier::ZaribaThornHedge,
        FireModifier::ZaribaTrenchEntrenched,
    ];
    let i: usize = kani::any();
    kani::assume(i < mods.len());
    let out = any_roll().apply_modifier(mods[i].die_modifier());
    assert!(out.value() >= 1 && out.value() <= 10);
}

/// Same for melee modifiers (§7.7, §9.232).
// §7.7
#[kani::proof]
fn melee_modifier_keeps_roll_legal() {
    let mods = [
        MeleeModifier::DervishStandard,
        MeleeModifier::AngloEgyptianStandard,
        MeleeModifier::DervishVsTrenchedDefender,
        MeleeModifier::FriendliesStandard,
    ];
    let i: usize = kani::any();
    kani::assume(i < mods.len());
    let out = any_roll().apply_modifier(mods[i].die_modifier());
    assert!(out.value() >= 1 && out.value() <= 10);
}

// -- Night movement (§8.1) ---------------------------------------------

/// §8.1: "all Anglo-Egyptian movement allowances are halved (round down)"
/// -- and *only* Anglo-Egyptian, and only at night. The Dervish player's
/// allowances and every day-turn allowance pass through unchanged.
// §8.1
#[kani::proof]
fn night_halving_is_ae_only_and_day_neutral() {
    use super::effective_movement_at_night;
    use omdurman_types::{DayNight, Player};
    let i: usize = kani::any();
    kani::assume(i < MovementAllowance::ALL.len());
    let a = MovementAllowance::ALL[i];
    // Dervish: never halved, day or night.
    assert!(effective_movement_at_night(a, Player::Dervish, DayNight::Night) == a);
    assert!(effective_movement_at_night(a, Player::Dervish, DayNight::Day) == a);
    // Anglo-Egyptian: day is neutral; night halves exactly (round down).
    assert!(effective_movement_at_night(a, Player::AngloEgyptian, DayNight::Day) == a);
    let night = effective_movement_at_night(a, Player::AngloEgyptian, DayNight::Night);
    assert!(night.value() == a.value() / 2);
    assert!(night.value() <= a.value());
}

// -- Brigade integrity (§5.54) -----------------------------------------

/// §5.54: a stack has brigade integrity exactly when all four *distinct*
/// battalions of one Anglo-Egyptian brigade are present. Proven over a
/// symbolic four-counter stack (two symbolic brigades, each counter a
/// symbolic battalion): any duplicate or mixed brigade destroys the bonus,
/// any full same-brigade set grants it carrying that brigade.
// §5.54
#[kani::proof]
fn brigade_integrity_requires_all_four_distinct_battalions_of_one_brigade() {
    use super::brigade_integrity;
    use super::{BattalionOrdinal, BrigadeIntegrity, UnitIdentity};
    use omdurman_types::{BrigadeId, BrigadeNationality};
    let battalions = [
        BattalionOrdinal::First,
        BattalionOrdinal::Second,
        BattalionOrdinal::Third,
        BattalionOrdinal::Fourth,
    ];
    let infantry = |brigade: usize, battalion: usize| UnitIdentity::AngloEgyptianInfantry {
        brigade: BrigadeId {
            number: brigade as u8 + 1,
            nationality: BrigadeNationality::British,
        },
        battalion: battalions[battalion],
    };
    let b: usize = kani::any();
    kani::assume(b < 2);
    let o0: usize = kani::any();
    let o1: usize = kani::any();
    let o2: usize = kani::any();
    let o3: usize = kani::any();
    kani::assume(o0 < 4);
    kani::assume(o1 < 4);
    kani::assume(o2 < 4);
    kani::assume(o3 < 4);
    let i0 = infantry(b, o0);
    let b1: usize = kani::any();
    let b2: usize = kani::any();
    let b3: usize = kani::any();
    let i1 = infantry(b1 % 2, o1);
    let i2 = infantry(b2 % 2, o2);
    let i3 = infantry(b3 % 2, o3);
    let stack = [i0, i1, i2, i3];
    let all_same_brigade = stack.iter().all(|i| i.brigade() == i0.brigade());
    let ordinals = [o0, o1, o2, o3];
    let pairwise_distinct = ordinals[0] != ordinals[1]
        && ordinals[0] != ordinals[2]
        && ordinals[0] != ordinals[3]
        && ordinals[1] != ordinals[2]
        && ordinals[1] != ordinals[3]
        && ordinals[2] != ordinals[3];
    let expected_brigade = i0.brigade();
    match brigade_integrity(&stack) {
        BrigadeIntegrity::Integrated(brigade) => {
            assert!(all_same_brigade && pairwise_distinct);
            assert!(Some(brigade) == expected_brigade);
        }
        BrigadeIntegrity::None => {
            assert!(!(all_same_brigade && pairwise_distinct));
        }
    }
    // Non-infantry units never form a brigade (no brigade designation).
    assert!(brigade_integrity(&[UnitIdentity::AngloEgyptianCavalry; 4]) == BrigadeIntegrity::None);
    assert!(brigade_integrity(&[]) == BrigadeIntegrity::None);
}

// -- Fire-factor bands (§6.22) -----------------------------------------

/// §6.22: the printed CRT bands are 1-5, 6-10, 11-15, ..., 36-40, 41+.
/// `from_total` places every total in the right band (arithmetic spec:
/// band index = `(total-1)/5` clamped to the top row) and is monotone,
/// so a stronger attack never consults a weaker row.
// §6.22
#[kani::proof]
fn fire_factor_row_from_total_matches_printed_bands() {
    use crate::combat_results_table::FireFactorRow;
    let total: u16 = kani::any();
    let row = FireFactorRow::from_total(total);
    let expected_index = if total == 0 {
        0
    } else {
        (((total - 1) / 5) as usize).min(FireFactorRow::ALL.len() - 1)
    };
    assert!(row.index() == expected_index);
    // Monotone: more factors never select an earlier row.
    let smaller: u16 = kani::any();
    if smaller <= total {
        assert!(FireFactorRow::from_total(smaller).index() <= row.index());
    }
    // The printed band edges land exactly.
    assert!(FireFactorRow::from_total(5) == FireFactorRow::Row01to05);
    assert!(FireFactorRow::from_total(6) == FireFactorRow::Row06to10);
    assert!(FireFactorRow::from_total(40) == FireFactorRow::Row36to40);
    assert!(FireFactorRow::from_total(41) == FireFactorRow::Row41Plus);
}

// -- Victory level ladders (§9.14, §9.24, §9.35) -----------------------

/// Rank a campaign level on the signed ladder (Dervish-favourable low).
fn campaign_level_rank(level: &CampaignVictoryLevel) -> i32 {
    use super::CampaignVictoryLevel as V;
    use omdurman_types::Player::{AngloEgyptian as AE, Dervish as D};
    match level {
        V::Decisive(D) => -3,
        V::Tactical(D) => -2,
        V::Marginal(D) => -1,
        V::Draw => 0,
        V::Marginal(AE) => 1,
        V::Tactical(AE) => 2,
        V::Decisive(AE) => 3,
    }
}

/// §9.14: the printed superiority schedule, proven exact over every i32:
/// Anglo-Egyptian bands 1-14 draw / 15-29 marginal / 30-49 tactical /
/// 50+ decisive; Dervish bands 1-9 draw / 10-19 marginal / 20-29
/// tactical / 30+ decisive; and net superiority never scores *against*
/// the side that holds it (monotone along the whole ladder).
// §9.14
#[kani::proof]
fn campaign_victory_levels_match_manual_superiority_table() {
    use super::CampaignVictoryLevel as V;
    use super::VictoryPoints;
    use omdurman_types::Player;
    use omdurman_types::Player::{AngloEgyptian as AE, Dervish as D};
    let net: i32 = kani::any();
    let level = CampaignVictoryLevel::from_superiority(VictoryPoints::new(net));
    // The side named by a non-draw level matches the sign of the net.
    match level {
        CampaignVictoryLevel::Draw => assert!(net >= -9 && net <= 14),
        CampaignVictoryLevel::Marginal(player)
        | CampaignVictoryLevel::Tactical(player)
        | CampaignVictoryLevel::Decisive(player) => {
            if net > 0 {
                assert!(player == Player::AngloEgyptian && net >= 15);
            } else {
                assert!(player == Player::Dervish && net <= -10);
            }
        }
    }
    // Monotone: more net superiority never helps the opponent.
    let smaller: i32 = kani::any();
    if smaller <= net {
        assert!(
            campaign_level_rank(&CampaignVictoryLevel::from_superiority(VictoryPoints::new(
                smaller
            ))) <= campaign_level_rank(&level)
        );
    }
    // Exact printed band edges.
    assert!(CampaignVictoryLevel::from_superiority(VictoryPoints::new(14)) == V::Draw);
    assert!(CampaignVictoryLevel::from_superiority(VictoryPoints::new(15)) == V::Marginal(AE));
    assert!(CampaignVictoryLevel::from_superiority(VictoryPoints::new(49)) == V::Tactical(AE));
    assert!(CampaignVictoryLevel::from_superiority(VictoryPoints::new(50)) == V::Decisive(AE));
    assert!(CampaignVictoryLevel::from_superiority(VictoryPoints::new(-9)) == V::Draw);
    assert!(CampaignVictoryLevel::from_superiority(VictoryPoints::new(-10)) == V::Marginal(D));
    assert!(CampaignVictoryLevel::from_superiority(VictoryPoints::new(-29)) == V::Tactical(D));
    assert!(CampaignVictoryLevel::from_superiority(VictoryPoints::new(-30)) == V::Decisive(D));
}

/// §9.24: both elimination ladders, proven exact over every i16: the
/// Anglo-Egyptian column 0-29/30-44/45-59/60-99/100+ and the Dervish
/// column 0-4/5-9/10-14/15-29/30+ (draw through decisive), monotone in
/// eliminations, with negative counts clamped to draw.
// §9.24
#[kani::proof]
fn historical_victory_ladders_match_manual_bands() {
    use super::HistoricalVictoryLevel;
    let n: i16 = kani::any();
    let ae = HistoricalVictoryLevel::for_anglo_egyptian(n);
    let expected_ae = if n >= 100 {
        HistoricalVictoryLevel::Decisive
    } else if n >= 60 {
        HistoricalVictoryLevel::Strategic
    } else if n >= 45 {
        HistoricalVictoryLevel::Tactical
    } else if n >= 30 {
        HistoricalVictoryLevel::Marginal
    } else {
        HistoricalVictoryLevel::Draw
    };
    assert!(ae == expected_ae);
    let d = HistoricalVictoryLevel::for_dervish(n);
    let expected_d = if n >= 30 {
        HistoricalVictoryLevel::Decisive
    } else if n >= 15 {
        HistoricalVictoryLevel::Strategic
    } else if n >= 10 {
        HistoricalVictoryLevel::Tactical
    } else if n >= 5 {
        HistoricalVictoryLevel::Marginal
    } else {
        HistoricalVictoryLevel::Draw
    };
    assert!(d == expected_d);
    // Monotone in eliminations on both ladders.
    let smaller: i16 = kani::any();
    if smaller <= n {
        assert!(HistoricalVictoryLevel::for_anglo_egyptian(smaller) <= ae);
        assert!(HistoricalVictoryLevel::for_dervish(smaller) <= d);
    }
}

/// §9.35: the FoK ladder. The Dervish loss penalty kicks in exactly at
/// 16/24/32 units (monotone, and `next_loss_threshold` always names the
/// next penalty step); more losses shift the final level toward the
/// British end (never backwards); and the rulebook's own worked example
/// resolves exactly: GORDON dies turn 5 with 24 Dervish losses nets a
/// British marginal.
// §9.35
#[kani::proof]
fn fok_victory_ladder_penalties_shift_monotonically() {
    use super::FoKVictoryLevel;
    let lost: i16 = kani::any();
    // Penalty schedule: 0 below 16, then 1/2/3 at 16/24/32.
    let expected = if lost >= 32 {
        3
    } else if lost >= 24 {
        2
    } else if lost >= 16 {
        1
    } else {
        0
    };
    assert!(FoKVictoryLevel::loss_penalty(lost) == expected);
    // `next_loss_threshold` names exactly the next penalty step.
    match FoKVictoryLevel::next_loss_threshold(lost) {
        Some(t) => assert!(FoKVictoryLevel::loss_penalty(t) == expected + 1),
        None => assert!(expected == 3),
    }
    // More losses never move the result toward the Dervish end.
    let died: Option<u8> = if kani::any() {
        let t: u8 = kani::any();
        kani::assume((1..=8).contains(&t));
        Some(t)
    } else {
        None
    };
    let end: u8 = kani::any();
    let base = FoKVictoryLevel::resolve(died, end, lost);
    let less = FoKVictoryLevel::resolve(died, end, if lost > i16::MIN { lost - 1 } else { lost });
    let ladder_rank = |l: &FoKVictoryLevel| {
        FoKVictoryLevel::LADDER
            .iter()
            .position(|x| x == l)
            .unwrap_or(FoKVictoryLevel::DEFAULT_LADDER_IDX) as i32
    };
    assert!(ladder_rank(&base) >= ladder_rank(&less));
    // The rulebook's worked example (§9.35).
    assert!(FoKVictoryLevel::resolve(Some(5), 8, 24) == FoKVictoryLevel::BritishMarginal);
}

// -- Range-band scaling (§6.16, §6.22) ---------------------------------

/// §6.16: halving rounds down per unit and "a unit's firing strength is
/// never reduced below one by halving". So `Halved` is exactly
/// `max(1, raw/2)` for *every* raw factor -- held symbolically for a
/// saturating-division-free reference so a new band or a rounding drift
/// is caught.
// §6.16
#[kani::proof]
fn range_band_halved_is_max_of_one_and_floor_half() {
    let raw: u16 = kani::any();
    let folded = RangeBand::Halved.apply(raw);
    assert!(folded == (raw / 2).max(1));
    // Floor at 1: never falls below one.
    assert!(folded >= 1);
    // Rounding down: never exceeds `raw` for a positive factor.
    if raw >= 1 {
        assert!(folded <= raw);
    }
}

/// §6.16: scaling is monotone non-decreasing in the printed fire factor --
/// a unit with more printed strength can never fire *less* after any band
/// is applied. Holds for every band (howitzers' minimum range is about
/// *distance*, not raw strength, so it does not violate this).
// §6.16
#[kani::proof]
fn range_band_apply_is_monotone_in_raw() {
    let a: u16 = kani::any();
    let b: u16 = kani::any();
    kani::assume(a <= b);
    for band in [
        RangeBand::Tripled,
        RangeBand::Doubled,
        RangeBand::Normal,
        RangeBand::Halved,
    ] {
        assert!(band.apply(a) <= band.apply(b));
    }
}

/// §6.22: the printed table's multiplier rows are exact arithmetic --
/// Tripled/Doubled/Normal scale by 3/2/1 (with saturating arithmetic so a
/// huge factor cannot wrap), and `OutOfRange` zeroes the strength.
// §6.22
#[kani::proof]
fn range_band_multiplier_arithmetic_is_exact() {
    let raw: u16 = kani::any();
    assert!(RangeBand::Tripled.apply(raw) == raw.saturating_mul(3));
    assert!(RangeBand::Doubled.apply(raw) == raw.saturating_mul(2));
    assert!(RangeBand::Normal.apply(raw) == raw);
    assert!(RangeBand::OutOfRange.apply(raw) == 0);
    // Scaling up never reduces a non-zero factor.
    if raw >= 1 {
        assert!(RangeBand::Doubled.apply(raw) >= raw);
        assert!(RangeBand::Tripled.apply(raw) >= raw);
    }
}

/// §6.16: D's "round up" interpretation of the CRT `D` (½ of target units
/// disrupted) -- the CRT `D` result disrupts ceil(n/2) of `n` target units,
/// i.e. half rounded *up*, never leaving a full disrupted stack at zero.
/// Mirrors `apply_combat_results_table_result`'s `div_ceil(2)` (see
/// `fire.rs`) without depending on table cell values. `div_ceil` is the
/// overflow-safe spelling of `(n+1)/2` that the CRT path already uses.
// §CRT
#[kani::proof]
fn disrupt_half_is_rounded_up() {
    let n: usize = kani::any();
    let disrupted = n.div_ceil(2);
    // Rounding up: at least half (half the stack or more), and never more
    // than the stack itself.
    assert!(disrupted >= n / 2);
    assert!(disrupted <= n);
    // The empty stack is the degenerate case (0 of 0).
    if n == 0 {
        assert!(disrupted == 0);
    }
    // div_ceil is exactly floor-half plus the odd remainder -- the printed
    // "round up" of half.
    assert!(disrupted == n / 2 + n % 2);
}

// -- River mines (§10.12) ----------------------------------------------

/// §10.12: the printed mine resolution bands are 1-4 no effect, 5-7
/// engines lost, 8-10 sunk. `from_roll` reproduces exactly those bands
/// over the whole d10 domain -- a reshuffled band edge would desync the
/// engine from the printed optional rule.
// §10.12
#[kani::proof]
fn mine_result_bands_match_the_printed_rule() {
    let roll = any_roll();
    let v = roll.value();
    let result = super::MineResult::from_roll(roll);
    if v <= 4 {
        assert!(result == super::MineResult::NoEffect);
    } else if v <= 7 {
        assert!(result == super::MineResult::EnginesLost);
    } else {
        assert!(result == super::MineResult::Sunk);
    }
}

// -- Victory-point schedule (§9.14) ------------------------------------

/// §9.14: every printed VP award, proven exact over the whole source
/// enum: Mahdi's Tomb 25 to whichever side controls it at the end, Khalifa
/// 10, Isa Zachneih 1, 1 per Dervish unit, 10 per British leader / gunboat
/// sunk, 1/3 per Friendlies (east/west bank), 3 per Anglo-Egyptian land
/// unit -- and each award goes to the player whose section of the printed
/// schedule it comes from (the Friendlies losses are in the *Dervish*
/// player's list, like every other Anglo-Egyptian loss). This is
/// the table the whole victory ledger folds over, so a drifted value
/// silently changes every scenario verdict.
// §9.14
#[kani::proof]
fn vp_source_points_and_scorer_match_the_printed_schedule() {
    use super::VpSource;
    use omdurman_types::Player;
    let table = [
        (VpSource::MahdisTombTaken, 25, Player::AngloEgyptian),
        (VpSource::IsaZachneihEliminated, 1, Player::AngloEgyptian),
        (VpSource::KhalifaEliminated, 10, Player::AngloEgyptian),
        (VpSource::DervishUnitEliminated, 1, Player::AngloEgyptian),
        (VpSource::MahdisTombHeld, 25, Player::Dervish),
        (VpSource::FriendliesEastBankEliminated, 1, Player::Dervish),
        (VpSource::FriendliesWestBankEliminated, 3, Player::Dervish),
        (VpSource::BritishLeaderEliminated, 10, Player::Dervish),
        (VpSource::BritishGunboatSunk, 10, Player::Dervish),
        (
            VpSource::AngloEgyptianLandUnitEliminated,
            3,
            Player::Dervish,
        ),
    ];
    let i: usize = kani::any();
    kani::assume(i < table.len());
    let (source, points, scorer) = table[i];
    assert!(source.points() == super::VictoryPoints::new(points));
    assert!(source.who_scores() == scorer);
    // Every award is positive (0-pt sources are modelled as `None`,
    // never as a zero-valued variant).
    assert!(source.points().value() >= 1);
}

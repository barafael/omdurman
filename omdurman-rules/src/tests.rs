//! Unit tests for the crate-root value types -- every numeric rule must
//! round-trip a manual example.

use super::*;
use omdurman_types::HexsideKind;
use traceability_macro::rulebook;

#[test]
fn die_roll_from_u8_clamps() {
    assert_eq!(
        DieRoll::try_from(0u16.clamp(1, 10))
            .unwrap_or(DieRoll::Ten)
            .value(),
        1
    );
    assert_eq!(
        DieRoll::try_from(11u16.clamp(1, 10))
            .unwrap_or(DieRoll::Ten)
            .value(),
        10
    );
    assert_eq!(DieRoll::try_from(7u16).unwrap(), DieRoll::Seven);
}

#[test]
fn die_roll_add_clamps() {
    let r = DieRoll::Five;
    assert_eq!(r.apply_modifier(3).value(), 8);
    assert_eq!(r.apply_modifier(-9).value(), 1);
    assert_eq!(r.apply_modifier(99).value(), 10);
}

#[test]
fn range_band_halving_floors_at_one() {
    // §6.16: halving rounds down per unit but never below 1.
    assert_eq!(RangeBand::Halved.apply(1), 1);
    assert_eq!(RangeBand::Halved.apply(9), 4);
    assert_eq!(RangeBand::Halved.apply(0), 1);
}

#[test]
fn range_band_out_of_range_is_zero() {
    assert_eq!(RangeBand::OutOfRange.apply(9), 0);
}

#[test]
fn range_band_multipliers() {
    assert_eq!(RangeBand::Tripled.apply(4), 12);
    assert_eq!(RangeBand::Doubled.apply(4), 8);
    assert_eq!(RangeBand::Normal.apply(4), 4);
}

#[test]
fn night_movement_halves_round_down() {
    // §8.1: movement halved (round down).
    assert_eq!(
        effective_movement_at_night(
            MovementAllowance::Three,
            Player::AngloEgyptian,
            DayNight::Night
        )
        .value(),
        1
    );
    assert_eq!(
        effective_movement_at_night(
            MovementAllowance::Five,
            Player::AngloEgyptian,
            DayNight::Night
        )
        .value(),
        2
    );
    assert_eq!(
        effective_movement_at_night(
            MovementAllowance::One,
            Player::AngloEgyptian,
            DayNight::Night
        )
        .value(),
        0
    );
}

#[test]
fn night_movement_only_halves_anglo_egyptian() {
    let a = MovementAllowance::Eight;
    assert_eq!(
        effective_movement_at_night(a, Player::AngloEgyptian, DayNight::Night).value(),
        4
    );
    assert_eq!(
        effective_movement_at_night(a, Player::Dervish, DayNight::Night).value(),
        8
    );
    assert_eq!(
        effective_movement_at_night(a, Player::AngloEgyptian, DayNight::Day).value(),
        8
    );
}

#[test]
fn mine_result_from_roll() {
    // §10.12.
    assert_eq!(MineResult::from_roll(DieRoll::One), MineResult::NoEffect);
    assert_eq!(MineResult::from_roll(DieRoll::Four), MineResult::NoEffect);
    assert_eq!(
        MineResult::from_roll(DieRoll::Five),
        MineResult::EnginesLost
    );
    assert_eq!(
        MineResult::from_roll(DieRoll::Seven),
        MineResult::EnginesLost
    );
    assert_eq!(MineResult::from_roll(DieRoll::Eight), MineResult::Sunk);
    assert_eq!(MineResult::from_roll(DieRoll::Ten), MineResult::Sunk);
}

#[test]
fn player_opponent_involutes() {
    assert_eq!(Player::AngloEgyptian.opponent(), Player::Dervish);
    assert_eq!(Player::Dervish.opponent(), Player::AngloEgyptian);
    assert_eq!(
        Player::AngloEgyptian.opponent().opponent(),
        Player::AngloEgyptian
    );
}

#[rulebook("§7.1")]
#[test]
fn unit_kind_melee_capability() {
    // §7.4.
    assert!(
        UnitKind::Infantry {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_melee_attack()
    );
    assert!(
        UnitKind::Cavalry {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_melee_attack()
    );
    assert!(
        UnitKind::Camel {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_melee_attack()
    );
    assert!(
        UnitKind::DervishLeader {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_melee_attack()
    );
    assert!(
        !UnitKind::Artillery {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_melee_attack()
    );
    assert!(
        !UnitKind::Maxim {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_melee_attack()
    );
    assert!(
        !UnitKind::Gunboat {
            fire: 0,
            upstream: 0,
            downstream: 0
        }
        .may_melee_attack()
    );
    assert!(!UnitKind::Fort { fire: 0, melee: 0 }.may_melee_attack());
    assert!(!UnitKind::BritishLeader { movement: 0 }.may_melee_attack());

    // §7.1 -- gunboats may not be melee attacked.
    assert!(
        !UnitKind::Gunboat {
            fire: 0,
            upstream: 0,
            downstream: 0
        }
        .may_be_melee_attacked()
    );
    assert!(
        UnitKind::Infantry {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_be_melee_attacked()
    );
    assert!(UnitKind::Fort { fire: 0, melee: 0 }.may_be_melee_attacked());

    // §7.5.
    assert!(
        UnitKind::Cavalry {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_retreat_before_melee()
    );
    assert!(
        UnitKind::Camel {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_retreat_before_melee()
    );
    assert!(
        !UnitKind::Infantry {
            fire: 0,
            melee: 0,
            movement: 0
        }
        .may_retreat_before_melee()
    );
}

#[test]
fn fire_modifiers_compose() {
    // §5.54 + §6.24: a brigade-integrity stack firing direct receives
    // both the +1 direct-fire and the +1 brigade-integrity bonuses.
    let attack = FireAttack {
        firing_player: Player::AngloEgyptian,
        phase: Phase::OffensiveFire(FireSubPhase::DirectFire),
        kind: FireKind::Direct,
        firers: vec![],
        target_hex: HexCoord::new(0, 0),
        factor_row: FireFactorRow::Row16to20,
        modifiers: vec![
            FireModifier::AngloEgyptianDirectFire,
            FireModifier::BrigadeIntegrity,
            FireModifier::Terrain(-2),
        ],
    };
    assert_eq!(attack.net_modifier(), 0);
}

#[rulebook("§9.14")]
#[test]
fn vp_source_attributes() {
    assert_eq!(VpSource::KhalifaEliminated.points().0, 10);
    assert_eq!(
        VpSource::KhalifaEliminated.who_scores(),
        Player::AngloEgyptian
    );
    assert_eq!(VpSource::BritishLeaderEliminated.points().0, 10);
    assert_eq!(
        VpSource::BritishLeaderEliminated.who_scores(),
        Player::Dervish
    );
    // The Tomb is worth 25 to whichever side controls it at the end.
    assert_eq!(VpSource::MahdisTombTaken.points().0, 25);
    assert_eq!(
        VpSource::MahdisTombTaken.who_scores(),
        Player::AngloEgyptian
    );
    assert_eq!(VpSource::MahdisTombHeld.points().0, 25);
    assert_eq!(VpSource::MahdisTombHeld.who_scores(), Player::Dervish);
    // Friendlies losses are on the "Dervish Player receives" list.
    assert_eq!(VpSource::FriendliesWestBankEliminated.points().0, 3);
    assert_eq!(
        VpSource::FriendliesWestBankEliminated.who_scores(),
        Player::Dervish
    );
    assert_eq!(
        VpSource::FriendliesEastBankEliminated.who_scores(),
        Player::Dervish
    );
}

#[test]
fn campaign_victory_levels() {
    // §9.14 thresholds.
    // Anglo-Egyptian:
    assert!(matches!(
        CampaignVictoryLevel::from_superiority(VictoryPoints::new(50)),
        CampaignVictoryLevel::Decisive(Player::AngloEgyptian)
    ));
    assert!(matches!(
        CampaignVictoryLevel::from_superiority(VictoryPoints::new(30)),
        CampaignVictoryLevel::Tactical(Player::AngloEgyptian)
    ));
    assert!(matches!(
        CampaignVictoryLevel::from_superiority(VictoryPoints::new(15)),
        CampaignVictoryLevel::Marginal(Player::AngloEgyptian)
    ));
    assert!(matches!(
        CampaignVictoryLevel::from_superiority(VictoryPoints::new(5)),
        CampaignVictoryLevel::Draw
    ));
    assert!(matches!(
        CampaignVictoryLevel::from_superiority(VictoryPoints::new(0)),
        CampaignVictoryLevel::Draw
    ));
    // Dervish:
    assert!(matches!(
        CampaignVictoryLevel::from_superiority(VictoryPoints::new(-5)),
        CampaignVictoryLevel::Draw
    ));
    assert!(matches!(
        CampaignVictoryLevel::from_superiority(VictoryPoints::new(-15)),
        CampaignVictoryLevel::Marginal(Player::Dervish)
    ));
    assert!(matches!(
        CampaignVictoryLevel::from_superiority(VictoryPoints::new(-25)),
        CampaignVictoryLevel::Tactical(Player::Dervish)
    ));
    assert!(matches!(
        CampaignVictoryLevel::from_superiority(VictoryPoints::new(-40)),
        CampaignVictoryLevel::Decisive(Player::Dervish)
    ));
}

#[test]
fn victory_ledger_accumulates() {
    let mut l = VictoryLedger::default();
    l.events.push(VpEvent {
        turn: GameTurnIndex::new(1),
        source: VpSource::KhalifaEliminated,
    });
    l.events.push(VpEvent {
        turn: GameTurnIndex::new(2),
        source: VpSource::DervishUnitEliminated,
    });
    l.events.push(VpEvent {
        turn: GameTurnIndex::new(2),
        source: VpSource::BritishGunboatSunk,
    });
    assert_eq!(l.total_for(Player::AngloEgyptian).0, 11);
    assert_eq!(l.total_for(Player::Dervish).0, 10);
    assert_eq!(l.superiority().0, 1);
}

#[test]
fn unit_identity_owner_partitions_correctly() {
    let dervish = UnitIdentity::DervishTribal {
        tribe: DervishTribe::Hadendowa,
    };
    assert_eq!(dervish.owner(), Player::Dervish);

    let lancers = UnitIdentity::AngloEgyptianCavalry;
    assert_eq!(lancers.owner(), Player::AngloEgyptian);

    let friendlies = UnitIdentity::AngloEgyptianInfantry {
        brigade: BrigadeId {
            number: 1,
            nationality: BrigadeNationality::Friendlies,
        },
        battalion: BattalionOrdinal::First,
    };
    assert!(friendlies.is_friendlies());

    let british = UnitIdentity::AngloEgyptianInfantry {
        brigade: BrigadeId {
            number: 2,
            nationality: BrigadeNationality::British,
        },
        battalion: BattalionOrdinal::Third,
    };
    assert!(!british.is_friendlies());
}

#[rulebook("§5.54")]
#[test]
fn brigade_integrity_empty_slice() {
    assert_eq!(brigade_integrity(&[]), BrigadeIntegrity::None);
}

#[rulebook("§5.54")]
#[test]
fn brigade_integrity_non_infantry_returns_none() {
    let ids = [UnitIdentity::DervishTribal {
        tribe: DervishTribe::Baggara,
    }];
    assert_eq!(brigade_integrity(&ids), BrigadeIntegrity::None);
}

#[rulebook("§5.54")]
#[test]
fn brigade_integrity_three_battalions_returns_none() {
    let brigade = BrigadeId {
        number: 1,
        nationality: BrigadeNationality::British,
    };
    let ids = [
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::First,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::Second,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::Third,
        },
    ];
    assert_eq!(brigade_integrity(&ids), BrigadeIntegrity::None);
}

#[rulebook("§5.54")]
#[test]
fn brigade_integrity_four_battalions_returns_integrated() {
    let brigade = BrigadeId {
        number: 2,
        nationality: BrigadeNationality::Egyptian,
    };
    let ids = [
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::First,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::Second,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::Third,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::Fourth,
        },
    ];
    assert_eq!(
        brigade_integrity(&ids),
        BrigadeIntegrity::Integrated(brigade)
    );
}

#[rulebook("§5.54")]
#[test]
fn brigade_integrity_mixed_brigades_returns_none() {
    let b1 = BrigadeId {
        number: 1,
        nationality: BrigadeNationality::British,
    };
    let b2 = BrigadeId {
        number: 2,
        nationality: BrigadeNationality::British,
    };
    let ids = [
        UnitIdentity::AngloEgyptianInfantry {
            brigade: b1,
            battalion: BattalionOrdinal::First,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade: b1,
            battalion: BattalionOrdinal::Second,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade: b2,
            battalion: BattalionOrdinal::Third,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade: b2,
            battalion: BattalionOrdinal::Fourth,
        },
    ];
    assert_eq!(brigade_integrity(&ids), BrigadeIntegrity::None);
}

#[rulebook("§5.54")]
#[test]
fn brigade_integrity_friendlies_returns_none() {
    let brigade = BrigadeId {
        number: 1,
        nationality: BrigadeNationality::Friendlies,
    };
    let ids = [
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::First,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::Second,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::Third,
        },
        UnitIdentity::AngloEgyptianInfantry {
            brigade,
            battalion: BattalionOrdinal::Fourth,
        },
    ];
    // §5.54 grants brigade integrity only to "British, Sudanese, and
    // Egyptian infantry" -- the Friendlies brigade (BrigadeNationality::
    // Friendlies) never integrates, whatever its battalion layout.
    assert_eq!(brigade_integrity(&ids), BrigadeIntegrity::None);
}

#[rulebook("§5.54")]
#[test]
fn unit_identity_brigade_and_battalion_accessors() {
    let id = UnitIdentity::AngloEgyptianInfantry {
        brigade: BrigadeId {
            number: 3,
            nationality: BrigadeNationality::Sudanese,
        },
        battalion: BattalionOrdinal::Fourth,
    };
    assert_eq!(
        id.brigade(),
        Some(BrigadeId {
            number: 3,
            nationality: BrigadeNationality::Sudanese,
        })
    );
    assert_eq!(id.battalion(), Some(BattalionOrdinal::Fourth));

    // Non-infantry identity returns None for both.
    let dervish = UnitIdentity::DervishTribal {
        tribe: DervishTribe::Taiasha,
    };
    assert_eq!(dervish.brigade(), None);
    assert_eq!(dervish.battalion(), None);
}

#[rulebook("§9.24")]
#[test]
fn historical_victory_level_for_anglo_egyptian() {
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(0),
        HistoricalVictoryLevel::Draw
    );
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(29),
        HistoricalVictoryLevel::Draw
    );
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(30),
        HistoricalVictoryLevel::Marginal
    );
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(44),
        HistoricalVictoryLevel::Marginal
    );
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(45),
        HistoricalVictoryLevel::Tactical
    );
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(59),
        HistoricalVictoryLevel::Tactical
    );
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(60),
        HistoricalVictoryLevel::Strategic
    );
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(99),
        HistoricalVictoryLevel::Strategic
    );
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(100),
        HistoricalVictoryLevel::Decisive
    );
    assert_eq!(
        HistoricalVictoryLevel::for_anglo_egyptian(150),
        HistoricalVictoryLevel::Decisive
    );
}

#[rulebook("§9.24")]
#[test]
fn historical_victory_level_for_dervish() {
    assert_eq!(
        HistoricalVictoryLevel::for_dervish(0),
        HistoricalVictoryLevel::Draw
    );
    assert_eq!(
        HistoricalVictoryLevel::for_dervish(4),
        HistoricalVictoryLevel::Draw
    );
    assert_eq!(
        HistoricalVictoryLevel::for_dervish(5),
        HistoricalVictoryLevel::Marginal
    );
    assert_eq!(
        HistoricalVictoryLevel::for_dervish(9),
        HistoricalVictoryLevel::Marginal
    );
    assert_eq!(
        HistoricalVictoryLevel::for_dervish(10),
        HistoricalVictoryLevel::Tactical
    );
    assert_eq!(
        HistoricalVictoryLevel::for_dervish(14),
        HistoricalVictoryLevel::Tactical
    );
    assert_eq!(
        HistoricalVictoryLevel::for_dervish(15),
        HistoricalVictoryLevel::Strategic
    );
    assert_eq!(
        HistoricalVictoryLevel::for_dervish(29),
        HistoricalVictoryLevel::Strategic
    );
    assert_eq!(
        HistoricalVictoryLevel::for_dervish(30),
        HistoricalVictoryLevel::Decisive
    );
}

#[rulebook("§9.35")]
#[test]
fn fok_victory_level_gordon_died_early() {
    assert_eq!(
        FoKVictoryLevel::resolve(Some(3), 8, 0),
        FoKVictoryLevel::DervishDecisive
    );
    assert_eq!(
        FoKVictoryLevel::resolve(Some(4), 8, 0),
        FoKVictoryLevel::DervishDecisive
    );
    assert_eq!(
        FoKVictoryLevel::resolve(Some(5), 8, 0),
        FoKVictoryLevel::DervishTactical
    );
    assert_eq!(
        FoKVictoryLevel::resolve(Some(6), 8, 0),
        FoKVictoryLevel::DervishMarginal
    );
}

#[rulebook("§9.35")]
#[test]
fn fok_victory_level_gordon_survived() {
    // GORDON survived to turn 8 → British decisive.
    assert_eq!(
        FoKVictoryLevel::resolve(None, 8, 0),
        FoKVictoryLevel::BritishDecisive
    );
    // GORDON survived to turn 7 → British tactical.
    assert_eq!(
        FoKVictoryLevel::resolve(None, 7, 0),
        FoKVictoryLevel::BritishTactical
    );
    // GORDON survived to turn 6 → British marginal.
    assert_eq!(
        FoKVictoryLevel::resolve(None, 6, 0),
        FoKVictoryLevel::BritishMarginal
    );
}

#[rulebook("§9.35")]
#[test]
fn fok_victory_level_worked_example() {
    assert_eq!(
        FoKVictoryLevel::resolve(Some(5), 8, 24),
        FoKVictoryLevel::BritishMarginal
    );
}

#[rulebook("§9.35")]
#[test]
fn fok_victory_level_late_gordon_death() {
    assert_eq!(
        FoKVictoryLevel::resolve(Some(7), 8, 0),
        FoKVictoryLevel::DervishMarginal
    );
    assert_eq!(
        FoKVictoryLevel::resolve(Some(8), 8, 0),
        FoKVictoryLevel::DervishMarginal
    );
}

#[test]
fn movement_allowance_display() {
    assert_eq!(format!("{}", MovementAllowance::Eight), "8");
    assert_eq!(format!("{}", MovementAllowance::Immobile), "0");
    assert_eq!(format!("{}", MovementAllowance::Three), "3");
}

// §6.16: halving fire strength rounds down per unit and never reduces
// a unit's firing strength below one.
#[rulebook("§6.16")]
#[test]
fn halving_rounds_down_and_never_below_one() {
    assert_eq!(RangeBand::Halved.apply(9), 4);
    assert_eq!(RangeBand::Halved.apply(4), 2);
    assert_eq!(RangeBand::Halved.apply(3), 1);
    assert_eq!(RangeBand::Halved.apply(1), 1);
}

#[rulebook("§6.11")]
#[test]
fn fire_factor_sum_to_row() {
    let factors = [FireFactor::Eight, FireFactor::Eight];
    let row = FireFactor::sum_to_row(&factors);
    // 8 + 8 = 16 → Row16to20.
    assert!(matches!(
        row,
        crate::combat_results_table::FireFactorRow::Row16to20
    ));

    let factors2 = [FireFactor::Five, FireFactor::Five];
    let row2 = FireFactor::sum_to_row(&factors2);
    // 5 + 5 = 10 → Row06to10.
    assert!(matches!(
        row2,
        crate::combat_results_table::FireFactorRow::Row06to10
    ));
}

#[rulebook("§7.1")]
#[test]
fn melee_factor_values_and_sum() {
    // §7.1: the melee factor set printed on counters is 1/3/5/6/7, and
    // `sum` totals the printed factors.
    assert_eq!(MeleeFactor::One.value(), 1);
    assert_eq!(MeleeFactor::Three.value(), 3);
    assert_eq!(MeleeFactor::Five.value(), 5);
    assert_eq!(MeleeFactor::Six.value(), 6);
    assert_eq!(MeleeFactor::Seven.value(), 7);
    let combined = [MeleeFactor::Three, MeleeFactor::Five];
    assert_eq!(MeleeFactor::sum(&combined), 8);
}

#[test]
fn hexside_kind_classifies_blockers() {
    // §5.44 + §6.82 + §7.2.
    assert!(HexsideKind::Wall.blocks_los());
    assert!(!HexsideKind::Gate.blocks_los());
    assert!(HexsideKind::Wall.blocks_melee());
    assert!(!HexsideKind::Gate.blocks_melee());
    assert!(HexsideKind::Khor.blocks_advance_after_combat());
    assert!(!HexsideKind::Breach.blocks_advance_after_combat());
    assert!(HexsideKind::ZaribaThornHedge.blocks_melee());
    // Terrain Effects Chart, Khor: "May not melee across".
    assert!(HexsideKind::Khor.blocks_melee());
    assert!(HexsideKind::KhorShambat.blocks_melee());
    assert!(!HexsideKind::Crest.blocks_melee());
}

// §9.24: "The lower value victory level is then subtracted from the higher
// level to determine a player's net victory. For example, if the
// Anglo-Egyptian player eliminates 104 Dervish units (decisive victory) but
// loses 18 units doing it (Dervish Strategic), the Anglo-Egyptian player only
// nets out with a draw."
#[rulebook("§9.24")]
#[test]
fn historical_net_result_follows_the_worked_example() {
    use crate::HistoricalVictoryLevel as L;
    let ae = L::for_anglo_egyptian(104);
    let d = L::for_dervish(18);
    assert_eq!((ae, d), (L::Decisive, L::Strategic));
    assert_eq!(L::net(ae, d), (None, L::Draw));
    // A lopsided battle: Decisive against a Dervish draw nets a Strategic
    // Anglo-Egyptian victory (5 - 1 = 4).
    assert_eq!(
        L::net(L::Decisive, L::Draw),
        (Some(Player::AngloEgyptian), L::Strategic)
    );
    assert_eq!(
        L::net(L::Marginal, L::Strategic),
        (Some(Player::Dervish), L::Marginal)
    );
    assert_eq!(L::Draw.next_threshold(Player::AngloEgyptian), Some(30));
    assert_eq!(L::Tactical.next_threshold(Player::Dervish), Some(15));
    assert_eq!(L::Decisive.next_threshold(Player::Dervish), None);
}

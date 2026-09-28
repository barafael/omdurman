//! The Campaign game's set-up and order of appearance (§9.111-§9.113) on the
//! real Omdurman board: where the Dervish initial force stands, where the
//! reinforcements come in, and what may arrive when.

use omdurman_rules::board::BoardInfo;
use omdurman_rules::board_data::campaign_map_data;
use omdurman_rules::effects::{GameEffect, GameState, RuleError, apply_effect};
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_rules::{GameTurnIndex, Phase, UnitId, UnitPlacement};
use omdurman_types::{HexCoord, Location, NamedArea, Player, Scenario};
use traceability_macro::rulebook;

fn campaign() -> GameState {
    let board = BoardInfo::from_map_data(&campaign_map_data());
    GameState::with_board(Scenario::Campaign, board)
}

fn at(id: UnitId, hex: HexCoord) -> UnitPlacement {
    UnitPlacement {
        id,
        position: hex,
        profile: profile_for_unit(id).expect("a printed counter"),
        state: Default::default(),
    }
}

fn located(state: &GameState, want: Location) -> Vec<HexCoord> {
    state
        .board
        .locations
        .iter()
        .filter(|(_, l)| **l == want)
        .map(|(h, _)| *h)
        .collect()
}

// §9.111: the Dervish initial force and where each part of it sets up.
#[rulebook("§9.111")]
#[test]
fn the_dervish_initial_force_sets_up_where_printed() {
    let state = campaign();
    // 1 Isa Zachneih + the Khalifa + 3 guns + 14 Taiasha + 17 forts + 2
    // gunboats; the Anglo-Egyptians start with nothing (§9.113).
    assert_eq!(state.setup_target(Player::Dervish), 38);
    assert_eq!(state.setup_target(Player::AngloEgyptian), 0);
    let initial_force = UnitId::ALL
        .iter()
        .filter_map(|id| profile_for_unit(*id))
        .filter(|p| omdurman_rules::effects::in_campaign_initial_force(&p.identity))
        .count();
    assert_eq!(initial_force, 38, "the roster matches the printed count");

    let palace = located(&state, Location::Palace)[0];
    let grounds = located(&state, Location::PalaceGrounds)[0];
    let khalifa = UnitId::KhalifaAbdullah_0_0;
    assert!(state.can_deploy_unit(&at(khalifa, palace)).is_ok());
    assert!(state.can_deploy_unit(&at(khalifa, grounds)).is_ok());
    let tomb = located(&state, Location::MahdisTomb)[0];
    assert!(matches!(
        state.can_deploy_unit(&at(khalifa, tomb)),
        Err(RuleError::CampaignSetUpArea { .. })
    ));

    // The Taiasha anywhere in the walled city, not outside it.
    assert!(
        state
            .can_deploy_unit(&at(UnitId::Taiasha_0_0, tomb))
            .is_ok()
    );
    let outside = HexCoord::new(30, 38);
    assert!(!state.board.is_walled_city(outside));
    assert!(
        state
            .can_deploy_unit(&at(UnitId::Taiasha_0_0, outside))
            .is_err()
    );

    // Isa Zachneih: east bank, in or south of El Debeba.
    let debeba = located(&state, Location::ElDebeba)
        .into_iter()
        .min_by_key(|h| h.r)
        .unwrap();
    let isa = UnitId::Hadendowa_0_0;
    assert!(state.can_deploy_unit(&at(isa, debeba)).is_ok());
    let north_of_debeba = HexCoord::new(debeba.q, debeba.r - 2);
    assert!(state.can_deploy_unit(&at(isa, north_of_debeba)).is_err());

    // Forts south of the Khor Shambat on the west bank -- not north of it.
    let fort = UnitId::HadendowaForts_0_0;
    assert!(state.board.south_of_khor_shambat.contains(&palace));
    let letter_a = HexCoord::new(15, 4);
    assert!(!state.board.south_of_khor_shambat.contains(&letter_a));
    assert!(state.can_deploy_unit(&at(fort, outside)).is_ok());
    assert!(matches!(
        state.can_deploy_unit(&at(fort, letter_a)),
        Err(RuleError::CampaignSetUpArea { .. })
    ));
    // ... or on the east bank south of every Halfaya hut.
    let halfaya_south = located(&state, Location::Halfaya)
        .into_iter()
        .map(|h| h.r)
        .max()
        .unwrap();
    let east_below = state
        .board
        .terrain
        .keys()
        .copied()
        .find(|h| {
            h.r == halfaya_south + 1
                && state.board.bank_of(*h) == Some(omdurman_rules::board::NileBank::East)
        })
        .unwrap();
    assert!(state.can_deploy_unit(&at(fort, east_below)).is_ok());

    // The two gunboats on south-edge Nile hexes.
    let boat = UnitId::KhalifaAbdullah_1_0;
    let max_r = state.board.terrain.keys().map(|h| h.r).max().unwrap();
    let south_nile = state
        .board
        .terrain
        .keys()
        .copied()
        .find(|h| h.r == max_r && state.board.is_nile(*h))
        .unwrap();
    assert!(state.can_deploy_unit(&at(boat, south_nile)).is_ok());
    let upriver = HexCoord::new(south_nile.q, south_nile.r - 4);
    assert!(state.board.is_nile(upriver));
    assert!(state.can_deploy_unit(&at(boat, upriver)).is_err());
}

fn movement(state: &mut GameState, player: Player, turn: u8) {
    state.phase = Phase::Movement;
    state.active_player = player;
    state.current_turn = GameTurnIndex::new(turn);
    state.reinforcements_placed_this_turn.clear();
}

// §9.112/§9.113: reinforcements enter through their printed entrance areas.
#[rulebook("§9.112")]
#[rulebook("§9.113")]
#[test]
fn reinforcements_enter_through_their_entrance_areas() {
    let mut state = campaign();
    for (area, n) in [
        (NamedArea::AngloEgyptianEntrance, 4),
        (NamedArea::AbuAlimHut, 1),
    ] {
        assert_eq!(state.board.entrance_hexes(area).len(), n, "{area}");
    }
    let west = state.board.entrance_hexes(NamedArea::DervishWestEdge);
    assert!(
        west.iter()
            .all(|h| state.board.south_of_khor_shambat.contains(h)),
        "the west edge south of the Khor Shambat"
    );
    let north_nile = state.board.entrance_hexes(NamedArea::GunboatNorthEdge);
    assert!(
        north_nile
            .iter()
            .all(|h| state.board.is_nile(*h) && h.r == 0)
    );

    movement(&mut state, Player::Dervish, 1);
    let baggara = UnitId::Baggara_0_0;
    assert!(
        state
            .can_place_single_reinforcement(&at(baggara, west[0]))
            .is_ok()
    );
    assert!(matches!(
        state.can_place_single_reinforcement(&at(baggara, HexCoord::new(20, 20))),
        Err(RuleError::OutsideEntranceArea(_))
    ));

    movement(&mut state, Player::AngloEgyptian, 1);
    let entrance = state.board.entrance_hexes(NamedArea::AngloEgyptianEntrance)[0];
    let cavalry = UnitId::EgyptianArmy_0_0;
    assert!(
        state
            .can_place_single_reinforcement(&at(cavalry, entrance))
            .is_ok()
    );
    assert!(matches!(
        state.can_place_single_reinforcement(&at(cavalry, HexCoord::new(30, 20))),
        Err(RuleError::OutsideEntranceArea(_))
    ));
}

// §9.113: turn 1 brings the printed first wave; turns 2 and 3 any twelve
// land units, leaders free; the leaders are all in by the end of turn 4.
#[rulebook("§9.113")]
#[test]
fn the_anglo_egyptian_waves_follow_the_order_of_appearance() {
    let mut state = campaign();
    let entrance = state.board.entrance_hexes(NamedArea::AngloEgyptianEntrance);
    movement(&mut state, Player::AngloEgyptian, 1);
    // The 21st Lancers (British) are not in the first wave.
    assert!(matches!(
        state.can_place_single_reinforcement(&at(UnitId::BritishArmy_0_0, entrance[0])),
        Err(RuleError::NotInFirstWave(_))
    ));
    // The Horse Artillery is.
    assert!(
        state
            .can_place_single_reinforcement(&at(UnitId::EgyptianArmy_2_0, entrance[0]))
            .is_ok()
    );

    // Turn 2: twelve land units, the leaders on top of them.
    movement(&mut state, Player::AngloEgyptian, 2);
    let land: Vec<UnitId> = UnitId::ALL
        .iter()
        .copied()
        .filter(|id| {
            profile_for_unit(*id).is_some_and(|p| {
                matches!(
                    p.identity,
                    omdurman_rules::UnitIdentity::AngloEgyptianInfantry { .. }
                ) && !p.identity.is_friendlies()
            })
        })
        .take(13)
        .collect();
    let mut placed = 0;
    for (i, id) in land.iter().take(12).enumerate() {
        let hex = entrance[i / 4];
        apply_effect(
            &mut state,
            &GameEffect::PlaceReinforcements(vec![at(*id, hex)]),
        )
        .unwrap_or_else(|e| panic!("arrival {i}: {e}"));
        // Move each arrival on so the entrance hexes stay clear.
        state.units.last_mut().unwrap().position = HexCoord::new(24 + i as i32, 3);
        placed += 1;
    }
    assert_eq!(placed, 12);
    assert!(matches!(
        state.can_place_single_reinforcement(&at(land[12], entrance[0])),
        Err(RuleError::ReinforcementCapExceeded { turn: 2, cap: 12 })
    ));
    let kitchener = UnitId::ALL
        .iter()
        .copied()
        .find(|id| {
            profile_for_unit(*id).is_some_and(|p| {
                p.identity
                    == omdurman_rules::UnitIdentity::AngloEgyptianLeader(
                        omdurman_rules::BritishLeader::Kitchener,
                    )
            })
        })
        .unwrap();
    assert!(
        state
            .can_place_single_reinforcement(&at(kitchener, entrance[3]))
            .is_ok(),
        "leaders do not count against the twelve"
    );

    // Turn 4 cannot end without all three leaders.
    movement(&mut state, Player::AngloEgyptian, 4);
    assert!(matches!(
        apply_effect(&mut state, &GameEffect::AdvancePhase),
        Err(RuleError::LeadersMustEnterByTurnFour)
    ));
}

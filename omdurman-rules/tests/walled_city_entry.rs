//! §5.23 on the real Omdurman board: "Only certain units may enter the walled
//! portion of Omdurman. For the Dervish player these are the Khalifa unit,
//! the three Dervish artillery units, and the Taiasha units ... Any
//! Anglo-Egyptian units that can get to the walled city may enter it (except
//! gunboats and 'Friendlies'). Units entering and/or exiting the walled city
//! may only do so through a gate or breach hexside."

use omdurman_rules::UnitId;
use omdurman_rules::board::BoardInfo;
use omdurman_rules::board_data::campaign_map_data;
use omdurman_rules::effects::{GameState, RuleError};
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_rules::{MovementPoints, Phase};
use omdurman_types::{HexCoord, HexsideKind, Location, Player, Scenario};
use traceability_macro::rulebook;

/// A Campaign movement phase for `player` with `id` standing on `at`.
fn with_unit(player: Player, id: UnitId, at: HexCoord) -> GameState {
    let board = BoardInfo::from_map_data(&campaign_map_data());
    let mut state = GameState::with_board(Scenario::Campaign, board);
    state.phase = Phase::Movement;
    state.active_player = player;
    state.units.push(omdurman_rules::UnitPlacement {
        id,
        position: at,
        profile: profile_for_unit(id).expect("compiled profile"),
        state: Default::default(),
    });
    state
}

// §5.23
#[rulebook("§5.23")]
#[test]
fn only_the_khalifas_men_pass_the_gates_of_omdurman() {
    let board = BoardInfo::from_map_data(&campaign_map_data());
    // The north gate: the Mahdi's Tomb inside, the town outside.
    let (outside, tomb) = (HexCoord::new(30, 38), HexCoord::new(30, 39));
    assert_eq!(
        board.hexside_between(outside, tomb),
        Some(HexsideKind::Gate)
    );
    assert_eq!(board.location_at(tomb), Some(Location::MahdisTomb));
    assert!(board.is_walled_city(tomb) && !board.is_walled_city(outside));
    // A wall hexside elsewhere on the ring.
    let (wall_out, wall_in) = board
        .hexsides
        .iter()
        .filter(|(_, k)| **k == HexsideKind::Wall)
        .map(|(e, _)| {
            if board.is_walled_city(e.a) {
                (e.b, e.a)
            } else {
                (e.a, e.b)
            }
        })
        .find(|(out, inn)| board.is_walled_city(*inn) && !board.is_walled_city(*out))
        .expect("the ring has wall hexsides");

    let step = MovementPoints::new(3);
    let enters =
        |player, id, from, to| with_unit(player, id, from).can_move_unit_to(id, Some(to), step);
    // The Khalifa's bodyguard through the gate: yes.
    assert!(enters(Player::Dervish, UnitId::Taiasha_0_0, outside, tomb).is_ok());
    // ...but not over the wall.
    assert!(enters(Player::Dervish, UnitId::Taiasha_0_0, wall_out, wall_in).is_err());
    // A Jaalin or a Mulazmin, even at the gate: no.
    for id in [UnitId::JaalinI_1_0, UnitId::MulazminI_0_0] {
        assert!(
            matches!(
                enters(Player::Dervish, id, outside, tomb),
                Err(RuleError::WalledCityEntry(_, _))
            ),
            "{id:?}"
        );
    }
    // Anglo-Egyptian infantry may; the Friendlies may not.
    assert!(
        enters(
            Player::AngloEgyptian,
            UnitId::BritishArmy_0_1,
            outside,
            tomb
        )
        .is_ok()
    );
    assert!(matches!(
        enters(Player::AngloEgyptian, UnitId::Kitchener_0_1, outside, tomb),
        Err(RuleError::WalledCityEntry(_, _))
    ));
}

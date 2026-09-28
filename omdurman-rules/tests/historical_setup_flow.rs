//! The Historical scenario's set-up (§9.211/§9.212) on the real Omdurman
//! board: the six Dervish leaders stand on the printed letters, the whole
//! Anglo-Egyptian army fits the Zariba, the Kerreri huts and the river beside
//! the Zariba, and every Dervish counter finds a hex within three hexes of its
//! leader and out of sight of that army. The engine refuses everything else,
//! so this checks both that the areas bind and that they can be satisfied.

use omdurman_rules::UnitId;
use omdurman_rules::board::BoardInfo;
use omdurman_rules::board_data::campaign_map_data;
use omdurman_rules::effects::{GameEffect, GameState, RuleError, apply_effect};
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_types::{HexCoord, Player, Scenario, SetupLetter};
use traceability_macro::rulebook;

fn placement(id: UnitId, hex: HexCoord) -> omdurman_rules::UnitPlacement {
    omdurman_rules::UnitPlacement {
        id,
        position: hex,
        profile: profile_for_unit(id).expect("compiled profile"),
        state: Default::default(),
    }
}

/// Deploy `id` on the first of `candidates` the engine accepts; `Ok(None)`
/// when the counter is not in play in this scenario at all.
fn deploy_first(
    state: &mut GameState,
    id: UnitId,
    candidates: &[HexCoord],
) -> Result<Option<HexCoord>, RuleError> {
    let mut last = None;
    for &hex in candidates {
        match state.can_deploy_unit(&placement(id, hex)) {
            Ok(()) => {
                apply_effect(state, &GameEffect::DeployUnit(placement(id, hex)))
                    .expect("a checked deployment applies");
                return Ok(Some(hex));
            }
            Err(RuleError::NotInPlay(_)) => return Ok(None),
            Err(e) => last = Some(e),
        }
    }
    Err(last.expect("at least one candidate hex"))
}

// §9.211, §9.212
#[rulebook("§9.211")]
#[rulebook("§9.212")]
#[test]
fn historical_setup_completes_on_the_campaign_board() {
    let map = campaign_map_data();
    let board = BoardInfo::from_map_data(&map);

    // §9.212: the lettered hexes as printed on the mapsheet, once each.
    let mut letters: Vec<(SetupLetter, HexCoord)> = map
        .tiles
        .iter()
        .filter_map(|((q, r), t)| t.setup_letter.map(|l| (l, HexCoord::new(*q, *r))))
        .collect();
    letters.sort_by_key(|(l, _)| l.to_string());
    assert_eq!(
        letters,
        vec![
            (SetupLetter::A, HexCoord::new(15, 4)),
            (SetupLetter::D, HexCoord::new(20, 10)),
            (SetupLetter::K, HexCoord::new(28, 21)),
            (SetupLetter::O, HexCoord::new(33, 23)),
            (SetupLetter::S, HexCoord::new(31, 23)),
            (SetupLetter::Y, HexCoord::new(26, 17)),
        ]
    );

    // §9.211: "the 13 hexes of the Zariba" -- open ground: the hex printed
    // "The Zariba" is a label, not a building (a Building there gave its
    // defenders -3 and cost 3 MP to enter).
    assert_eq!(board.zariba.len(), 13, "Zariba hexes: {:?}", board.zariba);
    for hex in &board.zariba {
        assert!(
            !matches!(
                board.terrain_at(*hex),
                Some(omdurman_types::Terrain::Building { .. })
            ),
            "{hex:?} inside the Zariba is no building"
        );
    }
    let zariba: Vec<HexCoord> = board.zariba.iter().copied().collect();
    let kerreri: Vec<HexCoord> = board
        .locations
        .iter()
        .filter(|(_, l)| **l == omdurman_types::Location::Kerreri)
        .map(|(h, _)| *h)
        .collect();
    assert!(!kerreri.is_empty(), "the Kerreri huts are a landmark");
    let beside_zariba: Vec<HexCoord> = board
        .terrain
        .keys()
        .copied()
        .filter(|h| board.is_nile(*h) && h.neighbors().iter().any(|n| board.is_zariba(*n)))
        .collect();

    let mut state = GameState::with_board(Scenario::Historical, board);

    // The scenario's fixed leaders on their letters (host auto-setup).
    for f in omdurman_rules::scenario_setup::fixed_placements(Scenario::Historical) {
        let id = omdurman_rules::unit_id_for_section_pos(f.section, f.col as u8, f.row as u8)
            .expect("a fixed placement names a real counter");
        let omdurman_rules::scenario_setup::SetupAnchor::Letter(letter) = f.anchor else {
            panic!("Historical leaders are anchored on letters");
        };
        let hex = letters.iter().find(|(l, _)| *l == letter).unwrap().1;
        deploy_first(&mut state, id, &[hex]).expect("a leader deploys on his letter");
    }
    assert_eq!(state.setup_deployed_count(Player::Dervish), 6);

    // §9.211: the Anglo-Egyptians set up first.
    let desert = HexCoord::new(20, 12);
    let lancers = UnitId::BritishArmy_0_0;
    assert!(matches!(
        state.can_deploy_unit(&placement(lancers, desert)),
        Err(RuleError::HistoricalSetUpArea { .. })
    ));
    assert!(matches!(
        state.can_deploy_unit(&placement(UnitId::EgyptianArmy_0_0, zariba[0])),
        Err(RuleError::HistoricalSetUpArea { .. })
    ));
    assert!(
        state
            .can_confirm_setup_ready(Player::AngloEgyptian)
            .is_err(),
        "all remaining units set up first (§9.211)"
    );
    let ae: Vec<UnitId> = UnitId::ALL
        .iter()
        .copied()
        .filter(|id| {
            profile_for_unit(*id).is_some_and(|p| p.identity.owner() == Player::AngloEgyptian)
        })
        .collect();
    // Spread the army round-robin over its areas, so every Zariba and
    // Kerreri hex is manned: the widest view the Dervishes must hide from.
    for (k, &id) in ae.iter().enumerate() {
        let profile = profile_for_unit(id).unwrap();
        let area: &[HexCoord] = if profile.kind.is_boat() {
            &beside_zariba
        } else if omdurman_rules::effects::HISTORICAL_KERRERI_UNITS.contains(&id) {
            &kerreri
        } else {
            &zariba
        };
        let mut rotated = area.to_vec();
        rotated.rotate_left(k % area.len());
        deploy_first(&mut state, id, &rotated)
            .unwrap_or_else(|e| panic!("{id:?} finds room in its §9.211 area: {e}"));
    }
    for hex in zariba.iter().chain(&kerreri) {
        assert!(!state.units_in_hex(*hex).is_empty(), "{hex:?} is manned");
    }
    assert_eq!(
        state.setup_target(Player::AngloEgyptian),
        Some(state.setup_deployed_count(Player::AngloEgyptian))
    );
    assert_eq!(state.setup_deployed_count(Player::AngloEgyptian), 52);
    assert!(state.setup_target_met(Player::AngloEgyptian));
    apply_effect(
        &mut state,
        &GameEffect::ConfirmSetupReady {
            player: Player::AngloEgyptian,
        },
    )
    .unwrap();

    // §9.212: within three hexes of the leader of the unit's colour, out of
    // the Anglo-Egyptians' sight.
    let sheik_el_din = HexCoord::new(20, 10);
    let far = HexCoord::new(20, 15);
    assert_eq!(sheik_el_din.distance(far), 5);
    assert!(matches!(
        state.can_deploy_unit(&placement(UnitId::MulazminI_0_0, far)),
        Err(RuleError::SetUpFarFromLeader { .. })
    ));
    // The engine-only ids that duplicate a printed counter are not in play.
    assert!(matches!(
        state.can_deploy_unit(&placement(UnitId::Kehena_0_0, HexCoord::new(15, 5))),
        Err(RuleError::NotInPlay(_))
    ));
    let mut dervish: Vec<UnitId> = UnitId::ALL
        .iter()
        .copied()
        .filter(|id| {
            profile_for_unit(*id).is_some_and(|p| p.identity.owner() == Player::Dervish)
                && state.find_unit(*id).is_none()
        })
        .collect();
    // Hidden ground is scarce south of Jebel Surgham: seat the Hadendowa
    // (Osman Digna has four hidden hexes, one of them Sherif's) before the
    // Danagla, as a careful Dervish player would.
    dervish.sort_by_key(|id| {
        !matches!(
            profile_for_unit(*id).unwrap().identity,
            omdurman_rules::UnitIdentity::DervishTribal {
                tribe: omdurman_types::DervishTribe::Hadendowa
            }
        )
    });
    let all_hexes: Vec<HexCoord> = state.board.terrain.keys().copied().collect();
    let mut deployed = 0;
    for &id in &dervish {
        let near_a_letter: Vec<HexCoord> = all_hexes
            .iter()
            .copied()
            .filter(|h| letters.iter().any(|(_, l)| l.distance(*h) <= 3))
            .collect();
        if deploy_first(&mut state, id, &near_a_letter)
            .unwrap_or_else(|e| panic!("{id:?} finds a hidden hex near its leader: {e}"))
            .is_some()
        {
            deployed += 1;
        }
    }
    // "All remaining Dervish units": every counter in play, or no Ready.
    assert_eq!(
        state.setup_target(Player::Dervish),
        Some(deployed + 6),
        "every Dervish counter found a hidden hex near its leader"
    );
    assert!(state.setup_target_met(Player::Dervish));
    apply_effect(
        &mut state,
        &GameEffect::ConfirmSetupReady {
            player: Player::Dervish,
        },
    )
    .unwrap();
    assert_ne!(state.phase, omdurman_rules::Phase::Setup);
}

/// Play-test helper (ignored): with `HIST_AE_STATE` pointing at a hex-probe
/// state file (`U owner q r disrupted Id ...` lines), print every hex the
/// Dervish may set up on, per leader, for that Anglo-Egyptian deployment.
#[test]
#[ignore]
fn print_hidden_dervish_setup_hexes() {
    let Ok(path) = std::env::var("HIST_AE_STATE") else {
        return;
    };
    let map = campaign_map_data();
    let mut state = GameState::with_board(Scenario::Historical, BoardInfo::from_map_data(&map));
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.first() != Some(&"U") {
            continue;
        }
        let id = *UnitId::ALL
            .iter()
            .find(|u| format!("{u:?}") == f[5])
            .unwrap();
        let hex = HexCoord::new(f[2].parse().unwrap(), f[3].parse().unwrap());
        state.units.push(placement(id, hex));
    }
    for leader in state.units.clone() {
        let omdurman_rules::UnitIdentity::DervishLeader(l) = leader.profile.identity else {
            continue;
        };
        let hexes: Vec<String> = state
            .board
            .terrain
            .keys()
            .copied()
            .filter(|h| {
                h.distance(leader.position) <= 3
                    && state.in_deployment_zone(Player::Dervish, *h, false)
            })
            .map(|h| format!("{},{}", h.q, h.r))
            .collect();
        println!("HIDDEN {l:?} {}", hexes.join(" "));
    }
}

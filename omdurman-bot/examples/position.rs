//! Replay a commander's movement phase from a position read off a running
//! game: the hex probe's unit listing (`driver.sh units`, one line per unit:
//! `owner q r disrupted id identity`).
//!
//! ```sh
//! driver.sh units > units.txt
//! cargo run --release -p omdurman-bot --example position -- historical units.txt dervish [turn]
//! ```
//!
//! Prints every move the commander makes in the phase.

use omdurman_bot::describe::unit_name;
use omdurman_bot::rng::BotRng;
use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_rules::{GameTurnIndex, Phase, UnitId, UnitPlacement, UnitState};
use omdurman_types::{HexCoord, Player, Scenario};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let scenario = match args.first().map(String::as_str) {
        Some("fok") => Scenario::FallOfKhartoum,
        Some("historical") => Scenario::Historical,
        _ => Scenario::Campaign,
    };
    let listing = std::fs::read_to_string(&args[1]).expect("units file");
    let side = match args.get(2).map(String::as_str) {
        Some("ae") => Player::AngloEgyptian,
        _ => Player::Dervish,
    };
    let turn: u8 = args.get(3).and_then(|t| t.parse().ok()).unwrap_or(1);
    let board = omdurman_bot::playthrough::board_for_scenario(scenario);
    let mut state = GameState::with_board(scenario, board);
    for line in listing.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 5 {
            continue;
        }
        let Some(&id) = UnitId::ALL.iter().find(|id| format!("{id:?}") == f[4]) else {
            eprintln!("unknown unit {}", f[4]);
            continue;
        };
        let (q, r): (i32, i32) = (f[1].parse().unwrap(), f[2].parse().unwrap());
        state.units.push(UnitPlacement {
            id,
            position: HexCoord::new(q, r),
            profile: profile_for_unit(id).expect("profile"),
            state: UnitState {
                disrupted: f[3] == "true",
                ..UnitState::default()
            },
        });
    }
    state.phase = Phase::Movement;
    state.active_player = side;
    state.current_turn = GameTurnIndex::new(turn);
    let mut rng = BotRng::from_seed(1);
    let mut memory = omdurman_bot::move_memory::MoveMemory::new();
    for _ in 0..2000 {
        let candidates = omdurman_bot::actions::legal_actions(&state, &mut rng);
        let effect = omdurman_bot::commanders::pick_validated(
            &state,
            side,
            &candidates,
            &mut rng,
            &mut memory,
        );
        if effect == GameEffect::AdvancePhase {
            println!("end of phase");
            break;
        }
        if let GameEffect::MoveUnit { unit_id, to, .. } = &effect {
            let from = state.find_unit(*unit_id).map(|u| u.position);
            println!("{} {:?} -> ({},{})", unit_name(*unit_id), from, to.q, to.r);
        } else {
            println!("{effect:?}");
        }
        if apply_effect(&mut state, &effect).is_err() {
            println!("refused");
            break;
        }
    }
}

//! Play-test helper: for an Anglo-Egyptian Historical deployment, print every
//! hex each Dervish leader's command may set up on (§9.212: within three
//! hexes of the leader, out of the army's line of sight).
//!
//! ```sh
//! cargo run -p omdurman-rules --example historical_hidden_hexes -- <state file>
//! ```
//!
//! The state file holds one `U owner q r disrupted UnitId ...` line per unit
//! (the app's `OMDURMAN_HEX_PROBE` state dump). Output: one
//! `HIDDEN <leader> q,r q,r ...` line per leader.

use omdurman_rules::board::BoardInfo;
use omdurman_rules::board_data::campaign_map_data;
use omdurman_rules::effects::GameState;
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_rules::{UnitId, UnitIdentity, UnitPlacement};
use omdurman_types::{HexCoord, Player, Scenario};

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: historical_hidden_hexes <state file>");
        std::process::exit(2);
    };
    let board = BoardInfo::from_map_data(&campaign_map_data());
    let mut state = GameState::with_board(Scenario::Historical, board);
    let text = std::fs::read_to_string(&path).expect("readable state file");
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.first() != Some(&"U") || f.len() < 6 {
            continue;
        }
        let Some(&id) = UnitId::ALL.iter().find(|u| format!("{u:?}") == f[5]) else {
            continue;
        };
        let (Ok(q), Ok(r)) = (f[2].parse(), f[3].parse()) else {
            continue;
        };
        state.units.push(UnitPlacement {
            id,
            position: HexCoord::new(q, r),
            profile: profile_for_unit(id).expect("a real counter"),
            state: Default::default(),
        });
    }
    for leader in &state.units {
        let UnitIdentity::DervishLeader(name) = leader.profile.identity else {
            continue;
        };
        let hexes: Vec<String> = state
            .board
            .terrain
            .keys()
            .filter(|h| {
                h.distance(leader.position) <= 3
                    && state.in_deployment_zone(Player::Dervish, **h, false)
            })
            .map(|h| format!("{},{}", h.q, h.r))
            .collect();
        println!("HIDDEN {name:?} {}", hexes.join(" "));
    }
}

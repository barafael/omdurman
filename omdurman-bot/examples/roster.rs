//! Print every counter of a scenario's order of battle with its factors.
//!
//! ```sh
//! cargo run --release -p omdurman-bot --example roster -- historical
//! ```

use omdurman_bot::describe::unit_name;
use omdurman_rules::unit_profiles::profile_for_unit;
use omdurman_types::Scenario;

fn main() {
    let scenario = match std::env::args().nth(1).as_deref() {
        Some("fok") => Scenario::FallOfKhartoum,
        Some("historical") => Scenario::Historical,
        _ => Scenario::Campaign,
    };
    for (player, id) in omdurman_bot::oob::deployable_oob(scenario) {
        let Some(p) = profile_for_unit(id) else {
            continue;
        };
        println!(
            "{player:?} {:<28} {:?} fire={:?} melee={:?} move={:?}",
            unit_name(id),
            p.identity,
            p.fire.map(|f| f.value()),
            p.melee.map(|m| m.value()),
            p.movement
        );
    }
}

//! One traced arena game (`#[ignore]`d diagnostics):
//! `ARENA_TRACE=1 ARENA_GAME=campaign:1000:current:baseline cargo test
//! --release -p omdurman-bot --test arena_trace -- --ignored --nocapture`.

use omdurman_bot::arena::{Version, play};
use omdurman_types::Scenario;

#[test]
#[ignore = "diagnostics"]
fn traced_game() {
    let spec =
        std::env::var("ARENA_GAME").unwrap_or_else(|_| "campaign:1000:current:baseline".into());
    let parts: Vec<&str> = spec.split(':').collect();
    let scenario = match parts[0] {
        "fok" => Scenario::FallOfKhartoum,
        "historical" => Scenario::Historical,
        _ => Scenario::Campaign,
    };
    let version = |s: &str| {
        if s == "baseline" {
            Version::Baseline
        } else {
            Version::Current
        }
    };
    let r = play(
        scenario,
        parts[1].parse().unwrap(),
        version(parts[2]),
        version(parts[3]),
    );
    println!("{}", r.line());
}

//! Commander strength measurement (`#[ignore]`d: minutes of CPU). Plays the
//! live commanders against the frozen baseline on the same seeds, both
//! sides, every scenario, and prints a table:
//!
//! ```sh
//! cargo test --release -p omdurman-bot --test arena -- --ignored --nocapture
//! ARENA_SEEDS=40 ARENA_SCENARIOS=fok,historical cargo test --release ...
//! ```
//!
//! `ae_score` is positive when the Anglo-Egyptians are ahead (Campaign: VP
//! superiority; Historical: net level x10 + tie-break; FoK: level -3..3).

use std::sync::mpsc;

use omdurman_bot::arena::{ArenaResult, Version, play};
use omdurman_types::{Player, Scenario};

fn scenarios() -> Vec<Scenario> {
    let raw = std::env::var("ARENA_SCENARIOS").unwrap_or_else(|_| "fok,historical,campaign".into());
    raw.split(',')
        .filter_map(|s| match s.trim() {
            "fok" => Some(Scenario::FallOfKhartoum),
            "historical" => Some(Scenario::Historical),
            "campaign" => Some(Scenario::Campaign),
            _ => None,
        })
        .collect()
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Play every `(seed, ae, dervish)` game of `pairings` on worker threads.
fn run(scenario: Scenario, seeds: &[u64], pairings: &[(Version, Version)]) -> Vec<ArenaResult> {
    let jobs: Vec<(u64, Version, Version)> = seeds
        .iter()
        .flat_map(|&s| pairings.iter().map(move |&(a, d)| (s, a, d)))
        .collect();
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(env_u64("ARENA_THREADS", 8) as usize);
    let jobs = std::sync::Arc::new(std::sync::Mutex::new(jobs));
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();
    for _ in 0..threads {
        let jobs = jobs.clone();
        let tx = tx.clone();
        handles.push(std::thread::spawn(move || {
            loop {
                let job = jobs.lock().unwrap().pop();
                let Some((seed, a, d)) = job else { break };
                let r = play(scenario, seed, a, d);
                if std::env::var("ARENA_VERBOSE").is_ok() {
                    eprintln!("{}", r.line());
                }
                tx.send(r).unwrap();
            }
        }));
    }
    drop(tx);
    let out: Vec<ArenaResult> = rx.iter().collect();
    for h in handles {
        h.join().unwrap();
    }
    out
}

struct Summary {
    games: usize,
    ae_wins: usize,
    d_wins: usize,
    draws: usize,
    mean_score: f64,
    mean_ae_lost: f64,
    mean_d_lost: f64,
    mean_turns: f64,
    unfinished: usize,
    tombs: usize,
}

fn summarize(rs: &[&ArenaResult]) -> Summary {
    let n = rs.len().max(1) as f64;
    Summary {
        games: rs.len(),
        ae_wins: rs
            .iter()
            .filter(|r| r.winner() == Some(Player::AngloEgyptian))
            .count(),
        d_wins: rs
            .iter()
            .filter(|r| r.winner() == Some(Player::Dervish))
            .count(),
        draws: rs
            .iter()
            .filter(|r| r.result.is_some() && r.winner().is_none())
            .count(),
        mean_score: rs.iter().map(|r| r.ae_score() as f64).sum::<f64>() / n,
        mean_ae_lost: rs.iter().map(|r| r.ae_lost as f64).sum::<f64>() / n,
        mean_d_lost: rs.iter().map(|r| r.dervish_lost as f64).sum::<f64>() / n,
        mean_turns: rs.iter().map(|r| r.turns as f64).sum::<f64>() / n,
        unfinished: rs.iter().filter(|r| !r.game_over).count(),
        tombs: rs.iter().filter(|r| r.tomb_taken).count(),
    }
}

#[test]
#[ignore = "minutes of CPU: the commander strength measurement"]
fn commanders_vs_baseline() {
    use Version::*;
    let n = env_u64("ARENA_SEEDS", 12);
    let first = env_u64("ARENA_FIRST_SEED", 1000);
    let seeds: Vec<u64> = (first..first + n).collect();
    let pairings = [
        (Baseline, Baseline),
        (Current, Baseline),
        (Baseline, Current),
        (Current, Current),
    ];
    for scenario in scenarios() {
        let started = std::time::Instant::now();
        let results = run(scenario, &seeds, &pairings);
        println!(
            "\n== {scenario:?}: {} seeds from {first}, {:.0}s",
            n,
            started.elapsed().as_secs_f64()
        );
        println!(
            "{:<9} {:<9} {:>5} {:>6} {:>6} {:>5} {:>8} {:>8} {:>8} {:>6} {:>5} {:>5}",
            "AE",
            "Dervish",
            "games",
            "AEwin",
            "Dwin",
            "draw",
            "AEscore",
            "AE lost",
            "D lost",
            "turns",
            "unfin",
            "tomb"
        );
        for &(a, d) in &pairings {
            let rs: Vec<&ArenaResult> = results
                .iter()
                .filter(|r| r.ae == a && r.dervish == d)
                .collect();
            let s = summarize(&rs);
            println!(
                "{:<9} {:<9} {:>5} {:>6} {:>6} {:>5} {:>+8.2} {:>8.1} {:>8.1} {:>6.1} {:>5} {:>5}",
                a.name(),
                d.name(),
                s.games,
                s.ae_wins,
                s.d_wins,
                s.draws,
                s.mean_score,
                s.mean_ae_lost,
                s.mean_d_lost,
                s.mean_turns,
                s.unfinished,
                s.tombs
            );
        }
    }
}

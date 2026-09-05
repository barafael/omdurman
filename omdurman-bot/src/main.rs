use std::env;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};

use omdurman_bot::agent::{AgentStrategy, Agents};
use omdurman_bot::audit::audit_log;
use omdurman_bot::doctrine::doctrine_brief;
use omdurman_bot::observer::{ReqwestCompletion, review};
use omdurman_bot::playthrough::{PlayConfig, playthrough};
use omdurman_net::llm::LlmConfig;
use omdurman_rules::tactics::{ScriptStep, all_scripts, run_step};
use omdurman_types::{Player, Scenario};
use serde::{Deserialize, Serialize};

const USAGE: &str = "\
omdurman-bot-cli — headless rule-verification playthroughs + offline rules audit

USAGE:
  omdurman-bot-cli play         [scenario] [seed] [strategy] [max_turns] [log_file]
  omdurman-bot-cli review       [log_file] [findings_prefix]
  omdurman-bot-cli audit        [log_file]
  omdurman-bot-cli audit-record [events.jsonl]
  omdurman-bot-cli run          [run.json]
  omdurman-bot-cli tactics

EXAMPLES:
  omdurman-bot-cli play Campaign 123 random 30
  omdurman-bot-cli play FallOfKhartoum               # random, seeded from system RNG
  omdurman-bot-cli review game.log findings
  omdurman-bot-cli audit game.log
  omdurman-bot-cli audit-record games/game_bot_<ts>/events.jsonl
  omdurman-bot-cli run run.json
  omdurman-bot-cli tactics
";

#[derive(Serialize, Deserialize)]
struct RunSpec {
    scenario: String,
    seed: Option<u64>,
    ae_strategy: String,
    dervish_strategy: String,
    max_turns: Option<u32>,
    output_log: Option<String>,
    output_findings: Option<String>,
    review: Option<bool>,
}

/// reqwest's async transport needs a Tokio reactor (hyper-util DNS panics
/// without one), so drive the bot's futures on a current-thread runtime
/// instead of `futures::executor::block_on`.
fn block_on<F: Future>(fut: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime")
        .block_on(fut)
}

fn resolve_scenario(name: &str) -> Scenario {
    match name.trim().to_lowercase().as_str() {
        "fok" | "fallofkhartoum" | "fall_of_khartoum" => Scenario::FallOfKhartoum,
        "historical" => Scenario::Historical,
        _ => Scenario::Campaign,
    }
}

fn strategy_from(name: &str, brief: &str) -> AgentStrategy {
    match name.trim().to_lowercase().as_str() {
        "random" | "rand" | "" => AgentStrategy::Random,
        "aggressive" | "agg" => AgentStrategy::Aggressive,
        "kitchener" => AgentStrategy::Commander(omdurman_bot::commanders::Commander::Kitchener),
        "khalifa" => AgentStrategy::Commander(omdurman_bot::commanders::Commander::Khalifa),
        "llm" | "llm-advised" | "llm_advised" => AgentStrategy::LlmAdvised {
            config: LlmConfig::default(),
            brief: brief.to_string(),
        },
        other => {
            eprintln!("warning: unknown strategy {other:?}, falling back to random");
            AgentStrategy::Random
        }
    }
}

fn cmd_play(args: &[String]) {
    let scenario = args
        .first()
        .map(|s| resolve_scenario(s))
        .unwrap_or(Scenario::Campaign);
    let seed = args
        .get(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(rand::random);
    let mut cfg = PlayConfig::default();
    if let Some(max_turns) = args.get(3).and_then(|s| s.parse::<u32>().ok()) {
        cfg.max_turns = max_turns as u8;
    }
    let strategy_name = args.get(2).map(|s| s.as_str()).unwrap_or("random");
    let agents = match strategy_name {
        "llm" => Agents {
            ae: strategy_from(
                strategy_name,
                &doctrine_brief(Player::AngloEgyptian, scenario),
            ),
            dervish: strategy_from(strategy_name, &doctrine_brief(Player::Dervish, scenario)),
        },
        "ae" => Agents {
            ae: strategy_from("llm", &doctrine_brief(Player::AngloEgyptian, scenario)),
            dervish: AgentStrategy::Random,
        },
        "dervish" => Agents {
            ae: AgentStrategy::Random,
            dervish: strategy_from("llm", &doctrine_brief(Player::Dervish, scenario)),
        },
        // The historical swarm: aggressive Dervish march on Khartoum /
        // GORDON (§9.346) against a random Anglo-Egyptian defence.
        "dervish-agg" | "agg-dervish" => Agents {
            ae: AgentStrategy::Random,
            dervish: AgentStrategy::Aggressive,
        },
        // LLM-directed storm: the Dervish advisor gets the storm-the-Palace
        // brief (see doctrine::storm_brief) against a random garrison.
        "storm" | "dervish-storm" => Agents {
            ae: AgentStrategy::Random,
            dervish: AgentStrategy::LlmAdvised {
                config: LlmConfig::default(),
                brief: omdurman_bot::doctrine::storm_brief(scenario),
            },
        },
        // Scripted-drama siege: the garrison defends in depth (gates,
        // western gap, interior ring, bodyguard) while the horde reduces
        // it layer by layer — GORDON falls only in the closing turns.
        "laststand" | "drama" | "final" => {
            // Director pacing: the Dervish may not end a move within two
            // hexes of the Palace before turn 5, so the layered defence
            // plays out before the final assault. (T6+ starves the
            // Dervish of clock and Gordon survives — measured on seeds
            // 777/2026; T5 is the longest defense that still falls.)
            if scenario == Scenario::FallOfKhartoum
                && let Some(palace) = omdurman_bot::playthrough::board_for_scenario(scenario)
                    .hex_of_location(omdurman_types::Location::Palace)
            {
                cfg.keep_out = Some(omdurman_bot::playthrough::KeepOutZone {
                    player: Player::Dervish,
                    center: palace,
                    radius: 2,
                    until_turn: 5,
                });
            }
            Agents {
                ae: AgentStrategy::LlmAdvised {
                    config: LlmConfig::default(),
                    brief: omdurman_bot::doctrine::defender_brief(scenario),
                },
                dervish: AgentStrategy::LlmAdvised {
                    config: LlmConfig::default(),
                    brief: omdurman_bot::doctrine::besieger_brief(scenario),
                },
            }
        }
        "ae-agg" => Agents {
            ae: AgentStrategy::Aggressive,
            dervish: AgentStrategy::Random,
        },
        // The two historical commanders against each other: Kitchener's
        // firepower defence vs the Khalifa's assault — the tuning match-up.
        "commanders" | "kitchener-vs-khalifa" | "kitchener_vs_khalifa" => Agents {
            ae: AgentStrategy::Commander(omdurman_bot::commanders::Commander::Kitchener),
            dervish: AgentStrategy::Commander(omdurman_bot::commanders::Commander::Khalifa),
        },
        // One historical commander against a random opponent (isolation runs).
        "ae-kitchener" => Agents {
            ae: AgentStrategy::Commander(omdurman_bot::commanders::Commander::Kitchener),
            dervish: AgentStrategy::Random,
        },
        "dervish-khalifa" => Agents {
            ae: AgentStrategy::Random,
            dervish: AgentStrategy::Commander(omdurman_bot::commanders::Commander::Khalifa),
        },
        // LLM-directed siege: the AE advisor gets fortress orders (maxim-gun
        // strongpoints, defensive depth) while the Dervish advisor gets horde
        // orders (multi-axis assault, wall breach, overwhelming casualties).
        "siege" | "fortress" => Agents {
            ae: AgentStrategy::LlmAdvised {
                config: LlmConfig::default(),
                brief: omdurman_bot::doctrine::fortress_brief(scenario),
            },
            dervish: AgentStrategy::LlmAdvised {
                config: LlmConfig::default(),
                brief: omdurman_bot::doctrine::horde_brief(scenario),
            },
        },
        "aggressive" | "agg" => Agents {
            ae: AgentStrategy::Aggressive,
            dervish: AgentStrategy::Aggressive,
        },
        _ => Agents::random(),
    };

    let result = block_on(playthrough(scenario, seed, cfg, agents));
    let log_file = args
        .get(4)
        .map(String::from)
        .unwrap_or_else(|| "game.log".to_string());
    fs::write(&log_file, result.log.render()).expect("write game log");
    let record_dir = write_replay_record(scenario, seed, &result.events);
    println!(
        "scenario={scenario:?} seed=0x{seed:x} turns={} events={} observations={}",
        result.final_state.current_turn.value(),
        result.log.events_logged(),
        result.observations_total
    );
    println!("log written to {log_file}");
    println!("replay record written to {record_dir}/events.jsonl (reviewable in the app's lobby)");
}

/// Persist a playthrough as an app-reviewable game record: a
/// `games/game_bot_<ts>/` directory holding `events.jsonl` in the exact
/// format `GameRecorder::init` + `flush_game_record` write (seed header,
/// then one JSON `RecordedEvent` per line). The trace's own leading
/// `StartGame` (the playthrough opens every record with one) selects the
/// scenario's board during the app's spectator rebuild; a synthetic one is
/// prepended only if it is missing.
fn write_replay_record(
    scenario: Scenario,
    seed: u64,
    events: &[omdurman_net::GameEvent],
) -> String {
    use omdurman_net::{GameEvent, RecordedEvent};

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S-%3fZ");
    let dir = format!("games/game_bot_{ts}");
    fs::create_dir_all(&dir).expect("create game record directory");
    let mut out = format!("{{\"seed\":{seed}}}\n");
    let mut seq: u32 = 0;
    if !matches!(events.first(), Some(GameEvent::StartGame { .. })) {
        let start = RecordedEvent {
            utc: chrono::Utc::now(),
            sender_idx: None,
            seq,
            uid: None,
            payload: GameEvent::StartGame {
                assignments: Vec::new(),
                scenario,
                optional_rule: None,
                ai: Vec::new(),
                commands: Vec::new(),
            },
        };
        out.push_str(&serde_json::to_string(&start).expect("serialize StartGame"));
        out.push('\n');
        seq += 1;
    }
    for event in events {
        let recorded = RecordedEvent {
            utc: chrono::Utc::now(),
            sender_idx: None,
            seq,
            uid: None,
            payload: event.clone(),
        };
        out.push_str(&serde_json::to_string(&recorded).expect("serialize event"));
        out.push('\n');
        seq += 1;
    }
    fs::write(format!("{dir}/events.jsonl"), out).expect("write events.jsonl");
    dir
}

fn cmd_review(args: &[String]) {
    let log_file = args
        .first()
        .map(String::from)
        .unwrap_or_else(|| "game.log".to_string());
    let prefix = args
        .get(1)
        .map(String::from)
        .unwrap_or_else(|| "findings".to_string());
    let log = fs::read_to_string(&log_file).expect("read log file");
    let crib = fs::read_to_string(crib_path()).unwrap_or_default();
    let config = LlmConfig::default();
    let completion = ReqwestCompletion;
    let report = block_on(review(&log, &config, &completion, &crib));
    fs::write(format!("{prefix}.md"), format!("{}\n", report)).expect("write findings.md");
    fs::write(
        format!("{prefix}.json"),
        serde_json::to_string_pretty(&report).expect("serialize findings"),
    )
    .expect("write findings.json");
    println!(
        "audited {} turns / {} events; {} findings ({} critical) -> {prefix}.md",
        report.turns_audited,
        report.events_audited,
        report.findings.len(),
        report
            .findings
            .iter()
            .filter(|f| matches!(f.severity, omdurman_bot::observer::Severity::Critical))
            .count()
    );
}

fn step_note(step: &ScriptStep) -> &'static str {
    match step {
        ScriptStep::Legal { note, .. }
        | ScriptStep::Illegal { note, .. }
        | ScriptStep::Assert { note, .. } => note,
    }
}

fn cmd_tactics() {
    let scripts = all_scripts();
    let mut failures = Vec::new();
    for script in &scripts {
        let mut state = script.state.clone();
        let mut failed = None;
        for (i, step) in script.steps.iter().enumerate() {
            if let Some(msg) = run_step(&mut state, step) {
                failed = Some(format!("step {} ({}) -- {}", i + 1, step_note(step), msg));
                break;
            }
        }
        match failed {
            None => println!("PASS  {:24} [{}]", script.name, script.citation),
            Some(msg) => {
                println!("FAIL  {:24} [{}] {}", script.name, script.citation, msg);
                failures.push((script.name, msg));
            }
        }
    }
    if failures.is_empty() {
        println!("all {} tactics scripts passed", scripts.len());
    } else {
        eprintln!("{} tactics script(s) failed:", failures.len());
        for (name, msg) in &failures {
            eprintln!("  [ ] {name}: {msg}");
        }
        std::process::exit(1);
    }
}

fn cmd_run(args: &[String]) {
    let spec_path = args
        .first()
        .map(String::from)
        .unwrap_or_else(|| "run.json".to_string());
    let raw = fs::read_to_string(&spec_path).expect("read run.json");
    let spec: RunSpec = serde_json::from_str(&raw).expect("parse run.json");

    let scenario = resolve_scenario(&spec.scenario);
    let seed = spec.seed.unwrap_or_else(rand::random);
    let mut cfg = PlayConfig::default();
    if let Some(mt) = spec.max_turns {
        cfg.max_turns = mt as u8;
    }
    let agents = Agents {
        ae: strategy_from(
            &spec.ae_strategy,
            &doctrine_brief(Player::AngloEgyptian, scenario),
        ),
        dervish: strategy_from(
            &spec.dervish_strategy,
            &doctrine_brief(Player::Dervish, scenario),
        ),
    };

    let result = block_on(playthrough(scenario, seed, cfg, agents));
    let log_path = spec.output_log.unwrap_or_else(|| "game.log".to_string());
    fs::write(&log_path, result.log.render()).expect("write game log");
    let record_dir = write_replay_record(scenario, seed, &result.events);
    println!("run complete: scenario={scenario:?} seed=0x{seed:x} log={log_path}");
    println!("replay record written to {record_dir}/events.jsonl (reviewable in the app's lobby)");

    if spec.review.unwrap_or(false) {
        let log = result.log.render();
        let crib = fs::read_to_string(crib_path()).unwrap_or_default();
        let config = LlmConfig::default();
        let completion = ReqwestCompletion;
        let report = block_on(review(&log, &config, &completion, &crib));
        let prefix = spec
            .output_findings
            .unwrap_or_else(|| "findings".to_string());
        fs::write(format!("{prefix}.md"), format!("{}\n", report)).expect("write findings.md");
        fs::write(
            format!("{prefix}.json"),
            serde_json::to_string_pretty(&report).expect("serialize findings"),
        )
        .expect("write findings.json");
        println!(
            "audited {} events; {} findings -> {prefix}.md",
            report.events_audited,
            report.findings.len()
        );
    }
}

fn cmd_audit(args: &[String]) {
    let log_file = args
        .first()
        .map(String::from)
        .unwrap_or_else(|| "game.log".to_string());
    let log = fs::read_to_string(&log_file).expect("read log file");
    let report = audit_log(&log);
    println!("{report}");
    if report.has_errors() {
        std::process::exit(1);
    }
}

fn crib_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("rules_crib_sheet.md")
}

fn main() {
    dotenvy::dotenv().ok();
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        print!("{USAGE}");
        return;
    };
    match cmd.as_str() {
        "play" => cmd_play(&args[1..]),
        "review" => cmd_review(&args[1..]),
        "audit" => cmd_audit(&args[1..]),
        "audit-record" => cmd_audit_record(&args[1..]),
        "run" => cmd_run(&args[1..]),
        "tactics" => cmd_tactics(),
        "help" | "-h" | "--help" => print!("{USAGE}"),
        other => {
            eprintln!("unknown command {other:?}");
            print!("{USAGE}");
            std::process::exit(2);
        }
    }
}

/// Replay a recorded `events.jsonl` through the rules engine and audit every
/// transition for stacking violations (§5.51-5.53) and wall trespass
/// (§5.23: a unit may only cross a wall hexside through a gate or breach).
/// Exit code 1 when any violation is found, so CI / scripts can gate on it.
///
/// This is the record-level counterpart of the `debug_assert!` post-condition
/// in `apply_effect`: the engine guarantees legal transitions at apply time,
/// this tool re-proves it on the persisted artifact.
fn cmd_audit_record(args: &[String]) {
    use omdurman_net::{GameEvent, RecordedEvent};
    use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
    use omdurman_types::HexsideKind;

    let Some(path) = args.first() else {
        eprintln!("usage: omdurman-bot-cli audit-record <path/to/events.jsonl>");
        std::process::exit(2);
    };
    let text = fs::read_to_string(path).expect("read events.jsonl");

    let mut state: Option<GameState> = None;
    let mut violations = 0usize;
    let mut effects = 0usize;

    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("{\"seed\"") {
            continue;
        }
        let rec: RecordedEvent = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} is not a RecordedEvent: {e}", i + 1));
        match rec.payload {
            GameEvent::StartGame { scenario, .. } => {
                state = Some(GameState::with_board(
                    scenario,
                    omdurman_bot::playthrough::board_for_scenario(scenario),
                ));
            }
            GameEvent::Effect(effect) => {
                let Some(st) = state.as_mut() else { continue };
                effects += 1;

                // Position deltas for the wall-trespass audit.
                let moved: Vec<(omdurman_rules::UnitId, omdurman_types::HexCoord)> = match &effect {
                    GameEffect::MoveUnit { unit_id, .. }
                    | GameEffect::RetreatBeforeMelee { unit_id, .. }
                    | GameEffect::AdvanceAfterCombat { unit_id, .. } => st
                        .find_unit(*unit_id)
                        .map(|u| vec![(*unit_id, u.position)])
                        .unwrap_or_default(),
                    _ => Vec::new(),
                };

                if let Err(e) = apply_effect(st, &effect) {
                    println!(
                        "VIOLATION line {} seq {}: effect rejected on replay (nondeterminism?): {e:?}",
                        i + 1,
                        rec.seq
                    );
                    violations += 1;
                    continue;
                }

                if let Err(v) = st.validate_stacking_invariants() {
                    println!("VIOLATION line {} seq {}: STACKING {v}", i + 1, rec.seq);
                    violations += 1;
                }

                for (id, from) in moved {
                    let Some(to) = st.find_unit(id).map(|u| u.position) else {
                        continue;
                    };
                    if from == to {
                        continue;
                    }
                    let dist = from.distance(to);
                    if dist == 1 {
                        if st.board.hexside_between(from, to) == Some(HexsideKind::Wall) {
                            println!(
                                "VIOLATION line {} seq {}: {id:?} crossed WALL hexside {from:?}->{to:?} [§5.23]",
                                i + 1,
                                rec.seq
                            );
                            violations += 1;
                        }
                    } else if dist == 2 {
                        // A two-hex displacement passes through some common
                        // neighbour; it is only legal if at least one such
                        // intermediate has both legs non-wall.
                        let legal_path = from
                            .neighbors()
                            .iter()
                            .filter(|mid| mid.neighbors().contains(&to))
                            .any(|mid| {
                                st.board.hexside_between(from, *mid) != Some(HexsideKind::Wall)
                                    && st.board.hexside_between(*mid, to) != Some(HexsideKind::Wall)
                            });
                        if !legal_path {
                            println!(
                                "VIOLATION line {} seq {}: {id:?} two-hex move {from:?}->{to:?} has no wall-free path [§5.23]",
                                i + 1,
                                rec.seq
                            );
                            violations += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    println!(
        "audited {path}: {} effects, {} violation(s)",
        effects, violations
    );
    if violations > 0 {
        std::process::exit(1);
    }
}

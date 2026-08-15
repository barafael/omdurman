use std::env;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};

use omdurman_bot::agent::{AgentStrategy, Agents};
use omdurman_bot::doctrine::doctrine_brief;
use omdurman_bot::observer::{review, ReqwestCompletion};
use omdurman_bot::playthrough::{playthrough, PlayConfig};
use omdurman_net::llm::LlmConfig;
use omdurman_rules::tactics::{all_scripts, run_step, ScriptStep};
use omdurman_types::{Player, Scenario};
use serde::{Deserialize, Serialize};

const USAGE: &str = "\
omdurman-bot-cli — headless rule-verification playthroughs + offline rules audit

USAGE:
  omdurman-bot-cli play   [scenario] [seed] [strategy] [max_turns] [log_file]
  omdurman-bot-cli review [log_file] [findings_prefix]
  omdurman-bot-cli run    [run.json]
  omdurman-bot-cli tactics

EXAMPLES:
  omdurman-bot-cli play Campaign 123 random 30
  omdurman-bot-cli play FallOfKhartoum               # random, seeded from system RNG
  omdurman-bot-cli review game.log findings
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
    let scenario = args.first().map(|s| resolve_scenario(s)).unwrap_or(Scenario::Campaign);
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
            ae: strategy_from(strategy_name, &doctrine_brief(Player::AngloEgyptian, scenario)),
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
        _ => Agents::random(),
    };

    let result = block_on(playthrough(scenario, seed, cfg, agents));
    let log_file = args.get(4).map(String::from).unwrap_or_else(|| "game.log".to_string());
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
fn write_replay_record(scenario: Scenario, seed: u64, events: &[omdurman_net::GameEvent]) -> String {
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
            payload: GameEvent::StartGame {
                assignments: Vec::new(),
                scenario,
                optional_rule: None,
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
    let log_file = args.first().map(String::from).unwrap_or_else(|| "game.log".to_string());
    let prefix = args.get(1).map(String::from).unwrap_or_else(|| "findings".to_string());
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
    let spec_path = args.first().map(String::from).unwrap_or_else(|| "run.json".to_string());
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
        let prefix = spec.output_findings.unwrap_or_else(|| "findings".to_string());
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

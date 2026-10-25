//! `cargo testbox`, a Cargo alias of this xtask (.cargo/config.toml): the multiplayer test box
//! (testbox/README.md). After the testbox of barafael/beep.

#[cfg(unix)]
mod checks;
#[cfg(unix)]
mod harness;
#[cfg(unix)]
mod phases;
mod pins;
#[cfg(unix)]
mod pod;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

/// What a step comes to: a failure says what failed, for a person to read.
type Result<T, E = String> = std::result::Result<T, E>;

/// Plays omdurman between several peers in one Podman pod: the game from the working tree, each
/// peer on a display of its own, meeting on a matchbox signalling server of the box's own.
///
/// The pod's network is internal: nothing in it reaches the internet, the deployed signalling
/// server or a STUN server. The peers connect to each other over WebRTC inside the pod.
#[derive(Parser)]
#[command(bin_name = "cargo testbox")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Builds the images and plays the pod.
    Up {
        /// How many peers (at most 5).
        #[arg(long, default_value_t = 2)]
        peers: usize,
    },
    /// Runs a phase's checks against the pod, which `up` brought up fresh with the phase's
    /// peers.
    Test {
        phase: Phase,
        /// Also takes each peer's window after each check, into the phase's record
        /// (target/testbox-record/<phase>/) beside the containers' logs.
        #[arg(long)]
        screenshots: bool,
    },
    /// Runs phases each on a fresh box with the peers it takes (`down --wipe`, `up`, `test`), in
    /// rounds, and sums up; each run's record is in target/testbox-record/run/.
    Run {
        /// The phases, in order; all of them if none.
        phases: Vec<Phase>,
        /// How many times each.
        #[arg(long, default_value_t = 1)]
        rounds: u32,
        /// As for `test`.
        #[arg(long)]
        screenshots: bool,
    },
    /// Shows a peer's display in TigerVNC's viewer (view only).
    Vnc { peer: String },
    /// Stops and removes the pod.
    Down {
        /// Also removes the volumes.
        #[arg(long)]
        wipe: bool,
    },
    /// Prints containers' logs.
    Logs {
        /// The containers' names: `matchbox`, `peer-a`, `peer-b`, ...
        services: Vec<String>,
    },
}

/// `test`'s phases.
#[derive(Clone, Copy, clap::ValueEnum)]
enum Phase {
    Core,
    Rejoin,
    Failover,
    Signalling,
    /// Two players a side, each commanding part of it.
    Shared,
    SharedRejoin,
    SharedFailover,
    /// Two players a side, each playing the whole of it.
    Team,
    TeamMixed,
}

impl Phase {
    const ALL: [Phase; 9] = [
        Phase::Core,
        Phase::Rejoin,
        Phase::Failover,
        Phase::Signalling,
        Phase::Shared,
        Phase::SharedRejoin,
        Phase::SharedFailover,
        Phase::Team,
        Phase::TeamMixed,
    ];

    fn name(self) -> String {
        clap::ValueEnum::to_possible_value(&self)
            .map(|value| value.get_name().to_owned())
            .unwrap_or_default()
    }

    /// The peers the phase plays with, which `up` brings.
    fn peers(self) -> usize {
        match self {
            Phase::Core | Phase::Rejoin | Phase::Signalling => 2,
            Phase::Failover => 3,
            Phase::Shared | Phase::SharedRejoin | Phase::Team => 4,
            Phase::SharedFailover | Phase::TeamMixed => 5,
        }
    }
}

fn main() -> ExitCode {
    match run_command(Cli::parse().command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason) => {
            eprintln!("testbox: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root")
        .to_owned()
}

#[cfg(not(unix))]
fn run_command(_command: Command) -> Result<()> {
    Err("the box needs Podman and Unix sockets: Linux only".into())
}

#[cfg(unix)]
fn run_command(command: Command) -> Result<()> {
    let repo = repo_root();
    let pod = pod::Pod::new(&repo);
    match command {
        Command::Up { peers } => pod.up(peers),
        Command::Test { phase, screenshots } => {
            test(&pod, &repo, phase, &phase.name(), screenshots)
        }
        Command::Run {
            phases,
            rounds,
            screenshots,
        } => run(&pod, &repo, &phases, rounds, screenshots),
        Command::Vnc { peer } => {
            let mut viewer = std::process::Command::new("vncviewer");
            viewer
                .arg("ViewOnly=1")
                .arg(pod.view_dir(&peer).join("vnc"));
            pod::run(viewer)
        }
        Command::Down { wipe } => pod.down(wipe),
        Command::Logs { services } => pod.logs(&services),
    }
}

/// Where `run` keeps its records, one directory per round and phase.
#[cfg(unix)]
const RUNS: &str = "target/testbox-record/run";

/// Each of `phases`, or all, `rounds` times, each on a fresh box; then how each went. Fails if
/// any did.
#[cfg(unix)]
fn run(
    pod: &pod::Pod,
    repo: &Path,
    phases: &[Phase],
    rounds: u32,
    screenshots: bool,
) -> Result<()> {
    let phases = if phases.is_empty() {
        &Phase::ALL[..]
    } else {
        phases
    };
    let runs = repo.join(RUNS);
    if runs.exists() {
        std::fs::remove_dir_all(&runs).map_err(|error| format!("{}: {error}", runs.display()))?;
    }
    let mut outcomes = Vec::new();
    for round in 1..=rounds {
        for &phase in phases {
            let name = phase.name();
            println!("== round {round}: {name}");
            let started = std::time::Instant::now();
            let outcome = pod
                .down(true)
                .and_then(|()| pod.up(phase.peers()))
                .and_then(|()| {
                    test(
                        pod,
                        repo,
                        phase,
                        &format!("run/{round}-{name}"),
                        screenshots,
                    )
                });
            let took = started.elapsed().as_secs();
            if let Err(reason) = &outcome {
                println!("== round {round}: {name}: {reason}");
            }
            outcomes.push((round, name, outcome.is_ok(), took));
        }
    }
    println!("== summary (records in {})", runs.display());
    for (round, name, passed, took) in &outcomes {
        let outcome = if *passed { "passed" } else { "FAILED" };
        println!("round {round} {name}: {outcome} ({took} s)");
    }
    let failed = outcomes.iter().filter(|(_, _, passed, _)| !passed).count();
    if failed == 0 {
        Ok(())
    } else {
        Err(format!("{failed} of {} failed", outcomes.len()))
    }
}

/// A phase's checks, recorded in target/testbox-record/`record`.
#[cfg(unix)]
fn test(pod: &pod::Pod, repo: &Path, phase: Phase, record: &str, screenshots: bool) -> Result<()> {
    let record = checks::Record::new(repo, record, screenshots)?;
    checks::phase(pod, record, |checks| match phase {
        Phase::Core => phases::core(checks, pod),
        Phase::Rejoin => phases::rejoin(checks, pod),
        Phase::Failover => phases::failover(checks, pod),
        Phase::Signalling => phases::signalling(checks, pod),
        Phase::Shared => phases::shared(checks, pod),
        Phase::SharedRejoin => phases::shared_rejoin(checks, pod),
        Phase::SharedFailover => phases::shared_failover(checks, pod),
        Phase::Team => phases::team(checks, pod),
        Phase::TeamMixed => phases::team_mixed(checks, pod),
    })
}

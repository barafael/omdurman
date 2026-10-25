//! The tally of a phase's checks, and the record it leaves behind for looking into it once the
//! box is wiped: `target/testbox-record/<phase>/`, with every container's log and each peer's
//! last snapshot at the phase's end, and with `--screenshots` each peer's window after each check.

use std::path::{Path, PathBuf};

use crate::Result;
use crate::harness::Harness;
use crate::pod::Pod;

/// Where the phases' records go, one directory each.
pub const RECORD: &str = "target/testbox-record";
/// How long a check's name may be in a screenshot's file name.
const LABEL: usize = 60;

/// Runs a phase's checks on a fresh tally, then fails if any of them failed. Last, also after a
/// check the rest depend on failed: that no peer panicked; then the logs go into the record.
pub fn phase(pod: &Pod, record: Record, run: impl FnOnce(&mut Checks) -> Result<()>) -> Result<()> {
    let mut checks = Checks {
        failed: Vec::new(),
        record,
        peers: Vec::new(),
    };
    let ran = run(&mut checks);
    checks.record("no peer panicked", || no_panics(pod));
    checks.keep(pod);
    ran?;
    checks.finish()
}

pub struct Checks {
    failed: Vec<String>,
    record: Record,
    /// The peers' harnesses, for the screenshots and the record's snapshots; the phase's code
    /// borrows them (`peers_mut`).
    peers: Vec<Harness>,
}

impl Checks {
    /// A check later checks depend on: its failure ends the run (the caller's `?`).
    pub fn require<Found>(
        &mut self,
        name: &str,
        check: impl FnOnce(&mut [Harness]) -> Result<Found>,
    ) -> Result<Found> {
        let outcome = check(&mut self.peers);
        self.report(name, outcome)
            .ok_or_else(|| format!("{name} failed"))
    }

    /// A check nothing depends on: a failure is tallied for [`Self::finish`] and the run goes on.
    pub fn record(&mut self, name: &str, check: impl FnOnce() -> Result<()>) {
        let outcome = check();
        self.report(name, outcome);
    }

    /// Takes on the peers' harnesses for the checks that follow.
    pub fn connect(&mut self, peers: Vec<Harness>) {
        self.peers = peers;
    }

    /// Prints the outcome; a failure is tallied and yields `None`.
    fn report<Found>(&mut self, name: &str, outcome: Result<Found>) -> Option<Found> {
        self.record.shoot(name, &mut self.peers);
        match outcome {
            Ok(found) => {
                println!("check {name}: passed");
                Some(found)
            }
            Err(reason) => {
                println!("check {name}: failed: {reason}");
                self.failed.push(name.to_owned());
                None
            }
        }
    }

    /// Every container's log and each peer's last snapshot into the record; says where it is.
    fn keep(&mut self, pod: &Pod) {
        for container in pod.containers() {
            let log = pod
                .log_named(&container, None)
                .unwrap_or_else(|error| error.to_string());
            self.record.write(&format!("{container}.log"), &log);
        }
        for peer in &mut self.peers {
            let snapshot = peer
                .snapshot()
                .map_err(|error| error.to_string())
                .and_then(|snapshot| {
                    serde_json::to_string_pretty(&snapshot).map_err(|error| error.to_string())
                })
                .unwrap_or_else(|error| error);
            self.record
                .write(&format!("{}-snapshot.json", peer.peer), &snapshot);
        }
        println!("record: {}", self.record.directory.display());
    }

    fn finish(self) -> Result<()> {
        if self.failed.is_empty() {
            Ok(())
        } else {
            Err(format!("failed: {}", self.failed.join(", ")))
        }
    }
}

/// No peer's log says a thread panicked.
fn no_panics(pod: &Pod) -> Result<()> {
    let panicked: Vec<&str> = pod
        .peers()
        .into_iter()
        .filter(|peer| {
            pod.log(peer, None)
                .is_ok_and(|log| log.contains("panicked at"))
        })
        .collect();
    if panicked.is_empty() {
        Ok(())
    } else {
        Err(format!("panicked: {}", panicked.join(", ")))
    }
}

/// A phase's record.
pub struct Record {
    directory: PathBuf,
    /// With `--screenshots`, how many rounds were taken, which numbers them in the order of the
    /// checks.
    shots: Option<usize>,
}

impl Record {
    /// The record in target/testbox-record/`name`, emptied of a former run's.
    pub fn new(repo: &Path, name: &str, screenshots: bool) -> Result<Self> {
        let directory = repo.join(RECORD).join(name);
        if directory.exists() {
            std::fs::remove_dir_all(&directory)
                .map_err(|error| format!("{}: {error}", directory.display()))?;
        }
        std::fs::create_dir_all(&directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        Ok(Self {
            directory,
            shots: screenshots.then_some(0),
        })
    }

    /// With `--screenshots`, each peer's window as it is after the check `name`. A screenshot
    /// that fails fails no check: it is said so, and the run goes on.
    fn shoot(&mut self, name: &str, peers: &mut [Harness]) {
        let Some(taken) = &mut self.shots else {
            return;
        };
        *taken += 1;
        let label = format!("{:02}-{}", taken, label(name));
        for peer in peers {
            let to = self.directory.join(format!("{label}-{}.png", peer.peer));
            let shot = peer
                .screenshot()
                .and_then(|shot| std::fs::rename(&shot, &to).map_err(|error| error.to_string()));
            if let Err(error) = shot {
                println!("screenshot {label}: {error}");
            }
        }
    }

    fn write(&self, name: &str, contents: &str) {
        let path = self.directory.join(name);
        if let Err(error) = std::fs::write(&path, contents) {
            println!("record: {}: {error}", path.display());
        }
    }
}

/// A check's name as part of a file name: lowercase letters and digits, the rest one dash.
fn label(name: &str) -> String {
    let mut label = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            label.push(c.to_ascii_lowercase());
        } else if !label.is_empty() && !label.ends_with('-') {
            label.push('-');
        }
    }
    let mut label: String = label.trim_end_matches('-').chars().take(LABEL).collect();
    while label.ends_with('-') {
        label.pop();
    }
    label
}

#[cfg(test)]
mod tests {
    use super::label;

    #[test]
    fn a_check_name_becomes_a_file_name() {
        assert_eq!(
            label("a guest that restarts gets its seat back"),
            "a-guest-that-restarts-gets-its-seat-back"
        );
        assert_eq!(
            label("  the host's record, again!  "),
            "the-host-s-record-again"
        );
        assert!(label(&"a long name ".repeat(20)).len() <= 60);
        assert!(!label(&"a long name ".repeat(20)).ends_with('-'));
    }
}

//! The box as one pod under `podman kube play` (testbox/pod.yaml): the signalling server and the
//! peers, on an internal network.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::Result;

/// pod.yaml's `metadata.name`; kube play names each container `<pod>-<container>`.
const POD: &str = "omdurman-testbox";
const NETWORK: &str = "omdurman-testbox";
/// The peers' harness sockets and screenshots, a directory per peer (pod.yaml volumes `run-*`).
pub const RUN: &str = "target/testbox-run";
/// The peers' display sockets, a directory per peer (pod.yaml volumes `view-*`).
pub const VIEW: &str = "target/testbox-view";
/// The lines of pod.yaml the peers' containers and volumes go in.
const PEERS: &str = "# @PEERS@";
const VOLUMES: &str = "# @VOLUMES@";
/// The peers by name, in the order `up --peers` takes them.
pub const PEER_NAMES: [&str; 3] = ["peer-a", "peer-b", "peer-c"];

/// An image pod.yaml runs, built from the repository before it plays.
struct Image {
    tag: &'static str,
    /// Relative to the repository.
    containerfile: &'static str,
    context: &'static str,
}

const IMAGES: [Image; 2] = [
    Image {
        tag: "localhost/omdurman-testbox-matchbox",
        containerfile: "testbox/matchbox/Containerfile",
        context: "testbox/matchbox",
    },
    Image {
        tag: "localhost/omdurman-testbox-peer",
        containerfile: "testbox/peer/Containerfile",
        context: ".",
    },
];

/// What the signalling server logs as it starts serving.
pub const MATCHBOX_READY: &str = "Matchbox Signaling Server";
pub const MATCHBOX: &str = "matchbox";

pub struct Pod {
    repo: PathBuf,
    /// pod.yaml with its paths and peers filled in: hostPath must be absolute.
    rendered: PathBuf,
}

impl Pod {
    pub fn new(repo: &Path) -> Self {
        Self {
            repo: repo.to_owned(),
            rendered: repo.join("target/testbox-pod.yaml"),
        }
    }

    /// Builds the images and plays the pod with the first `peers` of [`PEER_NAMES`].
    pub fn up(&self, peers: usize) -> Result<()> {
        if !(1..=PEER_NAMES.len()).contains(&peers) {
            return Err(format!("1 to {} peers", PEER_NAMES.len()));
        }
        for image in &IMAGES {
            let mut build = podman(&["build", "--jobs", "0", "-t", image.tag]);
            build
                .arg("-f")
                .arg(self.repo.join(image.containerfile))
                .arg(self.repo.join(image.context));
            run(build)?;
        }
        // Not in pod.yaml: Kubernetes YAML has no network objects.
        run(podman(&[
            "network",
            "create",
            "--ignore",
            "--internal",
            NETWORK,
        ]))?;
        // `--ignore` keeps a network of that name that was there: only if it has no egress.
        let internal = output(podman(&[
            "network",
            "inspect",
            "--format",
            "{{.Internal}}",
            NETWORK,
        ]))?;
        if internal.trim() != "true" {
            return Err(format!(
                "the network {NETWORK} has egress: `podman network rm {NETWORK}`"
            ));
        }
        // The sockets' directories: the owner's only.
        for peer in &PEER_NAMES[..peers] {
            for directory in [self.run_dir(peer), self.view_dir(peer)] {
                std::fs::create_dir_all(&directory)
                    .map_err(|error| format!("{}: {error}", directory.display()))?;
                std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))
                    .map_err(|error| format!("{}: {error}", directory.display()))?;
            }
        }
        let pod = self.render(peers)?;
        std::fs::write(&self.rendered, pod)
            .map_err(|error| format!("{}: {error}", self.rendered.display()))?;
        let mut play = podman(&["kube", "play", "--replace", "--network", NETWORK]);
        play.arg(&self.rendered);
        run(play)
    }

    /// pod.yaml with the repository filled in, and a container and volumes per peer.
    fn render(&self, peers: usize) -> Result<String> {
        let read = |path: &str| {
            std::fs::read_to_string(self.repo.join(path))
                .map_err(|error| format!("{path}: {error}"))
        };
        let (peer, volumes) = (read("testbox/peer.yaml")?, read("testbox/volumes.yaml")?);
        let each = |template: &str| -> String {
            PEER_NAMES[..peers]
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    template
                        .replace("@PEER@", name)
                        .replace("@SLOT@", &(index + 1).to_string())
                })
                .collect()
        };
        let (peers, volumes) = (each(&peer), each(&volumes));
        let pod: String = read("testbox/pod.yaml")?
            .lines()
            .map(|line| match line.trim() {
                PEERS => peers.clone(),
                VOLUMES => volumes.clone(),
                _ => format!("{line}\n"),
            })
            .collect();
        Ok(pod.replace("@REPO@", &self.repo.to_string_lossy()))
    }

    /// Where `peer`'s harness socket and screenshots are on the host.
    pub fn run_dir(&self, peer: &str) -> PathBuf {
        self.repo.join(RUN).join(peer)
    }

    /// Where `peer`'s display socket is on the host.
    pub fn view_dir(&self, peer: &str) -> PathBuf {
        self.repo.join(VIEW).join(peer)
    }

    /// The peers the pod was played with, running or not, in [`PEER_NAMES`] order.
    pub fn peers(&self) -> Vec<&'static str> {
        let containers = self.containers();
        PEER_NAMES
            .into_iter()
            .filter(|peer| containers.contains(&container(peer)))
            .collect()
    }

    /// `--force` also removes the volumes. Nothing to do if no pod was ever played from here:
    /// kube down needs the YAML it was played from.
    pub fn down(&self, wipe: bool) -> Result<()> {
        if !self.rendered.exists() {
            return Ok(());
        }
        let mut down = podman(&["kube", "down"]);
        if wipe {
            down.arg("--force");
        }
        down.arg(&self.rendered);
        run(down)
    }

    /// The pod's containers by name, stopped ones too; none if podman can't say.
    pub fn containers(&self) -> Vec<String> {
        output(podman(&[
            "ps",
            "--all",
            "--filter",
            &format!("pod={POD}"),
            "--format",
            "{{.Names}}",
        ]))
        .map(|names| names.lines().map(str::to_owned).collect())
        .unwrap_or_default()
    }

    /// `service`'s log so far, stdout and stderr alike, from `since` (seconds since the epoch).
    pub fn log(&self, service: &str, since: Option<u64>) -> Result<String> {
        self.log_named(&container(service), since)
    }

    /// [`Self::log`] of the container `name`.
    pub fn log_named(&self, name: &str, since: Option<u64>) -> Result<String> {
        let mut logs = podman(&["logs"]);
        if let Some(since) = since {
            logs.args(["--since", &since.to_string()]);
        }
        logs.arg(name);
        let logs = logs
            .output()
            .map_err(|error| format!("podman logs {name}: {error}"))?;
        if !logs.status.success() {
            return Err(format!(
                "podman logs {name}: {}",
                String::from_utf8_lossy(&logs.stderr)
            ));
        }
        let mut log = String::from_utf8_lossy(&logs.stdout).into_owned();
        log.push_str(&String::from_utf8_lossy(&logs.stderr));
        Ok(log)
    }

    pub fn logs(&self, services: &[String]) -> Result<()> {
        for service in services {
            run(podman(&["logs", &container(service)]))?;
        }
        Ok(())
    }

    /// Stops `service` until [`Self::start`]: the pod's restart policy leaves a stopped container
    /// stopped.
    pub fn stop(&self, service: &str) -> Result<()> {
        run(podman(&["stop", &container(service)]))
    }

    pub fn start(&self, service: &str) -> Result<()> {
        run(podman(&["start", &container(service)]))
    }

    pub fn restart(&self, service: &str) -> Result<()> {
        run(podman(&["restart", &container(service)]))
    }

    /// Waits until `service` logs a line containing `needle`, after `since`: what a service says
    /// when it is ready, as kube play has no readiness probes.
    pub fn await_log(
        &self,
        service: &str,
        needle: &str,
        since: Option<u64>,
        patience: Duration,
    ) -> Result<()> {
        let deadline = Instant::now() + patience;
        loop {
            if self.log(service, since)?.contains(needle) {
                return Ok(());
            }
            if Instant::now() > deadline {
                return Err(format!(
                    "{service} didn't log {needle:?} within {}s",
                    patience.as_secs()
                ));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }
}

fn podman(args: &[&str]) -> Command {
    let mut podman = Command::new("podman");
    podman.args(args);
    podman
}

/// What kube play names `service`'s container.
fn container(service: &str) -> String {
    format!("{POD}-{service}")
}

/// The time in seconds since the epoch, as `since` for [`Pod::await_log`].
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

fn describe(command: &Command) -> String {
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Runs `command` with its output on ours; fails on a non-zero exit.
pub fn run(mut command: Command) -> Result<()> {
    let status = command
        .status()
        .map_err(|error| format!("{}: {error}", describe(&command)))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} failed: {status}", describe(&command)))
    }
}

/// Runs `command` and returns its stdout; fails with what it printed on a non-zero exit.
fn output(mut command: Command) -> Result<String> {
    let output = command
        .output()
        .map_err(|error| format!("{}: {error}", describe(&command)))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!(
            "{} failed: {}",
            describe(&command),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

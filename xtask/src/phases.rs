//! The phases' checks. Each needs a fresh box (`cargo testbox down --wipe`, then `up` with the
//! phase's peers): the peers must start in an empty room.
//!
//! The peers play each other through the box's own signalling server and WebRTC between
//! containers of one pod, so a pass speaks for the protocol (election, sequencing, history
//! installs, seat rebinding, healing) over real data channels -- not for the internet's NATs and
//! TURN, nor for the deployed fly.io server.

use std::time::Duration;

use omdurman_harness::{Action, LISTENING, Snapshot};
use omdurman_types::{Player, Scenario};

use crate::Result;
use crate::checks::Checks;
use crate::harness::{Harness, gist};
use crate::pod::{MATCHBOX, MATCHBOX_READY, Pod, now};

/// How long a peer may take to say its harness listens: a cold start compiles every shader on
/// the software renderer.
const START: Duration = Duration::from_secs(10 * 60);
/// How long a step may take.
const PATIENCE: Duration = Duration::from_secs(120);
/// How long a stretch of play may take.
const PLAY_PATIENCE: Duration = Duration::from_secs(10 * 60);
/// Orders (recorded events) in a stretch of play.
const ORDERS: usize = 40;
/// The scenario the phases play: the shortest turns.
const SCENARIO: Scenario = Scenario::FallOfKhartoum;

/// A seat a phase gives a peer: a faction, or none (a spectator).
type Side = Option<Player>;

/// Two players.
pub fn core(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let sides = [Some(Player::AngloEgyptian), Some(Player::Dervish)];
    meet(checks, pod, sides.len())?;
    started(checks, &sides)?;
    played(checks, &sides, "both sides play")?;
    converged(checks)
}

/// Two players; the guest's game restarts mid-battle and takes its seat back.
pub fn rejoin(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let sides = [Some(Player::AngloEgyptian), Some(Player::Dervish)];
    meet(checks, pod, sides.len())?;
    started(checks, &sides)?;
    played(checks, &sides, "both sides play")?;
    checks.require("a guest whose game restarts gets its seat back", |peers| {
        let guest = host_index(peers)? ^ 1;
        let before = peers[guest].snapshot()?.net.record_len;
        let name = peers[guest].peer;
        peers[guest] = restart(pod, name)?;
        peers[guest].wait_for("its seat and the history", PATIENCE, |snapshot| {
            (snapshot.game.started
                && snapshot.game.seated == sides[guest]
                && snapshot.net.record_len >= before
                && !snapshot.game.paused)
                .then_some(())
        })
    })?;
    played(checks, &sides, "play goes on after the rejoin")?;
    converged(checks)
}

/// Two players and a spectator; the host leaves mid-battle, the others elect another (and pause
/// for the old host's seat, if it held one), and the old host comes back.
pub fn failover(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let sides = [Some(Player::AngloEgyptian), Some(Player::Dervish), None];
    meet(checks, pod, sides.len())?;
    started(checks, &sides)?;
    played(checks, &sides, "both sides play")?;
    let host = checks.require("the host leaves; the others elect a new one", |peers| {
        let host = host_index(peers)?;
        pod.stop(peers[host].peer)?;
        let mut rest: Vec<&mut Harness> = peers
            .iter_mut()
            .enumerate()
            .filter_map(|(index, peer)| (index != host).then_some(peer))
            .collect();
        // The game waits for an absent seat holder; a spectator host leaves no seat empty.
        let seated = sides[host].is_some();
        let waiting_for = if seated {
            "a new host, the game paused for the old one's seat"
        } else {
            "a new host, the game going on"
        };
        wait_all(&mut rest, waiting_for, PATIENCE, |snapshots| {
            (agreed_host(snapshots, 2).is_some()
                && snapshots
                    .iter()
                    .all(|snapshot| snapshot.game.paused == seated))
            .then_some(())
        })?;
        Ok(host)
    })?;
    checks.require(
        "the old host comes back, to its seat if it held one",
        |peers| {
            let name = peers[host].peer;
            pod.start(name)?;
            peers[host] = connect_when_listening(pod, name, Some(now().saturating_sub(1)))?;
            let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
            wait_all(
                &mut all,
                "every peer back, the game going on",
                PATIENCE,
                |snapshots| {
                    (agreed_host(snapshots, sides.len()).is_some()
                        && snapshots[host].game.seated == sides[host]
                        && snapshots
                            .iter()
                            .all(|snapshot| snapshot.game.started && !snapshot.game.paused))
                    .then_some(())
                },
            )
        },
    )?;
    played(checks, &sides, "play goes on under the new host")?;
    converged(checks)
}

/// Two players; the signalling server goes down mid-battle and comes back.
pub fn signalling(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let sides = [Some(Player::AngloEgyptian), Some(Player::Dervish)];
    meet(checks, pod, sides.len())?;
    started(checks, &sides)?;
    played(checks, &sides, "both sides play")?;
    checks.require("the signalling server goes down", |_| pod.stop(MATCHBOX))?;
    played(checks, &sides, "play goes on without the signalling server")?;
    checks.require("the signalling server comes back", |peers| {
        let since = now().saturating_sub(1);
        pod.start(MATCHBOX)?;
        pod.await_log(MATCHBOX, MATCHBOX_READY, Some(since), PATIENCE)?;
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        wait_all(
            &mut all,
            "every peer still seeing the others",
            PATIENCE,
            |snapshots| agreed_host(snapshots, sides.len()),
        )
        .map(drop)
    })?;
    played(checks, &sides, "play goes on after the server's return")?;
    converged(checks)
}

/// The signalling server serving, every peer's harness listening, and the peers seeing each
/// other with one host.
fn meet(checks: &mut Checks, pod: &Pod, peers: usize) -> Result<()> {
    checks.require("the signalling server serves", |_| {
        pod.await_log(MATCHBOX, MATCHBOX_READY, None, PATIENCE)
    })?;
    let names = pod.peers();
    if names.len() != peers {
        return Err(format!(
            "the phase takes {peers} peers, the box has {}: `cargo testbox up --peers {peers}`",
            names.len()
        ));
    }
    let harnesses = checks.require("every peer's harness listens", |_| {
        names
            .iter()
            .map(|name| connect_when_listening(pod, name, None))
            .collect::<Result<Vec<_>>>()
    })?;
    checks.connect(harnesses);
    checks.require("the peers see each other and agree on a host", |peers| {
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        wait_all(
            &mut all,
            "every peer in every roster",
            PATIENCE,
            |snapshots| agreed_host(snapshots, snapshots.len()),
        )
        .map(drop)
    })
}

/// The peers pick `sides` in the lobby, and the host starts a battle every peer joins.
fn started(checks: &mut Checks, sides: &[Side]) -> Result<()> {
    checks.require("the host starts a battle every peer joins", |peers| {
        for (peer, side) in peers.iter_mut().zip(sides) {
            match side {
                Some(faction) => peer.act(Action::PickFaction {
                    faction: Some(*faction),
                })?,
                None => peer.act(Action::Spectate { on: true })?,
            }
        }
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        wait_all(
            &mut all,
            "every pick in every roster",
            PATIENCE,
            |snapshots| {
                snapshots
                    .iter()
                    .all(|snapshot| {
                        snapshot.lobby.roster.len() == sides.len()
                            && snapshot.lobby.roster.iter().all(|row| {
                                row.key.is_some()
                                    && snapshots.iter().zip(sides).any(|(owner, side)| {
                                        owner.lobby.key == row.key.clone().unwrap_or_default()
                                            && row.faction == *side
                                            && row.spectating == side.is_none()
                                    })
                            })
                    })
                    .then_some(())
            },
        )?;
        let host = host_index(peers)?;
        peers[host].act(Action::PickScenario { scenario: SCENARIO })?;
        peers[host].act(Action::StartBattle)?;
        let players = sides.iter().flatten().count();
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        wait_all(
            &mut all,
            "the battle on every peer",
            PATIENCE,
            |snapshots| {
                snapshots
                    .iter()
                    .zip(sides)
                    .all(|(snapshot, side)| {
                        snapshot.game.started
                            && snapshot.game.scenario == SCENARIO
                            && snapshot.game.seats.len() == players
                            && snapshot.game.seated == *side
                            && snapshot.screen.as_deref() == Some("Board")
                    })
                    .then_some(())
            },
        )
    })
}

/// Every seated peer autoplays until [`ORDERS`] more events are in every record, both sides have
/// been to act and set-up is over (or the game is over), then stops.
fn played(checks: &mut Checks, sides: &[Side], name: &str) -> Result<()> {
    checks.require(&format!("{name}: {ORDERS} orders"), |peers| {
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        let start = wait_all(&mut all, "the records", PATIENCE, |snapshots| {
            snapshots
                .iter()
                .map(|snapshot| snapshot.net.record_len)
                .max()
        })?;
        for (peer, side) in all.iter_mut().zip(sides) {
            if side.is_some() {
                peer.act(Action::Autoplay { on: true })?;
            }
        }
        // Both sides acting in the stretch, as the peers saw it: set-up alone can be one side's.
        let mut acted = Vec::new();
        let played = wait_all(&mut all, "the orders", PLAY_PATIENCE, |snapshots| {
            for snapshot in snapshots {
                if let Some(side) = snapshot.game.to_act
                    && !acted.contains(&side)
                {
                    acted.push(side);
                }
            }
            snapshots
                .iter()
                .all(|snapshot| {
                    snapshot.game.game_over
                        || (snapshot.net.record_len >= start + ORDERS
                            && acted.len() == 2
                            && snapshot.game.phase != "Setup")
                })
                .then_some(())
        });
        for (peer, side) in all.iter_mut().zip(sides) {
            if side.is_some() {
                peer.act(Action::Autoplay { on: false })?;
            }
        }
        played
    })
}

/// Every peer holds the same record and the same rules state, nothing of its own unconfirmed.
fn converged(checks: &mut Checks) -> Result<()> {
    checks.require("every peer holds the same record and state", |peers| {
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        wait_all(
            &mut all,
            "the same record and state",
            PATIENCE,
            |snapshots| {
                let agree = |of: fn(&Snapshot) -> String| {
                    snapshots
                        .windows(2)
                        .all(|pair| of(&pair[0]) == of(&pair[1]))
                };
                (snapshots
                    .iter()
                    .all(|snapshot| snapshot.net.unconfirmed == 0)
                    && agree(|snapshot| snapshot.net.record_digest.clone())
                    && agree(|snapshot| format!("{:?}", snapshot.net.last_applied_seq))
                    && agree(|snapshot| snapshot.game.units_digest.clone())
                    && agree(|snapshot| format!("{} {}", snapshot.game.turn, snapshot.game.phase)))
                .then_some(())
            },
        )
    })
}

/// The index of the peer that is host, by its own word and every other's.
fn host_index(peers: &mut [Harness]) -> Result<usize> {
    let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
    let count = all.len();
    wait_all(&mut all, "one host", PATIENCE, |snapshots| {
        agreed_host(snapshots, count)
    })
}

/// Which of `snapshots` is the host, when each sees `count` peers, all name the same host and
/// only it says it is one.
fn agreed_host(snapshots: &[Snapshot], count: usize) -> Option<usize> {
    let host = snapshots.first()?.net.host.clone()?;
    let agreed = snapshots.iter().all(|snapshot| {
        snapshot.net.peers.len() == count
            && snapshot.net.host.as_ref() == Some(&host)
            && snapshot.net.is_host == (snapshot.net.me.as_ref() == Some(&host))
    });
    if !agreed {
        return None;
    }
    snapshots.iter().position(|snapshot| snapshot.net.is_host)
}

/// Waits until `accept` returns a value for the peers' snapshots, taken in turn, or fails after
/// `patience` with `waiting_for` and each peer's gist.
fn wait_all<Found>(
    peers: &mut [&mut Harness],
    waiting_for: &str,
    patience: Duration,
    mut accept: impl FnMut(&[Snapshot]) -> Option<Found>,
) -> Result<Found> {
    let deadline = std::time::Instant::now() + patience;
    loop {
        let snapshots = peers
            .iter_mut()
            .map(|peer| peer.snapshot())
            .collect::<Result<Vec<_>>>()?;
        if let Some(found) = accept(&snapshots) {
            return Ok(found);
        }
        if std::time::Instant::now() > deadline {
            let gists: Vec<String> = peers
                .iter()
                .zip(&snapshots)
                .map(|(peer, snapshot)| format!("{}: {}", peer.peer, gist(snapshot)))
                .collect();
            return Err(format!(
                "{}s waiting for {waiting_for}; {}",
                patience.as_secs(),
                gists.join(" | ")
            ));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// Waits for `peer` to say its harness listens (after `since`), and connects to it.
fn connect_when_listening(pod: &Pod, peer: &'static str, since: Option<u64>) -> Result<Harness> {
    pod.await_log(peer, LISTENING, since, START)?;
    Harness::connect(peer, &pod.run_dir(peer))
}

/// Restarts `peer`'s container and connects to its harness again.
fn restart(pod: &Pod, peer: &'static str) -> Result<Harness> {
    let since = now().saturating_sub(1);
    pod.restart(peer)?;
    connect_when_listening(pod, peer, Some(since))
}

//! The phases' checks. Each needs a fresh box (`cargo testbox down --wipe`, then `up` with the
//! phase's peers): the peers must start in an empty room.
//!
//! The peers play each other through the box's own signalling server and WebRTC between
//! containers of one pod, so a pass speaks for the protocol (election, sequencing, history
//! installs, seat rebinding, healing) over real data channels -- not for the internet's NATs and
//! TURN, nor for the deployed fly.io server.

use std::collections::BTreeSet;
use std::time::Duration;

use omdurman_harness::{Action, LISTENING, SeatRow, Snapshot};
use omdurman_types::{BrigadeId, CommandScope, DervishTribe, Player, Scenario};

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

/// A peer's place in a phase: its faction, none for a spectator, and its command within it
/// (rulebook §1.1), none for the whole faction.
#[derive(Clone)]
struct Role {
    side: Option<Player>,
    command: Option<CommandScope>,
}

/// A player of the whole of `faction`.
fn player(faction: Player) -> Role {
    Role {
        side: Some(faction),
        command: None,
    }
}

fn spectator() -> Role {
    Role {
        side: None,
        command: None,
    }
}

/// A Dervish player commanding `tribes`.
fn tribes(tribes: &[DervishTribe]) -> Role {
    Role {
        side: Some(Player::Dervish),
        command: Some(CommandScope::Tribes(tribes.iter().copied().collect())),
    }
}

/// An Anglo-Egyptian player commanding `brigades`.
fn brigades(brigades: &[BrigadeId]) -> Role {
    Role {
        side: Some(Player::AngloEgyptian),
        command: Some(CommandScope::Brigades(
            brigades.iter().copied().collect::<BTreeSet<_>>(),
        )),
    }
}

/// An Anglo-Egyptian player commanding the communal units only (`Army`).
fn army() -> Role {
    Role {
        side: Some(Player::AngloEgyptian),
        command: Some(CommandScope::Army),
    }
}

/// Fall of Khartoum's tribes (§9.322): the Mulazmin, and the rest.
const MULAZMIN: [DervishTribe; 1] = [DervishTribe::Mulazmin];
const OTHER_TRIBES: [DervishTribe; 3] = [
    DervishTribe::Hadendowa,
    DervishTribe::Kehena,
    DervishTribe::Degheim,
];
const BRITISH: [BrigadeId; 2] = [BrigadeId::british(1), BrigadeId::british(2)];
const EGYPTIAN: [BrigadeId; 4] = [
    BrigadeId::egyptian(1),
    BrigadeId::egyptian(2),
    BrigadeId::egyptian(3),
    BrigadeId::egyptian(4),
];

/// Two players.
pub fn core(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let roles = [player(Player::AngloEgyptian), player(Player::Dervish)];
    meet(checks, pod, roles.len())?;
    started(checks, &roles)?;
    played(checks, &roles, "both sides play")?;
    converged(checks)
}

/// Two players; the guest's game restarts mid-battle and takes its seat back.
pub fn rejoin(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let roles = [player(Player::AngloEgyptian), player(Player::Dervish)];
    meet(checks, pod, roles.len())?;
    started(checks, &roles)?;
    played(checks, &roles, "both sides play")?;
    checks.require("a guest whose game restarts gets its seat back", |peers| {
        let guest = host_index(peers)? ^ 1;
        let before = peers[guest].snapshot()?.net.record_len;
        let name = peers[guest].peer;
        peers[guest] = restart(pod, name)?;
        peers[guest].wait_for("its seat and the history", PATIENCE, |snapshot| {
            (snapshot.game.started
                && snapshot.game.seated == roles[guest].side
                && snapshot.net.record_len >= before
                && !snapshot.game.paused)
                .then_some(())
        })
    })?;
    played(checks, &roles, "play goes on after the rejoin")?;
    converged(checks)
}

/// Four players, two a side, each commanding part of it (rulebook §1.1): the British and the
/// Egyptian brigades, the Mulazmin and the other tribes. Each orders its own command, the side's
/// communal units being anyone's.
pub fn shared(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let roles = [
        brigades(&BRITISH),
        brigades(&EGYPTIAN),
        tribes(&MULAZMIN),
        tribes(&OTHER_TRIBES),
    ];
    meet(checks, pod, roles.len())?;
    started(checks, &roles)?;
    played(checks, &roles, "both sides deploy")?;
    played(checks, &roles, "both sides fight")?;
    own_commands(checks, &roles)?;
    converged(checks)
}

/// Four players, two a side; one Anglo-Egyptian commands every brigade, the other only the
/// communal units. A Dervish commander's game leaves mid-battle: the game pauses for everyone
/// until it is back, to its seat and its command.
pub fn shared_rejoin(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let every_brigade: Vec<BrigadeId> = BRITISH.iter().chain(&EGYPTIAN).copied().collect();
    let roles = [
        brigades(&every_brigade),
        army(),
        tribes(&MULAZMIN),
        tribes(&OTHER_TRIBES),
    ];
    meet(checks, pod, roles.len())?;
    started(checks, &roles)?;
    played(checks, &roles, "both sides deploy")?;
    let gone = checks.require(
        "a commander leaves; the game pauses for everyone",
        |peers| {
            let host = host_index(peers)?;
            let gone = (0..roles.len())
                .find(|&index| index != host && roles[index].side == Some(Player::Dervish))
                .ok_or("no Dervish guest")?;
            pod.stop(peers[gone].peer)?;
            let mut rest: Vec<&mut Harness> = peers
                .iter_mut()
                .enumerate()
                .filter_map(|(index, peer)| (index != gone).then_some(peer))
                .collect();
            let count = rest.len();
            wait_all(&mut rest, "the others paused", PATIENCE, |snapshots| {
                (agreed_host(snapshots, count).is_some()
                    && snapshots.iter().all(|snapshot| snapshot.game.paused))
                .then_some(())
            })?;
            Ok(gone)
        },
    )?;
    checks.require(
        "it comes back to its seat and command; the pause lifts",
        |peers| {
            let name = peers[gone].peer;
            pod.start(name)?;
            peers[gone] = connect_when_listening(pod, name, Some(now().saturating_sub(1)))?;
            let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
            wait_all(
                &mut all,
                "every peer back, the game going on",
                PATIENCE,
                |snapshots| {
                    (agreed_host(snapshots, roles.len()).is_some()
                        && seats_agree(snapshots, &roles).is_ok()
                        && snapshots
                            .iter()
                            .all(|snapshot| snapshot.game.started && !snapshot.game.paused))
                    .then_some(())
                },
            )
        },
    )?;
    played(checks, &roles, "play goes on after the return")?;
    own_commands(checks, &roles)?;
    converged(checks)
}

/// Five players: three Dervish (the Mulazmin, the Hadendowa, the Kehena and Degheim) and two
/// Anglo-Egyptian (1B, 2B and 1E; 2E to 4E). The host leaves mid-battle: the others elect a new
/// one and pause for its seat; it comes back to its seat and command.
pub fn shared_failover(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let roles = [
        brigades(&[BRITISH[0], BRITISH[1], EGYPTIAN[0]]),
        brigades(&EGYPTIAN[1..]),
        tribes(&MULAZMIN),
        tribes(&OTHER_TRIBES[..1]),
        tribes(&OTHER_TRIBES[1..]),
    ];
    meet(checks, pod, roles.len())?;
    started(checks, &roles)?;
    played(checks, &roles, "every side deploys")?;
    let host = checks.require(
        "the host leaves; the others elect a new one and pause",
        |peers| {
            let host = host_index(peers)?;
            pod.stop(peers[host].peer)?;
            let mut rest: Vec<&mut Harness> = peers
                .iter_mut()
                .enumerate()
                .filter_map(|(index, peer)| (index != host).then_some(peer))
                .collect();
            let count = rest.len();
            wait_all(
                &mut rest,
                "a new host, the game paused for the old one's seat",
                PATIENCE,
                |snapshots| {
                    (agreed_host(snapshots, count).is_some()
                        && snapshots.iter().all(|snapshot| snapshot.game.paused))
                    .then_some(())
                },
            )?;
            Ok(host)
        },
    )?;
    checks.require("the old host comes back to its seat and command", |peers| {
        let name = peers[host].peer;
        pod.start(name)?;
        peers[host] = connect_when_listening(pod, name, Some(now().saturating_sub(1)))?;
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        wait_all(
            &mut all,
            "every peer back, the game going on",
            PATIENCE,
            |snapshots| {
                (agreed_host(snapshots, roles.len()).is_some()
                    && seats_agree(snapshots, &roles).is_ok()
                    && snapshots
                        .iter()
                        .all(|snapshot| snapshot.game.started && !snapshot.game.paused))
                .then_some(())
            },
        )
    })?;
    played(checks, &roles, "play goes on under the new host")?;
    own_commands(checks, &roles)?;
    converged(checks)
}

/// Four players, two a side, every one of them playing the whole of its side, as two players who
/// pick the same faction and no command do: each may order any of the side's units. Both sides
/// deploy and fight; a Dervish player's game leaves mid-battle, the game pauses for everyone, and
/// it comes back to its seat.
pub fn team(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let roles = [
        player(Player::AngloEgyptian),
        player(Player::AngloEgyptian),
        player(Player::Dervish),
        player(Player::Dervish),
    ];
    meet(checks, pod, roles.len())?;
    started(checks, &roles)?;
    played(checks, &roles, "both sides deploy")?;
    played(checks, &roles, "both sides fight")?;
    every_player_ordered(checks, &roles)?;
    let gone = checks.require("a player leaves; the game pauses for everyone", |peers| {
        let host = host_index(peers)?;
        let gone = (0..roles.len())
            .find(|&index| index != host && roles[index].side == Some(Player::Dervish))
            .ok_or("no Dervish guest")?;
        pod.stop(peers[gone].peer)?;
        let mut rest: Vec<&mut Harness> = peers
            .iter_mut()
            .enumerate()
            .filter_map(|(index, peer)| (index != gone).then_some(peer))
            .collect();
        let count = rest.len();
        wait_all(&mut rest, "the others paused", PATIENCE, |snapshots| {
            (agreed_host(snapshots, count).is_some()
                && snapshots.iter().all(|snapshot| snapshot.game.paused))
            .then_some(())
        })?;
        Ok(gone)
    })?;
    checks.require("it comes back to its seat; the pause lifts", |peers| {
        let name = peers[gone].peer;
        pod.start(name)?;
        peers[gone] = connect_when_listening(pod, name, Some(now().saturating_sub(1)))?;
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        wait_all(
            &mut all,
            "every peer back, the game going on",
            PATIENCE,
            |snapshots| {
                (agreed_host(snapshots, roles.len()).is_some()
                    && seats_agree(snapshots, &roles).is_ok()
                    && snapshots
                        .iter()
                        .all(|snapshot| snapshot.game.started && !snapshot.game.paused))
                .then_some(())
            },
        )
    })?;
    played(checks, &roles, "play goes on after the return")?;
    converged(checks)
}

/// Five players: three Anglo-Egyptian -- two playing the whole side, one commanding the British
/// brigades -- and two Dervish, one playing the whole side, one commanding the Mulazmin. A
/// whole-side player orders every unit no teammate's command claims (rulebook §1.1).
pub fn team_mixed(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let roles = [
        player(Player::AngloEgyptian),
        player(Player::AngloEgyptian),
        brigades(&BRITISH),
        player(Player::Dervish),
        tribes(&MULAZMIN),
    ];
    meet(checks, pod, roles.len())?;
    started(checks, &roles)?;
    played(checks, &roles, "both sides deploy")?;
    played(checks, &roles, "both sides fight")?;
    every_player_ordered(checks, &roles)?;
    own_commands(checks, &roles)?;
    converged(checks)
}

/// Every seated player has given orders: no teammate was left out of its side's play.
fn every_player_ordered(checks: &mut Checks, roles: &[Role]) -> Result<()> {
    checks.require("every player has given orders", |peers| {
        let mut idle = Vec::new();
        for (peer, role) in peers.iter_mut().zip(roles) {
            let autoplayed = peer.snapshot()?.autoplayed;
            if role.side.is_some() && autoplayed.orders == 0 {
                idle.push(peer.peer);
            }
        }
        if idle.is_empty() {
            Ok(())
        } else {
            Err(format!("no orders from {}", idle.join(", ")))
        }
    })
}

/// Every commander has ordered units of its own command (not only the side's communal ones):
/// its autoplay kept to it, and its teammates left it to it.
fn own_commands(checks: &mut Checks, roles: &[Role]) -> Result<()> {
    checks.require("every commander has ordered its own command", |peers| {
        let mut idle = Vec::new();
        for (peer, role) in peers.iter_mut().zip(roles) {
            let claims = role
                .command
                .as_ref()
                .is_some_and(|command| !command.claims_nothing());
            let autoplayed = peer.snapshot()?.autoplayed;
            if claims && autoplayed.own_command == 0 {
                idle.push(format!("{} ({:?})", peer.peer, autoplayed));
            }
        }
        if idle.is_empty() {
            Ok(())
        } else {
            Err(format!("no order to its own command: {}", idle.join(", ")))
        }
    })
}

/// Every peer's seat table holds each player's seat: its faction, its command.
fn seats_agree(snapshots: &[Snapshot], roles: &[Role]) -> Result<()> {
    let expected: Vec<SeatRow> = snapshots
        .iter()
        .zip(roles)
        .filter_map(|(snapshot, role)| {
            Some(SeatRow {
                faction: role.side?,
                holder: snapshot.lobby.key.clone(),
                scope: role.command.clone(),
            })
        })
        .collect();
    for snapshot in snapshots {
        if snapshot.game.seats.len() != expected.len()
            || expected
                .iter()
                .any(|seat| !snapshot.game.seats.contains(seat))
        {
            return Err(format!("seats {:?}, not {expected:?}", snapshot.game.seats));
        }
    }
    Ok(())
}

/// Two players and a spectator; the host leaves mid-battle, the others elect another (and pause
/// for the old host's seat, if it held one), and the old host comes back.
pub fn failover(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let roles = [
        player(Player::AngloEgyptian),
        player(Player::Dervish),
        spectator(),
    ];
    meet(checks, pod, roles.len())?;
    started(checks, &roles)?;
    played(checks, &roles, "both sides play")?;
    let host = checks.require("the host leaves; the others elect a new one", |peers| {
        let host = host_index(peers)?;
        pod.stop(peers[host].peer)?;
        let mut rest: Vec<&mut Harness> = peers
            .iter_mut()
            .enumerate()
            .filter_map(|(index, peer)| (index != host).then_some(peer))
            .collect();
        // The game waits for an absent seat holder; a spectator host leaves no seat empty.
        let seated = roles[host].side.is_some();
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
                    (agreed_host(snapshots, roles.len()).is_some()
                        && snapshots[host].game.seated == roles[host].side
                        && snapshots
                            .iter()
                            .all(|snapshot| snapshot.game.started && !snapshot.game.paused))
                    .then_some(())
                },
            )
        },
    )?;
    played(checks, &roles, "play goes on under the new host")?;
    converged(checks)
}

/// Two players; the signalling server goes down mid-battle and comes back.
pub fn signalling(checks: &mut Checks, pod: &Pod) -> Result<()> {
    let roles = [player(Player::AngloEgyptian), player(Player::Dervish)];
    meet(checks, pod, roles.len())?;
    started(checks, &roles)?;
    played(checks, &roles, "both sides play")?;
    checks.require("the signalling server goes down", |_| pod.stop(MATCHBOX))?;
    played(checks, &roles, "play goes on without the signalling server")?;
    checks.require("the signalling server comes back", |peers| {
        let since = now().saturating_sub(1);
        pod.start(MATCHBOX)?;
        pod.await_log(MATCHBOX, MATCHBOX_READY, Some(since), PATIENCE)?;
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        wait_all(
            &mut all,
            "every peer still seeing the others",
            PATIENCE,
            |snapshots| agreed_host(snapshots, roles.len()),
        )
        .map(drop)
    })?;
    played(checks, &roles, "play goes on after the server's return")?;
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

/// The peers take their `roles` in the lobby, and the host starts a battle every peer joins, its
/// seat table theirs.
fn started(checks: &mut Checks, roles: &[Role]) -> Result<()> {
    checks.require("the host starts a battle every peer joins", |peers| {
        for (peer, role) in peers.iter_mut().zip(roles) {
            match role.side {
                Some(faction) => peer.act(Action::PickFaction {
                    faction: Some(faction),
                })?,
                None => peer.act(Action::Spectate { on: true })?,
            }
            if role.command.is_some() {
                peer.act(Action::PickCommand {
                    scope: role.command.clone(),
                })?;
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
                        snapshot.lobby.roster.len() == roles.len()
                            && snapshot.lobby.roster.iter().all(|row| {
                                row.key.is_some()
                                    && snapshots.iter().zip(roles).any(|(owner, role)| {
                                        owner.lobby.key == row.key.clone().unwrap_or_default()
                                            && row.faction == role.side
                                            && row.command == role.command
                                            && row.spectating == role.side.is_none()
                                    })
                            })
                    })
                    .then_some(())
            },
        )?;
        let host = host_index(peers)?;
        peers[host].act(Action::PickScenario { scenario: SCENARIO })?;
        peers[host].act(Action::StartBattle)?;
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        wait_all(
            &mut all,
            "the battle and its seats on every peer",
            PATIENCE,
            |snapshots| {
                (snapshots.iter().zip(roles).all(|(snapshot, role)| {
                    snapshot.game.started
                        && snapshot.game.scenario == SCENARIO
                        && snapshot.game.seated == role.side
                        && snapshot.screen.as_deref() == Some("Board")
                }) && seats_agree(snapshots, roles).is_ok())
                .then_some(())
            },
        )
    })
}

/// Every seated peer autoplays until [`ORDERS`] more events are in every record, both sides have
/// been to act and set-up is over (or the game is over), then stops.
fn played(checks: &mut Checks, roles: &[Role], name: &str) -> Result<()> {
    checks.require(&format!("{name}: {ORDERS} orders"), |peers| {
        let mut all: Vec<&mut Harness> = peers.iter_mut().collect();
        let start = wait_all(&mut all, "the records", PATIENCE, |snapshots| {
            snapshots
                .iter()
                .map(|snapshot| snapshot.net.record_len)
                .max()
        })?;
        for (peer, role) in all.iter_mut().zip(roles) {
            if role.side.is_some() {
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
        for (peer, role) in all.iter_mut().zip(roles) {
            if role.side.is_some() {
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

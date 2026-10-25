//! The agent harness: with `OMDURMAN_AGENT_SOCKET=<path>` set, the game
//! listens on that Unix socket for JSON lines in the protocol of the
//! `omdurman-harness` crate -- what the lobby, the net layer and the rules
//! engine hold, as data, and actions that take the same roads as the lobby's
//! buttons and the in-game AI. The testbox (`cargo testbox`, `testbox/`)
//! drives each peer of a multiplayer game through it. Inert when unset.
//!
//! A thread per connection reads requests and hands them to the app over a
//! channel; [`serve_agent`] answers them on the next frame (the reactive
//! frame pacing wakes at least every [`crate::activity::UNFOCUSED_WAIT`]).

use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use omdurman_bot::actions::engine_accepts;
use omdurman_harness::{
    Acted, Action, Call, GameSnapshot, LobbySnapshot, NetSnapshot, Outcome, Pong, Reply, Request,
    RosterRow, SeatRow, Snapshot,
};
use omdurman_net::{Ephemeral, GameEvent, NetMsg, NetState, SeatHolder};
use omdurman_rules::effects::GameEffect;

use crate::bot_player::BotDriver;
use crate::lobby::{self, LobbyContext};
use crate::settings::LocalPlayerSettings;
use crate::{AppState, PendingEdits, Screen, TurnState};

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        let Ok(path) = std::env::var(omdurman_harness::SOCKET_ENV) else {
            return;
        };
        // One left by an earlier run: a socket doesn't bind over it.
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                error!(%path, %error, "agent harness: the old socket stays; not listening");
                return;
            }
        }
        let listener = match UnixListener::bind(&path) {
            Ok(listener) => listener,
            Err(error) => {
                error!(%path, %error, "agent harness: not listening");
                return;
            }
        };
        let (sender, receiver) = channel();
        std::thread::Builder::new()
            .name("agent-harness".into())
            .spawn(move || accept(listener, sender))
            .expect("the agent harness's thread starts");
        info!(%path, "{}", omdurman_harness::LISTENING);
        app.insert_resource(AgentInbox(Mutex::new(receiver)))
            .init_resource::<Autoplay>()
            .add_systems(
                Update,
                (
                    serve_agent,
                    autoplay_act.run_if(crate::timeline::session_live),
                )
                    .chain()
                    .after(crate::timeline::catch_up_live_stash)
                    .before(crate::net_plugin::flush_pending),
            );
    }
}

/// A request and where its reply line goes.
struct Incoming {
    request: Request,
    reply: Sender<String>,
}

#[derive(Resource)]
struct AgentInbox(Mutex<Receiver<Incoming>>);

fn accept(listener: UnixListener, sender: Sender<Incoming>) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let sender = sender.clone();
                let spawned = std::thread::Builder::new()
                    .name("agent-connection".into())
                    .spawn(move || {
                        if let Err(error) = converse(stream, &sender) {
                            debug!(%error, "agent harness: a connection ended");
                        }
                    });
                if let Err(error) = spawned {
                    warn!(%error, "agent harness: no thread for a connection");
                }
            }
            Err(error) => warn!(%error, "agent harness: accept failed"),
        }
    }
}

/// Reads one connection's requests until it closes, each answered before the
/// next is read.
fn converse(stream: UnixStream, sender: &Sender<Incoming>) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    for line in BufReader::new(stream).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let answer = match serde_json::from_str::<Request>(&line) {
            Ok(request) => {
                let (reply, replied) = channel();
                if sender.send(Incoming { request, reply }).is_err() {
                    return Ok(()); // the app is gone
                }
                match replied.recv() {
                    Ok(answer) => answer,
                    Err(_) => return Ok(()),
                }
            }
            Err(error) => reply_line(0, Err::<Acted, _>(format!("not a request: {error}"))),
        };
        writer.write_all(answer.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn reply_line<T: serde::Serialize>(id: u64, result: Result<T, String>) -> String {
    serde_json::to_string(&Reply {
        id,
        outcome: Outcome::from(result),
    })
    .unwrap_or_else(|error| format!(r#"{{"id":{id},"error":"unserializable reply: {error}"}}"#))
}

/// The local seat played by the AI's commander, while on (see
/// [`Action::Autoplay`]).
#[derive(Resource, Default)]
struct Autoplay {
    on: bool,
    driver: BotDriver,
    /// The submission in flight, until its echo: the next decision needs the
    /// state it makes (as `BotDriver::in_flight`).
    in_flight: Option<u64>,
    cooldown: f32,
}

/// Seconds between autoplayed orders: brisk, but the peers' boards still
/// show each.
const AUTOPLAY_COOLDOWN_SECS: f32 = 0.2;

/// What the harness reads beyond the lobby's own resources.
#[derive(bevy::ecs::system::SystemParam)]
struct AgentView<'w> {
    net: Res<'w, NetState>,
    app_state: Res<'w, State<AppState>>,
    screen: Option<Res<'w, State<Screen>>>,
    local: Res<'w, LocalPlayerSettings>,
    live: crate::timeline::LiveGame<'w>,
    turn: Res<'w, TurnState>,
    seats: crate::seats::SeatView<'w>,
}

fn serve_agent(
    mut commands: Commands,
    inbox: Res<AgentInbox>,
    view: AgentView,
    mut lobby: LobbyContext,
    mut autoplay: ResMut<Autoplay>,
    mut activity: ResMut<crate::activity::Activity>,
) {
    let Ok(inbox) = inbox.0.lock() else {
        return;
    };
    while let Ok(Incoming { request, reply }) = inbox.try_recv() {
        let id = request.id;
        let answer = match request.call {
            Call::Ping => reply_line(
                id,
                Ok(Pong {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                }),
            ),
            Call::Snapshot => reply_line(id, Ok(snapshot(&view, &lobby, autoplay.on))),
            Call::Act { action } => reply_line(id, act(action, &view, &mut lobby, &mut autoplay)),
            Call::Screenshot { path } => {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path));
                activity.keep_running();
                reply_line(id, Ok(Acted {}))
            }
        };
        // The connection may have closed meanwhile; nobody needs the answer then.
        if reply.send(answer).is_err() {
            debug!("agent harness: a reply nobody waits for");
        }
        // Answered requests may change what shows: the next frame soon.
        activity.keep_running();
    }
}

fn act(
    action: Action,
    view: &AgentView,
    lobby: &mut LobbyContext,
    autoplay: &mut Autoplay,
) -> Result<Acted, String> {
    match action {
        Action::PickFaction { faction } => {
            if lobby.local_spectator.0 {
                return Err("spectating: no faction to pick".into());
            }
            lobby.local_faction.0 = faction;
            lobby
                .pending
                .outgoing_broadcast
                .push(NetMsg::Ephemeral(Ephemeral::FactionChoice(faction)));
        }
        Action::Spectate { on } => {
            lobby.local_spectator.0 = on;
            if on && lobby.local_faction.0.is_some() {
                lobby.local_faction.0 = None;
                lobby
                    .pending
                    .outgoing_broadcast
                    .push(NetMsg::Ephemeral(Ephemeral::FactionChoice(None)));
            }
            lobby
                .pending
                .outgoing_broadcast
                .push(NetMsg::Ephemeral(Ephemeral::SpectatorChoice(on)));
        }
        Action::PickScenario { scenario } => {
            if !view.net.is_host {
                return Err("only the host picks the scenario".into());
            }
            lobby.settings.scenario.0 = scenario;
            lobby.settings.chosen.0 = true;
            let message = lobby.settings.message();
            lobby.pending.outgoing_broadcast.push(message);
        }
        Action::StartBattle => {
            if !view.net.is_host {
                return Err("only the host starts the battle".into());
            }
            if lobby::battle_in_progress(&lobby.recorder, view.live.state().game_over) {
                return Err("a battle is being fought".into());
            }
            let roster = roster(view, lobby);
            let ai = lobby::effective_ai(&roster, &lobby.settings.ai_commanders.0);
            if !lobby::all_players_ready_with_ai(&roster, &ai) {
                return Err("the lobby isn't ready: every player picks, both factions held".into());
            }
            let scenario = lobby.settings.scenario.0;
            let optional_rules = match scenario {
                omdurman_types::Scenario::Campaign => lobby.settings.optional_rules.0.clone(),
                _ => Vec::new(),
            };
            let start = lobby::start_game(&roster, &ai, scenario, optional_rules);
            lobby.pending.submit_game(start);
        }
        Action::Autoplay { on } => {
            if on {
                let seat = crate::seats::seat_of(&view.seats.seats.0, lobby.local_key.0)
                    .map(|(_, seat)| seat)
                    .ok_or("no seat to play")?;
                if seat.scope.is_some() {
                    return Err("autoplay plays whole-faction seats only".into());
                }
            }
            autoplay.on = on;
        }
    }
    Ok(Acted {})
}

fn roster(view: &AgentView, lobby: &LobbyContext) -> Vec<lobby::RosterEntry> {
    lobby::build_roster(
        &view.net,
        &view.local,
        &lobby.local_faction,
        &lobby.local_spectator,
        &lobby.local_command,
        lobby.local_key.0,
        &lobby.peers,
    )
}

fn snapshot(view: &AgentView, lobby: &LobbyContext, autoplay: bool) -> Snapshot {
    let net = &view.net;
    let events = lobby
        .recorder
        .record
        .as_ref()
        .map_or(&[][..], |record| &record.events[..]);
    let record_digest = digest(
        events
            .iter()
            .map(|event| (event.seq, event.uid, &event.payload)),
    );
    let state = view.live.state();
    let seats = &view.seats.seats.0;
    Snapshot {
        screen: view
            .screen
            .as_ref()
            .map(|screen| format!("{:?}", screen.get())),
        app_state: format!("{:?}", view.app_state.get()),
        net: NetSnapshot {
            me: net.my_id().map(|peer| peer.0.to_string()),
            host: net.host_id().map(|peer| peer.0.to_string()),
            is_host: net.is_host,
            peers: net
                .sorted_all()
                .iter()
                .map(|peer| peer.0.to_string())
                .collect(),
            has_ever_peered: net.has_ever_peered,
            last_applied_seq: net.last_applied_seq,
            record_len: events.len(),
            record_digest,
            unconfirmed: lobby.pending.unconfirmed.len(),
        },
        lobby: LobbySnapshot {
            name: view.local.name.clone(),
            key: lobby.local_key.0.0.to_string(),
            faction: lobby.local_faction.0,
            spectating: lobby.local_spectator.0,
            scenario: lobby.settings.scenario.0,
            roster: roster(view, lobby)
                .into_iter()
                .map(|row| RosterRow {
                    peer: row.peer.0.to_string(),
                    key: row.key.map(|key| key.0.to_string()),
                    name: row.name,
                    faction: row.pick,
                    spectating: row.spectating,
                    is_host: row.is_host,
                })
                .collect(),
        },
        game: GameSnapshot {
            started: view.turn.game_started,
            scenario: state.scenario,
            turn: format!("{:?}", state.current_turn),
            phase: format!("{:?}", state.phase),
            to_act: state.player_to_act(),
            game_over: state.game_over,
            result: state
                .game_result
                .as_ref()
                .map(|result| format!("{result:?}")),
            units: state.units.len(),
            units_digest: digest(state.units.iter()),
            seated: crate::seats::seat_of(seats, lobby.local_key.0).map(|(_, seat)| seat.faction),
            paused: view.seats.presence.paused(),
            seats: seats
                .iter()
                .map(|seat| SeatRow {
                    faction: seat.faction,
                    holder: match seat.holder {
                        SeatHolder::Ai => "ai".to_string(),
                        SeatHolder::Human(key) => key.0.to_string(),
                    },
                })
                .collect(),
        },
        autoplay,
    }
}

/// A digest of `items` as JSON: equal on peers that hold equal items (one
/// build, so one hasher).
fn digest<T: serde::Serialize>(items: impl Iterator<Item = T>) -> String {
    let mut hasher = DefaultHasher::new();
    for item in items {
        serde_json::to_string(&item)
            .unwrap_or_else(|error| format!("unserializable: {error}"))
            .hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

/// The local seat's orders while [`Autoplay`] is on: the AI commander's
/// decision for the side to act, when that is ours and we may act, submitted
/// as a human's orders are ([`PendingEdits::submit_game`]), one at a time.
fn autoplay_act(
    time: Res<Time>,
    mut autoplay: ResMut<Autoplay>,
    peers: crate::peers::Peers,
    live: crate::timeline::LiveGame,
    mut pending: ResMut<PendingEdits>,
    mut activity: ResMut<crate::activity::Activity>,
) {
    if !autoplay.on {
        return;
    }
    let state = live.state();
    if state.game_over || state.board.terrain.is_empty() {
        return;
    }
    if let Some(uid) = autoplay.in_flight {
        if pending.unconfirmed.iter().any(|(u, _)| *u == uid) {
            return;
        }
        autoplay.in_flight = None;
    }
    let Some(side) = peers.local() else {
        return;
    };
    if omdurman_bot::actions::decider(state) != Some(side) || !peers.may_act(side) {
        return;
    }
    activity.keep_running();
    autoplay.cooldown -= time.delta_secs();
    if autoplay.cooldown > 0.0 {
        return;
    }
    autoplay.cooldown = AUTOPLAY_COOLDOWN_SECS;
    let effect = autoplay.driver.decide(state, side, &[side]);
    // As the AI: an `AdvancePhase` fallback the engine refuses would only be
    // rejected on the echo.
    if effect == GameEffect::AdvancePhase && !engine_accepts(state, &effect) {
        return;
    }
    autoplay.in_flight = Some(pending.submit_game(GameEvent::Effect(effect)));
}

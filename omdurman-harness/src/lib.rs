//! The protocol of the game's agent harness (`omdurman-app/src/agent.rs`): one JSON document per
//! line each way, these types serialized. A client sends [`Request`]s and reads [`Reply`]s.
//!
//! The harness is how the testbox (`cargo testbox`, `xtask/`) drives a peer and reads it back:
//! what the lobby, the net layer and the rules engine hold, and actions that take the same roads
//! as the lobby's buttons and the in-game AI. It listens only when `OMDURMAN_AGENT_SOCKET` names
//! a socket path.

use omdurman_types::{Player, Scenario};
use serde::{Deserialize, Serialize};

/// What the game logs once its harness listens, which its clients wait for.
pub const LISTENING: &str = "the agent harness is listening";

/// The environment variable naming the harness's socket.
pub const SOCKET_ENV: &str = "OMDURMAN_AGENT_SOCKET";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// Echoed in the reply; 0 when a request didn't parse.
    #[serde(default)]
    pub id: u64,
    #[serde(flatten)]
    pub call: Call,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Call {
    /// Answered with [`Pong`].
    Ping,
    /// Answered with [`Snapshot`].
    Snapshot,
    /// Answered with [`Acted`].
    Act { action: Action },
    /// Writes the window, as drawn, to `path` (in the game's file system) as a PNG, some frames
    /// later. Answered with [`Acted`] once requested.
    Screenshot { path: String },
}

/// The answer to request `id`: `{"id": …, "result": …}` or `{"id": …, "error": "…"}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reply<T> {
    pub id: u64,
    #[serde(flatten)]
    pub outcome: Outcome<T>,
}

/// A request's result, or why there is none.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Outcome<T> {
    #[serde(rename = "result")]
    Ok(T),
    #[serde(rename = "error")]
    Err(String),
}

impl<T> From<Result<T, String>> for Outcome<T> {
    fn from(result: Result<T, String>) -> Self {
        match result {
            Ok(found) => Self::Ok(found),
            Err(error) => Self::Err(error),
        }
    }
}

impl<T> From<Outcome<T>> for Result<T, String> {
    fn from(outcome: Outcome<T>) -> Self {
        match outcome {
            Outcome::Ok(found) => Ok(found),
            Outcome::Err(error) => Err(error),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pong {
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Acted {}

/// What a peer can be told to do, named for what a player does. Each takes the road its button
/// takes (the lobby's) or the AI's (autoplay).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// The lobby's faction pick; none is undecided.
    PickFaction { faction: Option<Player> },
    /// The lobby's Spectate toggle.
    Spectate { on: bool },
    /// The host's scenario pick.
    PickScenario { scenario: Scenario },
    /// The host's Start Battle: refused unless the lobby is ready, or while a battle is fought.
    StartBattle,
    /// The local player's seat played by the in-game AI's commander, through the same submission
    /// path as a human's orders; only whole-faction seats.
    Autoplay { on: bool },
}

/// What the peer holds, as data: the screen, the net layer, the lobby and the game.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// `Screen`, as its variant's name (`Title`, `Lobby`, `Board`, `Review`), if one is up.
    pub screen: Option<String>,
    /// `AppState`, as its variant's name.
    pub app_state: String,
    pub net: NetSnapshot,
    pub lobby: LobbySnapshot,
    pub game: GameSnapshot,
    pub autoplay: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetSnapshot {
    /// This peer's matchbox id, once the signalling server assigned one.
    pub me: Option<String>,
    pub host: Option<String>,
    pub is_host: bool,
    /// Every peer this one sees, itself included, in canonical order.
    pub peers: Vec<String>,
    pub has_ever_peered: bool,
    pub last_applied_seq: Option<u32>,
    /// Events in the canonical record.
    pub record_len: usize,
    /// A digest of the record's events (seq, uid and payload): equal on peers that agree.
    pub record_digest: String,
    /// Own submissions not yet echoed by the host.
    pub unconfirmed: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LobbySnapshot {
    pub name: String,
    /// This player's stable key.
    pub key: String,
    pub faction: Option<Player>,
    pub spectating: bool,
    pub scenario: Scenario,
    pub roster: Vec<RosterRow>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RosterRow {
    pub peer: String,
    /// `None` until the peer's `PlayerInfo` arrived.
    pub key: Option<String>,
    pub name: String,
    pub faction: Option<Player>,
    pub spectating: bool,
    pub is_host: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameSnapshot {
    /// A `StartGame` has been applied.
    pub started: bool,
    pub scenario: Scenario,
    pub turn: String,
    pub phase: String,
    /// The side to act now (the deployer in set-up, the non-moving side in defensive fire).
    pub to_act: Option<Player>,
    pub game_over: bool,
    pub result: Option<String>,
    pub units: usize,
    /// A digest of every unit's position and state: equal on peers that agree.
    pub units_digest: String,
    /// The faction this player's seat plays, if seated.
    pub seated: Option<Player>,
    /// The game waits for an absent seat holder.
    pub paused: bool,
    pub seats: Vec<SeatRow>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeatRow {
    pub faction: Player,
    /// `ai`, or the human holder's key.
    pub holder: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_reads_as_its_method_and_arguments() {
        let request: Request = serde_json::from_str(
            r#"{"id": 3, "method": "act", "action": {"type": "pick_faction", "faction": "Dervish"}}"#,
        )
        .unwrap();
        assert_eq!(
            request,
            Request {
                id: 3,
                call: Call::Act {
                    action: Action::PickFaction {
                        faction: Some(Player::Dervish)
                    }
                }
            }
        );
    }

    #[test]
    fn a_reply_is_tagged_by_its_field() {
        let ok = Reply {
            id: 1,
            outcome: Outcome::Ok(Acted {}),
        };
        assert_eq!(
            serde_json::to_string(&ok).unwrap(),
            r#"{"id":1,"result":{}}"#
        );
        let refused: Reply<Acted> = Reply {
            id: 2,
            outcome: Outcome::Err("not the host".into()),
        };
        assert_eq!(
            serde_json::to_string(&refused).unwrap(),
            r#"{"id":2,"error":"not the host"}"#
        );
    }
}
